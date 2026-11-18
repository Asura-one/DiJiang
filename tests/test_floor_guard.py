#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""floor_guard.py 自测：构造小仓库逐类注入「最便宜变绿」，断言每类都被咬住。

运行：python3 tests/test_floor_guard.py
"""
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "skills/engineering/dj-regression-guard/scripts/floor_guard.py"


class FloorGuardTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.git("init", "-q")
        self.git("config", "user.email", "t@example.com")
        self.git("config", "user.name", "t")
        self.write("src/app.py", "def add(a, b):\n    return a + b\n")
        self.write("tests/test_app.py", "from src.app import add\n\ndef test_add():\n    assert add(1, 2) == 3\n")
        self.write("config/limits.yaml", "coverage_min: 80\nlatency_budget_ms: 200\n")
        self.commit("base")

    def tearDown(self):
        self.tmp.cleanup()

    def git(self, *args):
        return subprocess.run(
            ["git", *args], cwd=str(self.root), capture_output=True, text=True, check=True
        ).stdout

    def write(self, rel, text):
        path = self.root / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")

    def commit(self, msg):
        self.git("add", "-A")
        self.git("commit", "-q", "-m", msg)

    def run_guard(self, *args):
        result = subprocess.run(
            [sys.executable, str(SCRIPT), "--base", "HEAD", *args],
            cwd=str(self.root),
            capture_output=True,
            text=True,
            check=False,
        )
        return result.returncode, result.stdout + result.stderr

    def test_clean_diff_passes(self):
        self.write("src/app.py", "def add(a, b):\n    return a + b\n\ndef sub(a, b):\n    return a - b\n")
        code, out = self.run_guard()
        self.assertEqual(code, 0, out)
        self.assertIn("floor_status: PASS", out)

    def test_type_ignore_is_caught(self):
        self.write("src/app.py", "def add(a, b):\n    # type: ignore\n    return a + b\n")
        code, out = self.run_guard()
        self.assertEqual(code, 1, out)
        self.assertIn("gate-silenced:type-ignore", out)

    def test_skipped_test_is_caught(self):
        self.write("tests/test_app.py", "from src.app import add\n\n@unittest.skip\ndef test_add():\n    assert add(1, 2) == 3\n")
        code, out = self.run_guard()
        self.assertEqual(code, 1, out)
        self.assertIn("test-easier", out)

    def test_deleted_test_file_is_caught(self):
        (self.root / "tests/test_app.py").unlink()
        code, out = self.run_guard()
        self.assertEqual(code, 1, out)
        self.assertIn("test-file-deleted", out)

    def test_lowered_threshold_is_caught(self):
        self.write("config/limits.yaml", "coverage_min: 60\nlatency_budget_ms: 200\n")
        code, out = self.run_guard()
        self.assertEqual(code, 1, out)
        self.assertIn("threshold-lowered", out)

    def test_empty_catch_is_caught(self):
        self.write("src/app.js", "try { run(); } catch (e) {}\n")
        code, out = self.run_guard()
        self.assertEqual(code, 1, out)
        self.assertIn("empty-catch", out)

    def test_contract_row_without_command_is_caught(self):
        self.write(
            "CONSTRAINTS.md",
            "| 维度 | 规则 | 裁决命令 | 运行位置 |\n|---|---|---|---|\n| 覆盖率 | 改动行 ≥ 80% | vitest --coverage | CI |\n| 性能 | LCP ≤ 2500ms |  | 愿景 |\n",
        )
        code, out = self.run_guard()
        self.assertEqual(code, 1, out)
        self.assertIn("contract-row-without-command", out)

    def test_bad_revision_exits_two(self):
        result = subprocess.run(
            [sys.executable, str(SCRIPT), "--base", "no-such-rev"],
            cwd=str(self.root),
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
        self.assertIn("floor_status: ERROR", result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)