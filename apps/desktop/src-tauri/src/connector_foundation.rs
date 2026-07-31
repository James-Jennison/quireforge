//! Local-only connector authority contracts.
//!
//! This module intentionally has no transport, provider, browser, process,
//! environment, persistence, or frontend dependency. Its deterministic mock
//! adapter exists solely to make the closed authority model testable.

use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashMap};
use uuid::Uuid;

const SCHEMA_VERSION: u16 = 1;
const PROPOSAL_TTL_MS: u64 = 5 * 60 * 1000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConnectorClass {
    LocalMock,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Scope {
    Metadata,
    Read,
    SearchFetch,
    MockMutation,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum OperationClass {
    DiscoverAvailability,
    ListAuthorizedAccounts,
    ReadMetadata,
    Search,
    FetchContent,
    FetchAttachment,
    ProposeMutation,
    ConfirmMutation,
    ExecuteConfirmedMockMutation,
    ReportResult,
    ReconcileAmbiguousMockResult,
    RevokeOrDisconnect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LifecycleState {
    KnownUnavailable,
    AvailableDisconnected,
    ConnectedUnauthorized,
    AuthorizedMetadata,
    AuthorizedBoundedRead,
    AuthorizedSearchFetch,
    PendingMutationConfirmation,
    ConfirmedNotDispatched,
    Dispatched,
    Completed,
    Expired,
    Revoked,
    Degraded,
    Quarantined,
    Removed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ResultState {
    Succeeded,
    Rejected,
    Cancelled,
    TimedOutBeforeDispatch,
    DispatchedOutcomeUnknown,
    PartiallyCompleted,
    DuplicatedExternally,
    RolledBackExternally,
    Irreversible,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CredentialOwnerClass {
    MockInert,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FoundationError {
    InvalidDescriptor,
    InvalidReference,
    UnknownBinding,
    UnknownProposal,
    UnknownConfirmation,
    InvalidTransition,
    Unauthorized,
    BindingMismatch,
    ScopeMismatch,
    Expired,
    Revoked,
    Quarantined,
    Consumed,
    Cancelled,
    DescriptorChanged,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ConnectorDescriptor {
    id: String,
    schema_version: u16,
    version: u32,
    name: String,
    description: String,
    class: ConnectorClass,
    operations: BTreeSet<OperationClass>,
    scopes: BTreeSet<Scope>,
    mock_only: bool,
    sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CredentialReference {
    id: String,
    owner: CredentialOwnerClass,
    descriptor_id: String,
    descriptor_sha256: String,
    provider_label: String,
    account_ref: String,
    project_id: String,
    scopes: BTreeSet<Scope>,
    issued_at_ms: u64,
    expires_at_ms: Option<u64>,
    revoked: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AuthorityBinding {
    id: String,
    descriptor_id: String,
    descriptor_sha256: String,
    project_id: String,
    account_ref: String,
    scopes: BTreeSet<Scope>,
    credential_reference_id: Option<String>,
    state: LifecycleState,
    created_at_ms: u64,
    version: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OperationProposal {
    id: String,
    binding_id: String,
    descriptor_id: String,
    descriptor_sha256: String,
    project_id: String,
    account_ref: String,
    scopes: BTreeSet<Scope>,
    operation: OperationClass,
    target_id: String,
    payload_sha256: String,
    created_at_ms: u64,
    expires_at_ms: u64,
    version: u16,
    binding_sha256: String,
    cancelled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MutationConfirmation {
    id: String,
    proposal_id: String,
    binding_sha256: String,
    expires_at_ms: u64,
    consumed: bool,
    cancelled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AuditRecord {
    descriptor_id: String,
    descriptor_sha256: String,
    class: ConnectorClass,
    project_id: String,
    account_ref: String,
    operation: OperationClass,
    requested_scopes: BTreeSet<Scope>,
    effective_scopes: BTreeSet<Scope>,
    credential_reference_id: Option<String>,
    proposal_id: String,
    confirmation_id: Option<String>,
    created_at_ms: u64,
    dispatch_at_ms: Option<u64>,
    completion_at_ms: Option<u64>,
    mock_object_id: String,
    mock_revision_sha256: String,
    result: ResultState,
    correlation_id: String,
    mock_only: bool,
}

#[derive(Default)]
struct ConnectorFoundationService {
    descriptors: HashMap<String, ConnectorDescriptor>,
    bindings: HashMap<String, AuthorityBinding>,
    credentials: HashMap<String, CredentialReference>,
    proposals: HashMap<String, OperationProposal>,
    confirmations: HashMap<String, MutationConfirmation>,
    audit: Vec<AuditRecord>,
}

impl ConnectorFoundationService {
    fn with_static_mock_descriptor() -> Self {
        let descriptor = static_mock_descriptor();
        let mut service = Self::default();
        service
            .descriptors
            .insert(descriptor.id.clone(), descriptor);
        service
    }

    fn descriptor(&self) -> &ConnectorDescriptor {
        self.descriptors
            .get("019a57c0-0000-7000-8000-000000000001")
            .expect("static descriptor is registered")
    }

    #[allow(clippy::too_many_arguments)]
    fn create_binding(
        &mut self,
        descriptor_id: &str,
        descriptor_sha256: &str,
        project_id: &str,
        account_ref: &str,
        scopes: BTreeSet<Scope>,
        credential: Option<CredentialReference>,
        now_ms: u64,
    ) -> Result<AuthorityBinding, FoundationError> {
        let descriptor = self
            .descriptors
            .get(descriptor_id)
            .ok_or(FoundationError::InvalidDescriptor)?;
        if !valid_descriptor(descriptor)
            || descriptor.sha256 != descriptor_sha256
            || !valid_uuid_v7(project_id)
            || !valid_uuid_v7(account_ref)
            || scopes.is_empty()
            || !scopes.is_subset(&descriptor.scopes)
        {
            return Err(FoundationError::InvalidReference);
        }
        let credential_reference_id = match credential {
            Some(reference) => {
                if !valid_credential_reference(
                    &reference,
                    descriptor,
                    project_id,
                    account_ref,
                    &scopes,
                    now_ms,
                ) {
                    return Err(FoundationError::InvalidReference);
                }
                let id = reference.id.clone();
                self.credentials.insert(id.clone(), reference);
                Some(id)
            }
            None => None,
        };
        let binding = AuthorityBinding {
            id: Uuid::now_v7().to_string(),
            descriptor_id: descriptor.id.clone(),
            descriptor_sha256: descriptor.sha256.clone(),
            project_id: project_id.to_owned(),
            account_ref: account_ref.to_owned(),
            scopes,
            credential_reference_id,
            state: LifecycleState::AuthorizedMetadata,
            created_at_ms: now_ms,
            version: 1,
        };
        self.bindings.insert(binding.id.clone(), binding.clone());
        Ok(binding)
    }

    fn transition(
        &mut self,
        binding_id: &str,
        next: LifecycleState,
    ) -> Result<(), FoundationError> {
        let binding = self
            .bindings
            .get_mut(binding_id)
            .ok_or(FoundationError::UnknownBinding)?;
        if !allowed_transition(binding.state, next) {
            return Err(FoundationError::InvalidTransition);
        }
        binding.state = next;
        binding.version = binding
            .version
            .checked_add(1)
            .ok_or(FoundationError::InvalidTransition)?;
        Ok(())
    }

    fn revoke(&mut self, binding_id: &str) -> Result<(), FoundationError> {
        self.transition(binding_id, LifecycleState::Revoked)?;
        for proposal in self.proposals.values_mut() {
            if proposal.binding_id == binding_id {
                proposal.cancelled = true;
            }
        }
        for confirmation in self.confirmations.values_mut() {
            if self
                .proposals
                .get(&confirmation.proposal_id)
                .is_some_and(|proposal| proposal.binding_id == binding_id)
            {
                confirmation.cancelled = true;
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn propose_mock_mutation(
        &mut self,
        binding_id: &str,
        project_id: &str,
        account_ref: &str,
        scopes: BTreeSet<Scope>,
        target_id: &str,
        payload: &str,
        now_ms: u64,
    ) -> Result<OperationProposal, FoundationError> {
        if !valid_uuid_v7(binding_id)
            || !valid_uuid_v7(project_id)
            || !valid_uuid_v7(account_ref)
            || !valid_target(target_id)
            || payload.is_empty()
            || payload.len() > 1024
        {
            return Err(FoundationError::InvalidReference);
        }
        let binding = self
            .bindings
            .get(binding_id)
            .cloned()
            .ok_or(FoundationError::UnknownBinding)?;
        self.require_active_binding(
            &binding,
            project_id,
            account_ref,
            &scopes,
            Scope::MockMutation,
            now_ms,
        )?;
        let descriptor = self
            .descriptors
            .get(&binding.descriptor_id)
            .ok_or(FoundationError::InvalidDescriptor)?;
        if descriptor.sha256 != binding.descriptor_sha256
            || !descriptor
                .operations
                .contains(&OperationClass::ProposeMutation)
        {
            return Err(FoundationError::DescriptorChanged);
        }
        let proposal = OperationProposal {
            id: Uuid::now_v7().to_string(),
            binding_id: binding.id.clone(),
            descriptor_id: binding.descriptor_id.clone(),
            descriptor_sha256: binding.descriptor_sha256.clone(),
            project_id: project_id.to_owned(),
            account_ref: account_ref.to_owned(),
            scopes,
            operation: OperationClass::ProposeMutation,
            target_id: target_id.to_owned(),
            payload_sha256: digest(payload),
            created_at_ms: now_ms,
            expires_at_ms: now_ms + PROPOSAL_TTL_MS,
            version: 1,
            binding_sha256: binding_digest(&binding),
            cancelled: false,
        };
        self.transition(binding_id, LifecycleState::PendingMutationConfirmation)?;
        self.proposals.insert(proposal.id.clone(), proposal.clone());
        Ok(proposal)
    }

    fn issue_confirmation(
        &mut self,
        proposal_id: &str,
        now_ms: u64,
    ) -> Result<MutationConfirmation, FoundationError> {
        let proposal = self
            .proposals
            .get(proposal_id)
            .cloned()
            .ok_or(FoundationError::UnknownProposal)?;
        self.require_live_proposal(&proposal, now_ms)?;
        let confirmation = MutationConfirmation {
            id: Uuid::now_v7().to_string(),
            proposal_id: proposal.id.clone(),
            binding_sha256: proposal.binding_sha256.clone(),
            expires_at_ms: proposal.expires_at_ms,
            consumed: false,
            cancelled: false,
        };
        self.confirmations
            .insert(confirmation.id.clone(), confirmation.clone());
        Ok(confirmation)
    }

    fn cancel_confirmation(&mut self, confirmation_id: &str) -> Result<(), FoundationError> {
        let confirmation = self
            .confirmations
            .get_mut(confirmation_id)
            .ok_or(FoundationError::UnknownConfirmation)?;
        if confirmation.consumed {
            return Err(FoundationError::Consumed);
        }
        confirmation.cancelled = true;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn dispatch_mock_mutation(
        &mut self,
        confirmation_id: &str,
        binding_id: &str,
        project_id: &str,
        account_ref: &str,
        scopes: BTreeSet<Scope>,
        target_id: &str,
        payload: &str,
        outcome: ResultState,
        now_ms: u64,
    ) -> Result<AuditRecord, FoundationError> {
        let confirmation = self
            .confirmations
            .get(confirmation_id)
            .cloned()
            .ok_or(FoundationError::UnknownConfirmation)?;
        if confirmation.consumed {
            return Err(FoundationError::Consumed);
        }
        if confirmation.cancelled {
            return Err(FoundationError::Cancelled);
        }
        if now_ms >= confirmation.expires_at_ms {
            return Err(FoundationError::Expired);
        }
        let proposal = self
            .proposals
            .get(&confirmation.proposal_id)
            .cloned()
            .ok_or(FoundationError::UnknownProposal)?;
        self.require_live_proposal(&proposal, now_ms)?;
        if proposal.binding_id != binding_id
            || proposal.project_id != project_id
            || proposal.account_ref != account_ref
            || proposal.scopes != scopes
            || proposal.target_id != target_id
            || proposal.payload_sha256 != digest(payload)
            || proposal.binding_sha256 != confirmation.binding_sha256
        {
            return Err(FoundationError::BindingMismatch);
        }
        let binding = self
            .bindings
            .get(binding_id)
            .cloned()
            .ok_or(FoundationError::UnknownBinding)?;
        self.require_active_binding(
            &binding,
            project_id,
            account_ref,
            &scopes,
            Scope::MockMutation,
            now_ms,
        )?;
        if binding_digest(&binding) != proposal.binding_sha256 {
            return Err(FoundationError::DescriptorChanged);
        }
        let descriptor = self
            .descriptors
            .get(&binding.descriptor_id)
            .ok_or(FoundationError::InvalidDescriptor)?;
        if descriptor.sha256 != proposal.descriptor_sha256
            || !descriptor
                .operations
                .contains(&OperationClass::ExecuteConfirmedMockMutation)
        {
            return Err(FoundationError::DescriptorChanged);
        }
        let descriptor_class = descriptor.class;
        self.transition(binding_id, LifecycleState::ConfirmedNotDispatched)?;
        self.transition(binding_id, LifecycleState::Dispatched)?;
        let confirmation = self
            .confirmations
            .get_mut(confirmation_id)
            .ok_or(FoundationError::UnknownConfirmation)?;
        confirmation.consumed = true;
        let audit = AuditRecord {
            descriptor_id: proposal.descriptor_id.clone(),
            descriptor_sha256: proposal.descriptor_sha256.clone(),
            class: descriptor_class,
            project_id: proposal.project_id.clone(),
            account_ref: proposal.account_ref.clone(),
            operation: OperationClass::ExecuteConfirmedMockMutation,
            requested_scopes: proposal.scopes.clone(),
            effective_scopes: proposal.scopes.clone(),
            credential_reference_id: binding.credential_reference_id.clone(),
            proposal_id: proposal.id,
            confirmation_id: Some(confirmation_id.to_owned()),
            created_at_ms: proposal.created_at_ms,
            dispatch_at_ms: Some(now_ms),
            completion_at_ms: Some(now_ms),
            mock_object_id: target_id.to_owned(),
            mock_revision_sha256: digest("local-mock-revision-v1"),
            result: outcome,
            correlation_id: Uuid::now_v7().to_string(),
            mock_only: true,
        };
        if outcome != ResultState::DispatchedOutcomeUnknown {
            self.transition(binding_id, LifecycleState::Completed)?;
        }
        self.audit.push(audit.clone());
        Ok(audit)
    }

    fn require_live_proposal(
        &self,
        proposal: &OperationProposal,
        now_ms: u64,
    ) -> Result<(), FoundationError> {
        if proposal.cancelled {
            return Err(FoundationError::Cancelled);
        }
        if now_ms >= proposal.expires_at_ms {
            return Err(FoundationError::Expired);
        }
        let binding = self
            .bindings
            .get(&proposal.binding_id)
            .ok_or(FoundationError::UnknownBinding)?;
        if binding.state == LifecycleState::Revoked {
            return Err(FoundationError::Revoked);
        }
        if binding.state == LifecycleState::Quarantined {
            return Err(FoundationError::Quarantined);
        }
        Ok(())
    }

    fn require_active_binding(
        &self,
        binding: &AuthorityBinding,
        project_id: &str,
        account_ref: &str,
        scopes: &BTreeSet<Scope>,
        required_scope: Scope,
        now_ms: u64,
    ) -> Result<(), FoundationError> {
        if binding.project_id != project_id || binding.account_ref != account_ref {
            return Err(FoundationError::BindingMismatch);
        }
        if !scopes.is_subset(&binding.scopes) || !binding.scopes.contains(&required_scope) {
            return Err(FoundationError::ScopeMismatch);
        }
        match binding.state {
            LifecycleState::Revoked => return Err(FoundationError::Revoked),
            LifecycleState::Quarantined => return Err(FoundationError::Quarantined),
            LifecycleState::Expired | LifecycleState::Removed | LifecycleState::Degraded => {
                return Err(FoundationError::Unauthorized)
            }
            _ => {}
        }
        if let Some(id) = &binding.credential_reference_id {
            let credential = self
                .credentials
                .get(id)
                .ok_or(FoundationError::InvalidReference)?;
            let descriptor = self
                .descriptors
                .get(&binding.descriptor_id)
                .ok_or(FoundationError::InvalidDescriptor)?;
            if !valid_credential_reference(
                credential,
                descriptor,
                project_id,
                account_ref,
                scopes,
                now_ms,
            ) {
                return Err(FoundationError::Unauthorized);
            }
        }
        Ok(())
    }
}

fn static_mock_descriptor() -> ConnectorDescriptor {
    let mut descriptor = ConnectorDescriptor {
        id: "019a57c0-0000-7000-8000-000000000001".to_owned(),
        schema_version: SCHEMA_VERSION,
        version: 1,
        name: "Local mock connector".to_owned(),
        description: "Deterministic local contract fixture; no provider or network access."
            .to_owned(),
        class: ConnectorClass::LocalMock,
        operations: BTreeSet::from([
            OperationClass::DiscoverAvailability,
            OperationClass::ListAuthorizedAccounts,
            OperationClass::ReadMetadata,
            OperationClass::Search,
            OperationClass::FetchContent,
            OperationClass::FetchAttachment,
            OperationClass::ProposeMutation,
            OperationClass::ConfirmMutation,
            OperationClass::ExecuteConfirmedMockMutation,
            OperationClass::ReportResult,
            OperationClass::ReconcileAmbiguousMockResult,
            OperationClass::RevokeOrDisconnect,
        ]),
        scopes: BTreeSet::from([
            Scope::Metadata,
            Scope::Read,
            Scope::SearchFetch,
            Scope::MockMutation,
        ]),
        mock_only: true,
        sha256: String::new(),
    };
    descriptor.sha256 = descriptor_digest(&descriptor).expect("static descriptor valid");
    descriptor
}

fn valid_descriptor(descriptor: &ConnectorDescriptor) -> bool {
    descriptor.schema_version == SCHEMA_VERSION
        && valid_uuid_v7(&descriptor.id)
        && descriptor.version > 0
        && valid_label(&descriptor.name, 80)
        && valid_label(&descriptor.description, 240)
        && descriptor.class == ConnectorClass::LocalMock
        && descriptor.mock_only
        && !descriptor.operations.is_empty()
        && !descriptor.scopes.is_empty()
        && descriptor_digest(descriptor).is_some_and(|digest| digest == descriptor.sha256)
}

fn valid_credential_reference(
    reference: &CredentialReference,
    descriptor: &ConnectorDescriptor,
    project_id: &str,
    account_ref: &str,
    scopes: &BTreeSet<Scope>,
    now_ms: u64,
) -> bool {
    valid_uuid_v7(&reference.id)
        && reference.owner == CredentialOwnerClass::MockInert
        && reference.descriptor_id == descriptor.id
        && reference.descriptor_sha256 == descriptor.sha256
        && reference.provider_label == "local-mock"
        && reference.account_ref == account_ref
        && reference.project_id == project_id
        && !reference.scopes.is_empty()
        && scopes.is_subset(&reference.scopes)
        && !reference.revoked
        && reference.issued_at_ms <= now_ms
        && reference
            .expires_at_ms
            .is_none_or(|expires| now_ms < expires)
}

fn allowed_transition(from: LifecycleState, to: LifecycleState) -> bool {
    use LifecycleState::*;
    matches!(
        (from, to),
        (KnownUnavailable, AvailableDisconnected)
            | (AvailableDisconnected, ConnectedUnauthorized)
            | (ConnectedUnauthorized, AuthorizedMetadata)
            | (
                AuthorizedMetadata,
                AuthorizedBoundedRead
                    | PendingMutationConfirmation
                    | Revoked
                    | Quarantined
                    | Degraded
                    | Removed
            )
            | (
                AuthorizedBoundedRead,
                AuthorizedSearchFetch
                    | PendingMutationConfirmation
                    | Revoked
                    | Quarantined
                    | Degraded
                    | Removed
            )
            | (
                AuthorizedSearchFetch,
                PendingMutationConfirmation | Revoked | Quarantined | Degraded | Removed
            )
            | (
                PendingMutationConfirmation,
                ConfirmedNotDispatched | Expired | Revoked | Quarantined
            )
            | (
                ConfirmedNotDispatched,
                Dispatched | Expired | Revoked | Quarantined
            )
            | (Dispatched, Completed | Degraded | Quarantined)
            | (
                Completed,
                AuthorizedMetadata | Revoked | Quarantined | Removed
            )
            | (Expired, AuthorizedMetadata | Removed)
            | (Revoked, Removed)
            | (Degraded, AuthorizedMetadata | Quarantined | Removed)
            | (Quarantined, Removed)
    )
}

fn descriptor_digest(descriptor: &ConnectorDescriptor) -> Option<String> {
    if descriptor.operations.is_empty() || descriptor.scopes.is_empty() {
        return None;
    }
    let operations = descriptor
        .operations
        .iter()
        .map(operation_name)
        .collect::<Vec<_>>()
        .join(",");
    let scopes = descriptor
        .scopes
        .iter()
        .map(scope_name)
        .collect::<Vec<_>>()
        .join(",");
    Some(digest(&format!(
        "connector-descriptor-v1\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        descriptor.id,
        descriptor.schema_version,
        descriptor.version,
        descriptor.name,
        descriptor.description,
        "local-mock",
        operations,
        scopes
    )))
}

fn binding_digest(binding: &AuthorityBinding) -> String {
    let scopes = binding
        .scopes
        .iter()
        .map(scope_name)
        .collect::<Vec<_>>()
        .join(",");
    digest(&format!(
        "connector-binding-v1\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        binding.id,
        binding.descriptor_id,
        binding.descriptor_sha256,
        binding.project_id,
        binding.account_ref,
        scopes,
        binding.credential_reference_id.as_deref().unwrap_or(""),
    ))
}

fn operation_name(operation: &OperationClass) -> &'static str {
    match operation {
        OperationClass::DiscoverAvailability => "discover-availability",
        OperationClass::ListAuthorizedAccounts => "list-authorized-accounts",
        OperationClass::ReadMetadata => "read-metadata",
        OperationClass::Search => "search",
        OperationClass::FetchContent => "fetch-content",
        OperationClass::FetchAttachment => "fetch-attachment",
        OperationClass::ProposeMutation => "propose-mutation",
        OperationClass::ConfirmMutation => "confirm-mutation",
        OperationClass::ExecuteConfirmedMockMutation => "execute-confirmed-mock-mutation",
        OperationClass::ReportResult => "report-result",
        OperationClass::ReconcileAmbiguousMockResult => "reconcile-ambiguous-mock-result",
        OperationClass::RevokeOrDisconnect => "revoke-or-disconnect",
    }
}

fn scope_name(scope: &Scope) -> &'static str {
    match scope {
        Scope::Metadata => "metadata",
        Scope::Read => "read",
        Scope::SearchFetch => "search-fetch",
        Scope::MockMutation => "mock-mutation",
    }
}

fn valid_uuid_v7(value: &str) -> bool {
    Uuid::parse_str(value).is_ok_and(|id| id.get_version_num() == 7)
}

fn valid_label(value: &str, limit: usize) -> bool {
    !value.is_empty()
        && value.len() <= limit
        && !value.chars().any(|character| character.is_control())
}

fn valid_target(value: &str) -> bool {
    value.starts_with("mock-object-")
        && value.len() <= 80
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn digest(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id() -> String {
        Uuid::now_v7().to_string()
    }
    fn scopes() -> BTreeSet<Scope> {
        BTreeSet::from([Scope::Metadata, Scope::MockMutation])
    }
    fn make_binding(service: &mut ConnectorFoundationService, now_ms: u64) -> AuthorityBinding {
        let descriptor = service.descriptor().clone();
        service
            .create_binding(
                &descriptor.id,
                &descriptor.sha256,
                &id(),
                &id(),
                scopes(),
                None,
                now_ms,
            )
            .unwrap()
    }

    #[test]
    fn static_descriptor_is_digest_bound_non_executable_and_closed() {
        let descriptor = static_mock_descriptor();
        assert!(valid_descriptor(&descriptor));
        assert!(descriptor.mock_only);
        assert_eq!(descriptor.class, ConnectorClass::LocalMock);
        let mut changed = descriptor.clone();
        changed.description.push('!');
        assert_ne!(descriptor_digest(&changed), Some(descriptor.sha256));
        changed.operations.clear();
        assert!(!valid_descriptor(&changed));
    }

    #[test]
    fn bindings_require_exact_project_account_scope_and_inert_credentials() {
        let mut service = ConnectorFoundationService::with_static_mock_descriptor();
        let descriptor = service.descriptor().clone();
        let project = id();
        let account = id();
        let credential = CredentialReference {
            id: id(),
            owner: CredentialOwnerClass::MockInert,
            descriptor_id: descriptor.id.clone(),
            descriptor_sha256: descriptor.sha256.clone(),
            provider_label: "local-mock".into(),
            account_ref: account.clone(),
            project_id: project.clone(),
            scopes: scopes(),
            issued_at_ms: 10,
            expires_at_ms: Some(100),
            revoked: false,
        };
        let binding = service
            .create_binding(
                &descriptor.id,
                &descriptor.sha256,
                &project,
                &account,
                scopes(),
                Some(credential),
                11,
            )
            .unwrap();
        assert_eq!(binding.state, LifecycleState::AuthorizedMetadata);
        assert!(service
            .require_active_binding(
                &binding,
                &id(),
                &account,
                &scopes(),
                Scope::MockMutation,
                12
            )
            .is_err());
        assert!(service
            .require_active_binding(
                &binding,
                &project,
                &id(),
                &scopes(),
                Scope::MockMutation,
                12
            )
            .is_err());
        assert!(service
            .require_active_binding(
                &binding,
                &project,
                &account,
                &BTreeSet::from([Scope::MockMutation, Scope::Read]),
                Scope::MockMutation,
                12
            )
            .is_err());
        assert!(service
            .require_active_binding(
                &binding,
                &project,
                &account,
                &scopes(),
                Scope::MockMutation,
                100
            )
            .is_err());
    }

    #[test]
    fn lifecycle_is_fail_closed_and_revocation_cancels_pending_authority() {
        let mut service = ConnectorFoundationService::with_static_mock_descriptor();
        let binding = make_binding(&mut service, 10);
        assert_eq!(
            service.transition(&binding.id, LifecycleState::Dispatched),
            Err(FoundationError::InvalidTransition)
        );
        let proposal = service
            .propose_mock_mutation(
                &binding.id,
                &binding.project_id,
                &binding.account_ref,
                binding.scopes.clone(),
                "mock-object-alpha",
                "draft",
                11,
            )
            .unwrap();
        let confirmation = service.issue_confirmation(&proposal.id, 12).unwrap();
        service.revoke(&binding.id).unwrap();
        assert_eq!(
            service.issue_confirmation(&proposal.id, 13),
            Err(FoundationError::Cancelled)
        );
        assert_eq!(
            service.dispatch_mock_mutation(
                &confirmation.id,
                &binding.id,
                &binding.project_id,
                &binding.account_ref,
                binding.scopes.clone(),
                "mock-object-alpha",
                "draft",
                ResultState::Succeeded,
                13
            ),
            Err(FoundationError::Cancelled)
        );
    }

    #[test]
    fn proposal_confirmation_is_digest_bound_one_time_and_cancellable() {
        let mut service = ConnectorFoundationService::with_static_mock_descriptor();
        let binding = make_binding(&mut service, 10);
        let proposal = service
            .propose_mock_mutation(
                &binding.id,
                &binding.project_id,
                &binding.account_ref,
                binding.scopes.clone(),
                "mock-object-alpha",
                "draft",
                11,
            )
            .unwrap();
        let confirmation = service.issue_confirmation(&proposal.id, 12).unwrap();
        assert_eq!(
            service.dispatch_mock_mutation(
                &confirmation.id,
                &binding.id,
                &binding.project_id,
                &binding.account_ref,
                binding.scopes.clone(),
                "mock-object-beta",
                "draft",
                ResultState::Succeeded,
                13
            ),
            Err(FoundationError::BindingMismatch)
        );
        let audit = service
            .dispatch_mock_mutation(
                &confirmation.id,
                &binding.id,
                &binding.project_id,
                &binding.account_ref,
                binding.scopes.clone(),
                "mock-object-alpha",
                "draft",
                ResultState::Succeeded,
                13,
            )
            .unwrap();
        assert!(audit.mock_only);
        assert_eq!(
            service.dispatch_mock_mutation(
                &confirmation.id,
                &binding.id,
                &binding.project_id,
                &binding.account_ref,
                binding.scopes.clone(),
                "mock-object-alpha",
                "draft",
                ResultState::Succeeded,
                14
            ),
            Err(FoundationError::Consumed)
        );

        let mut cancellation_service = ConnectorFoundationService::with_static_mock_descriptor();
        let binding = make_binding(&mut cancellation_service, 10);
        let proposal = cancellation_service
            .propose_mock_mutation(
                &binding.id,
                &binding.project_id,
                &binding.account_ref,
                binding.scopes.clone(),
                "mock-object-alpha",
                "draft",
                11,
            )
            .unwrap();
        let confirmation = cancellation_service
            .issue_confirmation(&proposal.id, 12)
            .unwrap();
        cancellation_service
            .cancel_confirmation(&confirmation.id)
            .unwrap();
        assert_eq!(
            cancellation_service.dispatch_mock_mutation(
                &confirmation.id,
                &binding.id,
                &binding.project_id,
                &binding.account_ref,
                binding.scopes.clone(),
                "mock-object-alpha",
                "draft",
                ResultState::Succeeded,
                13
            ),
            Err(FoundationError::Cancelled)
        );
    }

    #[test]
    fn stale_quarantined_and_descriptor_changed_confirmations_fail_closed() {
        let mut service = ConnectorFoundationService::with_static_mock_descriptor();
        let binding = make_binding(&mut service, 10);
        let proposal = service
            .propose_mock_mutation(
                &binding.id,
                &binding.project_id,
                &binding.account_ref,
                binding.scopes.clone(),
                "mock-object-alpha",
                "draft",
                11,
            )
            .unwrap();
        let confirmation = service.issue_confirmation(&proposal.id, 12).unwrap();
        assert_eq!(
            service.dispatch_mock_mutation(
                &confirmation.id,
                &binding.id,
                &binding.project_id,
                &binding.account_ref,
                binding.scopes.clone(),
                "mock-object-alpha",
                "draft",
                ResultState::Succeeded,
                proposal.expires_at_ms
            ),
            Err(FoundationError::Expired)
        );

        let mut quarantined = ConnectorFoundationService::with_static_mock_descriptor();
        let binding = make_binding(&mut quarantined, 10);
        quarantined
            .transition(&binding.id, LifecycleState::Quarantined)
            .unwrap();
        assert_eq!(
            quarantined.propose_mock_mutation(
                &binding.id,
                &binding.project_id,
                &binding.account_ref,
                binding.scopes.clone(),
                "mock-object-alpha",
                "draft",
                11
            ),
            Err(FoundationError::Quarantined)
        );

        let mut changed = ConnectorFoundationService::with_static_mock_descriptor();
        let binding = make_binding(&mut changed, 10);
        let proposal = changed
            .propose_mock_mutation(
                &binding.id,
                &binding.project_id,
                &binding.account_ref,
                binding.scopes.clone(),
                "mock-object-alpha",
                "draft",
                11,
            )
            .unwrap();
        let confirmation = changed.issue_confirmation(&proposal.id, 12).unwrap();
        changed
            .descriptors
            .get_mut(&binding.descriptor_id)
            .unwrap()
            .sha256 = "0".repeat(64);
        assert_eq!(
            changed.dispatch_mock_mutation(
                &confirmation.id,
                &binding.id,
                &binding.project_id,
                &binding.account_ref,
                binding.scopes.clone(),
                "mock-object-alpha",
                "draft",
                ResultState::Succeeded,
                13
            ),
            Err(FoundationError::DescriptorChanged)
        );
    }

    #[test]
    fn mock_outcomes_are_content_free_and_ambiguous_outcomes_do_not_retry() {
        let mut service = ConnectorFoundationService::with_static_mock_descriptor();
        let binding = make_binding(&mut service, 10);
        let proposal = service
            .propose_mock_mutation(
                &binding.id,
                &binding.project_id,
                &binding.account_ref,
                binding.scopes.clone(),
                "mock-object-alpha",
                "draft",
                11,
            )
            .unwrap();
        let confirmation = service.issue_confirmation(&proposal.id, 12).unwrap();
        let audit = service
            .dispatch_mock_mutation(
                &confirmation.id,
                &binding.id,
                &binding.project_id,
                &binding.account_ref,
                binding.scopes.clone(),
                "mock-object-alpha",
                "draft",
                ResultState::DispatchedOutcomeUnknown,
                13,
            )
            .unwrap();
        assert_eq!(audit.result, ResultState::DispatchedOutcomeUnknown);
        assert!(audit.mock_only);
        assert_eq!(service.audit.len(), 1);
        assert_eq!(
            service.dispatch_mock_mutation(
                &confirmation.id,
                &binding.id,
                &binding.project_id,
                &binding.account_ref,
                binding.scopes.clone(),
                "mock-object-alpha",
                "draft",
                ResultState::DuplicatedExternally,
                14
            ),
            Err(FoundationError::Consumed)
        );
        for state in [
            ResultState::Rejected,
            ResultState::Cancelled,
            ResultState::TimedOutBeforeDispatch,
            ResultState::PartiallyCompleted,
            ResultState::DuplicatedExternally,
            ResultState::RolledBackExternally,
            ResultState::Irreversible,
        ] {
            assert_ne!(state, ResultState::Succeeded);
        }
    }
}
