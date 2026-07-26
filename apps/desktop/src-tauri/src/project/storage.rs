use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};
use thiserror::Error;
use uuid::Uuid;

use crate::advisor::{
    AdvisorContextKind, AdvisorContextReference, AdvisorConversationReference,
    AdvisorDispatchProposal, AdvisorDispatchState, AdvisorExecutionDispatchState,
    AdvisorFoundationSnapshot, AdvisorFreshness, AdvisorProvenance, AdvisorProvenanceSource,
    AdvisorTrust,
};

use super::{
    identity::DirectoryIdentity,
    types::{DirectoryAccessibilityState, ExpectedAccess},
    AdvisorConversationMetadata, ChatConversationMetadata, ConversationReference,
    ConversationSelectionMetadata,
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

pub(crate) struct ProjectRepository {
    connection: Connection,
}

impl ProjectRepository {
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
        Ok(Self { connection })
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
        transaction.execute(
            "INSERT INTO schema_migrations(version, name, applied_at_ms)
             VALUES (?1, ?2, ?3)",
            params![version, name, now_millis()],
        )?;
    }
    transaction.commit()?;
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

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(i64::MAX as u128) as i64
}

#[cfg(test)]
mod tests {
    use std::{fs, os::unix::fs::PermissionsExt};

    use rusqlite::{params, Connection};
    use uuid::Uuid;

    use super::{ProjectRepository, StorageError, INITIAL_MIGRATION};
    use crate::project::{
        ChatConversationMetadata, ConversationPendingSelection, ConversationReference,
        ConversationSelectionMetadata,
    };

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
                    OR (version = 8 AND name = 'advisor-reference-foundation')",
                [],
                |row| row.get(0),
            )
            .expect("migration ledger must be queryable");
        assert_eq!(migrated, 7);
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
                "conversation_references".to_owned(),
                "directory_associations".to_owned(),
                "projects".to_owned(),
                "schema_migrations".to_owned(),
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
}
