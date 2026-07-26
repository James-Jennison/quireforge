use std::{fs, path::Path};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    project::{ProjectExecutionError, ProjectService},
    project_state::ProjectStateContract,
};

use super::{inspect_status, run_git, GitRunError};

pub const REPOSITORY_STATE_READER_SCHEMA_VERSION: u16 = 1;
const MAX_EVIDENCE_BYTES: u64 = 128 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum RepositoryRemoteMode {
    LocalOnly,
    ExistingTracking,
    FetchAuthorized,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RepositoryStateReadRequest {
    pub project_id: String,
    pub remote_mode: RepositoryRemoteMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RepositoryStateDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryStateDiagnostic {
    pub id: String,
    pub severity: RepositoryStateDiagnosticSeverity,
    pub affected_field: String,
    pub source_ref: Option<String>,
    pub explanation: String,
    pub approval_required: bool,
    pub recommended_action: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryStateReadSnapshot {
    pub schema_version: u16,
    pub state: ProjectStateContract,
    pub diagnostics: Vec<RepositoryStateDiagnostic>,
}

#[derive(Default)]
pub struct RepositoryStateReader;

impl RepositoryStateReader {
    pub async fn read(
        &self,
        request: RepositoryStateReadRequest,
        projects: &ProjectService,
    ) -> RepositoryStateReadSnapshot {
        let root = match projects.review_root(&request.project_id) {
            Ok(root) => root,
            Err(error) => return unavailable_snapshot(request.project_id, error),
        };
        let mut diagnostics = Vec::new();
        if request.remote_mode == RepositoryRemoteMode::FetchAuthorized {
            if let Err(error) = fetch_tracking_refs(&root.attached_root).await {
                diagnostics.push(diagnostic(
                    "remote-fetch-unavailable",
                    RepositoryStateDiagnosticSeverity::Warning,
                    "repository.remoteHead",
                    None,
                    format!("The explicitly authorized remote refresh was unavailable: {error:?}."),
                    false,
                    "Use existing tracking evidence or retry an explicitly authorized refresh.",
                ));
            }
        }

        let (branch, changes, _) =
            match inspect_status(&root.attached_root, &root.worktree_root).await {
                Ok(status) => status,
                Err(error) => {
                    diagnostics.push(diagnostic(
                        "git-status-unavailable",
                        RepositoryStateDiagnosticSeverity::Error,
                        "repository.worktree",
                        None,
                        format!("Git status was unavailable: {error:?}."),
                        false,
                        "Verify the attached repository is accessible and retry.",
                    ));
                    return snapshot_from_parts(
                        &request.project_id,
                        None,
                        None,
                        None,
                        None,
                        false,
                        diagnostics,
                    );
                }
            };
        let local_head = git_value(&root.attached_root, &["rev-parse", "--verify", "HEAD"]).await;
        let remote_head = if request.remote_mode == RepositoryRemoteMode::LocalOnly {
            None
        } else if branch.upstream.is_some() {
            git_value(
                &root.attached_root,
                &["rev-parse", "--verify", "@{upstream}"],
            )
            .await
        } else {
            None
        };
        let counts = if local_head.is_some() && remote_head.is_some() {
            git_value(
                &root.attached_root,
                &["rev-list", "--left-right", "--count", "HEAD...@{upstream}"],
            )
            .await
            .and_then(|value| parse_counts(&value))
        } else {
            None
        };
        if branch.upstream.is_none() {
            diagnostics.push(diagnostic(
                "upstream-unavailable",
                RepositoryStateDiagnosticSeverity::Info,
                "repository.remoteHead",
                None,
                "The current branch has no configured upstream.".to_owned(),
                false,
                "Configure an upstream only if remote comparison is required.",
            ));
        }
        if request.remote_mode == RepositoryRemoteMode::LocalOnly {
            diagnostics.push(diagnostic(
                "remote-not-requested",
                RepositoryStateDiagnosticSeverity::Info,
                "repository.remoteHead",
                None,
                "Local-only inspection deliberately did not read remote-tracking refs.".to_owned(),
                false,
                "Use existing-tracking or an explicitly authorized fetch when remote evidence is needed.",
            ));
        }
        read_document_branch(
            &root.worktree_root,
            branch.head.as_deref(),
            &mut diagnostics,
        );
        snapshot_from_parts(
            &request.project_id,
            branch.head,
            local_head,
            remote_head,
            counts,
            !changes.is_empty(),
            diagnostics,
        )
    }
}

async fn fetch_tracking_refs(root: &Path) -> Result<(), GitRunError> {
    let output = run_git(
        root,
        &["fetch", "--no-tags", "--no-write-fetch-head", "origin"],
        8 * 1024,
    )
    .await?;
    if output.success {
        Ok(())
    } else {
        Err(GitRunError::Failed)
    }
}

async fn git_value(root: &Path, arguments: &[&str]) -> Option<String> {
    let output = run_git(root, arguments, 8 * 1024).await.ok()?;
    if !output.success {
        return None;
    }
    let value = std::str::from_utf8(&output.stdout).ok()?.trim();
    (!value.is_empty() && value.len() <= 4096).then(|| value.to_owned())
}

fn parse_counts(value: &str) -> Option<(u32, u32)> {
    let mut parts = value.split_whitespace();
    Some((parts.next()?.parse().ok()?, parts.next()?.parse().ok()?))
}

fn read_document_branch(
    root: &Path,
    branch: Option<&str>,
    diagnostics: &mut Vec<RepositoryStateDiagnostic>,
) {
    let path = root.join("docs/CURRENT_STATE.md");
    let Ok(metadata) = fs::symlink_metadata(&path) else {
        return;
    };
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_EVIDENCE_BYTES
    {
        diagnostics.push(diagnostic(
            "current-state-unreadable",
            RepositoryStateDiagnosticSeverity::Warning,
            "repository.currentBranch",
            Some("docs/CURRENT_STATE.md".to_owned()),
            "The supported current-state document is unsafe or exceeds the reader limit."
                .to_owned(),
            false,
            "Inspect the committed document manually.",
        ));
        return;
    }
    let Ok(contents) = fs::read_to_string(&path) else {
        return;
    };
    let reported = contents
        .lines()
        .find_map(|line| line.strip_prefix("- **Branch:** `"))
        .and_then(|line| line.strip_suffix('`'));
    if let (Some(reported), Some(actual)) = (reported, branch) {
        if reported != actual {
            diagnostics.push(diagnostic(
                "document-branch-mismatch",
                RepositoryStateDiagnosticSeverity::Warning,
                "repository.currentBranch",
                Some("docs/CURRENT_STATE.md".to_owned()),
                "The current-state branch claim does not match verified Git evidence.".to_owned(),
                false,
                "Update the document only after reviewing the mismatch.",
            ));
        }
    }
}

fn snapshot_from_parts(
    project_id: &str,
    branch: Option<String>,
    local_head: Option<String>,
    remote_head: Option<String>,
    counts: Option<(u32, u32)>,
    dirty: bool,
    diagnostics: Vec<RepositoryStateDiagnostic>,
) -> RepositoryStateReadSnapshot {
    let provenance = |trust: &str,
                      source_type: &str,
                      source_ref: Option<String>,
                      source_commit: Option<String>| json!({"trust":trust,"sourceType":source_type,"sourceRef":source_ref,"sourceCommit":source_commit,"observedAt":null,"verifiedAt":null,"note":null});
    let unknown = provenance("unknown", "unknown", None, None);
    let git = provenance(
        "verified",
        "git",
        Some("closed-git-reader".to_owned()),
        local_head.clone(),
    );
    let approval = |scope: &str| json!({"decision":"required","authority":null,"approvedAt":null,"scope":scope,"supersededAt":null,"provenance":unknown.clone()});
    let state = json!({
        "schemaVersion": 1,
        "project": {"id":project_id,"displayName":"Attached project","repository":"unknown/unknown","localWorkspaceId":project_id,"primaryPlatform":"linux","activeUiPlatform":"tauri-react","productDirectionRef":"docs/ROADMAP.md","lifecyclePhase":"unknown","provenance":git.clone()},
        "roadmapRef":"docs/ROADMAP.md",
        "repository":{"currentBranch":branch,"baseBranch":null,"localHead":local_head,"remoteHead":remote_head,"ahead":counts.map(|value| value.0),"behind":counts.map(|value| value.1),"worktree":if dirty {"dirty"} else {"clean"},"lastVerifiedCheckpoint":null,"mergeAuthorization":approval("merge"),"releaseAuthorization":approval("release"),"provenance":git.clone()},
        "milestone":{"id":"unknown","title":"Unknown","status":"planned","objective":"Unknown","approvedScope":[],"exclusions":[],"completionRequirements":[],"predecessorId":null,"successorId":null,"ownerApproval":approval("milestone"),"provenance":unknown},
        "workSessions":[],"checkpoints":[],"validations":[],"packages":{"required":false,"evidence":[],"provenance":unknown},"boundaries":{"approvedActions":[],"prohibitedActions":[],"confirmationRequiredActions":[],"approvals":[],"provenance":unknown},"blockers":[],"contradictions":[],"nextAction":null,"handoff":{"status":"paused","phrase":"Codex paused. Continue.","generatedAt":null,"sourceCheckpoint":null,"provenance":unknown},"provenance":git
    });
    let state =
        serde_json::from_value(state).expect("reader scaffold must satisfy project-state v1");
    RepositoryStateReadSnapshot {
        schema_version: REPOSITORY_STATE_READER_SCHEMA_VERSION,
        state,
        diagnostics,
    }
}

fn unavailable_snapshot(
    project_id: String,
    error: ProjectExecutionError,
) -> RepositoryStateReadSnapshot {
    let diagnostic = diagnostic(
        "attached-project-unavailable",
        RepositoryStateDiagnosticSeverity::Error,
        "project",
        None,
        format!("The requested attached project was unavailable: {error:?}."),
        false,
        "Select or relink an attached project before reading repository state.",
    );
    snapshot_from_parts(&project_id, None, None, None, None, false, vec![diagnostic])
}

fn diagnostic(
    id: &str,
    severity: RepositoryStateDiagnosticSeverity,
    affected_field: &str,
    source_ref: Option<String>,
    explanation: String,
    approval_required: bool,
    recommended_action: &str,
) -> RepositoryStateDiagnostic {
    RepositoryStateDiagnostic {
        id: id.to_owned(),
        severity,
        affected_field: affected_field.to_owned(),
        source_ref,
        explanation,
        approval_required,
        recommended_action: recommended_action.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_only_snapshot_preserves_a_verified_local_head() {
        let snapshot = snapshot_from_parts(
            "018f0000-0000-7000-8000-000000000001",
            Some("main".to_owned()),
            Some("0123456789abcdef0123456789abcdef01234567".to_owned()),
            None,
            None,
            false,
            Vec::new(),
        );

        snapshot
            .state
            .validate()
            .expect("local-only state is valid");
        assert_eq!(
            snapshot.state.repository.local_head.as_deref(),
            Some("0123456789abcdef0123456789abcdef01234567")
        );
        assert_eq!(snapshot.state.repository.remote_head, None);
    }

    #[test]
    fn diagnostics_remain_separate_from_contract_truth() {
        let snapshot = unavailable_snapshot(
            "018f0000-0000-7000-8000-000000000001".to_owned(),
            ProjectExecutionError::ProjectNotFound,
        );

        assert_eq!(snapshot.diagnostics.len(), 1);
        assert_eq!(
            snapshot.state.repository.worktree,
            crate::project_state::WorktreeState::Clean
        );
    }
}
