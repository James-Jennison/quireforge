#!/usr/bin/env python3
"""Normalize Tauri's Linux bundles into QuireForge release candidates."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import sys
import tempfile
from pathlib import Path

from package_sandboxd import build as build_sandboxd

from release_contract import (
    CANONICAL_DESKTOP,
    DEBIAN_PACKAGE,
    LEGACY_DESKTOP,
    RELEASE_WORKFLOW_COMMAND,
    ROOT,
    TAURI_BUNDLE_BASENAME,
    assert_authoritative_release_builder,
    architectures,
    builder_record,
    cargo_target_dir,
    debian_artifact_filename,
    debian_version,
    glibc_requirement,
    glibc_version_text,
    package_output_dir,
    replace_control_field,
    run,
    set_tree_timestamp,
    sha256,
    source_date_epoch,
    source_record,
    source_version,
    sandboxd_artifact_filename,
    write_sha256sums,
)


def one_match(root: Path, pattern: str, label: str) -> Path:
    matches = sorted(root.glob(pattern))
    if len(matches) != 1:
        rendered = ", ".join(path.name for path in matches) or "none"
        raise RuntimeError(f"expected one {label}, found {rendered}")
    return matches[0]


def rebuild_debian(
    raw_package: Path,
    output: Path,
    version: str,
    timestamp: int,
) -> None:
    with tempfile.TemporaryDirectory(prefix="quireforge-deb-") as temporary:
        root = Path(temporary) / "root"
        run(["dpkg-deb", "--raw-extract", str(raw_package), str(root)])

        control_path = root / "DEBIAN/control"
        control = control_path.read_text(encoding="utf-8")
        control = replace_control_field(control, "Package", DEBIAN_PACKAGE)
        control = replace_control_field(control, "Version", debian_version(version))
        control_path.write_text(control, encoding="utf-8")

        applications = root / "usr/share/applications"
        generated_desktop = applications / LEGACY_DESKTOP
        canonical_desktop = applications / CANONICAL_DESKTOP
        if not generated_desktop.is_file() or canonical_desktop.exists():
            raise RuntimeError("unexpected Debian desktop-entry layout")
        generated_desktop.rename(canonical_desktop)

        package_files = sorted(
            path
            for path in root.rglob("*")
            if path.is_file() and root / "DEBIAN" not in path.parents
        )
        md5_lines = []
        for path in package_files:
            digest = hashlib.md5(path.read_bytes(), usedforsecurity=False).hexdigest()
            md5_lines.append(f"{digest}  {path.relative_to(root).as_posix()}")
        (root / "DEBIAN/md5sums").write_text(
            "\n".join(md5_lines) + "\n",
            encoding="utf-8",
        )

        set_tree_timestamp(root, timestamp)
        environment = os.environ.copy()
        environment["SOURCE_DATE_EPOCH"] = str(timestamp)
        run(
            [
                "dpkg-deb",
                "--root-owner-group",
                "--uniform-compression",
                "-Zxz",
                "-z9",
                "--build",
                str(root),
                str(output),
            ],
            env=environment,
        )


def staging_dir(output_dir: Path, version: str) -> Path:
    return output_dir / f".candidate-{version}"


def abi_evidence(debian: Path, sandboxd: Path) -> dict[str, object]:
    """Inspect the shipped executable in each installable artifact."""
    observed = []
    with tempfile.TemporaryDirectory(prefix="quireforge-release-abi-") as temporary:
        root = Path(temporary)
        deb_root = root / "deb"
        run(["dpkg-deb", "--extract", str(debian), str(deb_root)])
        deb_binary = deb_root / "usr/bin/quireforge"
        if not deb_binary.is_file():
            raise RuntimeError("Debian package is missing its shipped executable")
        observed.append(("deb", glibc_requirement(deb_binary)))

        sandbox_root = root / "sandboxd"
        run(["dpkg-deb", "--extract", str(sandboxd), str(sandbox_root)])
        sandbox_binary = sandbox_root / "usr/sbin/quireforge-sandboxd"
        if not sandbox_binary.is_file():
            raise RuntimeError("sandbox worker package is missing its shipped executable")
        observed.append(("sandboxd-deb", glibc_requirement(sandbox_binary)))

    by_format = [
        {"format": artifact_format, "highestRequired": glibc_version_text(version)}
        for artifact_format, version in sorted(observed)
    ]
    return {
        "baseline": "GLIBC_2.35",
        "highestRequired": glibc_version_text(max(version for _, version in observed)),
        "binaries": by_format,
    }


def finalize(output_dir: Path, version: str) -> int:
    candidate = staging_dir(output_dir, version)
    expected = {
        "release-manifest.json",
        "SHA256SUMS",
        debian_artifact_filename(version),
        sandboxd_artifact_filename(version),
    }
    if not candidate.is_dir() or {path.name for path in candidate.iterdir()} != expected:
        raise RuntimeError("validated candidate set is incomplete")
    candidate_manifest = coherent_release_set(candidate)
    if candidate_manifest["version"] != version:
        raise RuntimeError("validated candidate version is incoherent")
    archive_root = output_dir.parent / "archive"
    existing = [path for path in output_dir.iterdir() if path != candidate]
    prior_archive: Path | None = None
    if existing:
        if any(not path.is_file() for path in existing):
            raise RuntimeError("refusing non-file canonical release entry")
        prior = coherent_release_set(output_dir)
        archive = archive_root / prior["version"]
        prior_archive = archive
        archive_root.mkdir(exist_ok=True)
        if archive.exists():
            if not archive.is_dir() or not release_sets_identical(output_dir, archive):
                raise RuntimeError("existing release archive conflicts")
        else:
            archive.mkdir()
            moved = []
            try:
                for path in existing:
                    destination = archive / path.name
                    shutil.move(path, destination)
                    moved.append((path, destination))
                coherent_release_set(archive)
            except Exception:
                for source, destination in reversed(moved):
                    if destination.exists():
                        shutil.move(destination, source)
                raise
        if archive.exists():
            coherent_release_set(archive)
            for path in existing:
                if path.exists():
                    path.unlink()
    moved_candidate = []
    try:
        for artifact in candidate.iterdir():
            destination = output_dir / artifact.name
            shutil.move(artifact, destination)
            moved_candidate.append((artifact, destination))
        candidate.rmdir()
    except Exception:
        for source, destination in reversed(moved_candidate):
            if destination.exists():
                shutil.move(destination, source)
        if prior_archive is not None:
            try:
                for archived in prior_archive.iterdir():
                    if archived.is_file():
                        shutil.copy2(archived, output_dir / archived.name)
                coherent_release_set(output_dir)
            except Exception as restoration_error:
                raise RuntimeError(
                    "candidate promotion failed and prior release restoration failed"
                ) from restoration_error
        raise RuntimeError("candidate promotion failed without replacement")
    print(f"promoted validated Linux release candidates: {output_dir}")
    return 0


def coherent_release_set(root: Path) -> dict[str, object]:
    manifest_path = root / "release-manifest.json"
    checksums_path = root / "SHA256SUMS"
    if not manifest_path.is_file() or not checksums_path.is_file():
        raise RuntimeError("canonical release set is incomplete")
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise RuntimeError("canonical release manifest is malformed") from error
    artifacts = manifest.get("artifacts")
    version = manifest.get("version")
    source = manifest.get("source")
    if (manifest.get("schemaVersion") != 3 or manifest.get("state") != "release-candidate"
            or not isinstance(version, str) or not isinstance(source, dict)
            or not isinstance(source.get("commit"), str)
            or len(source["commit"]) != 40
            or any(character not in "0123456789abcdef" for character in source["commit"])
            or not isinstance(artifacts, list)
            or len(artifacts) != 2):
        raise RuntimeError("canonical release manifest is incoherent")
    names = {"release-manifest.json", "SHA256SUMS"}
    checksum_lines = []
    expected_names = {
        debian_artifact_filename(version),
        sandboxd_artifact_filename(version),
    }
    for artifact in artifacts:
        if not isinstance(artifact, dict):
            raise RuntimeError("canonical release artifact is malformed")
        name, digest, size = artifact.get("filename"), artifact.get("sha256"), artifact.get("size")
        if not isinstance(name, str) or Path(name).name != name or not isinstance(digest, str) or not isinstance(size, int):
            raise RuntimeError("canonical release artifact is incoherent")
        path = root / name
        if not path.is_file() or path.stat().st_size != size or sha256(path) != digest:
            raise RuntimeError("canonical release artifact does not match manifest")
        names.add(name)
        checksum_lines.append(f"{digest}  {name}")
    if ({item["filename"] for item in artifacts} != expected_names
            or {item.get("format") for item in artifacts} != {"deb", "sandboxd-deb"}):
        raise RuntimeError("canonical release artifact names are incoherent")
    if {path.name for path in root.iterdir() if path.is_file()} != names:
        raise RuntimeError("canonical release file set is incoherent")
    if checksums_path.read_text(encoding="utf-8").splitlines() != sorted(checksum_lines):
        raise RuntimeError("canonical release checksums are incoherent")
    return manifest


def release_sets_identical(source: Path, archive: Path) -> bool:
    try:
        source_manifest = coherent_release_set(source)
        archive_manifest = coherent_release_set(archive)
    except RuntimeError:
        return False
    if source_manifest.get("version") != archive_manifest.get("version"):
        return False
    source_names = {path.name for path in source.iterdir() if path.is_file()}
    return source_names == {path.name for path in archive.iterdir() if path.is_file()} and all(
        sha256(source / name) == sha256(archive / name) for name in source_names
    )


def main() -> int:
    assert_authoritative_release_builder()
    version = source_version()
    commit, tree_state, diff_digest = source_record()
    if tree_state != "clean" or diff_digest:
        raise RuntimeError(
            "authoritative Linux release artifacts require a clean source tree"
        )
    release_arch, tauri_arch, deb_arch = architectures()
    output_dir = package_output_dir()
    output_dir.mkdir(parents=True, exist_ok=True)
    if sys.argv[1:] == ["--finalize"]:
        return finalize(output_dir, version)
    if len(sys.argv) != 1:
        raise RuntimeError("expected no arguments or --finalize")
    target = cargo_target_dir()
    bundle_root = target / "release/bundle"
    raw_deb = one_match(
        bundle_root / "deb",
        f"{TAURI_BUNDLE_BASENAME}_{version}_{tauri_arch}.deb",
        "raw Debian package",
    )

    candidate_dir = staging_dir(output_dir, version)
    if candidate_dir.exists():
        shutil.rmtree(candidate_dir)
    candidate_dir.mkdir()

    timestamp = source_date_epoch()
    deb_output = candidate_dir / debian_artifact_filename(version, deb_arch)
    rebuild_debian(raw_deb, deb_output, version, timestamp)
    sandboxd_output, sandboxd_evidence = build_sandboxd(candidate_dir)
    abi = abi_evidence(deb_output, sandboxd_output)
    builder = builder_record()

    artifacts = []
    for artifact_format, path, package_version in (
        ("deb", deb_output, debian_version(version)),
        ("sandboxd-deb", sandboxd_output, debian_version(version)),
    ):
        artifacts.append(
            {
                "format": artifact_format,
                "filename": path.name,
                "architecture": release_arch,
                "packageVersion": package_version,
                "sha256": sha256(path),
                "size": path.stat().st_size,
            }
        )

    manifest = {
        "schemaVersion": 3,
        "state": "release-candidate",
        "version": version,
        "source": {"commit": commit, "treeState": "clean"},
        "builder": builder,
        "provenance": {
            "command": RELEASE_WORKFLOW_COMMAND,
            "containerImage": builder["image"],
            "kind": "pinned-ubuntu-22.04-container",
        },
        "abi": abi,
        "sandboxd": sandboxd_evidence,
        "artifacts": artifacts,
    }
    (candidate_dir / "release-manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    write_sha256sums(candidate_dir, [deb_output, sandboxd_output])
    print(f"normalized Linux release candidates: {candidate_dir.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
