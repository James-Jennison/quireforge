use std::{collections::HashMap, time::Duration};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::project::{ChatConversationMetadata, ProjectService};

use super::{
    app_server::{
        validate_uuid_v7, AppServerCommand, AppServerNotification, AppServerProcess,
        ConversationItemKind, ConversationNotification, ConversationTurnStatus,
    },
    auth::types::CodexAuthSnapshot,
    conversation_mode::{managed_chat_authentication_state, ChatAuthenticationState},
    error::CodexAdapterError,
};

const CHAT_SCHEMA_VERSION: u16 = 1;
const MAX_CHAT_PROMPT_BYTES: usize = 64 * 1024;
const FIRST_POLL_WAIT: Duration = Duration::from_millis(200);
const DRAIN_POLL_WAIT: Duration = Duration::from_millis(1);
const MAX_POLL_EVENTS: usize = 32;
const MAX_RECENT_CONVERSATIONS: usize = 64;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChatConversationStartRequest {
    pub prompt: String,
    #[serde(default)]
    pub interaction_profile: crate::codex::conversation::types::InteractionProfile,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChatConversationState {
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
pub enum ChatConversationDiagnosticCode {
    AuthenticationRequired,
    AuthenticationUnavailable,
    ConversationNotFound,
    ConversationActive,
    InvalidRequest,
    RuntimeUnavailable,
    ProtocolInvalid,
    CapabilityBlocked,
    MetadataUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ChatConversationEvent {
    AgentMessageDelta { sequence: u64, delta: String },
    ReasoningSummaryDelta { sequence: u64, delta: String },
    Error { sequence: u64, code: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatConversationSnapshot {
    pub schema_version: u16,
    pub mode: &'static str,
    pub state: ChatConversationState,
    pub conversation_id: Option<String>,
    pub thread_id: Option<String>,
    pub events: Vec<ChatConversationEvent>,
    pub diagnostic_code: Option<ChatConversationDiagnosticCode>,
}

impl ChatConversationSnapshot {
    fn empty() -> Self {
        Self {
            schema_version: CHAT_SCHEMA_VERSION,
            mode: "chat",
            state: ChatConversationState::Empty,
            conversation_id: None,
            thread_id: None,
            events: Vec::new(),
            diagnostic_code: None,
        }
    }

    fn unavailable(diagnostic_code: ChatConversationDiagnosticCode) -> Self {
        Self {
            state: ChatConversationState::Unavailable,
            diagnostic_code: Some(diagnostic_code),
            ..Self::empty()
        }
    }
}

pub struct ChatConversationService {
    state: Mutex<ChatConversationServiceState>,
    command: AppServerCommand,
}

struct ChatConversationServiceState {
    active: Option<ActiveChatConversation>,
    recent: HashMap<String, ChatConversationSnapshot>,
    last: ChatConversationSnapshot,
    starting: bool,
}

struct ActiveChatConversation {
    conversation_id: String,
    thread_id: String,
    // A completed no-project Chat turn intentionally leaves only this bounded
    // in-memory managed session alive so the next explicit send can continue
    // the same provider thread. Nothing is persisted as transcript content.
    turn_id: Option<String>,
    next_sequence: u64,
    process: AppServerProcess,
}

impl ChatConversationServiceState {
    fn empty() -> Self {
        Self {
            active: None,
            recent: HashMap::new(),
            last: ChatConversationSnapshot::empty(),
            starting: false,
        }
    }

    fn remember(&mut self, snapshot: ChatConversationSnapshot) {
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

impl Default for ChatConversationService {
    fn default() -> Self {
        Self {
            state: Mutex::new(ChatConversationServiceState::empty()),
            command: AppServerCommand::codex("codex"),
        }
    }
}

impl ChatConversationService {
    #[cfg(test)]
    fn with_command(command: AppServerCommand) -> Self {
        Self {
            state: Mutex::new(ChatConversationServiceState::empty()),
            command,
        }
    }

    pub async fn status(&self) -> ChatConversationSnapshot {
        let state = self.state.lock().await;
        let mut snapshot = state.last.clone();
        snapshot.events.clear();
        snapshot
    }

    pub async fn start(
        &self,
        request: ChatConversationStartRequest,
        authentication: &CodexAuthSnapshot,
        projects: &ProjectService,
    ) -> ChatConversationSnapshot {
        match managed_chat_authentication_state(authentication) {
            ChatAuthenticationState::SignInRequired | ChatAuthenticationState::SignInPending => {
                return ChatConversationSnapshot::unavailable(
                    ChatConversationDiagnosticCode::AuthenticationRequired,
                );
            }
            ChatAuthenticationState::Unavailable => {
                return ChatConversationSnapshot::unavailable(
                    ChatConversationDiagnosticCode::AuthenticationUnavailable,
                );
            }
            ChatAuthenticationState::Ready => {}
        }
        if !valid_prompt(&request.prompt) {
            return ChatConversationSnapshot::unavailable(
                ChatConversationDiagnosticCode::InvalidRequest,
            );
        }

        {
            let mut state = self.state.lock().await;
            if state.starting {
                return ChatConversationSnapshot::unavailable(
                    ChatConversationDiagnosticCode::ConversationActive,
                );
            }
            if let Some(active) = state.active.as_mut() {
                if active.turn_id.is_some() {
                    return ChatConversationSnapshot::unavailable(
                        ChatConversationDiagnosticCode::ConversationActive,
                    );
                }
                match start_chat_turn_on_thread(&mut active.process, &active.thread_id, &request)
                    .await
                {
                    Ok(turn_id) => {
                        active.turn_id = Some(turn_id);
                        let snapshot = active.snapshot(Vec::new(), None);
                        state.remember(snapshot.clone());
                        return snapshot;
                    }
                    Err(diagnostic_code) => {
                        let snapshot = ChatConversationSnapshot {
                            state: ChatConversationState::Failed,
                            ..active.snapshot(Vec::new(), Some(diagnostic_code))
                        };
                        let _ = active.process.shutdown().await;
                        let _ = state.active.take();
                        state.remember(snapshot.clone());
                        return snapshot;
                    }
                }
            }
            state.starting = true;
        }

        let started = start_chat_process(&self.command, &request, projects).await;
        let mut state = self.state.lock().await;
        state.starting = false;
        match started {
            Ok(active) => {
                let snapshot = active.snapshot(Vec::new(), None);
                state.active = Some(active);
                state.remember(snapshot.clone());
                snapshot
            }
            Err(diagnostic_code) => {
                let snapshot = ChatConversationSnapshot::unavailable(diagnostic_code);
                state.remember(snapshot.clone());
                snapshot
            }
        }
    }

    pub async fn poll(&self, conversation_id: String) -> ChatConversationSnapshot {
        if validate_uuid_v7(&conversation_id).is_err() {
            return ChatConversationSnapshot::unavailable(
                ChatConversationDiagnosticCode::ConversationNotFound,
            );
        }
        let mut state = self.state.lock().await;
        let Some(active) = state.active.as_mut() else {
            return state
                .recent
                .get(&conversation_id)
                .cloned()
                .unwrap_or_else(|| {
                    ChatConversationSnapshot::unavailable(
                        ChatConversationDiagnosticCode::ConversationNotFound,
                    )
                });
        };
        if active.conversation_id != conversation_id {
            return state
                .recent
                .get(&conversation_id)
                .cloned()
                .unwrap_or_else(|| {
                    ChatConversationSnapshot::unavailable(
                        ChatConversationDiagnosticCode::ConversationNotFound,
                    )
                });
        }
        if active.turn_id.is_none() {
            return state
                .recent
                .get(&conversation_id)
                .cloned()
                .unwrap_or_else(|| {
                    ChatConversationSnapshot::unavailable(
                        ChatConversationDiagnosticCode::ConversationNotFound,
                    )
                });
        }

        let mut events = Vec::new();
        let mut terminal: Option<(
            ChatConversationState,
            Option<ChatConversationDiagnosticCode>,
        )> = None;
        for index in 0..MAX_POLL_EVENTS {
            let wait = if index == 0 {
                FIRST_POLL_WAIT
            } else {
                DRAIN_POLL_WAIT
            };
            match active.process.next_notification_with_timeout(wait).await {
                Ok(Some(notification)) => match apply_chat_notification(active, notification) {
                    Ok(ChatNotificationOutcome::Event(event)) => events.push(event),
                    Ok(ChatNotificationOutcome::None) => {}
                    Ok(ChatNotificationOutcome::Completed) => {
                        terminal = Some((ChatConversationState::Completed, None));
                        break;
                    }
                    Err(diagnostic) => {
                        terminal = Some((ChatConversationState::Blocked, Some(diagnostic)));
                        break;
                    }
                },
                Ok(None) => break,
                Err(error) => {
                    terminal = Some((
                        ChatConversationState::Failed,
                        Some(map_adapter_error(error)),
                    ));
                    break;
                }
            }
        }

        if let Some((terminal_state, diagnostic_code)) = terminal {
            let snapshot = active.snapshot(events, diagnostic_code);
            let snapshot = ChatConversationSnapshot {
                state: terminal_state,
                ..snapshot
            };
            if terminal_state == ChatConversationState::Completed {
                active.turn_id = None;
            } else {
                let _ = active.process.shutdown().await;
                let _ = state.active.take();
            }
            state.remember(snapshot.clone());
            return snapshot;
        }

        let snapshot = active.snapshot(events, None);
        state.remember(snapshot.clone());
        snapshot
    }

    pub async fn interrupt(&self, conversation_id: String) -> ChatConversationSnapshot {
        if validate_uuid_v7(&conversation_id).is_err() {
            return ChatConversationSnapshot::unavailable(
                ChatConversationDiagnosticCode::ConversationNotFound,
            );
        }
        let mut state = self.state.lock().await;
        let Some(active) = state.active.as_mut() else {
            return state
                .recent
                .get(&conversation_id)
                .cloned()
                .unwrap_or_else(|| {
                    ChatConversationSnapshot::unavailable(
                        ChatConversationDiagnosticCode::ConversationNotFound,
                    )
                });
        };
        if active.conversation_id != conversation_id {
            return ChatConversationSnapshot::unavailable(
                ChatConversationDiagnosticCode::ConversationNotFound,
            );
        }
        let Some(turn_id) = active.turn_id.clone() else {
            return state
                .recent
                .get(&conversation_id)
                .cloned()
                .unwrap_or_else(|| {
                    ChatConversationSnapshot::unavailable(
                        ChatConversationDiagnosticCode::ConversationNotFound,
                    )
                });
        };
        let interruption = active
            .process
            .request(
                "turn/interrupt",
                json!({
                    "threadId": active.thread_id.clone(),
                    "turnId": turn_id,
                }),
            )
            .await;
        let diagnostic_code = interruption.err().map(map_adapter_error);
        let snapshot = ChatConversationSnapshot {
            state: if diagnostic_code.is_some() {
                ChatConversationState::Failed
            } else {
                ChatConversationState::Interrupted
            },
            ..active.snapshot(Vec::new(), diagnostic_code)
        };
        let _ = active.process.shutdown().await;
        let _ = state.active.take();
        state.remember(snapshot.clone());
        snapshot
    }
}

impl ActiveChatConversation {
    fn snapshot(
        &self,
        events: Vec<ChatConversationEvent>,
        diagnostic_code: Option<ChatConversationDiagnosticCode>,
    ) -> ChatConversationSnapshot {
        ChatConversationSnapshot {
            schema_version: CHAT_SCHEMA_VERSION,
            mode: "chat",
            state: ChatConversationState::Running,
            conversation_id: Some(self.conversation_id.clone()),
            thread_id: Some(self.thread_id.clone()),
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

async fn start_chat_process(
    command: &AppServerCommand,
    request: &ChatConversationStartRequest,
    projects: &ProjectService,
) -> Result<ActiveChatConversation, ChatConversationDiagnosticCode> {
    let mut process = AppServerProcess::spawn(command.clone())
        .map_err(|_| ChatConversationDiagnosticCode::RuntimeUnavailable)?;
    let started = start_chat_session(&mut process, request, projects).await;
    match started {
        Ok((conversation_id, thread_id, turn_id)) => Ok(ActiveChatConversation {
            conversation_id,
            thread_id,
            turn_id: Some(turn_id),
            next_sequence: 1,
            process,
        }),
        Err(error) => {
            let _ = process.shutdown().await;
            Err(error)
        }
    }
}

async fn start_chat_session(
    process: &mut AppServerProcess,
    request: &ChatConversationStartRequest,
    projects: &ProjectService,
) -> Result<(String, String, String), ChatConversationDiagnosticCode> {
    // This fixed profile has neither a filesystem root nor any enabled tool
    // environment. There is intentionally no permissive fallback request.
    // Keep the chat adapter on the same documented app-server initialization
    // contract as the managed Advisor adapter.  Current Codex app-server
    // versions require clientInfo before they will accept any conversation
    // request; an empty initialize payload leaves the visible composer unable
    // to start a turn.
    process.initialize().await.map_err(map_adapter_error)?;
    let thread_result = process
        .request(
            "thread/start",
            chat_thread_start_params(request.interaction_profile),
        )
        .await
        .map_err(map_adapter_error)?;
    let thread_id = parse_thread_start(thread_result)?;
    let conversation_id = Uuid::now_v7().to_string();
    projects
        .record_chat_conversation_metadata(ChatConversationMetadata {
            conversation_id: &conversation_id,
            codex_thread_id: &thread_id,
        })
        .map_err(|_| ChatConversationDiagnosticCode::MetadataUnavailable)?;
    let turn_id = start_chat_turn_on_thread(process, &thread_id, request).await?;
    Ok((conversation_id, thread_id, turn_id))
}

async fn start_chat_turn_on_thread(
    process: &mut AppServerProcess,
    thread_id: &str,
    request: &ChatConversationStartRequest,
) -> Result<String, ChatConversationDiagnosticCode> {
    let turn_result = process
        .request(
            "turn/start",
            chat_turn_start_params(thread_id, &request.prompt, request.interaction_profile),
        )
        .await
        .map_err(map_adapter_error)?;
    parse_turn_start(turn_result)
}

fn chat_thread_start_params(
    profile: crate::codex::conversation::types::InteractionProfile,
) -> Value {
    // `never` is permitted only because this exact profile has no cwd, no
    // tools, read-only filesystem policy, and no network. Adding a capability
    // here requires replacing this fixed policy with an explicit approval path.
    let params = json!({
        "cwd": Value::Null,
        "environments": [],
        "dynamicTools": [],
        "approvalPolicy": "never",
        "sandbox": "read-only",
        "personality": profile.as_protocol_value(),
    });
    assert_isolated_never_policy(&params);
    params
}

fn assert_isolated_never_policy(params: &Value) {
    assert_eq!(params["approvalPolicy"], "never");
    assert_eq!(params["cwd"], Value::Null);
    assert_eq!(params["environments"], json!([]));
    assert_eq!(params["dynamicTools"], json!([]));
    assert_eq!(params["sandbox"], "read-only");
}

fn chat_turn_start_params(
    thread_id: &str,
    prompt: &str,
    profile: crate::codex::conversation::types::InteractionProfile,
) -> Value {
    json!({
        "threadId": thread_id,
        "personality": profile.as_protocol_value(),
        "input": [{"type": "text", "text": prompt}],
        "cwd": Value::Null,
        "approvalPolicy": "never",
        "sandboxPolicy": {"type": "readOnly", "networkAccess": false},
    })
}

enum ChatNotificationOutcome {
    None,
    Event(ChatConversationEvent),
    Completed,
}

fn apply_chat_notification(
    active: &mut ActiveChatConversation,
    notification: AppServerNotification,
) -> Result<ChatNotificationOutcome, ChatConversationDiagnosticCode> {
    let notification = match notification {
        AppServerNotification::Conversation(notification) => notification,
        AppServerNotification::ConversationRequest(_) => {
            return Err(ChatConversationDiagnosticCode::CapabilityBlocked);
        }
        AppServerNotification::AccountLoginCompleted { .. }
        | AppServerNotification::AccountUpdated
        | AppServerNotification::McpOauthLoginCompleted { .. }
        | AppServerNotification::IntegrationRefresh(_) => return Ok(ChatNotificationOutcome::None),
    };
    match notification {
        ConversationNotification::ThreadStarted { thread_id }
        | ConversationNotification::ThreadArchived { thread_id }
        | ConversationNotification::ThreadUnarchived { thread_id } => {
            ensure_thread(active, &thread_id)?;
            Ok(ChatNotificationOutcome::None)
        }
        ConversationNotification::TurnStarted { thread_id, turn_id } => {
            ensure_turn(active, &thread_id, &turn_id)?;
            Ok(ChatNotificationOutcome::None)
        }
        ConversationNotification::AgentMessageDelta {
            thread_id,
            turn_id,
            item_id: _,
            delta,
        } => {
            ensure_turn(active, &thread_id, &turn_id)?;
            Ok(ChatNotificationOutcome::Event(
                ChatConversationEvent::AgentMessageDelta {
                    sequence: active.next_sequence(),
                    delta,
                },
            ))
        }
        ConversationNotification::AgentMessageCompleted { .. } => Ok(ChatNotificationOutcome::None),
        ConversationNotification::ReasoningSummaryDelta {
            thread_id,
            turn_id,
            delta,
        } => {
            ensure_turn(active, &thread_id, &turn_id)?;
            Ok(ChatNotificationOutcome::Event(
                ChatConversationEvent::ReasoningSummaryDelta {
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
            Ok(ChatNotificationOutcome::Event(
                ChatConversationEvent::Error {
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
                ConversationTurnStatus::Completed => Ok(ChatNotificationOutcome::Completed),
                ConversationTurnStatus::Interrupted | ConversationTurnStatus::Failed => {
                    Err(ChatConversationDiagnosticCode::ProtocolInvalid)
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
                Ok(ChatNotificationOutcome::None)
            } else {
                Err(ChatConversationDiagnosticCode::CapabilityBlocked)
            }
        }
        ConversationNotification::PlanUpdated { .. }
        | ConversationNotification::ActivityDelta { .. }
        | ConversationNotification::ServerRequestResolved { .. } => {
            Err(ChatConversationDiagnosticCode::CapabilityBlocked)
        }
    }
}

fn parse_thread_start(value: Value) -> Result<String, ChatConversationDiagnosticCode> {
    #[derive(Deserialize)]
    struct ThreadStartResult {
        thread: ProtocolId,
    }
    let result: ThreadStartResult = serde_json::from_value(value)
        .map_err(|_| ChatConversationDiagnosticCode::ProtocolInvalid)?;
    validate_uuid_v7(&result.thread.id)
        .map_err(|_| ChatConversationDiagnosticCode::ProtocolInvalid)?;
    Ok(result.thread.id)
}

fn parse_turn_start(value: Value) -> Result<String, ChatConversationDiagnosticCode> {
    #[derive(Deserialize)]
    struct TurnStartResult {
        turn: Turn,
    }
    #[derive(Deserialize)]
    struct Turn {
        id: String,
        status: String,
    }
    let result: TurnStartResult = serde_json::from_value(value)
        .map_err(|_| ChatConversationDiagnosticCode::ProtocolInvalid)?;
    validate_uuid_v7(&result.turn.id)
        .map_err(|_| ChatConversationDiagnosticCode::ProtocolInvalid)?;
    if result.turn.status != "inProgress" {
        return Err(ChatConversationDiagnosticCode::ProtocolInvalid);
    }
    Ok(result.turn.id)
}

#[derive(Deserialize)]
struct ProtocolId {
    id: String,
}

fn ensure_thread(
    active: &ActiveChatConversation,
    thread_id: &str,
) -> Result<(), ChatConversationDiagnosticCode> {
    (active.thread_id == thread_id)
        .then_some(())
        .ok_or(ChatConversationDiagnosticCode::ProtocolInvalid)
}

fn ensure_turn(
    active: &ActiveChatConversation,
    thread_id: &str,
    turn_id: &str,
) -> Result<(), ChatConversationDiagnosticCode> {
    ensure_thread(active, thread_id)?;
    (active.turn_id.as_deref() == Some(turn_id))
        .then_some(())
        .ok_or(ChatConversationDiagnosticCode::ProtocolInvalid)
}

fn valid_prompt(prompt: &str) -> bool {
    !prompt.trim().is_empty() && prompt.len() <= MAX_CHAT_PROMPT_BYTES && !prompt.contains('\0')
}

fn map_adapter_error(error: CodexAdapterError) -> ChatConversationDiagnosticCode {
    match error {
        CodexAdapterError::ProcessSpawnFailed | CodexAdapterError::ProcessExited => {
            ChatConversationDiagnosticCode::RuntimeUnavailable
        }
        _ => ChatConversationDiagnosticCode::ProtocolInvalid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codex::{auth::types::AuthAccountKind, InteractionProfile};

    #[test]
    fn rejects_invalid_prompt_without_starting_a_runtime() {
        assert!(!valid_prompt("   "));
        assert!(!valid_prompt("a\0b"));
        assert!(!valid_prompt(&"x".repeat(MAX_CHAT_PROMPT_BYTES + 1)));
        assert!(valid_prompt("Explain this error."));
    }

    #[test]
    fn fixed_chat_wire_parameters_cannot_carry_project_or_tool_authority() {
        let thread = chat_thread_start_params(InteractionProfile::Direct);
        assert_eq!(thread["cwd"], Value::Null);
        assert_eq!(thread["environments"], json!([]));
        assert_eq!(thread["dynamicTools"], json!([]));
        assert_eq!(thread["approvalPolicy"], "never");
        assert_eq!(thread["sandbox"], "read-only");
        assert!(thread.get("projectId").is_none());
        assert!(thread.get("integrations").is_none());
        assert!(thread.get("attachments").is_none());

        let turn = chat_turn_start_params("thread", "Prompt", InteractionProfile::Direct);
        assert_eq!(turn["cwd"], Value::Null);
        assert_eq!(turn["approvalPolicy"], "never");
        assert_eq!(turn["sandboxPolicy"]["networkAccess"], false);
        assert_eq!(turn["input"], json!([{"type": "text", "text": "Prompt"}]));
    }

    #[tokio::test]
    async fn managed_chat_starts_without_project_or_tool_capabilities() {
        let script = r#"
read -r initialize
case "$initialize" in
  *'"method":"initialize"'*'"clientInfo"'*) ;;
  *) exit 91 ;;
esac
printf '%s\n' '{"id":1,"result":{}}'
read -r thread
printf '%s\n' '{"id":2,"result":{"thread":{"id":"018f0000-0000-7000-8000-000000000020"}}}'
read -r turn
printf '%s\n' '{"id":3,"result":{"turn":{"id":"018f0000-0000-7000-8000-000000000030","status":"inProgress"}}}'
printf '%s\n' '{"method":"turn/completed","params":{"threadId":"018f0000-0000-7000-8000-000000000020","turn":{"id":"018f0000-0000-7000-8000-000000000030","status":"completed"}}}'
"#;
        let service =
            ChatConversationService::with_command(AppServerCommand::test("sh", &["-c", script]));
        let projects = ProjectService::in_memory();
        let started = service
            .start(
                ChatConversationStartRequest {
                    prompt: "Explain the failing test.".to_owned(),
                    interaction_profile: InteractionProfile::Direct,
                },
                &CodexAuthSnapshot::authenticated(AuthAccountKind::Chatgpt),
                &projects,
            )
            .await;
        assert_eq!(
            started.state,
            ChatConversationState::Running,
            "unexpected Chat start snapshot: {started:?}"
        );
        assert_eq!(started.mode, "chat");
        assert!(started.conversation_id.is_some());
        assert!(started.thread_id.is_some());

        let completed = service
            .poll(
                started
                    .conversation_id
                    .expect("start must allocate a bounded ID"),
            )
            .await;
        assert_eq!(completed.state, ChatConversationState::Completed);
        assert_eq!(completed.diagnostic_code, None);
    }

    #[tokio::test]
    async fn completed_chat_turn_reuses_the_explicitly_selected_managed_thread() {
        let script = r#"
read -r initialize
printf '%s\n' '{"id":1,"result":{}}'
read -r thread
case "$thread" in *'"method":"thread/start"'*) ;; *) exit 91 ;; esac
printf '%s\n' '{"id":2,"result":{"thread":{"id":"018f0000-0000-7000-8000-000000000020"}}}'
read -r first_turn
case "$first_turn" in *'"method":"turn/start"'*) ;; *) exit 92 ;; esac
printf '%s\n' '{"id":3,"result":{"turn":{"id":"018f0000-0000-7000-8000-000000000030","status":"inProgress"}}}'
printf '%s\n' '{"method":"turn/completed","params":{"threadId":"018f0000-0000-7000-8000-000000000020","turn":{"id":"018f0000-0000-7000-8000-000000000030","status":"completed"}}}'
read -r second_turn
case "$second_turn" in *'"method":"turn/start"'*'"threadId":"018f0000-0000-7000-8000-000000000020"'*) ;; *) exit 93 ;; esac
printf '%s\n' '{"id":4,"result":{"turn":{"id":"018f0000-0000-7000-8000-000000000040","status":"inProgress"}}}'
printf '%s\n' '{"method":"turn/completed","params":{"threadId":"018f0000-0000-7000-8000-000000000020","turn":{"id":"018f0000-0000-7000-8000-000000000040","status":"completed"}}}'
"#;
        let service =
            ChatConversationService::with_command(AppServerCommand::test("sh", &["-c", script]));
        let projects = ProjectService::in_memory();
        let authentication = CodexAuthSnapshot::authenticated(AuthAccountKind::Chatgpt);
        let first = service
            .start(
                ChatConversationStartRequest {
                    prompt: "First question".to_owned(),
                    interaction_profile: InteractionProfile::Direct,
                },
                &authentication,
                &projects,
            )
            .await;
        let conversation_id = first.conversation_id.clone().expect("conversation ID");
        let thread_id = first.thread_id.clone().expect("thread ID");
        assert_eq!(
            service.poll(conversation_id.clone()).await.state,
            ChatConversationState::Completed
        );

        let second = service
            .start(
                ChatConversationStartRequest {
                    prompt: "Follow-up question".to_owned(),
                    interaction_profile: InteractionProfile::Conversational,
                },
                &authentication,
                &projects,
            )
            .await;
        assert_eq!(second.state, ChatConversationState::Running);
        assert_eq!(
            second.conversation_id.as_deref(),
            Some(conversation_id.as_str())
        );
        assert_eq!(second.thread_id.as_deref(), Some(thread_id.as_str()));
        assert_eq!(
            service.poll(conversation_id).await.state,
            ChatConversationState::Completed
        );
    }

    #[tokio::test]
    async fn api_key_auth_is_not_a_chat_fallback() {
        let service =
            ChatConversationService::with_command(AppServerCommand::test("sh", &["-c", "exit 99"]));
        let unavailable = service
            .start(
                ChatConversationStartRequest {
                    prompt: "Hello".to_owned(),
                    interaction_profile: InteractionProfile::Direct,
                },
                &CodexAuthSnapshot::authenticated(AuthAccountKind::ApiKey),
                &ProjectService::in_memory(),
            )
            .await;
        assert_eq!(unavailable.state, ChatConversationState::Unavailable);
        assert_eq!(
            unavailable.diagnostic_code,
            Some(ChatConversationDiagnosticCode::AuthenticationUnavailable)
        );
    }
}
