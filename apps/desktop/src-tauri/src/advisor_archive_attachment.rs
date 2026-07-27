//! One-use, metadata-only ZIP manifests for Advisor.
//!
//! M35 intentionally accepts only ZIP metadata. No entry is extracted,
//! decompressed, opened, or transported; the parser is delegated only for ZIP
//! container/central-directory interpretation.

use std::{
    collections::HashSet,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom},
    os::{
        fd::AsRawFd,
        unix::fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use zip::ZipArchive;

use crate::advisor_attachment::{
    AdvisorContentCategory, AdvisorContentConfirmationState, AdvisorContentDisposal,
};

pub const MAX_ADVISOR_ARCHIVE_BYTES: usize = 32 * 1024 * 1024;
const MAX_ENTRIES_INSPECTED: usize = 10_000;
const MAX_ENTRIES_INCLUDED: usize = 2_000;
const MAX_ENTRY_NAME_BYTES: usize = 512;
const MAX_PATH_COMPONENTS: usize = 32;
const MAX_MANIFEST_BYTES: usize = 256 * 1024;
const MAX_ENTRY_DECLARED_BYTES: u64 = 64 * 1024 * 1024;
const MAX_AGGREGATE_DECLARED_BYTES: u64 = 256 * 1024 * 1024;
const MAX_DECLARED_COMPRESSION_RATIO: u64 = 100;
const ATTACHMENT_TTL: Duration = Duration::from_secs(15 * 60);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorArchiveMediaType {
    Zip,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorArchiveProjectionKind {
    ArchiveManifestV1,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorArchiveEntryKind {
    File,
    Directory,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorArchiveAttachmentState {
    Empty,
    Ready,
    Unavailable,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorArchiveAttachmentDiagnosticCode {
    InvalidRequest,
    UnsupportedType,
    InvalidSignature,
    SourceTooLarge,
    SourceUnavailable,
    SourceChanged,
    EncryptedArchive,
    MalformedOrUnsupportedArchive,
    EntryLimitExceeded,
    ManifestSizeLimitExceeded,
    ExpandedSizeLimitExceeded,
    CompressionRatioLimitExceeded,
    UnsafeEntryPath,
    DuplicateEntry,
    SymlinkEntry,
    UnsupportedEntryKind,
    UnsafeName,
    ReadFailed,
    AttachmentNotFound,
    AttachmentExpired,
    ManifestMismatch,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorArchiveManifestEntry {
    pub name: String,
    pub kind: AdvisorArchiveEntryKind,
    pub compressed_size: u64,
    pub declared_uncompressed_size: u64,
    pub nested_archive_like: bool,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorArchiveProjection {
    pub kind: AdvisorArchiveProjectionKind,
    pub schema_version: u16,
    pub discovered_entry_count: u32,
    pub included_entry_count: u32,
    pub omitted_entry_count: u32,
    pub declared_aggregate_uncompressed_bytes: u64,
    pub manifest_byte_size: u32,
    pub truncated: bool,
    pub warnings: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorArchiveAttachmentManifest {
    pub attachment_id: String,
    pub display_name: String,
    pub content_category: AdvisorContentCategory,
    pub media_type: AdvisorArchiveMediaType,
    pub byte_size: u64,
    pub sha256: String,
    pub projection: AdvisorArchiveProjection,
    pub disposal: AdvisorContentDisposal,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorArchiveAttachmentSnapshot {
    pub schema_version: u16,
    pub state: AdvisorArchiveAttachmentState,
    pub attachment: Option<AdvisorArchiveAttachmentManifest>,
    pub entries: Vec<AdvisorArchiveManifestEntry>,
    pub confirmation_state: Option<AdvisorContentConfirmationState>,
    pub diagnostic_code: Option<AdvisorArchiveAttachmentDiagnosticCode>,
}
impl AdvisorArchiveAttachmentSnapshot {
    pub fn empty() -> Self {
        Self {
            schema_version: 1,
            state: AdvisorArchiveAttachmentState::Empty,
            attachment: None,
            entries: Vec::new(),
            confirmation_state: None,
            diagnostic_code: None,
        }
    }
    fn unavailable(code: AdvisorArchiveAttachmentDiagnosticCode) -> Self {
        Self {
            schema_version: 1,
            state: AdvisorArchiveAttachmentState::Unavailable,
            attachment: None,
            entries: Vec::new(),
            confirmation_state: None,
            diagnostic_code: Some(code),
        }
    }
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorArchiveAttachmentClaimRequest {
    pub attachment_id: String,
    pub manifest_sha256: String,
    pub confirmation: AdvisorContentConfirmationState,
}
pub(crate) struct ClaimedAdvisorArchiveAttachment {
    pub(crate) manifest: AdvisorArchiveAttachmentManifest,
    pub(crate) projection_text: String,
}
struct PendingArchive {
    manifest: AdvisorArchiveAttachmentManifest,
    entries: Vec<AdvisorArchiveManifestEntry>,
    projection_text: String,
    created_at: Instant,
}
#[derive(Default)]
pub struct AdvisorArchiveAttachmentService {
    pending: Mutex<Option<PendingArchive>>,
}

impl AdvisorArchiveAttachmentService {
    pub fn snapshot(&self) -> AdvisorArchiveAttachmentSnapshot {
        let Ok(mut pending) = self.pending.lock() else {
            return AdvisorArchiveAttachmentSnapshot::unavailable(
                AdvisorArchiveAttachmentDiagnosticCode::ReadFailed,
            );
        };
        if pending
            .as_ref()
            .is_some_and(|item| item.created_at.elapsed() > ATTACHMENT_TTL)
        {
            *pending = None;
            return AdvisorArchiveAttachmentSnapshot::unavailable(
                AdvisorArchiveAttachmentDiagnosticCode::AttachmentExpired,
            );
        }
        pending
            .as_ref()
            .map_or_else(AdvisorArchiveAttachmentSnapshot::empty, |item| {
                AdvisorArchiveAttachmentSnapshot {
                    schema_version: 1,
                    state: AdvisorArchiveAttachmentState::Ready,
                    attachment: Some(item.manifest.clone()),
                    entries: item.entries.clone(),
                    confirmation_state: Some(AdvisorContentConfirmationState::ConfirmationRequired),
                    diagnostic_code: None,
                }
            })
    }
    pub fn stage_path(&self, path: PathBuf) -> AdvisorArchiveAttachmentSnapshot {
        match prepare(&path) {
            Ok((manifest, entries, projection_text)) => match self.pending.lock() {
                Ok(mut pending) => {
                    *pending = Some(PendingArchive {
                        manifest: manifest.clone(),
                        entries: entries.clone(),
                        projection_text,
                        created_at: Instant::now(),
                    });
                    AdvisorArchiveAttachmentSnapshot {
                        schema_version: 1,
                        state: AdvisorArchiveAttachmentState::Ready,
                        attachment: Some(manifest),
                        entries,
                        confirmation_state: Some(
                            AdvisorContentConfirmationState::ConfirmationRequired,
                        ),
                        diagnostic_code: None,
                    }
                }
                Err(_) => AdvisorArchiveAttachmentSnapshot::unavailable(
                    AdvisorArchiveAttachmentDiagnosticCode::ReadFailed,
                ),
            },
            Err(code) => AdvisorArchiveAttachmentSnapshot::unavailable(code),
        }
    }
    pub fn clear(&self) -> AdvisorArchiveAttachmentSnapshot {
        match self.pending.lock() {
            Ok(mut pending) => {
                *pending = None;
                AdvisorArchiveAttachmentSnapshot::empty()
            }
            Err(_) => AdvisorArchiveAttachmentSnapshot::unavailable(
                AdvisorArchiveAttachmentDiagnosticCode::ReadFailed,
            ),
        }
    }
    pub fn claim(
        &self,
        request: &AdvisorArchiveAttachmentClaimRequest,
    ) -> Result<ClaimedAdvisorArchiveAttachment, AdvisorArchiveAttachmentDiagnosticCode> {
        if !valid_uuid_v7(&request.attachment_id)
            || !valid_hash(&request.manifest_sha256)
            || request.confirmation != AdvisorContentConfirmationState::ConfirmedForSingleSend
        {
            return Err(AdvisorArchiveAttachmentDiagnosticCode::InvalidRequest);
        }
        let mut pending = self
            .pending
            .lock()
            .map_err(|_| AdvisorArchiveAttachmentDiagnosticCode::ReadFailed)?;
        let Some(item) = pending.take() else {
            return Err(AdvisorArchiveAttachmentDiagnosticCode::AttachmentNotFound);
        };
        if item.created_at.elapsed() > ATTACHMENT_TTL {
            return Err(AdvisorArchiveAttachmentDiagnosticCode::AttachmentExpired);
        }
        if item.manifest.attachment_id != request.attachment_id
            || item.manifest.sha256 != request.manifest_sha256
        {
            return Err(AdvisorArchiveAttachmentDiagnosticCode::ManifestMismatch);
        }
        Ok(ClaimedAdvisorArchiveAttachment {
            manifest: item.manifest,
            projection_text: item.projection_text,
        })
    }
}

fn prepare(
    path: &Path,
) -> Result<
    (
        AdvisorArchiveAttachmentManifest,
        Vec<AdvisorArchiveManifestEntry>,
        String,
    ),
    AdvisorArchiveAttachmentDiagnosticCode,
> {
    if !path.is_absolute() {
        return Err(AdvisorArchiveAttachmentDiagnosticCode::InvalidRequest);
    }
    let selected = path
        .symlink_metadata()
        .map_err(|_| AdvisorArchiveAttachmentDiagnosticCode::SourceUnavailable)?;
    if selected.file_type().is_symlink() || !selected.is_file() {
        return Err(AdvisorArchiveAttachmentDiagnosticCode::SourceUnavailable);
    }
    if selected.len() > MAX_ADVISOR_ARCHIVE_BYTES as u64 {
        return Err(AdvisorArchiveAttachmentDiagnosticCode::SourceTooLarge);
    }
    let display_name = validate_display_name(
        path.file_name()
            .and_then(|name| name.to_str())
            .ok_or(AdvisorArchiveAttachmentDiagnosticCode::UnsafeName)?,
    )?;
    if !display_name.to_ascii_lowercase().ends_with(".zip") {
        return Err(AdvisorArchiveAttachmentDiagnosticCode::UnsupportedType);
    }
    let resolved = path
        .canonicalize()
        .map_err(|_| AdvisorArchiveAttachmentDiagnosticCode::SourceUnavailable)?;
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(&resolved)
        .map_err(|_| AdvisorArchiveAttachmentDiagnosticCode::SourceUnavailable)?;
    let opened = file
        .metadata()
        .map_err(|_| AdvisorArchiveAttachmentDiagnosticCode::ReadFailed)?;
    if !opened.is_file()
        || opened.len() != selected.len()
        || opened.dev() != selected.dev()
        || opened.ino() != selected.ino()
        || descriptor_path(&file)? != resolved
    {
        return Err(AdvisorArchiveAttachmentDiagnosticCode::SourceChanged);
    }
    let mut signature = [0_u8; 4];
    file.read_exact(&mut signature)
        .map_err(|_| AdvisorArchiveAttachmentDiagnosticCode::InvalidSignature)?;
    if !matches!(signature, [b'P', b'K', 3 | 5 | 7, 4 | 6 | 8]) {
        return Err(AdvisorArchiveAttachmentDiagnosticCode::InvalidSignature);
    }
    file.seek(SeekFrom::Start(0))
        .map_err(|_| AdvisorArchiveAttachmentDiagnosticCode::ReadFailed)?;
    let mut bytes = Vec::with_capacity(opened.len() as usize);
    (&mut file)
        .take(MAX_ADVISOR_ARCHIVE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| AdvisorArchiveAttachmentDiagnosticCode::ReadFailed)?;
    if bytes.len() as u64 != opened.len() {
        return Err(AdvisorArchiveAttachmentDiagnosticCode::SourceChanged);
    }
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    let mut archive = ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|_| AdvisorArchiveAttachmentDiagnosticCode::MalformedOrUnsupportedArchive)?;
    if archive.len() > MAX_ENTRIES_INSPECTED {
        return Err(AdvisorArchiveAttachmentDiagnosticCode::EntryLimitExceeded);
    }
    let mut entries = Vec::with_capacity(archive.len().min(MAX_ENTRIES_INCLUDED));
    let mut seen = HashSet::new();
    let mut declared_total = 0_u64;
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|_| AdvisorArchiveAttachmentDiagnosticCode::MalformedOrUnsupportedArchive)?;
        if entry.encrypted() {
            return Err(AdvisorArchiveAttachmentDiagnosticCode::EncryptedArchive);
        }
        let kind = entry_kind(&entry)?;
        let name = validate_entry_name(entry.name_raw())?;
        let normalized_key = name.to_ascii_lowercase();
        if !seen.insert(normalized_key) {
            return Err(AdvisorArchiveAttachmentDiagnosticCode::DuplicateEntry);
        }
        let declared = entry.size();
        let compressed = entry.compressed_size();
        if declared > MAX_ENTRY_DECLARED_BYTES {
            return Err(AdvisorArchiveAttachmentDiagnosticCode::ExpandedSizeLimitExceeded);
        }
        declared_total = declared_total
            .checked_add(declared)
            .ok_or(AdvisorArchiveAttachmentDiagnosticCode::ExpandedSizeLimitExceeded)?;
        if declared_total > MAX_AGGREGATE_DECLARED_BYTES {
            return Err(AdvisorArchiveAttachmentDiagnosticCode::ExpandedSizeLimitExceeded);
        }
        if declared > 0
            && (compressed == 0
                || declared.saturating_div(compressed.max(1)) > MAX_DECLARED_COMPRESSION_RATIO)
        {
            return Err(AdvisorArchiveAttachmentDiagnosticCode::CompressionRatioLimitExceeded);
        }
        if index < MAX_ENTRIES_INCLUDED {
            entries.push(AdvisorArchiveManifestEntry {
                nested_archive_like: looks_like_archive(&name),
                name,
                kind,
                compressed_size: compressed,
                declared_uncompressed_size: declared,
            });
        }
    }
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    let mut projected_entries = Vec::with_capacity(entries.len());
    let mut projection_text = format!("archive-manifest-v1\narchive-type: zip\nentry-count: {}\nincluded-entry-count: PLACEHOLDER\nomitted-entry-count: PLACEHOLDER\ndeclared-aggregate-uncompressed-bytes: {}\n", archive.len(), declared_total);
    let header_bytes = projection_text.len();
    let mut truncated = archive.len() > entries.len();
    for entry in &entries {
        let row = format!(
            "{}\t{:?}\t{}\t{}{}\n",
            entry.name,
            entry.kind,
            entry.compressed_size,
            entry.declared_uncompressed_size,
            if entry.nested_archive_like {
                "\tnested-archive-like"
            } else {
                ""
            }
        );
        if projection_text.len().saturating_add(row.len()) > MAX_MANIFEST_BYTES {
            truncated = true;
            break;
        }
        projection_text.push_str(&row);
        projected_entries.push(entry.clone());
    }
    let omitted = archive.len().saturating_sub(projected_entries.len());
    projection_text = format!("archive-manifest-v1\narchive-type: zip\nentry-count: {}\nincluded-entry-count: {}\nomitted-entry-count: {}\ndeclared-aggregate-uncompressed-bytes: {}\n", archive.len(), projected_entries.len(), omitted, declared_total)
        + &projection_text[header_bytes..];
    if projection_text.len() > MAX_MANIFEST_BYTES {
        return Err(AdvisorArchiveAttachmentDiagnosticCode::ManifestSizeLimitExceeded);
    }
    let warnings = if truncated {
        vec!["manifest-truncated".to_owned()]
    } else {
        Vec::new()
    };
    let manifest = AdvisorArchiveAttachmentManifest {
        attachment_id: Uuid::now_v7().to_string(),
        display_name,
        content_category: AdvisorContentCategory::Archive,
        media_type: AdvisorArchiveMediaType::Zip,
        byte_size: opened.len(),
        sha256,
        projection: AdvisorArchiveProjection {
            kind: AdvisorArchiveProjectionKind::ArchiveManifestV1,
            schema_version: 1,
            discovered_entry_count: archive.len() as u32,
            included_entry_count: projected_entries.len() as u32,
            omitted_entry_count: omitted as u32,
            declared_aggregate_uncompressed_bytes: declared_total,
            manifest_byte_size: projection_text.len() as u32,
            truncated,
            warnings,
        },
        disposal: AdvisorContentDisposal::TransientMemoryOneSend,
    };
    Ok((manifest, projected_entries, projection_text))
}

fn entry_kind(
    entry: &zip::read::ZipFile<'_, std::io::Cursor<Vec<u8>>>,
) -> Result<AdvisorArchiveEntryKind, AdvisorArchiveAttachmentDiagnosticCode> {
    if let Some(mode) = entry.unix_mode() {
        match mode & 0o170000 {
            0 | 0o100000 => {}
            0o040000 => return Ok(AdvisorArchiveEntryKind::Directory),
            0o120000 => return Err(AdvisorArchiveAttachmentDiagnosticCode::SymlinkEntry),
            _ => return Err(AdvisorArchiveAttachmentDiagnosticCode::UnsupportedEntryKind),
        }
    }
    if entry.is_dir() {
        Ok(AdvisorArchiveEntryKind::Directory)
    } else if entry.is_file() {
        Ok(AdvisorArchiveEntryKind::File)
    } else {
        Err(AdvisorArchiveAttachmentDiagnosticCode::UnsupportedEntryKind)
    }
}
fn validate_entry_name(raw: &[u8]) -> Result<String, AdvisorArchiveAttachmentDiagnosticCode> {
    if raw.is_empty() || raw.len() > MAX_ENTRY_NAME_BYTES || !raw.is_ascii() {
        return Err(AdvisorArchiveAttachmentDiagnosticCode::UnsafeEntryPath);
    }
    let name = std::str::from_utf8(raw)
        .map_err(|_| AdvisorArchiveAttachmentDiagnosticCode::UnsafeEntryPath)?;
    if name.starts_with('/')
        || name.starts_with('\\')
        || name.contains('\\')
        || name.contains('\0')
        || name.len() >= 2 && name.as_bytes()[1] == b':'
    {
        return Err(AdvisorArchiveAttachmentDiagnosticCode::UnsafeEntryPath);
    }
    let normalized = name.strip_suffix('/').unwrap_or(name);
    let components: Vec<_> = normalized.split('/').collect();
    if normalized.is_empty()
        || components.len() > MAX_PATH_COMPONENTS
        || components.iter().any(|part| {
            part.is_empty()
                || *part == "."
                || *part == ".."
                || part.ends_with(' ')
                || part.ends_with('.')
                || part.chars().any(|character| character.is_control())
        })
    {
        return Err(AdvisorArchiveAttachmentDiagnosticCode::UnsafeEntryPath);
    }
    Ok(normalized.to_owned())
}
fn looks_like_archive(name: &str) -> bool {
    [".zip", ".tar", ".tgz", ".gz", ".bz2", ".xz", ".7z", ".rar"]
        .iter()
        .any(|suffix| name.to_ascii_lowercase().ends_with(suffix))
}
fn validate_display_name(value: &str) -> Result<String, AdvisorArchiveAttachmentDiagnosticCode> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.len() > 255
        || value.contains(['/', '\\'])
        || value.chars().any(|c| {
            c.is_control() || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
    {
        Err(AdvisorArchiveAttachmentDiagnosticCode::UnsafeName)
    } else {
        Ok(value.to_owned())
    }
}
fn descriptor_path(file: &File) -> Result<PathBuf, AdvisorArchiveAttachmentDiagnosticCode> {
    PathBuf::from("/proc/self/fd")
        .join(file.as_raw_fd().to_string())
        .canonicalize()
        .map_err(|_| AdvisorArchiveAttachmentDiagnosticCode::ReadFailed)
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
    use std::io::Write;
    fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let cursor = std::io::Cursor::new(Vec::new());
        let mut writer = zip::ZipWriter::new(cursor);
        for (name, bytes) in entries {
            writer
                .start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }
    #[test]
    fn stages_a_zip_manifest_once_without_contents_or_paths() {
        let dir = std::env::temp_dir().join(Uuid::now_v7().to_string());
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bundle.zip");
        std::fs::write(
            &path,
            zip_bytes(&[
                ("docs/readme.txt", b"secret"),
                ("nested.zip", b"not opened"),
            ]),
        )
        .unwrap();
        let service = AdvisorArchiveAttachmentService::default();
        let snapshot = service.stage_path(path);
        let attachment = snapshot.attachment.clone().unwrap();
        assert_eq!(attachment.content_category, AdvisorContentCategory::Archive);
        assert_eq!(snapshot.entries.len(), 2);
        assert!(!snapshot
            .entries
            .iter()
            .any(|entry| entry.name.contains("secret")));
        let claim = service
            .claim(&AdvisorArchiveAttachmentClaimRequest {
                attachment_id: attachment.attachment_id,
                manifest_sha256: attachment.sha256,
                confirmation: AdvisorContentConfirmationState::ConfirmedForSingleSend,
            })
            .unwrap();
        assert!(claim.projection_text.contains("docs/readme.txt"));
        assert!(!claim.projection_text.contains("secret"));
        assert_eq!(
            service.snapshot().state,
            AdvisorArchiveAttachmentState::Empty
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn rejects_traversal_and_duplicate_names() {
        let dir = std::env::temp_dir().join(Uuid::now_v7().to_string());
        std::fs::create_dir_all(&dir).unwrap();
        for (name, bytes) in [
            ("traversal.zip", zip_bytes(&[("../nope", b"x")])),
            (
                "duplicate.zip",
                zip_bytes(&[("same", b"x"), ("SAME", b"y")]),
            ),
        ] {
            let path = dir.join(name);
            std::fs::write(&path, bytes).unwrap();
            let result = AdvisorArchiveAttachmentService::default().stage_path(path);
            assert_eq!(result.state, AdvisorArchiveAttachmentState::Unavailable);
        }
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn rejects_non_zip_sources_and_oversized_inputs_without_staging() {
        let dir = std::env::temp_dir().join(Uuid::now_v7().to_string());
        std::fs::create_dir_all(&dir).unwrap();
        let invalid_extension = dir.join("bundle.tar");
        std::fs::write(&invalid_extension, zip_bytes(&[("safe.txt", b"x")])).unwrap();
        let invalid_signature = dir.join("bundle.zip");
        std::fs::write(&invalid_signature, b"not a zip").unwrap();
        let oversized = dir.join("oversized.zip");
        let file = std::fs::File::create(&oversized).unwrap();
        file.set_len(MAX_ADVISOR_ARCHIVE_BYTES as u64 + 1).unwrap();
        for (path, expected) in [
            (
                invalid_extension,
                AdvisorArchiveAttachmentDiagnosticCode::UnsupportedType,
            ),
            (
                invalid_signature,
                AdvisorArchiveAttachmentDiagnosticCode::InvalidSignature,
            ),
            (
                oversized,
                AdvisorArchiveAttachmentDiagnosticCode::SourceTooLarge,
            ),
        ] {
            let snapshot = AdvisorArchiveAttachmentService::default().stage_path(path);
            assert_eq!(snapshot.state, AdvisorArchiveAttachmentState::Unavailable);
            assert_eq!(snapshot.diagnostic_code, Some(expected));
            assert!(snapshot.attachment.is_none());
        }
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn confirmation_hash_binding_and_claim_are_one_use() {
        let dir = std::env::temp_dir().join(Uuid::now_v7().to_string());
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bundle.zip");
        std::fs::write(&path, zip_bytes(&[("safe.txt", b"x")])).unwrap();
        let service = AdvisorArchiveAttachmentService::default();
        let snapshot = service.stage_path(path);
        let manifest = snapshot.attachment.unwrap();
        let wrong = service.claim(&AdvisorArchiveAttachmentClaimRequest {
            attachment_id: manifest.attachment_id.clone(),
            manifest_sha256: "0".repeat(64),
            confirmation: AdvisorContentConfirmationState::ConfirmedForSingleSend,
        });
        assert_eq!(
            wrong.err(),
            Some(AdvisorArchiveAttachmentDiagnosticCode::ManifestMismatch)
        );
        assert_eq!(
            service.snapshot().state,
            AdvisorArchiveAttachmentState::Empty
        );
        let replacement = dir.join("replacement.zip");
        std::fs::write(&replacement, zip_bytes(&[("other.txt", b"y")])).unwrap();
        let snapshot = service.stage_path(replacement);
        let manifest = snapshot.attachment.unwrap();
        let request = AdvisorArchiveAttachmentClaimRequest {
            attachment_id: manifest.attachment_id,
            manifest_sha256: manifest.sha256,
            confirmation: AdvisorContentConfirmationState::ConfirmedForSingleSend,
        };
        assert!(service.claim(&request).is_ok());
        assert_eq!(
            service.claim(&request).err(),
            Some(AdvisorArchiveAttachmentDiagnosticCode::AttachmentNotFound)
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn rejects_selected_symlinks_without_following_them() {
        use std::os::unix::fs::symlink;
        let dir = std::env::temp_dir().join(Uuid::now_v7().to_string());
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("source.zip");
        std::fs::write(&source, zip_bytes(&[("safe.txt", b"x")])).unwrap();
        let selected = dir.join("selected.zip");
        symlink(&source, &selected).unwrap();
        let snapshot = AdvisorArchiveAttachmentService::default().stage_path(selected);
        assert_eq!(snapshot.state, AdvisorArchiveAttachmentState::Unavailable);
        assert_eq!(
            snapshot.diagnostic_code,
            Some(AdvisorArchiveAttachmentDiagnosticCode::SourceUnavailable)
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}
