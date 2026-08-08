#!/usr/bin/env python3
"""Validate the closed M63 llama.cpp source boundary without network access."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
VENDOR = ROOT / "third_party" / "llama.cpp"
MANIFEST = VENDOR / "PROVENANCE.json"
BUILD_SCRIPT = ROOT / "apps" / "desktop" / "src-tauri" / "build.rs"
EXPECTED_COMMIT = "3653e6d6d547ec763317d9ecd0ace334a7e21359"
EXPECTED_FINGERPRINT = "968479A1AFF927E37D1A566BB5690EEEBB952194"


def tree_digest() -> str:
    digest = hashlib.sha256()
    for path in sorted(path for path in VENDOR.rglob("*") if path.is_file() and path != MANIFEST):
        relative = path.relative_to(VENDOR).as_posix().encode("utf-8")
        digest.update(relative)
        digest.update(b"\0")
        digest.update(hashlib.sha256(path.read_bytes()).digest())
        digest.update(b"\0")
    return digest.hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"M63 llama.cpp validation failed: {message}")


def main() -> None:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    require(manifest["upstream_commit"] == EXPECTED_COMMIT, "unexpected upstream commit")
    require(manifest["observed_fingerprint"] == EXPECTED_FINGERPRINT, "unexpected signing fingerprint")
    require(manifest["source_only"] is True, "source-only declaration is missing")
    require(manifest["vendored_tree_sha256"] == tree_digest(), "vendored tree digest mismatch")
    for license_path in manifest["license_files"]:
        require((VENDOR / license_path).is_file(), f"missing license evidence: {license_path}")
    require(not (VENDOR / ".git").exists(), "Git history was vendored")
    prohibited_suffixes = {".gguf", ".safetensors", ".bin"}
    for path in VENDOR.rglob("*"):
        require(path.suffix.lower() not in prohibited_suffixes, f"model artifact found: {path.relative_to(VENDOR)}")
    build = BUILD_SCRIPT.read_text(encoding="utf-8")
    required_options = [
        "-DBUILD_SHARED_LIBS=OFF", "-DLLAMA_BUILD_COMMON=OFF", "-DLLAMA_BUILD_TESTS=OFF",
        "-DLLAMA_BUILD_TOOLS=OFF", "-DLLAMA_BUILD_EXAMPLES=OFF", "-DLLAMA_BUILD_SERVER=OFF",
        "-DLLAMA_BUILD_APP=OFF", "-DLLAMA_BUILD_UI=OFF", "-DLLAMA_OPENSSL=OFF",
        "-DLLAMA_SUBPROCESS=OFF", "-DGGML_BACKEND_DL=OFF", "-DGGML_CPU=ON",
        "-DGGML_CUDA=OFF", "-DGGML_HIP=OFF", "-DGGML_VULKAN=OFF", "-DGGML_METAL=OFF",
        "-DGGML_SYCL=OFF", "-DGGML_OPENCL=OFF", "-DGGML_RPC=OFF",
    ]
    for option in required_options:
        require(option in build, f"closed build option missing: {option}")
    print("M63 llama.cpp vendor validation passed.")


if __name__ == "__main__":
    main()
