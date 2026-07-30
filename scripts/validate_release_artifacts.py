#!/usr/bin/env python3
"""Validate QuireForge Linux package artifacts and disposable lifecycle."""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import tempfile
from pathlib import Path

from release_contract import (
    CANONICAL_DESKTOP,
    DEBIAN_PACKAGE,
    SANDBOXD_DEBIAN_PACKAGE,
    EXPECTED_IMAGE,
    GLIBC_BASELINE,
    LEGACY_DESKTOP,
    RELEASE_OUTPUT_DIR,
    RELEASE_WORKFLOW_COMMAND,
    ROOT,
    appstream_validation_command,
    debian_artifact_filename,
    debian_version,
    glibc_requirement,
    glibc_version_text,
    replace_control_field,
    run,
    sha256,
    sandboxd_artifact_filename,
    source_version,
)


EXPECTED_DEPENDENCIES = {"libgtk-3-0", "libwebkit2gtk-4.1-0"}
EXPECTED_FILES = {
    "release-manifest.json",
    "SHA256SUMS",
}
DESKTOP_FIELDS = {
    "Categories": "Development;IDE;",
    "Comment": "An unofficial native Linux workspace for Codex",
    "Exec": "quireforge",
    "Icon": "quireforge",
    "Name": "QuireForge",
    "StartupNotify": "true",
    "StartupWMClass": "quireforge",
    "Terminal": "false",
    "Type": "Application",
}


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--artifact-dir",
        type=Path,
        default=RELEASE_OUTPUT_DIR,
    )
    parser.add_argument("--lifecycle", action="store_true")
    parser.add_argument("--smoke", action="store_true")
    parser.add_argument("--require-publishable", action="store_true")
    parser.add_argument("--expected-tag")
    return parser.parse_args()


def parse_desktop(path: Path) -> dict[str, str]:
    lines = path.read_text(encoding="utf-8").splitlines()
    if not lines or lines[0] != "[Desktop Entry]":
        raise RuntimeError(f"invalid desktop entry header: {path}")
    result = {}
    for line in lines[1:]:
        if not line or line.startswith("#"):
            continue
        key, separator, value = line.partition("=")
        if separator != "=" or key in result:
            raise RuntimeError(f"invalid desktop entry line: {line}")
        result[key] = value
    return result


def validate_desktop(path: Path) -> None:
    fields = parse_desktop(path)
    if fields != DESKTOP_FIELDS:
        raise RuntimeError(f"desktop entry fields do not match contract: {fields}")
    validator = shutil.which("desktop-file-validate")
    if validator:
        run([validator, str(path)])


def validate_metainfo(path: Path) -> None:
    validator = shutil.which("appstreamcli")
    if validator:
        run(appstream_validation_command(validator, path))


def validate_glibc_baseline(path: Path) -> tuple[int, int]:
    return glibc_requirement(path)


def validate_manifest_provenance(manifest: dict[str, object]) -> dict[str, object]:
    """Validate the fixed release-candidate provenance contract."""
    source = manifest.get("source")
    builder = manifest.get("builder")
    if not isinstance(source, dict) or not isinstance(builder, dict):
        raise RuntimeError("release manifest source or builder is malformed")
    if not re.fullmatch(r"[0-9a-f]{40}", str(source.get("commit", ""))):
        raise RuntimeError("release manifest source commit is invalid")
    required_builder = {
        "distribution": "ubuntu",
        "version": "22.04",
        "architecture": "x86_64",
        "image": EXPECTED_IMAGE,
    }
    required_provenance = {
        "kind": "pinned-ubuntu-22.04-container",
        "command": RELEASE_WORKFLOW_COMMAND,
        "containerImage": EXPECTED_IMAGE,
    }
    if (
        manifest.get("state") != "release-candidate"
        or source.get("treeState") != "clean"
        or "diffSha256" in source
        or builder != required_builder
        or manifest.get("provenance") != required_provenance
    ):
        raise RuntimeError("release artifacts require clean pinned-container provenance")
    return source


def validate_sandboxd_provenance(
    manifest: dict[str, object], artifacts: dict[str, Path], version: str
) -> None:
    source = manifest.get("source")
    sandboxd = manifest.get("sandboxd")
    if not isinstance(sandboxd, dict):
        raise RuntimeError("sandbox worker provenance is absent")
    artifact = sandboxd.get("artifact")
    if (
        sandboxd.get("version") != version
        or sandboxd.get("source") != source
        or not isinstance(artifact, dict)
        or artifact.get("filename") != artifacts["sandboxd-deb"].name
        or artifact.get("sha256") != sha256(artifacts["sandboxd-deb"])
        or artifact.get("size") != artifacts["sandboxd-deb"].stat().st_size
    ):
        raise RuntimeError("sandbox worker provenance is malformed or inconsistent")


def validate_manifest(
    artifact_dir: Path,
    require_publishable: bool,
    expected_tag: str | None,
) -> tuple[dict[str, object], dict[str, Path]]:
    manifest_path = artifact_dir / "release-manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    version = source_version()
    if manifest.get("schemaVersion") != 3 or manifest.get("version") != version:
        raise RuntimeError("release manifest schema or version mismatch")

    validate_manifest_provenance(manifest)

    if require_publishable:
        if expected_tag != f"v{version}":
            raise RuntimeError(
                f"release tag must be v{version}, not {expected_tag or 'unset'}"
            )

    artifacts = manifest.get("artifacts")
    if not isinstance(artifacts, list) or len(artifacts) != 2:
        raise RuntimeError("release manifest must contain exactly two Debian artifacts")
    by_format: dict[str, Path] = {}
    for entry in artifacts:
        if not isinstance(entry, dict):
            raise RuntimeError("release artifact entry is malformed")
        artifact_format = entry.get("format")
        filename = entry.get("filename")
        if artifact_format not in {"deb", "sandboxd-deb"} or not isinstance(filename, str):
            raise RuntimeError("release artifact format or filename is invalid")
        path = artifact_dir / filename
        if path.parent != artifact_dir or not path.is_file():
            raise RuntimeError(f"release artifact is missing: {filename}")
        if entry.get("sha256") != sha256(path):
            raise RuntimeError(f"release artifact checksum mismatch: {filename}")
        if entry.get("size") != path.stat().st_size:
            raise RuntimeError(f"release artifact size mismatch: {filename}")
        if entry.get("architecture") != "x86_64":
            raise RuntimeError("release artifact architecture mismatch")
        by_format[artifact_format] = path

    if set(by_format) != {"deb", "sandboxd-deb"}:
        raise RuntimeError("release manifest artifact formats are incomplete")
    expected_names = EXPECTED_FILES | {path.name for path in by_format.values()}
    actual_names = {path.name for path in artifact_dir.iterdir() if path.is_file()}
    if actual_names != expected_names:
        raise RuntimeError(
            f"unexpected release artifact set: {sorted(actual_names ^ expected_names)}"
        )

    checksum_lines = (artifact_dir / "SHA256SUMS").read_text(
        encoding="utf-8"
    ).splitlines()
    expected_lines = [
        f"{sha256(path)}  {path.name}" for path in sorted(by_format.values())
    ]
    if checksum_lines != expected_lines:
        raise RuntimeError("SHA256SUMS does not match the release artifacts")
    return manifest, by_format


def deb_field(package: Path, field: str) -> str:
    result = run(
        ["dpkg-deb", "--field", str(package), field],
        capture=True,
    )
    return result.stdout.strip()


def validate_debian(package: Path, version: str) -> tuple[int, int]:
    expected_name = debian_artifact_filename(version)
    if package.name != expected_name:
        raise RuntimeError(f"Debian filename mismatch: {package.name}")
    expected_fields = {
        "Package": DEBIAN_PACKAGE,
        "Version": debian_version(version),
        "Architecture": "amd64",
        "Homepage": "https://quireforge.jamesjennison.net",
        "Section": "devel",
        "Priority": "optional",
    }
    for field, expected in expected_fields.items():
        actual = deb_field(package, field)
        if actual != expected:
            raise RuntimeError(f"Debian {field} mismatch: {actual}")
    dependencies = {
        item.strip().split(" ", 1)[0]
        for item in deb_field(package, "Depends").split(",")
    }
    if not EXPECTED_DEPENDENCIES.issubset(dependencies):
        raise RuntimeError(f"Debian dependencies are incomplete: {dependencies}")

    with tempfile.TemporaryDirectory(prefix="quireforge-validate-deb-") as temporary:
        root = Path(temporary)
        run(["dpkg-deb", "--raw-extract", str(package), str(root)])
        if any((root / "DEBIAN").glob("*inst")) or any(
            (root / "DEBIAN").glob("*rm")
        ):
            raise RuntimeError("QuireForge packages must not contain maintainer scripts")
        canonical = root / "usr/share/applications" / CANONICAL_DESKTOP
        legacy = root / "usr/share/applications" / LEGACY_DESKTOP
        if legacy.exists() or not canonical.is_file():
            raise RuntimeError("Debian desktop filename does not match the canonical ID")
        validate_desktop(canonical)
        binary = root / "usr/bin/quireforge"
        if not binary.is_file() or not os.access(binary, os.X_OK):
            raise RuntimeError("Debian executable is missing or not executable")
        glibc = validate_glibc_baseline(binary)
        metainfo = (
            root
            / "usr/share/metainfo/io.github.codeframe78.QuireForge.metainfo.xml"
        )
        if not metainfo.is_file():
            raise RuntimeError("Debian AppStream metadata is missing")
        validate_metainfo(metainfo)
        md5sums = (root / "DEBIAN/md5sums").read_text(encoding="utf-8")
        md5_paths = {
            line.split(maxsplit=1)[1]
            for line in md5sums.splitlines()
            if len(line.split(maxsplit=1)) == 2
        }
        canonical_md5_path = f"usr/share/applications/{CANONICAL_DESKTOP}"
        legacy_md5_path = f"usr/share/applications/{LEGACY_DESKTOP}"
        if (
            canonical_md5_path not in md5_paths
            or legacy_md5_path in md5_paths
        ):
            raise RuntimeError("Debian md5sums retain the wrong desktop filename")
        return glibc


def validate_sandboxd(package: Path, version: str) -> tuple[int, int]:
    if package.name != sandboxd_artifact_filename(version):
        raise RuntimeError(f"sandbox worker Debian filename mismatch: {package.name}")
    expected_fields = {
        "Package": SANDBOXD_DEBIAN_PACKAGE,
        "Version": debian_version(version),
        "Architecture": "amd64",
    }
    for field, expected in expected_fields.items():
        if deb_field(package, field) != expected:
            raise RuntimeError(f"sandbox worker Debian {field} mismatch")
    with tempfile.TemporaryDirectory(prefix="quireforge-validate-sandboxd-") as temporary:
        root = Path(temporary)
        run(["dpkg-deb", "--raw-extract", str(package), str(root)])
        worker = root / "usr/sbin/quireforge-sandboxd"
        assets = root / "usr/lib/quireforge-sandboxd"
        service = root / "lib/systemd/system/quireforge-sandboxd.service"
        expected_assets = {"firecracker", "jailer", "vmlinux", "initramfs.cpio.gz", "SHA256SUMS"}
        if not worker.is_file() or not os.access(worker, os.X_OK):
            raise RuntimeError("sandbox worker executable is missing or not executable")
        if not service.is_file() or "PrivateNetwork=true" not in service.read_text(encoding="utf-8"):
            raise RuntimeError("sandbox worker must retain its zero-network systemd policy")
        if {path.name for path in assets.iterdir() if path.is_file()} != expected_assets:
            raise RuntimeError("sandbox worker assets are incomplete")
        sums = {
            fields[1]: fields[0]
            for fields in (
                line.split(maxsplit=1)
                for line in (assets / "SHA256SUMS").read_text(encoding="utf-8").splitlines()
            )
            if len(fields) == 2
        }
        for name in expected_assets - {"SHA256SUMS"}:
            if sums.get(name) != sha256(assets / name):
                raise RuntimeError("sandbox worker asset checksum mismatch")
        return validate_glibc_baseline(worker)


def validate_abi_evidence(
    manifest: dict[str, object],
    debian_glibc: tuple[int, int],
    sandboxd_glibc: tuple[int, int],
) -> None:
    expected = {
        "baseline": glibc_version_text(GLIBC_BASELINE),
        "highestRequired": glibc_version_text(max(debian_glibc, sandboxd_glibc)),
        "binaries": [
            {"format": "deb", "highestRequired": glibc_version_text(debian_glibc)},
            {"format": "sandboxd-deb", "highestRequired": glibc_version_text(sandboxd_glibc)},
        ],
    }
    if manifest.get("abi") != expected:
        raise RuntimeError("release ABI evidence is absent, malformed, or inconsistent")


def smoke_packages(debian: Path) -> None:
    helper = ROOT / "scripts/smoke_linux_package.py"
    with tempfile.TemporaryDirectory(prefix="quireforge-smoke-deb-") as temporary:
        root = Path(temporary)
        run(["dpkg-deb", "--extract", str(debian), str(root)])
        debian_binary = root / "usr/bin/quireforge"
        run(
            [
                "xvfb-run",
                "--auto-servernum",
                "python3",
                str(helper),
                "--label",
                "Debian package",
                str(debian_binary),
            ]
        )


def build_previous_package(current: Path, output: Path) -> None:
    with tempfile.TemporaryDirectory(prefix="quireforge-previous-deb-") as temporary:
        root = Path(temporary)
        run(["dpkg-deb", "--raw-extract", str(current), str(root)])
        control = root / "DEBIAN/control"
        updated = replace_control_field(
            control.read_text(encoding="utf-8"),
            "Version",
            "0.0.0",
        )
        control.write_text(updated, encoding="utf-8")
        run(
            [
                "dpkg-deb",
                "--root-owner-group",
                "--build",
                str(root),
                str(output),
            ]
        )


def lifecycle(package: Path, expected_version: str) -> None:
    with tempfile.TemporaryDirectory(prefix="quireforge-lifecycle-") as temporary:
        root = Path(temporary) / "root"
        admin = root / "var/lib/dpkg"
        admin.mkdir(parents=True)
        (admin / "status").write_text("", encoding="utf-8")
        project = root / "home/tester/project/README.md"
        metadata = root / "home/tester/.local/share/quireforge/metadata.db"
        project.parent.mkdir(parents=True)
        metadata.parent.mkdir(parents=True)
        project.write_text("preserve project\n", encoding="utf-8")
        metadata.write_text("preserve metadata\n", encoding="utf-8")
        previous = Path(temporary) / "quireforge_0.0.0_amd64.deb"
        build_previous_package(package, previous)

        base = [
            "dpkg",
            f"--root={root}",
            "--force-not-root",
            "--force-depends",
            "--force-script-chrootless",
        ]
        run([*base, "--install", str(previous)])
        query = ["dpkg-query", f"--admindir={admin}", "--showformat=${Version}", "--show"]
        if run([*query, DEBIAN_PACKAGE], capture=True).stdout != "0.0.0":
            raise RuntimeError("disposable initial package installation failed")

        run([*base, "--install", str(package)])
        if run([*query, DEBIAN_PACKAGE], capture=True).stdout != expected_version:
            raise RuntimeError("disposable package upgrade failed")
        if not (root / "usr/bin/quireforge").is_file():
            raise RuntimeError("upgraded disposable executable is missing")

        run([*base, "--remove", DEBIAN_PACKAGE])
        if (root / "usr/bin/quireforge").exists():
            raise RuntimeError("package uninstall retained the executable")
        if (root / "usr/share/applications" / CANONICAL_DESKTOP).exists():
            raise RuntimeError("package uninstall retained the desktop entry")
        if project.read_text(encoding="utf-8") != "preserve project\n":
            raise RuntimeError("package uninstall altered the attached project")
        if metadata.read_text(encoding="utf-8") != "preserve metadata\n":
            raise RuntimeError("package uninstall altered application metadata")


def main() -> int:
    arguments = parse_arguments()
    artifact_dir = arguments.artifact_dir.resolve()
    manifest, artifacts = validate_manifest(
        artifact_dir,
        arguments.require_publishable,
        arguments.expected_tag,
    )
    version = str(manifest["version"])
    debian_glibc = validate_debian(artifacts["deb"], version)
    sandboxd_glibc = validate_sandboxd(artifacts["sandboxd-deb"], version)
    validate_sandboxd_provenance(manifest, artifacts, version)
    validate_abi_evidence(manifest, debian_glibc, sandboxd_glibc)
    if arguments.lifecycle:
        lifecycle(artifacts["deb"], debian_version(version))
    if arguments.smoke:
        smoke_packages(artifacts["deb"])
    print(f"validated Linux release artifacts: {artifact_dir.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
