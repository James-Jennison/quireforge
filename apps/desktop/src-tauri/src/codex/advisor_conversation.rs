//! Managed, no-project Advisor conversations.
//!
//! This service uses the documented local Codex app-server boundary, but keeps
//! Advisor distinct from Chat and Codex execution: there is no cwd, tool,
//! integration, approval, or project-write capability. Prompt and response
//! text remain in the live process/UI only; the local database receives one
//! opaque thread reference after a successful thread start.

use std::{collections::HashMap, time::Duration};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::advisor_archive_attachment::ClaimedAdvisorArchiveAttachment;
use crate::advisor_attachment::ClaimedAdvisorTextAttachment;
use crate::advisor_document_attachment::ClaimedAdvisorDocumentAttachment;
use crate::advisor_image_attachment::ClaimedAdvisorImageAttachment;

use crate::{
    advisor::AdvisorSelectedProjectStateSnapshot,
    project::{AdvisorConversationMetadata, ProjectService},
};

use super::{
    app_server::{
        validate_uuid_v7, AppServerCommand, AppServerNotification, AppServerProcess,
        ConversationItemKind, ConversationNotification, ConversationTurnStatus,
    },
    auth::types::CodexAuthSnapshot,
    conversation_mode::{managed_chat_authentication_state, ChatAuthenticationState},
    error::CodexAdapterError,
};

const ADVISOR_CONVERSATION_SCHEMA_VERSION: u16 = 1;
const MAX_ADVISOR_PROMPT_BYTES: usize = 64 * 1024;
const FIRST_POLL_WAIT: Duration = Duration::from_millis(200);
const DRAIN_POLL_WAIT: Duration = Duration::from_millis(1);
const MAX_POLL_EVENTS: usize = 32;
const MAX_RECENT_CONVERSATIONS: usize = 64;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorConversationStartRequest {
    pub prompt: String,
    /// A UI-held, application-owned attached-project identifier. When present,
    /// the command re-reads the fixed safe projection before a turn starts.
    pub project_id: Option<String>,
    /// A one-time, native-held text attachment. React receives only its safe
    /// manifest; the file path and bytes never cross the UI boundary.
    pub attachment_id: Option<String>,
    pub attachment_manifest_sha256: Option<String>,
    pub attachment_confirmation: Option<crate::advisor_attachment::AdvisorContentConfirmationState>,
    /// A one-time, native-held PNG/JPEG image. React sees only its safe
    /// manifest; source path and image bytes remain native-only.
    #[serde(default)]
    pub image_attachment_id: Option<String>,
    #[serde(default)]
    pub image_attachment_manifest_sha256: Option<String>,
    #[serde(default)]
    pub image_attachment_confirmation:
        Option<crate::advisor_attachment::AdvisorContentConfirmationState>,
    #[serde(default)]
    pub document_attachment_id: Option<String>,
    #[serde(default)]
    pub document_attachment_manifest_sha256: Option<String>,
    #[serde(default)]
    pub document_attachment_confirmation:
        Option<crate::advisor_attachment::AdvisorContentConfirmationState>,
    pub archive_attachment_id: Option<String>,
    pub archive_attachment_manifest_sha256: Option<String>,
    pub archive_attachment_confirmation:
        Option<crate::advisor_attachment::AdvisorContentConfirmationState>,
}

impl AdvisorConversationStartRequest {
    pub fn is_valid(&self) -> bool {
        valid_prompt(&self.prompt)
            && self
                .project_id
                .as_deref()
                .is_none_or(|project_id| validate_uuid_v7(project_id).is_ok())
            && (self.attachment_id.is_some() == self.attachment_manifest_sha256.is_some())
            && (self.attachment_id.is_some() == self.attachment_confirmation.is_some())
            && (self.image_attachment_id.is_some()
                == self.image_attachment_manifest_sha256.is_some())
            && (self.image_attachment_id.is_some() == self.image_attachment_confirmation.is_some())
            && (self.document_attachment_id.is_some()
                == self.document_attachment_manifest_sha256.is_some())
            && (self.document_attachment_id.is_some()
                == self.document_attachment_confirmation.is_some())
            && (self.archive_attachment_id.is_some()
                == self.archive_attachment_manifest_sha256.is_some())
            && (self.archive_attachment_id.is_some()
                == self.archive_attachment_confirmation.is_some())
            && [
                self.attachment_id.is_some(),
                self.image_attachment_id.is_some(),
                self.document_attachment_id.is_some(),
                self.archive_attachment_id.is_some(),
            ]
            .into_iter()
            .filter(|value| *value)
            .count()
                <= 1
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorConversationState {
    Empty,
    Running,
    Completed,
    Interrupted,
    Blocked,
    Failed,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorConversationDiagnosticCode {
    AuthenticationRequired,
    AuthenticationUnavailable,
    ConversationNotFound,
    ConversationActive,
    InvalidRequest,
    ContextUnavailable,
    RuntimeUnavailable,
    ThreadStartRejected,
    ProtocolInvalid,
    CapabilityBlocked,
    MetadataUnavailable,
    AttachmentUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum AdvisorConversationEvent {
    AgentMessageDelta { sequence: u64, delta: String },
    ReasoningSummaryDelta { sequence: u64, delta: String },
    Error { sequence: u64, code: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvisorConversationSnapshot {
    pub schema_version: u16,
    pub mode: &'static str,
    pub state: AdvisorConversationState,
    /// QuireForge-owned UUID only. Codex thread IDs never cross this command
    /// boundary after metadata is recorded.
    pub conversation_id: Option<String>,
    pub project_state_included: bool,
    pub events: Vec<AdvisorConversationEvent>,
    pub diagnostic_code: Option<AdvisorConversationDiagnosticCode>,
}

impl AdvisorConversationSnapshot {
    pub(crate) fn unavailable(diagnostic_code: AdvisorConversationDiagnosticCode) -> Self {
        Self {
            state: AdvisorConversationState::Unavailable,
            diagnostic_code: Some(diagnostic_code),
            ..Self::empty()
        }
    }

    fn empty() -> Self {
        Self {
            schema_version: ADVISOR_CONVERSATION_SCHEMA_VERSION,
            mode: "advisor",
            state: AdvisorConversationState::Empty,
            conversation_id: None,
            project_state_included: false,
            events: Vec::new(),
            diagnostic_code: None,
        }
    }
}

pub struct AdvisorConversationService {
    state: Mutex<AdvisorConversationServiceState>,
    command: AppServerCommand,
}

struct AdvisorConversationServiceState {
    active: Option<ActiveAdvisorConversation>,
    recent: HashMap<String, AdvisorConversationSnapshot>,
    last: AdvisorConversationSnapshot,
    starting: bool,
}

struct ActiveAdvisorConversation {
    conversation_id: String,
    thread_id: String,
    turn_id: String,
    project_state_included: bool,
    next_sequence: u64,
    process: AppServerProcess,
    _image_attachment: Option<ClaimedAdvisorImageAttachment>,
}

impl AdvisorConversationServiceState {
    fn empty() -> Self {
        Self {
            active: None,
            recent: HashMap::new(),
            last: AdvisorConversationSnapshot::empty(),
            starting: false,
        }
    }

    fn remember(&mut self, snapshot: AdvisorConversationSnapshot) {
        if let Some(conversation_id) = snapshot.conversation_id.clone() {
            if self.recent.len() >= MAX_RECENT_CONVERSATIONS
                && !self.recent.contains_key(&conversation_id)
            {
                if let Some(oldest) = self.recent.keys().next().cloned() {
                    self.recent.remove(&oldest);
                }
            }
            let mut retained = snapshot.clone();
            retained.events.clear();
            self.recent.insert(conversation_id, retained);
        }
        self.last = snapshot;
    }
}

impl Default for AdvisorConversationService {
    fn default() -> Self {
        Self {
            state: Mutex::new(AdvisorConversationServiceState::empty()),
            command: AppServerCommand::codex("codex"),
        }
    }
}

impl AdvisorConversationService {
    #[cfg(test)]
    fn with_command(command: AppServerCommand) -> Self {
        Self {
            state: Mutex::new(AdvisorConversationServiceState::empty()),
            command,
        }
    }

    pub async fn status(&self) -> AdvisorConversationSnapshot {
        let state = self.state.lock().await;
        let mut snapshot = state.last.clone();
        snapshot.events.clear();
        snapshot
    }

    // Each attachment is independently claimed and then held only for the
    // active turn, so keep these explicit rather than using a generic upload.
    #[allow(clippy::too_many_arguments)]
    pub async fn start(
        &self,
        request: AdvisorConversationStartRequest,
        authentication: &CodexAuthSnapshot,
        projects: &ProjectService,
        selected_project_state: Option<AdvisorSelectedProjectStateSnapshot>,
        attachment: Option<ClaimedAdvisorTextAttachment>,
        image_attachment: Option<ClaimedAdvisorImageAttachment>,
        document_attachment: Option<ClaimedAdvisorDocumentAttachment>,
        archive_attachment: Option<ClaimedAdvisorArchiveAttachment>,
    ) -> AdvisorConversationSnapshot {
        match managed_chat_authentication_state(authentication) {
            ChatAuthenticationState::SignInRequired | ChatAuthenticationState::SignInPending => {
                return AdvisorConversationSnapshot::unavailable(
                    AdvisorConversationDiagnosticCode::AuthenticationRequired,
                );
            }
            ChatAuthenticationState::Unavailable => {
                return AdvisorConversationSnapshot::unavailable(
                    AdvisorConversationDiagnosticCode::AuthenticationUnavailable,
                );
            }
            ChatAuthenticationState::Ready => {}
        }
        if !valid_prompt(&request.prompt)
            || request
                .project_id
                .as_deref()
                .is_some_and(|project_id| validate_uuid_v7(project_id).is_err())
            || (request.project_id.is_some() != selected_project_state.is_some())
        {
            return AdvisorConversationSnapshot::unavailable(
                AdvisorConversationDiagnosticCode::InvalidRequest,
            );
        }

        {
            let mut state = self.state.lock().await;
            if state.active.is_some() || state.starting {
                return AdvisorConversationSnapshot::unavailable(
                    AdvisorConversationDiagnosticCode::ConversationActive,
                );
            }
            state.starting = true;
        }

        let started = start_advisor_process(
            &self.command,
            &request.prompt,
            selected_project_state.as_ref(),
            attachment.as_ref(),
            image_attachment.as_ref(),
            document_attachment.as_ref(),
            archive_attachment.as_ref(),
            projects,
        )
        .await;
        let mut state = self.state.lock().await;
        state.starting = false;
        match started {
            Ok(mut active) => {
                active._image_attachment = image_attachment;
                let snapshot = active.snapshot(Vec::new(), None);
                state.active = Some(active);
                state.remember(snapshot.clone());
                snapshot
            }
            Err(diagnostic_code) => {
                let snapshot = AdvisorConversationSnapshot::unavailable(diagnostic_code);
                state.remember(snapshot.clone());
                snapshot
            }
        }
    }

    pub async fn poll(&self, conversation_id: String) -> AdvisorConversationSnapshot {
        if validate_uuid_v7(&conversation_id).is_err() {
            return AdvisorConversationSnapshot::unavailable(
                AdvisorConversationDiagnosticCode::ConversationNotFound,
            );
        }
        let mut state = self.state.lock().await;
        let Some(active) = state.active.as_mut() else {
            return state
                .recent
                .get(&conversation_id)
                .cloned()
                .unwrap_or_else(|| {
                    AdvisorConversationSnapshot::unavailable(
                        AdvisorConversationDiagnosticCode::ConversationNotFound,
                    )
                });
        };
        if active.conversation_id != conversation_id {
            return state
                .recent
                .get(&conversation_id)
                .cloned()
                .unwrap_or_else(|| {
                    AdvisorConversationSnapshot::unavailable(
                        AdvisorConversationDiagnosticCode::ConversationNotFound,
                    )
                });
        }

        let mut events = Vec::new();
        let mut terminal = None;
        for index in 0..MAX_POLL_EVENTS {
            let wait = if index == 0 {
                FIRST_POLL_WAIT
            } else {
                DRAIN_POLL_WAIT
            };
            match active.process.next_notification_with_timeout(wait).await {
                Ok(Some(notification)) => match apply_notification(active, notification) {
                    Ok(AdvisorNotificationOutcome::Event(event)) => events.push(event),
                    Ok(AdvisorNotificationOutcome::None) => {}
                    Ok(AdvisorNotificationOutcome::Completed) => {
                        terminal = Some((AdvisorConversationState::Completed, None));
                        break;
                    }
                    Err(diagnostic) => {
                        terminal = Some((AdvisorConversationState::Blocked, Some(diagnostic)));
                        break;
                    }
                },
                Ok(None) => break,
                Err(error) => {
                    terminal = Some((
                        AdvisorConversationState::Failed,
                        Some(map_adapter_error(error)),
                    ));
                    break;
                }
            }
        }

        if let Some((terminal_state, diagnostic_code)) = terminal {
            let snapshot = AdvisorConversationSnapshot {
                state: terminal_state,
                ..active.snapshot(events, diagnostic_code)
            };
            let _ = active.process.shutdown().await;
            let _ = state.active.take();
            state.remember(snapshot.clone());
            return snapshot;
        }
        let snapshot = active.snapshot(events, None);
        state.remember(snapshot.clone());
        snapshot
    }

    pub async fn interrupt(&self, conversation_id: String) -> AdvisorConversationSnapshot {
        if validate_uuid_v7(&conversation_id).is_err() {
            return AdvisorConversationSnapshot::unavailable(
                AdvisorConversationDiagnosticCode::ConversationNotFound,
            );
        }
        let mut state = self.state.lock().await;
        let Some(active) = state.active.as_mut() else {
            return state
                .recent
                .get(&conversation_id)
                .cloned()
                .unwrap_or_else(|| {
                    AdvisorConversationSnapshot::unavailable(
                        AdvisorConversationDiagnosticCode::ConversationNotFound,
                    )
                });
        };
        if active.conversation_id != conversation_id {
            return AdvisorConversationSnapshot::unavailable(
                AdvisorConversationDiagnosticCode::ConversationNotFound,
            );
        }
        let result = active
            .process
            .request(
                "turn/interrupt",
                json!({"threadId": active.thread_id.clone(), "turnId": active.turn_id.clone()}),
            )
            .await;
        let diagnostic_code = result.err().map(map_adapter_error);
        let snapshot = AdvisorConversationSnapshot {
            state: if diagnostic_code.is_some() {
                AdvisorConversationState::Failed
            } else {
                AdvisorConversationState::Interrupted
            },
            ..active.snapshot(Vec::new(), diagnostic_code)
        };
        let _ = active.process.shutdown().await;
        let _ = state.active.take();
        state.remember(snapshot.clone());
        snapshot
    }
}

impl ActiveAdvisorConversation {
    fn snapshot(
        &self,
        events: Vec<AdvisorConversationEvent>,
        diagnostic_code: Option<AdvisorConversationDiagnosticCode>,
    ) -> AdvisorConversationSnapshot {
        AdvisorConversationSnapshot {
            schema_version: ADVISOR_CONVERSATION_SCHEMA_VERSION,
            mode: "advisor",
            state: AdvisorConversationState::Running,
            conversation_id: Some(self.conversation_id.clone()),
            project_state_included: self.project_state_included,
            events,
            diagnostic_code,
        }
    }

    fn next_sequence(&mut self) -> u64 {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        sequence
    }
}

#[allow(clippy::too_many_arguments)]
async fn start_advisor_process(
    command: &AppServerCommand,
    prompt: &str,
    selected_project_state: Option<&AdvisorSelectedProjectStateSnapshot>,
    attachment: Option<&ClaimedAdvisorTextAttachment>,
    image_attachment: Option<&ClaimedAdvisorImageAttachment>,
    document_attachment: Option<&ClaimedAdvisorDocumentAttachment>,
    archive_attachment: Option<&ClaimedAdvisorArchiveAttachment>,
    projects: &ProjectService,
) -> Result<ActiveAdvisorConversation, AdvisorConversationDiagnosticCode> {
    let mut process = AppServerProcess::spawn(command.clone())
        .map_err(|_| AdvisorConversationDiagnosticCode::RuntimeUnavailable)?;
    let result = async {
        process.initialize().await.map_err(map_adapter_error)?;
        let thread = start_advisor_thread(&mut process).await?;
        let thread_id = parse_thread_start(thread)?;
        let conversation_id = Uuid::now_v7().to_string();
        projects
            .record_advisor_conversation_metadata(AdvisorConversationMetadata {
                conversation_id: &conversation_id,
                codex_thread_id: &thread_id,
            })
            .map_err(|_| AdvisorConversationDiagnosticCode::MetadataUnavailable)?;
        let turn = process
            .request(
                "turn/start",
                advisor_turn_start_params(
                    &thread_id,
                    &advisor_input(
                        prompt,
                        selected_project_state,
                        attachment,
                        document_attachment,
                        archive_attachment,
                    ),
                    image_attachment,
                ),
            )
            .await
            .map_err(map_adapter_error)?;
        let turn_id = parse_turn_start(turn)?;
        Ok((conversation_id, thread_id, turn_id))
    }
    .await;
    match result {
        Ok((conversation_id, thread_id, turn_id)) => Ok(ActiveAdvisorConversation {
            conversation_id,
            thread_id,
            turn_id,
            project_state_included: selected_project_state.is_some(),
            next_sequence: 1,
            process,
            _image_attachment: None,
        }),
        Err(error) => {
            let _ = process.shutdown().await;
            Err(error)
        }
    }
}

fn advisor_thread_start_params() -> Value {
    json!({
        "cwd": Value::Null,
        "approvalPolicy": "never",
        "sandbox": "read-only",
    })
}

/// Compatibility-only profile for managed app-server versions that require an
/// explicit empty capability declaration. It grants no more authority than
/// the minimal Advisor profile above.
fn advisor_thread_start_compatibility_params() -> Value {
    json!({
        "cwd": Value::Null,
        "environments": [],
        "dynamicTools": [],
        "approvalPolicy": "never",
        "sandbox": "read-only",
    })
}

async fn start_advisor_thread(
    process: &mut AppServerProcess,
) -> Result<Value, AdvisorConversationDiagnosticCode> {
    match process
        .request("thread/start", advisor_thread_start_params())
        .await
    {
        Ok(thread) => Ok(thread),
        Err(CodexAdapterError::RpcRejected) => process
            .request("thread/start", advisor_thread_start_compatibility_params())
            .await
            .map_err(map_thread_start_error),
        Err(error) => Err(map_thread_start_error(error)),
    }
}

fn advisor_turn_start_params(
    thread_id: &str,
    prompt: &str,
    image_attachment: Option<&ClaimedAdvisorImageAttachment>,
) -> Value {
    let mut input = vec![json!({"type": "text", "text": prompt})];
    if let Some(image_attachment) = image_attachment {
        input.push(image_attachment.protocol_input());
    }
    json!({
        "threadId": thread_id,
        "input": input,
        "cwd": Value::Null,
        "approvalPolicy": "never",
        "sandboxPolicy": {"type": "readOnly", "networkAccess": false},
    })
}

fn advisor_input(
    prompt: &str,
    context: Option<&AdvisorSelectedProjectStateSnapshot>,
    attachment: Option<&ClaimedAdvisorTextAttachment>,
    document_attachment: Option<&ClaimedAdvisorDocumentAttachment>,
    archive_attachment: Option<&ClaimedAdvisorArchiveAttachment>,
) -> String {
    let mut input = prompt.to_owned();
    if let Some(context) = context {
        input.push_str(&format!(
            "\n\nUser-confirmed Project State summary (safe projection only):\nsource: project-state\ntrust: {:?}\nfreshness: {:?}\nworktree: {:?}\ndiagnostic count: {}",
            context.trust, context.freshness, context.worktree, context.diagnostic_count
        ));
    }
    if let Some(attachment) = attachment {
        input.push_str(&format!(
            "\n\nUser-confirmed text attachment (transient normalized text; no file path):\nname: {}\nkind: {:?}\nsha256: {}\n--- begin attachment ---\n{}\n--- end attachment ---",
            attachment.manifest.display_name, attachment.manifest.content_type, attachment.manifest.sha256, attachment.text
        ));
    }
    if let Some(attachment) = document_attachment {
        input.push_str(&format!("\n\nUser-confirmed PDF projection (transient, bounded, path-free):\nname: {}\nsha256: {}\nprojection: pdf-plain-text-v1\n--- begin projection ---\n{}\n--- end projection ---", attachment.manifest.display_name, attachment.manifest.sha256, attachment.projection_text));
    }
    if let Some(attachment) = archive_attachment {
        input.push_str(&format!("\n\nUser-confirmed ZIP archive manifest (transient, bounded, path-free):\nname: {}\nsha256: {}\nprojection: archive-manifest-v1\n--- begin manifest ---\n{}\n--- end manifest ---", attachment.manifest.display_name, attachment.manifest.sha256, attachment.projection_text));
    }
    input
}

enum AdvisorNotificationOutcome {
    None,
    Event(AdvisorConversationEvent),
    Completed,
}

fn apply_notification(
    active: &mut ActiveAdvisorConversation,
    notification: AppServerNotification,
) -> Result<AdvisorNotificationOutcome, AdvisorConversationDiagnosticCode> {
    let notification = match notification {
        AppServerNotification::Conversation(notification) => notification,
        AppServerNotification::ConversationRequest(_) => {
            return Err(AdvisorConversationDiagnosticCode::CapabilityBlocked)
        }
        AppServerNotification::AccountLoginCompleted { .. }
        | AppServerNotification::AccountUpdated
        | AppServerNotification::McpOauthLoginCompleted { .. }
        | AppServerNotification::IntegrationRefresh(_) => {
            return Ok(AdvisorNotificationOutcome::None)
        }
    };
    match notification {
        ConversationNotification::ThreadStarted { thread_id }
        | ConversationNotification::ThreadArchived { thread_id }
        | ConversationNotification::ThreadUnarchived { thread_id } => {
            ensure_thread(active, &thread_id)?;
            Ok(AdvisorNotificationOutcome::None)
        }
        ConversationNotification::TurnStarted { thread_id, turn_id } => {
            ensure_turn(active, &thread_id, &turn_id)?;
            Ok(AdvisorNotificationOutcome::None)
        }
        ConversationNotification::AgentMessageDelta {
            thread_id,
            turn_id,
            delta,
        } => {
            ensure_turn(active, &thread_id, &turn_id)?;
            Ok(AdvisorNotificationOutcome::Event(
                AdvisorConversationEvent::AgentMessageDelta {
                    sequence: active.next_sequence(),
                    delta,
                },
            ))
        }
        ConversationNotification::ReasoningSummaryDelta {
            thread_id,
            turn_id,
            delta,
        } => {
            ensure_turn(active, &thread_id, &turn_id)?;
            Ok(AdvisorNotificationOutcome::Event(
                AdvisorConversationEvent::ReasoningSummaryDelta {
                    sequence: active.next_sequence(),
                    delta,
                },
            ))
        }
        ConversationNotification::Error {
            thread_id,
            turn_id,
            code,
            ..
        } => {
            ensure_turn(active, &thread_id, &turn_id)?;
            Ok(AdvisorNotificationOutcome::Event(
                AdvisorConversationEvent::Error {
                    sequence: active.next_sequence(),
                    code: format!("{code:?}"),
                },
            ))
        }
        ConversationNotification::TurnCompleted {
            thread_id,
            turn_id,
            status,
        } => {
            ensure_turn(active, &thread_id, &turn_id)?;
            match status {
                ConversationTurnStatus::Completed => Ok(AdvisorNotificationOutcome::Completed),
                ConversationTurnStatus::Interrupted | ConversationTurnStatus::Failed => {
                    Err(AdvisorConversationDiagnosticCode::ProtocolInvalid)
                }
            }
        }
        ConversationNotification::ItemLifecycle {
            thread_id, item, ..
        } => {
            ensure_thread(active, &thread_id)?;
            if matches!(
                item.kind,
                ConversationItemKind::UserMessage
                    | ConversationItemKind::AgentMessage
                    | ConversationItemKind::Reasoning
                    | ConversationItemKind::Plan
            ) {
                Ok(AdvisorNotificationOutcome::None)
            } else {
                Err(AdvisorConversationDiagnosticCode::CapabilityBlocked)
            }
        }
        ConversationNotification::PlanUpdated { .. }
        | ConversationNotification::ActivityDelta { .. }
        | ConversationNotification::ServerRequestResolved { .. } => {
            Err(AdvisorConversationDiagnosticCode::CapabilityBlocked)
        }
    }
}

fn parse_thread_start(value: Value) -> Result<String, AdvisorConversationDiagnosticCode> {
    #[derive(Deserialize)]
    struct ResultValue {
        thread: ProtocolId,
    }
    let value: ResultValue = serde_json::from_value(value)
        .map_err(|_| AdvisorConversationDiagnosticCode::ProtocolInvalid)?;
    validate_uuid_v7(&value.thread.id)
        .map_err(|_| AdvisorConversationDiagnosticCode::ProtocolInvalid)?;
    Ok(value.thread.id)
}

fn parse_turn_start(value: Value) -> Result<String, AdvisorConversationDiagnosticCode> {
    #[derive(Deserialize)]
    struct ResultValue {
        turn: Turn,
    }
    #[derive(Deserialize)]
    struct Turn {
        id: String,
        status: String,
    }
    let value: ResultValue = serde_json::from_value(value)
        .map_err(|_| AdvisorConversationDiagnosticCode::ProtocolInvalid)?;
    validate_uuid_v7(&value.turn.id)
        .map_err(|_| AdvisorConversationDiagnosticCode::ProtocolInvalid)?;
    (value.turn.status == "inProgress")
        .then_some(value.turn.id)
        .ok_or(AdvisorConversationDiagnosticCode::ProtocolInvalid)
}

#[derive(Deserialize)]
struct ProtocolId {
    id: String,
}

fn ensure_thread(
    active: &ActiveAdvisorConversation,
    thread_id: &str,
) -> Result<(), AdvisorConversationDiagnosticCode> {
    (active.thread_id == thread_id)
        .then_some(())
        .ok_or(AdvisorConversationDiagnosticCode::ProtocolInvalid)
}

fn ensure_turn(
    active: &ActiveAdvisorConversation,
    thread_id: &str,
    turn_id: &str,
) -> Result<(), AdvisorConversationDiagnosticCode> {
    ensure_thread(active, thread_id)?;
    (active.turn_id == turn_id)
        .then_some(())
        .ok_or(AdvisorConversationDiagnosticCode::ProtocolInvalid)
}

fn valid_prompt(prompt: &str) -> bool {
    !prompt.trim().is_empty() && prompt.len() <= MAX_ADVISOR_PROMPT_BYTES && !prompt.contains('\0')
}

fn map_adapter_error(error: CodexAdapterError) -> AdvisorConversationDiagnosticCode {
    match error {
        CodexAdapterError::ProcessSpawnFailed | CodexAdapterError::ProcessExited => {
            AdvisorConversationDiagnosticCode::RuntimeUnavailable
        }
        _ => AdvisorConversationDiagnosticCode::ProtocolInvalid,
    }
}

fn map_thread_start_error(error: CodexAdapterError) -> AdvisorConversationDiagnosticCode {
    match error {
        CodexAdapterError::RpcRejected => AdvisorConversationDiagnosticCode::ThreadStartRejected,
        other => map_adapter_error(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codex::auth::types::AuthAccountKind;

    #[test]
    fn fixed_wire_parameters_never_add_project_or_tool_authority() {
        let thread = advisor_thread_start_params();
        assert_eq!(thread["cwd"], Value::Null);
        assert_eq!(thread["approvalPolicy"], "never");
        assert!(thread.get("projectId").is_none());
        assert!(thread.get("environments").is_none());
        assert!(thread.get("dynamicTools").is_none());
        let compatibility = advisor_thread_start_compatibility_params();
        assert_eq!(compatibility["cwd"], Value::Null);
        assert_eq!(compatibility["environments"], json!([]));
        assert_eq!(compatibility["dynamicTools"], json!([]));
        assert_eq!(compatibility["approvalPolicy"], "never");
        let turn = advisor_turn_start_params("thread", "Prompt", None);
        assert_eq!(turn["cwd"], Value::Null);
        assert_eq!(turn["sandboxPolicy"]["networkAccess"], false);
        assert_eq!(turn["input"], json!([{"type":"text","text":"Prompt"}]));
    }

    #[test]
    fn context_is_a_safe_projection_and_is_only_added_when_selected() {
        assert_eq!(
            advisor_input("Plan this", None, None, None, None),
            "Plan this"
        );
        let context = AdvisorSelectedProjectStateSnapshot {
            schema_version: 1,
            source_kind: crate::advisor::AdvisorContextKind::ProjectState,
            selected_at_ms: 1,
            trust: crate::advisor::AdvisorTrust::Verified,
            freshness: crate::advisor::AdvisorFreshness::Current,
            provenance_source: crate::advisor::AdvisorProvenanceSource::ProjectStateSnapshot,
            worktree: crate::project_state::WorktreeState::Clean,
            diagnostic_count: 0,
        };
        let input = advisor_input("Plan this", Some(&context), None, None, None);
        assert!(input.contains("User-confirmed Project State summary"));
        assert!(!input.contains("/mnt/"));
        assert!(!input.contains("main"));
    }

    #[test]
    fn document_transport_is_bounded_text_only_and_path_free() {
        use crate::advisor_attachment::{AdvisorContentCategory, AdvisorContentDisposal};
        use crate::advisor_document_attachment::{
            AdvisorDocumentAttachmentManifest, AdvisorDocumentMediaType, AdvisorDocumentProjection,
            AdvisorDocumentProjectionKind,
        };
        let document = ClaimedAdvisorDocumentAttachment {
            manifest: AdvisorDocumentAttachmentManifest {
                attachment_id: "018f0000-0000-7000-8000-000000000099".to_owned(),
                display_name: "brief.pdf".to_owned(),
                content_category: AdvisorContentCategory::Document,
                media_type: AdvisorDocumentMediaType::Pdf,
                byte_size: 12,
                sha256: "a".repeat(64),
                projection: AdvisorDocumentProjection {
                    kind: AdvisorDocumentProjectionKind::PdfPlainTextV1,
                    schema_version: 1,
                    page_count: 1,
                    processed_page_count: 1,
                    included_page_count: 1,
                    omitted_page_count: 0,
                    partial_page_count: 0,
                    projected_byte_size: 11,
                    outline_entry_count: 0,
                    truncated: false,
                    warnings: Vec::new(),
                },
                disposal: AdvisorContentDisposal::TransientMemoryOneSend,
            },
            projection_text: "safe summary".to_owned(),
        };
        let input = advisor_input("Plan safely", None, None, Some(&document), None);
        assert!(input.contains("Plan safely"));
        assert!(input.contains("safe summary"));
        assert!(!input.contains("/mnt/"));
        assert!(!input.contains("localImage"));
        assert!(!input.contains("data:application/pdf"));
        assert!(!input.contains("%PDF-"));
    }

    #[test]
    fn archive_transport_is_manifest_text_only_and_path_free() {
        use crate::advisor_archive_attachment::{
            AdvisorArchiveAttachmentManifest, AdvisorArchiveMediaType, AdvisorArchiveProjection,
            AdvisorArchiveProjectionKind,
        };
        use crate::advisor_attachment::{AdvisorContentCategory, AdvisorContentDisposal};
        let archive = ClaimedAdvisorArchiveAttachment {
            manifest: AdvisorArchiveAttachmentManifest {
                attachment_id: "018f0000-0000-7000-8000-000000000098".to_owned(),
                display_name: "safe.zip".to_owned(),
                content_category: AdvisorContentCategory::Archive,
                media_type: AdvisorArchiveMediaType::Zip,
                byte_size: 12,
                sha256: "b".repeat(64),
                projection: AdvisorArchiveProjection {
                    kind: AdvisorArchiveProjectionKind::ArchiveManifestV1,
                    schema_version: 1,
                    discovered_entry_count: 1,
                    included_entry_count: 1,
                    omitted_entry_count: 0,
                    declared_aggregate_uncompressed_bytes: 4,
                    manifest_byte_size: 90,
                    truncated: false,
                    warnings: Vec::new(),
                },
                disposal: AdvisorContentDisposal::TransientMemoryOneSend,
            },
            projection_text: "archive-manifest-v1\nnotes.txt\tFile\t2\t4\n".to_owned(),
        };
        let input = advisor_input("Review safely", None, None, None, Some(&archive));
        assert!(input.contains("archive-manifest-v1"));
        assert!(input.contains("notes.txt"));
        assert!(!input.contains("/mnt/"));
        assert!(!input.contains("localImage"));
        assert!(!input.contains("data:application/zip"));
        assert!(!input.contains("PK\\x03\\x04"));
    }

    #[tokio::test]
    async fn managed_advisor_starts_without_project_or_tool_capabilities() {
        let script = r#"
read -r initialize
case "$initialize" in
  *'"method":"initialize"'*'"clientInfo":{"name":"quireforge","title":"QuireForge","version":"0.1.0-beta.30"}'*) ;;
  *) exit 70 ;;
esac
printf '%s\n' '{"id":1,"result":{}}'
read -r thread
printf '%s\n' '{"id":2,"result":{"thread":{"id":"018f0000-0000-7000-8000-000000000020"}}}'
read -r turn
printf '%s\n' '{"id":3,"result":{"turn":{"id":"018f0000-0000-7000-8000-000000000030","status":"inProgress"}}}'
printf '%s\n' '{"method":"turn/completed","params":{"threadId":"018f0000-0000-7000-8000-000000000020","turn":{"id":"018f0000-0000-7000-8000-000000000030","status":"completed"}}}'
"#;
        let service =
            AdvisorConversationService::with_command(AppServerCommand::test("sh", &["-c", script]));
        let started = service
            .start(
                AdvisorConversationStartRequest {
                    prompt: "Plan the next safe step.".to_owned(),
                    project_id: None,
                    attachment_id: None,
                    attachment_manifest_sha256: None,
                    attachment_confirmation: None,
                    image_attachment_id: None,
                    image_attachment_manifest_sha256: None,
                    image_attachment_confirmation: None,
                    document_attachment_id: None,
                    document_attachment_manifest_sha256: None,
                    document_attachment_confirmation: None,
                    archive_attachment_id: None,
                    archive_attachment_manifest_sha256: None,
                    archive_attachment_confirmation: None,
                },
                &CodexAuthSnapshot::authenticated(AuthAccountKind::Chatgpt),
                &ProjectService::in_memory(),
                None,
                None,
                None,
                None,
                None,
            )
            .await;
        assert_eq!(
            started.state,
            AdvisorConversationState::Running,
            "unexpected Advisor start snapshot: {started:?}"
        );
        assert_eq!(started.mode, "advisor");
        assert!(!started.project_state_included);
        assert!(started.conversation_id.is_some());
        let completed = service
            .poll(started.conversation_id.expect("conversation ID is present"))
            .await;
        assert_eq!(completed.state, AdvisorConversationState::Completed);
    }

    #[tokio::test]
    async fn advisor_retries_thread_start_once_with_the_same_empty_capability_boundary() {
        let script = r#"
read -r initialize
case "$initialize" in
  *'"method":"initialize"'*'"clientInfo":{"name":"quireforge","title":"QuireForge","version":"0.1.0-beta.30"}'*) ;;
  *) exit 70 ;;
esac
printf '%s\n' '{"id":1,"result":{}}'
read -r _primary_thread
printf '%s\n' '{"id":2,"error":{"code":-32602,"message":"redacted"}}'
read -r _compatibility_thread
printf '%s\n' '{"id":3,"result":{"thread":{"id":"018f0000-0000-7000-8000-000000000020"}}}'
read -r turn
printf '%s\n' '{"id":4,"result":{"turn":{"id":"018f0000-0000-7000-8000-000000000030","status":"inProgress"}}}'
printf '%s\n' '{"method":"turn/completed","params":{"threadId":"018f0000-0000-7000-8000-000000000020","turn":{"id":"018f0000-0000-7000-8000-000000000030","status":"completed"}}}'
"#;
        let service =
            AdvisorConversationService::with_command(AppServerCommand::test("sh", &["-c", script]));
        let started = service
            .start(
                AdvisorConversationStartRequest {
                    prompt: "Plan the next safe step.".to_owned(),
                    project_id: None,
                    attachment_id: None,
                    attachment_manifest_sha256: None,
                    attachment_confirmation: None,
                    image_attachment_id: None,
                    image_attachment_manifest_sha256: None,
                    image_attachment_confirmation: None,
                    document_attachment_id: None,
                    document_attachment_manifest_sha256: None,
                    document_attachment_confirmation: None,
                    archive_attachment_id: None,
                    archive_attachment_manifest_sha256: None,
                    archive_attachment_confirmation: None,
                },
                &CodexAuthSnapshot::authenticated(AuthAccountKind::Chatgpt),
                &ProjectService::in_memory(),
                None,
                None,
                None,
                None,
                None,
            )
            .await;
        assert_eq!(
            started.state,
            AdvisorConversationState::Running,
            "unexpected Advisor start snapshot: {started:?}"
        );
        let completed = service
            .poll(started.conversation_id.expect("conversation ID is present"))
            .await;
        assert_eq!(completed.state, AdvisorConversationState::Completed);
    }

    #[tokio::test]
    async fn advisor_reports_only_the_closed_thread_start_rejection_code() {
        let script = r#"
read -r _initialize
printf '%s\n' '{"id":1,"result":{}}'
read -r _primary_thread
printf '%s\n' '{"id":2,"error":{"code":-32602,"message":"do not expose this"}}'
read -r _compatibility_thread
printf '%s\n' '{"id":3,"error":{"code":-32602,"message":"do not expose this either"}}'
"#;
        let service =
            AdvisorConversationService::with_command(AppServerCommand::test("sh", &["-c", script]));
        let snapshot = service
            .start(
                AdvisorConversationStartRequest {
                    prompt: "Plan the next safe step.".to_owned(),
                    project_id: None,
                    attachment_id: None,
                    attachment_manifest_sha256: None,
                    attachment_confirmation: None,
                    image_attachment_id: None,
                    image_attachment_manifest_sha256: None,
                    image_attachment_confirmation: None,
                    document_attachment_id: None,
                    document_attachment_manifest_sha256: None,
                    document_attachment_confirmation: None,
                    archive_attachment_id: None,
                    archive_attachment_manifest_sha256: None,
                    archive_attachment_confirmation: None,
                },
                &CodexAuthSnapshot::authenticated(AuthAccountKind::Chatgpt),
                &ProjectService::in_memory(),
                None,
                None,
                None,
                None,
                None,
            )
            .await;
        assert_eq!(snapshot.state, AdvisorConversationState::Unavailable);
        assert_eq!(
            snapshot.diagnostic_code,
            Some(AdvisorConversationDiagnosticCode::ThreadStartRejected)
        );
        assert!(snapshot.events.is_empty());
    }

    #[tokio::test]
    async fn only_managed_chatgpt_auth_can_start_advisor() {
        let service = AdvisorConversationService::with_command(AppServerCommand::test(
            "sh",
            &["-c", "exit 99"],
        ));
        let snapshot = service
            .start(
                AdvisorConversationStartRequest {
                    prompt: "Hello".to_owned(),
                    project_id: None,
                    attachment_id: None,
                    attachment_manifest_sha256: None,
                    attachment_confirmation: None,
                    image_attachment_id: None,
                    image_attachment_manifest_sha256: None,
                    image_attachment_confirmation: None,
                    document_attachment_id: None,
                    document_attachment_manifest_sha256: None,
                    document_attachment_confirmation: None,
                    archive_attachment_id: None,
                    archive_attachment_manifest_sha256: None,
                    archive_attachment_confirmation: None,
                },
                &CodexAuthSnapshot::authenticated(AuthAccountKind::ApiKey),
                &ProjectService::in_memory(),
                None,
                None,
                None,
                None,
                None,
            )
            .await;
        assert_eq!(snapshot.state, AdvisorConversationState::Unavailable);
        assert_eq!(
            snapshot.diagnostic_code,
            Some(AdvisorConversationDiagnosticCode::AuthenticationUnavailable)
        );
    }
}
