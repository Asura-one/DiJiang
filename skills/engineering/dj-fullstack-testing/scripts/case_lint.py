#!/usr/bin/env python3
"""测试用例红线检查器（docs/test-cases/*.md）——不调模型、零第三方依赖。

Adapted from fishzjp/qa-skills core/scripts/case_lint.py (MIT)，适配 DiJiang 用例契约
（dj-fullstack-testing/references/test-artifacts.md §1）。规则权威定义在该契约文件，
本脚本只把可机械判定的部分变成闸门，供写用例方自查、评审方复核。

用法：
  python3 case_lint.py docs/test-cases/test-case-spec.md [--json]

检查项（E = 错误，阻断定稿；W = 告警，需人工确认）：
  E1 TC 编号不符合 TC-{模块号}-{序号}（如 TC-02-03）
  E2 TC 编号重复
  E3 用例缺「操作步骤」或「预期结果」（展开式查字段；紧凑式要求 id 后有实质内容）
  E4 占位符残留：{xxx} / <xxx> / 某某、某数据、某账号
  E5 预期结果不可判定：「不报错/正常/正确处理/符合预期/视情况」类占位（SKILL 红线）
  E6 TC 模块号与所在 `## N.` 章节号不符
  W1 异步预期缺判定时限（有 自动/异步/生效/轮询 而无时间词）
  W2 未标优先级 [P0/P1/P2]
  W3 P0 占比 ≥35%（用例 ≥6 条时，冒烟集失去筛选意义）
  W4 预期结果非唯一（含「或」，应拆条）

附录区（`## 附录…` / `## 审查记录…` 之后）豁免 E4/E5——代码证据与缺陷记录放那里。
exit 0 = 无错误（告警不阻断）；1 = 有错误；2 = 用法/读取错误。
"""

import argparse
import json
import re
import sys
from pathlib import Path

HEADING_RE = re.compile(r"^(#{1,6})\s+(.*)$")
MODULE_HEADING_RE = re.compile(r"^##\s+(\d+)[.、]")
CASE_LINE_RE = re.compile(r"^(\s*)[-*]\s+(?:\*\*)?\s*(TC-[A-Za-z0-9-]+)\s*(.*)$")
TC_CANON_RE = re.compile(r"^TC-(\d+)-(\d+)$")
FIELD_RE = re.compile(r"^(?:\s*[-*]\s+|\s+)(前置条件|操作步骤|预期结果|验证方式)\s*[：:]\s*(.*)$")
BULLET_RE = re.compile(r"^(\s*)[-*]\s+(.*)$")
PRIORITY_RE = re.compile(r"\[(P0|P1|P2)\]")
PLACEHOLDER_RE = re.compile(r"\{[^{}\n]{1,40}\}|<[^<>\n]{1,40}>|某(?:某|数据|账号|账户|用户|个)")
UNDETERMINABLE_RE = re.compile(
    r"不报错|无报错|没有.{0,6}报错|不会出错|不出错|正确处理|符合预期|视情况|"
    r"正常运行|功能正常|正常工作|正常显示|显示正常")
ASYNC_RE = re.compile(r"自动|异步|稍后|生效|轮询|超时")
TIME_WORDS_RE = re.compile(r"毫秒|秒|分钟|小时|工作日|当天|次日|立即|实时|即刻")
# 中文无空格，「A或B」两侧取 CJK；排掉 或者/或许/或以
ALTERNATIVE_RE = re.compile(r"[\u4e00-\u9fff]或[^者许\s，。；]")
# 疑似用例但漏编号：bullet 行含「预期结果/预期：」或优先级标签，却无 TC 编号
CASE_LIKE_RE = re.compile(r"预期[结果：:]|预期行为|\[(?:P0|P1|P2)\]")
APPENDIX_MARKS = ("附录", "审查记录")


class Case:
    def __init__(self, line, tc, rest, module):
        self.line = line
        self.tc = tc
        self.rest = rest.strip().rstrip("*").strip()
        self.module = module
        self.fields = {}
        self.compact = True  # 无字段行时按紧凑式处理

    def field_text(self, name):
        return self.fields.get(name, "")

    @property
    def expected(self):
        return self.field_text("预期结果") or (self.compact and self.rest or "")

    @property
    def has_steps(self):
        return bool(self.field_text("操作步骤")) or (self.compact and len(self.rest) >= 10)


def parse(text):
    lines = text.splitlines()
    cases, current_module, in_appendix, current = [], None, False, None
    pending_module = None  # 正在读模块标题下的用例（E6 判定用）

    for idx, line in enumerate(lines, 1):
        h = HEADING_RE.match(line)
        if h:
            current = None
            title = h.group(2).strip()
            if h.group(1) == "##" and title.startswith(APPENDIX_MARKS):
                in_appendix = True
            elif h.group(1) == "##":
                m = MODULE_HEADING_RE.match(line)
                pending_module = int(m.group(1)) if m else None
            continue
        if in_appendix:
            current = None
            continue

        m = CASE_LINE_RE.match(line)
        if m:
            current = Case(idx, m.group(2), m.group(3), pending_module)
            cases.append(current)
            continue

        if current is not None:
            fm = FIELD_RE.match(line)
            if fm:
                current.compact = False
                current.fields[fm.group(1)] = fm.group(2)
            elif BULLET_RE.match(line) or not line.strip():
                continue  # 续行（多行步骤）或空行，允许
            else:
                current = None  # 遇普通段落，用例块结束
    return cases


def is_appendix_line(text, line_no, appendix_starts):
    return any(line_no >= s for s in appendix_starts)


def lint(path: Path):
    text = path.read_text(encoding="utf-8", errors="replace")
    # 附录起始行：## 附录 / ## 审查记录 标题行号
    appendix_starts = [i for i, l in enumerate(text.splitlines(), 1)
                       if (m := HEADING_RE.match(l)) and m.group(1) == "##"
                       and m.group(2).strip().startswith(APPENDIX_MARKS)]
    cases = parse(text)
    errors, warns = [], []

    if not cases:
        return [], [f"E0 第 1 行: 未识别到任何用例（编号须为 TC-{{模块号}}-{{序号}}）"], []

    seen = {}
    p0 = total_tagged = 0
    case_lines = {c.line for c in cases}
    for c in cases:
        canon = TC_CANON_RE.match(c.tc)
        if not canon:
            errors.append(f"E1 第 {c.line} 行: 编号 {c.tc} 不符合 TC-模块号-序号")
        else:
            if c.module is not None and int(canon.group(1)) != c.module:
                errors.append(
                    f"E6 第 {c.line} 行: {c.tc} 模块号 {canon.group(1)} 与所在章节 "
                    f"`## {c.module}.` 不符")
        if c.tc in seen:
            errors.append(f"E2 第 {c.line} 行: {c.tc} 重复（首次出现于第 {seen[c.tc]} 行）")
        else:
            seen[c.tc] = c.line

        if not c.has_steps or not c.expected:
            missing = []
            if not c.has_steps:
                missing.append("操作步骤")
            if not c.expected:
                missing.append("预期结果")
            errors.append(f"E3 第 {c.line} 行: {c.tc} 缺 {'/'.join(missing)}")

        body = " ".join(c.fields.values()) + (" " + c.rest if c.compact else "")
        # E4/E5 仅查非附录区的正文（附录代码证据豁免）——用例行本身必在正文区
        if PLACEHOLDER_RE.search(body):
            errors.append(f"E4 第 {c.line} 行: {c.tc} 存在占位符（{{xxx}}/<xxx>/某某）。"
                          f"待补数据写 TODO 并注明向谁索取")
        if c.expected and UNDETERMINABLE_RE.search(c.expected):
            errors.append(f"E5 第 {c.line} 行: {c.tc} 预期结果不可判定，"
                          f"须写具体可观察断言（SKILL 步骤 1 红线）")

        if c.expected and ASYNC_RE.search(c.expected) and not TIME_WORDS_RE.search(c.expected):
            warns.append(f"W1 第 {c.line} 行: {c.tc} 异步预期缺判定时限")
        if c.expected and ALTERNATIVE_RE.search(c.expected):
            warns.append(f"W4 第 {c.line} 行: {c.tc} 预期含「或」非唯一，建议拆条")

        if PRIORITY_RE.search(c.rest):
            total_tagged += 1
            if PRIORITY_RE.search(c.rest).group(1) == "P0":
                p0 += 1
        else:
            warns.append(f"W2 第 {c.line} 行: {c.tc} 未标优先级 [P0/P1/P2]")

    # 漏编号扫描：像用例但没写 TC 编号（非附录、非字段行、非已识别用例行）
    for idx, line in enumerate(text.splitlines(), 1):
        if is_appendix_line(text, idx, appendix_starts):
            continue
        if idx in case_lines or FIELD_RE.match(line) or not BULLET_RE.match(line):
            continue
        if CASE_LIKE_RE.search(line):
            errors.append(f"E1 第 {idx} 行: 疑似用例未编号——bullet 含预期/优先级但无 "
                          f"TC-{{模块号}}-{{序号}}（契约见 test-artifacts.md §1）")

    n = len(cases)
    if n >= 6 and p0 / n >= 0.35:
        warns.append(f"W3 P0 占比 {p0}/{n} = {p0/n:.0%} ≥35%——逐条复核是否真属冒烟级，"
                     f"或降级（冒烟集要保持筛选意义）")

    summary = {"file": str(path), "cases": n, "errors": len(errors), "warnings": len(warns),
               "p0": p0, "tagged": total_tagged}
    return summary, errors, warns


def main() -> int:
    parser = argparse.ArgumentParser(description="测试用例红线检查器（契约见 test-artifacts.md §1）")
    parser.add_argument("file", help="用例 markdown 文件（docs/test-cases/*.md）")
    parser.add_argument("--json", action="store_true", help="输出 JSON")
    args = parser.parse_args()

    path = Path(args.file)
    if not path.exists():
        print(f"文件不存在: {path}", file=sys.stderr)
        return 2
    summary, errors, warns = lint(path)

    if args.json:
        print(json.dumps({**summary, "error_items": errors, "warning_items": warns},
                         ensure_ascii=False, indent=2))
    else:
        for e in errors:
            print(f"ERROR  {e}")
        for w in warns:
            print(f"WARN   {w}")
        print(f"\n用例 {summary['cases']} 条 | ERROR {len(errors)} | WARN {len(warns)}"
              + ("  → 阻断定稿，修复后重跑" if errors else "  → 通过（告警项人工确认）"))
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
