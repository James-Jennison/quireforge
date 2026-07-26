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
    pub git: RepositoryGitEvidence,
    pub diagnostics: Vec<RepositoryStateDiagnostic>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryGitEvidence {
    pub upstream: Option<String>,
    pub detached: bool,
    pub staged_count: u32,
    pub unstaged_count: u32,
    pub untracked_count: u32,
    pub merge_in_progress: bool,
    pub rebase_in_progress: bool,
    pub cherry_pick_in_progress: bool,
    pub bisect_in_progress: bool,
    pub shallow: Option<bool>,
}

#[derive(Default)]
struct RepositoryStateParts {
    branch: Option<String>,
    local_head: Option<String>,
    remote_head: Option<String>,
    counts: Option<(u32, u32)>,
    dirty: bool,
    git: RepositoryGitEvidence,
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
                        RepositoryStateParts::default(),
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
        let evidence = git_evidence(&branch, &changes, &root.git_dir, &root.attached_root).await;
        snapshot_from_parts(
            &request.project_id,
            RepositoryStateParts {
                branch: branch.head.clone(),
                local_head,
                remote_head,
                counts,
                dirty: !changes.is_empty(),
                git: evidence,
            },
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
        diagnostics.push(diagnostic(
            "current-state-missing",
            RepositoryStateDiagnosticSeverity::Info,
            "repository.currentBranch",
            Some("docs/CURRENT_STATE.md".to_owned()),
            "The supported current-state document is absent.".to_owned(),
            false,
            "Add supported project-state documentation only after review.",
        ));
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
        diagnostics.push(diagnostic(
            "current-state-malformed",
            RepositoryStateDiagnosticSeverity::Warning,
            "repository.currentBranch",
            Some("docs/CURRENT_STATE.md".to_owned()),
            "The supported current-state document is not valid UTF-8.".to_owned(),
            false,
            "Inspect the committed document manually.",
        ));
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
    parts: RepositoryStateParts,
    diagnostics: Vec<RepositoryStateDiagnostic>,
) -> RepositoryStateReadSnapshot {
    let provenance = |trust: &str,
                      source_type: &str,
                      source_ref: Option<String>,
                      source_commit: Option<String>| json!({"trust":trust,"sourceType":source_type,"sourceRef":source_ref,"sourceCommit":source_commit,"observedAt":null,"verifiedAt":null,"note":null});
    let unknown = provenance("unknown", "unknown", None, None);
    let git_provenance = provenance(
        "verified",
        "git",
        Some("closed-git-reader".to_owned()),
        parts.local_head.clone(),
    );
    let approval = |scope: &str| json!({"decision":"required","authority":null,"approvedAt":null,"scope":scope,"supersededAt":null,"provenance":unknown.clone()});
    let state = json!({
        "schemaVersion": 1,
        "project": {"id":project_id,"displayName":"Attached project","repository":"unknown/unknown","localWorkspaceId":project_id,"primaryPlatform":"linux","activeUiPlatform":"tauri-react","productDirectionRef":"docs/ROADMAP.md","lifecyclePhase":"unknown","provenance":git_provenance.clone()},
        "roadmapRef":"docs/ROADMAP.md",
        "repository":{"currentBranch":parts.branch,"baseBranch":null,"localHead":parts.local_head,"remoteHead":parts.remote_head,"ahead":parts.counts.map(|value| value.0),"behind":parts.counts.map(|value| value.1),"worktree":if parts.dirty {"dirty"} else {"clean"},"lastVerifiedCheckpoint":null,"mergeAuthorization":approval("merge"),"releaseAuthorization":approval("release"),"provenance":git_provenance.clone()},
        "milestone":{"id":"unknown","title":"Unknown","status":"planned","objective":"Unknown","approvedScope":[],"exclusions":[],"completionRequirements":[],"predecessorId":null,"successorId":null,"ownerApproval":approval("milestone"),"provenance":unknown},
        "workSessions":[],"checkpoints":[],"validations":[],"packages":{"required":false,"evidence":[],"provenance":unknown},"boundaries":{"approvedActions":[],"prohibitedActions":[],"confirmationRequiredActions":[],"approvals":[],"provenance":unknown},"blockers":[],"contradictions":[],"nextAction":null,"handoff":{"status":"paused","phrase":"Codex paused. Continue.","generatedAt":null,"sourceCheckpoint":null,"provenance":unknown},"provenance":git_provenance
    });
    let state =
        serde_json::from_value(state).expect("reader scaffold must satisfy project-state v1");
    RepositoryStateReadSnapshot {
        schema_version: REPOSITORY_STATE_READER_SCHEMA_VERSION,
        state,
        git: parts.git,
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
    snapshot_from_parts(
        &project_id,
        RepositoryStateParts::default(),
        vec![diagnostic],
    )
}

async fn git_evidence(
    branch: &super::types::GitBranchSummary,
    changes: &[super::types::GitFileChange],
    git_dir: &Path,
    root: &Path,
) -> RepositoryGitEvidence {
    let mut evidence = RepositoryGitEvidence {
        upstream: branch.upstream.clone(),
        detached: branch.detached,
        ..RepositoryGitEvidence::default()
    };
    for change in changes {
        evidence.staged_count += u32::from(change.staged.is_some());
        evidence.unstaged_count += u32::from(
            change.worktree.is_some()
                && change.worktree != Some(super::types::GitChangeKind::Untracked),
        );
        evidence.untracked_count +=
            u32::from(change.worktree == Some(super::types::GitChangeKind::Untracked));
    }
    evidence.merge_in_progress = git_dir.join("MERGE_HEAD").is_file();
    evidence.rebase_in_progress =
        git_dir.join("rebase-apply").is_dir() || git_dir.join("rebase-merge").is_dir();
    evidence.cherry_pick_in_progress = git_dir.join("CHERRY_PICK_HEAD").is_file();
    evidence.bisect_in_progress = git_dir.join("BISECT_LOG").is_file();
    evidence.shallow = git_value(root, &["rev-parse", "--is-shallow-repository"])
        .await
        .and_then(|value| match value.as_str() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        });
    evidence
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
    use std::{collections::BTreeMap, fs, process::Command};

    use super::*;
    use uuid::Uuid;

    #[derive(Debug, Eq, PartialEq)]
    struct RepositoryFingerprint {
        symbolic_head: Vec<u8>,
        head: Vec<u8>,
        refs: Vec<u8>,
        remotes: Vec<u8>,
        status: Vec<u8>,
        index: Vec<u8>,
        fetch_head: Option<Vec<u8>>,
        files: BTreeMap<String, Vec<u8>>,
        config: Vec<u8>,
        markers: BTreeMap<String, bool>,
    }

    struct FixtureRepository {
        root: std::path::PathBuf,
    }

    impl FixtureRepository {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!("quireforge-reader-{}", Uuid::now_v7()));
            fs::create_dir(&root).expect("fixture root must exist");
            for arguments in [
                &["init", "--quiet"][..],
                &["config", "user.email", "fixture@example.invalid"][..],
                &["config", "user.name", "Fixture"][..],
            ] {
                assert!(Command::new("git")
                    .args(arguments)
                    .current_dir(&root)
                    .status()
                    .expect("git fixture setup must start")
                    .success());
            }
            fs::write(root.join("tracked.txt"), "base\n").expect("tracked file must exist");
            assert!(Command::new("git")
                .args(["add", "tracked.txt"])
                .current_dir(&root)
                .status()
                .expect("git add must start")
                .success());
            assert!(Command::new("git")
                .args(["commit", "--quiet", "-m", "fixture"])
                .current_dir(&root)
                .status()
                .expect("git commit must start")
                .success());
            Self { root }
        }
        fn dirty(&self) {
            fs::write(self.root.join("tracked.txt"), "staged\n").unwrap();
            assert!(Command::new("git")
                .args(["add", "tracked.txt"])
                .current_dir(&self.root)
                .status()
                .unwrap()
                .success());
            fs::write(self.root.join("tracked.txt"), "unstaged\n").unwrap();
            fs::write(self.root.join("untracked.txt"), "keep\n").unwrap();
        }
        fn branch(&self) -> String {
            String::from_utf8(command_output(&self.root, &["branch", "--show-current"]))
                .expect("branch is utf-8")
                .trim()
                .to_owned()
        }
        fn with_remote(&self) -> std::path::PathBuf {
            let remote = self
                .root
                .with_file_name(format!("quireforge-reader-remote-{}", Uuid::now_v7()));
            assert!(Command::new("git")
                .args(["init", "--bare", "--quiet", remote.to_str().unwrap()])
                .status()
                .unwrap()
                .success());
            assert!(Command::new("git")
                .args(["remote", "add", "origin", remote.to_str().unwrap()])
                .current_dir(&self.root)
                .status()
                .unwrap()
                .success());
            let branch = self.branch();
            assert!(Command::new("git")
                .args(["push", "--quiet", "-u", "origin", &branch])
                .current_dir(&self.root)
                .status()
                .unwrap()
                .success());
            remote
        }
        fn advance_remote(&self, remote: &Path) -> String {
            let writer = self
                .root
                .with_file_name(format!("quireforge-reader-writer-{}", Uuid::now_v7()));
            assert!(Command::new("git")
                .args([
                    "clone",
                    "--quiet",
                    remote.to_str().unwrap(),
                    writer.to_str().unwrap()
                ])
                .status()
                .unwrap()
                .success());
            for arguments in [
                &["config", "user.email", "fixture@example.invalid"][..],
                &["config", "user.name", "Fixture"][..],
            ] {
                assert!(Command::new("git")
                    .args(arguments)
                    .current_dir(&writer)
                    .status()
                    .unwrap()
                    .success());
            }
            fs::write(writer.join("remote.txt"), "remote advance\n").unwrap();
            assert!(Command::new("git")
                .args(["add", "remote.txt"])
                .current_dir(&writer)
                .status()
                .unwrap()
                .success());
            assert!(Command::new("git")
                .args(["commit", "--quiet", "-m", "remote advance"])
                .current_dir(&writer)
                .status()
                .unwrap()
                .success());
            assert!(Command::new("git")
                .args(["tag", "remote-only-tag"])
                .current_dir(&writer)
                .status()
                .unwrap()
                .success());
            assert!(Command::new("git")
                .args(["push", "--quiet", "origin", "HEAD"])
                .current_dir(&writer)
                .status()
                .unwrap()
                .success());
            assert!(Command::new("git")
                .args(["push", "--quiet", "--tags", "origin"])
                .current_dir(&writer)
                .status()
                .unwrap()
                .success());
            let head = String::from_utf8(command_output(&writer, &["rev-parse", "HEAD"]))
                .unwrap()
                .trim()
                .to_owned();
            let _ = fs::remove_dir_all(writer);
            head
        }
        fn attach_project(&self) -> (ProjectService, String) {
            let projects = ProjectService::in_memory();
            projects.prepare_attachment(self.root.clone());
            let project_id = projects.confirm_pending().projects[0].id.clone();
            (projects, project_id)
        }
        fn fingerprint(&self) -> RepositoryFingerprint {
            RepositoryFingerprint {
                symbolic_head: command_output(&self.root, &["symbolic-ref", "-q", "HEAD"]),
                head: command_output(&self.root, &["rev-parse", "HEAD"]),
                refs: command_output(&self.root, &["show-ref", "--heads"]),
                remotes: command_output(&self.root, &["for-each-ref", "refs/remotes"]),
                status: command_output(&self.root, &["status", "--porcelain=v2", "-z"]),
                index: fs::read(self.root.join(".git/index")).unwrap(),
                fetch_head: fs::read(self.root.join(".git/FETCH_HEAD")).ok(),
                files: ["tracked.txt", "untracked.txt"]
                    .into_iter()
                    .filter_map(|name| {
                        fs::read(self.root.join(name))
                            .ok()
                            .map(|bytes| (name.to_owned(), bytes))
                    })
                    .collect(),
                config: command_output(&self.root, &["config", "--local", "--list"]),
                markers: [
                    "MERGE_HEAD",
                    "CHERRY_PICK_HEAD",
                    "BISECT_LOG",
                    "rebase-merge",
                    "rebase-apply",
                ]
                .into_iter()
                .map(|marker| {
                    (
                        marker.to_owned(),
                        self.root.join(".git").join(marker).exists(),
                    )
                })
                .collect(),
            }
        }
    }

    impl Drop for FixtureRepository {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn assert_only_tracking_refs_changed(
        before: &RepositoryFingerprint,
        after: &RepositoryFingerprint,
    ) {
        assert_eq!(before.symbolic_head, after.symbolic_head);
        assert_eq!(before.head, after.head);
        assert_eq!(before.refs, after.refs);
        assert_eq!(before.status, after.status);
        assert_eq!(before.index, after.index);
        assert_eq!(before.fetch_head, after.fetch_head);
        assert_eq!(before.files, after.files);
        assert_eq!(before.config, after.config);
        assert_eq!(before.markers, after.markers);
    }

    #[test]
    fn local_only_snapshot_preserves_a_verified_local_head() {
        let snapshot = snapshot_from_parts(
            "018f0000-0000-7000-8000-000000000001",
            RepositoryStateParts {
                branch: Some("main".to_owned()),
                local_head: Some("0123456789abcdef0123456789abcdef01234567".to_owned()),
                ..RepositoryStateParts::default()
            },
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

    #[tokio::test]
    async fn local_only_read_preserves_the_inspected_repository() {
        let root = std::env::temp_dir().join(format!("quireforge-reader-{}", Uuid::now_v7()));
        fs::create_dir(&root).expect("fixture root must exist");
        assert!(Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&root)
            .status()
            .expect("git init must start")
            .success());
        assert!(Command::new("git")
            .args(["config", "user.email", "fixture@example.invalid"])
            .current_dir(&root)
            .status()
            .expect("email config must start")
            .success());
        assert!(Command::new("git")
            .args(["config", "user.name", "Fixture"])
            .current_dir(&root)
            .status()
            .expect("name config must start")
            .success());
        fs::write(root.join("tracked.txt"), "base\n").expect("tracked file must exist");
        assert!(Command::new("git")
            .args(["add", "tracked.txt"])
            .current_dir(&root)
            .status()
            .expect("git add must start")
            .success());
        assert!(Command::new("git")
            .args(["commit", "--quiet", "-m", "fixture"])
            .current_dir(&root)
            .status()
            .expect("git commit must start")
            .success());
        fs::write(root.join("tracked.txt"), "staged\n").expect("staged file must exist");
        assert!(Command::new("git")
            .args(["add", "tracked.txt"])
            .current_dir(&root)
            .status()
            .expect("git add must start")
            .success());
        fs::write(root.join("tracked.txt"), "unstaged\n").expect("unstaged file must exist");
        fs::write(root.join("untracked.txt"), "keep\n").expect("untracked file must exist");
        let before_head = command_output(&root, &["rev-parse", "HEAD"]);
        let before_status = command_output(&root, &["status", "--porcelain=v2", "-z"]);

        let projects = ProjectService::in_memory();
        projects.prepare_attachment(root.clone());
        let project_id = projects.confirm_pending().projects[0].id.clone();
        let snapshot = RepositoryStateReader
            .read(
                RepositoryStateReadRequest {
                    project_id,
                    remote_mode: RepositoryRemoteMode::LocalOnly,
                },
                &projects,
            )
            .await;

        assert_eq!(
            (
                snapshot.git.staged_count,
                snapshot.git.unstaged_count,
                snapshot.git.untracked_count
            ),
            (1, 1, 1)
        );
        assert_eq!(before_head, command_output(&root, &["rev-parse", "HEAD"]));
        assert_eq!(
            before_status,
            command_output(&root, &["status", "--porcelain=v2", "-z"])
        );
        assert_eq!(
            fs::read_to_string(root.join("untracked.txt")).expect("untracked file remains"),
            "keep\n"
        );
        fs::remove_dir_all(root).expect("fixture must be removed");
    }

    #[tokio::test]
    async fn local_and_tracking_reads_preserve_fixture_fingerprints() {
        let fixture = FixtureRepository::new();
        fixture.dirty();
        let projects = ProjectService::in_memory();
        projects.prepare_attachment(fixture.root.clone());
        let project_id = projects.confirm_pending().projects[0].id.clone();
        for remote_mode in [
            RepositoryRemoteMode::LocalOnly,
            RepositoryRemoteMode::ExistingTracking,
        ] {
            let before = fixture.fingerprint();
            let snapshot = RepositoryStateReader
                .read(
                    RepositoryStateReadRequest {
                        project_id: project_id.clone(),
                        remote_mode,
                    },
                    &projects,
                )
                .await;
            assert_eq!(
                (
                    snapshot.git.staged_count,
                    snapshot.git.unstaged_count,
                    snapshot.git.untracked_count
                ),
                (1, 1, 1)
            );
            assert_eq!(before, fixture.fingerprint());
        }
    }

    #[tokio::test]
    async fn fetch_authorized_updates_only_the_tracking_ref_without_fetch_head() {
        let fixture = FixtureRepository::new();
        fixture.dirty();
        let remote = fixture.with_remote();
        let branch = fixture.branch();
        let remote_commit = fixture.advance_remote(&remote);
        let (projects, project_id) = fixture.attach_project();
        let before = fixture.fingerprint();

        let snapshot = RepositoryStateReader
            .read(
                RepositoryStateReadRequest {
                    project_id,
                    remote_mode: RepositoryRemoteMode::FetchAuthorized,
                },
                &projects,
            )
            .await;
        let after = fixture.fingerprint();

        assert_only_tracking_refs_changed(&before, &after);
        assert_ne!(before.remotes, after.remotes);
        assert_eq!(
            String::from_utf8(command_output(
                &fixture.root,
                &["rev-parse", &format!("refs/remotes/origin/{branch}")]
            ))
            .unwrap()
            .trim(),
            remote_commit
        );
        assert_eq!(
            snapshot.state.repository.remote_head.as_deref(),
            Some(remote_commit.as_str())
        );
        assert!(
            command_output(&fixture.root, &["tag", "--list"]).is_empty(),
            "--no-tags prevents the remote-only tag from being created"
        );
        let _ = fs::remove_dir_all(remote);
    }

    #[tokio::test]
    async fn tracking_mode_reads_existing_refs_without_contacting_a_missing_remote() {
        let fixture = FixtureRepository::new();
        let remote = fixture.with_remote();
        assert!(Command::new("git")
            .args([
                "remote",
                "set-url",
                "origin",
                fixture.root.join("unavailable-remote").to_str().unwrap()
            ])
            .current_dir(&fixture.root)
            .status()
            .unwrap()
            .success());
        let (projects, project_id) = fixture.attach_project();
        let before = fixture.fingerprint();
        let snapshot = RepositoryStateReader
            .read(
                RepositoryStateReadRequest {
                    project_id,
                    remote_mode: RepositoryRemoteMode::ExistingTracking,
                },
                &projects,
            )
            .await;
        assert!(snapshot.state.repository.remote_head.is_some());
        assert_eq!(before, fixture.fingerprint());
        let _ = fs::remove_dir_all(remote);
    }

    #[tokio::test]
    async fn reader_represents_ahead_detached_and_missing_upstream_states() {
        let fixture = FixtureRepository::new();
        let remote = fixture.with_remote();
        let branch = fixture.branch();
        fs::write(fixture.root.join("ahead.txt"), "ahead\n").unwrap();
        assert!(Command::new("git")
            .args(["add", "ahead.txt"])
            .current_dir(&fixture.root)
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .args(["commit", "--quiet", "-m", "ahead"])
            .current_dir(&fixture.root)
            .status()
            .unwrap()
            .success());
        let (projects, project_id) = fixture.attach_project();
        let ahead = RepositoryStateReader
            .read(
                RepositoryStateReadRequest {
                    project_id: project_id.clone(),
                    remote_mode: RepositoryRemoteMode::ExistingTracking,
                },
                &projects,
            )
            .await;
        assert!(ahead.state.repository.ahead.unwrap_or_default() > 0);
        assert_eq!(ahead.state.repository.behind, Some(0));

        assert!(Command::new("git")
            .args(["checkout", "--quiet", "--detach"])
            .current_dir(&fixture.root)
            .status()
            .unwrap()
            .success());
        let detached = RepositoryStateReader
            .read(
                RepositoryStateReadRequest {
                    project_id: project_id.clone(),
                    remote_mode: RepositoryRemoteMode::LocalOnly,
                },
                &projects,
            )
            .await;
        assert!(detached.git.detached);
        assert!(detached.state.repository.current_branch.is_none());

        assert!(Command::new("git")
            .args(["checkout", "--quiet", &branch])
            .current_dir(&fixture.root)
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .args(["branch", "--unset-upstream"])
            .current_dir(&fixture.root)
            .status()
            .unwrap()
            .success());
        let no_upstream = RepositoryStateReader
            .read(
                RepositoryStateReadRequest {
                    project_id,
                    remote_mode: RepositoryRemoteMode::ExistingTracking,
                },
                &projects,
            )
            .await;
        assert!(no_upstream.git.upstream.is_none());
        assert!(no_upstream
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.id == "upstream-unavailable"));
        let _ = fs::remove_dir_all(remote);
    }

    #[tokio::test]
    async fn reader_reports_behind_and_diverged_tracking_refs() {
        let behind_fixture = FixtureRepository::new();
        let behind_remote = behind_fixture.with_remote();
        behind_fixture.advance_remote(&behind_remote);
        let (behind_projects, behind_id) = behind_fixture.attach_project();
        let before_fetch = RepositoryStateReader
            .read(
                RepositoryStateReadRequest {
                    project_id: behind_id.clone(),
                    remote_mode: RepositoryRemoteMode::ExistingTracking,
                },
                &behind_projects,
            )
            .await;
        assert_eq!(before_fetch.state.repository.behind, Some(0));
        let behind = RepositoryStateReader
            .read(
                RepositoryStateReadRequest {
                    project_id: behind_id,
                    remote_mode: RepositoryRemoteMode::FetchAuthorized,
                },
                &behind_projects,
            )
            .await;
        assert_eq!(behind.state.repository.ahead, Some(0));
        assert!(behind.state.repository.behind.unwrap_or_default() > 0);
        let _ = fs::remove_dir_all(behind_remote);

        let fixture = FixtureRepository::new();
        let remote = fixture.with_remote();
        fs::write(fixture.root.join("local.txt"), "local\n").unwrap();
        assert!(Command::new("git")
            .args(["add", "local.txt"])
            .current_dir(&fixture.root)
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .args(["commit", "--quiet", "-m", "local advance"])
            .current_dir(&fixture.root)
            .status()
            .unwrap()
            .success());
        fixture.advance_remote(&remote);
        let (projects, project_id) = fixture.attach_project();
        let diverged = RepositoryStateReader
            .read(
                RepositoryStateReadRequest {
                    project_id,
                    remote_mode: RepositoryRemoteMode::FetchAuthorized,
                },
                &projects,
            )
            .await;
        assert!(diverged.state.repository.ahead.unwrap_or_default() > 0);
        assert!(diverged.state.repository.behind.unwrap_or_default() > 0);
        let _ = fs::remove_dir_all(remote);
    }

    #[tokio::test]
    async fn operation_markers_are_reported_without_mutating_the_fixture() {
        let fixture = FixtureRepository::new();
        let git_dir = fixture.root.join(".git");
        for marker in ["MERGE_HEAD", "CHERRY_PICK_HEAD", "BISECT_LOG"] {
            fs::write(git_dir.join(marker), "fixture\n").unwrap();
        }
        fs::create_dir_all(git_dir.join("rebase-merge")).unwrap();
        let branch = inspect_status(&fixture.root, &fixture.root)
            .await
            .unwrap()
            .0;
        let evidence = git_evidence(&branch, &[], &git_dir, &fixture.root).await;
        assert!(evidence.merge_in_progress);
        assert!(evidence.rebase_in_progress);
        assert!(evidence.cherry_pick_in_progress);
        assert!(evidence.bisect_in_progress);
    }

    fn command_output(root: &Path, arguments: &[&str]) -> Vec<u8> {
        let output = Command::new("git")
            .args(arguments)
            .current_dir(root)
            .output()
            .expect("fixture git command must start");
        assert!(output.status.success());
        output.stdout
    }
}
