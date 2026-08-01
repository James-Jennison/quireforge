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
    pub capability_profile_sha256: String,
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
    ManifestExpired,
    LeaseExpired,
    LeaseRevoked,
    LeaseQuarantined,
    ExplicitInvalidation,
    DescriptorDrift,
    AdapterIncompatible,
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
    TimedOut,
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
    RecoveryRequired,
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
    destination: crate::provider_capability_registry::LocalMockDestinationProjection,
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
    destination_id: &'static str,
    scenario: MockScenario,
}

const PROFILES: [ProfileDefinition; 14] = [
    ProfileDefinition {
        id: "lantern-stream",
        destination_id: "lantern",
        scenario: MockScenario::StreamedText,
    },
    ProfileDefinition {
        id: "lantern-structured",
        destination_id: "lantern",
        scenario: MockScenario::Structured,
    },
    ProfileDefinition {
        id: "ember-refusal",
        destination_id: "ember",
        scenario: MockScenario::Refusal,
    },
    ProfileDefinition {
        id: "ember-failure",
        destination_id: "ember",
        scenario: MockScenario::Failure,
    },
    ProfileDefinition {
        id: "ember-timeout",
        destination_id: "ember",
        scenario: MockScenario::Timeout,
    },
    ProfileDefinition {
        id: "ember-interrupted",
        destination_id: "ember",
        scenario: MockScenario::Interrupted,
    },
    ProfileDefinition {
        id: "ember-ambiguous",
        destination_id: "ember",
        scenario: MockScenario::Ambiguous,
    },
    ProfileDefinition {
        id: "lantern-manifest-expired",
        destination_id: "lantern",
        scenario: MockScenario::ManifestExpired,
    },
    ProfileDefinition {
        id: "ember-lease-expired",
        destination_id: "ember",
        scenario: MockScenario::LeaseExpired,
    },
    ProfileDefinition {
        id: "ember-lease-revoked",
        destination_id: "ember",
        scenario: MockScenario::LeaseRevoked,
    },
    ProfileDefinition {
        id: "ember-lease-quarantined",
        destination_id: "ember",
        scenario: MockScenario::LeaseQuarantined,
    },
    ProfileDefinition {
        id: "ember-explicit-invalidation",
        destination_id: "ember",
        scenario: MockScenario::ExplicitInvalidation,
    },
    ProfileDefinition {
        id: "lantern-descriptor-drift",
        destination_id: "lantern",
        scenario: MockScenario::DescriptorDrift,
    },
    ProfileDefinition {
        id: "ember-adapter-incompatible",
        destination_id: "ember",
        scenario: MockScenario::AdapterIncompatible,
    },
];

impl MockInferenceService {
    pub(crate) fn catalog(&self) -> MockInferenceCatalog {
        MockInferenceCatalog {
            schema_version: SCHEMA_VERSION,
            profiles: PROFILES
                .iter()
                .map(|profile| {
                    let destination =
                        crate::provider_capability_registry::local_mock_destination_projection(
                            profile.destination_id,
                        )
                        .expect("built-in local mock registry fixture must remain valid");
                    profile_snapshot(profile, &destination)
                })
                .collect(),
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
        let destination =
            match crate::provider_capability_registry::local_mock_destination_projection(
                profile.destination_id,
            ) {
                Ok(destination) => destination,
                Err(_) => {
                    return diagnostic(
                        MockAttemptState::Invalidated,
                        MockDiagnostic::InvalidRequest,
                    )
                }
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
        let destination_digest = destination.descriptor_sha256.clone();
        let manifest_sha256 = digest(&format!(
            "{}:{}:{}:{}",
            binding.project_id, binding.task_id, input_sha256, destination_digest
        ));
        let authorization_sha256 = digest(&format!(
            "{}:{}:{}:{}:{}",
            attempt_id, manifest_sha256, lease_id, destination_digest, POLICY_VERSION
        ));
        let lease_state = lease_state_for(profile.scenario);
        let record = AttemptRecord {
            id: attempt_id.clone(),
            binding,
            profile,
            destination,
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
            lease_state,
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
        if let Some(code) = authority_diagnostic(record) {
            record.state = MockAttemptState::Invalidated;
            return snapshot(record, tick, Some(code));
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
        if let Some(code) = authority_diagnostic(record) {
            record.state = MockAttemptState::Invalidated;
            return snapshot(record, tick, Some(code));
        }
        if !valid_for_submit(record, tick) {
            record.state = MockAttemptState::Invalidated;
            return snapshot(record, tick, Some(MockDiagnostic::ManifestInvalidated));
        }
        record.consumed = true;
        record.state = MockAttemptState::Submitted;
        snapshot(record, tick, None)
    }

    pub(crate) fn poll(
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
        if record.state == MockAttemptState::Cancelling {
            record.events.push(event(
                record.events.len() as u16 + 1,
                "terminal",
                Some("cancelled"),
                None,
            ));
            record.state = MockAttemptState::Cancelled;
            return snapshot(record, tick, None);
        }
        if !matches!(
            record.state,
            MockAttemptState::Submitted | MockAttemptState::Streaming
        ) {
            return diagnostic(record.state, MockDiagnostic::AuthorizationInvalid);
        }
        let fixtures = fixture_events(record);
        let index = record.events.len();
        if let Some(next) = fixtures.get(index).cloned() {
            let terminal = next.kind == "terminal";
            record.events.push(next);
            record.state = if terminal {
                terminal_for(record.profile.scenario)
            } else {
                MockAttemptState::Streaming
            };
            return snapshot(record, tick, None);
        }
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
        if is_terminal(record.state) || record.state == MockAttemptState::Cancelling {
            return diagnostic(record.state, MockDiagnostic::TerminalAttempt);
        }
        record.state = MockAttemptState::Cancelling;
        record.events.push(event(
            record.events.len() as u16 + 1,
            "cancellation-requested",
            None,
            None,
        ));
        // Confirmation is deliberately emitted only by the next bounded poll.
        snapshot(record, tick, None)
    }
}

fn snapshot(
    record: &AttemptRecord,
    tick: u64,
    diagnostic_value: Option<MockDiagnostic>,
) -> MockInferenceSnapshot {
    let destination = MockDestination {
        provider_id: record.destination.provider_id.clone(),
        endpoint_id: record.destination.endpoint_id.clone(),
        model_id: record.destination.model_id.clone(),
        adapter_id: record.destination.adapter_id.clone(),
        descriptor_sha256: record.destination.descriptor_sha256.clone(),
        capability_profile_sha256: record.destination.capability_profile_sha256.clone(),
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
        && crate::provider_capability_registry::local_mock_destination_projection(
            &record.destination.id,
        )
        .is_ok_and(|current| current == record.destination)
        && record.manifest_sha256
            == digest(&format!(
                "{}:{}:{}:{}",
                record.binding.project_id,
                record.binding.task_id,
                record.input_sha256,
                record.destination.descriptor_sha256
            ))
}
fn lease_state_for(scenario: MockScenario) -> MockLeaseState {
    match scenario {
        MockScenario::LeaseExpired => MockLeaseState::Expired,
        MockScenario::LeaseRevoked => MockLeaseState::Revoked,
        MockScenario::LeaseQuarantined => MockLeaseState::Quarantined,
        _ => MockLeaseState::Issued,
    }
}
fn authority_diagnostic(record: &AttemptRecord) -> Option<MockDiagnostic> {
    match record.profile.scenario {
        MockScenario::ManifestExpired
        | MockScenario::DescriptorDrift
        | MockScenario::AdapterIncompatible => Some(MockDiagnostic::ManifestInvalidated),
        MockScenario::ExplicitInvalidation => Some(MockDiagnostic::RecoveryRequired),
        _ => None,
    }
}
fn terminal_for(scenario: MockScenario) -> MockAttemptState {
    match scenario {
        MockScenario::StreamedText | MockScenario::Structured => MockAttemptState::Completed,
        MockScenario::Refusal => MockAttemptState::Refused,
        MockScenario::Failure => MockAttemptState::Failed,
        MockScenario::Timeout => MockAttemptState::TimedOut,
        MockScenario::Interrupted => MockAttemptState::Interrupted,
        MockScenario::Ambiguous => MockAttemptState::Ambiguous,
        MockScenario::ManifestExpired
        | MockScenario::LeaseExpired
        | MockScenario::LeaseRevoked
        | MockScenario::LeaseQuarantined
        | MockScenario::ExplicitInvalidation
        | MockScenario::DescriptorDrift
        | MockScenario::AdapterIncompatible => MockAttemptState::Invalidated,
    }
}
fn is_terminal(state: MockAttemptState) -> bool {
    matches!(
        state,
        MockAttemptState::Cancelled
            | MockAttemptState::Completed
            | MockAttemptState::Refused
            | MockAttemptState::Failed
            | MockAttemptState::TimedOut
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
            events.push(event(4, "terminal", Some("timed-out"), None));
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
        MockScenario::ManifestExpired
        | MockScenario::LeaseExpired
        | MockScenario::LeaseRevoked
        | MockScenario::LeaseQuarantined
        | MockScenario::ExplicitInvalidation
        | MockScenario::DescriptorDrift
        | MockScenario::AdapterIncompatible => {
            events.push(event(
                3,
                "authority-invalidated",
                Some("fresh preparation required"),
                None,
            ));
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
fn profile_snapshot(
    profile: &ProfileDefinition,
    destination: &crate::provider_capability_registry::LocalMockDestinationProjection,
) -> MockInferenceProfile {
    MockInferenceProfile {
        id: profile.id.into(),
        provider_label: destination.provider_label.clone(),
        endpoint_label: destination.endpoint_label.clone(),
        model_label: destination.model_label.clone(),
        adapter_label: destination.adapter_label.clone(),
        scenario: profile.scenario,
        descriptor_sha256: destination.descriptor_sha256.clone(),
        capability_profile_sha256: destination.capability_profile_sha256.clone(),
    }
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
        assert_eq!(catalog.profiles.len(), 14);
        assert!(catalog
            .profiles
            .iter()
            .all(|profile| profile.provider_label.starts_with("Fictional")));
        assert!(catalog
            .profiles
            .iter()
            .any(|profile| profile.provider_label.contains("Lantern")));
        assert!(catalog
            .profiles
            .iter()
            .any(|profile| profile.provider_label.contains("Ember")));
        assert_eq!(catalog.profiles[0].descriptor_sha256.len(), 64);
    }
    #[test]
    fn successful_flow_requires_one_use_authorization_and_exposes_one_ordered_event_per_poll() {
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
        let submitted = service.submit(
            MockInferenceAuthorizationRequest {
                task_id: binding().task_id,
                attempt_id: attempt_id.clone(),
                authorization_id: authorization_id.clone(),
            },
            &binding(),
        );
        assert_eq!(submitted.state, MockAttemptState::Submitted);
        let mut completed = submitted;
        for expected_events in 1..=5 {
            completed = service.poll(
                MockInferenceAttemptRequest {
                    task_id: binding().task_id,
                    attempt_id: attempt_id.clone(),
                },
                &binding(),
            );
            assert_eq!(completed.events.len(), expected_events);
            assert_eq!(
                completed.events.last().map(|event| event.sequence),
                Some(expected_events as u16)
            );
        }
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
    fn cancellation_remains_available_after_submission_and_is_terminal() {
        let service = MockInferenceService::default();
        let ready = prepared(&service);
        let attempt_id = ready.attempt_id.unwrap();
        let authorization_id = ready.authorization.unwrap().id;
        service.authorize(
            MockInferenceAuthorizationRequest {
                task_id: binding().task_id,
                attempt_id: attempt_id.clone(),
                authorization_id: authorization_id.clone(),
            },
            &binding(),
        );
        let submitted = service.submit(
            MockInferenceAuthorizationRequest {
                task_id: binding().task_id,
                attempt_id: attempt_id.clone(),
                authorization_id,
            },
            &binding(),
        );
        assert_eq!(submitted.state, MockAttemptState::Submitted);
        let cancelled = service.cancel(
            MockInferenceAttemptRequest {
                task_id: binding().task_id,
                attempt_id: attempt_id.clone(),
            },
            &binding(),
        );
        assert_eq!(cancelled.state, MockAttemptState::Cancelling);
        assert_eq!(
            cancelled
                .events
                .iter()
                .map(|event| event.kind.as_str())
                .collect::<Vec<_>>(),
            vec!["cancellation-requested"]
        );
        let confirmed = service.poll(
            MockInferenceAttemptRequest {
                task_id: binding().task_id,
                attempt_id: attempt_id.clone(),
            },
            &binding(),
        );
        assert_eq!(confirmed.state, MockAttemptState::Cancelled);
        assert_eq!(
            confirmed.events.last().map(|event| event.text.as_deref()),
            Some(Some("cancelled"))
        );
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
            ("ember-timeout", MockAttemptState::TimedOut),
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
            service.submit(
                MockInferenceAuthorizationRequest {
                    task_id: binding().task_id,
                    attempt_id: attempt_id.clone(),
                    authorization_id,
                },
                &binding(),
            );
            let mut final_snapshot = None;
            for _ in 0..6 {
                let next = service.poll(
                    MockInferenceAttemptRequest {
                        task_id: binding().task_id,
                        attempt_id: attempt_id.clone(),
                    },
                    &binding(),
                );
                if is_terminal(next.state) {
                    final_snapshot = Some(next);
                    break;
                }
            }
            assert_eq!(final_snapshot.expect("terminal fixture").state, state);
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
            record.destination.adapter_id = "019a5800-0000-7000-8000-000000000014".into();
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
    #[test]
    fn polling_before_submit_is_rejected_and_retry_is_a_fresh_attempt() {
        let service = MockInferenceService::default();
        let ready = prepared(&service);
        let attempt_id = ready.attempt_id.unwrap();
        assert_eq!(
            service
                .poll(
                    MockInferenceAttemptRequest {
                        task_id: binding().task_id,
                        attempt_id,
                    },
                    &binding(),
                )
                .diagnostic,
            Some(MockDiagnostic::AuthorizationInvalid)
        );

        let first = prepared(&service);
        let retry = prepared(&service);
        assert_ne!(first.attempt_id, retry.attempt_id);
        assert_ne!(
            first.authorization.unwrap().id,
            retry.authorization.unwrap().id
        );
    }

    #[test]
    fn authority_fixture_paths_fail_closed_with_distinct_explanations() {
        for (profile_id, diagnostic) in [
            (
                "lantern-manifest-expired",
                MockDiagnostic::ManifestInvalidated,
            ),
            ("ember-lease-expired", MockDiagnostic::LeaseUnavailable),
            ("ember-lease-revoked", MockDiagnostic::LeaseUnavailable),
            ("ember-lease-quarantined", MockDiagnostic::LeaseUnavailable),
            (
                "ember-explicit-invalidation",
                MockDiagnostic::RecoveryRequired,
            ),
            (
                "lantern-descriptor-drift",
                MockDiagnostic::ManifestInvalidated,
            ),
            (
                "ember-adapter-incompatible",
                MockDiagnostic::ManifestInvalidated,
            ),
        ] {
            let service = MockInferenceService::default();
            let ready = service.prepare(
                MockInferencePrepareRequest {
                    task_id: binding().task_id,
                    profile_id: profile_id.into(),
                    input: "visible".into(),
                },
                binding(),
            );
            let rejected = service.authorize(
                MockInferenceAuthorizationRequest {
                    task_id: binding().task_id,
                    attempt_id: ready.attempt_id.unwrap(),
                    authorization_id: ready.authorization.unwrap().id,
                },
                &binding(),
            );
            assert_eq!(
                rejected.state,
                MockAttemptState::Invalidated,
                "{profile_id}"
            );
            assert_eq!(rejected.diagnostic, Some(diagnostic), "{profile_id}");
        }
    }

    #[test]
    fn ephemeral_state_loss_requires_a_fresh_attempt() {
        let first = MockInferenceService::default();
        let ready = prepared(&first);
        let after_restart = MockInferenceService::default().authorize(
            MockInferenceAuthorizationRequest {
                task_id: binding().task_id,
                attempt_id: ready.attempt_id.unwrap(),
                authorization_id: ready.authorization.unwrap().id,
            },
            &binding(),
        );
        assert_eq!(after_restart.state, MockAttemptState::Invalidated);
        assert_eq!(
            after_restart.diagnostic,
            Some(MockDiagnostic::AttemptUnavailable)
        );
    }
}
