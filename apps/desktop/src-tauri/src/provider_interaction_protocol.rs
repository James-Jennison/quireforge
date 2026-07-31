//! Private, local canonical interaction and mock-adapter contracts.
//!
//! This module contains only fictional, deterministic fixture translation. It
//! has no transport, secret custody, persistence, command, frontend, or native
//! operation path.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use uuid::Uuid;

const SCHEMA_VERSION: u16 = 1;
const PROTOCOL_VERSION: u16 = 1;
const FIXTURE_PROVIDER_ID: &str = "019a5700-0000-7000-8000-000000000001";
const FIXTURE_ENDPOINT_ID: &str = "019a5700-0000-7000-8000-000000000002";
const FIXTURE_MODEL_ID: &str = "019a5700-0000-7000-8000-000000000003";
const FIXTURE_ADAPTER_ID: &str = "019a5700-0000-7000-8000-000000000006";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum AttemptState {
    Open,
    Streaming,
    CancellationRequested,
    Completed,
    Refused,
    CancellationConfirmed,
    TimedOut,
    Interrupted,
    Ambiguous,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum RecoveryPath {
    New,
    Retry,
    Regeneration,
    Continuation,
    ProviderSwitch,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ProjectionKind {
    Image,
    Audio,
    Video,
    Document,
    Artifact,
    ProviderFile,
    Citation,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ProviderReferenceKind {
    Session,
    Continuation,
    Response,
    Run,
    File,
    Cache,
    RealtimeSession,
    BatchJob,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum StreamKind {
    Text,
    Structured,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum StructuredState {
    Partial,
    CompleteValid,
    CompleteInvalid,
    Nonconforming,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ToolReceiptState {
    Succeeded,
    Rejected,
    Ambiguous,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum SourceAdmissionState {
    Unadmitted,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum GroundingState {
    ProviderClaimed,
    Unverified,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum UsageBasis {
    Reported,
    Estimated,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum TerminalState {
    Completed,
    Refused,
    CancellationConfirmed,
    TimedOut,
    Interrupted,
    Ambiguous,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum FailureClass {
    FixtureRejected,
    TransportClosed,
    FixtureTimeout,
    FixtureInterrupted,
    FixtureAmbiguous,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ExtensionKind {
    EventDetail,
    DisplayHint,
    ProviderDiagnostic,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AttemptBinding {
    id: String,
    schema_version: u16,
    version: u16,
    project_id: String,
    task_id: String,
    provider_id: String,
    endpoint_id: String,
    model_id: String,
    adapter_id: String,
    adapter_version: u32,
    protocol_version: u16,
    registry_sha256: String,
    capability_profile_sha256: String,
    account_class: String,
    recovery_path: RecoveryPath,
    prior_attempt_id: Option<String>,
    sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BoundedReference {
    kind: ProjectionKind,
    id: String,
    sha256: String,
    byte_count: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StructuredPart {
    schema_id: String,
    schema_sha256: String,
    state: StructuredState,
    fragment: String,
    sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "kind", deny_unknown_fields)]
enum ContentPart {
    Text { text: String, sha256: String },
    Structured { value: StructuredPart },
    Reference { value: BoundedReference },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProviderReference {
    kind: ProviderReferenceKind,
    id: String,
    provider_id: String,
    endpoint_id: String,
    account_class: String,
    adapter_id: String,
    protocol_version: u16,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FictionalReceipt {
    id: String,
    sha256: String,
    state: ToolReceiptState,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UsageUnit {
    unit: String,
    quantity: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExtensionEnvelope {
    namespace: String,
    version: u16,
    kind: ExtensionKind,
    payload_sha256: String,
    item_count: u16,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "kind", deny_unknown_fields)]
enum EventPayload {
    Input {
        parts: Vec<ContentPart>,
    },
    Output {
        parts: Vec<ContentPart>,
    },
    Delta {
        stream: StreamKind,
        text: String,
    },
    Replacement {
        replaces_sequence: u64,
        parts: Vec<ContentPart>,
    },
    ProviderReference {
        value: ProviderReference,
    },
    ToolProposal {
        proposal_id: String,
        label: String,
        arguments_sha256: String,
    },
    ToolResult {
        receipt: FictionalReceipt,
    },
    Citation {
        reference: BoundedReference,
        admission: SourceAdmissionState,
        grounding: GroundingState,
    },
    Usage {
        basis: UsageBasis,
        units: Vec<UsageUnit>,
    },
    Structured {
        value: StructuredPart,
    },
    CancellationRequested,
    Failure {
        class: FailureClass,
    },
    Terminal {
        state: TerminalState,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EventEnvelope {
    schema_version: u16,
    attempt_id: String,
    event_id: String,
    sequence: u64,
    correlation_id: String,
    payload: EventPayload,
    extensions: Vec<ExtensionEnvelope>,
    sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ProtocolError {
    InvalidIdentity,
    InvalidVersion,
    InvalidDigest,
    InvalidBinding,
    InvalidPayload,
    InvalidExtension,
    UnknownFixture,
    DuplicateSequence,
    OutOfOrderSequence,
    InvalidTransition,
    TerminalAttempt,
    AdapterMismatch,
}

struct AttemptTracker {
    binding: AttemptBinding,
    state: AttemptState,
    next_sequence: u64,
    seen_event_ids: BTreeSet<String>,
}

impl AttemptTracker {
    fn new(binding: AttemptBinding) -> Result<Self, ProtocolError> {
        validate_binding(&binding)?;
        Ok(Self {
            binding,
            state: AttemptState::Open,
            next_sequence: 1,
            seen_event_ids: BTreeSet::new(),
        })
    }

    fn accept(&mut self, envelope: &EventEnvelope) -> Result<(), ProtocolError> {
        validate_envelope(envelope, &self.binding)?;
        if is_terminal(&self.state) {
            return Err(ProtocolError::TerminalAttempt);
        }
        if !self.seen_event_ids.insert(envelope.event_id.clone()) {
            return Err(ProtocolError::DuplicateSequence);
        }
        if envelope.sequence != self.next_sequence {
            return Err(ProtocolError::OutOfOrderSequence);
        }
        self.state = transition(&self.state, &envelope.payload)?;
        self.next_sequence += 1;
        Ok(())
    }
}

fn fictional_attempt() -> AttemptBinding {
    let mut binding = AttemptBinding {
        id: fixture_id(20),
        schema_version: SCHEMA_VERSION,
        version: 1,
        project_id: fixture_id(21),
        task_id: fixture_id(22),
        provider_id: FIXTURE_PROVIDER_ID.into(),
        endpoint_id: FIXTURE_ENDPOINT_ID.into(),
        model_id: FIXTURE_MODEL_ID.into(),
        adapter_id: FIXTURE_ADAPTER_ID.into(),
        adapter_version: 1,
        protocol_version: PROTOCOL_VERSION,
        registry_sha256: fixture_digest("registry"),
        capability_profile_sha256: fixture_digest("capability-profile"),
        account_class: "fictional-fixture-account-class".into(),
        recovery_path: RecoveryPath::New,
        prior_attempt_id: None,
        sha256: String::new(),
    };
    binding.sha256 = binding_digest(&binding);
    binding
}

fn fictional_input() -> EventEnvelope {
    envelope(
        &fictional_attempt(),
        1,
        EventPayload::Input {
            parts: vec![text_part("fixture-input")],
        },
    )
}

fn parse_envelope(input: &str) -> Result<EventEnvelope, ProtocolError> {
    let envelope = serde_json::from_str(input).map_err(|_| ProtocolError::UnknownFixture)?;
    let binding = fictional_attempt();
    validate_envelope(&envelope, &binding)?;
    Ok(envelope)
}

fn validate_binding(value: &AttemptBinding) -> Result<(), ProtocolError> {
    if !all_uuidv7(&[
        &value.id,
        &value.project_id,
        &value.task_id,
        &value.provider_id,
        &value.endpoint_id,
        &value.model_id,
        &value.adapter_id,
    ]) || value.schema_version != SCHEMA_VERSION
        || value.version == 0
        || value.adapter_version == 0
        || value.protocol_version != PROTOCOL_VERSION
        || !valid_digest(&value.registry_sha256)
        || !valid_digest(&value.capability_profile_sha256)
        || !valid_label(&value.account_class)
        || value.sha256 != binding_digest(value)
    {
        return Err(ProtocolError::InvalidBinding);
    }
    match (&value.recovery_path, &value.prior_attempt_id) {
        (RecoveryPath::New, None) => Ok(()),
        (RecoveryPath::New, Some(_)) | (_, None) => Err(ProtocolError::InvalidBinding),
        (_, Some(id)) if valid_uuidv7(id) && id != &value.id => Ok(()),
        _ => Err(ProtocolError::InvalidBinding),
    }
}

fn validate_envelope(value: &EventEnvelope, binding: &AttemptBinding) -> Result<(), ProtocolError> {
    if value.schema_version != SCHEMA_VERSION
        || value.attempt_id != binding.id
        || !valid_uuidv7(&value.event_id)
        || !valid_uuidv7(&value.correlation_id)
        || value.sequence == 0
        || value.sha256 != envelope_digest(value)
    {
        return Err(ProtocolError::InvalidDigest);
    }
    validate_payload(&value.payload, binding)?;
    if value.extensions.iter().any(invalid_extension) {
        return Err(ProtocolError::InvalidExtension);
    }
    Ok(())
}

fn validate_payload(value: &EventPayload, binding: &AttemptBinding) -> Result<(), ProtocolError> {
    match value {
        EventPayload::Input { parts }
        | EventPayload::Output { parts }
        | EventPayload::Replacement { parts, .. } => valid_parts(parts),
        EventPayload::Delta { text, .. } => {
            if valid_text(text) {
                Ok(())
            } else {
                Err(ProtocolError::InvalidPayload)
            }
        }
        EventPayload::ProviderReference { value } => valid_provider_reference(value, binding),
        EventPayload::ToolProposal {
            proposal_id,
            label,
            arguments_sha256,
        } => {
            if valid_uuidv7(proposal_id) && valid_label(label) && valid_digest(arguments_sha256) {
                Ok(())
            } else {
                Err(ProtocolError::InvalidPayload)
            }
        }
        EventPayload::ToolResult { receipt } => {
            if valid_uuidv7(&receipt.id)
                && valid_digest(&receipt.sha256)
                && receipt.state != ToolReceiptState::Ambiguous
            {
                Ok(())
            } else {
                Err(ProtocolError::InvalidPayload)
            }
        }
        EventPayload::Citation {
            reference,
            admission,
            grounding: _,
        } => {
            if *admission == SourceAdmissionState::Unadmitted {
                valid_reference(reference)
            } else {
                Err(ProtocolError::InvalidPayload)
            }
        }
        EventPayload::Usage { units, .. } => {
            if units.is_empty() || units.iter().any(|unit| !valid_label(&unit.unit)) {
                Err(ProtocolError::InvalidPayload)
            } else {
                Ok(())
            }
        }
        EventPayload::Structured { value } => valid_structured(value),
        EventPayload::CancellationRequested
        | EventPayload::Failure { .. }
        | EventPayload::Terminal { .. } => Ok(()),
    }
}

fn valid_parts(parts: &[ContentPart]) -> Result<(), ProtocolError> {
    if parts.is_empty() || parts.len() > 4 {
        return Err(ProtocolError::InvalidPayload);
    }
    for part in parts {
        match part {
            ContentPart::Text { text, sha256 }
                if valid_text(text) && &digest_text(text) == sha256 => {}
            ContentPart::Structured { value } => valid_structured(value)?,
            ContentPart::Reference { value } => valid_reference(value)?,
            _ => return Err(ProtocolError::InvalidPayload),
        }
    }
    Ok(())
}

fn valid_structured(value: &StructuredPart) -> Result<(), ProtocolError> {
    if valid_label(&value.schema_id)
        && valid_digest(&value.schema_sha256)
        && valid_text(&value.fragment)
        && value.sha256 == structured_digest(value)
    {
        Ok(())
    } else {
        Err(ProtocolError::InvalidPayload)
    }
}

fn valid_reference(value: &BoundedReference) -> Result<(), ProtocolError> {
    if valid_uuidv7(&value.id)
        && valid_digest(&value.sha256)
        && value.byte_count > 0
        && value.byte_count <= 65_536
    {
        Ok(())
    } else {
        Err(ProtocolError::InvalidPayload)
    }
}

fn valid_provider_reference(
    value: &ProviderReference,
    binding: &AttemptBinding,
) -> Result<(), ProtocolError> {
    if valid_uuidv7(&value.id)
        && value.provider_id == binding.provider_id
        && value.endpoint_id == binding.endpoint_id
        && value.account_class == binding.account_class
        && value.adapter_id == binding.adapter_id
        && value.protocol_version == binding.protocol_version
    {
        Ok(())
    } else {
        Err(ProtocolError::InvalidPayload)
    }
}

fn invalid_extension(value: &ExtensionEnvelope) -> bool {
    value.version == 0
        || value.item_count == 0
        || !value.namespace.starts_with("fictional.adapter.")
        || !value
            .namespace
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '.' || c == '-')
        || !valid_digest(&value.payload_sha256)
}

fn transition(
    current: &AttemptState,
    payload: &EventPayload,
) -> Result<AttemptState, ProtocolError> {
    match (current, payload) {
        (AttemptState::Open, EventPayload::Input { .. }) => Ok(AttemptState::Open),
        (
            AttemptState::Open | AttemptState::Streaming,
            EventPayload::Output { .. }
            | EventPayload::Delta { .. }
            | EventPayload::Replacement { .. }
            | EventPayload::Structured { .. }
            | EventPayload::ProviderReference { .. }
            | EventPayload::ToolProposal { .. }
            | EventPayload::ToolResult { .. }
            | EventPayload::Citation { .. }
            | EventPayload::Usage { .. },
        ) => Ok(AttemptState::Streaming),
        (AttemptState::Open | AttemptState::Streaming, EventPayload::CancellationRequested) => {
            Ok(AttemptState::CancellationRequested)
        }
        (
            AttemptState::CancellationRequested,
            EventPayload::Usage { .. } | EventPayload::ProviderReference { .. },
        ) => Ok(AttemptState::CancellationRequested),
        (
            _,
            EventPayload::Failure {
                class: FailureClass::TransportClosed,
            },
        ) => Ok(AttemptState::Ambiguous),
        (
            _,
            EventPayload::Failure {
                class: FailureClass::FixtureTimeout,
            },
        ) => Ok(AttemptState::TimedOut),
        (
            _,
            EventPayload::Failure {
                class: FailureClass::FixtureInterrupted,
            },
        ) => Ok(AttemptState::Interrupted),
        (
            _,
            EventPayload::Failure {
                class: FailureClass::FixtureAmbiguous,
            },
        ) => Ok(AttemptState::Ambiguous),
        (
            _,
            EventPayload::Failure {
                class: FailureClass::FixtureRejected,
            },
        ) => Ok(AttemptState::Failed),
        (_, EventPayload::Terminal { state }) => Ok(terminal_attempt_state(state)),
        _ => Err(ProtocolError::InvalidTransition),
    }
}

fn terminal_attempt_state(value: &TerminalState) -> AttemptState {
    match value {
        TerminalState::Completed => AttemptState::Completed,
        TerminalState::Refused => AttemptState::Refused,
        TerminalState::CancellationConfirmed => AttemptState::CancellationConfirmed,
        TerminalState::TimedOut => AttemptState::TimedOut,
        TerminalState::Interrupted => AttemptState::Interrupted,
        TerminalState::Ambiguous => AttemptState::Ambiguous,
        TerminalState::Failed => AttemptState::Failed,
    }
}

fn is_terminal(value: &AttemptState) -> bool {
    !matches!(
        value,
        AttemptState::Open | AttemptState::Streaming | AttemptState::CancellationRequested
    )
}

fn envelope(binding: &AttemptBinding, sequence: u64, payload: EventPayload) -> EventEnvelope {
    let mut value = EventEnvelope {
        schema_version: SCHEMA_VERSION,
        attempt_id: binding.id.clone(),
        event_id: fixture_id(40 + sequence as u8),
        sequence,
        correlation_id: fixture_id(60),
        payload,
        extensions: vec![ExtensionEnvelope {
            namespace: "fictional.adapter.event".into(),
            version: 1,
            kind: ExtensionKind::EventDetail,
            payload_sha256: fixture_digest("extension"),
            item_count: 1,
        }],
        sha256: String::new(),
    };
    value.sha256 = envelope_digest(&value);
    value
}

fn text_part(value: &str) -> ContentPart {
    ContentPart::Text {
        text: value.into(),
        sha256: digest_text(value),
    }
}

fn binding_digest(value: &AttemptBinding) -> String {
    digest_debug(&[
        format!("{:?}", value.schema_version),
        value.id.clone(),
        value.project_id.clone(),
        value.task_id.clone(),
        value.provider_id.clone(),
        value.endpoint_id.clone(),
        value.model_id.clone(),
        value.adapter_id.clone(),
        value.adapter_version.to_string(),
        value.protocol_version.to_string(),
        value.registry_sha256.clone(),
        value.capability_profile_sha256.clone(),
        value.account_class.clone(),
        format!("{:?}", value.recovery_path),
        format!("{:?}", value.prior_attempt_id),
    ])
}

fn envelope_digest(value: &EventEnvelope) -> String {
    digest_debug(&[
        value.schema_version.to_string(),
        value.attempt_id.clone(),
        value.event_id.clone(),
        value.sequence.to_string(),
        value.correlation_id.clone(),
        format!("{:?}", value.payload),
        format!("{:?}", value.extensions),
    ])
}

fn structured_digest(value: &StructuredPart) -> String {
    digest_debug(&[
        value.schema_id.clone(),
        value.schema_sha256.clone(),
        format!("{:?}", value.state),
        value.fragment.clone(),
    ])
}

fn digest_text(value: &str) -> String {
    digest_debug(&[value.into()])
}
fn fixture_digest(value: &str) -> String {
    digest_text(value)
}
fn digest_debug(parts: &[String]) -> String {
    let mut hash = Sha256::new();
    for part in parts {
        hash.update(part.as_bytes());
        hash.update([0]);
    }
    format!("{:x}", hash.finalize())
}
fn valid_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
fn valid_uuidv7(value: &str) -> bool {
    Uuid::parse_str(value).is_ok_and(|id| id.get_version_num() == 7)
}
fn all_uuidv7(values: &[&str]) -> bool {
    values.iter().all(|value| valid_uuidv7(value))
}
fn valid_label(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 120
        && value.is_ascii()
        && !value.contains('/')
        && !value.contains('\\')
}
fn valid_text(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 512
        && value.is_ascii()
        && !value.contains('\0')
        && !value.contains('/')
        && !value.contains('\\')
}
fn fixture_id(number: u8) -> String {
    format!("019a5700-0000-7000-8000-{number:012}")
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum FictionalNativeEvent {
    OutputText { label: String },
    DeltaText { label: String },
    Complete,
    Refusal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FictionalRequestRepresentation {
    adapter_id: String,
    protocol_version: u16,
    input_digest: String,
    part_count: u16,
}

struct DeterministicMockAdapter;

impl DeterministicMockAdapter {
    fn map_input(
        binding: &AttemptBinding,
        input: &EventEnvelope,
    ) -> Result<FictionalRequestRepresentation, ProtocolError> {
        validate_binding(binding)?;
        validate_envelope(input, binding)?;
        if binding.provider_id != FIXTURE_PROVIDER_ID
            || binding.endpoint_id != FIXTURE_ENDPOINT_ID
            || binding.model_id != FIXTURE_MODEL_ID
            || binding.adapter_id != FIXTURE_ADAPTER_ID
            || binding.adapter_version != 1
            || !matches!(input.payload, EventPayload::Input { .. })
        {
            return Err(ProtocolError::AdapterMismatch);
        }
        let EventPayload::Input { parts } = &input.payload else {
            return Err(ProtocolError::AdapterMismatch);
        };
        Ok(FictionalRequestRepresentation {
            adapter_id: binding.adapter_id.clone(),
            protocol_version: binding.protocol_version,
            input_digest: envelope_digest(input),
            part_count: parts.len() as u16,
        })
    }

    fn translate(
        binding: &AttemptBinding,
        event: FictionalNativeEvent,
        sequence: u64,
    ) -> Result<EventEnvelope, ProtocolError> {
        validate_binding(binding)?;
        if binding.provider_id != FIXTURE_PROVIDER_ID
            || binding.endpoint_id != FIXTURE_ENDPOINT_ID
            || binding.model_id != FIXTURE_MODEL_ID
            || binding.adapter_id != FIXTURE_ADAPTER_ID
            || binding.protocol_version != PROTOCOL_VERSION
        {
            return Err(ProtocolError::AdapterMismatch);
        }
        let payload = match event {
            FictionalNativeEvent::OutputText { label } => EventPayload::Output {
                parts: vec![text_part(&label)],
            },
            FictionalNativeEvent::DeltaText { label } => EventPayload::Delta {
                stream: StreamKind::Text,
                text: label,
            },
            FictionalNativeEvent::Complete => EventPayload::Terminal {
                state: TerminalState::Completed,
            },
            FictionalNativeEvent::Refusal => EventPayload::Terminal {
                state: TerminalState::Refused,
            },
        };
        Ok(envelope(binding, sequence, payload))
    }

    fn native_identity(binding: &AttemptBinding, label: &str) -> String {
        digest_debug(&[
            binding.adapter_id.clone(),
            binding.protocol_version.to_string(),
            label.into(),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelopes_are_canonical_and_digest_deterministic() {
        let binding = fictional_attempt();
        let a = fictional_input();
        let b = fictional_input();
        assert_eq!(a, b);
        validate_binding(&binding).unwrap();
        validate_envelope(&a, &binding).unwrap();
    }

    #[test]
    fn fixture_parsing_rejects_unknown_fields_and_enums() {
        let serialized = serde_json::to_string(&fictional_input()).unwrap();
        assert_eq!(
            parse_envelope(&serialized.replacen(
                "\"sequence\":1",
                "\"unknown\":true,\"sequence\":1",
                1
            )),
            Err(ProtocolError::UnknownFixture)
        );
        assert_eq!(
            parse_envelope(&serialized.replacen("\"input\"", "\"unsafe\"", 1)),
            Err(ProtocolError::UnknownFixture)
        );
    }

    #[test]
    fn tracker_rejects_duplicate_out_of_order_and_terminal_streaming() {
        let binding = fictional_attempt();
        let mut tracker = AttemptTracker::new(binding.clone()).unwrap();
        let input = fictional_input();
        tracker.accept(&input).unwrap();
        assert_eq!(
            tracker.accept(&input),
            Err(ProtocolError::DuplicateSequence)
        );
        let skipped = DeterministicMockAdapter::translate(
            &binding,
            FictionalNativeEvent::OutputText {
                label: "fixture-output".into(),
            },
            3,
        )
        .unwrap();
        assert_eq!(
            tracker.accept(&skipped),
            Err(ProtocolError::OutOfOrderSequence)
        );
        let complete =
            DeterministicMockAdapter::translate(&binding, FictionalNativeEvent::Complete, 2)
                .unwrap();
        tracker.accept(&complete).unwrap();
        let after = DeterministicMockAdapter::translate(
            &binding,
            FictionalNativeEvent::DeltaText {
                label: "later".into(),
            },
            3,
        )
        .unwrap();
        assert_eq!(tracker.accept(&after), Err(ProtocolError::TerminalAttempt));
    }

    #[test]
    fn cancellation_timeout_and_transport_closure_remain_distinct() {
        let binding = fictional_attempt();
        let mut cancelled = AttemptTracker::new(binding.clone()).unwrap();
        cancelled.accept(&fictional_input()).unwrap();
        cancelled
            .accept(&envelope(&binding, 2, EventPayload::CancellationRequested))
            .unwrap();
        assert_eq!(cancelled.state, AttemptState::CancellationRequested);
        cancelled
            .accept(&envelope(
                &binding,
                3,
                EventPayload::Terminal {
                    state: TerminalState::CancellationConfirmed,
                },
            ))
            .unwrap();
        assert_eq!(cancelled.state, AttemptState::CancellationConfirmed);
        assert_eq!(
            transition(
                &AttemptState::Streaming,
                &EventPayload::Failure {
                    class: FailureClass::FixtureTimeout
                }
            ),
            Ok(AttemptState::TimedOut)
        );
        assert_eq!(
            transition(
                &AttemptState::Streaming,
                &EventPayload::Failure {
                    class: FailureClass::TransportClosed
                }
            ),
            Ok(AttemptState::Ambiguous)
        );
    }

    #[test]
    fn recovery_paths_sessions_tools_and_citations_fail_closed_when_mismatched() {
        let binding = fictional_attempt();
        for (number, path) in [
            (23, RecoveryPath::Retry),
            (24, RecoveryPath::Regeneration),
            (25, RecoveryPath::Continuation),
            (26, RecoveryPath::ProviderSwitch),
        ] {
            let mut recovery = fictional_attempt();
            recovery.id = fixture_id(number);
            recovery.recovery_path = path;
            recovery.prior_attempt_id = Some(binding.id.clone());
            recovery.sha256 = binding_digest(&recovery);
            validate_binding(&recovery).unwrap();
            assert_ne!(recovery.id, binding.id);
        }
        let reference = ProviderReference {
            kind: ProviderReferenceKind::Session,
            id: fixture_id(24),
            provider_id: binding.provider_id.clone(),
            endpoint_id: binding.endpoint_id.clone(),
            account_class: binding.account_class.clone(),
            adapter_id: binding.adapter_id.clone(),
            protocol_version: binding.protocol_version,
        };
        validate_payload(
            &EventPayload::ProviderReference {
                value: reference.clone(),
            },
            &binding,
        )
        .unwrap();
        let mut mismatched_reference = reference;
        mismatched_reference.account_class = "other-fixture-account-class".into();
        assert_eq!(
            validate_payload(
                &EventPayload::ProviderReference {
                    value: mismatched_reference
                },
                &binding
            ),
            Err(ProtocolError::InvalidPayload)
        );
        let tool = EventPayload::ToolProposal {
            proposal_id: fixture_id(25),
            label: "fixture-tool-proposal".into(),
            arguments_sha256: fixture_digest("arguments"),
        };
        validate_payload(&tool, &binding).unwrap();
        let ambiguous = EventPayload::ToolResult {
            receipt: FictionalReceipt {
                id: fixture_id(26),
                sha256: fixture_digest("receipt"),
                state: ToolReceiptState::Ambiguous,
            },
        };
        assert_eq!(
            validate_payload(&ambiguous, &binding),
            Err(ProtocolError::InvalidPayload)
        );
        let citation = EventPayload::Citation {
            reference: BoundedReference {
                kind: ProjectionKind::Citation,
                id: fixture_id(27),
                sha256: fixture_digest("citation"),
                byte_count: 1,
            },
            admission: SourceAdmissionState::Unadmitted,
            grounding: GroundingState::ProviderClaimed,
        };
        validate_payload(&citation, &binding).unwrap();
    }

    #[test]
    fn structured_usage_extensions_and_adapter_translation_are_bounded_and_deterministic() {
        let binding = fictional_attempt();
        for state in [
            StructuredState::Partial,
            StructuredState::CompleteValid,
            StructuredState::CompleteInvalid,
            StructuredState::Nonconforming,
        ] {
            let structured = StructuredPart {
                schema_id: "fixture-schema".into(),
                schema_sha256: fixture_digest("schema"),
                state,
                fragment: "fixture-structured".into(),
                sha256: String::new(),
            };
            let structured = StructuredPart {
                sha256: structured_digest(&structured),
                ..structured
            };
            validate_payload(&EventPayload::Structured { value: structured }, &binding).unwrap();
        }
        for basis in [UsageBasis::Reported, UsageBasis::Estimated] {
            validate_payload(
                &EventPayload::Usage {
                    basis,
                    units: vec![UsageUnit {
                        unit: "fictional-provider-unit".into(),
                        quantity: 2,
                    }],
                },
                &binding,
            )
            .unwrap();
        }
        let input = fictional_input();
        let mapped = DeterministicMockAdapter::map_input(&binding, &input).unwrap();
        assert_eq!(mapped.part_count, 1);
        let translated = DeterministicMockAdapter::translate(
            &binding,
            FictionalNativeEvent::OutputText {
                label: "fixture-output".into(),
            },
            2,
        )
        .unwrap();
        validate_envelope(&translated, &binding).unwrap();
        let mut unsafe_extension = translated.clone();
        unsafe_extension.extensions[0].namespace = "canonical.lifecycle".into();
        unsafe_extension.sha256 = envelope_digest(&unsafe_extension);
        assert_eq!(
            validate_envelope(&unsafe_extension, &binding),
            Err(ProtocolError::InvalidExtension)
        );
    }

    #[test]
    fn same_named_native_events_do_not_collapse_across_adapters() {
        let binding = fictional_attempt();
        let mut other = fictional_attempt();
        other.adapter_id = fixture_id(28);
        other.sha256 = binding_digest(&other);
        assert_ne!(
            DeterministicMockAdapter::native_identity(&binding, "fixture-event"),
            DeterministicMockAdapter::native_identity(&other, "fixture-event")
        );
        assert_eq!(
            DeterministicMockAdapter::translate(&other, FictionalNativeEvent::Complete, 1),
            Err(ProtocolError::AdapterMismatch)
        );
    }
}
