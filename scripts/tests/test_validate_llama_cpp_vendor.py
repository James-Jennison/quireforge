"""Focused tests for the closed M63 llama.cpp build configuration guard."""

from __future__ import annotations

import importlib.util
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

    def test_rejects_a_non_vendored_cmake_source(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace(
            'manifest_dir.join("../../../third_party/llama.cpp")',
            'manifest_dir.join("../../../third_party/not-llama.cpp")',
        )

        with self.assertRaisesRegex(SystemExit, "source is not the verified"):
            VALIDATOR.require_verified_cmake_source(build)

    def test_rejects_a_cmake_build_with_an_extra_target(self) -> None:
        build = (ROOT / "apps" / "desktop" / "src-tauri" / "build.rs").read_text(encoding="utf-8")
        build = build.replace('.arg("llama");', '.arg("all");', 1)

        with self.assertRaisesRegex(SystemExit, "must target only"):
            VALIDATOR.require_closed_cmake_build_invocation(build)


if __name__ == "__main__":
    unittest.main()
