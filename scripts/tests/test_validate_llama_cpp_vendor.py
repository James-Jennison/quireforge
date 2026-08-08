"""Focused tests for the closed M63 llama.cpp build configuration guard."""

from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "validate_llama_cpp_vendor", ROOT / "scripts" / "validate_llama_cpp_vendor.py"
)
assert SPEC is not None and SPEC.loader is not None
VALIDATOR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VALIDATOR)


class CmakeOptionsTests(unittest.TestCase):
    def test_rejects_a_symlinked_vendored_source_root(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            source = temporary_root / "source"
            source.mkdir()
            source_link = temporary_root / "source-link"
            source_link.symlink_to(source, target_is_directory=True)

            with self.assertRaisesRegex(SystemExit, "source root must be a real directory"):
                VALIDATOR.require_regular_vendored_source_tree(source_link)

    def test_rejects_a_symlink_inside_the_vendored_source_tree(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            source = Path(temporary_directory) / "source"
            source.mkdir()
            source_file = source / "source-file"
            source_file.write_text("source", encoding="utf-8")
            (source / "source-link").symlink_to(source_file)

            with self.assertRaisesRegex(SystemExit, "vendored symlink found: source-link"):
                VALIDATOR.require_regular_vendored_source_tree(source)

    def test_reads_the_exact_closed_option_list(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")

        self.assertEqual(VALIDATOR.cmake_options(build), VALIDATOR.EXPECTED_CMAKE_OPTIONS)

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

    def test_requires_explicit_curl_and_vendored_git_probe_disablement(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")

        options = VALIDATOR.cmake_options(build)

        self.assertIn("-DLLAMA_CURL=OFF", options)
        self.assertIn("-DGIT_EXE=", options)

    def test_requires_the_optional_llamafile_integration_to_be_disabled(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")

        options = VALIDATOR.cmake_options(build)

        self.assertIn("-DGGML_LLAMAFILE=OFF", options)

    def test_requires_the_closed_option_list_once_without_extra_definitions(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")

        VALIDATOR.require_closed_cmake_invocation(build)

    def test_accepts_rustfmt_whitespace_in_the_configure_declaration(self) -> None:
        build = '''let mut configure =
            Command::new("cmake");
            configure.arg("-DCMAKE_BUILD_TYPE=Release");
            configure.args(LLAMA_CPP_CMAKE_OPTIONS);
            run(
                &mut configure,
                "closed llama.cpp static-library configuration",
            );'''

        VALIDATOR.require_closed_cmake_invocation(build)

    def test_rejects_an_unguarded_cmake_definition(self) -> None:
        build = '''let mut configure = Command::new("cmake");
            configure.arg("-DGGML_CUDA=ON");
            configure.args(LLAMA_CPP_CMAKE_OPTIONS);
            run(&mut configure, "closed llama.cpp static-library configuration");'''

        with self.assertRaisesRegex(SystemExit, "unguarded -D option"):
            VALIDATOR.require_closed_cmake_invocation(build)

    def test_requires_the_closed_toolchain_environment_scrub(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")

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

    def test_rejects_a_configuration_that_does_not_pass_the_verified_source(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('.arg(&source_dir)', '.arg(&build_dir)', 1)

        with self.assertRaisesRegex(SystemExit, "must use only the verified source"):
            VALIDATOR.require_verified_cmake_configure_arguments(build)

    def test_rejects_a_cmake_build_with_an_extra_target(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('.arg("llama");', '.arg("all");', 1)

        with self.assertRaisesRegex(SystemExit, "must target only"):
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
