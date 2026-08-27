//! Internal-only package-validation handoff.
//!
//! This module deliberately has no Tauri command, serde request type, or
//! frontend surface.  It is reserved for a future native packaging workflow
//! that can prove validation results without granting scripts database or
//! project-selection authority.
#![allow(dead_code)] // Reserved for the separately approved native workflow.

use std::io::Write;
use std::{
    collections::{BTreeSet, HashSet},
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

use super::{
    storage::{
        PackageValidationInstalledHostFacts, PackageValidationPhase, PackageValidationRecordInput,
        PackageValidationRecordOutcome, ProjectRepository,
    },
    types::LocalReviewEvidenceCheckState,
    ProjectService,
};

const PROTOCOL_VERSION: u8 = 1;
const NONCE_BYTES: usize = 32;
const RESULT_MAX_BYTES: usize = 4096;
const SESSION_TTL: Duration = Duration::from_secs(5 * 60);
const MAX_ACTIVE_RUNS: usize = 4;
const STAGE_TIMEOUT: Duration = Duration::from_secs(30);
const CANDIDATE_IDENTITY_DOMAIN: &str = "quireforge-package-candidate-identity-v1";
const INSTALLED_HOST_HELPER: &str = "/usr/local/sbin/quireforge-validate-deb";
const SUDO: &str = "/usr/bin/sudo";
const DPKG_QUERY: &str = "/usr/bin/dpkg-query";

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct InstalledHostRequest<'a> {
    schema_version: u8,
    session_id: &'a str,
    nonce: &'a str,
    expected_application_version: &'a str,
    expected_debian_version: &'a str,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
struct InstalledHostResult {
    schema_version: u8,
    session_id: String,
    nonce: String,
    outcome: Outcome,
    facts: Option<InstalledHostFacts>,
    result_sha256: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
struct InstalledHostFacts {
    kind: String,
    schema_version: u8,
    package_state: String,
    version_match: bool,
    ownership_verified: bool,
    permissions_safe: bool,
    package_integrity_verified: bool,
}

#[cfg(test)]
struct InstalledHostProcessResult {
    exit_code: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct VerifiedPackageCandidate {
    candidate_identity_sha256: String,
    application_version: String,
    debian_version: String,
    artifacts: Vec<VerifiedPackageArtifact>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct VerifiedPackageArtifact {
    format: CandidateArtifactFormat,
    sha256: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum CandidateArtifactFormat {
    Deb,
    SandboxdDeb,
}

impl CandidateArtifactFormat {
    fn name(self) -> &'static str {
        match self {
            Self::Deb => "deb",
            Self::SandboxdDeb => "sandboxd-deb",
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CandidateManifest {
    schema_version: u8,
    state: String,
    version: String,
    artifacts: Vec<CandidateManifestArtifact>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CandidateManifestArtifact {
    format: CandidateArtifactFormat,
    filename: String,
    architecture: String,
    package_version: String,
    sha256: String,
    size: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalCandidateIdentity<'a> {
    domain: &'static str,
    application_version: &'a str,
    debian_version: &'a str,
    artifact_count: u8,
    artifacts: Vec<CanonicalCandidateArtifact<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalCandidateArtifact<'a> {
    format: &'static str,
    sha256: &'a str,
}

fn verify_package_candidate(
    root: &Path,
) -> Result<VerifiedPackageCandidate, PackageValidationControllerError> {
    let root_metadata =
        fs::symlink_metadata(root).map_err(|_| PackageValidationControllerError::InvalidResult)?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err(PackageValidationControllerError::InvalidResult);
    }
    let root =
        fs::canonicalize(root).map_err(|_| PackageValidationControllerError::InvalidResult)?;
    let manifest_path = root.join("release-manifest.json");
    let manifest_metadata = fs::symlink_metadata(&manifest_path)
        .map_err(|_| PackageValidationControllerError::InvalidResult)?;
    if manifest_metadata.file_type().is_symlink() || !manifest_metadata.is_file() {
        return Err(PackageValidationControllerError::InvalidResult);
    }
    let manifest_path = fs::canonicalize(&manifest_path)
        .map_err(|_| PackageValidationControllerError::InvalidResult)?;
    if manifest_path.parent() != Some(root.as_path()) {
        return Err(PackageValidationControllerError::InvalidResult);
    }
    let manifest: CandidateManifest = serde_json::from_slice(
        &fs::read(&manifest_path).map_err(|_| PackageValidationControllerError::InvalidResult)?,
    )
    .map_err(|_| PackageValidationControllerError::InvalidResult)?;
    if manifest.schema_version != 3
        || manifest.state != "release-candidate"
        || !valid_version(&manifest.version, false)
        || manifest.artifacts.len() != 2
    {
        return Err(PackageValidationControllerError::InvalidResult);
    }
    let debian_version = manifest.version.replacen('-', "~", 1);
    if !valid_version(&debian_version, true) {
        return Err(PackageValidationControllerError::InvalidResult);
    }

    let mut seen_formats = HashSet::new();
    let mut seen_names = HashSet::new();
    let mut artifacts = Vec::with_capacity(2);
    for artifact in manifest.artifacts {
        if !seen_formats.insert(artifact.format)
            || !seen_names.insert(artifact.filename.clone())
            || artifact.filename.is_empty()
            || artifact.filename == "."
            || artifact.filename == ".."
            || artifact.filename.contains('/')
            || artifact.filename.contains('\\')
            || artifact.architecture != "x86_64"
            || artifact.package_version != debian_version
            || !valid_lower_sha256(&artifact.sha256)
        {
            return Err(PackageValidationControllerError::InvalidResult);
        }
        let candidate_path = root.join(&artifact.filename);
        let metadata = fs::symlink_metadata(&candidate_path)
            .map_err(|_| PackageValidationControllerError::InvalidResult)?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.len() != artifact.size
        {
            return Err(PackageValidationControllerError::InvalidResult);
        }
        let artifact_path = fs::canonicalize(&candidate_path)
            .map_err(|_| PackageValidationControllerError::InvalidResult)?;
        if artifact_path.parent() != Some(root.as_path()) {
            return Err(PackageValidationControllerError::InvalidResult);
        }
        let digest = format!(
            "{:x}",
            Sha256::digest(
                fs::read(&artifact_path)
                    .map_err(|_| PackageValidationControllerError::InvalidResult)?,
            )
        );
        if digest != artifact.sha256 {
            return Err(PackageValidationControllerError::InvalidResult);
        }
        artifacts.push(VerifiedPackageArtifact {
            format: artifact.format,
            sha256: digest,
        });
    }
    artifacts.sort_by_key(|artifact| artifact.format);
    if artifacts
        .iter()
        .map(|artifact| artifact.format)
        .collect::<Vec<_>>()
        != vec![
            CandidateArtifactFormat::Deb,
            CandidateArtifactFormat::SandboxdDeb,
        ]
    {
        return Err(PackageValidationControllerError::InvalidResult);
    }
    let canonical =
        canonical_candidate_identity_bytes(&manifest.version, &debian_version, &artifacts)?;
    Ok(VerifiedPackageCandidate {
        candidate_identity_sha256: format!("{:x}", Sha256::digest(canonical)),
        application_version: manifest.version,
        debian_version,
        artifacts,
    })
}

fn canonical_candidate_identity_bytes(
    application_version: &str,
    debian_version: &str,
    artifacts: &[VerifiedPackageArtifact],
) -> Result<Vec<u8>, PackageValidationControllerError> {
    serde_json::to_vec(&CanonicalCandidateIdentity {
        domain: CANDIDATE_IDENTITY_DOMAIN,
        application_version,
        debian_version,
        artifact_count: artifacts.len() as u8,
        artifacts: artifacts
            .iter()
            .map(|artifact| CanonicalCandidateArtifact {
                format: artifact.format.name(),
                sha256: &artifact.sha256,
            })
            .collect(),
    })
    .map_err(|_| PackageValidationControllerError::InvalidResult)
}

/// Kept private to native project code: it carries the trusted project root
/// and opaque identity, neither of which may cross IPC or a child protocol.
#[derive(Clone, Debug)]
pub(super) struct TrustedValidationContext {
    project_id: String,
    project_root: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Stage {
    Manifest,
    Checksum,
    Abi,
    Provenance,
    VisibleLaunch,
}
impl Stage {
    fn name(self) -> &'static str {
        match self {
            Self::Manifest => "manifest",
            Self::Checksum => "checksum",
            Self::Abi => "abi",
            Self::Provenance => "provenance",
            Self::VisibleLaunch => "visible-launch",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Outcome {
    Passed,
    Failed,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum StageFacts {
    Abi {
        schema_version: u8,
        glibc_baseline: String,
        highest_required: String,
    },
    Provenance {
        schema_version: u8,
        evidence_state: String,
        artifact_coverage: u8,
        identity_consistent: bool,
    },
    VisibleLaunch {
        schema_version: u8,
        launch_state: String,
        artifact_coverage: u8,
        visibility_confirmed: bool,
        lifecycle_clean: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FixedStageOperation {
    Abi,
    Provenance,
    VisibleLaunch,
}

fn stage_state(
    stage: Stage,
    outcome: Outcome,
) -> Result<LocalReviewEvidenceCheckState, PackageValidationControllerError> {
    match outcome {
        Outcome::Passed if matches!(stage, Stage::Manifest | Stage::Checksum) => {
            Ok(LocalReviewEvidenceCheckState::Passed)
        }
        Outcome::Passed => Err(PackageValidationControllerError::InvalidResult),
        Outcome::Failed => Ok(LocalReviewEvidenceCheckState::Failed),
        Outcome::Unavailable => Ok(LocalReviewEvidenceCheckState::Unavailable),
    }
}

fn verified_stage_state(
    stage: Stage,
    outcome: Outcome,
    facts: Option<&StageFacts>,
    operation: Option<FixedStageOperation>,
) -> Result<LocalReviewEvidenceCheckState, PackageValidationControllerError> {
    match (stage, outcome, facts, operation) {
        (Stage::Manifest | Stage::Checksum, Outcome::Passed, None, None) => {
            stage_state(stage, outcome)
        }
        (
            Stage::Abi,
            Outcome::Passed,
            Some(StageFacts::Abi {
                schema_version: 1,
                glibc_baseline,
                highest_required,
            }),
            Some(FixedStageOperation::Abi),
        ) if glibc_baseline == "GLIBC_2.35"
            && valid_glibc_requirement(highest_required)
                .is_some_and(|required| required <= (2, 35)) =>
        {
            Ok(LocalReviewEvidenceCheckState::Passed)
        }
        (
            Stage::Provenance,
            Outcome::Passed,
            Some(StageFacts::Provenance {
                schema_version: 1,
                evidence_state,
                artifact_coverage: 2,
                identity_consistent: true,
            }),
            Some(FixedStageOperation::Provenance),
        ) if evidence_state == "pinned-release-candidate" => {
            Ok(LocalReviewEvidenceCheckState::Passed)
        }
        (
            Stage::VisibleLaunch,
            Outcome::Passed,
            Some(StageFacts::VisibleLaunch {
                schema_version: 1,
                launch_state,
                artifact_coverage: 1,
                visibility_confirmed: true,
                lifecycle_clean: true,
            }),
            Some(FixedStageOperation::VisibleLaunch),
        ) if launch_state == "visible-window-confirmed" => {
            Ok(LocalReviewEvidenceCheckState::Passed)
        }
        (_, Outcome::Passed, _, _) => Err(PackageValidationControllerError::InvalidResult),
        (_, Outcome::Failed | Outcome::Unavailable, None, None) => stage_state(stage, outcome),
        _ => Err(PackageValidationControllerError::InvalidResult),
    }
}

fn fixed_adapter_operation(stage: Stage) -> Option<FixedStageOperation> {
    match stage {
        Stage::Abi => Some(FixedStageOperation::Abi),
        Stage::Provenance => Some(FixedStageOperation::Provenance),
        Stage::VisibleLaunch => Some(FixedStageOperation::VisibleLaunch),
        Stage::Manifest | Stage::Checksum => None,
    }
}

/// The only child-to-controller payload. It intentionally excludes paths,
/// filenames, commands, output, project identity, and database information.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StageResultV1 {
    schema_version: u8,
    session_id: String,
    nonce: String,
    stage: Stage,
    outcome: Outcome,
    application_version: String,
    debian_version: String,
    artifact_count: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    facts: Option<StageFacts>,
    result_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct UnsignedStageResult<'a> {
    schema_version: u8,
    session_id: &'a str,
    nonce: &'a str,
    stage: Stage,
    outcome: Outcome,
    application_version: &'a str,
    debian_version: &'a str,
    artifact_count: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    facts: Option<&'a StageFacts>,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub(crate) enum PackageValidationControllerError {
    #[error("trusted validation context is unavailable")]
    ContextUnavailable,
    #[error("validation run is unavailable")]
    Unavailable,
    #[error("validation result is invalid")]
    InvalidResult,
    #[error("validation result is expired")]
    Expired,
    #[error("validation stage is duplicated or out of order")]
    InvalidStageOrder,
    #[error("validation result channel is unsafe")]
    UnsafeChannel,
}

/// Bounded installed-host completion result. It intentionally carries no
/// receipt, project, package, or helper detail across the executable boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InstalledHostValidationOutcome {
    Created,
    Existing,
    Failed,
    Unavailable,
}

/// Fixed, process-owned controller. `begin` is intentionally crate-private;
/// future callers must first obtain a trusted conversation/workspace context.
#[derive(Default)]
pub(crate) struct PackageValidationController {
    active_projects: BTreeSet<String>,
}

impl PackageValidationController {
    pub(crate) fn trusted_context_from_live_project(
        projects: &ProjectService,
        project_id: &str,
    ) -> Result<TrustedValidationContext, PackageValidationControllerError> {
        let root = projects
            .review_root(project_id)
            .map_err(|_| PackageValidationControllerError::ContextUnavailable)?;
        Ok(TrustedValidationContext {
            project_id: project_id.to_owned(),
            project_root: root.attached_root,
        })
    }

    pub(crate) fn trusted_context_from_conversation(
        projects: &ProjectService,
        conversation_id: &str,
    ) -> Result<TrustedValidationContext, PackageValidationControllerError> {
        let reference = projects
            .conversation_reference(conversation_id)
            .map_err(|_| PackageValidationControllerError::ContextUnavailable)?;
        let root = projects
            .review_root(&reference.project_id)
            .map_err(|_| PackageValidationControllerError::ContextUnavailable)?;
        Ok(TrustedValidationContext {
            project_id: reference.project_id,
            project_root: root.attached_root,
        })
    }

    pub(crate) fn begin(
        &mut self,
        context: TrustedValidationContext,
    ) -> Result<ValidationSession, PackageValidationControllerError> {
        if self.active_projects.len() >= MAX_ACTIVE_RUNS
            || !self.active_projects.insert(context.project_id.clone())
        {
            return Err(PackageValidationControllerError::Unavailable);
        }
        let project_id = context.project_id.clone();
        ValidationSession::new(context).inspect_err(|_| {
            self.active_projects.retain(|id| id != &project_id);
        })
    }

    pub(crate) fn finish(&mut self, session: ValidationSession) {
        self.active_projects.remove(&session.context.project_id);
        let _ = fs::remove_dir_all(session.channel_dir);
    }

    /// The only production runner: fixed interpreter, fixed adapter, fixed
    /// ordered stages, minimal environment, and no IPC-supplied parameters.
    pub(crate) fn run_and_record(
        &mut self,
        repository: &mut ProjectRepository,
        context: TrustedValidationContext,
    ) -> Result<PackageValidationRecordOutcome, PackageValidationControllerError> {
        let mut session = self.begin(context)?;
        let result = (|| {
            let candidate_root = session
                .context
                .project_root
                .join("target/ubuntu-22.04/release/packages");
            let verified_candidate = verify_package_candidate(&candidate_root)?;
            let mut accepted = Vec::new();
            for stage in [
                Stage::Manifest,
                Stage::Checksum,
                Stage::Abi,
                Stage::Provenance,
                Stage::VisibleLaunch,
            ] {
                let result_file = session.channel_dir.join(format!("{}.json", stage.name()));
                let mut command = Command::new("python3");
                command
                    .arg("scripts/package_validation_stage_adapter.py")
                    .arg("--stage")
                    .arg(stage.name())
                    .arg("--session-id")
                    .arg(&session.id)
                    .arg("--nonce")
                    .arg(&session.nonce)
                    .arg("--candidate-root")
                    .arg(&candidate_root)
                    .arg("--result-file")
                    .arg(&result_file)
                    .current_dir(&session.context.project_root)
                    .env_clear()
                    .env("PATH", "/usr/bin:/bin")
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null());
                // The visible-launch stage opens the packaged desktop app on
                // the already-selected local display. Keep the child closed
                // to every other ambient variable; these two values are used
                // only for that local X11 connection and are never recorded.
                for key in ["DISPLAY", "XAUTHORITY"] {
                    if let Some(value) = std::env::var_os(key) {
                        command.env(key, value);
                    }
                }
                let mut child = command
                    .spawn()
                    .map_err(|_| PackageValidationControllerError::Unavailable)?;
                let status = wait_for_stage(&mut child)?;
                if !status.success() {
                    return Err(PackageValidationControllerError::InvalidResult);
                }
                if fs::read_dir(&session.channel_dir)
                    .map_err(|_| PackageValidationControllerError::InvalidResult)?
                    .count()
                    != session.accepted.len() + 1
                {
                    return Err(PackageValidationControllerError::InvalidResult);
                }
                let bytes = fs::read(result_file)
                    .map_err(|_| PackageValidationControllerError::InvalidResult)?;
                accepted.push(session.accept(&bytes, 0)?);
            }
            record_verified_package_candidate(
                repository,
                &session.context.project_id,
                &candidate_root,
                &verified_candidate,
                &accepted,
            )
        })();
        self.finish(session);
        result
    }

    /// Read-only installed-package gate for the fixed executable bootstrap.
    /// The version never comes from argv, metadata, or the candidate directory.
    pub(crate) fn installed_debian_version_is(
        expected_debian_version: &str,
    ) -> Result<bool, PackageValidationControllerError> {
        let output = Command::new(DPKG_QUERY)
            .args([
                "--showformat=${db:Status-Status}\\n${Version}\\n",
                "--show",
                "quireforge",
            ])
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .current_dir("/")
            .output()
            .map_err(|_| PackageValidationControllerError::Unavailable)?;
        if !output.status.success() || output.stdout.len() > 256 {
            return Ok(false);
        }
        Ok(std::str::from_utf8(&output.stdout)
            .ok()
            .is_some_and(|value| value == format!("installed\n{expected_debian_version}\n")))
    }

    /// Internal-only installed-host phase. The predecessor receipt is native
    /// durable state, never an IPC or frontend input.
    pub(crate) fn run_installed_host_and_record(
        &mut self,
        repository: &mut ProjectRepository,
        context: TrustedValidationContext,
        predecessor_receipt_id: &str,
    ) -> Result<InstalledHostValidationOutcome, PackageValidationControllerError> {
        self.run_installed_host_with(repository, context, predecessor_receipt_id, |request| {
            run_installed_host_process(request)
        })
    }

    fn run_installed_host_with<F>(
        &mut self,
        repository: &mut ProjectRepository,
        context: TrustedValidationContext,
        predecessor_receipt_id: &str,
        run: F,
    ) -> Result<InstalledHostValidationOutcome, PackageValidationControllerError>
    where
        F: FnOnce(&[u8]) -> Result<(i32, Vec<u8>, Vec<u8>), PackageValidationControllerError>,
    {
        let session = self.begin(context)?;
        let result = (|| {
            let root_predecessor = repository
                .package_validation_summary_for_internal(predecessor_receipt_id)
                .map_err(|_| PackageValidationControllerError::InvalidResult)?;
            if root_predecessor.project_id != session.context.project_id
                || root_predecessor.input.validation_phase != PackageValidationPhase::Unprivileged
                || root_predecessor.input.validation_complete
                || root_predecessor.input.installed_host_state
                    != LocalReviewEvidenceCheckState::Unavailable
            {
                return Err(PackageValidationControllerError::InvalidResult);
            }
            let predecessor = repository
                .package_validation_installed_host_predecessor_for_internal(
                    &session.context.project_id,
                    &root_predecessor.input.candidate_identity_sha256,
                )
                .map_err(|_| PackageValidationControllerError::InvalidResult)?;
            if predecessor.project_id != root_predecessor.project_id
                || predecessor.input.candidate_identity_sha256
                    != root_predecessor.input.candidate_identity_sha256
                || predecessor.input.application_version
                    != root_predecessor.input.application_version
                || predecessor.input.debian_version != root_predecessor.input.debian_version
                || predecessor.input.artifact_count != root_predecessor.input.artifact_count
                || predecessor.input.manifest_state != root_predecessor.input.manifest_state
                || predecessor.input.checksum_state != root_predecessor.input.checksum_state
                || predecessor.input.abi_state != root_predecessor.input.abi_state
                || predecessor.input.provenance_state != root_predecessor.input.provenance_state
                || predecessor.input.visible_launch_state
                    != root_predecessor.input.visible_launch_state
            {
                return Err(PackageValidationControllerError::InvalidResult);
            }
            let request = InstalledHostRequest {
                schema_version: 1,
                session_id: &session.id,
                nonce: &session.nonce,
                expected_application_version: &predecessor.input.application_version,
                expected_debian_version: &predecessor.input.debian_version,
            };
            let request = serde_json::to_vec(&request)
                .map_err(|_| PackageValidationControllerError::InvalidResult)?;
            if request.len() > RESULT_MAX_BYTES {
                return Err(PackageValidationControllerError::InvalidResult);
            }
            let (exit_code, stdout, _stderr) = run(&request)?;
            let result =
                verify_installed_host_result(&stdout, exit_code, &session.id, &session.nonce)?;
            let installed_host_state = match result.outcome {
                Outcome::Passed => LocalReviewEvidenceCheckState::Passed,
                Outcome::Failed => LocalReviewEvidenceCheckState::Failed,
                Outcome::Unavailable => LocalReviewEvidenceCheckState::Unavailable,
            };
            let facts = result
                .facts
                .map(|facts| PackageValidationInstalledHostFacts {
                    package_state: facts.package_state,
                    version_match: facts.version_match,
                    ownership_verified: facts.ownership_verified,
                    permissions_safe: facts.permissions_safe,
                    package_integrity_verified: facts.package_integrity_verified,
                });
            // The helper runs outside SQLite's write transaction. Re-read the
            // immutable tail immediately before recording so a stale or raced
            // predecessor can never be superseded.
            let current = repository
                .package_validation_installed_host_predecessor_for_internal(
                    &session.context.project_id,
                    &root_predecessor.input.candidate_identity_sha256,
                )
                .map_err(|_| PackageValidationControllerError::InvalidResult)?;
            if current.id != predecessor.id
                || current.record_sha256 != predecessor.record_sha256
                || current.input != predecessor.input
            {
                return Err(PackageValidationControllerError::InvalidResult);
            }
            let authoritative = repository
                .record_package_validation_summary(
                    &session.context.project_id,
                    PackageValidationRecordInput {
                        candidate_identity_sha256: predecessor
                            .input
                            .candidate_identity_sha256
                            .clone(),
                        validation_phase: PackageValidationPhase::InstalledHost,
                        attempt_identity_sha256: None,
                        installed_host_facts: facts,
                        application_version: predecessor.input.application_version.clone(),
                        debian_version: predecessor.input.debian_version.clone(),
                        manifest_state: predecessor.input.manifest_state,
                        checksum_state: predecessor.input.checksum_state,
                        abi_state: predecessor.input.abi_state,
                        provenance_state: predecessor.input.provenance_state,
                        visible_launch_state: predecessor.input.visible_launch_state,
                        installed_host_state,
                        artifact_count: predecessor.input.artifact_count,
                        validation_complete: installed_host_state
                            == LocalReviewEvidenceCheckState::Passed
                            && predecessor.input.artifact_count == 2
                            && [
                                predecessor.input.manifest_state,
                                predecessor.input.checksum_state,
                                predecessor.input.abi_state,
                                predecessor.input.provenance_state,
                                predecessor.input.visible_launch_state,
                            ]
                            .into_iter()
                            .all(|state| state == LocalReviewEvidenceCheckState::Passed),
                        supersedes_record_id: Some(predecessor.id),
                    },
                )
                .map_err(|_| PackageValidationControllerError::Unavailable)?;
            Ok(match (result.outcome, authoritative) {
                (Outcome::Failed, _) => InstalledHostValidationOutcome::Failed,
                (Outcome::Unavailable, _) => InstalledHostValidationOutcome::Unavailable,
                (Outcome::Passed, PackageValidationRecordOutcome::Created(_)) => {
                    InstalledHostValidationOutcome::Created
                }
                (Outcome::Passed, PackageValidationRecordOutcome::Existing(_)) => {
                    InstalledHostValidationOutcome::Existing
                }
            })
        })();
        self.finish(session);
        result
    }
}

fn run_installed_host_process(
    request: &[u8],
) -> Result<(i32, Vec<u8>, Vec<u8>), PackageValidationControllerError> {
    let mut child = Command::new(SUDO)
        .args(["-n", INSTALLED_HOST_HELPER])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .current_dir("/")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| PackageValidationControllerError::Unavailable)?;
    child
        .stdin
        .as_mut()
        .ok_or(PackageValidationControllerError::Unavailable)?
        .write_all(request)
        .map_err(|_| PackageValidationControllerError::Unavailable)?;
    drop(child.stdin.take());
    let status = wait_for_stage(&mut child)?;
    let output = child
        .wait_with_output()
        .map_err(|_| PackageValidationControllerError::Unavailable)?;
    if output.stdout.len() > RESULT_MAX_BYTES || output.stderr.len() > RESULT_MAX_BYTES {
        return Err(PackageValidationControllerError::InvalidResult);
    }
    Ok((status.code().unwrap_or(-1), output.stdout, output.stderr))
}

fn verify_installed_host_result(
    bytes: &[u8],
    exit_code: i32,
    session_id: &str,
    nonce: &str,
) -> Result<InstalledHostResult, PackageValidationControllerError> {
    if exit_code != 0 || bytes.len() > RESULT_MAX_BYTES {
        return Err(PackageValidationControllerError::InvalidResult);
    }
    let result: InstalledHostResult = serde_json::from_slice(bytes)
        .map_err(|_| PackageValidationControllerError::InvalidResult)?;
    if result.schema_version != 1
        || result.session_id != session_id
        || result.nonce != nonce
        || installed_host_result_digest(&result) != result.result_sha256
    {
        return Err(PackageValidationControllerError::InvalidResult);
    }
    match (&result.outcome, &result.facts) {
        (Outcome::Passed, Some(facts))
            if facts.kind == "installed-host"
                && facts.schema_version == 1
                && facts.package_state == "installed"
                && facts.version_match
                && facts.ownership_verified
                && facts.permissions_safe
                && facts.package_integrity_verified =>
        {
            Ok(result)
        }
        (Outcome::Failed | Outcome::Unavailable, None) => Ok(result),
        _ => Err(PackageValidationControllerError::InvalidResult),
    }
}

fn installed_host_result_digest(value: &InstalledHostResult) -> String {
    #[derive(Serialize)]
    #[serde(rename_all = "snake_case")]
    struct Unsigned<'a> {
        schema_version: u8,
        session_id: &'a str,
        nonce: &'a str,
        outcome: Outcome,
        facts: Option<&'a InstalledHostFacts>,
    }
    let bytes = serde_json::to_vec(
        &serde_json::to_value(Unsigned {
            schema_version: value.schema_version,
            session_id: &value.session_id,
            nonce: &value.nonce,
            outcome: value.outcome,
            facts: value.facts.as_ref(),
        })
        .expect("fixed protocol"),
    )
    .expect("fixed protocol");
    format!("{:x}", Sha256::digest(bytes))
}

fn wait_for_stage(
    child: &mut std::process::Child,
) -> Result<std::process::ExitStatus, PackageValidationControllerError> {
    wait_for_stage_until(child, STAGE_TIMEOUT)
}

fn wait_for_stage_until(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Result<std::process::ExitStatus, PackageValidationControllerError> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|_| PackageValidationControllerError::Unavailable)?
        {
            return Ok(status);
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(PackageValidationControllerError::Expired);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn record_verified_package_candidate(
    repository: &mut ProjectRepository,
    project_id: &str,
    candidate_root: &Path,
    initial: &VerifiedPackageCandidate,
    stages: &[StageResultV1],
) -> Result<PackageValidationRecordOutcome, PackageValidationControllerError> {
    require_verified_candidate_unchanged(candidate_root, initial, stages)?;
    let authoritative = repository
        .record_package_validation_summary(
            project_id,
            PackageValidationRecordInput {
                candidate_identity_sha256: initial.candidate_identity_sha256.clone(),
                validation_phase: PackageValidationPhase::Unprivileged,
                attempt_identity_sha256: None,
                installed_host_facts: None,
                application_version: initial.application_version.clone(),
                debian_version: initial.debian_version.clone(),
                manifest_state: verified_stage_state(
                    Stage::Manifest,
                    stages[0].outcome,
                    stages[0].facts.as_ref(),
                    None,
                )?,
                checksum_state: verified_stage_state(
                    Stage::Checksum,
                    stages[1].outcome,
                    stages[1].facts.as_ref(),
                    None,
                )?,
                abi_state: verified_stage_state(
                    Stage::Abi,
                    stages[2].outcome,
                    stages[2].facts.as_ref(),
                    fixed_adapter_operation(Stage::Abi),
                )?,
                provenance_state: verified_stage_state(
                    Stage::Provenance,
                    stages[3].outcome,
                    stages[3].facts.as_ref(),
                    fixed_adapter_operation(Stage::Provenance),
                )?,
                visible_launch_state: verified_stage_state(
                    Stage::VisibleLaunch,
                    stages[4].outcome,
                    stages[4].facts.as_ref(),
                    fixed_adapter_operation(Stage::VisibleLaunch),
                )?,
                installed_host_state: LocalReviewEvidenceCheckState::Unavailable,
                artifact_count: initial.artifacts.len() as u8,
                validation_complete: false,
                supersedes_record_id: None,
            },
        )
        .map_err(|_| PackageValidationControllerError::Unavailable)?;
    Ok(authoritative)
}

fn require_verified_candidate_unchanged(
    candidate_root: &Path,
    initial: &VerifiedPackageCandidate,
    stages: &[StageResultV1],
) -> Result<(), PackageValidationControllerError> {
    if stages.len() != 5
        || stages.iter().any(|stage| {
            stage.application_version != initial.application_version
                || stage.debian_version != initial.debian_version
                || stage.artifact_count != initial.artifacts.len() as u8
        })
        || verify_package_candidate(candidate_root)? != *initial
    {
        return Err(PackageValidationControllerError::InvalidResult);
    }
    Ok(())
}

pub(super) struct ValidationSession {
    context: TrustedValidationContext,
    id: String,
    nonce: String,
    expires: Instant,
    channel_dir: PathBuf,
    accepted: BTreeSet<Stage>,
}

impl ValidationSession {
    fn new(context: TrustedValidationContext) -> Result<Self, PackageValidationControllerError> {
        let id = Uuid::now_v7().to_string();
        let mut nonce = [0u8; NONCE_BYTES];
        let mut source = fs::File::open("/dev/urandom")
            .map_err(|_| PackageValidationControllerError::Unavailable)?;
        use std::io::Read;
        source
            .read_exact(&mut nonce)
            .map_err(|_| PackageValidationControllerError::Unavailable)?;
        let nonce = hex(&nonce);
        let channel_dir = std::env::temp_dir().join(format!("quireforge-package-validation-{id}"));
        fs::create_dir(&channel_dir).map_err(|_| PackageValidationControllerError::Unavailable)?;
        fs::set_permissions(&channel_dir, fs::Permissions::from_mode(0o700))
            .map_err(|_| PackageValidationControllerError::UnsafeChannel)?;
        Ok(Self {
            context,
            id,
            nonce,
            expires: Instant::now() + SESSION_TTL,
            channel_dir,
            accepted: BTreeSet::new(),
        })
    }

    #[cfg(test)]
    fn test_session() -> Self {
        Self {
            context: TrustedValidationContext {
                project_id: Uuid::now_v7().to_string(),
                project_root: PathBuf::from("/trusted"),
            },
            id: Uuid::now_v7().to_string(),
            nonce: "a".repeat(NONCE_BYTES * 2),
            expires: Instant::now() + SESSION_TTL,
            channel_dir: std::env::temp_dir(),
            accepted: BTreeSet::new(),
        }
    }

    fn accept(
        &mut self,
        bytes: &[u8],
        exit_code: i32,
    ) -> Result<StageResultV1, PackageValidationControllerError> {
        if Instant::now() > self.expires {
            return Err(PackageValidationControllerError::Expired);
        }
        if exit_code != 0 || bytes.len() > RESULT_MAX_BYTES {
            return Err(PackageValidationControllerError::InvalidResult);
        }
        let result: StageResultV1 = serde_json::from_slice(bytes)
            .map_err(|_| PackageValidationControllerError::InvalidResult)?;
        if result.schema_version != PROTOCOL_VERSION
            || result.session_id != self.id
            || result.nonce != self.nonce
            || !valid_version(&result.application_version, false)
            || !valid_version(&result.debian_version, true)
            || result.artifact_count > 2
            || verified_stage_state(
                result.stage,
                result.outcome,
                result.facts.as_ref(),
                fixed_adapter_operation(result.stage),
            )
            .is_err()
            || digest(&result) != result.result_sha256
        {
            return Err(PackageValidationControllerError::InvalidResult);
        }
        let expected = [
            Stage::Manifest,
            Stage::Checksum,
            Stage::Abi,
            Stage::Provenance,
            Stage::VisibleLaunch,
        ];
        if expected.get(self.accepted.len()) != Some(&result.stage)
            || !self.accepted.insert(result.stage)
        {
            return Err(PackageValidationControllerError::InvalidStageOrder);
        }
        // Candidate roots are resolved solely by this trusted context. A real
        // adapter must verify this layout before a recorder call is permitted.
        if !self
            .context
            .project_root
            .join("target/ubuntu-22.04/release/packages")
            .starts_with(&self.context.project_root)
        {
            return Err(PackageValidationControllerError::InvalidResult);
        }
        Ok(result)
    }
}

fn digest(value: &StageResultV1) -> String {
    let bytes = serde_json::to_vec(
        &serde_json::to_value(UnsignedStageResult {
            schema_version: value.schema_version,
            session_id: &value.session_id,
            nonce: &value.nonce,
            stage: value.stage,
            outcome: value.outcome,
            application_version: &value.application_version,
            debian_version: &value.debian_version,
            artifact_count: value.artifact_count,
            facts: value.facts.as_ref(),
        })
        .expect("fixed protocol serializes"),
    )
    .expect("fixed protocol serializes");
    format!("{:x}", Sha256::digest(bytes))
}
fn valid_version(value: &str, debian: bool) -> bool {
    (1..=64).contains(&value.len())
        && value.as_bytes().first().is_some_and(u8::is_ascii_digit)
        && value.bytes().all(|b| {
            b.is_ascii_alphanumeric()
                || matches!(b, b'.' | b'+' | b'-')
                || (debian && matches!(b, b':' | b'~'))
        })
}
fn valid_lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}
fn valid_glibc_requirement(value: &str) -> Option<(u16, u16)> {
    let (prefix, version) = value.split_once('_')?;
    if prefix != "GLIBC" {
        return None;
    }
    let (major, minor) = version.split_once('.')?;
    if major.is_empty()
        || minor.is_empty()
        || !major.bytes().all(|byte| byte.is_ascii_digit())
        || !minor.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    Some((major.parse().ok()?, minor.parse().ok()?))
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use std::{fs, os::unix::fs::symlink, path::PathBuf};

    use crate::project::storage::PackageValidationSummary;

    use super::*;

    fn candidate_root(label: &str, reverse: bool, names: (&str, &str)) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "quireforge-package-candidate-{label}-{}",
            Uuid::now_v7()
        ));
        fs::create_dir(&root).expect("fixture root");
        let artifacts = [
            (
                CandidateArtifactFormat::Deb,
                names.0,
                b"desktop package".as_slice(),
            ),
            (
                CandidateArtifactFormat::SandboxdDeb,
                names.1,
                b"sandbox package".as_slice(),
            ),
        ];
        for (_, name, bytes) in artifacts {
            fs::write(root.join(name), bytes).expect("artifact");
        }
        let records = artifacts.into_iter().map(|(format, name, bytes)| serde_json::json!({
            "format": format.name(), "filename": name, "architecture": "x86_64",
            "packageVersion": "0.1.0~beta.46", "sha256": format!("{:x}", Sha256::digest(bytes)),
            "size": bytes.len(),
        })).collect::<Vec<_>>();
        let artifacts = if reverse {
            vec![records[1].clone(), records[0].clone()]
        } else {
            records
        };
        fs::write(root.join("release-manifest.json"), serde_json::to_vec(&serde_json::json!({
            "schemaVersion": 3, "state": "release-candidate", "version": "0.1.0-beta.46",
            "source": {"commit": "0".repeat(40)}, "unrelated": {"path": "/excluded", "command": "excluded"},
            "artifacts": artifacts,
        })).expect("manifest")).expect("write manifest");
        root
    }

    fn rewrite_manifest(root: &Path, update: impl FnOnce(&mut serde_json::Value)) {
        let path = root.join("release-manifest.json");
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).expect("manifest")).expect("json");
        update(&mut value);
        fs::write(path, serde_json::to_vec(&value).expect("json")).expect("write manifest");
    }

    fn remove_candidate(root: &Path) {
        fs::remove_dir_all(root).expect("fixture cleanup");
    }

    fn complete_stage_results(session: &ValidationSession) -> Vec<StageResultV1> {
        [
            Stage::Manifest,
            Stage::Checksum,
            Stage::Abi,
            Stage::Provenance,
            Stage::VisibleLaunch,
        ]
        .into_iter()
        .map(|stage| result(session, stage))
        .collect()
    }
    fn result(session: &ValidationSession, stage: Stage) -> StageResultV1 {
        let mut value = StageResultV1 {
            schema_version: 1,
            session_id: session.id.clone(),
            nonce: session.nonce.clone(),
            stage,
            outcome: Outcome::Passed,
            application_version: "0.1.0-beta.46".into(),
            debian_version: "0.1.0~beta.46".into(),
            artifact_count: 2,
            facts: None,
            result_sha256: String::new(),
        };
        value.result_sha256 = digest(&value);
        value
    }
    #[test]
    fn package_validation_protocol_is_nonce_bound_closed_and_ordered() {
        let mut session = ValidationSession::test_session();
        let manifest = result(&session, Stage::Manifest);
        assert_eq!(
            session
                .accept(&serde_json::to_vec(&manifest).unwrap(), 0)
                .unwrap()
                .stage,
            Stage::Manifest
        );
        assert_eq!(
            session.accept(&serde_json::to_vec(&manifest).unwrap(), 0),
            Err(PackageValidationControllerError::InvalidStageOrder)
        );
        let mut wrong = result(&session, Stage::Checksum);
        wrong.nonce = "b".repeat(64);
        assert_eq!(
            session.accept(&serde_json::to_vec(&wrong).unwrap(), 0),
            Err(PackageValidationControllerError::InvalidResult)
        );
        let checksum = result(&session, Stage::Checksum);
        assert_eq!(
            session.accept(&serde_json::to_vec(&checksum).unwrap(), 1),
            Err(PackageValidationControllerError::InvalidResult)
        );
    }
    #[test]
    fn package_validation_protocol_rejects_unknown_fields_oversize_and_expiry() {
        let mut session = ValidationSession::test_session();
        let mut bytes = serde_json::to_vec(&result(&session, Stage::Manifest)).unwrap();
        bytes.pop();
        bytes.extend_from_slice(br#",\"path\":\"/private\"}"#);
        assert_eq!(
            session.accept(&bytes, 0),
            Err(PackageValidationControllerError::InvalidResult)
        );
        assert_eq!(
            session.accept(&vec![b'x'; RESULT_MAX_BYTES + 1], 0),
            Err(PackageValidationControllerError::InvalidResult)
        );
        session.expires = Instant::now() - Duration::from_secs(1);
        assert_eq!(
            session.accept(
                &serde_json::to_vec(&result(&session, Stage::Manifest)).unwrap(),
                0
            ),
            Err(PackageValidationControllerError::Expired)
        );
    }

    #[test]
    fn package_validation_candidate_identity_is_path_free_deterministic_and_session_independent() {
        let first = candidate_root("identity-a", false, ("desktop.deb", "sandbox.deb"));
        let second = candidate_root(
            "identity-b",
            true,
            ("renamed-desktop.deb", "renamed-sandbox.deb"),
        );
        let first_verified = verify_package_candidate(&first).expect("first candidate");
        let second_verified = verify_package_candidate(&second).expect("second candidate");
        assert_eq!(
            first_verified.candidate_identity_sha256,
            second_verified.candidate_identity_sha256
        );
        assert_eq!(first_verified.candidate_identity_sha256.len(), 64);
        assert_eq!(first_verified.application_version, "0.1.0-beta.46");
        assert_eq!(first_verified.debian_version, "0.1.0~beta.46");
        assert_eq!(first_verified.artifacts.len(), 2);
        assert_eq!(
            first_verified.candidate_identity_sha256,
            "32ac62152ec9ff0f4878bc7a3935609fb948f0624df6199188d37945731461b4"
        );
        let canonical = String::from_utf8(
            canonical_candidate_identity_bytes(
                &first_verified.application_version,
                &first_verified.debian_version,
                &first_verified.artifacts,
            )
            .expect("canonical bytes"),
        )
        .expect("utf8");
        for excluded in [
            "desktop.deb",
            "sandbox.deb",
            "source",
            "commit",
            "path",
            "command",
            "session",
            "nonce",
        ] {
            assert!(!canonical.contains(excluded));
        }
        let mut session = ValidationSession::test_session();
        session.id = Uuid::now_v7().to_string();
        session.nonce = "b".repeat(64);
        assert_ne!(session.id, "");
        assert_eq!(
            verify_package_candidate(&first)
                .expect("session independent")
                .candidate_identity_sha256,
            first_verified.candidate_identity_sha256
        );
        fs::write(first.join("desktop.deb"), b"changed desktop package").expect("change artifact");
        assert!(verify_package_candidate(&first).is_err());
        rewrite_manifest(&first, |value| {
            value["artifacts"][0]["sha256"] =
                serde_json::json!(format!("{:x}", Sha256::digest(b"changed desktop package")));
            value["artifacts"][0]["size"] = serde_json::json!(23);
        });
        assert_ne!(
            verify_package_candidate(&first)
                .expect("changed candidate")
                .candidate_identity_sha256,
            first_verified.candidate_identity_sha256
        );
        remove_candidate(&first);
        remove_candidate(&second);
    }

    #[test]
    fn package_validation_candidate_rejects_manifest_and_artifact_contract_failures() {
        let root = candidate_root("manifest", false, ("desktop.deb", "sandbox.deb"));
        for update in [
            Box::new(|value: &mut serde_json::Value| {
                value["state"] = serde_json::json!("published")
            }) as Box<dyn Fn(&mut serde_json::Value)>,
            Box::new(|value: &mut serde_json::Value| {
                value["version"] = serde_json::json!("bad version")
            }),
            Box::new(|value: &mut serde_json::Value| {
                value["artifacts"][0]["sha256"] = serde_json::json!("0".repeat(64))
            }),
            Box::new(|value: &mut serde_json::Value| {
                let duplicate = value["artifacts"][0].clone();
                value["artifacts"]
                    .as_array_mut()
                    .expect("array")
                    .push(duplicate);
            }),
            Box::new(|value: &mut serde_json::Value| {
                value["artifacts"][1]["format"] = serde_json::json!("deb")
            }),
            Box::new(|value: &mut serde_json::Value| {
                value["artifacts"][1]["format"] = serde_json::json!("unsupported")
            }),
        ] {
            let case = candidate_root("manifest-case", false, ("desktop.deb", "sandbox.deb"));
            rewrite_manifest(&case, update);
            assert!(verify_package_candidate(&case).is_err());
            remove_candidate(&case);
        }
        fs::write(root.join("release-manifest.json"), b"not-json").expect("malformed manifest");
        assert!(verify_package_candidate(&root).is_err());
        remove_candidate(&root);
    }

    #[test]
    fn package_validation_candidate_rejects_missing_symlink_and_escape_artifacts() {
        let missing = candidate_root("missing", false, ("desktop.deb", "sandbox.deb"));
        fs::remove_file(missing.join("desktop.deb")).expect("remove artifact");
        assert!(verify_package_candidate(&missing).is_err());
        remove_candidate(&missing);

        let linked = candidate_root("linked", false, ("desktop.deb", "sandbox.deb"));
        let external =
            std::env::temp_dir().join(format!("quireforge-package-external-{}", Uuid::now_v7()));
        fs::write(&external, b"desktop package").expect("external artifact");
        fs::remove_file(linked.join("desktop.deb")).expect("replace artifact");
        symlink(&external, linked.join("desktop.deb")).expect("symlink");
        assert!(verify_package_candidate(&linked).is_err());
        fs::remove_file(external).expect("external cleanup");
        remove_candidate(&linked);

        let rooted = candidate_root("root-symlink", false, ("desktop.deb", "sandbox.deb"));
        let root_link =
            std::env::temp_dir().join(format!("quireforge-package-root-link-{}", Uuid::now_v7()));
        symlink(&rooted, &root_link).expect("root symlink");
        assert!(verify_package_candidate(&root_link).is_err());
        fs::remove_file(root_link).expect("root-link cleanup");
        remove_candidate(&rooted);

        let escaped = candidate_root("escaped", false, ("desktop.deb", "sandbox.deb"));
        rewrite_manifest(&escaped, |value| {
            value["artifacts"][0]["filename"] = serde_json::json!("../outside.deb")
        });
        assert!(verify_package_candidate(&escaped).is_err());
        remove_candidate(&escaped);
    }

    #[test]
    fn package_validation_candidate_stage_disagreement_and_mutation_prevent_recording() {
        let root = candidate_root("stage-cross-check", false, ("desktop.deb", "sandbox.deb"));
        let initial = verify_package_candidate(&root).expect("initial candidate");
        let session = ValidationSession::test_session();
        let stages = complete_stage_results(&session);
        assert!(require_verified_candidate_unchanged(&root, &initial, &stages).is_ok());

        let mut wrong_version = stages.clone();
        wrong_version[2].application_version = "0.1.0-beta.47".to_owned();
        assert_eq!(
            require_verified_candidate_unchanged(&root, &initial, &wrong_version),
            Err(PackageValidationControllerError::InvalidResult)
        );
        let mut wrong_count = stages.clone();
        wrong_count[3].artifact_count = 1;
        assert_eq!(
            require_verified_candidate_unchanged(&root, &initial, &wrong_count),
            Err(PackageValidationControllerError::InvalidResult)
        );
        fs::write(root.join("desktop.deb"), b"replacement desktop package")
            .expect("change artifact");
        rewrite_manifest(&root, |value| {
            value["artifacts"][0]["sha256"] = serde_json::json!(format!(
                "{:x}",
                Sha256::digest(b"replacement desktop package")
            ));
            value["artifacts"][0]["size"] = serde_json::json!(b"replacement desktop package".len());
        });
        assert_eq!(
            require_verified_candidate_unchanged(&root, &initial, &stages),
            Err(PackageValidationControllerError::InvalidResult)
        );
        remove_candidate(&root);
    }

    #[test]
    fn package_validation_stage_outcomes_are_truthful_and_closed() {
        assert_eq!(
            stage_state(Stage::Manifest, Outcome::Passed),
            Ok(LocalReviewEvidenceCheckState::Passed)
        );
        assert_eq!(
            stage_state(Stage::Checksum, Outcome::Passed),
            Ok(LocalReviewEvidenceCheckState::Passed)
        );
        for stage in [Stage::Abi, Stage::Provenance, Stage::VisibleLaunch] {
            assert_eq!(
                stage_state(stage, Outcome::Unavailable),
                Ok(LocalReviewEvidenceCheckState::Unavailable)
            );
            assert_eq!(
                stage_state(stage, Outcome::Failed),
                Ok(LocalReviewEvidenceCheckState::Failed)
            );
            assert_eq!(
                stage_state(stage, Outcome::Passed),
                Err(PackageValidationControllerError::InvalidResult)
            );
        }
    }

    #[test]
    fn package_validation_fixed_stage_facts_require_a_trusted_operation() {
        let facts = StageFacts::Abi {
            schema_version: 1,
            glibc_baseline: "GLIBC_2.35".to_owned(),
            highest_required: "GLIBC_2.34".to_owned(),
        };
        assert_eq!(
            verified_stage_state(Stage::Abi, Outcome::Passed, Some(&facts), None),
            Err(PackageValidationControllerError::InvalidResult)
        );
        assert_eq!(
            verified_stage_state(
                Stage::Abi,
                Outcome::Passed,
                Some(&facts),
                Some(FixedStageOperation::Abi)
            ),
            Ok(LocalReviewEvidenceCheckState::Passed)
        );
        assert!(serde_json::from_value::<StageFacts>(serde_json::json!({
            "kind": "abi", "schema_version": 1, "glibc_baseline": "GLIBC_2.35",
            "highest_required": "GLIBC_2.34", "filename": "forbidden.deb"
        }))
        .is_err());
    }

    #[test]
    fn package_validation_runner_reaps_normal_and_timed_out_children() {
        let mut normal = Command::new("python3")
            .args(["-c", "raise SystemExit(0)"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("normal child");
        assert!(wait_for_stage_until(&mut normal, Duration::from_secs(1))
            .expect("normal exit")
            .success());
        assert!(normal.try_wait().expect("reaped").is_some());

        let mut slow = Command::new("python3")
            .args(["-c", "import time; time.sleep(10)"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("slow child");
        assert_eq!(
            wait_for_stage_until(&mut slow, Duration::from_millis(1)),
            Err(PackageValidationControllerError::Expired)
        );
        assert!(slow.try_wait().expect("reaped timeout child").is_some());
    }

    #[test]
    fn package_validation_runner_session_cleanup_and_concurrency_are_bounded() {
        let context = |project_id: &str| TrustedValidationContext {
            project_id: project_id.to_owned(),
            project_root: PathBuf::from("/trusted"),
        };
        let mut controller = PackageValidationController::default();
        let first = controller
            .begin(context("018f0000-0000-7000-8000-000000000801"))
            .expect("first run");
        let directory = first.channel_dir.clone();
        assert!(directory.is_dir());
        assert!(matches!(
            controller.begin(context("018f0000-0000-7000-8000-000000000801")),
            Err(PackageValidationControllerError::Unavailable)
        ));
        controller.finish(first);
        assert!(!directory.exists());

        let mut sessions = Vec::new();
        for suffix in 802..806 {
            sessions.push(
                controller
                    .begin(context(&format!("018f0000-0000-7000-8000-{suffix:012}")))
                    .expect("global slot"),
            );
        }
        assert!(matches!(
            controller.begin(context("018f0000-0000-7000-8000-000000000806")),
            Err(PackageValidationControllerError::Unavailable)
        ));
        let released = sessions.pop().expect("slot");
        controller.finish(released);
        let retry = controller
            .begin(context("018f0000-0000-7000-8000-000000000806"))
            .expect("released slot");
        controller.finish(retry);
        for session in sessions {
            controller.finish(session);
        }
    }

    #[test]
    fn package_validation_runner_protocol_rejects_all_result_failures() {
        let mut session = ValidationSession::test_session();
        let malformed = b"not-json";
        assert_eq!(
            session.accept(malformed, 0),
            Err(PackageValidationControllerError::InvalidResult)
        );
        let mut wrong_session = result(&session, Stage::Manifest);
        wrong_session.session_id = Uuid::now_v7().to_string();
        wrong_session.result_sha256 = digest(&wrong_session);
        assert_eq!(
            session.accept(&serde_json::to_vec(&wrong_session).unwrap(), 0),
            Err(PackageValidationControllerError::InvalidResult)
        );
        let mut wrong_nonce = result(&session, Stage::Manifest);
        wrong_nonce.nonce = "b".repeat(64);
        wrong_nonce.result_sha256 = digest(&wrong_nonce);
        assert_eq!(
            session.accept(&serde_json::to_vec(&wrong_nonce).unwrap(), 0),
            Err(PackageValidationControllerError::InvalidResult)
        );
        let mut wrong_stage = result(&session, Stage::Checksum);
        wrong_stage.result_sha256 = digest(&wrong_stage);
        assert_eq!(
            session.accept(&serde_json::to_vec(&wrong_stage).unwrap(), 0),
            Err(PackageValidationControllerError::InvalidStageOrder)
        );
        let mut wrong_digest = result(&session, Stage::Manifest);
        wrong_digest.result_sha256 = "0".repeat(64);
        assert_eq!(
            session.accept(&serde_json::to_vec(&wrong_digest).unwrap(), 0),
            Err(PackageValidationControllerError::InvalidResult)
        );
        let valid = result(&session, Stage::Manifest);
        assert!(session
            .accept(&serde_json::to_vec(&valid).unwrap(), 0)
            .is_ok());
        assert_eq!(
            session.accept(&serde_json::to_vec(&valid).unwrap(), 0),
            Err(PackageValidationControllerError::InvalidStageOrder)
        );
    }

    #[test]
    fn package_validation_abi_passed_requires_digest_bound_closed_facts() {
        let mut session = ValidationSession::test_session();
        for stage in [Stage::Manifest, Stage::Checksum] {
            let value = result(&session, stage);
            assert!(session
                .accept(&serde_json::to_vec(&value).unwrap(), 0)
                .is_ok());
        }
        let mut abi = result(&session, Stage::Abi);
        abi.facts = Some(StageFacts::Abi {
            schema_version: 1,
            glibc_baseline: "GLIBC_2.35".to_owned(),
            highest_required: "GLIBC_2.34".to_owned(),
        });
        abi.result_sha256 = digest(&abi);
        assert!(session
            .accept(&serde_json::to_vec(&abi).unwrap(), 0)
            .is_ok());

        let mut missing = result(&ValidationSession::test_session(), Stage::Abi);
        missing.result_sha256 = digest(&missing);
        assert_eq!(
            verified_stage_state(
                Stage::Abi,
                missing.outcome,
                missing.facts.as_ref(),
                fixed_adapter_operation(Stage::Abi)
            ),
            Err(PackageValidationControllerError::InvalidResult)
        );
    }

    #[test]
    fn package_validation_abi_rejects_forged_unknown_and_malformed_facts() {
        for facts in [
            serde_json::json!({"kind":"abi","schema_version":2,"glibc_baseline":"GLIBC_2.35","highest_required":"GLIBC_2.34"}),
            serde_json::json!({"kind":"abi","schema_version":1,"glibc_baseline":"bad","highest_required":"GLIBC_2.34"}),
            serde_json::json!({"kind":"abi","schema_version":1,"glibc_baseline":"GLIBC_2.35","highest_required":"GLIBC_2.34","path":"/forbidden"}),
            serde_json::json!({"kind":"abi","schema_version":1,"glibc_baseline":"GLIBC_2.35","highest_required":"GLIBC_not-a-version"}),
            serde_json::json!({"kind":"abi","schema_version":1,"glibc_baseline":"GLIBC_2.35","highest_required":"GLIBC_2.36"}),
            serde_json::json!({"kind":"provenance","schema_version":1,"evidence_state":"pinned-release-candidate","artifact_coverage":2,"identity_consistent":true}),
        ] {
            let parsed = serde_json::from_value::<StageFacts>(facts);
            assert!(
                parsed.as_ref().is_err()
                    || verified_stage_state(
                        Stage::Abi,
                        Outcome::Passed,
                        parsed.ok().as_ref(),
                        fixed_adapter_operation(Stage::Abi)
                    )
                    .is_err()
            );
        }
        assert_eq!(
            verified_stage_state(Stage::Abi, Outcome::Failed, None, None),
            Ok(LocalReviewEvidenceCheckState::Failed)
        );
        assert_eq!(
            verified_stage_state(Stage::Abi, Outcome::Unavailable, None, None),
            Ok(LocalReviewEvidenceCheckState::Unavailable)
        );
    }

    #[test]
    fn package_validation_abi_protocol_rejects_nonzero_and_bound_result_mismatches() {
        let mut session = ValidationSession::test_session();
        for stage in [Stage::Manifest, Stage::Checksum] {
            let value = result(&session, stage);
            session
                .accept(&serde_json::to_vec(&value).unwrap(), 0)
                .unwrap();
        }
        let mut abi = result(&session, Stage::Abi);
        abi.facts = Some(StageFacts::Abi {
            schema_version: 1,
            glibc_baseline: "GLIBC_2.35".to_owned(),
            highest_required: "GLIBC_2.34".to_owned(),
        });
        abi.result_sha256 = digest(&abi);
        assert_eq!(
            session.accept(&serde_json::to_vec(&abi).unwrap(), 1),
            Err(PackageValidationControllerError::InvalidResult)
        );
        for mutate in [
            Box::new(|value: &mut StageResultV1| value.session_id = Uuid::now_v7().to_string())
                as Box<dyn Fn(&mut StageResultV1)>,
            Box::new(|value: &mut StageResultV1| value.nonce = "b".repeat(64)),
            Box::new(|value: &mut StageResultV1| value.stage = Stage::Provenance),
            Box::new(|value: &mut StageResultV1| value.result_sha256 = "0".repeat(64)),
        ] {
            let mut forged = abi.clone();
            mutate(&mut forged);
            if forged.result_sha256 != "0".repeat(64) {
                forged.result_sha256 = digest(&forged);
            }
            assert!(session
                .accept(&serde_json::to_vec(&forged).unwrap(), 0)
                .is_err());
        }
    }

    #[test]
    fn package_validation_abi_closed_facts_allow_only_the_fixed_schema() {
        for field in ["path", "filename", "command", "output"] {
            let mut value = serde_json::json!({
                "kind": "abi", "schema_version": 1, "glibc_baseline": "GLIBC_2.35",
                "highest_required": "GLIBC_2.34"
            });
            value[field] = serde_json::json!("forbidden");
            assert!(serde_json::from_value::<StageFacts>(value).is_err());
        }
        let facts = StageFacts::Abi {
            schema_version: 1,
            glibc_baseline: "GLIBC_2.35".to_owned(),
            highest_required: "GLIBC_2.34".to_owned(),
        };
        assert_eq!(
            verified_stage_state(
                Stage::Abi,
                Outcome::Passed,
                Some(&facts),
                fixed_adapter_operation(Stage::Abi)
            ),
            Ok(LocalReviewEvidenceCheckState::Passed)
        );
        for stage in [Stage::Provenance, Stage::VisibleLaunch] {
            assert_eq!(
                verified_stage_state(stage, Outcome::Unavailable, None, None),
                Ok(LocalReviewEvidenceCheckState::Unavailable)
            );
        }
    }

    #[test]
    fn package_validation_abi_maps_only_abi_to_passed_without_completion() {
        let facts = StageFacts::Abi {
            schema_version: 1,
            glibc_baseline: "GLIBC_2.35".to_owned(),
            highest_required: "GLIBC_2.34".to_owned(),
        };
        assert_eq!(
            verified_stage_state(Stage::Manifest, Outcome::Passed, None, None),
            Ok(LocalReviewEvidenceCheckState::Passed)
        );
        assert_eq!(
            verified_stage_state(Stage::Checksum, Outcome::Passed, None, None),
            Ok(LocalReviewEvidenceCheckState::Passed)
        );
        assert_eq!(
            verified_stage_state(
                Stage::Abi,
                Outcome::Passed,
                Some(&facts),
                fixed_adapter_operation(Stage::Abi)
            ),
            Ok(LocalReviewEvidenceCheckState::Passed)
        );
        let record = PackageValidationRecordInput {
            candidate_identity_sha256: "a".repeat(64),
            validation_phase: PackageValidationPhase::Unprivileged,
            attempt_identity_sha256: None,
            installed_host_facts: None,
            application_version: "0.1.0-beta.46".to_owned(),
            debian_version: "0.1.0~beta.46".to_owned(),
            manifest_state: LocalReviewEvidenceCheckState::Passed,
            checksum_state: LocalReviewEvidenceCheckState::Passed,
            abi_state: LocalReviewEvidenceCheckState::Passed,
            provenance_state: LocalReviewEvidenceCheckState::Unavailable,
            visible_launch_state: LocalReviewEvidenceCheckState::Unavailable,
            installed_host_state: LocalReviewEvidenceCheckState::Unavailable,
            artifact_count: 2,
            validation_complete: false,
            supersedes_record_id: None,
        };
        assert_eq!(record.abi_state, LocalReviewEvidenceCheckState::Passed);
        assert_eq!(
            record.provenance_state,
            LocalReviewEvidenceCheckState::Unavailable
        );
        assert_eq!(
            record.visible_launch_state,
            LocalReviewEvidenceCheckState::Unavailable
        );
        assert_eq!(
            record.installed_host_state,
            LocalReviewEvidenceCheckState::Unavailable
        );
        assert!(!record.validation_complete);
    }

    #[test]
    fn package_validation_abi_candidate_mutation_prevents_recording() {
        let root = candidate_root("abi-mutation", false, ("desktop.deb", "sandbox.deb"));
        let initial = verify_package_candidate(&root).expect("initial candidate");
        let session = ValidationSession::test_session();
        let stages = complete_stage_results(&session);
        fs::write(root.join("desktop.deb"), b"mutated desktop package").expect("mutate");
        assert_eq!(
            require_verified_candidate_unchanged(&root, &initial, &stages),
            Err(PackageValidationControllerError::InvalidResult)
        );
        remove_candidate(&root);
    }

    fn provenance_facts() -> StageFacts {
        StageFacts::Provenance {
            schema_version: 1,
            evidence_state: "pinned-release-candidate".to_owned(),
            artifact_coverage: 2,
            identity_consistent: true,
        }
    }

    #[test]
    fn package_validation_provenance_passed_requires_digest_bound_closed_facts() {
        let mut session = ValidationSession::test_session();
        for stage in [Stage::Manifest, Stage::Checksum, Stage::Abi] {
            let mut value = result(&session, stage);
            if stage == Stage::Abi {
                value.facts = Some(StageFacts::Abi {
                    schema_version: 1,
                    glibc_baseline: "GLIBC_2.35".to_owned(),
                    highest_required: "GLIBC_2.34".to_owned(),
                });
                value.result_sha256 = digest(&value);
            }
            session
                .accept(&serde_json::to_vec(&value).unwrap(), 0)
                .unwrap();
        }
        let mut provenance = result(&session, Stage::Provenance);
        provenance.facts = Some(provenance_facts());
        provenance.result_sha256 = digest(&provenance);
        assert!(session
            .accept(&serde_json::to_vec(&provenance).unwrap(), 0)
            .is_ok());
        assert_eq!(
            verified_stage_state(
                Stage::Provenance,
                Outcome::Passed,
                None,
                fixed_adapter_operation(Stage::Provenance)
            ),
            Err(PackageValidationControllerError::InvalidResult)
        );
    }

    #[test]
    fn package_validation_provenance_rejects_forged_unknown_and_prohibited_facts() {
        for field in ["source_commit", "path", "filename", "command", "output"] {
            let mut value = serde_json::json!({
                "kind": "provenance", "schema_version": 1,
                "evidence_state": "pinned-release-candidate", "artifact_coverage": 2,
                "identity_consistent": true
            });
            value[field] = serde_json::json!("forbidden");
            assert!(serde_json::from_value::<StageFacts>(value).is_err());
        }
        for facts in [
            StageFacts::Provenance {
                schema_version: 2,
                evidence_state: "pinned-release-candidate".to_owned(),
                artifact_coverage: 2,
                identity_consistent: true,
            },
            StageFacts::Provenance {
                evidence_state: "forged".to_owned(),
                schema_version: 1,
                artifact_coverage: 2,
                identity_consistent: true,
            },
            StageFacts::Provenance {
                artifact_coverage: 1,
                schema_version: 1,
                evidence_state: "pinned-release-candidate".to_owned(),
                identity_consistent: true,
            },
            StageFacts::Provenance {
                identity_consistent: false,
                schema_version: 1,
                evidence_state: "pinned-release-candidate".to_owned(),
                artifact_coverage: 2,
            },
        ] {
            assert_eq!(
                verified_stage_state(
                    Stage::Provenance,
                    Outcome::Passed,
                    Some(&facts),
                    fixed_adapter_operation(Stage::Provenance)
                ),
                Err(PackageValidationControllerError::InvalidResult)
            );
        }
    }

    #[test]
    fn package_validation_provenance_rejects_nonzero_and_bound_result_mismatches() {
        let mut session = ValidationSession::test_session();
        for stage in [Stage::Manifest, Stage::Checksum, Stage::Abi] {
            let mut value = result(&session, stage);
            if stage == Stage::Abi {
                value.facts = Some(StageFacts::Abi {
                    schema_version: 1,
                    glibc_baseline: "GLIBC_2.35".to_owned(),
                    highest_required: "GLIBC_2.34".to_owned(),
                });
                value.result_sha256 = digest(&value);
            }
            session
                .accept(&serde_json::to_vec(&value).unwrap(), 0)
                .unwrap();
        }
        let mut provenance = result(&session, Stage::Provenance);
        provenance.facts = Some(provenance_facts());
        provenance.result_sha256 = digest(&provenance);
        assert_eq!(
            session.accept(&serde_json::to_vec(&provenance).unwrap(), 1),
            Err(PackageValidationControllerError::InvalidResult)
        );
        for mutate in [
            Box::new(|value: &mut StageResultV1| value.session_id = Uuid::now_v7().to_string())
                as Box<dyn Fn(&mut StageResultV1)>,
            Box::new(|value: &mut StageResultV1| value.nonce = "b".repeat(64)),
            Box::new(|value: &mut StageResultV1| value.stage = Stage::VisibleLaunch),
            Box::new(|value: &mut StageResultV1| value.result_sha256 = "0".repeat(64)),
        ] {
            let mut forged = provenance.clone();
            mutate(&mut forged);
            if forged.result_sha256 != "0".repeat(64) {
                forged.result_sha256 = digest(&forged);
            }
            assert!(session
                .accept(&serde_json::to_vec(&forged).unwrap(), 0)
                .is_err());
        }
    }

    #[test]
    fn package_validation_provenance_candidate_mutation_prevents_recording() {
        let root = candidate_root("provenance-mutation", false, ("desktop.deb", "sandbox.deb"));
        let initial = verify_package_candidate(&root).expect("initial candidate");
        let session = ValidationSession::test_session();
        let stages = complete_stage_results(&session);
        fs::write(root.join("sandbox.deb"), b"mutated sandbox package").expect("mutate");
        assert_eq!(
            require_verified_candidate_unchanged(&root, &initial, &stages),
            Err(PackageValidationControllerError::InvalidResult)
        );
        remove_candidate(&root);
    }

    #[test]
    fn package_validation_provenance_maps_only_provenance_to_passed_without_completion() {
        assert_eq!(
            verified_stage_state(
                Stage::Provenance,
                Outcome::Passed,
                Some(&provenance_facts()),
                fixed_adapter_operation(Stage::Provenance)
            ),
            Ok(LocalReviewEvidenceCheckState::Passed)
        );
        let record = PackageValidationRecordInput {
            candidate_identity_sha256: "a".repeat(64),
            validation_phase: PackageValidationPhase::Unprivileged,
            attempt_identity_sha256: None,
            installed_host_facts: None,
            application_version: "0.1.0-beta.46".to_owned(),
            debian_version: "0.1.0~beta.46".to_owned(),
            manifest_state: LocalReviewEvidenceCheckState::Passed,
            checksum_state: LocalReviewEvidenceCheckState::Passed,
            abi_state: LocalReviewEvidenceCheckState::Unavailable,
            provenance_state: LocalReviewEvidenceCheckState::Passed,
            visible_launch_state: LocalReviewEvidenceCheckState::Unavailable,
            installed_host_state: LocalReviewEvidenceCheckState::Unavailable,
            artifact_count: 2,
            validation_complete: false,
            supersedes_record_id: None,
        };
        assert_eq!(
            record.provenance_state,
            LocalReviewEvidenceCheckState::Passed
        );
        assert_eq!(
            record.visible_launch_state,
            LocalReviewEvidenceCheckState::Unavailable
        );
        assert_eq!(
            record.installed_host_state,
            LocalReviewEvidenceCheckState::Unavailable
        );
        assert!(!record.validation_complete);
    }

    fn visible_launch_facts() -> StageFacts {
        StageFacts::VisibleLaunch {
            schema_version: 1,
            launch_state: "visible-window-confirmed".to_owned(),
            artifact_coverage: 1,
            visibility_confirmed: true,
            lifecycle_clean: true,
        }
    }

    fn accept_visible_launch_prefix(session: &mut ValidationSession) {
        for stage in [
            Stage::Manifest,
            Stage::Checksum,
            Stage::Abi,
            Stage::Provenance,
        ] {
            let mut value = result(session, stage);
            match stage {
                Stage::Abi => {
                    value.facts = Some(StageFacts::Abi {
                        schema_version: 1,
                        glibc_baseline: "GLIBC_2.35".to_owned(),
                        highest_required: "GLIBC_2.34".to_owned(),
                    })
                }
                Stage::Provenance => value.facts = Some(provenance_facts()),
                _ => {}
            }
            value.result_sha256 = digest(&value);
            session
                .accept(&serde_json::to_vec(&value).unwrap(), 0)
                .unwrap();
        }
    }

    #[test]
    fn package_validation_visible_launch_passed_requires_digest_bound_closed_facts() {
        let mut session = ValidationSession::test_session();
        accept_visible_launch_prefix(&mut session);
        let mut visible = result(&session, Stage::VisibleLaunch);
        visible.facts = Some(visible_launch_facts());
        visible.result_sha256 = digest(&visible);
        assert!(session
            .accept(&serde_json::to_vec(&visible).unwrap(), 0)
            .is_ok());
        assert_eq!(
            verified_stage_state(
                Stage::VisibleLaunch,
                Outcome::Passed,
                None,
                fixed_adapter_operation(Stage::VisibleLaunch)
            ),
            Err(PackageValidationControllerError::InvalidResult)
        );
    }

    #[test]
    fn package_validation_visible_launch_rejects_forged_unknown_and_unclean_facts() {
        for field in [
            "path",
            "filename",
            "display",
            "window_title",
            "screenshot",
            "pid",
            "command",
            "output",
            "commit",
        ] {
            let mut value = serde_json::json!({
                "kind":"visible-launch", "schema_version":1, "launch_state":"visible-window-confirmed",
                "artifact_coverage":1, "visibility_confirmed":true, "lifecycle_clean":true
            });
            value[field] = serde_json::json!("forbidden");
            assert!(serde_json::from_value::<StageFacts>(value).is_err());
        }
        for facts in [
            StageFacts::VisibleLaunch {
                schema_version: 2,
                launch_state: "visible-window-confirmed".to_owned(),
                artifact_coverage: 1,
                visibility_confirmed: true,
                lifecycle_clean: true,
            },
            StageFacts::VisibleLaunch {
                schema_version: 1,
                launch_state: "forged".to_owned(),
                artifact_coverage: 1,
                visibility_confirmed: true,
                lifecycle_clean: true,
            },
            StageFacts::VisibleLaunch {
                schema_version: 1,
                launch_state: "visible-window-confirmed".to_owned(),
                artifact_coverage: 1,
                visibility_confirmed: true,
                lifecycle_clean: false,
            },
        ] {
            assert_eq!(
                verified_stage_state(
                    Stage::VisibleLaunch,
                    Outcome::Passed,
                    Some(&facts),
                    fixed_adapter_operation(Stage::VisibleLaunch)
                ),
                Err(PackageValidationControllerError::InvalidResult)
            );
        }
    }

    #[test]
    fn package_validation_visible_launch_rejects_nonzero_bound_mismatches_and_reaps_timeout() {
        let mut session = ValidationSession::test_session();
        accept_visible_launch_prefix(&mut session);
        let mut visible = result(&session, Stage::VisibleLaunch);
        visible.facts = Some(visible_launch_facts());
        visible.result_sha256 = digest(&visible);
        assert_eq!(
            session.accept(&serde_json::to_vec(&visible).unwrap(), 1),
            Err(PackageValidationControllerError::InvalidResult)
        );
        for mutate in [
            Box::new(|value: &mut StageResultV1| value.session_id = Uuid::now_v7().to_string())
                as Box<dyn Fn(&mut StageResultV1)>,
            Box::new(|value: &mut StageResultV1| value.nonce = "b".repeat(64)),
            Box::new(|value: &mut StageResultV1| value.stage = Stage::Provenance),
            Box::new(|value: &mut StageResultV1| value.result_sha256 = "0".repeat(64)),
        ] {
            let mut forged = visible.clone();
            mutate(&mut forged);
            if forged.result_sha256 != "0".repeat(64) {
                forged.result_sha256 = digest(&forged);
            }
            assert!(session
                .accept(&serde_json::to_vec(&forged).unwrap(), 0)
                .is_err());
        }
        let mut child = Command::new("python3")
            .args(["-c", "import time; time.sleep(10)"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        assert_eq!(
            wait_for_stage_until(&mut child, Duration::from_millis(1)),
            Err(PackageValidationControllerError::Expired)
        );
        assert!(child.try_wait().unwrap().is_some());
    }

    #[test]
    fn package_validation_visible_launch_candidate_mutation_prevents_recording() {
        let root = candidate_root("visible-mutation", false, ("desktop.deb", "sandbox.deb"));
        let initial = verify_package_candidate(&root).unwrap();
        let session = ValidationSession::test_session();
        let stages = complete_stage_results(&session);
        fs::write(root.join("desktop.deb"), b"mutated desktop package").unwrap();
        assert_eq!(
            require_verified_candidate_unchanged(&root, &initial, &stages),
            Err(PackageValidationControllerError::InvalidResult)
        );
        remove_candidate(&root);
    }

    #[test]
    fn package_validation_visible_launch_maps_truthfully_without_host_completion() {
        assert_eq!(
            verified_stage_state(
                Stage::VisibleLaunch,
                Outcome::Passed,
                Some(&visible_launch_facts()),
                fixed_adapter_operation(Stage::VisibleLaunch)
            ),
            Ok(LocalReviewEvidenceCheckState::Passed)
        );
        let record = PackageValidationRecordInput {
            candidate_identity_sha256: "a".repeat(64),
            validation_phase: PackageValidationPhase::Unprivileged,
            attempt_identity_sha256: None,
            installed_host_facts: None,
            application_version: "0.1.0-beta.46".to_owned(),
            debian_version: "0.1.0~beta.46".to_owned(),
            manifest_state: LocalReviewEvidenceCheckState::Passed,
            checksum_state: LocalReviewEvidenceCheckState::Passed,
            abi_state: LocalReviewEvidenceCheckState::Unavailable,
            provenance_state: LocalReviewEvidenceCheckState::Unavailable,
            visible_launch_state: LocalReviewEvidenceCheckState::Passed,
            installed_host_state: LocalReviewEvidenceCheckState::Unavailable,
            artifact_count: 2,
            validation_complete: false,
            supersedes_record_id: None,
        };
        assert_eq!(
            record.visible_launch_state,
            LocalReviewEvidenceCheckState::Passed
        );
        assert_eq!(
            record.installed_host_state,
            LocalReviewEvidenceCheckState::Unavailable
        );
        assert!(!record.validation_complete);
    }

    fn installed_host_result(session: &ValidationSession, outcome: Outcome) -> InstalledHostResult {
        let facts = (outcome == Outcome::Passed).then(|| InstalledHostFacts {
            kind: "installed-host".to_owned(),
            schema_version: 1,
            package_state: "installed".to_owned(),
            version_match: true,
            ownership_verified: true,
            permissions_safe: true,
            package_integrity_verified: true,
        });
        let mut result = InstalledHostResult {
            schema_version: 1,
            session_id: session.id.clone(),
            nonce: session.nonce.clone(),
            outcome,
            facts,
            result_sha256: String::new(),
        };
        result.result_sha256 = installed_host_result_digest(&result);
        result
    }

    fn installed_host_predecessor(
        repository: &mut ProjectRepository,
        project_id: &str,
        candidate_identity_sha256: &str,
    ) -> PackageValidationSummary {
        match repository
            .record_package_validation_summary(
                project_id,
                PackageValidationRecordInput {
                    candidate_identity_sha256: candidate_identity_sha256.to_owned(),
                    validation_phase: PackageValidationPhase::Unprivileged,
                    attempt_identity_sha256: None,
                    installed_host_facts: None,
                    application_version: "0.1.0-beta.46".to_owned(),
                    debian_version: "0.1.0~beta.46".to_owned(),
                    manifest_state: LocalReviewEvidenceCheckState::Passed,
                    checksum_state: LocalReviewEvidenceCheckState::Passed,
                    abi_state: LocalReviewEvidenceCheckState::Passed,
                    provenance_state: LocalReviewEvidenceCheckState::Passed,
                    visible_launch_state: LocalReviewEvidenceCheckState::Passed,
                    installed_host_state: LocalReviewEvidenceCheckState::Unavailable,
                    artifact_count: 2,
                    validation_complete: false,
                    supersedes_record_id: None,
                },
            )
            .expect("unprivileged predecessor")
        {
            PackageValidationRecordOutcome::Created(summary) => summary,
            PackageValidationRecordOutcome::Existing(_) => panic!("fresh predecessor"),
        }
    }

    fn fake_installed_host_result(request: &[u8], outcome: Outcome) -> Vec<u8> {
        let request: serde_json::Value = serde_json::from_slice(request).expect("request");
        let mut result = InstalledHostResult {
            schema_version: 1,
            session_id: request["session_id"].as_str().expect("session").to_owned(),
            nonce: request["nonce"].as_str().expect("nonce").to_owned(),
            outcome,
            facts: (outcome == Outcome::Passed).then(|| InstalledHostFacts {
                kind: "installed-host".to_owned(),
                schema_version: 1,
                package_state: "installed".to_owned(),
                version_match: true,
                ownership_verified: true,
                permissions_safe: true,
                package_integrity_verified: true,
            }),
            result_sha256: String::new(),
        };
        result.result_sha256 = installed_host_result_digest(&result);
        serde_json::to_vec(&result).expect("result")
    }

    fn installed_host_context(project_id: String) -> TrustedValidationContext {
        TrustedValidationContext {
            project_id,
            project_root: PathBuf::from("/trusted"),
        }
    }

    #[test]
    fn package_validation_installed_host_protocol_is_closed_digest_bound_and_noninteractive() {
        let session = ValidationSession::test_session();
        let request = serde_json::to_vec(&InstalledHostRequest {
            schema_version: 1,
            session_id: &session.id,
            nonce: &session.nonce,
            expected_application_version: "0.1.0-beta.46",
            expected_debian_version: "0.1.0~beta.46",
        })
        .unwrap();
        let request_text = String::from_utf8(request).unwrap();
        assert!(request_text.len() < RESULT_MAX_BYTES);
        for forbidden in ["project", "candidate", "path", "database", "command"] {
            assert!(!request_text.contains(forbidden));
        }
        assert_eq!(SUDO, "/usr/bin/sudo");
        assert_eq!(
            INSTALLED_HOST_HELPER,
            "/usr/local/sbin/quireforge-validate-deb"
        );
        let result = installed_host_result(&session, Outcome::Passed);
        assert!(verify_installed_host_result(
            &serde_json::to_vec(&result).unwrap(),
            0,
            &session.id,
            &session.nonce
        )
        .is_ok());
        assert!(matches!(
            verify_installed_host_result(
                &serde_json::to_vec(&result).unwrap(),
                1,
                &session.id,
                &session.nonce
            ),
            Err(PackageValidationControllerError::InvalidResult)
        ));
    }

    #[test]
    fn package_validation_installed_host_rejects_forged_prohibited_and_mismatched_results() {
        let session = ValidationSession::test_session();
        let mut result = installed_host_result(&session, Outcome::Passed);
        result.nonce = "b".repeat(64);
        result.result_sha256 = installed_host_result_digest(&result);
        assert!(verify_installed_host_result(
            &serde_json::to_vec(&result).unwrap(),
            0,
            &session.id,
            &session.nonce
        )
        .is_err());
        let mut forged =
            serde_json::to_value(installed_host_result(&session, Outcome::Passed)).unwrap();
        forged["facts"]["path"] = serde_json::json!("/forbidden");
        assert!(verify_installed_host_result(
            &serde_json::to_vec(&forged).unwrap(),
            0,
            &session.id,
            &session.nonce
        )
        .is_err());
        let mut contradictory = installed_host_result(&session, Outcome::Passed);
        contradictory.facts.as_mut().unwrap().permissions_safe = false;
        contradictory.result_sha256 = installed_host_result_digest(&contradictory);
        assert!(verify_installed_host_result(
            &serde_json::to_vec(&contradictory).unwrap(),
            0,
            &session.id,
            &session.nonce
        )
        .is_err());
    }

    #[test]
    fn package_validation_installed_host_snake_case_golden_vectors_are_compatible() {
        let session = ValidationSession::test_session();
        let request = InstalledHostRequest {
            schema_version: 1,
            session_id: "019d4e3c-3b14-7a2b-8c91-3f27d4f7aa10",
            nonce: &"a".repeat(64),
            expected_application_version: "0.1.0-beta.46",
            expected_debian_version: "0.1.0~beta.46",
        };
        let request = String::from_utf8(serde_json::to_vec(&request).unwrap()).unwrap();
        assert_eq!(request, "{\"schema_version\":1,\"session_id\":\"019d4e3c-3b14-7a2b-8c91-3f27d4f7aa10\",\"nonce\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"expected_application_version\":\"0.1.0-beta.46\",\"expected_debian_version\":\"0.1.0~beta.46\"}");
        let passed = br#"{"facts":{"kind":"installed-host","ownership_verified":true,"package_integrity_verified":true,"package_state":"installed","permissions_safe":true,"schema_version":1,"version_match":true},"nonce":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","outcome":"passed","result_sha256":"f717187b6bee5c97948ebe75196b6c19921711b1feebb60e17b967443348e4a8","schema_version":1,"session_id":"019d4e3c-3b14-7a2b-8c91-3f27d4f7aa10"}"#;
        let failed = br#"{"facts":null,"nonce":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","outcome":"failed","result_sha256":"41810f1ac1fa8e58224b9bc7f87c1986d6d2a1676c4a6518c3c2ca0f7c50e95b","schema_version":1,"session_id":"019d4e3c-3b14-7a2b-8c91-3f27d4f7aa10"}"#;
        let unavailable = br#"{"facts":null,"nonce":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","outcome":"unavailable","result_sha256":"b406376fa2ecd097ebf7fca0cc9b75b3552e3a6a6041c889655efc33f4630074","schema_version":1,"session_id":"019d4e3c-3b14-7a2b-8c91-3f27d4f7aa10"}"#;
        for value in [passed.as_slice(), failed.as_slice(), unavailable.as_slice()] {
            assert!(verify_installed_host_result(
                value,
                0,
                "019d4e3c-3b14-7a2b-8c91-3f27d4f7aa10",
                &"a".repeat(64)
            )
            .is_ok());
        }
        assert!(serde_json::from_slice::<InstalledHostResult>(br#"{"schemaVersion":1}"#).is_err());
        assert!(serde_json::from_slice::<InstalledHostResult>(
            br#"{"schema_version":1,"extra":true}"#
        )
        .is_err());
        assert_ne!(session.id, "");
    }

    #[test]
    fn package_validation_installed_host_fake_runner_persists_complete_supersession() {
        let (mut repository, project_id) = ProjectRepository::package_validation_test_repository();
        let predecessor = repository
            .record_package_validation_summary(
                &project_id,
                PackageValidationRecordInput {
                    candidate_identity_sha256: "c".repeat(64),
                    validation_phase: PackageValidationPhase::Unprivileged,
                    attempt_identity_sha256: None,
                    installed_host_facts: None,
                    application_version: "0.1.0-beta.46".to_owned(),
                    debian_version: "0.1.0~beta.46".to_owned(),
                    manifest_state: LocalReviewEvidenceCheckState::Passed,
                    checksum_state: LocalReviewEvidenceCheckState::Passed,
                    abi_state: LocalReviewEvidenceCheckState::Passed,
                    provenance_state: LocalReviewEvidenceCheckState::Passed,
                    visible_launch_state: LocalReviewEvidenceCheckState::Passed,
                    installed_host_state: LocalReviewEvidenceCheckState::Unavailable,
                    artifact_count: 2,
                    validation_complete: false,
                    supersedes_record_id: None,
                },
            )
            .unwrap();
        let predecessor = match predecessor {
            PackageValidationRecordOutcome::Created(summary) => summary,
            _ => panic!("fresh"),
        };
        let context = TrustedValidationContext {
            project_id: project_id.clone(),
            project_root: PathBuf::from("/trusted"),
        };
        let mut controller = PackageValidationController::default();
        let outcome = controller
            .run_installed_host_with(&mut repository, context, &predecessor.id, |request| {
                let request: serde_json::Value = serde_json::from_slice(request).unwrap();
                assert_eq!(request["schema_version"], 1);
                assert!(request.get("schemaVersion").is_none());
                assert_eq!(request.as_object().unwrap().len(), 5);
                for forbidden in [
                    "project",
                    "candidate",
                    "receipt",
                    "path",
                    "filename",
                    "database",
                    "command",
                    "artifact",
                    "stage",
                ] {
                    assert!(!request.to_string().contains(forbidden));
                }
                let mut result = InstalledHostResult {
                    schema_version: 1,
                    session_id: request["session_id"].as_str().unwrap().to_owned(),
                    nonce: request["nonce"].as_str().unwrap().to_owned(),
                    outcome: Outcome::Passed,
                    facts: Some(InstalledHostFacts {
                        kind: "installed-host".to_owned(),
                        schema_version: 1,
                        package_state: "installed".to_owned(),
                        version_match: true,
                        ownership_verified: true,
                        permissions_safe: true,
                        package_integrity_verified: true,
                    }),
                    result_sha256: String::new(),
                };
                result.result_sha256 = installed_host_result_digest(&result);
                Ok((0, serde_json::to_vec(&result).unwrap(), Vec::new()))
            })
            .unwrap();
        assert_eq!(outcome, InstalledHostValidationOutcome::Created);
        let stored = repository
            .package_validation_phase_summary_for_test(
                &project_id,
                PackageValidationPhase::InstalledHost,
            )
            .unwrap();
        assert!(stored.input.validation_complete);
        assert_eq!(
            stored.input.installed_host_state,
            LocalReviewEvidenceCheckState::Passed
        );
        assert_eq!(
            stored.input.supersedes_record_id,
            Some(predecessor.id.clone())
        );
        assert_eq!(stored.input.artifact_count, 2);
        assert_eq!(
            stored.input.manifest_state,
            predecessor.input.manifest_state
        );
    }

    #[test]
    fn package_validation_installed_host_fake_runner_persists_signed_unavailable_not_process_failure(
    ) {
        let (mut repository, project_id) = ProjectRepository::package_validation_test_repository();
        let predecessor = repository
            .record_package_validation_summary(
                &project_id,
                PackageValidationRecordInput {
                    candidate_identity_sha256: "d".repeat(64),
                    validation_phase: PackageValidationPhase::Unprivileged,
                    attempt_identity_sha256: None,
                    installed_host_facts: None,
                    application_version: "0.1.0-beta.46".to_owned(),
                    debian_version: "0.1.0~beta.46".to_owned(),
                    manifest_state: LocalReviewEvidenceCheckState::Passed,
                    checksum_state: LocalReviewEvidenceCheckState::Passed,
                    abi_state: LocalReviewEvidenceCheckState::Passed,
                    provenance_state: LocalReviewEvidenceCheckState::Passed,
                    visible_launch_state: LocalReviewEvidenceCheckState::Passed,
                    installed_host_state: LocalReviewEvidenceCheckState::Unavailable,
                    artifact_count: 2,
                    validation_complete: false,
                    supersedes_record_id: None,
                },
            )
            .unwrap();
        let predecessor = match predecessor {
            PackageValidationRecordOutcome::Created(summary) => summary,
            _ => panic!("fresh"),
        };
        let context = || TrustedValidationContext {
            project_id: project_id.clone(),
            project_root: PathBuf::from("/trusted"),
        };
        let mut controller = PackageValidationController::default();
        let outcome = controller
            .run_installed_host_with(&mut repository, context(), &predecessor.id, |request| {
                let request: serde_json::Value = serde_json::from_slice(request).unwrap();
                let mut result = InstalledHostResult {
                    schema_version: 1,
                    session_id: request["session_id"].as_str().unwrap().to_owned(),
                    nonce: request["nonce"].as_str().unwrap().to_owned(),
                    outcome: Outcome::Unavailable,
                    facts: None,
                    result_sha256: String::new(),
                };
                result.result_sha256 = installed_host_result_digest(&result);
                Ok((0, serde_json::to_vec(&result).unwrap(), Vec::new()))
            })
            .unwrap();
        assert_eq!(outcome, InstalledHostValidationOutcome::Unavailable);
        let unavailable = repository
            .package_validation_phase_summary_for_test(
                &project_id,
                PackageValidationPhase::InstalledHost,
            )
            .unwrap();
        assert_eq!(
            unavailable.input.installed_host_state,
            LocalReviewEvidenceCheckState::Unavailable
        );
        assert!(!unavailable.input.validation_complete);
        let mut second = PackageValidationController::default();
        assert!(matches!(
            second.run_installed_host_with(&mut repository, context(), &predecessor.id, |_| Err(
                PackageValidationControllerError::Unavailable
            )),
            Err(PackageValidationControllerError::Unavailable)
        ));
        let newest = repository
            .package_validation_phase_summary_for_test(
                &project_id,
                PackageValidationPhase::InstalledHost,
            )
            .unwrap();
        assert_eq!(newest.id, unavailable.id);
    }

    #[test]
    fn package_validation_installed_host_fake_runner_builds_linear_failed_unavailable_passed_chains(
    ) {
        let (mut repository, project_id) = ProjectRepository::package_validation_test_repository();
        let root = installed_host_predecessor(&mut repository, &project_id, &"e".repeat(64));
        let mut controller = PackageValidationController::default();
        let unavailable_outcome = controller
            .run_installed_host_with(
                &mut repository,
                installed_host_context(project_id.clone()),
                &root.id,
                |request| {
                    Ok((
                        0,
                        fake_installed_host_result(request, Outcome::Unavailable),
                        vec![],
                    ))
                },
            )
            .expect("verified unavailable");
        assert_eq!(
            unavailable_outcome,
            InstalledHostValidationOutcome::Unavailable
        );
        let unavailable = repository
            .package_validation_phase_summary_for_test(
                &project_id,
                PackageValidationPhase::InstalledHost,
            )
            .expect("unavailable attempt");
        let mut controller = PackageValidationController::default();
        let failed_outcome = controller
            .run_installed_host_with(
                &mut repository,
                installed_host_context(project_id.clone()),
                &root.id,
                |request| {
                    Ok((
                        0,
                        fake_installed_host_result(request, Outcome::Failed),
                        vec![],
                    ))
                },
            )
            .expect("verified failed");
        assert_eq!(failed_outcome, InstalledHostValidationOutcome::Failed);
        let failed = repository
            .package_validation_phase_summary_for_test(
                &project_id,
                PackageValidationPhase::InstalledHost,
            )
            .expect("failed attempt");
        assert_eq!(
            failed.input.supersedes_record_id,
            Some(unavailable.id.clone())
        );
        assert_eq!(
            failed.input.installed_host_state,
            LocalReviewEvidenceCheckState::Failed
        );
        let mut controller = PackageValidationController::default();
        let created_outcome = controller
            .run_installed_host_with(
                &mut repository,
                installed_host_context(project_id.clone()),
                &root.id,
                |request| {
                    Ok((
                        0,
                        fake_installed_host_result(request, Outcome::Passed),
                        vec![],
                    ))
                },
            )
            .expect("verified passed");
        assert_eq!(created_outcome, InstalledHostValidationOutcome::Created);
        let passed = repository
            .package_validation_phase_summary_for_test(
                &project_id,
                PackageValidationPhase::InstalledHost,
            )
            .expect("passed attempt");
        assert_eq!(passed.input.supersedes_record_id, Some(failed.id.clone()));
        assert!(passed.input.validation_complete);
        assert_eq!(
            unavailable.input.supersedes_record_id,
            Some(root.id.clone())
        );
        assert!(!unavailable.input.validation_complete);
        assert!(!failed.input.validation_complete);

        // A reconstructed controller replays the durable passed attempt rather
        // than adding a duplicate, and completion blocks genuinely new work.
        let mut reopened = PackageValidationController::default();
        let existing_outcome = reopened
            .run_installed_host_with(
                &mut repository,
                installed_host_context(project_id.clone()),
                &root.id,
                |request| {
                    Ok((
                        0,
                        fake_installed_host_result(request, Outcome::Passed),
                        vec![],
                    ))
                },
            )
            .expect("durable replay");
        assert_eq!(existing_outcome, InstalledHostValidationOutcome::Existing);
        assert_eq!(
            repository
                .package_validation_phase_summary_for_test(
                    &project_id,
                    PackageValidationPhase::InstalledHost,
                )
                .expect("same attempt")
                .id,
            passed.id
        );
        let mut replayed_failed = PackageValidationController::default();
        replayed_failed
            .run_installed_host_with(
                &mut repository,
                installed_host_context(project_id),
                &root.id,
                |request| {
                    Ok((
                        0,
                        fake_installed_host_result(request, Outcome::Failed),
                        vec![],
                    ))
                },
            )
            .expect("old attempt is replay-only after completion");
        assert_eq!(
            repository
                .package_validation_phase_summary_for_test(
                    &root.project_id,
                    PackageValidationPhase::InstalledHost,
                )
                .expect("complete tail")
                .id,
            passed.id
        );
    }

    #[test]
    fn package_validation_installed_host_fake_runner_failures_persist_nothing_and_release_session()
    {
        let (mut repository, project_id) = ProjectRepository::package_validation_test_repository();
        let root = installed_host_predecessor(&mut repository, &project_id, &"f".repeat(64));
        for response in [
            Err(PackageValidationControllerError::Unavailable),
            Ok((1, Vec::new(), Vec::new())),
            Ok((0, b"{malformed".to_vec(), Vec::new())),
            Ok((0, vec![b'x'; RESULT_MAX_BYTES + 1], Vec::new())),
        ] {
            let mut controller = PackageValidationController::default();
            assert!(controller
                .run_installed_host_with(
                    &mut repository,
                    installed_host_context(project_id.clone()),
                    &root.id,
                    |_| response,
                )
                .is_err());
            assert!(repository
                .package_validation_phase_summary_for_test(
                    &project_id,
                    PackageValidationPhase::InstalledHost,
                )
                .is_err());
        }
        // The controller's per-project/global slot is released after every
        // fake failure: a later verified helper result can record normally.
        let mut controller = PackageValidationController::default();
        controller
            .run_installed_host_with(
                &mut repository,
                installed_host_context(project_id.clone()),
                &root.id,
                |request| {
                    Ok((
                        0,
                        fake_installed_host_result(request, Outcome::Unavailable),
                        vec![],
                    ))
                },
            )
            .expect("later run");
        assert!(repository
            .package_validation_phase_summary_for_test(
                &project_id,
                PackageValidationPhase::InstalledHost,
            )
            .is_ok());
    }
}
