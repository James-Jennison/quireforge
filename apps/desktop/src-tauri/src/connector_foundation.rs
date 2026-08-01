//! Local-only connector authority contracts.
//!
//! This module intentionally has no transport, provider, browser, process,
//! environment, persistence, or frontend dependency. Its deterministic mock
//! adapter exists solely to make the closed authority model testable.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashMap};
use std::sync::Mutex;
use uuid::Uuid;

const SCHEMA_VERSION: u16 = 1;
const PROPOSAL_TTL_MS: u64 = 5 * 60 * 1000;

/// Closed, local-only Tauri façade for the ratified M57 fixture. It deliberately
/// owns no transport, credential, process, browser, MCP, or provider path.
pub(crate) struct ConnectorGovernanceService {
    inner: Mutex<ConnectorFoundationService>,
    sessions: Mutex<HashMap<String, GovernanceSession>>,
}

impl Default for ConnectorGovernanceService {
    fn default() -> Self {
        Self {
            inner: Mutex::new(ConnectorFoundationService::with_static_mock_descriptor()),
            sessions: Mutex::new(HashMap::new()),
        }
    }
}

#[derive(Clone)]
struct GovernanceSession {
    binding_id: String,
    project_id: String,
    task_id: String,
    account_ref: String,
    scopes: BTreeSet<Scope>,
    target: String,
    operation: String,
    authorization_id: String,
    descriptor_id: String,
    descriptor_version: u32,
    descriptor_sha256: String,
    scope_digest: String,
    request_digest: String,
    expires_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ConnectorPrepareRequest {
    pub task_id: String,
    pub operation: String,
    pub target: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ConnectorConfirmRequest {
    pub task_id: String,
    pub operation_id: String,
    pub authorization_id: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ConnectorCancelRequest {
    pub task_id: String,
    pub authorization_id: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ConnectorOperationRequest {
    pub task_id: String,
    pub operation_id: String,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConnectorSnapshot {
    pub schema_version: u16,
    pub fictional_local_only: bool,
    pub state: String,
    pub project_id: Option<String>,
    pub task_id: Option<String>,
    pub operation_id: Option<String>,
    pub authorization_id: Option<String>,
    pub operation: Option<String>,
    pub diagnostic: Option<String>,
    pub binding_id: Option<String>,
    pub descriptor_id: Option<String>,
    pub descriptor_version: Option<u32>,
    pub descriptor_sha256: Option<String>,
    pub scope_digest: Option<String>,
    pub request_digest: Option<String>,
    pub expires_at_ms: Option<u64>,
    pub declared_capabilities: Vec<String>,
    pub granted_authority: Vec<String>,
    pub audit_state: String,
}

impl ConnectorGovernanceService {
    pub(crate) fn catalog(&self) -> ConnectorSnapshot {
        let service = self.inner.lock().expect("connector fixture lock");
        let descriptor = service.descriptor();
        ConnectorSnapshot {
            schema_version: 1,
            fictional_local_only: true,
            state: "ready".into(),
            project_id: None,
            task_id: None,
            operation_id: None,
            authorization_id: None,
            operation: None,
            diagnostic: None,
            binding_id: None,
            descriptor_id: Some(descriptor.id.clone()),
            descriptor_version: Some(descriptor.version),
            descriptor_sha256: Some(descriptor.sha256.clone()),
            scope_digest: None,
            request_digest: None,
            expires_at_ms: None,
            declared_capabilities: vec!["read".into(), "mutation".into()],
            granted_authority: Vec::new(),
            audit_state: format!(
                "fictional descriptor v{} ({})",
                descriptor.version,
                &descriptor.sha256[..12]
            ),
        }
    }
    pub(crate) fn prepare(
        &self,
        request: ConnectorPrepareRequest,
        project_id: String,
    ) -> ConnectorSnapshot {
        let now = now_millis();
        if !valid_uuid_v7(&project_id)
            || !valid_uuid_v7(&request.task_id)
            || !valid_target(&request.target)
        {
            return unavailable("rejected", "invalid-request");
        }
        let mut service = self.inner.lock().expect("connector fixture lock");
        let descriptor = service.descriptor().clone();
        let account = Uuid::now_v7().to_string();
        let scopes = if request.operation == "read" {
            BTreeSet::from([Scope::Read])
        } else if request.operation == "mutation" {
            BTreeSet::from([Scope::MockMutation])
        } else {
            return unavailable("rejected", "unsupported-operation");
        };
        let binding = match service.create_binding(
            &descriptor.id,
            &descriptor.sha256,
            &project_id,
            &account,
            scopes.clone(),
            None,
            now,
        ) {
            Ok(binding) => binding,
            Err(_) => return unavailable("rejected", "binding-unavailable"),
        };
        let scope_digest = digest(&format!("{:?}", scopes));
        let request_digest = digest(&format!(
            "{}:{}:{}:{}",
            descriptor.sha256, project_id, request.operation, request.target
        ));
        if request.operation == "read" {
            let _ = service.transition(&binding.id, LifecycleState::AuthorizedBoundedRead);
            return ConnectorSnapshot {
                schema_version: 1,
                fictional_local_only: true,
                state: "succeeded".into(),
                project_id: Some(project_id),
                task_id: Some(request.task_id),
                operation_id: Some(Uuid::now_v7().to_string()),
                authorization_id: None,
                operation: Some("read".into()),
                diagnostic: None,
                binding_id: Some(binding.id),
                descriptor_id: Some(descriptor.id),
                descriptor_version: Some(descriptor.version),
                descriptor_sha256: Some(descriptor.sha256),
                scope_digest: Some(scope_digest),
                request_digest: Some(request_digest),
                expires_at_ms: Some(now + PROPOSAL_TTL_MS),
                declared_capabilities: vec!["read".into(), "mutation".into()],
                granted_authority: vec!["read".into()],
                audit_state: "fictional local read completed; no network or external effect".into(),
            };
        }
        let proposal = match service.propose_mock_mutation(
            &binding.id,
            &project_id,
            &account,
            scopes.clone(),
            &request.target,
            "fictional-local-mutation",
            now,
        ) {
            Ok(proposal) => proposal,
            Err(_) => return unavailable("rejected", "mutation-not-granted"),
        };
        let confirmation = match service.issue_confirmation(&proposal.id, now) {
            Ok(value) => value,
            Err(_) => return unavailable("rejected", "authorization-unavailable"),
        };
        let session = GovernanceSession {
            binding_id: binding.id.clone(),
            project_id: project_id.clone(),
            task_id: request.task_id.clone(),
            account_ref: account,
            scopes,
            target: request.target,
            operation: "mutation".into(),
            authorization_id: confirmation.id.clone(),
            descriptor_id: descriptor.id.clone(),
            descriptor_version: descriptor.version,
            descriptor_sha256: descriptor.sha256.clone(),
            scope_digest: scope_digest.clone(),
            request_digest: request_digest.clone(),
            expires_at_ms: proposal.expires_at_ms,
        };
        self.sessions
            .lock()
            .expect("connector sessions lock")
            .insert(proposal.id.clone(), session);
        ConnectorSnapshot {
            schema_version: 1,
            fictional_local_only: true,
            state: "prepared".into(),
            project_id: Some(project_id),
            task_id: Some(request.task_id),
            operation_id: Some(proposal.id),
            authorization_id: Some(confirmation.id),
            operation: Some("mutation".into()),
            diagnostic: None,
            binding_id: Some(binding.id),
            descriptor_id: Some(descriptor.id),
            descriptor_version: Some(descriptor.version),
            descriptor_sha256: Some(descriptor.sha256),
            scope_digest: Some(scope_digest),
            request_digest: Some(request_digest),
            expires_at_ms: Some(proposal.expires_at_ms),
            declared_capabilities: vec!["read".into(), "mutation".into()],
            granted_authority: vec!["mutation".into()],
            audit_state: "review required; no fictional mutation dispatched".into(),
        }
    }
    pub(crate) fn confirm(&self, request: ConnectorConfirmRequest) -> ConnectorSnapshot {
        let now = now_millis();
        let Some(session) = self
            .sessions
            .lock()
            .expect("connector sessions lock")
            .get(&request.operation_id)
            .cloned()
        else {
            return unavailable("rejected", "operation-unavailable");
        };
        if session.task_id != request.task_id || session.operation != "mutation" {
            return unavailable("rejected", "task-or-operation-mismatch");
        }
        let outcome = if session.target == "mock-object-ambiguous" {
            ResultState::DispatchedOutcomeUnknown
        } else {
            ResultState::Succeeded
        };
        let mut service = self.inner.lock().expect("connector fixture lock");
        match service.dispatch_mock_mutation(
            &request.authorization_id,
            &session.binding_id,
            &session.project_id,
            &session.account_ref,
            session.scopes,
            &session.target,
            "fictional-local-mutation",
            outcome,
            now,
        ) {
            Ok(audit) => ConnectorSnapshot {
                schema_version: 1,
                fictional_local_only: true,
                state: if audit.result == ResultState::DispatchedOutcomeUnknown {
                    "outcome-unknown".into()
                } else {
                    "succeeded".into()
                },
                project_id: Some(session.project_id),
                task_id: Some(session.task_id),
                operation_id: Some(request.operation_id.clone()),
                authorization_id: Some(request.authorization_id),
                operation: Some("mutation".into()),
                diagnostic: None,
                binding_id: Some(session.binding_id),
                descriptor_id: Some(session.descriptor_id),
                descriptor_version: Some(session.descriptor_version),
                descriptor_sha256: Some(session.descriptor_sha256),
                scope_digest: Some(session.scope_digest),
                request_digest: Some(session.request_digest),
                expires_at_ms: Some(session.expires_at_ms),
                declared_capabilities: vec!["read".into(), "mutation".into()],
                granted_authority: vec!["mutation".into()],
                audit_state: if audit.result == ResultState::DispatchedOutcomeUnknown {
                    "ambiguous fictional outcome; automatic retry prohibited".into()
                } else {
                    "fictional local mutation completed; no external effect".into()
                },
            },
            Err(error) => unavailable("rejected", error_name(error)),
        }
    }
    pub(crate) fn cancel(&self, request: ConnectorCancelRequest) -> ConnectorSnapshot {
        let Some((operation_id, session)) = self
            .sessions
            .lock()
            .expect("connector sessions lock")
            .iter()
            .find(|(_, session)| session.authorization_id == request.authorization_id)
            .map(|(id, session)| (id.clone(), session.clone()))
        else {
            return unavailable("rejected", "operation-unavailable");
        };
        if session.task_id != request.task_id {
            return unavailable("rejected", "task-or-operation-mismatch");
        }
        let mut service = self.inner.lock().expect("connector fixture lock");
        match service.cancel_confirmation(&request.authorization_id) {
            Ok(()) => ConnectorSnapshot {
                schema_version: 1,
                fictional_local_only: true,
                state: "cancelled".into(),
                project_id: Some(session.project_id),
                task_id: Some(session.task_id),
                operation_id: Some(operation_id),
                authorization_id: Some(request.authorization_id),
                operation: Some("mutation".into()),
                diagnostic: None,
                binding_id: Some(session.binding_id),
                descriptor_id: Some(session.descriptor_id),
                descriptor_version: Some(session.descriptor_version),
                descriptor_sha256: Some(session.descriptor_sha256),
                scope_digest: Some(session.scope_digest),
                request_digest: Some(session.request_digest),
                expires_at_ms: Some(session.expires_at_ms),
                declared_capabilities: vec!["read".into(), "mutation".into()],
                granted_authority: Vec::new(),
                audit_state: "confirmation cancelled; no fictional mutation dispatched".into(),
            },
            Err(error) => unavailable("rejected", error_name(error)),
        }
    }
    pub(crate) fn revoke(&self, request: ConnectorOperationRequest) -> ConnectorSnapshot {
        let Some(session) = self
            .sessions
            .lock()
            .expect("connector sessions lock")
            .get(&request.operation_id)
            .cloned()
        else {
            return unavailable("rejected", "operation-unavailable");
        };
        if session.task_id != request.task_id {
            return unavailable("rejected", "task-or-operation-mismatch");
        }
        let mut service = self.inner.lock().expect("connector fixture lock");
        match service.revoke(&session.binding_id) {
            Ok(()) => ConnectorSnapshot {
                schema_version: 1,
                fictional_local_only: true,
                state: "revoked".into(),
                project_id: Some(session.project_id),
                task_id: Some(session.task_id),
                operation_id: Some(request.operation_id),
                authorization_id: None,
                operation: Some("mutation".into()),
                diagnostic: None,
                binding_id: Some(session.binding_id),
                descriptor_id: Some(session.descriptor_id),
                descriptor_version: Some(session.descriptor_version),
                descriptor_sha256: Some(session.descriptor_sha256),
                scope_digest: Some(session.scope_digest),
                request_digest: Some(session.request_digest),
                expires_at_ms: Some(session.expires_at_ms),
                declared_capabilities: vec!["read".into(), "mutation".into()],
                granted_authority: Vec::new(),
                audit_state: "fictional connector revoked; pending authorization invalidated"
                    .into(),
            },
            Err(error) => unavailable("rejected", error_name(error)),
        }
    }
    pub(crate) fn unavailable(&self) -> ConnectorSnapshot {
        unavailable("unavailable", "project-or-task-unavailable")
    }
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis() as u64)
}
fn error_name(error: FoundationError) -> &'static str {
    match error {
        FoundationError::Expired => "expired",
        FoundationError::Revoked => "revoked",
        FoundationError::Quarantined => "quarantined",
        FoundationError::Consumed => "authorization-replayed",
        FoundationError::DescriptorChanged => "descriptor-drift",
        FoundationError::Cancelled => "cancelled",
        _ => "authority-rejected",
    }
}

fn unavailable(state: &str, diagnostic: &str) -> ConnectorSnapshot {
    ConnectorSnapshot {
        schema_version: 1,
        fictional_local_only: true,
        state: state.into(),
        project_id: None,
        task_id: None,
        operation_id: None,
        authorization_id: None,
        operation: None,
        diagnostic: Some(diagnostic.into()),
        binding_id: None,
        descriptor_id: None,
        descriptor_version: None,
        descriptor_sha256: None,
        scope_digest: None,
        request_digest: None,
        expires_at_ms: None,
        declared_capabilities: vec!["read".into(), "mutation".into()],
        granted_authority: Vec::new(),
        audit_state: "no connector operation occurred".into(),
    }
}

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

    #[test]
    fn bridge_facade_separates_read_from_one_use_fictional_mutation() {
        let service = ConnectorGovernanceService::default();
        let project_id = id();
        let task_id = id();
        let read = service.prepare(
            ConnectorPrepareRequest {
                task_id: task_id.clone(),
                operation: "read".into(),
                target: "mock-object-read".into(),
            },
            project_id.clone(),
        );
        assert_eq!(read.state, "succeeded");
        assert_eq!(read.granted_authority, vec!["read"]);
        let prepared = service.prepare(
            ConnectorPrepareRequest {
                task_id: task_id.clone(),
                operation: "mutation".into(),
                target: "mock-object-ambiguous".into(),
            },
            project_id,
        );
        assert_eq!(prepared.state, "prepared");
        let confirmed = service.confirm(ConnectorConfirmRequest {
            task_id,
            operation_id: prepared.operation_id.clone().unwrap(),
            authorization_id: prepared.authorization_id.clone().unwrap(),
        });
        assert_eq!(confirmed.state, "outcome-unknown");
        let replay = service.confirm(ConnectorConfirmRequest {
            task_id: confirmed.task_id.clone().unwrap(),
            operation_id: confirmed.operation_id.clone().unwrap(),
            authorization_id: confirmed.authorization_id.clone().unwrap(),
        });
        assert_eq!(replay.state, "rejected");
        assert_eq!(replay.diagnostic.as_deref(), Some("authorization-replayed"));
    }
}
