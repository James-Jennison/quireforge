//! Static, local provider-capability registry contracts.
//!
//! This private module describes fictional capability metadata only. It has no
//! transport, credential, context, process, persistence, or frontend path.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use uuid::Uuid;

const SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ProvenanceClass {
    BuiltInStatic,
    PackagedStatic,
    UntrustedDeclaration,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum TrustClass {
    ReviewedStatic,
    Unverified,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum LifecycleState {
    Active,
    Preview,
    Deprecated,
    Retired,
    Unavailable,
    Quarantined,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum CompatibilityState {
    Compatible,
    Incompatible,
    Unknown,
    DriftDetected,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Capability {
    TextInput,
    TextOutput,
    StructuredOutput,
    Streaming,
    Cancellation,
    Continuation,
    UsageReporting,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum SupportState {
    Supported,
    Advertised,
    Observed,
    Unknown,
    Conditional,
    Deprecated,
    TemporarilyUnavailable,
    Incompatible,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "kind", content = "value")]
enum Limit {
    ContextTokens(u32),
    OutputTokens(u32),
    ParallelCalls(u16),
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Qualifier {
    StaticFixtureOnly,
    RequiresCompatibleAdapter,
    EndpointSpecific,
    PreviewOnly,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ExtensionKind {
    EventDetail,
    DisplayHint,
    ResponseControl,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExtensionDescriptor {
    namespace: String,
    version: u16,
    kinds: BTreeSet<ExtensionKind>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProviderDescriptor {
    id: String,
    schema_version: u16,
    version: u32,
    display_name: String,
    provenance: ProvenanceClass,
    trust: TrustClass,
    lifecycle: LifecycleState,
    sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EndpointDescriptor {
    id: String,
    schema_version: u16,
    version: u32,
    provider_id: String,
    deployment_label: String,
    provenance: ProvenanceClass,
    trust: TrustClass,
    lifecycle: LifecycleState,
    sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ModelDescriptor {
    id: String,
    schema_version: u16,
    version: u32,
    provider_id: String,
    endpoint_id: String,
    model_label: String,
    lifecycle: LifecycleState,
    compatibility: CompatibilityState,
    sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LocalRuntimeDescriptor {
    id: String,
    schema_version: u16,
    version: u32,
    runtime_label: String,
    lifecycle: LifecycleState,
    compatibility: CompatibilityState,
    sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AdapterIdentityDescriptor {
    id: String,
    schema_version: u16,
    version: u32,
    provider_id: String,
    protocol_version: u16,
    compatibility: CompatibilityState,
    sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CapabilityDefinition {
    id: Capability,
    schema_version: u16,
    version: u16,
    description: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CapabilityClaim {
    id: String,
    schema_version: u16,
    version: u32,
    model_id: String,
    model_sha256: String,
    capability: Capability,
    support: SupportState,
    qualifiers: BTreeSet<Qualifier>,
    limits: Vec<Limit>,
    extensions: Vec<ExtensionDescriptor>,
    sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RegistryFixture {
    provider: ProviderDescriptor,
    endpoints: Vec<EndpointDescriptor>,
    models: Vec<ModelDescriptor>,
    runtime: LocalRuntimeDescriptor,
    adapter: AdapterIdentityDescriptor,
    capabilities: Vec<CapabilityDefinition>,
    claims: Vec<CapabilityClaim>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum RegistryError {
    InvalidIdentity,
    InvalidVersion,
    InvalidDigest,
    InvalidDescriptor,
    InvalidClaim,
    DescriptorDrift,
    UnknownFixture,
}

fn fictional_fixture() -> RegistryFixture {
    let provider = ProviderDescriptor {
        id: id(1),
        schema_version: SCHEMA_VERSION,
        version: 1,
        display_name: "Fictional Lantern Platform".into(),
        provenance: ProvenanceClass::BuiltInStatic,
        trust: TrustClass::ReviewedStatic,
        lifecycle: LifecycleState::Active,
        sha256: String::new(),
    };
    let provider = with_provider_digest(provider);
    let endpoint = with_endpoint_digest(EndpointDescriptor {
        id: id(2),
        schema_version: SCHEMA_VERSION,
        version: 1,
        provider_id: provider.id.clone(),
        deployment_label: "lantern-static-deployment".into(),
        provenance: ProvenanceClass::BuiltInStatic,
        trust: TrustClass::ReviewedStatic,
        lifecycle: LifecycleState::Active,
        sha256: String::new(),
    });
    let alternate_endpoint = with_endpoint_digest(EndpointDescriptor {
        id: id(8),
        schema_version: SCHEMA_VERSION,
        version: 1,
        provider_id: provider.id.clone(),
        deployment_label: "lantern-static-alternate".into(),
        provenance: ProvenanceClass::BuiltInStatic,
        trust: TrustClass::ReviewedStatic,
        lifecycle: LifecycleState::Preview,
        sha256: String::new(),
    });
    let first = with_model_digest(ModelDescriptor {
        id: id(3),
        schema_version: SCHEMA_VERSION,
        version: 1,
        provider_id: provider.id.clone(),
        endpoint_id: alternate_endpoint.id.clone(),
        model_label: "aurora-text".into(),
        lifecycle: LifecycleState::Active,
        compatibility: CompatibilityState::Compatible,
        sha256: String::new(),
    });
    let second = with_model_digest(ModelDescriptor {
        id: id(4),
        schema_version: SCHEMA_VERSION,
        version: 1,
        provider_id: provider.id.clone(),
        endpoint_id: endpoint.id.clone(),
        model_label: "aurora-text".into(),
        lifecycle: LifecycleState::Preview,
        compatibility: CompatibilityState::Unknown,
        sha256: String::new(),
    });
    let runtime = with_runtime_digest(LocalRuntimeDescriptor {
        id: id(5),
        schema_version: SCHEMA_VERSION,
        version: 1,
        runtime_label: "fictional-local-lantern".into(),
        lifecycle: LifecycleState::Unavailable,
        compatibility: CompatibilityState::Unknown,
        sha256: String::new(),
    });
    let adapter = with_adapter_digest(AdapterIdentityDescriptor {
        id: id(6),
        schema_version: SCHEMA_VERSION,
        version: 1,
        provider_id: provider.id.clone(),
        protocol_version: 1,
        compatibility: CompatibilityState::Compatible,
        sha256: String::new(),
    });
    let capabilities = vec![
        definition(Capability::TextInput, "bounded fictional text input"),
        definition(Capability::TextOutput, "bounded fictional text output"),
        definition(Capability::Streaming, "ordered fictional event translation"),
    ];
    let claim = with_claim_digest(CapabilityClaim {
        id: id(7),
        schema_version: SCHEMA_VERSION,
        version: 1,
        model_id: first.id.clone(),
        model_sha256: first.sha256.clone(),
        capability: Capability::TextOutput,
        support: SupportState::Advertised,
        qualifiers: BTreeSet::from([
            Qualifier::StaticFixtureOnly,
            Qualifier::RequiresCompatibleAdapter,
        ]),
        limits: vec![Limit::ContextTokens(2048), Limit::OutputTokens(512)],
        extensions: vec![ExtensionDescriptor {
            namespace: "fictional.lantern.response".into(),
            version: 1,
            kinds: BTreeSet::from([ExtensionKind::EventDetail]),
        }],
        sha256: String::new(),
    });
    RegistryFixture {
        provider,
        endpoints: vec![endpoint, alternate_endpoint],
        models: vec![first, second],
        runtime,
        adapter,
        capabilities,
        claims: vec![claim],
    }
}

fn parse_fixture(input: &str) -> Result<RegistryFixture, RegistryError> {
    let fixture: RegistryFixture =
        serde_json::from_str(input).map_err(|_| RegistryError::UnknownFixture)?;
    validate_fixture(&fixture)?;
    Ok(fixture)
}

fn validate_fixture(fixture: &RegistryFixture) -> Result<(), RegistryError> {
    valid_provider(&fixture.provider)?;
    if fixture.endpoints.len() < 2 {
        return Err(RegistryError::InvalidDescriptor);
    }
    for endpoint in &fixture.endpoints {
        valid_endpoint(endpoint, &fixture.provider)?;
    }
    valid_runtime(&fixture.runtime)?;
    valid_adapter(&fixture.adapter, &fixture.provider)?;
    if fixture.capabilities.is_empty() || fixture.models.len() < 2 {
        return Err(RegistryError::InvalidDescriptor);
    }
    for definition in &fixture.capabilities {
        if definition.schema_version != SCHEMA_VERSION
            || definition.version == 0
            || !valid_label(&definition.description)
        {
            return Err(RegistryError::InvalidVersion);
        }
    }
    for model in &fixture.models {
        let endpoint = fixture
            .endpoints
            .iter()
            .find(|endpoint| endpoint.id == model.endpoint_id)
            .ok_or(RegistryError::InvalidDescriptor)?;
        valid_model(model, &fixture.provider, endpoint)?;
    }
    for claim in &fixture.claims {
        let model = fixture
            .models
            .iter()
            .find(|model| model.id == claim.model_id)
            .ok_or(RegistryError::InvalidClaim)?;
        valid_claim(claim, model, &fixture.capabilities)?;
    }
    Ok(())
}

fn valid_provider(value: &ProviderDescriptor) -> Result<(), RegistryError> {
    validate_common(
        &value.id,
        value.schema_version,
        value.version,
        &value.display_name,
        &value.sha256,
        provider_digest(value),
    )
}
fn valid_endpoint(
    value: &EndpointDescriptor,
    provider: &ProviderDescriptor,
) -> Result<(), RegistryError> {
    if value.provider_id != provider.id {
        return Err(RegistryError::InvalidDescriptor);
    }
    validate_common(
        &value.id,
        value.schema_version,
        value.version,
        &value.deployment_label,
        &value.sha256,
        endpoint_digest(value),
    )
}
fn valid_model(
    value: &ModelDescriptor,
    provider: &ProviderDescriptor,
    endpoint: &EndpointDescriptor,
) -> Result<(), RegistryError> {
    if value.provider_id != provider.id || value.endpoint_id != endpoint.id {
        return Err(RegistryError::InvalidDescriptor);
    }
    if value.compatibility == CompatibilityState::DriftDetected {
        return Err(RegistryError::DescriptorDrift);
    }
    validate_common(
        &value.id,
        value.schema_version,
        value.version,
        &value.model_label,
        &value.sha256,
        model_digest(value),
    )
}
fn valid_runtime(value: &LocalRuntimeDescriptor) -> Result<(), RegistryError> {
    validate_common(
        &value.id,
        value.schema_version,
        value.version,
        &value.runtime_label,
        &value.sha256,
        runtime_digest(value),
    )
}
fn valid_adapter(
    value: &AdapterIdentityDescriptor,
    provider: &ProviderDescriptor,
) -> Result<(), RegistryError> {
    if value.provider_id != provider.id || value.protocol_version == 0 {
        return Err(RegistryError::InvalidDescriptor);
    }
    validate_common(
        &value.id,
        value.schema_version,
        value.version,
        "adapter",
        &value.sha256,
        adapter_digest(value),
    )
}
fn valid_claim(
    value: &CapabilityClaim,
    model: &ModelDescriptor,
    definitions: &[CapabilityDefinition],
) -> Result<(), RegistryError> {
    if !valid_uuid(&value.id)
        || value.schema_version != SCHEMA_VERSION
        || value.version == 0
        || value.model_sha256 != model.sha256
        || !definitions
            .iter()
            .any(|definition| definition.id == value.capability)
        || value.limits.iter().any(invalid_limit)
        || value.extensions.iter().any(invalid_extension)
    {
        return Err(RegistryError::InvalidClaim);
    }
    if value.sha256 != claim_digest(value) {
        return Err(RegistryError::InvalidDigest);
    }
    Ok(())
}
fn validate_common(
    id: &str,
    schema: u16,
    version: u32,
    label: &str,
    actual: &str,
    expected: String,
) -> Result<(), RegistryError> {
    if !valid_uuid(id) {
        Err(RegistryError::InvalidIdentity)
    } else if schema != SCHEMA_VERSION || version == 0 || !valid_label(label) {
        Err(RegistryError::InvalidVersion)
    } else if actual != expected {
        Err(RegistryError::InvalidDigest)
    } else {
        Ok(())
    }
}

fn invalid_limit(value: &Limit) -> bool {
    match value {
        Limit::ContextTokens(v) | Limit::OutputTokens(v) => *v == 0,
        Limit::ParallelCalls(v) => *v == 0,
    }
}
fn invalid_extension(value: &ExtensionDescriptor) -> bool {
    value.version == 0
        || value.kinds.is_empty()
        || !value.namespace.starts_with("fictional.")
        || !value
            .namespace
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '.' || c == '-')
}
fn valid_uuid(value: &str) -> bool {
    Uuid::parse_str(value).is_ok_and(|id| id.get_version_num() == 7)
}
fn valid_label(value: &str) -> bool {
    !value.is_empty() && value.len() <= 120 && value.is_ascii() && !value.contains("http")
}
fn id(number: u8) -> String {
    format!("019a5700-0000-7000-8000-{number:012}")
}
fn digest(parts: &[String]) -> String {
    let mut hash = Sha256::new();
    for part in parts {
        hash.update(part.as_bytes());
        hash.update([0]);
    }
    format!("{:x}", hash.finalize())
}
fn provider_digest(v: &ProviderDescriptor) -> String {
    digest(&[
        v.id.clone(),
        v.schema_version.to_string(),
        v.version.to_string(),
        v.display_name.clone(),
        format!("{:?}", v.provenance),
        format!("{:?}", v.trust),
        format!("{:?}", v.lifecycle),
    ])
}
fn endpoint_digest(v: &EndpointDescriptor) -> String {
    digest(&[
        v.id.clone(),
        v.schema_version.to_string(),
        v.version.to_string(),
        v.provider_id.clone(),
        v.deployment_label.clone(),
        format!("{:?}", v.provenance),
        format!("{:?}", v.trust),
        format!("{:?}", v.lifecycle),
    ])
}
fn model_digest(v: &ModelDescriptor) -> String {
    digest(&[
        v.id.clone(),
        v.schema_version.to_string(),
        v.version.to_string(),
        v.provider_id.clone(),
        v.endpoint_id.clone(),
        v.model_label.clone(),
        format!("{:?}", v.lifecycle),
        format!("{:?}", v.compatibility),
    ])
}
fn runtime_digest(v: &LocalRuntimeDescriptor) -> String {
    digest(&[
        v.id.clone(),
        v.schema_version.to_string(),
        v.version.to_string(),
        v.runtime_label.clone(),
        format!("{:?}", v.lifecycle),
        format!("{:?}", v.compatibility),
    ])
}
fn adapter_digest(v: &AdapterIdentityDescriptor) -> String {
    digest(&[
        v.id.clone(),
        v.schema_version.to_string(),
        v.version.to_string(),
        v.provider_id.clone(),
        v.protocol_version.to_string(),
        format!("{:?}", v.compatibility),
    ])
}
fn claim_digest(v: &CapabilityClaim) -> String {
    digest(&[
        v.id.clone(),
        v.schema_version.to_string(),
        v.version.to_string(),
        v.model_id.clone(),
        v.model_sha256.clone(),
        format!("{:?}", v.capability),
        format!("{:?}", v.support),
        format!("{:?}", v.qualifiers),
        format!("{:?}", v.limits),
        format!("{:?}", v.extensions),
    ])
}
fn with_provider_digest(mut v: ProviderDescriptor) -> ProviderDescriptor {
    v.sha256 = provider_digest(&v);
    v
}
fn with_endpoint_digest(mut v: EndpointDescriptor) -> EndpointDescriptor {
    v.sha256 = endpoint_digest(&v);
    v
}
fn with_model_digest(mut v: ModelDescriptor) -> ModelDescriptor {
    v.sha256 = model_digest(&v);
    v
}
fn with_runtime_digest(mut v: LocalRuntimeDescriptor) -> LocalRuntimeDescriptor {
    v.sha256 = runtime_digest(&v);
    v
}
fn with_adapter_digest(mut v: AdapterIdentityDescriptor) -> AdapterIdentityDescriptor {
    v.sha256 = adapter_digest(&v);
    v
}
fn with_claim_digest(mut v: CapabilityClaim) -> CapabilityClaim {
    v.sha256 = claim_digest(&v);
    v
}
fn definition(id: Capability, description: &str) -> CapabilityDefinition {
    CapabilityDefinition {
        id,
        schema_version: SCHEMA_VERSION,
        version: 1,
        description: description.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn static_fixture_is_deterministic_fictional_and_valid() {
        let a = fictional_fixture();
        let b = fictional_fixture();
        assert_eq!(a, b);
        validate_fixture(&a).unwrap();
        assert!(!format!("{a:?}").contains("http"));
    }
    #[test]
    fn digest_drift_is_detected() {
        let mut fixture = fictional_fixture();
        fixture.models[0].model_label = "changed".into();
        assert_eq!(
            validate_fixture(&fixture),
            Err(RegistryError::InvalidDigest)
        );
    }
    #[test]
    fn drifted_descriptor_is_invalidated_even_with_a_matching_digest() {
        let mut fixture = fictional_fixture();
        fixture.models[0].compatibility = CompatibilityState::DriftDetected;
        fixture.models[0].sha256 = model_digest(&fixture.models[0]);
        assert_eq!(
            validate_fixture(&fixture),
            Err(RegistryError::DescriptorDrift)
        );
    }
    #[test]
    fn malformed_identity_version_and_limits_fail_closed() {
        let mut fixture = fictional_fixture();
        fixture.provider.id = "bad".into();
        assert_eq!(
            validate_fixture(&fixture),
            Err(RegistryError::InvalidIdentity)
        );
        let mut fixture = fictional_fixture();
        fixture.models[0].version = 0;
        assert_eq!(
            validate_fixture(&fixture),
            Err(RegistryError::InvalidVersion)
        );
        let mut fixture = fictional_fixture();
        fixture.claims[0].limits = vec![Limit::OutputTokens(0)];
        assert_eq!(validate_fixture(&fixture), Err(RegistryError::InvalidClaim));
    }
    #[test]
    fn unknown_fields_and_enums_are_rejected() {
        let serialized = serde_json::to_string(&fictional_fixture()).unwrap();
        let unknown = serialized.replacen("\"provider\":", "\"unexpected\":true,\"provider\":", 1);
        assert_eq!(parse_fixture(&unknown), Err(RegistryError::UnknownFixture));
        for (known, invalid) in [
            ("\"active\"", "\"invalid-lifecycle\""),
            ("\"compatible\"", "\"invalid-compatibility\""),
            ("\"text-output\"", "\"invalid-capability\""),
            ("\"advertised\"", "\"invalid-support\""),
            ("\"static-fixture-only\"", "\"invalid-qualifier\""),
            ("\"event-detail\"", "\"invalid-extension\""),
        ] {
            assert_eq!(
                parse_fixture(&serialized.replacen(known, invalid, 1)),
                Err(RegistryError::UnknownFixture)
            );
        }
    }
    #[test]
    fn claims_preserve_closed_support_states_and_namespaced_extensions() {
        for state in [
            SupportState::Advertised,
            SupportState::Observed,
            SupportState::Unknown,
            SupportState::Conditional,
            SupportState::Deprecated,
            SupportState::TemporarilyUnavailable,
            SupportState::Incompatible,
        ] {
            assert_ne!(state, SupportState::Supported);
        }
        let mut fixture = fictional_fixture();
        fixture.claims[0].extensions[0].namespace = "unsafe-authority".into();
        assert_eq!(validate_fixture(&fixture), Err(RegistryError::InvalidClaim));
    }
    #[test]
    fn same_named_models_remain_endpoint_aware() {
        let fixture = fictional_fixture();
        assert_eq!(fixture.models[0].model_label, fixture.models[1].model_label);
        assert_ne!(fixture.models[0].endpoint_id, fixture.models[1].endpoint_id);
        assert_ne!(fixture.models[0].id, fixture.models[1].id);
        assert_ne!(fixture.models[0].sha256, fixture.models[1].sha256);
    }
}
