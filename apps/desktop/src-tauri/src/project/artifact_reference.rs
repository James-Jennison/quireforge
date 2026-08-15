//! M65's metadata-only references to transient M48 artifacts. No bytes or paths cross this boundary.
use super::{
    durable_source::binding,
    storage::{ArtifactReferenceInsert, ProjectRepository},
    types::{
        ArtifactReferenceAvailability, ArtifactReferenceConfirmRequest,
        ArtifactReferenceDeleteConfirmRequest, ArtifactReferenceDeletePrepareRequest,
        ArtifactReferenceDiagnosticCode, ArtifactReferencePreparation,
        ArtifactReferencePrepareRequest, ArtifactReferenceSummary,
    },
};
use crate::advisor_generated_artifact::{
    AdvisorGeneratedArtifactService, GeneratedArtifactClaimRequest,
};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use uuid::Uuid;

const TTL: Duration = Duration::from_secs(5 * 60);
enum PendingKind {
    Create {
        project_id: String,
        task_id: Option<String>,
        artifact_id: String,
        sha256: String,
        class: String,
        label: String,
    },
    Delete {
        reference_id: String,
    },
}
struct Pending {
    nonce: String,
    expires: Instant,
    kind: PendingKind,
}
#[derive(Default)]
pub(crate) struct ArtifactReferenceController {
    pending: HashMap<String, Pending>,
}
impl ArtifactReferenceController {
    fn expire(&mut self) {
        self.pending.retain(|_, item| item.expires > Instant::now());
    }
    pub(crate) fn observe(
        &self,
        mut reference: ArtifactReferenceSummary,
        artifacts: &AdvisorGeneratedArtifactService,
    ) -> ArtifactReferenceSummary {
        let claim = GeneratedArtifactClaimRequest {
            artifact_id: reference.artifact_id.clone(),
            manifest_sha256: reference.artifact_sha256.clone(),
        };
        reference.availability = if artifacts.local_review_metadata_source(&claim).is_ok() {
            ArtifactReferenceAvailability::Live
        } else {
            ArtifactReferenceAvailability::Unavailable
        };
        reference
    }
    pub(crate) fn prepare(
        &mut self,
        repository: &ProjectRepository,
        request: ArtifactReferencePrepareRequest,
        artifacts: &AdvisorGeneratedArtifactService,
    ) -> ArtifactReferencePreparation {
        if binding(repository, &request.project_id, request.task_id.as_deref()).is_err() {
            return ArtifactReferencePreparation::unavailable(
                ArtifactReferenceDiagnosticCode::ProjectTaskMismatch,
            );
        }
        let claim = GeneratedArtifactClaimRequest {
            artifact_id: request.artifact_id.clone(),
            manifest_sha256: request.artifact_sha256.clone(),
        };
        let source = match artifacts.local_review_metadata_source(&claim) {
            Ok(value) => value,
            Err(_) => {
                return ArtifactReferencePreparation::unavailable(
                    ArtifactReferenceDiagnosticCode::ArtifactUnavailable,
                )
            }
        };
        self.expire();
        let id = Uuid::now_v7().to_string();
        let nonce = Uuid::now_v7().to_string();
        let class = format!("{:?}", source.class).to_lowercase();
        let label = source.display_label;
        self.pending.insert(
            id.clone(),
            Pending {
                nonce: nonce.clone(),
                expires: Instant::now() + TTL,
                kind: PendingKind::Create {
                    project_id: request.project_id.clone(),
                    task_id: request.task_id.clone(),
                    artifact_id: request.artifact_id.clone(),
                    sha256: request.artifact_sha256.clone(),
                    class: class.clone(),
                    label: label.clone(),
                },
            },
        );
        ArtifactReferencePreparation {
            schema_version: 1,
            preparation_id: id,
            nonce,
            expires_at_ms: super::storage::now_millis() + TTL.as_millis() as i64,
            reference_id: None,
            project_id: request.project_id,
            task_id: request.task_id,
            artifact_id: request.artifact_id,
            artifact_sha256: request.artifact_sha256,
            artifact_class: class,
            display_label: label,
            diagnostic_code: None,
        }
    }
    pub(crate) fn confirm(
        &mut self,
        repository: &mut ProjectRepository,
        request: ArtifactReferenceConfirmRequest,
        artifacts: &AdvisorGeneratedArtifactService,
    ) -> Result<ArtifactReferenceSummary, ArtifactReferenceDiagnosticCode> {
        self.expire();
        let pending = self
            .pending
            .remove(&request.preparation_id)
            .ok_or(ArtifactReferenceDiagnosticCode::PreparationMissing)?;
        let PendingKind::Create {
            project_id,
            task_id,
            artifact_id,
            sha256,
            class,
            label,
        } = pending.kind
        else {
            return Err(ArtifactReferenceDiagnosticCode::ConfirmationMismatch);
        };
        if pending.nonce != request.nonce || sha256 != request.artifact_sha256 {
            return Err(ArtifactReferenceDiagnosticCode::ConfirmationMismatch);
        }
        if binding(repository, &project_id, task_id.as_deref()).is_err() {
            return Err(ArtifactReferenceDiagnosticCode::ProjectTaskMismatch);
        }
        let claim = GeneratedArtifactClaimRequest {
            artifact_id: artifact_id.clone(),
            manifest_sha256: sha256.clone(),
        };
        let source = artifacts
            .local_review_metadata_source(&claim)
            .map_err(|_| ArtifactReferenceDiagnosticCode::ArtifactUnavailable)?;
        if format!("{:?}", source.class).to_lowercase() != class || source.display_label != label {
            return Err(ArtifactReferenceDiagnosticCode::ArtifactMismatch);
        }
        repository
            .artifact_reference_insert(ArtifactReferenceInsert {
                id: &Uuid::now_v7().to_string(),
                project_id: &project_id,
                task_id: task_id.as_deref(),
                artifact_id: &artifact_id,
                artifact_sha256: &sha256,
                artifact_class: &class,
                display_label: &label,
            })
            .map_err(|_| ArtifactReferenceDiagnosticCode::PrivateStorageFailure)
    }
    pub(crate) fn prepare_delete(
        &mut self,
        repository: &ProjectRepository,
        request: ArtifactReferenceDeletePrepareRequest,
    ) -> ArtifactReferencePreparation {
        match repository.artifact_reference(&request.reference_id) {
            Ok(Some(reference))
                if matches!(
                    reference.state,
                    super::types::ArtifactReferenceState::Active
                ) => {}
            _ => {
                return ArtifactReferencePreparation::unavailable(
                    ArtifactReferenceDiagnosticCode::ReferenceUnavailable,
                )
            }
        }
        self.expire();
        let id = Uuid::now_v7().to_string();
        let nonce = Uuid::now_v7().to_string();
        self.pending.insert(
            id.clone(),
            Pending {
                nonce: nonce.clone(),
                expires: Instant::now() + TTL,
                kind: PendingKind::Delete {
                    reference_id: request.reference_id.clone(),
                },
            },
        );
        ArtifactReferencePreparation {
            schema_version: 1,
            preparation_id: id,
            nonce,
            expires_at_ms: super::storage::now_millis() + TTL.as_millis() as i64,
            reference_id: Some(request.reference_id),
            project_id: String::new(),
            task_id: None,
            artifact_id: String::new(),
            artifact_sha256: String::new(),
            artifact_class: String::new(),
            display_label: String::new(),
            diagnostic_code: None,
        }
    }
    pub(crate) fn confirm_delete(
        &mut self,
        repository: &mut ProjectRepository,
        request: ArtifactReferenceDeleteConfirmRequest,
    ) -> Result<(), ArtifactReferenceDiagnosticCode> {
        self.expire();
        let pending = self
            .pending
            .remove(&request.preparation_id)
            .ok_or(ArtifactReferenceDiagnosticCode::PreparationMissing)?;
        let PendingKind::Delete { reference_id } = pending.kind else {
            return Err(ArtifactReferenceDiagnosticCode::ConfirmationMismatch);
        };
        if pending.nonce != request.nonce || reference_id != request.reference_id {
            return Err(ArtifactReferenceDiagnosticCode::ConfirmationMismatch);
        }
        repository
            .artifact_reference_delete(&reference_id)
            .map_err(|_| ArtifactReferenceDiagnosticCode::ReferenceUnavailable)
    }
}
