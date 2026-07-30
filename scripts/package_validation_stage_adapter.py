#!/usr/bin/env python3
"""Private, nonce-bound adapter for native package-validation stages.

This is not a general validator: the desktop process chooses one of five
closed stages and supplies a private result file.  The emitted JSON never
contains candidate paths, filenames, commands, or diagnostics.
"""
from __future__ import annotations
import argparse, hashlib, json, os, re, stat
from pathlib import Path

from validate_release_artifacts import (
    validate_abi_evidence, validate_debian, validate_manifest_provenance,
    validate_sandboxd, validate_sandboxd_provenance, smoke_packages,
)

STAGES = ("manifest", "checksum", "abi", "provenance", "visible-launch")
VERSION = re.compile(r"^[0-9][0-9A-Za-z.+-]{0,63}$")
DEBIAN_VERSION = re.compile(r"^[0-9][0-9A-Za-z.+:~-]{0,63}$")

class CandidateUnavailable(Exception): pass

def digest(value: dict[str, object]) -> str:
    unsigned = {key: value[key] for key in (
        "schemaVersion", "sessionId", "nonce", "stage", "outcome",
        "applicationVersion", "debianVersion", "artifactCount")}
    if "facts" in value: unsigned["facts"]=value["facts"]
    return hashlib.sha256(json.dumps(unsigned, separators=(",", ":"), sort_keys=True).encode()).hexdigest()

def candidate(root: Path, verify_bytes: bool) -> tuple[str, str, int]:
    root = root.resolve(strict=True)
    manifest_path = root / "release-manifest.json"
    if not manifest_path.exists(): raise CandidateUnavailable("manifest")
    if manifest_path.is_symlink() or not manifest_path.is_file(): raise ValueError("manifest")
    data = json.loads(manifest_path.read_text(encoding="utf-8"))
    version = data.get("version")
    artifacts = data.get("artifacts")
    if data.get("schemaVersion") != 3 or data.get("state") != "release-candidate" or not isinstance(version, str) or not VERSION.fullmatch(version) or not isinstance(artifacts, list) or len(artifacts) != 2: raise ValueError("shape")
    expected = {"deb", "sandboxd-deb"}; seen=set()
    for record in artifacts:
        if not isinstance(record, dict) or set(record) - {"format","filename","architecture","packageVersion","sha256","size"}: raise ValueError("artifact")
        name=record.get("filename"); kind=record.get("format"); sha=record.get("sha256")
        if kind not in expected or kind in seen or not isinstance(name,str) or "/" in name or name in {"", ".", ".."} or not isinstance(sha,str) or not re.fullmatch(r"[0-9a-f]{64}",sha): raise ValueError("artifact")
        path=(root/name).resolve(strict=True)
        if path.parent != root or path.is_symlink() or not stat.S_ISREG(path.stat().st_mode): raise ValueError("artifact")
        if verify_bytes and hashlib.sha256(path.read_bytes()).hexdigest()!=sha: raise ValueError("digest")
        seen.add(kind)
    if seen != expected: raise ValueError("formats")
    debian = "~".join(version.split("-",1)) if "-" in version else version
    if not DEBIAN_VERSION.fullmatch(debian): raise ValueError("version")
    return version, debian, 2

def abi(root: Path) -> dict[str, object]:
    root = root.resolve(strict=True)
    version, _, _ = candidate(root, True)
    data = json.loads((root / "release-manifest.json").read_text(encoding="utf-8"))
    artifacts = {entry["format"]: root / entry["filename"] for entry in data["artifacts"]}
    debian = validate_debian(artifacts["deb"], version)
    sandboxd = validate_sandboxd(artifacts["sandboxd-deb"], version)
    validate_abi_evidence(data, debian, sandboxd)
    return {"kind":"abi","schema_version":1,"glibc_baseline":"GLIBC_2.35","highest_required":f"GLIBC_{max(debian, sandboxd)[0]}.{max(debian, sandboxd)[1]}"}

def provenance(root: Path) -> dict[str, object]:
    root = root.resolve(strict=True)
    version, _, _ = candidate(root, True)
    data = json.loads((root / "release-manifest.json").read_text(encoding="utf-8"))
    artifacts = {entry["format"]: root / entry["filename"] for entry in data["artifacts"]}
    validate_manifest_provenance(data)
    validate_sandboxd_provenance(data, artifacts, version)
    return {"kind":"provenance","schema_version":1,"evidence_state":"pinned-release-candidate","artifact_coverage":2,"identity_consistent":True}

def visible_launch(root: Path) -> dict[str, object]:
    root = root.resolve(strict=True)
    candidate(root, True)
    data = json.loads((root / "release-manifest.json").read_text(encoding="utf-8"))
    artifacts = {entry["format"]: root / entry["filename"] for entry in data["artifacts"]}
    smoke_packages(artifacts["deb"])
    return {"kind":"visible-launch","schema_version":1,"launch_state":"visible-window-confirmed","artifact_coverage":1,"visibility_confirmed":True,"lifecycle_clean":True}

def evaluate_stage(root: Path, stage: str) -> tuple[str, str, int, str, dict[str, object] | None]:
    """Run one closed stage and reduce all tool detail to protocol-safe state."""
    try:
        version, debian, count = candidate(root, stage == "checksum")
        if stage == "abi":
            return version, debian, count, "passed", abi(root)
        if stage == "provenance":
            return version, debian, count, "passed", provenance(root)
        if stage == "visible-launch":
            return version, debian, count, "passed", visible_launch(root)
        return version, debian, count, ("passed" if stage in {"manifest", "checksum"} else "unavailable"), None
    except (CandidateUnavailable, FileNotFoundError):
        return "0", "0", 0, "unavailable", None
    except Exception:
        return "0", "0", 0, "failed", None

def main() -> int:
    p=argparse.ArgumentParser(); p.add_argument("--stage", choices=STAGES, required=True); p.add_argument("--session-id", required=True); p.add_argument("--nonce", required=True); p.add_argument("--candidate-root", type=Path, required=True); p.add_argument("--result-file", type=Path, required=True); a=p.parse_args()
    if not re.fullmatch(r"[0-9a-f-]{36}",a.session_id) or not re.fullmatch(r"[0-9a-f]{64}",a.nonce): return 2
    version, debian, count, outcome, facts = evaluate_stage(a.candidate_root, a.stage)
    value={"schemaVersion":1,"sessionId":a.session_id,"nonce":a.nonce,"stage":a.stage,"outcome":outcome,"applicationVersion":version,"debianVersion":debian,"artifactCount":count}
    if facts is not None: value["facts"]=facts
    value["resultSha256"]=digest(value)
    a.result_file.write_text(json.dumps(value,separators=(",",":"),sort_keys=True),encoding="utf-8")
    os.chmod(a.result_file,0o600)
    return 0
if __name__ == "__main__": raise SystemExit(main())
