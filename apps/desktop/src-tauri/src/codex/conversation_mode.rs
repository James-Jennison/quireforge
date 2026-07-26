use serde::Serialize;

use super::auth::types::{AuthAccountKind, AuthLoginMethod, AuthState, CodexAuthSnapshot};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationMode {
    Chat,
    Codex,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChatAuthenticationState {
    Ready,
    SignInRequired,
    SignInPending,
    Unavailable,
}

pub const CHAT_AUTHENTICATION_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatAuthenticationSnapshot {
    pub schema_version: u16,
    pub state: ChatAuthenticationState,
    /// A closed capability catalog. The native process, rather than a UI
    /// label, owns the distinction between the Chat and Codex modes.
    pub capabilities: [ConversationModeCapability; 2],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationModeCapability {
    pub mode: ConversationMode,
    pub requires_attached_project: bool,
    pub allows_native_actions: bool,
    pub allows_terminal: bool,
    pub allows_git: bool,
    pub allows_worktrees: bool,
    pub allows_integrations: bool,
    pub requires_managed_chat_gpt_auth: bool,
}

impl ConversationModeCapability {
    pub const fn for_mode(mode: ConversationMode) -> Self {
        match mode {
            ConversationMode::Chat => Self {
                mode,
                requires_attached_project: false,
                allows_native_actions: false,
                allows_terminal: false,
                allows_git: false,
                allows_worktrees: false,
                allows_integrations: false,
                requires_managed_chat_gpt_auth: true,
            },
            ConversationMode::Codex => Self {
                mode,
                requires_attached_project: true,
                allows_native_actions: true,
                allows_terminal: true,
                allows_git: true,
                allows_worktrees: true,
                allows_integrations: true,
                requires_managed_chat_gpt_auth: false,
            },
        }
    }
}

pub fn managed_chat_authentication_state(auth: &CodexAuthSnapshot) -> ChatAuthenticationState {
    match (auth.state, auth.account_kind, auth.pending_method) {
        (AuthState::Authenticated, Some(AuthAccountKind::Chatgpt), None) => {
            ChatAuthenticationState::Ready
        }
        (AuthState::LoginPending, None, Some(AuthLoginMethod::Browser)) => {
            ChatAuthenticationState::SignInPending
        }
        (AuthState::Unauthenticated, None, None) => ChatAuthenticationState::SignInRequired,
        _ => ChatAuthenticationState::Unavailable,
    }
}

pub fn chat_authentication_snapshot(auth: &CodexAuthSnapshot) -> ChatAuthenticationSnapshot {
    ChatAuthenticationSnapshot {
        schema_version: CHAT_AUTHENTICATION_SCHEMA_VERSION,
        state: managed_chat_authentication_state(auth),
        capabilities: [
            ConversationModeCapability::for_mode(ConversationMode::Chat),
            ConversationModeCapability::for_mode(ConversationMode::Codex),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_is_not_a_project_or_native_action_capability() {
        let capability = ConversationModeCapability::for_mode(ConversationMode::Chat);
        assert!(!capability.requires_attached_project);
        assert!(!capability.allows_native_actions);
        assert!(!capability.allows_terminal);
        assert!(capability.requires_managed_chat_gpt_auth);
        let codex = ConversationModeCapability::for_mode(ConversationMode::Codex);
        assert!(codex.requires_attached_project);
        assert!(codex.allows_native_actions);
    }

    #[test]
    fn only_managed_chatgpt_auth_can_enable_chat() {
        assert_eq!(
            managed_chat_authentication_state(&CodexAuthSnapshot::authenticated(
                AuthAccountKind::Chatgpt,
            )),
            ChatAuthenticationState::Ready
        );
        assert_eq!(
            managed_chat_authentication_state(&CodexAuthSnapshot::authenticated(
                AuthAccountKind::ApiKey,
            )),
            ChatAuthenticationState::Unavailable
        );
    }

    #[test]
    fn snapshot_exposes_the_closed_mode_catalog_and_readiness() {
        let snapshot = chat_authentication_snapshot(&CodexAuthSnapshot::authenticated(
            AuthAccountKind::Chatgpt,
        ));
        assert_eq!(snapshot.schema_version, CHAT_AUTHENTICATION_SCHEMA_VERSION);
        assert_eq!(snapshot.state, ChatAuthenticationState::Ready);
        assert_eq!(snapshot.capabilities[0].mode, ConversationMode::Chat);
        assert_eq!(snapshot.capabilities[1].mode, ConversationMode::Codex);
    }
}
