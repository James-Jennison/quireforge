"""Focused tests for the closed M63 llama.cpp build configuration guard."""

from __future__ import annotations

import importlib.util
import os
import tempfile
import unittest
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "validate_llama_cpp_vendor", ROOT / "scripts" / "validate_llama_cpp_vendor.py"
)
assert SPEC is not None and SPEC.loader is not None
VALIDATOR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VALIDATOR)


class CmakeOptionsTests(unittest.TestCase):
    def test_rejects_a_symlinked_m63_build_script(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            source = temporary_root / "build.rs"
            source.write_text("fn main() {}\n", encoding="utf-8")
            source_link = temporary_root / "build-link.rs"
            source_link.symlink_to(source)

            with self.assertRaisesRegex(SystemExit, "M63 build script must be a real regular file"):
                VALIDATOR.require_regular_source_file(source_link, "M63 build script")

    def test_rejects_a_symlinked_rust_runtime_source_root(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            source = temporary_root / "source"
            source.mkdir()
            source_link = temporary_root / "source-link"
            source_link.symlink_to(source, target_is_directory=True)

            with self.assertRaisesRegex(SystemExit, "Rust runtime source root must be a real directory"):
                VALIDATOR.require_no_rust_runtime_api_usage(source_link)

    def test_rejects_a_symlinked_rust_runtime_source_parent(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            source_parent = temporary_root / "real-source-parent"
            source = source_parent / "source"
            source.mkdir(parents=True)
            linked_parent = temporary_root / "source-parent"
            linked_parent.symlink_to(source_parent, target_is_directory=True)

            with self.assertRaisesRegex(
                SystemExit, "Rust runtime source root parent must be a real directory"
            ):
                VALIDATOR.require_no_rust_runtime_api_usage(linked_parent / "source")

    def test_rejects_a_symlinked_vendored_source_root(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            source = temporary_root / "source"
            source.mkdir()
            source_link = temporary_root / "source-link"
            source_link.symlink_to(source, target_is_directory=True)

            with self.assertRaisesRegex(SystemExit, "source root must be a real directory"):
                VALIDATOR.require_regular_vendored_source_tree(source_link)

    def test_rejects_a_symlinked_vendored_source_parent(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            source_parent = temporary_root / "real-third-party"
            source = source_parent / "llama.cpp"
            source.mkdir(parents=True)
            linked_parent = temporary_root / "third_party"
            linked_parent.symlink_to(source_parent, target_is_directory=True)

            with self.assertRaisesRegex(SystemExit, "source parent must be a real directory"):
                VALIDATOR.require_regular_vendored_source_tree(linked_parent / "llama.cpp")

    def test_rejects_a_symlink_inside_the_vendored_source_tree(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            source = Path(temporary_directory) / "source"
            source.mkdir()
            source_file = source / "source-file"
            source_file.write_text("source", encoding="utf-8")
            (source / "source-link").symlink_to(source_file)

            with self.assertRaisesRegex(SystemExit, "vendored symlink found: source-link"):
                VALIDATOR.require_regular_vendored_source_tree(source)

    def test_rejects_common_model_artifact_formats(self) -> None:
        for suffix in sorted(VALIDATOR.MODEL_ARTIFACT_SUFFIXES):
            with self.subTest(suffix=suffix), tempfile.TemporaryDirectory() as temporary_directory:
                source = Path(temporary_directory)
                artifact = source / f"unapproved-model{suffix}"
                artifact.write_bytes(b"not a model")

                with self.assertRaisesRegex(SystemExit, "model artifact found"):
                    VALIDATOR.require_no_model_artifacts(source)

    def test_rejects_a_renamed_gguf_model_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            source = Path(temporary_directory)
            (source / "vendored-data").write_bytes(b"GGUF\x03\x00\x00\x00")

            with self.assertRaisesRegex(SystemExit, "model artifact signature found \\(GGUF\\)"):
                VALIDATOR.require_no_model_artifacts(source)

    def test_rejects_a_renamed_legacy_ggml_model_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            source = Path(temporary_directory)
            (source / "vendored-data").write_bytes(b"ggml\x03\x00\x00\x00")

            with self.assertRaisesRegex(SystemExit, "model artifact signature found \\(GGML\\)"):
                VALIDATOR.require_no_model_artifacts(source)

    def test_rejects_a_model_artifact_named_inside_a_zip_archive(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            source = Path(temporary_directory)
            with zipfile.ZipFile(source / "archive.zip", "w") as archive:
                archive.writestr("nested/unapproved-model.gguf", b"not a model")

            with self.assertRaisesRegex(SystemExit, "model artifact found in ZIP archive"):
                VALIDATOR.require_no_model_artifacts(source)

    def test_rejects_a_renamed_gguf_model_artifact_inside_a_zip_archive(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            source = Path(temporary_directory)
            with zipfile.ZipFile(source / "archive.zip", "w") as archive:
                archive.writestr("nested/unapproved-model", b"GGUF\\x03\\x00\\x00\\x00")

            with self.assertRaisesRegex(
                SystemExit, "model artifact signature found \\(GGUF\\) in ZIP archive"
            ):
                VALIDATOR.require_no_model_artifacts(source)

    def test_allows_a_zip_archive_without_model_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            source = Path(temporary_directory)
            with zipfile.ZipFile(source / "source-only.zip", "w") as archive:
                archive.writestr("nested/readme.txt", "source only")

            VALIDATOR.require_no_model_artifacts(source)

    def test_allows_non_model_source_files(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            source = Path(temporary_directory)
            (source / "llama.cpp").write_text("// source only\n", encoding="utf-8")

            VALIDATOR.require_no_model_artifacts(source)

    def test_rejects_a_symlink_before_reading_its_target(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            source = Path(temporary_directory) / "source"
            source.mkdir()
            target = Path(temporary_directory) / "external-target"
            target.write_bytes(b"GGUF\x03\x00\x00\x00")
            (source / "unapproved-link").symlink_to(target)

            with self.assertRaisesRegex(SystemExit, "must not follow symlinks"):
                VALIDATOR.require_no_model_artifacts(source)

    def test_rejects_a_non_regular_entry_before_scanning_for_model_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            source = Path(temporary_directory)
            os.mkfifo(source / "unapproved-entry")

            with self.assertRaisesRegex(SystemExit, "found a non-regular entry"):
                VALIDATOR.require_no_model_artifacts(source)

    def test_rejects_a_model_artifact_anywhere_in_repository_owned_content(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repository = Path(temporary_directory)
            documentation = repository / "docs"
            documentation.mkdir()
            (documentation / "unapproved-model").write_bytes(b"GGUF\x03\x00\x00\x00")

            with self.assertRaisesRegex(SystemExit, "model artifact signature found \\(GGUF\\)"):
                VALIDATOR.require_no_repository_model_artifacts(repository)

    def test_rejects_a_model_artifact_at_the_repository_root(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repository = Path(temporary_directory)
            (repository / "unapproved-model.gguf").write_bytes(b"not a model")

            with self.assertRaisesRegex(SystemExit, "model artifact found"):
                VALIDATOR.require_no_repository_model_artifacts(repository)

    def test_excludes_transient_repository_directories_from_model_admission_scan(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repository = Path(temporary_directory)
            transient = repository / "apps" / "desktop" / "node_modules"
            transient.parent.mkdir(parents=True)
            transient.mkdir()
            (transient / "unapproved-model.gguf").write_bytes(b"GGUF\x03\x00\x00\x00")

            VALIDATOR.require_no_repository_model_artifacts(repository)

    def test_requires_every_model_artifact_suffix_to_be_gitignored(self) -> None:
        gitignore = (ROOT / ".gitignore").read_text(encoding="utf-8")

        VALIDATOR.require_model_artifact_ignores(gitignore)

    def test_rejects_a_missing_model_artifact_gitignore_pattern(self) -> None:
        gitignore = (ROOT / ".gitignore").read_text(encoding="utf-8")
        gitignore = gitignore.replace("*.[gG][gG][uU][fF]\n", "")

        with self.assertRaisesRegex(SystemExit, "must exclude every guarded model artifact suffix"):
            VALIDATOR.require_model_artifact_ignores(gitignore)

    def test_rejects_a_lowercase_only_model_artifact_gitignore_pattern(self) -> None:
        gitignore = (ROOT / ".gitignore").read_text(encoding="utf-8")
        gitignore = gitignore.replace("*.[gG][gG][uU][fF]", "*.gguf")

        with self.assertRaisesRegex(SystemExit, "case-insensitively"):
            VALIDATOR.require_model_artifact_ignores(gitignore)

    def test_rejects_a_symlinked_rust_source_before_reading_its_target(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            source = Path(temporary_directory) / "source"
            source.mkdir()
            target = Path(temporary_directory) / "external.rs"
            target.write_text("// outside verified source\n", encoding="utf-8")
            (source / "runtime.rs").symlink_to(target)

            with self.assertRaisesRegex(SystemExit, "must not follow symlinks"):
                VALIDATOR.require_no_rust_runtime_api_usage(source)

    def test_rejects_a_non_regular_native_source_entry(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            source = Path(temporary_directory) / "source"
            source.mkdir()
            fifo = source / "unapproved-entry"
            os.mkfifo(fifo)

            with self.assertRaisesRegex(SystemExit, "non-regular entry"):
                VALIDATOR.require_no_rust_runtime_api_usage(source)

    def test_reads_the_exact_closed_option_list(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")

        self.assertEqual(VALIDATOR.cmake_options(build), VALIDATOR.EXPECTED_CMAKE_OPTIONS)
        VALIDATOR.require_closed_cmake_options(build)

    def test_rejects_a_duplicate_closed_cmake_option(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            '    "-DGGML_CPU=ON",\n',
            '    "-DGGML_CPU=ON",\n    "-DGGML_CPU=ON",\n',
        )

        with self.assertRaisesRegex(SystemExit, "must not contain duplicate definitions"):
            VALIDATOR.require_closed_cmake_options(build)

    def test_rejects_a_non_literal_closed_cmake_option(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            '    "-DGGML_CPU=ON",\n',
            '    "-DGGML_CPU=ON",\n    UNAPPROVED_CMAKE_OPTION,\n',
        )

        with self.assertRaisesRegex(SystemExit, "must contain only literal definitions"):
            VALIDATOR.require_closed_cmake_options(build)

    def test_rejects_a_missing_option_list(self) -> None:
        with self.assertRaisesRegex(SystemExit, "option list is missing"):
            VALIDATOR.cmake_options("fn main() {}")

    def test_exposes_conflicting_or_extra_options(self) -> None:
        build = '''const LLAMA_CPP_CMAKE_OPTIONS: &[&str] = &[
            "-DGGML_CPU=ON",
            "-DGGML_CUDA=ON",
        ];'''

        self.assertNotEqual(VALIDATOR.cmake_options(build), VALIDATOR.EXPECTED_CMAKE_OPTIONS)
        self.assertIn("-DGGML_CUDA=ON", VALIDATOR.cmake_options(build))

    def test_requires_explicit_curl_and_git_probe_disablement(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")

        options = VALIDATOR.cmake_options(build)

        self.assertIn("-DLLAMA_CURL=OFF", options)
        self.assertIn("-DGIT_EXE=", options)
        self.assertIn("-DGIT_EXECUTABLE=", options)
        self.assertIn("-DCMAKE_DISABLE_FIND_PACKAGE_Git=ON", options)

    def test_disables_cmake_package_registries_and_search_path_discovery(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")

        options = VALIDATOR.cmake_options(build)

        self.assertTrue(
            {
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
            }.issubset(options)
        )

    def test_forbids_cmake_writes_to_the_verified_vendored_source_tree(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")

        options = VALIDATOR.cmake_options(build)

        self.assertTrue(
            {
                "-DCMAKE_DISABLE_SOURCE_CHANGES=ON",
                "-DCMAKE_DISABLE_IN_SOURCE_BUILD=ON",
            }.issubset(options)
        )

    def test_requires_fixed_system_compiler_and_archive_tool_paths(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")

        options = VALIDATOR.cmake_options(build)

        self.assertTrue(
            {
                "-DCMAKE_C_COMPILER=/usr/bin/cc",
                "-DCMAKE_CXX_COMPILER=/usr/bin/c++",
                "-DCMAKE_C_COMPILER_AR=/usr/bin/ar",
                "-DCMAKE_CXX_COMPILER_AR=/usr/bin/ar",
                "-DCMAKE_C_COMPILER_RANLIB=/usr/bin/ranlib",
                "-DCMAKE_CXX_COMPILER_RANLIB=/usr/bin/ranlib",
                "-DCMAKE_MAKE_PROGRAM=/usr/bin/make",
            }.issubset(options)
        )

    def test_requires_the_optional_llamafile_integration_to_be_disabled(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")

        options = VALIDATOR.cmake_options(build)

        self.assertIn("-DGGML_LLAMAFILE=OFF", options)

    def test_requires_the_closed_option_list_once_without_extra_definitions(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")

        VALIDATOR.require_closed_cmake_invocation(build)

    def test_accepts_rustfmt_whitespace_in_the_configure_declaration(self) -> None:
        build = '''let mut configure =
            Command::new(SYSTEM_CMAKE);
            configure.arg("-DCMAKE_BUILD_TYPE=Release");
            configure.args(LLAMA_CPP_CMAKE_OPTIONS);
            run(
                &mut configure,
                "closed llama.cpp static-library configuration",
            );'''

        VALIDATOR.require_closed_cmake_invocation(build)

    def test_rejects_an_unguarded_cmake_definition(self) -> None:
        build = '''let mut configure = Command::new(SYSTEM_CMAKE);
            configure.arg("-DGGML_CUDA=ON");
            configure.args(LLAMA_CPP_CMAKE_OPTIONS);
            run(&mut configure, "closed llama.cpp static-library configuration");'''

        with self.assertRaisesRegex(SystemExit, "unguarded -D option"):
            VALIDATOR.require_closed_cmake_invocation(build)

    def test_rejects_an_extra_batched_cmake_configuration_argument(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "    configure.args(LLAMA_CPP_CMAKE_OPTIONS);",
            '    configure.args(LLAMA_CPP_CMAKE_OPTIONS);\n    configure.args(["--toolchain", "/tmp/unapproved.cmake"]);',
        )

        with self.assertRaisesRegex(SystemExit, "must apply only the approved option list once"):
            VALIDATOR.require_closed_cmake_invocation(build)

    def test_rejects_an_unrecognized_cmake_configuration_argument_expression(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            '        .arg("-DCMAKE_BUILD_TYPE=Release");',
            '        .arg("-DCMAKE_BUILD_TYPE=Release")\n        .arg(unapproved_option);',
        )

        with self.assertRaisesRegex(SystemExit, "unrecognized argument expression"):
            VALIDATOR.require_verified_cmake_configure_arguments(build)

    def test_requires_the_closed_toolchain_environment_scrub(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")

        VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_a_duplicate_closed_cmake_environment_variable(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('    "CMAKE_TOOLCHAIN_FILE",\n', '    "CMAKE_TOOLCHAIN_FILE",\n    "CMAKE_TOOLCHAIN_FILE",\n')

        with self.assertRaisesRegex(SystemExit, "must not contain duplicate variables"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_a_non_system_cmake_executable(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            'const SYSTEM_CMAKE: &str = "/usr/bin/cmake";',
            'const SYSTEM_CMAKE: &str = "cmake";',
        )

        with self.assertRaisesRegex(SystemExit, "only the two approved CMake subprocesses"):
            VALIDATOR.require_closed_build_process_boundary(build)

    def test_rejects_a_missing_fixed_system_build_path(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('    configure.env("PATH", CLOSED_BUILD_PATH);\n', "")

        with self.assertRaisesRegex(SystemExit, "fixed system build path"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_a_configuration_that_inherits_the_environment(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace("    configure.env_clear();\n", "")

        with self.assertRaisesRegex(SystemExit, "must clear the inherited environment"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_a_build_that_inherits_the_environment(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace("    build.env_clear();\n", "")

        with self.assertRaisesRegex(SystemExit, "must clear the inherited environment"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_an_extra_configuration_environment_assignment(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            '    configure.env("PATH", CLOSED_BUILD_PATH);\n',
            '    configure.env("PATH", CLOSED_BUILD_PATH);\n'
            '    configure.env("CMAKE_TOOLCHAIN_FILE", "/tmp/untrusted");\n',
        )

        with self.assertRaisesRegex(SystemExit, "must not add environment assignments"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_an_incomplete_toolchain_environment_scrub(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('    "CMAKE_TOOLCHAIN_FILE",\n', "")

        with self.assertRaisesRegex(SystemExit, "environment list changed"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_a_missing_compiler_launcher_environment_scrub(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('    "CMAKE_CXX_COMPILER_LAUNCHER",\n', "")

        with self.assertRaisesRegex(SystemExit, "environment list changed"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_a_missing_linker_launcher_environment_scrub(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('    "CMAKE_CXX_LINKER_LAUNCHER",\n', "")

        with self.assertRaisesRegex(SystemExit, "environment list changed"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_a_missing_static_analysis_tool_environment_scrub(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('    "CMAKE_CXX_CLANG_TIDY",\n', "")

        with self.assertRaisesRegex(SystemExit, "environment list changed"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_a_missing_generic_binary_tool_environment_scrub(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('    "CMAKE_LINKER",\n', "")

        with self.assertRaisesRegex(SystemExit, "environment list changed"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_a_missing_cmake_parallelism_environment_scrub(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('    "CMAKE_BUILD_PARALLEL_LEVEL",\n', "")

        with self.assertRaisesRegex(SystemExit, "environment list changed"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_a_missing_cmake_configuration_environment_scrub(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('    "CMAKE_CONFIG_TYPE",\n', "")

        with self.assertRaisesRegex(SystemExit, "environment list changed"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_a_missing_cmake_build_type_environment_scrub(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('    "CMAKE_BUILD_TYPE",\n', "")

        with self.assertRaisesRegex(SystemExit, "environment list changed"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_a_missing_cmake_configuration_types_environment_scrub(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('    "CMAKE_CONFIGURATION_TYPES",\n', "")

        with self.assertRaisesRegex(SystemExit, "environment list changed"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_a_missing_compiler_target_environment_scrub(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('    "CMAKE_CXX_COMPILER_TARGET",\n', "")

        with self.assertRaisesRegex(SystemExit, "environment list changed"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_a_missing_compiler_archive_tool_environment_scrub(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('    "CMAKE_C_COMPILER_AR",\n', "")

        with self.assertRaisesRegex(SystemExit, "environment list changed"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_a_missing_initial_linker_flags_environment_scrub(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('    "CMAKE_SHARED_LINKER_FLAGS_INIT",\n', "")

        with self.assertRaisesRegex(SystemExit, "environment list changed"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_a_missing_cmake_project_include_scrub(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('    "CMAKE_PROJECT_TOP_LEVEL_INCLUDES",\n', "")

        with self.assertRaisesRegex(SystemExit, "environment list changed"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_a_missing_cross_compilation_environment_scrub(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('    "CMAKE_SYSROOT",\n', "")

        with self.assertRaisesRegex(SystemExit, "environment list changed"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_a_missing_cmake_make_rules_override_scrub(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('    "CMAKE_USER_MAKE_RULES_OVERRIDE_CXX",\n', "")

        with self.assertRaisesRegex(SystemExit, "environment list changed"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_a_missing_inherited_make_flags_scrub(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('    "GNUMAKEFLAGS",\n', "")

        with self.assertRaisesRegex(SystemExit, "environment list changed"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_a_missing_toolchain_environment_scrub(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "    for variable in CLOSED_CMAKE_ENVIRONMENT {\n"
            "        configure.env_remove(variable);\n"
            "    }\n",
            "",
        )

        with self.assertRaisesRegex(SystemExit, "must remove the inherited toolchain environment"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_a_missing_build_toolchain_environment_scrub(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "    for variable in CLOSED_CMAKE_ENVIRONMENT {\n"
            "        build.env_remove(variable);\n"
            "    }\n",
            "",
        )

        with self.assertRaisesRegex(SystemExit, "static-library build must remove"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_a_non_vendored_cmake_source(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            'manifest_dir.join("../../../third_party/llama.cpp")',
            'manifest_dir.join("../../../third_party/not-llama.cpp")',
        )

        with self.assertRaisesRegex(SystemExit, "source is not the verified"):
            VALIDATOR.require_verified_cmake_source(build)

    def test_requires_the_pinned_cmake_source_and_build_directory_declarations(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")

        VALIDATOR.require_closed_build_directories(build)

    def test_rejects_a_shadowed_vendored_source_directory(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            '    let build_dir = output_dir.join("m63-llama.cpp-build");\n',
            '    let source_dir = PathBuf::from("/unreviewed-source");\n'
            '    let build_dir = output_dir.join("m63-llama.cpp-build");\n',
        )

        with self.assertRaisesRegex(SystemExit, "verified vendored source exactly once"):
            VALIDATOR.require_closed_build_directories(build)

    def test_rejects_a_non_private_cmake_build_directory(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            'let build_dir = output_dir.join("m63-llama.cpp-build");',
            'let build_dir = source_dir.join("unapproved-build");',
        )

        with self.assertRaisesRegex(SystemExit, "pinned private build directory"):
            VALIDATOR.require_closed_build_directories(build)

    def test_requires_complete_vendored_source_tracking(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")

        VALIDATOR.require_complete_vendored_source_tracking(build)

    def test_requires_build_time_vendored_tree_verification(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")

        VALIDATOR.require_build_time_vendored_tree_verification(build)

    def test_rejects_missing_build_time_vendored_tree_verification(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace("    verify_vendored_tree_digest(&source_dir);\n", "")

        with self.assertRaisesRegex(SystemExit, "does not verify the vendored source tree"):
            VALIDATOR.require_build_time_vendored_tree_verification(build)

    def test_rejects_a_digest_verifier_that_does_not_recheck_the_source_root(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "    require_vendored_source_root(directory);\n    let mut digest = Sha256::new();",
            "    let mut digest = Sha256::new();",
        )

        with self.assertRaisesRegex(SystemExit, "must re-check that the source root"):
            VALIDATOR.require_build_time_vendored_tree_verification(build)

    def test_rejects_digest_verification_after_cmake_command_construction(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "    verify_vendored_tree_digest(&source_dir);\n    register_vendored_source_tree(&source_dir);\n\n"
            '    let mut configure = Command::new(SYSTEM_CMAKE);',
            "    register_vendored_source_tree(&source_dir);\n\n"
            '    let mut configure = Command::new(SYSTEM_CMAKE);\n'
            "    verify_vendored_tree_digest(&source_dir);",
        )

        with self.assertRaisesRegex(SystemExit, "before constructing CMake commands"):
            VALIDATOR.require_build_time_vendored_tree_verification(build)

    def test_rejects_a_missing_initial_vendored_tree_verification(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "    verify_vendored_tree_digest(&source_dir);\n    register_vendored_source_tree(&source_dir);",
            "    register_vendored_source_tree(&source_dir);",
        )

        with self.assertRaisesRegex(SystemExit, "before constructing CMake commands"):
            VALIDATOR.require_build_time_vendored_tree_verification(build)

    def test_rejects_missing_pre_configuration_vendored_tree_verification(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "    // Change-registration walks the full source tree. Re-check after that walk\n"
            "    // and immediately before CMake evaluates the vendored configuration.\n"
            "    verify_vendored_tree_digest(&source_dir);\n"
            "    run(\n"
            "        &mut configure,",
            "    run(\n"
            "        &mut configure,",
        )

        with self.assertRaisesRegex(
            SystemExit,
            "immediately before CMake configuration",
        ):
            VALIDATOR.require_build_time_vendored_tree_verification(build)

    def test_rejects_missing_post_configuration_vendored_tree_verification(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "    verify_vendored_tree_digest(&source_dir);\n\n    let mut build = Command::new(SYSTEM_CMAKE);",
            "    let mut build = Command::new(SYSTEM_CMAKE);",
        )

        with self.assertRaisesRegex(
            SystemExit,
            "re-verify the vendored source tree after configuration and before compiling",
        ):
            VALIDATOR.require_build_time_vendored_tree_verification(build)

    def test_rejects_post_configuration_verification_after_compiler_construction(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "    verify_vendored_tree_digest(&source_dir);\n\n    let mut build = Command::new(SYSTEM_CMAKE);",
            "    let mut build = Command::new(SYSTEM_CMAKE);\n"
            "    verify_vendored_tree_digest(&source_dir);",
        )

        with self.assertRaisesRegex(
            SystemExit,
            "re-verify the vendored source tree after configuration and before compiling",
        ):
            VALIDATOR.require_build_time_vendored_tree_verification(build)

    def test_rejects_missing_post_build_vendored_tree_verification(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            '    verify_vendored_tree_digest(&source_dir);\n\n    println!(\n'
            '        "cargo:rustc-link-search=native={}",',
            '    println!(\n        "cargo:rustc-link-search=native={}",',
        )

        with self.assertRaisesRegex(
            SystemExit,
            "re-verify the vendored source tree after compiling and before Cargo linkage",
        ):
            VALIDATOR.require_build_time_vendored_tree_verification(build)

    def test_rejects_a_symlinked_or_non_directory_vendored_source_root(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "!source_metadata.file_type().is_symlink() && source_metadata.is_dir()",
            "source_metadata.is_dir()",
        )

        with self.assertRaisesRegex(SystemExit, "must reject a symlinked or non-directory"):
            VALIDATOR.require_complete_vendored_source_tracking(build)

    def test_rejects_directory_only_vendored_source_tracking(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "    register_vendored_source_tree(&source_dir);\n",
            '    println!("cargo:rerun-if-changed={}", source_dir.display());\n',
        )

        with self.assertRaisesRegex(SystemExit, "does not track every"):
            VALIDATOR.require_complete_vendored_source_tracking(build)

    def test_rejects_a_tracker_without_directory_change_tracking(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            '    println!("cargo:rerun-if-changed={}", directory.display());\n',
            "",
        )

        with self.assertRaisesRegex(SystemExit, "must register every directory"):
            VALIDATOR.require_complete_vendored_source_tracking(build)

    def test_rejects_a_tracker_without_symlink_rejection(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace("!file_type.is_symlink()", "file_type.is_symlink()")

        with self.assertRaisesRegex(SystemExit, "must reject symlinks"):
            VALIDATOR.require_complete_vendored_source_tracking(build)

    def test_rejects_an_unapproved_build_subprocess(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            '    tauri_build::build();',
            '    Command::new("curl");\n    tauri_build::build();',
        )

        with self.assertRaisesRegex(SystemExit, "only the two approved CMake subprocesses"):
            VALIDATOR.require_closed_build_process_boundary(build)

    def test_rejects_a_default_constructed_unapproved_build_subprocess(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            '    tauri_build::build();',
            '    let _unapproved = Command::default();\n    tauri_build::build();',
        )

        with self.assertRaisesRegex(SystemExit, "only the two approved CMake subprocesses"):
            VALIDATOR.require_closed_build_process_boundary(build)

    def test_rejects_a_function_item_reference_to_an_unapproved_build_subprocess(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            '    tauri_build::build();',
            '    let launch = Command::new;\n'
            '    let _unapproved = launch("curl");\n'
            '    tauri_build::build();',
        )

        with self.assertRaisesRegex(SystemExit, "only the two approved CMake subprocesses"):
            VALIDATOR.require_closed_build_process_boundary(build)

    def test_rejects_a_fully_qualified_unapproved_build_subprocess(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            '    tauri_build::build();',
            '    std::process::Command::new("curl");\n    tauri_build::build();',
        )

        with self.assertRaisesRegex(SystemExit, "only the two approved CMake subprocesses"):
            VALIDATOR.require_closed_build_process_boundary(build)

    def test_rejects_a_whitespace_qualified_unapproved_build_subprocess(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            '    tauri_build::build();',
            '    std :: process :: Command :: new("curl");\n    tauri_build::build();',
        )

        with self.assertRaisesRegex(SystemExit, "only the two approved CMake subprocesses"):
            VALIDATOR.require_closed_build_process_boundary(build)

    def test_rejects_a_comment_separated_unapproved_build_subprocess(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            '    tauri_build::build();',
            '    std::process::Command /* hidden */ ::new /* hidden */ ("curl");\n'
            '    tauri_build::build();',
        )

        with self.assertRaisesRegex(SystemExit, "only the two approved CMake subprocesses"):
            VALIDATOR.require_closed_build_process_boundary(build)

    def test_rejects_an_aliased_unapproved_build_subprocess(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "use std::{",
            "use std::process::Command as UnapprovedCommand;\n\nuse std::{",
        ).replace(
            '    tauri_build::build();',
            '    UnapprovedCommand::new("curl");\n    tauri_build::build();',
        )

        with self.assertRaisesRegex(SystemExit, "must not alias (the process module|Command)"):
            VALIDATOR.require_closed_build_process_boundary(build)

    def test_rejects_a_unicode_aliased_unapproved_build_subprocess(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "use std::{",
            "use std::process::Command as Ā;\n\nuse std::{",
        ).replace(
            '    tauri_build::build();',
            '    let _unapproved = Ā::new("curl");\n    tauri_build::build();',
        )

        with self.assertRaisesRegex(SystemExit, "must remain ASCII-only"):
            VALIDATOR.require_closed_build_process_boundary(build)

    def test_rejects_a_process_module_aliased_unapproved_build_subprocess(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "use std::{",
            "use std::process as process_alias;\n\nuse std::{",
        ).replace(
            '    tauri_build::build();',
            '    process_alias::Command::new("curl");\n    tauri_build::build();',
        )

        with self.assertRaisesRegex(SystemExit, "must not alias the process module"):
            VALIDATOR.require_closed_build_process_boundary(build)

    def test_rejects_a_type_aliased_unapproved_build_subprocess(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "const EXPECTED_VENDORED_TREE_SHA256",
            "type UnapprovedCommand = std::process::Command;\n\nconst EXPECTED_VENDORED_TREE_SHA256",
        ).replace(
            '    tauri_build::build();',
            '    UnapprovedCommand::new("curl");\n    tauri_build::build();',
        )

        with self.assertRaisesRegex(SystemExit, "must not alias Command"):
            VALIDATOR.require_closed_build_process_boundary(build)

    def test_rejects_a_type_alias_through_an_imported_process_module(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "use std::{",
            "use std::process as process_alias;\n\nuse std::{",
        ).replace(
            "const EXPECTED_VENDORED_TREE_SHA256",
            "type UnapprovedCommand = process_alias::Command;\n\n"
            "const EXPECTED_VENDORED_TREE_SHA256",
        ).replace(
            '    tauri_build::build();',
            '    UnapprovedCommand::new("curl");\n    tauri_build::build();',
        )

        with self.assertRaisesRegex(SystemExit, "must not alias (the process module|Command)"):
            VALIDATOR.require_closed_build_process_boundary(build)

    def test_requires_only_the_reviewed_read_only_build_script_filesystem_calls(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")

        VALIDATOR.require_read_only_build_script_filesystem_boundary(build)

    def test_rejects_a_build_script_filesystem_mutation(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            '    tauri_build::build();',
            '    fs::write("unapproved-build-output", "unexpected");\n    tauri_build::build();',
        )

        with self.assertRaisesRegex(SystemExit, "approved read-only filesystem calls"):
            VALIDATOR.require_read_only_build_script_filesystem_boundary(build)

    def test_rejects_a_fully_qualified_build_script_filesystem_mutation(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            '    tauri_build::build();',
            '    std::fs::write("unapproved-build-output", "unexpected");\n    tauri_build::build();',
        )

        with self.assertRaisesRegex(SystemExit, "only the reviewed filesystem calls"):
            VALIDATOR.require_read_only_build_script_filesystem_boundary(build)

    def test_rejects_an_aliased_build_script_filesystem_module(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "use std::{",
            "use std::fs as unapproved_fs;\n\nuse std::{",
        )

        with self.assertRaisesRegex(SystemExit, "must not alias the filesystem module"):
            VALIDATOR.require_read_only_build_script_filesystem_boundary(build)

    def test_rejects_a_nested_import_of_a_build_script_filesystem_mutation(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "use std::{",
            "use std::{fs::{write},",
        )

        with self.assertRaisesRegex(SystemExit, "reviewed filesystem calls"):
            VALIDATOR.require_read_only_build_script_filesystem_boundary(build)

    def test_rejects_build_script_rust_source_includes(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "fn build_llama_cpp() {",
            'include! ("unreviewed-build.rs");\n\nfn build_llama_cpp() {',
        )

        with self.assertRaisesRegex(SystemExit, "build script must not import unscanned Rust source"):
            VALIDATOR.require_no_build_script_source_injection(build)

    def test_rejects_build_script_string_and_byte_source_includes(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")

        for macro in ("include_str", "include_bytes"):
            with self.subTest(macro=macro):
                injected = build.replace(
                    "fn build_llama_cpp() {",
                    f'{macro}! ("unreviewed-build-input");\n\nfn build_llama_cpp() {{',
                )

                with self.assertRaisesRegex(
                    SystemExit, "build script must not import unscanned Rust source"
                ):
                    VALIDATOR.require_no_build_script_source_injection(injected)

    def test_rejects_build_script_path_modules(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "fn build_llama_cpp() {",
            '#[path = "unreviewed-build.rs"]\nmod unreviewed_build;\n\nfn build_llama_cpp() {',
        )

        with self.assertRaisesRegex(SystemExit, "build script must not import unscanned Rust source"):
            VALIDATOR.require_no_build_script_source_injection(build)

    def test_rejects_comment_separated_build_script_out_of_line_modules(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "fn build_llama_cpp() {",
            "mod /* unapproved */ hidden_build;\n\nfn build_llama_cpp() {",
        )

        with self.assertRaisesRegex(SystemExit, "build script must not import unscanned Rust source"):
            VALIDATOR.require_no_build_script_source_injection(build)

    def test_rejects_conditional_build_script_path_modules(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "fn build_llama_cpp() {",
            '#[cfg_attr(feature = "hidden", path = "unreviewed-build.rs")]\n'
            "mod hidden_build;\n\nfn build_llama_cpp() {",
        )

        with self.assertRaisesRegex(SystemExit, "build script must not import unscanned Rust source"):
            VALIDATOR.require_no_build_script_source_injection(build)

    def test_rejects_a_helper_that_adds_a_cmake_argument(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "fn build_llama_cpp() {",
            'fn add_unapproved_argument(command: &mut Command) {\n'
            '    command.arg("-DGGML_CUDA=ON");\n'
            "}\n\n"
            "fn build_llama_cpp() {",
        )

        with self.assertRaisesRegex(SystemExit, "must mutate CMake commands only"):
            VALIDATOR.require_closed_command_mutation_boundary(build)

    def test_rejects_a_helper_that_adds_a_cmake_environment_assignment(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "fn build_llama_cpp() {",
            'fn add_unapproved_environment(command: &mut Command) {\n'
            '    command.env("CMAKE_TOOLCHAIN_FILE", "/tmp/unapproved.cmake");\n'
            "}\n\n"
            "fn build_llama_cpp() {",
        )

        with self.assertRaisesRegex(SystemExit, "must mutate CMake commands only"):
            VALIDATOR.require_closed_command_mutation_boundary(build)

    def test_rejects_a_helper_that_adds_batched_cmake_environment_assignments(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "fn build_llama_cpp() {",
            'fn add_unapproved_environment(command: &mut Command) {\n'
            '    command.envs([("CMAKE_TOOLCHAIN_FILE", "/tmp/untrusted")]);\n'
            "}\n\n"
            "fn build_llama_cpp() {",
        )

        with self.assertRaisesRegex(SystemExit, "must mutate CMake commands only"):
            VALIDATOR.require_closed_command_mutation_boundary(build)

    def test_rejects_a_helper_that_reuses_an_approved_command_name(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "fn build_llama_cpp() {",
            'fn add_unapproved_argument(configure: &mut Command) {\n'
            '    configure.arg("-DGGML_CUDA=ON");\n'
            "}\n\n"
            "fn build_llama_cpp() {",
        )

        with self.assertRaisesRegex(SystemExit, "must mutate CMake commands only"):
            VALIDATOR.require_closed_command_mutation_boundary(build)

    def test_rejects_a_cmake_working_directory_override(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "    configure.args(LLAMA_CPP_CMAKE_OPTIONS);",
            '    configure.args(LLAMA_CPP_CMAKE_OPTIONS);\n    configure.current_dir("/tmp/unapproved");',
        )

        with self.assertRaisesRegex(SystemExit, "must not override the closed CMake working directory"):
            VALIDATOR.require_closed_command_mutation_boundary(build)

    def test_rejects_a_comment_separated_cmake_environment_assignment(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            '    configure.env("PATH", CLOSED_BUILD_PATH);',
            '    configure.env("PATH", CLOSED_BUILD_PATH);\n'
            '    configure /* hidden */ .env("CC", "/tmp/unapproved");',
        )

        with self.assertRaisesRegex(SystemExit, "must not add environment assignments"):
            VALIDATOR.require_closed_cmake_environment(build)

    def test_rejects_a_comment_separated_cmake_working_directory_override(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "    configure.args(LLAMA_CPP_CMAKE_OPTIONS);",
            '    configure.args(LLAMA_CPP_CMAKE_OPTIONS);\n'
            '    configure /* hidden */ .current_dir("/tmp/unapproved");',
        )

        with self.assertRaisesRegex(SystemExit, "must not override the closed CMake working directory"):
            VALIDATOR.require_closed_command_mutation_boundary(build)

    def test_requires_only_the_closed_cmake_execution_paths(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")

        VALIDATOR.require_closed_command_execution_boundary(build)

    def test_rejects_direct_command_execution_outside_the_closed_run_helper(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            "    configure.args(LLAMA_CPP_CMAKE_OPTIONS);",
            "    configure.args(LLAMA_CPP_CMAKE_OPTIONS);\n    configure.status();",
        )

        with self.assertRaisesRegex(SystemExit, "must execute CMake only through the closed run helper"):
            VALIDATOR.require_closed_command_execution_boundary(build)

    def test_rejects_an_extra_closed_run_helper_invocation(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            '    run(&mut build, "closed llama.cpp static-library build");',
            '    run(&mut build, "closed llama.cpp static-library build");\n'
            '    run(&mut configure, "closed llama.cpp static-library configuration");',
        )

        with self.assertRaisesRegex(SystemExit, "must run only the approved configuration"):
            VALIDATOR.require_closed_command_execution_boundary(build)

    def test_rejects_a_configuration_that_does_not_pass_the_verified_source(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('.arg(&source_dir)', '.arg(&build_dir)', 1)

        with self.assertRaisesRegex(SystemExit, "must use only the verified source"):
            VALIDATOR.require_verified_cmake_configure_arguments(build)

    def test_rejects_a_configuration_that_reuses_a_cmake_cache(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('        .arg("--fresh")\n', "")

        with self.assertRaisesRegex(SystemExit, "must use only the verified source"):
            VALIDATOR.require_verified_cmake_configure_arguments(build)

    def test_rejects_a_cmake_build_with_an_extra_target(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('.arg("llama");', '.arg("all");', 1)

        with self.assertRaisesRegex(SystemExit, "target only"):
            VALIDATOR.require_closed_cmake_build_invocation(build)

    def test_rejects_batched_cmake_build_arguments(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            '        .arg("llama");',
            '        .arg("llama");\n    build.args(["--parallel", "8"]);',
            1,
        )

        with self.assertRaisesRegex(SystemExit, "must not use batched arguments"):
            VALIDATOR.require_closed_cmake_build_invocation(build)

    def test_rejects_an_unrecognized_cmake_build_argument_expression(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            '        .arg("llama");',
            '        .arg("llama")\n        .arg(unapproved_target);',
            1,
        )

        with self.assertRaisesRegex(SystemExit, "unrecognized argument expression"):
            VALIDATOR.require_closed_cmake_build_invocation(build)

    def test_rejects_a_cmake_build_without_the_explicit_release_configuration(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('        .arg("--config")\n        .arg("Release")\n', "")

        with self.assertRaisesRegex(SystemExit, "must use the Release configuration"):
            VALIDATOR.require_closed_cmake_build_invocation(build)

    def test_requires_only_the_closed_static_cpu_linkage(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")

        VALIDATOR.require_closed_cargo_linkage(build)

    def test_rejects_an_extra_native_link_library(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            'println!("cargo:rustc-link-lib=dylib=stdc++");',
            'println!("cargo:rustc-link-lib=dylib=stdc++");\n    println!("cargo:rustc-link-lib=dylib=cuda");',
        )

        with self.assertRaisesRegex(SystemExit, "unexpected native library directive"):
            VALIDATOR.require_closed_cargo_linkage(build)

    def test_rejects_an_extra_native_library_search_directory(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            'println!("cargo:rustc-link-lib=dylib=stdc++");',
            'println!("cargo:rustc-link-search=native=/tmp/untrusted");\n    '
            'println!("cargo:rustc-link-lib=dylib=stdc++");',
        )

        with self.assertRaisesRegex(SystemExit, "unexpected native library search directive"):
            VALIDATOR.require_closed_cargo_linkage(build)

    def test_requires_only_the_approved_cargo_directives(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")

        VALIDATOR.require_closed_cargo_directives(build)

    def test_rejects_an_extra_cargo_compile_configuration_directive(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            '    tauri_build::build();',
            '    println!("cargo:rustc-cfg=unapproved_runtime");\n    tauri_build::build();',
        )

        with self.assertRaisesRegex(SystemExit, "approved println calls"):
            VALIDATOR.require_closed_cargo_directives(build)

    def test_rejects_a_non_directive_build_script_output_macro(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            '    tauri_build::build();',
            '    print!("unapproved build output");\n    tauri_build::build();',
        )

        with self.assertRaisesRegex(SystemExit, "only through the approved println calls"):
            VALIDATOR.require_closed_cargo_directives(build)

    def test_rejects_an_indirect_whitelisted_cargo_directive(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            '    println!("cargo:rustc-link-lib=dylib=stdc++");',
            '    let directive = "cargo:rustc-link-lib=dylib=stdc++";\n'
            '    println!("{directive}");',
        )

        with self.assertRaisesRegex(SystemExit, "every approved Cargo directive directly"):
            VALIDATOR.require_closed_cargo_directives(build)

    def test_rejects_rust_runtime_references_to_vendored_c_apis(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            source = Path(temporary_directory)
            (source / "runtime.rs").write_text(
                "fn unavailable() { let _ = llama_model_load_from_file; }\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                SystemExit,
                "Rust runtime source must not reference llama.cpp or ggml APIs",
            ):
                VALIDATOR.require_no_rust_runtime_api_usage(source)

    def test_rejects_native_ffi_declarations_in_rust_runtime_source(self) -> None:
        for declaration in ('extern "C" {', 'unsafe extern "C" {'):
            with self.subTest(declaration=declaration), tempfile.TemporaryDirectory() as temporary_directory:
                source = Path(temporary_directory)
                (source / "runtime.rs").write_text(
                    f"{declaration}\n    fn hidden_runtime_entry();\n}}\n",
                    encoding="utf-8",
                )

                with self.assertRaisesRegex(
                    SystemExit,
                    "must not reference llama.cpp or ggml APIs or declare native FFI",
                ):
                    VALIDATOR.require_no_rust_runtime_api_usage(source)

    def test_rejects_comment_separated_native_ffi_declarations(self) -> None:
        for declaration in (
            'extern /* unapproved */ "C" {',
            'unsafe /* unapproved */ extern /* unapproved */ "C" {',
        ):
            with self.subTest(declaration=declaration), tempfile.TemporaryDirectory() as temporary_directory:
                source = Path(temporary_directory)
                (source / "runtime.rs").write_text(
                    f"{declaration}\n    fn hidden_runtime_entry();\n}}\n",
                    encoding="utf-8",
                )

                with self.assertRaisesRegex(
                    SystemExit,
                    "must not reference llama.cpp or ggml APIs or declare native FFI",
                ):
                    VALIDATOR.require_no_rust_runtime_api_usage(source)

    def test_rejects_exported_native_ffi_functions_in_rust_runtime_source(self) -> None:
        for definition in (
            'extern "C" fn hidden_runtime_entry() {}',
            'pub unsafe extern "C" fn hidden_runtime_entry() {}',
            'pub /* unapproved */ extern /* unapproved */ "C" '
            '/* unapproved */ fn hidden_runtime_entry() {}',
        ):
            with self.subTest(definition=definition), tempfile.TemporaryDirectory() as temporary_directory:
                source = Path(temporary_directory)
                (source / "runtime.rs").write_text(f"{definition}\n", encoding="utf-8")

                with self.assertRaisesRegex(
                    SystemExit,
                    "must not reference llama.cpp or ggml APIs or declare native FFI",
                ):
                    VALIDATOR.require_no_rust_runtime_api_usage(source)

    def test_rejects_rust_source_include_macros(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            source = Path(temporary_directory)
            (source / "runtime.rs").write_text(
                'include! ("../unreviewed-runtime.rs");\n',
                encoding="utf-8",
            )

            with self.assertRaisesRegex(SystemExit, "or import unscanned Rust source"):
                VALIDATOR.require_no_rust_runtime_api_usage(source)

    def test_rejects_comment_separated_rust_source_include_macros(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            source = Path(temporary_directory)
            (source / "runtime.rs").write_text(
                'include /* unapproved */! ("../unreviewed-runtime.rs");\n',
                encoding="utf-8",
            )

            with self.assertRaisesRegex(SystemExit, "or import unscanned Rust source"):
                VALIDATOR.require_no_rust_runtime_api_usage(source)

    def test_rejects_rust_path_module_attributes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            source = Path(temporary_directory)
            (source / "runtime.rs").write_text(
                '#[path = "../unreviewed-runtime.txt"]\nmod unreviewed_runtime;\n',
                encoding="utf-8",
            )

            with self.assertRaisesRegex(SystemExit, "or import unscanned Rust source"):
                VALIDATOR.require_no_rust_runtime_api_usage(source)

    def test_rejects_comment_separated_rust_path_module_attributes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            source = Path(temporary_directory)
            (source / "runtime.rs").write_text(
                '# /* unapproved */ [path /* unapproved */ = "../unreviewed-runtime.txt"]\n'
                "mod unreviewed_runtime;\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(SystemExit, "or import unscanned Rust source"):
                VALIDATOR.require_no_rust_runtime_api_usage(source)

    def test_rejects_conditional_rust_path_module_attributes(self) -> None:
        for attribute in (
            '#[cfg_attr(feature = "unapproved", path = "../unreviewed-runtime.rs")]',
            '# /* unapproved */ [cfg_attr /* unapproved */ (feature = "unapproved", '
            'path /* unapproved */ = "../unreviewed-runtime.rs")]',
        ):
            with self.subTest(attribute=attribute), tempfile.TemporaryDirectory() as temporary_directory:
                source = Path(temporary_directory)
                (source / "runtime.rs").write_text(
                    f"{attribute}\nmod unreviewed_runtime;\n",
                    encoding="utf-8",
                )

                with self.assertRaisesRegex(SystemExit, "or import unscanned Rust source"):
                    VALIDATOR.require_no_rust_runtime_api_usage(source)

    def test_allows_rust_source_without_vendored_c_api_references(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            source = Path(temporary_directory)
            (source / "runtime.rs").write_text(
                "fn unavailable() { let status = \"unavailable\"; }\n",
                encoding="utf-8",
            )

            VALIDATOR.require_no_rust_runtime_api_usage(source)


if __name__ == "__main__":
    unittest.main()
