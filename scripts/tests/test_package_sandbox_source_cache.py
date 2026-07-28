from __future__ import annotations

import hashlib
import subprocess
import tempfile
import unittest
from pathlib import Path

from scripts.release_contract import ROOT


HELPER = ROOT / "packaging/sandbox/cache_verified_source.sh"


class SandboxSourceCacheTests(unittest.TestCase):
    def run_helper(
        self,
        cache: Path,
        name: str,
        url: str,
        checksum: str,
        destination: Path,
        *,
        expect_success: bool = True,
    ) -> subprocess.CompletedProcess[str]:
        result = subprocess.run(
            ["bash", str(HELPER), str(cache), name, url, checksum, str(destination)],
            check=False,
            capture_output=True,
            text=True,
        )
        if expect_success:
            self.assertEqual(result.returncode, 0, result.stderr)
        return result

    def test_reuses_only_a_checksum_verified_immutable_source(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "source.tar.xz"
            source.write_bytes(b"pinned source bytes")
            checksum = hashlib.sha256(source.read_bytes()).hexdigest()
            cache = root / "cache"
            first = root / "first"
            self.run_helper(cache, "source.tar.xz", source.as_uri(), checksum, first)
            self.assertEqual(first.read_bytes(), b"pinned source bytes")

            source.unlink()
            second = root / "second"
            self.run_helper(cache, "source.tar.xz", source.as_uri(), checksum, second)
            self.assertEqual(second.read_bytes(), b"pinned source bytes")

    def test_replaces_a_tampered_cache_entry_only_after_revalidation(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "source.tgz"
            source.write_bytes(b"trusted archive")
            checksum = hashlib.sha256(source.read_bytes()).hexdigest()
            cache = root / "cache"
            cache.mkdir()
            (cache / "source.tgz").write_bytes(b"tampered")
            destination = root / "destination"
            self.run_helper(cache, "source.tgz", source.as_uri(), checksum, destination)
            self.assertEqual(destination.read_bytes(), b"trusted archive")
            self.assertEqual((cache / "source.tgz").read_bytes(), b"trusted archive")

    def test_rejects_an_unsafe_cache_name(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "source"
            source.write_bytes(b"source")
            checksum = hashlib.sha256(source.read_bytes()).hexdigest()
            result = self.run_helper(
                root / "cache",
                "../unsafe",
                source.as_uri(),
                checksum,
                root / "destination",
                expect_success=False,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("unsafe", result.stderr)

    def test_rejects_a_symlinked_cache_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "source"
            source.write_bytes(b"source")
            checksum = hashlib.sha256(source.read_bytes()).hexdigest()
            target = root / "cache-target"
            target.mkdir()
            cache = root / "cache"
            cache.symlink_to(target, target_is_directory=True)
            result = self.run_helper(
                cache,
                "source",
                source.as_uri(),
                checksum,
                root / "destination",
                expect_success=False,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("must not be a symlink", result.stderr)
