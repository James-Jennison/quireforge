//! Closed, one-use client for the separately installed M39 sandbox worker.
//!
//! This module never executes a sample. It only validates a static ELF64 source,
//! keeps the bytes transiently in memory until one explicit claim, and sends the
//! sealed typed request to the root-owned worker Unix socket. The worker is the
//! only process permitted to invoke Firecracker or its jailer.

use std::{
    fs::OpenOptions,
    io::{Read, Write},
    os::unix::{
        fs::{MetadataExt, OpenOptionsExt},
        net::UnixStream,
    },
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub const MAX_DYNAMIC_ANALYSIS_BYTES: usize = 32 * 1024 * 1024;
pub const WORKER_SOCKET: &str = "/run/quireforge-sandboxd/worker.sock";
const ATTACHMENT_TTL: Duration = Duration::from_secs(15 * 60);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DynamicAnalysisState {
    Empty,
    Ready,
    Unavailable,
    Completed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DynamicAnalysisDiagnosticCode {
    WorkerUnavailable,
    InvalidRequest,
    UnsupportedType,
    InvalidSignature,
    UnsupportedRuntime,
    SourceTooLarge,
    SourceUnavailable,
    SourceChanged,
    UnsafeName,
    AttachmentNotFound,
    AttachmentExpired,
    ManifestMismatch,
    WorkerRejected,
    ReadFailed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DynamicAnalysisOutcome {
    Completed,
    NonzeroExit,
    Signal,
    Timeout,
    PolicyDenied,
    SetupFailed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DynamicAnalysisManifest {
    pub run_id: String,
    pub display_name: String,
    pub byte_size: u64,
    pub sha256: String,
    pub elf_type: String,
    pub static_runtime: bool,
    pub max_memory_bytes: u64,
    pub max_wall_time_ms: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DynamicAnalysisResult {
    pub kind: String,
    pub schema_version: u16,
    pub run_id: String,
    pub outcome: DynamicAnalysisOutcome,
    pub elapsed_ms: u32,
    pub guest_started: bool,
    pub resource_limits: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DynamicAnalysisSnapshot {
    pub schema_version: u16,
    pub state: DynamicAnalysisState,
    pub manifest: Option<DynamicAnalysisManifest>,
    pub result: Option<DynamicAnalysisResult>,
    pub diagnostic_code: Option<DynamicAnalysisDiagnosticCode>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DynamicAnalysisRunRequest {
    pub run_id: String,
    pub sha256: String,
    pub confirmed: bool,
}

struct Pending {
    manifest: DynamicAnalysisManifest,
    bytes: Vec<u8>,
    created_at: Instant,
}

#[derive(Default)]
pub struct DynamicAnalysisService {
    pending: Mutex<Option<Pending>>,
}

impl DynamicAnalysisService {
    pub fn snapshot(&self) -> DynamicAnalysisSnapshot {
        let Ok(mut pending) = self.pending.lock() else {
            return unavailable(DynamicAnalysisDiagnosticCode::ReadFailed);
        };
        if pending
            .as_ref()
            .is_some_and(|item| item.created_at.elapsed() > ATTACHMENT_TTL)
        {
            *pending = None;
            return unavailable(DynamicAnalysisDiagnosticCode::AttachmentExpired);
        }
        pending
            .as_ref()
            .map_or_else(empty, |item| DynamicAnalysisSnapshot {
                schema_version: 1,
                state: DynamicAnalysisState::Ready,
                manifest: Some(item.manifest.clone()),
                result: None,
                diagnostic_code: None,
            })
    }

    pub fn stage_path(&self, path: PathBuf) -> DynamicAnalysisSnapshot {
        match prepare(&path) {
            Ok((manifest, bytes)) => match self.pending.lock() {
                Ok(mut pending) => {
                    *pending = Some(Pending {
                        manifest: manifest.clone(),
                        bytes,
                        created_at: Instant::now(),
                    });
                    DynamicAnalysisSnapshot {
                        schema_version: 1,
                        state: DynamicAnalysisState::Ready,
                        manifest: Some(manifest),
                        result: None,
                        diagnostic_code: None,
                    }
                }
                Err(_) => unavailable(DynamicAnalysisDiagnosticCode::ReadFailed),
            },
            Err(code) => unavailable(code),
        }
    }

    pub fn clear(&self) -> DynamicAnalysisSnapshot {
        match self.pending.lock() {
            Ok(mut pending) => {
                *pending = None;
                empty()
            }
            Err(_) => unavailable(DynamicAnalysisDiagnosticCode::ReadFailed),
        }
    }

    pub fn run(&self, request: DynamicAnalysisRunRequest) -> DynamicAnalysisSnapshot {
        if !request.confirmed || !valid_uuid(&request.run_id) || !valid_hash(&request.sha256) {
            return unavailable(DynamicAnalysisDiagnosticCode::InvalidRequest);
        }
        let item = match self.pending.lock() {
            Ok(mut pending) => pending.take(),
            Err(_) => return unavailable(DynamicAnalysisDiagnosticCode::ReadFailed),
        };
        let Some(item) = item else {
            return unavailable(DynamicAnalysisDiagnosticCode::AttachmentNotFound);
        };
        if item.created_at.elapsed() > ATTACHMENT_TTL {
            return unavailable(DynamicAnalysisDiagnosticCode::AttachmentExpired);
        }
        if item.manifest.run_id != request.run_id || item.manifest.sha256 != request.sha256 {
            return unavailable(DynamicAnalysisDiagnosticCode::ManifestMismatch);
        }
        match send_to_worker(&item.manifest, &item.bytes) {
            Ok(result) => DynamicAnalysisSnapshot {
                schema_version: 1,
                state: DynamicAnalysisState::Completed,
                manifest: Some(item.manifest),
                result: Some(result),
                diagnostic_code: None,
            },
            Err(code) => unavailable(code),
        }
    }
}

fn empty() -> DynamicAnalysisSnapshot {
    DynamicAnalysisSnapshot {
        schema_version: 1,
        state: DynamicAnalysisState::Empty,
        manifest: None,
        result: None,
        diagnostic_code: None,
    }
}
fn unavailable(code: DynamicAnalysisDiagnosticCode) -> DynamicAnalysisSnapshot {
    DynamicAnalysisSnapshot {
        schema_version: 1,
        state: DynamicAnalysisState::Unavailable,
        manifest: None,
        result: None,
        diagnostic_code: Some(code),
    }
}

fn prepare(
    path: &Path,
) -> Result<(DynamicAnalysisManifest, Vec<u8>), DynamicAnalysisDiagnosticCode> {
    if !path.is_absolute() {
        return Err(DynamicAnalysisDiagnosticCode::InvalidRequest);
    }
    let selected = path
        .symlink_metadata()
        .map_err(|_| DynamicAnalysisDiagnosticCode::SourceUnavailable)?;
    if selected.file_type().is_symlink() || !selected.is_file() {
        return Err(DynamicAnalysisDiagnosticCode::SourceUnavailable);
    }
    if selected.len() > MAX_DYNAMIC_ANALYSIS_BYTES as u64 {
        return Err(DynamicAnalysisDiagnosticCode::SourceTooLarge);
    }
    let display_name = safe_name(
        path.file_name()
            .and_then(|name| name.to_str())
            .ok_or(DynamicAnalysisDiagnosticCode::UnsafeName)?,
    )?;
    let resolved = path
        .canonicalize()
        .map_err(|_| DynamicAnalysisDiagnosticCode::SourceUnavailable)?;
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(&resolved)
        .map_err(|_| DynamicAnalysisDiagnosticCode::SourceUnavailable)?;
    let opened = file
        .metadata()
        .map_err(|_| DynamicAnalysisDiagnosticCode::ReadFailed)?;
    if !opened.is_file()
        || opened.len() != selected.len()
        || opened.dev() != selected.dev()
        || opened.ino() != selected.ino()
    {
        return Err(DynamicAnalysisDiagnosticCode::SourceChanged);
    }
    let mut bytes = Vec::with_capacity(opened.len() as usize);
    file.read_to_end(&mut bytes)
        .map_err(|_| DynamicAnalysisDiagnosticCode::ReadFailed)?;
    let after = file
        .metadata()
        .map_err(|_| DynamicAnalysisDiagnosticCode::ReadFailed)?;
    if after.len() != opened.len() || after.dev() != opened.dev() || after.ino() != opened.ino() {
        return Err(DynamicAnalysisDiagnosticCode::SourceChanged);
    }
    let elf_type = verify_static_elf64(&bytes)?;
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    Ok((
        DynamicAnalysisManifest {
            run_id: Uuid::now_v7().to_string(),
            display_name,
            byte_size: opened.len(),
            sha256,
            elf_type,
            static_runtime: true,
            max_memory_bytes: 512 * 1024 * 1024,
            max_wall_time_ms: 30_000,
        },
        bytes,
    ))
}

fn verify_static_elf64(bytes: &[u8]) -> Result<String, DynamicAnalysisDiagnosticCode> {
    if bytes.len() < 64 || &bytes[..4] != b"\x7fELF" {
        return Err(DynamicAnalysisDiagnosticCode::InvalidSignature);
    }
    if bytes[4] != 2
        || bytes[5] != 1
        || bytes[6] != 1
        || u16::from_le_bytes([bytes[18], bytes[19]]) != 62
    {
        return Err(DynamicAnalysisDiagnosticCode::UnsupportedType);
    }
    let kind = match u16::from_le_bytes([bytes[16], bytes[17]]) {
        2 => "executable",
        3 => "shared-object",
        _ => return Err(DynamicAnalysisDiagnosticCode::UnsupportedType),
    };
    let offset = u64::from_le_bytes(bytes[32..40].try_into().expect("ELF header"));
    let entry_size = u16::from_le_bytes(bytes[54..56].try_into().expect("ELF header")) as u64;
    let count = u16::from_le_bytes(bytes[56..58].try_into().expect("ELF header")) as u64;
    if entry_size < 56
        || count > 256
        || offset
            .checked_add(
                entry_size
                    .checked_mul(count)
                    .ok_or(DynamicAnalysisDiagnosticCode::UnsupportedType)?,
            )
            .is_none_or(|end| end > bytes.len() as u64)
    {
        return Err(DynamicAnalysisDiagnosticCode::UnsupportedType);
    }
    for index in 0..count {
        let start = (offset + index * entry_size) as usize;
        if u32::from_le_bytes(bytes[start..start + 4].try_into().expect("program header")) == 3 {
            return Err(DynamicAnalysisDiagnosticCode::UnsupportedRuntime);
        }
    }
    Ok(kind.to_owned())
}

fn send_to_worker(
    manifest: &DynamicAnalysisManifest,
    bytes: &[u8],
) -> Result<DynamicAnalysisResult, DynamicAnalysisDiagnosticCode> {
    let mut stream = UnixStream::connect(WORKER_SOCKET)
        .map_err(|_| DynamicAnalysisDiagnosticCode::WorkerUnavailable)?;
    stream
        .set_read_timeout(Some(Duration::from_secs(35)))
        .map_err(|_| DynamicAnalysisDiagnosticCode::WorkerUnavailable)?;
    let request = serde_json::json!({"kind":"dynamic-analysis-request-v1","runId":manifest.run_id,"sha256":manifest.sha256,"byteSize":manifest.byte_size,"elfType":manifest.elf_type,"staticRuntime":true});
    let header =
        serde_json::to_vec(&request).map_err(|_| DynamicAnalysisDiagnosticCode::ReadFailed)?;
    stream
        .write_all(&(header.len() as u32).to_be_bytes())
        .and_then(|_| stream.write_all(&header))
        .and_then(|_| stream.write_all(bytes))
        .map_err(|_| DynamicAnalysisDiagnosticCode::WorkerUnavailable)?;
    let mut length = [0_u8; 4];
    stream
        .read_exact(&mut length)
        .map_err(|_| DynamicAnalysisDiagnosticCode::WorkerRejected)?;
    let length = u32::from_be_bytes(length) as usize;
    if length > 8 * 1024 {
        return Err(DynamicAnalysisDiagnosticCode::WorkerRejected);
    }
    let mut response = vec![0_u8; length];
    stream
        .read_exact(&mut response)
        .map_err(|_| DynamicAnalysisDiagnosticCode::WorkerRejected)?;
    let result: DynamicAnalysisResult = serde_json::from_slice(&response)
        .map_err(|_| DynamicAnalysisDiagnosticCode::WorkerRejected)?;
    if result.kind != "dynamic-analysis-result-v1"
        || result.schema_version != 1
        || result.run_id != manifest.run_id
        || result.resource_limits.len() > 8
    {
        return Err(DynamicAnalysisDiagnosticCode::WorkerRejected);
    }
    Ok(result)
}

fn safe_name(value: &str) -> Result<String, DynamicAnalysisDiagnosticCode> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.len() > 255
        || value.contains(['/', '\\', '\0'])
    {
        Err(DynamicAnalysisDiagnosticCode::UnsafeName)
    } else {
        Ok(value.to_owned())
    }
}
fn valid_uuid(value: &str) -> bool {
    Uuid::parse_str(value).is_ok_and(|id| id.get_version() == Some(uuid::Version::SortRand))
}
fn valid_hash(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, time::SystemTime};

    fn elf(kind: u16, interpreter: bool) -> Vec<u8> {
        let mut value = vec![0_u8; 120];
        value[..4].copy_from_slice(b"\x7fELF");
        value[4] = 2;
        value[5] = 1;
        value[6] = 1;
        value[16..18].copy_from_slice(&kind.to_le_bytes());
        value[18..20].copy_from_slice(&62_u16.to_le_bytes());
        value[32..40].copy_from_slice(&64_u64.to_le_bytes());
        value[54..56].copy_from_slice(&56_u16.to_le_bytes());
        value[56..58].copy_from_slice(&1_u16.to_le_bytes());
        if interpreter {
            value[64..68].copy_from_slice(&3_u32.to_le_bytes());
        }
        value
    }
    #[test]
    fn accepts_static_x86_64_exec_and_pie() {
        assert_eq!(verify_static_elf64(&elf(2, false)).unwrap(), "executable");
        assert_eq!(
            verify_static_elf64(&elf(3, false)).unwrap(),
            "shared-object"
        );
    }
    #[test]
    fn rejects_dynamic_interpreter_without_exposing_details() {
        assert_eq!(
            verify_static_elf64(&elf(3, true)),
            Err(DynamicAnalysisDiagnosticCode::UnsupportedRuntime)
        );
    }

    fn temporary_elf() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("quireforge-m39-{suffix}"));
        fs::write(&path, elf(2, false)).unwrap();
        path
    }

    #[test]
    fn confirmed_claim_is_one_use_and_never_retains_worker_failures() {
        let path = temporary_elf();
        let service = DynamicAnalysisService::default();
        let ready = service.stage_path(path.clone());
        let manifest = ready.manifest.unwrap();
        assert_eq!(ready.state, DynamicAnalysisState::Ready);

        let unconfirmed = service.run(DynamicAnalysisRunRequest {
            run_id: manifest.run_id.clone(),
            sha256: manifest.sha256.clone(),
            confirmed: false,
        });
        assert_eq!(
            unconfirmed.diagnostic_code,
            Some(DynamicAnalysisDiagnosticCode::InvalidRequest)
        );
        assert_eq!(service.snapshot().state, DynamicAnalysisState::Ready);

        let rejected = service.run(DynamicAnalysisRunRequest {
            run_id: Uuid::now_v7().to_string(),
            sha256: manifest.sha256.clone(),
            confirmed: true,
        });
        assert_eq!(
            rejected.diagnostic_code,
            Some(DynamicAnalysisDiagnosticCode::ManifestMismatch)
        );
        assert_eq!(service.snapshot().state, DynamicAnalysisState::Empty);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn source_failures_do_not_stage_a_path_or_partial_sample() {
        let path = temporary_elf();
        let service = DynamicAnalysisService::default();
        fs::write(&path, b"not an elf").unwrap();
        let result = service.stage_path(path.clone());
        assert_eq!(result.state, DynamicAnalysisState::Unavailable);
        assert_eq!(result.manifest, None);
        assert_eq!(
            result.diagnostic_code,
            Some(DynamicAnalysisDiagnosticCode::InvalidSignature)
        );
        assert_eq!(service.snapshot().state, DynamicAnalysisState::Empty);
        let _ = fs::remove_file(path);
    }
}
