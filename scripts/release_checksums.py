"""Closed checksum contract for QuireForge desktop release artifacts."""

from __future__ import annotations

import hashlib
import re
from pathlib import Path
from typing import Mapping, Sequence


SHA256 = re.compile(r"^[0-9a-f]{64}$")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def parse_sha256sums(path: Path) -> dict[str, str]:
    entries: dict[str, str] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line:
            continue
        parts = line.split("  ")
        if len(parts) != 2:
            raise RuntimeError("release checksum entry is malformed")
        digest, name = parts
        if (
            not SHA256.fullmatch(digest)
            or not name
            or Path(name).name != name
            or "/" in name
            or "\\" in name
            or name in {".", ".."}
            or name in entries
        ):
            raise RuntimeError("release checksum entry is incoherent")
        entries[name] = digest
    return entries


def validate_sha256sums(path: Path, expected: Mapping[str, str]) -> None:
    if parse_sha256sums(path) != dict(expected):
        raise RuntimeError("SHA256SUMS does not match the release artifacts")


def write_sha256sums(output_dir: Path, artifacts: Sequence[Path]) -> None:
    (output_dir / "SHA256SUMS").write_text(
        "\n".join(f"{sha256(path)}  {path.name}" for path in artifacts) + "\n",
        encoding="utf-8",
    )
