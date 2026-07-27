"""Shared, dependency-free QuireForge release-contract helpers."""

from __future__ import annotations

import hashlib
import json
import os
import platform
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
TAURI_CONFIG = ROOT / "apps/desktop/src-tauri/tauri.conf.json"
TAURI_CARGO = ROOT / "apps/desktop/src-tauri/Cargo.toml"
SANDBOXD_CARGO = ROOT / "apps/sandboxd/Cargo.toml"
SCHEMA_PATH = ROOT / "packaging/release-manifest.schema.json"
CANONICAL_DESKTOP = "io.github.codeframe78.QuireForge.desktop"
LEGACY_DESKTOP = "QuireForge.desktop"
TAURI_BUNDLE_BASENAME = "QuireForge"
DEBIAN_PACKAGE = "quireforge"
SANDBOXD_DEBIAN_PACKAGE = "quireforge-sandboxd"
EXPECTED_IMAGE = (
    "ubuntu:22.04@sha256:"
    "0e0a0fc6d18feda9db1590da249ac93e8d5abfea8f4c3c0c849ce512b5ef8982"
)
RELEASE_BUILDER_ENV = "QUIRE_FORGE_RELEASE_BUILDER"
RELEASE_BUILDER_VALUE = "pinned-ubuntu-22.04"
RELEASE_WORKFLOW_COMMAND = "scripts/run_linux_package_container.sh"
RELEASE_OUTPUT_DIR = ROOT / "target/ubuntu-22.04/release/packages"
HOST_DEVELOPMENT_TARGET_DIR = ROOT / "target/host-development"
GLIBC_BASELINE = (2, 35)
SEMVER_RE = re.compile(
    r"^(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\."
    r"(?P<patch>0|[1-9]\d*)(?:-(?P<prerelease>[0-9A-Za-z.-]+))?$"
)


def run(
    arguments: list[str],
    *,
    cwd: Path | None = None,
    env: dict[str, str] | None = None,
    capture: bool = False,
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        arguments,
        cwd=cwd,
        env=env,
        check=True,
        text=True,
        stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.PIPE if capture else None,
    )


def appstream_validation_command(validator: str, metadata: Path) -> list[str]:
    return [validator, "validate", "--no-net", str(metadata)]


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def glibc_requirement(path: Path) -> tuple[int, int]:
    result = run(["readelf", "--version-info", str(path)], capture=True)
    versions = {
        (int(major), int(minor))
        for major, minor in re.findall(r"GLIBC_(\d+)\.(\d+)", result.stdout)
    }
    if not versions:
        raise RuntimeError(f"no GLIBC version contract found in {path}")
    newest = max(versions)
    if newest > GLIBC_BASELINE:
        rendered = ".".join(str(component) for component in newest)
        raise RuntimeError(
            f"{path} requires GLIBC {rendered}, newer than Ubuntu 22.04"
        )
    return newest


def glibc_version_text(version: tuple[int, int]) -> str:
    return f"GLIBC_{version[0]}.{version[1]}"


def source_version() -> str:
    versions = {
        "root package": json.loads(
            (ROOT / "package.json").read_text(encoding="utf-8")
        )["version"],
        "desktop package": json.loads(
            (ROOT / "apps/desktop/package.json").read_text(encoding="utf-8")
        )["version"],
        "website package": json.loads(
            (ROOT / "apps/website/package.json").read_text(encoding="utf-8")
        )["version"],
    }
    cargo_text = TAURI_CARGO.read_text(encoding="utf-8")
    cargo_match = re.search(
        r"(?ms)^\[package\]\s.*?^version\s*=\s*\"([^\"]+)\"",
        cargo_text,
    )
    if not cargo_match:
        raise RuntimeError("Cargo package version is missing")
    versions["Cargo package"] = cargo_match.group(1)
    sandboxd_text = SANDBOXD_CARGO.read_text(encoding="utf-8")
    sandboxd_match = re.search(
        r"(?ms)^\[package\]\s.*?^version\s*=\s*\"([^\"]+)\"",
        sandboxd_text,
    )
    if not sandboxd_match:
        raise RuntimeError("sandbox worker Cargo package version is missing")
    versions["sandbox worker Cargo package"] = sandboxd_match.group(1)

    distinct = set(versions.values())
    if len(distinct) != 1:
        details = ", ".join(f"{name}={value}" for name, value in versions.items())
        raise RuntimeError(f"release versions disagree: {details}")
    version = distinct.pop()
    if not SEMVER_RE.fullmatch(version):
        raise RuntimeError(f"unsupported release version: {version}")
    return version


def debian_version(version: str) -> str:
    match = SEMVER_RE.fullmatch(version)
    if not match:
        raise RuntimeError(f"unsupported release version: {version}")
    base = f"{match.group('major')}.{match.group('minor')}.{match.group('patch')}"
    prerelease = match.group("prerelease")
    return f"{base}~{prerelease}" if prerelease else base


def debian_artifact_filename(version: str, architecture: str = "amd64") -> str:
    """Return the GitHub-safe outer filename for a Debian package.

    Debian prereleases retain ``~`` in their control metadata so they sort
    before the corresponding stable version. GitHub Releases normalizes ``~``
    in uploaded asset names, so the outer filename deliberately uses ``.``.
    """
    artifact_version = debian_version(version).replace("~", ".")
    return f"{DEBIAN_PACKAGE}_{artifact_version}_{architecture}.deb"


def sandboxd_artifact_filename(version: str, architecture: str = "amd64") -> str:
    """Return the review-artifact name for the separately installed worker."""
    artifact_version = debian_version(version).replace("~", ".")
    return f"{SANDBOXD_DEBIAN_PACKAGE}_{artifact_version}_{architecture}.deb"


def architectures() -> tuple[str, str, str]:
    machine = platform.machine()
    if machine not in {"x86_64", "amd64"}:
        raise RuntimeError(
            f"Milestone 20 packages support only x86_64, not {machine or 'unknown'}"
        )
    return ("x86_64", "amd64", "amd64")


def cargo_target_dir() -> Path:
    configured = os.environ.get("CARGO_TARGET_DIR")
    if not configured:
        return ROOT / "target"
    path = Path(configured)
    return path.resolve() if path.is_absolute() else (ROOT / path).resolve()


def release_builder_active() -> bool:
    return os.environ.get(RELEASE_BUILDER_ENV) == RELEASE_BUILDER_VALUE


def package_output_dir() -> Path:
    if release_builder_active():
        return RELEASE_OUTPUT_DIR
    return HOST_DEVELOPMENT_TARGET_DIR / "packages"


def assert_authoritative_release_builder() -> None:
    """Reject host normalization into the authoritative Ubuntu release path."""
    if not release_builder_active():
        raise RuntimeError(
            "authoritative Linux release artifacts require "
            "scripts/run_linux_package_container.sh"
        )
    os_release = Path("/etc/os-release")
    fields = {}
    for line in os_release.read_text(encoding="utf-8").splitlines():
        key, separator, value = line.partition("=")
        if separator:
            fields[key] = value.strip().strip('"')
    if fields.get("ID") != "ubuntu" or fields.get("VERSION_ID") != "22.04":
        raise RuntimeError(
            "authoritative Linux release artifacts require Ubuntu 22.04"
        )


def source_date_epoch() -> int:
    configured = os.environ.get("SOURCE_DATE_EPOCH")
    if configured:
        if not configured.isdigit():
            raise RuntimeError("SOURCE_DATE_EPOCH must be a positive integer")
        return int(configured)
    result = run(
        ["git", "log", "-1", "--format=%ct"],
        cwd=ROOT,
        capture=True,
    )
    value = result.stdout.strip()
    if not value.isdigit():
        raise RuntimeError("could not derive SOURCE_DATE_EPOCH from Git")
    return int(value)


def source_record() -> tuple[str, str, str | None]:
    commit = run(
        ["git", "rev-parse", "HEAD"],
        cwd=ROOT,
        capture=True,
    ).stdout.strip()
    status = run(
        ["git", "status", "--short", "--untracked-files=all"],
        cwd=ROOT,
        capture=True,
    ).stdout
    if not status:
        return commit, "clean", None
    diff = run(
        ["git", "diff", "--binary", "HEAD", "--", "."],
        cwd=ROOT,
        capture=True,
    ).stdout.encode("utf-8")
    untracked = "\n".join(
        line[3:]
        for line in status.splitlines()
        if line.startswith("?? ")
    ).encode("utf-8")
    digest = hashlib.sha256(diff + b"\0" + untracked).hexdigest()
    return commit, "dirty", digest


def builder_record() -> dict[str, str]:
    assert_authoritative_release_builder()
    distribution = os.environ.get("QUIRE_FORGE_BUILD_DISTRIBUTION")
    version = os.environ.get("QUIRE_FORGE_BUILD_VERSION")
    image = os.environ.get("QUIRE_FORGE_BUILD_IMAGE")
    if distribution != "ubuntu" or version != "22.04" or image != EXPECTED_IMAGE:
        raise RuntimeError("authoritative builder identity is not pinned")
    return {
        "distribution": distribution,
        "version": version,
        "architecture": "x86_64",
        "image": image,
    }


def replace_control_field(text: str, field: str, value: str) -> str:
    pattern = re.compile(rf"(?m)^{re.escape(field)}:\s*.*$")
    updated, count = pattern.subn(f"{field}: {value}", text, count=1)
    if count != 1:
        raise RuntimeError(f"Debian control field is missing or duplicated: {field}")
    return updated


def set_tree_timestamp(root: Path, timestamp: int) -> None:
    for path in sorted(root.rglob("*"), key=lambda item: len(item.parts), reverse=True):
        try:
            os.utime(path, (timestamp, timestamp), follow_symlinks=False)
        except (NotImplementedError, PermissionError):
            if not path.is_symlink():
                raise
    os.utime(root, (timestamp, timestamp))


def write_sha256sums(output_dir: Path, artifacts: list[Path]) -> None:
    lines = [f"{sha256(path)}  {path.name}" for path in sorted(artifacts)]
    (output_dir / "SHA256SUMS").write_text(
        "\n".join(lines) + "\n",
        encoding="utf-8",
    )
