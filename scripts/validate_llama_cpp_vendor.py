#!/usr/bin/env python3
"""Validate the closed M63 llama.cpp source boundary without network access."""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
VENDOR = ROOT / "third_party" / "llama.cpp"
MANIFEST = VENDOR / "PROVENANCE.json"
BUILD_SCRIPT = ROOT / "apps" / "desktop" / "src-tauri" / "build.rs"
EXPECTED_COMMIT = "3653e6d6d547ec763317d9ecd0ace334a7e21359"
EXPECTED_FINGERPRINT = "968479A1AFF927E37D1A566BB5690EEEBB952194"
EXPECTED_MANIFEST = {
    "schema_version": 1,
    "upstream_repository": "https://github.com/ggml-org/llama.cpp.git",
    "release": "b10326",
    "upstream_commit": EXPECTED_COMMIT,
    "signing_key_endpoint": "https://github.com/web-flow.gpg",
    "required_fingerprint": EXPECTED_FINGERPRINT,
    "observed_fingerprint": EXPECTED_FINGERPRINT,
    "verification_result": (
        "git verify-commit succeeded with a cryptographically good GitHub "
        "web-flow signature in a temporary keyring"
    ),
    "source_archive_sha256": "fcfe5963280153da830bed0e1dd06fa9bcba1de27155dd2afc731ab05348f9de",
    "tree_digest_scope": (
        "all regular vendored files except this manifest; SHA-256 of sorted UTF-8 "
        "relative path, NUL, file SHA-256 bytes, NUL"
    ),
    "license_identifier": "MIT",
    "license_files": ["LICENSE", "licenses/LICENSE-jsonhpp"],
    "acquisition_date": "2026-08-07",
    "source_only": True,
    "model_artifact_included": False,
}
EXPECTED_CMAKE_OPTIONS = {
    "-DBUILD_SHARED_LIBS=OFF",
    "-DLLAMA_BUILD_NUMBER=10326",
    f"-DLLAMA_BUILD_COMMIT={EXPECTED_COMMIT}",
    "-DGGML_BUILD_NUMBER=10326",
    f"-DGGML_BUILD_COMMIT={EXPECTED_COMMIT}",
    "-DLLAMA_BUILD_COMMON=OFF",
    "-DLLAMA_BUILD_TESTS=OFF",
    "-DLLAMA_BUILD_TOOLS=OFF",
    "-DLLAMA_BUILD_EXAMPLES=OFF",
    "-DLLAMA_BUILD_SERVER=OFF",
    "-DLLAMA_BUILD_APP=OFF",
    "-DLLAMA_BUILD_UI=OFF",
    "-DLLAMA_USE_PREBUILT_UI=OFF",
    "-DLLAMA_BUILD_MTMD=OFF",
    "-DLLAMA_TOOLS_INSTALL=OFF",
    "-DLLAMA_TESTS_INSTALL=OFF",
    "-DLLAMA_OPENSSL=OFF",
    "-DLLAMA_SUBPROCESS=OFF",
    "-DLLAMA_USE_SYSTEM_GGML=OFF",
    "-DGGML_STATIC=ON",
    "-DGGML_BACKEND_DL=OFF",
    "-DGGML_CPU=ON",
    "-DGGML_NATIVE=OFF",
    "-DGGML_BLAS=OFF",
    "-DGGML_ACCELERATE=OFF",
    "-DGGML_OPENMP=OFF",
    "-DGGML_CCACHE=OFF",
    "-DGGML_CUDA=OFF",
    "-DGGML_HIP=OFF",
    "-DGGML_VULKAN=OFF",
    "-DGGML_KOMPUTE=OFF",
    "-DGGML_METAL=OFF",
    "-DGGML_SYCL=OFF",
    "-DGGML_CANN=OFF",
    "-DGGML_MUSA=OFF",
    "-DGGML_OPENCL=OFF",
    "-DGGML_WEBGPU=OFF",
    "-DGGML_OPENVINO=OFF",
    "-DGGML_ET=OFF",
    "-DGGML_HEXAGON=OFF",
    "-DGGML_ZDNN=OFF",
    "-DGGML_VIRTGPU=OFF",
    "-DGGML_RPC=OFF",
}


def tree_digest() -> str:
    digest = hashlib.sha256()
    paths = sorted(VENDOR.rglob("*"))
    for path in paths:
        require(not path.is_symlink(), f"vendored symlink found: {path.relative_to(VENDOR)}")
    for path in (path for path in paths if path.is_file() and path != MANIFEST):
        relative = path.relative_to(VENDOR).as_posix().encode("utf-8")
        digest.update(relative)
        digest.update(b"\0")
        digest.update(hashlib.sha256(path.read_bytes()).digest())
        digest.update(b"\0")
    return digest.hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"M63 llama.cpp validation failed: {message}")


def cmake_options(build: str) -> set[str]:
    match = re.search(
        r"const LLAMA_CPP_CMAKE_OPTIONS: &\[&str\] = &\[(?P<options>.*?)\];",
        build,
        flags=re.DOTALL,
    )
    require(match is not None, "closed CMake option list is missing")
    return set(re.findall(r'"(-D[^"\\]+)"', match.group("options")))


def main() -> None:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    require(set(manifest) == set(EXPECTED_MANIFEST) | {"vendored_tree_sha256"}, "unexpected provenance manifest schema")
    for field, expected in EXPECTED_MANIFEST.items():
        require(manifest.get(field) == expected, f"unexpected provenance value: {field}")
    require(manifest["vendored_tree_sha256"] == tree_digest(), "vendored tree digest mismatch")
    for license_path in manifest["license_files"]:
        require((VENDOR / license_path).is_file(), f"missing license evidence: {license_path}")
    require(not (VENDOR / ".git").exists(), "Git history was vendored")
    prohibited_suffixes = {".gguf", ".safetensors", ".bin"}
    for path in VENDOR.rglob("*"):
        require(path.suffix.lower() not in prohibited_suffixes, f"model artifact found: {path.relative_to(VENDOR)}")
    build = BUILD_SCRIPT.read_text(encoding="utf-8")
    require(cmake_options(build) == EXPECTED_CMAKE_OPTIONS, "closed CMake options changed")
    print("M63 llama.cpp vendor validation passed.")


if __name__ == "__main__":
    main()
