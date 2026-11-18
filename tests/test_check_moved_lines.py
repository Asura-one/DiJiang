#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""check_moved_lines.py 自测：构造小仓库，验证「搬走算保留、丢失报 exit 1、坏参数 exit 2」。

运行：python3 tests/test_check_moved_lines.py
"""
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "skills/engineering/dj-meta/scripts/check_moved_lines.py"


def run(*args, cwd):
    result = subprocess.run(
        [sys.executable, str(SCRIPT), *args],
        cwd=str(cwd),
        capture_output=True,
        text=True,
        check=False,
    )
    return result.returncode, result.stdout + result.stderr


class MovedLinesTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.git("init", "-q")
        self.git("config", "user.email", "t@example.com")
        self.git("config", "user.name", "t")
        self.write("skills/demo/SKILL.md", "# Demo\n\n不要删除生产数据。\n每次改码先跑基线回归。\n")
        self.commit("base")
        self.base = self.git("rev-parse", "HEAD").strip()

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

    def test_move_into_reference_passes(self):
        self.write(
            "skills/demo/SKILL.md",
            "# Demo\n\n不要删除生产数据。\n\n详见 references/regression.md。\n",
        )
        self.write("skills/demo/references/regression.md", "每次改码先跑基线回归。\n")
        code, out = run(self.base, ".", cwd=self.root)
        self.assertEqual(code, 0, out)
        self.assertIn("moved_lines_status: PASS", out)

    def test_deleted_line_warns(self):
        self.write("skills/demo/SKILL.md", "# Demo\n\n不要删除生产数据。\n")
        code, out = run(self.base, ".", cwd=self.root)
        self.assertEqual(code, 1, out)
        self.assertIn("moved_lines_status: WARN", out)
        self.assertIn("每次改码先跑基线回归", out)

    def test_head_mode_audits_committed_prune(self):
        self.write("skills/demo/SKILL.md", "# Demo\n\n不要删除生产数据。\n")
        self.commit("prune")
        head = self.git("rev-parse", "HEAD").strip()
        self.write("skills/demo/SKILL.md", "# Demo\n\n不要删除生产数据。\n每次改码先跑基线回归。\n")
        self.commit("restore")
        code, out = run(self.base, ".", "--head", head, cwd=self.root)
        self.assertEqual(code, 1, out)
        self.assertIn("moved_lines_base_lines", out)

    def test_bad_revision_exits_two(self):
        code, out = run("no-such-rev", ".", cwd=self.root)
        self.assertEqual(code, 2, out)
        self.assertIn("moved_lines_status: ERROR", out)


if __name__ == "__main__":
    unittest.main(verbosity=2)