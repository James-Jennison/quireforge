from __future__ import annotations

import json
import re
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts.release_contract import (
    EXPECTED_IMAGE,
    HOST_DEVELOPMENT_TARGET_DIR,
    RELEASE_BUILDER_ENV,
    RELEASE_BUILDER_VALUE,
    RELEASE_OUTPUT_DIR,
    ROOT,
    assert_authoritative_release_builder,
    appstream_validation_command,
    architectures,
    debian_artifact_filename,
    debian_version,
    package_output_dir,
    replace_control_field,
    source_version,
)


class PackageContractTests(unittest.TestCase):
    def test_all_source_versions_match_the_beta_candidate(self) -> None:
        self.assertEqual(source_version(), "0.1.0-beta.35")

    def test_debian_metadata_and_artifact_versions_are_deliberately_distinct(
        self,
    ) -> None:
        self.assertEqual(debian_version("0.1.0-beta.2"), "0.1.0~beta.2")
        self.assertEqual(
            debian_artifact_filename("0.1.0-beta.2"),
            "quireforge_0.1.0.beta.2_amd64.deb",
        )
        self.assertNotIn("~", debian_artifact_filename("0.1.0-beta.2"))
        self.assertEqual(debian_version("0.1.0"), "0.1.0")
        self.assertEqual(
            debian_artifact_filename("0.1.0"),
            "quireforge_0.1.0_amd64.deb",
        )
        self.assertEqual(architectures(), ("x86_64", "amd64", "amd64"))

    def test_control_replacement_is_exact_and_fails_closed(self) -> None:
        control = "Package: quire-forge\nVersion: 0.1.0-beta.2\n"
        self.assertEqual(
            replace_control_field(control, "Package", "quireforge"),
            "Package: quireforge\nVersion: 0.1.0-beta.2\n",
        )
        with self.assertRaises(RuntimeError):
            replace_control_field(control, "Architecture", "amd64")

    def test_appstream_validation_is_offline_and_not_skipped(self) -> None:
        metadata = Path("/app/usr/share/metainfo/quireforge.appdata.xml")
        self.assertEqual(
            appstream_validation_command("/usr/bin/appstreamcli", metadata),
            [
                "/usr/bin/appstreamcli",
                "validate",
                "--no-net",
                str(metadata),
            ],
        )

    def test_packaging_images_and_tools_are_digest_pinned(self) -> None:
        dockerfile = (ROOT / "packaging/linux/Dockerfile").read_text(
            encoding="utf-8"
        )
        from_lines = [
            line for line in dockerfile.splitlines() if line.startswith("FROM ")
        ]
        self.assertEqual(len(from_lines), 3)
        for line in from_lines:
            self.assertRegex(line, r"@sha256:[0-9a-f]{64}(?:\s+AS\s+\w+)?$")
        self.assertIn(EXPECTED_IMAGE, dockerfile)
        self.assertIn("FROM rust:1.95.0-slim-bookworm@", dockerfile)
        cargo_manifest = (
            ROOT / "apps/desktop/src-tauri/Cargo.toml"
        ).read_text(encoding="utf-8")
        self.assertIn('rust-version = "1.95"', cargo_manifest)


    def test_tauri_bundle_contract_is_active_and_canonical(self) -> None:
        config = json.loads(
            (ROOT / "apps/desktop/src-tauri/tauri.conf.json").read_text(
                encoding="utf-8"
            )
        )
        bundle = config["bundle"]
        self.assertTrue(bundle["active"])
        self.assertEqual(bundle["targets"], ["deb"])
        self.assertEqual(bundle["category"], "DeveloperTool")
        self.assertEqual(bundle["license"], "Apache-2.0")
        self.assertEqual(
            bundle["homepage"], "https://quireforge.jamesjennison.net"
        )
        self.assertEqual(
            bundle["linux"]["deb"]["desktopTemplate"],
            "desktop-template.desktop",
        )
        metainfo = (
            ROOT
            / "apps/desktop/src-tauri/metainfo"
            / "io.github.codeframe78.QuireForge.metainfo.xml"
        )
        self.assertTrue(metainfo.is_file())

    def test_host_and_authoritative_package_outputs_are_separate(self) -> None:
        with patch.dict("os.environ", {}, clear=False):
            self.assertEqual(
                package_output_dir(), HOST_DEVELOPMENT_TARGET_DIR / "packages"
            )
        with patch.dict(
            "os.environ",
            {RELEASE_BUILDER_ENV: RELEASE_BUILDER_VALUE},
            clear=False,
        ):
            self.assertEqual(package_output_dir(), RELEASE_OUTPUT_DIR)

        package_scripts = json.loads((ROOT / "package.json").read_text())[
            "scripts"
        ]
        self.assertIn("CARGO_TARGET_DIR=target/host-development", package_scripts["package:linux"])
        self.assertNotIn("package_linux.py", package_scripts["package:linux"])
        self.assertIn("package_linux.py", package_scripts["package:linux:release"])
        with patch.dict("os.environ", {}, clear=False):
            with self.assertRaisesRegex(RuntimeError, "container"):
                assert_authoritative_release_builder()

    def test_release_manifest_contract_requires_pinned_provenance_and_abi(self) -> None:
        schema = json.loads(
            (ROOT / "packaging/release-manifest.schema.json").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(schema["properties"]["schemaVersion"], {"const": 3})
        self.assertEqual(schema["properties"]["state"], {"const": "release-candidate"})
        self.assertEqual(
            schema["properties"]["source"]["properties"]["treeState"],
            {"const": "clean"},
        )
        self.assertIn("provenance", schema["required"])
        self.assertIn("abi", schema["required"])
        self.assertIn("sandboxd", schema["required"])
        self.assertEqual(
            schema["properties"]["provenance"]["properties"]["command"],
            {"const": "scripts/run_linux_package_container.sh"},
        )
        self.assertIn(
            "for artifact_format, version in sorted(observed)",
            (ROOT / "scripts/package_linux.py").read_text(encoding="utf-8"),
        )
        self.assertIn(
            "sandboxd-deb",
            (ROOT / "scripts/validate_release_artifacts.py").read_text(encoding="utf-8"),
        )

    def test_release_ci_uses_the_authoritative_container_entrypoint(self) -> None:
        workflow = (ROOT / ".github/workflows/linux-release.yml").read_text(
            encoding="utf-8"
        )
        self.assertIn("bash scripts/run_linux_package_container.sh", workflow)
        self.assertIn("target/ubuntu-22.04/release/packages/", workflow)


if __name__ == "__main__":
    unittest.main()
