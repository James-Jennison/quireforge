//! One-use, memory-backed Advisor image input.
//!
//! The picker path is consumed in native code. React receives only a safe
//! manifest; the app-server receives only a temporary `/proc/<pid>/fd` path to
//! a sealed, memory-backed descriptor while a turn is active.

use std::{
    ffi::CString,
    fs::{File, OpenOptions},
    io::{Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant},
};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    advisor_attachment::{
        AdvisorContentCategory, AdvisorContentConfirmationState, AdvisorContentDisposal,
    },
    preview::{types::FilePreviewDiagnosticCode, validate_attachment_image},
};

pub const MAX_ADVISOR_IMAGE_BYTES: usize = 4 * 1024 * 1024;
const ATTACHMENT_TTL: Duration = Duration::from_secs(15 * 60);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorImageMediaType {
    Png,
    Jpeg,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorImageProjectionKind {
    LocalImage,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorImageProjection {
    pub kind: AdvisorImageProjectionKind,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorImageAttachmentState {
    Empty,
    Ready,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorImageAttachmentDiagnosticCode {
    InvalidRequest,
    UnsupportedType,
    FileTooLarge,
    InvalidContent,
    UnsafeName,
    ReadFailed,
    AttachmentNotFound,
    AttachmentExpired,
    ManifestMismatch,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorImageAttachmentManifest {
    pub attachment_id: String,
    pub display_name: String,
    pub content_category: AdvisorContentCategory,
    pub media_type: AdvisorImageMediaType,
    pub byte_size: u64,
    pub width: u32,
    pub height: u32,
    pub sha256: String,
    pub projection: AdvisorImageProjection,
    pub disposal: AdvisorContentDisposal,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorImageAttachmentSnapshot {
    pub schema_version: u16,
    pub state: AdvisorImageAttachmentState,
    pub attachment: Option<AdvisorImageAttachmentManifest>,
    pub preview_data_url: Option<String>,
    pub confirmation_state: Option<AdvisorContentConfirmationState>,
    pub diagnostic_code: Option<AdvisorImageAttachmentDiagnosticCode>,
}

impl AdvisorImageAttachmentSnapshot {
    pub fn empty() -> Self {
        Self {
            schema_version: 1,
            state: AdvisorImageAttachmentState::Empty,
            attachment: None,
            preview_data_url: None,
            confirmation_state: None,
            diagnostic_code: None,
        }
    }
    fn unavailable(code: AdvisorImageAttachmentDiagnosticCode) -> Self {
        Self {
            schema_version: 1,
            state: AdvisorImageAttachmentState::Unavailable,
            attachment: None,
            preview_data_url: None,
            confirmation_state: None,
            diagnostic_code: Some(code),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorImageAttachmentClaimRequest {
    pub attachment_id: String,
    pub manifest_sha256: String,
    pub confirmation: AdvisorContentConfirmationState,
}

pub(crate) struct ClaimedAdvisorImageAttachment {
    file: File,
}
impl ClaimedAdvisorImageAttachment {
    pub(crate) fn protocol_input(&self) -> serde_json::Value {
        // The descriptor is memory-backed and only readable by the current user
        // while this process holds it. The selected source path is never reused.
        serde_json::json!({"type": "localImage", "path": format!("/proc/{}/fd/{}", std::process::id(), self.file.as_raw_fd())})
    }
}

struct PendingImage {
    manifest: AdvisorImageAttachmentManifest,
    preview_data_url: String,
    bytes: Vec<u8>,
    created_at: Instant,
}
#[derive(Default)]
pub struct AdvisorImageAttachmentService {
    pending: Mutex<Option<PendingImage>>,
}

impl AdvisorImageAttachmentService {
    pub fn snapshot(&self) -> AdvisorImageAttachmentSnapshot {
        let Ok(mut pending) = self.pending.lock() else {
            return AdvisorImageAttachmentSnapshot::unavailable(
                AdvisorImageAttachmentDiagnosticCode::ReadFailed,
            );
        };
        if pending
            .as_ref()
            .is_some_and(|item| item.created_at.elapsed() > ATTACHMENT_TTL)
        {
            *pending = None;
            return AdvisorImageAttachmentSnapshot::unavailable(
                AdvisorImageAttachmentDiagnosticCode::AttachmentExpired,
            );
        }
        pending
            .as_ref()
            .map_or_else(AdvisorImageAttachmentSnapshot::empty, |item| {
                AdvisorImageAttachmentSnapshot {
                    schema_version: 1,
                    state: AdvisorImageAttachmentState::Ready,
                    attachment: Some(item.manifest.clone()),
                    preview_data_url: Some(item.preview_data_url.clone()),
                    confirmation_state: Some(AdvisorContentConfirmationState::ConfirmationRequired),
                    diagnostic_code: None,
                }
            })
    }

    pub fn stage_path(&self, path: PathBuf) -> AdvisorImageAttachmentSnapshot {
        match prepare(path.as_path()) {
            Ok((manifest, bytes, preview_data_url)) => match self.pending.lock() {
                Ok(mut pending) => {
                    *pending = Some(PendingImage {
                        manifest: manifest.clone(),
                        preview_data_url: preview_data_url.clone(),
                        bytes,
                        created_at: Instant::now(),
                    });
                    AdvisorImageAttachmentSnapshot {
                        schema_version: 1,
                        state: AdvisorImageAttachmentState::Ready,
                        attachment: Some(manifest),
                        preview_data_url: Some(preview_data_url),
                        confirmation_state: Some(
                            AdvisorContentConfirmationState::ConfirmationRequired,
                        ),
                        diagnostic_code: None,
                    }
                }
                Err(_) => AdvisorImageAttachmentSnapshot::unavailable(
                    AdvisorImageAttachmentDiagnosticCode::ReadFailed,
                ),
            },
            Err(code) => AdvisorImageAttachmentSnapshot::unavailable(code),
        }
    }
    pub fn clear(&self) -> AdvisorImageAttachmentSnapshot {
        match self.pending.lock() {
            Ok(mut pending) => {
                *pending = None;
                AdvisorImageAttachmentSnapshot::empty()
            }
            Err(_) => AdvisorImageAttachmentSnapshot::unavailable(
                AdvisorImageAttachmentDiagnosticCode::ReadFailed,
            ),
        }
    }
    pub fn claim(
        &self,
        request: &AdvisorImageAttachmentClaimRequest,
    ) -> Result<ClaimedAdvisorImageAttachment, AdvisorImageAttachmentDiagnosticCode> {
        if !valid_uuid_v7(&request.attachment_id)
            || !valid_hash(&request.manifest_sha256)
            || request.confirmation != AdvisorContentConfirmationState::ConfirmedForSingleSend
        {
            return Err(AdvisorImageAttachmentDiagnosticCode::InvalidRequest);
        }
        let mut pending = self
            .pending
            .lock()
            .map_err(|_| AdvisorImageAttachmentDiagnosticCode::ReadFailed)?;
        let Some(item) = pending.take() else {
            return Err(AdvisorImageAttachmentDiagnosticCode::AttachmentNotFound);
        };
        if item.created_at.elapsed() > ATTACHMENT_TTL {
            return Err(AdvisorImageAttachmentDiagnosticCode::AttachmentExpired);
        }
        if item.manifest.attachment_id != request.attachment_id
            || item.manifest.sha256 != request.manifest_sha256
        {
            return Err(AdvisorImageAttachmentDiagnosticCode::ManifestMismatch);
        }
        let file = memory_file(&item.bytes)?;
        Ok(ClaimedAdvisorImageAttachment { file })
    }
}

fn prepare(
    path: &Path,
) -> Result<(AdvisorImageAttachmentManifest, Vec<u8>, String), AdvisorImageAttachmentDiagnosticCode>
{
    if !path.is_absolute() {
        return Err(AdvisorImageAttachmentDiagnosticCode::InvalidRequest);
    }
    let selected = path
        .symlink_metadata()
        .map_err(|_| AdvisorImageAttachmentDiagnosticCode::ReadFailed)?;
    if selected.file_type().is_symlink() || !selected.is_file() {
        return Err(AdvisorImageAttachmentDiagnosticCode::ReadFailed);
    }
    if selected.len() > MAX_ADVISOR_IMAGE_BYTES as u64 {
        return Err(AdvisorImageAttachmentDiagnosticCode::FileTooLarge);
    }
    let display_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(AdvisorImageAttachmentDiagnosticCode::UnsafeName)
        .and_then(validate_display_name)?;
    let extension = Path::new(&display_name)
        .extension()
        .and_then(|part| part.to_str())
        .ok_or(AdvisorImageAttachmentDiagnosticCode::UnsupportedType)?
        .to_ascii_lowercase();
    if !matches!(extension.as_str(), "png" | "jpg" | "jpeg") {
        return Err(AdvisorImageAttachmentDiagnosticCode::UnsupportedType);
    }
    let resolved = path
        .canonicalize()
        .map_err(|_| AdvisorImageAttachmentDiagnosticCode::ReadFailed)?;
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(&resolved)
        .map_err(|_| AdvisorImageAttachmentDiagnosticCode::ReadFailed)?;
    let opened = file
        .metadata()
        .map_err(|_| AdvisorImageAttachmentDiagnosticCode::ReadFailed)?;
    if !opened.is_file()
        || opened.len() != selected.len()
        || opened.dev() != selected.dev()
        || opened.ino() != selected.ino()
        || descriptor_path(&file)? != resolved
    {
        return Err(AdvisorImageAttachmentDiagnosticCode::ReadFailed);
    }
    let mut bytes = Vec::with_capacity(opened.len() as usize);
    (&mut file)
        .take(MAX_ADVISOR_IMAGE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| AdvisorImageAttachmentDiagnosticCode::ReadFailed)?;
    if bytes.len() as u64 != opened.len() {
        return Err(AdvisorImageAttachmentDiagnosticCode::ReadFailed);
    }
    let image = validate_attachment_image(&bytes).map_err(map_preview_error)?;
    let media_type = match image.mime_type {
        "image/png" if extension == "png" => AdvisorImageMediaType::Png,
        "image/jpeg" if matches!(extension.as_str(), "jpg" | "jpeg") => AdvisorImageMediaType::Jpeg,
        _ => return Err(AdvisorImageAttachmentDiagnosticCode::InvalidContent),
    };
    let preview_data_url = format!("data:{};base64,{}", image.mime_type, BASE64.encode(&bytes));
    Ok((
        AdvisorImageAttachmentManifest {
            attachment_id: Uuid::now_v7().to_string(),
            display_name,
            content_category: AdvisorContentCategory::Image,
            media_type,
            byte_size: bytes.len() as u64,
            width: image.width,
            height: image.height,
            sha256: format!("{:x}", Sha256::digest(&bytes)),
            projection: AdvisorImageProjection {
                kind: AdvisorImageProjectionKind::LocalImage,
                width: image.width,
                height: image.height,
            },
            disposal: AdvisorContentDisposal::TransientMemoryOneSend,
        },
        bytes,
        preview_data_url,
    ))
}
fn memory_file(bytes: &[u8]) -> Result<File, AdvisorImageAttachmentDiagnosticCode> {
    let name = CString::new("quireforge-advisor-image")
        .map_err(|_| AdvisorImageAttachmentDiagnosticCode::ReadFailed)?;
    let fd = unsafe { libc::memfd_create(name.as_ptr(), libc::MFD_CLOEXEC) };
    if fd < 0 {
        return Err(AdvisorImageAttachmentDiagnosticCode::ReadFailed);
    }
    let mut file = unsafe { File::from_raw_fd(fd) };
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| AdvisorImageAttachmentDiagnosticCode::ReadFailed)?;
    Ok(file)
}
fn map_preview_error(error: FilePreviewDiagnosticCode) -> AdvisorImageAttachmentDiagnosticCode {
    match error {
        FilePreviewDiagnosticCode::FileTooLarge => {
            AdvisorImageAttachmentDiagnosticCode::FileTooLarge
        }
        FilePreviewDiagnosticCode::UnsupportedType => {
            AdvisorImageAttachmentDiagnosticCode::UnsupportedType
        }
        _ => AdvisorImageAttachmentDiagnosticCode::InvalidContent,
    }
}
fn validate_display_name(value: &str) -> Result<String, AdvisorImageAttachmentDiagnosticCode> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.len() > 255
        || value.contains(['/', '\\'])
        || value.chars().any(|c| {
            c.is_control() || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
    {
        Err(AdvisorImageAttachmentDiagnosticCode::UnsafeName)
    } else {
        Ok(value.to_owned())
    }
}
fn descriptor_path(file: &File) -> Result<PathBuf, AdvisorImageAttachmentDiagnosticCode> {
    PathBuf::from("/proc/self/fd")
        .join(file.as_raw_fd().to_string())
        .canonicalize()
        .map_err(|_| AdvisorImageAttachmentDiagnosticCode::ReadFailed)
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
    fn png() -> Vec<u8> {
        fn chunk(bytes: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
            bytes.extend_from_slice(&(data.len() as u32).to_be_bytes());
            bytes.extend_from_slice(kind);
            bytes.extend_from_slice(data);
            bytes.extend_from_slice(&[0; 4]);
        }
        let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut header = Vec::new();
        header.extend_from_slice(&1u32.to_be_bytes());
        header.extend_from_slice(&1u32.to_be_bytes());
        header.extend_from_slice(&[8, 6, 0, 0, 0]);
        chunk(&mut bytes, b"IHDR", &header);
        chunk(&mut bytes, b"IDAT", &[]);
        chunk(&mut bytes, b"IEND", &[]);
        bytes
    }
    #[test]
    fn image_is_path_free_and_claimed_once() {
        let dir = std::env::temp_dir().join(Uuid::now_v7().to_string());
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("pixel.png");
        std::fs::write(&path, png()).unwrap();
        let service = AdvisorImageAttachmentService::default();
        let manifest = service.stage_path(path.clone()).attachment.unwrap();
        assert_eq!(manifest.content_category, AdvisorContentCategory::Image);
        assert_eq!(manifest.width, 1);
        assert!(!format!("{manifest:?}").contains(path.to_str().unwrap()));
        let claimed = service
            .claim(&AdvisorImageAttachmentClaimRequest {
                attachment_id: manifest.attachment_id,
                manifest_sha256: manifest.sha256,
                confirmation: AdvisorContentConfirmationState::ConfirmedForSingleSend,
            })
            .unwrap();
        assert!(claimed.protocol_input()["path"]
            .as_str()
            .unwrap()
            .contains("/proc/"));
        assert_eq!(service.snapshot().state, AdvisorImageAttachmentState::Empty);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn rejects_extension_content_mismatch() {
        let dir = std::env::temp_dir().join(Uuid::now_v7().to_string());
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("pixel.jpg");
        std::fs::write(&path, png()).unwrap();
        let service = AdvisorImageAttachmentService::default();
        assert_eq!(
            service.stage_path(path).diagnostic_code,
            Some(AdvisorImageAttachmentDiagnosticCode::InvalidContent)
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}
