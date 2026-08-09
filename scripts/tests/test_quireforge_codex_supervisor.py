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
        self.assertEqual(
            result.stdout,
            "Supervisor completion-state, worker-access, and tmux-recovery checks passed.\n",
        )

    def test_only_the_worker_uses_the_full_access_sandbox_setting(self) -> None:
        script = SUPERVISOR.read_text(encoding="utf-8")

        self.assertIn('readonly worker_sandbox_mode="danger-full-access"', script)
        self.assertIn(
            'codex exec --ephemeral --sandbox "$worker_sandbox_mode"',
            script,
        )
        self.assertEqual(script.count("--sandbox"), 1)

    def test_worker_safety_protections_remain_explicit(self) -> None:
        script = SUPERVISOR.read_text(encoding="utf-8")

        self.assertIn("post_push_completion_state", script)
        self.assertIn("collect_admissible_untracked_task_paths", script)
        self.assertIn("is_admissible_untracked_task_path", script)
        self.assertIn("PRIVATE KEY", script)
        self.assertIn("outside the approved source-file boundary", script)
        self.assertIn("Worker test or validation failed", script)

    def test_worker_untracked_file_admission_is_bounded_to_safe_source_files(self) -> None:
        script = SUPERVISOR.read_text(encoding="utf-8")

        self.assertIn(".github/*|apps/*|docs/*|packaging/*|scripts/*", script)
        self.assertIn("*.md|*.rs|*.toml|*.ts|*.tsx", script)
        self.assertIn('"$(stat -c %s -- "$absolute_path")" -le 1048576', script)
        self.assertIn("! -L", script)

    def test_tmux_recovery_restarts_only_continuable_states(self) -> None:
        script = SUPERVISOR.read_text(encoding="utf-8")

        self.assertIn("tmux_session_exit_action", script)
        self.assertIn('"$state" == "running"', script)
        self.assertIn('"$task" == "Task committed and pushed"', script)
        self.assertIn("tmux_session_exit_requires_restart", script)
        self.assertIn("tmux worker session ended in a terminal state", script)


if __name__ == "__main__":
    unittest.main()
