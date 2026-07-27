//! Root-owned, closed M39 Firecracker worker.
//!
//! It accepts exactly one framed typed static-ELF request, validates the bytes
//! itself, creates only disposable run state, and emits a bounded metadata-only
//! result. It has no TCP listener, shell command surface, project context, or
//! generic file protocol.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    os::{
        fd::AsRawFd,
        unix::net::{UnixListener, UnixStream},
    },
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use uuid::Uuid;

const SOCKET: &str = "/run/quireforge-sandboxd/worker.sock";
const ROOT: &str = "/var/lib/quireforge-sandboxd";
const ASSETS: &str = "/usr/lib/quireforge-sandboxd";
const MAX_BYTES: usize = 32 * 1024 * 1024;
const MAX_RESULT: usize = 8 * 1024;
const RESOURCE_LIMITS: [&str; 4] = ["1-vcpu", "512-mib-memory", "30-s-wall-time", "no-network"];

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Request {
    kind: String,
    run_id: String,
    sha256: String,
    byte_size: u64,
    elf_type: String,
    static_runtime: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ResultMessage {
    kind: &'static str,
    schema_version: u16,
    run_id: String,
    outcome: &'static str,
    elapsed_ms: u32,
    guest_started: bool,
    resource_limits: Vec<String>,
}

fn main() -> std::io::Result<()> {
    if unsafe { libc::geteuid() } != 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "root-owned worker required",
        ));
    }
    let socket = Path::new(SOCKET);
    if let Some(parent) = socket.parent() {
        fs::create_dir_all(parent)?;
        fs::set_permissions(parent, std::os::unix::fs::PermissionsExt::from_mode(0o755))?;
    }
    if socket.exists() {
        fs::remove_file(socket)?;
    }
    let listener = UnixListener::bind(socket)?;
    let group = std::ffi::CString::new("quireforge-sandbox").expect("static group name");
    let group_record = unsafe { libc::getgrnam(group.as_ptr()) };
    if group_record.is_null() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "sandbox group missing",
        ));
    }
    if unsafe {
        libc::chown(
            socket.as_os_str().as_encoded_bytes().as_ptr() as *const libc::c_char,
            0,
            (*group_record).gr_gid,
        )
    } != 0
    {
        return Err(std::io::Error::last_os_error());
    }
    fs::set_permissions(socket, std::os::unix::fs::PermissionsExt::from_mode(0o660))?;
    for stream in listener.incoming().flatten() {
        let _ = handle(stream);
    }
    Ok(())
}

fn handle(mut stream: UnixStream) -> std::io::Result<()> {
    let credentials = peer_credentials(&stream)?;
    if credentials.uid == 0 || credentials.uid == u32::MAX {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "invalid peer",
        ));
    }
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let request = read_request(&mut stream)?;
    let mut bytes = vec![0; request.byte_size as usize];
    stream.read_exact(&mut bytes)?;
    let result = run(&request, &bytes);
    let encoded = serde_json::to_vec(&result).expect("fixed result serialization");
    stream.write_all(&(encoded.len() as u32).to_be_bytes())?;
    stream.write_all(&encoded)
}

fn read_request(stream: &mut UnixStream) -> std::io::Result<Request> {
    let mut length = [0; 4];
    stream.read_exact(&mut length)?;
    let length = u32::from_be_bytes(length) as usize;
    if length == 0 || length > MAX_RESULT {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "request limit",
        ));
    }
    let mut data = vec![0; length];
    stream.read_exact(&mut data)?;
    let request: Request = serde_json::from_slice(&data)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "typed request"))?;
    if request.kind != "dynamic-analysis-request-v1"
        || Uuid::parse_str(&request.run_id).is_err()
        || request.byte_size == 0
        || request.byte_size as usize > MAX_BYTES
        || request.sha256.len() != 64
        || !request
            .sha256
            .bytes()
            .all(|value| value.is_ascii_hexdigit())
        || !request.static_runtime
        || !matches!(request.elf_type.as_str(), "executable" | "shared-object")
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "closed request",
        ));
    }
    Ok(request)
}

struct PeerCredentials {
    uid: u32,
}
fn peer_credentials(stream: &UnixStream) -> std::io::Result<PeerCredentials> {
    let mut credentials: libc::ucred = unsafe { std::mem::zeroed() };
    let mut length = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    let result = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            &mut credentials as *mut _ as *mut libc::c_void,
            &mut length,
        )
    };
    if result != 0 || length as usize != std::mem::size_of::<libc::ucred>() {
        return Err(std::io::Error::last_os_error());
    }
    Ok(PeerCredentials {
        uid: credentials.uid,
    })
}

fn run(request: &Request, bytes: &[u8]) -> ResultMessage {
    let start = Instant::now();
    let base = ResultMessage {
        kind: "dynamic-analysis-result-v1",
        schema_version: 1,
        run_id: request.run_id.clone(),
        outcome: "setup-failed",
        elapsed_ms: 0,
        guest_started: false,
        resource_limits: RESOURCE_LIMITS
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
    };
    if format!("{:x}", Sha256::digest(bytes)) != request.sha256 || !static_elf64(bytes) {
        return base;
    }
    let run_dir = PathBuf::from(ROOT)
        .join("firecracker")
        .join(&request.run_id)
        .join("root");
    if fs::create_dir_all(&run_dir).is_err() {
        return base;
    }
    let input = run_dir.join("input.raw");
    let mut raw = b"QFELF001".to_vec();
    raw.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    raw.extend_from_slice(bytes);
    if fs::write(&input, raw).is_err() {
        remove_run(&run_dir);
        return base;
    }
    let serial = run_dir.join("serial.log");
    if fs::File::create(&serial).is_err() {
        remove_run(&run_dir);
        return base;
    }
    let config = run_dir.join("config.json");
    let assets = Path::new(ASSETS);
    let kernel = assets.join("vmlinux");
    let initrd = assets.join("initramfs.cpio.gz");
    if !kernel.is_file()
        || !initrd.is_file()
        || !assets.join("firecracker").is_file()
        || !assets.join("jailer").is_file()
    {
        remove_run(&run_dir);
        return base;
    }
    for (source, name) in [(&kernel, "vmlinux"), (&initrd, "initramfs.cpio.gz")] {
        if fs::hard_link(source, run_dir.join(name)).is_err() {
            remove_run(&run_dir);
            return base;
        }
    }
    let config_value = serde_json::json!({"boot-source":{"kernel_image_path":"/vmlinux","initrd_path":"/initramfs.cpio.gz","boot_args":"console=ttyS0 rdinit=/init panic=-1"},"machine-config":{"vcpu_count":1,"mem_size_mib":512,"smt":false},"drives":[{"drive_id":"input","path_on_host":"/input.raw","is_root_device":false,"is_read_only":true}],"serial":{"serial_out_path":"/serial.log"}});
    if fs::write(&config, serde_json::to_vec(&config_value).expect("config")).is_err() {
        remove_run(&run_dir);
        return base;
    }
    let mut child = match Command::new(assets.join("jailer"))
        .args([
            "--id",
            &request.run_id,
            "--exec-file",
            "/usr/lib/quireforge-sandboxd/firecracker",
            "--uid",
            "65534",
            "--gid",
            "65534",
            "--chroot-base-dir",
            ROOT,
            "--cgroup-version",
            "2",
            "--new-pid-ns",
            "--",
            "--config-file",
            "/config.json",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => {
            remove_run(&run_dir);
            return base;
        }
    };
    let mut timed_out = false;
    while child.try_wait().ok().flatten().is_none() {
        if start.elapsed() > Duration::from_secs(30) {
            let _ = child.kill();
            let _ = child.wait();
            timed_out = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    let serial_text = fs::read_to_string(&serial).unwrap_or_default();
    let outcome = if timed_out {
        "timeout"
    } else if serial_text.contains("QF_DYNAMIC_RESULT_V1:completed") {
        "completed"
    } else if serial_text.contains("QF_DYNAMIC_RESULT_V1:nonzero-exit") {
        "nonzero-exit"
    } else if serial_text.contains("QF_DYNAMIC_RESULT_V1:signal") {
        "signal"
    } else {
        "setup-failed"
    };
    let elapsed_ms = start.elapsed().as_millis().min(30_000) as u32;
    remove_run(&run_dir);
    ResultMessage {
        outcome,
        elapsed_ms,
        guest_started: outcome != "setup-failed",
        ..base
    }
}

fn remove_run(run_root: &Path) {
    if let Some(run) = run_root.parent() {
        let _ = fs::remove_dir_all(run);
    }
}

fn static_elf64(bytes: &[u8]) -> bool {
    if bytes.len() < 64
        || &bytes[..4] != b"\x7fELF"
        || bytes[4] != 2
        || bytes[5] != 1
        || bytes[6] != 1
        || u16::from_le_bytes([bytes[18], bytes[19]]) != 62
        || !matches!(u16::from_le_bytes([bytes[16], bytes[17]]), 2 | 3)
    {
        return false;
    }
    let off = u64::from_le_bytes(bytes[32..40].try_into().unwrap());
    let size = u16::from_le_bytes(bytes[54..56].try_into().unwrap()) as u64;
    let count = u16::from_le_bytes(bytes[56..58].try_into().unwrap()) as u64;
    if size < 56
        || count > 256
        || off
            .checked_add(size.saturating_mul(count))
            .is_none_or(|end| end > bytes.len() as u64)
    {
        return false;
    }
    !(0..count).any(|i| {
        let start = (off + i * size) as usize;
        u32::from_le_bytes(bytes[start..start + 4].try_into().unwrap()) == 3
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn accepts_only_static_x86_64_exec_or_static_pie() {
        assert!(static_elf64(&elf(2, false)));
        assert!(static_elf64(&elf(3, false)));
        assert!(!static_elf64(&elf(3, true)));
        assert!(!static_elf64(&elf(4, false)));
    }

    #[test]
    fn closed_request_rejects_untrusted_or_dynamic_values() {
        let request = serde_json::json!({
            "kind": "dynamic-analysis-request-v1",
            "runId": Uuid::now_v7().to_string(),
            "sha256": "a".repeat(64),
            "byteSize": 64,
            "elfType": "shared-object",
            "staticRuntime": true
        });
        let valid: Request = serde_json::from_value(request).unwrap();
        assert_eq!(valid.kind, "dynamic-analysis-request-v1");
        assert!(!serde_json::from_value::<Request>(serde_json::json!({
            "kind": "dynamic-analysis-request-v1",
            "runId": Uuid::now_v7().to_string(),
            "sha256": "a".repeat(64),
            "byteSize": 64,
            "elfType": "shared-object",
            "staticRuntime": true,
            "path": "/outside"
        }))
        .is_ok());
    }
}
