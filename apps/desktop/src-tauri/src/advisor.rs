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
    pub state: AdvisorDispatchState,
    pub requires_explicit_approval: bool,
    pub requested_model: Option<String>,
    pub requested_reasoning_effort: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
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

impl AdvisorFoundationSnapshot {
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
                || !proposal.requires_explicit_approval
                || proposal.created_at_ms < 0
                || proposal.updated_at_ms < proposal.created_at_ms
                || proposal
                    .requested_model
                    .as_deref()
                    .is_some_and(|value| !valid_label(value, 128))
                || proposal
                    .requested_reasoning_effort
                    .as_deref()
                    .is_some_and(|value| !valid_label(value, 64))
                || !valid_provenance(&proposal.provenance)
                || !proposal_ids.insert(&proposal.id)
            {
                return Err(AdvisorContractError::InvalidDispatchProposal);
            }
        }
        Ok(())
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
