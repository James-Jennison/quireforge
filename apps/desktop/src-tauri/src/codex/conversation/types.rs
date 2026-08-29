use serde::{Deserialize, Serialize};

use crate::codex::model_selection::{
    ModelSelectionApplication, ModelSelectionChoice, ModelSelectionPolicy, ModelSelectionSnapshot,
};

pub const CONVERSATION_SCHEMA_VERSION: u16 = 3;
pub const CONVERSATION_REGISTRY_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConversationStartRequest {
    pub project_id: String,
    pub prompt: String,
    pub attachment_ids: Vec<String>,
    pub integration_entry_ids: Vec<String>,
    pub model_id: String,
    pub reasoning_effort: String,
    pub selection_policy: ModelSelectionPolicy,
    pub sandbox_mode: ConversationSandboxMode,
    pub approval_policy: ConversationApprovalPolicy,
    #[serde(default)]
    pub interaction_profile: InteractionProfile,
}

/// A closed, presentation-only mapping to Codex's supported personality values.
/// This must never be reused by approval, sandbox, objective, or Action Card code.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InteractionProfile {
    #[default]
    Direct,
    Conversational,
}

impl InteractionProfile {
    pub(crate) const fn as_protocol_value(self) -> &'static str {
        match self {
            Self::Direct => "pragmatic",
            Self::Conversational => "friendly",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConversationApprovalDecisionRequest {
    pub conversation_id: String,
    pub approval_id: String,
    pub decision: ConversationApprovalDecision,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationSandboxMode {
    ReadOnly,
    WorkspaceWrite,
    DangerFullAccess,
}

impl ConversationSandboxMode {
    pub(crate) const fn as_protocol_value(self) -> &'static str {
        match self {
            Self::ReadOnly => "read-only",
            Self::WorkspaceWrite => "workspace-write",
            Self::DangerFullAccess => "danger-full-access",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationApprovalPolicy {
    Untrusted,
    OnRequest,
    Never,
}

impl ConversationApprovalPolicy {
    pub(crate) const fn as_protocol_value(self) -> &'static str {
        match self {
            Self::Untrusted => "untrusted",
            Self::OnRequest => "on-request",
            Self::Never => "never",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationState {
    Empty,
    Running,
    WaitingForApproval,
    Stopping,
    Completed,
    Interrupted,
    Blocked,
    Failed,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationDiagnosticCode {
    ConversationActive,
    ParallelCapacityReached,
    ConversationNotFound,
    InvalidRequest,
    ProjectUnavailable,
    ProjectIdentityChanged,
    ProjectNotWritable,
    ProjectBusy,
    RuntimeUnavailable,
    ModelUnavailable,
    ReasoningUnavailable,
    IntegrationUnavailable,
    AttachmentUnavailable,
    MetadataUnavailable,
    ApprovalRequired,
    ApprovalNotFound,
    ApprovalDecisionUnavailable,
    ProcessExited,
    TransportFailed,
    ProtocolInvalid,
    RpcRejected,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationSnapshot {
    pub schema_version: u16,
    pub delivery_id: Option<String>,
    pub state: ConversationState,
    pub conversation_id: Option<String>,
    pub project_id: Option<String>,
    pub model_id: Option<String>,
    pub reasoning_effort: Option<String>,
    pub model_selection: Option<ModelSelectionSnapshot>,
    pub sandbox_mode: Option<ConversationSandboxMode>,
    pub approval_policy: Option<ConversationApprovalPolicy>,
    pub pending_approval: Option<ConversationApproval>,
    pub events: Vec<ConversationEvent>,
    pub diagnostic_code: Option<ConversationDiagnosticCode>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationRegistrySnapshot {
    pub schema_version: u16,
    pub capacity: u8,
    pub conversations: Vec<ConversationSnapshot>,
}

impl ConversationSnapshot {
    pub(crate) fn empty() -> Self {
        Self {
            schema_version: CONVERSATION_SCHEMA_VERSION,
            delivery_id: None,
            state: ConversationState::Empty,
            conversation_id: None,
            project_id: None,
            model_id: None,
            reasoning_effort: None,
            model_selection: None,
            sandbox_mode: None,
            approval_policy: None,
            pending_approval: None,
            events: Vec::new(),
            diagnostic_code: None,
        }
    }

    pub(crate) fn unavailable(diagnostic_code: ConversationDiagnosticCode) -> Self {
        Self {
            state: ConversationState::Unavailable,
            diagnostic_code: Some(diagnostic_code),
            ..Self::empty()
        }
    }

    pub(crate) fn turn_in_flight(&self) -> bool {
        matches!(
            self.state,
            ConversationState::Running
                | ConversationState::WaitingForApproval
                | ConversationState::Stopping
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(
    tag = "type",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum ConversationEvent {
    Lifecycle {
        sequence: u64,
        phase: ConversationLifecyclePhase,
    },
    AgentMessageDelta {
        sequence: u64,
        item_id: String,
        delta: String,
    },
    AgentMessageCompleted {
        sequence: u64,
        item_id: String,
        text: String,
    },
    ReasoningSummaryDelta {
        sequence: u64,
        delta: String,
    },
    PlanUpdated {
        sequence: u64,
        explanation: Option<String>,
        steps: Vec<ConversationPlanStep>,
    },
    Activity {
        sequence: u64,
        activity_id: String,
        kind: ConversationActivityKind,
        status: ConversationActivityStatus,
        title: String,
        detail: Option<String>,
        exit_code: Option<i32>,
    },
    ActivityOutputDelta {
        sequence: u64,
        activity_id: String,
        delta: String,
    },
    ApprovalRequested {
        sequence: u64,
        approval_id: String,
        activity_id: String,
        kind: ConversationApprovalKind,
    },
    ApprovalResolved {
        sequence: u64,
        approval_id: String,
        resolution: ConversationApprovalResolution,
    },
    ModelSelectionRequested {
        sequence: u64,
        choice: ModelSelectionChoice,
        application: ModelSelectionApplication,
        rationale: String,
    },
    Error {
        sequence: u64,
        code: ConversationStreamErrorCode,
        will_retry: bool,
    },
}

impl ConversationEvent {
    pub(crate) const fn sequence(&self) -> u64 {
        match self {
            Self::Lifecycle { sequence, .. }
            | Self::AgentMessageDelta { sequence, .. }
            | Self::AgentMessageCompleted { sequence, .. }
            | Self::ReasoningSummaryDelta { sequence, .. }
            | Self::PlanUpdated { sequence, .. }
            | Self::Activity { sequence, .. }
            | Self::ActivityOutputDelta { sequence, .. }
            | Self::ApprovalRequested { sequence, .. }
            | Self::ApprovalResolved { sequence, .. }
            | Self::ModelSelectionRequested { sequence, .. }
            | Self::Error { sequence, .. } => *sequence,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationLifecyclePhase {
    Starting,
    Running,
    Stopping,
    Completed,
    Interrupted,
    Blocked,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationPlanStep {
    pub step: String,
    pub status: ConversationPlanStepStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationPlanStepStatus {
    Pending,
    InProgress,
    Completed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationActivityKind {
    UserMessage,
    AgentMessage,
    Plan,
    Reasoning,
    CommandExecution,
    FileChange,
    ToolCall,
    WebSearch,
    Image,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationActivityStatus {
    Started,
    Completed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationApprovalDecision {
    Approve,
    Decline,
    Cancel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationApprovalResolution {
    Approved,
    Declined,
    Canceled,
    ResolvedExternally,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationApprovalKind {
    CommandExecution,
    FileChange,
    Permissions,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationApprovalDetail {
    pub label: String,
    pub value: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationApproval {
    pub approval_id: String,
    pub activity_id: String,
    pub kind: ConversationApprovalKind,
    pub title: String,
    pub reason: Option<String>,
    pub details: Vec<ConversationApprovalDetail>,
    pub decisions: Vec<ConversationApprovalDecision>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationStreamErrorCode {
    ContextWindowExceeded,
    UsageLimitExceeded,
    Unauthorized,
    Sandbox,
    Server,
    Other,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use serde_json::Value;

    use super::*;

    const ID: &str = "018f0000-0000-7000-8000-000000000001";

    fn assert_event_contract(
        event: ConversationEvent,
        expected_type: &str,
        expected_fields: &[&str],
        rejected_snake_case_fields: &[&str],
    ) {
        let value = serde_json::to_value(event).expect("conversation event must serialize");
        let object = value
            .as_object()
            .expect("conversation event must serialize as an object");

        assert_eq!(
            object.get("type"),
            Some(&Value::String(expected_type.to_owned()))
        );
        let actual_fields = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
        let expected_fields = std::iter::once("type")
            .chain(expected_fields.iter().copied())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            actual_fields, expected_fields,
            "conversation event wire fields must exactly match the frontend contract"
        );
        for field in expected_fields {
            assert!(object.contains_key(field), "missing wire field {field}");
        }
        for field in rejected_snake_case_fields {
            assert!(
                !object.contains_key(*field),
                "unexpected snake_case wire field {field}"
            );
        }
    }

    #[test]
    fn conversation_events_serialize_the_frontend_camel_case_contract() {
        assert_event_contract(
            ConversationEvent::Lifecycle {
                sequence: 1,
                phase: ConversationLifecyclePhase::Running,
            },
            "lifecycle",
            &["sequence", "phase"],
            &[],
        );
        assert_event_contract(
            ConversationEvent::AgentMessageDelta {
                sequence: 2,
                item_id: "message-1".to_owned(),
                delta: "Partial response.".to_owned(),
            },
            "agent-message-delta",
            &["sequence", "itemId", "delta"],
            &["item_id"],
        );
        assert_event_contract(
            ConversationEvent::AgentMessageCompleted {
                sequence: 3,
                item_id: "message-1".to_owned(),
                text: "Complete response.".to_owned(),
            },
            "agent-message-completed",
            &["sequence", "itemId", "text"],
            &["item_id"],
        );
        assert_event_contract(
            ConversationEvent::ReasoningSummaryDelta {
                sequence: 4,
                delta: "Reasoning summary.".to_owned(),
            },
            "reasoning-summary-delta",
            &["sequence", "delta"],
            &[],
        );
        assert_event_contract(
            ConversationEvent::PlanUpdated {
                sequence: 5,
                explanation: Some("Plan updated.".to_owned()),
                steps: vec![ConversationPlanStep {
                    step: "Inspect.".to_owned(),
                    status: ConversationPlanStepStatus::InProgress,
                }],
            },
            "plan-updated",
            &["sequence", "explanation", "steps"],
            &[],
        );
        assert_event_contract(
            ConversationEvent::Activity {
                sequence: 6,
                activity_id: ID.to_owned(),
                kind: ConversationActivityKind::CommandExecution,
                status: ConversationActivityStatus::Completed,
                title: "Run command".to_owned(),
                detail: Some("pnpm validate".to_owned()),
                exit_code: Some(0),
            },
            "activity",
            &[
                "sequence",
                "activityId",
                "kind",
                "status",
                "title",
                "detail",
                "exitCode",
            ],
            &["activity_id", "exit_code"],
        );
        assert_event_contract(
            ConversationEvent::ActivityOutputDelta {
                sequence: 7,
                activity_id: ID.to_owned(),
                delta: "Output.".to_owned(),
            },
            "activity-output-delta",
            &["sequence", "activityId", "delta"],
            &["activity_id"],
        );
        assert_event_contract(
            ConversationEvent::ApprovalRequested {
                sequence: 8,
                approval_id: ID.to_owned(),
                activity_id: ID.to_owned(),
                kind: ConversationApprovalKind::CommandExecution,
            },
            "approval-requested",
            &["sequence", "approvalId", "activityId", "kind"],
            &["approval_id", "activity_id"],
        );
        assert_event_contract(
            ConversationEvent::ApprovalResolved {
                sequence: 9,
                approval_id: ID.to_owned(),
                resolution: ConversationApprovalResolution::Approved,
            },
            "approval-resolved",
            &["sequence", "approvalId", "resolution"],
            &["approval_id"],
        );
        assert_event_contract(
            ConversationEvent::ModelSelectionRequested {
                sequence: 10,
                choice: ModelSelectionChoice {
                    model_id: "gpt-5.6-sol".to_owned(),
                    reasoning_effort: "medium".to_owned(),
                },
                application: ModelSelectionApplication::Manual,
                rationale: "Use the selected model.".to_owned(),
            },
            "model-selection-requested",
            &["sequence", "choice", "application", "rationale"],
            &[],
        );
        assert_event_contract(
            ConversationEvent::Error {
                sequence: 11,
                code: ConversationStreamErrorCode::Other,
                will_retry: true,
            },
            "error",
            &["sequence", "code", "willRetry"],
            &["will_retry"],
        );
    }
}
