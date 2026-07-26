//! Reference-only contracts for the future Advisor workspace.
//!
//! This module deliberately does not start a model turn, read an attached
//! project, dispatch a Codex action, or retain a transcript.  Codex remains
//! the owner of account state and conversation content.  QuireForge persists
//! only opaque references and digests so a later, separately approved
//! workspace can ask the user to review an explicit dispatch proposal.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use uuid::{Uuid, Version};

use crate::{
    git::repository_state::{Freshness as RepositoryFreshness, RepositoryStateReadSnapshot},
    project_state::{TrustClassification, WorktreeState},
};

pub const ADVISOR_FOUNDATION_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorContextKind {
    ProjectState,
    Roadmap,
    CurrentState,
    ExecutionReport,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorTrust {
    Verified,
    Reported,
    Inferred,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorFreshness {
    Current,
    Stale,
    Unknown,
    Conflicting,
    NotApplicable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorProvenanceSource {
    GitObservation,
    ProjectStateSnapshot,
    RepositoryDocument,
    ExecutionReport,
    UserSelection,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorProvenance {
    pub trust: AdvisorTrust,
    pub source: AdvisorProvenanceSource,
    /// An opaque, bounded source label. It is never an arbitrary filesystem
    /// path or a transcript fragment.
    pub source_ref: Option<String>,
    pub source_commit: Option<String>,
    pub observed_at_ms: Option<i64>,
    pub note: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorConversationReference {
    pub id: String,
    /// Opaque reference issued by the supported Codex app-server. The value is
    /// not an authentication credential and does not contain conversation
    /// text.
    pub codex_thread_id: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorContextReference {
    pub id: String,
    pub advisor_conversation_id: String,
    pub kind: AdvisorContextKind,
    /// A closed, opaque source label such as `project-state-snapshot`; never a
    /// caller supplied path. Reading context is deferred to a later milestone.
    pub source_ref: String,
    pub source_commit: Option<String>,
    pub content_sha256: String,
    pub selected_at_ms: i64,
    pub freshness: AdvisorFreshness,
    pub provenance: AdvisorProvenance,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorDispatchState {
    Draft,
    Approved,
    Rejected,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorExecutionDispatchState {
    Dispatching,
    Started,
    FailedToStart,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorDeclaredCapability {
    ReadOnly,
    WorkspaceWrite,
    DangerFullAccess,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorDraftCreateRequest {
    pub advisor_conversation_id: String,
    pub target_project_id: String,
    pub prompt: String,
    pub selected_project_state: Option<AdvisorSelectedProjectStateSnapshot>,
    pub declared_capabilities: Vec<AdvisorDeclaredCapability>,
    pub requested_model: String,
    pub requested_reasoning_effort: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorApprovalDecisionRequest {
    pub proposal_id: String,
    pub decision: AdvisorDispatchState,
    pub binding: AdvisorDraftCreateRequest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvisorApprovalSnapshot {
    pub proposal_id: String,
    pub state: AdvisorDispatchState,
    pub expires_at_ms: i64,
    pub dispatch_available: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorDispatchRequest {
    pub proposal_id: String,
    pub binding: AdvisorDraftCreateRequest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvisorDispatchSnapshot {
    pub proposal_id: String,
    pub state: AdvisorExecutionDispatchState,
    pub execution_conversation_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorDispatchProposal {
    pub id: String,
    pub advisor_conversation_id: String,
    pub target_project_id: String,
    /// SHA-256 of an editable prompt held only by the future UI/controller. No
    /// prompt body is persisted in this foundation.
    pub prompt_sha256: String,
    pub context_manifest_sha256: String,
    /// SHA-256 of the closed declared-capability list. The list itself is held
    /// only by the controller UI and is never an execution grant.
    pub capability_manifest_sha256: String,
    pub state: AdvisorDispatchState,
    pub requires_explicit_approval: bool,
    pub requested_model: Option<String>,
    pub requested_reasoning_effort: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub decided_at_ms: Option<i64>,
    pub expires_at_ms: i64,
    pub execution_dispatch_state: Option<AdvisorExecutionDispatchState>,
    pub execution_conversation_id: Option<String>,
    pub provenance: AdvisorProvenance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorFoundationSnapshot {
    pub schema_version: u16,
    pub conversations: Vec<AdvisorConversationReference>,
    pub context_references: Vec<AdvisorContextReference>,
    pub dispatch_proposals: Vec<AdvisorDispatchProposal>,
}

/// Safe, read-only projection for the Advisor workspace. This deliberately
/// omits opaque identifiers, digests, model requests, target project IDs, and
/// every other field unnecessary for presentation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvisorWorkspaceSnapshot {
    pub schema_version: u16,
    pub conversation_count: u16,
    pub context_reference_count: u16,
    pub proposal_count: u16,
    pub context_summaries: Vec<AdvisorContextSummary>,
    pub proposal_summaries: Vec<AdvisorProposalSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvisorContextSummary {
    pub kind: AdvisorContextKind,
    pub trust: AdvisorTrust,
    pub freshness: AdvisorFreshness,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvisorProposalSummary {
    pub state: AdvisorDispatchState,
    pub requires_explicit_approval: bool,
}

/// The only project-derived data the reference-only Advisor may receive in
/// this checkpoint. It is a deliberately small projection of the existing
/// normalized repository-state reader, never the reader's full payload.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorSelectedProjectStateSnapshot {
    pub schema_version: u16,
    pub source_kind: AdvisorContextKind,
    pub selected_at_ms: i64,
    pub trust: AdvisorTrust,
    pub freshness: AdvisorFreshness,
    pub provenance_source: AdvisorProvenanceSource,
    pub worktree: WorktreeState,
    pub diagnostic_count: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorProjectStateReadRequest {
    pub project_id: String,
}

impl AdvisorProjectStateReadRequest {
    pub fn is_valid(&self) -> bool {
        valid_uuid_v7(&self.project_id)
    }
}

impl AdvisorFoundationSnapshot {
    pub fn empty() -> Self {
        Self {
            schema_version: ADVISOR_FOUNDATION_SCHEMA_VERSION,
            conversations: Vec::new(),
            context_references: Vec::new(),
            dispatch_proposals: Vec::new(),
        }
    }

    pub fn validate(&self) -> Result<(), AdvisorContractError> {
        if self.schema_version != ADVISOR_FOUNDATION_SCHEMA_VERSION {
            return Err(AdvisorContractError::UnsupportedSchemaVersion);
        }
        if self.conversations.len() > 256
            || self.context_references.len() > 1024
            || self.dispatch_proposals.len() > 256
        {
            return Err(AdvisorContractError::CollectionTooLarge);
        }

        let mut conversation_ids = HashSet::with_capacity(self.conversations.len());
        for conversation in &self.conversations {
            if !valid_uuid_v7(&conversation.id)
                || !valid_opaque_id(&conversation.codex_thread_id)
                || conversation.created_at_ms < 0
                || conversation.updated_at_ms < conversation.created_at_ms
                || !conversation_ids.insert(&conversation.id)
            {
                return Err(AdvisorContractError::InvalidConversationReference);
            }
        }
        let mut context_ids = HashSet::with_capacity(self.context_references.len());
        for context in &self.context_references {
            if !valid_uuid_v7(&context.id)
                || !valid_uuid_v7(&context.advisor_conversation_id)
                || !conversation_ids.contains(&context.advisor_conversation_id)
                || !valid_source_ref(&context.source_ref)
                || !valid_sha256(&context.content_sha256)
                || context.selected_at_ms < 0
                || !valid_provenance(&context.provenance)
                || !context_ids.insert(&context.id)
            {
                return Err(AdvisorContractError::InvalidContextReference);
            }
        }
        let mut proposal_ids = HashSet::with_capacity(self.dispatch_proposals.len());
        for proposal in &self.dispatch_proposals {
            if !valid_uuid_v7(&proposal.id)
                || !valid_uuid_v7(&proposal.advisor_conversation_id)
                || !valid_uuid_v7(&proposal.target_project_id)
                || !conversation_ids.contains(&proposal.advisor_conversation_id)
                || !valid_sha256(&proposal.prompt_sha256)
                || !valid_sha256(&proposal.context_manifest_sha256)
                || !valid_sha256(&proposal.capability_manifest_sha256)
                || !proposal.requires_explicit_approval
                || proposal.created_at_ms < 0
                || proposal.updated_at_ms < proposal.created_at_ms
                || (proposal.expires_at_ms != 0 && proposal.expires_at_ms < proposal.created_at_ms)
                || proposal.decided_at_ms.is_some_and(|value| {
                    value < proposal.created_at_ms || value > proposal.updated_at_ms
                })
                || proposal
                    .requested_model
                    .as_deref()
                    .is_some_and(|value| !valid_label(value, 128))
                || proposal
                    .requested_reasoning_effort
                    .as_deref()
                    .is_some_and(|value| !valid_label(value, 64))
                || match (
                    proposal.execution_dispatch_state,
                    proposal.execution_conversation_id.as_deref(),
                ) {
                    (None, None)
                    | (Some(AdvisorExecutionDispatchState::Dispatching), None)
                    | (Some(AdvisorExecutionDispatchState::FailedToStart), None) => false,
                    (Some(AdvisorExecutionDispatchState::Started), Some(id)) => !valid_uuid_v7(id),
                    _ => true,
                }
                || !valid_provenance(&proposal.provenance)
                || !proposal_ids.insert(&proposal.id)
            {
                return Err(AdvisorContractError::InvalidDispatchProposal);
            }
        }
        Ok(())
    }

    pub fn workspace_snapshot(&self) -> AdvisorWorkspaceSnapshot {
        AdvisorWorkspaceSnapshot {
            schema_version: self.schema_version,
            conversation_count: self.conversations.len() as u16,
            context_reference_count: self.context_references.len() as u16,
            proposal_count: self.dispatch_proposals.len() as u16,
            context_summaries: self
                .context_references
                .iter()
                .map(|reference| AdvisorContextSummary {
                    kind: reference.kind,
                    trust: reference.provenance.trust,
                    freshness: reference.freshness,
                })
                .collect(),
            proposal_summaries: self
                .dispatch_proposals
                .iter()
                .map(|proposal| AdvisorProposalSummary {
                    state: proposal.state,
                    requires_explicit_approval: proposal.requires_explicit_approval,
                })
                .collect(),
        }
    }
}

impl AdvisorSelectedProjectStateSnapshot {
    pub fn is_valid(&self) -> bool {
        self.schema_version == ADVISOR_FOUNDATION_SCHEMA_VERSION
            && matches!(self.source_kind, AdvisorContextKind::ProjectState)
            && self.selected_at_ms >= 0
            && matches!(
                self.provenance_source,
                AdvisorProvenanceSource::ProjectStateSnapshot
            )
    }

    pub fn from_repository_snapshot(
        snapshot: RepositoryStateReadSnapshot,
        selected_at_ms: i64,
    ) -> Self {
        let validation_freshness = snapshot
            .evidence
            .validations
            .iter()
            .map(|record| record.freshness);
        let package_freshness = snapshot
            .evidence
            .packages
            .iter()
            .map(|record| record.freshness);
        let handoff_freshness = snapshot
            .evidence
            .handoff
            .iter()
            .map(|record| record.freshness);
        let freshness = advisor_freshness(
            validation_freshness
                .chain(package_freshness)
                .chain(handoff_freshness),
            snapshot.state.repository.local_head.is_some(),
        );
        Self {
            schema_version: ADVISOR_FOUNDATION_SCHEMA_VERSION,
            source_kind: AdvisorContextKind::ProjectState,
            selected_at_ms,
            trust: advisor_trust(&snapshot.state.provenance.trust),
            freshness,
            provenance_source: AdvisorProvenanceSource::ProjectStateSnapshot,
            worktree: snapshot.state.repository.worktree,
            diagnostic_count: snapshot.diagnostics.len() as u32,
        }
    }
}

fn advisor_trust(value: &TrustClassification) -> AdvisorTrust {
    match value {
        TrustClassification::Verified => AdvisorTrust::Verified,
        TrustClassification::Reported => AdvisorTrust::Reported,
        TrustClassification::Inferred => AdvisorTrust::Inferred,
        TrustClassification::Unknown => AdvisorTrust::Unknown,
    }
}

fn advisor_freshness(
    values: impl Iterator<Item = RepositoryFreshness>,
    has_local_head: bool,
) -> AdvisorFreshness {
    let mut unknown = !has_local_head;
    let mut stale = false;
    for value in values {
        match value {
            RepositoryFreshness::Conflicting => return AdvisorFreshness::Conflicting,
            RepositoryFreshness::Stale => stale = true,
            RepositoryFreshness::Unknown => unknown = true,
            RepositoryFreshness::Current | RepositoryFreshness::NotApplicable => {}
        }
    }
    if stale {
        AdvisorFreshness::Stale
    } else if unknown {
        AdvisorFreshness::Unknown
    } else {
        AdvisorFreshness::Current
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdvisorContractError {
    UnsupportedSchemaVersion,
    CollectionTooLarge,
    InvalidConversationReference,
    InvalidContextReference,
    InvalidDispatchProposal,
}

fn valid_uuid_v7(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| match index {
            8 | 13 | 18 | 23 => byte == b'-',
            14 => byte == b'7',
            19 => matches!(byte, b'8' | b'9' | b'a' | b'b'),
            _ => valid_lower_hex_byte(byte),
        })
        && Uuid::parse_str(value).ok().and_then(|id| id.get_version()) == Some(Version::SortRand)
}

fn valid_opaque_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_source_ref(value: &str) -> bool {
    valid_label(value, 96)
}

fn valid_label(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(valid_lower_hex_byte)
}

fn valid_commit(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(valid_lower_hex_byte)
}

fn valid_lower_hex_byte(byte: u8) -> bool {
    byte.is_ascii_digit() || matches!(byte, b'a'..=b'f')
}

fn valid_provenance(value: &AdvisorProvenance) -> bool {
    value.source_ref.as_deref().is_none_or(valid_source_ref)
        && value.source_commit.as_deref().is_none_or(valid_commit)
        && value.observed_at_ms.is_none_or(|timestamp| timestamp >= 0)
        && value
            .note
            .as_deref()
            .is_none_or(|note| note.len() <= 512 && !note.chars().any(char::is_control))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../fixtures/advisor-foundation.json");

    #[test]
    fn shared_reference_only_fixture_is_a_valid_closed_contract() {
        let snapshot: AdvisorFoundationSnapshot =
            serde_json::from_str(FIXTURE).expect("fixture must deserialize");
        snapshot.validate().expect("fixture must be valid");
        let serialized = serde_json::to_value(&snapshot).expect("fixture must serialize");
        assert!(serialized.get("prompt").is_none());
        assert!(serialized.get("transcript").is_none());
    }

    #[test]
    fn accepts_an_empty_reference_only_snapshot() {
        assert_eq!(AdvisorFoundationSnapshot::empty().validate(), Ok(()));
    }

    #[test]
    fn workspace_snapshot_excludes_opaque_and_sensitive_metadata() {
        let snapshot: AdvisorFoundationSnapshot =
            serde_json::from_str(FIXTURE).expect("fixture must deserialize");
        let value = serde_json::to_value(snapshot.workspace_snapshot())
            .expect("workspace summary must serialize");
        let serialized = value.to_string();
        assert!(!serialized.contains("advisor-thread-fixture-01"));
        assert!(!serialized.contains("promptSha256"));
        assert!(!serialized.contains("targetProjectId"));
        assert_eq!(value["conversationCount"], 1);
    }

    #[test]
    fn selected_project_state_projection_excludes_repository_identity_and_content() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../fixtures/project-state.json"))
                .expect("project-state fixture collection must deserialize");
        let state = serde_json::from_value(fixture["minimalValid"].clone())
            .expect("representative project state must deserialize");
        let selected = AdvisorSelectedProjectStateSnapshot::from_repository_snapshot(
            RepositoryStateReadSnapshot {
                schema_version: 1,
                state,
                git: Default::default(),
                evidence: Default::default(),
                diagnostics: Vec::new(),
            },
            1,
        );
        let serialized = serde_json::to_string(&selected).expect("projection must serialize");
        assert!(serialized.contains("project-state"));
        assert!(!serialized.contains("repository"));
        assert!(!serialized.contains("sourceRef"));
        assert!(!serialized.contains("localHead"));
    }

    #[test]
    fn selected_project_state_request_rejects_paths() {
        assert!(AdvisorProjectStateReadRequest {
            project_id: "019d4e3c-3b14-7a2b-8c91-3f27d4f7aa10".to_owned(),
        }
        .is_valid());
        assert!(!AdvisorProjectStateReadRequest {
            project_id: "../../private/project".to_owned(),
        }
        .is_valid());
    }

    #[test]
    fn rejects_an_unknown_json_field() {
        let mut value: serde_json::Value =
            serde_json::from_str(FIXTURE).expect("fixture must parse");
        value["unexpected"] = serde_json::Value::Bool(true);
        assert!(serde_json::from_value::<AdvisorFoundationSnapshot>(value).is_err());
    }

    #[test]
    fn rejects_raw_paths_and_non_explicit_dispatches() {
        let mut snapshot: AdvisorFoundationSnapshot =
            serde_json::from_str(FIXTURE).expect("fixture must deserialize");
        snapshot.context_references[0].source_ref = "../../etc/passwd".to_owned();
        assert_eq!(
            snapshot.validate(),
            Err(AdvisorContractError::InvalidContextReference)
        );

        let mut snapshot: AdvisorFoundationSnapshot =
            serde_json::from_str(FIXTURE).expect("fixture must deserialize");
        snapshot.dispatch_proposals[0].requires_explicit_approval = false;
        assert_eq!(
            snapshot.validate(),
            Err(AdvisorContractError::InvalidDispatchProposal)
        );

        let mut snapshot: AdvisorFoundationSnapshot =
            serde_json::from_str(FIXTURE).expect("fixture must deserialize");
        snapshot.context_references[0].content_sha256 = "A".repeat(64);
        assert_eq!(
            snapshot.validate(),
            Err(AdvisorContractError::InvalidContextReference)
        );
    }

    #[test]
    fn dispatch_receipts_are_opaque_and_state_consistent() {
        let mut snapshot: AdvisorFoundationSnapshot =
            serde_json::from_str(FIXTURE).expect("fixture must deserialize");
        snapshot.dispatch_proposals[0].execution_dispatch_state =
            Some(AdvisorExecutionDispatchState::Started);
        assert_eq!(
            snapshot.validate(),
            Err(AdvisorContractError::InvalidDispatchProposal)
        );

        snapshot.dispatch_proposals[0].execution_conversation_id =
            Some("019d4e3c-3b14-7a2b-8c91-3f27d4f7aa10".to_owned());
        assert_eq!(snapshot.validate(), Ok(()));
    }

    #[test]
    fn rejects_orphaned_context_and_dispatch_references() {
        let mut snapshot: AdvisorFoundationSnapshot =
            serde_json::from_str(FIXTURE).expect("fixture must deserialize");
        snapshot.context_references[0].advisor_conversation_id = Uuid::now_v7().to_string();
        assert_eq!(
            snapshot.validate(),
            Err(AdvisorContractError::InvalidContextReference)
        );

        let mut snapshot: AdvisorFoundationSnapshot =
            serde_json::from_str(FIXTURE).expect("fixture must deserialize");
        snapshot.dispatch_proposals[0].advisor_conversation_id = Uuid::now_v7().to_string();
        assert_eq!(
            snapshot.validate(),
            Err(AdvisorContractError::InvalidDispatchProposal)
        );
    }
}
