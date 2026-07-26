use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const PROJECT_STATE_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrustClassification {
    Verified,
    Reported,
    Inferred,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StateSourceType {
    Git,
    RepositoryDocument,
    ValidationReport,
    PackageManifest,
    ApplicationStorage,
    AgentSession,
    UserApproval,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Provenance {
    pub trust: TrustClassification,
    pub source_type: StateSourceType,
    pub source_ref: Option<String>,
    pub source_commit: Option<String>,
    pub observed_at: Option<String>,
    pub verified_at: Option<String>,
    pub note: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApprovalDecision {
    Required,
    Approved,
    Rejected,
    Superseded,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApprovalState {
    pub decision: ApprovalDecision,
    pub authority: Option<String>,
    pub approved_at: Option<String>,
    pub scope: Option<String>,
    pub superseded_at: Option<String>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectIdentity {
    pub id: String,
    pub display_name: String,
    pub repository: String,
    pub local_workspace_id: Option<String>,
    pub primary_platform: String,
    pub active_ui_platform: String,
    pub product_direction_ref: String,
    pub lifecycle_phase: String,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorktreeState {
    Clean,
    Dirty,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RepositoryState {
    pub current_branch: Option<String>,
    pub base_branch: Option<String>,
    pub local_head: Option<String>,
    pub remote_head: Option<String>,
    pub ahead: Option<u32>,
    pub behind: Option<u32>,
    pub worktree: WorktreeState,
    pub last_verified_checkpoint: Option<String>,
    pub merge_authorization: ApprovalState,
    pub release_authorization: ApprovalState,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MilestoneStatus {
    Planned,
    Active,
    Paused,
    Complete,
    Blocked,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MilestoneState {
    pub id: String,
    pub title: String,
    pub status: MilestoneStatus,
    pub objective: String,
    pub approved_scope: Vec<String>,
    pub exclusions: Vec<String>,
    pub completion_requirements: Vec<String>,
    pub predecessor_id: Option<String>,
    pub successor_id: Option<String>,
    pub owner_approval: ApprovalState,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionContext {
    Local,
    Remote,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SessionStatus {
    Active,
    Paused,
    Complete,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkSessionState {
    pub id: String,
    pub actor: String,
    pub execution_context: ExecutionContext,
    pub status: SessionStatus,
    pub target: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub last_activity_at: Option<String>,
    pub pause_reason: Option<String>,
    pub uncommitted_work_may_exist: bool,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckpointStatus {
    Pushed,
    Paused,
    Finished,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CheckpointState {
    pub status: CheckpointStatus,
    pub commit: Option<String>,
    pub branch: Option<String>,
    pub pushed: bool,
    pub validations_current: bool,
    pub documentation_current: bool,
    pub completion_claimed: bool,
    pub timestamp: Option<String>,
    pub remaining_work: Vec<String>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ValidationResult {
    Passed,
    Failed,
    Blocked,
    NotRun,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ValidationState {
    pub category: String,
    pub check_id: String,
    pub command: Option<String>,
    pub result: ValidationResult,
    pub scope: String,
    pub timestamp: Option<String>,
    pub evidence_ref: Option<String>,
    pub commit_tested: Option<String>,
    pub current: bool,
    pub blocker_id: Option<String>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageEvidence {
    pub artifact_type: String,
    pub path: Option<String>,
    pub filename: Option<String>,
    pub source_commit: Option<String>,
    pub size_bytes: Option<u64>,
    pub sha256: Option<String>,
    pub manifest_ref: Option<String>,
    pub platform_baseline: Option<String>,
    pub install_result: Option<ValidationResult>,
    pub launch_result: Option<ValidationResult>,
    pub desktop_integration_result: Option<ValidationResult>,
    pub smoke_test_result: Option<ValidationResult>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageRequirements {
    pub required: bool,
    pub evidence: Vec<PackageEvidence>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Boundaries {
    pub approved_actions: Vec<String>,
    pub prohibited_actions: Vec<String>,
    pub confirmation_required_actions: Vec<String>,
    pub approvals: Vec<ApprovalState>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BlockerSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Blocker {
    pub id: String,
    pub description: String,
    pub severity: BlockerSeverity,
    pub affected_requirement: String,
    pub external: bool,
    pub pre_existing: bool,
    pub milestone_caused: bool,
    pub recommended_action: String,
    pub approval_required: bool,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Contradiction {
    pub id: String,
    pub kind: String,
    pub description: String,
    pub affected_requirement: Option<String>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NextAction {
    pub action: String,
    pub why: String,
    pub approval_required: bool,
    pub target_milestone: Option<String>,
    pub required_starting_commit: Option<String>,
    pub required_branch: Option<String>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HandoffState {
    pub status: CheckpointStatus,
    pub phrase: String,
    pub generated_at: Option<String>,
    pub source_checkpoint: Option<String>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectStateContract {
    pub schema_version: u16,
    pub project: ProjectIdentity,
    pub roadmap_ref: String,
    pub repository: RepositoryState,
    pub milestone: MilestoneState,
    pub work_sessions: Vec<WorkSessionState>,
    pub checkpoints: Vec<CheckpointState>,
    pub validations: Vec<ValidationState>,
    pub packages: PackageRequirements,
    pub boundaries: Boundaries,
    pub blockers: Vec<Blocker>,
    pub contradictions: Vec<Contradiction>,
    pub next_action: Option<NextAction>,
    pub handoff: HandoffState,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ProjectStateContractError {
    #[error("project-state schema version is unsupported")]
    UnsupportedVersion,
    #[error("project-state contract contains an invalid identifier")]
    InvalidIdentifier,
    #[error("project-state contract contains duplicate identities")]
    DuplicateIdentity,
    #[error("project-state contract has an invalid checkpoint state")]
    InvalidCheckpoint,
    #[error("project-state contract has an invalid approval state")]
    InvalidApproval,
    #[error("project-state contract has inconsistent repository state")]
    InvalidRepository,
}

impl ProjectStateContract {
    pub fn validate(&self) -> Result<(), ProjectStateContractError> {
        if self.schema_version != PROJECT_STATE_SCHEMA_VERSION {
            return Err(ProjectStateContractError::UnsupportedVersion);
        }
        if !is_identifier(&self.project.id)
            || !is_repository(&self.project.repository)
            || !is_identifier(&self.milestone.id)
            || self.roadmap_ref.is_empty()
        {
            return Err(ProjectStateContractError::InvalidIdentifier);
        }
        validate_approval(&self.milestone.owner_approval)?;
        validate_approval(&self.repository.merge_authorization)?;
        validate_approval(&self.repository.release_authorization)?;
        for approval in &self.boundaries.approvals {
            validate_approval(approval)?;
        }
        if self.repository.ahead.is_some() != self.repository.behind.is_some() {
            return Err(ProjectStateContractError::InvalidRepository);
        }
        if let Some(commit) = &self.repository.local_head {
            validate_sha(commit)?;
        }
        if let Some(commit) = &self.repository.remote_head {
            validate_sha(commit)?;
        }
        let mut checkpoint_commits = HashSet::new();
        for checkpoint in &self.checkpoints {
            validate_checkpoint(checkpoint)?;
            if let Some(commit) = checkpoint.commit.as_deref() {
                if !checkpoint_commits.insert(commit) {
                    return Err(ProjectStateContractError::DuplicateIdentity);
                }
            }
        }
        for session in &self.work_sessions {
            if !is_identifier(&session.id) {
                return Err(ProjectStateContractError::InvalidIdentifier);
            }
        }
        for validation in &self.validations {
            if !is_identifier(&validation.check_id) {
                return Err(ProjectStateContractError::InvalidIdentifier);
            }
            if let Some(commit) = &validation.commit_tested {
                validate_sha(commit)?;
            }
        }
        for evidence in &self.packages.evidence {
            if let Some(commit) = &evidence.source_commit {
                validate_sha(commit)?;
            }
            if let Some(hash) = &evidence.sha256 {
                if hash.len() != 64
                    || !hash
                        .bytes()
                        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
                {
                    return Err(ProjectStateContractError::InvalidIdentifier);
                }
            }
        }
        Ok(())
    }
}

fn validate_checkpoint(checkpoint: &CheckpointState) -> Result<(), ProjectStateContractError> {
    if let Some(commit) = &checkpoint.commit {
        validate_sha(commit)?;
    }
    let valid = match checkpoint.status {
        CheckpointStatus::Pushed => {
            checkpoint.pushed && checkpoint.commit.is_some() && !checkpoint.completion_claimed
        }
        CheckpointStatus::Paused => !checkpoint.completion_claimed,
        CheckpointStatus::Finished => {
            checkpoint.pushed
                && checkpoint.commit.is_some()
                && checkpoint.completion_claimed
                && checkpoint.remaining_work.is_empty()
        }
    };
    if valid {
        Ok(())
    } else {
        Err(ProjectStateContractError::InvalidCheckpoint)
    }
}

fn validate_approval(approval: &ApprovalState) -> Result<(), ProjectStateContractError> {
    let requires_authority = matches!(
        approval.decision,
        ApprovalDecision::Approved | ApprovalDecision::Rejected | ApprovalDecision::Superseded
    );
    if requires_authority
        && approval
            .authority
            .as_deref()
            .filter(|value| !value.is_empty())
            .is_none()
    {
        return Err(ProjectStateContractError::InvalidApproval);
    }
    Ok(())
}

fn validate_sha(value: &str) -> Result<(), ProjectStateContractError> {
    if value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    {
        Ok(())
    } else {
        Err(ProjectStateContractError::InvalidIdentifier)
    }
}

fn is_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'-' | b'.' | b'_' | b'/')
        })
}

fn is_repository(value: &str) -> bool {
    let mut parts = value.split('/');
    matches!((parts.next(), parts.next(), parts.next()), (Some(owner), Some(name), None) if is_identifier(owner) && is_identifier(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Fixtures {
        minimal_valid: ProjectStateContract,
        active_milestone: ProjectStateContract,
        pushed_checkpoint: CheckpointState,
        paused_session: WorkSessionState,
        completed_milestone: CheckpointState,
        missing_evidence: PackageRequirements,
        missing_validation_evidence: ValidationState,
        contradictory_evidence: Vec<Contradiction>,
    }

    fn fixtures() -> Fixtures {
        serde_json::from_str(include_str!("../../fixtures/project-state.json"))
            .expect("project-state fixtures must deserialize")
    }

    #[test]
    fn shared_fixtures_are_valid_and_round_trip() {
        let fixtures = fixtures();
        let mut pushed = fixtures.active_milestone.clone();
        pushed.checkpoints = vec![fixtures.pushed_checkpoint];
        let mut paused = fixtures.active_milestone.clone();
        paused.work_sessions = vec![fixtures.paused_session];
        let mut completed = fixtures.active_milestone.clone();
        completed.checkpoints = vec![fixtures.completed_milestone];
        completed.milestone.status = MilestoneStatus::Complete;
        let mut missing = fixtures.active_milestone.clone();
        missing.packages = fixtures.missing_evidence;
        let mut missing_validation = fixtures.active_milestone.clone();
        missing_validation.validations = vec![fixtures.missing_validation_evidence];
        let mut contradictory = fixtures.active_milestone.clone();
        contradictory.contradictions = fixtures.contradictory_evidence;
        for state in [
            fixtures.minimal_valid,
            fixtures.active_milestone,
            pushed,
            paused,
            completed,
            missing,
            missing_validation,
            contradictory,
        ] {
            state.validate().expect("fixture must validate");
            let encoded = serde_json::to_string(&state).expect("state must serialize");
            let decoded: ProjectStateContract =
                serde_json::from_str(&encoded).expect("state must deserialize");
            assert_eq!(state, decoded);
        }
    }

    #[test]
    fn rejects_future_versions_bad_ids_and_invalid_completion() {
        let mut state = fixtures().active_milestone;
        state.schema_version = 2;
        assert_eq!(
            state.validate(),
            Err(ProjectStateContractError::UnsupportedVersion)
        );
        state.schema_version = 1;
        state.project.id = "invalid id".to_owned();
        assert_eq!(
            state.validate(),
            Err(ProjectStateContractError::InvalidIdentifier)
        );
        state.project.id = "quireforge".to_owned();
        state.checkpoints[0].status = CheckpointStatus::Finished;
        assert_eq!(
            state.validate(),
            Err(ProjectStateContractError::InvalidCheckpoint)
        );
    }

    #[test]
    fn rejects_unknown_fields_and_approvals_without_authority() {
        let raw = include_str!("../../fixtures/project-state.json").replace(
            "\"schemaVersion\": 1",
            "\"schemaVersion\": 1, \"unknown\": true",
        );
        assert!(serde_json::from_str::<Fixtures>(&raw).is_err());
        let invalid_trust = include_str!("../../fixtures/project-state.json")
            .replace("\"trust\": \"verified\"", "\"trust\": \"unsupported\"");
        assert!(serde_json::from_str::<Fixtures>(&invalid_trust).is_err());
        let mut state = fixtures().active_milestone;
        state.milestone.owner_approval.decision = ApprovalDecision::Approved;
        state.milestone.owner_approval.authority = None;
        assert_eq!(
            state.validate(),
            Err(ProjectStateContractError::InvalidApproval)
        );
    }
}
