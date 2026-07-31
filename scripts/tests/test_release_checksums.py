from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts.release_checksums import parse_sha256sums, validate_sha256sums


FIRST = "a" * 64
SECOND = "b" * 64
EXPECTED = {"quireforge.deb": FIRST, "quireforge-sandboxd.deb": SECOND}


class ReleaseChecksumContractTests(unittest.TestCase):
    def mapping(self, text: str) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "SHA256SUMS"
            path.write_text(text, encoding="utf-8")
            validate_sha256sums(path, EXPECTED)

    def test_accepts_manifest_mapping_in_either_order(self) -> None:
        self.mapping(f"{FIRST}  quireforge.deb\n{SECOND}  quireforge-sandboxd.deb\n")
        self.mapping(f"{SECOND}  quireforge-sandboxd.deb\n{FIRST}  quireforge.deb\n")

    def test_rejects_closed_mapping_violations(self) -> None:
        invalid = [
            f"{FIRST}  quireforge.deb\n{FIRST}  quireforge.deb\n",
            f"{FIRST}  quireforge.deb\n",
            f"{FIRST}  quireforge.deb\n{SECOND}  quireforge-sandboxd.deb\n{FIRST}  extra.deb\n",
            f"{SECOND}  quireforge.deb\n{FIRST}  quireforge-sandboxd.deb\n",
            f"{'A' * 64}  quireforge.deb\n{SECOND}  quireforge-sandboxd.deb\n",
            f"{'a' * 63}  quireforge.deb\n{SECOND}  quireforge-sandboxd.deb\n",
            f"{FIRST}  nested/quireforge.deb\n{SECOND}  quireforge-sandboxd.deb\n",
        ]
        for text in invalid:
            with self.subTest(text=text):
                with tempfile.TemporaryDirectory() as temporary:
                    path = Path(temporary) / "SHA256SUMS"
                    path.write_text(text, encoding="utf-8")
                    with self.assertRaises(RuntimeError):
                        validate_sha256sums(path, EXPECTED)

    def test_parser_rejects_unsupported_line_format(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "SHA256SUMS"
            path.write_text(f"{FIRST} quireforge.deb\n", encoding="utf-8")
            with self.assertRaises(RuntimeError):
                parse_sha256sums(path)


if __name__ == "__main__":
    unittest.main()
