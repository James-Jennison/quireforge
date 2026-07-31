use uuid::Uuid;

use super::{
    storage::{
        ProjectRepository, StorageError, StoredLocalTaskTemplate,
        TaskTemplateApplicationReservation, TEMPLATE_APPLICATION_RESERVATION_TTL_MS,
    },
    task_template::{
        builtins, canonical, digest, normalized_single, valid_instructions, warning, TaskTemplate,
        TemplateOrigin, TemplateState, TEMPLATE_COUNT_LIMIT, TEMPLATE_PAYLOAD_LIMIT,
        TEMPLATE_SCHEMA_VERSION,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TemplateCapacityFacts {
    pub record_count: usize,
    pub canonical_bytes: usize,
    pub warning: bool,
    pub count_limit: usize,
    pub canonical_byte_limit: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TemplateCatalog {
    pub templates: Vec<TaskTemplate>,
    pub capacity: TemplateCapacityFacts,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TemplateInspection {
    pub template: TaskTemplate,
    pub schema_version: u16,
    pub created_at_ms: Option<i64>,
    pub updated_at_ms: Option<i64>,
    pub archived_at_ms: Option<i64>,
    authority: Option<TemplateMutationAuthority>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TemplateMutationAuthority {
    id: String,
    version: u32,
    digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TemplateContentInput {
    pub title: String,
    pub purpose: String,
    pub instructions: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DeletionConfirmation {
    Confirmed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TemplateLifecycleError {
    InvalidInput,
    NotFound,
    BuiltInImmutable,
    ArchivedReadOnly,
    ActiveAlready,
    ArchivedAlready,
    Stale,
    Capacity,
    Unavailable,
}

pub(crate) struct TemplateLifecycleService {
    repository: ProjectRepository,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TemplateApplicationPreview {
    pub reservation_id: String,
    pub expires_at_ms: i64,
    pub template: TaskTemplate,
    pub binding_sha256: String,
}

impl TemplateLifecycleService {
    pub(crate) fn preview_application(
        &mut self,
        template_id: &str,
        task_id: &str,
        plan_id: &str,
        title: &str,
        plan_text: &str,
    ) -> Result<TemplateApplicationPreview, TemplateLifecycleError> {
        let template = self.inspect(template_id)?.template;
        if template.state != TemplateState::Active {
            return Err(TemplateLifecycleError::ArchivedReadOnly);
        }
        let title = super::storage::normalize_task_text(title, 120, 480).map_err(map_storage)?;
        super::storage::validate_plan_body(plan_text).map_err(map_storage)?;
        let context = self
            .repository
            .task_template_application_context(task_id, plan_id)
            .map_err(map_storage)?;
        let binding_sha256 = super::storage::task_template_application_binding_digest(
            &template, &context, &title, plan_text,
        )
        .ok_or(TemplateLifecycleError::InvalidInput)?;
        let now = super::storage::now_millis();
        let expires_at_ms = now + TEMPLATE_APPLICATION_RESERVATION_TTL_MS;
        let reservation_id = Uuid::now_v7().to_string();
        self.repository
            .create_task_template_application_reservation(&TaskTemplateApplicationReservation {
                id: reservation_id.clone(),
                binding_sha256: binding_sha256.clone(),
                template_id: template.id.clone(),
                template_origin: match template.origin {
                    TemplateOrigin::BuiltIn => "built-in",
                    TemplateOrigin::Local => "local",
                }
                .into(),
                template_version: template.version,
                template_sha256: template.sha256.clone(),
                context,
                created_at_ms: now,
                expires_at_ms,
            })
            .map_err(map_storage)?;
        Ok(TemplateApplicationPreview {
            reservation_id,
            expires_at_ms,
            template,
            binding_sha256,
        })
    }
    pub(crate) fn confirm_application(
        &mut self,
        reservation_id: &str,
        title: &str,
        plan_text: &str,
    ) -> Result<(), TemplateLifecycleError> {
        self.repository
            .confirm_task_template_application(reservation_id, title, plan_text)
            .map_err(map_storage)
    }
    pub(crate) fn new(repository: ProjectRepository) -> Self {
        Self { repository }
    }

    pub(crate) fn catalog(&self) -> Result<TemplateCatalog, TemplateLifecycleError> {
        let mut templates = builtins().into_iter().collect::<Vec<_>>();
        templates.extend(
            self.repository
                .local_templates()
                .map_err(map_storage)?
                .into_iter()
                .map(|r| r.template),
        );
        templates.sort_by(|left, right| {
            template_group(left)
                .cmp(&template_group(right))
                .then(left.title.cmp(&right.title))
                .then(left.id.cmp(&right.id))
        });
        Ok(TemplateCatalog {
            capacity: capacity(&templates)?,
            templates,
        })
    }

    pub(crate) fn inspect(&self, id: &str) -> Result<TemplateInspection, TemplateLifecycleError> {
        if let Some(template) = builtins().into_iter().find(|template| template.id == id) {
            return Ok(TemplateInspection {
                template,
                schema_version: TEMPLATE_SCHEMA_VERSION,
                created_at_ms: None,
                updated_at_ms: None,
                archived_at_ms: None,
                authority: None,
            });
        }
        let record = self
            .repository
            .local_template(id)
            .map_err(map_storage)?
            .ok_or(TemplateLifecycleError::NotFound)?;
        Ok(inspection(record))
    }

    pub(crate) fn create(
        &mut self,
        input: TemplateContentInput,
    ) -> Result<TemplateInspection, TemplateLifecycleError> {
        let template = new_template(input, TemplateState::Active)?;
        self.repository
            .insert_local_template(&template)
            .map(inspection)
            .map_err(map_storage)
    }

    pub(crate) fn update(
        &mut self,
        authority: &TemplateMutationAuthority,
        input: TemplateContentInput,
    ) -> Result<TemplateInspection, TemplateLifecycleError> {
        let record = self.current(authority)?;
        if record.template.state == TemplateState::Archived {
            return Err(TemplateLifecycleError::ArchivedReadOnly);
        }
        let mut template = content_template(&record.template, input)?;
        template.version = record
            .template
            .version
            .checked_add(1)
            .ok_or(TemplateLifecycleError::InvalidInput)?;
        template.sha256 = digest(&template).ok_or(TemplateLifecycleError::InvalidInput)?;
        self.repository
            .replace_local_template(authority.version, &template)
            .map(inspection)
            .map_err(map_storage)
    }

    pub(crate) fn archive(
        &mut self,
        authority: &TemplateMutationAuthority,
    ) -> Result<TemplateInspection, TemplateLifecycleError> {
        self.transition(
            authority,
            TemplateState::Active,
            TemplateState::Archived,
            TemplateLifecycleError::ArchivedAlready,
        )
    }
    pub(crate) fn reactivate(
        &mut self,
        authority: &TemplateMutationAuthority,
    ) -> Result<TemplateInspection, TemplateLifecycleError> {
        self.transition(
            authority,
            TemplateState::Archived,
            TemplateState::Active,
            TemplateLifecycleError::ActiveAlready,
        )
    }
    pub(crate) fn duplicate(
        &mut self,
        authority: &TemplateMutationAuthority,
    ) -> Result<TemplateInspection, TemplateLifecycleError> {
        let record = self.current(authority)?;
        let template = new_template(
            TemplateContentInput {
                title: record.template.title,
                purpose: record.template.purpose,
                instructions: record.template.instructions,
            },
            TemplateState::Active,
        )?;
        self.repository
            .insert_local_template(&template)
            .map(inspection)
            .map_err(map_storage)
    }
    pub(crate) fn delete(
        &mut self,
        authority: &TemplateMutationAuthority,
        _confirmation: DeletionConfirmation,
    ) -> Result<(), TemplateLifecycleError> {
        let record = self.current(authority)?;
        if record.template.state != TemplateState::Archived {
            return Err(TemplateLifecycleError::ArchivedReadOnly);
        }
        self.repository
            .delete_local_template(&record.template.id)
            .map_err(map_storage)
    }

    fn current(
        &self,
        authority: &TemplateMutationAuthority,
    ) -> Result<StoredLocalTaskTemplate, TemplateLifecycleError> {
        if builtins()
            .iter()
            .any(|template| template.id == authority.id)
        {
            return Err(TemplateLifecycleError::BuiltInImmutable);
        }
        let record = self
            .repository
            .local_template(&authority.id)
            .map_err(map_storage)?
            .ok_or(TemplateLifecycleError::NotFound)?;
        if record.template.version != authority.version
            || record.template.sha256 != authority.digest
        {
            return Err(TemplateLifecycleError::Stale);
        }
        Ok(record)
    }
    fn transition(
        &mut self,
        authority: &TemplateMutationAuthority,
        from: TemplateState,
        to: TemplateState,
        already: TemplateLifecycleError,
    ) -> Result<TemplateInspection, TemplateLifecycleError> {
        let record = self.current(authority)?;
        if record.template.state != from {
            return Err(already);
        }
        let mut template = record.template;
        template.state = to;
        template.version = template
            .version
            .checked_add(1)
            .ok_or(TemplateLifecycleError::InvalidInput)?;
        template.sha256 = digest(&template).ok_or(TemplateLifecycleError::InvalidInput)?;
        self.repository
            .replace_local_template(authority.version, &template)
            .map(inspection)
            .map_err(map_storage)
    }
}

fn inspection(record: StoredLocalTaskTemplate) -> TemplateInspection {
    let authority = TemplateMutationAuthority {
        id: record.template.id.clone(),
        version: record.template.version,
        digest: record.template.sha256.clone(),
    };
    TemplateInspection {
        template: record.template,
        schema_version: TEMPLATE_SCHEMA_VERSION,
        created_at_ms: Some(record.created_at_ms),
        updated_at_ms: Some(record.updated_at_ms),
        archived_at_ms: record.archived_at_ms,
        authority: Some(authority),
    }
}
fn template_group(template: &TaskTemplate) -> u8 {
    match (template.origin, template.state) {
        (TemplateOrigin::BuiltIn, _) => 0,
        (TemplateOrigin::Local, TemplateState::Active) => 1,
        (TemplateOrigin::Local, TemplateState::Archived) => 2,
    }
}
fn capacity(templates: &[TaskTemplate]) -> Result<TemplateCapacityFacts, TemplateLifecycleError> {
    let canonical_bytes = templates
        .iter()
        .map(|template| {
            canonical(template)
                .map(|v| v.len())
                .ok_or(TemplateLifecycleError::Unavailable)
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .sum();
    Ok(TemplateCapacityFacts {
        record_count: templates.len(),
        canonical_bytes,
        warning: warning(templates.len(), canonical_bytes),
        count_limit: TEMPLATE_COUNT_LIMIT,
        canonical_byte_limit: TEMPLATE_PAYLOAD_LIMIT,
    })
}
fn new_template(
    input: TemplateContentInput,
    state: TemplateState,
) -> Result<TaskTemplate, TemplateLifecycleError> {
    let title =
        normalized_single(&input.title, 80, 320).ok_or(TemplateLifecycleError::InvalidInput)?;
    let purpose =
        normalized_single(&input.purpose, 240, 960).ok_or(TemplateLifecycleError::InvalidInput)?;
    let instructions = normalize_instructions(&input.instructions)?;
    let mut template = TaskTemplate {
        id: Uuid::now_v7().to_string(),
        origin: TemplateOrigin::Local,
        title,
        purpose,
        instructions,
        version: 1,
        state,
        sha256: String::new(),
    };
    template.sha256 = digest(&template).ok_or(TemplateLifecycleError::InvalidInput)?;
    Ok(template)
}
fn content_template(
    previous: &TaskTemplate,
    input: TemplateContentInput,
) -> Result<TaskTemplate, TemplateLifecycleError> {
    let mut template = new_template(input, previous.state)?;
    template.id = previous.id.clone();
    template.version = previous.version;
    template.sha256.clear();
    Ok(template)
}
fn normalize_instructions(value: &str) -> Result<String, TemplateLifecycleError> {
    let value = value
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    valid_instructions(&value)
        .then_some(value)
        .ok_or(TemplateLifecycleError::InvalidInput)
}
fn map_storage(error: StorageError) -> TemplateLifecycleError {
    match error {
        StorageError::TaskCapacity => TemplateLifecycleError::Capacity,
        StorageError::TaskNotFound => TemplateLifecycleError::NotFound,
        StorageError::InvalidStatusTransition => TemplateLifecycleError::Stale,
        StorageError::InvalidStoredValue => TemplateLifecycleError::Unavailable,
        _ => TemplateLifecycleError::Unavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::task_template::TEMPLATE_WARNING_COUNT;
    fn service() -> TemplateLifecycleService {
        TemplateLifecycleService::new(ProjectRepository::in_memory().expect("database"))
    }
    fn input(title: &str) -> TemplateContentInput {
        TemplateContentInput {
            title: title.into(),
            purpose: "A bounded purpose.".into(),
            instructions: " A\n bounded\t instruction. ".into(),
        }
    }
    fn task_and_plan(service: &mut TemplateLifecycleService) -> (String, String) {
        let task_id = service.repository.create_task().unwrap();
        let plan_id = service
            .repository
            .connection
            .query_row(
                "SELECT selected_plan_id FROM task_records WHERE id = ?1",
                [&task_id],
                |row| row.get(0),
            )
            .unwrap();
        (task_id, plan_id)
    }
    fn authority(inspection: &TemplateInspection) -> TemplateMutationAuthority {
        inspection.authority.clone().expect("local authority")
    }
    #[test]
    fn catalog_and_inspection_are_complete_and_deterministic() {
        let mut service = service();
        let local = service.create(input("Local")).unwrap();
        let catalog = service.catalog().unwrap();
        assert_eq!(catalog.templates.len(), 5);
        assert_eq!(catalog.templates[0].origin, TemplateOrigin::BuiltIn);
        assert_eq!(service.inspect(&local.template.id).unwrap(), local);
        assert!(service
            .inspect(&catalog.templates[0].id)
            .unwrap()
            .authority
            .is_none());
    }
    #[test]
    fn create_update_and_stale_rejection_preserve_authority() {
        let mut service = service();
        let created = service.create(input("Create")).unwrap();
        assert_eq!(created.template.version, 1);
        assert_eq!(created.template.instructions, "A bounded instruction.");
        let updated = service
            .update(&authority(&created), input("Updated"))
            .unwrap();
        assert_eq!(updated.template.version, 2);
        assert!(matches!(
            service.update(&authority(&created), input("Stale")),
            Err(TemplateLifecycleError::Stale)
        ));
        assert_eq!(
            service.inspect(&updated.template.id).unwrap().template,
            updated.template
        );
    }
    #[test]
    fn lifecycle_transitions_duplicate_and_confirmed_archived_delete_are_closed() {
        let mut service = service();
        let created = service.create(input("Lifecycle")).unwrap();
        assert!(matches!(
            service.reactivate(&authority(&created)),
            Err(TemplateLifecycleError::ActiveAlready)
        ));
        let archived = service.archive(&authority(&created)).unwrap();
        assert!(matches!(
            service.update(&authority(&archived), input("No edit")),
            Err(TemplateLifecycleError::ArchivedReadOnly)
        ));
        assert!(matches!(
            service.delete(&authority(&created), DeletionConfirmation::Confirmed),
            Err(TemplateLifecycleError::Stale)
        ));
        let duplicate = service.duplicate(&authority(&archived)).unwrap();
        assert_eq!(duplicate.template.state, TemplateState::Active);
        let active_duplicate = authority(&duplicate);
        assert!(matches!(
            service.delete(&active_duplicate, DeletionConfirmation::Confirmed),
            Err(TemplateLifecycleError::ArchivedReadOnly)
        ));
        let restored = service.reactivate(&authority(&archived)).unwrap();
        let rearchived = service.archive(&authority(&restored)).unwrap();
        service
            .delete(&authority(&rearchived), DeletionConfirmation::Confirmed)
            .unwrap();
        assert!(matches!(
            service.inspect(&rearchived.template.id),
            Err(TemplateLifecycleError::NotFound)
        ));
    }
    #[test]
    fn builtins_cannot_mutate_and_invalid_storage_fails_closed() {
        let mut service = service();
        let builtin = service.inspect(&builtins()[0].id).unwrap();
        let forged = TemplateMutationAuthority {
            id: builtin.template.id,
            version: 1,
            digest: builtin.template.sha256,
        };
        assert!(matches!(
            service.archive(&forged),
            Err(TemplateLifecycleError::BuiltInImmutable)
        ));
        let created = service.create(input("Corrupt")).unwrap();
        service
            .repository
            .connection
            .execute(
                "UPDATE local_task_templates SET sha256 = ?1",
                ["0".repeat(64)],
            )
            .unwrap();
        assert!(matches!(
            service.catalog(),
            Err(TemplateLifecycleError::Unavailable)
        ));
        assert!(created.template.id.len() == 36);
    }
    #[test]
    fn capacity_warning_and_hard_limit_are_authoritative() {
        let mut service = service();
        for index in 0..44 {
            service.create(input(&format!("T{index}"))).unwrap();
        }
        assert!(service.catalog().unwrap().capacity.warning);
        assert_eq!(
            service.catalog().unwrap().capacity.record_count,
            TEMPLATE_WARNING_COUNT
        );
        service
            .repository
            .connection
            .execute_batch("PRAGMA ignore_check_constraints = ON")
            .unwrap();
        for _ in 0..17 {
            let template = new_template(input("Over"), TemplateState::Active).unwrap();
            service.repository.insert_local_template(&template).ok();
        }
        assert!(matches!(
            service.create(input("Blocked")),
            Err(TemplateLifecycleError::Capacity)
        ));
    }
    #[test]
    fn preview_and_confirmation_update_only_bound_task_and_plan() {
        let mut service = service();
        let (task_id, plan_id) = task_and_plan(&mut service);
        let preview = service
            .preview_application(
                &builtins()[0].id,
                &task_id,
                &plan_id,
                "  Applied title  ",
                "Applied plan\ntext",
            )
            .unwrap();
        service
            .confirm_application(
                &preview.reservation_id,
                "Applied title",
                "Applied plan\ntext",
            )
            .unwrap();
        let values: (String, String, String) = service.repository.connection.query_row(
            "SELECT t.title, p.body, r.state FROM task_records t JOIN task_plans p ON p.id=?1 JOIN task_template_application_reservations r ON r.id=?2 WHERE t.id=?3",
            rusqlite::params![plan_id, preview.reservation_id, task_id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?)),
        ).unwrap();
        assert_eq!(
            values,
            (
                "Applied title".into(),
                "Applied plan\ntext".into(),
                "consumed".into()
            )
        );
        assert!(matches!(
            service.confirm_application(
                &preview.reservation_id,
                "Applied title",
                "Applied plan\ntext"
            ),
            Err(TemplateLifecycleError::NotFound)
        ));
    }
    #[test]
    fn application_rejects_changed_draft_template_and_concurrency() {
        let mut service = service();
        let (task_id, plan_id) = task_and_plan(&mut service);
        let preview = service
            .preview_application(&builtins()[0].id, &task_id, &plan_id, "Draft", "Plan")
            .unwrap();
        assert!(matches!(
            service.confirm_application(&preview.reservation_id, "Different", "Plan"),
            Err(TemplateLifecycleError::Stale)
        ));
        let preview = service
            .preview_application(&builtins()[0].id, &task_id, &plan_id, "Draft", "Plan")
            .unwrap();
        service.repository.rename_task(&task_id, "Changed").unwrap();
        assert!(matches!(
            service.confirm_application(&preview.reservation_id, "Draft", "Plan"),
            Err(TemplateLifecycleError::Stale)
        ));
    }
    #[test]
    fn application_reservations_never_store_drafts_or_instructions() {
        let mut service = service();
        let (task_id, plan_id) = task_and_plan(&mut service);
        let preview = service
            .preview_application(
                &builtins()[0].id,
                &task_id,
                &plan_id,
                "Secret draft",
                "Secret plan",
            )
            .unwrap();
        let sql: String = service.repository.connection.query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='task_template_application_reservations'", [], |row| row.get(0),
        ).unwrap();
        assert!(
            !sql.contains("instructions") && !sql.contains("draft") && !sql.contains("plan_text")
        );
        service
            .repository
            .cancel_task_template_application_reservation(&preview.reservation_id)
            .unwrap();
        assert!(matches!(
            service.confirm_application(&preview.reservation_id, "Secret draft", "Secret plan"),
            Err(TemplateLifecycleError::NotFound)
        ));
    }
}
