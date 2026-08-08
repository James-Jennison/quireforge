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
NATIVE_SOURCE = ROOT / "apps" / "desktop" / "src-tauri" / "src"
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
    "-DCMAKE_FIND_USE_PACKAGE_REGISTRY=OFF",
    "-DCMAKE_FIND_USE_SYSTEM_PACKAGE_REGISTRY=OFF",
    "-DCMAKE_FIND_PACKAGE_NO_PACKAGE_REGISTRY=ON",
    "-DCMAKE_FIND_PACKAGE_NO_SYSTEM_PACKAGE_REGISTRY=ON",
    "-DCMAKE_EXPORT_NO_PACKAGE_REGISTRY=ON",
}
ALLOWED_CONFIGURE_DEFINITIONS = {"-DCMAKE_BUILD_TYPE=Release"}
EXPECTED_SOURCE_DIR_EXPRESSION = 'manifest_dir.join("../../../third_party/llama.cpp")'
EXPECTED_SOURCE_TRACKER = "register_vendored_source_tree(&source_dir);"
EXPECTED_SOURCE_ROOT_INSPECTION = "fs::symlink_metadata(&source_dir)"
EXPECTED_VENDORED_TREE_SHA256 = "9892c22a1a05adf0775615f1b845886f8f1be96ad7b6f71093103eaec546a511"
EXPECTED_SOURCE_DIGEST_VERIFIER = "verify_vendored_tree_digest(&source_dir);"
EXPECTED_LINK_SEARCH_DIRECTORIES = [
    'build_dir.join("src")',
    'build_dir.join("ggml/src")',
]
EXPECTED_STATIC_LIBRARIES = ["llama", "ggml", "ggml-base", "ggml-cpu"]
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
    ".mlmodel",
    ".onnx",
    ".pt",
    ".pth",
    ".safetensors",
    ".tflite",
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
    require(
        build.find(EXPECTED_SOURCE_DIGEST_VERIFIER) < first_cmake_command,
        "build script must verify the vendored source tree before constructing CMake commands",
    )
    build_command = build.find("let mut build = Command::new(SYSTEM_CMAKE);")
    require(build_command != -1, "closed CMake static-library build command is missing")
    require(
        len(re.findall(re.escape(EXPECTED_SOURCE_DIGEST_VERIFIER), build)) == 2
        and build.find(EXPECTED_SOURCE_DIGEST_VERIFIER, first_cmake_command + 1)
        < build_command,
        "build script must re-verify the vendored source tree after configuration and before compiling",
    )


def require_closed_build_process_boundary(build: str) -> None:
    command_calls = re.findall(
        r"(?:std::)?(?:process::)?Command\s*::\s*new\s*\(", build
    )
    require(
        len(command_calls) == 2
        and f'const SYSTEM_CMAKE: &str = "{EXPECTED_SYSTEM_CMAKE}";' in build
        and len(re.findall(r"Command::new\(SYSTEM_CMAKE\)", build)) == 2,
        "build script must start only the two approved CMake subprocesses",
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
            len(re.findall(rf"{command_name}\.env_clear\(\);", command_match.group("body")))
            == 1,
            f"{description} must clear the inherited environment exactly once",
        )
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
        require(
            len(re.findall(rf'{command_name}\.env\("PATH", CLOSED_BUILD_PATH\);', command_match.group("body")))
            == 1,
            f"{description} must use the fixed system build path exactly once",
        )
        environment_methods = re.findall(
            rf"\b{command_name}\.(env[A-Za-z_]*)\s*\(", command_match.group("body")
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


def require_no_rust_runtime_api_usage(source_directory: Path) -> None:
    """Keep the M63 source boundary build-only until a later adapter is approved."""
    prohibited_api = re.compile(r"\b(?:llama|ggml)_[A-Za-z0-9_]*\b")
    violations = []
    for path in sorted(source_directory.rglob("*.rs")):
        relative = path.relative_to(source_directory)
        matches = sorted(set(prohibited_api.findall(path.read_text(encoding="utf-8"))))
        if matches:
            violations.append(f"{relative}: {', '.join(matches)}")
    require(
        not violations,
        "Rust runtime source must not reference llama.cpp or ggml APIs: "
        + "; ".join(violations),
    )


def require_no_model_artifacts(directory: Path) -> None:
    for path in directory.rglob("*"):
        require(
            path.suffix.lower() not in MODEL_ARTIFACT_SUFFIXES,
            f"model artifact found: {path.relative_to(directory)}",
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
    require_no_model_artifacts(VENDOR)
    build = BUILD_SCRIPT.read_text(encoding="utf-8")
    require_closed_cmake_options(build)
    require_verified_cmake_source(build)
    require_complete_vendored_source_tracking(build)
    require_build_time_vendored_tree_verification(build)
    require_closed_build_process_boundary(build)
    require_verified_cmake_configure_arguments(build)
    require_closed_cmake_invocation(build)
    require_closed_cmake_environment(build)
    require_closed_cmake_build_invocation(build)
    require_closed_cargo_linkage(build)
    require_no_rust_runtime_api_usage(NATIVE_SOURCE)
    print("M63 llama.cpp vendor validation passed.")


if __name__ == "__main__":
    main()
