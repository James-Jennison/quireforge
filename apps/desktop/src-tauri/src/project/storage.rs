use std::{
    collections::HashSet,
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rusqlite::{params, Connection, OptionalExtension, Row, Transaction, TransactionBehavior};
use sha2::{Digest, Sha256};
use thiserror::Error;
use unicode_casefold::{Locale, UnicodeCaseFold, Variant};
use uuid::Uuid;

use crate::advisor::{
    AdvisorContextKind, AdvisorContextReference, AdvisorConversationReference,
    AdvisorDispatchProposal, AdvisorDispatchState, AdvisorExecutionDispatchState,
    AdvisorFoundationSnapshot, AdvisorFreshness, AdvisorProvenance, AdvisorProvenanceSource,
    AdvisorTrust,
};
use crate::preview::validate_attachment_image;

use super::task_template::{
    builtins, canonical as canonical_template, valid as valid_template, TaskTemplate,
    TemplateOrigin, TemplateState, TEMPLATE_COUNT_LIMIT, TEMPLATE_PAYLOAD_LIMIT,
    TEMPLATE_SCHEMA_VERSION,
};
use super::types::{
    ArtifactReferenceState, ArtifactReferenceSummary, ContextLedgerEntry, DurableSourceClass,
    DurableSourceLifecycleState, DurableSourceSummary, EvidenceEnvelopeV1, KnowledgeRecordKind,
    KnowledgeRecordStatus, KnowledgeRecordSummary, LocalReviewActivityPresentationDetails,
    LocalReviewActivityPresentationEvidencePreview, LocalReviewActivityScope,
    LocalReviewAnnotationState, LocalReviewAnnotationSummary,
    LocalReviewApprovalPresentationDetails, LocalReviewApprovalPresentationEvidencePreview,
    LocalReviewCollectionState, LocalReviewCollectionSummary, LocalReviewComparisonState,
    LocalReviewComparisonSummary, LocalReviewDiagnosticCode, LocalReviewEvidenceApprovalState,
    LocalReviewEvidenceCheckState, LocalReviewEvidenceSource, LocalReviewEvidenceWorkspaceState,
    LocalReviewGitStatusDiffSummaryDetails, LocalReviewGitStatusDiffSummaryEvidencePreview,
    LocalReviewImagePreview, LocalReviewItemClass, LocalReviewItemState, LocalReviewItemSummary,
    LocalReviewLineComparison, LocalReviewLineKind, LocalReviewLineRecord,
    LocalReviewM48GeneratedArtifactMetadataDetails,
    LocalReviewM48GeneratedArtifactMetadataEvidencePreview, LocalReviewManualEvidencePreview,
    LocalReviewManualValidationDetails, LocalReviewPackageManifestSummaryDetails,
    LocalReviewPackageManifestSummaryEvidencePreview, LocalReviewSafePreviewMetadataDetails,
    LocalReviewSafePreviewMetadataEvidencePreview, LocalReviewSourceKind, LocalReviewTextFormat,
    LocalReviewTextPreview, LocalReviewValidationState, TaskPlanSummary, TaskRecordSummary,
    TaskStatus,
};
use super::{
    identity::DirectoryIdentity,
    types::{DirectoryAccessibilityState, ExpectedAccess},
    AdvisorConversationMetadata, ChatConversationMetadata, ControlledBrowserVerificationRecord,
    ConversationReference, ConversationSelectionMetadata, FictionalConnectorOperationRecord,
};

const INITIAL_MIGRATION: &str = r#"
CREATE TABLE projects (
    id TEXT PRIMARY KEY NOT NULL,
    display_name TEXT NOT NULL CHECK(length(display_name) BETWEEN 1 AND 120),
    active_directory_association_id TEXT,
    archived_at_ms INTEGER,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    FOREIGN KEY(active_directory_association_id)
        REFERENCES directory_associations(id)
        DEFERRABLE INITIALLY DEFERRED
);

CREATE TABLE directory_associations (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL,
    selected_path TEXT NOT NULL,
    resolved_path TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('primary', 'additional-writable', 'read-only-context')),
    is_primary INTEGER NOT NULL CHECK(is_primary IN (0, 1)),
    expected_access TEXT NOT NULL CHECK(expected_access IN ('read-write')),
    device_id TEXT,
    inode TEXT,
    filesystem_type TEXT,
    mount_id TEXT,
    git_common_dir TEXT,
    git_worktree_root TEXT,
    git_is_linked_worktree INTEGER NOT NULL CHECK(git_is_linked_worktree IN (0, 1)),
    has_agents_guidance INTEGER NOT NULL CHECK(has_agents_guidance IN (0, 1)),
    has_codex_config INTEGER NOT NULL CHECK(has_codex_config IN (0, 1)),
    accessibility_state TEXT NOT NULL,
    last_verified_at_ms INTEGER NOT NULL,
    detached_at_ms INTEGER,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE RESTRICT
);

CREATE UNIQUE INDEX active_directory_resolved_path
    ON directory_associations(resolved_path)
    WHERE detached_at_ms IS NULL;
CREATE INDEX directory_associations_project
    ON directory_associations(project_id, is_primary, detached_at_ms);
CREATE INDEX projects_archive_state ON projects(archived_at_ms, updated_at_ms);
"#;

const CONVERSATION_REFERENCES_MIGRATION: &str = r#"
CREATE TABLE conversation_references (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL,
    codex_thread_id TEXT NOT NULL UNIQUE,
    active_turn_id TEXT,
    model_id TEXT NOT NULL,
    reasoning_effort TEXT NOT NULL,
    sandbox_mode TEXT NOT NULL CHECK(sandbox_mode IN (
        'read-only', 'workspace-write', 'danger-full-access'
    )),
    approval_policy TEXT NOT NULL CHECK(approval_policy IN (
        'untrusted', 'on-request', 'never'
    )),
    status TEXT NOT NULL CHECK(status IN (
        'thread-started', 'running', 'stopping', 'completed', 'interrupted',
        'blocked', 'failed'
    )),
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE RESTRICT
);

CREATE INDEX conversation_references_project
    ON conversation_references(project_id, updated_at_ms DESC);
"#;

const SESSION_LIFECYCLE_MIGRATION: &str = r#"
ALTER TABLE conversation_references
    ADD COLUMN parent_conversation_id TEXT
    REFERENCES conversation_references(id) ON DELETE RESTRICT;
ALTER TABLE conversation_references ADD COLUMN archived_at_ms INTEGER;

CREATE INDEX conversation_references_parent
    ON conversation_references(parent_conversation_id, created_at_ms);
"#;

const WORKTREE_RELATIONS_MIGRATION: &str = r#"
CREATE TABLE worktree_relations (
    id TEXT PRIMARY KEY NOT NULL,
    source_project_id TEXT NOT NULL,
    worktree_project_id TEXT NOT NULL UNIQUE,
    ownership TEXT NOT NULL CHECK(ownership IN ('managed', 'attached')),
    branch_name TEXT,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    CHECK(source_project_id <> worktree_project_id),
    CHECK(branch_name IS NULL OR length(branch_name) BETWEEN 1 AND 96),
    FOREIGN KEY(source_project_id) REFERENCES projects(id) ON DELETE RESTRICT,
    FOREIGN KEY(worktree_project_id) REFERENCES projects(id) ON DELETE RESTRICT
);

CREATE INDEX worktree_relations_source
    ON worktree_relations(source_project_id, created_at_ms, id);
"#;

const TERMINAL_SESSIONS_MIGRATION: &str = r#"
CREATE TABLE terminal_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL,
    title TEXT NOT NULL CHECK(length(title) BETWEEN 1 AND 80),
    status TEXT NOT NULL CHECK(status IN (
        'running', 'closing', 'exited', 'interrupted', 'failed'
    )),
    columns INTEGER NOT NULL CHECK(columns BETWEEN 2 AND 500),
    rows INTEGER NOT NULL CHECK(rows BETWEEN 2 AND 200),
    exit_code INTEGER,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE RESTRICT
);

CREATE INDEX terminal_sessions_recent
    ON terminal_sessions(updated_at_ms DESC, id);
CREATE INDEX terminal_sessions_project
    ON terminal_sessions(project_id, updated_at_ms DESC, id);
"#;

const MODEL_SELECTION_MIGRATION: &str = r#"
ALTER TABLE conversation_references
    ADD COLUMN selector_availability TEXT NOT NULL DEFAULT 'recommendation-only'
    CHECK(selector_availability IN ('ready', 'recommendation-only', 'unavailable'));
ALTER TABLE conversation_references
    ADD COLUMN selector_mode TEXT NOT NULL DEFAULT 'manual'
    CHECK(selector_mode IN ('manual', 'recommend', 'automatic'));
ALTER TABLE conversation_references
    ADD COLUMN selector_user_locked INTEGER NOT NULL DEFAULT 0
    CHECK(selector_user_locked IN (0, 1));
ALTER TABLE conversation_references
    ADD COLUMN selector_allowed_model_ids_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE conversation_references ADD COLUMN selector_reasoning_ceiling TEXT;
ALTER TABLE conversation_references ADD COLUMN selector_pending_model_id TEXT;
ALTER TABLE conversation_references ADD COLUMN selector_pending_reasoning_effort TEXT;
ALTER TABLE conversation_references ADD COLUMN selector_pending_rationale TEXT;
ALTER TABLE conversation_references
    ADD COLUMN selector_pending_provenance TEXT
    CHECK(selector_pending_provenance IS NULL OR selector_pending_provenance IN ('user', 'codex'));
ALTER TABLE conversation_references
    ADD COLUMN selector_pending_application TEXT
    CHECK(selector_pending_application IS NULL OR selector_pending_application IN (
        'manual', 'recommendation', 'automatic'
    ));
ALTER TABLE conversation_references ADD COLUMN selector_pending_requested_at_ms INTEGER;
"#;

// This table deliberately stores only bounded QuireForge conversation metadata.
// Codex continues to own threads, account state, credentials, and transcripts.
// Existing project-scoped references are backfilled as Codex records; Chat rows
// cannot carry an attached-project or legacy project-reference association.
const UNIFIED_CONVERSATION_METADATA_MIGRATION: &str = r#"
CREATE TABLE unified_conversation_metadata (
    id TEXT PRIMARY KEY NOT NULL,
    mode TEXT NOT NULL CHECK(mode IN ('chat', 'codex')),
    project_id TEXT REFERENCES projects(id) ON DELETE RESTRICT,
    conversation_reference_id TEXT UNIQUE
        REFERENCES conversation_references(id) ON DELETE RESTRICT,
    codex_thread_id TEXT UNIQUE,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    CHECK(
        (mode = 'chat' AND project_id IS NULL AND conversation_reference_id IS NULL
         AND codex_thread_id IS NOT NULL)
        OR
        (mode = 'codex' AND project_id IS NOT NULL AND conversation_reference_id IS NOT NULL
         AND codex_thread_id IS NULL)
    )
);

CREATE INDEX unified_conversation_metadata_mode_recent
    ON unified_conversation_metadata(mode, updated_at_ms DESC, id);
CREATE INDEX unified_conversation_metadata_project_recent
    ON unified_conversation_metadata(project_id, updated_at_ms DESC, id);

INSERT INTO unified_conversation_metadata (
    id, mode, project_id, conversation_reference_id, created_at_ms, updated_at_ms
)
SELECT id, 'codex', project_id, id, created_at_ms, updated_at_ms
FROM conversation_references;
"#;

// The Advisor foundation persists references and digests only. It must never
// retain prompt bodies, replies, credentials, or arbitrary project paths.
// Reading selected context and dispatching a proposal are intentionally
// deferred to later, separately approved milestones.
const ADVISOR_REFERENCE_FOUNDATION_MIGRATION: &str = r#"
CREATE TABLE advisor_conversations (
    id TEXT PRIMARY KEY NOT NULL,
    codex_thread_id TEXT NOT NULL UNIQUE,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    CHECK(length(id) = 36),
    CHECK(length(codex_thread_id) BETWEEN 1 AND 160)
);

CREATE TABLE advisor_context_references (
    id TEXT PRIMARY KEY NOT NULL,
    advisor_conversation_id TEXT NOT NULL,
    kind TEXT NOT NULL CHECK(kind IN (
        'project-state', 'roadmap', 'current-state', 'execution-report'
    )),
    source_ref TEXT NOT NULL CHECK(length(source_ref) BETWEEN 1 AND 96),
    source_commit TEXT CHECK(source_commit IS NULL OR length(source_commit) = 40),
    source_sha256 TEXT NOT NULL CHECK(length(source_sha256) = 64),
    selected_at_ms INTEGER NOT NULL,
    freshness TEXT NOT NULL CHECK(freshness IN (
        'current', 'stale', 'unknown', 'conflicting', 'not-applicable'
    )),
    trust TEXT NOT NULL CHECK(trust IN ('verified', 'reported', 'inferred', 'unknown')),
    provenance_source TEXT NOT NULL CHECK(provenance_source IN (
        'git-observation', 'project-state-snapshot', 'repository-document',
        'execution-report', 'user-selection', 'unknown'
    )),
    provenance_ref TEXT CHECK(provenance_ref IS NULL OR length(provenance_ref) BETWEEN 1 AND 96),
    provenance_commit TEXT CHECK(provenance_commit IS NULL OR length(provenance_commit) = 40),
    observed_at_ms INTEGER,
    provenance_note TEXT CHECK(provenance_note IS NULL OR length(provenance_note) <= 512),
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    FOREIGN KEY(advisor_conversation_id) REFERENCES advisor_conversations(id) ON DELETE RESTRICT
);

CREATE INDEX advisor_context_references_conversation
    ON advisor_context_references(advisor_conversation_id, selected_at_ms DESC, id);

CREATE TABLE advisor_dispatch_records (
    id TEXT PRIMARY KEY NOT NULL,
    advisor_conversation_id TEXT NOT NULL,
    target_project_id TEXT NOT NULL,
    request_sha256 TEXT NOT NULL CHECK(length(request_sha256) = 64),
    context_manifest_sha256 TEXT NOT NULL CHECK(length(context_manifest_sha256) = 64),
    state TEXT NOT NULL CHECK(state IN ('draft', 'approved', 'rejected')),
    requires_explicit_approval INTEGER NOT NULL CHECK(requires_explicit_approval = 1),
    requested_model TEXT CHECK(requested_model IS NULL OR length(requested_model) BETWEEN 1 AND 128),
    requested_reasoning_effort TEXT CHECK(
        requested_reasoning_effort IS NULL OR length(requested_reasoning_effort) BETWEEN 1 AND 64
    ),
    trust TEXT NOT NULL CHECK(trust IN ('verified', 'reported', 'inferred', 'unknown')),
    provenance_source TEXT NOT NULL CHECK(provenance_source IN (
        'git-observation', 'project-state-snapshot', 'repository-document',
        'execution-report', 'user-selection', 'unknown'
    )),
    provenance_ref TEXT CHECK(provenance_ref IS NULL OR length(provenance_ref) BETWEEN 1 AND 96),
    provenance_commit TEXT CHECK(provenance_commit IS NULL OR length(provenance_commit) = 40),
    observed_at_ms INTEGER,
    provenance_note TEXT CHECK(provenance_note IS NULL OR length(provenance_note) <= 512),
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    FOREIGN KEY(advisor_conversation_id) REFERENCES advisor_conversations(id) ON DELETE RESTRICT,
    FOREIGN KEY(target_project_id) REFERENCES projects(id) ON DELETE RESTRICT
);

CREATE INDEX advisor_dispatch_records_conversation
    ON advisor_dispatch_records(advisor_conversation_id, updated_at_ms DESC, id);
"#;

// Phase A stores only binding digests and approval timing. Prompt bodies,
// context content, credentials, and execution results remain absent.
const ADVISOR_APPROVAL_CONTROLLER_MIGRATION: &str = r#"
ALTER TABLE advisor_dispatch_records
    ADD COLUMN capability_manifest_sha256 TEXT NOT NULL DEFAULT '0000000000000000000000000000000000000000000000000000000000000000'
    CHECK(length(capability_manifest_sha256) = 64);
ALTER TABLE advisor_dispatch_records
    ADD COLUMN decided_at_ms INTEGER;
ALTER TABLE advisor_dispatch_records
    ADD COLUMN expires_at_ms INTEGER NOT NULL DEFAULT 0;
"#;

// A B2 handoff is consumed once before the existing project execution service
// is invoked. It retains only an opaque execution reference and fixed status.
const ADVISOR_ONE_TIME_DISPATCH_MIGRATION: &str = r#"
ALTER TABLE advisor_dispatch_records
    ADD COLUMN execution_dispatch_state TEXT CHECK(execution_dispatch_state IS NULL OR execution_dispatch_state IN (
        'dispatching', 'started', 'failed-to-start'
    ));
ALTER TABLE advisor_dispatch_records
    ADD COLUMN execution_conversation_id TEXT CHECK(execution_conversation_id IS NULL OR length(execution_conversation_id) = 36);
"#;

// M52 deliberately added only local organisational metadata. Migration 14
// later adds an immutable nullable native project binding to task records;
// neither task table retains conversation, attachment, artifact, approval,
// dispatch, terminal, browser, connector, credential, or Advisor data.
const DURABLE_TASK_RECORDS_MIGRATION: &str = r#"
CREATE TABLE task_records (
    id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
    schema_version INTEGER NOT NULL CHECK(schema_version = 1),
    title TEXT NOT NULL CHECK(length(title) BETWEEN 1 AND 120),
    status TEXT NOT NULL CHECK(status IN ('active', 'paused', 'completed')),
    created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0),
    updated_at_ms INTEGER NOT NULL CHECK(updated_at_ms >= 0),
    archived_at_ms INTEGER CHECK(archived_at_ms >= 0),
    last_opened_at_ms INTEGER CHECK(last_opened_at_ms >= 0),
    selected_plan_id TEXT NOT NULL CHECK(length(selected_plan_id) = 36)
);
CREATE TABLE task_plans (
    id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
    schema_version INTEGER NOT NULL CHECK(schema_version = 1),
    task_id TEXT NOT NULL CHECK(length(task_id) = 36)
      REFERENCES task_records(id) ON DELETE CASCADE,
    label TEXT NOT NULL CHECK(length(label) BETWEEN 1 AND 80),
    position INTEGER NOT NULL CHECK(position BETWEEN 0 AND 3),
    body TEXT NOT NULL CHECK(length(body) <= 8192),
    created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0),
    updated_at_ms INTEGER NOT NULL CHECK(updated_at_ms >= 0),
    UNIQUE(task_id, position)
);
CREATE INDEX task_records_visible_recent ON task_records(archived_at_ms, updated_at_ms DESC, id);
CREATE INDEX task_records_status_recent ON task_records(archived_at_ms, status, updated_at_ms DESC, id);
CREATE INDEX task_plans_task_position ON task_plans(task_id, position, id);
"#;

// M54 review records are deliberately task-contextual but do not have a
// foreign-key cascade to task records: deleting a task must orphan review
// content rather than erase it. The schema contains copied bounded payloads
// and opaque identifiers only; it never retains source paths, URLs, approval,
// dispatch, execution, provider, or browser state.
const LOCAL_REVIEW_COLLECTIONS_MIGRATION: &str = r#"
CREATE TABLE local_review_collections (
    id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
    schema_version INTEGER NOT NULL CHECK(schema_version = 1),
    task_id TEXT NOT NULL CHECK(length(task_id) = 36),
    plan_id TEXT CHECK(plan_id IS NULL OR length(plan_id) = 36),
    observed_plan_updated_at_ms INTEGER CHECK(observed_plan_updated_at_ms IS NULL OR observed_plan_updated_at_ms >= 0),
    state TEXT NOT NULL CHECK(state IN ('active', 'frozen', 'orphaned', 'unavailable', 'discarded')),
    title TEXT NOT NULL CHECK(length(title) BETWEEN 1 AND 480),
    created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0),
    updated_at_ms INTEGER NOT NULL CHECK(updated_at_ms >= 0),
    discarded_at_ms INTEGER CHECK(discarded_at_ms >= 0)
);
CREATE TABLE local_review_items (
    id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
    schema_version INTEGER NOT NULL CHECK(schema_version = 1),
    collection_id TEXT NOT NULL REFERENCES local_review_collections(id) ON DELETE CASCADE,
    class TEXT NOT NULL CHECK(class IN ('text', 'image-mockup', 'evidence')),
    text_format TEXT CHECK(text_format IS NULL OR text_format IN ('plain', 'markdown', 'json', 'csv', 'python')),
    mime_type TEXT NOT NULL CHECK(mime_type IN (
        'text/plain; charset=utf-8', 'text/markdown; charset=utf-8',
        'application/json', 'text/csv; charset=utf-8', 'text/x-python',
        'image/png', 'image/jpeg', 'application/json; profile=evidence-envelope-v1'
    )),
    width INTEGER CHECK(width IS NULL OR width BETWEEN 1 AND 4096),
    height INTEGER CHECK(height IS NULL OR height BETWEEN 1 AND 4096),
    state TEXT NOT NULL CHECK(state IN ('ready', 'stale', 'unavailable', 'discarded')),
    title TEXT NOT NULL CHECK(length(title) BETWEEN 1 AND 480),
    source_kind TEXT NOT NULL CHECK(source_kind IN (
        'user-authored-text', 'm48-artifact-copy', 'native-image-input',
        'typed-evidence-snapshot'
    )),
    provenance TEXT NOT NULL CHECK(length(provenance) <= 256),
    content BLOB NOT NULL,
    sha256 TEXT NOT NULL CHECK(length(sha256) = 64),
    byte_size INTEGER NOT NULL CHECK(byte_size BETWEEN 1 AND 1048576),
    created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0),
    updated_at_ms INTEGER NOT NULL CHECK(updated_at_ms >= 0),
    discarded_at_ms INTEGER CHECK(discarded_at_ms >= 0),
    CHECK(
        (class = 'text' AND text_format IS NOT NULL AND width IS NULL AND height IS NULL
         AND mime_type IN ('text/plain; charset=utf-8', 'text/markdown; charset=utf-8',
             'application/json', 'text/csv; charset=utf-8', 'text/x-python')
         AND source_kind IN ('user-authored-text', 'm48-artifact-copy')
         AND byte_size <= 262144)
        OR
        (class = 'image-mockup' AND text_format IS NULL AND width IS NOT NULL AND height IS NOT NULL
         AND mime_type IN ('image/png', 'image/jpeg') AND source_kind = 'native-image-input')
        OR
        (class = 'evidence' AND text_format IS NULL AND width IS NULL AND height IS NULL
         AND mime_type = 'application/json; profile=evidence-envelope-v1'
         AND source_kind = 'typed-evidence-snapshot' AND byte_size <= 16384)
    )
);
CREATE TABLE local_review_annotations (
    id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
    schema_version INTEGER NOT NULL CHECK(schema_version = 1),
    item_id TEXT NOT NULL REFERENCES local_review_items(id) ON DELETE CASCADE,
    state TEXT NOT NULL CHECK(state IN ('open', 'resolved')),
    body TEXT NOT NULL CHECK(length(body) BETWEEN 1 AND 1024),
    created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0),
    updated_at_ms INTEGER NOT NULL CHECK(updated_at_ms >= 0)
);
CREATE TABLE local_review_comparisons (
    id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
    schema_version INTEGER NOT NULL CHECK(schema_version = 1),
    collection_id TEXT NOT NULL REFERENCES local_review_collections(id) ON DELETE CASCADE,
    left_item_id TEXT NOT NULL REFERENCES local_review_items(id) ON DELETE RESTRICT,
    right_item_id TEXT NOT NULL REFERENCES local_review_items(id) ON DELETE RESTRICT,
    left_sha256 TEXT NOT NULL CHECK(length(left_sha256) = 64),
    right_sha256 TEXT NOT NULL CHECK(length(right_sha256) = 64),
    text_format TEXT NOT NULL CHECK(text_format IN ('plain', 'markdown', 'json', 'csv', 'python')),
    state TEXT NOT NULL CHECK(state IN ('ready', 'stale', 'unavailable')),
    created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0),
    CHECK(left_item_id <> right_item_id)
);
CREATE INDEX local_review_collections_task_recent ON local_review_collections(task_id, updated_at_ms DESC, id);
CREATE INDEX local_review_items_collection_recent ON local_review_items(collection_id, created_at_ms DESC, id);
CREATE INDEX local_review_annotations_item_recent ON local_review_annotations(item_id, created_at_ms, id);
CREATE INDEX local_review_comparisons_collection_recent ON local_review_comparisons(collection_id, created_at_ms DESC, id);
"#;

// Migration 13 adds a closed envelope source column without rebuilding the
// existing item table. The Rust backfill below rewrites prior v12 manual rows
// atomically to the ratified canonical envelope before the migration is
// recorded.
const LOCAL_REVIEW_EVIDENCE_SOURCES_MIGRATION: &str = r#"
ALTER TABLE local_review_items ADD COLUMN evidence_source TEXT CHECK(
    evidence_source IS NULL OR evidence_source IN (
        'manual-validation-summary',
        'm48-generated-artifact-metadata',
        'safe-preview-metadata',
        'git-status-diff-summary',
        'activity-presentation',
        'approval-presentation',
        'package-manifest-summary'
    )
);
CREATE TRIGGER local_review_items_evidence_source_insert
BEFORE INSERT ON local_review_items
BEGIN
    SELECT CASE WHEN
        (NEW.class = 'evidence' AND (
            NEW.source_kind != 'typed-evidence-snapshot'
            OR NEW.evidence_source IS NULL
            OR NEW.provenance != NEW.evidence_source
        ))
        OR (NEW.class != 'evidence' AND NEW.evidence_source IS NOT NULL)
    THEN RAISE(ABORT, 'invalid local review evidence source') END;
END;
CREATE TRIGGER local_review_items_evidence_source_update
BEFORE UPDATE OF class, source_kind, provenance, evidence_source ON local_review_items
BEGIN
    SELECT CASE WHEN
        (NEW.class = 'evidence' AND (
            NEW.source_kind != 'typed-evidence-snapshot'
            OR NEW.evidence_source IS NULL
            OR NEW.provenance != NEW.evidence_source
        ))
        OR (NEW.class != 'evidence' AND NEW.evidence_source IS NOT NULL)
    THEN RAISE(ABORT, 'invalid local review evidence source') END;
END;
"#;

// Task/project binding is intentionally nullable: existing and no-project
// tasks remain unbound. A binding is written only as part of native-owned
// context-bound task creation and is immutable thereafter.
const TASK_PROJECT_BINDING_MIGRATION: &str = r#"
ALTER TABLE task_records ADD COLUMN project_id TEXT
    CHECK(project_id IS NULL OR length(project_id) = 36)
    REFERENCES projects(id) ON DELETE RESTRICT;

CREATE INDEX task_records_project_binding_idx
    ON task_records(project_id, updated_at_ms DESC, id)
    WHERE project_id IS NOT NULL;

CREATE TRIGGER task_records_project_binding_immutable
BEFORE UPDATE OF project_id ON task_records
WHEN NEW.project_id IS NOT OLD.project_id
BEGIN
    SELECT RAISE(ABORT, 'task project binding is immutable');
END;
"#;

// Immutable, native-owned package-validation summaries. The table stores only
// the redacted, closed result required by a later package-evidence capture;
// package paths, filenames, commands, diagnostics, and artifact bytes never
// enter this schema.
const PROJECT_PACKAGE_VALIDATION_SUMMARIES_MIGRATION: &str = r#"
CREATE TABLE project_package_validation_summaries (
    id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
    application_version TEXT NOT NULL CHECK(
        length(application_version) BETWEEN 1 AND 64
        AND substr(application_version, 1, 1) GLOB '[0-9]'
        AND application_version NOT GLOB '*[^0-9A-Za-z.+-]*'
    ),
    debian_version TEXT NOT NULL CHECK(
        length(debian_version) BETWEEN 1 AND 64
        AND substr(debian_version, 1, 1) GLOB '[0-9]'
        AND debian_version NOT GLOB '*[^0-9A-Za-z.+:~-]*'
    ),
    manifest_state TEXT NOT NULL CHECK(manifest_state IN ('passed', 'failed', 'skipped', 'unavailable')),
    checksum_state TEXT NOT NULL CHECK(checksum_state IN ('passed', 'failed', 'skipped', 'unavailable')),
    abi_state TEXT NOT NULL CHECK(abi_state IN ('passed', 'failed', 'skipped', 'unavailable')),
    provenance_state TEXT NOT NULL CHECK(provenance_state IN ('passed', 'failed', 'skipped', 'unavailable')),
    visible_launch_state TEXT NOT NULL CHECK(visible_launch_state IN ('passed', 'failed', 'skipped', 'unavailable')),
    installed_host_state TEXT NOT NULL CHECK(installed_host_state IN ('passed', 'failed', 'skipped', 'unavailable')),
    artifact_count INTEGER NOT NULL CHECK(artifact_count BETWEEN 0 AND 2),
    validation_complete INTEGER NOT NULL CHECK(validation_complete IN (0, 1)),
    record_sha256 TEXT NOT NULL CHECK(
        length(record_sha256) = 64
        AND record_sha256 NOT GLOB '*[^0-9a-f]*'
    ),
    created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0),
    supersedes_record_id TEXT REFERENCES project_package_validation_summaries(id) ON DELETE RESTRICT,
    CHECK(supersedes_record_id IS NULL OR supersedes_record_id <> id)
);

CREATE INDEX project_package_validation_summaries_project_newest
    ON project_package_validation_summaries(project_id, created_at_ms DESC, id DESC);
CREATE INDEX project_package_validation_summaries_project_newest_complete
    ON project_package_validation_summaries(project_id, created_at_ms DESC, id DESC)
    WHERE validation_complete = 1;
CREATE INDEX project_package_validation_summaries_supersession
    ON project_package_validation_summaries(supersedes_record_id)
    WHERE supersedes_record_id IS NOT NULL;

CREATE TRIGGER project_package_validation_summaries_immutable
BEFORE UPDATE ON project_package_validation_summaries
BEGIN
    SELECT RAISE(ABORT, 'package validation summaries are immutable');
END;
"#;

const PROJECT_PACKAGE_VALIDATION_IDENTITIES_MIGRATION: &str = r#"
CREATE TABLE project_package_validation_candidate_identities (
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
    candidate_identity_sha256 TEXT NOT NULL CHECK(length(candidate_identity_sha256) = 64 AND candidate_identity_sha256 NOT GLOB '*[^0-9a-f]*'),
    package_validation_summary_id TEXT NOT NULL REFERENCES project_package_validation_summaries(id) ON DELETE RESTRICT,
    created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0),
    UNIQUE(project_id, candidate_identity_sha256),
    UNIQUE(package_validation_summary_id)
);
CREATE TRIGGER project_package_validation_candidate_identities_immutable
BEFORE UPDATE ON project_package_validation_candidate_identities
BEGIN SELECT RAISE(ABORT, 'package validation candidate identities are immutable'); END;
CREATE INDEX project_package_validation_candidate_identities_summary
    ON project_package_validation_candidate_identities(package_validation_summary_id);
"#;

const PROJECT_PACKAGE_VALIDATION_PHASED_IDENTITIES_MIGRATION: &str = r#"
CREATE TABLE project_package_validation_candidate_identities_v17 (
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
    candidate_identity_sha256 TEXT NOT NULL CHECK(length(candidate_identity_sha256) = 64 AND candidate_identity_sha256 NOT GLOB '*[^0-9a-f]*'),
    validation_phase TEXT NOT NULL CHECK(validation_phase IN ('unprivileged', 'installed-host')),
    package_validation_summary_id TEXT NOT NULL REFERENCES project_package_validation_summaries(id) ON DELETE RESTRICT,
    created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0),
    UNIQUE(project_id, candidate_identity_sha256, validation_phase),
    UNIQUE(package_validation_summary_id)
);
INSERT INTO project_package_validation_candidate_identities_v17 (
    project_id, candidate_identity_sha256, validation_phase,
    package_validation_summary_id, created_at_ms
) SELECT project_id, candidate_identity_sha256, 'unprivileged',
         package_validation_summary_id, created_at_ms
  FROM project_package_validation_candidate_identities;
DROP TABLE project_package_validation_candidate_identities;
ALTER TABLE project_package_validation_candidate_identities_v17
    RENAME TO project_package_validation_candidate_identities;
CREATE TRIGGER project_package_validation_candidate_identities_immutable
BEFORE UPDATE ON project_package_validation_candidate_identities
BEGIN SELECT RAISE(ABORT, 'package validation candidate identities are immutable'); END;
CREATE INDEX project_package_validation_candidate_identities_summary
    ON project_package_validation_candidate_identities(package_validation_summary_id);
CREATE INDEX project_package_validation_candidate_identities_lookup
    ON project_package_validation_candidate_identities(project_id, candidate_identity_sha256, validation_phase);
"#;

const PROJECT_PACKAGE_VALIDATION_ATTEMPT_IDENTITIES_MIGRATION: &str = r#"
CREATE TABLE project_package_validation_candidate_identities_v18 (
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
    candidate_identity_sha256 TEXT NOT NULL CHECK(length(candidate_identity_sha256) = 64 AND candidate_identity_sha256 NOT GLOB '*[^0-9a-f]*'),
    validation_phase TEXT NOT NULL CHECK(validation_phase IN ('unprivileged', 'installed-host')),
    attempt_identity_sha256 TEXT NOT NULL CHECK(length(attempt_identity_sha256) = 64 AND attempt_identity_sha256 NOT GLOB '*[^0-9a-f]*'),
    package_validation_summary_id TEXT NOT NULL REFERENCES project_package_validation_summaries(id) ON DELETE RESTRICT,
    created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0),
    UNIQUE(package_validation_summary_id),
    UNIQUE(project_id, candidate_identity_sha256, validation_phase, attempt_identity_sha256)
);
INSERT INTO project_package_validation_candidate_identities_v18 (
    project_id, candidate_identity_sha256, validation_phase, attempt_identity_sha256,
    package_validation_summary_id, created_at_ms
) SELECT identity.project_id, identity.candidate_identity_sha256, identity.validation_phase,
         CASE WHEN identity.validation_phase = 'unprivileged'
              THEN identity.candidate_identity_sha256 ELSE summary.record_sha256 END,
         identity.package_validation_summary_id, identity.created_at_ms
  FROM project_package_validation_candidate_identities AS identity
  JOIN project_package_validation_summaries AS summary
    ON summary.id = identity.package_validation_summary_id;
DROP TABLE project_package_validation_candidate_identities;
ALTER TABLE project_package_validation_candidate_identities_v18 RENAME TO project_package_validation_candidate_identities;
CREATE TRIGGER project_package_validation_candidate_identities_immutable
BEFORE UPDATE ON project_package_validation_candidate_identities
BEGIN SELECT RAISE(ABORT, 'package validation candidate identities are immutable'); END;
CREATE UNIQUE INDEX project_package_validation_candidate_identities_unprivileged
    ON project_package_validation_candidate_identities(project_id, candidate_identity_sha256)
    WHERE validation_phase = 'unprivileged';
CREATE INDEX project_package_validation_candidate_identities_lookup
    ON project_package_validation_candidate_identities(project_id, candidate_identity_sha256, validation_phase, attempt_identity_sha256);
CREATE INDEX project_package_validation_candidate_identities_newest_attempt
    ON project_package_validation_candidate_identities(project_id, candidate_identity_sha256, validation_phase, created_at_ms DESC, package_validation_summary_id DESC);
CREATE INDEX project_package_validation_candidate_identities_predecessor
    ON project_package_validation_candidate_identities(package_validation_summary_id);
CREATE INDEX project_package_validation_candidate_identities_summary
    ON project_package_validation_candidate_identities(package_validation_summary_id);
"#;

const LOCAL_REVIEW_ACTIVITY_LEDGER_MIGRATION: &str = r#"
CREATE TABLE local_review_activity_ledger (
    id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
    collection_id TEXT NOT NULL CHECK(length(collection_id) = 36),
    task_id TEXT NOT NULL CHECK(length(task_id) = 36),
    session_id TEXT NOT NULL CHECK(length(session_id) = 36),
    kind TEXT NOT NULL CHECK(kind IN ('item-added', 'activity-evidence-captured')),
    created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0)
);
CREATE INDEX local_review_activity_ledger_collection_session_recent
    ON local_review_activity_ledger(collection_id, task_id, session_id, created_at_ms DESC, id DESC);
CREATE TRIGGER local_review_activity_ledger_immutable
BEFORE UPDATE ON local_review_activity_ledger BEGIN SELECT RAISE(ABORT, 'activity ledger immutable'); END;
CREATE TRIGGER local_review_activity_ledger_append_only
BEFORE DELETE ON local_review_activity_ledger BEGIN SELECT RAISE(ABORT, 'activity ledger append only'); END;
"#;

// Advisor dispatches create a distinct execution conversation. When that
// native conversation later creates a task, retain the immutable origin pair
// needed to distinguish it from other tasks in the same project. Existing
// tasks deliberately remain unbound: no historical origin is inferred.
const TASK_ADVISOR_DISPATCH_ORIGIN_MIGRATION: &str = r#"
ALTER TABLE task_records ADD COLUMN origin_advisor_conversation_id TEXT
    CHECK(origin_advisor_conversation_id IS NULL OR length(origin_advisor_conversation_id) = 36)
    REFERENCES advisor_conversations(id) ON DELETE RESTRICT;
ALTER TABLE task_records ADD COLUMN origin_advisor_dispatch_record_id TEXT
    CHECK(origin_advisor_dispatch_record_id IS NULL OR length(origin_advisor_dispatch_record_id) = 36)
    REFERENCES advisor_dispatch_records(id) ON DELETE RESTRICT;

CREATE INDEX task_records_advisor_dispatch_origin_idx
    ON task_records(origin_advisor_dispatch_record_id)
    WHERE origin_advisor_dispatch_record_id IS NOT NULL;

CREATE TRIGGER task_records_advisor_dispatch_origin_pair_insert
BEFORE INSERT ON task_records
WHEN (NEW.origin_advisor_conversation_id IS NULL) != (NEW.origin_advisor_dispatch_record_id IS NULL)
BEGIN
    SELECT RAISE(ABORT, 'task advisor dispatch origin must be complete');
END;

CREATE TRIGGER task_records_advisor_dispatch_origin_match_insert
BEFORE INSERT ON task_records
WHEN NEW.origin_advisor_dispatch_record_id IS NOT NULL
 AND NOT EXISTS (
    SELECT 1 FROM advisor_dispatch_records
     WHERE id = NEW.origin_advisor_dispatch_record_id
       AND advisor_conversation_id = NEW.origin_advisor_conversation_id
 )
BEGIN
    SELECT RAISE(ABORT, 'task advisor dispatch origin must match dispatch');
END;

CREATE TRIGGER task_records_advisor_dispatch_origin_pair_update
BEFORE UPDATE OF origin_advisor_conversation_id, origin_advisor_dispatch_record_id ON task_records
WHEN (NEW.origin_advisor_conversation_id IS NULL) != (NEW.origin_advisor_dispatch_record_id IS NULL)
BEGIN
    SELECT RAISE(ABORT, 'task advisor dispatch origin must be complete');
END;

CREATE TRIGGER task_records_advisor_dispatch_origin_match_update
BEFORE UPDATE OF origin_advisor_conversation_id, origin_advisor_dispatch_record_id ON task_records
WHEN NEW.origin_advisor_dispatch_record_id IS NOT NULL
 AND NOT EXISTS (
    SELECT 1 FROM advisor_dispatch_records
     WHERE id = NEW.origin_advisor_dispatch_record_id
       AND advisor_conversation_id = NEW.origin_advisor_conversation_id
 )
BEGIN
    SELECT RAISE(ABORT, 'task advisor dispatch origin must match dispatch');
END;

CREATE TRIGGER task_records_advisor_dispatch_origin_immutable
BEFORE UPDATE OF origin_advisor_conversation_id, origin_advisor_dispatch_record_id ON task_records
WHEN NEW.origin_advisor_conversation_id IS NOT OLD.origin_advisor_conversation_id
  OR NEW.origin_advisor_dispatch_record_id IS NOT OLD.origin_advisor_dispatch_record_id
BEGIN
    SELECT RAISE(ABORT, 'task advisor dispatch origin is immutable');
END;
"#;

const LOCAL_TASK_TEMPLATES_MIGRATION: &str = r#"
CREATE TABLE local_task_templates (
    id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
    schema_version INTEGER NOT NULL CHECK(schema_version = 1),
    origin TEXT NOT NULL CHECK(origin = 'local'),
    title TEXT NOT NULL CHECK(length(title) BETWEEN 1 AND 80),
    purpose TEXT NOT NULL CHECK(length(purpose) BETWEEN 1 AND 240),
    instructions TEXT NOT NULL CHECK(length(instructions) <= 32768),
    version INTEGER NOT NULL CHECK(version >= 1),
    sha256 TEXT NOT NULL CHECK(length(sha256) = 64),
    state TEXT NOT NULL CHECK(state IN ('active', 'archived')),
    created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0),
    updated_at_ms INTEGER NOT NULL CHECK(updated_at_ms >= created_at_ms),
    archived_at_ms INTEGER CHECK(archived_at_ms >= created_at_ms),
    CHECK((state = 'active' AND archived_at_ms IS NULL) OR
          (state = 'archived' AND archived_at_ms IS NOT NULL))
);
CREATE INDEX local_task_templates_state_recent
 ON local_task_templates(state, updated_at_ms DESC, id);
CREATE TRIGGER local_task_templates_identity_immutable
BEFORE UPDATE OF id, schema_version, origin, created_at_ms ON local_task_templates
BEGIN SELECT RAISE(ABORT, 'template identity is immutable'); END;
CREATE TRIGGER local_task_templates_version_monotonic
BEFORE UPDATE OF version ON local_task_templates
WHEN NEW.version <= OLD.version
BEGIN SELECT RAISE(ABORT, 'template version must increase'); END;
"#;

const TASK_TEMPLATE_APPLICATION_RESERVATIONS_MIGRATION: &str = r#"
CREATE TABLE task_template_application_reservations (
    id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
    binding_sha256 TEXT NOT NULL CHECK(length(binding_sha256) = 64),
    template_id TEXT NOT NULL CHECK(length(template_id) = 36),
    template_origin TEXT NOT NULL CHECK(template_origin IN ('built-in', 'local')),
    template_version INTEGER NOT NULL CHECK(template_version >= 1),
    template_sha256 TEXT NOT NULL CHECK(length(template_sha256) = 64),
    task_id TEXT NOT NULL REFERENCES task_records(id) ON DELETE RESTRICT,
    plan_id TEXT NOT NULL REFERENCES task_plans(id) ON DELETE RESTRICT,
    task_updated_at_ms INTEGER NOT NULL CHECK(task_updated_at_ms >= 0),
    plan_updated_at_ms INTEGER NOT NULL CHECK(plan_updated_at_ms >= 0),
    state TEXT NOT NULL CHECK(state IN ('pending', 'consumed', 'cancelled', 'expired')),
    created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0),
    expires_at_ms INTEGER NOT NULL CHECK(expires_at_ms > created_at_ms),
    consumed_at_ms INTEGER,
    CHECK((state = 'consumed' AND consumed_at_ms IS NOT NULL) OR
          (state != 'consumed' AND consumed_at_ms IS NULL))
);
CREATE INDEX task_template_application_reservations_pending
 ON task_template_application_reservations(state, expires_at_ms, id);
CREATE TRIGGER task_template_application_reservations_binding_immutable
BEFORE UPDATE OF id, binding_sha256, template_id, template_origin, template_version,
 template_sha256, task_id, plan_id, task_updated_at_ms, plan_updated_at_ms,
 created_at_ms, expires_at_ms ON task_template_application_reservations
BEGIN SELECT RAISE(ABORT, 'template application binding is immutable'); END;
CREATE TRIGGER task_template_application_reservations_state_transition
BEFORE UPDATE OF state ON task_template_application_reservations
WHEN NOT ((OLD.state = 'pending' AND NEW.state IN ('consumed', 'cancelled', 'expired'))
       OR OLD.state = NEW.state)
BEGIN SELECT RAISE(ABORT, 'template application reservation transition is invalid'); END;
"#;

// M55 keeps only a native-controlled opaque locator in SQLite. Canonical
// bytes are private application data, never repository content or provider
// context. A deleted record is a metadata-only tombstone.
const DURABLE_SOURCES_MIGRATION: &str = r#"
CREATE TABLE durable_sources (
 id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
 schema_version INTEGER NOT NULL CHECK(schema_version = 1),
 project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
 task_id TEXT REFERENCES task_records(id) ON DELETE RESTRICT,
 source_class TEXT NOT NULL CHECK(source_class IN ('manual-text', 'local-text-file', 'reviewed-artifact-text')),
 title TEXT NOT NULL CHECK(length(title) BETWEEN 1 AND 240),
 origin_display TEXT CHECK(origin_display IS NULL OR length(origin_display) BETWEEN 1 AND 255),
 byte_size INTEGER NOT NULL CHECK(byte_size BETWEEN 0 AND 131072),
 line_count INTEGER NOT NULL CHECK(line_count BETWEEN 0 AND 2000),
 sha256 TEXT NOT NULL CHECK(length(sha256) = 64),
 content_locator TEXT NOT NULL CHECK(length(content_locator) = 36),
 state TEXT NOT NULL CHECK(state IN ('active', 'deleted')),
 created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0),
 updated_at_ms INTEGER NOT NULL CHECK(updated_at_ms >= created_at_ms),
 deleted_at_ms INTEGER CHECK(deleted_at_ms >= created_at_ms),
 CHECK((state = 'active' AND deleted_at_ms IS NULL) OR (state = 'deleted' AND deleted_at_ms IS NOT NULL))
);
CREATE INDEX durable_sources_project_active ON durable_sources(project_id, state, created_at_ms DESC, id);
CREATE INDEX durable_sources_task_active ON durable_sources(task_id, state, created_at_ms DESC, id) WHERE task_id IS NOT NULL;
CREATE TRIGGER durable_sources_identity_immutable BEFORE UPDATE OF id, schema_version, project_id, task_id, source_class, byte_size, line_count, sha256, content_locator, created_at_ms ON durable_sources BEGIN SELECT RAISE(ABORT, 'durable source identity is immutable'); END;
CREATE TRIGGER durable_sources_lifecycle_transition BEFORE UPDATE OF state ON durable_sources WHEN NOT ((OLD.state = 'active' AND NEW.state = 'deleted') OR OLD.state = NEW.state) BEGIN SELECT RAISE(ABORT, 'durable source lifecycle transition is invalid'); END;
"#;

// M57 persists only fictional/local governance evidence. No connector payload,
// credential, URL, path, provider identifier, or external result can enter
// these tables.
const FICTIONAL_CONNECTOR_GOVERNANCE_MIGRATION: &str = r#"
CREATE TABLE fictional_connector_bindings (
 id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
 schema_version INTEGER NOT NULL CHECK(schema_version = 1),
 project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
 task_id TEXT REFERENCES task_records(id) ON DELETE RESTRICT,
 descriptor_id TEXT NOT NULL CHECK(length(descriptor_id) BETWEEN 1 AND 80),
 descriptor_version INTEGER NOT NULL CHECK(descriptor_version > 0),
 descriptor_sha256 TEXT NOT NULL CHECK(length(descriptor_sha256) = 64),
 scope_digest TEXT NOT NULL CHECK(length(scope_digest) = 64),
 state TEXT NOT NULL CHECK(state IN ('ready','revoked','quarantined','incompatible','expired')),
 created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0),
 updated_at_ms INTEGER NOT NULL CHECK(updated_at_ms >= created_at_ms)
);
CREATE TABLE fictional_connector_operations (
 id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
 schema_version INTEGER NOT NULL CHECK(schema_version = 1),
 binding_id TEXT NOT NULL REFERENCES fictional_connector_bindings(id) ON DELETE RESTRICT,
 project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
 task_id TEXT REFERENCES task_records(id) ON DELETE RESTRICT,
 operation_class TEXT NOT NULL CHECK(operation_class IN ('read','mutation')),
 request_digest TEXT NOT NULL CHECK(length(request_digest) = 64),
 authorization_id TEXT UNIQUE CHECK(authorization_id IS NULL OR length(authorization_id) = 36),
 state TEXT NOT NULL CHECK(state IN ('prepared','cancelled','expired','revoked','rejected','completed','outcome-unknown')),
 created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0),
 expires_at_ms INTEGER NOT NULL CHECK(expires_at_ms >= created_at_ms),
 completed_at_ms INTEGER CHECK(completed_at_ms >= created_at_ms)
);
CREATE TABLE fictional_connector_audit (
 id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
 operation_id TEXT NOT NULL REFERENCES fictional_connector_operations(id) ON DELETE RESTRICT,
 binding_id TEXT NOT NULL REFERENCES fictional_connector_bindings(id) ON DELETE RESTRICT,
 event_kind TEXT NOT NULL CHECK(length(event_kind) BETWEEN 1 AND 64),
 outcome TEXT NOT NULL CHECK(length(outcome) BETWEEN 1 AND 64),
 evidence_digest TEXT NOT NULL CHECK(length(evidence_digest) = 64),
 created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0)
);
CREATE TRIGGER fictional_connector_binding_identity_immutable BEFORE UPDATE OF id, schema_version, project_id, task_id, descriptor_id, descriptor_version, descriptor_sha256, scope_digest, created_at_ms ON fictional_connector_bindings BEGIN SELECT RAISE(ABORT, 'fictional connector binding identity is immutable'); END;
CREATE TRIGGER fictional_connector_operation_identity_immutable BEFORE UPDATE OF id, schema_version, binding_id, project_id, task_id, operation_class, request_digest, authorization_id, created_at_ms, expires_at_ms ON fictional_connector_operations BEGIN SELECT RAISE(ABORT, 'fictional connector operation identity is immutable'); END;
CREATE INDEX fictional_connector_bindings_project_active ON fictional_connector_bindings(project_id, state, updated_at_ms DESC);
CREATE INDEX fictional_connector_operations_binding_recent ON fictional_connector_operations(binding_id, created_at_ms DESC);
"#;

const CONTROLLED_BROWSER_VERIFICATION_MIGRATION: &str = r#"
CREATE TABLE controlled_browser_verification_attempts (
 id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
 schema_version INTEGER NOT NULL CHECK(schema_version = 1),
 project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
 task_id TEXT REFERENCES task_records(id) ON DELETE RESTRICT,
 fixture_id TEXT NOT NULL CHECK(fixture_id = 'fictional-webkitgtk-local-v1'),
 target_digest TEXT NOT NULL CHECK(length(target_digest) = 64),
 request_digest TEXT NOT NULL CHECK(length(request_digest) = 64),
 authorization_id TEXT NOT NULL CHECK(length(authorization_id) = 36),
 state TEXT NOT NULL CHECK(state IN ('prepared','confirmed','verified','verification_failed','cancelled','denied','expired','revoked','redirect_blocked','origin_drift','timed_out','ambiguous','quarantined','incompatible','closed')),
 expires_at_ms INTEGER NOT NULL CHECK(expires_at_ms >= created_at_ms),
 created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0),
 completed_at_ms INTEGER CHECK(completed_at_ms >= created_at_ms),
 evidence_digest TEXT CHECK(evidence_digest IS NULL OR length(evidence_digest) = 64)
);
CREATE TABLE controlled_browser_verification_audit (
 id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
 attempt_id TEXT NOT NULL REFERENCES controlled_browser_verification_attempts(id) ON DELETE RESTRICT,
 event_kind TEXT NOT NULL CHECK(length(event_kind) BETWEEN 1 AND 64),
 outcome TEXT NOT NULL CHECK(length(outcome) BETWEEN 1 AND 64),
 evidence_digest TEXT NOT NULL CHECK(length(evidence_digest) = 64),
 created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0)
);
CREATE TRIGGER controlled_browser_verification_identity_immutable BEFORE UPDATE OF id, schema_version, project_id, task_id, fixture_id, target_digest, request_digest, authorization_id, created_at_ms ON controlled_browser_verification_attempts BEGIN SELECT RAISE(ABORT, 'controlled browser verification identity is immutable'); END;
CREATE INDEX controlled_browser_verification_attempts_project_recent ON controlled_browser_verification_attempts(project_id, created_at_ms DESC);
"#;

const CONTEXT_ASSEMBLY_MIGRATION: &str = r#"
CREATE TABLE context_bundles (
 id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
 schema_version INTEGER NOT NULL CHECK(schema_version = 1),
 project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
 task_id TEXT REFERENCES task_records(id) ON DELETE RESTRICT,
 bundle_digest TEXT NOT NULL CHECK(length(bundle_digest) = 64),
 canonical_bytes BLOB CHECK(canonical_bytes IS NULL OR length(canonical_bytes) <= 98304),
 policy_version INTEGER NOT NULL CHECK(policy_version = 1),
 assembly_version INTEGER NOT NULL CHECK(assembly_version = 1),
 state TEXT NOT NULL CHECK(state IN ('prepared','awaiting_review','awaiting_confirmation','authorized','dispatching','accepted_delivery','rejected_delivery','cancelled','denied','expired','revoked','drifted','timed_out','ambiguous','failed','closed')),
 expires_at_ms INTEGER NOT NULL, created_at_ms INTEGER NOT NULL,
 completed_at_ms INTEGER,
 authorization_id TEXT NOT NULL CHECK(length(authorization_id) = 36),
 CHECK(expires_at_ms >= created_at_ms)
);
CREATE TABLE context_bundle_items (
 id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
 bundle_id TEXT NOT NULL REFERENCES context_bundles(id) ON DELETE RESTRICT,
 ordinal INTEGER NOT NULL CHECK(ordinal >= 0 AND ordinal < 16),
 source_ref TEXT NOT NULL CHECK(length(source_ref) BETWEEN 1 AND 160),
 source_class TEXT NOT NULL CHECK(length(source_class) BETWEEN 1 AND 80),
 provenance TEXT NOT NULL CHECK(length(provenance) BETWEEN 1 AND 120),
 content_digest TEXT NOT NULL CHECK(length(content_digest) = 64),
 byte_size INTEGER NOT NULL CHECK(byte_size >= 0 AND byte_size <= 24576),
 redaction_count INTEGER NOT NULL CHECK(redaction_count >= 0),
 truncated INTEGER NOT NULL CHECK(truncated IN (0,1)),
 UNIQUE(bundle_id, ordinal)
);
CREATE TABLE context_bundle_audit (
 id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
 bundle_id TEXT NOT NULL REFERENCES context_bundles(id) ON DELETE RESTRICT,
 event_kind TEXT NOT NULL CHECK(length(event_kind) BETWEEN 1 AND 64),
 outcome TEXT NOT NULL CHECK(length(outcome) BETWEEN 1 AND 64),
 evidence_digest TEXT NOT NULL CHECK(length(evidence_digest) = 64),
 created_at_ms INTEGER NOT NULL
);
CREATE TRIGGER context_bundle_identity_immutable BEFORE UPDATE OF id, schema_version, project_id, task_id, bundle_digest, policy_version, assembly_version, created_at_ms, authorization_id ON context_bundles BEGIN SELECT RAISE(ABORT, 'context bundle identity is immutable'); END;
CREATE INDEX context_bundles_project_recent ON context_bundles(project_id, created_at_ms DESC);
"#;

// M65 records only a user-confirmed relationship to a transient artifact. It
// deliberately contains no artifact bytes, path, preview, transcript, or
// provider data; expiry of the original artifact is observed separately.
const ARTIFACT_REFERENCES_MIGRATION: &str = r#"
CREATE TABLE artifact_references (
 id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
 schema_version INTEGER NOT NULL CHECK(schema_version = 1),
 project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
 task_id TEXT REFERENCES task_records(id) ON DELETE RESTRICT,
 artifact_id TEXT NOT NULL CHECK(length(artifact_id) = 36),
 artifact_sha256 TEXT NOT NULL CHECK(length(artifact_sha256) = 64),
 artifact_class TEXT NOT NULL CHECK(artifact_class IN ('text','markdown','json','csv','python')),
 display_label TEXT NOT NULL CHECK(length(display_label) BETWEEN 1 AND 120),
 state TEXT NOT NULL CHECK(state IN ('active','deleted')),
 created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0),
 deleted_at_ms INTEGER CHECK(deleted_at_ms >= created_at_ms),
 CHECK((state = 'active' AND deleted_at_ms IS NULL) OR (state = 'deleted' AND deleted_at_ms IS NOT NULL))
);
CREATE INDEX artifact_references_project_active ON artifact_references(project_id, state, created_at_ms DESC, id);
CREATE INDEX artifact_references_task_active ON artifact_references(task_id, state, created_at_ms DESC, id) WHERE task_id IS NOT NULL;
CREATE TRIGGER artifact_references_identity_immutable BEFORE UPDATE OF id, schema_version, project_id, task_id, artifact_id, artifact_sha256, artifact_class, display_label, created_at_ms ON artifact_references BEGIN SELECT RAISE(ABORT, 'artifact reference identity is immutable'); END;
CREATE TRIGGER artifact_references_lifecycle_transition BEFORE UPDATE OF state ON artifact_references WHEN NOT ((OLD.state = 'active' AND NEW.state = 'deleted') OR OLD.state = NEW.state) BEGIN SELECT RAISE(ABORT, 'artifact reference lifecycle transition is invalid'); END;
"#;
const KNOWLEDGE_LEDGER_MIGRATION: &str = r#"
CREATE TABLE knowledge_records (
 id TEXT PRIMARY KEY NOT NULL CHECK(length(id)=36), project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
 task_id TEXT REFERENCES task_records(id) ON DELETE RESTRICT,
 kind TEXT NOT NULL CHECK(kind IN ('owner-decision','constraint','observed-fact','verified-implementation','agent-claim','assumption','recommendation','rejected-approach','unresolved-question')),
 status TEXT NOT NULL CHECK(status IN ('proposed','pending-owner-binding','recorded','active','validated','disproven','resolved','superseded','retired')),
 title TEXT NOT NULL CHECK(length(title) BETWEEN 1 AND 240), body TEXT NOT NULL CHECK(length(body) BETWEEN 1 AND 8192), supersedes_id TEXT REFERENCES knowledge_records(id) ON DELETE RESTRICT,
 created_at_ms INTEGER NOT NULL, updated_at_ms INTEGER NOT NULL);
CREATE TABLE knowledge_record_events (id TEXT PRIMARY KEY NOT NULL, record_id TEXT NOT NULL REFERENCES knowledge_records(id) ON DELETE RESTRICT, event_kind TEXT NOT NULL, created_at_ms INTEGER NOT NULL);
CREATE INDEX knowledge_records_project_recent ON knowledge_records(project_id, updated_at_ms DESC, id);
CREATE TRIGGER knowledge_record_identity_immutable BEFORE UPDATE OF id, project_id, task_id, kind, title, body, supersedes_id, created_at_ms ON knowledge_records BEGIN SELECT RAISE(ABORT, 'knowledge identity immutable'); END;
"#;

const MIGRATIONS: &[(i64, &str, &str)] = &[
    (1, "projects-and-directory-associations", INITIAL_MIGRATION),
    (
        2,
        "conversation-references",
        CONVERSATION_REFERENCES_MIGRATION,
    ),
    (3, "session-lifecycle", SESSION_LIFECYCLE_MIGRATION),
    (4, "worktree-relations", WORKTREE_RELATIONS_MIGRATION),
    (5, "terminal-sessions", TERMINAL_SESSIONS_MIGRATION),
    (6, "model-selection", MODEL_SELECTION_MIGRATION),
    (
        7,
        "unified-conversation-metadata",
        UNIFIED_CONVERSATION_METADATA_MIGRATION,
    ),
    (
        8,
        "advisor-reference-foundation",
        ADVISOR_REFERENCE_FOUNDATION_MIGRATION,
    ),
    (
        9,
        "advisor-approval-controller",
        ADVISOR_APPROVAL_CONTROLLER_MIGRATION,
    ),
    (
        10,
        "advisor-one-time-dispatch",
        ADVISOR_ONE_TIME_DISPATCH_MIGRATION,
    ),
    (
        11,
        "durable-task-records-v1",
        DURABLE_TASK_RECORDS_MIGRATION,
    ),
    (
        12,
        "local-review-collections-v1",
        LOCAL_REVIEW_COLLECTIONS_MIGRATION,
    ),
    (
        13,
        "local-review-evidence-sources-v1",
        LOCAL_REVIEW_EVIDENCE_SOURCES_MIGRATION,
    ),
    (
        14,
        "task-project-binding-v1",
        TASK_PROJECT_BINDING_MIGRATION,
    ),
    (
        15,
        "project-package-validation-summaries-v1",
        PROJECT_PACKAGE_VALIDATION_SUMMARIES_MIGRATION,
    ),
    (
        16,
        "project-package-validation-candidate-identities-v1",
        PROJECT_PACKAGE_VALIDATION_IDENTITIES_MIGRATION,
    ),
    (
        17,
        "project-package-validation-candidate-identities-phased-v1",
        PROJECT_PACKAGE_VALIDATION_PHASED_IDENTITIES_MIGRATION,
    ),
    (
        18,
        "project-package-validation-attempt-identities-v1",
        PROJECT_PACKAGE_VALIDATION_ATTEMPT_IDENTITIES_MIGRATION,
    ),
    (
        19,
        "local-review-activity-ledger-v1",
        LOCAL_REVIEW_ACTIVITY_LEDGER_MIGRATION,
    ),
    (
        20,
        "task-advisor-dispatch-origin-v1",
        TASK_ADVISOR_DISPATCH_ORIGIN_MIGRATION,
    ),
    (
        21,
        "local-task-templates-v1",
        LOCAL_TASK_TEMPLATES_MIGRATION,
    ),
    (
        22,
        "task-template-application-reservations-v1",
        TASK_TEMPLATE_APPLICATION_RESERVATIONS_MIGRATION,
    ),
    (23, "durable-sources-v1", DURABLE_SOURCES_MIGRATION),
    (
        24,
        "fictional-connector-governance-v1",
        FICTIONAL_CONNECTOR_GOVERNANCE_MIGRATION,
    ),
    (
        25,
        "controlled-browser-verification-v1",
        CONTROLLED_BROWSER_VERIFICATION_MIGRATION,
    ),
    (26, "context-assembly-v1", CONTEXT_ASSEMBLY_MIGRATION),
    (27, "artifact-references-v1", ARTIFACT_REFERENCES_MIGRATION),
    (28, "knowledge-ledger-v1", KNOWLEDGE_LEDGER_MIGRATION),
];

#[derive(Debug, Error)]
pub(crate) enum StorageError {
    #[error("metadata database is unavailable")]
    Sqlite(#[from] rusqlite::Error),
    #[error("metadata directory is unavailable")]
    Filesystem,
    #[error("metadata schema is newer than this application")]
    FutureSchema,
    #[error("stored metadata is invalid")]
    InvalidStoredValue,
    #[error("directory is already attached")]
    DuplicateDirectory,
    #[error("project was not found")]
    ProjectNotFound,
    #[error("task capacity reached")]
    TaskCapacity,
    #[error("task was not found")]
    TaskNotFound,
    #[error("task is archived")]
    TaskArchived,
    #[error("plan was not found")]
    PlanNotFound,
    #[error("plan capacity reached")]
    PlanCapacity,
    #[error("task status transition is invalid")]
    InvalidStatusTransition,
    #[error("generated task identifier collided")]
    DuplicateId,
}

fn knowledge_kind_value(value: KnowledgeRecordKind) -> &'static str {
    match value {
        KnowledgeRecordKind::OwnerDecision => "owner-decision",
        KnowledgeRecordKind::Constraint => "constraint",
        KnowledgeRecordKind::ObservedFact => "observed-fact",
        KnowledgeRecordKind::VerifiedImplementation => "verified-implementation",
        KnowledgeRecordKind::AgentClaim => "agent-claim",
        KnowledgeRecordKind::Assumption => "assumption",
        KnowledgeRecordKind::Recommendation => "recommendation",
        KnowledgeRecordKind::RejectedApproach => "rejected-approach",
        KnowledgeRecordKind::UnresolvedQuestion => "unresolved-question",
    }
}
fn knowledge_kind(value: &str) -> Result<KnowledgeRecordKind, rusqlite::Error> {
    serde_json::from_value(serde_json::Value::String(value.into()))
        .map_err(|_| rusqlite::Error::InvalidQuery)
}
fn knowledge_status_value(value: KnowledgeRecordStatus) -> &'static str {
    match value {
        KnowledgeRecordStatus::Proposed => "proposed",
        KnowledgeRecordStatus::PendingOwnerBinding => "pending-owner-binding",
        KnowledgeRecordStatus::Recorded => "recorded",
        KnowledgeRecordStatus::Active => "active",
        KnowledgeRecordStatus::Validated => "validated",
        KnowledgeRecordStatus::Disproven => "disproven",
        KnowledgeRecordStatus::Resolved => "resolved",
        KnowledgeRecordStatus::Superseded => "superseded",
        KnowledgeRecordStatus::Retired => "retired",
    }
}
fn knowledge_status(value: &str) -> Result<KnowledgeRecordStatus, rusqlite::Error> {
    serde_json::from_value(serde_json::Value::String(value.into()))
        .map_err(|_| rusqlite::Error::InvalidQuery)
}

pub(crate) struct DurableSourceInsert<'a> {
    pub id: &'a str,
    pub project_id: &'a str,
    pub task_id: Option<&'a str>,
    pub source_class: DurableSourceClass,
    pub title: &'a str,
    pub origin_display: Option<&'a str>,
    pub byte_size: u64,
    pub line_count: u32,
    pub sha256: &'a str,
}

pub(crate) struct ArtifactReferenceInsert<'a> {
    pub id: &'a str,
    pub project_id: &'a str,
    pub task_id: Option<&'a str>,
    pub artifact_id: &'a str,
    pub artifact_sha256: &'a str,
    pub artifact_class: &'a str,
    pub display_label: &'a str,
}

const PACKAGE_VALIDATION_RECORD_LIMIT: usize = 32;
const PACKAGE_VALIDATION_PROTECTION_MS: i64 = 180 * 24 * 60 * 60 * 1000;
const PACKAGE_ARTIFACT_COUNT_LIMIT: u8 = 2;
pub(crate) const TEMPLATE_APPLICATION_RESERVATION_TTL_MS: i64 = 5 * 60 * 1000;
pub(crate) const TEMPLATE_APPLICATION_PENDING_RESERVATION_LIMIT: i64 = 32;
const INSTALLED_HOST_ATTEMPT_LIMIT: i64 = 8;
const INSTALLED_HOST_ATTEMPT_DOMAIN: &str = "quireforge-installed-host-attempt-v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PackageValidationPhase {
    Unprivileged,
    InstalledHost,
}

fn package_validation_phase_value(phase: PackageValidationPhase) -> &'static str {
    match phase {
        PackageValidationPhase::Unprivileged => "unprivileged",
        PackageValidationPhase::InstalledHost => "installed-host",
    }
}

fn package_validation_phase(value: &str) -> Result<PackageValidationPhase, StorageError> {
    match value {
        "unprivileged" => Ok(PackageValidationPhase::Unprivileged),
        "installed-host" => Ok(PackageValidationPhase::InstalledHost),
        _ => Err(StorageError::InvalidStoredValue),
    }
}

/// Redacted result produced by the native package-validation controller. This
/// is deliberately not deserializable and has no IPC representation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PackageValidationRecordInput {
    pub candidate_identity_sha256: String,
    pub validation_phase: PackageValidationPhase,
    pub attempt_identity_sha256: Option<String>,
    pub installed_host_facts: Option<PackageValidationInstalledHostFacts>,
    pub application_version: String,
    pub debian_version: String,
    pub manifest_state: LocalReviewEvidenceCheckState,
    pub checksum_state: LocalReviewEvidenceCheckState,
    pub abi_state: LocalReviewEvidenceCheckState,
    pub provenance_state: LocalReviewEvidenceCheckState,
    pub visible_launch_state: LocalReviewEvidenceCheckState,
    pub installed_host_state: LocalReviewEvidenceCheckState,
    pub artifact_count: u8,
    pub validation_complete: bool,
    pub supersedes_record_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PackageValidationInstalledHostFacts {
    pub package_state: String,
    pub version_match: bool,
    pub ownership_verified: bool,
    pub permissions_safe: bool,
    pub package_integrity_verified: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PackageValidationSummary {
    pub(crate) id: String,
    pub(crate) project_id: String,
    pub(crate) input: PackageValidationRecordInput,
    pub(crate) created_at_ms: i64,
    pub(crate) record_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PackageValidationRecordOutcome {
    Created(PackageValidationSummary),
    Existing(PackageValidationSummary),
}

fn task_status(value: &str) -> Result<TaskStatus, StorageError> {
    match value {
        "active" => Ok(TaskStatus::Active),
        "paused" => Ok(TaskStatus::Paused),
        "completed" => Ok(TaskStatus::Completed),
        _ => Err(StorageError::InvalidStoredValue),
    }
}

fn durable_source_class_name(value: DurableSourceClass) -> &'static str {
    match value {
        DurableSourceClass::ManualText => "manual-text",
        DurableSourceClass::LocalTextFile => "local-text-file",
        DurableSourceClass::ReviewedArtifactText => "reviewed-artifact-text",
    }
}

fn durable_source_class(value: &str) -> Result<DurableSourceClass, rusqlite::Error> {
    match value {
        "manual-text" => Ok(DurableSourceClass::ManualText),
        "local-text-file" => Ok(DurableSourceClass::LocalTextFile),
        "reviewed-artifact-text" => Ok(DurableSourceClass::ReviewedArtifactText),
        _ => Err(rusqlite::Error::InvalidQuery),
    }
}

fn durable_source_state(value: &str) -> Result<DurableSourceLifecycleState, rusqlite::Error> {
    match value {
        "active" => Ok(DurableSourceLifecycleState::Active),
        "deleted" => Ok(DurableSourceLifecycleState::Deleted),
        _ => Err(rusqlite::Error::InvalidQuery),
    }
}

fn durable_source_from_row(row: &Row<'_>) -> Result<DurableSourceSummary, rusqlite::Error> {
    let class: String = row.get(3)?;
    let state: String = row.get(9)?;
    Ok(DurableSourceSummary {
        source_id: row.get(0)?,
        project_id: row.get(1)?,
        task_id: row.get(2)?,
        source_class: durable_source_class(&class)?,
        title: row.get(4)?,
        origin_display: row.get(5)?,
        byte_size: row.get::<_, i64>(6)? as u64,
        line_count: row.get::<_, i64>(7)? as u32,
        sha256: row.get(8)?,
        state: durable_source_state(&state)?,
        created_at_ms: row.get(10)?,
    })
}

fn artifact_reference_from_row(row: &Row<'_>) -> Result<ArtifactReferenceSummary, rusqlite::Error> {
    let state: String = row.get(7)?;
    Ok(ArtifactReferenceSummary {
        reference_id: row.get(0)?,
        project_id: row.get(1)?,
        task_id: row.get(2)?,
        artifact_id: row.get(3)?,
        artifact_sha256: row.get(4)?,
        artifact_class: row.get(5)?,
        display_label: row.get(6)?,
        state: match state.as_str() {
            "active" => ArtifactReferenceState::Active,
            "deleted" => ArtifactReferenceState::Deleted,
            _ => return Err(rusqlite::Error::InvalidQuery),
        },
        availability: super::types::ArtifactReferenceAvailability::Unavailable,
        created_at_ms: row.get(8)?,
    })
}

pub(super) fn normalize_task_text(
    value: &str,
    char_limit: usize,
    byte_limit: usize,
) -> Result<String, StorageError> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty()
        || normalized.chars().count() > char_limit
        || normalized.len() > byte_limit
        || normalized
            .chars()
            .any(|c| c.is_control() || is_bidirectional_format_control(c))
    {
        return Err(StorageError::InvalidStoredValue);
    }
    Ok(normalized)
}

pub(super) fn validate_plan_body(value: &str) -> Result<(), StorageError> {
    if value.chars().count() > 8_192
        || value.len() > 32 * 1024
        || value.chars().any(|character| {
            (character.is_control() && !matches!(character, '\n' | '\t'))
                || is_bidirectional_format_control(character)
        })
    {
        return Err(StorageError::InvalidStoredValue);
    }
    Ok(())
}

fn is_bidirectional_format_control(character: char) -> bool {
    matches!(
        character,
        '\u{061C}'
            | '\u{200E}'
            | '\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2066}'..='\u{2069}'
    )
}

fn simple_case_fold(value: &str) -> String {
    value
        .case_fold_with(Variant::Simple, Locale::NonTurkic)
        .collect()
}

fn valid_task_id(value: &str) -> bool {
    value.len() == 36
        && value == value.to_ascii_lowercase()
        && Uuid::parse_str(value).is_ok_and(|identifier| identifier.get_version_num() == 7)
}

const TASK_COUNT_LIMIT: i64 = 200;
const TASK_PLAN_LIMIT: i64 = 4;
const TASK_PAYLOAD_LIMIT: i64 = 8 * 1024 * 1024;
const TASK_RECORD_PAYLOAD_LIMIT: i64 = 48 * 1024;
const TASK_CLEANUP_AGE_MS: i64 = 180 * 24 * 60 * 60 * 1_000;
const ID_GENERATION_ATTEMPTS: usize = 4;
const REVIEW_COLLECTION_LIMIT: i64 = 24;
const REVIEW_ACTIVE_COLLECTION_LIMIT: i64 = 12;
const REVIEW_ITEMS_PER_COLLECTION_LIMIT: i64 = 12;
const REVIEW_TEXT_BYTES_LIMIT: usize = 256 * 1024;
const REVIEW_TEXT_PREVIEW_BYTES_LIMIT: usize = 128 * 1024;
const REVIEW_TEXT_PREVIEW_LINES_LIMIT: usize = 2_000;
const REVIEW_TEXT_PREVIEW_CODEPOINT_LIMIT: usize = 32_768;
const REVIEW_IMAGE_BYTES_LIMIT: usize = 1024 * 1024;
const REVIEW_EVIDENCE_BYTES_LIMIT: usize = 16 * 1024;
const REVIEW_EVIDENCE_WARNING_BYTES: usize = 12 * 1024;
const REVIEW_ANNOTATION_BYTES_LIMIT: usize = 1024;
const REVIEW_ANNOTATION_CODEPOINT_LIMIT: usize = 1024;
const REVIEW_ANNOTATIONS_PER_ITEM_LIMIT: i64 = 32;
const REVIEW_ANNOTATIONS_WARNING_COUNT: i64 = 24;
const REVIEW_ANNOTATION_WARNING_BYTES: usize = 768;
const REVIEW_COLLECTION_PAYLOAD_LIMIT: i64 = 4 * 1024 * 1024;
const REVIEW_PAYLOAD_LIMIT: i64 = 32 * 1024 * 1024;

fn review_text_format_value(format: LocalReviewTextFormat) -> &'static str {
    match format {
        LocalReviewTextFormat::Plain => "plain",
        LocalReviewTextFormat::Markdown => "markdown",
        LocalReviewTextFormat::Json => "json",
        LocalReviewTextFormat::Csv => "csv",
        LocalReviewTextFormat::Python => "python",
    }
}

fn review_mime_type(format: LocalReviewTextFormat) -> &'static str {
    match format {
        LocalReviewTextFormat::Plain => "text/plain; charset=utf-8",
        LocalReviewTextFormat::Markdown => "text/markdown; charset=utf-8",
        LocalReviewTextFormat::Json => "application/json",
        LocalReviewTextFormat::Csv => "text/csv; charset=utf-8",
        LocalReviewTextFormat::Python => "text/x-python",
    }
}

fn review_text_format(value: &str) -> Result<LocalReviewTextFormat, StorageError> {
    match value {
        "plain" => Ok(LocalReviewTextFormat::Plain),
        "markdown" => Ok(LocalReviewTextFormat::Markdown),
        "json" => Ok(LocalReviewTextFormat::Json),
        "csv" => Ok(LocalReviewTextFormat::Csv),
        "python" => Ok(LocalReviewTextFormat::Python),
        _ => Err(StorageError::InvalidStoredValue),
    }
}

fn review_collection_state(value: &str) -> Result<LocalReviewCollectionState, StorageError> {
    match value {
        "active" => Ok(LocalReviewCollectionState::Active),
        "frozen" => Ok(LocalReviewCollectionState::Frozen),
        "orphaned" => Ok(LocalReviewCollectionState::Orphaned),
        "unavailable" => Ok(LocalReviewCollectionState::Unavailable),
        "discarded" => Ok(LocalReviewCollectionState::Discarded),
        _ => Err(StorageError::InvalidStoredValue),
    }
}

fn review_source_kind(value: &str) -> Result<LocalReviewSourceKind, StorageError> {
    match value {
        "user-authored-text" => Ok(LocalReviewSourceKind::UserAuthoredText),
        "m48-artifact-copy" => Ok(LocalReviewSourceKind::M48ArtifactCopy),
        "native-image-input" => Ok(LocalReviewSourceKind::NativeImageInput),
        "typed-evidence-snapshot" => Ok(LocalReviewSourceKind::TypedEvidenceSnapshot),
        _ => Err(StorageError::InvalidStoredValue),
    }
}

fn review_evidence_source(value: &str) -> Result<LocalReviewEvidenceSource, StorageError> {
    match value {
        "manual-validation-summary" => Ok(LocalReviewEvidenceSource::ManualValidationSummary),
        "m48-generated-artifact-metadata" => {
            Ok(LocalReviewEvidenceSource::M48GeneratedArtifactMetadata)
        }
        "safe-preview-metadata" => Ok(LocalReviewEvidenceSource::SafePreviewMetadata),
        "git-status-diff-summary" => Ok(LocalReviewEvidenceSource::GitStatusDiffSummary),
        "activity-presentation" => Ok(LocalReviewEvidenceSource::ActivityPresentation),
        "approval-presentation" => Ok(LocalReviewEvidenceSource::ApprovalPresentation),
        "package-manifest-summary" => Ok(LocalReviewEvidenceSource::PackageManifestSummary),
        _ => Err(StorageError::InvalidStoredValue),
    }
}

fn manual_evidence_envelope_bytes(title: &str, summary: &str) -> Result<Vec<u8>, StorageError> {
    serde_json::to_vec(&EvidenceEnvelopeV1 {
        schema_version: 1,
        source: LocalReviewEvidenceSource::ManualValidationSummary,
        source_schema_version: 1,
        title: title.to_owned(),
        summary: summary.to_owned(),
        details: LocalReviewManualValidationDetails {
            validation_state: LocalReviewValidationState::NotRun,
        },
    })
    .map_err(|_| StorageError::InvalidStoredValue)
}

fn m48_metadata_evidence_envelope_bytes(
    title: &str,
    summary: &str,
    details: &LocalReviewM48GeneratedArtifactMetadataDetails,
) -> Result<Vec<u8>, StorageError> {
    serde_json::to_vec(&EvidenceEnvelopeV1 {
        schema_version: 1,
        source: LocalReviewEvidenceSource::M48GeneratedArtifactMetadata,
        source_schema_version: 1,
        title: title.to_owned(),
        summary: summary.to_owned(),
        details,
    })
    .map_err(|_| StorageError::InvalidStoredValue)
}

fn safe_preview_metadata_evidence_envelope_bytes(
    title: &str,
    summary: &str,
    details: &LocalReviewSafePreviewMetadataDetails,
) -> Result<Vec<u8>, StorageError> {
    serde_json::to_vec(&EvidenceEnvelopeV1 {
        schema_version: 1,
        source: LocalReviewEvidenceSource::SafePreviewMetadata,
        source_schema_version: 1,
        title: title.to_owned(),
        summary: summary.to_owned(),
        details,
    })
    .map_err(|_| StorageError::InvalidStoredValue)
}

fn package_manifest_summary_evidence_envelope_bytes(
    title: &str,
    summary: &str,
    details: &LocalReviewPackageManifestSummaryDetails,
) -> Result<Vec<u8>, StorageError> {
    serde_json::to_vec(&EvidenceEnvelopeV1 {
        schema_version: 1,
        source: LocalReviewEvidenceSource::PackageManifestSummary,
        source_schema_version: 1,
        title: title.to_owned(),
        summary: summary.to_owned(),
        details,
    })
    .map_err(|_| StorageError::InvalidStoredValue)
}

fn git_status_diff_summary_evidence_envelope_bytes(
    title: &str,
    summary: &str,
    details: &LocalReviewGitStatusDiffSummaryDetails,
) -> Result<Vec<u8>, StorageError> {
    serde_json::to_vec(&EvidenceEnvelopeV1 {
        schema_version: 1,
        source: LocalReviewEvidenceSource::GitStatusDiffSummary,
        source_schema_version: 1,
        title: title.to_owned(),
        summary: summary.to_owned(),
        details,
    })
    .map_err(|_| StorageError::InvalidStoredValue)
}

fn activity_presentation_evidence_envelope_bytes(
    title: &str,
    summary: &str,
    details: &LocalReviewActivityPresentationDetails,
) -> Result<Vec<u8>, StorageError> {
    serde_json::to_vec(&EvidenceEnvelopeV1 {
        schema_version: 1,
        source: LocalReviewEvidenceSource::ActivityPresentation,
        source_schema_version: 1,
        title: title.to_owned(),
        summary: summary.to_owned(),
        details,
    })
    .map_err(|_| StorageError::InvalidStoredValue)
}

fn parse_activity_presentation_evidence_envelope(
    bytes: &[u8],
    title: &str,
) -> Result<(String, LocalReviewActivityPresentationDetails), StorageError> {
    let envelope: EvidenceEnvelopeV1<LocalReviewActivityPresentationDetails> =
        serde_json::from_slice(bytes).map_err(|_| StorageError::InvalidStoredValue)?;
    if envelope.schema_version != 1
        || envelope.source_schema_version != 1
        || envelope.source != LocalReviewEvidenceSource::ActivityPresentation
        || envelope.title != title
        || normalize_review_label(&envelope.title)? != envelope.title
        || normalize_review_text(&envelope.summary, LocalReviewTextFormat::Plain)?
            != envelope.summary
        || envelope.details.scope != LocalReviewActivityScope::CurrentSession
        || envelope.details.event_count > 12
        || [
            envelope.details.item_added_count,
            envelope.details.item_discarded_count,
            envelope.details.annotation_changed_count,
            envelope.details.comparison_changed_count,
            envelope.details.promotion_prepared_count,
            envelope.details.promotion_completed_count,
            envelope.details.collection_changed_count,
        ]
        .into_iter()
        .any(|count| count > 12)
    {
        return Err(StorageError::InvalidStoredValue);
    }
    Ok((envelope.summary, envelope.details))
}

fn approval_presentation_evidence_envelope_bytes(
    title: &str,
    summary: &str,
    details: &LocalReviewApprovalPresentationDetails,
) -> Result<Vec<u8>, StorageError> {
    serde_json::to_vec(&EvidenceEnvelopeV1 {
        schema_version: 1,
        source: LocalReviewEvidenceSource::ApprovalPresentation,
        source_schema_version: 1,
        title: title.to_owned(),
        summary: summary.to_owned(),
        details,
    })
    .map_err(|_| StorageError::InvalidStoredValue)
}

fn parse_approval_presentation_evidence_envelope(
    bytes: &[u8],
    title: &str,
) -> Result<(String, LocalReviewApprovalPresentationDetails), StorageError> {
    let envelope: EvidenceEnvelopeV1<LocalReviewApprovalPresentationDetails> =
        serde_json::from_slice(bytes).map_err(|_| StorageError::InvalidStoredValue)?;
    if envelope.schema_version != 1
        || envelope.source_schema_version != 1
        || envelope.source != LocalReviewEvidenceSource::ApprovalPresentation
        || envelope.title != title
        || normalize_review_label(&envelope.title)? != envelope.title
        || normalize_review_text(&envelope.summary, LocalReviewTextFormat::Plain)?
            != envelope.summary
        || envelope.details.approval_state != LocalReviewEvidenceApprovalState::Approved
        || !envelope.details.request_present
        || !envelope.details.decision_present
        || !envelope.details.dispatch_present
        || !envelope.details.execution_present
    {
        return Err(StorageError::InvalidStoredValue);
    }
    let canonical = approval_presentation_evidence_envelope_bytes(
        &envelope.title,
        &envelope.summary,
        &envelope.details,
    )?;
    if canonical != bytes {
        return Err(StorageError::InvalidStoredValue);
    }
    Ok((envelope.summary, envelope.details))
}

fn append_local_review_activity(
    tx: &Transaction<'_>,
    collection_id: &str,
    task_id: &str,
    session_id: &str,
    kind: &str,
    now: i64,
) -> Result<(), StorageError> {
    if !matches!(kind, "item-added" | "activity-evidence-captured")
        || !valid_task_id(collection_id)
        || !valid_task_id(task_id)
        || !valid_task_id(session_id)
    {
        return Err(StorageError::InvalidStoredValue);
    }
    tx.execute("INSERT INTO local_review_activity_ledger (id, collection_id, task_id, session_id, kind, created_at_ms) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", params![Uuid::now_v7().to_string(), collection_id, task_id, session_id, kind, now])?;
    Ok(())
}

fn parse_manual_evidence_envelope(bytes: &[u8], title: &str) -> Result<String, StorageError> {
    let envelope: EvidenceEnvelopeV1<LocalReviewManualValidationDetails> =
        serde_json::from_slice(bytes).map_err(|_| StorageError::InvalidStoredValue)?;
    if envelope.schema_version != 1
        || envelope.source_schema_version != 1
        || envelope.source != LocalReviewEvidenceSource::ManualValidationSummary
        || envelope.title != title
        || !matches!(normalize_review_label(&envelope.title), Ok(ref value) if value == &envelope.title)
        || !matches!(normalize_review_text(&envelope.summary, LocalReviewTextFormat::Plain), Ok(ref value) if value == &envelope.summary)
    {
        return Err(StorageError::InvalidStoredValue);
    }
    Ok(envelope.summary)
}

fn parse_m48_metadata_evidence_envelope(
    bytes: &[u8],
    title: &str,
) -> Result<(String, LocalReviewM48GeneratedArtifactMetadataDetails), StorageError> {
    let envelope: EvidenceEnvelopeV1<LocalReviewM48GeneratedArtifactMetadataDetails> =
        serde_json::from_slice(bytes).map_err(|_| StorageError::InvalidStoredValue)?;
    if envelope.schema_version != 1
        || envelope.source != LocalReviewEvidenceSource::M48GeneratedArtifactMetadata
        || envelope.source_schema_version != 1
        || envelope.title != title
        || normalize_review_label(&envelope.title)? != envelope.title
        || normalize_review_text(&envelope.summary, LocalReviewTextFormat::Plain)?
            != envelope.summary
        || !valid_review_sha256(&envelope.details.manifest_sha256)
    {
        return Err(StorageError::InvalidStoredValue);
    }
    Ok((envelope.summary, envelope.details))
}

fn parse_safe_preview_metadata_evidence_envelope(
    bytes: &[u8],
    title: &str,
) -> Result<(String, LocalReviewSafePreviewMetadataDetails), StorageError> {
    let envelope: EvidenceEnvelopeV1<LocalReviewSafePreviewMetadataDetails> =
        serde_json::from_slice(bytes).map_err(|_| StorageError::InvalidStoredValue)?;
    if envelope.schema_version != 1
        || envelope.source != LocalReviewEvidenceSource::SafePreviewMetadata
        || envelope.source_schema_version != 1
        || envelope.title != title
        || normalize_review_label(&envelope.title)? != envelope.title
        || normalize_review_text(&envelope.summary, LocalReviewTextFormat::Plain)?
            != envelope.summary
    {
        return Err(StorageError::InvalidStoredValue);
    }
    Ok((envelope.summary, envelope.details))
}

fn parse_package_manifest_summary_evidence_envelope(
    bytes: &[u8],
    title: &str,
) -> Result<(String, LocalReviewPackageManifestSummaryDetails), StorageError> {
    let envelope: EvidenceEnvelopeV1<LocalReviewPackageManifestSummaryDetails> =
        serde_json::from_slice(bytes).map_err(|_| StorageError::InvalidStoredValue)?;
    if envelope.schema_version != 1
        || envelope.source != LocalReviewEvidenceSource::PackageManifestSummary
        || envelope.source_schema_version != 1
        || envelope.title != title
        || normalize_review_label(&envelope.title)? != envelope.title
        || normalize_review_text(&envelope.summary, LocalReviewTextFormat::Plain)?
            != envelope.summary
        || !valid_package_version(&envelope.details.application_version, false)
        || !valid_package_version(&envelope.details.debian_version, true)
        || envelope.details.artifact_count != u32::from(PACKAGE_ARTIFACT_COUNT_LIMIT)
        || !envelope.details.validation_complete
        || ![
            envelope.details.manifest_state,
            envelope.details.checksum_state,
            envelope.details.abi_state,
            envelope.details.provenance_state,
            envelope.details.visible_launch_state,
            envelope.details.installed_host_state,
        ]
        .into_iter()
        .all(|state| state == LocalReviewEvidenceCheckState::Passed)
    {
        return Err(StorageError::InvalidStoredValue);
    }
    Ok((envelope.summary, envelope.details))
}

fn parse_git_status_diff_summary_evidence_envelope(
    bytes: &[u8],
    title: &str,
) -> Result<(String, LocalReviewGitStatusDiffSummaryDetails), StorageError> {
    let envelope: EvidenceEnvelopeV1<LocalReviewGitStatusDiffSummaryDetails> =
        serde_json::from_slice(bytes).map_err(|_| StorageError::InvalidStoredValue)?;
    if envelope.schema_version != 1
        || envelope.source_schema_version != 1
        || envelope.source != LocalReviewEvidenceSource::GitStatusDiffSummary
        || envelope.title != title
        || normalize_review_label(&envelope.title)? != envelope.title
        || normalize_review_text(&envelope.summary, LocalReviewTextFormat::Plain)?
            != envelope.summary
        || envelope.details.changed_file_count > 512
        || envelope.details.staged_count > 512
        || envelope.details.modified_count > 512
        || envelope.details.added_count > 512
        || envelope.details.deleted_count > 512
        || envelope.details.renamed_count > 512
        || envelope.details.untracked_count > 512
        || envelope.details.conflicted_count > 512
        || (!envelope.details.dirty
            && (envelope.details.changed_file_count != 0
                || envelope.details.workspace_state != LocalReviewEvidenceWorkspaceState::Clean))
    {
        return Err(StorageError::InvalidStoredValue);
    }
    Ok((envelope.summary, envelope.details))
}

fn normalize_review_label(value: &str) -> Result<String, StorageError> {
    let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if value.is_empty()
        || value.chars().count() > 120
        || value.len() > 480
        || value.chars().any(|character| {
            character.is_control()
                || is_bidirectional_format_control(character)
                || matches!(character, '/' | '\\')
        })
    {
        return Err(StorageError::InvalidStoredValue);
    }
    Ok(value)
}

fn normalize_review_text(
    value: &str,
    format: LocalReviewTextFormat,
) -> Result<String, StorageError> {
    let value = value.replace("\r\n", "\n").replace('\r', "\n");
    if value.is_empty()
        || value.len() > REVIEW_TEXT_BYTES_LIMIT
        || value.chars().count() > 32_768
        || value.chars().any(|character| {
            (character.is_control() && !matches!(character, '\n' | '\t'))
                || is_bidirectional_format_control(character)
        })
    {
        return Err(StorageError::InvalidStoredValue);
    }
    if format == LocalReviewTextFormat::Json
        && serde_json::from_str::<serde_json::Value>(&value).is_err()
    {
        return Err(StorageError::InvalidStoredValue);
    }
    if format == LocalReviewTextFormat::Csv {
        let widths: Vec<usize> = value.lines().map(|line| line.split(',').count()).collect();
        if widths.is_empty()
            || widths
                .iter()
                .any(|width| *width == 0 || *width != widths[0])
        {
            return Err(StorageError::InvalidStoredValue);
        }
    }
    Ok(value)
}

fn normalize_review_annotation_text(value: &str) -> Result<String, StorageError> {
    let value = value.replace("\r\n", "\n").replace('\r', "\n");
    if value.is_empty()
        || value.len() > REVIEW_ANNOTATION_BYTES_LIMIT
        || value.chars().count() > REVIEW_ANNOTATION_CODEPOINT_LIMIT
        || value.chars().any(|character| {
            (character.is_control() && !matches!(character, '\n' | '\t'))
                || is_bidirectional_format_control(character)
        })
    {
        return Err(StorageError::InvalidStoredValue);
    }
    Ok(value)
}

fn review_digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn valid_review_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn bounded_review_text_preview(text: &str) -> (String, u64, u16, u16, bool) {
    let mut projected = String::new();
    let mut lines = 0usize;
    let mut code_points = 0usize;
    let mut truncated = false;
    for line in text.split_inclusive('\n') {
        let next_lines = lines + 1;
        let next_code_points = code_points + line.chars().count();
        let next_bytes = projected.len() + line.len();
        if next_lines > REVIEW_TEXT_PREVIEW_LINES_LIMIT
            || next_code_points > REVIEW_TEXT_PREVIEW_CODEPOINT_LIMIT
            || next_bytes > REVIEW_TEXT_PREVIEW_BYTES_LIMIT
        {
            truncated = true;
            break;
        }
        projected.push_str(line);
        lines = next_lines;
        code_points = next_code_points;
    }
    if projected.len() < text.len() {
        truncated = true;
    }
    let projected_byte_size = projected.len() as u64;
    (
        projected,
        projected_byte_size,
        u16::try_from(lines).expect("text preview line limit fits u16"),
        u16::try_from(code_points).expect("text preview code-point limit fits u16"),
        truncated,
    )
}

fn unavailable_review_text_preview(
    collection_id: &str,
    item_id: &str,
    diagnostic_code: LocalReviewDiagnosticCode,
) -> LocalReviewTextPreview {
    LocalReviewTextPreview {
        schema_version: 1,
        collection_id: collection_id.to_owned(),
        item_id: item_id.to_owned(),
        title: None,
        text_format: None,
        byte_size: None,
        sha256: None,
        created_at_ms: None,
        state: LocalReviewItemState::Unavailable,
        text: None,
        projected_byte_size: 0,
        projected_line_count: 0,
        projected_code_point_count: 0,
        truncated: false,
        diagnostic_code: Some(diagnostic_code),
    }
}

fn review_payload_bytes(
    connection: &Connection,
    collection_id: Option<&str>,
) -> Result<i64, StorageError> {
    let where_clause = if collection_id.is_some() {
        "WHERE collection_id = ?1"
    } else {
        ""
    };
    let sql = format!(
        "SELECT
            COALESCE(sum(length(content) + length(CAST(title AS BLOB)) + length(CAST(provenance AS BLOB)) + 256), 0)
            + COALESCE((SELECT sum(length(CAST(body AS BLOB)) + 128)
                FROM local_review_annotations
                JOIN local_review_items ON local_review_items.id = local_review_annotations.item_id
                {where_clause}), 0)
         FROM local_review_items {where_clause}"
    );
    if let Some(collection_id) = collection_id {
        Ok(connection.query_row(&sql, [collection_id], |row| row.get(0))?)
    } else {
        Ok(connection.query_row(&sql, [], |row| row.get(0))?)
    }
}

const REVIEW_COMPARISON_BYTES_LIMIT: usize = 128 * 1024;
const REVIEW_COMPARISON_LINES_LIMIT: usize = 2_000;
const REVIEW_COMPARISONS_PER_COLLECTION_LIMIT: i64 = 8;
const REVIEW_COMPARISONS_WARNING_COUNT: i64 = 6;

fn comparison_lines(left: &str, right: &str) -> Vec<LocalReviewLineRecord> {
    let left: Vec<&str> = left.lines().collect();
    let right: Vec<&str> = right.lines().collect();
    let mut table = vec![vec![0usize; right.len() + 1]; left.len() + 1];
    for left_index in (0..left.len()).rev() {
        for right_index in (0..right.len()).rev() {
            table[left_index][right_index] = if left[left_index] == right[right_index] {
                table[left_index + 1][right_index + 1] + 1
            } else {
                table[left_index + 1][right_index].max(table[left_index][right_index + 1])
            };
        }
    }
    let (mut left_index, mut right_index) = (0, 0);
    let mut records = Vec::new();
    while left_index < left.len() || right_index < right.len() {
        if left_index < left.len()
            && right_index < right.len()
            && left[left_index] == right[right_index]
        {
            records.push(LocalReviewLineRecord {
                kind: LocalReviewLineKind::Unchanged,
                text: left[left_index].to_owned(),
                left_line_number: Some((left_index + 1) as u32),
                right_line_number: Some((right_index + 1) as u32),
            });
            left_index += 1;
            right_index += 1;
        } else if right_index < right.len()
            && (left_index == left.len()
                || table[left_index][right_index + 1] >= table[left_index + 1][right_index])
        {
            records.push(LocalReviewLineRecord {
                kind: LocalReviewLineKind::Added,
                text: right[right_index].to_owned(),
                left_line_number: None,
                right_line_number: Some((right_index + 1) as u32),
            });
            right_index += 1;
        } else {
            records.push(LocalReviewLineRecord {
                kind: LocalReviewLineKind::Removed,
                text: left[left_index].to_owned(),
                left_line_number: Some((left_index + 1) as u32),
                right_line_number: None,
            });
            left_index += 1;
        }
    }
    records
}

/// The only lifecycle gate for Local Review mutations.  Content-changing
/// operations must revalidate the current task and immutable plan binding in
/// the same immediate transaction as their write.  Explicit discard is the
/// intentional recovery exception: copied review data remains disposable even
/// when its task or plan is no longer available.
#[derive(Clone, Copy)]
enum LocalReviewMutationPermission {
    ActiveContent,
    RecoveryDiscard,
    ExplicitResume,
}

struct LocalReviewMutationContext {
    task_id: String,
    updated_at_ms: i64,
}

#[expect(
    clippy::type_complexity,
    reason = "The row tuple mirrors the fixed local_review_collections query and is immediately destructured."
)]
fn local_review_mutation_context(
    tx: &Transaction<'_>,
    collection_id: &str,
    expected_updated_at_ms: Option<i64>,
    permission: LocalReviewMutationPermission,
) -> Result<LocalReviewMutationContext, StorageError> {
    if !valid_task_id(collection_id)
        || expected_updated_at_ms.is_some_and(|updated_at_ms| updated_at_ms < 0)
    {
        return Err(StorageError::InvalidStoredValue);
    }
    let collection: Option<(String, String, i64, Option<String>, Option<i64>)> = tx
        .query_row(
            "SELECT task_id, state, updated_at_ms, plan_id, observed_plan_updated_at_ms
             FROM local_review_collections WHERE id = ?1 AND discarded_at_ms IS NULL",
            [collection_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .optional()?;
    let Some((task_id, state, updated_at_ms, plan_id, observed_plan_updated_at_ms)) = collection
    else {
        return Err(StorageError::TaskNotFound);
    };
    if expected_updated_at_ms.is_some_and(|expected| expected != updated_at_ms) {
        return Err(StorageError::InvalidStatusTransition);
    }
    if matches!(permission, LocalReviewMutationPermission::RecoveryDiscard) {
        return Ok(LocalReviewMutationContext {
            task_id,
            updated_at_ms,
        });
    }
    if matches!(permission, LocalReviewMutationPermission::ActiveContent) && state != "active" {
        return Err(StorageError::InvalidStatusTransition);
    }
    let task: Option<(String, Option<i64>)> = tx
        .query_row(
            "SELECT status, archived_at_ms FROM task_records WHERE id = ?1",
            [&task_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let Some((status, archived_at_ms)) = task else {
        return Err(StorageError::TaskNotFound);
    };
    if archived_at_ms.is_some() || !matches!(status.as_str(), "active" | "paused") {
        return Err(StorageError::TaskArchived);
    }
    if let (Some(plan_id), Some(observed)) = (plan_id.as_deref(), observed_plan_updated_at_ms) {
        let current: Option<i64> = tx
            .query_row(
                "SELECT updated_at_ms FROM task_plans WHERE id = ?1 AND task_id = ?2",
                params![plan_id, task_id],
                |row| row.get(0),
            )
            .optional()?;
        if current != Some(observed) {
            return Err(StorageError::PlanNotFound);
        }
    } else if plan_id.is_some() {
        return Err(StorageError::InvalidStoredValue);
    }
    Ok(LocalReviewMutationContext {
        task_id,
        updated_at_ms,
    })
}

fn annotation_mutation_context(
    tx: &Transaction<'_>,
    collection_id: &str,
    item_id: &str,
    annotation_id: &str,
    expected_updated_at_ms: i64,
    permission: LocalReviewMutationPermission,
) -> Result<(String, i64, String, String, i64, i64), StorageError> {
    if !valid_task_id(collection_id)
        || !valid_task_id(item_id)
        || !valid_task_id(annotation_id)
        || expected_updated_at_ms < 0
    {
        return Err(StorageError::InvalidStoredValue);
    }
    let collection =
        local_review_mutation_context(tx, collection_id, Some(expected_updated_at_ms), permission)?;
    let item_state: Option<String> = tx
        .query_row(
            "SELECT state FROM local_review_items
             WHERE id = ?1 AND collection_id = ?2 AND discarded_at_ms IS NULL",
            params![item_id, collection_id],
            |row| row.get(0),
        )
        .optional()?;
    if !matches!(permission, LocalReviewMutationPermission::RecoveryDiscard)
        && item_state.as_deref() != Some("ready")
    {
        return Err(StorageError::TaskNotFound);
    }
    if item_state.is_none() {
        return Err(StorageError::TaskNotFound);
    }
    let annotation: Option<(String, String, i64, i64)> = tx
        .query_row(
            "SELECT state, body, created_at_ms, updated_at_ms
             FROM local_review_annotations WHERE id = ?1 AND item_id = ?2",
            params![annotation_id, item_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()?;
    let Some((state, body, created_at_ms, updated_at_ms)) = annotation else {
        return Err(StorageError::TaskNotFound);
    };
    if !matches!(state.as_str(), "open" | "resolved")
        || created_at_ms < 0
        || updated_at_ms < created_at_ms
        || !matches!(normalize_review_annotation_text(&body), Ok(ref normalized) if normalized == &body)
    {
        return Err(StorageError::InvalidStoredValue);
    }
    Ok((
        state,
        collection.updated_at_ms,
        body,
        collection.task_id,
        created_at_ms,
        updated_at_ms,
    ))
}

fn task_payload_bytes(connection: &Connection, task_id: Option<&str>) -> Result<i64, StorageError> {
    let value = if let Some(task_id) = task_id {
        connection.query_row(
            "SELECT COALESCE(
                length(CAST(id AS BLOB)) + length(CAST(title AS BLOB))
                + length(CAST(status AS BLOB)) + length(CAST(selected_plan_id AS BLOB)) + 64
                + (SELECT COALESCE(sum(
                    length(CAST(id AS BLOB)) + length(CAST(task_id AS BLOB))
                    + length(CAST(label AS BLOB)) + length(CAST(body AS BLOB)) + 48
                  ), 0)
                  FROM task_plans WHERE task_id = task_records.id),
                0)
             FROM task_records WHERE id = ?1",
            [task_id],
            |row| row.get(0),
        )?
    } else {
        connection.query_row(
            "SELECT COALESCE(sum(
                length(CAST(id AS BLOB)) + length(CAST(title AS BLOB))
                + length(CAST(status AS BLOB)) + length(CAST(selected_plan_id AS BLOB)) + 64
                + (SELECT COALESCE(sum(
                    length(CAST(id AS BLOB)) + length(CAST(task_id AS BLOB))
                    + length(CAST(label AS BLOB)) + length(CAST(body AS BLOB)) + 48
                  ), 0)
                  FROM task_plans WHERE task_id = task_records.id)
              ), 0)
             FROM task_records",
            [],
            |row| row.get(0),
        )?
    };
    Ok(value)
}

fn unique_task_id(
    transaction: &Transaction<'_>,
    generate: &mut impl FnMut() -> String,
    reserved: &mut HashSet<String>,
) -> Result<String, StorageError> {
    for _ in 0..ID_GENERATION_ATTEMPTS {
        let candidate = generate();
        if !valid_task_id(&candidate) || reserved.contains(&candidate) {
            continue;
        }
        let exists: bool = transaction.query_row(
            "SELECT EXISTS(
                SELECT 1 FROM task_records WHERE id = ?1
                UNION ALL
                SELECT 1 FROM task_plans WHERE id = ?1
             )",
            [&candidate],
            |row| row.get(0),
        )?;
        if !exists {
            reserved.insert(candidate.clone());
            return Ok(candidate);
        }
    }
    Err(StorageError::DuplicateId)
}

fn task_status_value(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Active => "active",
        TaskStatus::Paused => "paused",
        TaskStatus::Completed => "completed",
    }
}

fn ensure_editable_task(
    transaction: &Transaction<'_>,
    task_id: &str,
) -> Result<String, StorageError> {
    let state: Option<(String, Option<i64>)> = transaction
        .query_row(
            "SELECT status, archived_at_ms FROM task_records WHERE id = ?1",
            [task_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    match state {
        None => Err(StorageError::TaskNotFound),
        Some((_, Some(_))) => Err(StorageError::TaskArchived),
        Some((status, None)) => Ok(status),
    }
}

fn ensure_task_capacity(transaction: &Transaction<'_>, task_id: &str) -> Result<(), StorageError> {
    if task_payload_bytes(transaction, Some(task_id))? > TASK_RECORD_PAYLOAD_LIMIT
        || task_payload_bytes(transaction, None)? > TASK_PAYLOAD_LIMIT
    {
        return Err(StorageError::TaskCapacity);
    }
    Ok(())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn task_template_application_context_in_connection(
    connection: &Connection,
    task_id: &str,
    plan_id: &str,
) -> Result<TaskTemplateApplicationContext, StorageError> {
    if !valid_task_id(task_id) || !valid_task_id(plan_id) {
        return Err(StorageError::InvalidStoredValue);
    }
    let (status, archived_at_ms, task_updated_at_ms): (String, Option<i64>, i64) = connection
        .query_row(
            "SELECT status, archived_at_ms, updated_at_ms FROM task_records WHERE id = ?1",
            [task_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?
        .ok_or(StorageError::TaskNotFound)?;
    if archived_at_ms.is_some()
        || !matches!(status.as_str(), "active" | "paused")
        || task_updated_at_ms < 0
    {
        return Err(StorageError::TaskArchived);
    }
    let plan_updated_at_ms: i64 = connection
        .query_row(
            "SELECT updated_at_ms FROM task_plans WHERE id = ?1 AND task_id = ?2",
            params![plan_id, task_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or(StorageError::PlanNotFound)?;
    if plan_updated_at_ms < 0 {
        return Err(StorageError::InvalidStoredValue);
    }
    Ok(TaskTemplateApplicationContext {
        task_id: task_id.to_owned(),
        plan_id: plan_id.to_owned(),
        task_updated_at_ms,
        plan_updated_at_ms,
    })
}

fn task_template_application_context_in_transaction(
    transaction: &Transaction<'_>,
    task_id: &str,
    plan_id: &str,
) -> Result<TaskTemplateApplicationContext, StorageError> {
    if !valid_task_id(task_id) || !valid_task_id(plan_id) {
        return Err(StorageError::InvalidStoredValue);
    }
    let (status, archived_at_ms, task_updated_at_ms): (String, Option<i64>, i64) = transaction
        .query_row(
            "SELECT status, archived_at_ms, updated_at_ms FROM task_records WHERE id = ?1",
            [task_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?
        .ok_or(StorageError::TaskNotFound)?;
    if archived_at_ms.is_some()
        || !matches!(status.as_str(), "active" | "paused")
        || task_updated_at_ms < 0
    {
        return Err(StorageError::TaskArchived);
    }
    let plan_updated_at_ms: i64 = transaction
        .query_row(
            "SELECT updated_at_ms FROM task_plans WHERE id = ?1 AND task_id = ?2",
            params![plan_id, task_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or(StorageError::PlanNotFound)?;
    if plan_updated_at_ms < 0 {
        return Err(StorageError::InvalidStoredValue);
    }
    Ok(TaskTemplateApplicationContext {
        task_id: task_id.to_owned(),
        plan_id: plan_id.to_owned(),
        task_updated_at_ms,
        plan_updated_at_ms,
    })
}

#[derive(Clone, Debug)]
pub(crate) struct StoredProject {
    pub id: String,
    pub display_name: String,
    pub archived: bool,
    pub association: Option<StoredAssociation>,
}

#[derive(Clone, Debug)]
pub(crate) struct StoredAssociation {
    pub id: String,
    pub selected_path: String,
    pub resolved_path: String,
    pub expected_access: ExpectedAccess,
    pub device_id: Option<u64>,
    pub inode: Option<u64>,
    pub filesystem_type: Option<String>,
    pub mount_id: Option<u64>,
    pub git_common_dir: Option<String>,
    pub git_worktree_root: Option<String>,
    pub git_is_linked_worktree: bool,
    pub has_agents_guidance: bool,
    pub has_codex_config: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StoredTerminalSession {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub status: String,
    pub columns: u16,
    pub rows: u16,
    pub exit_code: Option<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StoredConversationReference {
    pub id: String,
    pub project_id: String,
    pub codex_thread_id: String,
    pub active_turn_id: Option<String>,
    pub model_id: String,
    pub reasoning_effort: String,
    pub sandbox_mode: String,
    pub approval_policy: String,
    pub status: String,
    pub parent_conversation_id: Option<String>,
    pub archived: bool,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub selector_mode: String,
    pub selector_availability: String,
    pub selector_user_locked: bool,
    pub selector_allowed_model_ids_json: String,
    pub selector_reasoning_ceiling: Option<String>,
    pub selector_pending_model_id: Option<String>,
    pub selector_pending_reasoning_effort: Option<String>,
    pub selector_pending_rationale: Option<String>,
    pub selector_pending_provenance: Option<String>,
    pub selector_pending_application: Option<String>,
    pub selector_pending_requested_at_ms: Option<i64>,
}

#[derive(Clone, Debug)]
pub(crate) struct StoredWorktreeRelation {
    pub source_project_id: String,
    pub worktree_project_id: String,
    pub ownership: String,
    pub branch_name: Option<String>,
}

type TaskCatalogProjection = (
    Vec<TaskRecordSummary>,
    Option<TaskRecordSummary>,
    Vec<TaskPlanSummary>,
    u16,
    u64,
    bool,
);

pub(crate) struct ProjectRepository {
    pub(super) connection: Connection,
    activity_session_id: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StoredLocalTaskTemplate {
    pub template: TaskTemplate,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub archived_at_ms: Option<i64>,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LocalTemplateCapacity {
    pub record_count: usize,
    pub canonical_bytes: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TaskTemplateApplicationContext {
    pub task_id: String,
    pub plan_id: String,
    pub task_updated_at_ms: i64,
    pub plan_updated_at_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TaskTemplateApplicationReservation {
    pub id: String,
    pub binding_sha256: String,
    pub template_id: String,
    pub template_origin: String,
    pub template_version: u32,
    pub template_sha256: String,
    pub context: TaskTemplateApplicationContext,
    pub created_at_ms: i64,
    pub expires_at_ms: i64,
}

#[allow(dead_code)]
type LocalTemplateRow = (
    String,
    i64,
    String,
    String,
    String,
    String,
    i64,
    String,
    String,
    i64,
    i64,
    Option<i64>,
);

#[allow(dead_code)]
fn local_template_row(row: &Row<'_>) -> rusqlite::Result<LocalTemplateRow> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
        row.get(7)?,
        row.get(8)?,
        row.get(9)?,
        row.get(10)?,
        row.get(11)?,
    ))
}

#[allow(dead_code)]
fn validate_local_template_id(id: &str) -> Result<(), StorageError> {
    if Uuid::parse_str(id).is_ok_and(|value| value.get_version_num() == 7)
        && !builtins().iter().any(|builtin| builtin.id == id)
    {
        Ok(())
    } else {
        Err(StorageError::InvalidStoredValue)
    }
}

#[allow(dead_code)]
fn validate_local_template(template: &TaskTemplate) -> Result<(), StorageError> {
    if template.origin != TemplateOrigin::Local || !valid_template(template) {
        return Err(StorageError::InvalidStoredValue);
    }
    validate_local_template_id(&template.id)
}

#[allow(dead_code)]
fn template_state_value(state: TemplateState) -> &'static str {
    match state {
        TemplateState::Active => "active",
        TemplateState::Archived => "archived",
    }
}

#[allow(dead_code)]
fn stored_local_template_from_row(
    raw: LocalTemplateRow,
) -> Result<StoredLocalTaskTemplate, StorageError> {
    let (
        id,
        schema_version,
        origin,
        title,
        purpose,
        instructions,
        version,
        sha256,
        state,
        created_at_ms,
        updated_at_ms,
        archived_at_ms,
    ) = raw;
    if schema_version != i64::from(TEMPLATE_SCHEMA_VERSION)
        || origin != "local"
        || created_at_ms < 0
        || updated_at_ms < created_at_ms
    {
        return Err(StorageError::InvalidStoredValue);
    }
    let state = match state.as_str() {
        "active" if archived_at_ms.is_none() => TemplateState::Active,
        "archived"
            if archived_at_ms.is_some_and(|archived_at_ms| {
                archived_at_ms >= created_at_ms && archived_at_ms <= updated_at_ms
            }) =>
        {
            TemplateState::Archived
        }
        _ => return Err(StorageError::InvalidStoredValue),
    };
    let version = u32::try_from(version).map_err(|_| StorageError::InvalidStoredValue)?;
    let template = TaskTemplate {
        id,
        origin: TemplateOrigin::Local,
        title,
        purpose,
        instructions,
        version,
        state,
        sha256,
    };
    validate_local_template(&template)?;
    Ok(StoredLocalTaskTemplate {
        template,
        created_at_ms,
        updated_at_ms,
        archived_at_ms,
    })
}

fn task_template_for_application(
    transaction: &Transaction<'_>,
    id: &str,
    origin: &str,
) -> Result<TaskTemplate, StorageError> {
    match origin {
        "built-in" => builtins().into_iter().find(|template| template.id == id).ok_or(StorageError::InvalidStoredValue),
        "local" => transaction.query_row(
            "SELECT id, schema_version, origin, title, purpose, instructions, version, sha256, state, created_at_ms, updated_at_ms, archived_at_ms FROM local_task_templates WHERE id=?1",
            [id], local_template_row,
        ).optional()?.map(stored_local_template_from_row).transpose()?.map(|record| record.template).ok_or(StorageError::InvalidStoredValue),
        _ => Err(StorageError::InvalidStoredValue),
    }
}

pub(super) fn task_template_application_binding_digest(
    template: &TaskTemplate,
    context: &TaskTemplateApplicationContext,
    title: &str,
    plan_text: &str,
) -> Option<String> {
    if template.state != TemplateState::Active
        || !valid_sha256(&template.sha256)
        || title.is_empty()
        || plan_text.len() > 32 * 1024
        || context.task_updated_at_ms < 0
        || context.plan_updated_at_ms < 0
    {
        return None;
    }
    let origin = match template.origin {
        TemplateOrigin::BuiltIn => "built-in",
        TemplateOrigin::Local => "local",
    };
    Some(format!(
        "{:x}",
        Sha256::digest(
            format!(
                "task-template-application-v1\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
                template.id,
                origin,
                template.version,
                template.sha256,
                context.task_id,
                context.plan_id,
                context.task_updated_at_ms,
                context.plan_updated_at_ms,
                title,
                plan_text
            )
            .as_bytes()
        )
    ))
}

#[allow(dead_code)]
fn insert_local_template_row(
    transaction: &Transaction<'_>,
    template: &TaskTemplate,
    created_at_ms: i64,
    updated_at_ms: i64,
    archived_at_ms: Option<i64>,
) -> Result<(), StorageError> {
    transaction.execute(
        "INSERT INTO local_task_templates (
             id, schema_version, origin, title, purpose, instructions, version, sha256, state,
             created_at_ms, updated_at_ms, archived_at_ms
         ) VALUES (?1, ?2, 'local', ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            template.id,
            i64::from(TEMPLATE_SCHEMA_VERSION),
            template.title,
            template.purpose,
            template.instructions,
            template.version,
            template.sha256,
            template_state_value(template.state),
            created_at_ms,
            updated_at_ms,
            archived_at_ms,
        ],
    )?;
    Ok(())
}

fn ensure_local_template_capacity(
    transaction: &Transaction<'_>,
    replacing_id: Option<&str>,
    incoming: Option<&TaskTemplate>,
) -> Result<(), StorageError> {
    let mut count = builtins().len();
    let mut canonical_bytes: usize = builtins()
        .iter()
        .map(|template| {
            canonical_template(template)
                .expect("builtins are canonical")
                .len()
        })
        .sum();
    let mut statement = transaction.prepare(
        "SELECT id, schema_version, origin, title, purpose, instructions, version, sha256,
                state, created_at_ms, updated_at_ms, archived_at_ms FROM local_task_templates",
    )?;
    let rows = statement
        .query_map([], local_template_row)?
        .collect::<Result<Vec<_>, _>>()?;
    for row in rows {
        let record = stored_local_template_from_row(row)?;
        if Some(record.template.id.as_str()) != replacing_id {
            count += 1;
            canonical_bytes += canonical_template(&record.template)
                .ok_or(StorageError::InvalidStoredValue)?
                .len();
        }
    }
    if let Some(template) = incoming {
        count += 1;
        canonical_bytes += canonical_template(template)
            .ok_or(StorageError::InvalidStoredValue)?
            .len();
    }
    if count > TEMPLATE_COUNT_LIMIT || canonical_bytes > TEMPLATE_PAYLOAD_LIMIT {
        return Err(StorageError::TaskCapacity);
    }
    Ok(())
}

pub(crate) struct LocalReviewPromotionSource {
    pub collection_id: String,
    pub item_id: String,
    pub task_id: String,
    pub plan_id: Option<String>,
    pub observed_plan_updated_at_ms: Option<i64>,
    pub title: String,
    pub text_format: LocalReviewTextFormat,
    pub sha256: String,
    pub content: String,
}

impl ProjectRepository {
    pub(crate) fn knowledge_records_for_record(&self, id: &str) -> Result<String, StorageError> {
        self.connection
            .query_row(
                "SELECT project_id FROM knowledge_records WHERE id=?1",
                [id],
                |r| r.get(0),
            )
            .map_err(StorageError::from)
    }
    pub(crate) fn knowledge_records(
        &self,
        project_id: &str,
    ) -> Result<Vec<KnowledgeRecordSummary>, StorageError> {
        let mut statement = self.connection.prepare("SELECT id,project_id,task_id,kind,status,title,body,supersedes_id,created_at_ms,updated_at_ms FROM knowledge_records WHERE project_id=?1 ORDER BY updated_at_ms DESC,id DESC LIMIT 128")?;
        let records = statement
            .query_map([project_id], |row| {
                Ok(KnowledgeRecordSummary {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    task_id: row.get(2)?,
                    kind: knowledge_kind(&row.get::<_, String>(3)?)?,
                    status: knowledge_status(&row.get::<_, String>(4)?)?,
                    title: row.get(5)?,
                    body: row.get(6)?,
                    supersedes_id: row.get(7)?,
                    created_at_ms: row.get(8)?,
                    updated_at_ms: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(StorageError::from);
        records
    }
    pub(crate) fn create_knowledge_record(
        &mut self,
        project_id: &str,
        task_id: Option<&str>,
        kind: KnowledgeRecordKind,
        title: &str,
        body: &str,
        supersedes_id: Option<&str>,
    ) -> Result<String, StorageError> {
        if Uuid::parse_str(project_id).is_err()
            || task_id.is_some_and(|id| Uuid::parse_str(id).is_err())
            || supersedes_id.is_some_and(|id| Uuid::parse_str(id).is_err())
            || title.trim().is_empty()
            || title.len() > 240
            || body.trim().is_empty()
            || body.len() > 8192
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let id = Uuid::now_v7().to_string();
        let now = now_millis();
        let status = match kind {
            KnowledgeRecordKind::OwnerDecision | KnowledgeRecordKind::Constraint => "proposed",
            _ => "recorded",
        };
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(predecessor) = supersedes_id {
            let predecessor_project: String = tx.query_row(
                "SELECT project_id FROM knowledge_records WHERE id=?1",
                [predecessor],
                |row| row.get(0),
            )?;
            if predecessor_project != project_id {
                return Err(StorageError::InvalidStoredValue);
            }
        }
        tx.execute("INSERT INTO knowledge_records(id,project_id,task_id,kind,status,title,body,supersedes_id,created_at_ms,updated_at_ms) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?9)", params![id,project_id,task_id,knowledge_kind_value(kind),status,title.trim(),body.trim(),supersedes_id,now])?;
        tx.execute("INSERT INTO knowledge_record_events(id,record_id,event_kind,created_at_ms) VALUES(?1,?2,'created',?3)", params![Uuid::now_v7().to_string(),id,now])?;
        tx.commit()?;
        Ok(id)
    }
    pub(crate) fn bind_knowledge_record(&mut self, id: &str) -> Result<(), StorageError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (kind, status): (String, String) = tx.query_row(
            "SELECT kind,status FROM knowledge_records WHERE id=?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        if !matches!(kind.as_str(), "owner-decision" | "constraint")
            || status != "pending-owner-binding"
        {
            return Err(StorageError::InvalidStatusTransition);
        }
        let now = now_millis();
        tx.execute(
            "UPDATE knowledge_records SET status='active',updated_at_ms=?2 WHERE id=?1",
            params![id, now],
        )?;
        tx.execute("INSERT INTO knowledge_record_events(id,record_id,event_kind,created_at_ms) VALUES(?1,?2,'owner-bound',?3)",params![Uuid::now_v7().to_string(),id,now])?;
        Ok(tx.commit()?)
    }
    pub(crate) fn transition_knowledge_record(
        &mut self,
        id: &str,
        next: KnowledgeRecordStatus,
    ) -> Result<(), StorageError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (kind, current): (String, String) = tx.query_row(
            "SELECT kind,status FROM knowledge_records WHERE id=?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        let next = knowledge_status_value(next);
        let binding = matches!(kind.as_str(), "owner-decision" | "constraint");
        let allowed = if binding {
            matches!(
                (current.as_str(), next),
                ("proposed", "pending-owner-binding") | ("active", "superseded" | "retired")
            )
        } else {
            matches!(
                (current.as_str(), next),
                ("recorded", "active")
                    | (
                        "active",
                        "validated" | "disproven" | "resolved" | "superseded" | "retired"
                    )
                    | (
                        "validated" | "disproven" | "resolved",
                        "superseded" | "retired"
                    )
            )
        };
        if !allowed {
            return Err(StorageError::InvalidStatusTransition);
        }
        let now = now_millis();
        tx.execute(
            "UPDATE knowledge_records SET status=?2,updated_at_ms=?3 WHERE id=?1",
            params![id, next, now],
        )?;
        tx.execute(
            "INSERT INTO knowledge_record_events(id,record_id,event_kind,created_at_ms) VALUES(?1,?2,?3,?4)",
            params![Uuid::now_v7().to_string(), id, format!("status-{next}"), now],
        )?;
        Ok(tx.commit()?)
    }
    pub(crate) fn context_ledger(
        &self,
        project_id: &str,
    ) -> Result<Vec<ContextLedgerEntry>, StorageError> {
        let mut statement = self.connection.prepare(
            r#"SELECT * FROM (
                SELECT 'context-bundle', b.id, b.project_id, b.task_id, b.state, b.bundle_digest,
                    (SELECT count(*) FROM context_bundle_items i WHERE i.bundle_id = b.id),
                    b.expires_at_ms, b.created_at_ms, b.completed_at_ms,
                    COALESCE((SELECT a.outcome FROM context_bundle_audit a
                      WHERE a.bundle_id = b.id ORDER BY a.created_at_ms DESC, a.id DESC LIMIT 1), 'none')
                FROM context_bundles b WHERE b.project_id = ?1
                UNION ALL SELECT 'durable-source', d.id, d.project_id, d.task_id, d.state, d.sha256,
                    0, 0, d.created_at_ms, d.deleted_at_ms, d.state
                FROM durable_sources d WHERE d.project_id = ?1
                UNION ALL SELECT 'artifact-reference', r.id, r.project_id, r.task_id, r.state, r.artifact_sha256,
                    0, 0, r.created_at_ms, r.deleted_at_ms, r.state
                FROM artifact_references r WHERE r.project_id = ?1
                UNION ALL SELECT 'connector-operation', o.id, o.project_id, o.task_id, o.state, o.request_digest,
                    0, o.expires_at_ms, o.created_at_ms, o.completed_at_ms,
                    COALESCE((SELECT a.outcome FROM fictional_connector_audit a
                      WHERE a.operation_id = o.id ORDER BY a.created_at_ms DESC, a.id DESC LIMIT 1), o.state)
                FROM fictional_connector_operations o WHERE o.project_id = ?1
                UNION ALL SELECT 'browser-verification', v.id, v.project_id, v.task_id, v.state, v.request_digest,
                    0, v.expires_at_ms, v.created_at_ms, v.completed_at_ms,
                    COALESCE((SELECT a.outcome FROM controlled_browser_verification_audit a
                      WHERE a.attempt_id = v.id ORDER BY a.created_at_ms DESC, a.id DESC LIMIT 1), v.state)
                FROM controlled_browser_verification_attempts v WHERE v.project_id = ?1
             ) ORDER BY 9 DESC, 2 DESC LIMIT 64"#,
        )?;
        let entries = statement
            .query_map([project_id], |row| {
                Ok(ContextLedgerEntry {
                    record_kind: row.get(0)?,
                    record_id: row.get(1)?,
                    project_id: row.get(2)?,
                    task_id: row.get(3)?,
                    state: row.get(4)?,
                    bundle_digest: row.get(5)?,
                    item_count: row.get::<_, i64>(6)? as u8,
                    expires_at_ms: row.get(7)?,
                    created_at_ms: row.get(8)?,
                    completed_at_ms: row.get(9)?,
                    audit_outcome: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(StorageError::from)?;
        Ok(entries)
    }
    pub(crate) fn create_task_template_application_reservation(
        &mut self,
        reservation: &TaskTemplateApplicationReservation,
    ) -> Result<(), StorageError> {
        if !valid_task_id(&reservation.id)
            || !matches!(reservation.template_origin.as_str(), "built-in" | "local")
            || reservation.template_version == 0
            || reservation.created_at_ms < 0
            || reservation.expires_at_ms
                != reservation.created_at_ms + TEMPLATE_APPLICATION_RESERVATION_TTL_MS
            || !valid_sha256(&reservation.binding_sha256)
            || !valid_sha256(&reservation.template_sha256)
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let now = now_millis();
        tx.execute("UPDATE task_template_application_reservations SET state='expired' WHERE state='pending' AND expires_at_ms <= ?1", [now])?;
        let context = task_template_application_context_in_transaction(
            &tx,
            &reservation.context.task_id,
            &reservation.context.plan_id,
        )?;
        if context != reservation.context || reservation.expires_at_ms <= now {
            return Err(StorageError::InvalidStatusTransition);
        }
        let count: i64 = tx.query_row(
            "SELECT count(*) FROM task_template_application_reservations WHERE state='pending'",
            [],
            |r| r.get(0),
        )?;
        if count >= TEMPLATE_APPLICATION_PENDING_RESERVATION_LIMIT {
            return Err(StorageError::TaskCapacity);
        }
        tx.execute("INSERT INTO task_template_application_reservations(id,binding_sha256,template_id,template_origin,template_version,template_sha256,task_id,plan_id,task_updated_at_ms,plan_updated_at_ms,state,created_at_ms,expires_at_ms,consumed_at_ms) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,'pending',?11,?12,NULL)", params![reservation.id,reservation.binding_sha256,reservation.template_id,reservation.template_origin,reservation.template_version,reservation.template_sha256,reservation.context.task_id,reservation.context.plan_id,reservation.context.task_updated_at_ms,reservation.context.plan_updated_at_ms,reservation.created_at_ms,reservation.expires_at_ms])?;
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn confirm_task_template_application(
        &mut self,
        reservation_id: &str,
        title: &str,
        plan_text: &str,
    ) -> Result<(), StorageError> {
        if !valid_task_id(reservation_id) {
            return Err(StorageError::InvalidStoredValue);
        }
        let title = normalize_task_text(title, 120, 480)?;
        validate_plan_body(plan_text)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let now = now_millis();
        tx.execute("UPDATE task_template_application_reservations SET state='expired' WHERE state='pending' AND expires_at_ms <= ?1", [now])?;
        let reservation: (String, String, String, i64, String, String, String, i64, i64) = tx.query_row(
            "SELECT binding_sha256, template_id, template_origin, template_version, template_sha256, task_id, plan_id, task_updated_at_ms, plan_updated_at_ms FROM task_template_application_reservations WHERE id=?1 AND state='pending' AND expires_at_ms > ?2",
            params![reservation_id, now], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?)),
        ).optional()?.ok_or(StorageError::TaskNotFound)?;
        let (
            binding,
            template_id,
            origin,
            version,
            template_sha256,
            task_id,
            plan_id,
            task_updated,
            plan_updated,
        ) = reservation;
        let version = u32::try_from(version).map_err(|_| StorageError::InvalidStoredValue)?;
        let context = task_template_application_context_in_transaction(&tx, &task_id, &plan_id)?;
        if context.task_updated_at_ms != task_updated || context.plan_updated_at_ms != plan_updated
        {
            return Err(StorageError::InvalidStatusTransition);
        }
        let template = task_template_for_application(&tx, &template_id, &origin)?;
        if template.state != TemplateState::Active
            || template.version != version
            || template.sha256 != template_sha256
        {
            return Err(StorageError::InvalidStatusTransition);
        }
        let actual =
            task_template_application_binding_digest(&template, &context, &title, plan_text)
                .ok_or(StorageError::InvalidStoredValue)?;
        if actual != binding {
            return Err(StorageError::InvalidStatusTransition);
        }
        let update_time = now
            .max(context.task_updated_at_ms.saturating_add(1))
            .max(context.plan_updated_at_ms.saturating_add(1));
        ensure_editable_task(&tx, &task_id)?;
        if tx.execute(
            "UPDATE task_plans SET body=?1, updated_at_ms=?2 WHERE id=?3 AND task_id=?4",
            params![plan_text, update_time, plan_id, task_id],
        )? != 1
        {
            return Err(StorageError::PlanNotFound);
        }
        tx.execute(
            "UPDATE task_records SET title=?1, updated_at_ms=?2 WHERE id=?3",
            params![title, update_time, task_id],
        )?;
        ensure_task_capacity(&tx, &task_id)?;
        if tx.execute("UPDATE task_template_application_reservations SET state='consumed', consumed_at_ms=?1 WHERE id=?2 AND state='pending'", params![update_time, reservation_id])? != 1 { return Err(StorageError::InvalidStatusTransition); }
        tx.commit()?;
        Ok(())
    }
    pub(crate) fn cancel_task_template_application_reservation(
        &mut self,
        reservation_id: &str,
    ) -> Result<(), StorageError> {
        if !valid_task_id(reservation_id) {
            return Err(StorageError::InvalidStoredValue);
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if tx.execute(
            "UPDATE task_template_application_reservations SET state = 'cancelled'
             WHERE id = ?1 AND state = 'pending' AND expires_at_ms > ?2",
            params![reservation_id, now_millis()],
        )? != 1
        {
            return Err(StorageError::TaskNotFound);
        }
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn task_template_application_context(
        &self,
        task_id: &str,
        plan_id: &str,
    ) -> Result<TaskTemplateApplicationContext, StorageError> {
        task_template_application_context_in_connection(&self.connection, task_id, plan_id)
    }
    #[allow(dead_code)]
    pub(crate) fn insert_local_template(
        &mut self,
        template: &TaskTemplate,
    ) -> Result<StoredLocalTaskTemplate, StorageError> {
        validate_local_template(template)?;
        let timestamp = now_millis();
        let archived_at_ms = matches!(template.state, TemplateState::Archived).then_some(timestamp);
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        ensure_local_template_capacity(&tx, None, Some(template))?;
        insert_local_template_row(&tx, template, timestamp, timestamp, archived_at_ms)?;
        tx.commit()?;
        Ok(StoredLocalTaskTemplate {
            template: template.clone(),
            created_at_ms: timestamp,
            updated_at_ms: timestamp,
            archived_at_ms,
        })
    }

    #[allow(dead_code)]
    pub(crate) fn local_template(
        &self,
        id: &str,
    ) -> Result<Option<StoredLocalTaskTemplate>, StorageError> {
        validate_local_template_id(id)?;
        self.connection
            .query_row(
                "SELECT id, schema_version, origin, title, purpose, instructions, version, sha256,
                        state, created_at_ms, updated_at_ms, archived_at_ms
                 FROM local_task_templates WHERE id = ?1",
                [id],
                local_template_row,
            )
            .optional()?
            .map(stored_local_template_from_row)
            .transpose()
    }

    #[allow(dead_code)]
    pub(crate) fn local_templates(&self) -> Result<Vec<StoredLocalTaskTemplate>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT id, schema_version, origin, title, purpose, instructions, version, sha256,
                    state, created_at_ms, updated_at_ms, archived_at_ms
             FROM local_task_templates ORDER BY updated_at_ms DESC, id ASC",
        )?;
        let rows = statement
            .query_map([], local_template_row)?
            .collect::<Result<Vec<_>, _>>()?;
        rows.into_iter()
            .map(stored_local_template_from_row)
            .collect()
    }

    #[allow(dead_code)]
    pub(crate) fn local_template_capacity(&self) -> Result<LocalTemplateCapacity, StorageError> {
        let templates = self.local_templates()?;
        let canonical_bytes = templates
            .iter()
            .map(|record| {
                canonical_template(&record.template).ok_or(StorageError::InvalidStoredValue)
            })
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .map(String::len)
            .sum();
        Ok(LocalTemplateCapacity {
            record_count: templates.len(),
            canonical_bytes,
        })
    }

    #[allow(dead_code)]
    pub(crate) fn replace_local_template(
        &mut self,
        expected_version: u32,
        template: &TaskTemplate,
    ) -> Result<StoredLocalTaskTemplate, StorageError> {
        validate_local_template(template)?;
        if expected_version == 0 || expected_version.checked_add(1) != Some(template.version) {
            return Err(StorageError::InvalidStoredValue);
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let previous = tx
            .query_row(
                "SELECT id, schema_version, origin, title, purpose, instructions, version, sha256,
                        state, created_at_ms, updated_at_ms, archived_at_ms
                 FROM local_task_templates WHERE id = ?1",
                [&template.id],
                local_template_row,
            )
            .optional()?
            .map(stored_local_template_from_row)
            .transpose()?
            .ok_or(StorageError::TaskNotFound)?;
        if previous.template.version != expected_version {
            return Err(StorageError::InvalidStatusTransition);
        }
        ensure_local_template_capacity(&tx, Some(&template.id), Some(template))?;
        let updated_at_ms = now_millis().max(previous.updated_at_ms.saturating_add(1));
        let archived_at_ms = match template.state {
            TemplateState::Active => None,
            TemplateState::Archived => previous.archived_at_ms.or(Some(updated_at_ms)),
        };
        let changed = tx.execute(
            "UPDATE local_task_templates
             SET title = ?1, purpose = ?2, instructions = ?3, version = ?4, sha256 = ?5,
                 state = ?6, updated_at_ms = ?7, archived_at_ms = ?8
             WHERE id = ?9 AND version = ?10",
            params![
                template.title,
                template.purpose,
                template.instructions,
                template.version,
                template.sha256,
                template_state_value(template.state),
                updated_at_ms,
                archived_at_ms,
                template.id,
                expected_version,
            ],
        )?;
        if changed != 1 {
            return Err(StorageError::InvalidStatusTransition);
        }
        tx.commit()?;
        Ok(StoredLocalTaskTemplate {
            template: template.clone(),
            created_at_ms: previous.created_at_ms,
            updated_at_ms,
            archived_at_ms,
        })
    }

    #[allow(dead_code)]
    pub(crate) fn delete_local_template(&mut self, id: &str) -> Result<(), StorageError> {
        validate_local_template_id(id)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if tx.execute("DELETE FROM local_task_templates WHERE id = ?1", [id])? != 1 {
            return Err(StorageError::TaskNotFound);
        }
        tx.commit()?;
        Ok(())
    }
    pub(crate) fn local_review_promotion_source(
        &mut self,
        collection_id: &str,
        item_id: &str,
        expected_updated_at_ms: Option<i64>,
    ) -> Result<LocalReviewPromotionSource, StorageError> {
        if !valid_task_id(collection_id) || !valid_task_id(item_id) {
            return Err(StorageError::InvalidStoredValue);
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let collection = local_review_mutation_context(
            &tx,
            collection_id,
            expected_updated_at_ms,
            LocalReviewMutationPermission::ActiveContent,
        )?;
        let (plan_id, observed_plan_updated_at_ms): (Option<String>, Option<i64>) = tx.query_row(
            "SELECT plan_id, observed_plan_updated_at_ms FROM local_review_collections WHERE id = ?1",
            [collection_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        let (class, format, item_state, title, content, sha256, byte_size): (String, Option<String>, String, String, Vec<u8>, String, i64) = tx.query_row(
            "SELECT class, text_format, state, title, content, sha256, byte_size FROM local_review_items WHERE id = ?1 AND collection_id = ?2 AND discarded_at_ms IS NULL",
            params![item_id, collection_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?)),
        )?;
        if class != "text"
            || item_state != "ready"
            || byte_size < 1
            || byte_size as usize != content.len()
            || content.len() > 512 * 1024
            || sha256 != review_digest(&content)
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let text_format =
            review_text_format(format.as_deref().ok_or(StorageError::InvalidStoredValue)?)?;
        let content = String::from_utf8(content).map_err(|_| StorageError::InvalidStoredValue)?;
        if !matches!(normalize_review_text(&content, text_format), Ok(ref normalized) if normalized == &content)
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let source = LocalReviewPromotionSource {
            collection_id: collection_id.to_owned(),
            item_id: item_id.to_owned(),
            task_id: collection.task_id,
            plan_id,
            observed_plan_updated_at_ms,
            title,
            text_format,
            sha256,
            content,
        };
        tx.commit()?;
        Ok(source)
    }

    pub(crate) fn context_review_evidence_materials(
        &self,
        project_id: &str,
        task_id: Option<&str>,
        item_ids: &[String],
    ) -> Result<Vec<crate::context_assembly::Material>, StorageError> {
        let Some(task_id) = task_id else {
            return Err(StorageError::TaskNotFound);
        };
        let task_project: String = self.connection.query_row(
            "SELECT project_id FROM task_records WHERE id=?1 AND archived_at_ms IS NULL",
            [task_id],
            |row| row.get(0),
        )?;
        if task_project != project_id {
            return Err(StorageError::TaskNotFound);
        }
        let mut materials = Vec::with_capacity(item_ids.len());
        for item_id in item_ids {
            let (content, digest, byte_size): (Vec<u8>, String, i64) = self.connection.query_row(
                "SELECT i.content, i.sha256, i.byte_size FROM local_review_items i JOIN local_review_collections c ON c.id=i.collection_id WHERE i.id=?1 AND i.class='evidence' AND i.state='ready' AND i.discarded_at_ms IS NULL AND c.task_id=?2 AND c.state='active' AND c.discarded_at_ms IS NULL",
                params![item_id, task_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?;
            if byte_size < 1
                || byte_size as usize != content.len()
                || content.len() > 8 * 1024
                || digest != review_digest(&content)
            {
                return Err(StorageError::InvalidStoredValue);
            }
            let text = String::from_utf8(content).map_err(|_| StorageError::InvalidStoredValue)?;
            materials.push(crate::context_assembly::Material {
                id: item_id.clone(),
                source_class: "local-review-evidence".into(),
                provenance: "M54-approved-local-review-evidence".into(),
                text,
            });
        }
        Ok(materials)
    }

    pub(crate) fn context_scope_metadata_material(
        &self,
        project_id: &str,
        task_id: Option<&str>,
    ) -> Result<crate::context_assembly::Material, StorageError> {
        let project_name: String = self.connection.query_row(
            "SELECT display_name FROM projects WHERE id=?1 AND archived_at_ms IS NULL",
            [project_id],
            |row| row.get(0),
        )?;
        let task = if let Some(task_id) = task_id {
            let (title, status, task_project): (String, String, String) = self.connection.query_row("SELECT title,status,project_id FROM task_records WHERE id=?1 AND archived_at_ms IS NULL", [task_id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?)))?;
            if task_project != project_id {
                return Err(StorageError::TaskNotFound);
            }
            format!("task-title={title}\ntask-status={status}\n")
        } else {
            String::new()
        };
        Ok(crate::context_assembly::Material {
            id: format!("scope:{project_id}:{}", task_id.unwrap_or("project")),
            source_class: "scope-metadata".into(),
            provenance: "M60-bounded-project-task-metadata".into(),
            text: format!("project-name={project_name}\n{task}"),
        })
    }
    #[expect(
        clippy::type_complexity,
        reason = "This private storage projection deliberately matches the fixed LocalReview snapshot fields."
    )]
    pub(crate) fn local_review_snapshot(
        &mut self,
        selected_collection_id: Option<&str>,
    ) -> Result<
        (
            Vec<LocalReviewCollectionSummary>,
            Option<LocalReviewCollectionSummary>,
            Vec<LocalReviewItemSummary>,
            u8,
            u64,
            bool,
        ),
        StorageError,
    > {
        if selected_collection_id.is_some_and(|id| !valid_task_id(id)) {
            return Err(StorageError::InvalidStoredValue);
        }
        let mut collections = Vec::new();
        let mut statement = self.connection.prepare(
            "SELECT id, task_id, plan_id, state, title, updated_at_ms
             FROM local_review_collections WHERE discarded_at_ms IS NULL
             ORDER BY updated_at_ms DESC, id LIMIT 24",
        )?;
        for row in statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })? {
            let (id, task_id, plan_id, state, title, updated_at_ms) = row?;
            if !valid_task_id(&id)
                || !valid_task_id(&task_id)
                || plan_id.as_deref().is_some_and(|id| !valid_task_id(id))
                || !matches!(normalize_review_label(&title), Ok(ref normalized) if normalized == &title)
            {
                return Err(StorageError::InvalidStoredValue);
            }
            let task_context: Option<(String, Option<i64>)> = self
                .connection
                .query_row(
                    "SELECT status, archived_at_ms FROM task_records WHERE id = ?1",
                    [&task_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()?;
            let state = match task_context {
                None => LocalReviewCollectionState::Orphaned,
                Some((status, archived)) if archived.is_some() || status == "completed" => {
                    LocalReviewCollectionState::Frozen
                }
                Some((status, _)) if matches!(status.as_str(), "active" | "paused") => {
                    review_collection_state(&state)?
                }
                _ => LocalReviewCollectionState::Unavailable,
            };
            let item_count: i64 = self.connection.query_row("SELECT count(*) FROM local_review_items WHERE collection_id = ?1 AND discarded_at_ms IS NULL", [&id], |row| row.get(0))?;
            let payload = review_payload_bytes(&self.connection, Some(&id))?;
            let (evidence_count, evidence_bytes): (i64, i64) = self.connection.query_row("SELECT count(*), COALESCE(sum(byte_size), 0) FROM local_review_items WHERE collection_id = ?1 AND class = 'evidence'", [&id], |row| Ok((row.get(0)?, row.get(1)?)))?;
            let (annotation_count, annotation_max_bytes): (i64, i64) = self.connection.query_row(
                "SELECT count(*), COALESCE(max(length(CAST(body AS BLOB))), 0)
                 FROM local_review_annotations
                 JOIN local_review_items ON local_review_items.id = local_review_annotations.item_id
                 WHERE local_review_items.collection_id = ?1",
                [&id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;
            let comparison_count: i64 = self.connection.query_row(
                "SELECT count(*) FROM local_review_comparisons WHERE collection_id = ?1",
                [&id],
                |row| row.get(0),
            )?;
            collections.push(LocalReviewCollectionSummary {
                collection_id: id,
                task_id,
                plan_id,
                title,
                state,
                item_count: u8::try_from(item_count)
                    .map_err(|_| StorageError::InvalidStoredValue)?,
                payload_bytes: payload.max(0) as u64,
                updated_at_ms,
                warning: item_count >= 10
                    || payload >= 3 * 1024 * 1024
                    || evidence_count >= 5
                    || evidence_bytes >= REVIEW_EVIDENCE_WARNING_BYTES as i64
                    || annotation_count >= REVIEW_ANNOTATIONS_WARNING_COUNT
                    || annotation_max_bytes >= REVIEW_ANNOTATION_WARNING_BYTES as i64
                    || comparison_count >= REVIEW_COMPARISONS_WARNING_COUNT,
                annotation_count_warning: annotation_count >= REVIEW_ANNOTATIONS_WARNING_COUNT,
                annotation_byte_warning: annotation_max_bytes
                    >= REVIEW_ANNOTATION_WARNING_BYTES as i64,
                comparison_count_warning: comparison_count >= REVIEW_COMPARISONS_WARNING_COUNT,
            });
        }
        let selected = selected_collection_id.and_then(|id| {
            collections
                .iter()
                .find(|item| item.collection_id == id)
                .cloned()
        });
        let mut items = Vec::new();
        if let Some(collection) = &selected {
            let mut statement = self.connection.prepare(
                "SELECT id, class, text_format, state, title, source_kind, evidence_source, mime_type, width, height, byte_size, sha256, created_at_ms, content
                 FROM local_review_items WHERE collection_id = ?1 AND discarded_at_ms IS NULL
                 ORDER BY created_at_ms DESC, id LIMIT 12",
            )?;
            for row in statement.query_map([&collection.collection_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, Option<i64>>(8)?,
                    row.get::<_, Option<i64>>(9)?,
                    row.get::<_, i64>(10)?,
                    row.get::<_, String>(11)?,
                    row.get::<_, i64>(12)?,
                    row.get::<_, Vec<u8>>(13)?,
                ))
            })? {
                let (
                    id,
                    class,
                    format,
                    state,
                    title,
                    source_kind,
                    evidence_source,
                    mime_type,
                    width,
                    height,
                    byte_size,
                    sha256,
                    created_at_ms,
                    content,
                ) = row?;
                if !valid_task_id(&id)
                    || state != "ready"
                    || !matches!(normalize_review_label(&title), Ok(ref normalized) if normalized == &title)
                    || byte_size < 1
                    || byte_size as usize != content.len()
                    || sha256 != review_digest(&content)
                {
                    return Err(StorageError::InvalidStoredValue);
                }
                let (class, text_format, evidence_source, width, height, line_count) = match class
                    .as_str()
                {
                    "text" => {
                        let format = review_text_format(
                            format.as_deref().ok_or(StorageError::InvalidStoredValue)?,
                        )?;
                        let text = std::str::from_utf8(&content)
                            .map_err(|_| StorageError::InvalidStoredValue)?;
                        if !matches!(normalize_review_text(text, format), Ok(ref normalized) if normalized == text)
                            || mime_type != review_mime_type(format)
                            || width.is_some()
                            || height.is_some()
                        {
                            return Err(StorageError::InvalidStoredValue);
                        }
                        (
                            LocalReviewItemClass::Text,
                            Some(format),
                            None,
                            None,
                            None,
                            Some(
                                u16::try_from(text.lines().count())
                                    .map_err(|_| StorageError::InvalidStoredValue)?,
                            ),
                        )
                    }
                    "image-mockup" => {
                        let image = validate_attachment_image(&content)
                            .map_err(|_| StorageError::InvalidStoredValue)?;
                        if content.len() > REVIEW_IMAGE_BYTES_LIMIT
                            || format.is_some()
                            || mime_type != image.mime_type
                            || width != Some(image.width as i64)
                            || height != Some(image.height as i64)
                            || source_kind != "native-image-input"
                        {
                            return Err(StorageError::InvalidStoredValue);
                        }
                        (
                            LocalReviewItemClass::ImageMockup,
                            None,
                            None,
                            Some(image.width),
                            Some(image.height),
                            None,
                        )
                    }
                    "evidence" => {
                        let evidence_source = evidence_source
                            .as_deref()
                            .ok_or(StorageError::InvalidStoredValue)
                            .and_then(review_evidence_source)?;
                        let valid_envelope = match evidence_source {
                            LocalReviewEvidenceSource::ManualValidationSummary => {
                                parse_manual_evidence_envelope(&content, &title).is_ok()
                            }
                            LocalReviewEvidenceSource::M48GeneratedArtifactMetadata => {
                                parse_m48_metadata_evidence_envelope(&content, &title).is_ok()
                            }
                            LocalReviewEvidenceSource::SafePreviewMetadata => {
                                parse_safe_preview_metadata_evidence_envelope(&content, &title)
                                    .is_ok()
                            }
                            _ => false,
                        };
                        if format.is_some()
                            || width.is_some()
                            || height.is_some()
                            || mime_type != "application/json; profile=evidence-envelope-v1"
                            || source_kind != "typed-evidence-snapshot"
                            || !valid_envelope
                        {
                            return Err(StorageError::InvalidStoredValue);
                        }
                        (
                            LocalReviewItemClass::Evidence,
                            None,
                            Some(evidence_source),
                            None,
                            None,
                            None,
                        )
                    }
                    _ => return Err(StorageError::InvalidStoredValue),
                };
                let mut annotations = Vec::new();
                let mut annotations_statement = self.connection.prepare(
                    "SELECT id, schema_version, state, body, created_at_ms, updated_at_ms
                     FROM local_review_annotations WHERE item_id = ?1
                     ORDER BY CASE state WHEN 'open' THEN 0 WHEN 'resolved' THEN 1 ELSE 2 END,
                              created_at_ms, id",
                )?;
                for annotation in annotations_statement.query_map([&id], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, i64>(5)?,
                    ))
                })? {
                    let (
                        annotation_id,
                        schema_version,
                        annotation_state,
                        body,
                        created_at_ms,
                        updated_at_ms,
                    ) = annotation?;
                    if !valid_task_id(&annotation_id)
                        || schema_version != 1
                        || created_at_ms < 0
                        || updated_at_ms < created_at_ms
                        || !matches!(normalize_review_annotation_text(&body), Ok(ref normalized) if normalized == &body)
                    {
                        return Err(StorageError::InvalidStoredValue);
                    }
                    let state = match annotation_state.as_str() {
                        "open" => LocalReviewAnnotationState::Open,
                        "resolved" => LocalReviewAnnotationState::Resolved,
                        _ => return Err(StorageError::InvalidStoredValue),
                    };
                    annotations.push(LocalReviewAnnotationSummary {
                        schema_version: 1,
                        annotation_id,
                        item_id: id.clone(),
                        text: body,
                        state,
                        created_at_ms,
                        updated_at_ms,
                    });
                }
                items.push(LocalReviewItemSummary {
                    item_id: id,
                    class,
                    text_format,
                    source_kind: review_source_kind(&source_kind)?,
                    evidence_source,
                    state: LocalReviewItemState::Ready,
                    title,
                    mime_type,
                    width,
                    height,
                    byte_size: byte_size as u64,
                    line_count,
                    sha256,
                    created_at_ms,
                    annotations,
                });
            }
        }
        let count = collections.len() as u8;
        let payload = review_payload_bytes(&self.connection, None)?.max(0) as u64;
        Ok((
            collections,
            selected,
            items,
            count,
            payload,
            count >= 20 || payload >= 24 * 1024 * 1024,
        ))
    }

    pub(crate) fn create_local_review_collection(
        &mut self,
        task_id: &str,
        plan_id: Option<&str>,
        title: &str,
    ) -> Result<String, StorageError> {
        if !valid_task_id(task_id) || plan_id.is_some_and(|id| !valid_task_id(id)) {
            return Err(StorageError::InvalidStoredValue);
        }
        let title = normalize_review_label(title)?;
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let total: i64 = tx.query_row(
            "SELECT count(*) FROM local_review_collections WHERE discarded_at_ms IS NULL",
            [],
            |row| row.get(0),
        )?;
        if total >= REVIEW_COLLECTION_LIMIT {
            return Err(StorageError::TaskCapacity);
        }
        let task: Option<(String, Option<i64>)> = tx
            .query_row(
                "SELECT status, archived_at_ms FROM task_records WHERE id = ?1",
                [task_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let Some((status, archived)) = task else {
            return Err(StorageError::TaskNotFound);
        };
        if archived.is_some() || !matches!(status.as_str(), "active" | "paused") {
            return Err(StorageError::TaskArchived);
        }
        let active: i64 = tx.query_row("SELECT count(*) FROM local_review_collections WHERE state = 'active' AND discarded_at_ms IS NULL", [], |row| row.get(0))?;
        if active >= REVIEW_ACTIVE_COLLECTION_LIMIT {
            return Err(StorageError::TaskCapacity);
        }
        let observed_plan_updated_at_ms: i64 = if let Some(plan_id) = plan_id {
            tx.query_row(
                "SELECT updated_at_ms FROM task_plans WHERE id = ?1 AND task_id = ?2",
                params![plan_id, task_id],
                |row| row.get(0),
            )
            .optional()?
            .ok_or(StorageError::PlanNotFound)?
        } else {
            -1
        };
        let id = Uuid::now_v7().to_string();
        tx.execute("INSERT INTO local_review_collections (id, schema_version, task_id, plan_id, observed_plan_updated_at_ms, state, title, created_at_ms, updated_at_ms) VALUES (?1, 1, ?2, ?3, ?4, 'active', ?5, ?6, ?6)", params![id, task_id, plan_id, (observed_plan_updated_at_ms >= 0).then_some(observed_plan_updated_at_ms), title, now])?;
        tx.commit()?;
        Ok(id)
    }

    pub(crate) fn create_local_review_text_item(
        &mut self,
        collection_id: &str,
        expected_updated_at_ms: i64,
        title: &str,
        format: LocalReviewTextFormat,
        content: &str,
    ) -> Result<String, StorageError> {
        if !valid_task_id(collection_id) || expected_updated_at_ms < 0 {
            return Err(StorageError::InvalidStoredValue);
        }
        let title = normalize_review_label(title)?;
        let content = normalize_review_text(content, format)?;
        let bytes = content.as_bytes();
        let now = now_millis();
        let session_id = self.activity_session_id.clone();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let context = local_review_mutation_context(
            &tx,
            collection_id,
            Some(expected_updated_at_ms),
            LocalReviewMutationPermission::ActiveContent,
        )?;
        let items: i64 = tx.query_row("SELECT count(*) FROM local_review_items WHERE collection_id = ?1 AND discarded_at_ms IS NULL", [collection_id], |row| row.get(0))?;
        if items >= REVIEW_ITEMS_PER_COLLECTION_LIMIT {
            return Err(StorageError::TaskCapacity);
        }
        let collection_payload = review_payload_bytes(&tx, Some(collection_id))?;
        let total_payload = review_payload_bytes(&tx, None)?;
        let added = bytes.len() as i64 + title.len() as i64 + 256;
        if collection_payload + added > REVIEW_COLLECTION_PAYLOAD_LIMIT
            || total_payload + added > REVIEW_PAYLOAD_LIMIT
        {
            return Err(StorageError::TaskCapacity);
        }
        let id = Uuid::now_v7().to_string();
        tx.execute("INSERT INTO local_review_items (id, schema_version, collection_id, class, text_format, mime_type, width, height, state, title, source_kind, provenance, content, sha256, byte_size, created_at_ms, updated_at_ms) VALUES (?1, 1, ?2, 'text', ?3, ?4, NULL, NULL, 'ready', ?5, 'user-authored-text', '', ?6, ?7, ?8, ?9, ?9)", params![id, collection_id, review_text_format_value(format), review_mime_type(format), title, bytes, review_digest(bytes), bytes.len() as i64, now])?;
        tx.execute(
            "UPDATE local_review_collections SET updated_at_ms = CASE WHEN updated_at_ms >= ?1 THEN updated_at_ms + 1 ELSE ?1 END WHERE id = ?2",
            params![now, collection_id],
        )?;
        append_local_review_activity(
            &tx,
            collection_id,
            &context.task_id,
            &session_id,
            "item-added",
            now,
        )?;
        tx.commit()?;
        Ok(id)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "The fixed M48 claim is kept scalar to avoid admitting an extensible artifact payload envelope."
    )]
    pub(crate) fn create_local_review_m48_artifact_copy(
        &mut self,
        collection_id: &str,
        expected_updated_at_ms: i64,
        artifact_id: &str,
        artifact_sha256: &str,
        title: &str,
        format: LocalReviewTextFormat,
        content: &str,
    ) -> Result<String, StorageError> {
        if !valid_task_id(collection_id)
            || !valid_task_id(artifact_id)
            || expected_updated_at_ms < 0
            || artifact_sha256.len() != 64
            || !artifact_sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let title = normalize_review_label(title)?;
        let content = normalize_review_text(content, format)?;
        let bytes = content.as_bytes();
        if review_digest(bytes) != artifact_sha256 {
            return Err(StorageError::InvalidStoredValue);
        }
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        local_review_mutation_context(
            &tx,
            collection_id,
            Some(expected_updated_at_ms),
            LocalReviewMutationPermission::ActiveContent,
        )?;
        let items: i64 = tx.query_row(
            "SELECT count(*) FROM local_review_items WHERE collection_id = ?1 AND discarded_at_ms IS NULL",
            [collection_id],
            |row| row.get(0),
        )?;
        if items >= REVIEW_ITEMS_PER_COLLECTION_LIMIT {
            return Err(StorageError::TaskCapacity);
        }
        let collection_payload = review_payload_bytes(&tx, Some(collection_id))?;
        let total_payload = review_payload_bytes(&tx, None)?;
        let added = bytes.len() as i64 + title.len() as i64 + 256;
        if collection_payload + added > REVIEW_COLLECTION_PAYLOAD_LIMIT
            || total_payload + added > REVIEW_PAYLOAD_LIMIT
        {
            return Err(StorageError::TaskCapacity);
        }
        let id = Uuid::now_v7().to_string();
        tx.execute(
            "INSERT INTO local_review_items (id, schema_version, collection_id, class, text_format, mime_type, width, height, state, title, source_kind, provenance, content, sha256, byte_size, created_at_ms, updated_at_ms) VALUES (?1, 1, ?2, 'text', ?3, ?4, NULL, NULL, 'ready', ?5, 'm48-artifact-copy', '', ?6, ?7, ?8, ?9, ?9)",
            params![id, collection_id, review_text_format_value(format), review_mime_type(format), title, bytes, artifact_sha256, bytes.len() as i64, now],
        )?;
        tx.execute(
            "UPDATE local_review_collections SET updated_at_ms = CASE WHEN updated_at_ms >= ?1 THEN updated_at_ms + 1 ELSE ?1 END WHERE id = ?2",
            params![now, collection_id],
        )?;
        tx.commit()?;
        Ok(id)
    }

    pub(crate) fn create_local_review_text_comparison(
        &mut self,
        collection_id: &str,
        left_item_id: &str,
        right_item_id: &str,
        expected_updated_at_ms: i64,
    ) -> Result<String, StorageError> {
        if !valid_task_id(collection_id)
            || !valid_task_id(left_item_id)
            || !valid_task_id(right_item_id)
            || left_item_id == right_item_id
            || expected_updated_at_ms < 0
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        local_review_mutation_context(
            &tx,
            collection_id,
            Some(expected_updated_at_ms),
            LocalReviewMutationPermission::ActiveContent,
        )?;
        let comparison_count: i64 = tx.query_row(
            "SELECT count(*) FROM local_review_comparisons WHERE collection_id = ?1",
            [collection_id],
            |row| row.get(0),
        )?;
        if comparison_count >= REVIEW_COMPARISONS_PER_COLLECTION_LIMIT {
            return Err(StorageError::TaskCapacity);
        }
        #[expect(
            clippy::type_complexity,
            reason = "The closed comparison query returns a fixed six-field item record."
        )]
        let read_item = |item_id: &str| -> Result<
            (String, String, String, Vec<u8>, String, i64),
            StorageError,
        > {
            tx.query_row(
                "SELECT class, COALESCE(text_format, ''), state, content, sha256, byte_size FROM local_review_items
                 WHERE id = ?1 AND collection_id = ?2 AND discarded_at_ms IS NULL",
                params![item_id, collection_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
            ).map_err(StorageError::from)
        };
        let left = read_item(left_item_id)?;
        let right = read_item(right_item_id)?;
        let validate = |item: &(String, String, String, Vec<u8>, String, i64)| -> Result<LocalReviewTextFormat, StorageError> {
            if item.0 != "text" || item.2 != "ready" || item.5 < 1 || item.5 as usize != item.3.len() || item.3.len() > REVIEW_COMPARISON_BYTES_LIMIT || item.4 != review_digest(&item.3) {
                return Err(StorageError::InvalidStoredValue);
            }
            let format = review_text_format(&item.1)?;
            let text = std::str::from_utf8(&item.3).map_err(|_| StorageError::InvalidStoredValue)?;
            if !matches!(normalize_review_text(text, format), Ok(ref normalized) if normalized == text) || text.lines().count() > REVIEW_COMPARISON_LINES_LIMIT {
                return Err(StorageError::InvalidStoredValue);
            }
            Ok(format)
        };
        let left_format = validate(&left)?;
        let right_format = validate(&right)?;
        if left_format != right_format {
            return Err(StorageError::InvalidStoredValue);
        }
        let id = Uuid::now_v7().to_string();
        tx.execute(
            "INSERT INTO local_review_comparisons (id, schema_version, collection_id, left_item_id, right_item_id, left_sha256, right_sha256, text_format, state, created_at_ms)
             VALUES (?1, 1, ?2, ?3, ?4, ?5, ?6, ?7, 'ready', ?8)",
            params![id, collection_id, left_item_id, right_item_id, left.4, right.4, review_text_format_value(left_format), now],
        )?;
        tx.execute(
            "UPDATE local_review_collections SET updated_at_ms = CASE WHEN updated_at_ms >= ?1 THEN updated_at_ms + 1 ELSE ?1 END WHERE id = ?2",
            params![now, collection_id],
        )?;
        tx.commit()?;
        Ok(id)
    }

    pub(crate) fn discard_local_review_text_comparison(
        &mut self,
        collection_id: &str,
        comparison_id: &str,
        expected_updated_at_ms: i64,
    ) -> Result<(), StorageError> {
        if !valid_task_id(collection_id)
            || !valid_task_id(comparison_id)
            || expected_updated_at_ms < 0
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        local_review_mutation_context(
            &tx,
            collection_id,
            Some(expected_updated_at_ms),
            LocalReviewMutationPermission::RecoveryDiscard,
        )?;
        if tx.execute(
            "DELETE FROM local_review_comparisons WHERE id = ?1 AND collection_id = ?2",
            params![comparison_id, collection_id],
        )? != 1
        {
            return Err(StorageError::TaskNotFound);
        }
        tx.execute(
            "UPDATE local_review_collections SET updated_at_ms = CASE WHEN updated_at_ms >= ?1 THEN updated_at_ms + 1 ELSE ?1 END WHERE id = ?2",
            params![now, collection_id],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn local_review_comparisons(
        &self,
        collection_id: &str,
    ) -> Result<Vec<LocalReviewComparisonSummary>, StorageError> {
        if !valid_task_id(collection_id) {
            return Err(StorageError::InvalidStoredValue);
        }
        let mut statement = self.connection.prepare(
            "SELECT id, left_item_id, right_item_id, left_sha256, right_sha256, text_format, created_at_ms
             FROM local_review_comparisons WHERE collection_id = ?1 ORDER BY created_at_ms, id",
        )?;
        let mut comparisons = Vec::new();
        for row in statement.query_map([collection_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, i64>(6)?,
            ))
        })? {
            let (id, left_item_id, right_item_id, left_sha256, right_sha256, format, created_at_ms) =
                row?;
            let text_format = review_text_format(&format)?;
            #[expect(
                clippy::type_complexity,
                reason = "The closed comparison-state query returns a fixed six-field item record."
            )]
            let current = |item_id: &str| -> Result<
                Option<(String, String, String, Vec<u8>, String, i64)>,
                StorageError,
            > {
                self.connection.query_row(
                    "SELECT class, COALESCE(text_format, ''), state, content, sha256, byte_size FROM local_review_items WHERE id = ?1 AND collection_id = ?2 AND discarded_at_ms IS NULL",
                    params![item_id, collection_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
                ).optional().map_err(StorageError::from)
            };
            let state = match (current(&left_item_id)?, current(&right_item_id)?) {
                (Some(left), Some(right))
                    if left.0 == "text"
                        && right.0 == "text"
                        && left.1 == format
                        && right.1 == format
                        && left.2 == "ready"
                        && right.2 == "ready"
                        && left.5 == left.3.len() as i64
                        && right.5 == right.3.len() as i64
                        && left.4 == review_digest(&left.3)
                        && right.4 == review_digest(&right.3) =>
                {
                    if left.4 == left_sha256 && right.4 == right_sha256 {
                        LocalReviewComparisonState::Ready
                    } else {
                        LocalReviewComparisonState::Stale
                    }
                }
                (Some(_), Some(_)) => LocalReviewComparisonState::Unavailable,
                _ => LocalReviewComparisonState::Unavailable,
            };
            comparisons.push(LocalReviewComparisonSummary {
                schema_version: 1,
                comparison_id: id,
                collection_id: collection_id.to_owned(),
                left_item_id,
                right_item_id,
                left_sha256,
                right_sha256,
                text_format,
                state,
                created_at_ms,
            });
        }
        Ok(comparisons)
    }

    pub(crate) fn local_review_line_comparison(
        &self,
        comparison_id: &str,
    ) -> Result<LocalReviewLineComparison, StorageError> {
        if !valid_task_id(comparison_id) {
            return Err(StorageError::InvalidStoredValue);
        }
        let (collection_id, left_item_id, right_item_id, left_sha256, right_sha256, format): (String, String, String, String, String, String) = self.connection.query_row(
            "SELECT collection_id, left_item_id, right_item_id, left_sha256, right_sha256, text_format FROM local_review_comparisons WHERE id = ?1",
            [comparison_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
        )?;
        let state = self
            .local_review_comparisons(&collection_id)?
            .iter()
            .find(|comparison| comparison.comparison_id == comparison_id)
            .map(|comparison| comparison.state)
            .ok_or(StorageError::TaskNotFound)?;
        if state != LocalReviewComparisonState::Ready {
            return Ok(LocalReviewLineComparison {
                comparison_id: comparison_id.to_owned(),
                left_item_id,
                left_sha256,
                right_item_id,
                right_sha256,
                text_format: review_text_format(&format)?,
                state,
                lines: Vec::new(),
            });
        }
        let read = |item_id: &str| -> Result<String, StorageError> {
            self.connection
                .query_row(
                    "SELECT content FROM local_review_items WHERE id = ?1",
                    [item_id],
                    |row| row.get::<_, Vec<u8>>(0),
                )
                .and_then(|bytes| {
                    String::from_utf8(bytes).map_err(|_| rusqlite::Error::InvalidQuery)
                })
                .map_err(StorageError::from)
        };
        let lines = comparison_lines(&read(&left_item_id)?, &read(&right_item_id)?);
        Ok(LocalReviewLineComparison {
            comparison_id: comparison_id.to_owned(),
            left_item_id,
            left_sha256,
            right_item_id,
            right_sha256,
            text_format: review_text_format(&format)?,
            state: LocalReviewComparisonState::Ready,
            lines,
        })
    }

    pub(crate) fn create_local_review_annotation(
        &mut self,
        collection_id: &str,
        item_id: &str,
        expected_updated_at_ms: i64,
        text: &str,
    ) -> Result<String, StorageError> {
        if !valid_task_id(collection_id) || !valid_task_id(item_id) || expected_updated_at_ms < 0 {
            return Err(StorageError::InvalidStoredValue);
        }
        let text = normalize_review_annotation_text(text)?;
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        local_review_mutation_context(
            &tx,
            collection_id,
            Some(expected_updated_at_ms),
            LocalReviewMutationPermission::ActiveContent,
        )?;
        let item_state: Option<String> = tx
            .query_row(
                "SELECT state FROM local_review_items
                 WHERE id = ?1 AND collection_id = ?2 AND discarded_at_ms IS NULL",
                params![item_id, collection_id],
                |row| row.get(0),
            )
            .optional()?;
        if item_state.as_deref() != Some("ready") {
            return Err(StorageError::TaskNotFound);
        }
        let annotation_count: i64 = tx.query_row(
            "SELECT count(*) FROM local_review_annotations WHERE item_id = ?1",
            [item_id],
            |row| row.get(0),
        )?;
        if annotation_count >= REVIEW_ANNOTATIONS_PER_ITEM_LIMIT {
            return Err(StorageError::TaskCapacity);
        }
        let added = text.len() as i64 + 128;
        if review_payload_bytes(&tx, Some(collection_id))? + added > REVIEW_COLLECTION_PAYLOAD_LIMIT
            || review_payload_bytes(&tx, None)? + added > REVIEW_PAYLOAD_LIMIT
        {
            return Err(StorageError::TaskCapacity);
        }
        let annotation_id = Uuid::now_v7().to_string();
        tx.execute(
            "INSERT INTO local_review_annotations (
                id, schema_version, item_id, state, body, created_at_ms, updated_at_ms
             ) VALUES (?1, 1, ?2, 'open', ?3, ?4, ?4)",
            params![annotation_id, item_id, text, now],
        )?;
        tx.execute(
            "UPDATE local_review_collections
             SET updated_at_ms = CASE WHEN updated_at_ms >= ?1 THEN updated_at_ms + 1 ELSE ?1 END
             WHERE id = ?2",
            params![now, collection_id],
        )?;
        tx.commit()?;
        Ok(annotation_id)
    }

    pub(crate) fn edit_local_review_annotation(
        &mut self,
        collection_id: &str,
        item_id: &str,
        annotation_id: &str,
        expected_updated_at_ms: i64,
        text: &str,
    ) -> Result<(), StorageError> {
        let text = normalize_review_annotation_text(text)?;
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (
            _state,
            collection_updated_at_ms,
            previous_text,
            _task_id,
            _created_at_ms,
            updated_at_ms,
        ) = annotation_mutation_context(
            &tx,
            collection_id,
            item_id,
            annotation_id,
            expected_updated_at_ms,
            LocalReviewMutationPermission::ActiveContent,
        )?;
        let delta = text.len() as i64 - previous_text.len() as i64;
        if delta > 0
            && (review_payload_bytes(&tx, Some(collection_id))? + delta
                > REVIEW_COLLECTION_PAYLOAD_LIMIT
                || review_payload_bytes(&tx, None)? + delta > REVIEW_PAYLOAD_LIMIT)
        {
            return Err(StorageError::TaskCapacity);
        }
        let next_annotation_updated_at_ms = now.max(updated_at_ms + 1);
        let next_collection_updated_at_ms = now.max(collection_updated_at_ms + 1);
        tx.execute(
            "UPDATE local_review_annotations SET body = ?1, updated_at_ms = ?2 WHERE id = ?3 AND item_id = ?4",
            params![text, next_annotation_updated_at_ms, annotation_id, item_id],
        )?;
        tx.execute(
            "UPDATE local_review_collections SET updated_at_ms = ?1 WHERE id = ?2",
            params![next_collection_updated_at_ms, collection_id],
        )?;
        tx.commit()?;
        Ok(())
    }

    fn transition_local_review_annotation(
        &mut self,
        collection_id: &str,
        item_id: &str,
        annotation_id: &str,
        expected_updated_at_ms: i64,
        from: &str,
        to: &str,
    ) -> Result<(), StorageError> {
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (state, collection_updated_at_ms, _body, _task_id, _created_at_ms, updated_at_ms) =
            annotation_mutation_context(
                &tx,
                collection_id,
                item_id,
                annotation_id,
                expected_updated_at_ms,
                LocalReviewMutationPermission::ActiveContent,
            )?;
        if state != from {
            return Err(StorageError::InvalidStatusTransition);
        }
        let next_annotation_updated_at_ms = now.max(updated_at_ms + 1);
        let next_collection_updated_at_ms = now.max(collection_updated_at_ms + 1);
        tx.execute(
            "UPDATE local_review_annotations SET state = ?1, updated_at_ms = ?2 WHERE id = ?3 AND item_id = ?4",
            params![to, next_annotation_updated_at_ms, annotation_id, item_id],
        )?;
        tx.execute(
            "UPDATE local_review_collections SET updated_at_ms = ?1 WHERE id = ?2",
            params![next_collection_updated_at_ms, collection_id],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn resolve_local_review_annotation(
        &mut self,
        collection_id: &str,
        item_id: &str,
        annotation_id: &str,
        expected_updated_at_ms: i64,
    ) -> Result<(), StorageError> {
        self.transition_local_review_annotation(
            collection_id,
            item_id,
            annotation_id,
            expected_updated_at_ms,
            "open",
            "resolved",
        )
    }

    pub(crate) fn reopen_local_review_annotation(
        &mut self,
        collection_id: &str,
        item_id: &str,
        annotation_id: &str,
        expected_updated_at_ms: i64,
    ) -> Result<(), StorageError> {
        self.transition_local_review_annotation(
            collection_id,
            item_id,
            annotation_id,
            expected_updated_at_ms,
            "resolved",
            "open",
        )
    }

    pub(crate) fn delete_local_review_annotation(
        &mut self,
        collection_id: &str,
        item_id: &str,
        annotation_id: &str,
        expected_updated_at_ms: i64,
    ) -> Result<(), StorageError> {
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (_state, collection_updated_at_ms, _body, _task_id, _created_at_ms, _updated_at_ms) =
            annotation_mutation_context(
                &tx,
                collection_id,
                item_id,
                annotation_id,
                expected_updated_at_ms,
                LocalReviewMutationPermission::RecoveryDiscard,
            )?;
        if tx.execute(
            "DELETE FROM local_review_annotations WHERE id = ?1 AND item_id = ?2",
            params![annotation_id, item_id],
        )? != 1
        {
            return Err(StorageError::TaskNotFound);
        }
        tx.execute(
            "UPDATE local_review_collections SET updated_at_ms = ?1 WHERE id = ?2",
            params![now.max(collection_updated_at_ms + 1), collection_id],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn create_local_review_image_item(
        &mut self,
        collection_id: &str,
        expected_updated_at_ms: i64,
        title: &str,
        bytes: &[u8],
    ) -> Result<String, StorageError> {
        if !valid_task_id(collection_id)
            || expected_updated_at_ms < 0
            || bytes.is_empty()
            || bytes.len() > REVIEW_IMAGE_BYTES_LIMIT
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let title = normalize_review_label(title)?;
        let image =
            validate_attachment_image(bytes).map_err(|_| StorageError::InvalidStoredValue)?;
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        local_review_mutation_context(
            &tx,
            collection_id,
            Some(expected_updated_at_ms),
            LocalReviewMutationPermission::ActiveContent,
        )?;
        let count: i64 = tx.query_row("SELECT count(*) FROM local_review_items WHERE collection_id = ?1 AND class = 'image-mockup'", [collection_id], |row| row.get(0))?;
        if count >= 3 {
            return Err(StorageError::TaskCapacity);
        }
        let payload = review_payload_bytes(&tx, Some(collection_id))?;
        let total = review_payload_bytes(&tx, None)?;
        let added = bytes.len() as i64 + title.len() as i64 + 256;
        if payload + added > REVIEW_COLLECTION_PAYLOAD_LIMIT || total + added > REVIEW_PAYLOAD_LIMIT
        {
            return Err(StorageError::TaskCapacity);
        }
        let id = Uuid::now_v7().to_string();
        tx.execute("INSERT INTO local_review_items (id, schema_version, collection_id, class, text_format, mime_type, width, height, state, title, source_kind, provenance, content, sha256, byte_size, created_at_ms, updated_at_ms) VALUES (?1, 1, ?2, 'image-mockup', NULL, ?3, ?4, ?5, 'ready', ?6, 'native-image-input', '', ?7, ?8, ?9, ?10, ?10)", params![id, collection_id, image.mime_type, image.width, image.height, title, bytes, review_digest(bytes), bytes.len() as i64, now])?;
        tx.execute(
            "UPDATE local_review_collections SET updated_at_ms = CASE WHEN updated_at_ms >= ?1 THEN updated_at_ms + 1 ELSE ?1 END WHERE id = ?2",
            params![now, collection_id],
        )?;
        tx.commit()?;
        Ok(id)
    }

    pub(crate) fn create_local_review_manual_evidence_item(
        &mut self,
        collection_id: &str,
        expected_updated_at_ms: i64,
        title: &str,
        summary: &str,
    ) -> Result<String, StorageError> {
        if !valid_task_id(collection_id) || expected_updated_at_ms < 0 {
            return Err(StorageError::InvalidStoredValue);
        }
        let title = normalize_review_label(title)?;
        let summary = normalize_review_text(summary, LocalReviewTextFormat::Plain)?;
        if summary.contains('/') || summary.contains("://") {
            return Err(StorageError::InvalidStoredValue);
        }
        let bytes = manual_evidence_envelope_bytes(&title, &summary)?;
        if bytes.len() > REVIEW_EVIDENCE_BYTES_LIMIT {
            return Err(StorageError::InvalidStoredValue);
        }
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        local_review_mutation_context(
            &tx,
            collection_id,
            Some(expected_updated_at_ms),
            LocalReviewMutationPermission::ActiveContent,
        )?;
        let evidence_count: i64 = tx.query_row("SELECT count(*) FROM local_review_items WHERE collection_id = ?1 AND class = 'evidence'", [collection_id], |row| row.get(0))?;
        if evidence_count >= 6 {
            return Err(StorageError::TaskCapacity);
        }
        let added = bytes.len() as i64 + title.len() as i64 + 256;
        if review_payload_bytes(&tx, Some(collection_id))? + added > REVIEW_COLLECTION_PAYLOAD_LIMIT
            || review_payload_bytes(&tx, None)? + added > REVIEW_PAYLOAD_LIMIT
        {
            return Err(StorageError::TaskCapacity);
        }
        let id = Uuid::now_v7().to_string();
        tx.execute("INSERT INTO local_review_items (id, schema_version, collection_id, class, text_format, mime_type, width, height, state, title, source_kind, provenance, evidence_source, content, sha256, byte_size, created_at_ms, updated_at_ms) VALUES (?1, 1, ?2, 'evidence', NULL, 'application/json; profile=evidence-envelope-v1', NULL, NULL, 'ready', ?3, 'typed-evidence-snapshot', 'manual-validation-summary', 'manual-validation-summary', ?4, ?5, ?6, ?7, ?7)", params![id, collection_id, title, bytes, review_digest(&bytes), bytes.len() as i64, now])?;
        tx.execute(
            "UPDATE local_review_collections SET updated_at_ms = CASE WHEN updated_at_ms >= ?1 THEN updated_at_ms + 1 ELSE ?1 END WHERE id = ?2",
            params![now, collection_id],
        )?;
        tx.commit()?;
        Ok(id)
    }

    pub(crate) fn create_local_review_m48_generated_artifact_metadata_evidence_item(
        &mut self,
        collection_id: &str,
        expected_updated_at_ms: i64,
        title: &str,
        summary: &str,
        details: &LocalReviewM48GeneratedArtifactMetadataDetails,
    ) -> Result<String, StorageError> {
        if !valid_task_id(collection_id) || expected_updated_at_ms < 0 {
            return Err(StorageError::InvalidStoredValue);
        }
        let title = normalize_review_label(title)?;
        let summary = normalize_review_text(summary, LocalReviewTextFormat::Plain)?;
        let bytes = m48_metadata_evidence_envelope_bytes(&title, &summary, details)?;
        if bytes.len() > REVIEW_EVIDENCE_BYTES_LIMIT {
            return Err(StorageError::InvalidStoredValue);
        }
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        local_review_mutation_context(
            &tx,
            collection_id,
            Some(expected_updated_at_ms),
            LocalReviewMutationPermission::ActiveContent,
        )?;
        let evidence_count: i64 = tx.query_row(
            "SELECT count(*) FROM local_review_items WHERE collection_id = ?1 AND class = 'evidence'",
            [collection_id],
            |row| row.get(0),
        )?;
        if evidence_count >= 6 {
            return Err(StorageError::TaskCapacity);
        }
        let added = bytes.len() as i64 + title.len() as i64 + 256;
        if review_payload_bytes(&tx, Some(collection_id))? + added > REVIEW_COLLECTION_PAYLOAD_LIMIT
            || review_payload_bytes(&tx, None)? + added > REVIEW_PAYLOAD_LIMIT
        {
            return Err(StorageError::TaskCapacity);
        }
        let id = Uuid::now_v7().to_string();
        tx.execute(
            "INSERT INTO local_review_items (id, schema_version, collection_id, class, text_format, mime_type, width, height, state, title, source_kind, provenance, evidence_source, content, sha256, byte_size, created_at_ms, updated_at_ms) VALUES (?1, 1, ?2, 'evidence', NULL, 'application/json; profile=evidence-envelope-v1', NULL, NULL, 'ready', ?3, 'typed-evidence-snapshot', 'm48-generated-artifact-metadata', 'm48-generated-artifact-metadata', ?4, ?5, ?6, ?7, ?7)",
            params![id, collection_id, title, bytes, review_digest(&bytes), bytes.len() as i64, now],
        )?;
        tx.execute(
            "UPDATE local_review_collections SET updated_at_ms = CASE WHEN updated_at_ms >= ?1 THEN updated_at_ms + 1 ELSE ?1 END WHERE id = ?2",
            params![now, collection_id],
        )?;
        tx.commit()?;
        Ok(id)
    }

    pub(crate) fn local_review_manual_evidence_preview(
        &self,
        item_id: &str,
        sha256: &str,
    ) -> Result<LocalReviewManualEvidencePreview, StorageError> {
        if !valid_task_id(item_id) || sha256.len() != 64 {
            return Err(StorageError::InvalidStoredValue);
        }
        let (title, source_kind, provenance, evidence_source, content, stored_sha, byte_size, created_at_ms): (String, String, String, Option<String>, Vec<u8>, String, i64, i64) = self.connection.query_row("SELECT title, source_kind, provenance, evidence_source, content, sha256, byte_size, created_at_ms FROM local_review_items WHERE id = ?1 AND class = 'evidence' AND state = 'ready'", [item_id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?,row.get(6)?,row.get(7)?)))?;
        if source_kind != "typed-evidence-snapshot"
            || provenance != "manual-validation-summary"
            || evidence_source.as_deref() != Some("manual-validation-summary")
            || sha256 != stored_sha
            || byte_size != content.len() as i64
            || content.len() > REVIEW_EVIDENCE_BYTES_LIMIT
            || stored_sha != review_digest(&content)
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let summary = parse_manual_evidence_envelope(&content, &title)?;
        Ok(LocalReviewManualEvidencePreview {
            schema_version: 1,
            item_id: item_id.to_owned(),
            source: "manual-validation-summary".to_owned(),
            title,
            summary,
            byte_size: byte_size as u64,
            sha256: stored_sha,
            created_at_ms,
        })
    }

    pub(crate) fn local_review_m48_generated_artifact_metadata_evidence_preview(
        &self,
        item_id: &str,
        sha256: &str,
    ) -> Result<LocalReviewM48GeneratedArtifactMetadataEvidencePreview, StorageError> {
        if !valid_task_id(item_id) || !valid_review_sha256(sha256) {
            return Err(StorageError::InvalidStoredValue);
        }
        let (title, source_kind, provenance, evidence_source, content, stored_sha, byte_size, created_at_ms): (String, String, String, Option<String>, Vec<u8>, String, i64, i64) = self.connection.query_row("SELECT title, source_kind, provenance, evidence_source, content, sha256, byte_size, created_at_ms FROM local_review_items WHERE id = ?1 AND class = 'evidence' AND state = 'ready'", [item_id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?,row.get(6)?,row.get(7)?)))?;
        if source_kind != "typed-evidence-snapshot"
            || provenance != "m48-generated-artifact-metadata"
            || evidence_source.as_deref() != Some("m48-generated-artifact-metadata")
            || sha256 != stored_sha
            || byte_size != content.len() as i64
            || content.len() > REVIEW_EVIDENCE_BYTES_LIMIT
            || stored_sha != review_digest(&content)
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let (summary, details) = parse_m48_metadata_evidence_envelope(&content, &title)?;
        Ok(LocalReviewM48GeneratedArtifactMetadataEvidencePreview {
            schema_version: 1,
            item_id: item_id.to_owned(),
            source: LocalReviewEvidenceSource::M48GeneratedArtifactMetadata,
            title,
            summary,
            details,
            byte_size: byte_size as u64,
            sha256: stored_sha,
            created_at_ms,
        })
    }

    pub(crate) fn create_local_review_safe_preview_metadata_evidence_item(
        &mut self,
        collection_id: &str,
        expected_updated_at_ms: i64,
        title: &str,
        summary: &str,
        details: &LocalReviewSafePreviewMetadataDetails,
    ) -> Result<String, StorageError> {
        if !valid_task_id(collection_id) || expected_updated_at_ms < 0 {
            return Err(StorageError::InvalidStoredValue);
        }
        let title = normalize_review_label(title)?;
        let summary = normalize_review_text(summary, LocalReviewTextFormat::Plain)?;
        let bytes = safe_preview_metadata_evidence_envelope_bytes(&title, &summary, details)?;
        if bytes.len() > REVIEW_EVIDENCE_BYTES_LIMIT {
            return Err(StorageError::InvalidStoredValue);
        }
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        local_review_mutation_context(
            &tx,
            collection_id,
            Some(expected_updated_at_ms),
            LocalReviewMutationPermission::ActiveContent,
        )?;
        let evidence_count: i64 = tx.query_row("SELECT count(*) FROM local_review_items WHERE collection_id = ?1 AND class = 'evidence'", [collection_id], |row| row.get(0))?;
        if evidence_count >= 6 {
            return Err(StorageError::TaskCapacity);
        }
        let added = bytes.len() as i64 + title.len() as i64 + 256;
        if review_payload_bytes(&tx, Some(collection_id))? + added > REVIEW_COLLECTION_PAYLOAD_LIMIT
            || review_payload_bytes(&tx, None)? + added > REVIEW_PAYLOAD_LIMIT
        {
            return Err(StorageError::TaskCapacity);
        }
        let id = Uuid::now_v7().to_string();
        tx.execute("INSERT INTO local_review_items (id, schema_version, collection_id, class, text_format, mime_type, width, height, state, title, source_kind, provenance, evidence_source, content, sha256, byte_size, created_at_ms, updated_at_ms) VALUES (?1, 1, ?2, 'evidence', NULL, 'application/json; profile=evidence-envelope-v1', NULL, NULL, 'ready', ?3, 'typed-evidence-snapshot', 'safe-preview-metadata', 'safe-preview-metadata', ?4, ?5, ?6, ?7, ?7)", params![id, collection_id, title, bytes, review_digest(&bytes), bytes.len() as i64, now])?;
        tx.execute("UPDATE local_review_collections SET updated_at_ms = CASE WHEN updated_at_ms >= ?1 THEN updated_at_ms + 1 ELSE ?1 END WHERE id = ?2", params![now, collection_id])?;
        tx.commit()?;
        Ok(id)
    }

    pub(crate) fn local_review_safe_preview_metadata_evidence_preview(
        &self,
        item_id: &str,
        sha256: &str,
    ) -> Result<LocalReviewSafePreviewMetadataEvidencePreview, StorageError> {
        if !valid_task_id(item_id) || !valid_review_sha256(sha256) {
            return Err(StorageError::InvalidStoredValue);
        }
        let (title, source_kind, provenance, evidence_source, content, stored_sha, byte_size, created_at_ms): (String, String, String, Option<String>, Vec<u8>, String, i64, i64) = self.connection.query_row("SELECT title, source_kind, provenance, evidence_source, content, sha256, byte_size, created_at_ms FROM local_review_items WHERE id = ?1 AND class = 'evidence' AND state = 'ready'", [item_id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?,row.get(6)?,row.get(7)?)))?;
        if source_kind != "typed-evidence-snapshot"
            || provenance != "safe-preview-metadata"
            || evidence_source.as_deref() != Some("safe-preview-metadata")
            || sha256 != stored_sha
            || byte_size != content.len() as i64
            || content.len() > REVIEW_EVIDENCE_BYTES_LIMIT
            || stored_sha != review_digest(&content)
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let (summary, details) = parse_safe_preview_metadata_evidence_envelope(&content, &title)?;
        Ok(LocalReviewSafePreviewMetadataEvidencePreview {
            schema_version: 1,
            item_id: item_id.to_owned(),
            source: LocalReviewEvidenceSource::SafePreviewMetadata,
            title,
            summary,
            details,
            byte_size: byte_size as u64,
            sha256: stored_sha,
            created_at_ms,
        })
    }

    pub(crate) fn package_manifest_summary_available_for_local_review(
        &self,
        collection_id: &str,
    ) -> bool {
        self.package_manifest_summary_source_for_local_review(collection_id)
            .is_ok()
    }

    pub(crate) fn git_status_diff_summary_project_for_local_review(
        &self,
        collection_id: &str,
    ) -> Result<String, StorageError> {
        if !valid_task_id(collection_id) {
            return Err(StorageError::InvalidStoredValue);
        }
        self.connection.query_row(
            "SELECT task.project_id FROM local_review_collections AS collection
             JOIN task_records AS task ON task.id = collection.task_id
             JOIN projects AS project ON project.id = task.project_id
             JOIN directory_associations AS association ON association.id = project.active_directory_association_id
             WHERE collection.id = ?1 AND collection.discarded_at_ms IS NULL AND collection.state = 'active'
               AND task.archived_at_ms IS NULL AND task.status IN ('active', 'paused')
               AND task.project_id IS NOT NULL AND project.archived_at_ms IS NULL
               AND association.detached_at_ms IS NULL",
            [collection_id], |row| row.get(0),
        ).map_err(|_| StorageError::ProjectNotFound)
    }

    pub(crate) fn activity_presentation_available_for_local_review(
        &self,
        collection_id: &str,
    ) -> bool {
        self.activity_presentation_details_for_local_review(collection_id)
            .is_ok()
    }

    fn activity_presentation_details_for_local_review(
        &self,
        collection_id: &str,
    ) -> Result<LocalReviewActivityPresentationDetails, StorageError> {
        let task_id = self.connection.query_row("SELECT task_id FROM local_review_collections WHERE id = ?1 AND state = 'active' AND discarded_at_ms IS NULL", [collection_id], |row| row.get::<_, String>(0))?;
        let valid: i64 = self.connection.query_row("SELECT count(*) FROM task_records WHERE id = ?1 AND archived_at_ms IS NULL AND status IN ('active', 'paused')", [&task_id], |row| row.get(0))?;
        if valid != 1 {
            return Err(StorageError::TaskArchived);
        }
        let mut statement = self.connection.prepare("SELECT kind FROM local_review_activity_ledger WHERE collection_id = ?1 AND task_id = ?2 AND session_id = ?3 ORDER BY created_at_ms DESC, id DESC LIMIT 13")?;
        let rows = statement
            .query_map(
                params![collection_id, task_id, self.activity_session_id],
                |row| row.get::<_, String>(0),
            )?
            .collect::<Result<Vec<_>, _>>()?;
        if rows.is_empty() {
            return Err(StorageError::InvalidStoredValue);
        }
        let mut details = LocalReviewActivityPresentationDetails {
            scope: LocalReviewActivityScope::CurrentSession,
            event_count: rows.len().min(12) as u8,
            item_added_count: 0,
            item_discarded_count: 0,
            annotation_changed_count: 0,
            comparison_changed_count: 0,
            promotion_prepared_count: 0,
            promotion_completed_count: 0,
            collection_changed_count: 0,
            truncated: rows.len() > 12,
        };
        for kind in rows.iter().take(12) {
            match kind.as_str() {
                "item-added" | "activity-evidence-captured" => details.item_added_count += 1,
                _ => return Err(StorageError::InvalidStoredValue),
            }
        }
        Ok(details)
    }

    pub(crate) fn create_local_review_activity_presentation_evidence_item(
        &mut self,
        collection_id: &str,
        expected_updated_at_ms: i64,
    ) -> Result<String, StorageError> {
        let details = self.activity_presentation_details_for_local_review(collection_id)?;
        let title = "Activity presentation";
        let summary = "Captured current-session native Local Review activity.";
        let bytes = activity_presentation_evidence_envelope_bytes(title, summary, &details)?;
        parse_activity_presentation_evidence_envelope(&bytes, title)?;
        let now = now_millis();
        let session_id = self.activity_session_id.clone();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let context = local_review_mutation_context(
            &tx,
            collection_id,
            Some(expected_updated_at_ms),
            LocalReviewMutationPermission::ActiveContent,
        )?;
        if bytes.len() > REVIEW_EVIDENCE_BYTES_LIMIT {
            return Err(StorageError::InvalidStoredValue);
        }
        let id = Uuid::now_v7().to_string();
        tx.execute("INSERT INTO local_review_items (id, schema_version, collection_id, class, text_format, mime_type, width, height, state, title, source_kind, provenance, evidence_source, content, sha256, byte_size, created_at_ms, updated_at_ms) VALUES (?1, 1, ?2, 'evidence', NULL, 'application/json; profile=evidence-envelope-v1', NULL, NULL, 'ready', ?3, 'typed-evidence-snapshot', 'activity-presentation', 'activity-presentation', ?4, ?5, ?6, ?7, ?7)", params![id, collection_id, title, bytes, review_digest(&bytes), bytes.len() as i64, now])?;
        tx.execute("UPDATE local_review_collections SET updated_at_ms = CASE WHEN updated_at_ms >= ?1 THEN updated_at_ms + 1 ELSE ?1 END WHERE id = ?2", params![now, collection_id])?;
        append_local_review_activity(
            &tx,
            collection_id,
            &context.task_id,
            &session_id,
            "activity-evidence-captured",
            now,
        )?;
        tx.commit()?;
        Ok(id)
    }

    pub(crate) fn local_review_activity_presentation_evidence_preview(
        &self,
        item_id: &str,
        sha256: &str,
    ) -> Result<LocalReviewActivityPresentationEvidencePreview, StorageError> {
        let (title, content, stored_sha, byte_size, created_at_ms): (String, Vec<u8>, String, i64, i64) = self.connection.query_row("SELECT title, content, sha256, byte_size, created_at_ms FROM local_review_items WHERE id = ?1 AND provenance = 'activity-presentation' AND evidence_source = 'activity-presentation'", [item_id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?)))?;
        if sha256 != stored_sha
            || stored_sha != review_digest(&content)
            || byte_size != content.len() as i64
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let (summary, details) = parse_activity_presentation_evidence_envelope(&content, &title)?;
        Ok(LocalReviewActivityPresentationEvidencePreview {
            schema_version: 1,
            item_id: item_id.to_owned(),
            source: LocalReviewEvidenceSource::ActivityPresentation,
            title,
            summary,
            details,
            byte_size: byte_size as u64,
            sha256: stored_sha,
            created_at_ms,
        })
    }

    pub(crate) fn approval_presentation_available_for_local_review(
        &self,
        collection_id: &str,
    ) -> bool {
        let tx = match self.connection.unchecked_transaction() {
            Ok(tx) => tx,
            Err(_) => return false,
        };
        let result = local_review_mutation_context(
            &tx,
            collection_id,
            None,
            LocalReviewMutationPermission::ActiveContent,
        )
        .and_then(|context| approval_presentation_details_for_task(&tx, &context.task_id));
        let _ = tx.rollback();
        result.is_ok()
    }

    pub(crate) fn create_local_review_approval_presentation_evidence_item(
        &mut self,
        collection_id: &str,
        expected_updated_at_ms: i64,
    ) -> Result<String, StorageError> {
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let context = local_review_mutation_context(
            &tx,
            collection_id,
            Some(expected_updated_at_ms),
            LocalReviewMutationPermission::ActiveContent,
        )?;
        let details = approval_presentation_details_for_task(&tx, &context.task_id)?;
        let title = "Approval presentation";
        let summary = "Captured approved Advisor dispatch presentation.";
        let bytes = approval_presentation_evidence_envelope_bytes(title, summary, &details)?;
        parse_approval_presentation_evidence_envelope(&bytes, title)?;
        if bytes.len() > REVIEW_EVIDENCE_BYTES_LIMIT {
            return Err(StorageError::InvalidStoredValue);
        }
        let id = Uuid::now_v7().to_string();
        tx.execute("INSERT INTO local_review_items (id, schema_version, collection_id, class, text_format, mime_type, width, height, state, title, source_kind, provenance, evidence_source, content, sha256, byte_size, created_at_ms, updated_at_ms) VALUES (?1, 1, ?2, 'evidence', NULL, 'application/json; profile=evidence-envelope-v1', NULL, NULL, 'ready', ?3, 'typed-evidence-snapshot', 'approval-presentation', 'approval-presentation', ?4, ?5, ?6, ?7, ?7)", params![id, collection_id, title, bytes, review_digest(&bytes), bytes.len() as i64, now])?;
        tx.execute("UPDATE local_review_collections SET updated_at_ms = CASE WHEN updated_at_ms >= ?1 THEN updated_at_ms + 1 ELSE ?1 END WHERE id = ?2", params![now, collection_id])?;
        tx.commit()?;
        Ok(id)
    }

    pub(crate) fn local_review_approval_presentation_evidence_preview(
        &self,
        item_id: &str,
        sha256: &str,
    ) -> Result<LocalReviewApprovalPresentationEvidencePreview, StorageError> {
        let (title, content, stored_sha, byte_size, created_at_ms): (String, Vec<u8>, String, i64, i64) = self.connection.query_row("SELECT title, content, sha256, byte_size, created_at_ms FROM local_review_items WHERE id = ?1 AND provenance = 'approval-presentation' AND evidence_source = 'approval-presentation'", [item_id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?)))?;
        if sha256 != stored_sha
            || stored_sha != review_digest(&content)
            || byte_size != content.len() as i64
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let (summary, details) = parse_approval_presentation_evidence_envelope(&content, &title)?;
        Ok(LocalReviewApprovalPresentationEvidencePreview {
            schema_version: 1,
            item_id: item_id.to_owned(),
            source: LocalReviewEvidenceSource::ApprovalPresentation,
            title,
            summary,
            details,
            byte_size: byte_size as u64,
            sha256: stored_sha,
            created_at_ms,
        })
    }

    pub(crate) fn create_local_review_git_status_diff_summary_evidence_item(
        &mut self,
        collection_id: &str,
        expected_updated_at_ms: i64,
        project_id: &str,
        details: &LocalReviewGitStatusDiffSummaryDetails,
    ) -> Result<String, StorageError> {
        if self.git_status_diff_summary_project_for_local_review(collection_id)? != project_id {
            return Err(StorageError::InvalidStoredValue);
        }
        let title = "Git status and diff summary";
        let summary = "Captured native Git status and diff aggregate summary.";
        let bytes = git_status_diff_summary_evidence_envelope_bytes(title, summary, details)?;
        parse_git_status_diff_summary_evidence_envelope(&bytes, title)?;
        if bytes.len() > REVIEW_EVIDENCE_BYTES_LIMIT {
            return Err(StorageError::InvalidStoredValue);
        }
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        local_review_mutation_context(
            &tx,
            collection_id,
            Some(expected_updated_at_ms),
            LocalReviewMutationPermission::ActiveContent,
        )?;
        let bound: String = tx.query_row("SELECT project_id FROM task_records WHERE id = (SELECT task_id FROM local_review_collections WHERE id = ?1)", [collection_id], |row| row.get(0))?;
        if bound != project_id {
            return Err(StorageError::InvalidStoredValue);
        }
        let count: i64 = tx.query_row("SELECT count(*) FROM local_review_items WHERE collection_id = ?1 AND class = 'evidence'", [collection_id], |row| row.get(0))?;
        if count >= 6 {
            return Err(StorageError::TaskCapacity);
        }
        let added = bytes.len() as i64 + title.len() as i64 + 256;
        if review_payload_bytes(&tx, Some(collection_id))? + added > REVIEW_COLLECTION_PAYLOAD_LIMIT
            || review_payload_bytes(&tx, None)? + added > REVIEW_PAYLOAD_LIMIT
        {
            return Err(StorageError::TaskCapacity);
        }
        let id = Uuid::now_v7().to_string();
        tx.execute("INSERT INTO local_review_items (id, schema_version, collection_id, class, text_format, mime_type, width, height, state, title, source_kind, provenance, evidence_source, content, sha256, byte_size, created_at_ms, updated_at_ms) VALUES (?1, 1, ?2, 'evidence', NULL, 'application/json; profile=evidence-envelope-v1', NULL, NULL, 'ready', ?3, 'typed-evidence-snapshot', 'git-status-diff-summary', 'git-status-diff-summary', ?4, ?5, ?6, ?7, ?7)", params![id, collection_id, title, bytes, review_digest(&bytes), bytes.len() as i64, now])?;
        tx.execute("UPDATE local_review_collections SET updated_at_ms = CASE WHEN updated_at_ms >= ?1 THEN updated_at_ms + 1 ELSE ?1 END WHERE id = ?2", params![now, collection_id])?;
        tx.commit()?;
        Ok(id)
    }

    pub(crate) fn local_review_git_status_diff_summary_evidence_preview(
        &self,
        item_id: &str,
        sha256: &str,
    ) -> Result<LocalReviewGitStatusDiffSummaryEvidencePreview, StorageError> {
        if !valid_task_id(item_id) || !valid_review_sha256(sha256) {
            return Err(StorageError::InvalidStoredValue);
        }
        let (title, provenance, evidence_source, content, stored_sha, byte_size, created_at_ms): (String, String, Option<String>, Vec<u8>, String, i64, i64) = self.connection.query_row("SELECT title, provenance, evidence_source, content, sha256, byte_size, created_at_ms FROM local_review_items WHERE id = ?1 AND class = 'evidence' AND state = 'ready'", [item_id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?,row.get(6)?)))?;
        if provenance != "git-status-diff-summary"
            || evidence_source.as_deref() != Some("git-status-diff-summary")
            || sha256 != stored_sha
            || byte_size != content.len() as i64
            || stored_sha != review_digest(&content)
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let (summary, details) = parse_git_status_diff_summary_evidence_envelope(&content, &title)?;
        Ok(LocalReviewGitStatusDiffSummaryEvidencePreview {
            schema_version: 1,
            item_id: item_id.to_owned(),
            source: LocalReviewEvidenceSource::GitStatusDiffSummary,
            title,
            summary,
            details,
            byte_size: byte_size as u64,
            sha256: stored_sha,
            created_at_ms,
        })
    }

    fn package_manifest_summary_source_for_local_review(
        &self,
        collection_id: &str,
    ) -> Result<PackageValidationSummary, StorageError> {
        if !valid_task_id(collection_id) {
            return Err(StorageError::InvalidStoredValue);
        }
        let project_id: String = self.connection.query_row(
            "SELECT task.project_id FROM local_review_collections AS collection
             JOIN task_records AS task ON task.id = collection.task_id
             JOIN projects AS project ON project.id = task.project_id
             JOIN directory_associations AS association ON association.id = project.active_directory_association_id
             WHERE collection.id = ?1 AND collection.discarded_at_ms IS NULL
               AND collection.state = 'active' AND task.archived_at_ms IS NULL
               AND task.status IN ('active', 'paused') AND task.project_id IS NOT NULL
               AND project.archived_at_ms IS NULL AND association.detached_at_ms IS NULL",
            [collection_id], |row| row.get(0),
        ).map_err(|_| StorageError::ProjectNotFound)?;
        let ids = self
            .connection
            .prepare(
                "SELECT record.id FROM project_package_validation_summaries AS record
             JOIN project_package_validation_candidate_identities AS identity
               ON identity.package_validation_summary_id = record.id
             WHERE record.project_id = ?1 AND record.validation_complete = 1
               AND identity.validation_phase = 'installed-host'
             ORDER BY record.created_at_ms DESC, record.id DESC LIMIT 2",
            )?
            .query_map([&project_id], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        let Some(id) = ids.first() else {
            return Err(StorageError::InvalidStoredValue);
        };
        let record = package_validation_summary_record(&self.connection, id)?;
        if record.project_id != project_id
            || record.input.validation_phase != PackageValidationPhase::InstalledHost
            || !record.input.validation_complete
            || record.input.artifact_count != PACKAGE_ARTIFACT_COUNT_LIMIT
            || !validation_is_complete(&record.input)
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let candidate = &record.input.candidate_identity_sha256;
        let attempt = record
            .input
            .attempt_identity_sha256
            .as_deref()
            .ok_or(StorageError::InvalidStoredValue)?;
        if installed_host_attempt_identity(
            candidate,
            record.input.installed_host_state,
            record.input.installed_host_facts.as_ref(),
        )? != attempt
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let predecessor_id = record
            .input
            .supersedes_record_id
            .as_deref()
            .ok_or(StorageError::InvalidStoredValue)?;
        let predecessor = package_validation_summary_record(&self.connection, predecessor_id)?;
        let phase: String = self.connection.query_row(
            "SELECT validation_phase FROM project_package_validation_candidate_identities
             WHERE package_validation_summary_id = ?1 AND project_id = ?2
               AND candidate_identity_sha256 = ?3",
            params![predecessor_id, project_id, candidate],
            |row| row.get(0),
        )?;
        if predecessor.project_id != project_id
            || package_validation_phase(&phase)? != PackageValidationPhase::Unprivileged
            || predecessor.input.validation_complete
            || predecessor.input.installed_host_state != LocalReviewEvidenceCheckState::Unavailable
        {
            return Err(StorageError::InvalidStoredValue);
        }
        Ok(record)
    }

    pub(crate) fn create_local_review_package_manifest_summary_evidence_item(
        &mut self,
        collection_id: &str,
        expected_updated_at_ms: i64,
    ) -> Result<String, StorageError> {
        let source = self.package_manifest_summary_source_for_local_review(collection_id)?;
        let title = "Package validation summary";
        let summary = "Captured completed package-validation summary.";
        let details = LocalReviewPackageManifestSummaryDetails {
            application_version: source.input.application_version,
            debian_version: source.input.debian_version,
            manifest_state: source.input.manifest_state,
            checksum_state: source.input.checksum_state,
            abi_state: source.input.abi_state,
            provenance_state: source.input.provenance_state,
            visible_launch_state: source.input.visible_launch_state,
            installed_host_state: source.input.installed_host_state,
            artifact_count: source.input.artifact_count.into(),
            validation_complete: source.input.validation_complete,
        };
        let bytes = package_manifest_summary_evidence_envelope_bytes(title, summary, &details)?;
        if bytes.len() > REVIEW_EVIDENCE_BYTES_LIMIT {
            return Err(StorageError::InvalidStoredValue);
        }
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        local_review_mutation_context(
            &tx,
            collection_id,
            Some(expected_updated_at_ms),
            LocalReviewMutationPermission::ActiveContent,
        )?;
        // Repeat resolution inside the write transaction so stale lifecycle or record state fails closed.
        drop(tx);
        let source = self.package_manifest_summary_source_for_local_review(collection_id)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        local_review_mutation_context(
            &tx,
            collection_id,
            Some(expected_updated_at_ms),
            LocalReviewMutationPermission::ActiveContent,
        )?;
        if source.input.validation_phase != PackageValidationPhase::InstalledHost {
            return Err(StorageError::InvalidStoredValue);
        }
        let count: i64 = tx.query_row("SELECT count(*) FROM local_review_items WHERE collection_id = ?1 AND class = 'evidence'", [collection_id], |row| row.get(0))?;
        if count >= 6 {
            return Err(StorageError::TaskCapacity);
        }
        let added = bytes.len() as i64 + title.len() as i64 + 256;
        if review_payload_bytes(&tx, Some(collection_id))? + added > REVIEW_COLLECTION_PAYLOAD_LIMIT
            || review_payload_bytes(&tx, None)? + added > REVIEW_PAYLOAD_LIMIT
        {
            return Err(StorageError::TaskCapacity);
        }
        let id = Uuid::now_v7().to_string();
        tx.execute("INSERT INTO local_review_items (id, schema_version, collection_id, class, text_format, mime_type, width, height, state, title, source_kind, provenance, evidence_source, content, sha256, byte_size, created_at_ms, updated_at_ms) VALUES (?1, 1, ?2, 'evidence', NULL, 'application/json; profile=evidence-envelope-v1', NULL, NULL, 'ready', ?3, 'typed-evidence-snapshot', 'package-manifest-summary', 'package-manifest-summary', ?4, ?5, ?6, ?7, ?7)", params![id, collection_id, title, bytes, review_digest(&bytes), bytes.len() as i64, now])?;
        tx.execute("UPDATE local_review_collections SET updated_at_ms = CASE WHEN updated_at_ms >= ?1 THEN updated_at_ms + 1 ELSE ?1 END WHERE id = ?2", params![now, collection_id])?;
        tx.commit()?;
        Ok(id)
    }

    pub(crate) fn local_review_package_manifest_summary_evidence_preview(
        &self,
        item_id: &str,
        sha256: &str,
    ) -> Result<LocalReviewPackageManifestSummaryEvidencePreview, StorageError> {
        if !valid_task_id(item_id) || !valid_review_sha256(sha256) {
            return Err(StorageError::InvalidStoredValue);
        }
        let (title, source_kind, provenance, evidence_source, content, stored_sha, byte_size, created_at_ms): (String, String, String, Option<String>, Vec<u8>, String, i64, i64) = self.connection.query_row("SELECT title, source_kind, provenance, evidence_source, content, sha256, byte_size, created_at_ms FROM local_review_items WHERE id = ?1 AND class = 'evidence' AND state = 'ready'", [item_id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?,row.get(6)?,row.get(7)?)))?;
        if source_kind != "typed-evidence-snapshot"
            || provenance != "package-manifest-summary"
            || evidence_source.as_deref() != Some("package-manifest-summary")
            || sha256 != stored_sha
            || byte_size != content.len() as i64
            || content.len() > REVIEW_EVIDENCE_BYTES_LIMIT
            || stored_sha != review_digest(&content)
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let (summary, details) =
            parse_package_manifest_summary_evidence_envelope(&content, &title)?;
        Ok(LocalReviewPackageManifestSummaryEvidencePreview {
            schema_version: 1,
            item_id: item_id.to_owned(),
            source: LocalReviewEvidenceSource::PackageManifestSummary,
            title,
            summary,
            details,
            byte_size: byte_size as u64,
            sha256: stored_sha,
            created_at_ms,
        })
    }

    #[expect(
        clippy::type_complexity,
        reason = "The private preview query is immediately destructured into the fixed path-free projection."
    )]
    pub(crate) fn local_review_text_preview(
        &self,
        collection_id: &str,
        item_id: &str,
        sha256: &str,
    ) -> Result<LocalReviewTextPreview, StorageError> {
        if !valid_task_id(collection_id) || !valid_task_id(item_id) || !valid_review_sha256(sha256)
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let row: Option<(
            String,
            String,
            Option<String>,
            String,
            String,
            String,
            Option<i64>,
            Option<i64>,
            Vec<u8>,
            String,
            i64,
            i64,
        )> = self
            .connection
            .query_row(
                "SELECT collection_id, class, text_format, state, title, source_kind, width, height, content, sha256, byte_size, created_at_ms FROM local_review_items WHERE id = ?1 AND collection_id = ?2 AND discarded_at_ms IS NULL",
                params![item_id, collection_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?, row.get(9)?, row.get(10)?, row.get(11)?)),
            )
            .optional()?;
        let Some((
            stored_collection_id,
            class,
            text_format,
            state,
            title,
            source_kind,
            width,
            height,
            content,
            stored_sha,
            byte_size,
            created_at_ms,
        )) = row
        else {
            return Ok(unavailable_review_text_preview(
                collection_id,
                item_id,
                LocalReviewDiagnosticCode::ItemNotFound,
            ));
        };
        if stored_collection_id != collection_id || state == "discarded" {
            return Ok(unavailable_review_text_preview(
                collection_id,
                item_id,
                LocalReviewDiagnosticCode::ItemNotFound,
            ));
        }
        let collection_state: Option<String> = self
            .connection
            .query_row(
                "SELECT state FROM local_review_collections WHERE id = ?1 AND discarded_at_ms IS NULL",
                [collection_id],
                |row| row.get(0),
            )
            .optional()?;
        if !matches!(
            collection_state.as_deref(),
            Some("active" | "frozen" | "orphaned")
        ) {
            return Ok(unavailable_review_text_preview(
                collection_id,
                item_id,
                LocalReviewDiagnosticCode::CollectionNotFound,
            ));
        }
        let format = match text_format
            .as_deref()
            .and_then(|value| review_text_format(value).ok())
        {
            Some(value) => value,
            None => {
                return Ok(unavailable_review_text_preview(
                    collection_id,
                    item_id,
                    LocalReviewDiagnosticCode::InvalidReference,
                ))
            }
        };
        if class != "text"
            || !matches!(
                source_kind.as_str(),
                "user-authored-text" | "m48-artifact-copy"
            )
            || width.is_some()
            || height.is_some()
            || !matches!(normalize_review_label(&title), Ok(ref normalized) if normalized == &title)
            || sha256 != stored_sha
            || !valid_review_sha256(&stored_sha)
            || byte_size < 1
            || created_at_ms < 0
            || byte_size as usize != content.len()
            || content.len() > REVIEW_TEXT_BYTES_LIMIT
            || stored_sha != review_digest(&content)
        {
            return Ok(unavailable_review_text_preview(
                collection_id,
                item_id,
                LocalReviewDiagnosticCode::IntegrityFailed,
            ));
        }
        let canonical_text = match std::str::from_utf8(&content) {
            Ok(value) if matches!(normalize_review_text(value, format), Ok(ref normalized) if normalized == value) => {
                value
            }
            _ => {
                return Ok(unavailable_review_text_preview(
                    collection_id,
                    item_id,
                    LocalReviewDiagnosticCode::IntegrityFailed,
                ))
            }
        };
        if state == "stale" {
            return Ok(LocalReviewTextPreview {
                schema_version: 1,
                collection_id: collection_id.to_owned(),
                item_id: item_id.to_owned(),
                title: Some(title),
                text_format: Some(format),
                byte_size: Some(byte_size as u64),
                sha256: Some(stored_sha),
                created_at_ms: Some(created_at_ms),
                state: LocalReviewItemState::Stale,
                text: None,
                projected_byte_size: 0,
                projected_line_count: 0,
                projected_code_point_count: 0,
                truncated: false,
                diagnostic_code: None,
            });
        }
        if state != "ready" {
            return Ok(unavailable_review_text_preview(
                collection_id,
                item_id,
                LocalReviewDiagnosticCode::IntegrityFailed,
            ));
        }
        let (
            text,
            projected_byte_size,
            projected_line_count,
            projected_code_point_count,
            truncated,
        ) = bounded_review_text_preview(canonical_text);
        Ok(LocalReviewTextPreview {
            schema_version: 1,
            collection_id: collection_id.to_owned(),
            item_id: item_id.to_owned(),
            title: Some(title),
            text_format: Some(format),
            byte_size: Some(byte_size as u64),
            sha256: Some(stored_sha),
            created_at_ms: Some(created_at_ms),
            state: LocalReviewItemState::Ready,
            text: Some(text),
            projected_byte_size,
            projected_line_count,
            projected_code_point_count,
            truncated,
            diagnostic_code: None,
        })
    }

    pub(crate) fn local_review_image_preview(
        &self,
        item_id: &str,
        sha256: &str,
    ) -> Result<LocalReviewImagePreview, StorageError> {
        if !valid_task_id(item_id) || sha256.len() != 64 {
            return Err(StorageError::InvalidStoredValue);
        }
        let (mime_type, width, height, byte_size, stored_sha, bytes): (String, i64, i64, i64, String, Vec<u8>) = self.connection.query_row("SELECT mime_type, width, height, byte_size, sha256, content FROM local_review_items WHERE id = ?1 AND class = 'image-mockup' AND state = 'ready'", [item_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)))?;
        let image =
            validate_attachment_image(&bytes).map_err(|_| StorageError::InvalidStoredValue)?;
        if sha256 != stored_sha
            || stored_sha != review_digest(&bytes)
            || byte_size != bytes.len() as i64
            || mime_type != image.mime_type
            || width != image.width as i64
            || height != image.height as i64
            || bytes.len() > REVIEW_IMAGE_BYTES_LIMIT
        {
            return Err(StorageError::InvalidStoredValue);
        }
        Ok(LocalReviewImagePreview {
            schema_version: 1,
            item_id: item_id.to_owned(),
            mime_type,
            width: image.width,
            height: image.height,
            byte_size: byte_size as u64,
            sha256: stored_sha,
            data_url: format!("data:{};base64,{}", image.mime_type, BASE64.encode(bytes)),
        })
    }

    pub(crate) fn resume_local_review_collection(
        &mut self,
        collection_id: &str,
        expected_updated_at_ms: i64,
    ) -> Result<(), StorageError> {
        if !valid_task_id(collection_id) || expected_updated_at_ms < 0 {
            return Err(StorageError::InvalidStoredValue);
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        local_review_mutation_context(
            &tx,
            collection_id,
            Some(expected_updated_at_ms),
            LocalReviewMutationPermission::ExplicitResume,
        )?;
        tx.execute("UPDATE local_review_collections SET state = 'active', updated_at_ms = ?1 WHERE id = ?2", params![now_millis(), collection_id])?;
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn discard_local_review_collection(
        &mut self,
        collection_id: &str,
        expected_updated_at_ms: i64,
    ) -> Result<(), StorageError> {
        if !valid_task_id(collection_id) || expected_updated_at_ms < 0 {
            return Err(StorageError::InvalidStoredValue);
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        local_review_mutation_context(
            &tx,
            collection_id,
            Some(expected_updated_at_ms),
            LocalReviewMutationPermission::RecoveryDiscard,
        )?;
        tx.execute(
            "DELETE FROM local_review_collections WHERE id = ?1",
            [collection_id],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn discard_local_review_item(
        &mut self,
        collection_id: &str,
        item_id: &str,
        expected_updated_at_ms: i64,
    ) -> Result<(), StorageError> {
        if !valid_task_id(collection_id) || !valid_task_id(item_id) || expected_updated_at_ms < 0 {
            return Err(StorageError::InvalidStoredValue);
        }
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        local_review_mutation_context(
            &tx,
            collection_id,
            Some(expected_updated_at_ms),
            LocalReviewMutationPermission::RecoveryDiscard,
        )?;
        if tx.execute(
            "DELETE FROM local_review_items WHERE id = ?1 AND collection_id = ?2",
            params![item_id, collection_id],
        )? != 1
        {
            return Err(StorageError::TaskNotFound);
        }
        tx.execute(
            "UPDATE local_review_collections SET updated_at_ms = ?1 WHERE id = ?2",
            params![now, collection_id],
        )?;
        tx.commit()?;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn create_task(&mut self) -> Result<String, StorageError> {
        let now = now_millis();
        self.create_task_with(now, || Uuid::now_v7().to_string())
    }

    /// Creates a normal Task Catalog record for one currently attached,
    /// non-archived project. Project binding is resolved and validated inside
    /// the same immediate transaction as the durable insert.
    pub(crate) fn create_task_for_project(
        &mut self,
        project_id: &str,
        title: &str,
    ) -> Result<String, StorageError> {
        if !is_uuid_v7(project_id) {
            return Err(StorageError::InvalidStoredValue);
        }
        let title = normalize_task_text(title, 120, 480)?;
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let project_id = task_catalog_project_id(&tx, project_id)?;
        let task_id = Self::create_task_in_transaction(
            &tx,
            now,
            Some(project_id.as_str()),
            None,
            &mut || Uuid::now_v7().to_string(),
        )?;
        tx.execute(
            "UPDATE task_records SET title = ?1 WHERE id = ?2",
            params![title, task_id],
        )?;
        tx.commit()?;
        Ok(task_id)
    }

    #[cfg(test)]
    fn create_task_with(
        &mut self,
        now: i64,
        mut generate: impl FnMut() -> String,
    ) -> Result<String, StorageError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let task_id = Self::create_task_in_transaction(&tx, now, None, None, &mut generate)?;
        tx.commit()?;
        Ok(task_id)
    }

    pub(crate) fn create_task_from_conversation_context(
        &mut self,
        conversation_id: &str,
    ) -> Result<String, StorageError> {
        if !is_uuid_v7(conversation_id) {
            return Err(StorageError::InvalidStoredValue);
        }
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let project_id = task_context_project_id(&tx, conversation_id)?;
        let origin = task_advisor_dispatch_origin(&tx, conversation_id, &project_id)?;
        let task_id = Self::create_task_in_transaction(
            &tx,
            now,
            Some(project_id.as_str()),
            origin.as_ref().map(|(conversation_id, dispatch_id)| {
                (conversation_id.as_str(), dispatch_id.as_str())
            }),
            &mut || Uuid::now_v7().to_string(),
        )?;
        tx.commit()?;
        Ok(task_id)
    }

    pub(crate) fn task_project_binding(
        &self,
        task_id: &str,
    ) -> Result<Option<String>, StorageError> {
        self.connection
            .query_row(
                "SELECT project_id FROM task_records WHERE id = ?1",
                [task_id],
                |row| row.get(0),
            )
            .optional()?
            .ok_or(StorageError::TaskNotFound)
    }

    pub(crate) fn durable_source_insert(
        &mut self,
        source: DurableSourceInsert<'_>,
    ) -> Result<DurableSourceSummary, StorageError> {
        if !valid_task_id(source.id)
            || !valid_task_id(source.project_id)
            || source.task_id.is_some_and(|id| !valid_task_id(id))
            || source.sha256.len() != 64
            || !source
                .sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            || source.byte_size > 128 * 1024
            || source.line_count > 2_000
            || source.title.chars().count() > 240
            || source.title.trim().is_empty()
            || source.origin_display.is_some_and(|value| {
                value.contains('/') || value.contains('\\') || value.len() > 255
            })
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let project_exists: Option<String> = tx
            .query_row(
                "SELECT id FROM projects WHERE id = ?1 AND archived_at_ms IS NULL",
                [source.project_id],
                |row| row.get(0),
            )
            .optional()?;
        if project_exists.is_none() {
            return Err(StorageError::ProjectNotFound);
        }
        if let Some(task_id) = source.task_id {
            let binding: Option<String> = tx
                .query_row(
                    "SELECT project_id FROM task_records WHERE id = ?1 AND archived_at_ms IS NULL",
                    [task_id],
                    |row| row.get(0),
                )
                .optional()?;
            if binding.as_deref() != Some(source.project_id) {
                return Err(StorageError::TaskNotFound);
            }
        }
        let class = durable_source_class_name(source.source_class);
        tx.execute(
            "INSERT INTO durable_sources (id, schema_version, project_id, task_id, source_class, title, origin_display, byte_size, line_count, sha256, content_locator, state, created_at_ms, updated_at_ms, deleted_at_ms) VALUES (?1, 1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?1, 'active', ?10, ?10, NULL)",
            params![source.id, source.project_id, source.task_id, class, source.title, source.origin_display, source.byte_size as i64, source.line_count as i64, source.sha256, now]
        )?;
        tx.commit()?;
        Ok(DurableSourceSummary {
            source_id: source.id.to_owned(),
            project_id: source.project_id.to_owned(),
            task_id: source.task_id.map(str::to_owned),
            source_class: source.source_class,
            title: source.title.to_owned(),
            origin_display: source.origin_display.map(str::to_owned),
            byte_size: source.byte_size,
            line_count: source.line_count,
            sha256: source.sha256.to_owned(),
            state: DurableSourceLifecycleState::Active,
            created_at_ms: now,
        })
    }

    pub(crate) fn durable_sources_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<DurableSourceSummary>, StorageError> {
        if !valid_task_id(project_id) {
            return Err(StorageError::InvalidStoredValue);
        }
        let mut statement = self.connection.prepare("SELECT id, project_id, task_id, source_class, title, origin_display, byte_size, line_count, sha256, state, created_at_ms FROM durable_sources WHERE project_id = ?1 AND state = 'active' ORDER BY created_at_ms DESC, id DESC LIMIT 100")?;
        let rows = statement
            .query_map([project_id], durable_source_from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub(crate) fn artifact_references_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<ArtifactReferenceSummary>, StorageError> {
        if !valid_task_id(project_id) {
            return Err(StorageError::InvalidStoredValue);
        }
        let mut statement = self.connection.prepare("SELECT id, project_id, task_id, artifact_id, artifact_sha256, artifact_class, display_label, state, created_at_ms FROM artifact_references WHERE project_id = ?1 AND state = 'active' ORDER BY created_at_ms DESC, id DESC LIMIT 100")?;
        let rows = statement
            .query_map([project_id], artifact_reference_from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub(crate) fn artifact_reference(
        &self,
        reference_id: &str,
    ) -> Result<Option<ArtifactReferenceSummary>, StorageError> {
        if !valid_task_id(reference_id) {
            return Err(StorageError::InvalidStoredValue);
        }
        Ok(self.connection.query_row(
            "SELECT id, project_id, task_id, artifact_id, artifact_sha256, artifact_class, display_label, state, created_at_ms FROM artifact_references WHERE id = ?1",
            [reference_id], artifact_reference_from_row,
        ).optional()?)
    }

    pub(crate) fn artifact_reference_insert(
        &mut self,
        source: ArtifactReferenceInsert<'_>,
    ) -> Result<ArtifactReferenceSummary, StorageError> {
        if !valid_task_id(source.id)
            || !valid_task_id(source.project_id)
            || source.task_id.is_some_and(|value| !valid_task_id(value))
            || !valid_task_id(source.artifact_id)
            || source.artifact_sha256.len() != 64
            || !matches!(
                source.artifact_class,
                "text" | "markdown" | "json" | "csv" | "python"
            )
            || source.display_label.is_empty()
            || source.display_label.chars().count() > 120
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let created_at_ms = now_millis();
        self.connection.execute(
            "INSERT INTO artifact_references (id, schema_version, project_id, task_id, artifact_id, artifact_sha256, artifact_class, display_label, state, created_at_ms, deleted_at_ms) VALUES (?1, 1, ?2, ?3, ?4, ?5, ?6, ?7, 'active', ?8, NULL)",
            params![source.id, source.project_id, source.task_id, source.artifact_id, source.artifact_sha256, source.artifact_class, source.display_label, created_at_ms],
        )?;
        Ok(ArtifactReferenceSummary {
            reference_id: source.id.into(),
            project_id: source.project_id.into(),
            task_id: source.task_id.map(str::to_owned),
            artifact_id: source.artifact_id.into(),
            artifact_sha256: source.artifact_sha256.into(),
            artifact_class: source.artifact_class.into(),
            display_label: source.display_label.into(),
            state: ArtifactReferenceState::Active,
            availability: super::types::ArtifactReferenceAvailability::Unavailable,
            created_at_ms,
        })
    }

    pub(crate) fn artifact_reference_delete(
        &mut self,
        reference_id: &str,
    ) -> Result<(), StorageError> {
        if !valid_task_id(reference_id) {
            return Err(StorageError::InvalidStoredValue);
        }
        let changed = self.connection.execute("UPDATE artifact_references SET state = 'deleted', deleted_at_ms = ?2 WHERE id = ?1 AND state = 'active'", params![reference_id, now_millis()])?;
        if changed == 1 {
            Ok(())
        } else {
            Err(StorageError::InvalidStoredValue)
        }
    }

    pub(crate) fn durable_source(
        &self,
        source_id: &str,
    ) -> Result<Option<DurableSourceSummary>, StorageError> {
        if !valid_task_id(source_id) {
            return Err(StorageError::InvalidStoredValue);
        }
        Ok(self.connection.query_row("SELECT id, project_id, task_id, source_class, title, origin_display, byte_size, line_count, sha256, state, created_at_ms FROM durable_sources WHERE id = ?1", [source_id], durable_source_from_row).optional()?)
    }

    pub(crate) fn durable_source_delete(
        &mut self,
        source_id: &str,
    ) -> Result<DurableSourceSummary, StorageError> {
        let mut source = self
            .durable_source(source_id)?
            .ok_or(StorageError::TaskNotFound)?;
        if source.state == DurableSourceLifecycleState::Deleted {
            return Err(StorageError::InvalidStoredValue);
        }
        let now = now_millis();
        if self.connection.execute("UPDATE durable_sources SET state = 'deleted', deleted_at_ms = ?1, updated_at_ms = ?1 WHERE id = ?2 AND state = 'active'", params![now, source_id])? != 1 { return Err(StorageError::InvalidStoredValue); }
        source.state = DurableSourceLifecycleState::Deleted;
        Ok(source)
    }

    /// Returns only the durable project/task identity required by the local
    /// mock-inference boundary. It intentionally projects no task text, plan,
    /// artifact, or other project content.
    pub(crate) fn task_mock_inference_binding(
        &self,
        task_id: &str,
    ) -> Result<Option<String>, StorageError> {
        self.connection
            .query_row(
                "SELECT project_id FROM task_records WHERE id = ?1 AND archived_at_ms IS NULL",
                [task_id],
                |row| row.get(0),
            )
            .optional()?
            .ok_or(StorageError::TaskNotFound)
    }

    pub(crate) fn record_fictional_connector_operation(
        &mut self,
        record: &FictionalConnectorOperationRecord<'_>,
    ) -> Result<(), StorageError> {
        let bound_project: String = self.connection.query_row(
            "SELECT project_id FROM task_records WHERE id = ?1 AND archived_at_ms IS NULL",
            [record.task_id],
            |row| row.get(0),
        )?;
        if bound_project != record.project_id {
            return Err(StorageError::TaskNotFound);
        }
        if !is_uuid_v7(record.binding_id)
            || !is_uuid_v7(record.operation_id)
            || record
                .authorization_id
                .is_some_and(|value| !is_uuid_v7(value))
            || !matches!(record.operation_class, "read" | "mutation")
            || !matches!(record.state, "prepared" | "completed")
            || record.descriptor_id != "019a57c0-0000-7000-8000-000000000001"
            || record.descriptor_version != 1
            || !valid_review_sha256(record.descriptor_sha256)
            || !valid_review_sha256(record.scope_digest)
            || !valid_review_sha256(record.request_digest)
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let now = now_millis();
        if record.expires_at_ms < now {
            return Err(StorageError::InvalidStoredValue);
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute("INSERT INTO fictional_connector_bindings (id, schema_version, project_id, task_id, descriptor_id, descriptor_version, descriptor_sha256, scope_digest, state, created_at_ms, updated_at_ms) VALUES (?1,1,?2,?3,?4,?5,?6,?7,'ready',?8,?8)", params![record.binding_id, record.project_id, record.task_id, record.descriptor_id, record.descriptor_version, record.descriptor_sha256, record.scope_digest, now])?;
        tx.execute("INSERT INTO fictional_connector_operations (id, schema_version, binding_id, project_id, task_id, operation_class, request_digest, authorization_id, state, created_at_ms, expires_at_ms, completed_at_ms) VALUES (?1,1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)", params![record.operation_id, record.binding_id, record.project_id, record.task_id, record.operation_class, record.request_digest, record.authorization_id, record.state, now, record.expires_at_ms, if record.state == "completed" { Some(now) } else { None }])?;
        tx.execute("INSERT INTO fictional_connector_audit (id, operation_id, binding_id, event_kind, outcome, evidence_digest, created_at_ms) VALUES (?1,?2,?3,'prepared-or-read',?4,?5,?6)", params![Uuid::now_v7().to_string(), record.operation_id, record.binding_id, record.state, record.request_digest, now])?;
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn record_controlled_browser_verification(
        &mut self,
        record: &ControlledBrowserVerificationRecord<'_>,
    ) -> Result<(), StorageError> {
        let exists: bool = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?1 AND archived_at_ms IS NULL)",
            [record.project_id],
            |row| row.get(0),
        )?;
        if !exists
            || !is_uuid_v7(record.attempt_id)
            || !is_uuid_v7(record.authorization_id)
            || !valid_review_sha256(record.target_digest)
            || !valid_review_sha256(record.request_digest)
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute(
            "INSERT INTO controlled_browser_verification_attempts (id,schema_version,project_id,task_id,fixture_id,target_digest,request_digest,authorization_id,state,expires_at_ms,created_at_ms,completed_at_ms,evidence_digest) VALUES (?1,1,?2,?3,'fictional-webkitgtk-local-v1',?4,?5,?6,'prepared',?7,?8,NULL,NULL)",
            params![record.attempt_id, record.project_id, record.task_id, record.target_digest, record.request_digest, record.authorization_id, record.expires_at_ms, now],
        )?;
        tx.execute(
            "INSERT INTO controlled_browser_verification_audit (id,attempt_id,event_kind,outcome,evidence_digest,created_at_ms) VALUES (?1,?2,'prepared','prepared',?3,?4)",
            params![Uuid::now_v7().to_string(), record.attempt_id, record.request_digest, now],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn complete_controlled_browser_verification(
        &mut self,
        attempt_id: &str,
        state: &str,
        evidence_digest: Option<&str>,
    ) -> Result<(), StorageError> {
        let now = now_millis();
        let digest = evidence_digest
            .unwrap_or("0000000000000000000000000000000000000000000000000000000000000000");
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if tx.execute(
            "UPDATE controlled_browser_verification_attempts SET state = ?1, completed_at_ms = ?2, evidence_digest = ?3 WHERE id = ?4 AND state = 'prepared'",
            params![state, now, evidence_digest, attempt_id],
        )? != 1 {
            return Err(StorageError::InvalidStoredValue);
        }
        tx.execute(
            "INSERT INTO controlled_browser_verification_audit (id,attempt_id,event_kind,outcome,evidence_digest,created_at_ms) VALUES (?1,?2,'terminal',?3,?4,?5)",
            params![Uuid::now_v7().to_string(), attempt_id, state, digest, now],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn record_context_bundle(
        &mut self,
        record: &crate::project::ContextBundleRecord<'_>,
    ) -> Result<(), StorageError> {
        if !is_uuid_v7(record.bundle_id)
            || !is_uuid_v7(record.authorization_id)
            || !valid_review_sha256(record.bundle_digest)
            || record.items.len() > 16
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM projects WHERE id=?1 AND archived_at_ms IS NULL)",
            [record.project_id],
            |row| row.get(0),
        )?;
        if !exists {
            return Err(StorageError::ProjectNotFound);
        }
        if let Some(task_id) = record.task_id {
            let project: Option<String> = tx
                .query_row(
                    "SELECT project_id FROM task_records WHERE id=?1 AND archived_at_ms IS NULL",
                    [task_id],
                    |row| row.get(0),
                )
                .optional()?;
            if project.as_deref() != Some(record.project_id) {
                return Err(StorageError::TaskNotFound);
            }
        }
        if record.canonical_bytes.len() > 96 * 1024
            || crate::context_assembly::digest(record.canonical_bytes) != record.bundle_digest
        {
            return Err(StorageError::InvalidStoredValue);
        }
        tx.execute("INSERT INTO context_bundles (id,schema_version,project_id,task_id,bundle_digest,canonical_bytes,policy_version,assembly_version,state,expires_at_ms,created_at_ms,completed_at_ms,authorization_id) VALUES (?1,1,?2,?3,?4,?5,1,1,'prepared',?6,?7,NULL,?8)",params![record.bundle_id,record.project_id,record.task_id,record.bundle_digest,record.canonical_bytes,record.expires_at_ms,now,record.authorization_id])?;
        for (ordinal, item) in record.items.iter().enumerate() {
            tx.execute("INSERT INTO context_bundle_items (id,bundle_id,ordinal,source_ref,source_class,provenance,content_digest,byte_size,redaction_count,truncated) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",params![Uuid::now_v7().to_string(),record.bundle_id,ordinal as i64,item.source_ref,item.source_class,item.provenance,item.digest,item.byte_size as i64,item.redaction_count as i64,item.truncated as i64])?;
        }
        tx.execute("INSERT INTO context_bundle_audit (id,bundle_id,event_kind,outcome,evidence_digest,created_at_ms) VALUES (?1,?2,'prepared','prepared',?3,?4)",params![Uuid::now_v7().to_string(),record.bundle_id,record.bundle_digest,now])?;
        tx.commit()?;
        Ok(())
    }
    pub(crate) fn complete_context_bundle(
        &mut self,
        bundle_id: &str,
        state: &str,
    ) -> Result<(), StorageError> {
        if !matches!(
            state,
            "accepted_delivery"
                | "cancelled"
                | "revoked"
                | "expired"
                | "ambiguous"
                | "timed_out"
                | "failed"
                | "rejected_delivery"
                | "closed"
        ) {
            return Err(StorageError::InvalidStoredValue);
        }
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let changed=tx.execute("UPDATE context_bundles SET state=?1, canonical_bytes=NULL, completed_at_ms=?2 WHERE id=?3 AND state IN ('prepared','awaiting_review','awaiting_confirmation','dispatching')",params![state,now,bundle_id])?;
        if changed != 1 {
            return Err(StorageError::InvalidStoredValue);
        };
        let digest: String = tx.query_row(
            "SELECT bundle_digest FROM context_bundles WHERE id=?1",
            [bundle_id],
            |row| row.get(0),
        )?;
        if matches!(
            state,
            "accepted_delivery" | "rejected_delivery" | "timed_out" | "ambiguous"
        ) {
            tx.execute("INSERT INTO context_bundle_audit (id,bundle_id,event_kind,outcome,evidence_digest,created_at_ms) VALUES (?1,?2,'authorized','authorized',?3,?4)",params![Uuid::now_v7().to_string(),bundle_id,digest,now])?;
            tx.execute("INSERT INTO context_bundle_audit (id,bundle_id,event_kind,outcome,evidence_digest,created_at_ms) VALUES (?1,?2,'dispatching','dispatching',?3,?4)",params![Uuid::now_v7().to_string(),bundle_id,digest,now])?;
        }
        tx.execute("INSERT INTO context_bundle_audit (id,bundle_id,event_kind,outcome,evidence_digest,created_at_ms) VALUES (?1,?2,'terminal',?3,?4,?5)",params![Uuid::now_v7().to_string(),bundle_id,state,digest,now])?;
        tx.commit()?;
        Ok(())
    }
    /// Consumes a reviewed bundle before the M63 in-process call begins.  The
    /// durable record never retains its private bytes while that call runs.
    pub(crate) fn start_local_runtime_context_bundle(
        &mut self,
        bundle_id: &str,
    ) -> Result<(), StorageError> {
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let changed = tx.execute(
            "UPDATE context_bundles SET state='dispatching', canonical_bytes=NULL WHERE id=?1 AND state='awaiting_confirmation'",
            [bundle_id],
        )?;
        if changed != 1 {
            return Err(StorageError::InvalidStoredValue);
        }
        let digest: String = tx.query_row(
            "SELECT bundle_digest FROM context_bundles WHERE id=?1",
            [bundle_id],
            |row| row.get(0),
        )?;
        tx.execute("INSERT INTO context_bundle_audit (id,bundle_id,event_kind,outcome,evidence_digest,created_at_ms) VALUES (?1,?2,'authorized','authorized',?3,?4)",params![Uuid::now_v7().to_string(),bundle_id,digest,now])?;
        tx.execute("INSERT INTO context_bundle_audit (id,bundle_id,event_kind,outcome,evidence_digest,created_at_ms) VALUES (?1,?2,'dispatching','local-runtime-running',?3,?4)",params![Uuid::now_v7().to_string(),bundle_id,digest,now])?;
        tx.commit()?;
        Ok(())
    }
    pub(crate) fn review_context_bundle(&mut self, bundle_id: &str) -> Result<(), StorageError> {
        let now = now_millis();
        let changed = self.connection.execute(
            "UPDATE context_bundles SET state='awaiting_review' WHERE id=?1 AND state='prepared'",
            [bundle_id],
        )?;
        if changed != 1 {
            return Err(StorageError::InvalidStoredValue);
        }
        let digest: String = self.connection.query_row(
            "SELECT bundle_digest FROM context_bundles WHERE id=?1",
            [bundle_id],
            |row| row.get(0),
        )?;
        self.connection.execute("INSERT INTO context_bundle_audit (id,bundle_id,event_kind,outcome,evidence_digest,created_at_ms) VALUES (?1,?2,'review','awaiting_review',?3,?4)", params![Uuid::now_v7().to_string(),bundle_id,digest,now])?;
        Ok(())
    }
    pub(crate) fn acknowledge_context_bundle_review(
        &mut self,
        bundle_id: &str,
    ) -> Result<(), StorageError> {
        let now = now_millis();
        let changed = self.connection.execute("UPDATE context_bundles SET state='awaiting_confirmation' WHERE id=?1 AND state='awaiting_review'", [bundle_id])?;
        if changed != 1 {
            return Err(StorageError::InvalidStoredValue);
        }
        let digest: String = self.connection.query_row(
            "SELECT bundle_digest FROM context_bundles WHERE id=?1",
            [bundle_id],
            |row| row.get(0),
        )?;
        self.connection.execute("INSERT INTO context_bundle_audit (id,bundle_id,event_kind,outcome,evidence_digest,created_at_ms) VALUES (?1,?2,'review-acknowledged','awaiting_confirmation',?3,?4)", params![Uuid::now_v7().to_string(),bundle_id,digest,now])?;
        Ok(())
    }

    pub(crate) fn complete_fictional_connector_operation(
        &mut self,
        project_id: &str,
        task_id: &str,
        operation_id: &str,
        state: &str,
        evidence_digest: &str,
    ) -> Result<(), StorageError> {
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let binding_id: String = tx.query_row(
            "SELECT binding_id FROM fictional_connector_operations WHERE id = ?1 AND project_id = ?2 AND task_id = ?3 AND state = 'prepared'",
            params![operation_id, project_id, task_id], |row| row.get(0),
        )?;
        if tx.execute("UPDATE fictional_connector_operations SET state = ?1, completed_at_ms = ?2 WHERE id = ?3 AND state = 'prepared'", params![state, now, operation_id])? != 1 { return Err(StorageError::InvalidStoredValue); }
        tx.execute("INSERT INTO fictional_connector_audit (id, operation_id, binding_id, event_kind, outcome, evidence_digest, created_at_ms) VALUES (?1,?2,?3,'fictional-local-terminal',?4,?5,?6)", params![Uuid::now_v7().to_string(), operation_id, binding_id, state, evidence_digest, now])?;
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn transition_fictional_connector_binding(
        &mut self,
        project_id: &str,
        task_id: &str,
        operation_id: &str,
        binding_state: &str,
        operation_state: &str,
        evidence_digest: &str,
    ) -> Result<(), StorageError> {
        if !matches!(
            binding_state,
            "revoked" | "quarantined" | "incompatible" | "expired"
        ) || !matches!(operation_state, "revoked" | "rejected" | "expired")
            || !valid_review_sha256(evidence_digest)
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let binding_id: String = tx.query_row(
            "SELECT binding_id FROM fictional_connector_operations WHERE id = ?1 AND project_id = ?2 AND task_id = ?3 AND state = 'prepared'",
            params![operation_id, project_id, task_id], |row| row.get(0),
        )?;
        if tx.execute("UPDATE fictional_connector_bindings SET state = ?1, updated_at_ms = ?2 WHERE id = ?3 AND state = 'ready'", params![binding_state, now, binding_id])? != 1
            || tx.execute("UPDATE fictional_connector_operations SET state = ?1, completed_at_ms = ?2 WHERE id = ?3 AND state = 'prepared'", params![operation_state, now, operation_id])? != 1
        { return Err(StorageError::InvalidStoredValue); }
        tx.execute("INSERT INTO fictional_connector_audit (id, operation_id, binding_id, event_kind, outcome, evidence_digest, created_at_ms) VALUES (?1,?2,?3,'binding-invalidated',?4,?5,?6)", params![Uuid::now_v7().to_string(), operation_id, binding_id, binding_state, evidence_digest, now])?;
        tx.commit()?;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn task_advisor_dispatch_origin(
        &self,
        task_id: &str,
    ) -> Result<Option<(String, String)>, StorageError> {
        self.connection
            .query_row(
                "SELECT origin_advisor_conversation_id, origin_advisor_dispatch_record_id
                 FROM task_records WHERE id = ?1",
                [task_id],
                |row| {
                    Ok((
                        row.get::<_, Option<String>>(0)?,
                        row.get::<_, Option<String>>(1)?,
                    ))
                },
            )
            .optional()?
            .ok_or(StorageError::TaskNotFound)
            .and_then(
                |(conversation_id, dispatch_id)| match (conversation_id, dispatch_id) {
                    (None, None) => Ok(None),
                    (Some(conversation_id), Some(dispatch_id)) => {
                        Ok(Some((conversation_id, dispatch_id)))
                    }
                    _ => Err(StorageError::InvalidStoredValue),
                },
            )
    }

    fn create_task_in_transaction(
        tx: &Transaction<'_>,
        now: i64,
        project_id: Option<&str>,
        advisor_dispatch_origin: Option<(&str, &str)>,
        mut generate: &mut impl FnMut() -> String,
    ) -> Result<String, StorageError> {
        let count: i64 = tx.query_row("SELECT count(*) FROM task_records", [], |row| row.get(0))?;
        if count >= TASK_COUNT_LIMIT {
            return Err(StorageError::TaskCapacity);
        }
        let mut reserved = HashSet::new();
        let task_id = unique_task_id(tx, &mut generate, &mut reserved)?;
        let plan_id = unique_task_id(tx, &mut generate, &mut reserved)?;
        tx.execute(
            "INSERT INTO task_records (
                id, schema_version, title, status, created_at_ms, updated_at_ms,
                archived_at_ms, last_opened_at_ms, selected_plan_id, project_id,
                origin_advisor_conversation_id, origin_advisor_dispatch_record_id
             ) VALUES (?1, 1, 'Untitled task', 'active', ?2, ?2, NULL, ?2, ?3, ?4, ?5, ?6)",
            params![
                task_id,
                now,
                plan_id,
                project_id,
                advisor_dispatch_origin.map(|(conversation_id, _)| conversation_id),
                advisor_dispatch_origin.map(|(_, dispatch_id)| dispatch_id),
            ],
        )?;
        tx.execute(
            "INSERT INTO task_plans (
                id, schema_version, task_id, label, position, body,
                created_at_ms, updated_at_ms
             ) VALUES (?1, 1, ?2, 'Primary plan', 0, '', ?3, ?3)",
            params![plan_id, task_id, now],
        )?;
        if task_payload_bytes(tx, Some(&task_id))? > TASK_RECORD_PAYLOAD_LIMIT
            || task_payload_bytes(tx, None)? > TASK_PAYLOAD_LIMIT
        {
            return Err(StorageError::TaskCapacity);
        }
        Ok(task_id)
    }

    #[cfg(test)]
    pub(crate) fn task_catalog(
        &mut self,
        selected: Option<&str>,
        include_archived: bool,
        query: Option<&str>,
    ) -> Result<TaskCatalogProjection, StorageError> {
        self.task_catalog_at(selected, include_archived, query, None, now_millis())
    }

    pub(crate) fn task_catalog_for_project(
        &mut self,
        project_id: &str,
        selected: Option<&str>,
        include_archived: bool,
        query: Option<&str>,
    ) -> Result<TaskCatalogProjection, StorageError> {
        let project_exists: Option<String> = self
            .connection
            .query_row(
                "SELECT project.id
                 FROM projects AS project
                 JOIN directory_associations AS association
                   ON association.id = project.active_directory_association_id
                 WHERE project.id = ?1
                   AND project.archived_at_ms IS NULL
                   AND association.detached_at_ms IS NULL",
                [project_id],
                |row| row.get(0),
            )
            .optional()?;
        if !project_exists.is_some_and(|id| is_uuid_v7(&id)) {
            return Err(StorageError::ProjectNotFound);
        }
        self.task_catalog_at(
            selected,
            include_archived,
            query,
            Some(project_id),
            now_millis(),
        )
    }

    fn task_catalog_at(
        &mut self,
        selected: Option<&str>,
        include_archived: bool,
        query: Option<&str>,
        project_id: Option<&str>,
        now: i64,
    ) -> Result<TaskCatalogProjection, StorageError> {
        let search = simple_case_fold(&match query {
            Some(value) if !value.trim().is_empty() => normalize_task_text(value, 120, 480)?,
            _ => String::new(),
        });
        if let Some(selected) = selected {
            self.repair_selected_plan(selected)?;
        }
        let mut stmt = self.connection.prepare(
            "SELECT t.id, t.schema_version, t.title, t.status, t.archived_at_ms,
                    t.selected_plan_id, t.updated_at_ms,
                    (SELECT count(*) FROM task_plans p WHERE p.task_id = t.id)
             FROM task_records t
             WHERE (?1 OR t.archived_at_ms IS NULL)
               AND (?2 IS NULL OR t.project_id = ?2)
             ORDER BY t.archived_at_ms IS NOT NULL, t.updated_at_ms DESC, t.id",
        )?;
        let rows = stmt.query_map(params![include_archived, project_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<i64>>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, i64>(7)?,
            ))
        })?;
        let mut tasks = Vec::new();
        let mut corrupt = false;
        for row in rows {
            let Ok((
                id,
                schema_version,
                title,
                status,
                archived_at,
                selected_plan_id,
                updated,
                plan_count,
            )) = row
            else {
                corrupt = true;
                continue;
            };
            let normalized_title = normalize_task_text(&title, 120, 480);
            let parsed_status = task_status(&status);
            let valid = schema_version == 1
                && valid_task_id(&id)
                && valid_task_id(&selected_plan_id)
                && normalized_title.as_ref().is_ok_and(|value| value == &title)
                && parsed_status.is_ok()
                && archived_at.is_none_or(|value| value >= 0)
                && updated >= 0
                && (1..=TASK_PLAN_LIMIT).contains(&plan_count);
            if !valid {
                corrupt = true;
                continue;
            }
            let mut labels = self
                .connection
                .prepare("SELECT label FROM task_plans WHERE task_id = ?1 LIMIT 4")?;
            let label_rows = labels.query_map([&id], |row| row.get::<_, String>(0))?;
            let mut label_match = search.is_empty() || simple_case_fold(&title).contains(&search);
            let mut task_corrupt = false;
            for label in label_rows {
                match label {
                    Ok(label)
                        if normalize_task_text(&label, 80, 320)
                            .is_ok_and(|normalized| normalized == label) =>
                    {
                        label_match |= simple_case_fold(&label).contains(&search);
                    }
                    _ => {
                        corrupt = true;
                        task_corrupt = true;
                    }
                }
            }
            if task_corrupt || !label_match {
                continue;
            }
            tasks.push(TaskRecordSummary {
                id,
                title,
                status: parsed_status.expect("validated task status"),
                archived: archived_at.is_some(),
                selected_plan_id,
                plan_count: plan_count as u8,
                updated_at_ms: updated,
                cleanup_eligible: (archived_at.is_some() || status == "completed")
                    && now.saturating_sub(updated) >= TASK_CLEANUP_AGE_MS,
            });
            if tasks.len() == 50 {
                break;
            }
        }
        let mut selected_task =
            selected.and_then(|id| tasks.iter().find(|task| task.id == id).cloned());
        let plans = if let Some(task) = &selected_task {
            let mut plans = Vec::new();
            let mut statement = self.connection.prepare(
                "SELECT id, schema_version, label, position, body
                 FROM task_plans WHERE task_id = ?1
                 ORDER BY position, id LIMIT 4",
            )?;
            for row in statement.query_map([&task.id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })? {
                let Ok((id, schema_version, label, position, body)) = row else {
                    corrupt = true;
                    continue;
                };
                if schema_version != 1
                    || !valid_task_id(&id)
                    || !normalize_task_text(&label, 80, 320)
                        .is_ok_and(|normalized| normalized == label)
                    || !(0..=3).contains(&position)
                    || validate_plan_body(&body).is_err()
                {
                    corrupt = true;
                    continue;
                }
                plans.push(TaskPlanSummary {
                    id,
                    label,
                    position: position as u8,
                    body,
                });
            }
            if plans.len() != task.plan_count as usize
                || !plans.iter().any(|plan| plan.id == task.selected_plan_id)
            {
                corrupt = true;
                Vec::new()
            } else {
                plans
            }
        } else {
            Vec::new()
        };
        if plans.is_empty() {
            if let Some(selected) = selected_task.take() {
                tasks.retain(|task| task.id != selected.id);
            }
        }
        let count: u16 = self.connection.query_row(
            "SELECT count(*) FROM task_records WHERE (?1 IS NULL OR project_id = ?1)",
            [project_id],
            |r| r.get(0),
        )?;
        let bytes = task_payload_bytes(&self.connection, None)?;
        Ok((
            tasks,
            selected_task,
            plans,
            count,
            bytes.max(0) as u64,
            corrupt,
        ))
    }

    pub(crate) fn rename_task(&mut self, id: &str, title: &str) -> Result<(), StorageError> {
        let title = normalize_task_text(title, 120, 480)?;
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        ensure_editable_task(&tx, id)?;
        tx.execute(
            "UPDATE task_records SET title = ?1, updated_at_ms = ?2 WHERE id = ?3",
            params![title, now, id],
        )?;
        ensure_task_capacity(&tx, id)?;
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn set_task_status(
        &mut self,
        id: &str,
        status: TaskStatus,
    ) -> Result<(), StorageError> {
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current = ensure_editable_task(&tx, id)?;
        let allowed = matches!(
            (current.as_str(), status),
            ("active", TaskStatus::Paused | TaskStatus::Completed)
                | ("paused", TaskStatus::Active | TaskStatus::Completed)
                | ("completed", TaskStatus::Active)
        );
        if !allowed {
            return Err(StorageError::InvalidStatusTransition);
        }
        tx.execute(
            "UPDATE task_records SET status = ?1, updated_at_ms = ?2 WHERE id = ?3",
            params![task_status_value(status), now, id],
        )?;
        if status == TaskStatus::Completed {
            tx.execute(
                "UPDATE local_review_collections
                 SET state = 'frozen', updated_at_ms = CASE WHEN updated_at_ms >= ?1 THEN updated_at_ms + 1 ELSE ?1 END
                 WHERE task_id = ?2 AND discarded_at_ms IS NULL",
                params![now, id],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn archive_task(&mut self, id: &str, restore: bool) -> Result<(), StorageError> {
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let archived_at: Option<Option<i64>> = tx
            .query_row(
                "SELECT archived_at_ms FROM task_records WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
            .optional()?;
        let Some(archived_at) = archived_at else {
            return Err(StorageError::TaskNotFound);
        };
        if restore == archived_at.is_none() {
            return Err(if restore {
                StorageError::TaskNotFound
            } else {
                StorageError::TaskArchived
            });
        }
        let sql = if restore {
            "UPDATE task_records SET archived_at_ms = NULL, updated_at_ms = ?1 WHERE id = ?2"
        } else {
            "UPDATE task_records SET archived_at_ms = ?1, updated_at_ms = ?1 WHERE id = ?2"
        };
        tx.execute(sql, params![now, id])?;
        if !restore {
            tx.execute(
                "UPDATE local_review_collections
                 SET state = 'frozen', updated_at_ms = CASE WHEN updated_at_ms >= ?1 THEN updated_at_ms + 1 ELSE ?1 END
                 WHERE task_id = ?2 AND discarded_at_ms IS NULL",
                params![now, id],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn delete_task(&mut self, id: &str) -> Result<(), StorageError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if tx.execute("DELETE FROM task_records WHERE id = ?1", [id])? != 1 {
            return Err(StorageError::TaskNotFound);
        }
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn create_plan(
        &mut self,
        task_id: &str,
        copy: bool,
    ) -> Result<String, StorageError> {
        let now = now_millis();
        self.create_plan_with(task_id, copy, now, || Uuid::now_v7().to_string())
    }

    fn create_plan_with(
        &mut self,
        task_id: &str,
        copy: bool,
        now: i64,
        mut generate: impl FnMut() -> String,
    ) -> Result<String, StorageError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        ensure_editable_task(&tx, task_id)?;
        let (count, primary_body, next_position): (i64, String, i64) = tx.query_row(
            "SELECT count(*),
                    COALESCE((SELECT body FROM task_plans WHERE task_id = ?1 AND position = 0), ''),
                    COALESCE(max(position), -1) + 1
             FROM task_plans WHERE task_id = ?1",
            [task_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        if count >= TASK_PLAN_LIMIT || next_position > 3 {
            return Err(StorageError::PlanCapacity);
        }
        let mut reserved = HashSet::new();
        let id = unique_task_id(&tx, &mut generate, &mut reserved)?;
        tx.execute(
            "INSERT INTO task_plans (
                id, schema_version, task_id, label, position, body,
                created_at_ms, updated_at_ms
             ) VALUES (?1, 1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![
                id,
                task_id,
                format!("Alternate plan {count}"),
                next_position,
                if copy { primary_body } else { String::new() },
                now
            ],
        )?;
        ensure_task_capacity(&tx, task_id)?;
        tx.execute(
            "UPDATE task_records
             SET selected_plan_id = ?1, last_opened_at_ms = ?2, updated_at_ms = ?2
             WHERE id = ?3",
            params![id, now, task_id],
        )?;
        tx.commit()?;
        Ok(id)
    }

    pub(crate) fn select_plan(&mut self, task_id: &str, plan_id: &str) -> Result<(), StorageError> {
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        ensure_editable_task(&tx, task_id)?;
        let exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM task_plans WHERE id = ?1 AND task_id = ?2)",
            params![plan_id, task_id],
            |row| row.get(0),
        )?;
        if !exists {
            return Err(StorageError::PlanNotFound);
        }
        tx.execute(
            "UPDATE task_records
             SET selected_plan_id = ?1, last_opened_at_ms = ?2, updated_at_ms = ?2
             WHERE id = ?3",
            params![plan_id, now, task_id],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn edit_plan(
        &mut self,
        task_id: &str,
        plan_id: &str,
        label: &str,
        body: &str,
    ) -> Result<(), StorageError> {
        let label = normalize_task_text(label, 80, 320)?;
        validate_plan_body(body)?;
        let now = now_millis();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        ensure_editable_task(&tx, task_id)?;
        if tx.execute(
            "UPDATE task_plans SET label = ?1, body = ?2, updated_at_ms = ?3
             WHERE id = ?4 AND task_id = ?5",
            params![label, body, now, plan_id, task_id],
        )? != 1
        {
            return Err(StorageError::PlanNotFound);
        }
        ensure_task_capacity(&tx, task_id)?;
        tx.execute(
            "UPDATE task_records SET updated_at_ms = ?1 WHERE id = ?2",
            params![now, task_id],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub(crate) fn delete_plan(&mut self, task_id: &str, plan_id: &str) -> Result<(), StorageError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        ensure_editable_task(&tx, task_id)?;
        let count: i64 = tx.query_row(
            "SELECT count(*) FROM task_plans WHERE task_id = ?1",
            [task_id],
            |row| row.get(0),
        )?;
        if count <= 1 {
            return Err(StorageError::PlanCapacity);
        }
        if tx.execute(
            "DELETE FROM task_plans WHERE id = ?1 AND task_id = ?2",
            params![plan_id, task_id],
        )? != 1
        {
            return Err(StorageError::PlanNotFound);
        }
        let selected: String = tx.query_row(
            "SELECT selected_plan_id FROM task_records WHERE id = ?1",
            [task_id],
            |row| row.get(0),
        )?;
        let remaining: Vec<String> = {
            let mut statement =
                tx.prepare("SELECT id FROM task_plans WHERE task_id = ?1 ORDER BY position, id")?;
            let rows = statement
                .query_map([task_id], |row| row.get(0))?
                .collect::<Result<_, _>>()?;
            rows
        };
        for (position, remaining_id) in remaining.iter().enumerate() {
            tx.execute(
                "UPDATE task_plans SET position = ?1 WHERE id = ?2 AND task_id = ?3",
                params![position as i64, remaining_id, task_id],
            )?;
        }
        let selected = if selected == plan_id {
            remaining.first().ok_or(StorageError::InvalidStoredValue)?
        } else {
            &selected
        };
        let now = now_millis();
        tx.execute(
            "UPDATE task_records SET selected_plan_id = ?1, updated_at_ms = ?2 WHERE id = ?3",
            params![selected, now, task_id],
        )?;
        tx.commit()?;
        Ok(())
    }

    fn repair_selected_plan(&mut self, task_id: &str) -> Result<(), StorageError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let selected: Option<String> = tx
            .query_row(
                "SELECT selected_plan_id FROM task_records WHERE id = ?1",
                [task_id],
                |row| row.get(0),
            )
            .optional()?;
        let Some(selected) = selected else {
            return Ok(());
        };
        let selected_exists: bool = tx.query_row(
            "SELECT EXISTS(
                SELECT 1 FROM task_plans WHERE id = ?1 AND task_id = ?2
             )",
            params![selected, task_id],
            |row| row.get(0),
        )?;
        if !selected_exists {
            let fallback: Option<String> = tx
                .query_row(
                    "SELECT id FROM task_plans
                     WHERE task_id = ?1 ORDER BY position, id LIMIT 1",
                    [task_id],
                    |row| row.get(0),
                )
                .optional()?;
            let Some(fallback) = fallback else {
                return Err(StorageError::InvalidStoredValue);
            };
            tx.execute(
                "UPDATE task_records SET selected_plan_id = ?1 WHERE id = ?2",
                params![fallback, task_id],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
    pub(crate) fn open(path: &Path) -> Result<Self, StorageError> {
        let parent = path.parent().ok_or(StorageError::Filesystem)?;
        fs::create_dir_all(parent).map_err(|_| StorageError::Filesystem)?;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
            .map_err(|_| StorageError::Filesystem)?;

        let connection = Connection::open(path)?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|_| StorageError::Filesystem)?;
        Self::from_connection(connection)
    }

    #[cfg(test)]
    pub(crate) fn in_memory() -> Result<Self, StorageError> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    #[cfg(test)]
    pub(crate) fn from_test_connection(connection: Connection) -> Result<Self, StorageError> {
        Self::from_connection(connection)
    }

    #[cfg(test)]
    pub(crate) fn fail_worktree_registration_for_test(&self) -> Result<(), StorageError> {
        self.connection.execute_batch(
            "CREATE TEMP TRIGGER fail_worktree_registration
             BEFORE INSERT ON worktree_relations
             BEGIN SELECT RAISE(ABORT, 'test worktree registration failure'); END;",
        )?;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn allow_worktree_registration_for_test(&self) -> Result<(), StorageError> {
        self.connection
            .execute_batch("DROP TRIGGER IF EXISTS fail_worktree_registration")?;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn fail_worktree_retirement_for_test(&self) -> Result<(), StorageError> {
        self.connection.execute_batch(
            "CREATE TEMP TRIGGER fail_worktree_retirement
             BEFORE UPDATE OF active_directory_association_id ON projects
             BEGIN SELECT RAISE(ABORT, 'test worktree retirement failure'); END;",
        )?;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn allow_worktree_retirement_for_test(&self) -> Result<(), StorageError> {
        self.connection
            .execute_batch("DROP TRIGGER IF EXISTS fail_worktree_retirement")?;
        Ok(())
    }

    fn from_connection(mut connection: Connection) -> Result<Self, StorageError> {
        connection.pragma_update(None, "foreign_keys", true)?;
        connection.pragma_update(None, "trusted_schema", false)?;
        connection.busy_timeout(Duration::from_secs(5))?;
        apply_migrations(&mut connection)?;
        verify_schema(&connection)?;
        recover_interrupted_conversations(&connection)?;
        recover_interrupted_terminals(&connection)?;
        recover_interrupted_fictional_connector_operations(&connection)?;
        recover_interrupted_controlled_browser_verifications(&connection)?;
        recover_interrupted_context_bundles(&connection)?;
        Ok(Self {
            connection,
            activity_session_id: Uuid::now_v7().to_string(),
        })
    }

    pub(crate) fn list_projects(&self) -> Result<Vec<StoredProject>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT id, display_name, archived_at_ms, active_directory_association_id
             FROM projects
             ORDER BY archived_at_ms IS NOT NULL, updated_at_ms DESC, id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<i64>>(2)?.is_some(),
                row.get::<_, Option<String>>(3)?,
            ))
        })?;

        let mut projects = Vec::new();
        for row in rows {
            let (id, display_name, archived, association_id) = row?;
            let association = association_id
                .as_deref()
                .map(|association_id| self.load_association(association_id))
                .transpose()?;
            projects.push(StoredProject {
                id,
                display_name,
                archived,
                association,
            });
        }
        Ok(projects)
    }

    /// Reads only bounded metadata already owned by QuireForge. It does not
    /// inspect a project, contact Codex, read context, or mutate SQLite.
    pub(crate) fn advisor_snapshot(&self) -> Result<AdvisorFoundationSnapshot, StorageError> {
        let snapshot = AdvisorFoundationSnapshot {
            schema_version: crate::advisor::ADVISOR_FOUNDATION_SCHEMA_VERSION,
            conversations: self.load_advisor_conversations()?,
            context_references: self.load_advisor_context_references()?,
            dispatch_proposals: self.load_advisor_dispatch_proposals()?,
        };
        snapshot
            .validate()
            .map_err(|_| StorageError::InvalidStoredValue)?;
        Ok(snapshot)
    }

    fn load_advisor_conversations(
        &self,
    ) -> Result<Vec<AdvisorConversationReference>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT id, codex_thread_id, created_at_ms, updated_at_ms
             FROM advisor_conversations ORDER BY updated_at_ms DESC, id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(AdvisorConversationReference {
                id: row.get(0)?,
                codex_thread_id: row.get(1)?,
                created_at_ms: row.get(2)?,
                updated_at_ms: row.get(3)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    fn load_advisor_context_references(
        &self,
    ) -> Result<Vec<AdvisorContextReference>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT id, advisor_conversation_id, kind, source_ref, source_commit,
                    source_sha256, selected_at_ms, freshness, trust, provenance_source,
                    provenance_ref, provenance_commit, observed_at_ms, provenance_note
             FROM advisor_context_references
             ORDER BY selected_at_ms DESC, id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, Option<String>>(10)?,
                row.get::<_, Option<String>>(11)?,
                row.get::<_, Option<i64>>(12)?,
                row.get::<_, Option<String>>(13)?,
            ))
        })?;
        let mut references = Vec::new();
        for row in rows {
            let (
                id,
                advisor_conversation_id,
                kind,
                source_ref,
                source_commit,
                content_sha256,
                selected_at_ms,
                freshness,
                trust,
                provenance_source,
                provenance_ref,
                provenance_commit,
                observed_at_ms,
                note,
            ) = row?;
            references.push(AdvisorContextReference {
                id,
                advisor_conversation_id,
                kind: parse_advisor_context_kind(&kind)?,
                source_ref,
                source_commit,
                content_sha256,
                selected_at_ms,
                freshness: parse_advisor_freshness(&freshness)?,
                provenance: AdvisorProvenance {
                    trust: parse_advisor_trust(&trust)?,
                    source: parse_advisor_provenance_source(&provenance_source)?,
                    source_ref: provenance_ref,
                    source_commit: provenance_commit,
                    observed_at_ms,
                    note,
                },
            });
        }
        Ok(references)
    }

    fn load_advisor_dispatch_proposals(
        &self,
    ) -> Result<Vec<AdvisorDispatchProposal>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT id, advisor_conversation_id, target_project_id, request_sha256,
                    context_manifest_sha256, state, requires_explicit_approval,
                    requested_model, requested_reasoning_effort, trust,
                    provenance_source, provenance_ref, provenance_commit,
                    observed_at_ms, provenance_note, created_at_ms, updated_at_ms,
                    capability_manifest_sha256, decided_at_ms, expires_at_ms,
                    execution_dispatch_state, execution_conversation_id
             FROM advisor_dispatch_records ORDER BY updated_at_ms DESC, id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, String>(10)?,
                row.get::<_, Option<String>>(11)?,
                row.get::<_, Option<String>>(12)?,
                row.get::<_, Option<i64>>(13)?,
                row.get::<_, Option<String>>(14)?,
                row.get::<_, i64>(15)?,
                row.get::<_, i64>(16)?,
                row.get::<_, String>(17)?,
                row.get::<_, Option<i64>>(18)?,
                row.get::<_, i64>(19)?,
                row.get::<_, Option<String>>(20)?,
                row.get::<_, Option<String>>(21)?,
            ))
        })?;
        let mut proposals = Vec::new();
        for row in rows {
            let (
                id,
                advisor_conversation_id,
                target_project_id,
                prompt_sha256,
                context_manifest_sha256,
                state,
                requires_explicit_approval,
                requested_model,
                requested_reasoning_effort,
                trust,
                provenance_source,
                provenance_ref,
                provenance_commit,
                observed_at_ms,
                note,
                created_at_ms,
                updated_at_ms,
                capability_manifest_sha256,
                decided_at_ms,
                expires_at_ms,
                execution_dispatch_state,
                execution_conversation_id,
            ) = row?;
            proposals.push(AdvisorDispatchProposal {
                id,
                advisor_conversation_id,
                target_project_id,
                prompt_sha256,
                context_manifest_sha256,
                capability_manifest_sha256,
                state: parse_advisor_dispatch_state(&state)?,
                requires_explicit_approval: requires_explicit_approval == 1,
                requested_model,
                requested_reasoning_effort,
                created_at_ms,
                updated_at_ms,
                decided_at_ms,
                expires_at_ms,
                execution_dispatch_state: execution_dispatch_state
                    .as_deref()
                    .map(parse_advisor_execution_dispatch_state)
                    .transpose()?,
                execution_conversation_id,
                provenance: AdvisorProvenance {
                    trust: parse_advisor_trust(&trust)?,
                    source: parse_advisor_provenance_source(&provenance_source)?,
                    source_ref: provenance_ref,
                    source_commit: provenance_commit,
                    observed_at_ms,
                    note,
                },
            });
        }
        Ok(proposals)
    }

    pub(crate) fn project(&self, project_id: &str) -> Result<StoredProject, StorageError> {
        self.list_projects()?
            .into_iter()
            .find(|project| project.id == project_id)
            .ok_or(StorageError::ProjectNotFound)
    }

    /// Internal-only writer for results that a native package-validation
    /// controller has already validated. No command or frontend surface calls
    /// this method.
    #[allow(dead_code)] // Reserved exclusively for the native package-validation controller.
    pub(crate) fn record_package_validation_summary(
        &mut self,
        project_id: &str,
        input: PackageValidationRecordInput,
    ) -> Result<PackageValidationRecordOutcome, StorageError> {
        self.record_package_validation_summary_at(project_id, input, now_millis())
    }

    fn record_package_validation_summary_at(
        &mut self,
        project_id: &str,
        input: PackageValidationRecordInput,
        now: i64,
    ) -> Result<PackageValidationRecordOutcome, StorageError> {
        if !is_uuid_v7(project_id) || now < 0 {
            return Err(StorageError::InvalidStoredValue);
        }
        let mut input = input;
        if input.validation_phase == PackageValidationPhase::InstalledHost {
            let derived = installed_host_attempt_identity(
                &input.candidate_identity_sha256,
                input.installed_host_state,
                input.installed_host_facts.as_ref(),
            )?;
            if input
                .attempt_identity_sha256
                .as_deref()
                .is_some_and(|value| value != derived)
            {
                return Err(StorageError::InvalidStoredValue);
            }
            input.attempt_identity_sha256 = Some(derived);
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        validate_package_validation_input(&input)?;
        let project_available = tx
            .query_row(
                "SELECT 1
                 FROM projects AS project
                 JOIN directory_associations AS association
                   ON association.id = project.active_directory_association_id
                 WHERE project.id = ?1
                   AND project.archived_at_ms IS NULL
                   AND association.detached_at_ms IS NULL",
                [project_id],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        if !project_available {
            return Err(StorageError::ProjectNotFound);
        }

        if let Some(summary_id) = tx
            .query_row(
                "SELECT package_validation_summary_id
                 FROM project_package_validation_candidate_identities
                 WHERE project_id = ?1 AND candidate_identity_sha256 = ?2
                   AND validation_phase = ?3 AND attempt_identity_sha256 = ?4",
                params![
                    project_id,
                    input.candidate_identity_sha256,
                    package_validation_phase_value(input.validation_phase),
                    input
                        .attempt_identity_sha256
                        .as_deref()
                        .unwrap_or(&input.candidate_identity_sha256),
                ],
                |row| row.get::<_, String>(0),
            )
            .optional()?
        {
            let summary = package_validation_summary_record(&tx, &summary_id)?;
            if summary.project_id != project_id
                || summary.input.candidate_identity_sha256 != input.candidate_identity_sha256
                || summary.input.validation_phase != input.validation_phase
                || summary.input.attempt_identity_sha256 != input.attempt_identity_sha256
            {
                return Err(StorageError::InvalidStoredValue);
            }
            return Ok(PackageValidationRecordOutcome::Existing(summary));
        }

        let mut created_at_ms = now;
        match input.validation_phase {
            PackageValidationPhase::Unprivileged => {}
            PackageValidationPhase::InstalledHost => {
                if tx.query_row(
                    "SELECT 1 FROM project_package_validation_candidate_identities AS identity
                     JOIN project_package_validation_summaries AS summary
                       ON summary.id = identity.package_validation_summary_id
                     WHERE identity.project_id = ?1 AND identity.candidate_identity_sha256 = ?2
                       AND identity.validation_phase = 'installed-host' AND summary.validation_complete = 1",
                    params![project_id, input.candidate_identity_sha256], |_| Ok(())
                ).optional()?.is_some() {
                    return Err(StorageError::InvalidStoredValue);
                }
                let attempts: i64 = tx.query_row(
                    "SELECT count(*) FROM project_package_validation_candidate_identities
                     WHERE project_id = ?1 AND candidate_identity_sha256 = ?2 AND validation_phase = 'installed-host'",
                    params![project_id, input.candidate_identity_sha256], |row| row.get(0)
                )?;
                if attempts >= INSTALLED_HOST_ATTEMPT_LIMIT {
                    return Err(StorageError::TaskCapacity);
                }
                let supersedes_id = input
                    .supersedes_record_id
                    .as_deref()
                    .ok_or(StorageError::InvalidStoredValue)?;
                let previous = package_validation_summary_record(&tx, supersedes_id)?;
                let predecessor_phase: String = tx.query_row(
                    "SELECT validation_phase FROM project_package_validation_candidate_identities
                     WHERE project_id = ?1 AND candidate_identity_sha256 = ?2
                       AND package_validation_summary_id = ?3",
                    params![project_id, input.candidate_identity_sha256, supersedes_id],
                    |row| row.get(0),
                )?;
                let predecessor_phase = package_validation_phase(&predecessor_phase)?;
                let newest: Option<String> = tx.query_row(
                    "SELECT package_validation_summary_id FROM project_package_validation_candidate_identities
                     WHERE project_id = ?1 AND candidate_identity_sha256 = ?2
                     ORDER BY CASE validation_phase WHEN 'installed-host' THEN 1 ELSE 0 END DESC, created_at_ms DESC, package_validation_summary_id DESC LIMIT 1",
                    params![project_id, input.candidate_identity_sha256], |row| row.get(0)
                ).optional()?;
                if previous.project_id != project_id
                    || !matches!(
                        predecessor_phase,
                        PackageValidationPhase::Unprivileged
                            | PackageValidationPhase::InstalledHost
                    )
                    || newest.as_deref() != Some(supersedes_id)
                    || previous.input.application_version != input.application_version
                    || previous.input.debian_version != input.debian_version
                    || previous.input.artifact_count != input.artifact_count
                    || previous.input.manifest_state != input.manifest_state
                    || previous.input.checksum_state != input.checksum_state
                    || previous.input.abi_state != input.abi_state
                    || previous.input.provenance_state != input.provenance_state
                    || previous.input.visible_launch_state != input.visible_launch_state
                {
                    return Err(StorageError::InvalidStoredValue);
                }
                created_at_ms = created_at_ms.max(previous.created_at_ms.saturating_add(1));
            }
        }

        prune_package_validation_summaries(
            &tx,
            project_id,
            created_at_ms,
            input.supersedes_record_id.as_deref(),
        )?;
        let count: i64 = tx.query_row(
            "SELECT count(*) FROM project_package_validation_summaries WHERE project_id = ?1",
            [project_id],
            |row| row.get(0),
        )?;
        if count >= PACKAGE_VALIDATION_RECORD_LIMIT as i64 {
            return Err(StorageError::TaskCapacity);
        }

        let id = Uuid::now_v7().to_string();
        let record_sha256 =
            package_validation_record_digest(&id, project_id, &input, created_at_ms)?;
        tx.execute(
            "INSERT INTO project_package_validation_summaries (
                id, project_id, application_version, debian_version,
                manifest_state, checksum_state, abi_state, provenance_state,
                visible_launch_state, installed_host_state, artifact_count,
                validation_complete, record_sha256, created_at_ms, supersedes_record_id
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15
             )",
            params![
                id,
                project_id,
                input.application_version,
                input.debian_version,
                package_validation_state_value(input.manifest_state),
                package_validation_state_value(input.checksum_state),
                package_validation_state_value(input.abi_state),
                package_validation_state_value(input.provenance_state),
                package_validation_state_value(input.visible_launch_state),
                package_validation_state_value(input.installed_host_state),
                input.artifact_count,
                input.validation_complete,
                record_sha256,
                created_at_ms,
                input.supersedes_record_id,
            ],
        )?;
        tx.execute(
            "INSERT INTO project_package_validation_candidate_identities (
                project_id, candidate_identity_sha256, validation_phase,
                attempt_identity_sha256, package_validation_summary_id, created_at_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                project_id,
                input.candidate_identity_sha256,
                package_validation_phase_value(input.validation_phase),
                input
                    .attempt_identity_sha256
                    .as_deref()
                    .unwrap_or(&input.candidate_identity_sha256),
                id,
                created_at_ms
            ],
        )?;
        let summary = package_validation_summary_record(&tx, &id)?;
        tx.commit()?;
        Ok(PackageValidationRecordOutcome::Created(summary))
    }

    #[cfg(test)]
    fn record_package_validation_summary_at_for_test(
        &mut self,
        project_id: &str,
        input: PackageValidationRecordInput,
        now: i64,
    ) -> Result<PackageValidationRecordOutcome, StorageError> {
        self.record_package_validation_summary_at(project_id, input, now)
    }

    #[cfg(test)]
    fn package_validation_summary_for_test(
        &self,
        id: &str,
    ) -> Result<PackageValidationSummary, StorageError> {
        package_validation_summary_record(&self.connection, id)
    }

    pub(crate) fn package_validation_summary_for_internal(
        &self,
        id: &str,
    ) -> Result<PackageValidationSummary, StorageError> {
        package_validation_summary_record(&self.connection, id)
    }

    /// Finds the single durable unprivileged receipt that the fixed headless
    /// executable may extend. Both the active project context and the receipt
    /// are deliberately resolved from migration-18 state, never from argv.
    #[cfg(test)]
    pub(crate) fn installed_host_headless_predecessor_for_internal(
        &self,
    ) -> Result<(String, PackageValidationSummary), StorageError> {
        let project_ids = self
            .connection
            .prepare(
                "SELECT project.id
                 FROM projects AS project
                 JOIN directory_associations AS association
                   ON association.id = project.active_directory_association_id
                 WHERE project.archived_at_ms IS NULL
                   AND association.detached_at_ms IS NULL
                 ORDER BY project.id
                 LIMIT 2",
            )?
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        let [project_id] = project_ids.as_slice() else {
            return Err(StorageError::InvalidStoredValue);
        };

        let predecessors = self
            .connection
            .prepare(
                "SELECT identity.package_validation_summary_id,
                        identity.candidate_identity_sha256,
                        identity.attempt_identity_sha256
                 FROM project_package_validation_candidate_identities AS identity
                 JOIN project_package_validation_summaries AS summary
                   ON summary.id = identity.package_validation_summary_id
                 WHERE identity.project_id = ?1
                   AND identity.validation_phase = 'unprivileged'
                   AND summary.validation_complete = 0
                   AND summary.installed_host_state = 'unavailable'
                 ORDER BY identity.created_at_ms DESC, identity.package_validation_summary_id DESC
                 LIMIT 2",
            )?
            .query_map([project_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let [(receipt_id, candidate_identity_sha256, attempt_identity_sha256)] =
            predecessors.as_slice()
        else {
            return Err(StorageError::InvalidStoredValue);
        };
        if candidate_identity_sha256 != attempt_identity_sha256 {
            return Err(StorageError::InvalidStoredValue);
        }
        let predecessor = package_validation_summary_record(&self.connection, receipt_id)?;
        if predecessor.project_id != *project_id
            || predecessor.input.validation_phase != PackageValidationPhase::Unprivileged
            || predecessor.input.validation_complete
            || predecessor.input.installed_host_state != LocalReviewEvidenceCheckState::Unavailable
            || predecessor.input.candidate_identity_sha256 != *candidate_identity_sha256
        {
            return Err(StorageError::InvalidStoredValue);
        }
        Ok((project_id.clone(), predecessor))
    }

    pub(crate) fn installed_host_headless_context_project_id_for_internal(
        &self,
    ) -> Result<String, StorageError> {
        let project_ids = self
            .connection
            .prepare(
                "SELECT project.id FROM projects AS project
             JOIN directory_associations AS association
               ON association.id = project.active_directory_association_id
             WHERE project.archived_at_ms IS NULL AND association.detached_at_ms IS NULL
             ORDER BY project.id LIMIT 2",
            )?
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        let [project_id] = project_ids.as_slice() else {
            return Err(StorageError::InvalidStoredValue);
        };
        Ok(project_id.clone())
    }

    /// Returns the immutable tail that an installed-host result may supersede.
    /// The caller first authenticates the supplied unprivileged receipt, then
    /// uses this lookup to extend its one linear durable chain.
    pub(crate) fn package_validation_installed_host_predecessor_for_internal(
        &self,
        project_id: &str,
        candidate_identity_sha256: &str,
    ) -> Result<PackageValidationSummary, StorageError> {
        let id: String = self.connection.query_row(
            "SELECT package_validation_summary_id
             FROM project_package_validation_candidate_identities
             WHERE project_id = ?1 AND candidate_identity_sha256 = ?2
             ORDER BY CASE validation_phase WHEN 'installed-host' THEN 1 ELSE 0 END DESC,
                      created_at_ms DESC, package_validation_summary_id DESC
             LIMIT 1",
            params![project_id, candidate_identity_sha256],
            |row| row.get(0),
        )?;
        package_validation_summary_record(&self.connection, &id)
    }

    #[cfg(test)]
    pub(crate) fn package_validation_test_repository() -> (Self, String) {
        let repository = Self::in_memory().expect("test repository");
        let project_id = Uuid::now_v7().to_string();
        let association_id = Uuid::now_v7().to_string();
        let path = format!("/package-validation-test-{project_id}");
        repository.connection.execute(
            "INSERT INTO projects (id, display_name, active_directory_association_id, archived_at_ms, created_at_ms, updated_at_ms) VALUES (?1, 'fixture', NULL, NULL, 1, 1)", [&project_id]
        ).expect("project");
        repository.connection.execute(
            "INSERT INTO directory_associations (id, project_id, selected_path, resolved_path, role, is_primary, expected_access, device_id, inode, filesystem_type, mount_id, git_common_dir, git_worktree_root, git_is_linked_worktree, has_agents_guidance, has_codex_config, accessibility_state, last_verified_at_ms, detached_at_ms, created_at_ms, updated_at_ms) VALUES (?1, ?2, ?3, ?3, 'primary', 1, 'read-write', NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, 0, 'available', 1, NULL, 1, 1)",
            params![association_id, project_id, path]
        ).expect("association");
        repository
            .connection
            .execute(
                "UPDATE projects SET active_directory_association_id = ?1 WHERE id = ?2",
                params![association_id, project_id],
            )
            .expect("active");
        (repository, project_id)
    }

    #[cfg(test)]
    pub(crate) fn package_validation_phase_summary_for_test(
        &self,
        project_id: &str,
        phase: PackageValidationPhase,
    ) -> Result<PackageValidationSummary, StorageError> {
        let id: String = self.connection.query_row(
            "SELECT package_validation_summary_id FROM project_package_validation_candidate_identities WHERE project_id = ?1 AND validation_phase = ?2 ORDER BY created_at_ms DESC LIMIT 1",
            params![project_id, package_validation_phase_value(phase)], |row| row.get(0)
        )?;
        package_validation_summary_record(&self.connection, &id)
    }

    pub(crate) fn ensure_directory_available(
        &self,
        identity: &DirectoryIdentity,
        excluding_association_id: Option<&str>,
    ) -> Result<(), StorageError> {
        ensure_directory_available(&self.connection, identity, excluding_association_id)
    }

    pub(crate) fn insert_project(
        &mut self,
        display_name: &str,
        identity: &DirectoryIdentity,
    ) -> Result<String, StorageError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        ensure_directory_available(&transaction, identity, None)?;

        let project_id = Uuid::now_v7().to_string();
        let association_id = Uuid::now_v7().to_string();
        let timestamp = now_millis();
        transaction.execute(
            "INSERT INTO projects
             (id, display_name, active_directory_association_id, archived_at_ms,
              created_at_ms, updated_at_ms)
             VALUES (?1, ?2, NULL, NULL, ?3, ?3)",
            params![project_id, display_name, timestamp],
        )?;
        insert_association(
            &transaction,
            &association_id,
            &project_id,
            identity,
            timestamp,
        )?;
        transaction.execute(
            "UPDATE projects SET active_directory_association_id = ?1 WHERE id = ?2",
            params![association_id, project_id],
        )?;
        transaction.commit()?;
        Ok(project_id)
    }

    pub(crate) fn insert_worktree_project(
        &mut self,
        source_project_id: &str,
        display_name: &str,
        identity: &DirectoryIdentity,
        ownership: &str,
        branch_name: Option<&str>,
    ) -> Result<String, StorageError> {
        if !matches!(ownership, "managed" | "attached")
            || display_name.is_empty()
            || display_name.chars().count() > 120
            || branch_name.is_some_and(|branch| branch.is_empty() || branch.len() > 96)
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let source_exists = transaction
            .query_row(
                "SELECT 1 FROM projects
                 WHERE id = ?1 AND archived_at_ms IS NULL
                   AND active_directory_association_id IS NOT NULL",
                [source_project_id],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        if !source_exists {
            return Err(StorageError::ProjectNotFound);
        }
        ensure_directory_available(&transaction, identity, None)?;

        let project_id = Uuid::now_v7().to_string();
        let association_id = Uuid::now_v7().to_string();
        let relation_id = Uuid::now_v7().to_string();
        let timestamp = now_millis();
        transaction.execute(
            "INSERT INTO projects
             (id, display_name, active_directory_association_id, archived_at_ms,
              created_at_ms, updated_at_ms)
             VALUES (?1, ?2, NULL, NULL, ?3, ?3)",
            params![project_id, display_name, timestamp],
        )?;
        insert_association(
            &transaction,
            &association_id,
            &project_id,
            identity,
            timestamp,
        )?;
        transaction.execute(
            "UPDATE projects SET active_directory_association_id = ?1 WHERE id = ?2",
            params![association_id, project_id],
        )?;
        transaction.execute(
            "INSERT INTO worktree_relations
             (id, source_project_id, worktree_project_id, ownership, branch_name,
              created_at_ms, updated_at_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![
                relation_id,
                source_project_id,
                project_id,
                ownership,
                branch_name,
                timestamp
            ],
        )?;
        transaction.commit()?;
        Ok(project_id)
    }

    pub(crate) fn worktree_source_project_id(
        &self,
        project_id: &str,
    ) -> Result<String, StorageError> {
        if let Some(source_id) = self
            .connection
            .query_row(
                "SELECT source_project_id FROM worktree_relations
                 WHERE worktree_project_id = ?1",
                [project_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
        {
            return Ok(source_id);
        }
        self.project(project_id).map(|project| project.id)
    }

    pub(crate) fn list_worktree_relations(
        &self,
        source_project_id: &str,
    ) -> Result<Vec<StoredWorktreeRelation>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT source_project_id, worktree_project_id, ownership, branch_name
             FROM worktree_relations WHERE source_project_id = ?1
             ORDER BY created_at_ms, id LIMIT 256",
        )?;
        let relations = statement
            .query_map([source_project_id], |row| {
                Ok(StoredWorktreeRelation {
                    source_project_id: row.get(0)?,
                    worktree_project_id: row.get(1)?,
                    ownership: row.get(2)?,
                    branch_name: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(StorageError::from)?;
        Ok(relations)
    }

    pub(crate) fn retire_worktree_project(
        &mut self,
        source_project_id: &str,
        worktree_project_id: &str,
        expected_ownership: &str,
    ) -> Result<(), StorageError> {
        if source_project_id == worktree_project_id
            || !matches!(expected_ownership, "managed" | "attached")
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let relation_matches = transaction
            .query_row(
                "SELECT 1 FROM worktree_relations
                 WHERE source_project_id = ?1 AND worktree_project_id = ?2
                   AND ownership = ?3",
                params![source_project_id, worktree_project_id, expected_ownership],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        if !relation_matches {
            return Err(StorageError::ProjectNotFound);
        }
        let association_id = transaction
            .query_row(
                "SELECT active_directory_association_id FROM projects WHERE id = ?1",
                [worktree_project_id],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()?
            .ok_or(StorageError::ProjectNotFound)?;
        let timestamp = now_millis();
        if let Some(association_id) = association_id {
            transaction.execute(
                "UPDATE directory_associations
                 SET detached_at_ms = ?1, updated_at_ms = ?1 WHERE id = ?2",
                params![timestamp, association_id],
            )?;
        }
        transaction.execute(
            "UPDATE projects
             SET active_directory_association_id = NULL,
                 archived_at_ms = COALESCE(archived_at_ms, ?1), updated_at_ms = ?1
             WHERE id = ?2",
            params![timestamp, worktree_project_id],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub(crate) fn relink_project(
        &mut self,
        project_id: &str,
        identity: &DirectoryIdentity,
    ) -> Result<(), StorageError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let project_exists = transaction
            .query_row("SELECT 1 FROM projects WHERE id = ?1", [project_id], |_| {
                Ok(())
            })
            .optional()?
            .is_some();
        if !project_exists {
            return Err(StorageError::ProjectNotFound);
        }

        let association_id = transaction
            .query_row(
                "SELECT id FROM directory_associations
                 WHERE project_id = ?1 AND is_primary = 1
                 ORDER BY detached_at_ms IS NULL DESC, updated_at_ms DESC LIMIT 1",
                [project_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .unwrap_or_else(|| Uuid::now_v7().to_string());
        ensure_directory_available(&transaction, identity, Some(&association_id))?;
        let timestamp = now_millis();
        let association_exists = transaction
            .query_row(
                "SELECT 1 FROM directory_associations WHERE id = ?1",
                [&association_id],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        if association_exists {
            update_association(&transaction, &association_id, identity, timestamp)?;
        } else {
            insert_association(
                &transaction,
                &association_id,
                project_id,
                identity,
                timestamp,
            )?;
        }
        transaction.execute(
            "UPDATE projects
             SET active_directory_association_id = ?1, archived_at_ms = NULL,
                 updated_at_ms = ?2
             WHERE id = ?3",
            params![association_id, timestamp, project_id],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub(crate) fn detach_project(&mut self, project_id: &str) -> Result<(), StorageError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let association_id = transaction
            .query_row(
                "SELECT active_directory_association_id FROM projects WHERE id = ?1",
                [project_id],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()?
            .ok_or(StorageError::ProjectNotFound)?;
        let timestamp = now_millis();
        if let Some(association_id) = association_id {
            transaction.execute(
                "UPDATE directory_associations
                 SET detached_at_ms = ?1, updated_at_ms = ?1 WHERE id = ?2",
                params![timestamp, association_id],
            )?;
        }
        transaction.execute(
            "UPDATE projects
             SET active_directory_association_id = NULL, updated_at_ms = ?1
             WHERE id = ?2",
            params![timestamp, project_id],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub(crate) fn archive_project(&mut self, project_id: &str) -> Result<(), StorageError> {
        let timestamp = now_millis();
        let updated = self.connection.execute(
            "UPDATE projects SET archived_at_ms = ?1, updated_at_ms = ?1 WHERE id = ?2",
            params![timestamp, project_id],
        )?;
        if updated == 0 {
            return Err(StorageError::ProjectNotFound);
        }
        Ok(())
    }

    pub(crate) fn insert_conversation_reference(
        &mut self,
        reference: &ConversationReference<'_>,
    ) -> Result<(), StorageError> {
        let timestamp = now_millis();
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute(
            "INSERT INTO conversation_references (
                id, project_id, codex_thread_id, active_turn_id, model_id,
                reasoning_effort, sandbox_mode, approval_policy, status,
                created_at_ms, updated_at_ms, parent_conversation_id, archived_at_ms,
                selector_availability, selector_mode, selector_user_locked,
                selector_allowed_model_ids_json, selector_reasoning_ceiling
             ) VALUES (
                ?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7, 'thread-started', ?8, ?8, ?9, NULL,
                ?10, ?11, ?12, ?13, ?14
             )",
            params![
                reference.conversation_id,
                reference.project_id,
                reference.codex_thread_id,
                reference.model_id,
                reference.reasoning_effort,
                reference.sandbox_mode,
                reference.approval_policy,
                timestamp,
                reference.parent_conversation_id,
                reference.selection.availability,
                reference.selection.ownership,
                reference.selection.user_locked,
                reference.selection.allowed_model_ids_json,
                reference.selection.reasoning_ceiling,
            ],
        )?;
        transaction.execute(
            "INSERT INTO unified_conversation_metadata (
                id, mode, project_id, conversation_reference_id, created_at_ms, updated_at_ms
             ) VALUES (?1, 'codex', ?2, ?1, ?3, ?3)",
            params![reference.conversation_id, reference.project_id, timestamp],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub(crate) fn insert_chat_conversation_metadata(
        &mut self,
        metadata: &ChatConversationMetadata<'_>,
    ) -> Result<(), StorageError> {
        let timestamp = now_millis();
        self.connection.execute(
            "INSERT INTO unified_conversation_metadata (
                id, mode, project_id, conversation_reference_id, codex_thread_id,
                created_at_ms, updated_at_ms
             ) VALUES (?1, 'chat', NULL, NULL, ?2, ?3, ?3)",
            params![
                metadata.conversation_id,
                metadata.codex_thread_id,
                timestamp
            ],
        )?;
        Ok(())
    }

    pub(crate) fn insert_advisor_conversation_metadata(
        &mut self,
        metadata: &AdvisorConversationMetadata<'_>,
    ) -> Result<(), StorageError> {
        let timestamp = now_millis();
        self.connection.execute(
            "INSERT INTO advisor_conversations (id, codex_thread_id, created_at_ms, updated_at_ms)
             VALUES (?1, ?2, ?3, ?3)",
            params![
                metadata.conversation_id,
                metadata.codex_thread_id,
                timestamp
            ],
        )?;
        Ok(())
    }

    pub(crate) fn insert_advisor_dispatch_proposal(
        &mut self,
        proposal: &AdvisorDispatchProposal,
    ) -> Result<(), StorageError> {
        self.connection.execute(
            "INSERT INTO advisor_dispatch_records (
                id, advisor_conversation_id, target_project_id, request_sha256,
                context_manifest_sha256, capability_manifest_sha256, state,
                requires_explicit_approval, requested_model, requested_reasoning_effort,
                trust, provenance_source, provenance_ref, provenance_commit,
                observed_at_ms, provenance_note, created_at_ms, updated_at_ms,
                decided_at_ms, expires_at_ms
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?9, ?10, ?11, ?12, ?13,
                ?14, ?15, ?16, ?17, ?18, ?19
             )",
            params![
                proposal.id,
                proposal.advisor_conversation_id,
                proposal.target_project_id,
                proposal.prompt_sha256,
                proposal.context_manifest_sha256,
                proposal.capability_manifest_sha256,
                advisor_dispatch_state_value(proposal.state),
                proposal.requested_model,
                proposal.requested_reasoning_effort,
                advisor_trust_value(proposal.provenance.trust),
                advisor_provenance_source_value(proposal.provenance.source),
                proposal.provenance.source_ref,
                proposal.provenance.source_commit,
                proposal.provenance.observed_at_ms,
                proposal.provenance.note,
                proposal.created_at_ms,
                proposal.updated_at_ms,
                proposal.decided_at_ms,
                proposal.expires_at_ms,
            ],
        )?;
        Ok(())
    }

    pub(crate) fn decide_advisor_dispatch_proposal(
        &mut self,
        proposal_id: &str,
        decision: AdvisorDispatchState,
    ) -> Result<AdvisorDispatchProposal, StorageError> {
        if matches!(decision, AdvisorDispatchState::Draft) {
            return Err(StorageError::InvalidStoredValue);
        }
        let timestamp = now_millis();
        let updated = self.connection.execute(
            "UPDATE advisor_dispatch_records
             SET state = ?1, decided_at_ms = ?2, updated_at_ms = ?2
             WHERE id = ?3 AND state = 'draft' AND expires_at_ms > ?2",
            params![
                advisor_dispatch_state_value(decision),
                timestamp,
                proposal_id
            ],
        )?;
        if updated != 1 {
            return Err(StorageError::ProjectNotFound);
        }
        self.load_advisor_dispatch_proposals()?
            .into_iter()
            .find(|proposal| proposal.id == proposal_id)
            .ok_or(StorageError::ProjectNotFound)
    }

    pub(crate) fn advisor_dispatch_proposal(
        &self,
        proposal_id: &str,
    ) -> Result<AdvisorDispatchProposal, StorageError> {
        self.load_advisor_dispatch_proposals()?
            .into_iter()
            .find(|proposal| proposal.id == proposal_id)
            .ok_or(StorageError::ProjectNotFound)
    }

    pub(crate) fn claim_advisor_dispatch_proposal(
        &mut self,
        proposal_id: &str,
    ) -> Result<(), StorageError> {
        let updated = self.connection.execute(
            "UPDATE advisor_dispatch_records
             SET execution_dispatch_state = 'dispatching', updated_at_ms = ?1
             WHERE id = ?2 AND state = 'approved' AND expires_at_ms > ?1
               AND execution_dispatch_state IS NULL",
            params![now_millis(), proposal_id],
        )?;
        if updated == 1 {
            Ok(())
        } else {
            Err(StorageError::ProjectNotFound)
        }
    }

    pub(crate) fn finish_advisor_dispatch_proposal(
        &mut self,
        proposal_id: &str,
        conversation_id: Option<&str>,
    ) -> Result<(), StorageError> {
        let state = if conversation_id.is_some() {
            "started"
        } else {
            "failed-to-start"
        };
        let updated = self.connection.execute(
            "UPDATE advisor_dispatch_records
             SET execution_dispatch_state = ?1, execution_conversation_id = ?2, updated_at_ms = ?3
             WHERE id = ?4 AND execution_dispatch_state = 'dispatching'",
            params![state, conversation_id, now_millis(), proposal_id],
        )?;
        if updated == 1 {
            Ok(())
        } else {
            Err(StorageError::ProjectNotFound)
        }
    }

    pub(crate) fn conversation_reference(
        &self,
        conversation_id: &str,
    ) -> Result<StoredConversationReference, StorageError> {
        self.connection
            .query_row(
                "SELECT id, project_id, codex_thread_id, active_turn_id, model_id,
                        reasoning_effort, sandbox_mode, approval_policy, status,
                        parent_conversation_id, archived_at_ms, created_at_ms, updated_at_ms,
                        selector_availability, selector_mode, selector_user_locked,
                        selector_allowed_model_ids_json, selector_reasoning_ceiling,
                        selector_pending_model_id,
                        selector_pending_reasoning_effort, selector_pending_rationale,
                        selector_pending_provenance, selector_pending_application,
                        selector_pending_requested_at_ms
                 FROM conversation_references WHERE id = ?1",
                [conversation_id],
                stored_conversation_reference,
            )
            .optional()?
            .ok_or(StorageError::InvalidStoredValue)
    }

    pub(crate) fn list_conversation_references(
        &self,
        project_id: Option<&str>,
    ) -> Result<Vec<StoredConversationReference>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, codex_thread_id, active_turn_id, model_id,
                    reasoning_effort, sandbox_mode, approval_policy, status,
                    parent_conversation_id, archived_at_ms, created_at_ms, updated_at_ms,
                    selector_availability, selector_mode, selector_user_locked,
                    selector_allowed_model_ids_json, selector_reasoning_ceiling,
                    selector_pending_model_id,
                    selector_pending_reasoning_effort, selector_pending_rationale,
                    selector_pending_provenance, selector_pending_application,
                    selector_pending_requested_at_ms
             FROM conversation_references
             WHERE (?1 IS NULL OR project_id = ?1)
             ORDER BY updated_at_ms DESC, id
             LIMIT 256",
        )?;
        let references = statement
            .query_map([project_id], stored_conversation_reference)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(StorageError::from)?;
        Ok(references)
    }

    pub(crate) fn update_conversation_turn(
        &mut self,
        conversation_id: &str,
        active_turn_id: &str,
    ) -> Result<(), StorageError> {
        let updated = self.connection.execute(
            "UPDATE conversation_references
             SET active_turn_id = ?1, status = 'running', updated_at_ms = ?2
             WHERE id = ?3",
            params![active_turn_id, now_millis(), conversation_id],
        )?;
        if updated == 0 {
            return Err(StorageError::InvalidStoredValue);
        }
        Ok(())
    }

    pub(crate) fn update_conversation_status(
        &mut self,
        conversation_id: &str,
        status: &str,
    ) -> Result<(), StorageError> {
        let terminal = matches!(status, "completed" | "interrupted" | "blocked" | "failed");
        let updated = self.connection.execute(
            "UPDATE conversation_references
             SET status = ?1,
                 active_turn_id = CASE WHEN ?2 THEN NULL ELSE active_turn_id END,
                 updated_at_ms = ?3
             WHERE id = ?4",
            params![status, terminal, now_millis(), conversation_id],
        )?;
        if updated == 0 {
            return Err(StorageError::InvalidStoredValue);
        }
        Ok(())
    }

    pub(crate) fn update_conversation_archived(
        &mut self,
        conversation_id: &str,
        archived: bool,
    ) -> Result<(), StorageError> {
        let timestamp = now_millis();
        let updated = self.connection.execute(
            "UPDATE conversation_references
             SET archived_at_ms = CASE WHEN ?1 THEN ?2 ELSE NULL END,
                 updated_at_ms = ?2
             WHERE id = ?3 AND active_turn_id IS NULL",
            params![archived, timestamp, conversation_id],
        )?;
        if updated == 0 {
            return Err(StorageError::InvalidStoredValue);
        }
        Ok(())
    }

    pub(crate) fn update_model_selection(
        &mut self,
        conversation_id: &str,
        effective: Option<(&str, &str)>,
        selection: &ConversationSelectionMetadata<'_>,
    ) -> Result<(), StorageError> {
        let pending = selection.pending.as_ref();
        let updated = self.connection.execute(
            "UPDATE conversation_references
             SET model_id = COALESCE(?1, model_id),
                 reasoning_effort = COALESCE(?2, reasoning_effort),
                 selector_availability = ?3,
                 selector_mode = ?4,
                 selector_user_locked = ?5,
                 selector_allowed_model_ids_json = ?6,
                 selector_reasoning_ceiling = ?7,
                 selector_pending_model_id = ?8,
                 selector_pending_reasoning_effort = ?9,
                 selector_pending_rationale = ?10,
                 selector_pending_provenance = ?11,
                 selector_pending_application = ?12,
                 selector_pending_requested_at_ms = ?13,
                 updated_at_ms = ?14
             WHERE id = ?15",
            params![
                effective.map(|value| value.0),
                effective.map(|value| value.1),
                selection.availability,
                selection.ownership,
                selection.user_locked,
                selection.allowed_model_ids_json,
                selection.reasoning_ceiling,
                pending.map(|value| value.model_id),
                pending.map(|value| value.reasoning_effort),
                pending.map(|value| value.rationale),
                pending.map(|value| value.provenance),
                pending.map(|value| value.application),
                pending.map(|value| value.requested_at_ms),
                now_millis(),
                conversation_id,
            ],
        )?;
        if updated == 0 {
            return Err(StorageError::InvalidStoredValue);
        }
        Ok(())
    }

    pub(crate) fn insert_terminal_session(
        &mut self,
        terminal_id: &str,
        project_id: &str,
        title: &str,
        columns: u16,
        rows: u16,
    ) -> Result<(), StorageError> {
        let timestamp = now_millis();
        self.connection.execute(
            "INSERT INTO terminal_sessions (
                id, project_id, title, status, columns, rows, exit_code,
                created_at_ms, updated_at_ms
             ) VALUES (?1, ?2, ?3, 'running', ?4, ?5, NULL, ?6, ?6)",
            params![terminal_id, project_id, title, columns, rows, timestamp],
        )?;
        Ok(())
    }

    pub(crate) fn update_terminal_session(
        &mut self,
        terminal_id: &str,
        status: &str,
        columns: u16,
        rows: u16,
        exit_code: Option<i32>,
    ) -> Result<(), StorageError> {
        if !matches!(
            status,
            "running" | "closing" | "exited" | "interrupted" | "failed"
        ) {
            return Err(StorageError::InvalidStoredValue);
        }
        let updated = self.connection.execute(
            "UPDATE terminal_sessions
             SET status = ?1, columns = ?2, rows = ?3, exit_code = ?4,
                 updated_at_ms = ?5
             WHERE id = ?6",
            params![status, columns, rows, exit_code, now_millis(), terminal_id],
        )?;
        if updated == 0 {
            return Err(StorageError::InvalidStoredValue);
        }
        Ok(())
    }

    pub(crate) fn delete_terminal_session(
        &mut self,
        terminal_id: &str,
    ) -> Result<(), StorageError> {
        let updated = self
            .connection
            .execute("DELETE FROM terminal_sessions WHERE id = ?1", [terminal_id])?;
        if updated == 0 {
            return Err(StorageError::InvalidStoredValue);
        }
        Ok(())
    }

    pub(crate) fn list_terminal_sessions(
        &self,
    ) -> Result<Vec<StoredTerminalSession>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, title, status, columns, rows, exit_code
             FROM terminal_sessions
             ORDER BY updated_at_ms, id
             LIMIT 9",
        )?;
        let sessions = statement
            .query_map([], |row| {
                let columns = row.get::<_, i64>(4)?;
                let rows = row.get::<_, i64>(5)?;
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    columns,
                    rows,
                    row.get::<_, Option<i32>>(6)?,
                ))
            })?
            .map(|row| {
                let (id, project_id, title, status, columns, rows, exit_code) = row?;
                if !matches!(
                    status.as_str(),
                    "running" | "closing" | "exited" | "interrupted" | "failed"
                ) {
                    return Err(StorageError::InvalidStoredValue);
                }
                Ok(StoredTerminalSession {
                    id,
                    project_id,
                    title,
                    status,
                    columns: u16::try_from(columns)
                        .ok()
                        .filter(|value| (2..=500).contains(value))
                        .ok_or(StorageError::InvalidStoredValue)?,
                    rows: u16::try_from(rows)
                        .ok()
                        .filter(|value| (2..=200).contains(value))
                        .ok_or(StorageError::InvalidStoredValue)?,
                    exit_code,
                })
            })
            .collect::<Result<Vec<_>, StorageError>>()?;
        if sessions.len() > 8 {
            return Err(StorageError::InvalidStoredValue);
        }
        Ok(sessions)
    }

    fn load_association(&self, association_id: &str) -> Result<StoredAssociation, StorageError> {
        self.connection
            .query_row(
                "SELECT id, selected_path, resolved_path, expected_access,
                        device_id, inode, filesystem_type, mount_id,
                        git_common_dir, git_worktree_root, git_is_linked_worktree,
                        has_agents_guidance, has_codex_config, accessibility_state
                 FROM directory_associations WHERE id = ?1 AND detached_at_ms IS NULL",
                [association_id],
                |row| {
                    let expected_access = row.get::<_, String>(3)?;
                    let accessibility = row.get::<_, String>(13)?;
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        expected_access,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, Option<String>>(6)?,
                        row.get::<_, Option<String>>(7)?,
                        row.get::<_, Option<String>>(8)?,
                        row.get::<_, Option<String>>(9)?,
                        row.get::<_, bool>(10)?,
                        row.get::<_, bool>(11)?,
                        row.get::<_, bool>(12)?,
                        accessibility,
                    ))
                },
            )
            .optional()?
            .map(|row| {
                DirectoryAccessibilityState::from_storage_value(&row.13)
                    .ok_or(StorageError::InvalidStoredValue)?;
                Ok::<StoredAssociation, StorageError>(StoredAssociation {
                    id: row.0,
                    selected_path: row.1,
                    resolved_path: row.2,
                    expected_access: ExpectedAccess::from_storage_value(&row.3)
                        .ok_or(StorageError::InvalidStoredValue)?,
                    device_id: parse_optional_u64(row.4)?,
                    inode: parse_optional_u64(row.5)?,
                    filesystem_type: row.6,
                    mount_id: parse_optional_u64(row.7)?,
                    git_common_dir: row.8,
                    git_worktree_root: row.9,
                    git_is_linked_worktree: row.10,
                    has_agents_guidance: row.11,
                    has_codex_config: row.12,
                })
            })
            .transpose()?
            .ok_or(StorageError::InvalidStoredValue)
    }
}

fn parse_advisor_context_kind(value: &str) -> Result<AdvisorContextKind, StorageError> {
    match value {
        "project-state" => Ok(AdvisorContextKind::ProjectState),
        "roadmap" => Ok(AdvisorContextKind::Roadmap),
        "current-state" => Ok(AdvisorContextKind::CurrentState),
        "execution-report" => Ok(AdvisorContextKind::ExecutionReport),
        _ => Err(StorageError::InvalidStoredValue),
    }
}

fn parse_advisor_freshness(value: &str) -> Result<AdvisorFreshness, StorageError> {
    match value {
        "current" => Ok(AdvisorFreshness::Current),
        "stale" => Ok(AdvisorFreshness::Stale),
        "unknown" => Ok(AdvisorFreshness::Unknown),
        "conflicting" => Ok(AdvisorFreshness::Conflicting),
        "not-applicable" => Ok(AdvisorFreshness::NotApplicable),
        _ => Err(StorageError::InvalidStoredValue),
    }
}

fn parse_advisor_trust(value: &str) -> Result<AdvisorTrust, StorageError> {
    match value {
        "verified" => Ok(AdvisorTrust::Verified),
        "reported" => Ok(AdvisorTrust::Reported),
        "inferred" => Ok(AdvisorTrust::Inferred),
        "unknown" => Ok(AdvisorTrust::Unknown),
        _ => Err(StorageError::InvalidStoredValue),
    }
}

fn parse_advisor_provenance_source(value: &str) -> Result<AdvisorProvenanceSource, StorageError> {
    match value {
        "git-observation" => Ok(AdvisorProvenanceSource::GitObservation),
        "project-state-snapshot" => Ok(AdvisorProvenanceSource::ProjectStateSnapshot),
        "repository-document" => Ok(AdvisorProvenanceSource::RepositoryDocument),
        "execution-report" => Ok(AdvisorProvenanceSource::ExecutionReport),
        "user-selection" => Ok(AdvisorProvenanceSource::UserSelection),
        "unknown" => Ok(AdvisorProvenanceSource::Unknown),
        _ => Err(StorageError::InvalidStoredValue),
    }
}

fn parse_advisor_dispatch_state(value: &str) -> Result<AdvisorDispatchState, StorageError> {
    match value {
        "draft" => Ok(AdvisorDispatchState::Draft),
        "approved" => Ok(AdvisorDispatchState::Approved),
        "rejected" => Ok(AdvisorDispatchState::Rejected),
        _ => Err(StorageError::InvalidStoredValue),
    }
}

fn parse_advisor_execution_dispatch_state(
    value: &str,
) -> Result<AdvisorExecutionDispatchState, StorageError> {
    match value {
        "dispatching" => Ok(AdvisorExecutionDispatchState::Dispatching),
        "started" => Ok(AdvisorExecutionDispatchState::Started),
        "failed-to-start" => Ok(AdvisorExecutionDispatchState::FailedToStart),
        _ => Err(StorageError::InvalidStoredValue),
    }
}

fn advisor_dispatch_state_value(value: AdvisorDispatchState) -> &'static str {
    match value {
        AdvisorDispatchState::Draft => "draft",
        AdvisorDispatchState::Approved => "approved",
        AdvisorDispatchState::Rejected => "rejected",
    }
}

fn advisor_trust_value(value: AdvisorTrust) -> &'static str {
    match value {
        AdvisorTrust::Verified => "verified",
        AdvisorTrust::Reported => "reported",
        AdvisorTrust::Inferred => "inferred",
        AdvisorTrust::Unknown => "unknown",
    }
}

fn advisor_provenance_source_value(value: AdvisorProvenanceSource) -> &'static str {
    match value {
        AdvisorProvenanceSource::GitObservation => "git-observation",
        AdvisorProvenanceSource::ProjectStateSnapshot => "project-state-snapshot",
        AdvisorProvenanceSource::RepositoryDocument => "repository-document",
        AdvisorProvenanceSource::ExecutionReport => "execution-report",
        AdvisorProvenanceSource::UserSelection => "user-selection",
        AdvisorProvenanceSource::Unknown => "unknown",
    }
}

/// Resolves task creation's project binding entirely from native conversation
/// metadata while the task insert transaction is held. A conversation must be
/// live and its project must still have an active, attached association.
fn task_context_project_id(
    transaction: &Transaction<'_>,
    conversation_id: &str,
) -> Result<String, StorageError> {
    transaction
        .query_row(
            "SELECT conversation.project_id
             FROM conversation_references AS conversation
             JOIN projects AS project ON project.id = conversation.project_id
             JOIN directory_associations AS association
               ON association.id = project.active_directory_association_id
             WHERE conversation.id = ?1
               AND conversation.archived_at_ms IS NULL
               AND conversation.status IN ('thread-started', 'running')
               AND project.archived_at_ms IS NULL
               AND association.detached_at_ms IS NULL",
            [conversation_id],
            |row| row.get(0),
        )
        .optional()?
        .filter(|project_id: &String| is_uuid_v7(project_id))
        .ok_or(StorageError::ProjectNotFound)
}

/// Validates the UI-selected project at the native persistence boundary. A
/// task may bind only to a live, non-archived project with an attached active
/// directory association; callers cannot create a dangling task binding.
fn task_catalog_project_id(
    transaction: &Transaction<'_>,
    project_id: &str,
) -> Result<String, StorageError> {
    transaction
        .query_row(
            "SELECT project.id
             FROM projects AS project
             JOIN directory_associations AS association
               ON association.id = project.active_directory_association_id
             WHERE project.id = ?1
               AND project.archived_at_ms IS NULL
               AND association.detached_at_ms IS NULL",
            [project_id],
            |row| row.get(0),
        )
        .optional()?
        .filter(|id: &String| is_uuid_v7(id))
        .ok_or(StorageError::ProjectNotFound)
}

/// Resolves an Advisor origin only from the native execution conversation that
/// is creating the task. The dispatch record, its Advisor conversation, and
/// its target project must all agree before the origin pair can be inserted.
/// A conversation without an Advisor dispatch remains an ordinary native
/// project-bound task context.
fn task_advisor_dispatch_origin(
    transaction: &Transaction<'_>,
    execution_conversation_id: &str,
    project_id: &str,
) -> Result<Option<(String, String)>, StorageError> {
    let mut statement = transaction.prepare(
        "SELECT dispatch.id, dispatch.advisor_conversation_id, dispatch.target_project_id,
                dispatch.state, dispatch.execution_dispatch_state
         FROM advisor_dispatch_records AS dispatch
         JOIN advisor_conversations AS advisor
           ON advisor.id = dispatch.advisor_conversation_id
         WHERE dispatch.execution_conversation_id = ?1",
    )?;
    let mut rows = statement.query([execution_conversation_id])?;
    let Some(row) = rows.next()? else {
        return Ok(None);
    };
    let dispatch_id: String = row.get(0)?;
    let advisor_conversation_id: String = row.get(1)?;
    let target_project_id: String = row.get(2)?;
    let state: String = row.get(3)?;
    let execution_state: Option<String> = row.get(4)?;
    if rows.next()?.is_some()
        || !is_uuid_v7(&dispatch_id)
        || !is_uuid_v7(&advisor_conversation_id)
        || target_project_id != project_id
        || state != "approved"
        || execution_state.as_deref() != Some("started")
    {
        return Err(StorageError::InvalidStoredValue);
    }
    Ok(Some((advisor_conversation_id, dispatch_id)))
}

fn approval_presentation_details_for_task(
    tx: &Transaction<'_>,
    task_id: &str,
) -> Result<LocalReviewApprovalPresentationDetails, StorageError> {
    let (project_id, advisor_conversation_id, dispatch_id): (Option<String>, Option<String>, Option<String>) = tx.query_row(
        "SELECT project_id, origin_advisor_conversation_id, origin_advisor_dispatch_record_id FROM task_records WHERE id = ?1",
        [task_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;
    let (Some(project_id), Some(origin_conversation_id), Some(origin_dispatch_id)) =
        (project_id, advisor_conversation_id, dispatch_id)
    else {
        return Err(StorageError::InvalidStoredValue);
    };
    if !is_uuid_v7(&project_id)
        || !is_uuid_v7(&origin_conversation_id)
        || !is_uuid_v7(&origin_dispatch_id)
    {
        return Err(StorageError::InvalidStoredValue);
    }
    #[expect(clippy::type_complexity, reason = "fixed advisor dispatch row is immediately validated")]
    let row: Option<(String, String, String, String, String, String, i64, Option<i64>, i64, Option<String>, Option<String>, String, String)> = tx.query_row(
        "SELECT id, advisor_conversation_id, target_project_id, request_sha256, context_manifest_sha256, capability_manifest_sha256, requires_explicit_approval, decided_at_ms, expires_at_ms, execution_dispatch_state, execution_conversation_id, state, provenance_source FROM advisor_dispatch_records WHERE id = ?1",
        [&origin_dispatch_id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?,row.get(6)?,row.get(7)?,row.get(8)?,row.get(9)?,row.get(10)?,row.get(11)?,row.get(12)?)),
    ).optional()?;
    let Some((
        id,
        conversation_id,
        target_project_id,
        request_sha,
        context_sha,
        capability_sha,
        explicit,
        decided_at,
        expires_at,
        execution_state,
        execution_id,
        state,
        provenance_source,
    )) = row
    else {
        return Err(StorageError::TaskNotFound);
    };
    if id != origin_dispatch_id
        || conversation_id != origin_conversation_id
        || target_project_id != project_id
        || !is_uuid_v7(&id)
        || !is_uuid_v7(&conversation_id)
        || !valid_lower_sha256(&request_sha)
        || !valid_lower_sha256(&context_sha)
        || !valid_lower_sha256(&capability_sha)
        || explicit != 1
        || state != "approved"
        || decided_at.is_none()
        || expires_at <= decided_at.unwrap_or_default()
        || execution_state.as_deref() != Some("started")
        || !execution_id.as_deref().is_some_and(is_uuid_v7)
        || parse_advisor_provenance_source(&provenance_source).is_err()
    {
        return Err(StorageError::InvalidStoredValue);
    }
    let execution_matches: Option<i64> = tx
        .query_row(
            "SELECT 1 FROM conversation_references WHERE id = ?1 AND project_id = ?2",
            params![execution_id, project_id],
            |row| row.get(0),
        )
        .optional()?;
    if execution_matches != Some(1) {
        return Err(StorageError::InvalidStoredValue);
    }
    Ok(LocalReviewApprovalPresentationDetails {
        approval_state: LocalReviewEvidenceApprovalState::Approved,
        request_present: true,
        decision_present: true,
        dispatch_present: true,
        execution_present: true,
    })
}

fn is_uuid_v7(value: &str) -> bool {
    Uuid::parse_str(value).is_ok_and(|uuid| uuid.get_version_num() == 7)
}

fn package_validation_state_value(value: LocalReviewEvidenceCheckState) -> &'static str {
    match value {
        LocalReviewEvidenceCheckState::Passed => "passed",
        LocalReviewEvidenceCheckState::Failed => "failed",
        LocalReviewEvidenceCheckState::Skipped => "skipped",
        LocalReviewEvidenceCheckState::Unavailable => "unavailable",
    }
}

fn package_validation_state(value: &str) -> Result<LocalReviewEvidenceCheckState, StorageError> {
    match value {
        "passed" => Ok(LocalReviewEvidenceCheckState::Passed),
        "failed" => Ok(LocalReviewEvidenceCheckState::Failed),
        "skipped" => Ok(LocalReviewEvidenceCheckState::Skipped),
        "unavailable" => Ok(LocalReviewEvidenceCheckState::Unavailable),
        _ => Err(StorageError::InvalidStoredValue),
    }
}

fn valid_package_version(value: &str, permit_debian_tilde: bool) -> bool {
    let allowed = |byte: u8| {
        byte.is_ascii_alphanumeric()
            || matches!(byte, b'.' | b'+' | b'-')
            || (permit_debian_tilde && matches!(byte, b':' | b'~'))
    };
    (1..=64).contains(&value.len())
        && value.as_bytes().first().is_some_and(u8::is_ascii_digit)
        && value.bytes().all(allowed)
}

fn valid_lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalInstalledHostAttempt<'a> {
    domain: &'static str,
    candidate_identity_sha256: &'a str,
    outcome: &'a str,
    facts: CanonicalInstalledHostAttemptFacts<'a>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalInstalledHostAttemptFacts<'a> {
    kind: &'static str,
    schema_version: u8,
    package_state: Option<&'a str>,
    version_match: Option<bool>,
    ownership_verified: Option<bool>,
    permissions_safe: Option<bool>,
    package_integrity_verified: Option<bool>,
}

fn installed_host_attempt_identity(
    candidate_identity_sha256: &str,
    outcome: LocalReviewEvidenceCheckState,
    facts: Option<&PackageValidationInstalledHostFacts>,
) -> Result<String, StorageError> {
    let outcome = package_validation_state_value(outcome);
    let bytes = serde_json::to_vec(&CanonicalInstalledHostAttempt {
        domain: INSTALLED_HOST_ATTEMPT_DOMAIN,
        candidate_identity_sha256,
        outcome,
        facts: CanonicalInstalledHostAttemptFacts {
            kind: "installed-host",
            schema_version: 1,
            package_state: facts.map(|value| value.package_state.as_str()),
            version_match: facts.map(|value| value.version_match),
            ownership_verified: facts.map(|value| value.ownership_verified),
            permissions_safe: facts.map(|value| value.permissions_safe),
            package_integrity_verified: facts.map(|value| value.package_integrity_verified),
        },
    })
    .map_err(|_| StorageError::InvalidStoredValue)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn validation_is_complete(input: &PackageValidationRecordInput) -> bool {
    [
        input.manifest_state,
        input.checksum_state,
        input.abi_state,
        input.provenance_state,
        input.visible_launch_state,
        input.installed_host_state,
    ]
    .into_iter()
    .all(|state| state == LocalReviewEvidenceCheckState::Passed)
}

fn validate_package_validation_input(
    input: &PackageValidationRecordInput,
) -> Result<(), StorageError> {
    if !valid_lower_sha256(&input.candidate_identity_sha256) {
        return Err(StorageError::InvalidStoredValue);
    }
    validate_package_validation_summary_input(input)
}

fn validate_package_validation_summary_input(
    input: &PackageValidationRecordInput,
) -> Result<(), StorageError> {
    if !valid_package_version(&input.application_version, false)
        || !valid_package_version(&input.debian_version, true)
        || input.artifact_count > PACKAGE_ARTIFACT_COUNT_LIMIT
        || input.validation_complete != validation_is_complete(input)
        || (input.validation_complete && input.artifact_count != PACKAGE_ARTIFACT_COUNT_LIMIT)
        || input
            .supersedes_record_id
            .as_deref()
            .is_some_and(|id| !is_uuid_v7(id))
        || matches!(input.validation_phase, PackageValidationPhase::Unprivileged)
            && (input.installed_host_state != LocalReviewEvidenceCheckState::Unavailable
                || input.validation_complete
                || input.supersedes_record_id.is_some())
        || matches!(
            input.validation_phase,
            PackageValidationPhase::InstalledHost
        ) && input.supersedes_record_id.is_none()
        || matches!(input.validation_phase, PackageValidationPhase::Unprivileged)
            && input.attempt_identity_sha256.is_some()
        || input
            .attempt_identity_sha256
            .as_deref()
            .is_some_and(|value| !valid_lower_sha256(value))
        || matches!(input.validation_phase, PackageValidationPhase::Unprivileged)
            && input.installed_host_facts.is_some()
        || matches!(
            input.validation_phase,
            PackageValidationPhase::InstalledHost
        ) && input.installed_host_state == LocalReviewEvidenceCheckState::Passed
            && input.installed_host_facts.as_ref().is_none_or(|facts| {
                facts.package_state != "installed"
                    || !facts.version_match
                    || !facts.ownership_verified
                    || !facts.permissions_safe
                    || !facts.package_integrity_verified
            })
    {
        return Err(StorageError::InvalidStoredValue);
    }
    Ok(())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalPackageValidationRecord<'a> {
    id: &'a str,
    project_id: &'a str,
    application_version: &'a str,
    debian_version: &'a str,
    manifest_state: LocalReviewEvidenceCheckState,
    checksum_state: LocalReviewEvidenceCheckState,
    abi_state: LocalReviewEvidenceCheckState,
    provenance_state: LocalReviewEvidenceCheckState,
    visible_launch_state: LocalReviewEvidenceCheckState,
    installed_host_state: LocalReviewEvidenceCheckState,
    artifact_count: u8,
    validation_complete: bool,
    created_at_ms: i64,
    supersedes_record_id: Option<&'a str>,
}

fn package_validation_record_digest(
    id: &str,
    project_id: &str,
    input: &PackageValidationRecordInput,
    created_at_ms: i64,
) -> Result<String, StorageError> {
    let bytes = serde_json::to_vec(&CanonicalPackageValidationRecord {
        id,
        project_id,
        application_version: &input.application_version,
        debian_version: &input.debian_version,
        manifest_state: input.manifest_state,
        checksum_state: input.checksum_state,
        abi_state: input.abi_state,
        provenance_state: input.provenance_state,
        visible_launch_state: input.visible_launch_state,
        installed_host_state: input.installed_host_state,
        artifact_count: input.artifact_count,
        validation_complete: input.validation_complete,
        created_at_ms,
        supersedes_record_id: input.supersedes_record_id.as_deref(),
    })
    .map_err(|_| StorageError::InvalidStoredValue)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn package_validation_summary_record(
    connection: &Connection,
    id: &str,
) -> Result<PackageValidationSummary, StorageError> {
    let row = connection
        .query_row(
            "SELECT project_id, application_version, debian_version,
                    manifest_state, checksum_state, abi_state, provenance_state,
                    visible_launch_state, installed_host_state, artifact_count,
                    validation_complete, record_sha256, created_at_ms, supersedes_record_id
             FROM project_package_validation_summaries WHERE id = ?1",
            [id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, u8>(9)?,
                    row.get::<_, bool>(10)?,
                    row.get::<_, String>(11)?,
                    row.get::<_, i64>(12)?,
                    row.get::<_, Option<String>>(13)?,
                ))
            },
        )
        .optional()?
        .ok_or(StorageError::InvalidStoredValue)?;
    let association = connection
        .query_row(
            "SELECT candidate_identity_sha256, validation_phase, attempt_identity_sha256
             FROM project_package_validation_candidate_identities
             WHERE package_validation_summary_id = ?1",
            [id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .optional()?;
    let (candidate_identity_sha256, validation_phase, attempt_identity_sha256) = match association {
        Some((identity, phase, attempt)) => {
            let phase = package_validation_phase(&phase)?;
            let attempt = (phase == PackageValidationPhase::InstalledHost).then_some(attempt);
            (identity, phase, attempt)
        }
        None => (String::new(), PackageValidationPhase::Unprivileged, None),
    };
    let input = PackageValidationRecordInput {
        candidate_identity_sha256,
        validation_phase,
        attempt_identity_sha256,
        installed_host_facts: (validation_phase == PackageValidationPhase::InstalledHost
            && package_validation_state(&row.8)? == LocalReviewEvidenceCheckState::Passed)
            .then(|| PackageValidationInstalledHostFacts {
                package_state: "installed".to_owned(),
                version_match: true,
                ownership_verified: true,
                permissions_safe: true,
                package_integrity_verified: true,
            }),
        application_version: row.1,
        debian_version: row.2,
        manifest_state: package_validation_state(&row.3)?,
        checksum_state: package_validation_state(&row.4)?,
        abi_state: package_validation_state(&row.5)?,
        provenance_state: package_validation_state(&row.6)?,
        visible_launch_state: package_validation_state(&row.7)?,
        installed_host_state: package_validation_state(&row.8)?,
        artifact_count: row.9,
        validation_complete: row.10,
        supersedes_record_id: row.13,
    };
    if !is_uuid_v7(id)
        || !is_uuid_v7(&row.0)
        || row.12 < 0
        || !valid_lower_sha256(&row.11)
        || validate_package_validation_summary_input(&input).is_err()
        || package_validation_record_digest(id, &row.0, &input, row.12)? != row.11
    {
        return Err(StorageError::InvalidStoredValue);
    }
    Ok(PackageValidationSummary {
        id: id.to_owned(),
        project_id: row.0,
        input,
        created_at_ms: row.12,
        record_sha256: row.11,
    })
}

fn prune_package_validation_summaries(
    transaction: &Transaction<'_>,
    project_id: &str,
    now: i64,
    incoming_supersedes: Option<&str>,
) -> Result<(), StorageError> {
    let count: i64 = transaction.query_row(
        "SELECT count(*) FROM project_package_validation_summaries WHERE project_id = ?1",
        [project_id],
        |row| row.get(0),
    )?;
    let required = (count + 1 - PACKAGE_VALIDATION_RECORD_LIMIT as i64).max(0) as usize;
    if required == 0 {
        return Ok(());
    }
    let newest: Option<String> = transaction
        .query_row(
            "SELECT id FROM project_package_validation_summaries
             WHERE project_id = ?1 ORDER BY created_at_ms DESC, id DESC LIMIT 1",
            [project_id],
            |row| row.get(0),
        )
        .optional()?;
    let newest_complete: Option<String> = transaction
        .query_row(
            "SELECT id FROM project_package_validation_summaries
             WHERE project_id = ?1 AND validation_complete = 1
             ORDER BY created_at_ms DESC, id DESC LIMIT 1",
            [project_id],
            |row| row.get(0),
        )
        .optional()?;
    let protected = [
        newest,
        newest_complete,
        incoming_supersedes.map(str::to_owned),
    ]
    .into_iter()
    .flatten()
    .collect::<HashSet<_>>();
    let cutoff = now.saturating_sub(PACKAGE_VALIDATION_PROTECTION_MS);
    let mut statement = transaction.prepare(
        "SELECT id FROM project_package_validation_summaries AS record
         WHERE record.project_id = ?1
           AND record.created_at_ms < ?2
           AND NOT EXISTS (
               SELECT 1 FROM project_package_validation_summaries AS child
               WHERE child.supersedes_record_id = record.id
           )
           AND NOT EXISTS (
               SELECT 1 FROM project_package_validation_candidate_identities AS identity
               WHERE identity.package_validation_summary_id = record.id
           )
         ORDER BY record.created_at_ms, record.id",
    )?;
    let candidates = statement
        .query_map(params![project_id, cutoff], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|id| !protected.contains(id))
        .take(required)
        .collect::<Vec<_>>();
    if candidates.len() != required {
        return Err(StorageError::TaskCapacity);
    }
    for id in candidates {
        if transaction.execute(
            "DELETE FROM project_package_validation_summaries WHERE id = ?1 AND project_id = ?2",
            params![id, project_id],
        )? != 1
        {
            return Err(StorageError::InvalidStoredValue);
        }
    }
    Ok(())
}

fn apply_migrations(connection: &mut Connection) -> Result<(), StorageError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    transaction.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            applied_at_ms INTEGER NOT NULL
        );",
    )?;
    let applied: Vec<(i64, String)> = {
        let mut statement =
            transaction.prepare("SELECT version, name FROM schema_migrations ORDER BY version")?;
        let rows = statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<_, _>>()?;
        rows
    };
    if applied
        .iter()
        .any(|(version, _)| *version > MIGRATIONS.len() as i64)
    {
        return Err(StorageError::FutureSchema);
    }
    for (index, (version, name)) in applied.iter().enumerate() {
        let expected = MIGRATIONS.get(index).ok_or(StorageError::FutureSchema)?;
        if *version != expected.0 || name != expected.1 {
            return Err(StorageError::InvalidStoredValue);
        }
    }

    for (version, name, sql) in MIGRATIONS.iter().skip(applied.len()) {
        transaction.execute_batch(sql)?;
        if *version == 13 {
            backfill_local_review_evidence_envelopes(&transaction)?;
        }
        transaction.execute(
            "INSERT INTO schema_migrations(version, name, applied_at_ms)
             VALUES (?1, ?2, ?3)",
            params![version, name, now_millis()],
        )?;
    }
    transaction.commit()?;
    Ok(())
}

fn backfill_local_review_evidence_envelopes(
    transaction: &Transaction<'_>,
) -> Result<(), StorageError> {
    let mut statement = transaction.prepare(
        "SELECT id, title, content, sha256, byte_size
         FROM local_review_items
         WHERE class = 'evidence' AND provenance = 'manual-validation-summary'",
    )?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Vec<u8>>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    drop(statement);
    for (id, title, content, sha256, byte_size) in rows {
        if byte_size != content.len() as i64 || sha256 != review_digest(&content) {
            return Err(StorageError::InvalidStoredValue);
        }
        let value: serde_json::Value =
            serde_json::from_slice(&content).map_err(|_| StorageError::InvalidStoredValue)?;
        let object = value.as_object().ok_or(StorageError::InvalidStoredValue)?;
        if object.len() != 4
            || object.get("schemaVersion").and_then(|value| value.as_i64()) != Some(1)
            || object.get("source").and_then(|value| value.as_str())
                != Some("manual-validation-summary")
            || object.get("title").and_then(|value| value.as_str()) != Some(title.as_str())
        {
            return Err(StorageError::InvalidStoredValue);
        }
        let summary = object
            .get("summary")
            .and_then(|value| value.as_str())
            .ok_or(StorageError::InvalidStoredValue)?;
        let bytes = manual_evidence_envelope_bytes(&title, summary)?;
        if bytes.len() > REVIEW_EVIDENCE_BYTES_LIMIT {
            return Err(StorageError::InvalidStoredValue);
        }
        transaction.execute(
            "UPDATE local_review_items
             SET evidence_source = 'manual-validation-summary', content = ?1,
                 sha256 = ?2, byte_size = ?3
             WHERE id = ?4",
            params![bytes, review_digest(&bytes), bytes.len() as i64, id],
        )?;
    }
    let invalid: i64 = transaction.query_row(
        "SELECT count(*) FROM local_review_items
         WHERE (class = 'evidence' AND evidence_source IS NULL)
            OR (class != 'evidence' AND evidence_source IS NOT NULL)",
        [],
        |row| row.get(0),
    )?;
    if invalid != 0 {
        return Err(StorageError::InvalidStoredValue);
    }
    Ok(())
}

fn verify_schema(connection: &Connection) -> Result<(), StorageError> {
    for table in [
        "schema_migrations",
        "projects",
        "directory_associations",
        "conversation_references",
        "unified_conversation_metadata",
        "advisor_conversations",
        "advisor_context_references",
        "advisor_dispatch_records",
        "worktree_relations",
        "terminal_sessions",
        "task_records",
        "task_plans",
        "controlled_browser_verification_attempts",
        "controlled_browser_verification_audit",
    ] {
        let exists = connection
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [table],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        if !exists {
            return Err(StorageError::InvalidStoredValue);
        }
    }
    Ok(())
}

fn recover_interrupted_conversations(connection: &Connection) -> Result<(), StorageError> {
    let timestamp = now_millis();
    connection.execute(
        "UPDATE conversation_references
         SET active_turn_id = NULL, status = 'interrupted', updated_at_ms = ?1
         WHERE status IN ('thread-started', 'running', 'stopping')",
        [timestamp],
    )?;
    Ok(())
}

fn recover_interrupted_terminals(connection: &Connection) -> Result<(), StorageError> {
    connection.execute(
        "UPDATE terminal_sessions
         SET status = 'interrupted', exit_code = NULL, updated_at_ms = ?1
         WHERE status IN ('running', 'closing')",
        [now_millis()],
    )?;
    Ok(())
}

/// A prepared fictional operation is intentionally process-local. A restart
/// cannot restore its in-memory authorization, so recovery closes it as
/// expired and preserves only content-free audit evidence.
fn recover_interrupted_fictional_connector_operations(
    connection: &Connection,
) -> Result<(), StorageError> {
    let timestamp = now_millis();
    let mut statement = connection.prepare(
        "SELECT id, binding_id FROM fictional_connector_operations WHERE state = 'prepared'",
    )?;
    let pending = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    drop(statement);
    if pending.is_empty() {
        return Ok(());
    }
    let transaction = connection.unchecked_transaction()?;
    for (operation_id, binding_id) in pending {
        transaction.execute(
            "UPDATE fictional_connector_operations SET state = 'expired', completed_at_ms = ?1 WHERE id = ?2 AND state = 'prepared'",
            params![timestamp, operation_id],
        )?;
        transaction.execute(
            "UPDATE fictional_connector_bindings SET state = 'expired', updated_at_ms = ?1 WHERE id = ?2 AND state = 'ready'",
            params![timestamp, binding_id],
        )?;
        transaction.execute(
            "INSERT INTO fictional_connector_audit (id, operation_id, binding_id, event_kind, outcome, evidence_digest, created_at_ms) VALUES (?1, ?2, ?3, 'process-recovery', 'expired', ?4, ?5)",
            params![Uuid::now_v7().to_string(), operation_id, binding_id, review_digest(b"fictional-process-recovery-v1"), timestamp],
        )?;
    }
    transaction.commit()?;
    Ok(())
}

fn recover_interrupted_controlled_browser_verifications(
    connection: &Connection,
) -> Result<(), StorageError> {
    let timestamp = now_millis();
    let pending = {
        let mut statement = connection.prepare(
            "SELECT id, request_digest FROM controlled_browser_verification_attempts WHERE state = 'prepared'",
        )?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };
    if pending.is_empty() {
        return Ok(());
    }
    let transaction = connection.unchecked_transaction()?;
    for (attempt_id, digest) in pending {
        transaction.execute(
            "UPDATE controlled_browser_verification_attempts SET state = 'expired', completed_at_ms = ?1 WHERE id = ?2 AND state = 'prepared'",
            params![timestamp, attempt_id],
        )?;
        transaction.execute(
            "INSERT INTO controlled_browser_verification_audit (id,attempt_id,event_kind,outcome,evidence_digest,created_at_ms) VALUES (?1,?2,'process-recovery','expired',?3,?4)",
            params![Uuid::now_v7().to_string(), attempt_id, digest, timestamp],
        )?;
    }
    transaction.commit()?;
    Ok(())
}

/// Context transmission authorizations are process-owned. On restart no
/// in-memory confirmation handle may be reconstructed, so every prepared
/// bundle is closed as expired while retaining only its existing audit digest.
fn recover_interrupted_context_bundles(connection: &Connection) -> Result<(), StorageError> {
    let timestamp = now_millis();
    let pending = {
        let mut statement = connection
            .prepare("SELECT id, bundle_digest FROM context_bundles WHERE state IN ('prepared','awaiting_review','awaiting_confirmation','dispatching')")?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };
    if pending.is_empty() {
        return Ok(());
    }
    let transaction = connection.unchecked_transaction()?;
    for (bundle_id, digest) in pending {
        transaction.execute(
            "UPDATE context_bundles SET state = 'expired', canonical_bytes = NULL, completed_at_ms = ?1 WHERE id = ?2 AND state IN ('prepared','awaiting_review','awaiting_confirmation','dispatching')",
            params![timestamp, bundle_id],
        )?;
        transaction.execute(
            "INSERT INTO context_bundle_audit (id,bundle_id,event_kind,outcome,evidence_digest,created_at_ms) VALUES (?1,?2,'process-recovery','expired',?3,?4)",
            params![Uuid::now_v7().to_string(), bundle_id, digest, timestamp],
        )?;
    }
    transaction.commit()?;
    Ok(())
}

fn stored_conversation_reference(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<StoredConversationReference> {
    Ok(StoredConversationReference {
        id: row.get(0)?,
        project_id: row.get(1)?,
        codex_thread_id: row.get(2)?,
        active_turn_id: row.get(3)?,
        model_id: row.get(4)?,
        reasoning_effort: row.get(5)?,
        sandbox_mode: row.get(6)?,
        approval_policy: row.get(7)?,
        status: row.get(8)?,
        parent_conversation_id: row.get(9)?,
        archived: row.get::<_, Option<i64>>(10)?.is_some(),
        created_at_ms: row.get(11)?,
        updated_at_ms: row.get(12)?,
        selector_availability: row.get(13)?,
        selector_mode: row.get(14)?,
        selector_user_locked: row.get(15)?,
        selector_allowed_model_ids_json: row.get(16)?,
        selector_reasoning_ceiling: row.get(17)?,
        selector_pending_model_id: row.get(18)?,
        selector_pending_reasoning_effort: row.get(19)?,
        selector_pending_rationale: row.get(20)?,
        selector_pending_provenance: row.get(21)?,
        selector_pending_application: row.get(22)?,
        selector_pending_requested_at_ms: row.get(23)?,
    })
}

fn ensure_directory_available(
    connection: &Connection,
    identity: &DirectoryIdentity,
    excluding_association_id: Option<&str>,
) -> Result<(), StorageError> {
    let resolved_path = path_text(&identity.resolved_path)?;
    let duplicate = connection
        .query_row(
            "SELECT id FROM directory_associations
             WHERE resolved_path = ?1 AND detached_at_ms IS NULL
               AND (?2 IS NULL OR id <> ?2)
             LIMIT 1",
            params![resolved_path, excluding_association_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    if duplicate.is_some() {
        return Err(StorageError::DuplicateDirectory);
    }
    Ok(())
}

fn insert_association(
    transaction: &Transaction<'_>,
    association_id: &str,
    project_id: &str,
    identity: &DirectoryIdentity,
    timestamp: i64,
) -> Result<(), StorageError> {
    let git_common_dir = identity
        .git
        .as_ref()
        .map(|git| path_text(&git.common_dir))
        .transpose()?;
    let git_worktree_root = identity
        .git
        .as_ref()
        .map(|git| path_text(&git.worktree_root))
        .transpose()?;
    transaction.execute(
        "INSERT INTO directory_associations (
            id, project_id, selected_path, resolved_path, role, is_primary,
            expected_access, device_id, inode, filesystem_type, mount_id,
            git_common_dir, git_worktree_root, git_is_linked_worktree,
            has_agents_guidance, has_codex_config, accessibility_state,
            last_verified_at_ms, detached_at_ms, created_at_ms, updated_at_ms
         ) VALUES (
            ?1, ?2, ?3, ?4, 'primary', 1, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
            ?12, ?13, ?14, ?15, ?16, NULL, ?16, ?16
         )",
        params![
            association_id,
            project_id,
            path_text(&identity.selected_path)?,
            path_text(&identity.resolved_path)?,
            ExpectedAccess::ReadWrite.as_storage_value(),
            identity.device_id.to_string(),
            identity.inode.to_string(),
            identity.filesystem_type,
            identity.mount_id.map(|value| value.to_string()),
            git_common_dir,
            git_worktree_root,
            identity
                .git
                .as_ref()
                .is_some_and(|git| git.is_linked_worktree),
            identity.has_agents_guidance,
            identity.has_codex_config,
            identity.accessibility.as_storage_value(),
            timestamp,
        ],
    )?;
    Ok(())
}

fn update_association(
    transaction: &Transaction<'_>,
    association_id: &str,
    identity: &DirectoryIdentity,
    timestamp: i64,
) -> Result<(), StorageError> {
    let git_common_dir = identity
        .git
        .as_ref()
        .map(|git| path_text(&git.common_dir))
        .transpose()?;
    let git_worktree_root = identity
        .git
        .as_ref()
        .map(|git| path_text(&git.worktree_root))
        .transpose()?;
    transaction.execute(
        "UPDATE directory_associations SET
            selected_path = ?1, resolved_path = ?2, device_id = ?3, inode = ?4,
            filesystem_type = ?5, mount_id = ?6, git_common_dir = ?7,
            git_worktree_root = ?8, git_is_linked_worktree = ?9,
            has_agents_guidance = ?10, has_codex_config = ?11,
            accessibility_state = ?12, last_verified_at_ms = ?13,
            detached_at_ms = NULL, updated_at_ms = ?13
         WHERE id = ?14",
        params![
            path_text(&identity.selected_path)?,
            path_text(&identity.resolved_path)?,
            identity.device_id.to_string(),
            identity.inode.to_string(),
            identity.filesystem_type,
            identity.mount_id.map(|value| value.to_string()),
            git_common_dir,
            git_worktree_root,
            identity
                .git
                .as_ref()
                .is_some_and(|git| git.is_linked_worktree),
            identity.has_agents_guidance,
            identity.has_codex_config,
            identity.accessibility.as_storage_value(),
            timestamp,
            association_id,
        ],
    )?;
    Ok(())
}

fn path_text(path: &Path) -> Result<&str, StorageError> {
    path.to_str().ok_or(StorageError::InvalidStoredValue)
}

fn parse_optional_u64(value: Option<String>) -> Result<Option<u64>, StorageError> {
    value
        .map(|value| value.parse().map_err(|_| StorageError::InvalidStoredValue))
        .transpose()
}

pub(super) fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(i64::MAX as u128) as i64
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        fs,
        os::unix::fs::PermissionsExt,
        sync::atomic::{AtomicU64, Ordering},
    };

    use rusqlite::{params, Connection};
    use uuid::Uuid;

    use super::{
        apply_migrations, installed_host_attempt_identity, parse_manual_evidence_envelope,
        recover_interrupted_context_bundles, recover_interrupted_controlled_browser_verifications,
        review_digest, review_payload_bytes, valid_task_id, PackageValidationInstalledHostFacts,
        PackageValidationPhase, PackageValidationRecordInput, PackageValidationRecordOutcome,
        PackageValidationSummary, ProjectRepository, StorageError, INITIAL_MIGRATION, MIGRATIONS,
        PACKAGE_VALIDATION_PROTECTION_MS, TASK_CLEANUP_AGE_MS, TASK_COUNT_LIMIT,
        TASK_PAYLOAD_LIMIT,
    };
    use crate::project::task_template::{
        builtins, canonical as canonical_template, digest as template_digest, TaskTemplate,
        TemplateOrigin, TemplateState,
    };
    use crate::project::types::{
        KnowledgeRecordKind, KnowledgeRecordStatus, LocalReviewAnnotationState,
        LocalReviewCollectionState, LocalReviewComparisonState, LocalReviewEvidenceApprovalState,
        LocalReviewEvidenceCheckState, LocalReviewItemClass, LocalReviewItemState,
        LocalReviewLineKind, LocalReviewSourceKind, LocalReviewTextFormat, TaskStatus,
    };
    use crate::project::{
        ChatConversationMetadata, ConversationPendingSelection, ConversationReference,
        ConversationSelectionMetadata,
    };
    use crate::project::{ControlledBrowserVerificationRecord, FictionalConnectorOperationRecord};

    fn local_template(id: String, title: &str) -> TaskTemplate {
        let mut template = TaskTemplate {
            id,
            origin: TemplateOrigin::Local,
            title: title.to_owned(),
            purpose: format!("Purpose for {title}."),
            instructions: format!("Instructions for {title}."),
            version: 1,
            state: TemplateState::Active,
            sha256: String::new(),
        };
        template.sha256 = template_digest(&template).expect("template fixture is canonical");
        template
    }

    fn png(width: u32, height: u32, animated: bool) -> Vec<u8> {
        let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut chunk = |kind: &[u8; 4], payload: &[u8]| {
            bytes.extend_from_slice(&(payload.len() as u32).to_be_bytes());
            bytes.extend_from_slice(kind);
            bytes.extend_from_slice(payload);
            bytes.extend_from_slice(&[0; 4]);
        };
        let mut ihdr = Vec::new();
        ihdr.extend_from_slice(&width.to_be_bytes());
        ihdr.extend_from_slice(&height.to_be_bytes());
        ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
        chunk(b"IHDR", &ihdr);
        if animated {
            chunk(b"acTL", &[0; 8]);
        }
        chunk(b"IDAT", &[0]);
        chunk(b"IEND", &[]);
        bytes
    }

    fn jpeg(width: u16, height: u16, end: bool) -> Vec<u8> {
        let mut bytes = vec![0xff, 0xd8, 0xff, 0xc0, 0, 11, 8];
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&[1, 1, 0x11, 0]);
        if end {
            bytes.extend_from_slice(&[0xff, 0xd9]);
        }
        bytes
    }

    fn insert_live_task_context(repository: &ProjectRepository) -> (String, String) {
        let project_id = "018f0000-0000-7000-8000-000000000101".to_owned();
        let association_id = "018f0000-0000-7000-8000-000000000102";
        let conversation_id = "018f0000-0000-7000-8000-000000000103".to_owned();
        let thread_id = "018f0000-0000-7000-8000-000000000104";
        repository
            .connection
            .execute(
                "INSERT INTO projects (
                    id, display_name, active_directory_association_id, archived_at_ms,
                    created_at_ms, updated_at_ms
                 ) VALUES (?1, 'Bound project', NULL, NULL, 1, 1)",
                [&project_id],
            )
            .expect("project fixture");
        repository
            .connection
            .execute(
                "INSERT INTO directory_associations (
                    id, project_id, selected_path, resolved_path, role, is_primary,
                    expected_access, device_id, inode, filesystem_type, mount_id,
                    git_common_dir, git_worktree_root, git_is_linked_worktree,
                    has_agents_guidance, has_codex_config, accessibility_state,
                    last_verified_at_ms, detached_at_ms, created_at_ms, updated_at_ms
                 ) VALUES (
                    ?1, ?2, '/fixture', '/fixture', 'primary', 1, 'read-write',
                    NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, 0, 'available',
                    1, NULL, 1, 1
                 )",
                params![association_id, project_id],
            )
            .expect("association fixture");
        repository
            .connection
            .execute(
                "UPDATE projects SET active_directory_association_id = ?1 WHERE id = ?2",
                params![association_id, project_id],
            )
            .expect("active association");
        repository
            .connection
            .execute(
                "INSERT INTO conversation_references (
                    id, project_id, codex_thread_id, model_id, reasoning_effort,
                    sandbox_mode, approval_policy, status, created_at_ms, updated_at_ms
                 ) VALUES (?1, ?2, ?3, 'fixture', 'medium', 'read-only',
                    'untrusted', 'thread-started', 1, 1)",
                params![conversation_id, project_id, thread_id],
            )
            .expect("conversation fixture");
        (project_id, conversation_id)
    }

    fn insert_started_advisor_dispatch(
        repository: &ProjectRepository,
        advisor_conversation_id: &str,
        dispatch_id: &str,
        target_project_id: &str,
        execution_conversation_id: &str,
    ) {
        repository
            .connection
            .execute(
                "INSERT INTO advisor_conversations (id, codex_thread_id, created_at_ms, updated_at_ms)
                 VALUES (?1, ?2, 1, 1)",
                params![advisor_conversation_id, format!("advisor-{advisor_conversation_id}")],
            )
            .expect("advisor conversation fixture");
        repository
            .connection
            .execute(
                "INSERT INTO advisor_dispatch_records (
                    id, advisor_conversation_id, target_project_id, request_sha256,
                    context_manifest_sha256, capability_manifest_sha256, state,
                    requires_explicit_approval, requested_model, requested_reasoning_effort,
                    trust, provenance_source, provenance_ref, provenance_commit,
                    observed_at_ms, provenance_note, created_at_ms, updated_at_ms,
                    decided_at_ms, expires_at_ms, execution_dispatch_state,
                    execution_conversation_id
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, 'approved', 1, NULL, NULL,
                    'verified', 'user-selection', 'advisor-dispatch', NULL,
                    1, NULL, 1, 1, 1, 9999999999999, 'started', ?7
                 )",
                params![
                    dispatch_id,
                    advisor_conversation_id,
                    target_project_id,
                    "1".repeat(64),
                    "2".repeat(64),
                    "3".repeat(64),
                    execution_conversation_id,
                ],
            )
            .expect("started advisor dispatch fixture");
    }

    fn package_validation_input(
        complete: bool,
        supersedes_record_id: Option<String>,
    ) -> PackageValidationRecordInput {
        let passed = LocalReviewEvidenceCheckState::Passed;
        static NEXT_TEST_IDENTITY: AtomicU64 = AtomicU64::new(1);
        PackageValidationRecordInput {
            candidate_identity_sha256: format!(
                "{:064x}",
                NEXT_TEST_IDENTITY.fetch_add(1, Ordering::Relaxed)
            ),
            validation_phase: if complete {
                PackageValidationPhase::InstalledHost
            } else {
                PackageValidationPhase::Unprivileged
            },
            attempt_identity_sha256: None,
            installed_host_facts: complete.then(|| PackageValidationInstalledHostFacts {
                package_state: "installed".to_owned(),
                version_match: true,
                ownership_verified: true,
                permissions_safe: true,
                package_integrity_verified: true,
            }),
            application_version: "0.1.0-beta.46".to_owned(),
            debian_version: "0.1.0~beta.46".to_owned(),
            manifest_state: passed,
            checksum_state: if complete {
                passed
            } else {
                LocalReviewEvidenceCheckState::Skipped
            },
            abi_state: if complete {
                passed
            } else {
                LocalReviewEvidenceCheckState::Skipped
            },
            provenance_state: if complete {
                passed
            } else {
                LocalReviewEvidenceCheckState::Skipped
            },
            visible_launch_state: if complete {
                passed
            } else {
                LocalReviewEvidenceCheckState::Skipped
            },
            installed_host_state: if complete {
                passed
            } else {
                LocalReviewEvidenceCheckState::Unavailable
            },
            artifact_count: if complete { 2 } else { 0 },
            validation_complete: complete,
            supersedes_record_id,
        }
    }

    fn headless_predecessor_input(identity: &str) -> PackageValidationRecordInput {
        PackageValidationRecordInput {
            candidate_identity_sha256: identity.to_owned(),
            validation_phase: PackageValidationPhase::Unprivileged,
            attempt_identity_sha256: None,
            installed_host_facts: None,
            application_version: "0.1.0-beta.51".to_owned(),
            debian_version: "0.1.0~beta.51".to_owned(),
            manifest_state: LocalReviewEvidenceCheckState::Passed,
            checksum_state: LocalReviewEvidenceCheckState::Passed,
            abi_state: LocalReviewEvidenceCheckState::Passed,
            provenance_state: LocalReviewEvidenceCheckState::Passed,
            visible_launch_state: LocalReviewEvidenceCheckState::Passed,
            installed_host_state: LocalReviewEvidenceCheckState::Unavailable,
            artifact_count: 2,
            validation_complete: false,
            supersedes_record_id: None,
        }
    }

    #[test]
    fn installed_host_headless_predecessor_requires_one_context_and_one_receipt() {
        let (mut repository, project_id) = ProjectRepository::package_validation_test_repository();
        assert!(repository
            .installed_host_headless_predecessor_for_internal()
            .is_err());

        let first = package_validation_created(
            repository
                .record_package_validation_summary(
                    &project_id,
                    headless_predecessor_input(&"a".repeat(64)),
                )
                .expect("first receipt"),
        );
        let (resolved_project_id, predecessor) = repository
            .installed_host_headless_predecessor_for_internal()
            .expect("single receipt");
        assert_eq!(resolved_project_id, project_id);
        assert_eq!(predecessor.id, first.id);

        package_validation_created(
            repository
                .record_package_validation_summary(
                    &project_id,
                    headless_predecessor_input(&"b".repeat(64)),
                )
                .expect("second receipt"),
        );
        assert!(repository
            .installed_host_headless_predecessor_for_internal()
            .is_err());

        let (mut repository, project_id) = ProjectRepository::package_validation_test_repository();
        let receipt = package_validation_created(
            repository
                .record_package_validation_summary(
                    &project_id,
                    headless_predecessor_input(&"d".repeat(64)),
                )
                .expect("receipt"),
        );
        repository
            .connection
            .execute_batch(
                "DROP TRIGGER project_package_validation_candidate_identities_immutable;",
            )
            .expect("test removes identity immutability trigger");
        repository
            .connection
            .execute(
                "UPDATE project_package_validation_candidate_identities
                    SET attempt_identity_sha256 = ?1
                  WHERE package_validation_summary_id = ?2",
                params!["e".repeat(64), receipt.id],
            )
            .expect("corrupt v18 association");
        assert!(repository
            .installed_host_headless_predecessor_for_internal()
            .is_err());
    }

    #[test]
    fn installed_host_headless_context_requires_exactly_one_live_attached_project() {
        let (repository, project_id) = ProjectRepository::package_validation_test_repository();
        assert_eq!(
            repository
                .installed_host_headless_context_project_id_for_internal()
                .expect("single live attached project"),
            project_id,
        );
    }

    #[test]
    fn installed_host_headless_predecessor_fails_closed_on_corrupt_digest_or_v18_association() {
        let (mut repository, project_id) = ProjectRepository::package_validation_test_repository();
        let receipt = package_validation_created(
            repository
                .record_package_validation_summary(
                    &project_id,
                    headless_predecessor_input(&"c".repeat(64)),
                )
                .expect("receipt"),
        );
        repository
            .connection
            .execute_batch("DROP TRIGGER project_package_validation_summaries_immutable;")
            .expect("test removes immutability trigger");
        repository
            .connection
            .execute(
                "UPDATE project_package_validation_summaries SET record_sha256 = ?1 WHERE id = ?2",
                params!["0".repeat(64), receipt.id],
            )
            .expect("corrupt digest");
        assert!(repository
            .installed_host_headless_predecessor_for_internal()
            .is_err());
    }

    fn package_validation_created(
        outcome: PackageValidationRecordOutcome,
    ) -> PackageValidationSummary {
        match outcome {
            PackageValidationRecordOutcome::Created(summary) => summary,
            PackageValidationRecordOutcome::Existing(_) => panic!("expected fresh record"),
        }
    }

    fn package_validation_input_with_identity(identity: &str) -> PackageValidationRecordInput {
        let mut input = package_validation_input(false, None);
        input.candidate_identity_sha256 = identity.to_owned();
        input
    }

    fn insert_active_project(
        repository: &ProjectRepository,
        project_id: &str,
        association_id: &str,
    ) {
        let fixture_path = format!("/fixture-{project_id}");
        repository
            .connection
            .execute(
                "INSERT INTO projects (
                    id, display_name, active_directory_association_id, archived_at_ms,
                    created_at_ms, updated_at_ms
                 ) VALUES (?1, 'Second project', NULL, NULL, 1, 1)",
                [project_id],
            )
            .expect("second project");
        repository
            .connection
            .execute(
                "INSERT INTO directory_associations (
                    id, project_id, selected_path, resolved_path, role, is_primary,
                    expected_access, device_id, inode, filesystem_type, mount_id,
                    git_common_dir, git_worktree_root, git_is_linked_worktree,
                    has_agents_guidance, has_codex_config, accessibility_state,
                    last_verified_at_ms, detached_at_ms, created_at_ms, updated_at_ms
                 ) VALUES (
                    ?1, ?2, ?3, ?3, 'primary', 1, 'read-write',
                    NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, 0, 'available',
                    1, NULL, 1, 1
                 )",
                params![association_id, project_id, fixture_path],
            )
            .expect("second association");
        repository
            .connection
            .execute(
                "UPDATE projects SET active_directory_association_id = ?1 WHERE id = ?2",
                params![association_id, project_id],
            )
            .expect("second active association");
    }

    #[test]
    fn rejects_a_database_from_a_newer_application() {
        let connection = Connection::open_in_memory().expect("database must open");
        connection
            .execute_batch(
                "CREATE TABLE schema_migrations (
                    version INTEGER PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL,
                    applied_at_ms INTEGER NOT NULL
                 );
                 INSERT INTO schema_migrations VALUES (999, 'future', 0);",
            )
            .expect("future schema fixture must be created");

        assert!(matches!(
            ProjectRepository::from_test_connection(connection),
            Err(StorageError::FutureSchema)
        ));
    }

    #[test]
    fn migrates_an_existing_project_database_through_model_selection() {
        let connection = Connection::open_in_memory().expect("database must open");
        connection
            .execute_batch(
                "CREATE TABLE schema_migrations (
                    version INTEGER PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL,
                    applied_at_ms INTEGER NOT NULL
                 );",
            )
            .expect("migration ledger must be created");
        connection
            .execute_batch(INITIAL_MIGRATION)
            .expect("Milestone 6 schema must be created");
        connection
            .execute(
                "INSERT INTO schema_migrations VALUES (
                    1, 'projects-and-directory-associations', 1
                 )",
                [],
            )
            .expect("Milestone 6 migration must be recorded");

        let repository =
            ProjectRepository::from_test_connection(connection).expect("schema must migrate");
        let migrated: i64 = repository
            .connection
            .query_row(
                "SELECT COUNT(*) FROM schema_migrations
                 WHERE (version = 2 AND name = 'conversation-references')
                    OR (version = 3 AND name = 'session-lifecycle')
                    OR (version = 4 AND name = 'worktree-relations')
                    OR (version = 5 AND name = 'terminal-sessions')
                    OR (version = 6 AND name = 'model-selection')
                    OR (version = 7 AND name = 'unified-conversation-metadata')
                    OR (version = 8 AND name = 'advisor-reference-foundation')
                    OR (version = 9 AND name = 'advisor-approval-controller')
                    OR (version = 10 AND name = 'advisor-one-time-dispatch')
                    OR (version = 11 AND name = 'durable-task-records-v1')
                    OR (version = 12 AND name = 'local-review-collections-v1')",
                [],
                |row| row.get(0),
            )
            .expect("migration ledger must be queryable");
        assert_eq!(migrated, 11);
        let lifecycle_columns: i64 = repository
            .connection
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('conversation_references')
                 WHERE name IN ('parent_conversation_id', 'archived_at_ms')",
                [],
                |row| row.get(0),
            )
            .expect("lifecycle columns must be queryable");
        assert_eq!(lifecycle_columns, 2);
        let selector_columns: i64 = repository
            .connection
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('conversation_references')
                 WHERE name IN (
                    'selector_availability',
                    'selector_mode',
                    'selector_user_locked',
                    'selector_allowed_model_ids_json',
                    'selector_reasoning_ceiling',
                    'selector_pending_model_id',
                    'selector_pending_reasoning_effort',
                    'selector_pending_rationale',
                    'selector_pending_provenance',
                    'selector_pending_application',
                    'selector_pending_requested_at_ms'
                 )",
                [],
                |row| row.get(0),
            )
            .expect("selector columns must be queryable");
        assert_eq!(selector_columns, 11);
        let migrated_availability_default: String = repository
            .connection
            .query_row(
                "SELECT dflt_value FROM pragma_table_info('conversation_references')
                 WHERE name = 'selector_availability'",
                [],
                |row| row.get(0),
            )
            .expect("pre-selector conversations must receive an honest fallback");
        assert_eq!(migrated_availability_default, "'recommendation-only'");
    }

    #[test]
    fn local_review_evidence_source_migration_backfills_manual_rows_atomically() {
        let connection = Connection::open_in_memory().expect("database opens");
        connection.execute_batch("CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY NOT NULL, name TEXT NOT NULL, applied_at_ms INTEGER NOT NULL);").expect("ledger");
        for (version, name, sql) in MIGRATIONS.iter().take(12) {
            connection.execute_batch(sql).expect("v12 migration");
            connection
                .execute(
                    "INSERT INTO schema_migrations VALUES (?1, ?2, 1)",
                    params![version, name],
                )
                .expect("ledger row");
        }
        let collection = "018f0000-0000-7000-8000-000000000001";
        let item = "018f0000-0000-7000-8000-000000000002";
        let old = br#"{"schemaVersion":1,"source":"manual-validation-summary","title":"Validation","summary":"passed"}"#.to_vec();
        connection.execute("INSERT INTO local_review_collections (id, schema_version, task_id, state, title, created_at_ms, updated_at_ms) VALUES (?1, 1, ?1, 'active', 'Review', 1, 1)", [collection]).expect("collection");
        connection.execute("INSERT INTO local_review_items (id, schema_version, collection_id, class, text_format, mime_type, width, height, state, title, source_kind, provenance, content, sha256, byte_size, created_at_ms, updated_at_ms) VALUES (?1, 1, ?2, 'evidence', NULL, 'application/json; profile=evidence-envelope-v1', NULL, NULL, 'ready', 'Validation', 'typed-evidence-snapshot', 'manual-validation-summary', ?3, ?4, ?5, 1, 1)", params![item, collection, old, review_digest(&old), old.len() as i64]).expect("v12 manual evidence");
        let repository =
            ProjectRepository::from_test_connection(connection).expect("latest schema migrates");
        let (source, content): (String, Vec<u8>) = repository
            .connection
            .query_row(
                "SELECT evidence_source, content FROM local_review_items WHERE id = ?1",
                [item],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("backfill");
        assert_eq!(source, "manual-validation-summary");
        assert_eq!(
            parse_manual_evidence_envelope(&content, "Validation").expect("canonical envelope"),
            "passed"
        );
        assert_eq!(
            repository
                .connection
                .query_row("SELECT max(version) FROM schema_migrations", [], |row| row
                    .get::<_, i64>(
                    0
                ))
                .expect("version"),
            28
        );
    }

    #[test]
    fn local_review_evidence_source_migration_rejects_invalid_rows_without_partial_upgrade() {
        let mut connection = Connection::open_in_memory().expect("database opens");
        connection.execute_batch("CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY NOT NULL, name TEXT NOT NULL, applied_at_ms INTEGER NOT NULL);").expect("ledger");
        for (version, name, sql) in MIGRATIONS.iter().take(12) {
            connection.execute_batch(sql).expect("v12 migration");
            connection
                .execute(
                    "INSERT INTO schema_migrations VALUES (?1, ?2, 1)",
                    params![version, name],
                )
                .expect("ledger row");
        }
        let collection = "018f0000-0000-7000-8000-000000000003";
        let item = "018f0000-0000-7000-8000-000000000004";
        let corrupt = b"not-json".to_vec();
        connection.execute("INSERT INTO local_review_collections (id, schema_version, task_id, state, title, created_at_ms, updated_at_ms) VALUES (?1, 1, ?1, 'active', 'Review', 1, 1)", [collection]).expect("collection");
        connection.execute("INSERT INTO local_review_items (id, schema_version, collection_id, class, text_format, mime_type, width, height, state, title, source_kind, provenance, content, sha256, byte_size, created_at_ms, updated_at_ms) VALUES (?1, 1, ?2, 'evidence', NULL, 'application/json; profile=evidence-envelope-v1', NULL, NULL, 'ready', 'Validation', 'typed-evidence-snapshot', 'manual-validation-summary', ?3, ?4, ?5, 1, 1)", params![item, collection, corrupt, review_digest(&corrupt), corrupt.len() as i64]).expect("corrupt v12 evidence");
        assert!(matches!(
            apply_migrations(&mut connection),
            Err(StorageError::InvalidStoredValue)
        ));
        assert_eq!(
            connection
                .query_row("SELECT max(version) FROM schema_migrations", [], |row| row
                    .get::<_, i64>(
                    0
                ))
                .expect("prior ledger"),
            12
        );
        assert_eq!(connection.query_row("SELECT count(*) FROM pragma_table_info('local_review_items') WHERE name = 'evidence_source'", [], |row| row.get::<_, i64>(0)).expect("schema rollback"), 0);
    }

    #[test]
    fn local_review_evidence_source_constraints_reject_unknown_and_non_evidence_values() {
        let repository = ProjectRepository::in_memory().expect("repository");
        let collection = "018f0000-0000-7000-8000-000000000005";
        let item = "018f0000-0000-7000-8000-000000000006";
        repository.connection.execute("INSERT INTO local_review_collections (id, schema_version, task_id, state, title, created_at_ms, updated_at_ms) VALUES (?1, 1, ?1, 'active', 'Review', 1, 1)", [collection]).expect("collection");
        let content = b"x".to_vec();
        assert!(repository.connection.execute("INSERT INTO local_review_items (id, schema_version, collection_id, class, text_format, mime_type, width, height, state, title, source_kind, provenance, evidence_source, content, sha256, byte_size, created_at_ms, updated_at_ms) VALUES (?1, 1, ?2, 'evidence', NULL, 'application/json; profile=evidence-envelope-v1', NULL, NULL, 'ready', 'Validation', 'typed-evidence-snapshot', 'unknown', 'unknown', ?3, ?4, ?5, 1, 1)", params![item, collection, content, review_digest(b"x"), 1_i64]).is_err());
        assert!(repository.connection.execute("INSERT INTO local_review_items (id, schema_version, collection_id, class, text_format, mime_type, width, height, state, title, source_kind, provenance, evidence_source, content, sha256, byte_size, created_at_ms, updated_at_ms) VALUES (?1, 1, ?2, 'text', 'plain', 'text/plain; charset=utf-8', NULL, NULL, 'ready', 'Text', 'user-authored-text', '', 'manual-validation-summary', ?3, ?4, ?5, 1, 1)", params!["018f0000-0000-7000-8000-000000000007", collection, content, review_digest(b"x"), 1_i64]).is_err());
    }

    #[test]
    fn reads_advisor_metadata_without_mutating_the_database() {
        let repository = ProjectRepository::in_memory().expect("database must open");
        let conversation_id = "019d4e3c-3b14-7a2b-8c91-3f27d4f7aa10";
        let context_id = "019d4e3c-3b15-78d4-b71a-3f27d4f7aa11";
        let proposal_id = "019d4e3c-3b16-7c9f-a80b-3f27d4f7aa12";
        let project_id = "019d4e3c-3b17-7e50-9f35-3f27d4f7aa13";
        repository
            .connection
            .execute(
                "INSERT INTO projects (
                    id, display_name, active_directory_association_id, archived_at_ms,
                    created_at_ms, updated_at_ms
                 ) VALUES (?1, 'Advisor target', NULL, NULL, 1, 1)",
                [project_id],
            )
            .expect("target project must insert");
        repository
            .connection
            .execute(
                "INSERT INTO advisor_conversations (id, codex_thread_id, created_at_ms, updated_at_ms)
                 VALUES (?1, 'advisor-thread-fixture-01', 1, 1)",
                [conversation_id],
            )
            .expect("advisor conversation must insert");
        repository
            .connection
            .execute(
                "INSERT INTO advisor_context_references (
                    id, advisor_conversation_id, kind, source_ref, source_commit,
                    source_sha256, selected_at_ms, freshness, trust,
                    provenance_source, provenance_ref, provenance_commit,
                    observed_at_ms, provenance_note, created_at_ms, updated_at_ms
                 ) VALUES (
                    ?1, ?2, 'project-state', 'project-state-snapshot',
                    '7bf4a235904fc2c760daed81e899b040da96b5b4', ?3, 1, 'current',
                    'verified', 'project-state-snapshot', 'project-state-snapshot',
                    '7bf4a235904fc2c760daed81e899b040da96b5b4', 1, NULL, 1, 1
                 )",
                params![context_id, conversation_id, "1".repeat(64)],
            )
            .expect("advisor context must insert");
        repository
            .connection
            .execute(
                "INSERT INTO advisor_dispatch_records (
                    id, advisor_conversation_id, target_project_id, request_sha256,
                    context_manifest_sha256, state, requires_explicit_approval,
                    requested_model, requested_reasoning_effort, trust,
                    provenance_source, provenance_ref, provenance_commit,
                    observed_at_ms, provenance_note, created_at_ms, updated_at_ms
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, 'draft', 1, NULL, NULL, 'inferred',
                    'user-selection', 'advisor-dispatch-draft', NULL, 1, NULL, 1, 1
                 )",
                params![
                    proposal_id,
                    conversation_id,
                    project_id,
                    "2".repeat(64),
                    "3".repeat(64)
                ],
            )
            .expect("advisor proposal must insert");

        let before = repository.connection.total_changes();
        let snapshot = repository
            .advisor_snapshot()
            .expect("valid advisor metadata must read");
        assert_eq!(repository.connection.total_changes(), before);
        assert_eq!(snapshot.conversations.len(), 1);
        assert_eq!(snapshot.context_references.len(), 1);
        assert_eq!(snapshot.dispatch_proposals.len(), 1);

        repository
            .connection
            .execute_batch("PRAGMA ignore_check_constraints = ON;")
            .expect("test-only constraint bypass must enable");
        repository
            .connection
            .execute(
                "UPDATE advisor_context_references SET kind = 'unsafe-kind' WHERE id = ?1",
                [context_id],
            )
            .expect("invalid stored test value must write");
        assert!(matches!(
            repository.advisor_snapshot(),
            Err(StorageError::InvalidStoredValue)
        ));
    }

    #[test]
    fn creates_only_the_app_owned_metadata_schema() {
        let repository = ProjectRepository::in_memory().expect("schema must migrate");
        let mut statement = repository
            .connection
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .expect("schema must be queryable");
        let tables: Vec<String> = statement
            .query_map([], |row| row.get(0))
            .expect("tables must be queryable")
            .collect::<Result<_, _>>()
            .expect("table rows must be valid");

        assert_eq!(
            tables,
            vec![
                "advisor_context_references".to_owned(),
                "advisor_conversations".to_owned(),
                "advisor_dispatch_records".to_owned(),
                "artifact_references".to_owned(),
                "context_bundle_audit".to_owned(),
                "context_bundle_items".to_owned(),
                "context_bundles".to_owned(),
                "controlled_browser_verification_attempts".to_owned(),
                "controlled_browser_verification_audit".to_owned(),
                "conversation_references".to_owned(),
                "directory_associations".to_owned(),
                "durable_sources".to_owned(),
                "fictional_connector_audit".to_owned(),
                "fictional_connector_bindings".to_owned(),
                "fictional_connector_operations".to_owned(),
                "knowledge_record_events".to_owned(),
                "knowledge_records".to_owned(),
                "local_review_activity_ledger".to_owned(),
                "local_review_annotations".to_owned(),
                "local_review_collections".to_owned(),
                "local_review_comparisons".to_owned(),
                "local_review_items".to_owned(),
                "local_task_templates".to_owned(),
                "project_package_validation_candidate_identities".to_owned(),
                "project_package_validation_summaries".to_owned(),
                "projects".to_owned(),
                "schema_migrations".to_owned(),
                "task_plans".to_owned(),
                "task_records".to_owned(),
                "task_template_application_reservations".to_owned(),
                "terminal_sessions".to_owned(),
                "unified_conversation_metadata".to_owned(),
                "worktree_relations".to_owned(),
            ]
        );

        let foreign_keys: bool = repository
            .connection
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("foreign-key state must be queryable");
        assert!(foreign_keys);

        let mut columns = Vec::new();
        for table in [
            "projects",
            "directory_associations",
            "conversation_references",
            "advisor_conversations",
            "advisor_context_references",
            "advisor_dispatch_records",
            "terminal_sessions",
            "unified_conversation_metadata",
            "worktree_relations",
        ] {
            let mut statement = repository
                .connection
                .prepare(&format!("PRAGMA table_info({table})"))
                .expect("table metadata must be queryable");
            columns.extend(
                statement
                    .query_map([], |row| row.get::<_, String>(1))
                    .expect("columns must be queryable")
                    .collect::<Result<Vec<_>, _>>()
                    .expect("column rows must be valid"),
            );
        }
        assert!(columns.iter().all(|column| {
            !["token", "secret", "credential", "auth", "session"]
                .iter()
                .any(|term| column.contains(term))
        }));
        assert!(!columns.iter().any(|column| {
            ["prompt", "message", "content", "output"]
                .iter()
                .any(|term| column.contains(term))
        }));
    }

    #[test]
    fn protects_the_metadata_directory_and_database_file() {
        let directory = std::env::temp_dir().join(format!(
            "quireforge-metadata-permissions-{}",
            Uuid::now_v7()
        ));
        fs::create_dir(&directory).expect("metadata directory must be created");
        let database = directory.join("metadata.sqlite3");

        let repository = ProjectRepository::open(&database).expect("database must open");

        assert_eq!(
            fs::metadata(&directory)
                .expect("directory metadata must be readable")
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(&database)
                .expect("database metadata must be readable")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        drop(repository);
        fs::remove_dir_all(directory).expect("temporary metadata must be removed");
    }

    #[test]
    fn persists_only_bounded_conversation_references() {
        let mut repository = ProjectRepository::in_memory().expect("schema must migrate");
        let project_id = Uuid::now_v7().to_string();
        repository
            .connection
            .execute(
                "INSERT INTO projects (
                    id, display_name, active_directory_association_id, archived_at_ms,
                    created_at_ms, updated_at_ms
                 ) VALUES (?1, 'Fixture', NULL, NULL, 1, 1)",
                [&project_id],
            )
            .expect("fixture project must insert");
        let conversation_id = Uuid::now_v7().to_string();
        let thread_id = Uuid::now_v7().to_string();
        let turn_id = Uuid::now_v7().to_string();

        repository
            .insert_conversation_reference(&ConversationReference {
                conversation_id: &conversation_id,
                project_id: &project_id,
                codex_thread_id: &thread_id,
                model_id: "fixture-model",
                reasoning_effort: "medium",
                sandbox_mode: "read-only",
                approval_policy: "untrusted",
                parent_conversation_id: None,
                selection: ConversationSelectionMetadata {
                    availability: "ready",
                    ownership: "manual",
                    user_locked: false,
                    allowed_model_ids_json: "[]",
                    reasoning_ceiling: None,
                    pending: None,
                },
            })
            .expect("conversation reference must insert");
        repository
            .update_conversation_turn(&conversation_id, &turn_id)
            .expect("turn reference must update");
        repository
            .update_conversation_status(&conversation_id, "completed")
            .expect("conversation status must update");

        let stored: (String, String, Option<String>, String) = repository
            .connection
            .query_row(
                "SELECT project_id, codex_thread_id, active_turn_id, status
                 FROM conversation_references WHERE id = ?1",
                [&conversation_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .expect("conversation reference must be queryable");
        assert_eq!(
            stored,
            (project_id.clone(), thread_id, None, "completed".to_owned())
        );

        let unified: (String, String, String) = repository
            .connection
            .query_row(
                "SELECT mode, project_id, conversation_reference_id
                 FROM unified_conversation_metadata WHERE id = ?1",
                [&conversation_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("Codex metadata must be recorded atomically with its reference");
        assert_eq!(
            unified,
            ("codex".to_owned(), project_id.clone(), conversation_id)
        );

        let invalid_chat = repository.connection.execute(
            "INSERT INTO unified_conversation_metadata (
                id, mode, project_id, conversation_reference_id, created_at_ms, updated_at_ms
             ) VALUES (?1, 'chat', ?2, NULL, 1, 1)",
            params![Uuid::now_v7().to_string(), project_id],
        );
        assert!(invalid_chat.is_err());

        let chat_conversation_id = Uuid::now_v7().to_string();
        let chat_thread_id = Uuid::now_v7().to_string();
        repository
            .insert_chat_conversation_metadata(&ChatConversationMetadata {
                conversation_id: &chat_conversation_id,
                codex_thread_id: &chat_thread_id,
            })
            .expect("bounded Chat metadata must persist without a project");
        let chat: (String, Option<String>, String) = repository
            .connection
            .query_row(
                "SELECT mode, project_id, codex_thread_id
                 FROM unified_conversation_metadata WHERE id = ?1",
                [&chat_conversation_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("Chat metadata must be queryable");
        assert_eq!(chat, ("chat".to_owned(), None, chat_thread_id));
    }

    #[test]
    fn recovers_stale_active_turns_without_preserving_runtime_ownership() {
        let mut repository = ProjectRepository::in_memory().expect("schema must migrate");
        let project_id = Uuid::now_v7().to_string();
        repository
            .connection
            .execute(
                "INSERT INTO projects (
                    id, display_name, active_directory_association_id, archived_at_ms,
                    created_at_ms, updated_at_ms
                 ) VALUES (?1, 'Fixture', NULL, NULL, 1, 1)",
                [&project_id],
            )
            .expect("fixture project must insert");
        let conversation_id = Uuid::now_v7().to_string();
        let thread_id = Uuid::now_v7().to_string();
        let turn_id = Uuid::now_v7().to_string();
        repository
            .insert_conversation_reference(&ConversationReference {
                conversation_id: &conversation_id,
                project_id: &project_id,
                codex_thread_id: &thread_id,
                model_id: "fixture-model",
                reasoning_effort: "medium",
                sandbox_mode: "read-only",
                approval_policy: "untrusted",
                parent_conversation_id: None,
                selection: ConversationSelectionMetadata {
                    availability: "ready",
                    ownership: "manual",
                    user_locked: false,
                    allowed_model_ids_json: "[]",
                    reasoning_ceiling: None,
                    pending: None,
                },
            })
            .expect("conversation reference must insert");
        repository
            .update_conversation_turn(&conversation_id, &turn_id)
            .expect("turn reference must update");
        repository
            .update_model_selection(
                &conversation_id,
                None,
                &ConversationSelectionMetadata {
                    availability: "ready",
                    ownership: "automatic",
                    user_locked: false,
                    allowed_model_ids_json: r#"["fixture-next"]"#,
                    reasoning_ceiling: Some("high"),
                    pending: Some(ConversationPendingSelection {
                        model_id: "fixture-next",
                        reasoning_effort: "high",
                        rationale: "Use the larger context window.",
                        provenance: "codex",
                        application: "automatic",
                        requested_at_ms: 42,
                    }),
                },
            )
            .expect("pending selector request must persist");

        let connection = repository.connection;
        let recovered = ProjectRepository::from_test_connection(connection)
            .expect("reopened metadata must recover");
        let stored = recovered
            .conversation_reference(&conversation_id)
            .expect("conversation reference must remain available");

        assert_eq!(stored.status, "interrupted");
        assert!(stored.active_turn_id.is_none());
        assert_eq!(stored.codex_thread_id, thread_id);
        assert_eq!(stored.selector_mode, "automatic");
        assert_eq!(
            stored.selector_allowed_model_ids_json,
            r#"["fixture-next"]"#
        );
        assert_eq!(stored.selector_reasoning_ceiling.as_deref(), Some("high"));
        assert_eq!(
            stored.selector_pending_model_id.as_deref(),
            Some("fixture-next")
        );
        assert_eq!(
            stored.selector_pending_reasoning_effort.as_deref(),
            Some("high")
        );
        assert_eq!(stored.selector_pending_provenance.as_deref(), Some("codex"));
        assert_eq!(
            stored.selector_pending_application.as_deref(),
            Some("automatic")
        );
        assert_eq!(stored.selector_pending_requested_at_ms, Some(42));
    }

    #[test]
    fn persists_only_bounded_terminal_metadata_and_interrupts_stale_sessions() {
        let root =
            std::env::temp_dir().join(format!("quireforge-terminal-storage-{}", Uuid::now_v7()));
        let project = root.join("project");
        let database = root.join("data/metadata.sqlite3");
        fs::create_dir_all(&project).expect("project fixture must exist");
        let identity = crate::project::identity::inspect_directory(&project)
            .expect("project fixture must be inspectable");
        let mut repository = ProjectRepository::open(&database).expect("metadata must open");
        let project_id = repository
            .insert_project("terminal project", &identity)
            .expect("project must persist");
        let terminal_id = Uuid::now_v7().to_string();
        repository
            .insert_terminal_session(&terminal_id, &project_id, "Terminal 1", 100, 30)
            .expect("terminal metadata must persist");
        drop(repository);

        let reopened = ProjectRepository::open(&database).expect("metadata must reopen");
        let sessions = reopened
            .list_terminal_sessions()
            .expect("terminal metadata must load");

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].status, "interrupted");
        assert_eq!(sessions[0].columns, 100);
        assert_eq!(sessions[0].rows, 30);
        let columns: Vec<String> = reopened
            .connection
            .prepare("PRAGMA table_info(terminal_sessions)")
            .expect("terminal schema must be queryable")
            .query_map([], |row| row.get(1))
            .expect("terminal columns must be queryable")
            .collect::<Result<_, _>>()
            .expect("terminal columns must be valid");
        for forbidden in [
            "cwd",
            "environment",
            "input",
            "output",
            "pid",
            "process_group",
            "session_id",
            "shell_history",
        ] {
            assert!(!columns.iter().any(|column| column == forbidden));
        }
        drop(reopened);
        fs::remove_dir_all(root).expect("terminal storage fixture must be removed");
    }

    #[test]
    fn task_records_are_local_bounded_and_plan_switches_are_metadata_only() {
        let mut repository = ProjectRepository::in_memory().expect("schema must migrate");
        let task = repository.create_task().expect("task must create");
        repository
            .rename_task(&task, "  Local\tplan — release  ")
            .expect("title must normalize");
        assert!(matches!(
            repository.rename_task(&task, "unsafe\u{202e}title"),
            Err(StorageError::InvalidStoredValue)
        ));
        let plan = repository
            .create_plan(&task, true)
            .expect("alternate must create");
        repository
            .select_plan(&task, &plan)
            .expect("switch must persist");
        repository
            .edit_plan(&task, &plan, "Alternative", "visible text")
            .expect("edit must persist");
        let (tasks, selected, plans, count, _, corrupt) = repository
            .task_catalog(Some(&task), false, Some("alternative"))
            .expect("search must read");
        assert!(!corrupt);
        assert_eq!(count, 1);
        assert_eq!(tasks[0].title, "Local plan — release");
        assert_eq!(selected.expect("selection").selected_plan_id, plan);
        assert_eq!(plans.len(), 2);
        repository
            .archive_task(&task, false)
            .expect("archive must persist");
        assert!(repository
            .task_catalog(Some(&task), false, None)
            .expect("list")
            .0
            .is_empty());
        repository
            .archive_task(&task, true)
            .expect("restore must persist");
        repository
            .delete_plan(&task, &plan)
            .expect("alternate must delete");
        repository.delete_task(&task).expect("task must delete");
        assert_eq!(
            repository.task_catalog(None, true, None).expect("list").3,
            0
        );
    }

    #[test]
    fn task_project_binding_migration_preserves_legacy_tasks_and_rolls_back_atomically() {
        let connection = Connection::open_in_memory().expect("database opens");
        connection.execute_batch("CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY NOT NULL, name TEXT NOT NULL, applied_at_ms INTEGER NOT NULL);").expect("ledger");
        for (version, name, sql) in MIGRATIONS.iter().take(13) {
            connection.execute_batch(sql).expect("v13 migration");
            connection
                .execute(
                    "INSERT INTO schema_migrations VALUES (?1, ?2, 1)",
                    params![version, name],
                )
                .expect("ledger row");
        }
        let task_id = "018f0000-0000-7000-8000-000000000111";
        let plan_id = "018f0000-0000-7000-8000-000000000112";
        connection
            .execute(
                "INSERT INTO task_records (
                id, schema_version, title, status, created_at_ms, updated_at_ms,
                archived_at_ms, last_opened_at_ms, selected_plan_id
             ) VALUES (?1, 1, 'Legacy', 'active', 1, 1, NULL, 1, ?2)",
                params![task_id, plan_id],
            )
            .expect("legacy task");
        let repository =
            ProjectRepository::from_test_connection(connection).expect("latest upgrade");
        assert_eq!(
            repository.task_project_binding(task_id).expect("binding"),
            None
        );
        assert_eq!(
            repository
                .connection
                .query_row("SELECT max(version) FROM schema_migrations", [], |row| row
                    .get::<_, i64>(
                    0
                ),)
                .expect("version"),
            28
        );
        assert_eq!(
            repository.connection.query_row(
                "SELECT count(*) FROM pragma_table_info('task_records') WHERE name = 'project_id'",
                [], |row| row.get::<_, i64>(0),
            ).expect("column"),
            1
        );

        let mut failed = Connection::open_in_memory().expect("database opens");
        failed.execute_batch("CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY NOT NULL, name TEXT NOT NULL, applied_at_ms INTEGER NOT NULL);").expect("ledger");
        for (version, name, sql) in MIGRATIONS.iter().take(13) {
            failed.execute_batch(sql).expect("v13 migration");
            failed
                .execute(
                    "INSERT INTO schema_migrations VALUES (?1, ?2, 1)",
                    params![version, name],
                )
                .expect("ledger row");
        }
        failed
            .execute_batch("CREATE INDEX task_records_project_binding_idx ON task_records(id);")
            .expect("conflicting index");
        assert!(apply_migrations(&mut failed).is_err());
        assert_eq!(
            failed
                .query_row("SELECT max(version) FROM schema_migrations", [], |row| row
                    .get::<_, i64>(
                    0
                ))
                .expect("prior ledger"),
            13
        );
        assert_eq!(
            failed.query_row("SELECT count(*) FROM pragma_table_info('task_records') WHERE name = 'project_id'", [], |row| row.get::<_, i64>(0)).expect("rollback"),
            0
        );
    }

    #[test]
    fn task_catalog_creation_requires_a_live_project_and_binds_named_task_atomically() {
        let mut repository = ProjectRepository::in_memory().expect("schema must migrate");
        let (project_id, _) = insert_live_task_context(&repository);

        let task_id = repository
            .create_task_for_project(&project_id, "  Project\t task ")
            .expect("bound task must create");

        assert_eq!(
            repository.task_project_binding(&task_id).expect("binding"),
            Some(project_id.clone())
        );
        let title: String = repository
            .connection
            .query_row(
                "SELECT title FROM task_records WHERE id = ?1",
                [&task_id],
                |row| row.get(0),
            )
            .expect("title");
        assert_eq!(title, "Project task");
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT count(*) FROM task_plans WHERE task_id = ?1",
                    [&task_id],
                    |row| row.get::<_, i64>(0)
                )
                .expect("one explicit primary plan"),
            1
        );

        let legacy_unbound = repository.create_task().expect("legacy fixture");
        let (tasks, selected, _, task_count, _, _) = repository
            .task_catalog_for_project(&project_id, Some(&task_id), false, None)
            .expect("project catalog");
        assert_eq!(task_count, 1);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, task_id);
        assert_eq!(selected.expect("selected task").id, task_id);
        assert_ne!(tasks[0].id, legacy_unbound);

        let count_before: i64 = repository
            .connection
            .query_row("SELECT count(*) FROM task_records", [], |row| row.get(0))
            .expect("count");
        assert!(matches!(
            repository.create_task_for_project("018f0000-0000-7000-8000-000000000199", "Refused"),
            Err(StorageError::ProjectNotFound)
        ));
        assert_eq!(
            repository
                .connection
                .query_row("SELECT count(*) FROM task_records", [], |row| row
                    .get::<_, i64>(0))
                .expect("failed create must not persist"),
            count_before
        );
    }

    #[test]
    fn fictional_connector_records_are_project_task_bound_and_content_free() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let (project_id, _) = insert_live_task_context(&repository);
        let task_id = repository
            .create_task_for_project(&project_id, "Connector fixture")
            .expect("task");
        let operation_id = Uuid::now_v7().to_string();
        let authorization_id = Uuid::now_v7().to_string();
        let binding_id = Uuid::now_v7().to_string();
        let digest = review_digest(b"fictional-local-only-request");
        repository
            .record_fictional_connector_operation(&FictionalConnectorOperationRecord {
                project_id: &project_id,
                task_id: &task_id,
                binding_id: &binding_id,
                operation_id: &operation_id,
                authorization_id: Some(&authorization_id),
                operation_class: "mutation",
                state: "prepared",
                descriptor_id: "019a57c0-0000-7000-8000-000000000001",
                descriptor_version: 1,
                descriptor_sha256: &digest,
                scope_digest: &digest,
                request_digest: &digest,
                expires_at_ms: super::now_millis() + 5 * 60 * 1000,
            })
            .expect("prepared operation");
        repository
            .complete_fictional_connector_operation(
                &project_id,
                &task_id,
                &operation_id,
                "outcome-unknown",
                &digest,
            )
            .expect("ambiguous terminal result");
        let row: (String, String, String) = repository.connection.query_row(
            "SELECT operation_class, state, request_digest FROM fictional_connector_operations WHERE id = ?1",
            [&operation_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).expect("persisted operation");
        assert_eq!(row, ("mutation".into(), "outcome-unknown".into(), digest));
        let audit_count: i64 = repository
            .connection
            .query_row(
                "SELECT count(*) FROM fictional_connector_audit WHERE operation_id = ?1",
                [&operation_id],
                |row| row.get(0),
            )
            .expect("audit");
        assert_eq!(audit_count, 2);
        assert!(repository
            .record_fictional_connector_operation(&FictionalConnectorOperationRecord {
                project_id: &Uuid::now_v7().to_string(),
                task_id: &task_id,
                binding_id: &Uuid::now_v7().to_string(),
                operation_id: &Uuid::now_v7().to_string(),
                authorization_id: None,
                operation_class: "read",
                state: "completed",
                descriptor_id: "019a57c0-0000-7000-8000-000000000001",
                descriptor_version: 1,
                descriptor_sha256: &review_digest(b"wrong-descriptor"),
                scope_digest: &review_digest(b"wrong-scope"),
                request_digest: &review_digest(b"wrong-project"),
                expires_at_ms: super::now_millis() + 5 * 60 * 1000,
            })
            .is_err());
    }

    #[test]
    fn fictional_connector_process_recovery_expires_pending_authorization() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let (project_id, _) = insert_live_task_context(&repository);
        let task_id = repository
            .create_task_for_project(&project_id, "Connector recovery fixture")
            .expect("task");
        let binding_id = Uuid::now_v7().to_string();
        let operation_id = Uuid::now_v7().to_string();
        let authorization_id = Uuid::now_v7().to_string();
        let digest = review_digest(b"fictional-recovery-request");
        repository
            .record_fictional_connector_operation(&FictionalConnectorOperationRecord {
                project_id: &project_id,
                task_id: &task_id,
                binding_id: &binding_id,
                operation_id: &operation_id,
                authorization_id: Some(&authorization_id),
                operation_class: "mutation",
                state: "prepared",
                descriptor_id: "019a57c0-0000-7000-8000-000000000001",
                descriptor_version: 1,
                descriptor_sha256: &digest,
                scope_digest: &digest,
                request_digest: &digest,
                expires_at_ms: super::now_millis() + 5 * 60 * 1000,
            })
            .expect("prepared operation");
        super::recover_interrupted_fictional_connector_operations(&repository.connection)
            .expect("process recovery");
        let states: (String, String) = repository.connection.query_row(
            "SELECT operation.state, binding.state FROM fictional_connector_operations operation JOIN fictional_connector_bindings binding ON binding.id = operation.binding_id WHERE operation.id = ?1",
            [&operation_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).expect("expired state");
        assert_eq!(states, ("expired".into(), "expired".into()));
        let recovery_events: i64 = repository.connection.query_row(
            "SELECT count(*) FROM fictional_connector_audit WHERE operation_id = ?1 AND event_kind = 'process-recovery' AND outcome = 'expired'",
            [&operation_id],
            |row| row.get(0),
        ).expect("recovery audit");
        assert_eq!(recovery_events, 1);
    }

    #[test]
    fn task_project_binding_is_native_context_bound_immutable_and_isolated() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let (project_id, conversation_id) = insert_live_task_context(&repository);
        let bound = repository
            .create_task_from_conversation_context(&conversation_id)
            .expect("native context task");
        let unbound = repository.create_task().expect("no-project task");
        assert_eq!(
            repository.task_project_binding(&bound).expect("bound task"),
            Some(project_id.clone())
        );
        assert_eq!(
            repository
                .task_project_binding(&unbound)
                .expect("unbound task"),
            None
        );

        assert!(repository
            .connection
            .execute(
                "UPDATE task_records SET project_id = NULL WHERE id = ?1",
                [&bound],
            )
            .is_err());
        assert!(repository
            .connection
            .execute(
                "UPDATE task_records SET project_id = ?1 WHERE id = ?2",
                params![project_id, unbound],
            )
            .is_err());
        assert!(repository
            .connection
            .execute(
                "INSERT INTO task_records (
                id, schema_version, title, status, created_at_ms, updated_at_ms,
                archived_at_ms, last_opened_at_ms, selected_plan_id, project_id
             ) VALUES (
                '018f0000-0000-7000-8000-000000000113', 1, 'Invalid', 'active',
                1, 1, NULL, 1, '018f0000-0000-7000-8000-000000000114',
                '018f0000-0000-7000-8000-000000000115'
             )",
                [],
            )
            .is_err());

        repository
            .set_task_status(&bound, TaskStatus::Paused)
            .expect("pause");
        repository
            .set_task_status(&bound, TaskStatus::Completed)
            .expect("complete");
        repository
            .set_task_status(&bound, TaskStatus::Active)
            .expect("restore");
        repository.create_plan(&bound, false).expect("plan change");
        assert_eq!(
            repository
                .task_project_binding(&bound)
                .expect("lifecycle binding"),
            Some(project_id.clone())
        );

        repository
            .connection
            .execute(
                "DELETE FROM conversation_references WHERE id = ?1",
                [&conversation_id],
            )
            .expect("remove context only");
        repository
            .connection
            .execute(
                "UPDATE projects SET active_directory_association_id = NULL WHERE id = ?1",
                [&project_id],
            )
            .expect("detach association for deletion check");
        repository
            .connection
            .execute(
                "DELETE FROM directory_associations WHERE project_id = ?1",
                [&project_id],
            )
            .expect("remove association for deletion check");
        assert!(repository
            .connection
            .execute("DELETE FROM projects WHERE id = ?1", [&project_id])
            .is_err());
    }

    #[test]
    fn task_advisor_dispatch_origin_is_native_verified_complete_and_immutable() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let (project_id, execution_conversation_id) = insert_live_task_context(&repository);
        let advisor_conversation_id = "018f0000-0000-7000-8000-000000000121";
        let dispatch_id = "018f0000-0000-7000-8000-000000000122";
        insert_started_advisor_dispatch(
            &repository,
            advisor_conversation_id,
            dispatch_id,
            &project_id,
            &execution_conversation_id,
        );

        let task_id = repository
            .create_task_from_conversation_context(&execution_conversation_id)
            .expect("native Advisor-dispatched task");
        assert_eq!(
            repository
                .task_project_binding(&task_id)
                .expect("project binding"),
            Some(project_id.clone())
        );
        assert_eq!(
            repository
                .task_advisor_dispatch_origin(&task_id)
                .expect("origin binding"),
            Some((advisor_conversation_id.to_owned(), dispatch_id.to_owned()))
        );

        let ordinary = repository.create_task().expect("ordinary task");
        assert_eq!(
            repository
                .task_advisor_dispatch_origin(&ordinary)
                .expect("ordinary origin"),
            None
        );
        assert!(repository
            .connection
            .execute(
                "UPDATE task_records SET origin_advisor_conversation_id = NULL WHERE id = ?1",
                [&task_id],
            )
            .is_err());
        assert!(repository
            .connection
            .execute(
                "UPDATE task_records
                 SET origin_advisor_conversation_id = ?1,
                     origin_advisor_dispatch_record_id = ?2
                 WHERE id = ?3",
                params![advisor_conversation_id, dispatch_id, ordinary],
            )
            .is_err());
    }

    #[test]
    fn task_advisor_dispatch_origin_rejects_mismatch_and_never_leaves_partial_binding() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let (project_id, execution_conversation_id) = insert_live_task_context(&repository);
        let other_project_id = "018f0000-0000-7000-8000-000000000130";
        repository
            .connection
            .execute(
                "INSERT INTO projects (
                    id, display_name, active_directory_association_id, archived_at_ms,
                    created_at_ms, updated_at_ms
                 ) VALUES (?1, 'Other project', NULL, NULL, 1, 1)",
                [other_project_id],
            )
            .expect("other project");
        let dispatch_id = "018f0000-0000-7000-8000-000000000131";
        let advisor_conversation_id = "018f0000-0000-7000-8000-000000000132";
        insert_started_advisor_dispatch(
            &repository,
            advisor_conversation_id,
            dispatch_id,
            other_project_id,
            &execution_conversation_id,
        );
        assert!(matches!(
            repository.create_task_from_conversation_context(&execution_conversation_id),
            Err(StorageError::InvalidStoredValue)
        ));
        assert_eq!(
            repository
                .connection
                .query_row("SELECT count(*) FROM task_records", [], |row| row
                    .get::<_, i64>(0))
                .expect("no failed task"),
            0
        );

        let second_advisor_conversation_id = "018f0000-0000-7000-8000-000000000133";
        repository
            .connection
            .execute(
                "INSERT INTO advisor_conversations (id, codex_thread_id, created_at_ms, updated_at_ms)
                 VALUES (?1, 'second-advisor', 1, 1)",
                [second_advisor_conversation_id],
            )
            .expect("second advisor conversation");
        assert!(repository
            .connection
            .execute(
                "INSERT INTO task_records (
                id, schema_version, title, status, created_at_ms, updated_at_ms,
                archived_at_ms, last_opened_at_ms, selected_plan_id, project_id,
                origin_advisor_conversation_id, origin_advisor_dispatch_record_id
             ) VALUES (
                '018f0000-0000-7000-8000-000000000134', 1, 'Invalid', 'active',
                1, 1, NULL, 1, '018f0000-0000-7000-8000-000000000135', ?1, ?2, ?3
             )",
                params![project_id, second_advisor_conversation_id, dispatch_id],
            )
            .is_err());
    }

    #[test]
    fn task_advisor_dispatch_origin_migration_preserves_legacy_unbound_tasks() {
        let connection = Connection::open_in_memory().expect("database opens");
        connection
            .execute_batch("CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY NOT NULL, name TEXT NOT NULL, applied_at_ms INTEGER NOT NULL);")
            .expect("ledger");
        for (version, name, sql) in MIGRATIONS.iter().take(19) {
            connection.execute_batch(sql).expect("pre-origin migration");
            connection
                .execute(
                    "INSERT INTO schema_migrations VALUES (?1, ?2, 1)",
                    params![version, name],
                )
                .expect("ledger row");
        }
        connection
            .execute(
                "INSERT INTO task_records (
                    id, schema_version, title, status, created_at_ms, updated_at_ms,
                    archived_at_ms, last_opened_at_ms, selected_plan_id, project_id
                 ) VALUES (
                    '018f0000-0000-7000-8000-000000000141', 1, 'Legacy', 'active',
                    1, 1, NULL, 1, '018f0000-0000-7000-8000-000000000142', NULL
                 )",
                [],
            )
            .expect("legacy task");
        let repository = ProjectRepository::from_test_connection(connection).expect("migrated");
        assert_eq!(
            repository
                .task_advisor_dispatch_origin("018f0000-0000-7000-8000-000000000141")
                .expect("legacy origin"),
            None
        );
        assert_eq!(
            repository
                .connection
                .query_row("SELECT max(version) FROM schema_migrations", [], |row| row
                    .get::<_, i64>(
                    0
                ))
                .expect("migration version"),
            28
        );
    }

    #[test]
    fn local_review_approval_presentation_uses_only_immutable_task_origin_and_persisted_bytes() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let (project_id, execution_conversation_id) = insert_live_task_context(&repository);
        let advisor_conversation_id = "018f0000-0000-7000-8000-000000000151";
        let dispatch_id = "018f0000-0000-7000-8000-000000000152";
        insert_started_advisor_dispatch(
            &repository,
            advisor_conversation_id,
            dispatch_id,
            &project_id,
            &execution_conversation_id,
        );
        let task_id = repository
            .create_task_from_conversation_context(&execution_conversation_id)
            .expect("task");
        let collection_id = repository
            .create_local_review_collection(&task_id, None, "Approval")
            .expect("collection");
        let updated_at_ms: i64 = repository
            .connection
            .query_row(
                "SELECT updated_at_ms FROM local_review_collections WHERE id = ?1",
                [&collection_id],
                |row| row.get(0),
            )
            .expect("timestamp");
        let item_id = repository
            .create_local_review_approval_presentation_evidence_item(&collection_id, updated_at_ms)
            .expect("evidence");
        let sha256: String = repository
            .connection
            .query_row(
                "SELECT sha256 FROM local_review_items WHERE id = ?1",
                [&item_id],
                |row| row.get(0),
            )
            .expect("digest");
        let preview = repository
            .local_review_approval_presentation_evidence_preview(&item_id, &sha256)
            .expect("persisted preview");
        assert_eq!(
            preview.details.approval_state,
            LocalReviewEvidenceApprovalState::Approved
        );
        assert!(
            preview.details.request_present
                && preview.details.decision_present
                && preview.details.dispatch_present
                && preview.details.execution_present
        );
        repository
            .connection
            .execute(
                "UPDATE advisor_dispatch_records SET state = 'rejected' WHERE id = ?1",
                [dispatch_id],
            )
            .expect("live mutation");
        assert_eq!(
            repository
                .local_review_approval_presentation_evidence_preview(&item_id, &sha256)
                .expect("inert preview"),
            preview
        );
    }

    #[test]
    fn task_project_context_rejects_archived_or_detached_projects_without_rebinding_tasks() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let (project_id, conversation_id) = insert_live_task_context(&repository);
        let bound = repository
            .create_task_from_conversation_context(&conversation_id)
            .expect("bound task");
        repository.archive_project(&project_id).expect("archive");
        assert_eq!(
            repository
                .task_project_binding(&bound)
                .expect("binding after archive"),
            Some(project_id.clone())
        );
        repository.detach_project(&project_id).expect("detach");
        assert_eq!(
            repository
                .task_project_binding(&bound)
                .expect("binding after detach"),
            Some(project_id.clone())
        );
        assert!(matches!(
            repository.create_task_from_conversation_context(&conversation_id),
            Err(StorageError::ProjectNotFound)
        ));
        let unbound = repository
            .create_task()
            .expect("unbound creation remains supported");
        assert_eq!(
            repository.task_project_binding(&unbound).expect("unbound"),
            None
        );
    }

    #[test]
    fn task_project_context_fails_closed_for_missing_stale_or_archived_conversations() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let (_, conversation_id) = insert_live_task_context(&repository);
        assert!(matches!(
            repository
                .create_task_from_conversation_context("018f0000-0000-7000-8000-000000000199"),
            Err(StorageError::ProjectNotFound)
        ));
        repository
            .connection
            .execute(
                "UPDATE conversation_references SET status = 'completed' WHERE id = ?1",
                [&conversation_id],
            )
            .expect("complete conversation");
        assert!(matches!(
            repository.create_task_from_conversation_context(&conversation_id),
            Err(StorageError::ProjectNotFound)
        ));
        repository
            .connection
            .execute(
                "UPDATE conversation_references SET archived_at_ms = 2 WHERE id = ?1",
                [&conversation_id],
            )
            .expect("archive conversation");
        assert!(matches!(
            repository.create_task_from_conversation_context(&conversation_id),
            Err(StorageError::ProjectNotFound)
        ));
    }

    #[test]
    fn package_validation_summary_migration_upgrades_v14_without_synthesizing_history() {
        let connection = Connection::open_in_memory().expect("database opens");
        connection
            .execute_batch("CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY NOT NULL, name TEXT NOT NULL, applied_at_ms INTEGER NOT NULL);")
            .expect("ledger");
        for (version, name, sql) in MIGRATIONS.iter().take(14) {
            connection.execute_batch(sql).expect("v14 migration");
            connection
                .execute(
                    "INSERT INTO schema_migrations VALUES (?1, ?2, 1)",
                    params![version, name],
                )
                .expect("ledger row");
        }
        let repository = ProjectRepository::from_test_connection(connection).expect("v16 upgrade");
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT count(*) FROM project_package_validation_summaries",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .expect("no synthesized records"),
            0
        );
        assert_eq!(
            repository
                .connection
                .query_row("SELECT max(version) FROM schema_migrations", [], |row| row
                    .get::<_, i64>(
                    0
                ))
                .expect("version"),
            28
        );

        let mut failed = Connection::open_in_memory().expect("database opens");
        failed
            .execute_batch("CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY NOT NULL, name TEXT NOT NULL, applied_at_ms INTEGER NOT NULL);")
            .expect("ledger");
        for (version, name, sql) in MIGRATIONS.iter().take(14) {
            failed.execute_batch(sql).expect("v14 migration");
            failed
                .execute(
                    "INSERT INTO schema_migrations VALUES (?1, ?2, 1)",
                    params![version, name],
                )
                .expect("ledger row");
        }
        failed
            .execute_batch(
                "CREATE TABLE project_package_validation_summaries (marker INTEGER NOT NULL);",
            )
            .expect("conflicting fixture");
        assert!(apply_migrations(&mut failed).is_err());
        assert_eq!(
            failed
                .query_row("SELECT max(version) FROM schema_migrations", [], |row| row
                    .get::<_, i64>(
                    0
                ))
                .expect("prior ledger"),
            14
        );
        assert_eq!(
            failed
                .query_row(
                    "SELECT count(*) FROM pragma_table_info('project_package_validation_summaries') WHERE name = 'marker'",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .expect("conflicting table remains"),
            1
        );
    }

    #[test]
    fn package_validation_summaries_are_immutable_digest_bound_and_superseding() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let (project_id, _) = insert_live_task_context(&repository);
        let incomplete = package_validation_created(
            repository
                .record_package_validation_summary_at_for_test(
                    &project_id,
                    package_validation_input(false, None),
                    10,
                )
                .expect("truthful incomplete record"),
        );
        let stored = repository
            .package_validation_summary_for_test(&incomplete.id)
            .expect("digest verifies");
        assert_eq!(stored.record_sha256, incomplete.record_sha256);
        assert!(!stored.input.validation_complete);
        assert!(repository
            .connection
            .execute(
                "UPDATE project_package_validation_summaries
                 SET artifact_count = 2 WHERE id = ?1",
                [&incomplete.id],
            )
            .is_err());

        let invalid_complete = PackageValidationRecordInput {
            validation_complete: true,
            ..package_validation_input(false, None)
        };
        assert!(matches!(
            repository.record_package_validation_summary_at_for_test(
                &project_id,
                invalid_complete,
                11
            ),
            Err(StorageError::InvalidStoredValue)
        ));
        let mut complete_input = package_validation_input(true, Some(incomplete.id.clone()));
        complete_input.candidate_identity_sha256 =
            incomplete.input.candidate_identity_sha256.clone();
        complete_input.manifest_state = incomplete.input.manifest_state;
        complete_input.checksum_state = incomplete.input.checksum_state;
        complete_input.abi_state = incomplete.input.abi_state;
        complete_input.provenance_state = incomplete.input.provenance_state;
        complete_input.visible_launch_state = incomplete.input.visible_launch_state;
        complete_input.artifact_count = incomplete.input.artifact_count;
        complete_input.validation_complete = false;
        complete_input.installed_host_state = LocalReviewEvidenceCheckState::Unavailable;
        let complete = package_validation_created(
            repository
                .record_package_validation_summary_at_for_test(&project_id, complete_input, 10)
                .expect("installed-host follow-up supersedes by insertion"),
        );
        assert_ne!(complete.id, incomplete.id);
        assert_eq!(
            repository
                .package_validation_summary_for_test(&complete.id)
                .expect("complete record")
                .input
                .supersedes_record_id,
            Some(incomplete.id.clone())
        );
        assert!(repository
            .record_package_validation_summary_at_for_test(
                &project_id,
                package_validation_input(false, Some(incomplete.id.clone())),
                12,
            )
            .is_err());

        let other_project = "018f0000-0000-7000-8000-000000000201";
        insert_active_project(
            &repository,
            other_project,
            "018f0000-0000-7000-8000-000000000202",
        );
        assert!(matches!(
            repository.record_package_validation_summary_at_for_test(
                other_project,
                package_validation_input(false, Some(incomplete.id.clone())),
                12,
            ),
            Err(StorageError::InvalidStoredValue)
        ));
        assert!(matches!(
            repository.record_package_validation_summary_at_for_test(
                &project_id,
                package_validation_input(
                    false,
                    Some("018f0000-0000-7000-8000-000000000299".to_owned()),
                ),
                12,
            ),
            Err(StorageError::InvalidStoredValue)
        ));
        repository
            .record_package_validation_summary_at_for_test(
                other_project,
                package_validation_input(false, None),
                12,
            )
            .expect("other project record");
        repository
            .connection
            .execute(
                "UPDATE projects SET active_directory_association_id = NULL WHERE id = ?1",
                [other_project],
            )
            .expect("detach association for deletion check");
        repository
            .connection
            .execute(
                "DELETE FROM directory_associations WHERE project_id = ?1",
                [other_project],
            )
            .expect("remove association for deletion check");
        assert!(repository
            .connection
            .execute("DELETE FROM projects WHERE id = ?1", [other_project])
            .is_err());

        repository
            .connection
            .execute_batch("DROP TRIGGER project_package_validation_summaries_immutable;")
            .expect("test corruption setup");
        repository
            .connection
            .execute(
                "UPDATE project_package_validation_summaries
                 SET record_sha256 = ?1 WHERE id = ?2",
                params!["0".repeat(64), complete.id],
            )
            .expect("corrupt row");
        assert!(matches!(
            repository.package_validation_summary_for_test(&complete.id),
            Err(StorageError::InvalidStoredValue)
        ));
    }

    #[test]
    fn package_validation_summary_enforces_project_isolation_schema_bounds_and_retention() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let (project_id, conversation_id) = insert_live_task_context(&repository);
        let invalid_version = PackageValidationRecordInput {
            application_version: "invalid version".to_owned(),
            ..package_validation_input(false, None)
        };
        assert!(matches!(
            repository.record_package_validation_summary_at_for_test(
                &project_id,
                invalid_version,
                1
            ),
            Err(StorageError::InvalidStoredValue)
        ));
        let invalid_count = PackageValidationRecordInput {
            artifact_count: 3,
            ..package_validation_input(false, None)
        };
        assert!(matches!(
            repository.record_package_validation_summary_at_for_test(&project_id, invalid_count, 1),
            Err(StorageError::InvalidStoredValue)
        ));
        assert!(repository
            .connection
            .execute(
                "INSERT INTO project_package_validation_summaries (
                    id, project_id, application_version, debian_version,
                    manifest_state, checksum_state, abi_state, provenance_state,
                    visible_launch_state, installed_host_state, artifact_count,
                    validation_complete, record_sha256, created_at_ms, supersedes_record_id
                 ) VALUES (
                    '018f0000-0000-7000-8000-000000000188',
                    '018f0000-0000-7000-8000-000000000199', '0.1.0', '0.1.0',
                    'invalid', 'passed', 'passed', 'passed', 'passed', 'passed',
                    2, 1, '0', 1, NULL
                 )",
                [],
            )
            .is_err());

        let old = PACKAGE_VALIDATION_PROTECTION_MS + 1_000;
        let mut first = None;
        for timestamp in 0..32 {
            let receipt = package_validation_created(
                repository
                    .record_package_validation_summary_at_for_test(
                        &project_id,
                        package_validation_input(false, None),
                        timestamp,
                    )
                    .expect("capacity fixture"),
            );
            first.get_or_insert(receipt.id);
        }
        assert!(matches!(
            repository.record_package_validation_summary_at_for_test(
                &project_id,
                package_validation_input(false, None),
                old,
            ),
            Err(StorageError::TaskCapacity)
        ));
        assert!(repository
            .package_validation_summary_for_test(&first.expect("first"))
            .is_ok());
        let count: i64 = repository
            .connection
            .query_row(
                "SELECT count(*) FROM project_package_validation_summaries WHERE project_id = ?1",
                [&project_id],
                |row| row.get(0),
            )
            .expect("count");
        assert_eq!(count, 32);

        repository
            .connection
            .execute(
                "UPDATE conversation_references SET archived_at_ms = 1 WHERE id = ?1",
                [&conversation_id],
            )
            .expect("context unrelated to retention");
    }

    #[test]
    fn package_validation_summary_retention_protects_recent_and_complete_history() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let project_id = "018f0000-0000-7000-8000-000000000211";
        insert_active_project(
            &repository,
            project_id,
            "018f0000-0000-7000-8000-000000000212",
        );
        let mut predecessor_input = package_validation_input(false, None);
        predecessor_input.checksum_state = LocalReviewEvidenceCheckState::Passed;
        predecessor_input.abi_state = LocalReviewEvidenceCheckState::Passed;
        predecessor_input.provenance_state = LocalReviewEvidenceCheckState::Passed;
        predecessor_input.visible_launch_state = LocalReviewEvidenceCheckState::Passed;
        predecessor_input.artifact_count = 2;
        let predecessor = package_validation_created(
            repository
                .record_package_validation_summary_at_for_test(project_id, predecessor_input, 0)
                .expect("unprivileged history"),
        );
        let mut complete_input = package_validation_input(true, Some(predecessor.id.clone()));
        complete_input.candidate_identity_sha256 =
            predecessor.input.candidate_identity_sha256.clone();
        let complete = package_validation_created(
            repository
                .record_package_validation_summary_at_for_test(project_id, complete_input, 1)
                .expect("complete history"),
        );
        for timestamp in 2..32 {
            repository
                .record_package_validation_summary_at_for_test(
                    project_id,
                    package_validation_input(false, None),
                    timestamp,
                )
                .expect("old history");
        }
        assert!(matches!(
            repository.record_package_validation_summary_at_for_test(
                project_id,
                package_validation_input(false, None),
                PACKAGE_VALIDATION_PROTECTION_MS + 1_000,
            ),
            Err(StorageError::TaskCapacity)
        ));
        assert!(repository
            .package_validation_summary_for_test(&complete.id)
            .is_ok());

        let recent_project = "018f0000-0000-7000-8000-000000000213";
        insert_active_project(
            &repository,
            recent_project,
            "018f0000-0000-7000-8000-000000000214",
        );
        let recent = PACKAGE_VALIDATION_PROTECTION_MS.saturating_mul(2);
        for timestamp in 0..32 {
            repository
                .record_package_validation_summary_at_for_test(
                    recent_project,
                    package_validation_input(false, None),
                    recent + timestamp,
                )
                .expect("recent history");
        }
        assert!(matches!(
            repository.record_package_validation_summary_at_for_test(
                recent_project,
                package_validation_input(false, None),
                recent + 32,
            ),
            Err(StorageError::TaskCapacity)
        ));
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT count(*) FROM project_package_validation_summaries WHERE project_id = ?1",
                    [recent_project],
                    |row| row.get::<_, i64>(0),
                )
                .expect("no partial insert"),
            32
        );
    }

    #[test]
    fn controlled_browser_verification_schema_persists_bound_digests_and_recovers_prepared_work() {
        let mut repository = ProjectRepository::in_memory().expect("fresh schema");
        let project_id = Uuid::now_v7().to_string();
        insert_active_project(&repository, &project_id, &Uuid::now_v7().to_string());
        let attempt_id = Uuid::now_v7().to_string();
        repository
            .record_controlled_browser_verification(&ControlledBrowserVerificationRecord {
                attempt_id: &attempt_id,
                project_id: &project_id,
                task_id: None,
                target_digest: &"a".repeat(64),
                request_digest: &"b".repeat(64),
                authorization_id: &Uuid::now_v7().to_string(),
                expires_at_ms: i64::MAX,
            })
            .expect("prepared browser verification");
        let stored: (String, String, String) = repository
            .connection
            .query_row(
                "SELECT target_digest, request_digest, state FROM controlled_browser_verification_attempts WHERE id = ?1",
                [&attempt_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("stored attempt");
        assert_eq!(stored, ("a".repeat(64), "b".repeat(64), "prepared".into()));
        recover_interrupted_controlled_browser_verifications(&repository.connection)
            .expect("recovery is transactional");
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT state FROM controlled_browser_verification_attempts WHERE id = ?1",
                    [&attempt_id],
                    |row| row.get::<_, String>(0),
                )
                .expect("recovered state"),
            "expired"
        );
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT count(*) FROM controlled_browser_verification_audit WHERE attempt_id = ?1",
                    [&attempt_id],
                    |row| row.get::<_, i64>(0),
                )
                .expect("audit linkage"),
            2
        );
        let reviewed_id = Uuid::now_v7().to_string();
        repository.connection.execute(
            "INSERT INTO context_bundles (id,schema_version,project_id,task_id,bundle_digest,canonical_bytes,policy_version,assembly_version,state,expires_at_ms,created_at_ms,completed_at_ms,authorization_id) VALUES (?1,1,?2,NULL,?3,?4,1,1,'awaiting_review',9999999999999,1,NULL,?5)",
            params![reviewed_id, project_id, "d".repeat(64), b"reviewed bytes".as_slice(), Uuid::now_v7().to_string()],
        ).expect("reviewed bundle");
        repository.connection.execute(
            "INSERT INTO context_bundle_audit (id,bundle_id,event_kind,outcome,evidence_digest,created_at_ms) VALUES (?1,?2,'review','awaiting_review',?3,1)",
            params![Uuid::now_v7().to_string(), reviewed_id, "d".repeat(64)],
        ).expect("review audit");
        recover_interrupted_context_bundles(&repository.connection).expect("review recovery");
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT state FROM context_bundles WHERE id=?1",
                    [&reviewed_id],
                    |row| row.get::<_, String>(0)
                )
                .expect("reviewed state"),
            "expired"
        );
    }

    #[test]
    fn context_bundle_recovery_expires_prepared_authorization_without_retaining_execution() {
        let repository = ProjectRepository::in_memory().expect("fresh schema");
        assert_eq!(repository.connection.query_row("SELECT count(*) FROM pragma_table_info('context_bundles') WHERE name='canonical_bytes'", [], |row| row.get::<_, i64>(0)).expect("canonical-byte column"), 1);
        let project_id = Uuid::now_v7().to_string();
        insert_active_project(&repository, &project_id, &Uuid::now_v7().to_string());
        let bundle_id = Uuid::now_v7().to_string();
        let digest = "c".repeat(64);
        repository.connection.execute(
            "INSERT INTO context_bundles (id,schema_version,project_id,task_id,bundle_digest,canonical_bytes,policy_version,assembly_version,state,expires_at_ms,created_at_ms,completed_at_ms,authorization_id) VALUES (?1,1,?2,NULL,?3,?4,1,1,'prepared',9999999999999,1,NULL,?5)",
            params![bundle_id, project_id, digest, b"private canonical bytes".as_slice(), Uuid::now_v7().to_string()],
        ).expect("prepared context bundle");
        repository.connection.execute(
            "INSERT INTO context_bundle_audit (id,bundle_id,event_kind,outcome,evidence_digest,created_at_ms) VALUES (?1,?2,'prepared','prepared',?3,1)",
            params![Uuid::now_v7().to_string(), bundle_id, digest],
        ).expect("prepared audit");
        recover_interrupted_context_bundles(&repository.connection).expect("recovery");
        let stored: String = repository
            .connection
            .query_row(
                "SELECT state FROM context_bundles WHERE id=?1",
                [&bundle_id],
                |row| row.get(0),
            )
            .expect("state");
        assert_eq!(stored, "expired");
        assert!(repository
            .connection
            .query_row(
                "SELECT canonical_bytes IS NULL FROM context_bundles WHERE id=?1",
                [&bundle_id],
                |row| row.get::<_, bool>(0)
            )
            .expect("bytes cleared"));
        let audit_count: i64 = repository
            .connection
            .query_row(
                "SELECT count(*) FROM context_bundle_audit WHERE bundle_id=?1",
                [&bundle_id],
                |row| row.get(0),
            )
            .expect("audit");
        assert_eq!(audit_count, 2);
    }

    #[test]
    fn context_ledger_projects_only_content_free_receipt_metadata() {
        let repository = ProjectRepository::in_memory().expect("fresh schema");
        let project_id = Uuid::now_v7().to_string();
        insert_active_project(&repository, &project_id, &Uuid::now_v7().to_string());
        let bundle_id = Uuid::now_v7().to_string();
        repository.connection.execute(
            "INSERT INTO context_bundles (id,schema_version,project_id,task_id,bundle_digest,canonical_bytes,policy_version,assembly_version,state,expires_at_ms,created_at_ms,completed_at_ms,authorization_id) VALUES (?1,1,?2,NULL,?3,?4,1,1,'closed',9,3,4,?5)",
            params![bundle_id, project_id, "e".repeat(64), b"never project these bytes".as_slice(), Uuid::now_v7().to_string()],
        ).expect("receipt");
        repository.connection.execute(
            "INSERT INTO context_bundle_audit (id,bundle_id,event_kind,outcome,evidence_digest,created_at_ms) VALUES (?1,?2,'terminal','closed',?3,4)",
            params![Uuid::now_v7().to_string(), bundle_id, "e".repeat(64)],
        ).expect("audit");
        let durable_id = Uuid::now_v7().to_string();
        repository.connection.execute(
            "INSERT INTO durable_sources (id,schema_version,project_id,task_id,source_class,title,origin_display,byte_size,line_count,sha256,content_locator,state,created_at_ms,updated_at_ms,deleted_at_ms) VALUES (?1,1,?2,NULL,'manual-text','Safe local source',NULL,0,0,?3,?4,'active',5,5,NULL)",
            params![durable_id, project_id, "d".repeat(64), Uuid::now_v7().to_string()],
        ).expect("durable source");
        let artifact_id = Uuid::now_v7().to_string();
        repository.connection.execute(
            "INSERT INTO artifact_references (id,schema_version,project_id,task_id,artifact_id,artifact_sha256,artifact_class,display_label,state,created_at_ms,deleted_at_ms) VALUES (?1,1,?2,NULL,?3,?4,'text','Safe artifact','active',6,NULL)",
            params![artifact_id, project_id, Uuid::now_v7().to_string(), "a".repeat(64)],
        ).expect("artifact reference");
        let browser_id = Uuid::now_v7().to_string();
        repository.connection.execute(
            "INSERT INTO controlled_browser_verification_attempts (id,schema_version,project_id,task_id,fixture_id,target_digest,request_digest,authorization_id,state,expires_at_ms,created_at_ms,completed_at_ms,evidence_digest) VALUES (?1,1,?2,NULL,'fictional-webkitgtk-local-v1',?3,?4,?5,'prepared',9,7,NULL,NULL)",
            params![browser_id, project_id, "b".repeat(64), "c".repeat(64), Uuid::now_v7().to_string()],
        ).expect("browser verification");
        let entries = repository.context_ledger(&project_id).expect("ledger");
        assert_eq!(entries.len(), 4);
        let context_bundle = entries
            .iter()
            .find(|entry| entry.record_kind == "context-bundle")
            .expect("context bundle");
        assert_eq!(context_bundle.record_id, bundle_id);
        assert_eq!(context_bundle.state, "closed");
        assert_eq!(context_bundle.audit_outcome, "closed");
        assert_eq!(context_bundle.bundle_digest, "e".repeat(64));
        let durable_source = entries
            .iter()
            .find(|entry| entry.record_kind == "durable-source")
            .expect("durable source");
        assert_eq!(durable_source.record_id, durable_id);
        assert_eq!(durable_source.bundle_digest, "d".repeat(64));
        assert!(entries.iter().any(
            |entry| entry.record_kind == "artifact-reference" && entry.record_id == artifact_id
        ));
        assert!(entries
            .iter()
            .any(|entry| entry.record_kind == "browser-verification"
                && entry.record_id == browser_id));
    }

    #[test]
    fn local_runtime_start_consumes_bytes_before_execution_and_completes_once() {
        let mut repository = ProjectRepository::in_memory().expect("fresh schema");
        let project_id = Uuid::now_v7().to_string();
        insert_active_project(&repository, &project_id, &Uuid::now_v7().to_string());
        let bundle_id = Uuid::now_v7().to_string();
        let digest = "e".repeat(64);
        repository.connection.execute(
            "INSERT INTO context_bundles (id,schema_version,project_id,task_id,bundle_digest,canonical_bytes,policy_version,assembly_version,state,expires_at_ms,created_at_ms,completed_at_ms,authorization_id) VALUES (?1,1,?2,NULL,?3,?4,1,1,'awaiting_confirmation',9999999999999,1,NULL,?5)",
            params![bundle_id, project_id, digest, b"private canonical bytes".as_slice(), Uuid::now_v7().to_string()],
        ).expect("confirmed context bundle");
        repository
            .start_local_runtime_context_bundle(&bundle_id)
            .expect("runtime start");
        let stored: (String, bool) = repository
            .connection
            .query_row(
                "SELECT state, canonical_bytes IS NULL FROM context_bundles WHERE id=?1",
                [&bundle_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("running state");
        assert_eq!(stored, ("dispatching".into(), true));
        assert_eq!(repository.connection.query_row(
            "SELECT count(*) FROM context_bundle_audit WHERE bundle_id=?1 AND event_kind IN ('authorized','dispatching')",
            [&bundle_id],
            |row| row.get::<_, i64>(0),
        ).expect("pre-execution audit"), 2);
        repository
            .complete_context_bundle(&bundle_id, "closed")
            .expect("completed local runtime is terminal");
        let completed: (String, bool, i64) = repository
            .connection
            .query_row(
                "SELECT state, canonical_bytes IS NULL, completed_at_ms IS NOT NULL FROM context_bundles WHERE id=?1",
                [&bundle_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("completed runtime state");
        assert_eq!(
            completed,
            ("closed".into(), true, 1),
            "the one consumed local attempt has one durable terminal outcome"
        );
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT count(*) FROM context_bundle_audit WHERE bundle_id=?1 AND event_kind='terminal' AND outcome='closed'",
                    [&bundle_id],
                    |row| row.get::<_, i64>(0),
                )
                .expect("terminal audit"),
            1
        );
    }

    #[test]
    fn local_runtime_failure_is_a_terminal_bundle_outcome() {
        let mut repository = ProjectRepository::in_memory().expect("fresh schema");
        let project_id = Uuid::now_v7().to_string();
        insert_active_project(&repository, &project_id, &Uuid::now_v7().to_string());
        let bundle_id = Uuid::now_v7().to_string();
        repository.connection.execute(
            "INSERT INTO context_bundles (id,schema_version,project_id,task_id,bundle_digest,canonical_bytes,policy_version,assembly_version,state,expires_at_ms,created_at_ms,completed_at_ms,authorization_id) VALUES (?1,1,?2,NULL,?3,?4,1,1,'awaiting_confirmation',9999999999999,1,NULL,?5)",
            params![bundle_id, project_id, "f".repeat(64), b"private canonical bytes".as_slice(), Uuid::now_v7().to_string()],
        ).expect("confirmed context bundle");
        repository
            .start_local_runtime_context_bundle(&bundle_id)
            .expect("runtime start");
        repository
            .complete_context_bundle(&bundle_id, "failed")
            .expect("failed local runtime is terminal");
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT state FROM context_bundles WHERE id=?1",
                    [&bundle_id],
                    |row| row.get::<_, String>(0),
                )
                .expect("failed runtime state"),
            "failed"
        );
    }

    #[test]
    fn package_validation_identity_schema_and_v15_upgrade_are_closed() {
        let repository = ProjectRepository::in_memory().expect("fresh schema");
        for (kind, name) in [
            ("table", "project_package_validation_candidate_identities"),
            (
                "index",
                "project_package_validation_candidate_identities_summary",
            ),
            (
                "trigger",
                "project_package_validation_candidate_identities_immutable",
            ),
        ] {
            assert_eq!(
                repository
                    .connection
                    .query_row(
                        "SELECT count(*) FROM sqlite_master WHERE type = ?1 AND name = ?2",
                        params![kind, name],
                        |row| row.get::<_, i64>(0),
                    )
                    .expect("schema object"),
                1
            );
        }
        assert_eq!(MIGRATIONS.len(), 28);

        let connection = Connection::open_in_memory().expect("database");
        connection
            .execute_batch("CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY NOT NULL, name TEXT NOT NULL, applied_at_ms INTEGER NOT NULL);")
            .expect("ledger");
        for (version, name, sql) in MIGRATIONS.iter().take(15) {
            connection.execute_batch(sql).expect("v15 migration");
            connection
                .execute(
                    "INSERT INTO schema_migrations VALUES (?1, ?2, 1)",
                    params![version, name],
                )
                .expect("ledger row");
        }
        let historical_project_id = Uuid::now_v7().to_string();
        connection
            .execute(
                "INSERT INTO projects (id, display_name, active_directory_association_id, archived_at_ms, created_at_ms, updated_at_ms)
                 VALUES (?1, 'Historical project', NULL, NULL, 1, 1)",
                [&historical_project_id],
            )
            .expect("historical project");
        connection
            .execute(
                "INSERT INTO project_package_validation_summaries (
                    id, project_id, application_version, debian_version, manifest_state,
                    checksum_state, abi_state, provenance_state, visible_launch_state,
                    installed_host_state, artifact_count, validation_complete, record_sha256,
                    created_at_ms, supersedes_record_id
                 ) VALUES (?1, ?2, '0.1.0', '0.1.0', 'passed', 'skipped', 'skipped',
                    'skipped', 'skipped', 'unavailable', 2, 0, ?3, 1, NULL)",
                params![
                    Uuid::now_v7().to_string(),
                    historical_project_id,
                    "a".repeat(64)
                ],
            )
            .expect("historical v15 summary");
        let repository = ProjectRepository::from_test_connection(connection).expect("v17 upgrade");
        assert_eq!(
            repository
                .connection
                .query_row("SELECT max(version) FROM schema_migrations", [], |row| row
                    .get::<_, i64>(
                    0
                ))
                .expect("version"),
            28
        );
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT count(*) FROM project_package_validation_candidate_identities",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .expect("unassociated history"),
            0
        );
    }

    #[test]
    fn package_validation_phase_migration_upgrades_v16_associations_atomically() {
        let connection = Connection::open_in_memory().expect("database");
        connection.execute_batch("CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY NOT NULL, name TEXT NOT NULL, applied_at_ms INTEGER NOT NULL);").expect("ledger");
        for (version, name, sql) in MIGRATIONS.iter().take(16) {
            connection.execute_batch(sql).expect("v16 migration");
            connection
                .execute(
                    "INSERT INTO schema_migrations VALUES (?1, ?2, 1)",
                    params![version, name],
                )
                .expect("ledger row");
        }
        let project_id = Uuid::now_v7().to_string();
        let summary_id = Uuid::now_v7().to_string();
        connection.execute("INSERT INTO projects (id, display_name, active_directory_association_id, archived_at_ms, created_at_ms, updated_at_ms) VALUES (?1, 'fixture', NULL, NULL, 1, 1)", [&project_id]).expect("project");
        connection.execute(
            "INSERT INTO project_package_validation_summaries (id, project_id, application_version, debian_version, manifest_state, checksum_state, abi_state, provenance_state, visible_launch_state, installed_host_state, artifact_count, validation_complete, record_sha256, created_at_ms, supersedes_record_id) VALUES (?1, ?2, '0.1.0', '0.1.0', 'passed', 'unavailable', 'unavailable', 'unavailable', 'unavailable', 'unavailable', 2, 0, ?3, 1, NULL)",
            params![summary_id, project_id, "a".repeat(64)],
        ).expect("summary");
        connection.execute("INSERT INTO project_package_validation_candidate_identities VALUES (?1, ?2, ?3, 1)", params![project_id, "b".repeat(64), summary_id]).expect("v16 association");
        let repository =
            ProjectRepository::from_test_connection(connection).expect("v17 migration");
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT validation_phase FROM project_package_validation_candidate_identities",
                    [],
                    |row| row.get::<_, String>(0)
                )
                .expect("phase"),
            "unprivileged"
        );
        assert_eq!(
            repository
                .connection
                .query_row("SELECT max(version) FROM schema_migrations", [], |row| row
                    .get::<_, i64>(
                    0
                ))
                .expect("version"),
            28
        );
    }

    #[test]
    fn package_validation_attempt_identity_is_path_free_and_phase_chain_is_immutable() {
        let identity = "a".repeat(64);
        assert_eq!(
            installed_host_attempt_identity(&identity, LocalReviewEvidenceCheckState::Failed, None)
                .expect("vector"),
            "d63b4382e2db22a6d2e23530b5a4ca5dc7b884dc0fa84c990b89537792502124"
        );
        assert_ne!(
            installed_host_attempt_identity(&identity, LocalReviewEvidenceCheckState::Failed, None)
                .expect("failed"),
            installed_host_attempt_identity(
                &identity,
                LocalReviewEvidenceCheckState::Unavailable,
                None
            )
            .expect("unavailable")
        );
        let full_facts = PackageValidationInstalledHostFacts {
            package_state: "installed".to_owned(),
            version_match: true,
            ownership_verified: true,
            permissions_safe: true,
            package_integrity_verified: true,
        };
        assert_eq!(
            installed_host_attempt_identity(
                &identity,
                LocalReviewEvidenceCheckState::Passed,
                Some(&full_facts)
            )
            .expect("full vector"),
            "b9b196b15bc6852bcbe5e2ba9fe71e0aa722e2f93f0150bfb9953ae03158e501"
        );
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let (project_id, _) = insert_live_task_context(&repository);
        let root = package_validation_created(
            repository
                .record_package_validation_summary_at_for_test(
                    &project_id,
                    package_validation_input_with_identity(&identity),
                    1,
                )
                .expect("unprivileged"),
        );
        let mut failed = package_validation_input_with_identity(&identity);
        failed.validation_phase = PackageValidationPhase::InstalledHost;
        failed.supersedes_record_id = Some(root.id.clone());
        failed.installed_host_state = LocalReviewEvidenceCheckState::Failed;
        let failed = package_validation_created(
            repository
                .record_package_validation_summary_at_for_test(&project_id, failed, 2)
                .expect("first attempt"),
        );
        let mut passed = package_validation_input_with_identity(&identity);
        passed.validation_phase = PackageValidationPhase::InstalledHost;
        passed.supersedes_record_id = Some(failed.id.clone());
        passed.installed_host_state = LocalReviewEvidenceCheckState::Passed;
        passed.installed_host_facts = Some(PackageValidationInstalledHostFacts {
            package_state: "installed".to_owned(),
            version_match: true,
            ownership_verified: true,
            permissions_safe: true,
            package_integrity_verified: true,
        });
        let passed = package_validation_created(
            repository
                .record_package_validation_summary_at_for_test(&project_id, passed, 3)
                .expect("second attempt"),
        );
        assert_eq!(passed.input.supersedes_record_id, Some(failed.id));
        assert!(matches!(
            repository.record_package_validation_summary_at_for_test(
                &project_id,
                PackageValidationRecordInput {
                    validation_phase: PackageValidationPhase::InstalledHost,
                    supersedes_record_id: passed.input.supersedes_record_id.clone(),
                    installed_host_state: LocalReviewEvidenceCheckState::Passed,
                    installed_host_facts: Some(PackageValidationInstalledHostFacts {
                        package_state: "installed".to_owned(),
                        version_match: true,
                        ownership_verified: true,
                        permissions_safe: true,
                        package_integrity_verified: true,
                    }),
                    ..package_validation_input_with_identity(&identity)
                },
                4
            ),
            Ok(PackageValidationRecordOutcome::Existing(_))
        ));
        assert_eq!(repository.connection.query_row(
            "SELECT count(*) FROM project_package_validation_candidate_identities WHERE project_id = ?1 AND candidate_identity_sha256 = ?2",
            params![project_id, identity], |row| row.get::<_, i64>(0)
        ).expect("chain count"), 3);
    }

    #[test]
    fn package_validation_identity_recorder_is_idempotent_isolated_and_verified() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let (project_id, _) = insert_live_task_context(&repository);
        let identity = "1".repeat(64);
        for malformed in [
            "",
            "A".repeat(64).as_str(),
            "g".repeat(64).as_str(),
            "a".repeat(63).as_str(),
        ] {
            assert!(matches!(
                repository.record_package_validation_summary_at_for_test(
                    &project_id,
                    package_validation_input_with_identity(malformed),
                    1,
                ),
                Err(StorageError::InvalidStoredValue)
            ));
        }
        let created = package_validation_created(
            repository
                .record_package_validation_summary_at_for_test(
                    &project_id,
                    package_validation_input_with_identity(&identity),
                    2,
                )
                .expect("created"),
        );
        let existing = repository
            .record_package_validation_summary_at_for_test(
                &project_id,
                package_validation_input_with_identity(&identity),
                3,
            )
            .expect("existing");
        match existing {
            PackageValidationRecordOutcome::Existing(summary) => {
                assert_eq!(summary.id, created.id);
                assert_eq!(summary.record_sha256, created.record_sha256);
            }
            PackageValidationRecordOutcome::Created(_) => panic!("identity must be idempotent"),
        }
        assert_eq!(
            repository.connection.query_row("SELECT count(*) FROM project_package_validation_summaries WHERE project_id = ?1", [&project_id], |row| row.get::<_, i64>(0)).expect("summary count"),
            1
        );
        assert_eq!(
            repository.connection.query_row("SELECT count(*) FROM project_package_validation_candidate_identities WHERE project_id = ?1", [&project_id], |row| row.get::<_, i64>(0)).expect("identity count"),
            1
        );
        let mut installed = package_validation_input_with_identity(&identity);
        installed.validation_phase = PackageValidationPhase::InstalledHost;
        installed.supersedes_record_id = Some(created.id.clone());
        let installed = package_validation_created(
            repository
                .record_package_validation_summary_at_for_test(&project_id, installed, 4)
                .expect("installed-host phase creates a distinct immutable summary"),
        );
        assert_ne!(installed.id, created.id);
        let mut installed_retry = package_validation_input_with_identity(&identity);
        installed_retry.validation_phase = PackageValidationPhase::InstalledHost;
        installed_retry.supersedes_record_id = Some(created.id.clone());
        assert!(matches!(
            repository.record_package_validation_summary_at_for_test(&project_id, installed_retry, 5),
            Ok(PackageValidationRecordOutcome::Existing(summary)) if summary.id == installed.id
        ));
        assert!(matches!(
            repository.record_package_validation_summary_at_for_test(
                &project_id,
                PackageValidationRecordInput {
                    validation_phase: PackageValidationPhase::InstalledHost,
                    ..package_validation_input_with_identity(&"9".repeat(64))
                },
                5,
            ),
            Err(StorageError::InvalidStoredValue)
        ));
        assert_eq!(repository.connection.query_row(
            "SELECT count(*) FROM project_package_validation_candidate_identities WHERE project_id = ?1 AND candidate_identity_sha256 = ?2",
            params![project_id, identity], |row| row.get::<_, i64>(0)
        ).expect("one association per phase"), 2);
        let second = package_validation_created(
            repository
                .record_package_validation_summary_at_for_test(
                    &project_id,
                    package_validation_input_with_identity(&"2".repeat(64)),
                    4,
                )
                .expect("different identity"),
        );
        assert_ne!(second.id, created.id);

        let other_project = "018f0000-0000-7000-8000-000000000701";
        insert_active_project(
            &repository,
            other_project,
            "018f0000-0000-7000-8000-000000000702",
        );
        let isolated = package_validation_created(
            repository
                .record_package_validation_summary_at_for_test(
                    other_project,
                    package_validation_input_with_identity(&identity),
                    5,
                )
                .expect("cross-project identity is isolated"),
        );
        assert_ne!(isolated.id, created.id);
        assert!(repository.connection.execute(
            "UPDATE project_package_validation_candidate_identities SET created_at_ms = 9 WHERE project_id = ?1",
            [&project_id],
        ).is_err());
        assert!(repository.connection.execute(
            "INSERT INTO project_package_validation_candidate_identities VALUES (?1, ?2, 'unprivileged', ?3, 1)",
            params![project_id, identity, created.id],
        ).is_err());
    }

    #[test]
    fn package_validation_identity_corruption_rollback_and_retention_fail_closed() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let (project_id, _) = insert_live_task_context(&repository);
        let identity = "3".repeat(64);
        repository
            .connection
            .execute_batch(
                "CREATE TRIGGER package_validation_identity_abort
             BEFORE INSERT ON project_package_validation_candidate_identities
             BEGIN SELECT RAISE(ABORT, 'fixture'); END;",
            )
            .expect("abort trigger");
        assert!(repository
            .record_package_validation_summary_at_for_test(
                &project_id,
                package_validation_input_with_identity(&identity),
                1,
            )
            .is_err());
        for table in [
            "project_package_validation_summaries",
            "project_package_validation_candidate_identities",
        ] {
            assert_eq!(
                repository
                    .connection
                    .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| row
                        .get::<_, i64>(0))
                    .expect("rollback count"),
                0
            );
        }
        repository
            .connection
            .execute_batch("DROP TRIGGER package_validation_identity_abort;")
            .expect("drop trigger");
        let created = package_validation_created(
            repository
                .record_package_validation_summary_at_for_test(
                    &project_id,
                    package_validation_input_with_identity(&identity),
                    2,
                )
                .expect("retry after rollback"),
        );
        repository
            .connection
            .execute_batch("DROP TRIGGER project_package_validation_summaries_immutable;")
            .expect("corrupt summary");
        repository
            .connection
            .execute(
                "UPDATE project_package_validation_summaries SET record_sha256 = ?1 WHERE id = ?2",
                params!["0".repeat(64), created.id],
            )
            .expect("corruption fixture");
        assert!(matches!(
            repository.record_package_validation_summary_at_for_test(
                &project_id,
                package_validation_input_with_identity(&identity),
                3,
            ),
            Err(StorageError::InvalidStoredValue)
        ));

        let protected_project = "018f0000-0000-7000-8000-000000000711";
        insert_active_project(
            &repository,
            protected_project,
            "018f0000-0000-7000-8000-000000000712",
        );
        for index in 0..32 {
            package_validation_created(
                repository
                    .record_package_validation_summary_at_for_test(
                        protected_project,
                        package_validation_input_with_identity(&format!("{index:064x}")),
                        0,
                    )
                    .expect("protected record"),
            );
        }
        assert!(matches!(
            repository.record_package_validation_summary_at_for_test(
                protected_project,
                package_validation_input_with_identity(&"f".repeat(64)),
                PACKAGE_VALIDATION_PROTECTION_MS + 1,
            ),
            Err(StorageError::TaskCapacity)
        ));
        repository
            .connection
            .execute(
                "UPDATE projects SET active_directory_association_id = NULL WHERE id = ?1",
                [protected_project],
            )
            .expect("detach for deletion check");
        repository
            .connection
            .execute(
                "DELETE FROM directory_associations WHERE project_id = ?1",
                [protected_project],
            )
            .expect("remove association");
        assert!(repository
            .connection
            .execute("DELETE FROM projects WHERE id = ?1", [protected_project])
            .is_err());
    }

    #[test]
    fn package_validation_identity_missing_and_cross_project_associations_fail_closed() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let (first_project, _) = insert_live_task_context(&repository);
        let second_project = "018f0000-0000-7000-8000-000000000721";
        insert_active_project(
            &repository,
            second_project,
            "018f0000-0000-7000-8000-000000000722",
        );
        let first_identity = "7".repeat(64);
        let second_identity = "8".repeat(64);
        let first = package_validation_created(
            repository
                .record_package_validation_summary_at_for_test(
                    &first_project,
                    package_validation_input_with_identity(&first_identity),
                    1,
                )
                .expect("first"),
        );
        let second = package_validation_created(
            repository
                .record_package_validation_summary_at_for_test(
                    second_project,
                    package_validation_input_with_identity(&second_identity),
                    1,
                )
                .expect("second"),
        );
        repository
            .connection
            .execute_batch(
                "DROP TRIGGER project_package_validation_candidate_identities_immutable;",
            )
            .expect("corruption fixture");
        repository.connection.execute(
            "DELETE FROM project_package_validation_candidate_identities WHERE package_validation_summary_id = ?1",
            [&second.id],
        ).expect("free second summary");
        repository
            .connection
            .execute(
                "UPDATE project_package_validation_candidate_identities
             SET package_validation_summary_id = ?1 WHERE project_id = ?2",
                params![second.id, first_project],
            )
            .expect("cross-project corruption");
        assert!(matches!(
            repository.record_package_validation_summary_at_for_test(
                &first_project,
                package_validation_input_with_identity(&first_identity),
                2,
            ),
            Err(StorageError::InvalidStoredValue)
        ));
        repository
            .connection
            .execute_batch("PRAGMA foreign_keys = OFF;")
            .expect("disable fk for missing fixture");
        repository
            .connection
            .execute(
                "UPDATE project_package_validation_candidate_identities
             SET package_validation_summary_id = ?1 WHERE project_id = ?2",
                params![Uuid::now_v7().to_string(), first_project],
            )
            .expect("missing summary corruption");
        assert!(matches!(
            repository.record_package_validation_summary_at_for_test(
                &first_project,
                package_validation_input_with_identity(&first_identity),
                3,
            ),
            Err(StorageError::InvalidStoredValue)
        ));
        assert_ne!(first.id, second.id);
    }

    #[test]
    fn task_ids_retry_collisions_and_fail_closed_after_the_bound() {
        let mut repository = ProjectRepository::in_memory().expect("schema must migrate");
        let first_task = "018f0000-0000-7000-8000-000000000001".to_owned();
        let first_plan = "018f0000-0000-7000-8000-000000000002".to_owned();
        let mut initial = VecDeque::from([first_task.clone(), first_plan.clone()]);
        repository
            .create_task_with(10, || initial.pop_front().expect("fixture id"))
            .expect("first task must create");

        let second_task = "018f0000-0000-7000-8000-000000000003".to_owned();
        let second_plan = "018f0000-0000-7000-8000-000000000004".to_owned();
        let mut retry = VecDeque::from([
            first_task.clone(),
            second_task.clone(),
            first_plan.clone(),
            second_plan,
        ]);
        assert_eq!(
            repository
                .create_task_with(20, || retry.pop_front().expect("fixture id"))
                .expect("collisions must retry"),
            second_task
        );

        assert!(matches!(
            repository.create_task_with(30, || first_task.clone()),
            Err(StorageError::DuplicateId)
        ));
        assert_eq!(
            repository
                .connection
                .query_row("SELECT count(*) FROM task_records", [], |row| row
                    .get::<_, i64>(0))
                .expect("task count"),
            2
        );
    }

    #[test]
    fn failed_task_migration_rolls_back_without_rewriting_the_prior_schema() {
        let root =
            std::env::temp_dir().join(format!("quireforge-task-migration-{}", Uuid::now_v7()));
        fs::create_dir_all(&root).expect("fixture root");
        let database = root.join("metadata.sqlite3");
        let connection = Connection::open(&database).expect("database opens");
        connection
            .execute_batch(
                "CREATE TABLE schema_migrations (
                    version INTEGER PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL,
                    applied_at_ms INTEGER NOT NULL
                 );",
            )
            .expect("ledger");
        for (version, name, sql) in MIGRATIONS.iter().take(10) {
            connection.execute_batch(sql).expect("prior migration");
            connection
                .execute(
                    "INSERT INTO schema_migrations VALUES (?1, ?2, 1)",
                    params![version, name],
                )
                .expect("prior ledger row");
        }
        connection
            .execute_batch("CREATE TABLE task_records (marker TEXT NOT NULL);")
            .expect("conflicting partial fixture");
        drop(connection);

        let connection = Connection::open(&database).expect("database reopens");
        assert!(matches!(
            ProjectRepository::from_test_connection(connection),
            Err(StorageError::Sqlite(_))
        ));
        let verification = Connection::open(&database).expect("verification opens");
        assert_eq!(
            verification
                .query_row("SELECT max(version) FROM schema_migrations", [], |row| {
                    row.get::<_, i64>(0)
                })
                .expect("prior ledger remains"),
            10
        );
        assert_eq!(
            verification
                .query_row(
                    "SELECT count(*) FROM pragma_table_info('task_records') WHERE name = 'marker'",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .expect("conflicting table remains"),
            1
        );
        assert_eq!(
            verification
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'task_plans'",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .expect("task plan table remains absent"),
            0
        );
        drop(verification);
        fs::remove_file(database).expect("database cleanup");
        fs::remove_dir(root).expect("fixture root cleanup");
    }

    #[test]
    fn task_schema_is_closed_and_restart_preserves_only_task_metadata() {
        let root = std::env::temp_dir().join(format!("quireforge-task-restart-{}", Uuid::now_v7()));
        let database = root.join("data/metadata.sqlite3");
        let mut repository = ProjectRepository::open(&database).expect("metadata opens");
        let task = repository.create_task().expect("task creates");
        repository
            .rename_task(&task, "Restart-safe")
            .expect("title saves");
        drop(repository);

        let mut reopened = ProjectRepository::open(&database).expect("metadata reopens");
        let snapshot = reopened
            .task_catalog(Some(&task), false, None)
            .expect("task catalog reloads");
        assert_eq!(snapshot.1.expect("selected task").title, "Restart-safe");

        let task_columns: Vec<String> = reopened
            .connection
            .prepare("SELECT name FROM pragma_table_info('task_records') ORDER BY cid")
            .expect("task columns")
            .query_map([], |row| row.get(0))
            .expect("task column rows")
            .collect::<Result<_, _>>()
            .expect("valid task columns");
        assert_eq!(
            task_columns,
            vec![
                "id",
                "schema_version",
                "title",
                "status",
                "created_at_ms",
                "updated_at_ms",
                "archived_at_ms",
                "last_opened_at_ms",
                "selected_plan_id",
                "project_id",
                "origin_advisor_conversation_id",
                "origin_advisor_dispatch_record_id",
            ]
        );
        let plan_columns: Vec<String> = reopened
            .connection
            .prepare("SELECT name FROM pragma_table_info('task_plans') ORDER BY cid")
            .expect("plan columns")
            .query_map([], |row| row.get(0))
            .expect("plan column rows")
            .collect::<Result<_, _>>()
            .expect("valid plan columns");
        assert_eq!(
            plan_columns,
            vec![
                "id",
                "schema_version",
                "task_id",
                "label",
                "position",
                "body",
                "created_at_ms",
                "updated_at_ms",
            ]
        );
        let all_columns = task_columns
            .iter()
            .chain(plan_columns.iter())
            .cloned()
            .collect::<Vec<_>>();
        for forbidden in [
            "path",
            "conversation_id",
            "approval_id",
            "dispatch_id",
            "execution_id",
            "terminal_id",
            "attachment_id",
            "artifact_id",
            "credential",
            "provider",
            "transcript",
            "prompt",
        ] {
            assert!(!all_columns.iter().any(|column| column == forbidden));
        }
        drop(reopened);
        fs::remove_file(database).expect("database cleanup");
        fs::remove_dir_all(root).expect("fixture root cleanup");
    }

    #[test]
    fn task_status_archive_and_restore_follow_the_closed_lifecycle() {
        let mut repository = ProjectRepository::in_memory().expect("schema must migrate");
        let task = repository.create_task().expect("task must create");
        assert!(matches!(
            repository.set_task_status(&task, TaskStatus::Active),
            Err(StorageError::InvalidStatusTransition)
        ));
        repository
            .set_task_status(&task, TaskStatus::Paused)
            .expect("active may pause");
        repository
            .set_task_status(&task, TaskStatus::Completed)
            .expect("paused may complete");
        assert!(matches!(
            repository.set_task_status(&task, TaskStatus::Paused),
            Err(StorageError::InvalidStatusTransition)
        ));
        repository
            .set_task_status(&task, TaskStatus::Active)
            .expect("completed may explicitly reopen");
        repository
            .archive_task(&task, false)
            .expect("task may archive");
        assert!(matches!(
            repository.rename_task(&task, "Blocked"),
            Err(StorageError::TaskArchived)
        ));
        assert!(matches!(
            repository.create_plan(&task, false),
            Err(StorageError::TaskArchived)
        ));
        repository
            .archive_task(&task, true)
            .expect("task may restore");
        repository
            .rename_task(&task, "Restored")
            .expect("restored task may edit");
    }

    #[test]
    fn task_search_is_unicode_label_only_and_cleanup_is_deterministic() {
        let mut repository = ProjectRepository::in_memory().expect("schema must migrate");
        let mut ids = VecDeque::from([
            "018f0000-0000-7000-8000-000000000011".to_owned(),
            "018f0000-0000-7000-8000-000000000012".to_owned(),
        ]);
        let task = repository
            .create_task_with(100, || ids.pop_front().expect("fixture id"))
            .expect("task must create");
        repository
            .rename_task(&task, "Résumé planning ΟΣ")
            .expect("unicode title must persist");
        let plan = repository
            .create_plan(&task, false)
            .expect("alternate must create");
        repository
            .edit_plan(
                &task,
                &plan,
                "Überprüfung",
                "secret body-only search marker\nsecond line",
            )
            .expect("multiline body must persist");

        assert_eq!(
            repository
                .task_catalog_at(Some(&task), false, Some("RÉSUMÉ"), None, 100)
                .expect("unicode title search")
                .0
                .len(),
            1
        );
        assert_eq!(
            repository
                .task_catalog_at(Some(&task), false, Some("ÜBER"), None, 100)
                .expect("unicode label search")
                .0
                .len(),
            1
        );
        assert_eq!(
            repository
                .task_catalog_at(Some(&task), false, Some("οσ"), None, 100)
                .expect("Unicode simple-fold search")
                .0
                .len(),
            1
        );
        assert!(repository
            .task_catalog_at(Some(&task), false, Some("secret body-only"), None, 100)
            .expect("body must not be indexed")
            .0
            .is_empty());

        repository
            .set_task_status(&task, TaskStatus::Completed)
            .expect("task may complete");
        let updated: i64 = repository
            .connection
            .query_row(
                "SELECT updated_at_ms FROM task_records WHERE id = ?1",
                [&task],
                |row| row.get(0),
            )
            .expect("updated time");
        assert!(
            !repository
                .task_catalog_at(
                    Some(&task),
                    false,
                    None,
                    None,
                    updated + TASK_CLEANUP_AGE_MS - 1,
                )
                .expect("catalog")
                .0[0]
                .cleanup_eligible
        );
        assert!(
            repository
                .task_catalog_at(
                    Some(&task),
                    false,
                    None,
                    None,
                    updated + TASK_CLEANUP_AGE_MS,
                )
                .expect("catalog")
                .0[0]
                .cleanup_eligible
        );
    }

    #[test]
    fn plan_capacity_copy_delete_and_selected_repair_are_atomic() {
        let mut repository = ProjectRepository::in_memory().expect("schema must migrate");
        let task = repository.create_task().expect("task must create");
        let primary: String = repository
            .connection
            .query_row(
                "SELECT selected_plan_id FROM task_records WHERE id = ?1",
                [&task],
                |row| row.get(0),
            )
            .expect("primary id");
        repository
            .edit_plan(&task, &primary, "Primary plan", "visible primary text")
            .expect("primary text must save");
        let copied = repository
            .create_plan(&task, true)
            .expect("copy must create");
        let copied_body: String = repository
            .connection
            .query_row(
                "SELECT body FROM task_plans WHERE id = ?1",
                [&copied],
                |row| row.get(0),
            )
            .expect("copied body");
        assert_eq!(copied_body, "visible primary text");
        let third = repository.create_plan(&task, false).expect("third plan");
        repository.create_plan(&task, false).expect("fourth plan");
        assert!(matches!(
            repository.create_plan(&task, false),
            Err(StorageError::PlanCapacity)
        ));
        repository
            .delete_plan(&task, &copied)
            .expect("selected or non-selected plan may delete");
        let positions: Vec<i64> = repository
            .connection
            .prepare("SELECT position FROM task_plans WHERE task_id = ?1 ORDER BY position")
            .expect("positions query")
            .query_map([&task], |row| row.get(0))
            .expect("position rows")
            .collect::<Result<_, _>>()
            .expect("valid positions");
        assert_eq!(positions, vec![0, 1, 2]);

        repository
            .connection
            .execute(
                "UPDATE task_records SET selected_plan_id = ?1 WHERE id = ?2",
                params!["018f0000-0000-7000-8000-000000000099", task],
            )
            .expect("stale selection fixture");
        let selected = repository
            .task_catalog(Some(&task), false, None)
            .expect("selection must repair")
            .1
            .expect("task remains selected");
        assert_eq!(selected.selected_plan_id, primary);
        assert!(repository.delete_plan(&task, &third).is_ok());
    }

    #[test]
    fn task_capacity_refuses_without_eviction_or_partial_mutation() {
        let mut repository = ProjectRepository::in_memory().expect("schema must migrate");
        for _ in 0..TASK_COUNT_LIMIT {
            repository.create_task().expect("bounded task must create");
        }
        assert!(matches!(
            repository.create_task(),
            Err(StorageError::TaskCapacity)
        ));
        assert_eq!(
            repository
                .connection
                .query_row("SELECT count(*) FROM task_records", [], |row| row
                    .get::<_, i64>(0))
                .expect("count"),
            TASK_COUNT_LIMIT
        );

        let first: String = repository
            .connection
            .query_row(
                "SELECT id FROM task_records ORDER BY created_at_ms, id LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("first task");
        let primary: String = repository
            .connection
            .query_row(
                "SELECT selected_plan_id FROM task_records WHERE id = ?1",
                [&first],
                |row| row.get(0),
            )
            .expect("primary");
        repository
            .edit_plan(&first, &primary, "Primary plan", &"🦀".repeat(7_500))
            .expect("first body fits");
        let alternate = repository
            .create_plan(&first, false)
            .expect("alternate fits");
        assert!(matches!(
            repository.edit_plan(&first, &alternate, "Alternate plan", &"界".repeat(7_500)),
            Err(StorageError::TaskCapacity)
        ));
        let body: String = repository
            .connection
            .query_row(
                "SELECT body FROM task_plans WHERE id = ?1",
                [&alternate],
                |row| row.get(0),
            )
            .expect("rolled-back body");
        assert!(body.is_empty());
    }

    #[test]
    fn aggregate_payload_uses_utf8_bytes_and_refuses_at_eight_mib() {
        let mut repository = ProjectRepository::in_memory().expect("schema must migrate");
        let large = "🦀".repeat(8_000);
        let smaller = "界".repeat(4_000);
        for _ in 0..190 {
            let task = Uuid::now_v7().to_string();
            let primary = Uuid::now_v7().to_string();
            let alternate = Uuid::now_v7().to_string();
            repository
                .connection
                .execute(
                    "INSERT INTO task_records (
                        id, schema_version, title, status, created_at_ms, updated_at_ms,
                        archived_at_ms, last_opened_at_ms, selected_plan_id
                     ) VALUES (?1, 1, 'Payload', 'active', 1, 1, NULL, 1, ?2)",
                    params![task, primary],
                )
                .expect("task fixture");
            repository
                .connection
                .execute(
                    "INSERT INTO task_plans (
                        id, schema_version, task_id, label, position, body,
                        created_at_ms, updated_at_ms
                     ) VALUES (?1, 1, ?2, 'Primary plan', 0, ?3, 1, 1),
                              (?4, 1, ?2, 'Alternate plan 1', 1, ?5, 1, 1)",
                    params![primary, task, large, alternate, smaller],
                )
                .expect("plan fixtures");
        }
        let bytes = super::task_payload_bytes(&repository.connection, None)
            .expect("payload bytes must calculate");
        assert!(bytes > TASK_PAYLOAD_LIMIT);
        assert!(matches!(
            repository.create_task(),
            Err(StorageError::TaskCapacity)
        ));
        assert_eq!(
            repository
                .connection
                .query_row("SELECT count(*) FROM task_records", [], |row| row
                    .get::<_, i64>(0))
                .expect("count"),
            190
        );
    }

    #[test]
    fn corrupt_task_rows_are_omitted_with_one_bounded_warning() {
        let mut repository = ProjectRepository::in_memory().expect("schema must migrate");
        let task = repository.create_task().expect("task must create");
        repository
            .connection
            .execute_batch("PRAGMA ignore_check_constraints = ON;")
            .expect("constraint bypass");
        repository
            .connection
            .execute(
                "UPDATE task_records SET status = 'unsafe' WHERE id = ?1",
                [&task],
            )
            .expect("corrupt fixture");
        let (tasks, selected, plans, count, _, corrupt) = repository
            .task_catalog(Some(&task), false, None)
            .expect("bounded catalog read");
        assert!(tasks.is_empty());
        assert!(selected.is_none());
        assert!(plans.is_empty());
        assert_eq!(count, 1);
        assert!(corrupt);
    }

    #[test]
    fn knowledge_records_bind_only_owner_records_and_preserve_history() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let project_id = Uuid::now_v7().to_string();
        insert_active_project(&repository, &project_id, &Uuid::now_v7().to_string());
        let decision = repository
            .create_knowledge_record(
                &project_id,
                None,
                KnowledgeRecordKind::OwnerDecision,
                "Linux only",
                "The product remains Linux only.",
                None,
            )
            .expect("record");
        assert_eq!(
            repository.knowledge_records(&project_id).expect("list")[0].status,
            KnowledgeRecordStatus::Proposed
        );
        repository
            .transition_knowledge_record(&decision, KnowledgeRecordStatus::PendingOwnerBinding)
            .expect("owner prepares binding");
        repository
            .bind_knowledge_record(&decision)
            .expect("owner binds");
        assert_eq!(
            repository.knowledge_records(&project_id).expect("list")[0].status,
            KnowledgeRecordStatus::Active
        );
        let claim = repository
            .create_knowledge_record(
                &project_id,
                None,
                KnowledgeRecordKind::AgentClaim,
                "A claim",
                "A non-binding claim.",
                Some(&decision),
            )
            .expect("claim");
        assert!(repository.bind_knowledge_record(&claim).is_err());
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT count(*) FROM knowledge_record_events WHERE record_id=?1",
                    [&decision],
                    |row| row.get::<_, i64>(0)
                )
                .expect("events"),
            3
        );
    }

    #[test]
    fn knowledge_records_enforce_nonbinding_lifecycle_and_project_scoped_supersession() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let project_id = Uuid::now_v7().to_string();
        let other_project_id = Uuid::now_v7().to_string();
        insert_active_project(&repository, &project_id, &Uuid::now_v7().to_string());
        insert_active_project(&repository, &other_project_id, &Uuid::now_v7().to_string());
        let fact = repository
            .create_knowledge_record(
                &project_id,
                None,
                KnowledgeRecordKind::ObservedFact,
                "Verified source",
                "The source was checked locally.",
                None,
            )
            .expect("fact");
        assert!(repository
            .transition_knowledge_record(&fact, KnowledgeRecordStatus::Validated)
            .is_err());
        repository
            .transition_knowledge_record(&fact, KnowledgeRecordStatus::Active)
            .expect("activate fact");
        repository
            .transition_knowledge_record(&fact, KnowledgeRecordStatus::Validated)
            .expect("validate fact");
        repository
            .transition_knowledge_record(&fact, KnowledgeRecordStatus::Superseded)
            .expect("supersede fact");
        assert!(repository
            .transition_knowledge_record(&fact, KnowledgeRecordStatus::Retired)
            .is_err());
        assert!(repository
            .create_knowledge_record(
                &other_project_id,
                None,
                KnowledgeRecordKind::ObservedFact,
                "Invalid cross-project successor",
                "A successor cannot cross project boundaries.",
                Some(&fact),
            )
            .is_err());
        let successor = repository
            .create_knowledge_record(
                &project_id,
                None,
                KnowledgeRecordKind::ObservedFact,
                "Corrected source",
                "A successor remains linked without rewriting history.",
                Some(&fact),
            )
            .expect("same-project successor");
        assert_eq!(
            repository
                .knowledge_records(&project_id)
                .expect("records")
                .iter()
                .find(|record| record.id == successor)
                .expect("successor")
                .supersedes_id
                .as_deref(),
            Some(fact.as_str())
        );
    }

    #[test]
    fn task_deletion_cascades_only_plans_and_preserves_external_files() {
        let root = std::env::temp_dir().join(format!("quireforge-task-delete-{}", Uuid::now_v7()));
        fs::create_dir_all(&root).expect("fixture root");
        let external = root.join("project-source.txt");
        fs::write(&external, "must remain").expect("external fixture");

        let mut repository = ProjectRepository::in_memory().expect("schema must migrate");
        let task = repository.create_task().expect("task must create");
        repository.create_plan(&task, false).expect("alternate");
        repository.delete_task(&task).expect("task must delete");
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT count(*) FROM task_plans WHERE task_id = ?1",
                    [&task],
                    |row| row.get::<_, i64>(0),
                )
                .expect("plan count"),
            0
        );
        assert_eq!(
            fs::read_to_string(&external).expect("external file remains"),
            "must remain"
        );
        fs::remove_file(external).expect("fixture file cleanup");
        fs::remove_dir(root).expect("fixture directory cleanup");
    }

    #[test]
    fn local_review_text_is_task_scoped_digest_bound_and_capacity_accounted() {
        let mut repository = ProjectRepository::in_memory().expect("schema must migrate");
        let task_id = repository.create_task().expect("task must create");
        let (_, selected, plans, _, _, _) = repository
            .task_catalog(Some(&task_id), false, None)
            .expect("catalog must load");
        let plan_id = selected.expect("task selected").selected_plan_id;
        assert!(plans.iter().any(|plan| plan.id == plan_id));
        let collection_id = repository
            .create_local_review_collection(&task_id, Some(&plan_id), "Review brief")
            .expect("collection must create");
        let (_, collection, _, _, _, _) = repository
            .local_review_snapshot(Some(&collection_id))
            .expect("collection must project");
        let collection = collection.expect("collection selected");
        repository
            .create_local_review_text_item(
                &collection_id,
                collection.updated_at_ms,
                "Brief",
                LocalReviewTextFormat::Plain,
                "line one\r\nline two",
            )
            .expect("text item must create");
        let (_, collection, items, _, payload, _) = repository
            .local_review_snapshot(Some(&collection_id))
            .expect("review must project");
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].source_kind,
            LocalReviewSourceKind::UserAuthoredText
        );
        assert_eq!(items[0].byte_size, "line one\nline two".len() as u64);
        assert!(payload >= items[0].byte_size);
        assert!(collection.expect("collection selected").updated_at_ms >= 0);
    }

    #[test]
    fn local_review_package_manifest_summary_uses_only_completed_bound_record() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let (project_id, conversation_id) = insert_live_task_context(&repository);
        let task_id = repository
            .create_task_from_conversation_context(&conversation_id)
            .expect("bound task");
        let collection_id = repository
            .create_local_review_collection(&task_id, None, "Package review")
            .expect("collection");
        let identity = "d".repeat(64);
        let predecessor = package_validation_created(
            repository
                .record_package_validation_summary(
                    &project_id,
                    headless_predecessor_input(&identity),
                )
                .expect("unprivileged receipt"),
        );
        let mut completed = headless_predecessor_input(&identity);
        completed.validation_phase = PackageValidationPhase::InstalledHost;
        completed.installed_host_state = LocalReviewEvidenceCheckState::Passed;
        completed.validation_complete = true;
        completed.supersedes_record_id = Some(predecessor.id);
        completed.installed_host_facts = Some(PackageValidationInstalledHostFacts {
            package_state: "installed".to_owned(),
            version_match: true,
            ownership_verified: true,
            permissions_safe: true,
            package_integrity_verified: true,
        });
        let source_before = repository.package_validation_phase_summary_for_test(
            &project_id,
            PackageValidationPhase::InstalledHost,
        );
        assert!(source_before.is_err());
        package_validation_created(
            repository
                .record_package_validation_summary(&project_id, completed)
                .expect("complete receipt"),
        );
        let (_, collection, _, _, _, _) = repository
            .local_review_snapshot(Some(&collection_id))
            .expect("snapshot");
        let updated = collection.expect("collection").updated_at_ms;
        let item_id = repository
            .create_local_review_package_manifest_summary_evidence_item(&collection_id, updated)
            .expect("capture");
        let item = repository
            .local_review_package_manifest_summary_evidence_preview(
                &item_id,
                &repository
                    .connection
                    .query_row(
                        "SELECT sha256 FROM local_review_items WHERE id = ?1",
                        [&item_id],
                        |row| row.get::<_, String>(0),
                    )
                    .expect("digest"),
            )
            .expect("preview");
        assert_eq!(item.details.artifact_count, 2);
        assert!(item.details.validation_complete);
        assert_eq!(
            item.details.installed_host_state,
            LocalReviewEvidenceCheckState::Passed
        );
        assert!(repository
            .package_manifest_summary_source_for_local_review(&collection_id)
            .is_ok());
        let unbound = repository.create_task().expect("unbound");
        let unbound_collection = repository
            .create_local_review_collection(&unbound, None, "Unbound")
            .expect("collection");
        assert!(
            !repository.package_manifest_summary_available_for_local_review(&unbound_collection)
        );
    }

    #[test]
    fn local_review_annotation_is_item_scoped_normalized_and_stale_safe() {
        let mut repository = ProjectRepository::in_memory().expect("schema must migrate");
        let task_id = repository.create_task().expect("task must create");
        let (_, selected, _, _, _, _) = repository
            .task_catalog(Some(&task_id), false, None)
            .expect("catalog must load");
        let plan_id = selected.expect("selected task").selected_plan_id;
        let collection_id = repository
            .create_local_review_collection(&task_id, Some(&plan_id), "Annotation review")
            .expect("collection must create");
        let collection = repository
            .local_review_snapshot(Some(&collection_id))
            .expect("snapshot")
            .1
            .expect("selected collection");
        let first = repository
            .create_local_review_text_item(
                &collection_id,
                collection.updated_at_ms,
                "First item",
                LocalReviewTextFormat::Plain,
                "first content",
            )
            .expect("first item");
        let collection = repository
            .local_review_snapshot(Some(&collection_id))
            .expect("snapshot")
            .1
            .expect("selected collection");
        let second = repository
            .create_local_review_text_item(
                &collection_id,
                collection.updated_at_ms,
                "Second item",
                LocalReviewTextFormat::Plain,
                "second content",
            )
            .expect("second item");
        let (_, before, before_items, _, _, _) = repository
            .local_review_snapshot(Some(&collection_id))
            .expect("snapshot");
        let before = before.expect("selected collection");
        let first_before = before_items
            .iter()
            .find(|item| item.item_id == first)
            .expect("first projection")
            .clone();
        let task_before: (String, i64) = repository
            .connection
            .query_row(
                "SELECT status, updated_at_ms FROM task_records WHERE id = ?1",
                [&task_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("task record");
        let plan_before: (String, i64) = repository
            .connection
            .query_row(
                "SELECT body, updated_at_ms FROM task_plans WHERE id = ?1",
                [&plan_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("plan record");

        let annotation_id = repository
            .create_local_review_annotation(
                &collection_id,
                &first,
                before.updated_at_ms,
                "note one\r\nnote two",
            )
            .expect("annotation must create");
        assert!(valid_task_id(&annotation_id));
        let (_, after_first, _, _, _, _) = repository
            .local_review_snapshot(Some(&collection_id))
            .expect("authoritative snapshot");
        let after_first = after_first.expect("selected collection");
        let second_annotation_id = repository
            .create_local_review_annotation(
                &collection_id,
                &first,
                after_first.updated_at_ms,
                "later note",
            )
            .expect("second annotation must create");
        let (_, after, after_items, _, _, _) = repository
            .local_review_snapshot(Some(&collection_id))
            .expect("authoritative snapshot");
        let after = after.expect("selected collection");
        assert!(after.updated_at_ms > before.updated_at_ms);
        let annotated = after_items
            .iter()
            .find(|item| item.item_id == first)
            .expect("annotated item");
        let sibling = after_items
            .iter()
            .find(|item| item.item_id == second)
            .expect("sibling item");
        assert_eq!(annotated.annotations.len(), 2);
        assert!(sibling.annotations.is_empty());
        let annotation = &annotated.annotations[0];
        assert_eq!(annotation.annotation_id, annotation_id);
        assert_eq!(annotated.annotations[1].annotation_id, second_annotation_id);
        assert_eq!(annotation.item_id, first);
        assert_eq!(annotation.text, "note one\nnote two");
        assert_eq!(annotation.state, LocalReviewAnnotationState::Open);
        assert!(annotation.created_at_ms >= 0);
        assert_eq!(annotation.created_at_ms, annotation.updated_at_ms);
        assert_eq!(annotated.sha256, first_before.sha256);
        assert_eq!(annotated.byte_size, first_before.byte_size);
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT status, updated_at_ms FROM task_records WHERE id = ?1",
                    [&task_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .expect("task unchanged"),
            task_before
        );
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT body, updated_at_ms FROM task_plans WHERE id = ?1",
                    [&plan_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .expect("plan unchanged"),
            plan_before
        );
        let projected = serde_json::to_value(annotation).expect("annotation serializes");
        for forbidden in [
            "author",
            "path",
            "url",
            "range",
            "coordinate",
            "mention",
            "approval",
            "dispatch",
            "execution",
        ] {
            assert!(
                projected.get(forbidden).is_none(),
                "{forbidden} must not project"
            );
        }
        assert!(matches!(
            repository.create_local_review_annotation(
                &collection_id,
                &first,
                before.updated_at_ms,
                "stale annotation",
            ),
            Err(StorageError::InvalidStatusTransition)
        ));
        let (_, rejected, rejected_items, _, _, _) = repository
            .local_review_snapshot(Some(&collection_id))
            .expect("snapshot after rejection");
        assert_eq!(
            rejected.expect("selected").updated_at_ms,
            after.updated_at_ms
        );
        assert_eq!(
            rejected_items
                .iter()
                .find(|item| item.item_id == first)
                .expect("first remains")
                .annotations,
            annotated.annotations
        );
    }

    #[test]
    fn local_review_text_comparison_is_same_format_digest_bound_and_non_mutating() {
        let mut repository = ProjectRepository::in_memory().expect("schema migrates");
        let task = repository.create_task().expect("task");
        let (_, selected, _, _, _, _) = repository
            .task_catalog(Some(&task), false, None)
            .expect("catalog");
        let plan = selected.expect("selected").selected_plan_id;
        let collection = repository
            .create_local_review_collection(&task, Some(&plan), "Comparison")
            .expect("collection");
        let mut selected = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot")
            .1
            .expect("selected");
        let left = repository
            .create_local_review_text_item(
                &collection,
                selected.updated_at_ms,
                "Left",
                LocalReviewTextFormat::Plain,
                "same\nleft",
            )
            .expect("left");
        selected = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot")
            .1
            .expect("selected");
        let right = repository
            .create_local_review_text_item(
                &collection,
                selected.updated_at_ms,
                "Right",
                LocalReviewTextFormat::Plain,
                "same\nright",
            )
            .expect("right");
        let (_, before, items, _, _, _) = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot");
        let before = before.expect("selected");
        let left_before = items
            .iter()
            .find(|item| item.item_id == left)
            .expect("left")
            .clone();
        let right_before = items
            .iter()
            .find(|item| item.item_id == right)
            .expect("right")
            .clone();
        let task_before: (String, i64) = repository
            .connection
            .query_row(
                "SELECT status, updated_at_ms FROM task_records WHERE id = ?1",
                [&task],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("task");
        let plan_before: (String, i64) = repository
            .connection
            .query_row(
                "SELECT body, updated_at_ms FROM task_plans WHERE id = ?1",
                [&plan],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("plan");
        let comparison = repository
            .create_local_review_text_comparison(&collection, &left, &right, before.updated_at_ms)
            .expect("comparison");
        assert!(valid_task_id(&comparison));
        let comparisons = repository
            .local_review_comparisons(&collection)
            .expect("projection");
        assert_eq!(comparisons.len(), 1);
        assert_eq!(comparisons[0].left_sha256, left_before.sha256);
        assert_eq!(comparisons[0].right_sha256, right_before.sha256);
        assert_eq!(comparisons[0].state, LocalReviewComparisonState::Ready);
        let lines = repository
            .local_review_line_comparison(&comparison)
            .expect("line diff");
        assert_eq!(lines.state, LocalReviewComparisonState::Ready);
        assert_eq!(
            lines
                .lines
                .iter()
                .map(|line| (&line.kind, line.text.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (&LocalReviewLineKind::Unchanged, "same"),
                (&LocalReviewLineKind::Added, "right"),
                (&LocalReviewLineKind::Removed, "left")
            ]
        );
        let (_, after, items, _, _, _) = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot");
        let after = after.expect("selected");
        assert!(after.updated_at_ms > before.updated_at_ms);
        assert_eq!(
            items
                .iter()
                .find(|item| item.item_id == left)
                .expect("left"),
            &left_before
        );
        assert_eq!(
            items
                .iter()
                .find(|item| item.item_id == right)
                .expect("right"),
            &right_before
        );
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT status, updated_at_ms FROM task_records WHERE id = ?1",
                    [&task],
                    |row| Ok((row.get(0)?, row.get(1)?))
                )
                .expect("task"),
            task_before
        );
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT body, updated_at_ms FROM task_plans WHERE id = ?1",
                    [&plan],
                    |row| Ok((row.get(0)?, row.get(1)?))
                )
                .expect("plan"),
            plan_before
        );
        assert!(matches!(
            repository.create_local_review_text_comparison(
                &collection,
                &left,
                &left,
                after.updated_at_ms
            ),
            Err(StorageError::InvalidStoredValue)
        ));
        let markdown = repository
            .create_local_review_text_item(
                &collection,
                after.updated_at_ms,
                "Markdown",
                LocalReviewTextFormat::Markdown,
                "text",
            )
            .expect("markdown");
        let selected_updated_at_ms: i64 = repository
            .connection
            .query_row(
                "SELECT updated_at_ms FROM local_review_collections WHERE id = ?1",
                [&collection],
                |row| row.get(0),
            )
            .expect("collection timestamp");
        assert!(matches!(
            repository.create_local_review_text_comparison(
                &collection,
                &left,
                &markdown,
                selected_updated_at_ms
            ),
            Err(StorageError::InvalidStoredValue)
        ));
        let image = repository
            .create_local_review_image_item(
                &collection,
                selected_updated_at_ms,
                "Image",
                &png(1, 1, false),
            )
            .expect("image");
        let selected = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot")
            .1
            .expect("selected");
        let image_comparison = repository.create_local_review_text_comparison(
            &collection,
            &left,
            &image,
            selected.updated_at_ms,
        );
        assert!(
            matches!(image_comparison, Err(StorageError::InvalidStoredValue)),
            "{image_comparison:?}"
        );
        let evidence = repository
            .create_local_review_manual_evidence_item(
                &collection,
                selected.updated_at_ms,
                "Evidence",
                "summary",
            )
            .expect("evidence");
        let selected_updated_at_ms: i64 = repository
            .connection
            .query_row(
                "SELECT updated_at_ms FROM local_review_collections WHERE id = ?1",
                [&collection],
                |row| row.get(0),
            )
            .expect("collection timestamp");
        assert!(matches!(
            repository.create_local_review_text_comparison(
                &collection,
                &left,
                &evidence,
                selected_updated_at_ms
            ),
            Err(StorageError::InvalidStoredValue)
        ));
        assert!(matches!(
            repository.create_local_review_text_comparison(
                &collection,
                &left,
                &right,
                before.updated_at_ms
            ),
            Err(StorageError::InvalidStatusTransition)
        ));
        repository
            .connection
            .execute(
                "UPDATE local_review_items SET content = ?1, sha256 = ?2 WHERE id = ?3",
                params![
                    b"same\nchanged".to_vec(),
                    review_digest(b"same\nchanged"),
                    left
                ],
            )
            .expect("stale fixture");
        assert!(matches!(
            repository
                .local_review_comparisons(&collection)
                .expect("comparison projection")[0]
                .state,
            LocalReviewComparisonState::Stale | LocalReviewComparisonState::Unavailable
        ));
        let projected = serde_json::to_value(&comparisons[0]).expect("serializes");
        for forbidden in [
            "path",
            "git",
            "repository",
            "shell",
            "command",
            "provider",
            "approval",
            "dispatch",
            "execution",
        ] {
            assert!(projected.get(forbidden).is_none());
        }
    }

    #[test]
    fn local_review_annotation_lifecycle_is_strict_and_ordered() {
        let mut repository = ProjectRepository::in_memory().expect("schema migrates");
        let task = repository.create_task().expect("task");
        let (_, selected, _, _, _, _) = repository
            .task_catalog(Some(&task), false, None)
            .expect("catalog");
        let plan = selected.expect("selected").selected_plan_id;
        let collection = repository
            .create_local_review_collection(&task, Some(&plan), "Lifecycle annotations")
            .expect("collection");
        let mut selected = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot")
            .1
            .expect("selected");
        let item = repository
            .create_local_review_text_item(
                &collection,
                selected.updated_at_ms,
                "Item",
                LocalReviewTextFormat::Plain,
                "immutable content",
            )
            .expect("item");
        selected = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot")
            .1
            .expect("selected");
        let first = repository
            .create_local_review_annotation(&collection, &item, selected.updated_at_ms, "first")
            .expect("first annotation");
        selected = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot")
            .1
            .expect("selected");
        let second = repository
            .create_local_review_annotation(&collection, &item, selected.updated_at_ms, "second")
            .expect("second annotation");
        let (_, before_edit, items, _, _, _) = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot");
        let before_edit = before_edit.expect("selected");
        let item_before = items
            .iter()
            .find(|candidate| candidate.item_id == item)
            .expect("item")
            .clone();
        let first_before = item_before
            .annotations
            .iter()
            .find(|annotation| annotation.annotation_id == first)
            .expect("first")
            .clone();
        let task_before: (String, i64) = repository
            .connection
            .query_row(
                "SELECT status, updated_at_ms FROM task_records WHERE id = ?1",
                [&task],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("task unchanged fixture");
        let plan_before: (String, i64) = repository
            .connection
            .query_row(
                "SELECT body, updated_at_ms FROM task_plans WHERE id = ?1",
                [&plan],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("plan unchanged fixture");

        repository
            .edit_local_review_annotation(
                &collection,
                &item,
                &first,
                before_edit.updated_at_ms,
                "edited\r\ntext",
            )
            .expect("edit");
        let (_, after_edit, items, _, _, _) = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot");
        let after_edit = after_edit.expect("selected");
        let item_after_edit = items
            .iter()
            .find(|candidate| candidate.item_id == item)
            .expect("item");
        let first_after_edit = item_after_edit
            .annotations
            .iter()
            .find(|annotation| annotation.annotation_id == first)
            .expect("first");
        assert_eq!(first_after_edit.text, "edited\ntext");
        assert_eq!(first_after_edit.state, LocalReviewAnnotationState::Open);
        assert_eq!(first_after_edit.created_at_ms, first_before.created_at_ms);
        assert!(first_after_edit.updated_at_ms > first_before.updated_at_ms);
        assert_eq!(item_after_edit.sha256, item_before.sha256);

        repository
            .resolve_local_review_annotation(&collection, &item, &first, after_edit.updated_at_ms)
            .expect("resolve");
        let (_, after_resolve, items, _, _, _) = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot");
        let after_resolve = after_resolve.expect("selected");
        let annotations = &items
            .iter()
            .find(|candidate| candidate.item_id == item)
            .expect("item")
            .annotations;
        assert_eq!(
            annotations
                .iter()
                .map(|annotation| annotation.annotation_id.as_str())
                .collect::<Vec<_>>(),
            vec![second.as_str(), first.as_str()]
        );
        assert_eq!(annotations[1].state, LocalReviewAnnotationState::Resolved);
        assert_eq!(annotations[1].text, "edited\ntext");
        assert!(annotations[1].updated_at_ms > first_after_edit.updated_at_ms);
        assert!(matches!(
            repository.resolve_local_review_annotation(
                &collection,
                &item,
                &first,
                after_resolve.updated_at_ms
            ),
            Err(StorageError::InvalidStatusTransition)
        ));
        let after_repeated_resolve = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot")
            .1
            .expect("selected");
        assert_eq!(
            after_repeated_resolve.updated_at_ms,
            after_resolve.updated_at_ms
        );

        repository
            .reopen_local_review_annotation(&collection, &item, &first, after_resolve.updated_at_ms)
            .expect("reopen");
        let (_, after_reopen, items, _, _, _) = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot");
        let after_reopen = after_reopen.expect("selected");
        let annotations = &items
            .iter()
            .find(|candidate| candidate.item_id == item)
            .expect("item")
            .annotations;
        assert_eq!(
            annotations
                .iter()
                .map(|annotation| annotation.annotation_id.as_str())
                .collect::<Vec<_>>(),
            vec![first.as_str(), second.as_str()]
        );
        assert_eq!(annotations[0].state, LocalReviewAnnotationState::Open);
        assert!(matches!(
            repository.reopen_local_review_annotation(
                &collection,
                &item,
                &first,
                after_reopen.updated_at_ms
            ),
            Err(StorageError::InvalidStatusTransition)
        ));
        assert!(matches!(
            repository.edit_local_review_annotation(
                &collection,
                &item,
                &first,
                before_edit.updated_at_ms,
                "stale"
            ),
            Err(StorageError::InvalidStatusTransition)
        ));
        assert!(matches!(
            repository.delete_local_review_annotation(
                &collection,
                &item,
                &second,
                before_edit.updated_at_ms
            ),
            Err(StorageError::InvalidStatusTransition)
        ));
        let (_, before_delete, _, _, _, _) = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot");
        let before_delete = before_delete.expect("selected");
        repository
            .delete_local_review_annotation(
                &collection,
                &item,
                &second,
                before_delete.updated_at_ms,
            )
            .expect("delete");
        let (_, after_delete, items, _, _, _) = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot");
        assert_eq!(
            items
                .iter()
                .find(|candidate| candidate.item_id == item)
                .expect("item")
                .annotations
                .len(),
            1
        );
        assert!(after_delete.expect("selected").updated_at_ms > before_delete.updated_at_ms);
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT status, updated_at_ms FROM task_records WHERE id = ?1",
                    [&task],
                    |row| Ok((row.get(0)?, row.get(1)?))
                )
                .expect("task unchanged"),
            task_before
        );
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT body, updated_at_ms FROM task_plans WHERE id = ?1",
                    [&plan],
                    |row| Ok((row.get(0)?, row.get(1)?))
                )
                .expect("plan unchanged"),
            plan_before
        );
    }

    #[test]
    fn local_review_annotation_limits_and_discard_recover_capacity() {
        let mut repository = ProjectRepository::in_memory().expect("schema migrates");
        let task = repository.create_task().expect("task");
        let collection = repository
            .create_local_review_collection(&task, None, "Annotation limits")
            .expect("collection");
        let mut selected = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot")
            .1
            .expect("selected");
        let item = repository
            .create_local_review_text_item(
                &collection,
                selected.updated_at_ms,
                "Item",
                LocalReviewTextFormat::Plain,
                "content",
            )
            .expect("item");
        selected = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot")
            .1
            .expect("selected");
        let initial_collection_payload = selected.payload_bytes;
        let initial_total_payload = repository.local_review_snapshot(None).expect("snapshot").4;
        let mut ids = Vec::new();
        for index in 0..23 {
            let id = repository
                .create_local_review_annotation(
                    &collection,
                    &item,
                    selected.updated_at_ms,
                    &format!("n{index}"),
                )
                .expect("annotation");
            ids.push(id);
            selected = repository
                .local_review_snapshot(Some(&collection))
                .expect("snapshot")
                .1
                .expect("selected");
            assert!(!selected.warning);
        }
        let warning_id = repository
            .create_local_review_annotation(&collection, &item, selected.updated_at_ms, "warning")
            .expect("twenty fourth annotation");
        ids.push(warning_id);
        selected = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot")
            .1
            .expect("selected");
        assert!(selected.warning);
        for index in 24..32 {
            let id = repository
                .create_local_review_annotation(
                    &collection,
                    &item,
                    selected.updated_at_ms,
                    &format!("n{index}"),
                )
                .expect("within annotation limit");
            ids.push(id);
            selected = repository
                .local_review_snapshot(Some(&collection))
                .expect("snapshot")
                .1
                .expect("selected");
        }
        let before_rejection = selected.clone();
        assert!(matches!(
            repository.create_local_review_annotation(
                &collection,
                &item,
                selected.updated_at_ms,
                "overflow"
            ),
            Err(StorageError::TaskCapacity)
        ));
        let (_, after_rejection, items, _, total_after_rejection, _) = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot");
        assert_eq!(
            after_rejection.expect("selected").updated_at_ms,
            before_rejection.updated_at_ms
        );
        assert_eq!(items[0].annotations.len(), 32);
        assert!(total_after_rejection > initial_total_payload);
        assert!(before_rejection.payload_bytes > initial_collection_payload);
        repository
            .delete_local_review_annotation(
                &collection,
                &item,
                &ids[0],
                before_rejection.updated_at_ms,
            )
            .expect("discard annotation");
        selected = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot")
            .1
            .expect("selected");
        let (_, _, items, _, total_after_delete, _) = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot");
        assert_eq!(items[0].annotations.len(), 31);
        assert!(selected.payload_bytes < before_rejection.payload_bytes);
        assert!(total_after_delete < total_after_rejection);
        repository
            .create_local_review_annotation(
                &collection,
                &item,
                selected.updated_at_ms,
                "replacement",
            )
            .expect("recovered annotation capacity");

        let large_task = repository.create_task().expect("large task");
        let large_collection = repository
            .create_local_review_collection(&large_task, None, "Annotation bytes")
            .expect("collection");
        let mut large_selected = repository
            .local_review_snapshot(Some(&large_collection))
            .expect("snapshot")
            .1
            .expect("selected");
        let large_item = repository
            .create_local_review_text_item(
                &large_collection,
                large_selected.updated_at_ms,
                "Item",
                LocalReviewTextFormat::Plain,
                "content",
            )
            .expect("item");
        large_selected = repository
            .local_review_snapshot(Some(&large_collection))
            .expect("snapshot")
            .1
            .expect("selected");
        repository
            .create_local_review_annotation(
                &large_collection,
                &large_item,
                large_selected.updated_at_ms,
                &"a".repeat(767),
            )
            .expect("below byte warning");
        large_selected = repository
            .local_review_snapshot(Some(&large_collection))
            .expect("snapshot")
            .1
            .expect("selected");
        assert!(!large_selected.warning);
        repository
            .create_local_review_annotation(
                &large_collection,
                &large_item,
                large_selected.updated_at_ms,
                &"b".repeat(768),
            )
            .expect("byte warning");
        large_selected = repository
            .local_review_snapshot(Some(&large_collection))
            .expect("snapshot")
            .1
            .expect("selected");
        assert!(large_selected.warning);
        let before_invalid = large_selected.clone();
        assert!(matches!(
            repository.create_local_review_annotation(
                &large_collection,
                &large_item,
                large_selected.updated_at_ms,
                &"c".repeat(1025)
            ),
            Err(StorageError::InvalidStoredValue)
        ));
        assert!(matches!(
            repository.create_local_review_annotation(
                &large_collection,
                &large_item,
                large_selected.updated_at_ms,
                &"d".repeat(1025)
            ),
            Err(StorageError::InvalidStoredValue)
        ));
        assert_eq!(
            repository
                .local_review_snapshot(Some(&large_collection))
                .expect("snapshot")
                .1
                .expect("selected")
                .updated_at_ms,
            before_invalid.updated_at_ms
        );
    }

    #[test]
    fn local_review_lifecycle_projects_task_state_and_requires_explicit_fresh_resume() {
        let mut repository = ProjectRepository::in_memory().expect("schema migrates");
        let task = repository.create_task().expect("task creates");
        let (_, selected, _, _, _, _) = repository
            .task_catalog(Some(&task), false, None)
            .expect("catalog");
        let plan = selected.expect("selected").selected_plan_id;
        let collection = repository
            .create_local_review_collection(&task, Some(&plan), "Lifecycle")
            .expect("collection");
        let state = |repository: &mut ProjectRepository| {
            repository
                .local_review_snapshot(Some(&collection))
                .expect("snapshot")
                .1
                .expect("selected")
        };
        assert_eq!(
            state(&mut repository).state,
            LocalReviewCollectionState::Active
        );
        repository
            .set_task_status(&task, TaskStatus::Paused)
            .expect("pause");
        assert_eq!(
            state(&mut repository).state,
            LocalReviewCollectionState::Active
        );
        repository
            .set_task_status(&task, TaskStatus::Completed)
            .expect("complete");
        let frozen = state(&mut repository);
        assert_eq!(frozen.state, LocalReviewCollectionState::Frozen);
        assert!(matches!(
            repository.resume_local_review_collection(&collection, frozen.updated_at_ms),
            Err(StorageError::TaskArchived)
        ));
        repository
            .set_task_status(&task, TaskStatus::Active)
            .expect("restore status");
        assert_eq!(
            state(&mut repository).state,
            LocalReviewCollectionState::Frozen
        );
        repository
            .resume_local_review_collection(&collection, frozen.updated_at_ms)
            .expect("explicit resume");
        let resumed = state(&mut repository);
        repository
            .connection
            .execute(
                "UPDATE task_plans SET updated_at_ms = updated_at_ms + 1 WHERE id = ?1",
                [&plan],
            )
            .expect("stale plan");
        assert!(matches!(
            repository.resume_local_review_collection(&collection, resumed.updated_at_ms),
            Err(StorageError::PlanNotFound)
        ));
        repository.delete_task(&task).expect("delete task");
        assert_eq!(
            state(&mut repository).state,
            LocalReviewCollectionState::Orphaned
        );
    }

    #[test]
    fn local_review_mutations_revalidate_lifecycle_and_preserve_recovery_discards() {
        let mut repository = ProjectRepository::in_memory().expect("schema migrates");
        let task = repository.create_task().expect("task");
        let (_, selected_task, _, _, _, _) = repository
            .task_catalog(Some(&task), false, None)
            .expect("catalog");
        let plan = selected_task.expect("selected task").selected_plan_id;
        let collection = repository
            .create_local_review_collection(&task, Some(&plan), "Lifecycle gate")
            .expect("collection");
        let current = |repository: &mut ProjectRepository| {
            repository
                .local_review_snapshot(Some(&collection))
                .expect("snapshot")
                .1
                .expect("selected collection")
        };
        let initial = current(&mut repository);
        let retained_item = repository
            .create_local_review_text_item(
                &collection,
                initial.updated_at_ms,
                "Retained",
                LocalReviewTextFormat::Plain,
                "copied text",
            )
            .expect("initial text");
        let before_freeze = current(&mut repository);
        let (_, _, before_items, _, before_payload, _) = repository
            .local_review_snapshot(Some(&collection))
            .expect("before freeze");

        repository
            .set_task_status(&task, TaskStatus::Completed)
            .expect("complete task");
        let frozen = current(&mut repository);
        assert_eq!(frozen.state, LocalReviewCollectionState::Frozen);
        assert!(repository
            .create_local_review_text_item(
                &collection,
                frozen.updated_at_ms,
                "Blocked text",
                LocalReviewTextFormat::Plain,
                "blocked",
            )
            .is_err());
        assert!(repository
            .create_local_review_image_item(
                &collection,
                frozen.updated_at_ms,
                "Blocked image",
                &png(1, 1, false),
            )
            .is_err());
        assert!(repository
            .create_local_review_manual_evidence_item(
                &collection,
                frozen.updated_at_ms,
                "Blocked evidence",
                "blocked",
            )
            .is_err());
        let (_, frozen_selected, frozen_items, _, frozen_payload, _) = repository
            .local_review_snapshot(Some(&collection))
            .expect("frozen snapshot");
        assert_eq!(frozen_items, before_items);
        assert_eq!(frozen_payload, before_payload);
        assert_eq!(
            frozen_selected.expect("frozen selected").updated_at_ms,
            frozen.updated_at_ms
        );

        repository
            .set_task_status(&task, TaskStatus::Active)
            .expect("restore task");
        let restored = current(&mut repository);
        assert_eq!(restored.state, LocalReviewCollectionState::Frozen);
        assert!(repository
            .create_local_review_text_item(
                &collection,
                restored.updated_at_ms,
                "No automatic resume",
                LocalReviewTextFormat::Plain,
                "blocked",
            )
            .is_err());
        repository
            .resume_local_review_collection(&collection, restored.updated_at_ms)
            .expect("explicit resume");
        let resumed = current(&mut repository);
        assert_eq!(resumed.state, LocalReviewCollectionState::Active);

        repository.archive_task(&task, false).expect("archive task");
        let archived = current(&mut repository);
        assert_eq!(archived.state, LocalReviewCollectionState::Frozen);
        assert!(repository
            .create_local_review_text_item(
                &collection,
                archived.updated_at_ms,
                "Archived text",
                LocalReviewTextFormat::Plain,
                "blocked",
            )
            .is_err());
        repository
            .archive_task(&task, true)
            .expect("restore archive");
        repository
            .resume_local_review_collection(&collection, archived.updated_at_ms)
            .expect("resume after archive restore");
        let fresh = current(&mut repository);
        repository
            .connection
            .execute(
                "UPDATE task_plans SET updated_at_ms = updated_at_ms + 1 WHERE id = ?1",
                [&plan],
            )
            .expect("make observed plan stale");
        assert!(repository
            .create_local_review_manual_evidence_item(
                &collection,
                fresh.updated_at_ms,
                "Stale plan evidence",
                "blocked",
            )
            .is_err());
        let (_, _, after_stale_items, _, after_stale_payload, _) = repository
            .local_review_snapshot(Some(&collection))
            .expect("after stale plan");
        assert_eq!(after_stale_items.len(), before_items.len());
        assert_eq!(after_stale_payload, before_payload);

        let recovery_task = repository.create_task().expect("recovery task");
        let recovery_collection = repository
            .create_local_review_collection(&recovery_task, None, "Recovery")
            .expect("recovery collection");
        let recovery_selected = repository
            .local_review_snapshot(Some(&recovery_collection))
            .expect("recovery snapshot")
            .1
            .expect("recovery selected");
        let recovery_item = repository
            .create_local_review_text_item(
                &recovery_collection,
                recovery_selected.updated_at_ms,
                "Discardable",
                LocalReviewTextFormat::Plain,
                "copied text",
            )
            .expect("recovery item");
        let orphaned_at = repository
            .local_review_snapshot(Some(&recovery_collection))
            .expect("recovery item snapshot")
            .1
            .expect("recovery selected")
            .updated_at_ms;
        repository.delete_task(&recovery_task).expect("delete task");
        assert_eq!(
            repository
                .local_review_snapshot(Some(&recovery_collection))
                .expect("orphaned snapshot")
                .1
                .expect("orphaned selected")
                .state,
            LocalReviewCollectionState::Orphaned
        );
        assert!(matches!(
            repository.create_local_review_text_item(
                &recovery_collection,
                orphaned_at,
                "Orphaned mutation",
                LocalReviewTextFormat::Plain,
                "blocked",
            ),
            Err(StorageError::TaskNotFound)
        ));
        repository
            .discard_local_review_item(&recovery_collection, &recovery_item, orphaned_at)
            .expect("orphaned copied data remains explicitly discardable");
        assert!(repository
            .local_review_snapshot(Some(&recovery_collection))
            .expect("after recovery discard")
            .2
            .is_empty());
        assert!(before_freeze.updated_at_ms < frozen.updated_at_ms);
        assert_ne!(retained_item, recovery_item);
    }

    #[test]
    fn local_review_discards_are_isolated_accounted_and_stale_safe() {
        let mut repository = ProjectRepository::in_memory().expect("schema migrates");
        let task = repository.create_task().expect("task creates");
        let (_, selected, _, _, _, _) = repository
            .task_catalog(Some(&task), false, None)
            .expect("catalog");
        let plan = selected.expect("selected").selected_plan_id;
        let first = repository
            .create_local_review_collection(&task, Some(&plan), "First")
            .expect("first");
        let second = repository
            .create_local_review_collection(&task, None, "Second")
            .expect("second");
        let one = repository
            .local_review_snapshot(Some(&first))
            .expect("first snapshot")
            .1
            .expect("first selected");
        let item = repository
            .create_local_review_text_item(
                &first,
                one.updated_at_ms,
                "Item",
                LocalReviewTextFormat::Plain,
                "payload",
            )
            .expect("item");
        let (_, selected, items, _, total_before, _) = repository
            .local_review_snapshot(Some(&first))
            .expect("items");
        let selected = selected.expect("selected");
        assert!(matches!(
            repository.discard_local_review_item(&first, &item, selected.updated_at_ms - 1),
            Err(StorageError::InvalidStatusTransition)
        ));
        repository
            .discard_local_review_item(&first, &item, selected.updated_at_ms)
            .expect("discard item");
        let (_, selected, items_after, _, total_after, _) = repository
            .local_review_snapshot(Some(&first))
            .expect("after item");
        assert_eq!(items.len(), 1);
        assert!(items_after.is_empty());
        assert!(total_after < total_before);
        let updated = selected.expect("selected").updated_at_ms;
        assert!(matches!(
            repository.discard_local_review_collection(&first, updated - 1),
            Err(StorageError::InvalidStatusTransition)
        ));
        repository
            .discard_local_review_collection(&first, updated)
            .expect("discard collection");
        assert!(repository
            .local_review_snapshot(Some(&first))
            .expect("snapshot")
            .1
            .is_none());
        assert!(repository
            .local_review_snapshot(Some(&second))
            .expect("other")
            .1
            .is_some());
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT count(*) FROM task_records WHERE id = ?1",
                    [&task],
                    |row| row.get::<_, i64>(0)
                )
                .expect("task"),
            1
        );
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT count(*) FROM task_plans WHERE id = ?1",
                    [&plan],
                    |row| row.get::<_, i64>(0)
                )
                .expect("plan"),
            1
        );
    }

    #[test]
    fn local_review_images_validate_project_preview_and_recover_quota() {
        let mut repository = ProjectRepository::in_memory().expect("schema migrates");
        let task = repository.create_task().expect("task");
        let collection = repository
            .create_local_review_collection(&task, None, "Images")
            .expect("collection");
        let current = |repository: &mut ProjectRepository| {
            repository
                .local_review_snapshot(Some(&collection))
                .expect("snapshot")
                .1
                .expect("selected")
        };
        let png_bytes = png(1, 1, false);
        let first = current(&mut repository);
        let png_id = repository
            .create_local_review_image_item(&collection, first.updated_at_ms, "PNG", &png_bytes)
            .expect("png");
        let (_, selected, items, _, total_before, _) = repository
            .local_review_snapshot(Some(&collection))
            .expect("items");
        assert_eq!(items[0].class, LocalReviewItemClass::ImageMockup);
        assert_eq!(items[0].mime_type, "image/png");
        assert_eq!(items[0].width, Some(1));
        assert_eq!(items[0].height, Some(1));
        assert_eq!(
            items[0].source_kind,
            LocalReviewSourceKind::NativeImageInput
        );
        let preview = repository
            .local_review_image_preview(&png_id, &items[0].sha256)
            .expect("preview");
        assert!(preview.data_url.starts_with("data:image/png;base64,"));
        assert_eq!(preview.byte_size, png_bytes.len() as u64);
        let second = selected.expect("selected");
        let jpeg_bytes = jpeg(2, 3, true);
        let jpeg_id = repository
            .create_local_review_image_item(&collection, second.updated_at_ms, "JPEG", &jpeg_bytes)
            .expect("jpeg");
        let (_, selected, items, _, _, warning) = repository
            .local_review_snapshot(Some(&collection))
            .expect("two");
        assert!(warning || items.len() == 2);
        assert_eq!(
            items
                .iter()
                .find(|item| item.item_id == jpeg_id)
                .expect("jpeg item")
                .mime_type,
            "image/jpeg"
        );
        let updated = selected.expect("selected");
        let third = repository
            .create_local_review_image_item(
                &collection,
                updated.updated_at_ms,
                "Duplicate",
                &png_bytes,
            )
            .expect("third");
        let after_third = current(&mut repository);
        assert!(matches!(
            repository.create_local_review_image_item(
                &collection,
                after_third.updated_at_ms,
                "Fourth",
                &png_bytes
            ),
            Err(StorageError::TaskCapacity)
        ));
        assert!(matches!(
            repository.create_local_review_image_item(
                &collection,
                after_third.updated_at_ms - 1,
                "Stale",
                &png_bytes
            ),
            Err(StorageError::InvalidStatusTransition)
        ));
        repository
            .discard_local_review_item(&collection, &third, after_third.updated_at_ms)
            .expect("discard");
        let (_, selected, items, _, total_after, _) = repository
            .local_review_snapshot(Some(&collection))
            .expect("after");
        assert_eq!(items.len(), 2);
        assert!(total_after < total_before + png_bytes.len() as u64 + 512);
        let current = selected.expect("selected");
        for invalid in [
            vec![0],
            png(0, 1, false),
            png(1, 0, false),
            png(1, 1, true),
            jpeg(1, 1, false),
            jpeg(0, 1, true),
            jpeg(1, 0, true),
        ] {
            assert!(repository
                .create_local_review_image_item(
                    &collection,
                    current.updated_at_ms,
                    "Invalid",
                    &invalid
                )
                .is_err());
        }
        assert_eq!(
            repository
                .local_review_snapshot(Some(&collection))
                .expect("unchanged")
                .2
                .len(),
            2
        );
    }

    #[test]
    fn local_review_image_preview_withholds_corrupt_or_mismatched_rows() {
        let mut repository = ProjectRepository::in_memory().expect("schema migrates");
        let task = repository.create_task().expect("task");
        let collection = repository
            .create_local_review_collection(&task, None, "Image")
            .expect("collection");
        let selected = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot")
            .1
            .expect("selected");
        let id = repository
            .create_local_review_image_item(
                &collection,
                selected.updated_at_ms,
                "PNG",
                &png(1, 1, false),
            )
            .expect("image");
        let item = repository
            .local_review_snapshot(Some(&collection))
            .expect("items")
            .2
            .remove(0);
        assert!(repository
            .local_review_image_preview(&id, &"0".repeat(64))
            .is_err());
        repository
            .connection
            .execute(
                "UPDATE local_review_items SET byte_size = byte_size + 1 WHERE id = ?1",
                [&id],
            )
            .expect("corrupt");
        assert!(repository
            .local_review_image_preview(&id, &item.sha256)
            .is_err());
    }

    #[test]
    fn local_review_text_preview_is_canonical_bounded_and_read_only() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let task = repository.create_task().expect("task");
        let collection = repository
            .create_local_review_collection(&task, None, "Text previews")
            .expect("collection");
        let formats = [
            (LocalReviewTextFormat::Plain, "plain\r\ntext"),
            (LocalReviewTextFormat::Markdown, "# Markdown"),
            (LocalReviewTextFormat::Json, "{\"value\":1}"),
            (LocalReviewTextFormat::Csv, "left,right\n1,2"),
            (LocalReviewTextFormat::Python, "print('safe')"),
        ];
        for (index, (format, content)) in formats.into_iter().enumerate() {
            let selected = repository
                .local_review_snapshot(Some(&collection))
                .expect("snapshot")
                .1
                .expect("collection");
            let item = repository
                .create_local_review_text_item(
                    &collection,
                    selected.updated_at_ms,
                    &format!("Text {index}"),
                    format,
                    content,
                )
                .expect("text item");
            let snapshot_before = repository
                .local_review_snapshot(Some(&collection))
                .expect("before preview");
            let summary = snapshot_before
                .2
                .iter()
                .find(|value| value.item_id == item)
                .expect("item");
            let preview = repository
                .local_review_text_preview(&collection, &item, &summary.sha256)
                .expect("preview");
            assert_eq!(preview.state, LocalReviewItemState::Ready);
            assert_eq!(preview.text_format, Some(format));
            assert_eq!(
                preview.text.as_deref(),
                Some(content.replace("\r\n", "\n").as_str())
            );
            assert!(!preview.truncated);
            assert_eq!(
                repository
                    .local_review_snapshot(Some(&collection))
                    .expect("after preview"),
                snapshot_before
            );
        }

        let selected = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot")
            .1
            .expect("collection");
        let long = (0..2_001)
            .map(|index| format!("line {index}\n"))
            .collect::<String>();
        let item = repository
            .create_local_review_text_item(
                &collection,
                selected.updated_at_ms,
                "Long",
                LocalReviewTextFormat::Plain,
                &long,
            )
            .expect("long text");
        let sha: String = repository
            .connection
            .query_row(
                "SELECT sha256 FROM local_review_items WHERE id = ?1",
                [&item],
                |row| row.get(0),
            )
            .expect("sha");
        let preview = repository
            .local_review_text_preview(&collection, &item, &sha)
            .expect("bounded preview");
        assert!(preview.truncated);
        assert_eq!(preview.projected_line_count, 2_000);
        assert!(preview.projected_byte_size <= 128 * 1024);
        assert!(!preview.text.expect("text").contains('\r'));
    }

    #[test]
    fn local_review_text_preview_withholds_mismatched_corrupt_and_recovery_rows() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let task = repository.create_task().expect("task");
        let collection = repository
            .create_local_review_collection(&task, None, "Text previews")
            .expect("collection");
        let updated = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot")
            .1
            .expect("collection")
            .updated_at_ms;
        let item = repository
            .create_local_review_text_item(
                &collection,
                updated,
                "Text",
                LocalReviewTextFormat::Plain,
                "safe text",
            )
            .expect("item");
        let sha: String = repository
            .connection
            .query_row(
                "SELECT sha256 FROM local_review_items WHERE id = ?1",
                [&item],
                |row| row.get(0),
            )
            .expect("sha");
        assert_eq!(
            repository
                .local_review_text_preview(&collection, &item, &"0".repeat(64))
                .expect("mismatch")
                .text,
            None
        );
        repository
            .connection
            .execute(
                "UPDATE local_review_items SET content = x'00' WHERE id = ?1",
                [&item],
            )
            .expect("corrupt row");
        assert_eq!(
            repository
                .local_review_text_preview(&collection, &item, &sha)
                .expect("corrupt")
                .state,
            LocalReviewItemState::Unavailable
        );
        repository
            .connection
            .execute(
                "UPDATE local_review_items SET content = ?1 WHERE id = ?2",
                params![b"safe text".to_vec(), item],
            )
            .expect("restore fixture");
        repository
            .connection
            .execute(
                "UPDATE local_review_items SET sha256 = ?1 WHERE id = ?2",
                params![review_digest(b"safe text"), item],
            )
            .expect("restore digest");
        repository
            .connection
            .execute(
                "UPDATE local_review_items SET state = 'stale' WHERE id = ?1",
                [&item],
            )
            .expect("stale fixture");
        let stale = repository
            .local_review_text_preview(&collection, &item, &review_digest(b"safe text"))
            .expect("stale preview");
        assert_eq!(stale.state, LocalReviewItemState::Stale);
        assert_eq!(stale.text, None);
        repository
            .connection
            .execute(
                "UPDATE local_review_items SET state = 'ready' WHERE id = ?1",
                [&item],
            )
            .expect("ready fixture");
        repository
            .set_task_status(&task, TaskStatus::Completed)
            .expect("freeze");
        assert_eq!(
            repository
                .local_review_text_preview(&collection, &item, &review_digest(b"safe text"))
                .expect("frozen recovery preview")
                .text
                .as_deref(),
            Some("safe text")
        );
        repository.delete_task(&task).expect("orphan");
        assert_eq!(
            repository
                .local_review_text_preview(&collection, &item, &review_digest(b"safe text"))
                .expect("orphan recovery preview")
                .text
                .as_deref(),
            Some("safe text")
        );
        assert_eq!(
            repository
                .local_review_text_preview(
                    &Uuid::now_v7().to_string(),
                    &item,
                    &review_digest(b"safe text")
                )
                .expect("collection mismatch")
                .state,
            LocalReviewItemState::Unavailable
        );
    }

    #[test]
    fn local_review_manual_evidence_is_canonical_digest_bound_and_accounted() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let task = repository.create_task().expect("task");
        let collection = repository
            .create_local_review_collection(&task, None, "Evidence")
            .expect("collection");
        let before = repository
            .local_review_snapshot(Some(&collection))
            .expect("before");
        let selected = before.1.expect("selected");
        let id = repository
            .create_local_review_manual_evidence_item(
                &collection,
                selected.updated_at_ms,
                "Validation",
                "line one\r\nline two",
            )
            .expect("evidence");
        let (content, digest, byte_size, class, source): (Vec<u8>, String, i64, String, String) = repository.connection.query_row("SELECT content, sha256, byte_size, class, provenance FROM local_review_items WHERE id = ?1", [&id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))).expect("row");
        assert_eq!(class, "evidence");
        assert_eq!(source, "manual-validation-summary");
        assert_eq!(digest, review_digest(&content));
        assert_eq!(byte_size, content.len() as i64);
        let canonical = String::from_utf8(content).expect("utf8");
        assert!(canonical.contains("line one\\nline two"));
        assert!(!canonical.contains("\r"));
        assert!(!canonical.contains("path"));
        assert!(!canonical.contains("://"));
        assert!(
            review_payload_bytes(&repository.connection, None).expect("payload") > before.4 as i64
        );
        assert!(matches!(
            repository.create_local_review_manual_evidence_item(
                &collection,
                selected.updated_at_ms,
                "Stale",
                "summary"
            ),
            Err(StorageError::InvalidStatusTransition)
        ));
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT count(*) FROM local_review_items WHERE collection_id = ?1",
                    [&collection],
                    |row| row.get::<_, i64>(0)
                )
                .expect("count"),
            1
        );
    }

    #[test]
    fn local_review_manual_evidence_enforces_count_and_size_quotas() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let task = repository.create_task().expect("task");
        let collection = repository
            .create_local_review_collection(&task, None, "Evidence")
            .expect("collection");
        for index in 0..6 {
            let updated: i64 = repository
                .connection
                .query_row(
                    "SELECT updated_at_ms FROM local_review_collections WHERE id = ?1",
                    [&collection],
                    |row| row.get(0),
                )
                .expect("timestamp");
            repository
                .create_local_review_manual_evidence_item(
                    &collection,
                    updated,
                    &format!("Evidence {index}"),
                    "summary",
                )
                .expect("evidence");
        }
        let timestamp: i64 = repository
            .connection
            .query_row(
                "SELECT updated_at_ms FROM local_review_collections WHERE id = ?1",
                [&collection],
                |row| row.get(0),
            )
            .expect("timestamp");
        let count: i64 = repository.connection.query_row("SELECT count(*) FROM local_review_items WHERE collection_id = ?1 AND class = 'evidence'", [&collection], |row| row.get(0)).expect("count");
        assert_eq!(count, 6);
        assert!(matches!(
            repository.create_local_review_manual_evidence_item(
                &collection,
                timestamp,
                "Seventh",
                "summary"
            ),
            Err(StorageError::TaskCapacity)
        ));
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT count(*) FROM local_review_items WHERE collection_id = ?1",
                    [&collection],
                    |row| row.get::<_, i64>(0)
                )
                .expect("unchanged"),
            6
        );
        let other = repository
            .create_local_review_collection(&task, None, "Size")
            .expect("size collection");
        let selected: i64 = repository
            .connection
            .query_row(
                "SELECT updated_at_ms FROM local_review_collections WHERE id = ?1",
                [&other],
                |row| row.get(0),
            )
            .expect("timestamp");
        assert!(repository
            .create_local_review_manual_evidence_item(
                &other,
                selected,
                "Too large",
                &"x".repeat(16 * 1024)
            )
            .is_err());
    }

    #[test]
    fn local_review_manual_evidence_discard_recovers_quota_and_payload() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let task = repository.create_task().expect("task");
        let collection = repository
            .create_local_review_collection(&task, None, "Evidence")
            .expect("collection");
        let mut ids = Vec::new();
        for index in 0..6 {
            let selected: i64 = repository
                .connection
                .query_row(
                    "SELECT updated_at_ms FROM local_review_collections WHERE id = ?1",
                    [&collection],
                    |row| row.get(0),
                )
                .expect("timestamp");
            ids.push(
                repository
                    .create_local_review_manual_evidence_item(
                        &collection,
                        selected,
                        &format!("Evidence {index}"),
                        "summary",
                    )
                    .expect("evidence"),
            );
        }
        let before = review_payload_bytes(&repository.connection, None).expect("payload");
        let updated: i64 = repository
            .connection
            .query_row(
                "SELECT updated_at_ms FROM local_review_collections WHERE id = ?1",
                [&collection],
                |row| row.get(0),
            )
            .expect("updated");
        repository
            .discard_local_review_item(&collection, &ids[0], updated)
            .expect("discard");
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT count(*) FROM local_review_items WHERE collection_id = ?1",
                    [&collection],
                    |row| row.get::<_, i64>(0)
                )
                .expect("count"),
            5
        );
        assert!(review_payload_bytes(&repository.connection, None).expect("payload") < before);
        let refreshed: i64 = repository
            .connection
            .query_row(
                "SELECT updated_at_ms FROM local_review_collections WHERE id = ?1",
                [&collection],
                |row| row.get(0),
            )
            .expect("updated");
        repository
            .create_local_review_manual_evidence_item(
                &collection,
                refreshed,
                "Replacement",
                "summary",
            )
            .expect("replacement");
    }

    #[test]
    fn local_review_manual_evidence_projects_warning_thresholds() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let task = repository.create_task().expect("task");
        let collection = repository
            .create_local_review_collection(&task, None, "Evidence")
            .expect("collection");
        for index in 0..4 {
            let updated: i64 = repository
                .connection
                .query_row(
                    "SELECT updated_at_ms FROM local_review_collections WHERE id = ?1",
                    [&collection],
                    |row| row.get(0),
                )
                .expect("updated");
            repository
                .create_local_review_manual_evidence_item(
                    &collection,
                    updated,
                    &format!("E{index}"),
                    "summary",
                )
                .expect("evidence");
        }
        assert!(!repository.local_review_snapshot(None).expect("snapshot").0[0].warning);
        let updated: i64 = repository
            .connection
            .query_row(
                "SELECT updated_at_ms FROM local_review_collections WHERE id = ?1",
                [&collection],
                |row| row.get(0),
            )
            .expect("updated");
        repository
            .create_local_review_manual_evidence_item(&collection, updated, "E5", "summary")
            .expect("fifth");
        assert!(repository.local_review_snapshot(None).expect("snapshot").0[0].warning);
    }

    #[test]
    fn local_review_manual_evidence_preview_is_canonical_and_path_free() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let task = repository.create_task().expect("task");
        let collection = repository
            .create_local_review_collection(&task, None, "Evidence")
            .expect("collection");
        let updated: i64 = repository
            .connection
            .query_row(
                "SELECT updated_at_ms FROM local_review_collections WHERE id = ?1",
                [&collection],
                |row| row.get(0),
            )
            .expect("updated");
        let id = repository
            .create_local_review_manual_evidence_item(
                &collection,
                updated,
                "Validation",
                "line\r\nsummary",
            )
            .expect("evidence");
        let sha: String = repository
            .connection
            .query_row(
                "SELECT sha256 FROM local_review_items WHERE id = ?1",
                [&id],
                |row| row.get(0),
            )
            .expect("sha");
        let preview = repository
            .local_review_manual_evidence_preview(&id, &sha)
            .expect("preview");
        assert_eq!(preview.summary, "line\nsummary");
        assert_eq!(preview.source, "manual-validation-summary");
        assert!(!format!("{preview:?}").contains("path"));
        repository
            .connection
            .execute(
                "UPDATE local_review_items SET content = x'7B' WHERE id = ?1",
                [&id],
            )
            .expect("corrupt");
        assert!(repository
            .local_review_manual_evidence_preview(&id, &sha)
            .is_err());
    }

    #[test]
    fn failed_task_deletion_rolls_back_task_and_all_plans() {
        let mut repository = ProjectRepository::in_memory().expect("schema must migrate");
        let task = repository.create_task().expect("task must create");
        repository.create_plan(&task, false).expect("alternate");
        repository
            .connection
            .execute_batch(
                "CREATE TRIGGER reject_task_plan_delete
                 BEFORE DELETE ON task_plans
                 BEGIN
                   SELECT RAISE(ABORT, 'injected-delete-failure');
                 END;",
            )
            .expect("failure trigger");

        assert!(matches!(
            repository.delete_task(&task),
            Err(StorageError::Sqlite(_))
        ));
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT count(*) FROM task_records WHERE id = ?1",
                    [&task],
                    |row| row.get::<_, i64>(0),
                )
                .expect("task count"),
            1
        );
        assert_eq!(
            repository
                .connection
                .query_row(
                    "SELECT count(*) FROM task_plans WHERE task_id = ?1",
                    [&task],
                    |row| row.get::<_, i64>(0),
                )
                .expect("plan count"),
            2
        );

        repository
            .connection
            .execute_batch("DROP TRIGGER reject_task_plan_delete;")
            .expect("failure trigger cleanup");
        repository
            .delete_task(&task)
            .expect("recovered deletion must commit");
    }

    #[test]
    fn local_review_text_comparison_enforces_quota_and_side_limits() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let task = repository.create_task().expect("task");
        let plan = repository
            .task_catalog(Some(&task), false, None)
            .expect("catalog")
            .1
            .expect("task")
            .selected_plan_id;
        let collection = repository
            .create_local_review_collection(&task, Some(&plan), "Comparison quotas")
            .expect("collection");
        let timestamp = |repository: &ProjectRepository| {
            repository
                .connection
                .query_row(
                    "SELECT updated_at_ms FROM local_review_collections WHERE id = ?1",
                    [&collection],
                    |row| row.get::<_, i64>(0),
                )
                .expect("timestamp")
        };
        let left = repository
            .create_local_review_text_item(
                &collection,
                timestamp(&repository),
                "Left",
                LocalReviewTextFormat::Plain,
                "left",
            )
            .expect("left");
        let right = repository
            .create_local_review_text_item(
                &collection,
                timestamp(&repository),
                "Right",
                LocalReviewTextFormat::Plain,
                "right",
            )
            .expect("right");
        for index in 0..8 {
            repository
                .create_local_review_text_comparison(
                    &collection,
                    &left,
                    &right,
                    timestamp(&repository),
                )
                .unwrap_or_else(|error| panic!("comparison {index}: {error:?}"));
            let warning = repository
                .local_review_snapshot(Some(&collection))
                .expect("snapshot")
                .1
                .expect("selected")
                .comparison_count_warning;
            assert_eq!(warning, index >= 5);
        }
        let before = timestamp(&repository);
        assert!(matches!(
            repository.create_local_review_text_comparison(&collection, &left, &right, before),
            Err(StorageError::TaskCapacity)
        ));
        assert_eq!(timestamp(&repository), before);
        assert_eq!(
            repository
                .local_review_comparisons(&collection)
                .expect("bindings")
                .len(),
            8
        );

        let limits_collection = repository
            .create_local_review_collection(&task, Some(&plan), "Comparison side limits")
            .expect("limits collection");
        let limits_timestamp = |repository: &ProjectRepository| {
            repository
                .connection
                .query_row(
                    "SELECT updated_at_ms FROM local_review_collections WHERE id = ?1",
                    [&limits_collection],
                    |row| row.get::<_, i64>(0),
                )
                .expect("limits timestamp")
        };
        let limits_left = repository
            .create_local_review_text_item(
                &limits_collection,
                limits_timestamp(&repository),
                "Limits left",
                LocalReviewTextFormat::Plain,
                "left",
            )
            .expect("limits left");
        let oversized = "x".repeat(128 * 1024 + 1);
        let oversized_item = Uuid::now_v7().to_string();
        let oversized_now = 1_i64;
        repository
            .connection
            .execute(
                "INSERT INTO local_review_items (id, schema_version, collection_id, class, text_format, mime_type, width, height, state, title, source_kind, provenance, content, sha256, byte_size, created_at_ms, updated_at_ms) VALUES (?1, 1, ?2, 'text', 'plain', 'text/plain; charset=utf-8', NULL, NULL, 'ready', 'Oversized', 'user-authored-text', '', ?3, ?4, ?5, ?6, ?6)",
                params![oversized_item, limits_collection, oversized.as_bytes(), review_digest(oversized.as_bytes()), oversized.len() as i64, oversized_now],
            )
            .expect("oversized fixture");
        let before = limits_timestamp(&repository);
        assert!(matches!(
            repository.create_local_review_text_comparison(
                &limits_collection,
                &limits_left,
                &oversized_item,
                before
            ),
            Err(StorageError::InvalidStoredValue)
        ));
        assert_eq!(limits_timestamp(&repository), before);
        let lines = "line\n".repeat(2_001);
        let line_item = repository
            .create_local_review_text_item(
                &limits_collection,
                limits_timestamp(&repository),
                "Too many lines",
                LocalReviewTextFormat::Plain,
                &lines,
            )
            .expect("line text remains valid review text");
        let before = limits_timestamp(&repository);
        assert!(matches!(
            repository.create_local_review_text_comparison(
                &limits_collection,
                &limits_left,
                &line_item,
                before
            ),
            Err(StorageError::InvalidStoredValue)
        ));
        assert_eq!(limits_timestamp(&repository), before);
    }

    #[test]
    fn local_review_text_comparison_discard_isolated_and_recovers_capacity() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let task = repository.create_task().expect("task");
        let plan = repository
            .task_catalog(Some(&task), false, None)
            .expect("catalog")
            .1
            .expect("task")
            .selected_plan_id;
        let collection = repository
            .create_local_review_collection(&task, Some(&plan), "Comparison discard")
            .expect("collection");
        let timestamp = |repository: &ProjectRepository| {
            repository
                .connection
                .query_row(
                    "SELECT updated_at_ms FROM local_review_collections WHERE id = ?1",
                    [&collection],
                    |row| row.get::<_, i64>(0),
                )
                .expect("timestamp")
        };
        let left = repository
            .create_local_review_text_item(
                &collection,
                timestamp(&repository),
                "Left",
                LocalReviewTextFormat::Plain,
                "left",
            )
            .expect("left");
        let right = repository
            .create_local_review_text_item(
                &collection,
                timestamp(&repository),
                "Right",
                LocalReviewTextFormat::Plain,
                "right",
            )
            .expect("right");
        let source_before = repository
            .local_review_snapshot(Some(&collection))
            .expect("snapshot")
            .2;
        let mut bindings = Vec::new();
        for _ in 0..8 {
            bindings.push(
                repository
                    .create_local_review_text_comparison(
                        &collection,
                        &left,
                        &right,
                        timestamp(&repository),
                    )
                    .expect("binding"),
            );
        }
        let stale = timestamp(&repository) - 1;
        assert!(matches!(
            repository.discard_local_review_text_comparison(&collection, &bindings[0], stale),
            Err(StorageError::InvalidStatusTransition)
        ));
        let before = timestamp(&repository);
        repository
            .discard_local_review_text_comparison(&collection, &bindings[0], before)
            .expect("discard");
        assert_eq!(
            repository
                .local_review_comparisons(&collection)
                .expect("bindings")
                .len(),
            7
        );
        assert!(repository
            .local_review_comparisons(&collection)
            .expect("bindings")
            .iter()
            .all(|binding| binding.comparison_id != bindings[0]));
        assert_eq!(
            repository
                .local_review_snapshot(Some(&collection))
                .expect("snapshot")
                .2,
            source_before
        );
        repository
            .create_local_review_text_comparison(&collection, &left, &right, timestamp(&repository))
            .expect("replacement");
        let before_missing = timestamp(&repository);
        assert!(matches!(
            repository.discard_local_review_text_comparison(
                &collection,
                &Uuid::now_v7().to_string(),
                before_missing
            ),
            Err(StorageError::TaskNotFound)
        ));
        assert_eq!(timestamp(&repository), before_missing);
    }

    #[test]
    fn local_review_text_comparison_read_withholds_stale_or_corrupt_sides() {
        let mut repository = ProjectRepository::in_memory().expect("schema");
        let task = repository.create_task().expect("task");
        let plan = repository
            .task_catalog(Some(&task), false, None)
            .expect("catalog")
            .1
            .expect("task")
            .selected_plan_id;
        let collection = repository
            .create_local_review_collection(&task, Some(&plan), "Comparison integrity")
            .expect("collection");
        let timestamp = |repository: &ProjectRepository| {
            repository
                .connection
                .query_row(
                    "SELECT updated_at_ms FROM local_review_collections WHERE id = ?1",
                    [&collection],
                    |row| row.get::<_, i64>(0),
                )
                .expect("timestamp")
        };
        let left = repository
            .create_local_review_text_item(
                &collection,
                timestamp(&repository),
                "Left",
                LocalReviewTextFormat::Plain,
                "left",
            )
            .expect("left");
        let right = repository
            .create_local_review_text_item(
                &collection,
                timestamp(&repository),
                "Right",
                LocalReviewTextFormat::Plain,
                "right",
            )
            .expect("right");
        let comparison = repository
            .create_local_review_text_comparison(&collection, &left, &right, timestamp(&repository))
            .expect("binding");
        repository
            .connection
            .execute(
                "UPDATE local_review_items SET content = ?1, sha256 = ?2, byte_size = ?3 WHERE id = ?4",
                params![b"changed".to_vec(), review_digest(b"changed"), 7_i64, left],
            )
            .expect("changed fixture");
        assert_eq!(
            repository
                .local_review_comparisons(&collection)
                .expect("bindings")[0]
                .state,
            LocalReviewComparisonState::Stale
        );
        let stale = repository
            .local_review_line_comparison(&comparison)
            .expect("stale projection");
        assert_eq!(stale.state, LocalReviewComparisonState::Stale);
        assert!(stale.lines.is_empty());
        repository
            .connection
            .execute(
                "UPDATE local_review_items SET byte_size = byte_size + 1 WHERE id = ?1",
                [&right],
            )
            .expect("corrupt fixture");
        assert_eq!(
            repository
                .local_review_comparisons(&collection)
                .expect("bindings")[0]
                .state,
            LocalReviewComparisonState::Unavailable
        );
        assert_eq!(
            repository
                .local_review_line_comparison(&comparison)
                .expect("unavailable projection")
                .state,
            LocalReviewComparisonState::Unavailable
        );
        let columns: Vec<String> = repository
            .connection
            .prepare("PRAGMA table_info(local_review_comparisons)")
            .expect("pragma")
            .query_map([], |row| row.get(1))
            .expect("columns")
            .collect::<Result<_, _>>()
            .expect("columns");
        assert!(!columns.iter().any(|column| column.contains("result")));
        let projected = serde_json::to_value(
            repository
                .local_review_comparisons(&collection)
                .expect("bindings"),
        )
        .expect("serialize");
        for forbidden in ["path", "url", "git", "repository", "shell", "command"] {
            assert!(!projected.to_string().contains(forbidden));
        }
    }

    #[test]
    fn local_template_storage_inserts_fetches_and_lists_deterministically() {
        let mut repository = ProjectRepository::in_memory().expect("schema migrates");
        let first = local_template(Uuid::now_v7().to_string(), "First");
        let second = local_template(Uuid::now_v7().to_string(), "Second");
        repository
            .insert_local_template(&first)
            .expect("first template inserts");
        repository
            .insert_local_template(&second)
            .expect("second template inserts");
        repository
            .connection
            .execute(
                "UPDATE local_task_templates SET updated_at_ms = 9999999999999 WHERE id = ?1",
                [&first.id],
            )
            .expect("first timestamp fixture");
        repository
            .connection
            .execute(
                "UPDATE local_task_templates SET updated_at_ms = 9999999999999 WHERE id = ?1",
                [&second.id],
            )
            .expect("second timestamp fixture");

        assert_eq!(
            repository
                .local_template(&first.id)
                .expect("fetch succeeds")
                .expect("first exists")
                .template,
            first
        );
        let listed = repository.local_templates().expect("list succeeds");
        let mut expected = vec![first.id.clone(), second.id.clone()];
        expected.sort();
        assert_eq!(
            listed
                .iter()
                .map(|record| record.template.id.clone())
                .collect::<Vec<_>>(),
            expected
        );
    }

    #[test]
    fn local_template_storage_round_trips_canonical_fields_and_capacity() {
        let mut repository = ProjectRepository::in_memory().expect("schema migrates");
        let template = local_template(Uuid::now_v7().to_string(), "Round trip");
        let inserted = repository
            .insert_local_template(&template)
            .expect("template inserts");
        let fetched = repository
            .local_template(&template.id)
            .expect("fetch succeeds")
            .expect("template exists");
        assert_eq!(fetched, inserted);
        assert_eq!(
            canonical_template(&fetched.template),
            canonical_template(&template)
        );
        assert_eq!(fetched.template.sha256, template.sha256);
        assert_eq!(
            repository
                .local_template_capacity()
                .expect("capacity succeeds"),
            super::LocalTemplateCapacity {
                record_count: 1,
                canonical_bytes: canonical_template(&template).expect("canonical").len(),
            }
        );
    }

    #[test]
    fn local_template_storage_replaces_once_and_rejects_stale_authority() {
        let mut repository = ProjectRepository::in_memory().expect("schema migrates");
        let original = local_template(Uuid::now_v7().to_string(), "Original");
        repository
            .insert_local_template(&original)
            .expect("template inserts");
        let mut replacement = original.clone();
        replacement.title = "Replacement".to_owned();
        replacement.version = 2;
        replacement.sha256 = template_digest(&replacement).expect("replacement canonical");
        let stored = repository
            .replace_local_template(1, &replacement)
            .expect("current authority replaces");
        assert_eq!(stored.template, replacement);
        assert!(stored.updated_at_ms >= stored.created_at_ms);

        let mut stale = replacement.clone();
        stale.title = "Stale replacement".to_owned();
        stale.version = 2;
        stale.sha256 = template_digest(&stale).expect("stale fixture canonical");
        assert!(matches!(
            repository.replace_local_template(1, &stale),
            Err(StorageError::InvalidStatusTransition)
        ));
        assert_eq!(
            repository
                .local_template(&original.id)
                .expect("fetch succeeds")
                .expect("template remains")
                .template,
            replacement
        );
    }

    #[test]
    fn local_template_storage_deletes_only_the_requested_record_and_never_stores_builtins() {
        let mut repository = ProjectRepository::in_memory().expect("schema migrates");
        let first = local_template(Uuid::now_v7().to_string(), "Delete me");
        let second = local_template(Uuid::now_v7().to_string(), "Keep me");
        repository
            .insert_local_template(&first)
            .expect("first template inserts");
        repository
            .insert_local_template(&second)
            .expect("second template inserts");
        repository
            .delete_local_template(&first.id)
            .expect("requested template deletes");
        assert!(repository
            .local_template(&first.id)
            .expect("fetch succeeds")
            .is_none());
        assert!(repository
            .local_template(&second.id)
            .expect("fetch succeeds")
            .is_some());
        for builtin in builtins() {
            assert!(matches!(
                repository.insert_local_template(&builtin),
                Err(StorageError::InvalidStoredValue)
            ));
        }
        let builtins_in_sqlite: i64 = repository
            .connection
            .query_row(
                "SELECT COUNT(*) FROM local_task_templates WHERE id IN (?1, ?2, ?3, ?4)",
                rusqlite::params![
                    builtins()[0].id,
                    builtins()[1].id,
                    builtins()[2].id,
                    builtins()[3].id,
                ],
                |row| row.get(0),
            )
            .expect("builtin count reads");
        assert_eq!(builtins_in_sqlite, 0);
    }

    #[test]
    fn local_template_storage_fails_closed_for_corrupt_persisted_rows() {
        for (column, value) in [
            ("id", "'not-a-uuid'"),
            ("schema_version", "2"),
            ("origin", "'built-in'"),
            ("version", "0"),
            ("state", "'unknown'"),
            ("title", "' not canonical '"),
            ("updated_at_ms", "-1"),
            ("sha256", "replace(printf('%064d', 0), '0', 'f')"),
        ] {
            let mut repository = ProjectRepository::in_memory().expect("schema migrates");
            let template = local_template(Uuid::now_v7().to_string(), "Integrity");
            repository
                .insert_local_template(&template)
                .expect("template inserts");
            repository
                .connection
                .execute_batch(
                    "PRAGMA ignore_check_constraints = ON;
                     DROP TRIGGER local_task_templates_identity_immutable;
                     DROP TRIGGER local_task_templates_version_monotonic;",
                )
                .expect("test permits corrupt row");
            repository
                .connection
                .execute(
                    &format!("UPDATE local_task_templates SET {column} = {value}"),
                    [],
                )
                .expect("corrupt fixture writes");
            assert!(matches!(
                repository.local_templates(),
                Err(StorageError::InvalidStoredValue)
            ));
        }
    }

    #[test]
    fn local_template_storage_failed_mutation_rolls_back_completely() {
        let mut repository = ProjectRepository::in_memory().expect("schema migrates");
        repository
            .connection
            .execute_batch(
                "CREATE TEMP TRIGGER fail_local_template_insert
                 BEFORE INSERT ON local_task_templates
                 BEGIN SELECT RAISE(ABORT, 'forced template failure'); END;",
            )
            .expect("failure trigger installs");
        let template = local_template(Uuid::now_v7().to_string(), "Rollback");
        assert!(repository.insert_local_template(&template).is_err());
        assert_eq!(
            repository
                .local_template_capacity()
                .expect("capacity succeeds"),
            super::LocalTemplateCapacity {
                record_count: 0,
                canonical_bytes: 0,
            }
        );
    }
}
