//! M69A's isolated local-only chat boundary.
//!
//! This module deliberately owns only typed user text and the bounded runtime
//! result. It has no project, source, artifact, review, provider, or tool
//! types in its public interface.

use crate::local_runtime::{local_clock, LocalRuntimeService, LocalRuntimeSnapshot};
use serde::Deserialize;
use std::sync::Arc;

const CHAT_TEXT_BYTE_LIMIT: usize = 96 * 1024;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct LocalChatRequest {
    pub message: String,
    #[serde(default)]
    pub interaction_profile: LocalChatInteractionProfile,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum LocalChatInteractionProfile {
    #[default]
    Direct,
    Conversational,
}

impl LocalChatInteractionProfile {
    fn system_prompt_suffix(&self) -> &'static str {
        match self {
            Self::Direct => "Keep the answer concise and pragmatic.",
            Self::Conversational => {
                "Use a warmer, exploratory tone while remaining concise and factual."
            }
        }
    }
}

pub(crate) struct LocalChatService {
    runtime: Arc<LocalRuntimeService>,
}

impl LocalChatService {
    pub(crate) fn new(runtime: Arc<LocalRuntimeService>) -> Self {
        Self { runtime }
    }

    pub(crate) fn run(&self, request: LocalChatRequest) -> LocalRuntimeSnapshot {
        if request.message.trim().is_empty() || request.message.as_bytes().contains(&0) {
            return failed("invalid-request");
        }
        if request.message.len() > CHAT_TEXT_BYTE_LIMIT {
            return failed("input-too-large");
        }
        if asks_for_local_clock(&request.message) {
            return local_clock_snapshot();
        }
        if !self.runtime.availability().available {
            return LocalRuntimeService::unavailable_snapshot();
        }
        let Ok(reservation) = self.runtime.reserve_local_chat() else {
            return failed("runtime-busy");
        };
        reservation.run_local_chat(
            request.message.as_bytes(),
            request.interaction_profile.system_prompt_suffix(),
        )
    }

    pub(crate) fn cancel(&self) -> bool {
        self.runtime.request_cancel_local_chat()
    }
}

fn asks_for_local_clock(message: &str) -> bool {
    let normalized = message
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() {
                byte.to_ascii_lowercase()
            } else {
                b' '
            }
        })
        .collect::<Vec<_>>();
    let text = String::from_utf8_lossy(&normalized)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    [
        "what time is it",
        "what s the time",
        "current time",
        "tell me the time",
        "what is the date",
        "what s the date",
        "what date is it",
        "today s date",
        "what day is it",
    ]
    .iter()
    .any(|phrase| text.contains(phrase))
}

fn local_clock_snapshot() -> LocalRuntimeSnapshot {
    match local_clock() {
        Some(clock) => LocalRuntimeSnapshot {
            schema_version: 1,
            local_only: true,
            state: "completed".into(),
            output: Some(format!("The local date and time is {clock}.")),
            diagnostic: None,
            input_token_limit: 4096,
            output_token_limit: 512,
            deadline_seconds: 60,
            memory_ceiling_mib: 6144,
        },
        None => failed("clock-unavailable"),
    }
}

fn failed(diagnostic: &str) -> LocalRuntimeSnapshot {
    LocalRuntimeSnapshot {
        schema_version: 1,
        local_only: true,
        state: "failed".into(),
        output: None,
        diagnostic: Some(diagnostic.into()),
        input_token_limit: 4096,
        output_token_limit: 512,
        deadline_seconds: 60,
        memory_ceiling_mib: 6144,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LocalChatInteractionProfile, LocalChatRequest, LocalChatService, CHAT_TEXT_BYTE_LIMIT,
    };
    use crate::local_runtime::LocalRuntimeService;
    use std::sync::Arc;

    fn service() -> LocalChatService {
        LocalChatService::new(Arc::new(LocalRuntimeService::default()))
    }

    #[test]
    fn local_chat_rejects_empty_and_nul_text_before_runtime_admission() {
        for message in ["   ".to_owned(), "hello\0world".to_owned()] {
            let snapshot = service().run(LocalChatRequest {
                message,
                interaction_profile: LocalChatInteractionProfile::Direct,
            });
            assert_eq!(snapshot.state, "failed");
            assert_eq!(snapshot.diagnostic.as_deref(), Some("invalid-request"));
        }
    }

    #[test]
    fn interaction_profiles_change_only_the_local_response_style_instruction() {
        assert_eq!(
            LocalChatInteractionProfile::Direct.system_prompt_suffix(),
            "Keep the answer concise and pragmatic."
        );
        assert_eq!(
            LocalChatInteractionProfile::Conversational.system_prompt_suffix(),
            "Use a warmer, exploratory tone while remaining concise and factual."
        );
    }

    #[test]
    fn local_chat_rejects_oversized_text_without_runtime_admission() {
        let snapshot = service().run(LocalChatRequest {
            message: "x".repeat(CHAT_TEXT_BYTE_LIMIT + 1),
            interaction_profile: LocalChatInteractionProfile::Direct,
        });
        assert_eq!(snapshot.state, "failed");
        assert_eq!(snapshot.diagnostic.as_deref(), Some("input-too-large"));
    }

    #[test]
    fn local_chat_cancellation_is_scoped_to_its_own_absent_turn() {
        assert!(!service().cancel());
    }

    #[test]
    fn local_clock_questions_complete_without_a_model_attempt() {
        for message in ["What time is it?", "What's the date today?"] {
            let snapshot = service().run(LocalChatRequest {
                message: message.into(),
                interaction_profile: LocalChatInteractionProfile::Direct,
            });
            assert_eq!(snapshot.state, "completed");
            assert!(snapshot
                .output
                .as_deref()
                .is_some_and(|output| output.contains("local date and time")));
        }
    }

    #[test]
    fn ordinary_mentions_of_time_do_not_bypass_the_bounded_runtime() {
        let snapshot = service().run(LocalChatRequest {
            message: "Explain time complexity in one sentence.".into(),
            interaction_profile: LocalChatInteractionProfile::Direct,
        });
        assert_eq!(snapshot.state, "failed");
        assert_eq!(snapshot.diagnostic.as_deref(), Some("model-unavailable"));
    }
}
