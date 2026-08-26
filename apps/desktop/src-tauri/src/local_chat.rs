//! M69A's isolated local-only chat boundary.
//!
//! This module deliberately owns only typed user text and the bounded runtime
//! result. It has no project, source, artifact, review, provider, or tool
//! types in its public interface.

use crate::local_runtime::{LocalRuntimeService, LocalRuntimeSnapshot};
use serde::Deserialize;
use std::sync::Arc;

const CHAT_TEXT_BYTE_LIMIT: usize = 96 * 1024;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct LocalChatRequest {
    pub message: String,
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
        if !self.runtime.availability().available {
            return LocalRuntimeService::unavailable_snapshot();
        }
        let Ok(reservation) = self.runtime.reserve_local_chat() else {
            return failed("runtime-busy");
        };
        reservation.run_local_chat(request.message.as_bytes())
    }

    pub(crate) fn cancel(&self) -> bool {
        self.runtime.request_cancel_local_chat()
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
    use super::{LocalChatRequest, LocalChatService, CHAT_TEXT_BYTE_LIMIT};
    use crate::local_runtime::LocalRuntimeService;
    use std::sync::Arc;

    fn service() -> LocalChatService {
        LocalChatService::new(Arc::new(LocalRuntimeService::default()))
    }

    #[test]
    fn local_chat_rejects_empty_and_nul_text_before_runtime_admission() {
        for message in ["   ".to_owned(), "hello\0world".to_owned()] {
            let snapshot = service().run(LocalChatRequest { message });
            assert_eq!(snapshot.state, "failed");
            assert_eq!(snapshot.diagnostic.as_deref(), Some("invalid-request"));
        }
    }

    #[test]
    fn local_chat_rejects_oversized_text_without_runtime_admission() {
        let snapshot = service().run(LocalChatRequest {
            message: "x".repeat(CHAT_TEXT_BYTE_LIMIT + 1),
        });
        assert_eq!(snapshot.state, "failed");
        assert_eq!(snapshot.diagnostic.as_deref(), Some("input-too-large"));
    }

    #[test]
    fn local_chat_cancellation_is_scoped_to_its_own_absent_turn() {
        assert!(!service().cancel());
    }
}
