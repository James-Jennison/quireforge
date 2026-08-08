#!/usr/bin/env python3
"""Validate the closed M63 llama.cpp source boundary without network access."""

from __future__ import annotations

import hashlib
import json
import re
import stat
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
    "-DLLAMA_CURL=OFF",
    "-DLLAMA_OPENSSL=OFF",
    "-DLLAMA_SUBPROCESS=OFF",
    "-DLLAMA_USE_SYSTEM_GGML=OFF",
    "-DGGML_STATIC=ON",
    "-DGGML_BACKEND_DL=OFF",
    "-DGGML_CPU=ON",
    "-DGGML_LLAMAFILE=OFF",
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
    "-DGIT_EXE=",
}
ALLOWED_CONFIGURE_DEFINITIONS = {"-DCMAKE_BUILD_TYPE=Release"}
EXPECTED_SOURCE_DIR_EXPRESSION = 'manifest_dir.join("../../../third_party/llama.cpp")'
EXPECTED_SOURCE_TRACKER = "register_vendored_source_tree(&source_dir);"
EXPECTED_SOURCE_ROOT_INSPECTION = "fs::symlink_metadata(&source_dir)"
EXPECTED_LINK_SEARCH_DIRECTORIES = [
    'build_dir.join("src")',
    'build_dir.join("ggml/src")',
]
EXPECTED_STATIC_LIBRARIES = ["llama", "ggml", "ggml-base", "ggml-cpu"]
EXPECTED_CLOSED_CMAKE_ENVIRONMENT = {
    "CC",
    "CXX",
    "CPPFLAGS",
    "CFLAGS",
    "CXXFLAGS",
    "LDFLAGS",
    "CMAKE_GENERATOR",
    "CMAKE_GENERATOR_INSTANCE",
    "CMAKE_GENERATOR_PLATFORM",
    "CMAKE_GENERATOR_TOOLSET",
    "CMAKE_TOOLCHAIN_FILE",
    "CMAKE_PREFIX_PATH",
    "CMAKE_INCLUDE_PATH",
    "CMAKE_LIBRARY_PATH",
    "CMAKE_PROGRAM_PATH",
    "CMAKE_FRAMEWORK_PATH",
    "CMAKE_APPBUNDLE_PATH",
    "CMAKE_PROJECT_INCLUDE_BEFORE",
    "CMAKE_PROJECT_INCLUDE",
    "CMAKE_PROJECT_TOP_LEVEL_INCLUDES",
    "CMAKE_USER_MAKE_RULES_OVERRIDE",
    "CMAKE_USER_MAKE_RULES_OVERRIDE_C",
    "CMAKE_USER_MAKE_RULES_OVERRIDE_CXX",
    "MAKEFLAGS",
    "MFLAGS",
    "GNUMAKEFLAGS",
}


def tree_digest() -> str:
    require_regular_vendored_source_tree(VENDOR)
    digest = hashlib.sha256()
    paths = sorted(VENDOR.rglob("*"))
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


def require_regular_vendored_source_tree(directory: Path) -> None:
    root_metadata = directory.lstat()
    require(
        not stat.S_ISLNK(root_metadata.st_mode) and stat.S_ISDIR(root_metadata.st_mode),
        "vendored source root must be a real directory",
    )
    for path in sorted(directory.rglob("*")):
        metadata = path.lstat()
        relative = path.relative_to(directory)
        require(not stat.S_ISLNK(metadata.st_mode), f"vendored symlink found: {relative}")
        require(
            stat.S_ISDIR(metadata.st_mode) or stat.S_ISREG(metadata.st_mode),
            f"vendored source contains a non-regular entry: {relative}",
        )


def cmake_options(build: str) -> set[str]:
    match = re.search(
        r"const LLAMA_CPP_CMAKE_OPTIONS: &\[&str\] = &\[(?P<options>.*?)\];",
        build,
        flags=re.DOTALL,
    )
    require(match is not None, "closed CMake option list is missing")
    return set(re.findall(r'"(-D[^"\\]+)"', match.group("options")))


def require_verified_cmake_source(build: str) -> None:
    require(
        EXPECTED_SOURCE_DIR_EXPRESSION in build,
        "CMake source is not the verified vendored llama.cpp tree",
    )


def require_complete_vendored_source_tracking(build: str) -> None:
    require(
        EXPECTED_SOURCE_ROOT_INSPECTION in build,
        "build script does not inspect the vendored source root",
    )
    require(
        "!source_metadata.file_type().is_symlink() && source_metadata.is_dir()" in build,
        "build script must reject a symlinked or non-directory vendored source root",
    )
    require(
        EXPECTED_SOURCE_TRACKER in build,
        "build script does not track every verified vendored source file",
    )
    require(
        "println!(\"cargo:rerun-if-changed={}\", source_dir.display());" not in build,
        "build script must not rely on directory-only vendored source tracking",
    )
    tracker_match = re.search(
        r"fn register_vendored_source_tree\s*\(directory: &Path\)\s*\{(?P<body>.*?)\n\}",
        build,
        flags=re.DOTALL,
    )
    require(tracker_match is not None, "vendored source tracker is missing")
    tracker = tracker_match.group("body")
    require(
        "!file_type.is_symlink()" in tracker,
        "vendored source tracker must reject symlinks",
    )
    require(
        "register_vendored_source_tree(&path);" in tracker,
        "vendored source tracker must recurse into every directory",
    )
    require(
        'println!("cargo:rerun-if-changed={}", path.display());' in tracker,
        "vendored source tracker must register every regular file",
    )


def require_closed_build_process_boundary(build: str) -> None:
    command_calls = re.findall(r"Command::new\s*\(", build)
    command_executables = re.findall(
        r'Command::new\s*\(\s*"([^"\\]+)"\s*\)', build
    )
    require(
        len(command_calls) == 2 and command_executables == ["cmake", "cmake"],
        "build script must start only the two approved CMake subprocesses",
    )


def command_arguments(body: str) -> list[str]:
    return [
        literal or variable
        for literal, variable in re.findall(
            r'\.arg\((?:"([^\"]+)"|(&[a-z_]+))\)', body
        )
    ]


def require_verified_cmake_configure_arguments(build: str) -> None:
    configure_match = re.search(
        r"let mut configure\s*=\s*Command::new\(\"cmake\"\);(?P<body>.*?)"
        r"run\(\s*&mut configure,\s*\"closed llama\.cpp static-library configuration\",?\s*\);",
        build,
        flags=re.DOTALL,
    )
    require(configure_match is not None, "closed CMake configuration invocation is missing")
    require(
        command_arguments(configure_match.group("body"))
        == ["-S", "&source_dir", "-B", "&build_dir", "-DCMAKE_BUILD_TYPE=Release"],
        "closed CMake configuration must use only the verified source and private build directory",
    )


def require_closed_cmake_invocation(build: str) -> None:
    configure_match = re.search(
        r"let mut configure\s*=\s*Command::new\(\"cmake\"\);(?P<body>.*?)"
        r"run\(\s*&mut configure,\s*\"closed llama\.cpp static-library configuration\",?\s*\);",
        build,
        flags=re.DOTALL,
    )
    require(configure_match is not None, "closed CMake configuration invocation is missing")
    configure_body = configure_match.group("body")
    require(
        len(re.findall(r"configure\.args\(LLAMA_CPP_CMAKE_OPTIONS\);", configure_body)) == 1,
        "closed CMake options are not applied exactly once",
    )
    unguarded_definitions = set(re.findall(r'\.arg\("(-D[^"\\]+)"\)', configure_body))
    require(
        unguarded_definitions == ALLOWED_CONFIGURE_DEFINITIONS,
        "closed CMake configuration includes an unguarded -D option",
    )


def require_closed_cmake_environment(build: str) -> None:
    environment_match = re.search(
        r"const CLOSED_CMAKE_ENVIRONMENT: &\[&str\] = &\[(?P<variables>.*?)\];",
        build,
        flags=re.DOTALL,
    )
    require(environment_match is not None, "closed CMake environment list is missing")
    environment_variables = set(re.findall(r'"([^"\\]+)"', environment_match.group("variables")))
    require(
        environment_variables == EXPECTED_CLOSED_CMAKE_ENVIRONMENT,
        "closed CMake environment list changed",
    )
    for command_name, description in (
        ("configure", "closed llama.cpp static-library configuration"),
        ("build", "closed llama.cpp static-library build"),
    ):
        command_match = re.search(
            rf"let mut {command_name}\s*=\s*Command::new\(\"cmake\"\);(?P<body>.*?)"
            rf"run\(\s*&mut {command_name},\s*\"{re.escape(description)}\",?\s*\);",
            build,
            flags=re.DOTALL,
        )
        require(command_match is not None, f"{description} invocation is missing")
        require(
            len(
                re.findall(
                    rf"for variable in CLOSED_CMAKE_ENVIRONMENT \{{\s*"
                    rf"{command_name}\.env_remove\(variable\);\s*\}}",
                    command_match.group("body"),
                )
            )
            == 1,
            f"{description} must remove the inherited toolchain environment exactly once",
        )


def require_closed_cmake_build_invocation(build: str) -> None:
    build_match = re.search(
        r"let mut build\s*=\s*Command::new\(\"cmake\"\);(?P<body>.*?)"
        r"run\(\s*&mut build,\s*\"closed llama\.cpp static-library build\",?\s*\);",
        build,
        flags=re.DOTALL,
    )
    require(build_match is not None, "closed CMake build invocation is missing")
    build_arguments = command_arguments(build_match.group("body"))
    require(
        build_arguments == ["--build", "&build_dir", "--target", "llama"],
        "closed CMake build must target only the static llama library",
    )


def require_closed_cargo_linkage(build: str) -> None:
    link_search_directories = re.findall(
        r'cargo:rustc-link-search=native=\{\}\",\s*\n\s*([^\n]+)\.display\(\)',
        build,
    )
    require(
        len(re.findall(r'cargo:rustc-link-search=', build)) == 2,
        "closed Cargo linkage includes an unexpected native library search directive",
    )
    require(
        link_search_directories == EXPECTED_LINK_SEARCH_DIRECTORIES,
        "closed Cargo linkage must search only the private static-library directories",
    )
    library_match = re.search(r'for library in \[(?P<libraries>[^]]+)\]', build)
    require(library_match is not None, "closed static library list is missing")
    static_libraries = re.findall(r'"([^"]+)"', library_match.group("libraries"))
    require(
        static_libraries == EXPECTED_STATIC_LIBRARIES,
        "closed Cargo linkage includes an unexpected static library",
    )
    require(
        len(re.findall(r'cargo:rustc-link-lib=static=\{library\}', build)) == 1,
        "closed Cargo linkage must emit the static library list exactly once",
    )
    require(
        len(re.findall(r'cargo:rustc-link-lib=dylib=stdc\+\+', build)) == 1,
        "closed Cargo linkage must use only the standard C++ runtime dynamically",
    )
    link_libraries = re.findall(r'cargo:rustc-link-lib=([^"\\]+)', build)
    require(
        set(link_libraries) == {"static={library}", "dylib=stdc++"},
        "closed Cargo linkage includes an unexpected native library directive",
    )


def main() -> None:
    require_regular_vendored_source_tree(VENDOR)
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
    require_verified_cmake_source(build)
    require_complete_vendored_source_tracking(build)
    require_closed_build_process_boundary(build)
    require_verified_cmake_configure_arguments(build)
    require_closed_cmake_invocation(build)
    require_closed_cmake_environment(build)
    require_closed_cmake_build_invocation(build)
    require_closed_cargo_linkage(build)
    print("M63 llama.cpp vendor validation passed.")


if __name__ == "__main__":
    main()
