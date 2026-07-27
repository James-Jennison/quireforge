//! One-use, native-only bounded PDF projections for Advisor.
//!
//! PDF source bytes and paths are consumed here, then discarded. Only a
//! deliberately bounded plain-text projection can enter the Advisor turn.
//!
//! `lopdf` is the pinned, delegated syntax parser. A successful `load_mem`
//! means only that that parser accepted enough of the source to construct a
//! document. QuireForge deliberately does not claim independent xref, trailer,
//! EOF, offset, object-stream, or low-level syntax diagnostics.

use std::{
    fs::{File, OpenOptions},
    io::Read,
    os::{
        fd::AsRawFd,
        unix::fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant},
};

use lopdf::{Document, Object, ObjectId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use uuid::Uuid;

use crate::advisor_attachment::{
    AdvisorContentCategory, AdvisorContentConfirmationState, AdvisorContentDisposal,
};

pub const MAX_ADVISOR_DOCUMENT_BYTES: usize = 8 * 1024 * 1024;
const MAX_PAGES: usize = 200;
const MAX_PDF_OBJECTS: usize = 10_000;
const MAX_PDF_NESTING: usize = 64;
const MAX_PROJECTION_BYTES: usize = 256 * 1024;
const MAX_PROJECTION_CHARS: usize = 200_000;
const PARSE_BUDGET: Duration = Duration::from_secs(5);
const ATTACHMENT_TTL: Duration = Duration::from_secs(15 * 60);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorDocumentMediaType {
    Pdf,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorDocumentProjectionKind {
    PdfPlainTextV1,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorDocumentAttachmentState {
    Empty,
    Ready,
    Unavailable,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorDocumentAttachmentDiagnosticCode {
    InvalidRequest,
    UnsupportedType,
    FileTooLarge,
    MalformedOrUnsupportedDocument,
    UnsafeName,
    ReadFailed,
    Encrypted,
    ActiveContent,
    EmbeddedContent,
    ExternalAction,
    PageLimitExceeded,
    ParseBudgetExceeded,
    ObjectLimitExceeded,
    NestingLimitExceeded,
    AttachmentNotFound,
    AttachmentExpired,
    ManifestMismatch,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorDocumentProjection {
    pub kind: AdvisorDocumentProjectionKind,
    pub schema_version: u16,
    pub page_count: u32,
    pub processed_page_count: u32,
    pub included_page_count: u32,
    pub omitted_page_count: u32,
    pub partial_page_count: u32,
    pub projected_byte_size: u32,
    pub outline_entry_count: u16,
    pub truncated: bool,
    pub warnings: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorDocumentAttachmentManifest {
    pub attachment_id: String,
    pub display_name: String,
    pub content_category: AdvisorContentCategory,
    pub media_type: AdvisorDocumentMediaType,
    pub byte_size: u64,
    pub sha256: String,
    pub projection: AdvisorDocumentProjection,
    pub disposal: AdvisorContentDisposal,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorDocumentAttachmentSnapshot {
    pub schema_version: u16,
    pub state: AdvisorDocumentAttachmentState,
    pub attachment: Option<AdvisorDocumentAttachmentManifest>,
    pub confirmation_state: Option<AdvisorContentConfirmationState>,
    pub diagnostic_code: Option<AdvisorDocumentAttachmentDiagnosticCode>,
}
impl AdvisorDocumentAttachmentSnapshot {
    pub fn empty() -> Self {
        Self {
            schema_version: 1,
            state: AdvisorDocumentAttachmentState::Empty,
            attachment: None,
            confirmation_state: None,
            diagnostic_code: None,
        }
    }
    fn unavailable(code: AdvisorDocumentAttachmentDiagnosticCode) -> Self {
        Self {
            schema_version: 1,
            state: AdvisorDocumentAttachmentState::Unavailable,
            attachment: None,
            confirmation_state: None,
            diagnostic_code: Some(code),
        }
    }
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorDocumentAttachmentClaimRequest {
    pub attachment_id: String,
    pub manifest_sha256: String,
    pub confirmation: AdvisorContentConfirmationState,
}
pub(crate) struct ClaimedAdvisorDocumentAttachment {
    pub(crate) manifest: AdvisorDocumentAttachmentManifest,
    pub(crate) projection_text: String,
}
struct PendingDocument {
    manifest: AdvisorDocumentAttachmentManifest,
    projection_text: String,
    created_at: Instant,
}
#[derive(Default)]
pub struct AdvisorDocumentAttachmentService {
    pending: Mutex<Option<PendingDocument>>,
}

impl AdvisorDocumentAttachmentService {
    pub fn snapshot(&self) -> AdvisorDocumentAttachmentSnapshot {
        let Ok(mut pending) = self.pending.lock() else {
            return AdvisorDocumentAttachmentSnapshot::unavailable(
                AdvisorDocumentAttachmentDiagnosticCode::ReadFailed,
            );
        };
        if pending
            .as_ref()
            .is_some_and(|item| item.created_at.elapsed() > ATTACHMENT_TTL)
        {
            *pending = None;
            return AdvisorDocumentAttachmentSnapshot::unavailable(
                AdvisorDocumentAttachmentDiagnosticCode::AttachmentExpired,
            );
        }
        pending
            .as_ref()
            .map_or_else(AdvisorDocumentAttachmentSnapshot::empty, |item| {
                AdvisorDocumentAttachmentSnapshot {
                    schema_version: 1,
                    state: AdvisorDocumentAttachmentState::Ready,
                    attachment: Some(item.manifest.clone()),
                    confirmation_state: Some(AdvisorContentConfirmationState::ConfirmationRequired),
                    diagnostic_code: None,
                }
            })
    }
    pub fn stage_path(&self, path: PathBuf) -> AdvisorDocumentAttachmentSnapshot {
        match prepare(&path) {
            Ok((manifest, projection_text)) => match self.pending.lock() {
                Ok(mut pending) => {
                    *pending = Some(PendingDocument {
                        manifest: manifest.clone(),
                        projection_text,
                        created_at: Instant::now(),
                    });
                    AdvisorDocumentAttachmentSnapshot {
                        schema_version: 1,
                        state: AdvisorDocumentAttachmentState::Ready,
                        attachment: Some(manifest),
                        confirmation_state: Some(
                            AdvisorContentConfirmationState::ConfirmationRequired,
                        ),
                        diagnostic_code: None,
                    }
                }
                Err(_) => AdvisorDocumentAttachmentSnapshot::unavailable(
                    AdvisorDocumentAttachmentDiagnosticCode::ReadFailed,
                ),
            },
            Err(code) => AdvisorDocumentAttachmentSnapshot::unavailable(code),
        }
    }
    pub fn clear(&self) -> AdvisorDocumentAttachmentSnapshot {
        match self.pending.lock() {
            Ok(mut pending) => {
                *pending = None;
                AdvisorDocumentAttachmentSnapshot::empty()
            }
            Err(_) => AdvisorDocumentAttachmentSnapshot::unavailable(
                AdvisorDocumentAttachmentDiagnosticCode::ReadFailed,
            ),
        }
    }
    pub fn claim(
        &self,
        request: &AdvisorDocumentAttachmentClaimRequest,
    ) -> Result<ClaimedAdvisorDocumentAttachment, AdvisorDocumentAttachmentDiagnosticCode> {
        if !valid_uuid_v7(&request.attachment_id)
            || !valid_hash(&request.manifest_sha256)
            || request.confirmation != AdvisorContentConfirmationState::ConfirmedForSingleSend
        {
            return Err(AdvisorDocumentAttachmentDiagnosticCode::InvalidRequest);
        }
        let mut pending = self
            .pending
            .lock()
            .map_err(|_| AdvisorDocumentAttachmentDiagnosticCode::ReadFailed)?;
        let Some(item) = pending.take() else {
            return Err(AdvisorDocumentAttachmentDiagnosticCode::AttachmentNotFound);
        };
        if item.created_at.elapsed() > ATTACHMENT_TTL {
            return Err(AdvisorDocumentAttachmentDiagnosticCode::AttachmentExpired);
        }
        if item.manifest.attachment_id != request.attachment_id
            || item.manifest.sha256 != request.manifest_sha256
        {
            return Err(AdvisorDocumentAttachmentDiagnosticCode::ManifestMismatch);
        }
        Ok(ClaimedAdvisorDocumentAttachment {
            manifest: item.manifest,
            projection_text: item.projection_text,
        })
    }
}

fn prepare(
    path: &Path,
) -> Result<(AdvisorDocumentAttachmentManifest, String), AdvisorDocumentAttachmentDiagnosticCode> {
    if !path.is_absolute() {
        return Err(AdvisorDocumentAttachmentDiagnosticCode::InvalidRequest);
    }
    let selected = path
        .symlink_metadata()
        .map_err(|_| AdvisorDocumentAttachmentDiagnosticCode::ReadFailed)?;
    if selected.file_type().is_symlink() || !selected.is_file() {
        return Err(AdvisorDocumentAttachmentDiagnosticCode::ReadFailed);
    }
    if selected.len() > MAX_ADVISOR_DOCUMENT_BYTES as u64 {
        return Err(AdvisorDocumentAttachmentDiagnosticCode::FileTooLarge);
    }
    let display_name = validate_display_name(
        path.file_name()
            .and_then(|n| n.to_str())
            .ok_or(AdvisorDocumentAttachmentDiagnosticCode::UnsafeName)?,
    )?;
    if !display_name.to_ascii_lowercase().ends_with(".pdf") {
        return Err(AdvisorDocumentAttachmentDiagnosticCode::UnsupportedType);
    }
    let resolved = path
        .canonicalize()
        .map_err(|_| AdvisorDocumentAttachmentDiagnosticCode::ReadFailed)?;
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(&resolved)
        .map_err(|_| AdvisorDocumentAttachmentDiagnosticCode::ReadFailed)?;
    let opened = file
        .metadata()
        .map_err(|_| AdvisorDocumentAttachmentDiagnosticCode::ReadFailed)?;
    if !source_identity_matches(&selected, &opened, &descriptor_path(&file)?, &resolved) {
        return Err(AdvisorDocumentAttachmentDiagnosticCode::ReadFailed);
    }
    let mut bytes = Vec::with_capacity(opened.len() as usize);
    (&mut file)
        .take(MAX_ADVISOR_DOCUMENT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| AdvisorDocumentAttachmentDiagnosticCode::ReadFailed)?;
    if bytes.len() as u64 != opened.len() || !bytes.starts_with(b"%PDF-") {
        return Err(AdvisorDocumentAttachmentDiagnosticCode::MalformedOrUnsupportedDocument);
    }
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    let (projection, projection_text) = project_pdf(&bytes)?;
    Ok((
        AdvisorDocumentAttachmentManifest {
            attachment_id: Uuid::now_v7().to_string(),
            display_name,
            content_category: AdvisorContentCategory::Document,
            media_type: AdvisorDocumentMediaType::Pdf,
            byte_size: bytes.len() as u64,
            sha256,
            projection,
            disposal: AdvisorContentDisposal::TransientMemoryOneSend,
        },
        projection_text,
    ))
}
fn source_identity_matches(
    selected: &std::fs::Metadata,
    opened: &std::fs::Metadata,
    descriptor: &Path,
    resolved: &Path,
) -> bool {
    opened.is_file()
        && opened.len() == selected.len()
        && opened.dev() == selected.dev()
        && opened.ino() == selected.ino()
        && descriptor == resolved
}
fn project_pdf(
    bytes: &[u8],
) -> Result<(AdvisorDocumentProjection, String), AdvisorDocumentAttachmentDiagnosticCode> {
    let started = Instant::now();
    // Parser errors intentionally map to one stable, path-free category. They
    // are compatibility evidence for the pinned parser, not a second PDF parser.
    let document = Document::load_mem(bytes)
        .map_err(|_| AdvisorDocumentAttachmentDiagnosticCode::MalformedOrUnsupportedDocument)?;
    if document.is_encrypted() {
        return Err(AdvisorDocumentAttachmentDiagnosticCode::Encrypted);
    }
    inspect_pdf_object_graph(&document)?;
    let pages: Vec<u32> = document.get_pages().into_keys().collect();
    if pages.len() > MAX_PAGES {
        return Err(AdvisorDocumentAttachmentDiagnosticCode::PageLimitExceeded);
    }
    let mut text = String::new();
    let mut truncated = false;
    let mut inspected = 0u32;
    let mut included = 0u32;
    let mut partial = 0u32;
    for page in &pages {
        if started.elapsed() > PARSE_BUDGET {
            return Err(AdvisorDocumentAttachmentDiagnosticCode::ParseBudgetExceeded);
        }
        let page_text = document
            .extract_text(&[*page])
            .map_err(|_| AdvisorDocumentAttachmentDiagnosticCode::MalformedOrUnsupportedDocument)?;
        let before = text.len();
        append_bounded(&mut text, &page_text, &mut truncated);
        inspected += 1;
        if truncated {
            // A page is "included" only when its complete extracted text was
            // admitted to the projection.  Keeping a separate partial count
            // prevents the UI from claiming a fully projected page when the
            // byte limit stopped in the middle of it.
            if text.len() > before {
                partial = 1;
            }
            break;
        }
        if text.len() > before {
            included += 1;
        }
    }
    let mut warnings = Vec::new();
    if truncated {
        warnings.push("projection-truncated".to_owned());
    }
    let projection = AdvisorDocumentProjection {
        kind: AdvisorDocumentProjectionKind::PdfPlainTextV1,
        schema_version: 1,
        page_count: pages.len() as u32,
        processed_page_count: inspected,
        included_page_count: included,
        omitted_page_count: pages.len().saturating_sub(inspected as usize) as u32,
        partial_page_count: partial,
        projected_byte_size: text.len() as u32,
        outline_entry_count: 0,
        truncated,
        warnings,
    };
    Ok((projection, text))
}
fn inspect_pdf_object_graph(
    document: &Document,
) -> Result<(), AdvisorDocumentAttachmentDiagnosticCode> {
    if document.objects.len() > MAX_PDF_OBJECTS {
        return Err(AdvisorDocumentAttachmentDiagnosticCode::ObjectLimitExceeded);
    }
    let mut visited = HashSet::new();
    for (id, object) in &document.objects {
        inspect_pdf_object(document, object, Some(*id), 0, &mut visited)?;
    }
    Ok(())
}
fn inspect_pdf_object(
    document: &Document,
    object: &Object,
    object_id: Option<ObjectId>,
    depth: usize,
    visited: &mut HashSet<ObjectId>,
) -> Result<(), AdvisorDocumentAttachmentDiagnosticCode> {
    if depth > MAX_PDF_NESTING {
        return Err(AdvisorDocumentAttachmentDiagnosticCode::NestingLimitExceeded);
    }
    if let Some(id) = object_id {
        if !visited.insert(id) {
            return Ok(());
        }
        if visited.len() > MAX_PDF_OBJECTS {
            return Err(AdvisorDocumentAttachmentDiagnosticCode::ObjectLimitExceeded);
        }
    }
    match object {
        Object::Reference(id) => {
            let target = document
                .objects
                .get(id)
                .ok_or(AdvisorDocumentAttachmentDiagnosticCode::MalformedOrUnsupportedDocument)?;
            inspect_pdf_object(document, target, Some(*id), depth + 1, visited)?;
        }
        Object::Array(items) => {
            for item in items {
                inspect_pdf_object(document, item, None, depth + 1, visited)?;
            }
        }
        Object::Dictionary(dictionary) => {
            inspect_pdf_dictionary(document, dictionary, depth, visited)?
        }
        Object::Stream(stream) => inspect_pdf_dictionary(document, &stream.dict, depth, visited)?,
        _ => {}
    }
    Ok(())
}
fn inspect_pdf_dictionary(
    document: &Document,
    dictionary: &lopdf::Dictionary,
    depth: usize,
    visited: &mut HashSet<ObjectId>,
) -> Result<(), AdvisorDocumentAttachmentDiagnosticCode> {
    for (key, value) in dictionary.iter() {
        if let Some(code) = prohibited_key_code(key.as_slice()) {
            return Err(code);
        }
        if matches!(key.as_slice(), b"S" | b"Type") {
            if let Object::Name(name) = value {
                if let Some(code) = prohibited_key_code(name) {
                    return Err(code);
                }
            }
        }
        inspect_pdf_object(document, value, None, depth + 1, visited)?;
    }
    Ok(())
}
fn prohibited_key_code(key: &[u8]) -> Option<AdvisorDocumentAttachmentDiagnosticCode> {
    match key {
        b"EmbeddedFiles" | b"Filespec" => {
            Some(AdvisorDocumentAttachmentDiagnosticCode::EmbeddedContent)
        }
        b"URI" | b"GoToR" | b"Launch" | b"SubmitForm" | b"ImportData" => {
            Some(AdvisorDocumentAttachmentDiagnosticCode::ExternalAction)
        }
        b"JavaScript" | b"JS" | b"AA" | b"OpenAction" | b"AcroForm" | b"XFA" | b"RichMedia"
        | b"Movie" | b"Sound" | b"Collection" | b"Portfolio" => {
            Some(AdvisorDocumentAttachmentDiagnosticCode::ActiveContent)
        }
        _ => None,
    }
}
fn append_bounded(output: &mut String, addition: &str, truncated: &mut bool) {
    for character in addition.chars() {
        if output.len() + character.len_utf8() > MAX_PROJECTION_BYTES
            || output.chars().count() >= MAX_PROJECTION_CHARS
        {
            *truncated = true;
            break;
        }
        output.push(character);
    }
}
fn validate_display_name(value: &str) -> Result<String, AdvisorDocumentAttachmentDiagnosticCode> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.len() > 255
        || value.contains(['/', '\\'])
        || value.chars().any(|c| {
            c.is_control() || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
    {
        Err(AdvisorDocumentAttachmentDiagnosticCode::UnsafeName)
    } else {
        Ok(value.to_owned())
    }
}
fn descriptor_path(file: &File) -> Result<PathBuf, AdvisorDocumentAttachmentDiagnosticCode> {
    PathBuf::from("/proc/self/fd")
        .join(file.as_raw_fd().to_string())
        .canonicalize()
        .map_err(|_| AdvisorDocumentAttachmentDiagnosticCode::ReadFailed)
}
fn valid_uuid_v7(value: &str) -> bool {
    Uuid::parse_str(value)
        .ok()
        .is_some_and(|id| id.get_version_num() == 7)
}
fn valid_hash(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active_dictionary(key: &str) -> Object {
        let mut dictionary = lopdf::Dictionary::new();
        dictionary.set(key, Object::Null);
        Object::Dictionary(dictionary)
    }

    fn document_with_reference(value: Object) -> Document {
        let mut document = Document::with_version("1.5");
        let mut root = lopdf::Dictionary::new();
        root.set("Next", Object::Reference((2, 0)));
        document.objects.insert((1, 0), Object::Dictionary(root));
        document.objects.insert((2, 0), value);
        document
    }
    #[test]
    fn rejects_non_pdf_content_without_retaining_a_path() {
        let dir = std::env::temp_dir().join(Uuid::now_v7().to_string());
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("unsafe.pdf");
        std::fs::write(&path, b"not a pdf").unwrap();
        let service = AdvisorDocumentAttachmentService::default();
        let snapshot = service.stage_path(path.clone());
        assert_eq!(snapshot.state, AdvisorDocumentAttachmentState::Unavailable);
        assert_eq!(
            snapshot.diagnostic_code,
            Some(AdvisorDocumentAttachmentDiagnosticCode::MalformedOrUnsupportedDocument)
        );
        assert!(!format!("{snapshot:?}").contains(path.to_str().unwrap()));
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn bounded_projection_truncates_deterministically() {
        let mut result = String::new();
        let mut truncated = false;
        append_bounded(
            &mut result,
            &"x".repeat(MAX_PROJECTION_BYTES + 1),
            &mut truncated,
        );
        assert!(truncated);
        assert_eq!(result.chars().count(), MAX_PROJECTION_CHARS);
    }
    #[test]
    fn source_boundary_helpers_are_path_free_and_hash_stable() {
        assert_eq!(
            validate_display_name("brief.pdf"),
            Ok("brief.pdf".to_owned())
        );
        assert_eq!(
            validate_display_name("/tmp/brief.pdf"),
            Err(AdvisorDocumentAttachmentDiagnosticCode::UnsafeName)
        );
        let first = format!("{:x}", Sha256::digest(b"same bytes"));
        let second = format!("{:x}", Sha256::digest(b"same bytes"));
        let changed = format!("{:x}", Sha256::digest(b"changed bytes"));
        assert_eq!(first, second);
        assert_ne!(first, changed);
        assert!(valid_hash(&first));
    }

    #[cfg(unix)]
    #[test]
    fn stage_rejects_direct_symlinks_and_non_regular_sources() {
        use std::os::unix::fs::symlink;
        let dir = std::env::temp_dir().join(Uuid::now_v7().to_string());
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("target.pdf");
        std::fs::write(&target, b"%PDF-not-a-real-document").unwrap();
        let link = dir.join("linked.pdf");
        symlink(&target, &link).unwrap();
        let service = AdvisorDocumentAttachmentService::default();
        assert_eq!(
            service.stage_path(link).diagnostic_code,
            Some(AdvisorDocumentAttachmentDiagnosticCode::ReadFailed)
        );
        assert_eq!(
            service.stage_path(dir.clone()).diagnostic_code,
            Some(AdvisorDocumentAttachmentDiagnosticCode::ReadFailed)
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn source_identity_rejects_replacement_and_descriptor_mismatch() {
        let dir = std::env::temp_dir().join(Uuid::now_v7().to_string());
        std::fs::create_dir_all(&dir).unwrap();
        let original = dir.join("original.pdf");
        let replacement = dir.join("replacement.pdf");
        std::fs::write(&original, b"original").unwrap();
        std::fs::write(&replacement, b"replacement bytes").unwrap();
        let selected = std::fs::metadata(&original).unwrap();
        let opened = std::fs::metadata(&replacement).unwrap();
        assert!(!source_identity_matches(
            &selected,
            &opened,
            &replacement,
            &replacement
        ));
        assert!(!source_identity_matches(
            &selected,
            &selected,
            &replacement,
            &original
        ));
        assert!(source_identity_matches(
            &selected, &selected, &original, &original
        ));
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn active_content_dictionary_is_rejected() {
        let mut dictionary = lopdf::Dictionary::new();
        dictionary.set("JavaScript", Object::Null);
        let mut document = Document::with_version("1.5");
        document
            .objects
            .insert((1, 0), Object::Dictionary(dictionary));
        assert_eq!(
            inspect_pdf_object_graph(&document),
            Err(AdvisorDocumentAttachmentDiagnosticCode::ActiveContent)
        );
    }
    #[test]
    fn nested_indirect_active_content_is_rejected() {
        let mut document = Document::with_version("1.5");
        let mut nested = lopdf::Dictionary::new();
        nested.set("S", Object::Name(b"URI".to_vec()));
        let mut root = lopdf::Dictionary::new();
        root.set("Next", Object::Reference((2, 0)));
        document.objects.insert((1, 0), Object::Dictionary(root));
        document.objects.insert((2, 0), Object::Dictionary(nested));
        assert_eq!(
            inspect_pdf_object_graph(&document),
            Err(AdvisorDocumentAttachmentDiagnosticCode::ExternalAction)
        );
    }

    #[test]
    fn rejects_active_content_in_names_annotations_forms_and_streams() {
        for (key, expected) in [
            (
                "JavaScript",
                AdvisorDocumentAttachmentDiagnosticCode::ActiveContent,
            ),
            (
                "EmbeddedFiles",
                AdvisorDocumentAttachmentDiagnosticCode::EmbeddedContent,
            ),
            (
                "AcroForm",
                AdvisorDocumentAttachmentDiagnosticCode::ActiveContent,
            ),
            (
                "XFA",
                AdvisorDocumentAttachmentDiagnosticCode::ActiveContent,
            ),
            (
                "RichMedia",
                AdvisorDocumentAttachmentDiagnosticCode::ActiveContent,
            ),
            (
                "Collection",
                AdvisorDocumentAttachmentDiagnosticCode::ActiveContent,
            ),
        ] {
            let document = document_with_reference(active_dictionary(key));
            assert_eq!(
                inspect_pdf_object_graph(&document),
                Err(expected),
                "{key} must fail closed"
            );
        }

        let mut stream_dictionary = lopdf::Dictionary::new();
        stream_dictionary.set("S", Object::Name(b"Launch".to_vec()));
        let document = document_with_reference(Object::Stream(lopdf::Stream::new(
            stream_dictionary,
            Vec::new(),
        )));
        assert_eq!(
            inspect_pdf_object_graph(&document),
            Err(AdvisorDocumentAttachmentDiagnosticCode::ExternalAction)
        );
    }

    #[test]
    fn rejects_all_supported_action_names_when_nested() {
        for (action, expected) in [
            (
                "URI",
                AdvisorDocumentAttachmentDiagnosticCode::ExternalAction,
            ),
            (
                "GoToR",
                AdvisorDocumentAttachmentDiagnosticCode::ExternalAction,
            ),
            (
                "SubmitForm",
                AdvisorDocumentAttachmentDiagnosticCode::ExternalAction,
            ),
            (
                "ImportData",
                AdvisorDocumentAttachmentDiagnosticCode::ExternalAction,
            ),
            (
                "Filespec",
                AdvisorDocumentAttachmentDiagnosticCode::EmbeddedContent,
            ),
            (
                "Portfolio",
                AdvisorDocumentAttachmentDiagnosticCode::ActiveContent,
            ),
            (
                "Movie",
                AdvisorDocumentAttachmentDiagnosticCode::ActiveContent,
            ),
            (
                "Sound",
                AdvisorDocumentAttachmentDiagnosticCode::ActiveContent,
            ),
        ] {
            let mut dictionary = lopdf::Dictionary::new();
            dictionary.set("S", Object::Name(action.as_bytes().to_vec()));
            let document = document_with_reference(Object::Dictionary(dictionary));
            assert_eq!(
                inspect_pdf_object_graph(&document),
                Err(expected),
                "{action} must fail closed"
            );
        }
    }

    #[test]
    fn graph_walk_is_bounded_and_fails_closed_for_missing_references() {
        let mut cyclic = Document::with_version("1.5");
        cyclic.objects.insert((1, 0), Object::Reference((1, 0)));
        assert_eq!(inspect_pdf_object_graph(&cyclic), Ok(()));

        let mut missing = Document::with_version("1.5");
        missing.objects.insert((1, 0), Object::Reference((99, 0)));
        assert_eq!(
            inspect_pdf_object_graph(&missing),
            Err(AdvisorDocumentAttachmentDiagnosticCode::MalformedOrUnsupportedDocument)
        );

        let mut deep = Object::Null;
        for _ in 0..=MAX_PDF_NESTING {
            deep = Object::Array(vec![deep]);
        }
        let document = document_with_reference(deep);
        assert_eq!(
            inspect_pdf_object_graph(&document),
            Err(AdvisorDocumentAttachmentDiagnosticCode::NestingLimitExceeded)
        );
    }

    fn pending_document() -> PendingDocument {
        PendingDocument {
            manifest: AdvisorDocumentAttachmentManifest {
                attachment_id: "018f0000-0000-7000-8000-000000000099".to_owned(),
                display_name: "brief.pdf".to_owned(),
                content_category: AdvisorContentCategory::Document,
                media_type: AdvisorDocumentMediaType::Pdf,
                byte_size: 42,
                sha256: "a".repeat(64),
                projection: AdvisorDocumentProjection {
                    kind: AdvisorDocumentProjectionKind::PdfPlainTextV1,
                    schema_version: 1,
                    page_count: 1,
                    processed_page_count: 1,
                    included_page_count: 1,
                    omitted_page_count: 0,
                    partial_page_count: 0,
                    projected_byte_size: 4,
                    outline_entry_count: 0,
                    truncated: false,
                    warnings: Vec::new(),
                },
                disposal: AdvisorContentDisposal::TransientMemoryOneSend,
            },
            projection_text: "safe".to_owned(),
            created_at: Instant::now(),
        }
    }

    #[test]
    fn claim_is_confirmation_hash_bound_one_use_and_disposes_transient_data() {
        let service = AdvisorDocumentAttachmentService {
            pending: Mutex::new(Some(pending_document())),
        };
        let mismatch = AdvisorDocumentAttachmentClaimRequest {
            attachment_id: "018f0000-0000-7000-8000-000000000099".to_owned(),
            manifest_sha256: "b".repeat(64),
            confirmation: AdvisorContentConfirmationState::ConfirmedForSingleSend,
        };
        assert!(matches!(
            service.claim(&mismatch),
            Err(AdvisorDocumentAttachmentDiagnosticCode::ManifestMismatch)
        ));
        assert_eq!(
            service.snapshot().state,
            AdvisorDocumentAttachmentState::Empty,
            "a mismatched claim consumes and disposes the native-only staging"
        );

        *service.pending.lock().unwrap() = Some(pending_document());
        let request = AdvisorDocumentAttachmentClaimRequest {
            attachment_id: "018f0000-0000-7000-8000-000000000099".to_owned(),
            manifest_sha256: "a".repeat(64),
            confirmation: AdvisorContentConfirmationState::ConfirmedForSingleSend,
        };
        let claimed = service.claim(&request).unwrap();
        assert_eq!(claimed.projection_text, "safe");
        assert!(matches!(
            service.claim(&request),
            Err(AdvisorDocumentAttachmentDiagnosticCode::AttachmentNotFound)
        ));

        *service.pending.lock().unwrap() = Some(pending_document());
        let unconfirmed = AdvisorDocumentAttachmentClaimRequest {
            confirmation: AdvisorContentConfirmationState::ConfirmationRequired,
            ..request.clone()
        };
        assert!(matches!(
            service.claim(&unconfirmed),
            Err(AdvisorDocumentAttachmentDiagnosticCode::InvalidRequest)
        ));
        assert_eq!(
            service.snapshot().state,
            AdvisorDocumentAttachmentState::Ready
        );
        assert_eq!(service.clear().state, AdvisorDocumentAttachmentState::Empty);
        assert!(matches!(
            service.claim(&request),
            Err(AdvisorDocumentAttachmentDiagnosticCode::AttachmentNotFound)
        ));
    }

    #[test]
    fn expired_and_cleared_documents_are_never_recovered() {
        let service = AdvisorDocumentAttachmentService {
            pending: Mutex::new(Some(PendingDocument {
                created_at: Instant::now() - ATTACHMENT_TTL - Duration::from_secs(1),
                ..pending_document()
            })),
        };
        assert_eq!(
            service.snapshot().diagnostic_code,
            Some(AdvisorDocumentAttachmentDiagnosticCode::AttachmentExpired)
        );
        assert_eq!(service.clear().state, AdvisorDocumentAttachmentState::Empty);
        assert_eq!(
            service.snapshot().state,
            AdvisorDocumentAttachmentState::Empty
        );
    }
}
