//! One-use, metadata-only ELF manifests for Advisor.
//!
//! M36 accepts a closed ELF-only format. `elf` owns low-level ELF parsing after
//! QuireForge has enforced the source boundary and table bounds. No bytes,
//! names, addresses, notes, debug data, or executable content are transported.

use std::{
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

use elf::{abi, endian::AnyEndian, ElfStream};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::advisor_attachment::{
    AdvisorContentCategory, AdvisorContentConfirmationState, AdvisorContentDisposal,
};

pub const MAX_ADVISOR_BINARY_BYTES: usize = 32 * 1024 * 1024;
const MAX_PROGRAM_HEADERS: usize = 256;
const MAX_SECTION_HEADERS: usize = 1_024;
const MAX_HEADER_TABLE_BYTES: u64 = 1024 * 1024;
const MAX_DYNAMIC_ENTRIES: usize = 256;
const MAX_MANIFEST_BYTES: usize = 8 * 1024;
const ATTACHMENT_TTL: Duration = Duration::from_secs(15 * 60);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorBinaryMediaType {
    Elf,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorBinaryProjectionKind {
    StaticBinaryManifestV1,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorBinaryFileType {
    Relocatable,
    Executable,
    SharedObject,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorBinaryAttachmentState {
    Empty,
    Ready,
    Unavailable,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorBinaryAttachmentDiagnosticCode {
    InvalidRequest,
    UnsupportedType,
    InvalidSignature,
    SourceTooLarge,
    SourceUnavailable,
    SourceChanged,
    MalformedOrUnsupportedElf,
    UnsupportedElfLayout,
    MetadataLimitExceeded,
    UnsafeName,
    ReadFailed,
    AttachmentNotFound,
    AttachmentExpired,
    ManifestMismatch,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorBinaryProjection {
    pub kind: AdvisorBinaryProjectionKind,
    pub schema_version: u16,
    pub elf_class: String,
    pub endianness: String,
    pub file_type: AdvisorBinaryFileType,
    pub machine: u16,
    pub os_abi: u8,
    pub program_header_count: u16,
    pub section_header_count: u16,
    pub dynamic_section_present: bool,
    pub dynamic_entry_count: u16,
    pub manifest_byte_size: u32,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorBinaryAttachmentManifest {
    pub attachment_id: String,
    pub display_name: String,
    pub content_category: AdvisorContentCategory,
    pub media_type: AdvisorBinaryMediaType,
    pub byte_size: u64,
    pub sha256: String,
    pub projection: AdvisorBinaryProjection,
    pub disposal: AdvisorContentDisposal,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorBinaryAttachmentSnapshot {
    pub schema_version: u16,
    pub state: AdvisorBinaryAttachmentState,
    pub attachment: Option<AdvisorBinaryAttachmentManifest>,
    pub confirmation_state: Option<AdvisorContentConfirmationState>,
    pub diagnostic_code: Option<AdvisorBinaryAttachmentDiagnosticCode>,
}
impl AdvisorBinaryAttachmentSnapshot {
    pub fn empty() -> Self {
        Self {
            schema_version: 1,
            state: AdvisorBinaryAttachmentState::Empty,
            attachment: None,
            confirmation_state: None,
            diagnostic_code: None,
        }
    }
    fn unavailable(code: AdvisorBinaryAttachmentDiagnosticCode) -> Self {
        Self {
            schema_version: 1,
            state: AdvisorBinaryAttachmentState::Unavailable,
            attachment: None,
            confirmation_state: None,
            diagnostic_code: Some(code),
        }
    }
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdvisorBinaryAttachmentClaimRequest {
    pub attachment_id: String,
    pub manifest_sha256: String,
    pub confirmation: AdvisorContentConfirmationState,
}
#[derive(Debug)]
pub(crate) struct ClaimedAdvisorBinaryAttachment {
    pub(crate) manifest: AdvisorBinaryAttachmentManifest,
    pub(crate) projection_text: String,
}
struct PendingBinary {
    manifest: AdvisorBinaryAttachmentManifest,
    projection_text: String,
    created_at: Instant,
}
#[derive(Default)]
pub struct AdvisorBinaryAttachmentService {
    pending: Mutex<Option<PendingBinary>>,
}

impl AdvisorBinaryAttachmentService {
    pub fn snapshot(&self) -> AdvisorBinaryAttachmentSnapshot {
        let Ok(mut pending) = self.pending.lock() else {
            return AdvisorBinaryAttachmentSnapshot::unavailable(
                AdvisorBinaryAttachmentDiagnosticCode::ReadFailed,
            );
        };
        if pending
            .as_ref()
            .is_some_and(|item| item.created_at.elapsed() > ATTACHMENT_TTL)
        {
            *pending = None;
            return AdvisorBinaryAttachmentSnapshot::unavailable(
                AdvisorBinaryAttachmentDiagnosticCode::AttachmentExpired,
            );
        }
        pending
            .as_ref()
            .map_or_else(AdvisorBinaryAttachmentSnapshot::empty, |item| {
                AdvisorBinaryAttachmentSnapshot {
                    schema_version: 1,
                    state: AdvisorBinaryAttachmentState::Ready,
                    attachment: Some(item.manifest.clone()),
                    confirmation_state: Some(AdvisorContentConfirmationState::ConfirmationRequired),
                    diagnostic_code: None,
                }
            })
    }
    pub fn stage_path(&self, path: PathBuf) -> AdvisorBinaryAttachmentSnapshot {
        match prepare(&path) {
            Ok((manifest, projection_text)) => match self.pending.lock() {
                Ok(mut pending) => {
                    *pending = Some(PendingBinary {
                        manifest: manifest.clone(),
                        projection_text,
                        created_at: Instant::now(),
                    });
                    AdvisorBinaryAttachmentSnapshot {
                        schema_version: 1,
                        state: AdvisorBinaryAttachmentState::Ready,
                        attachment: Some(manifest),
                        confirmation_state: Some(
                            AdvisorContentConfirmationState::ConfirmationRequired,
                        ),
                        diagnostic_code: None,
                    }
                }
                Err(_) => AdvisorBinaryAttachmentSnapshot::unavailable(
                    AdvisorBinaryAttachmentDiagnosticCode::ReadFailed,
                ),
            },
            Err(code) => AdvisorBinaryAttachmentSnapshot::unavailable(code),
        }
    }
    pub fn clear(&self) -> AdvisorBinaryAttachmentSnapshot {
        match self.pending.lock() {
            Ok(mut pending) => {
                *pending = None;
                AdvisorBinaryAttachmentSnapshot::empty()
            }
            Err(_) => AdvisorBinaryAttachmentSnapshot::unavailable(
                AdvisorBinaryAttachmentDiagnosticCode::ReadFailed,
            ),
        }
    }
    pub fn claim(
        &self,
        request: &AdvisorBinaryAttachmentClaimRequest,
    ) -> Result<ClaimedAdvisorBinaryAttachment, AdvisorBinaryAttachmentDiagnosticCode> {
        if !valid_uuid_v7(&request.attachment_id)
            || !valid_hash(&request.manifest_sha256)
            || request.confirmation != AdvisorContentConfirmationState::ConfirmedForSingleSend
        {
            return Err(AdvisorBinaryAttachmentDiagnosticCode::InvalidRequest);
        }
        let mut pending = self
            .pending
            .lock()
            .map_err(|_| AdvisorBinaryAttachmentDiagnosticCode::ReadFailed)?;
        let Some(item) = pending.take() else {
            return Err(AdvisorBinaryAttachmentDiagnosticCode::AttachmentNotFound);
        };
        if item.created_at.elapsed() > ATTACHMENT_TTL {
            return Err(AdvisorBinaryAttachmentDiagnosticCode::AttachmentExpired);
        }
        if item.manifest.attachment_id != request.attachment_id
            || item.manifest.sha256 != request.manifest_sha256
        {
            return Err(AdvisorBinaryAttachmentDiagnosticCode::ManifestMismatch);
        }
        Ok(ClaimedAdvisorBinaryAttachment {
            manifest: item.manifest,
            projection_text: item.projection_text,
        })
    }
}

fn prepare(
    path: &Path,
) -> Result<(AdvisorBinaryAttachmentManifest, String), AdvisorBinaryAttachmentDiagnosticCode> {
    if !path.is_absolute() {
        return Err(AdvisorBinaryAttachmentDiagnosticCode::InvalidRequest);
    }
    let selected = path
        .symlink_metadata()
        .map_err(|_| AdvisorBinaryAttachmentDiagnosticCode::SourceUnavailable)?;
    if selected.file_type().is_symlink() || !selected.is_file() {
        return Err(AdvisorBinaryAttachmentDiagnosticCode::SourceUnavailable);
    }
    if selected.len() > MAX_ADVISOR_BINARY_BYTES as u64 {
        return Err(AdvisorBinaryAttachmentDiagnosticCode::SourceTooLarge);
    }
    let display_name = validate_display_name(
        path.file_name()
            .and_then(|name| name.to_str())
            .ok_or(AdvisorBinaryAttachmentDiagnosticCode::UnsafeName)?,
    )?;
    let resolved = path
        .canonicalize()
        .map_err(|_| AdvisorBinaryAttachmentDiagnosticCode::SourceUnavailable)?;
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(&resolved)
        .map_err(|_| AdvisorBinaryAttachmentDiagnosticCode::SourceUnavailable)?;
    let opened = file
        .metadata()
        .map_err(|_| AdvisorBinaryAttachmentDiagnosticCode::ReadFailed)?;
    if !opened.is_file()
        || opened.len() != selected.len()
        || opened.dev() != selected.dev()
        || opened.ino() != selected.ino()
        || descriptor_path(&file)? != resolved
    {
        return Err(AdvisorBinaryAttachmentDiagnosticCode::SourceChanged);
    }
    let mut header = [0_u8; 64];
    file.read_exact(&mut header)
        .map_err(|_| AdvisorBinaryAttachmentDiagnosticCode::InvalidSignature)?;
    let preflight = preflight_header(&header, opened.len())?;
    file.seek(SeekFrom::Start(0))
        .map_err(|_| AdvisorBinaryAttachmentDiagnosticCode::ReadFailed)?;
    let sha256 = hash_file(&mut file)?;
    let after_hash = file
        .metadata()
        .map_err(|_| AdvisorBinaryAttachmentDiagnosticCode::ReadFailed)?;
    if after_hash.len() != opened.len()
        || after_hash.dev() != opened.dev()
        || after_hash.ino() != opened.ino()
    {
        return Err(AdvisorBinaryAttachmentDiagnosticCode::SourceChanged);
    }
    file.seek(SeekFrom::Start(0))
        .map_err(|_| AdvisorBinaryAttachmentDiagnosticCode::ReadFailed)?;
    let stream = ElfStream::<AnyEndian, _>::open_stream(file)
        .map_err(|_| AdvisorBinaryAttachmentDiagnosticCode::MalformedOrUnsupportedElf)?;
    if stream.segments().len() != preflight.program_header_count
        || stream.section_headers().len() != preflight.section_header_count
    {
        return Err(AdvisorBinaryAttachmentDiagnosticCode::MalformedOrUnsupportedElf);
    }
    let dynamic_headers: Vec<_> = stream
        .section_headers()
        .iter()
        .filter(|header| header.sh_type == abi::SHT_DYNAMIC)
        .collect();
    if dynamic_headers.len() > 1 {
        return Err(AdvisorBinaryAttachmentDiagnosticCode::MalformedOrUnsupportedElf);
    }
    let dynamic_entry_count = dynamic_headers.first().map_or(Ok(0_usize), |header| {
        let entry_size = usize::try_from(header.sh_entsize)
            .map_err(|_| AdvisorBinaryAttachmentDiagnosticCode::MetadataLimitExceeded)?;
        let size = usize::try_from(header.sh_size)
            .map_err(|_| AdvisorBinaryAttachmentDiagnosticCode::MetadataLimitExceeded)?;
        if entry_size == 0 || size % entry_size != 0 {
            return Err(AdvisorBinaryAttachmentDiagnosticCode::MalformedOrUnsupportedElf);
        }
        Ok(size / entry_size)
    })?;
    if dynamic_entry_count > MAX_DYNAMIC_ENTRIES {
        return Err(AdvisorBinaryAttachmentDiagnosticCode::MetadataLimitExceeded);
    }
    let dynamic_section_present = stream
        .segments()
        .iter()
        .any(|header| header.p_type == abi::PT_DYNAMIC)
        || !dynamic_headers.is_empty();
    let projection = AdvisorBinaryProjection {
        kind: AdvisorBinaryProjectionKind::StaticBinaryManifestV1,
        schema_version: 1,
        elf_class: preflight.elf_class.to_owned(),
        endianness: preflight.endianness.to_owned(),
        file_type: preflight.file_type,
        machine: preflight.machine,
        os_abi: preflight.os_abi,
        program_header_count: preflight.program_header_count as u16,
        section_header_count: preflight.section_header_count as u16,
        dynamic_section_present,
        dynamic_entry_count: dynamic_entry_count as u16,
        manifest_byte_size: 0,
    };
    let projection_text = format!("static-binary-manifest-v1\nbinary-format: elf\nelf-class: {}\nendianness: {}\nfile-type: {}\nmachine: {}\nos-abi: {}\nprogram-header-count: {}\nsection-header-count: {}\ndynamic-section-present: {}\ndynamic-entry-count: {}\n", projection.elf_class, projection.endianness, file_type_name(projection.file_type), projection.machine, projection.os_abi, projection.program_header_count, projection.section_header_count, projection.dynamic_section_present, projection.dynamic_entry_count);
    if projection_text.len() > MAX_MANIFEST_BYTES {
        return Err(AdvisorBinaryAttachmentDiagnosticCode::MetadataLimitExceeded);
    }
    let mut projection = projection;
    projection.manifest_byte_size = projection_text.len() as u32;
    let manifest = AdvisorBinaryAttachmentManifest {
        attachment_id: Uuid::now_v7().to_string(),
        display_name,
        content_category: AdvisorContentCategory::StaticBinary,
        media_type: AdvisorBinaryMediaType::Elf,
        byte_size: opened.len(),
        sha256,
        projection,
        disposal: AdvisorContentDisposal::TransientMemoryOneSend,
    };
    // The projection is intentionally reconstructed only from bounded fields.
    Ok((manifest, projection_text))
}

struct Preflight {
    elf_class: &'static str,
    endianness: &'static str,
    file_type: AdvisorBinaryFileType,
    machine: u16,
    os_abi: u8,
    program_header_count: usize,
    section_header_count: usize,
}
fn preflight_header(
    header: &[u8; 64],
    file_len: u64,
) -> Result<Preflight, AdvisorBinaryAttachmentDiagnosticCode> {
    if &header[0..4] != b"\x7fELF" {
        return Err(AdvisorBinaryAttachmentDiagnosticCode::InvalidSignature);
    }
    let (elf_class, is_le) = match (header[4], header[5]) {
        (1, 1) => ("elf32", true),
        (1, 2) => ("elf32", false),
        (2, 1) => ("elf64", true),
        (2, 2) => ("elf64", false),
        _ => return Err(AdvisorBinaryAttachmentDiagnosticCode::UnsupportedType),
    };
    if header[6] != 1 {
        return Err(AdvisorBinaryAttachmentDiagnosticCode::MalformedOrUnsupportedElf);
    }
    let read16 = |offset: usize| {
        if is_le {
            u16::from_le_bytes([header[offset], header[offset + 1]])
        } else {
            u16::from_be_bytes([header[offset], header[offset + 1]])
        }
    };
    let read32 = |offset: usize| {
        if is_le {
            u32::from_le_bytes([
                header[offset],
                header[offset + 1],
                header[offset + 2],
                header[offset + 3],
            ])
        } else {
            u32::from_be_bytes([
                header[offset],
                header[offset + 1],
                header[offset + 2],
                header[offset + 3],
            ])
        }
    };
    let read64 = |offset: usize| {
        if is_le {
            u64::from_le_bytes(header[offset..offset + 8].try_into().expect("header range"))
        } else {
            u64::from_be_bytes(header[offset..offset + 8].try_into().expect("header range"))
        }
    };
    let file_type = match read16(16) {
        1 => AdvisorBinaryFileType::Relocatable,
        2 => AdvisorBinaryFileType::Executable,
        3 => AdvisorBinaryFileType::SharedObject,
        _ => return Err(AdvisorBinaryAttachmentDiagnosticCode::UnsupportedType),
    };
    let machine = read16(18);
    let (phoff, shoff, phentsize, phnum, shentsize, shnum) = if elf_class == "elf32" {
        (
            read32(28) as u64,
            read32(32) as u64,
            read16(42),
            read16(44),
            read16(46),
            read16(48),
        )
    } else {
        (
            read64(32),
            read64(40),
            read16(54),
            read16(56),
            read16(58),
            read16(60),
        )
    };
    if phnum == u16::MAX || shnum >= 0xff00 || (shnum == 0 && shoff != 0) {
        return Err(AdvisorBinaryAttachmentDiagnosticCode::UnsupportedElfLayout);
    }
    let program_header_count = phnum as usize;
    let section_header_count = shnum as usize;
    if program_header_count > MAX_PROGRAM_HEADERS || section_header_count > MAX_SECTION_HEADERS {
        return Err(AdvisorBinaryAttachmentDiagnosticCode::MetadataLimitExceeded);
    }
    bounded_table(phoff, phentsize, program_header_count, file_len)?;
    bounded_table(shoff, shentsize, section_header_count, file_len)?;
    // Dynamic-table entries are fixed-width (8 bytes ELF32, 16 bytes ELF64).
    // The preflight never reads dynamic content; an absent section table is rejected above.
    Ok(Preflight {
        elf_class,
        endianness: if is_le { "little" } else { "big" },
        file_type,
        machine,
        os_abi: header[7],
        program_header_count,
        section_header_count,
    })
}
fn bounded_table(
    offset: u64,
    entry_size: u16,
    count: usize,
    file_len: u64,
) -> Result<(), AdvisorBinaryAttachmentDiagnosticCode> {
    if count == 0 {
        return Ok(());
    }
    let size = u64::from(entry_size)
        .checked_mul(count as u64)
        .ok_or(AdvisorBinaryAttachmentDiagnosticCode::MetadataLimitExceeded)?;
    if size == 0
        || size > MAX_HEADER_TABLE_BYTES
        || offset.checked_add(size).is_none_or(|end| end > file_len)
    {
        return Err(AdvisorBinaryAttachmentDiagnosticCode::MetadataLimitExceeded);
    }
    Ok(())
}
fn hash_file(file: &mut File) -> Result<String, AdvisorBinaryAttachmentDiagnosticCode> {
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|_| AdvisorBinaryAttachmentDiagnosticCode::ReadFailed)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
fn file_type_name(value: AdvisorBinaryFileType) -> &'static str {
    match value {
        AdvisorBinaryFileType::Relocatable => "relocatable",
        AdvisorBinaryFileType::Executable => "executable",
        AdvisorBinaryFileType::SharedObject => "shared-object",
    }
}
fn validate_display_name(value: &str) -> Result<String, AdvisorBinaryAttachmentDiagnosticCode> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.len() > 255
        || value.contains(['/', '\\'])
        || value.chars().any(|c| {
            c.is_control() || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
    {
        Err(AdvisorBinaryAttachmentDiagnosticCode::UnsafeName)
    } else {
        Ok(value.to_owned())
    }
}
fn descriptor_path(file: &File) -> Result<PathBuf, AdvisorBinaryAttachmentDiagnosticCode> {
    std::fs::read_link(format!("/proc/self/fd/{}", file.as_raw_fd()))
        .map_err(|_| AdvisorBinaryAttachmentDiagnosticCode::ReadFailed)
}
fn valid_uuid_v7(value: &str) -> bool {
    Uuid::parse_str(value).is_ok_and(|id| id.get_version_num() == 7)
}
fn valid_hash(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    fn minimal_elf(file_type: u16) -> Vec<u8> {
        let mut bytes = vec![0_u8; 64 + 56 + 64];
        bytes[..4].copy_from_slice(b"\x7fELF");
        bytes[4] = 2;
        bytes[5] = 1;
        bytes[6] = 1;
        bytes[7] = 3;
        bytes[16..18].copy_from_slice(&file_type.to_le_bytes());
        bytes[18..20].copy_from_slice(&62_u16.to_le_bytes());
        bytes[20..24].copy_from_slice(&1_u32.to_le_bytes());
        bytes[32..40].copy_from_slice(&64_u64.to_le_bytes());
        bytes[40..48].copy_from_slice(&120_u64.to_le_bytes());
        bytes[52..54].copy_from_slice(&64_u16.to_le_bytes());
        bytes[54..56].copy_from_slice(&56_u16.to_le_bytes());
        bytes[56..58].copy_from_slice(&1_u16.to_le_bytes());
        bytes[58..60].copy_from_slice(&64_u16.to_le_bytes());
        bytes[60..62].copy_from_slice(&1_u16.to_le_bytes());
        bytes
    }
    fn staged(bytes: &[u8], name: &str) -> AdvisorBinaryAttachmentSnapshot {
        let directory = std::env::temp_dir().join(Uuid::now_v7().to_string());
        std::fs::create_dir_all(&directory).expect("directory");
        let path = directory.join(name);
        let mut file = File::create(&path).expect("file");
        file.write_all(bytes).expect("write");
        let snapshot = AdvisorBinaryAttachmentService::default().stage_path(path);
        std::fs::remove_dir_all(directory).expect("cleanup");
        snapshot
    }
    #[test]
    fn accepts_metadata_only_elf() {
        let snapshot = staged(&minimal_elf(2), "candidate");
        let attachment = snapshot.attachment.expect("attachment");
        assert_eq!(attachment.media_type, AdvisorBinaryMediaType::Elf);
        assert_eq!(
            attachment.projection.kind,
            AdvisorBinaryProjectionKind::StaticBinaryManifestV1
        );
        assert_eq!(attachment.projection.elf_class, "elf64");
        assert_eq!(
            attachment.projection.file_type,
            AdvisorBinaryFileType::Executable
        );
    }
    #[test]
    fn rejects_non_elf_and_core() {
        assert_eq!(
            staged(b"not an elf", "candidate").diagnostic_code,
            Some(AdvisorBinaryAttachmentDiagnosticCode::InvalidSignature)
        );
        assert_eq!(
            staged(&minimal_elf(4), "candidate").diagnostic_code,
            Some(AdvisorBinaryAttachmentDiagnosticCode::UnsupportedType)
        );
    }

    #[test]
    fn claim_is_confirmed_hash_bound_one_use_and_transport_is_metadata_only() {
        let directory = std::env::temp_dir().join(Uuid::now_v7().to_string());
        std::fs::create_dir_all(&directory).expect("directory");
        let path = directory.join("candidate");
        std::fs::write(&path, minimal_elf(3)).expect("write");
        let service = AdvisorBinaryAttachmentService::default();
        let manifest = service
            .stage_path(path)
            .attachment
            .expect("staged manifest");
        assert_eq!(
            service
                .claim(&AdvisorBinaryAttachmentClaimRequest {
                    attachment_id: manifest.attachment_id.clone(),
                    manifest_sha256: "0".repeat(64),
                    confirmation: AdvisorContentConfirmationState::ConfirmedForSingleSend,
                })
                .expect_err("mismatched hash"),
            AdvisorBinaryAttachmentDiagnosticCode::ManifestMismatch
        );
        assert_eq!(
            service.snapshot().state,
            AdvisorBinaryAttachmentState::Empty
        );
        let replacement = directory.join("replacement");
        std::fs::write(&replacement, minimal_elf(1)).expect("replacement");
        let manifest = service
            .stage_path(replacement)
            .attachment
            .expect("replacement manifest");
        let request = AdvisorBinaryAttachmentClaimRequest {
            attachment_id: manifest.attachment_id,
            manifest_sha256: manifest.sha256,
            confirmation: AdvisorContentConfirmationState::ConfirmedForSingleSend,
        };
        let claim = service.claim(&request).expect("claim");
        assert!(claim
            .projection_text
            .starts_with("static-binary-manifest-v1\n"));
        assert!(!claim.projection_text.contains("\x7fELF"));
        assert!(!claim.projection_text.contains("/proc/"));
        assert_eq!(
            service.claim(&request).expect_err("one-use claim"),
            AdvisorBinaryAttachmentDiagnosticCode::AttachmentNotFound
        );
        std::fs::remove_dir_all(directory).expect("cleanup");
    }

    #[test]
    fn preflight_fails_closed_for_excessive_or_unsupported_tables() {
        let mut excessive = minimal_elf(2);
        excessive[56..58].copy_from_slice(&257_u16.to_le_bytes());
        assert_eq!(
            staged(&excessive, "candidate").diagnostic_code,
            Some(AdvisorBinaryAttachmentDiagnosticCode::MetadataLimitExceeded)
        );
        let mut extended = minimal_elf(2);
        extended[60..62].copy_from_slice(&0_u16.to_le_bytes());
        assert_eq!(
            staged(&extended, "candidate").diagnostic_code,
            Some(AdvisorBinaryAttachmentDiagnosticCode::UnsupportedElfLayout)
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn rejects_selected_symlinks_without_following_them() {
        use std::os::unix::fs::symlink;
        let directory = std::env::temp_dir().join(Uuid::now_v7().to_string());
        std::fs::create_dir_all(&directory).expect("directory");
        let source = directory.join("candidate");
        std::fs::write(&source, minimal_elf(2)).expect("source");
        let selected = directory.join("selected");
        symlink(&source, &selected).expect("symlink");
        let snapshot = AdvisorBinaryAttachmentService::default().stage_path(selected);
        assert_eq!(
            snapshot.diagnostic_code,
            Some(AdvisorBinaryAttachmentDiagnosticCode::SourceUnavailable)
        );
        std::fs::remove_dir_all(directory).expect("cleanup");
    }
}
