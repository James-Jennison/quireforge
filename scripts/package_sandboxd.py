#!/usr/bin/env python3
"""Build the separately installed M39 sandbox worker Debian component.

It may run only inside the pinned Ubuntu 22.04 release container. AppImage
packaging never invokes this script and never receives worker assets.
"""
from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

from release_contract import (
    ROOT,
    assert_authoritative_release_builder,
    debian_version,
    sandboxd_artifact_filename,
    source_record,
    source_version,
)

def run(command: list[str], **kwargs: object) -> None:
    subprocess.run(command, check=True, **kwargs)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build(output: Path) -> tuple[Path, dict[str, object]]:
    """Build the worker into an authoritative release-candidate directory."""
    assert_authoritative_release_builder()
    version = source_version()
    if version != "0.1.0-beta.49":
        raise RuntimeError("M54 requires beta.49")
    if shutil.which("docker") is None and os.environ.get("QUIRE_FORGE_RELEASE_BUILDER") != "pinned-ubuntu-22.04":
        raise RuntimeError("sandbox worker builds only inside the authoritative container")
    run(["cargo", "build", "--release", "-p", "quireforge-sandboxd"])
    with tempfile.TemporaryDirectory(prefix="quireforge-sandboxd-") as temporary:
        temporary_root = Path(temporary)
        assets = temporary_root / "assets"
        work = temporary_root / "work"
        run(["bash", "packaging/sandbox/build_guest_assets.sh", str(assets), str(work)], cwd=ROOT)
        root = temporary_root / "root"
        (root / "DEBIAN").mkdir(parents=True)
        (root / "usr/sbin").mkdir(parents=True)
        (root / "usr/lib/quireforge-sandboxd").mkdir(parents=True)
        (root / "lib/systemd/system").mkdir(parents=True)
        target = Path(os.environ.get("CARGO_TARGET_DIR", str(ROOT / "target")))
        shutil.copy2(target / "release/quireforge-sandboxd", root / "usr/sbin/quireforge-sandboxd")
        for asset in ("firecracker", "jailer", "vmlinux", "initramfs.cpio.gz", "SHA256SUMS"):
            shutil.copy2(assets / asset, root / "usr/lib/quireforge-sandboxd" / asset)
        shutil.copy2(ROOT / "packaging/sandbox/quireforge-sandboxd.service", root / "lib/systemd/system/quireforge-sandboxd.service")
        (root / "DEBIAN/control").write_text(
            "Package: quireforge-sandboxd\n"
            f"Version: {debian_version(version)}\n"
            "Section: utils\nPriority: optional\nArchitecture: amd64\n"
            "Maintainer: QuireForge contributors\n"
            "Depends: systemd (>= 249)\n"
            "Description: QuireForge isolated static ELF analysis worker\n"
            " Separately installed KVM-only worker. It has no network or project mounts.\n",
            encoding="utf-8",
        )
        (root / "DEBIAN/postinst").write_text(
            "#!/bin/sh\nset -eu\ngetent group quireforge-sandbox >/dev/null || groupadd --system quireforge-sandbox\n",
            encoding="utf-8",
        )
        (root / "DEBIAN/postinst").chmod(0o755)
        output.mkdir(parents=True, exist_ok=True)
        artifact = output / sandboxd_artifact_filename(version)
        run(["dpkg-deb", "--root-owner-group", "--build", str(root), str(artifact)])
        commit, tree_state, diff_digest = source_record()
        if tree_state != "clean" or diff_digest:
            raise RuntimeError("sandbox worker release artifacts require a clean source tree")
        evidence: dict[str, object] = {
            "schemaVersion": 1,
            "version": version,
            "source": {"commit": commit, "treeState": tree_state},
            "worker": {
                "firecracker": "1.15.1",
                "jailer": "1.15.1",
                "guestKernel": "6.1.178",
                "network": "disabled",
                "guestMounts": "none",
                "runtime": "static-elf64-x86_64-only",
            },
            "artifact": {"filename": artifact.name, "sha256": sha256(artifact), "size": artifact.stat().st_size},
            "assets": {path.name: sha256(path) for path in sorted(assets.iterdir()) if path.is_file()},
        }
        return artifact, evidence


def main() -> None:
    output = ROOT / "target/ubuntu-22.04/release/sandboxd"
    artifact, evidence = build(output)
    (output / "sandboxd-manifest.json").write_text(
        json.dumps(evidence, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(json.dumps({"artifact": artifact.name, "evidence": evidence}, sort_keys=True))


if __name__ == "__main__":
    main()
