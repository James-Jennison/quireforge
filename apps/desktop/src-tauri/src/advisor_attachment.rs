use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
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

pub const MAX_ADVISOR_TEXT_ATTACHMENT_BYTES: usize = 512 * 1024;
const ATTACHMENT_TTL: Duration = Duration::from_secs(15 * 60);

/// Registry categories are deliberately closed. F1 enables only TextData;
/// the remaining categories reserve distinct future review boundaries and do
/// not add readers, parsers, or transport capabilities.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorContentCategory {
    TextData,
    Image,
    Document,
    Archive,
    StaticBinary,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorContentType {
    Text,
    Markdown,
    Csv,
    Json,
    Python,
}

impl AdvisorContentType {
    fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_ascii_lowercase().as_str() {
            "txt" => Some(Self::Text),
            "md" => Some(Self::Markdown),
            "csv" => Some(Self::Csv),
            "json" => Some(Self::Json),
            "py" => Some(Self::Python),
            _ => None,
        }
    }
    fn extension(self) -> &'static str {
        match self {
            Self::Text => "txt",
            Self::Markdown => "md",
            Self::Csv => "csv",
            Self::Json => "json",
            Self::Python => "py",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorContentProjectionKind {
    NormalizedUtf8Text,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorContentDisposal {
    TransientMemoryOneSend,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorContentConfirmationState {
    ConfirmationRequired,
    ConfirmedForSingleSend,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorContentProjection {
    pub kind: AdvisorContentProjectionKind,
    pub normalized_byte_size: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorTextAttachmentState {
    Empty,
    Ready,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorTextAttachmentDiagnosticCode {
    InvalidRequest,
    UnsupportedType,
    FileTooLarge,
    InvalidContent,
    UnsafeName,
    ReadFailed,
    AttachmentNotFound,
    AttachmentExpired,
    ManifestMismatch,
    SaveCancelled,
    SaveFailed,
    FileExists,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorTextAttachmentManifest {
    pub attachment_id: String,
    pub display_name: String,
    pub content_category: AdvisorContentCategory,
    pub content_type: AdvisorContentType,
    pub byte_size: u64,
    pub sha256: String,
    pub projection: AdvisorContentProjection,
    pub disposal: AdvisorContentDisposal,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorTextAttachmentSnapshot {
    pub schema_version: u16,
    pub state: AdvisorTextAttachmentState,
    pub attachment: Option<AdvisorTextAttachmentManifest>,
    pub confirmation_state: Option<AdvisorContentConfirmationState>,
    pub diagnostic_code: Option<AdvisorTextAttachmentDiagnosticCode>,
}

impl AdvisorTextAttachmentSnapshot {
    pub fn empty() -> Self {
        Self {
            schema_version: 1,
            state: AdvisorTextAttachmentState::Empty,
            attachment: None,
            confirmation_state: None,
            diagnostic_code: None,
        }
    }
    fn unavailable(code: AdvisorTextAttachmentDiagnosticCode) -> Self {
        Self {
            schema_version: 1,
            state: AdvisorTextAttachmentState::Unavailable,
            attachment: None,
            confirmation_state: None,
            diagnostic_code: Some(code),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorTextAttachmentClaimRequest {
    pub attachment_id: String,
    pub manifest_sha256: String,
    pub confirmation: AdvisorContentConfirmationState,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorTextExportRequest {
    pub suggested_name: String,
    pub content_type: AdvisorContentType,
    pub content: String,
}

pub(crate) struct ClaimedAdvisorTextAttachment {
    pub manifest: AdvisorTextAttachmentManifest,
    pub text: String,
}

struct PendingAttachment {
    manifest: AdvisorTextAttachmentManifest,
    text: String,
    created_at: Instant,
}

#[derive(Default)]
pub struct AdvisorTextAttachmentService {
    pending: Mutex<Option<PendingAttachment>>,
}

impl AdvisorTextAttachmentService {
    pub fn snapshot(&self) -> AdvisorTextAttachmentSnapshot {
        let Ok(mut pending) = self.pending.lock() else {
            return AdvisorTextAttachmentSnapshot::unavailable(
                AdvisorTextAttachmentDiagnosticCode::ReadFailed,
            );
        };
        if pending
            .as_ref()
            .is_some_and(|item| item.created_at.elapsed() > ATTACHMENT_TTL)
        {
            *pending = None;
            return AdvisorTextAttachmentSnapshot::unavailable(
                AdvisorTextAttachmentDiagnosticCode::AttachmentExpired,
            );
        }
        match pending.as_ref() {
            Some(item) => AdvisorTextAttachmentSnapshot {
                schema_version: 1,
                state: AdvisorTextAttachmentState::Ready,
                attachment: Some(item.manifest.clone()),
                confirmation_state: Some(AdvisorContentConfirmationState::ConfirmationRequired),
                diagnostic_code: None,
            },
            None => AdvisorTextAttachmentSnapshot::empty(),
        }
    }

    pub fn stage_path(&self, path: PathBuf) -> AdvisorTextAttachmentSnapshot {
        match prepare(path.as_path()) {
            Ok((manifest, text)) => match self.pending.lock() {
                Ok(mut pending) => {
                    *pending = Some(PendingAttachment {
                        manifest: manifest.clone(),
                        text,
                        created_at: Instant::now(),
                    });
                    AdvisorTextAttachmentSnapshot {
                        schema_version: 1,
                        state: AdvisorTextAttachmentState::Ready,
                        attachment: Some(manifest),
                        confirmation_state: Some(
                            AdvisorContentConfirmationState::ConfirmationRequired,
                        ),
                        diagnostic_code: None,
                    }
                }
                Err(_) => AdvisorTextAttachmentSnapshot::unavailable(
                    AdvisorTextAttachmentDiagnosticCode::ReadFailed,
                ),
            },
            Err(code) => AdvisorTextAttachmentSnapshot::unavailable(code),
        }
    }

    pub fn clear(&self) -> AdvisorTextAttachmentSnapshot {
        match self.pending.lock() {
            Ok(mut pending) => {
                *pending = None;
                AdvisorTextAttachmentSnapshot::empty()
            }
            Err(_) => AdvisorTextAttachmentSnapshot::unavailable(
                AdvisorTextAttachmentDiagnosticCode::ReadFailed,
            ),
        }
    }

    pub fn claim(
        &self,
        request: &AdvisorTextAttachmentClaimRequest,
    ) -> Result<ClaimedAdvisorTextAttachment, AdvisorTextAttachmentDiagnosticCode> {
        if !valid_uuid_v7(&request.attachment_id)
            || !valid_hash(&request.manifest_sha256)
            || request.confirmation != AdvisorContentConfirmationState::ConfirmedForSingleSend
        {
            return Err(AdvisorTextAttachmentDiagnosticCode::InvalidRequest);
        }
        let mut pending = self
            .pending
            .lock()
            .map_err(|_| AdvisorTextAttachmentDiagnosticCode::ReadFailed)?;
        let Some(item) = pending.take() else {
            return Err(AdvisorTextAttachmentDiagnosticCode::AttachmentNotFound);
        };
        if item.created_at.elapsed() > ATTACHMENT_TTL {
            return Err(AdvisorTextAttachmentDiagnosticCode::AttachmentExpired);
        }
        if item.manifest.attachment_id != request.attachment_id
            || item.manifest.sha256 != request.manifest_sha256
        {
            return Err(AdvisorTextAttachmentDiagnosticCode::ManifestMismatch);
        }
        Ok(ClaimedAdvisorTextAttachment {
            manifest: item.manifest,
            text: item.text,
        })
    }
}

pub fn save_export(
    path: PathBuf,
    request: &AdvisorTextExportRequest,
) -> Result<(), AdvisorTextAttachmentDiagnosticCode> {
    let name = validate_display_name(&request.suggested_name)?;
    if !valid_export_content(&request.content) {
        return Err(AdvisorTextAttachmentDiagnosticCode::InvalidContent);
    }
    let expected_suffix = format!(".{}", request.content_type.extension());
    if !name.ends_with(&expected_suffix) {
        return Err(AdvisorTextAttachmentDiagnosticCode::InvalidRequest);
    }
    if !path.is_absolute() || path.file_name().and_then(|part| part.to_str()) != Some(name.as_str())
    {
        return Err(AdvisorTextAttachmentDiagnosticCode::InvalidRequest);
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(path)
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                AdvisorTextAttachmentDiagnosticCode::FileExists
            } else {
                AdvisorTextAttachmentDiagnosticCode::SaveFailed
            }
        })?;
    file.write_all(request.content.as_bytes())
        .map_err(|_| AdvisorTextAttachmentDiagnosticCode::SaveFailed)?;
    file.sync_all()
        .map_err(|_| AdvisorTextAttachmentDiagnosticCode::SaveFailed)
}

fn prepare(
    path: &Path,
) -> Result<(AdvisorTextAttachmentManifest, String), AdvisorTextAttachmentDiagnosticCode> {
    if !path.is_absolute() {
        return Err(AdvisorTextAttachmentDiagnosticCode::InvalidRequest);
    }
    let selected = path
        .symlink_metadata()
        .map_err(|_| AdvisorTextAttachmentDiagnosticCode::ReadFailed)?;
    if selected.file_type().is_symlink() || !selected.is_file() {
        return Err(AdvisorTextAttachmentDiagnosticCode::ReadFailed);
    }
    if selected.len() > MAX_ADVISOR_TEXT_ATTACHMENT_BYTES as u64 {
        return Err(AdvisorTextAttachmentDiagnosticCode::FileTooLarge);
    }
    let display_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(AdvisorTextAttachmentDiagnosticCode::UnsafeName)
        .and_then(validate_display_name)?;
    let extension = Path::new(&display_name)
        .extension()
        .and_then(|part| part.to_str())
        .ok_or(AdvisorTextAttachmentDiagnosticCode::UnsupportedType)?;
    let content_type = AdvisorContentType::from_extension(extension)
        .ok_or(AdvisorTextAttachmentDiagnosticCode::UnsupportedType)?;
    let resolved = path
        .canonicalize()
        .map_err(|_| AdvisorTextAttachmentDiagnosticCode::ReadFailed)?;
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(&resolved)
        .map_err(|_| AdvisorTextAttachmentDiagnosticCode::ReadFailed)?;
    let opened = file
        .metadata()
        .map_err(|_| AdvisorTextAttachmentDiagnosticCode::ReadFailed)?;
    if !opened.is_file()
        || opened.len() != selected.len()
        || opened.dev() != selected.dev()
        || opened.ino() != selected.ino()
        || descriptor_path(&file)? != resolved
    {
        return Err(AdvisorTextAttachmentDiagnosticCode::ReadFailed);
    }
    let mut bytes = Vec::with_capacity(opened.len() as usize);
    (&mut file)
        .take(MAX_ADVISOR_TEXT_ATTACHMENT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| AdvisorTextAttachmentDiagnosticCode::ReadFailed)?;
    if bytes.len() as u64 != opened.len() {
        return Err(AdvisorTextAttachmentDiagnosticCode::ReadFailed);
    }
    let text = String::from_utf8(bytes)
        .map_err(|_| AdvisorTextAttachmentDiagnosticCode::InvalidContent)?
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    if !valid_export_content(&text) {
        return Err(AdvisorTextAttachmentDiagnosticCode::InvalidContent);
    }
    let normalized = text.as_bytes();
    if normalized.len() > MAX_ADVISOR_TEXT_ATTACHMENT_BYTES {
        return Err(AdvisorTextAttachmentDiagnosticCode::FileTooLarge);
    }
    let normalized_byte_size = normalized.len() as u64;
    let manifest = AdvisorTextAttachmentManifest {
        attachment_id: Uuid::now_v7().to_string(),
        display_name,
        content_category: AdvisorContentCategory::TextData,
        content_type,
        byte_size: normalized_byte_size,
        sha256: format!("{:x}", Sha256::digest(normalized)),
        projection: AdvisorContentProjection {
            kind: AdvisorContentProjectionKind::NormalizedUtf8Text,
            normalized_byte_size,
        },
        disposal: AdvisorContentDisposal::TransientMemoryOneSend,
    };
    Ok((manifest, text))
}

fn valid_export_content(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_ADVISOR_TEXT_ATTACHMENT_BYTES && !value.contains('\0')
}
fn validate_display_name(value: &str) -> Result<String, AdvisorTextAttachmentDiagnosticCode> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.len() > 255
        || value.contains(['/', '\\'])
        || value.chars().any(|c| {
            c.is_control() || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
    {
        return Err(AdvisorTextAttachmentDiagnosticCode::UnsafeName);
    }
    Ok(value.to_owned())
}
fn descriptor_path(file: &File) -> Result<PathBuf, AdvisorTextAttachmentDiagnosticCode> {
    PathBuf::from("/proc/self/fd")
        .join(file.as_raw_fd().to_string())
        .canonicalize()
        .map_err(|_| AdvisorTextAttachmentDiagnosticCode::ReadFailed)
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
    #[test]
    fn only_supported_normalized_text_can_be_claimed_once() {
        let directory = std::env::temp_dir().join(Uuid::now_v7().to_string());
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("note.md");
        std::fs::write(&path, "hello\r\nworld").unwrap();
        let service = AdvisorTextAttachmentService::default();
        let snapshot = service.stage_path(path);
        let manifest = snapshot.attachment.unwrap();
        assert_eq!(manifest.content_category, AdvisorContentCategory::TextData);
        assert_eq!(
            manifest.projection.kind,
            AdvisorContentProjectionKind::NormalizedUtf8Text
        );
        assert_eq!(
            manifest.disposal,
            AdvisorContentDisposal::TransientMemoryOneSend
        );
        assert_eq!(
            snapshot.confirmation_state,
            Some(AdvisorContentConfirmationState::ConfirmationRequired)
        );
        let claimed = service
            .claim(&AdvisorTextAttachmentClaimRequest {
                attachment_id: manifest.attachment_id.clone(),
                manifest_sha256: manifest.sha256.clone(),
                confirmation: AdvisorContentConfirmationState::ConfirmedForSingleSend,
            })
            .unwrap();
        assert_eq!(claimed.text, "hello\nworld");
        assert_eq!(service.snapshot().state, AdvisorTextAttachmentState::Empty);
        std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn export_never_overwrites_existing_file() {
        let directory = std::env::temp_dir().join(Uuid::now_v7().to_string());
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("answer.txt");
        let request = AdvisorTextExportRequest {
            suggested_name: "answer.txt".into(),
            content_type: AdvisorContentType::Text,
            content: "answer".into(),
        };
        save_export(path.clone(), &request).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "answer");
        assert_eq!(
            save_export(path, &request),
            Err(AdvisorTextAttachmentDiagnosticCode::FileExists)
        );
        std::fs::remove_dir_all(directory).unwrap();
    }
}
