"""Focused behavior checks for the QuireForge Codex supervisor."""

from __future__ import annotations

import os
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SUPERVISOR = ROOT / "scripts" / "quireforge-codex-supervisor.sh"


class SupervisorCompletionStateTests(unittest.TestCase):
    def test_completion_states_distinguish_alignment_from_worktree_cleanliness(self) -> None:
        with tempfile.TemporaryDirectory() as state_root:
            environment = os.environ | {"QUIRE_FORGE_SUPERVISOR_STATE_DIR": state_root}
            result = subprocess.run(
                ["bash", str(SUPERVISOR), "--self-test"],
                cwd=ROOT,
                check=False,
                capture_output=True,
                env=environment,
                text=True,
            )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "Supervisor completion-state checks passed.\n")


if __name__ == "__main__":
    unittest.main()
