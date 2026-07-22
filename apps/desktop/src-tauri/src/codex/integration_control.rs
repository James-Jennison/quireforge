use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use serde_json::{json, Value};
use tokio::{sync::Mutex, time::timeout};
use url::Url;
use uuid::Uuid;

use super::{
    app_server::{
        validate_protocol_identifier, AppServerCommand, AppServerNotification, AppServerProcess,
    },
    integration::{
        IntegrationAuthenticationState, IntegrationAvailability, IntegrationCatalogSnapshot,
        IntegrationControlActionRequest, IntegrationControlConfirmationRequest,
        IntegrationControlDiagnosticCode, IntegrationControlOperation,
        IntegrationControlPreviewRequest, IntegrationControlPreviewSnapshot,
        IntegrationControlPreviewState, IntegrationControlResultSnapshot,
        IntegrationControlResultState, IntegrationControlWarning, IntegrationEnablementState,
        IntegrationEntry, IntegrationEntryKind, IntegrationImplementation, IntegrationPermission,
        IntegrationPermissionAccess, IntegrationPermissionKind, IntegrationPolicyState,
        IntegrationScope, INTEGRATION_CONTROL_SCHEMA_VERSION,
    },
    integration_service::{
        known_object, normalized_entry_id, paginated_request, supports_integration_routes,
    },
    probe::probe_cli_version,
};

const MAX_CONFIRMATIONS: usize = 16;
const MAX_MENTIONS: usize = 8;
const CONFIRMATION_TTL: Duration = Duration::from_secs(5 * 60);
const ACTION_TTL: Duration = Duration::from_secs(10 * 60);
const NOTIFICATION_POLL: Duration = Duration::from_millis(2);
const CONTROL_TIMEOUT: Duration = Duration::from_secs(7);
const MAX_URL_BYTES: usize = 2048;
const MAX_SOURCE_ENTRIES: usize = 512;

pub struct IntegrationControlService {
    program: String,
    command: AppServerCommand,
    cli_version_override: Option<String>,
    state: Mutex<ControlState>,
}

#[derive(Default)]
struct ControlState {
    confirmations: HashMap<String, ConfirmationPlan>,
    pending: Option<PendingAction>,
    busy: bool,
}

struct ConfirmationPlan {
    operation: IntegrationControlOperation,
    target_entry_id: String,
    evidence: ControlEvidence,
    expires_at: Instant,
}

struct PendingAction {
    action_id: String,
    operation: IntegrationControlOperation,
    target_entry_id: String,
    evidence: ControlEvidence,
    url: String,
    opened: bool,
    process: Option<AppServerProcess>,
    expires_at: Instant,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ControlEvidence {
    Connector {
        raw_id: String,
        display_name: String,
        install_url: Option<String>,
        accessible: bool,
        enabled: bool,
        callable: bool,
    },
    Skill {
        raw_name: String,
        path: String,
        scope: String,
        enabled: bool,
    },
    Mcp {
        raw_name: String,
        auth_status: String,
    },
}

pub(crate) struct ResolvedIntegrationMention {
    pub(crate) name: String,
    pub(crate) path: String,
}

enum ControlExecution {
    Finished(IntegrationControlResultSnapshot),
    Pending(Box<PendingAction>, IntegrationControlResultSnapshot),
}

struct ValidatedPreview<'a> {
    entry: &'a IntegrationEntry,
    warnings: Vec<IntegrationControlWarning>,
    permissions: Vec<IntegrationPermission>,
}

impl Default for IntegrationControlService {
    fn default() -> Self {
        Self {
            program: "codex".to_owned(),
            command: AppServerCommand::codex("codex"),
            cli_version_override: None,
            state: Mutex::new(ControlState::default()),
        }
    }
}

impl IntegrationControlService {
    pub async fn preview(
        &self,
        request: IntegrationControlPreviewRequest,
        catalog: &IntegrationCatalogSnapshot,
    ) -> IntegrationControlPreviewSnapshot {
        if !valid_entry_id(&request.target_entry_id) {
            let bounded_request = IntegrationControlPreviewRequest {
                operation: request.operation,
                target_entry_id: "invalid:request".to_owned(),
            };
            return IntegrationControlPreviewSnapshot::unavailable(
                &bounded_request,
                IntegrationControlPreviewState::Unavailable,
                IntegrationControlDiagnosticCode::InvalidRequest,
            );
        }
        self.reap_expired_pending().await;
        let ValidatedPreview {
            entry,
            warnings,
            permissions,
        } = match validate_preview(&request, catalog) {
            Ok(review) => review,
            Err((state, code)) => {
                return IntegrationControlPreviewSnapshot::unavailable(&request, state, code)
            }
        };

        let evidence = match self
            .resolve_evidence(request.operation, &request.target_entry_id)
            .await
        {
            Ok((mut process, evidence)) => {
                let shutdown = process.shutdown().await;
                if shutdown.is_err() {
                    return IntegrationControlPreviewSnapshot::unavailable(
                        &request,
                        IntegrationControlPreviewState::Unavailable,
                        IntegrationControlDiagnosticCode::CliUnavailable,
                    );
                }
                evidence
            }
            Err(code) => {
                return IntegrationControlPreviewSnapshot::unavailable(
                    &request,
                    IntegrationControlPreviewState::Unavailable,
                    code,
                )
            }
        };
        if !evidence_matches_operation(&evidence, request.operation) {
            return IntegrationControlPreviewSnapshot::unavailable(
                &request,
                IntegrationControlPreviewState::Unavailable,
                IntegrationControlDiagnosticCode::OperationUnavailable,
            );
        }

        let mut state = self.state.lock().await;
        state
            .confirmations
            .retain(|_, plan| plan.expires_at > Instant::now());
        if state.confirmations.len() >= MAX_CONFIRMATIONS {
            return IntegrationControlPreviewSnapshot::unavailable(
                &request,
                IntegrationControlPreviewState::Unavailable,
                IntegrationControlDiagnosticCode::CapacityReached,
            );
        }
        let confirmation_id = Uuid::now_v7().to_string();
        state.confirmations.insert(
            confirmation_id.clone(),
            ConfirmationPlan {
                operation: request.operation,
                target_entry_id: request.target_entry_id.clone(),
                evidence,
                expires_at: Instant::now() + CONFIRMATION_TTL,
            },
        );

        IntegrationControlPreviewSnapshot {
            schema_version: INTEGRATION_CONTROL_SCHEMA_VERSION,
            state: IntegrationControlPreviewState::Ready,
            operation: request.operation,
            target_entry_id: request.target_entry_id,
            target_display_name: Some(entry.display_name.clone()),
            permissions,
            warnings,
            confirmation_id: Some(confirmation_id),
            diagnostic_code: None,
        }
    }

    pub async fn confirm(
        &self,
        request: IntegrationControlConfirmationRequest,
        catalog: &IntegrationCatalogSnapshot,
    ) -> IntegrationControlResultSnapshot {
        self.reap_expired_pending().await;
        if !is_uuid_v7(&request.confirmation_id) {
            return IntegrationControlResultSnapshot::unavailable(
                None,
                None,
                IntegrationControlDiagnosticCode::ConfirmationExpired,
            );
        }
        let plan = {
            let mut state = self.state.lock().await;
            state
                .confirmations
                .retain(|_, plan| plan.expires_at > Instant::now());
            if state.busy || state.pending.is_some() {
                return IntegrationControlResultSnapshot::unavailable(
                    None,
                    None,
                    IntegrationControlDiagnosticCode::CapacityReached,
                );
            }
            let Some(plan) = state.confirmations.remove(&request.confirmation_id) else {
                return IntegrationControlResultSnapshot::unavailable(
                    None,
                    None,
                    IntegrationControlDiagnosticCode::ConfirmationExpired,
                );
            };
            state.busy = true;
            plan
        };

        let preview_request = IntegrationControlPreviewRequest {
            operation: plan.operation,
            target_entry_id: plan.target_entry_id.clone(),
        };
        if validate_preview(&preview_request, catalog).is_err() {
            self.state.lock().await.busy = false;
            return IntegrationControlResultSnapshot::unavailable(
                Some(plan.operation),
                Some(plan.target_entry_id),
                IntegrationControlDiagnosticCode::StalePreview,
            );
        }

        let execution = self.execute(plan).await;
        let mut state = self.state.lock().await;
        state.busy = false;
        match execution {
            Ok(ControlExecution::Finished(result)) => result,
            Ok(ControlExecution::Pending(pending, result)) => {
                state.pending = Some(*pending);
                result
            }
            Err((operation, target_entry_id, code)) => {
                IntegrationControlResultSnapshot::unavailable(
                    Some(operation),
                    Some(target_entry_id),
                    code,
                )
            }
        }
    }

    pub async fn claim_handoff(
        &self,
        request: &IntegrationControlActionRequest,
    ) -> Result<(String, IntegrationControlResultSnapshot), IntegrationControlDiagnosticCode> {
        self.reap_expired_pending().await;
        if !is_uuid_v7(&request.action_id) {
            return Err(IntegrationControlDiagnosticCode::HandoffUnavailable);
        }
        let mut state = self.state.lock().await;
        let pending = state
            .pending
            .as_mut()
            .filter(|pending| {
                pending.action_id == request.action_id
                    && pending.expires_at > Instant::now()
                    && !pending.opened
            })
            .ok_or(IntegrationControlDiagnosticCode::HandoffUnavailable)?;
        pending.opened = true;
        Ok((
            pending.url.clone(),
            pending_snapshot(pending, IntegrationControlResultState::Pending),
        ))
    }

    pub async fn restore_handoff(&self, request: &IntegrationControlActionRequest) {
        let mut state = self.state.lock().await;
        if let Some(pending) = state.pending.as_mut().filter(|pending| {
            pending.action_id == request.action_id && pending.expires_at > Instant::now()
        }) {
            pending.opened = false;
        }
    }

    pub async fn status(
        &self,
        request: IntegrationControlActionRequest,
    ) -> IntegrationControlResultSnapshot {
        if !is_uuid_v7(&request.action_id) {
            return IntegrationControlResultSnapshot::unavailable(
                None,
                None,
                IntegrationControlDiagnosticCode::HandoffUnavailable,
            );
        }
        let mut pending = {
            let mut state = self.state.lock().await;
            let Some(pending) = state.pending.take() else {
                return IntegrationControlResultSnapshot::unavailable(
                    None,
                    None,
                    IntegrationControlDiagnosticCode::HandoffUnavailable,
                );
            };
            if pending.action_id != request.action_id {
                state.pending = Some(pending);
                return IntegrationControlResultSnapshot::unavailable(
                    None,
                    None,
                    IntegrationControlDiagnosticCode::HandoffUnavailable,
                );
            }
            pending
        };

        if pending.expires_at <= Instant::now() {
            if let Some(mut process) = pending.process.take() {
                let _ = process.shutdown().await;
            }
            return IntegrationControlResultSnapshot::unavailable(
                Some(pending.operation),
                Some(pending.target_entry_id),
                IntegrationControlDiagnosticCode::HandoffUnavailable,
            );
        }

        let result = match pending.operation {
            IntegrationControlOperation::ConnectorAuthorize => {
                self.connector_status(&pending).await
            }
            IntegrationControlOperation::McpAuthorize => mcp_status(&mut pending).await,
            IntegrationControlOperation::SkillEnable
            | IntegrationControlOperation::SkillDisable => {
                Err(IntegrationControlDiagnosticCode::OperationUnavailable)
            }
        };
        match result {
            Ok(Some(result)) => result,
            Ok(None) => {
                let result = pending_snapshot(&pending, IntegrationControlResultState::Pending);
                self.state.lock().await.pending = Some(pending);
                result
            }
            Err(code) => {
                if let Some(mut process) = pending.process.take() {
                    let _ = process.shutdown().await;
                }
                IntegrationControlResultSnapshot::unavailable(
                    Some(pending.operation),
                    Some(pending.target_entry_id),
                    code,
                )
            }
        }
    }

    pub async fn resolve_mentions(
        &self,
        entry_ids: &[String],
    ) -> Result<Vec<ResolvedIntegrationMention>, IntegrationControlDiagnosticCode> {
        if entry_ids.len() > MAX_MENTIONS
            || entry_ids.iter().any(|entry_id| !valid_entry_id(entry_id))
            || entry_ids.iter().collect::<HashSet<_>>().len() != entry_ids.len()
        {
            return Err(IntegrationControlDiagnosticCode::InvalidRequest);
        }
        if entry_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut process = self.spawn_initialized().await?;
        let mut mentions = Vec::with_capacity(entry_ids.len());
        for entry_id in entry_ids {
            let evidence = resolve_connector(&mut process, entry_id).await?;
            let ControlEvidence::Connector {
                raw_id,
                display_name,
                accessible,
                enabled,
                callable,
                ..
            } = evidence
            else {
                return Err(IntegrationControlDiagnosticCode::TargetNotFound);
            };
            if !accessible || !enabled || !callable || !valid_app_path_id(&raw_id) {
                let _ = process.shutdown().await;
                return Err(IntegrationControlDiagnosticCode::OperationUnavailable);
            }
            mentions.push(ResolvedIntegrationMention {
                name: display_name,
                path: format!("app://{raw_id}"),
            });
        }
        process
            .shutdown()
            .await
            .map_err(|_| IntegrationControlDiagnosticCode::CliUnavailable)?;
        Ok(mentions)
    }

    async fn execute(
        &self,
        plan: ConfirmationPlan,
    ) -> Result<
        ControlExecution,
        (
            IntegrationControlOperation,
            String,
            IntegrationControlDiagnosticCode,
        ),
    > {
        let operation = plan.operation;
        let target_entry_id = plan.target_entry_id.clone();
        let (mut process, current) = self
            .resolve_evidence(operation, &target_entry_id)
            .await
            .map_err(|code| (operation, target_entry_id.clone(), code))?;
        if current != plan.evidence {
            let _ = process.shutdown().await;
            return Err((
                operation,
                target_entry_id,
                IntegrationControlDiagnosticCode::StalePreview,
            ));
        }

        match (&plan.evidence, operation) {
            (ControlEvidence::Skill { path, .. }, IntegrationControlOperation::SkillEnable)
            | (ControlEvidence::Skill { path, .. }, IntegrationControlOperation::SkillDisable) => {
                let enabled = operation == IntegrationControlOperation::SkillEnable;
                let response = process
                    .request(
                        "skills/config/write",
                        json!({"path": path, "enabled": enabled}),
                    )
                    .await
                    .map_err(|_| {
                        (
                            operation,
                            target_entry_id.clone(),
                            IntegrationControlDiagnosticCode::MutationFailed,
                        )
                    })?;
                let valid_response = known_object(&response, &["effectiveEnabled"])
                    .and_then(|response| response.get("effectiveEnabled"))
                    .and_then(Value::as_bool)
                    == Some(enabled);
                if !valid_response {
                    let _ = process.shutdown().await;
                    return Err((
                        operation,
                        target_entry_id,
                        IntegrationControlDiagnosticCode::ResponseInvalid,
                    ));
                }
                let verified = resolve_skill(&mut process, &plan.target_entry_id)
                    .await
                    .is_ok_and(|evidence| {
                        matches!(evidence, ControlEvidence::Skill { enabled: value, .. } if value == enabled)
                    });
                let _ = process.shutdown().await;
                if !verified {
                    return Err((
                        operation,
                        target_entry_id,
                        IntegrationControlDiagnosticCode::PostconditionFailed,
                    ));
                }
                Ok(ControlExecution::Finished(
                    IntegrationControlResultSnapshot {
                        schema_version: INTEGRATION_CONTROL_SCHEMA_VERSION,
                        state: IntegrationControlResultState::Applied,
                        operation: Some(operation),
                        target_entry_id: Some(target_entry_id),
                        action_id: None,
                        browser_handoff_available: false,
                        catalog_refresh_required: true,
                        diagnostic_code: None,
                    },
                ))
            }
            (
                ControlEvidence::Connector {
                    install_url: Some(url),
                    ..
                },
                IntegrationControlOperation::ConnectorAuthorize,
            ) => {
                validate_handoff_url(url)
                    .map_err(|code| (operation, target_entry_id.clone(), code))?;
                let _ = process.shutdown().await;
                Ok(pending_execution(
                    operation,
                    target_entry_id,
                    plan.evidence.clone(),
                    url.clone(),
                    None,
                ))
            }
            (ControlEvidence::Mcp { raw_name, .. }, IntegrationControlOperation::McpAuthorize) => {
                let response = process
                    .request(
                        "mcpServer/oauth/login",
                        json!({
                            "name": raw_name,
                            "scopes": null,
                            "threadId": null,
                            "timeoutSecs": 300
                        }),
                    )
                    .await
                    .map_err(|_| {
                        (
                            operation,
                            target_entry_id.clone(),
                            IntegrationControlDiagnosticCode::HandoffUnavailable,
                        )
                    })?;
                let Some(url) = known_object(&response, &["authorizationUrl"])
                    .and_then(|response| response.get("authorizationUrl"))
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                else {
                    let _ = process.shutdown().await;
                    return Err((
                        operation,
                        target_entry_id,
                        IntegrationControlDiagnosticCode::ResponseInvalid,
                    ));
                };
                validate_handoff_url(&url)
                    .map_err(|code| (operation, target_entry_id.clone(), code))?;
                Ok(pending_execution(
                    operation,
                    target_entry_id,
                    plan.evidence,
                    url,
                    Some(process),
                ))
            }
            _ => {
                let _ = process.shutdown().await;
                Err((
                    operation,
                    target_entry_id,
                    IntegrationControlDiagnosticCode::OperationUnavailable,
                ))
            }
        }
    }

    async fn connector_status(
        &self,
        pending: &PendingAction,
    ) -> Result<Option<IntegrationControlResultSnapshot>, IntegrationControlDiagnosticCode> {
        let (mut process, evidence) = self
            .resolve_evidence(pending.operation, &pending.target_entry_id)
            .await?;
        let _ = process.shutdown().await;
        let ControlEvidence::Connector { accessible, .. } = evidence else {
            return Err(IntegrationControlDiagnosticCode::ResponseInvalid);
        };
        Ok(accessible.then(|| completed_snapshot(pending)))
    }

    async fn resolve_evidence(
        &self,
        operation: IntegrationControlOperation,
        target_entry_id: &str,
    ) -> Result<(AppServerProcess, ControlEvidence), IntegrationControlDiagnosticCode> {
        let mut process = self.spawn_initialized().await?;
        let evidence = match operation {
            IntegrationControlOperation::ConnectorAuthorize => {
                resolve_connector(&mut process, target_entry_id).await
            }
            IntegrationControlOperation::SkillEnable
            | IntegrationControlOperation::SkillDisable => {
                resolve_skill(&mut process, target_entry_id).await
            }
            IntegrationControlOperation::McpAuthorize => {
                resolve_mcp(&mut process, target_entry_id).await
            }
        };
        match evidence {
            Ok(evidence) => Ok((process, evidence)),
            Err(code) => {
                let _ = process.shutdown().await;
                Err(code)
            }
        }
    }

    async fn spawn_initialized(
        &self,
    ) -> Result<AppServerProcess, IntegrationControlDiagnosticCode> {
        let version = match self.cli_version_override.as_ref() {
            Some(version) => version.clone(),
            None => probe_cli_version(&self.program)
                .await
                .map_err(|_| IntegrationControlDiagnosticCode::CliUnavailable)?,
        };
        if !supports_integration_routes(&version) {
            return Err(IntegrationControlDiagnosticCode::VersionUnsupported);
        }
        let mut process = AppServerProcess::spawn(self.command.clone())
            .map_err(|_| IntegrationControlDiagnosticCode::CliUnavailable)?;
        timeout(CONTROL_TIMEOUT, process.initialize())
            .await
            .map_err(|_| IntegrationControlDiagnosticCode::CliUnavailable)?
            .map_err(|_| IntegrationControlDiagnosticCode::CliUnavailable)?;
        Ok(process)
    }

    async fn reap_expired_pending(&self) {
        let mut process = {
            let mut state = self.state.lock().await;
            if state
                .pending
                .as_ref()
                .is_some_and(|pending| pending.expires_at <= Instant::now())
            {
                state
                    .pending
                    .take()
                    .and_then(|mut pending| pending.process.take())
            } else {
                None
            }
        };
        if let Some(process) = process.as_mut() {
            let _ = process.shutdown().await;
        }
    }

    #[cfg(test)]
    fn with_command(command: AppServerCommand, cli_version: &str) -> Self {
        Self {
            program: "fixture-codex".to_owned(),
            command,
            cli_version_override: Some(cli_version.to_owned()),
            state: Mutex::new(ControlState::default()),
        }
    }
}

fn validate_preview<'a>(
    request: &IntegrationControlPreviewRequest,
    catalog: &'a IntegrationCatalogSnapshot,
) -> Result<
    ValidatedPreview<'a>,
    (
        IntegrationControlPreviewState,
        IntegrationControlDiagnosticCode,
    ),
> {
    if !valid_entry_id(&request.target_entry_id) {
        return Err((
            IntegrationControlPreviewState::Unavailable,
            IntegrationControlDiagnosticCode::InvalidRequest,
        ));
    }
    if catalog.catalog_state == IntegrationAvailability::Unavailable {
        return Err((
            IntegrationControlPreviewState::Unavailable,
            IntegrationControlDiagnosticCode::CatalogUnavailable,
        ));
    }
    let capability_id = match request.operation {
        IntegrationControlOperation::ConnectorAuthorize => "connector.authorize",
        IntegrationControlOperation::SkillEnable | IntegrationControlOperation::SkillDisable => {
            "skill.configure"
        }
        IntegrationControlOperation::McpAuthorize => "mcp.authorize",
    };
    let capability_ready = catalog.capabilities.iter().any(|capability| {
        capability.id == capability_id
            && capability.availability == IntegrationAvailability::Ready
            && capability.implementation == IntegrationImplementation::Ready
            && capability.mutating
            && capability.requires_confirmation
    });
    if !capability_ready {
        return Err((
            IntegrationControlPreviewState::Unavailable,
            IntegrationControlDiagnosticCode::OperationUnavailable,
        ));
    }
    let Some(entry) = catalog
        .entries
        .iter()
        .find(|entry| entry.id == request.target_entry_id)
    else {
        return Err((
            IntegrationControlPreviewState::Unavailable,
            IntegrationControlDiagnosticCode::TargetNotFound,
        ));
    };
    if entry.policy.state == IntegrationPolicyState::Blocked {
        return Err((
            IntegrationControlPreviewState::Blocked,
            IntegrationControlDiagnosticCode::PolicyBlocked,
        ));
    }

    let valid_state = match request.operation {
        IntegrationControlOperation::ConnectorAuthorize => {
            entry.kind == IntegrationEntryKind::Connector
                && entry.authentication == IntegrationAuthenticationState::Required
        }
        IntegrationControlOperation::SkillEnable => {
            entry.kind == IntegrationEntryKind::Skill
                && entry.enablement == IntegrationEnablementState::Disabled
                && entry.scope != IntegrationScope::Managed
        }
        IntegrationControlOperation::SkillDisable => {
            entry.kind == IntegrationEntryKind::Skill
                && entry.enablement == IntegrationEnablementState::Enabled
                && entry.scope != IntegrationScope::Managed
        }
        IntegrationControlOperation::McpAuthorize => {
            entry.kind == IntegrationEntryKind::McpServer
                && entry.authentication == IntegrationAuthenticationState::Required
        }
    };
    if !valid_state || !entry.capability_ids.iter().any(|id| id == capability_id) {
        return Err((
            IntegrationControlPreviewState::Unavailable,
            IntegrationControlDiagnosticCode::OperationUnavailable,
        ));
    }

    let (warnings, permissions) = match request.operation {
        IntegrationControlOperation::ConnectorAuthorize => (
            vec![
                IntegrationControlWarning::OpensExternalBrowser,
                IntegrationControlWarning::AccountAuthorization,
            ],
            vec![IntegrationPermission {
                kind: IntegrationPermissionKind::Account,
                access: IntegrationPermissionAccess::Authorize,
                target: "Connector account".to_owned(),
                required: true,
            }],
        ),
        IntegrationControlOperation::McpAuthorize => (
            vec![
                IntegrationControlWarning::OpensExternalBrowser,
                IntegrationControlWarning::NetworkAuthorization,
            ],
            vec![IntegrationPermission {
                kind: IntegrationPermissionKind::Network,
                access: IntegrationPermissionAccess::Authorize,
                target: "Configured MCP endpoint".to_owned(),
                required: true,
            }],
        ),
        IntegrationControlOperation::SkillEnable | IntegrationControlOperation::SkillDisable => {
            let mut warnings = vec![IntegrationControlWarning::ChangesCodexConfiguration];
            if entry.scope == IntegrationScope::Project {
                warnings.push(IntegrationControlWarning::ProjectScoped);
            }
            (warnings, entry.permissions.clone())
        }
    };
    Ok(ValidatedPreview {
        entry,
        warnings,
        permissions,
    })
}

async fn resolve_connector(
    process: &mut AppServerProcess,
    target_entry_id: &str,
) -> Result<ControlEvidence, IntegrationControlDiagnosticCode> {
    const APP_KEYS: &[&str] = &[
        "appMetadata",
        "branding",
        "description",
        "distributionChannel",
        "iconAssets",
        "iconDarkAssets",
        "id",
        "installUrl",
        "isAccessible",
        "isEnabled",
        "labels",
        "logoUrl",
        "logoUrlDark",
        "name",
        "pluginDisplayNames",
    ];
    let apps = paginated_request(
        process,
        "app/list",
        json!({"limit": 128, "forceRefetch": true, "threadId": null}),
    )
    .await
    .map_err(|_| IntegrationControlDiagnosticCode::CatalogUnavailable)?;
    let installed_response = process
        .request(
            "app/installed",
            json!({"forceRefresh": false, "threadId": null}),
        )
        .await
        .map_err(|_| IntegrationControlDiagnosticCode::CatalogUnavailable)?;
    let installed = known_object(&installed_response, &["apps"])
        .and_then(|object| object.get("apps"))
        .and_then(Value::as_array)
        .ok_or(IntegrationControlDiagnosticCode::ResponseInvalid)?;
    if installed.len() > MAX_SOURCE_ENTRIES {
        return Err(IntegrationControlDiagnosticCode::ResponseInvalid);
    }
    let mut installed_by_id = HashMap::new();
    for item in installed {
        let item = known_object(item, &["callable", "enabled", "id", "runtimeName"])
            .ok_or(IntegrationControlDiagnosticCode::ResponseInvalid)?;
        let raw_id = item
            .get("id")
            .and_then(Value::as_str)
            .ok_or(IntegrationControlDiagnosticCode::ResponseInvalid)?;
        validate_protocol_identifier(raw_id, 128)
            .map_err(|_| IntegrationControlDiagnosticCode::ResponseInvalid)?;
        let callable = item
            .get("callable")
            .and_then(Value::as_bool)
            .ok_or(IntegrationControlDiagnosticCode::ResponseInvalid)?;
        let enabled = item
            .get("enabled")
            .and_then(Value::as_bool)
            .ok_or(IntegrationControlDiagnosticCode::ResponseInvalid)?;
        if installed_by_id
            .insert(raw_id.to_owned(), (callable, enabled))
            .is_some()
        {
            return Err(IntegrationControlDiagnosticCode::ResponseInvalid);
        }
    }

    let mut found = None;
    for app in apps {
        let app = known_object(&app, APP_KEYS)
            .ok_or(IntegrationControlDiagnosticCode::ResponseInvalid)?;
        let raw_id = app
            .get("id")
            .and_then(Value::as_str)
            .ok_or(IntegrationControlDiagnosticCode::ResponseInvalid)?;
        let Some(entry_id) = normalized_entry_id("connector", raw_id) else {
            continue;
        };
        if entry_id != target_entry_id {
            continue;
        }
        if found.is_some() {
            return Err(IntegrationControlDiagnosticCode::ResponseInvalid);
        }
        validate_protocol_identifier(raw_id, 128)
            .map_err(|_| IntegrationControlDiagnosticCode::ResponseInvalid)?;
        let raw_name = app
            .get("name")
            .and_then(Value::as_str)
            .ok_or(IntegrationControlDiagnosticCode::ResponseInvalid)?;
        let display_name =
            safe_display(raw_name, 128).ok_or(IntegrationControlDiagnosticCode::ResponseInvalid)?;
        let accessible = app
            .get("isAccessible")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let list_enabled = app
            .get("isEnabled")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let (callable, installed_enabled) = installed_by_id
            .get(raw_id)
            .copied()
            .unwrap_or((false, false));
        let install_url = app
            .get("installUrl")
            .and_then(Value::as_str)
            .map(str::to_owned);
        if let Some(url) = install_url.as_deref() {
            validate_handoff_url(url)?;
        }
        found = Some(ControlEvidence::Connector {
            raw_id: raw_id.to_owned(),
            display_name,
            install_url,
            accessible,
            enabled: list_enabled && installed_enabled,
            callable,
        });
    }
    found.ok_or(IntegrationControlDiagnosticCode::TargetNotFound)
}

async fn resolve_skill(
    process: &mut AppServerProcess,
    target_entry_id: &str,
) -> Result<ControlEvidence, IntegrationControlDiagnosticCode> {
    let cwd = std::env::temp_dir().to_string_lossy().into_owned();
    let response = process
        .request("skills/list", json!({"cwds": [cwd], "forceReload": true}))
        .await
        .map_err(|_| IntegrationControlDiagnosticCode::CatalogUnavailable)?;
    let groups = known_object(&response, &["data"])
        .and_then(|object| object.get("data"))
        .and_then(Value::as_array)
        .ok_or(IntegrationControlDiagnosticCode::ResponseInvalid)?;
    if groups.len() > MAX_SOURCE_ENTRIES {
        return Err(IntegrationControlDiagnosticCode::ResponseInvalid);
    }
    let mut found = None;
    let mut skill_count = 0_usize;
    for group in groups {
        let group = known_object(group, &["cwd", "errors", "skills"])
            .ok_or(IntegrationControlDiagnosticCode::ResponseInvalid)?;
        let skills = group
            .get("skills")
            .and_then(Value::as_array)
            .ok_or(IntegrationControlDiagnosticCode::ResponseInvalid)?;
        skill_count = skill_count.saturating_add(skills.len());
        if skill_count > MAX_SOURCE_ENTRIES {
            return Err(IntegrationControlDiagnosticCode::ResponseInvalid);
        }
        for skill in skills {
            let skill = known_object(
                skill,
                &[
                    "dependencies",
                    "description",
                    "enabled",
                    "interface",
                    "name",
                    "path",
                    "scope",
                    "shortDescription",
                ],
            )
            .ok_or(IntegrationControlDiagnosticCode::ResponseInvalid)?;
            let raw_name = skill
                .get("name")
                .and_then(Value::as_str)
                .ok_or(IntegrationControlDiagnosticCode::ResponseInvalid)?;
            let raw_name = safe_display(raw_name, 128)
                .ok_or(IntegrationControlDiagnosticCode::ResponseInvalid)?;
            if normalized_entry_id("skill", &raw_name).as_deref() != Some(target_entry_id) {
                continue;
            }
            if found.is_some() {
                return Err(IntegrationControlDiagnosticCode::ResponseInvalid);
            }
            let path = skill
                .get("path")
                .and_then(Value::as_str)
                .filter(|path| path.starts_with('/') && path.len() <= 4096 && !path.contains('\0'))
                .ok_or(IntegrationControlDiagnosticCode::ResponseInvalid)?;
            let scope = skill
                .get("scope")
                .and_then(Value::as_str)
                .filter(|scope| matches!(*scope, "repo" | "user"))
                .ok_or(IntegrationControlDiagnosticCode::OperationUnavailable)?;
            let enabled = skill
                .get("enabled")
                .and_then(Value::as_bool)
                .ok_or(IntegrationControlDiagnosticCode::ResponseInvalid)?;
            found = Some(ControlEvidence::Skill {
                raw_name,
                path: path.to_owned(),
                scope: scope.to_owned(),
                enabled,
            });
        }
    }
    found.ok_or(IntegrationControlDiagnosticCode::TargetNotFound)
}

async fn resolve_mcp(
    process: &mut AppServerProcess,
    target_entry_id: &str,
) -> Result<ControlEvidence, IntegrationControlDiagnosticCode> {
    let servers = paginated_request(
        process,
        "mcpServerStatus/list",
        json!({"limit": 128, "detail": "toolsAndAuthOnly", "threadId": null}),
    )
    .await
    .map_err(|_| IntegrationControlDiagnosticCode::CatalogUnavailable)?;
    let mut found = None;
    for server in servers {
        let server = known_object(
            &server,
            &[
                "authStatus",
                "name",
                "resourceTemplates",
                "resources",
                "serverInfo",
                "tools",
            ],
        )
        .ok_or(IntegrationControlDiagnosticCode::ResponseInvalid)?;
        let raw_name = server
            .get("name")
            .and_then(Value::as_str)
            .ok_or(IntegrationControlDiagnosticCode::ResponseInvalid)?;
        if normalized_entry_id("mcp", raw_name).as_deref() != Some(target_entry_id) {
            continue;
        }
        if found.is_some() {
            return Err(IntegrationControlDiagnosticCode::ResponseInvalid);
        }
        validate_protocol_identifier(raw_name, 128)
            .map_err(|_| IntegrationControlDiagnosticCode::ResponseInvalid)?;
        let auth_status = server
            .get("authStatus")
            .and_then(Value::as_str)
            .filter(|status| {
                matches!(
                    *status,
                    "notLoggedIn" | "bearerToken" | "oAuth" | "unsupported"
                )
            })
            .ok_or(IntegrationControlDiagnosticCode::ResponseInvalid)?;
        found = Some(ControlEvidence::Mcp {
            raw_name: raw_name.to_owned(),
            auth_status: auth_status.to_owned(),
        });
    }
    found.ok_or(IntegrationControlDiagnosticCode::TargetNotFound)
}

async fn mcp_status(
    pending: &mut PendingAction,
) -> Result<Option<IntegrationControlResultSnapshot>, IntegrationControlDiagnosticCode> {
    let ControlEvidence::Mcp { raw_name, .. } = &pending.evidence else {
        return Err(IntegrationControlDiagnosticCode::ResponseInvalid);
    };
    let raw_name = raw_name.clone();
    let mut process = pending
        .process
        .take()
        .ok_or(IntegrationControlDiagnosticCode::ResponseInvalid)?;
    for _ in 0..8 {
        let Some(notification) = process
            .next_notification_with_timeout(NOTIFICATION_POLL)
            .await
            .map_err(|_| IntegrationControlDiagnosticCode::CliUnavailable)?
        else {
            pending.process = Some(process);
            return Ok(None);
        };
        match notification {
            AppServerNotification::McpOauthLoginCompleted { name, success } => {
                if name != raw_name {
                    return Err(IntegrationControlDiagnosticCode::ResponseInvalid);
                }
                if !success {
                    return Err(IntegrationControlDiagnosticCode::AuthorizationFailed);
                }
                let result = completed_snapshot(pending);
                let _ = process.shutdown().await;
                return Ok(Some(result));
            }
            AppServerNotification::IntegrationRefresh(_)
            | AppServerNotification::AccountUpdated
            | AppServerNotification::AccountLoginCompleted { .. } => continue,
            AppServerNotification::Conversation(_)
            | AppServerNotification::ConversationRequest(_) => {
                return Err(IntegrationControlDiagnosticCode::ResponseInvalid)
            }
        }
    }
    pending.process = Some(process);
    Ok(None)
}

fn pending_execution(
    operation: IntegrationControlOperation,
    target_entry_id: String,
    evidence: ControlEvidence,
    url: String,
    process: Option<AppServerProcess>,
) -> ControlExecution {
    let action_id = Uuid::now_v7().to_string();
    let pending = PendingAction {
        action_id: action_id.clone(),
        operation,
        target_entry_id: target_entry_id.clone(),
        evidence,
        url,
        opened: false,
        process,
        expires_at: Instant::now() + ACTION_TTL,
    };
    let result = IntegrationControlResultSnapshot {
        schema_version: INTEGRATION_CONTROL_SCHEMA_VERSION,
        state: IntegrationControlResultState::HandoffReady,
        operation: Some(operation),
        target_entry_id: Some(target_entry_id),
        action_id: Some(action_id),
        browser_handoff_available: true,
        catalog_refresh_required: false,
        diagnostic_code: None,
    };
    ControlExecution::Pending(Box::new(pending), result)
}

fn pending_snapshot(
    pending: &PendingAction,
    state: IntegrationControlResultState,
) -> IntegrationControlResultSnapshot {
    IntegrationControlResultSnapshot {
        schema_version: INTEGRATION_CONTROL_SCHEMA_VERSION,
        state,
        operation: Some(pending.operation),
        target_entry_id: Some(pending.target_entry_id.clone()),
        action_id: Some(pending.action_id.clone()),
        browser_handoff_available: !pending.opened,
        catalog_refresh_required: false,
        diagnostic_code: None,
    }
}

fn completed_snapshot(pending: &PendingAction) -> IntegrationControlResultSnapshot {
    IntegrationControlResultSnapshot {
        schema_version: INTEGRATION_CONTROL_SCHEMA_VERSION,
        state: IntegrationControlResultState::Completed,
        operation: Some(pending.operation),
        target_entry_id: Some(pending.target_entry_id.clone()),
        action_id: None,
        browser_handoff_available: false,
        catalog_refresh_required: true,
        diagnostic_code: None,
    }
}

fn evidence_matches_operation(
    evidence: &ControlEvidence,
    operation: IntegrationControlOperation,
) -> bool {
    match (evidence, operation) {
        (
            ControlEvidence::Connector {
                install_url: Some(_),
                accessible: false,
                ..
            },
            IntegrationControlOperation::ConnectorAuthorize,
        ) => true,
        (ControlEvidence::Mcp { auth_status, .. }, IntegrationControlOperation::McpAuthorize) => {
            auth_status == "notLoggedIn"
        }
        (
            ControlEvidence::Skill { enabled: false, .. },
            IntegrationControlOperation::SkillEnable,
        )
        | (
            ControlEvidence::Skill { enabled: true, .. },
            IntegrationControlOperation::SkillDisable,
        ) => true,
        _ => false,
    }
}

fn validate_handoff_url(value: &str) -> Result<(), IntegrationControlDiagnosticCode> {
    if value.is_empty() || value.len() > MAX_URL_BYTES {
        return Err(IntegrationControlDiagnosticCode::HandoffUnavailable);
    }
    let url =
        Url::parse(value).map_err(|_| IntegrationControlDiagnosticCode::HandoffUnavailable)?;
    if !url.username().is_empty() || url.password().is_some() || url.fragment().is_some() {
        return Err(IntegrationControlDiagnosticCode::HandoffUnavailable);
    }
    let host = url
        .host_str()
        .ok_or(IntegrationControlDiagnosticCode::HandoffUnavailable)?;
    let secure = url.scheme() == "https";
    let loopback = url.scheme() == "http" && matches!(host, "localhost" | "127.0.0.1" | "::1");
    if !secure && !loopback {
        return Err(IntegrationControlDiagnosticCode::HandoffUnavailable);
    }
    Ok(())
}

fn safe_display(value: &str, maximum: usize) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()
        && value.len() <= maximum
        && !value.chars().any(|character| {
            let code = u32::from(character);
            character.is_control()
                || (0x7f..=0x9f).contains(&code)
                || (0x200b..=0x200f).contains(&code)
                || (0x202a..=0x202e).contains(&code)
                || (0x2060..=0x206f).contains(&code)
                || code == 0xfeff
        }))
    .then(|| value.to_owned())
}

fn valid_entry_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'.' | b'_' | b':' | b'-')
        })
}

fn valid_app_path_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn is_uuid_v7(value: &str) -> bool {
    Uuid::parse_str(value).is_ok_and(|uuid| uuid.get_version_num() == 7)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codex::integration::IntegrationControlConfirmationRequest;

    #[tokio::test]
    async fn enables_one_exact_skill_with_a_one_use_confirmation() {
        let script = skill_control_script();
        let service = IntegrationControlService::with_command(
            AppServerCommand::test("sh", &["-c", &script]),
            "0.145.0",
        );
        let catalog = fixture_catalog();
        let request = IntegrationControlPreviewRequest {
            operation: IntegrationControlOperation::SkillEnable,
            target_entry_id: "skill:fixture-project-checks".to_owned(),
        };

        let preview = service.preview(request, &catalog).await;
        assert_eq!(preview.state, IntegrationControlPreviewState::Ready);
        assert!(preview
            .warnings
            .contains(&IntegrationControlWarning::ChangesCodexConfiguration));
        assert!(preview
            .warnings
            .contains(&IntegrationControlWarning::ProjectScoped));
        let confirmation_id = preview
            .confirmation_id
            .expect("ready skill preview needs a confirmation");

        let result = service
            .confirm(
                IntegrationControlConfirmationRequest {
                    confirmation_id: confirmation_id.clone(),
                },
                &catalog,
            )
            .await;
        assert_eq!(result.state, IntegrationControlResultState::Applied);
        assert!(result.catalog_refresh_required);

        let replay = service
            .confirm(
                IntegrationControlConfirmationRequest { confirmation_id },
                &catalog,
            )
            .await;
        assert_eq!(replay.state, IntegrationControlResultState::Unavailable);
        assert_eq!(
            replay.diagnostic_code,
            Some(IntegrationControlDiagnosticCode::ConfirmationExpired)
        );
    }

    #[tokio::test]
    async fn completes_only_the_exact_mcp_oauth_handoff_without_serializing_its_url() {
        let script = mcp_control_script("fixture-knowledge", true);
        let service = IntegrationControlService::with_command(
            AppServerCommand::test("sh", &["-c", &script]),
            "0.145.0",
        );
        let catalog = fixture_catalog();
        let preview = service
            .preview(
                IntegrationControlPreviewRequest {
                    operation: IntegrationControlOperation::McpAuthorize,
                    target_entry_id: "mcp:fixture-knowledge".to_owned(),
                },
                &catalog,
            )
            .await;
        let confirmation_id = preview
            .confirmation_id
            .expect("ready MCP preview needs a confirmation");

        let handoff = service
            .confirm(
                IntegrationControlConfirmationRequest { confirmation_id },
                &catalog,
            )
            .await;
        assert_eq!(handoff.state, IntegrationControlResultState::HandoffReady);
        let encoded = serde_json::to_string(&handoff).expect("result must serialize");
        assert!(!encoded.contains("browser-secret"));
        assert!(!encoded.contains("example.invalid"));
        let action_id = handoff.action_id.expect("handoff needs an action ID");
        let action = IntegrationControlActionRequest {
            action_id: action_id.clone(),
        };
        let (url, pending) = service
            .claim_handoff(&action)
            .await
            .expect("exact action must claim the handoff once");
        assert_eq!(url, "https://example.invalid/oauth?state=browser-secret");
        assert_eq!(pending.state, IntegrationControlResultState::Pending);
        assert!(!pending.browser_handoff_available);
        assert_eq!(
            service.claim_handoff(&action).await.err(),
            Some(IntegrationControlDiagnosticCode::HandoffUnavailable)
        );
        service.restore_handoff(&action).await;
        let (_, pending) = service
            .claim_handoff(&action)
            .await
            .expect("a failed browser open may restore one retry");
        assert_eq!(pending.state, IntegrationControlResultState::Pending);
        let completed = service.status(action).await;
        assert_eq!(completed.state, IntegrationControlResultState::Completed);
        assert!(completed.catalog_refresh_required);
        assert!(completed.action_id.is_none());
    }

    #[tokio::test]
    async fn rejects_a_mismatched_mcp_completion_and_unsafe_handoff_urls() {
        let script = mcp_control_script("another-server", true);
        let service = IntegrationControlService::with_command(
            AppServerCommand::test("sh", &["-c", &script]),
            "0.145.0",
        );
        let catalog = fixture_catalog();
        let preview = service
            .preview(
                IntegrationControlPreviewRequest {
                    operation: IntegrationControlOperation::McpAuthorize,
                    target_entry_id: "mcp:fixture-knowledge".to_owned(),
                },
                &catalog,
            )
            .await;
        let result = service
            .confirm(
                IntegrationControlConfirmationRequest {
                    confirmation_id: preview.confirmation_id.expect("confirmation"),
                },
                &catalog,
            )
            .await;
        let status = service
            .status(IntegrationControlActionRequest {
                action_id: result.action_id.expect("action"),
            })
            .await;
        assert_eq!(status.state, IntegrationControlResultState::Unavailable);
        assert_eq!(
            status.diagnostic_code,
            Some(IntegrationControlDiagnosticCode::ResponseInvalid)
        );

        for unsafe_url in [
            "http://example.invalid/oauth",
            "https://user:secret@example.invalid/oauth",
            "https://example.invalid/oauth#secret",
            "file:///tmp/oauth",
        ] {
            assert_eq!(
                validate_handoff_url(unsafe_url),
                Err(IntegrationControlDiagnosticCode::HandoffUnavailable)
            );
        }
        assert!(validate_handoff_url("http://127.0.0.1:43123/callback").is_ok());

        let invalid = service
            .preview(
                IntegrationControlPreviewRequest {
                    operation: IntegrationControlOperation::McpAuthorize,
                    target_entry_id: "../../private\nvalue".to_owned(),
                },
                &catalog,
            )
            .await;
        assert_eq!(invalid.target_entry_id, "invalid:request");
        assert_eq!(
            invalid.diagnostic_code,
            Some(IntegrationControlDiagnosticCode::InvalidRequest)
        );
        assert!(!serde_json::to_string(&invalid)
            .expect("invalid result must serialize")
            .contains("private"));
    }

    #[tokio::test]
    async fn resolves_only_an_authorized_callable_connector_to_an_app_mention() {
        let script = connector_control_script(true, true, true);
        let service = IntegrationControlService::with_command(
            AppServerCommand::test("sh", &["-c", &script]),
            "0.145.0",
        );
        let entry_id = "connector:fixture-calendar".to_owned();

        let mentions = service
            .resolve_mentions(std::slice::from_ref(&entry_id))
            .await
            .expect("ready connector must resolve");
        assert_eq!(mentions.len(), 1);
        assert_eq!(mentions[0].name, "Fixture calendar connector");
        assert_eq!(mentions[0].path, "app://fixture-calendar");

        let disabled_script = connector_control_script(true, false, true);
        let disabled = IntegrationControlService::with_command(
            AppServerCommand::test("sh", &["-c", &disabled_script]),
            "0.145.0",
        );
        assert_eq!(
            disabled
                .resolve_mentions(std::slice::from_ref(&entry_id))
                .await
                .err(),
            Some(IntegrationControlDiagnosticCode::OperationUnavailable)
        );
    }

    #[tokio::test]
    async fn reaps_an_expired_handoff_before_accepting_more_control_work() {
        let service = IntegrationControlService::with_command(
            AppServerCommand::test("sh", &["-c", "exit 91"]),
            "0.145.0",
        );
        service.state.lock().await.pending = Some(PendingAction {
            action_id: Uuid::now_v7().to_string(),
            operation: IntegrationControlOperation::McpAuthorize,
            target_entry_id: "mcp:expired".to_owned(),
            evidence: ControlEvidence::Mcp {
                raw_name: "expired".to_owned(),
                auth_status: "notLoggedIn".to_owned(),
            },
            url: "https://example.invalid/expired".to_owned(),
            opened: false,
            process: None,
            expires_at: Instant::now() - Duration::from_millis(1),
        });

        service.reap_expired_pending().await;

        assert!(service.state.lock().await.pending.is_none());
    }

    fn fixture_catalog() -> IntegrationCatalogSnapshot {
        let mut catalog: IntegrationCatalogSnapshot =
            serde_json::from_str(include_str!("../../../fixtures/integration-catalog.json"))
                .expect("catalog fixture");
        catalog.catalog_state = IntegrationAvailability::Ready;
        for capability in &mut catalog.capabilities {
            if matches!(
                capability.id.as_str(),
                "connector.authorize" | "skill.configure" | "mcp.authorize"
            ) {
                capability.availability = IntegrationAvailability::Ready;
                capability.implementation = IntegrationImplementation::Ready;
                capability.diagnostic_code = None;
            }
        }
        let skill = catalog
            .entries
            .iter_mut()
            .find(|entry| entry.id == "skill:fixture-project-checks")
            .expect("skill fixture");
        skill.enablement = IntegrationEnablementState::Disabled;
        skill.health.state = IntegrationAvailability::Ready;
        skill.health.diagnostic_codes.clear();
        skill.requirements.clear();
        catalog
    }

    fn skill_control_script() -> String {
        r#"
read -r _initialize
printf '%s\n' '{"id":1,"result":{}}'
read -r _skills
case "$_skills" in
  *'"method":"skills/list"'*) ;;
  *) exit 61 ;;
esac
printf '%s\n' '{"id":2,"result":{"data":[{"cwd":"/fixture/project","errors":[],"skills":[{"description":"Project checks.","enabled":false,"name":"fixture-project-checks","path":"/fixture/project/SKILL.md","scope":"repo"}]}]}}'
read -r _write || exit 0
case "$_write" in
  *'"method":"skills/config/write"'*'"enabled":true'*'"path":"/fixture/project/SKILL.md"'*) ;;
  *) exit 62 ;;
esac
printf '%s\n' '{"id":3,"result":{"effectiveEnabled":true}}'
read -r _verify
case "$_verify" in
  *'"method":"skills/list"'*) ;;
  *) exit 63 ;;
esac
printf '%s\n' '{"id":4,"result":{"data":[{"cwd":"/fixture/project","errors":[],"skills":[{"description":"Project checks.","enabled":true,"name":"fixture-project-checks","path":"/fixture/project/SKILL.md","scope":"repo"}]}]}}'
read -r _keep_open
"#
        .to_owned()
    }

    fn mcp_control_script(completion_name: &str, success: bool) -> String {
        format!(
            r#"
read -r _initialize
printf '%s\n' '{{"id":1,"result":{{}}}}'
read -r _mcp
case "$_mcp" in
  *'"method":"mcpServerStatus/list"'*) ;;
  *) exit 71 ;;
esac
printf '%s\n' '{{"id":2,"result":{{"data":[{{"authStatus":"notLoggedIn","name":"fixture-knowledge","resourceTemplates":[],"resources":[],"serverInfo":{{}},"tools":[]}}],"nextCursor":null}}}}'
read -r _oauth || exit 0
case "$_oauth" in
  *'"method":"mcpServer/oauth/login"'*'"name":"fixture-knowledge"'*) ;;
  *) exit 72 ;;
esac
printf '%s\n' '{{"id":3,"result":{{"authorizationUrl":"https://example.invalid/oauth?state=browser-secret"}}}}'
printf '%s\n' '{{"method":"mcpServer/oauthLogin/completed","params":{{"name":"{completion_name}","success":{success},"error":null,"threadId":null}}}}'
read -r _keep_open
"#
        )
    }

    fn connector_control_script(accessible: bool, enabled: bool, callable: bool) -> String {
        format!(
            r#"
read -r _initialize
printf '%s\n' '{{"id":1,"result":{{}}}}'
read -r _apps
printf '%s\n' '{{"id":2,"result":{{"data":[{{"description":"Calendar access.","id":"fixture-calendar","isAccessible":{accessible},"isEnabled":true,"name":"Fixture calendar connector"}}],"nextCursor":null}}}}'
read -r _installed
printf '%s\n' '{{"id":3,"result":{{"apps":[{{"callable":{callable},"enabled":{enabled},"id":"fixture-calendar","runtimeName":"fixture-calendar"}}]}}}}'
read -r _keep_open
"#
        )
    }
}
