//! Local, fictional mock inference vertical slice.
//!
//! This is an in-memory authority exercise only. It deliberately contains no
//! transport, credential material, filesystem access, persistence, or native
//! operation path. Every record is bound to the project and durable task that
//! native project metadata has already accepted.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

const SCHEMA_VERSION: u16 = 1;
const POLICY_VERSION: u16 = 1;
const INPUT_LIMIT: usize = 2_000;
const EXPIRY_TICKS: u64 = 24;

const LANTERN_PROVIDER_ID: &str = "019a5800-0000-7000-8000-000000000001";
const LANTERN_ENDPOINT_ID: &str = "019a5800-0000-7000-8000-000000000002";
const LANTERN_MODEL_ID: &str = "019a5800-0000-7000-8000-000000000003";
const LANTERN_ADAPTER_ID: &str = "019a5800-0000-7000-8000-000000000004";
const EMBER_PROVIDER_ID: &str = "019a5800-0000-7000-8000-000000000011";
const EMBER_ENDPOINT_ID: &str = "019a5800-0000-7000-8000-000000000012";
const EMBER_MODEL_ID: &str = "019a5800-0000-7000-8000-000000000013";
const EMBER_ADAPTER_ID: &str = "019a5800-0000-7000-8000-000000000014";

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct MockInferencePrepareRequest {
    pub task_id: String,
    pub profile_id: String,
    pub input: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct MockInferenceAttemptRequest {
    pub task_id: String,
    pub attempt_id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct MockInferenceAuthorizationRequest {
    pub task_id: String,
    pub attempt_id: String,
    pub authorization_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TaskBinding {
    pub project_id: String,
    pub task_id: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MockInferenceCatalog {
    pub schema_version: u16,
    pub profiles: Vec<MockInferenceProfile>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MockInferenceProfile {
    pub id: String,
    pub provider_label: String,
    pub endpoint_label: String,
    pub model_label: String,
    pub adapter_label: String,
    pub scenario: MockScenario,
    pub descriptor_sha256: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum MockScenario {
    StreamedText,
    Structured,
    Refusal,
    Failure,
    Timeout,
    Interrupted,
    Ambiguous,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)] // Public bridge taxonomy deliberately includes non-fixture lifecycle states.
pub(crate) enum MockAttemptState {
    Draft,
    Ready,
    Authorized,
    Submitted,
    Streaming,
    Cancelling,
    Cancelled,
    Completed,
    Refused,
    Failed,
    Interrupted,
    Ambiguous,
    Invalidated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)] // The contract is closed even though fixtures issue only inert leases.
pub(crate) enum MockLeaseState {
    Issued,
    Expired,
    Revoked,
    Quarantined,
    Invalidated,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MockInferenceSnapshot {
    pub schema_version: u16,
    pub mock_only: bool,
    pub attempt_id: Option<String>,
    pub state: MockAttemptState,
    pub diagnostic: Option<MockDiagnostic>,
    pub destination: Option<MockDestination>,
    pub manifest: Option<MockManifestSummary>,
    pub lease: Option<MockLeaseSummary>,
    pub authorization: Option<MockAuthorizationSummary>,
    pub events: Vec<MockInteractionEvent>,
    pub usage: Option<MockUsage>,
    pub evidence: Vec<MockEvidence>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)] // Closed diagnostic vocabulary for future rejected fixture states.
pub(crate) enum MockDiagnostic {
    InvalidRequest,
    TaskUnavailable,
    AttemptUnavailable,
    AuthorizationRequired,
    AuthorizationReplayed,
    AuthorizationInvalid,
    LeaseUnavailable,
    ManifestInvalidated,
    TerminalAttempt,
    CrossTaskRejected,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MockDestination {
    pub provider_id: String,
    pub endpoint_id: String,
    pub model_id: String,
    pub adapter_id: String,
    pub descriptor_sha256: String,
    pub capability_profile_sha256: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MockManifestSummary {
    pub id: String,
    pub sha256: String,
    pub input_sha256: String,
    pub item_count: u8,
    pub input_char_count: u16,
    pub exclusions: Vec<String>,
    pub retention: String,
    pub expires_at_tick: u64,
    pub state: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MockLeaseSummary {
    pub credential_reference_id: String,
    pub lease_id: String,
    pub account_reference: String,
    pub scopes: Vec<String>,
    pub state: MockLeaseState,
    pub expires_at_tick: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MockAuthorizationSummary {
    pub id: String,
    pub binding_sha256: String,
    pub state: String,
    pub expires_at_tick: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MockInteractionEvent {
    pub id: String,
    pub sequence: u16,
    pub kind: String,
    pub text: Option<String>,
    pub structured_state: Option<String>,
    pub sha256: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MockUsage {
    pub basis: String,
    pub units: Vec<MockUsageUnit>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MockUsageUnit {
    pub unit: String,
    pub quantity: u16,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MockEvidence {
    pub kind: String,
    pub sha256: String,
    pub detail: String,
}

#[derive(Default)]
pub(crate) struct MockInferenceService {
    store: Mutex<MockInferenceStore>,
}

#[derive(Default)]
struct MockInferenceStore {
    tick: u64,
    attempts: HashMap<String, AttemptRecord>,
}

struct AttemptRecord {
    id: String,
    binding: TaskBinding,
    profile: ProfileDefinition,
    input_sha256: String,
    input_char_count: u16,
    manifest_id: String,
    manifest_sha256: String,
    lease_id: String,
    credential_reference_id: String,
    authorization_id: String,
    authorization_sha256: String,
    created_tick: u64,
    state: MockAttemptState,
    lease_state: MockLeaseState,
    authorized: bool,
    consumed: bool,
    events: Vec<MockInteractionEvent>,
}

#[derive(Clone)]
struct ProfileDefinition {
    id: &'static str,
    provider_id: &'static str,
    endpoint_id: &'static str,
    model_id: &'static str,
    adapter_id: &'static str,
    provider_label: &'static str,
    endpoint_label: &'static str,
    model_label: &'static str,
    adapter_label: &'static str,
    scenario: MockScenario,
}

const PROFILES: [ProfileDefinition; 7] = [
    ProfileDefinition {
        id: "lantern-stream",
        provider_id: LANTERN_PROVIDER_ID,
        endpoint_id: LANTERN_ENDPOINT_ID,
        model_id: LANTERN_MODEL_ID,
        adapter_id: LANTERN_ADAPTER_ID,
        provider_label: "Fictional Lantern",
        endpoint_label: "Local fixture endpoint",
        model_label: "Lantern Text Fixture",
        adapter_label: "Lantern fixture adapter",
        scenario: MockScenario::StreamedText,
    },
    ProfileDefinition {
        id: "lantern-structured",
        provider_id: LANTERN_PROVIDER_ID,
        endpoint_id: LANTERN_ENDPOINT_ID,
        model_id: LANTERN_MODEL_ID,
        adapter_id: LANTERN_ADAPTER_ID,
        provider_label: "Fictional Lantern",
        endpoint_label: "Local fixture endpoint",
        model_label: "Lantern Structured Fixture",
        adapter_label: "Lantern fixture adapter",
        scenario: MockScenario::Structured,
    },
    ProfileDefinition {
        id: "ember-refusal",
        provider_id: EMBER_PROVIDER_ID,
        endpoint_id: EMBER_ENDPOINT_ID,
        model_id: EMBER_MODEL_ID,
        adapter_id: EMBER_ADAPTER_ID,
        provider_label: "Fictional Ember",
        endpoint_label: "Local fixture endpoint",
        model_label: "Ember Refusal Fixture",
        adapter_label: "Ember fixture adapter",
        scenario: MockScenario::Refusal,
    },
    ProfileDefinition {
        id: "ember-failure",
        provider_id: EMBER_PROVIDER_ID,
        endpoint_id: EMBER_ENDPOINT_ID,
        model_id: EMBER_MODEL_ID,
        adapter_id: EMBER_ADAPTER_ID,
        provider_label: "Fictional Ember",
        endpoint_label: "Local fixture endpoint",
        model_label: "Ember Failure Fixture",
        adapter_label: "Ember fixture adapter",
        scenario: MockScenario::Failure,
    },
    ProfileDefinition {
        id: "ember-timeout",
        provider_id: EMBER_PROVIDER_ID,
        endpoint_id: EMBER_ENDPOINT_ID,
        model_id: EMBER_MODEL_ID,
        adapter_id: EMBER_ADAPTER_ID,
        provider_label: "Fictional Ember",
        endpoint_label: "Local fixture endpoint",
        model_label: "Ember Timeout Fixture",
        adapter_label: "Ember fixture adapter",
        scenario: MockScenario::Timeout,
    },
    ProfileDefinition {
        id: "ember-interrupted",
        provider_id: EMBER_PROVIDER_ID,
        endpoint_id: EMBER_ENDPOINT_ID,
        model_id: EMBER_MODEL_ID,
        adapter_id: EMBER_ADAPTER_ID,
        provider_label: "Fictional Ember",
        endpoint_label: "Local fixture endpoint",
        model_label: "Ember Interrupted Fixture",
        adapter_label: "Ember fixture adapter",
        scenario: MockScenario::Interrupted,
    },
    ProfileDefinition {
        id: "ember-ambiguous",
        provider_id: EMBER_PROVIDER_ID,
        endpoint_id: EMBER_ENDPOINT_ID,
        model_id: EMBER_MODEL_ID,
        adapter_id: EMBER_ADAPTER_ID,
        provider_label: "Fictional Ember",
        endpoint_label: "Local fixture endpoint",
        model_label: "Ember Ambiguous Fixture",
        adapter_label: "Ember fixture adapter",
        scenario: MockScenario::Ambiguous,
    },
];

impl MockInferenceService {
    pub(crate) fn catalog(&self) -> MockInferenceCatalog {
        MockInferenceCatalog {
            schema_version: SCHEMA_VERSION,
            profiles: PROFILES.iter().map(profile_snapshot).collect(),
        }
    }

    pub(crate) fn prepare(
        &self,
        request: MockInferencePrepareRequest,
        binding: TaskBinding,
    ) -> MockInferenceSnapshot {
        let input = request.input.trim();
        let Some(profile) = PROFILES
            .iter()
            .find(|profile| profile.id == request.profile_id)
            .cloned()
        else {
            return diagnostic(
                MockAttemptState::Invalidated,
                MockDiagnostic::InvalidRequest,
            );
        };
        if request.task_id != binding.task_id
            || !valid_uuidv7(&binding.project_id)
            || !valid_uuidv7(&binding.task_id)
            || input.is_empty()
            || input.len() > INPUT_LIMIT
            || input.chars().count() > u16::MAX as usize
        {
            return diagnostic(
                MockAttemptState::Invalidated,
                MockDiagnostic::InvalidRequest,
            );
        }
        let mut store = match self.store.lock() {
            Ok(store) => store,
            Err(_) => {
                return diagnostic(
                    MockAttemptState::Invalidated,
                    MockDiagnostic::AttemptUnavailable,
                )
            }
        };
        store.tick += 1;
        let attempt_id = Uuid::now_v7().to_string();
        let manifest_id = Uuid::now_v7().to_string();
        let lease_id = Uuid::now_v7().to_string();
        let credential_reference_id = Uuid::now_v7().to_string();
        let authorization_id = Uuid::now_v7().to_string();
        let input_sha256 = digest(input);
        let destination = destination_digest(&profile);
        let manifest_sha256 = digest(&format!(
            "{}:{}:{}:{}",
            binding.project_id, binding.task_id, input_sha256, destination
        ));
        let authorization_sha256 = digest(&format!(
            "{}:{}:{}:{}:{}",
            attempt_id, manifest_sha256, lease_id, destination, POLICY_VERSION
        ));
        let record = AttemptRecord {
            id: attempt_id.clone(),
            binding,
            profile,
            input_sha256,
            input_char_count: input.chars().count() as u16,
            manifest_id,
            manifest_sha256,
            lease_id,
            credential_reference_id,
            authorization_id,
            authorization_sha256,
            created_tick: store.tick,
            state: MockAttemptState::Ready,
            lease_state: MockLeaseState::Issued,
            authorized: false,
            consumed: false,
            events: Vec::new(),
        };
        let snapshot = snapshot(&record, store.tick, None);
        store.attempts.insert(attempt_id.clone(), record);
        snapshot
    }

    pub(crate) fn authorize(
        &self,
        request: MockInferenceAuthorizationRequest,
        binding: &TaskBinding,
    ) -> MockInferenceSnapshot {
        let mut store = match self.store.lock() {
            Ok(store) => store,
            Err(_) => {
                return diagnostic(
                    MockAttemptState::Invalidated,
                    MockDiagnostic::AttemptUnavailable,
                )
            }
        };
        store.tick += 1;
        let tick = store.tick;
        let Some(record) = store.attempts.get_mut(&request.attempt_id) else {
            return diagnostic(
                MockAttemptState::Invalidated,
                MockDiagnostic::AttemptUnavailable,
            );
        };
        if &record.binding != binding {
            return diagnostic(
                MockAttemptState::Invalidated,
                MockDiagnostic::CrossTaskRejected,
            );
        }
        if record.authorization_id != request.authorization_id || record.consumed {
            return diagnostic(record.state, MockDiagnostic::AuthorizationReplayed);
        }
        if record.state != MockAttemptState::Ready {
            return diagnostic(record.state, MockDiagnostic::AuthorizationInvalid);
        }
        if record.lease_state != MockLeaseState::Issued {
            record.state = MockAttemptState::Invalidated;
            return snapshot(record, tick, Some(MockDiagnostic::LeaseUnavailable));
        }
        if !valid_for_submit(record, tick) {
            record.state = MockAttemptState::Invalidated;
            return snapshot(record, tick, Some(MockDiagnostic::ManifestInvalidated));
        }
        record.authorized = true;
        record.state = MockAttemptState::Authorized;
        snapshot(record, tick, None)
    }

    pub(crate) fn submit(
        &self,
        request: MockInferenceAuthorizationRequest,
        binding: &TaskBinding,
    ) -> MockInferenceSnapshot {
        let mut store = match self.store.lock() {
            Ok(store) => store,
            Err(_) => {
                return diagnostic(
                    MockAttemptState::Invalidated,
                    MockDiagnostic::AttemptUnavailable,
                )
            }
        };
        store.tick += 1;
        let tick = store.tick;
        let Some(record) = store.attempts.get_mut(&request.attempt_id) else {
            return diagnostic(
                MockAttemptState::Invalidated,
                MockDiagnostic::AttemptUnavailable,
            );
        };
        if &record.binding != binding {
            return diagnostic(
                MockAttemptState::Invalidated,
                MockDiagnostic::CrossTaskRejected,
            );
        }
        if record.authorization_id != request.authorization_id || record.consumed {
            return diagnostic(record.state, MockDiagnostic::AuthorizationReplayed);
        }
        if !record.authorized {
            return diagnostic(record.state, MockDiagnostic::AuthorizationRequired);
        }
        if record.lease_state != MockLeaseState::Issued {
            record.state = MockAttemptState::Invalidated;
            return snapshot(record, tick, Some(MockDiagnostic::LeaseUnavailable));
        }
        if !valid_for_submit(record, tick) {
            record.state = MockAttemptState::Invalidated;
            return snapshot(record, tick, Some(MockDiagnostic::ManifestInvalidated));
        }
        record.consumed = true;
        record.state = MockAttemptState::Submitted;
        record.events = fixture_events(record);
        record.state = terminal_for(record.profile.scenario);
        snapshot(record, tick, None)
    }

    pub(crate) fn cancel(
        &self,
        request: MockInferenceAttemptRequest,
        binding: &TaskBinding,
    ) -> MockInferenceSnapshot {
        let mut store = match self.store.lock() {
            Ok(store) => store,
            Err(_) => {
                return diagnostic(
                    MockAttemptState::Invalidated,
                    MockDiagnostic::AttemptUnavailable,
                )
            }
        };
        store.tick += 1;
        let tick = store.tick;
        let Some(record) = store.attempts.get_mut(&request.attempt_id) else {
            return diagnostic(
                MockAttemptState::Invalidated,
                MockDiagnostic::AttemptUnavailable,
            );
        };
        if &record.binding != binding {
            return diagnostic(
                MockAttemptState::Invalidated,
                MockDiagnostic::CrossTaskRejected,
            );
        }
        if is_terminal(record.state) {
            return diagnostic(record.state, MockDiagnostic::TerminalAttempt);
        }
        record.state = MockAttemptState::Cancelling;
        record.events.push(event(
            record.events.len() as u16 + 1,
            "cancellation-requested",
            None,
            None,
        ));
        record.events.push(event(
            record.events.len() as u16 + 1,
            "terminal",
            Some("cancelled"),
            None,
        ));
        record.state = MockAttemptState::Cancelled;
        snapshot(record, tick, None)
    }
}

fn snapshot(
    record: &AttemptRecord,
    tick: u64,
    diagnostic_value: Option<MockDiagnostic>,
) -> MockInferenceSnapshot {
    let destination = MockDestination {
        provider_id: record.profile.provider_id.into(),
        endpoint_id: record.profile.endpoint_id.into(),
        model_id: record.profile.model_id.into(),
        adapter_id: record.profile.adapter_id.into(),
        descriptor_sha256: destination_digest(&record.profile),
        capability_profile_sha256: digest("fictional-text-only-v1"),
    };
    MockInferenceSnapshot {
        schema_version: SCHEMA_VERSION,
        mock_only: true,
        attempt_id: Some(record.id.clone()),
        state: record.state,
        diagnostic: diagnostic_value,
        destination: Some(destination),
        manifest: Some(MockManifestSummary {
            id: record.manifest_id.clone(),
            sha256: record.manifest_sha256.clone(),
            input_sha256: record.input_sha256.clone(),
            item_count: 1,
            input_char_count: record.input_char_count,
            exclusions: vec![
                "ambient-context".into(),
                "files-and-repositories".into(),
                "credentials-and-sessions".into(),
                "retrieved-sources".into(),
            ],
            retention: "transient-local-mock".into(),
            expires_at_tick: record.created_tick + EXPIRY_TICKS,
            state: if record.state == MockAttemptState::Invalidated {
                "invalidated".into()
            } else {
                "ready".into()
            },
        }),
        lease: Some(MockLeaseSummary {
            credential_reference_id: record.credential_reference_id.clone(),
            lease_id: record.lease_id.clone(),
            account_reference: "fictional-account-reference".into(),
            scopes: vec!["mock-inference-submit".into()],
            state: record.lease_state,
            expires_at_tick: record.created_tick + EXPIRY_TICKS,
        }),
        authorization: Some(MockAuthorizationSummary {
            id: record.authorization_id.clone(),
            binding_sha256: record.authorization_sha256.clone(),
            state: if record.consumed {
                "consumed".into()
            } else if record.authorized {
                "authorized".into()
            } else {
                "pending".into()
            },
            expires_at_tick: record.created_tick + EXPIRY_TICKS,
        }),
        events: record.events.clone(),
        usage: usage_for(record),
        evidence: vec![
            MockEvidence {
                kind: "mock-attempt-binding".into(),
                sha256: record.authorization_sha256.clone(),
                detail: format!("local mock tick {}", tick),
            },
            MockEvidence {
                kind: "context-manifest".into(),
                sha256: record.manifest_sha256.clone(),
                detail: "one explicitly authored text item; exclusions recorded".into(),
            },
        ],
    }
}

pub(crate) fn diagnostic(state: MockAttemptState, code: MockDiagnostic) -> MockInferenceSnapshot {
    MockInferenceSnapshot {
        schema_version: SCHEMA_VERSION,
        mock_only: true,
        attempt_id: None,
        state,
        diagnostic: Some(code),
        destination: None,
        manifest: None,
        lease: None,
        authorization: None,
        events: Vec::new(),
        usage: None,
        evidence: Vec::new(),
    }
}

fn valid_for_submit(record: &AttemptRecord, tick: u64) -> bool {
    record.lease_state == MockLeaseState::Issued
        && tick <= record.created_tick + EXPIRY_TICKS
        && record.manifest_sha256
            == digest(&format!(
                "{}:{}:{}:{}",
                record.binding.project_id,
                record.binding.task_id,
                record.input_sha256,
                destination_digest(&record.profile)
            ))
}
fn terminal_for(scenario: MockScenario) -> MockAttemptState {
    match scenario {
        MockScenario::StreamedText | MockScenario::Structured => MockAttemptState::Completed,
        MockScenario::Refusal => MockAttemptState::Refused,
        MockScenario::Failure => MockAttemptState::Failed,
        MockScenario::Timeout => MockAttemptState::Interrupted,
        MockScenario::Interrupted => MockAttemptState::Interrupted,
        MockScenario::Ambiguous => MockAttemptState::Ambiguous,
    }
}
fn is_terminal(state: MockAttemptState) -> bool {
    matches!(
        state,
        MockAttemptState::Cancelled
            | MockAttemptState::Completed
            | MockAttemptState::Refused
            | MockAttemptState::Failed
            | MockAttemptState::Interrupted
            | MockAttemptState::Ambiguous
            | MockAttemptState::Invalidated
    )
}
fn usage_for(record: &AttemptRecord) -> Option<MockUsage> {
    matches!(
        record.state,
        MockAttemptState::Completed
            | MockAttemptState::Refused
            | MockAttemptState::Failed
            | MockAttemptState::Interrupted
            | MockAttemptState::Ambiguous
    )
    .then(|| MockUsage {
        basis: "fictional-reported".into(),
        units: vec![
            MockUsageUnit {
                unit: "fixture-input-units".into(),
                quantity: record.input_char_count,
            },
            MockUsageUnit {
                unit: "fixture-output-units".into(),
                quantity: record
                    .events
                    .iter()
                    .filter_map(|event| event.text.as_ref())
                    .map(|text| text.chars().count() as u16)
                    .sum(),
            },
        ],
    })
}
fn fixture_events(record: &AttemptRecord) -> Vec<MockInteractionEvent> {
    let mut events = vec![
        event(1, "input-authorized", None, None),
        event(2, "provider-session-reference", None, None),
    ];
    match record.profile.scenario {
        MockScenario::StreamedText => {
            events.push(event(
                3,
                "text-delta",
                Some("Fictional local mock response: "),
                None,
            ));
            events.push(event(
                4,
                "text-delta",
                Some("the task remains authoritative."),
                None,
            ));
            events.push(event(5, "terminal", Some("completed"), None));
        }
        MockScenario::Structured => {
            events.push(event(
                3,
                "structured-output",
                Some("{\"fixture\":\"complete\"}"),
                Some("complete-valid"),
            ));
            events.push(event(4, "terminal", Some("completed"), None));
        }
        MockScenario::Refusal => {
            events.push(event(
                3,
                "refusal",
                Some("Fictional fixture refusal."),
                None,
            ));
            events.push(event(4, "terminal", Some("refused"), None));
        }
        MockScenario::Failure => {
            events.push(event(3, "failure", Some("fictional-provider-error"), None));
            events.push(event(4, "terminal", Some("failed"), None));
        }
        MockScenario::Timeout => {
            events.push(event(
                3,
                "timeout",
                Some("submission outcome not established"),
                None,
            ));
            events.push(event(4, "terminal", Some("interrupted"), None));
        }
        MockScenario::Interrupted => {
            events.push(event(3, "text-delta", Some("partial fixture output"), None));
            events.push(event(
                4,
                "transport-closed",
                Some("stream interrupted"),
                None,
            ));
            events.push(event(5, "terminal", Some("interrupted"), None));
        }
        MockScenario::Ambiguous => {
            events.push(event(
                3,
                "ambiguous-outcome",
                Some("no automatic retry"),
                None,
            ));
            events.push(event(4, "terminal", Some("ambiguous"), None));
        }
    };
    events
}
fn event(
    sequence: u16,
    kind: &str,
    text: Option<&str>,
    structured_state: Option<&str>,
) -> MockInteractionEvent {
    let id = Uuid::now_v7().to_string();
    let rendered = format!(
        "{}:{}:{}:{}",
        sequence,
        kind,
        text.unwrap_or_default(),
        structured_state.unwrap_or_default()
    );
    MockInteractionEvent {
        id,
        sequence,
        kind: kind.into(),
        text: text.map(str::to_owned),
        structured_state: structured_state.map(str::to_owned),
        sha256: digest(&rendered),
    }
}
fn profile_snapshot(profile: &ProfileDefinition) -> MockInferenceProfile {
    MockInferenceProfile {
        id: profile.id.into(),
        provider_label: profile.provider_label.into(),
        endpoint_label: profile.endpoint_label.into(),
        model_label: profile.model_label.into(),
        adapter_label: profile.adapter_label.into(),
        scenario: profile.scenario,
        descriptor_sha256: destination_digest(profile),
    }
}
fn destination_digest(profile: &ProfileDefinition) -> String {
    digest(&format!(
        "{}:{}:{}:{}:{}",
        profile.id, profile.provider_id, profile.endpoint_id, profile.model_id, profile.adapter_id
    ))
}
fn digest(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}
fn valid_uuidv7(value: &str) -> bool {
    Uuid::parse_str(value).is_ok_and(|value| value.get_version_num() == 7)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding() -> TaskBinding {
        TaskBinding {
            project_id: "019a5800-0000-7000-8000-000000000101".into(),
            task_id: "019a5800-0000-7000-8000-000000000102".into(),
        }
    }
    fn prepared(service: &MockInferenceService) -> MockInferenceSnapshot {
        service.prepare(
            MockInferencePrepareRequest {
                task_id: binding().task_id,
                profile_id: "lantern-stream".into(),
                input: "bounded authored input".into(),
            },
            binding(),
        )
    }
    #[test]
    fn catalog_is_fictional_static_and_digest_bound() {
        let catalog = MockInferenceService::default().catalog();
        assert_eq!(catalog.profiles.len(), 7);
        assert!(catalog
            .profiles
            .iter()
            .all(|profile| profile.provider_label.starts_with("Fictional")));
        assert_eq!(catalog.profiles[0].descriptor_sha256.len(), 64);
    }
    #[test]
    fn successful_flow_requires_one_use_authorization_and_reconstructs_ordered_stream() {
        let service = MockInferenceService::default();
        let ready = prepared(&service);
        let attempt_id = ready.attempt_id.unwrap();
        let authorization_id = ready.authorization.unwrap().id;
        let authorized = service.authorize(
            MockInferenceAuthorizationRequest {
                task_id: binding().task_id,
                attempt_id: attempt_id.clone(),
                authorization_id: authorization_id.clone(),
            },
            &binding(),
        );
        assert_eq!(authorized.state, MockAttemptState::Authorized);
        let completed = service.submit(
            MockInferenceAuthorizationRequest {
                task_id: binding().task_id,
                attempt_id: attempt_id.clone(),
                authorization_id: authorization_id.clone(),
            },
            &binding(),
        );
        assert_eq!(completed.state, MockAttemptState::Completed);
        assert_eq!(
            completed
                .events
                .iter()
                .map(|event| event.sequence)
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5]
        );
        assert!(
            service
                .submit(
                    MockInferenceAuthorizationRequest {
                        task_id: binding().task_id,
                        attempt_id,
                        authorization_id
                    },
                    &binding()
                )
                .diagnostic
                == Some(MockDiagnostic::AuthorizationReplayed)
        );
    }
    #[test]
    fn cross_task_and_invalidated_manifest_fail_closed() {
        let service = MockInferenceService::default();
        let ready = prepared(&service);
        let request = MockInferenceAuthorizationRequest {
            task_id: binding().task_id,
            attempt_id: ready.attempt_id.unwrap(),
            authorization_id: ready.authorization.unwrap().id,
        };
        let other = TaskBinding {
            task_id: "019a5800-0000-7000-8000-000000000103".into(),
            ..binding()
        };
        assert_eq!(
            service.authorize(request, &other).diagnostic,
            Some(MockDiagnostic::CrossTaskRejected)
        );
    }
    #[test]
    fn cancellation_is_terminal_and_never_retries() {
        let service = MockInferenceService::default();
        let ready = prepared(&service);
        let attempt_id = ready.attempt_id.unwrap();
        let cancelled = service.cancel(
            MockInferenceAttemptRequest {
                task_id: binding().task_id,
                attempt_id: attempt_id.clone(),
            },
            &binding(),
        );
        assert_eq!(cancelled.state, MockAttemptState::Cancelled);
        assert_eq!(
            service
                .cancel(
                    MockInferenceAttemptRequest {
                        task_id: binding().task_id,
                        attempt_id,
                    },
                    &binding(),
                )
                .diagnostic,
            Some(MockDiagnostic::TerminalAttempt)
        );
    }
    #[test]
    fn fixture_failure_taxonomy_is_explicit() {
        let service = MockInferenceService::default();
        for (profile_id, state) in [
            ("ember-refusal", MockAttemptState::Refused),
            ("ember-failure", MockAttemptState::Failed),
            ("ember-timeout", MockAttemptState::Interrupted),
            ("ember-ambiguous", MockAttemptState::Ambiguous),
        ] {
            let snapshot = service.prepare(
                MockInferencePrepareRequest {
                    task_id: binding().task_id,
                    profile_id: profile_id.into(),
                    input: "fixture".into(),
                },
                binding(),
            );
            let attempt_id = snapshot.attempt_id.unwrap();
            let authorization_id = snapshot.authorization.unwrap().id;
            service.authorize(
                MockInferenceAuthorizationRequest {
                    task_id: binding().task_id,
                    attempt_id: attempt_id.clone(),
                    authorization_id: authorization_id.clone(),
                },
                &binding(),
            );
            assert_eq!(
                service
                    .submit(
                        MockInferenceAuthorizationRequest {
                            task_id: binding().task_id,
                            attempt_id,
                            authorization_id
                        },
                        &binding()
                    )
                    .state,
                state
            );
        }
    }

    #[test]
    fn expired_lease_and_descriptor_drift_invalidate_authorization() {
        let service = MockInferenceService::default();
        let ready = prepared(&service);
        let attempt_id = ready.attempt_id.unwrap();
        let authorization_id = ready.authorization.unwrap().id;
        {
            let mut store = service.store.lock().expect("fixture store");
            let record = store
                .attempts
                .get_mut(&attempt_id)
                .expect("fixture attempt");
            record.lease_state = MockLeaseState::Expired;
        }
        assert_eq!(
            service
                .authorize(
                    MockInferenceAuthorizationRequest {
                        task_id: binding().task_id,
                        attempt_id: attempt_id.clone(),
                        authorization_id: authorization_id.clone(),
                    },
                    &binding(),
                )
                .diagnostic,
            Some(MockDiagnostic::LeaseUnavailable)
        );

        let ready = prepared(&service);
        let attempt_id = ready.attempt_id.unwrap();
        let authorization_id = ready.authorization.unwrap().id;
        {
            let mut store = service.store.lock().expect("fixture store");
            let record = store
                .attempts
                .get_mut(&attempt_id)
                .expect("fixture attempt");
            record.profile.adapter_id = EMBER_ADAPTER_ID;
        }
        assert_eq!(
            service
                .authorize(
                    MockInferenceAuthorizationRequest {
                        task_id: binding().task_id,
                        attempt_id,
                        authorization_id,
                    },
                    &binding(),
                )
                .diagnostic,
            Some(MockDiagnostic::ManifestInvalidated)
        );
    }
}
