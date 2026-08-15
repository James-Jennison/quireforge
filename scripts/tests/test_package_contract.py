from __future__ import annotations

import json
import hashlib
import re
import sys
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
    source_date_epoch,
    source_record,
)
sys.path.insert(0, str(ROOT / "scripts"))
from package_linux import finalize, staging_dir


class PackageContractTests(unittest.TestCase):
    def write_release_set(
        self, root: Path, version: str, checksum_order: tuple[str, str] = ("deb", "sandboxd-deb"),
    ) -> None:
        artifacts = []
        for artifact_format, name in [
            ("deb", f"quireforge_0.1.0.beta.{version}_amd64.deb"),
            ("sandboxd-deb", f"quireforge-sandboxd_0.1.0.beta.{version}_amd64.deb"),
        ]:
            path = root / name
            path.write_bytes(f"{artifact_format}-{version}".encode())
            artifacts.append({"format": artifact_format, "filename": name,
                              "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                              "size": path.stat().st_size})
        (root / "release-manifest.json").write_text(json.dumps({
            "schemaVersion": 3, "state": "release-candidate",
            "version": f"0.1.0-beta.{version}",
            "source": {"commit": "a" * 40}, "artifacts": artifacts,
        }), encoding="utf-8")
        by_format = {item["format"]: item for item in artifacts}
        (root / "SHA256SUMS").write_text(
            "\n".join(
                f"{by_format[artifact_format]['sha256']}  {by_format[artifact_format]['filename']}"
                for artifact_format in checksum_order
            ) + "\n",
            encoding="utf-8")

    def test_finalizer_archives_a_coherent_prior_release_before_promoting(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "packages"
            output.mkdir()
            self.write_release_set(output, "48")
            candidate = staging_dir(output, "0.1.0-beta.51")
            candidate.mkdir()
            self.write_release_set(candidate, "51")
            self.assertEqual(finalize(output, "0.1.0-beta.51"), 0)
            archived = output.parent / "archive" / "0.1.0-beta.48"
            self.assertTrue((archived / "release-manifest.json").is_file())
            self.assertTrue((output / "quireforge_0.1.0.beta.51_amd64.deb").is_file())

    def test_finalizer_ignores_a_stale_staged_candidate_when_promoting(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "packages"
            output.mkdir()
            self.write_release_set(output, "48")
            stale = staging_dir(output, "0.1.0-beta.50")
            stale.mkdir()
            self.write_release_set(stale, "50")
            candidate = staging_dir(output, "0.1.0-beta.51")
            candidate.mkdir()
            self.write_release_set(candidate, "51")

            self.assertEqual(finalize(output, "0.1.0-beta.51"), 0)

            self.assertTrue((stale / "release-manifest.json").is_file())
            self.assertTrue((output / "quireforge_0.1.0.beta.51_amd64.deb").is_file())

    def test_finalizer_accepts_beta_48_checksum_order_and_preserves_it_verbatim(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "packages"
            output.mkdir()
            self.write_release_set(output, "48", ("sandboxd-deb", "deb"))
            prior_checksums = (output / "SHA256SUMS").read_bytes()
            candidate = staging_dir(output, "0.1.0-beta.51")
            candidate.mkdir()
            self.write_release_set(candidate, "51")
            finalize(output, "0.1.0-beta.51")
            archived = output.parent / "archive" / "0.1.0-beta.48"
            self.assertEqual((archived / "SHA256SUMS").read_bytes(), prior_checksums)
            self.assertTrue((output / "quireforge_0.1.0.beta.51_amd64.deb").is_file())

    def test_finalizer_rejects_bad_checksum_mapping_entries(self) -> None:
        for line in [
            "0" * 64 + "  duplicate.deb\n" + "0" * 64 + "  duplicate.deb\n",
            "0" * 64 + "  extra.deb\n",
            "X" * 64 + "  quireforge_0.1.0.beta.48_amd64.deb\n",
            "0" * 64 + "  nested/file.deb\n",
        ]:
            with tempfile.TemporaryDirectory() as temporary:
                output = Path(temporary) / "packages"
                output.mkdir()
                self.write_release_set(output, "48")
                (output / "SHA256SUMS").write_text(line, encoding="utf-8")
                candidate = staging_dir(output, "0.1.0-beta.51")
                candidate.mkdir()
                self.write_release_set(candidate, "51")
                with self.assertRaisesRegex(RuntimeError, "checksum"):
                    finalize(output, "0.1.0-beta.51")

    def test_finalizer_refuses_a_partial_prior_release_without_deleting_it(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "packages"
            output.mkdir()
            (output / "release-manifest.json").write_text("{}", encoding="utf-8")
            candidate = staging_dir(output, "0.1.0-beta.51")
            candidate.mkdir()
            self.write_release_set(candidate, "51")
            with self.assertRaisesRegex(RuntimeError, "incomplete|incoherent"):
                finalize(output, "0.1.0-beta.51")
            self.assertTrue((output / "release-manifest.json").is_file())
            self.assertTrue((candidate / "release-manifest.json").is_file())

    def test_finalizer_allows_first_promotion_without_an_archive(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "packages"
            output.mkdir()
            candidate = staging_dir(output, "0.1.0-beta.51")
            candidate.mkdir()
            self.write_release_set(candidate, "51")
            finalize(output, "0.1.0-beta.51")
            self.assertFalse((output.parent / "archive").exists())
            self.assertTrue((output / "release-manifest.json").is_file())

    def test_finalizer_preserves_a_conflicting_legacy_archive_with_provenance_name(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "packages"
            output.mkdir()
            self.write_release_set(output, "48")
            archive = output.parent / "archive" / "0.1.0-beta.48"
            archive.mkdir(parents=True)
            self.write_release_set(archive, "48")
            (archive / "quireforge_0.1.0.beta.48_amd64.deb").write_bytes(b"conflict")
            candidate = staging_dir(output, "0.1.0-beta.51")
            candidate.mkdir()
            self.write_release_set(candidate, "51")
            self.assertEqual(finalize(output, "0.1.0-beta.51"), 0)
            qualified_archive = output.parent / "archive" / f"0.1.0-beta.48-{'a' * 40}"
            self.assertEqual(
                (archive / "quireforge_0.1.0.beta.48_amd64.deb").read_bytes(),
                b"conflict",
            )
            self.assertTrue((qualified_archive / "release-manifest.json").is_file())
            self.assertTrue((output / "quireforge_0.1.0.beta.51_amd64.deb").is_file())

    def test_finalizer_refuses_a_conflicting_provenance_qualified_archive(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "packages"
            output.mkdir()
            self.write_release_set(output, "48")
            legacy_archive = output.parent / "archive" / "0.1.0-beta.48"
            legacy_archive.mkdir(parents=True)
            self.write_release_set(legacy_archive, "48")
            (legacy_archive / "quireforge_0.1.0.beta.48_amd64.deb").write_bytes(b"legacy")
            qualified_archive = output.parent / "archive" / f"0.1.0-beta.48-{'a' * 40}"
            qualified_archive.mkdir()
            self.write_release_set(qualified_archive, "48")
            (qualified_archive / "quireforge_0.1.0.beta.48_amd64.deb").write_bytes(b"conflict")
            candidate = staging_dir(output, "0.1.0-beta.51")
            candidate.mkdir()
            self.write_release_set(candidate, "51")
            with self.assertRaisesRegex(RuntimeError, "provenance-qualified release archive conflicts"):
                finalize(output, "0.1.0-beta.51")
            self.assertTrue((output / "quireforge_0.1.0.beta.48_amd64.deb").is_file())

    def test_finalizer_restores_prior_canonical_set_after_promotion_failure(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "packages"
            output.mkdir()
            self.write_release_set(output, "48")
            prior_bytes = (output / "quireforge_0.1.0.beta.48_amd64.deb").read_bytes()
            candidate = staging_dir(output, "0.1.0-beta.51")
            candidate.mkdir()
            self.write_release_set(candidate, "51")
            real_move = __import__("shutil").move

            def fail_candidate_move(source: Path, destination: Path):
                if Path(source).parent == candidate:
                    raise OSError("promotion failed")
                return real_move(source, destination)

            with patch("package_linux.shutil.move", side_effect=fail_candidate_move):
                with self.assertRaisesRegex(RuntimeError, "promotion failed"):
                    finalize(output, "0.1.0-beta.51")
            self.assertEqual(
                (output / "quireforge_0.1.0.beta.48_amd64.deb").read_bytes(),
                prior_bytes,
            )
    def test_all_source_versions_match_the_beta_candidate(self) -> None:
        self.assertEqual(source_version(), "0.1.0-beta.76")

    def test_sandbox_worker_uses_the_aligned_release_version_contract(self) -> None:
        source = (ROOT / "scripts/package_sandboxd.py").read_text(encoding="utf-8")
        self.assertIn("version = source_version()", source)
        self.assertNotIn("requires beta", source)

    def test_permanent_desktop_bundle_budget_is_closed_and_bounded(self) -> None:
        budget = json.loads(
            (ROOT / "apps/desktop/scripts/bundle-budget.json").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(
            budget,
            {
                "entryBytes": 256 * 1024,
                "appShellBytes": 320 * 1024,
                "totalJavaScriptBytes": 1280 * 1024,
                "stylesheetsBytes": 144 * 1024,
            },
        )
        self.assertLess(budget["appShellBytes"], budget["totalJavaScriptBytes"])

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

    def test_container_provenance_is_explicit_and_does_not_require_git(self) -> None:
        revision = "b" * 40
        with patch.dict("os.environ", {
            RELEASE_BUILDER_ENV: RELEASE_BUILDER_VALUE,
            "QUIRE_FORGE_SOURCE_REVISION": revision,
            "SOURCE_DATE_EPOCH": "1786752436",
        }, clear=False):
            self.assertEqual(source_record(), (revision, "clean", None))
            self.assertEqual(source_date_epoch(), 1786752436)
        for environment in (
            {RELEASE_BUILDER_ENV: RELEASE_BUILDER_VALUE, "SOURCE_DATE_EPOCH": "1"},
            {RELEASE_BUILDER_ENV: RELEASE_BUILDER_VALUE, "QUIRE_FORGE_SOURCE_REVISION": "B" * 40, "SOURCE_DATE_EPOCH": "1"},
            {RELEASE_BUILDER_ENV: RELEASE_BUILDER_VALUE, "QUIRE_FORGE_SOURCE_REVISION": revision},
        ):
            with patch.dict("os.environ", environment, clear=True):
                with self.assertRaisesRegex(RuntimeError, "QUIRE_FORGE_SOURCE_REVISION|SOURCE_DATE_EPOCH"):
                    source_record() if "QUIRE_FORGE_SOURCE_REVISION" not in environment or "SOURCE_DATE_EPOCH" in environment else source_date_epoch()

    def test_container_packaging_path_does_not_inspect_git_metadata(self) -> None:
        builder = (ROOT / "scripts/run_linux_package_container.sh").read_text(encoding="utf-8")
        release_contract = (ROOT / "scripts/release_contract.py").read_text(encoding="utf-8")
        guest_assets = (ROOT / "packaging/sandbox/build_guest_assets.sh").read_text(encoding="utf-8")
        self.assertIn('QUIRE_FORGE_SOURCE_REVISION="$source_revision"', builder)
        self.assertIn('return supplied_source_revision(), "clean", None', release_contract)
        self.assertNotIn("git rev-parse", guest_assets)
        self.assertNotIn(".git", guest_assets)

    def test_authoritative_builder_uses_only_the_verified_sandbox_source_cache(self) -> None:
        builder = (ROOT / "scripts/run_linux_package_container.sh").read_text(
            encoding="utf-8"
        )
        guest_assets = (
            ROOT / "packaging/sandbox/build_guest_assets.sh"
        ).read_text(encoding="utf-8")
        self.assertIn("QUIRE_FORGE_SANDBOX_SOURCE_CACHE=/cache/sandbox-sources", builder)
        self.assertIn('"$cache_root/sandbox-sources"', builder)
        self.assertIn("authoritative sandbox source cache required", guest_assets)
        self.assertIn("cache_verified_source.sh", guest_assets)
        self.assertIn("KBUILD_BUILD_TIMESTAMP", guest_assets)
        self.assertIn("KBUILD_BUILD_USER=quireforge", guest_assets)
        self.assertIn("KBUILD_BUILD_HOST=ubuntu-22.04", guest_assets)
        self.assertIn("--reproducible", guest_assets)
        self.assertIn("gzip -n -9", guest_assets)

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
