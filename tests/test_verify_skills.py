#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""test_verify_skills.py — 门禁突变测试（DJ-004 α1）

故意注入损坏夹具，断言每条门禁真的咬合——
"评测管道本身有洞，别把管道的洞当壳不够聪明"（琅嬛 constraints-as-architecture）。

stdlib-only（unittest + tempfile），运行：
  python tests/test_verify_skills.py
"""
import os
import shutil
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "scripts"))
import verify_skills as vs  # noqa: E402

OK_SKILL = """---
name: dj-ok
description: "测试用正常技能。Use when running mutation tests. 触发词：突变测试。"
---

# Ok Skill

正常正文，长度超过五十字符，用于突变测试的正向对照样本。
正常正文，长度超过五十字符，用于突变测试的正向对照样本。
"""

BROKEN_FM = """---
name: dj-broken
description: "frontmatter 不闭合的损坏样本。"
# 缺少闭合的 ---

# Broken

frontmatter 损坏，应报 FAIL。frontmatter 损坏，应报 FAIL。frontmatter 损坏，应报 FAIL。
"""

ABS_PATH_SKILL = """---
name: dj-abs
description: "含绝对路径的样本。Use when testing absolute path detection. 触发词：绝对路径。"
---

# Abs Skill

示例路径：C:\\\\Work\\\\private 与 /Users/cimer/notes 都是绝对路径，应报 WARN。
"""

AP_BROKEN = """# 反模式表（编号断裂样本）

| # | 反模式 | ❌ Wrong | ✅ Right |
|---|--------|----------|----------|
| 1 | **甲** | 错 | 对 |
| 3 | **乙** | 错 | 对 |
"""

AP_EMPTY = """# 反模式表（空表样本）

| # | 反模式 | ❌ Wrong | ✅ Right |
|---|--------|----------|----------|
"""

DOC_ROT = """# 样本文档

这个模块不再使用旧方案，以前是另一套。这个模块不再使用旧方案，以前是另一套。这个模块不再使用旧方案，以前是另一套。这个模块不再使用旧方案，以前是另一套。"""


def build_fixture(tmp, kind):
    """构造夹具仓库结构，返回 (root, rep)。"""
    skills = os.path.join(tmp, "skills", "engineering")
    os.makedirs(os.path.join(skills, "dj-ok"), exist_ok=True)
    with open(os.path.join(skills, "dj-ok", "SKILL.md"), "w", encoding="utf-8") as f:
        f.write(OK_SKILL)

    if kind == "broken-fm":
        os.makedirs(os.path.join(skills, "dj-broken"), exist_ok=True)
        with open(os.path.join(skills, "dj-broken", "SKILL.md"), "w", encoding="utf-8") as f:
            f.write(BROKEN_FM)
    if kind == "abs-path":
        os.makedirs(os.path.join(skills, "dj-abs"), exist_ok=True)
        with open(os.path.join(skills, "dj-abs", "SKILL.md"), "w", encoding="utf-8") as f:
            f.write(ABS_PATH_SKILL)

    refs = os.path.join(tmp, "docs", "references")
    os.makedirs(refs, exist_ok=True)
    ap_path = os.path.join(refs, "anti-patterns.md")
    if kind == "ap-broken":
        with open(ap_path, "w", encoding="utf-8") as f:
            f.write(AP_BROKEN)
    elif kind == "ap-empty":
        with open(ap_path, "w", encoding="utf-8") as f:
            f.write(AP_EMPTY)
    else:
        with open(ap_path, "w", encoding="utf-8") as f:
            f.write(AP_BROKEN.replace("| 3 | **乙**", "| 2 | **乙**"))

    # canary 双入口文档（除 canary-missing 夹具外都提供）
    if kind != "canary-missing":
        for fn in vs.CANARY["worktree"]:
            with open(os.path.join(tmp, fn), "w", encoding="utf-8") as f:
                f.write("worktree 约定。Conventional Commits。使用中文编写提交信息。\n")
    else:
        with open(os.path.join(tmp, "AGENTS.md"), "w", encoding="utf-8") as f:
            f.write("只有 AGENTS，README 缺失。\n")

    if kind == "doc-rot":
        with open(os.path.join(tmp, "docs", "sample.md"), "w", encoding="utf-8") as f:
            f.write(DOC_ROT)

    if kind == "ttl-expired":
        with open(os.path.join(refs, "stale.md"), "w", encoding="utf-8") as f:
            f.write("---\nreview_before: 2020-01-01\n---\n\n# 过期样本\n\n正文。\n")

    rep = vs.Report()
    return rep


class MutationTests(unittest.TestCase):
    """门禁突变测试：每条损坏夹具都必须咬出 FAIL/WARN，否则门禁失明。"""

    def setUp(self):
        self._orig_root = vs.ROOT
        self.tmp = tempfile.mkdtemp(prefix="dj-mutation-")
        self.addCleanup(shutil.rmtree, self.tmp, True)
        vs.ROOT = self.tmp

    def tearDown(self):
        vs.ROOT = self._orig_root

    def run_checks(self, kind):
        rep = build_fixture(self.tmp, kind)
        skills = vs.discover()
        for bucket, name, path in skills:
            props, body = vs.check_frontmatter(rep, bucket, name, path)
            vs.check_description(rep, bucket, name, props)
            vs.check_refs(rep, bucket, name, path)
            vs.check_stom(rep, bucket, name, body)
        vs.check_antipatterns(rep)
        vs.check_canary(rep)
        vs.check_docs_freshness(rep)
        vs.check_ttl(rep)
        return rep

    def test_positive_baseline(self):
        """正向基线：正常夹具零 FAIL。"""
        rep = self.run_checks("clean")
        self.assertEqual(rep.fails, [])

    def test_broken_frontmatter_bites(self):
        rep = self.run_checks("broken-fm")
        self.assertTrue(any("frontmatter" in f for f in rep.fails), rep.fails)

    def test_antipatterns_broken_numbering_bites(self):
        rep = self.run_checks("ap-broken")
        self.assertTrue(any("编号不连续" in f for f in rep.fails), rep.fails)

    def test_antipatterns_empty_table_now_fails(self):
        """DJ-004 语义修复验证：解析 0 条目必须 FAIL（旧版只 warn，strict 放行）。"""
        rep = self.run_checks("ap-empty")
        self.assertTrue(any("解析到 0 条目" in f for f in rep.fails), rep.fails)

    def test_canary_missing_file_now_fails(self):
        """DJ-004 语义修复验证：canary 目标文件缺失必须 FAIL（旧版只 warn）。"""
        rep = self.run_checks("canary-missing")
        self.assertTrue(any("不存在" in f and "canary" in f for f in rep.fails), rep.fails)

    def test_absolute_path_warns(self):
        """STOM：绝对路径报 WARN。"""
        rep = self.run_checks("abs-path")
        self.assertTrue(any("绝对路径" in w for w in rep.warns), rep.warns)

    def test_doc_rot_words_warn(self):
        """docs 防腐：历史变迁词报 WARN。"""
        rep = self.run_checks("doc-rot")
        self.assertTrue(any("历史变迁词" in w for w in rep.warns), rep.warns)

    def test_ttl_expired_warns(self):
        """TTL：过期 review_before 报 WARN。"""
        rep = self.run_checks("ttl-expired")
        self.assertTrue(any("已过期" in w for w in rep.warns), rep.warns)


if __name__ == "__main__":
    unittest.main(verbosity=2)


class TestTailPointerGuards:
    """DJ-016 验收：双守卫突变测试——注入损坏必须 FAIL（咬合证明）。"""

    def test_tail_pointer_removed_bites(self, tmp_path, monkeypatch):
        """突变：dj-implement 的指针目标被抹 → tail-ptr FAIL。"""
        rep = vs.Report()
        skills = vs.discover()
        path = [p for _, n, p in skills if n == "dj-implement"][0]
        orig = open(path, encoding="utf-8").read()
        assert "dj-check" in orig, "前提：dj-implement 当前指向 dj-check"
        import io as _io
        monkeypatch.setattr("builtins.open", lambda *a, **k: _io.StringIO(orig.replace("dj-check", "dj-ZZZ")))
        vs.check_tail_pointers(rep, skills)
        assert rep.fails, "指针被抹后必须 FAIL——守卫失明"
        assert any("dj-implement" in f for f in rep.fails)

    def test_dead_pointer_target_bites(self):
        """突变：清单指向不存在技能 → tail-ptr FAIL（防死链）。"""
        rep = vs.Report()
        skills = vs.discover()
        saved = dict(vs.TAIL_POINTERS)
        try:
            vs.TAIL_POINTERS["dj-implement"] = [("dj-ghost-skill", "model")]
            vs.check_tail_pointers(rep, skills)
        finally:
            vs.TAIL_POINTERS.clear()
            vs.TAIL_POINTERS.update(saved)
        assert rep.fails, "死链指针必须被抓住"
        assert any("dj-ghost-skill" in f for f in rep.fails)

    def test_index_orphan_bites(self):
        """突变：孤儿技能名 → index-cov FAIL。"""
        rep = vs.Report()
        fake_skills = [("engineering", "dj-nonexistent-orphan", "/nowhere/SKILL.md")]
        vs.check_index_coverage(rep, fake_skills)
        assert rep.fails, "孤儿技能必须被 index-cov 抓住"
        assert any("dj-nonexistent-orphan" in f for f in rep.fails)

    def test_guards_green_on_current_repo(self):
        """绿灯基线：当前仓库两守卫必须零 FAIL（回归保护）。"""
        rep = vs.Report()
        skills = vs.discover()
        vs.check_tail_pointers(rep, skills)
        vs.check_index_coverage(rep, skills)
        assert not rep.fails, f"基线必须全绿：{rep.fails}"
