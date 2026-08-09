#!/usr/bin/env python3
"""Validate the closed M63 llama.cpp source boundary without network access."""

from __future__ import annotations

import hashlib
import json
import re
import stat
from collections import Counter
from itertools import chain
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
VENDOR = ROOT / "third_party" / "llama.cpp"
MANIFEST = VENDOR / "PROVENANCE.json"
BUILD_SCRIPT = ROOT / "apps" / "desktop" / "src-tauri" / "build.rs"
NATIVE_SOURCE = ROOT / "apps" / "desktop" / "src-tauri" / "src"
GITIGNORE = ROOT / ".gitignore"
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
    "-DCMAKE_C_COMPILER=/usr/bin/cc",
    "-DCMAKE_CXX_COMPILER=/usr/bin/c++",
    "-DCMAKE_C_COMPILER_AR=/usr/bin/ar",
    "-DCMAKE_CXX_COMPILER_AR=/usr/bin/ar",
    "-DCMAKE_C_COMPILER_RANLIB=/usr/bin/ranlib",
    "-DCMAKE_CXX_COMPILER_RANLIB=/usr/bin/ranlib",
    "-DCMAKE_MAKE_PROGRAM=/usr/bin/make",
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
    "-DGIT_EXECUTABLE=",
    "-DCMAKE_DISABLE_FIND_PACKAGE_Git=ON",
    "-DCMAKE_FIND_USE_PACKAGE_REGISTRY=OFF",
    "-DCMAKE_FIND_USE_SYSTEM_PACKAGE_REGISTRY=OFF",
    "-DCMAKE_FIND_USE_CMAKE_PATH=OFF",
    "-DCMAKE_FIND_USE_CMAKE_ENVIRONMENT_PATH=OFF",
    "-DCMAKE_FIND_USE_SYSTEM_ENVIRONMENT_PATH=OFF",
    "-DCMAKE_FIND_USE_CMAKE_SYSTEM_PATH=OFF",
    "-DCMAKE_FIND_USE_INSTALL_PREFIX=OFF",
    "-DCMAKE_FIND_PACKAGE_NO_PACKAGE_REGISTRY=ON",
    "-DCMAKE_FIND_PACKAGE_NO_SYSTEM_PACKAGE_REGISTRY=ON",
    "-DCMAKE_EXPORT_NO_PACKAGE_REGISTRY=ON",
    "-DCMAKE_DISABLE_SOURCE_CHANGES=ON",
    "-DCMAKE_DISABLE_IN_SOURCE_BUILD=ON",
}
ALLOWED_CONFIGURE_DEFINITIONS = {"-DCMAKE_BUILD_TYPE=Release"}
EXPECTED_SOURCE_DIR_EXPRESSION = 'manifest_dir.join("../../../third_party/llama.cpp")'
EXPECTED_SOURCE_DIR_DECLARATION = (
    'let source_dir = manifest_dir.join("../../../third_party/llama.cpp");'
)
EXPECTED_OUTPUT_DIR_DECLARATION = (
    'let output_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"));'
)
EXPECTED_BUILD_DIR_DECLARATION = 'let build_dir = output_dir.join("m63-llama.cpp-build");'
EXPECTED_SOURCE_TRACKER = "register_vendored_source_tree(&source_dir);"
EXPECTED_SOURCE_ROOT_INSPECTION = "fs::symlink_metadata(directory)"
EXPECTED_SOURCE_ROOT_VERIFIER = "require_vendored_source_root(directory);"
EXPECTED_VENDORED_TREE_SHA256 = "9892c22a1a05adf0775615f1b845886f8f1be96ad7b6f71093103eaec546a511"
EXPECTED_SOURCE_DIGEST_VERIFIER = "verify_vendored_tree_digest(&source_dir);"
EXPECTED_LINK_SEARCH_DIRECTORIES = [
    'build_dir.join("src")',
    'build_dir.join("ggml/src")',
]
EXPECTED_STATIC_LIBRARIES = ["llama", "ggml", "ggml-base", "ggml-cpu"]
EXPECTED_CARGO_DIRECTIVES = Counter(
    {
        "cargo:rerun-if-changed={}": 2,
        "cargo:rustc-link-search=native={}": 2,
        "cargo:rustc-link-lib=static={library}": 1,
        "cargo:rustc-link-lib=dylib=stdc++": 1,
    }
)
EXPECTED_CLOSED_BUILD_PATH = "/usr/bin:/bin"
EXPECTED_SYSTEM_CMAKE = "/usr/bin/cmake"
EXPECTED_CLOSED_CMAKE_ENVIRONMENT = {
    "CC",
    "CXX",
    "AR",
    "RANLIB",
    "NM",
    "STRIP",
    "OBJCOPY",
    "READELF",
    "CPPFLAGS",
    "CFLAGS",
    "CXXFLAGS",
    "LDFLAGS",
    "CPATH",
    "C_INCLUDE_PATH",
    "CPLUS_INCLUDE_PATH",
    "OBJC_INCLUDE_PATH",
    "LIBRARY_PATH",
    "COMPILER_PATH",
    "GCC_EXEC_PREFIX",
    "GCC_SPECS",
    "CCC_OVERRIDE_OPTIONS",
    "CLANG_CONFIG_FILE",
    "LD_PRELOAD",
    "LD_AUDIT",
    "LD_LIBRARY_PATH",
    "CMAKE_GENERATOR",
    "CMAKE_GENERATOR_INSTANCE",
    "CMAKE_GENERATOR_PLATFORM",
    "CMAKE_GENERATOR_TOOLSET",
    "CMAKE_BUILD_PARALLEL_LEVEL",
    "CMAKE_CONFIG_TYPE",
    "CMAKE_BUILD_TYPE",
    "CMAKE_CONFIGURATION_TYPES",
    "CMAKE_C_COMPILER",
    "CMAKE_CXX_COMPILER",
    "CMAKE_C_COMPILER_ARG1",
    "CMAKE_CXX_COMPILER_ARG1",
    "CMAKE_C_COMPILER_TARGET",
    "CMAKE_CXX_COMPILER_TARGET",
    "CMAKE_C_COMPILER_EXTERNAL_TOOLCHAIN",
    "CMAKE_CXX_COMPILER_EXTERNAL_TOOLCHAIN",
    "CMAKE_C_COMPILER_AR",
    "CMAKE_CXX_COMPILER_AR",
    "CMAKE_C_COMPILER_RANLIB",
    "CMAKE_CXX_COMPILER_RANLIB",
    "CMAKE_C_COMPILER_LAUNCHER",
    "CMAKE_CXX_COMPILER_LAUNCHER",
    "CMAKE_C_LINKER_LAUNCHER",
    "CMAKE_CXX_LINKER_LAUNCHER",
    "CMAKE_C_CLANG_TIDY",
    "CMAKE_CXX_CLANG_TIDY",
    "CMAKE_C_CPPCHECK",
    "CMAKE_CXX_CPPCHECK",
    "CMAKE_C_INCLUDE_WHAT_YOU_USE",
    "CMAKE_CXX_INCLUDE_WHAT_YOU_USE",
    "CMAKE_C_CPPLINT",
    "CMAKE_CXX_CPPLINT",
    "CMAKE_MAKE_PROGRAM",
    "CMAKE_AR",
    "CMAKE_RANLIB",
    "CMAKE_LINKER",
    "CMAKE_NM",
    "CMAKE_OBJCOPY",
    "CMAKE_STRIP",
    "CMAKE_READELF",
    "CMAKE_C_FLAGS_INIT",
    "CMAKE_CXX_FLAGS_INIT",
    "CMAKE_EXE_LINKER_FLAGS_INIT",
    "CMAKE_SHARED_LINKER_FLAGS_INIT",
    "CMAKE_MODULE_LINKER_FLAGS_INIT",
    "CMAKE_TOOLCHAIN_FILE",
    "CMAKE_SYSROOT",
    "CMAKE_SYSROOT_COMPILE",
    "CMAKE_SYSROOT_LINK",
    "CMAKE_STAGING_PREFIX",
    "CMAKE_FIND_ROOT_PATH",
    "CMAKE_FIND_ROOT_PATH_MODE_PROGRAM",
    "CMAKE_FIND_ROOT_PATH_MODE_LIBRARY",
    "CMAKE_FIND_ROOT_PATH_MODE_INCLUDE",
    "CMAKE_FIND_ROOT_PATH_MODE_PACKAGE",
    "CMAKE_SYSTEM_NAME",
    "CMAKE_SYSTEM_PROCESSOR",
    "CMAKE_CROSSCOMPILING_EMULATOR",
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
MODEL_ARTIFACT_SUFFIXES = {
    ".bin",
    ".ckpt",
    ".gguf",
    ".ggml",
    ".mlmodel",
    ".npy",
    ".npz",
    ".onnx",
    ".pt",
    ".pth",
    ".safetensors",
    ".tflite",
}
MODEL_ARTIFACT_MAGIC = {
    b"GGUF": "GGUF",
    b"ggml": "GGML",
}
REPOSITORY_MODEL_ARTIFACT_EXCLUSIONS = {
    ".agents",
    ".cache",
    ".cargo",
    ".codex",
    ".git",
    "node_modules",
    "target",
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


def require_regular_source_file(path: Path, description: str) -> None:
    metadata = path.lstat()
    require(
        not stat.S_ISLNK(metadata.st_mode) and stat.S_ISREG(metadata.st_mode),
        f"{description} must be a real regular file",
    )


def require_regular_source_directory(path: Path, description: str) -> None:
    metadata = path.lstat()
    require(
        not stat.S_ISLNK(metadata.st_mode) and stat.S_ISDIR(metadata.st_mode),
        f"{description} must be a real directory",
    )


def cmake_options(build: str) -> set[str]:
    return set(cmake_option_entries(build))


def cmake_option_entries(build: str) -> list[str]:
    match = re.search(
        r"const LLAMA_CPP_CMAKE_OPTIONS: &\[&str\] = &\[(?P<options>.*?)\];",
        build,
        flags=re.DOTALL,
    )
    require(match is not None, "closed CMake option list is missing")
    option_body = match.group("options")
    entries = re.findall(r'"([^"\\]+)"', option_body)
    remaining = re.sub(r'"[^"\\]+"', "", option_body)
    require(
        re.fullmatch(r"[\s,]*", remaining) is not None,
        "closed CMake option list must contain only literal definitions",
    )
    return entries


def require_closed_cmake_options(build: str) -> None:
    options = cmake_option_entries(build)
    require(
        len(options) == len(set(options)),
        "closed CMake option list must not contain duplicate definitions",
    )
    require(
        set(options) == EXPECTED_CMAKE_OPTIONS,
        "closed CMake options changed",
    )


def require_verified_cmake_source(build: str) -> None:
    require(
        EXPECTED_SOURCE_DIR_EXPRESSION in build,
        "CMake source is not the verified vendored llama.cpp tree",
    )


def require_closed_build_directories(build: str) -> None:
    """Pin the values passed to CMake, not just their local variable names."""
    for variable, declaration, description in (
        ("source_dir", EXPECTED_SOURCE_DIR_DECLARATION, "verified vendored source"),
        ("output_dir", EXPECTED_OUTPUT_DIR_DECLARATION, "Cargo output directory"),
        ("build_dir", EXPECTED_BUILD_DIR_DECLARATION, "private build directory"),
    ):
        declarations = re.findall(rf"\blet\s+{variable}\s*=", build)
        require(
            declarations == [f"let {variable} ="],
            f"build script must declare the {description} exactly once",
        )
        require(
            declaration in build,
            f"build script must use the pinned {description}",
        )


def require_complete_vendored_source_tracking(build: str) -> None:
    require(
        EXPECTED_SOURCE_ROOT_INSPECTION in build,
        "build script does not inspect the vendored source root",
    )
    require(
        "require_vendored_source_root(&source_dir);" in build,
        "build script does not validate the vendored source root before tracking it",
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
        'println!("cargo:rerun-if-changed={}", directory.display());' in tracker,
        "vendored source tracker must register every directory",
    )
    require(
        'println!("cargo:rerun-if-changed={}", path.display());' in tracker,
        "vendored source tracker must register every regular file",
    )


def require_build_time_vendored_tree_verification(build: str) -> None:
    require(
        f'const EXPECTED_VENDORED_TREE_SHA256: &str =\n    "{EXPECTED_VENDORED_TREE_SHA256}";'
        in build,
        "build script does not pin the vendored source tree digest",
    )
    require(
        EXPECTED_SOURCE_DIGEST_VERIFIER in build,
        "build script does not verify the vendored source tree before CMake",
    )
    verifier_match = re.search(
        r"fn verify_vendored_tree_digest\s*\(directory: &Path\)\s*\{(?P<body>.*?)\n\}",
        build,
        flags=re.DOTALL,
    )
    require(verifier_match is not None, "vendored source digest verifier is missing")
    verifier = verifier_match.group("body")
    require(
        EXPECTED_SOURCE_ROOT_VERIFIER in verifier,
        "vendored source digest verifier must re-check that the source root is a real directory",
    )
    require(
        "vendored_tree_digest(directory, directory, &mut digest);" in verifier
        and "observed, EXPECTED_VENDORED_TREE_SHA256" in verifier,
        "vendored source digest verifier must compare the complete tree to the pinned digest",
    )
    require(
        "use sha2::{Digest, Sha256};" in build,
        "build script must use SHA-256 for the vendored source digest",
    )
    first_cmake_command = build.find("Command::new(SYSTEM_CMAKE)")
    require(first_cmake_command != -1, "closed CMake build commands are missing")
    verification_positions = [
        match.start()
        for match in re.finditer(re.escape(EXPECTED_SOURCE_DIGEST_VERIFIER), build)
    ]
    require(
        len(verification_positions) >= 1
        and verification_positions[0] < first_cmake_command,
        "build script must verify the vendored source tree before constructing CMake commands",
    )
    configure_run = build.find(
        'run(\n        &mut configure,\n        "closed llama.cpp static-library configuration",'
    )
    require(configure_run != -1, "closed CMake configuration execution is missing")
    build_command = build.find("let mut build = Command::new(SYSTEM_CMAKE);")
    require(build_command != -1, "closed CMake static-library build command is missing")
    require(
        len(verification_positions) >= 2
        and first_cmake_command < verification_positions[1] < configure_run,
        "build script must re-verify the vendored source tree immediately before CMake configuration",
    )
    require(
        len(verification_positions) >= 3
        and configure_run < verification_positions[2] < build_command,
        "build script must re-verify the vendored source tree after configuration and before compiling",
    )
    post_build_verification = build.find(
        EXPECTED_SOURCE_DIGEST_VERIFIER,
        build.find('run(&mut build, "closed llama.cpp static-library build");'),
    )
    first_link_directive = build.find('cargo:rustc-link-search=native=')
    require(
        len(verification_positions) == 4
        and post_build_verification == verification_positions[3]
        and first_link_directive != -1
        and post_build_verification < first_link_directive,
        "build script must re-verify the vendored source tree after compiling and before Cargo linkage",
    )


def require_closed_build_process_boundary(build: str) -> None:
    # Rust permits comments between path segments and the constructor call. Treat
    # them as whitespace so they cannot conceal an additional subprocess.
    require(
        build.isascii(),
        "build script must remain ASCII-only so Unicode aliases cannot conceal subprocesses",
    )
    rust_separator = r"(?:\s|/\*[\s\S]*?\*/|//[^\n]*(?:\n|$))*"
    process_module_alias = re.search(
        r"\buse\b[\s\S]*?\bprocess\s+as\s+[A-Za-z_][A-Za-z0-9_]*",
        build,
    )
    require(
        process_module_alias is None,
        "build script must not alias the process module outside the approved CMake subprocesses",
    )
    command_aliases = (
        re.search(
            r"\buse\b[\s\S]*?\bCommand\s+as\s+[A-Za-z_][A-Za-z0-9_]*",
            build,
        )
        or re.search(
            r"\btype\s+[A-Za-z_][A-Za-z0-9_]*\s*=\s*"
            r"(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*Command\s*;",
            build,
        )
    )
    require(
        command_aliases is None,
        "build script must not alias Command outside the approved CMake subprocesses",
    )
    command_references = re.findall(
        rf"(?:std{rust_separator}::{rust_separator})?"
        rf"(?:process{rust_separator}::{rust_separator})?"
        rf"Command{rust_separator}::{rust_separator}(?:new|default)\b",
        build,
    )
    command_calls = re.findall(
        rf"(?:std{rust_separator}::{rust_separator})?"
        rf"(?:process{rust_separator}::{rust_separator})?"
        rf"Command{rust_separator}::{rust_separator}(?:new|default){rust_separator}\(",
        build,
    )
    require(
        len(command_references) == 2
        and len(command_calls) == 2
        and not re.search(
            rf"(?:std{rust_separator}::{rust_separator})?"
            rf"(?:process{rust_separator}::{rust_separator})?"
            rf"Command{rust_separator}::{rust_separator}default\b",
            build,
        )
        and f'const SYSTEM_CMAKE: &str = "{EXPECTED_SYSTEM_CMAKE}";' in build
        and len(
            re.findall(r"Command\s*::\s*new\s*\(SYSTEM_CMAKE\)", build)
        )
        == 2,
        "build script must start only the two approved CMake subprocesses",
    )


def require_no_build_script_source_injection(build: str) -> None:
    """Keep the closed build-process check from being bypassed by extra Rust files."""
    # ``build.rs`` is deliberately a single reviewed source file. An include or
    # out-of-line module could add build-time process behavior that the closed
    # command parser cannot inspect in this file.
    rust_separator = r"(?:\s|/\*[\s\S]*?\*/|//[^\n]*(?:\n|$))*"
    source_include = re.compile(rf"\binclude(?:_str|_bytes)?{rust_separator}!")
    path_module = re.compile(rf"#{rust_separator}\[{rust_separator}path{rust_separator}=")
    rust_attribute = re.compile(
        rf"#{rust_separator}\[{rust_separator}(?P<contents>[^\]]*)\]",
        flags=re.DOTALL,
    )
    out_of_line_module = re.compile(
        rf"\bmod{rust_separator}[A-Za-z_][A-Za-z0-9_]*{rust_separator};"
    )
    violations = []
    if source_include.search(build):
        violations.append("Rust source include macro")
    if path_module.search(build):
        violations.append("Rust path module attribute")
    for attribute in rust_attribute.finditer(build):
        contents = attribute.group("contents")
        if re.match(rf"cfg_attr{rust_separator}\(", contents) and re.search(
            rf"\bpath{rust_separator}=", contents
        ):
            violations.append("conditional Rust path module attribute")
    if out_of_line_module.search(build):
        violations.append("out-of-line Rust module")
    require(
        not violations,
        "build script must not import unscanned Rust source: " + "; ".join(violations),
    )


def without_rust_comments(source: str) -> str:
    """Normalize comment separators before checking closed build-script syntax."""
    return re.sub(r"/\*[\s\S]*?\*/|//[^\n]*", "", source)


def require_closed_command_mutation_boundary(build: str) -> None:
    """Prevent helpers from mutating either closed CMake command out of band."""
    build = without_rust_comments(build)
    mutation_pattern = (
        r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\.\s*"
        r"(arg|args|current_dir|env|envs|env_clear|env_remove)\s*\("
    )
    command_mutations = re.findall(mutation_pattern, build)
    require(
        all(method != "current_dir" for _, method in command_mutations),
        "build script must not override the closed CMake working directory",
    )
    approved_mutations: list[tuple[str, str]] = []
    for command_name, description in (
        ("configure", "closed llama.cpp static-library configuration"),
        ("build", "closed llama.cpp static-library build"),
    ):
        command_match = re.search(
            rf"let mut {command_name}\s*=\s*Command::new\(SYSTEM_CMAKE\);(?P<body>.*?)"
            rf"run\(\s*&mut {command_name},\s*\"{re.escape(description)}\",?\s*\);",
            build,
            flags=re.DOTALL,
        )
        require(
            command_match is not None,
            f"{description} invocation is missing",
        )
        approved_mutations.extend(re.findall(mutation_pattern, command_match.group("body")))
    require(
        command_mutations == approved_mutations,
        "build script must mutate CMake commands only in the approved command blocks",
    )


def require_closed_command_execution_boundary(build: str) -> None:
    """Allow the two approved CMake commands to execute only through ``run``."""
    # Ignore comments when counting executable method calls so explanatory text
    # cannot affect the closed source check.
    code = without_rust_comments(build)
    command_execution_calls = re.findall(
        r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\.\s*"
        r"(status|output|spawn|exec)\s*\(",
        code,
    )
    require(
        command_execution_calls == [("command", "status")],
        "build script must execute CMake only through the closed run helper",
    )
    run_calls = re.findall(
        r"\brun\s*\(\s*&mut\s*([A-Za-z_][A-Za-z0-9_]*)\s*,",
        code,
    )
    require(
        run_calls == ["configure", "build"],
        "build script must run only the approved configuration and static-library build",
    )


def command_arguments(body: str) -> list[str]:
    return [
        literal or variable
        for literal, variable in re.findall(
            r'\.arg\((?:"([^\"]+)"|(&[a-z_]+))\)', body
        )
    ]


def require_only_recognized_argument_calls(body: str, command_name: str) -> None:
    recognized_arguments = command_arguments(body)
    argument_calls = re.findall(
        rf"^\s*(?:{command_name}\.)?\.arg\s*\(", body, flags=re.MULTILINE
    )
    require(
        len(argument_calls) == len(recognized_arguments),
        f"closed CMake {command_name} invocation includes an unrecognized argument expression",
    )


def require_verified_cmake_configure_arguments(build: str) -> None:
    configure_match = re.search(
        r"let mut configure\s*=\s*Command::new\(SYSTEM_CMAKE\);(?P<body>.*?)"
        r"run\(\s*&mut configure,\s*\"closed llama\.cpp static-library configuration\",?\s*\);",
        build,
        flags=re.DOTALL,
    )
    require(configure_match is not None, "closed CMake configuration invocation is missing")
    configure_body = configure_match.group("body")
    require_only_recognized_argument_calls(configure_body, "configure")
    require(
        command_arguments(configure_body)
        == [
            "-S",
            "&source_dir",
            "-B",
            "&build_dir",
            "--fresh",
            "-DCMAKE_BUILD_TYPE=Release",
        ],
        "closed CMake configuration must use only the verified source and private build directory",
    )


def require_closed_cmake_invocation(build: str) -> None:
    configure_match = re.search(
        r"let mut configure\s*=\s*Command::new\(SYSTEM_CMAKE\);(?P<body>.*?)"
        r"run\(\s*&mut configure,\s*\"closed llama\.cpp static-library configuration\",?\s*\);",
        build,
        flags=re.DOTALL,
    )
    require(configure_match is not None, "closed CMake configuration invocation is missing")
    configure_body = configure_match.group("body")
    require(
        re.findall(r"configure\.args\s*\(([^)]+)\);", configure_body)
        == ["LLAMA_CPP_CMAKE_OPTIONS"],
        "closed CMake configuration must apply only the approved option list once",
    )
    unguarded_definitions = set(re.findall(r'\.arg\("(-D[^"\\]+)"\)', configure_body))
    require(
        unguarded_definitions == ALLOWED_CONFIGURE_DEFINITIONS,
        "closed CMake configuration includes an unguarded -D option",
    )


def require_closed_cmake_environment(build: str) -> None:
    build = without_rust_comments(build)
    environment_match = re.search(
        r"const CLOSED_CMAKE_ENVIRONMENT: &\[&str\] = &\[(?P<variables>.*?)\];",
        build,
        flags=re.DOTALL,
    )
    require(environment_match is not None, "closed CMake environment list is missing")
    environment_entries = re.findall(r'"([^"\\]+)"', environment_match.group("variables"))
    require(
        len(environment_entries) == len(set(environment_entries)),
        "closed CMake environment list must not contain duplicate variables",
    )
    environment_variables = set(environment_entries)
    require(
        environment_variables == EXPECTED_CLOSED_CMAKE_ENVIRONMENT,
        "closed CMake environment list changed",
    )
    require(
        f'const CLOSED_BUILD_PATH: &str = "{EXPECTED_CLOSED_BUILD_PATH}";' in build,
        "closed build path changed",
    )
    for command_name, description in (
        ("configure", "closed llama.cpp static-library configuration"),
        ("build", "closed llama.cpp static-library build"),
    ):
        command_match = re.search(
            rf"let mut {command_name}\s*=\s*Command::new\(SYSTEM_CMAKE\);(?P<body>.*?)"
            rf"run\(\s*&mut {command_name},\s*\"{re.escape(description)}\",?\s*\);",
            build,
            flags=re.DOTALL,
        )
        require(command_match is not None, f"{description} invocation is missing")
        require(
            len(re.findall(rf"{command_name}\s*\.\s*env_clear\(\);", command_match.group("body")))
            == 1,
            f"{description} must clear the inherited environment exactly once",
        )
        require(
            len(
                re.findall(
                    rf"for variable in CLOSED_CMAKE_ENVIRONMENT \{{\s*"
                    rf"{command_name}\s*\.\s*env_remove\(variable\);\s*\}}",
                    command_match.group("body"),
                )
            )
            == 1,
            f"{description} must remove the inherited toolchain environment exactly once",
        )
        require(
            len(re.findall(rf'{command_name}\s*\.\s*env\("PATH", CLOSED_BUILD_PATH\);', command_match.group("body")))
            == 1,
            f"{description} must use the fixed system build path exactly once",
        )
        environment_methods = re.findall(
            rf"\b{command_name}\s*\.\s*(env[A-Za-z_]*)\s*\(", command_match.group("body")
        )
        require(
            environment_methods == ["env_clear", "env_remove", "env"],
            f"{description} must not add environment assignments",
        )


def require_closed_cmake_build_invocation(build: str) -> None:
    build_match = re.search(
        r"let mut build\s*=\s*Command::new\(SYSTEM_CMAKE\);(?P<body>.*?)"
        r"run\(\s*&mut build,\s*\"closed llama\.cpp static-library build\",?\s*\);",
        build,
        flags=re.DOTALL,
    )
    require(build_match is not None, "closed CMake build invocation is missing")
    build_body = build_match.group("body")
    require(
        not re.findall(r"\bbuild\.args\s*\(", build_body),
        "closed CMake build must not use batched arguments",
    )
    require_only_recognized_argument_calls(build_body, "build")
    build_arguments = command_arguments(build_body)
    require(
        build_arguments
        == ["--build", "&build_dir", "--config", "Release", "--target", "llama"],
        "closed CMake build must use the Release configuration and target only the static llama library",
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


def require_closed_cargo_directives(build: str) -> None:
    """Allow only the fixed Cargo directives needed by the static build."""
    code = without_rust_comments(build)
    output_macros = re.findall(r"\b(print|println)\s*!\s*\(", code)
    require(
        output_macros == ["println"] * sum(EXPECTED_CARGO_DIRECTIVES.values()),
        "build script must emit Cargo directives only through the approved println calls",
    )
    cargo_directives = Counter(re.findall(r'"(cargo:[^"\\]*)"', code))
    require(
        cargo_directives == EXPECTED_CARGO_DIRECTIVES,
        "build script must emit only the approved Cargo directives",
    )
    direct_directives = re.findall(r'\bprintln\s*!\s*\(\s*"cargo:', code)
    require(
        len(direct_directives) == sum(EXPECTED_CARGO_DIRECTIVES.values()),
        "build script must emit every approved Cargo directive directly through println",
    )


def require_no_rust_runtime_api_usage(source_directory: Path) -> None:
    """Keep the M63 source boundary build-only until a later adapter is approved."""
    require_regular_source_directory(source_directory, "Rust runtime source root")
    prohibited_api = re.compile(r"\b(?:llama|ggml)_[A-Za-z0-9_]*\b")
    # Rust permits comments anywhere whitespace is accepted. Keep comments in
    # the scanned source (so a suspicious token in a comment remains visible),
    # while recognizing comment-separated syntax that could otherwise hide an
    # FFI declaration or an unscanned source import from this build-only guard.
    rust_separator = r"(?:\s|/\*[\s\S]*?\*/|//[^\n]*(?:\n|$))*"
    ffi_declaration = re.compile(
        rf"\b(?:unsafe{rust_separator})?extern{rust_separator}"
        rf"(?:\"[^\"]+\"{rust_separator})?\{{"
    )
    ffi_function = re.compile(
        rf"\b(?:pub(?:{rust_separator}\([^)]*\))?{rust_separator})?"
        rf"(?:unsafe{rust_separator})?extern{rust_separator}"
        rf"(?:\"[^\"]+\"{rust_separator})?fn\b"
    )
    source_include = re.compile(rf"\binclude{rust_separator}!")
    path_module = re.compile(rf"#{rust_separator}\[{rust_separator}path{rust_separator}=")
    rust_attribute = re.compile(
        rf"#{rust_separator}\[{rust_separator}(?P<contents>[^\]]*)\]",
        flags=re.DOTALL,
    )
    violations = []
    for path in sorted(source_directory.rglob("*")):
        relative = path.relative_to(source_directory)
        metadata = path.lstat()
        require(
            not stat.S_ISLNK(metadata.st_mode),
            "Rust runtime-source guard must not follow symlinks: " f"{relative}",
        )
        require(
            stat.S_ISDIR(metadata.st_mode) or stat.S_ISREG(metadata.st_mode),
            "Rust runtime-source guard found a non-regular entry: " f"{relative}",
        )
        if not stat.S_ISREG(metadata.st_mode) or path.suffix != ".rs":
            continue
        source = path.read_text(encoding="utf-8")
        matches = sorted(set(prohibited_api.findall(source)))
        if matches:
            violations.append(f"{relative}: {', '.join(matches)}")
        if ffi_declaration.search(source):
            violations.append(f"{relative}: native FFI declaration")
        if ffi_function.search(source):
            violations.append(f"{relative}: native FFI function")
        if source_include.search(source):
            violations.append(f"{relative}: Rust source include macro")
        if path_module.search(source):
            violations.append(f"{relative}: Rust path module attribute")
        for attribute in rust_attribute.finditer(source):
            contents = attribute.group("contents")
            if re.match(rf"cfg_attr{rust_separator}\(", contents) and re.search(
                rf"\bpath{rust_separator}=", contents
            ):
                violations.append(f"{relative}: conditional Rust path module attribute")
    require(
        not violations,
        "Rust runtime source must not reference llama.cpp or ggml APIs or declare native FFI "
        "or import unscanned Rust source: "
        + "; ".join(violations),
    )


def require_no_model_artifacts(
    directory: Path, excluded_directory_names: frozenset[str] = frozenset()
) -> None:
    for path in chain((directory,), directory.rglob("*")):
        relative = path.relative_to(directory)
        if any(part in excluded_directory_names for part in relative.parts):
            continue
        metadata = path.lstat()
        require(
            not stat.S_ISLNK(metadata.st_mode),
            "model artifact admission scan must not follow symlinks: "
            f"{relative}",
        )
        require(
            stat.S_ISDIR(metadata.st_mode) or stat.S_ISREG(metadata.st_mode),
            "model artifact admission scan found a non-regular entry: "
            f"{relative}",
        )
        require(
            path.suffix.lower() not in MODEL_ARTIFACT_SUFFIXES,
            f"model artifact found: {relative}",
        )
        if not stat.S_ISREG(metadata.st_mode):
            continue
        header = path.read_bytes()[:4]
        for magic, format_name in MODEL_ARTIFACT_MAGIC.items():
            require(
                header != magic,
                f"model artifact signature found ({format_name}): {relative}",
            )


def require_no_repository_model_artifacts(repository_root: Path) -> None:
    """Reject model data anywhere in repository-owned source content.

    The M63 artifact must stay outside this working copy.  Local tool caches,
    dependency installs, and generated build outputs are not repository
    content, so they are deliberately excluded from this admission guard.
    """
    require_no_model_artifacts(
        repository_root,
        frozenset(REPOSITORY_MODEL_ARTIFACT_EXCLUSIONS),
    )


def require_model_artifact_ignores(gitignore: str) -> None:
    ignored_patterns = {
        line.strip()
        for line in gitignore.splitlines()
        if line.strip() and not line.lstrip().startswith("#")
    }
    expected_patterns = {
        "*." + "".join(f"[{character.lower()}{character.upper()}]" for character in suffix[1:])
        for suffix in MODEL_ARTIFACT_SUFFIXES
    }
    require(
        expected_patterns.issubset(ignored_patterns),
        "Git ignore policy must exclude every guarded model artifact suffix case-insensitively",
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
    require_no_repository_model_artifacts(ROOT)
    require_model_artifact_ignores(GITIGNORE.read_text(encoding="utf-8"))
    require_regular_source_file(BUILD_SCRIPT, "M63 build script")
    build = BUILD_SCRIPT.read_text(encoding="utf-8")
    require_closed_cmake_options(build)
    require_verified_cmake_source(build)
    require_closed_build_directories(build)
    require_complete_vendored_source_tracking(build)
    require_build_time_vendored_tree_verification(build)
    require_closed_build_process_boundary(build)
    require_no_build_script_source_injection(build)
    require_closed_command_mutation_boundary(build)
    require_closed_command_execution_boundary(build)
    require_verified_cmake_configure_arguments(build)
    require_closed_cmake_invocation(build)
    require_closed_cmake_environment(build)
    require_closed_cmake_build_invocation(build)
    require_closed_cargo_linkage(build)
    require_closed_cargo_directives(build)
    require_no_rust_runtime_api_usage(NATIVE_SOURCE)
    print("M63 llama.cpp vendor validation passed.")


if __name__ == "__main__":
    main()
