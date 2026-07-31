use sha2::{Digest, Sha256};
use uuid::Uuid;

pub(crate) const TEMPLATE_SCHEMA_VERSION: u16 = 1;
pub(crate) const TEMPLATE_COUNT_LIMIT: usize = 64;
pub(crate) const TEMPLATE_PAYLOAD_LIMIT: usize = 2 * 1024 * 1024;
pub(crate) const TEMPLATE_WARNING_COUNT: usize = 48;
pub(crate) const TEMPLATE_WARNING_PAYLOAD: usize = 1536 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TemplateOrigin {
    BuiltIn,
    Local,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TemplateState {
    Active,
    Archived,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TaskTemplate {
    pub id: String,
    pub origin: TemplateOrigin,
    pub title: String,
    pub purpose: String,
    pub instructions: String,
    pub version: u32,
    pub state: TemplateState,
    pub sha256: String,
}

fn uuid(value: &str) -> bool {
    Uuid::parse_str(value).is_ok_and(|id| id.get_version_num() == 7)
}
fn bad(value: &str, instructions: bool) -> bool {
    value.chars().any(|c| {
        (c == '\0')
            || (c.is_control() && (!instructions || c != '\n' && c != '\t'))
            || matches!(c as u32, 0x061c|0x200e|0x200f|0x202a..=0x202e|0x2066..=0x2069)
    })
}
pub(crate) fn normalized_single(value: &str, chars: usize, bytes: usize) -> Option<String> {
    let v = value
        .trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    (!v.is_empty() && v.chars().count() <= chars && v.len() <= bytes && !bad(&v, false))
        .then_some(v)
}
pub(crate) fn valid_instructions(value: &str) -> bool {
    value.len() <= 32 * 1024 && !bad(value, true)
}
pub(crate) fn canonical(template: &TaskTemplate) -> Option<String> {
    if !uuid(&template.id)
        || template.version == 0
        || normalized_single(&template.title, 80, 320).as_deref() != Some(&template.title)
        || normalized_single(&template.purpose, 240, 960).as_deref() != Some(&template.purpose)
        || !valid_instructions(&template.instructions)
    {
        return None;
    }
    let origin = match template.origin {
        TemplateOrigin::BuiltIn => "built-in",
        TemplateOrigin::Local => "local",
    };
    let state = match template.state {
        TemplateState::Active => "active",
        TemplateState::Archived => "archived",
    };
    let value = format!(
        "task-template-v1\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        template.id,
        origin,
        template.title,
        template.purpose,
        template.instructions,
        template.version,
        state
    );
    (value.len() <= 64 * 1024).then_some(value)
}
pub(crate) fn digest(template: &TaskTemplate) -> Option<String> {
    canonical(template).map(|v| format!("{:x}", Sha256::digest(v.as_bytes())))
}
pub(crate) fn valid(template: &TaskTemplate) -> bool {
    digest(template).is_some_and(|d| {
        d == template.sha256
            && template.sha256.len() == 64
            && template
                .sha256
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    })
}
fn builtin(id: &str, title: &str, purpose: &str, instructions: &str) -> TaskTemplate {
    let mut t = TaskTemplate {
        id: id.into(),
        origin: TemplateOrigin::BuiltIn,
        title: title.into(),
        purpose: purpose.into(),
        instructions: instructions.into(),
        version: 1,
        state: TemplateState::Active,
        sha256: String::new(),
    };
    t.sha256 = digest(&t).expect("builtin valid");
    t
}
pub(crate) fn builtins() -> [TaskTemplate; 4] {
    [
 builtin("01980a10-0000-7000-8000-000000000001","Feature implementation","Plan a bounded feature.","Define outcome, constraints, evidence, tests, risks, and completion criteria."),
 builtin("01980a10-0000-7000-8000-000000000002","Bug investigation","Plan a bounded investigation.","Define observed behavior, evidence, hypotheses, tests, risks, and completion criteria."),
 builtin("01980a10-0000-7000-8000-000000000003","Code review","Plan a bounded review.","Define scope, constraints, evidence, tests, risks, and completion criteria."),
 builtin("01980a10-0000-7000-8000-000000000004","Documentation update","Plan a bounded documentation update.","Define audience, constraints, evidence, checks, risks, and completion criteria."),
]
}
pub(crate) fn warning(count: usize, payload: usize) -> bool {
    count >= TEMPLATE_WARNING_COUNT || payload >= TEMPLATE_WARNING_PAYLOAD
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builtins_are_valid_and_distinct() {
        let x = builtins();
        assert!(x.iter().all(valid));
        assert_eq!(
            x.iter()
                .map(|x| &x.id)
                .collect::<std::collections::HashSet<_>>()
                .len(),
            4
        );
        assert_eq!(
            x.iter()
                .map(|x| &x.sha256)
                .collect::<std::collections::HashSet<_>>()
                .len(),
            4
        );
    }
    #[test]
    fn normalization_and_bounds_fail_closed() {
        assert_eq!(normalized_single(" a\t b ", 80, 320), Some("a b".into()));
        assert!(normalized_single("\u{202e}", 80, 320).is_none());
        assert!(!valid_instructions("\0"));
        assert!(warning(48, 0));
        assert!(warning(0, TEMPLATE_WARNING_PAYLOAD));
        assert!(!warning(47, TEMPLATE_WARNING_PAYLOAD - 1));
    }
}
