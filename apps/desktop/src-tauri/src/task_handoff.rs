//! Transient, user-approved continuity between the two QuireForge workspaces.
//!
//! This is deliberately not a conversation store.  It holds one reviewed
//! envelope in memory only, has no project or runtime references, and consumes
//! the envelope when the receiving workspace explicitly accepts it.

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use uuid::Uuid;

const MAX_TITLE_BYTES: usize = 120;
const MAX_REQUEST_BYTES: usize = 8 * 1024;
const MAX_BRIEF_BYTES: usize = 12 * 1024;
const MAX_RECEIPT_BYTES: usize = 4 * 1024;
const LIFETIME: Duration = Duration::from_secs(30 * 60);

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskHandoffCreateRequest {
    pub title: String,
    pub original_request: String,
    pub brief: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskHandoffReceiptRequest {
    pub task_id: String,
    pub title: String,
    pub original_request: String,
    pub summary: String,
    pub status: TaskHandoffReceiptStatus,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum TaskHandoffReceiptStatus {
    Completed,
    Blocked,
    Cancelled,
    Failed,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum TaskHandoffDirection {
    AdvisorToQuireforge,
    QuireforgeToAdvisor,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskHandoffSnapshot {
    pub schema_version: u16,
    pub state: TaskHandoffState,
    pub task_id: Option<String>,
    pub direction: Option<TaskHandoffDirection>,
    pub title: Option<String>,
    pub original_request: Option<String>,
    pub brief: Option<String>,
    pub receipt_status: Option<TaskHandoffReceiptStatus>,
    pub expires_at_ms: Option<u64>,
    pub diagnostic_code: Option<TaskHandoffDiagnosticCode>,
}

#[derive(Clone, Copy, Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum TaskHandoffState {
    Empty,
    Pending,
    Accepted,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum TaskHandoffDiagnosticCode {
    InvalidRequest,
    NotFound,
    Expired,
    DirectionMismatch,
}

struct Envelope {
    task_id: String,
    direction: TaskHandoffDirection,
    title: String,
    original_request: String,
    brief: String,
    receipt_status: Option<TaskHandoffReceiptStatus>,
    expires_at_ms: u64,
}

pub struct TaskHandoffService {
    envelope: Mutex<Option<Envelope>>,
}

impl Default for TaskHandoffService {
    fn default() -> Self {
        Self {
            envelope: Mutex::new(None),
        }
    }
}

impl TaskHandoffService {
    pub async fn status(&self) -> TaskHandoffSnapshot {
        let mut state = self.envelope.lock().await;
        if state.as_ref().is_some_and(expired) {
            *state = None;
        }
        state.as_ref().map(snapshot).unwrap_or_else(empty)
    }

    pub async fn prepare_advisor_brief(
        &self,
        request: TaskHandoffCreateRequest,
    ) -> TaskHandoffSnapshot {
        if !valid(&request.title, MAX_TITLE_BYTES)
            || !valid(&request.original_request, MAX_REQUEST_BYTES)
            || !valid(&request.brief, MAX_BRIEF_BYTES)
        {
            return unavailable(TaskHandoffDiagnosticCode::InvalidRequest);
        }
        self.replace(
            TaskHandoffDirection::AdvisorToQuireforge,
            request.title,
            request.original_request,
            request.brief,
            None,
        )
        .await
    }

    pub async fn prepare_completion_receipt(
        &self,
        request: TaskHandoffReceiptRequest,
    ) -> TaskHandoffSnapshot {
        if validate_uuid_v7(&request.task_id).is_err()
            || !valid(&request.title, MAX_TITLE_BYTES)
            || !valid(&request.original_request, MAX_REQUEST_BYTES)
            || !valid(&request.summary, MAX_RECEIPT_BYTES)
        {
            return unavailable(TaskHandoffDiagnosticCode::InvalidRequest);
        }
        self.replace(
            TaskHandoffDirection::QuireforgeToAdvisor,
            request.title,
            request.original_request,
            request.summary,
            Some(request.status),
        )
        .await
    }

    pub async fn accept(&self, direction: TaskHandoffDirection) -> TaskHandoffSnapshot {
        let mut state = self.envelope.lock().await;
        let Some(envelope) = state.take() else {
            return unavailable(TaskHandoffDiagnosticCode::NotFound);
        };
        if expired(&envelope) {
            return unavailable(TaskHandoffDiagnosticCode::Expired);
        }
        if envelope.direction != direction {
            *state = Some(envelope);
            return unavailable(TaskHandoffDiagnosticCode::DirectionMismatch);
        }
        let mut value = snapshot(&envelope);
        value.state = TaskHandoffState::Accepted;
        value
    }

    pub async fn cancel(&self) -> TaskHandoffSnapshot {
        *self.envelope.lock().await = None;
        empty()
    }

    async fn replace(
        &self,
        direction: TaskHandoffDirection,
        title: String,
        original_request: String,
        brief: String,
        receipt_status: Option<TaskHandoffReceiptStatus>,
    ) -> TaskHandoffSnapshot {
        let now = now_ms();
        let envelope = Envelope {
            task_id: Uuid::now_v7().to_string(),
            direction,
            title,
            original_request,
            brief,
            receipt_status,
            expires_at_ms: now.saturating_add(LIFETIME.as_millis() as u64),
        };
        let result = snapshot(&envelope);
        *self.envelope.lock().await = Some(envelope);
        result
    }
}

fn valid(value: &str, maximum: usize) -> bool {
    !value.trim().is_empty() && value.len() <= maximum && !value.contains('\0')
}
fn validate_uuid_v7(value: &str) -> Result<(), ()> {
    Uuid::parse_str(value)
        .ok()
        .filter(|uuid| uuid.get_version_num() == 7)
        .map(|_| ())
        .ok_or(())
}
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}
fn expired(value: &Envelope) -> bool {
    now_ms() >= value.expires_at_ms
}
fn empty() -> TaskHandoffSnapshot {
    TaskHandoffSnapshot {
        schema_version: 1,
        state: TaskHandoffState::Empty,
        task_id: None,
        direction: None,
        title: None,
        original_request: None,
        brief: None,
        receipt_status: None,
        expires_at_ms: None,
        diagnostic_code: None,
    }
}
fn unavailable(code: TaskHandoffDiagnosticCode) -> TaskHandoffSnapshot {
    TaskHandoffSnapshot {
        state: TaskHandoffState::Unavailable,
        diagnostic_code: Some(code),
        ..empty()
    }
}
fn snapshot(value: &Envelope) -> TaskHandoffSnapshot {
    TaskHandoffSnapshot {
        schema_version: 1,
        state: TaskHandoffState::Pending,
        task_id: Some(value.task_id.clone()),
        direction: Some(value.direction),
        title: Some(value.title.clone()),
        original_request: Some(value.original_request.clone()),
        brief: Some(value.brief.clone()),
        receipt_status: value.receipt_status,
        expires_at_ms: Some(value.expires_at_ms),
        diagnostic_code: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn acceptance_is_one_use_and_direction_bound() {
        let service = TaskHandoffService::default();
        let prepared = service
            .prepare_advisor_brief(TaskHandoffCreateRequest {
                title: "Task".into(),
                original_request: "Request".into(),
                brief: "Brief".into(),
            })
            .await;
        assert_eq!(prepared.state, TaskHandoffState::Pending);
        assert_eq!(
            service
                .accept(TaskHandoffDirection::QuireforgeToAdvisor)
                .await
                .diagnostic_code,
            Some(TaskHandoffDiagnosticCode::DirectionMismatch)
        );
        assert_eq!(
            service
                .accept(TaskHandoffDirection::AdvisorToQuireforge)
                .await
                .state,
            TaskHandoffState::Accepted
        );
        assert_eq!(service.status().await.state, TaskHandoffState::Empty);
    }

    #[tokio::test]
    async fn invalid_and_cancelled_envelopes_never_remain_pending() {
        let service = TaskHandoffService::default();
        let invalid = service
            .prepare_advisor_brief(TaskHandoffCreateRequest {
                title: "Task".into(),
                original_request: "Request".into(),
                brief: "\0".into(),
            })
            .await;
        assert_eq!(
            invalid.diagnostic_code,
            Some(TaskHandoffDiagnosticCode::InvalidRequest)
        );
        assert_eq!(service.status().await.state, TaskHandoffState::Empty);
        service
            .prepare_advisor_brief(TaskHandoffCreateRequest {
                title: "Task".into(),
                original_request: "Request".into(),
                brief: "Brief".into(),
            })
            .await;
        assert_eq!(service.cancel().await.state, TaskHandoffState::Empty);
        assert_eq!(
            service
                .accept(TaskHandoffDirection::AdvisorToQuireforge)
                .await
                .diagnostic_code,
            Some(TaskHandoffDiagnosticCode::NotFound)
        );
    }

    #[tokio::test]
    async fn receipt_is_bounded_and_consumed_once() {
        let service = TaskHandoffService::default();
        let task_id = Uuid::now_v7().to_string();
        let prepared = service
            .prepare_completion_receipt(TaskHandoffReceiptRequest {
                task_id,
                title: "Task".into(),
                original_request: "Request".into(),
                summary: "Completed safely.".into(),
                status: TaskHandoffReceiptStatus::Completed,
            })
            .await;
        assert_eq!(
            prepared.direction,
            Some(TaskHandoffDirection::QuireforgeToAdvisor)
        );
        assert_eq!(
            prepared.receipt_status,
            Some(TaskHandoffReceiptStatus::Completed)
        );
        let accepted = service
            .accept(TaskHandoffDirection::QuireforgeToAdvisor)
            .await;
        assert_eq!(accepted.state, TaskHandoffState::Accepted);
        assert_eq!(accepted.brief.as_deref(), Some("Completed safely."));
        assert_eq!(service.status().await.state, TaskHandoffState::Empty);
    }
}
