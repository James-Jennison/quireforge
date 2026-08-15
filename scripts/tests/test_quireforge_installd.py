from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("quireforge_installd", ROOT / "scripts" / "quireforge_installd.py")
installd = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(installd)


class StagedPackageValidationTests(unittest.TestCase):
    def test_accepts_only_a_direct_root_owned_staged_deb(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            staging = Path(temporary) / "packages"
            staging.mkdir()
            package = staging / "quireforge.deb"
            package.write_bytes(b"fixture")
            outside = Path(temporary) / "outside.deb"
            outside.write_bytes(b"fixture")
            alias = staging / "alias.deb"
            alias.symlink_to(outside)
            with patch.object(installd, "STAGING_ROOT", staging), patch.object(installd, "safe_root_owned", return_value=True), patch.object(installd, "safe_staging_root", return_value=True):
                self.assertEqual(installd.validated_package(str(package)), package)
                with self.assertRaises(ValueError):
                    installd.validated_package(str(outside))
                with self.assertRaises(ValueError):
                    installd.validated_package(str(alias))
