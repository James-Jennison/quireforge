use serde::{Deserialize, Serialize};

pub const PROJECT_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectWorkspaceState {
    Empty,
    Ready,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DirectoryAccessibilityState {
    ConnectedAccessible,
    ConnectedReadOnly,
    MissingOrMoved,
    PermissionDenied,
    RemovableDisconnected,
    NetworkUnavailable,
    GitInvalid,
    SandboxRestricted,
    IdentityChanged,
    VerificationUnknown,
}

impl DirectoryAccessibilityState {
    pub(crate) const fn as_storage_value(self) -> &'static str {
        match self {
            Self::ConnectedAccessible => "connected-accessible",
            Self::ConnectedReadOnly => "connected-read-only",
            Self::MissingOrMoved => "missing-or-moved",
            Self::PermissionDenied => "permission-denied",
            Self::RemovableDisconnected => "removable-disconnected",
            Self::NetworkUnavailable => "network-unavailable",
            Self::GitInvalid => "git-invalid",
            Self::SandboxRestricted => "sandbox-restricted",
            Self::IdentityChanged => "identity-changed",
            Self::VerificationUnknown => "verification-unknown",
        }
    }

    pub(crate) fn from_storage_value(value: &str) -> Option<Self> {
        Some(match value {
            "connected-accessible" => Self::ConnectedAccessible,
            "connected-read-only" => Self::ConnectedReadOnly,
            "missing-or-moved" => Self::MissingOrMoved,
            "permission-denied" => Self::PermissionDenied,
            "removable-disconnected" => Self::RemovableDisconnected,
            "network-unavailable" => Self::NetworkUnavailable,
            "git-invalid" => Self::GitInvalid,
            "sandbox-restricted" => Self::SandboxRestricted,
            "identity-changed" => Self::IdentityChanged,
            "verification-unknown" => Self::VerificationUnknown,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExpectedAccess {
    ReadWrite,
}

impl ExpectedAccess {
    pub(crate) const fn as_storage_value(self) -> &'static str {
        match self {
            Self::ReadWrite => "read-write",
        }
    }

    pub(crate) fn from_storage_value(value: &str) -> Option<Self> {
        match value {
            "read-write" => Some(Self::ReadWrite),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PendingAttachmentKind {
    Attach,
    Relink,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectDiagnosticCode {
    MetadataUnavailable,
    PickerUnavailable,
    DirectoryUnavailable,
    DuplicateDirectory,
    ProjectNotFound,
    ProjectBusy,
    AttachmentNotPending,
    IdentityChanged,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitSummary {
    pub is_repository: bool,
    pub is_linked_worktree: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectorySummary {
    pub association_id: String,
    pub display_path: String,
    pub resolved_display_path: Option<String>,
    pub state: DirectoryAccessibilityState,
    pub expected_access: ExpectedAccess,
    pub is_primary: bool,
    pub git: GitSummary,
    pub has_agents_guidance: bool,
    pub has_codex_config: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: String,
    pub display_name: String,
    pub archived: bool,
    pub directory: Option<DirectorySummary>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingAttachmentPreview {
    pub operation: PendingAttachmentKind,
    pub project_id: Option<String>,
    pub display_name: String,
    pub selected_display_path: String,
    pub resolved_display_path: String,
    pub state: DirectoryAccessibilityState,
    pub git: GitSummary,
    pub has_agents_guidance: bool,
    pub has_codex_config: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspaceSnapshot {
    pub schema_version: u16,
    pub state: ProjectWorkspaceState,
    pub projects: Vec<ProjectSummary>,
    pub pending_attachment: Option<PendingAttachmentPreview>,
    pub diagnostic_code: Option<ProjectDiagnosticCode>,
}

impl ProjectWorkspaceSnapshot {
    pub(crate) fn unavailable(diagnostic_code: ProjectDiagnosticCode) -> Self {
        Self {
            schema_version: PROJECT_SCHEMA_VERSION,
            state: ProjectWorkspaceState::Unavailable,
            projects: Vec::new(),
            pending_attachment: None,
            diagnostic_code: Some(diagnostic_code),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPreflightSnapshot {
    pub schema_version: u16,
    pub project_id: String,
    pub cwd_ready: bool,
    pub display_path: Option<String>,
    pub state: DirectoryAccessibilityState,
    pub diagnostic_code: Option<ProjectDiagnosticCode>,
}

pub const TASK_RECORD_SCHEMA_VERSION: u16 = 1;
pub const LOCAL_REVIEW_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalReviewCollectionState {
    Active,
    Frozen,
    Orphaned,
    Unavailable,
    Discarded,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalReviewItemClass {
    Text,
    ImageMockup,
    Evidence,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalReviewTextFormat {
    Plain,
    Markdown,
    Json,
    Csv,
    Python,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalReviewItemState {
    Ready,
    Stale,
    Unavailable,
    Discarded,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalReviewAnnotationState {
    Open,
    Resolved,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalReviewComparisonState {
    Ready,
    Stale,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalReviewLineKind {
    Unchanged,
    Added,
    Removed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalReviewSourceKind {
    UserAuthoredText,
    M48ArtifactCopy,
    NativeImageInput,
    TypedEvidenceSnapshot,
}

/// Closed, copied evidence origins. These labels are envelope data, never live
/// handles, paths, or capability claims.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalReviewEvidenceSource {
    ManualValidationSummary,
    M48GeneratedArtifactMetadata,
    SafePreviewMetadata,
    GitStatusDiffSummary,
    ActivityPresentation,
    ApprovalPresentation,
    PackageManifestSummary,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalReviewValidationState {
    Passed,
    Failed,
    Mixed,
    NotRun,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewManualValidationDetails {
    pub validation_state: LocalReviewValidationState,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalReviewEvidenceArtifactState {
    Ready,
    Saving,
    Expired,
    Saved,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalReviewEvidenceArtifactKind {
    Text,
    Markdown,
    Json,
    Csv,
    Python,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewM48GeneratedArtifactMetadataDetails {
    pub artifact_state: LocalReviewEvidenceArtifactState,
    pub artifact_kind: LocalReviewEvidenceArtifactKind,
    pub format: LocalReviewTextFormat,
    pub byte_length: u32,
    pub truncated: bool,
    pub manifest_sha256: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalReviewEvidencePreviewState {
    Empty,
    Ready,
    Unavailable,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalReviewEvidencePreviewKind {
    Text,
    Image,
    Pdf,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalReviewEvidencePreviewRendering {
    NormalizedText,
    BoundedImage,
    MetadataOnly,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum LocalReviewEvidenceMediaType {
    #[serde(rename = "text/plain; charset=utf-8")]
    TextPlain,
    #[serde(rename = "image/png")]
    ImagePng,
    #[serde(rename = "image/jpeg")]
    ImageJpeg,
    #[serde(rename = "application/pdf")]
    ApplicationPdf,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewSafePreviewMetadataDetails {
    pub preview_state: LocalReviewEvidencePreviewState,
    pub kind: LocalReviewEvidencePreviewKind,
    pub rendering: LocalReviewEvidencePreviewRendering,
    pub media_type: LocalReviewEvidenceMediaType,
    pub byte_length: u32,
    pub truncated: bool,
    pub width_px: Option<u32>,
    pub height_px: Option<u32>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[allow(
    dead_code,
    reason = "The closed evidence vocabulary is intentionally defined before a native source-specific capture claim exists."
)]
#[serde(rename_all = "kebab-case")]
pub enum LocalReviewEvidenceWorkspaceState {
    Clean,
    Ready,
    Unavailable,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewGitStatusDiffSummaryDetails {
    pub workspace_state: LocalReviewEvidenceWorkspaceState,
    pub dirty: bool,
    pub staged_count: u32,
    pub modified_count: u32,
    pub added_count: u32,
    pub deleted_count: u32,
    pub renamed_count: u32,
    pub untracked_count: u32,
    pub conflicted_count: u32,
    pub changed_file_count: u32,
    pub additions: u32,
    pub deletions: u32,
    pub diff_available: bool,
    pub diff_truncated: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewActivityPresentationDetails {
    pub scope: LocalReviewActivityScope,
    pub event_count: u8,
    pub item_added_count: u8,
    pub item_discarded_count: u8,
    pub annotation_changed_count: u8,
    pub comparison_changed_count: u8,
    pub promotion_prepared_count: u8,
    pub promotion_completed_count: u8,
    pub collection_changed_count: u8,
    pub truncated: bool,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[allow(
    dead_code,
    reason = "The activity scope is reserved together with its closed evidence envelope."
)]
#[serde(rename_all = "kebab-case")]
pub enum LocalReviewActivityScope {
    CurrentSession,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalReviewEvidenceApprovalState {
    None,
    Pending,
    Approved,
    Rejected,
    Expired,
    Unavailable,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewApprovalPresentationDetails {
    pub approval_state: LocalReviewEvidenceApprovalState,
    pub request_present: bool,
    pub decision_present: bool,
    pub dispatch_present: bool,
    pub execution_present: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalReviewEvidenceCheckState {
    Passed,
    Failed,
    Skipped,
    Unavailable,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewPackageManifestSummaryDetails {
    pub application_version: String,
    pub debian_version: String,
    pub manifest_state: LocalReviewEvidenceCheckState,
    pub checksum_state: LocalReviewEvidenceCheckState,
    pub abi_state: LocalReviewEvidenceCheckState,
    pub provenance_state: LocalReviewEvidenceCheckState,
    pub visible_launch_state: LocalReviewEvidenceCheckState,
    pub installed_host_state: LocalReviewEvidenceCheckState,
    pub artifact_count: u32,
    pub validation_complete: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvidenceEnvelopeV1<T> {
    pub schema_version: u16,
    pub source: LocalReviewEvidenceSource,
    pub source_schema_version: u16,
    pub title: String,
    pub summary: String,
    pub details: T,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalReviewDiagnosticCode {
    MetadataUnavailable,
    InvalidRequest,
    CollectionCapacity,
    ActiveCollectionCapacity,
    ItemCapacity,
    ImageCapacity,
    EvidenceCapacity,
    PayloadCapacity,
    InvalidContent,
    InvalidLabel,
    InvalidReference,
    TaskUnavailable,
    TaskFrozen,
    PlanUnavailable,
    PlanStale,
    CollectionNotFound,
    ItemNotFound,
    IntegrityFailed,
    StaleWrite,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalReviewCollectionSummary {
    pub collection_id: String,
    pub task_id: String,
    pub plan_id: Option<String>,
    pub title: String,
    pub state: LocalReviewCollectionState,
    pub item_count: u8,
    pub payload_bytes: u64,
    pub updated_at_ms: i64,
    pub warning: bool,
    pub annotation_count_warning: bool,
    pub annotation_byte_warning: bool,
    pub comparison_count_warning: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalReviewItemSummary {
    pub item_id: String,
    pub class: LocalReviewItemClass,
    pub text_format: Option<LocalReviewTextFormat>,
    pub source_kind: LocalReviewSourceKind,
    pub evidence_source: Option<LocalReviewEvidenceSource>,
    pub state: LocalReviewItemState,
    pub title: String,
    pub mime_type: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub byte_size: u64,
    pub line_count: Option<u16>,
    pub sha256: String,
    pub created_at_ms: i64,
    pub annotations: Vec<LocalReviewAnnotationSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalReviewAnnotationSummary {
    pub schema_version: u16,
    pub annotation_id: String,
    pub item_id: String,
    pub text: String,
    pub state: LocalReviewAnnotationState,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalReviewSnapshot {
    pub schema_version: u16,
    pub collections: Vec<LocalReviewCollectionSummary>,
    pub selected_collection: Option<LocalReviewCollectionSummary>,
    pub items: Vec<LocalReviewItemSummary>,
    pub comparisons: Vec<LocalReviewComparisonSummary>,
    pub collection_count: u8,
    pub payload_bytes: u64,
    pub warning: bool,
    pub package_manifest_summary_available: bool,
    pub git_status_diff_summary_available: bool,
    pub activity_presentation_available: bool,
    pub approval_presentation_available: bool,
    pub diagnostic_code: Option<LocalReviewDiagnosticCode>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalReviewComparisonSummary {
    pub schema_version: u16,
    pub comparison_id: String,
    pub collection_id: String,
    pub left_item_id: String,
    pub right_item_id: String,
    pub left_sha256: String,
    pub right_sha256: String,
    pub text_format: LocalReviewTextFormat,
    pub state: LocalReviewComparisonState,
    pub created_at_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalReviewLineRecord {
    pub kind: LocalReviewLineKind,
    pub text: String,
    pub left_line_number: Option<u32>,
    pub right_line_number: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalReviewLineComparison {
    pub comparison_id: String,
    pub left_item_id: String,
    pub left_sha256: String,
    pub right_item_id: String,
    pub right_sha256: String,
    pub text_format: LocalReviewTextFormat,
    pub state: LocalReviewComparisonState,
    pub lines: Vec<LocalReviewLineRecord>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewListRequest {
    pub selected_collection_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewCollectionCreateRequest {
    pub task_id: String,
    pub plan_id: Option<String>,
    pub title: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewTextItemCreateRequest {
    pub collection_id: String,
    pub expected_collection_updated_at_ms: i64,
    pub title: String,
    pub text_format: LocalReviewTextFormat,
    pub content: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewM48ArtifactCopyRequest {
    pub collection_id: String,
    pub expected_collection_updated_at_ms: i64,
    pub artifact_id: String,
    pub manifest_sha256: String,
}

/// Fixed, metadata-only claim for one live M48 generated artifact. The
/// frontend never supplies artifact content or metadata for evidence capture.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewM48GeneratedArtifactMetadataEvidenceRequest {
    pub collection_id: String,
    pub expected_collection_updated_at_ms: i64,
    pub artifact_id: String,
    pub manifest_sha256: String,
}

/// The package record is selected solely from the immutable task/project
/// association.  Callers cannot nominate a project, record, or package fact.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewPackageManifestSummaryEvidenceRequest {
    pub collection_id: String,
    pub expected_collection_updated_at_ms: i64,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewGitStatusDiffSummaryEvidenceRequest {
    pub collection_id: String,
    pub expected_collection_updated_at_ms: i64,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewActivityPresentationEvidenceRequest {
    pub collection_id: String,
    pub expected_collection_updated_at_ms: i64,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewApprovalPresentationEvidenceRequest {
    pub collection_id: String,
    pub expected_collection_updated_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewSafePreviewMetadataEvidenceRequest {
    pub collection_id: String,
    pub expected_collection_updated_at_ms: i64,
    pub preview_claim_id: String,
    pub claim_sha256: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewManualEvidenceCreateRequest {
    pub collection_id: String,
    pub expected_collection_updated_at_ms: i64,
    pub title: String,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewCollectionMutationRequest {
    pub collection_id: String,
    pub expected_collection_updated_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewItemDiscardRequest {
    pub collection_id: String,
    pub item_id: String,
    pub expected_collection_updated_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewComparisonCreateRequest {
    pub collection_id: String,
    pub left_item_id: String,
    pub right_item_id: String,
    pub expected_collection_updated_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewComparisonReadRequest {
    pub collection_id: String,
    pub comparison_id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewComparisonDiscardRequest {
    pub collection_id: String,
    pub comparison_id: String,
    pub expected_collection_updated_at_ms: i64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalReviewPromotionReservationState {
    Prepared,
    Consumed,
    Expired,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewPromotionPrepareRequest {
    pub collection_id: String,
    pub item_id: String,
    pub expected_collection_updated_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewPromotionReservationRequest {
    pub reservation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalReviewPromotionCandidate {
    pub reservation_id: String,
    pub collection_id: String,
    pub item_id: String,
    pub title: String,
    pub sha256: String,
    pub text_format: LocalReviewTextFormat,
    pub destination_class: String,
    pub task_id: String,
    pub plan_id: Option<String>,
    pub created_at_ms: u64,
    pub expires_at_ms: u64,
    pub state: LocalReviewPromotionReservationState,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewAnnotationCreateRequest {
    pub collection_id: String,
    pub item_id: String,
    pub expected_collection_updated_at_ms: i64,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewAnnotationEditRequest {
    pub collection_id: String,
    pub item_id: String,
    pub annotation_id: String,
    pub expected_collection_updated_at_ms: i64,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewAnnotationMutationRequest {
    pub collection_id: String,
    pub item_id: String,
    pub annotation_id: String,
    pub expected_collection_updated_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewImagePickRequest {
    pub collection_id: String,
    pub expected_collection_updated_at_ms: i64,
    pub title: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewImagePreviewRequest {
    pub item_id: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalReviewTextPreviewRequest {
    pub collection_id: String,
    pub item_id: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalReviewTextPreview {
    pub schema_version: u16,
    pub collection_id: String,
    pub item_id: String,
    pub title: Option<String>,
    pub text_format: Option<LocalReviewTextFormat>,
    pub byte_size: Option<u64>,
    pub sha256: Option<String>,
    pub created_at_ms: Option<i64>,
    pub state: LocalReviewItemState,
    pub text: Option<String>,
    pub projected_byte_size: u64,
    pub projected_line_count: u16,
    pub projected_code_point_count: u16,
    pub truncated: bool,
    pub diagnostic_code: Option<LocalReviewDiagnosticCode>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalReviewImagePreview {
    pub schema_version: u16,
    pub item_id: String,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
    pub byte_size: u64,
    pub sha256: String,
    pub data_url: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalReviewManualEvidencePreview {
    pub schema_version: u16,
    pub item_id: String,
    pub source: String,
    pub title: String,
    pub summary: String,
    pub byte_size: u64,
    pub sha256: String,
    pub created_at_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalReviewM48GeneratedArtifactMetadataEvidencePreview {
    pub schema_version: u16,
    pub item_id: String,
    pub source: LocalReviewEvidenceSource,
    pub title: String,
    pub summary: String,
    pub details: LocalReviewM48GeneratedArtifactMetadataDetails,
    pub byte_size: u64,
    pub sha256: String,
    pub created_at_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalReviewSafePreviewMetadataEvidencePreview {
    pub schema_version: u16,
    pub item_id: String,
    pub source: LocalReviewEvidenceSource,
    pub title: String,
    pub summary: String,
    pub details: LocalReviewSafePreviewMetadataDetails,
    pub byte_size: u64,
    pub sha256: String,
    pub created_at_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalReviewPackageManifestSummaryEvidencePreview {
    pub schema_version: u16,
    pub item_id: String,
    pub source: LocalReviewEvidenceSource,
    pub title: String,
    pub summary: String,
    pub details: LocalReviewPackageManifestSummaryDetails,
    pub byte_size: u64,
    pub sha256: String,
    pub created_at_ms: i64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalReviewGitStatusDiffSummaryEvidencePreview {
    pub schema_version: u16,
    pub item_id: String,
    pub source: LocalReviewEvidenceSource,
    pub title: String,
    pub summary: String,
    pub details: LocalReviewGitStatusDiffSummaryDetails,
    pub byte_size: u64,
    pub sha256: String,
    pub created_at_ms: i64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalReviewActivityPresentationEvidencePreview {
    pub schema_version: u16,
    pub item_id: String,
    pub source: LocalReviewEvidenceSource,
    pub title: String,
    pub summary: String,
    pub details: LocalReviewActivityPresentationDetails,
    pub byte_size: u64,
    pub sha256: String,
    pub created_at_ms: i64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalReviewApprovalPresentationEvidencePreview {
    pub schema_version: u16,
    pub item_id: String,
    pub source: LocalReviewEvidenceSource,
    pub title: String,
    pub summary: String,
    pub details: LocalReviewApprovalPresentationDetails,
    pub byte_size: u64,
    pub sha256: String,
    pub created_at_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "outcome", rename_all = "kebab-case")]
pub enum LocalReviewManualEvidenceCreateResult {
    Created {
        created_item_id: String,
        source: LocalReviewEvidenceSource,
        snapshot: LocalReviewSnapshot,
    },
    Failed {
        snapshot: LocalReviewSnapshot,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "outcome", rename_all = "kebab-case")]
pub enum LocalReviewImagePickOutcome {
    Created { snapshot: LocalReviewSnapshot },
    Canceled { snapshot: LocalReviewSnapshot },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    Active,
    Paused,
    Completed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskPlanSummary {
    pub id: String,
    pub label: String,
    pub position: u8,
    pub body: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRecordSummary {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
    pub archived: bool,
    pub selected_plan_id: String,
    pub plan_count: u8,
    pub updated_at_ms: i64,
    pub cleanup_eligible: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskCatalogSnapshot {
    pub schema_version: u16,
    pub state: TaskCatalogState,
    pub tasks: Vec<TaskRecordSummary>,
    pub selected_task: Option<TaskRecordSummary>,
    pub plans: Vec<TaskPlanSummary>,
    pub task_count: u16,
    pub payload_bytes: u64,
    pub warning: bool,
    pub diagnostic_code: Option<TaskDiagnosticCode>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskCatalogState {
    Empty,
    Ready,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskDiagnosticCode {
    MetadataUnavailable,
    InvalidRequest,
    CapacityReached,
    TaskNotFound,
    TaskArchived,
    PlanNotFound,
    InvalidStatusTransition,
    DuplicateId,
    InvalidStoredValue,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskCatalogListRequest {
    pub query: Option<String>,
    pub include_archived: bool,
    pub selected_task_id: Option<String>,
}

/// Creates a task from a conversation context already owned by native project
/// metadata. The context resolves its project binding natively; callers never
/// assert a project identity.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskCatalogContextCreateRequest {
    pub conversation_id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskIdRequest {
    pub task_id: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskTitleRequest {
    pub task_id: String,
    pub title: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskStatusRequest {
    pub task_id: String,
    pub status: TaskStatus,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanCreateRequest {
    pub task_id: String,
    pub copy_primary_body: bool,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanIdRequest {
    pub task_id: String,
    pub plan_id: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanEditRequest {
    pub task_id: String,
    pub plan_id: String,
    pub label: String,
    pub body: String,
}

pub const TASK_TEMPLATE_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskTemplateBridgeState {
    Ready,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskTemplateDiagnosticCode {
    MetadataUnavailable,
    InvalidRequest,
    NotFound,
    BuiltInImmutable,
    ArchivedReadOnly,
    ActiveAlready,
    ArchivedAlready,
    Stale,
    CapacityReached,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskTemplateOrigin {
    BuiltIn,
    Local,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskTemplateState {
    Active,
    Archived,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskTemplateSummary {
    pub id: String,
    pub title: String,
    pub purpose: String,
    pub origin: TaskTemplateOrigin,
    pub state: TaskTemplateState,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskTemplateDetail {
    pub id: String,
    pub title: String,
    pub purpose: String,
    pub instructions: String,
    pub origin: TaskTemplateOrigin,
    pub state: TaskTemplateState,
    pub version: u32,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskTemplateCapacity {
    pub record_count: u16,
    pub canonical_bytes: u32,
    pub warning: bool,
    pub count_limit: u16,
    pub canonical_byte_limit: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskTemplateCatalogSnapshot {
    pub schema_version: u16,
    pub state: TaskTemplateBridgeState,
    pub templates: Vec<TaskTemplateSummary>,
    pub capacity: Option<TaskTemplateCapacity>,
    pub diagnostic_code: Option<TaskTemplateDiagnosticCode>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskTemplateInspectionSnapshot {
    pub schema_version: u16,
    pub state: TaskTemplateBridgeState,
    pub template: Option<TaskTemplateDetail>,
    pub mutation_handle: Option<String>,
    pub diagnostic_code: Option<TaskTemplateDiagnosticCode>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskTemplateApplicationChecklist {
    pub template_active: bool,
    pub task_plan_available: bool,
    pub exact_draft_required: bool,
    pub confirmation_required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskTemplatePreviewSnapshot {
    pub schema_version: u16,
    pub state: TaskTemplateBridgeState,
    pub reservation_id: Option<String>,
    pub binding_sha256: Option<String>,
    pub expires_at_ms: Option<i64>,
    pub checklist: Option<TaskTemplateApplicationChecklist>,
    pub diagnostic_code: Option<TaskTemplateDiagnosticCode>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskTemplateApplicationOutcome {
    pub schema_version: u16,
    pub state: TaskTemplateBridgeState,
    pub applied: bool,
    pub cancelled: bool,
    pub diagnostic_code: Option<TaskTemplateDiagnosticCode>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskTemplateIdRequest {
    pub template_id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskTemplateContentRequest {
    pub title: String,
    pub purpose: String,
    pub instructions: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskTemplateMutationRequest {
    pub mutation_handle: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskTemplateEditRequest {
    pub mutation_handle: String,
    pub title: String,
    pub purpose: String,
    pub instructions: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskTemplateDeleteRequest {
    pub mutation_handle: String,
    pub confirmation: TaskTemplateDeletionConfirmation,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskTemplateDeletionConfirmation {
    Confirmed,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskTemplatePreviewRequest {
    pub template_id: String,
    pub task_id: String,
    pub plan_id: String,
    pub title: String,
    pub plan_text: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskTemplateConfirmRequest {
    pub reservation_id: String,
    pub title: String,
    pub plan_text: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskTemplateCancelRequest {
    pub reservation_id: String,
}
