#!/usr/bin/env python3
"""测试报告「机读摘要段」校验器——把聚合契约变成可机器判定的门。

Adapted from fishzjp/qa-skills core/scripts/report_meta_check.py (MIT)，契约改为
DiJiang dj-fullstack-testing/references/test-artifacts.md §2（report_meta 嵌套结构）。
背景同上游：报告正文人读没问题，但摘要段写坏（缺字段/计数不平/枚举越界）时聚合侧才炸；
产出方有多个（dj-fullstack-testing / 回归复验 / 验收报告），此前没有任何脚本校验它。

用法：
  python3 report_lint.py docs/testing/report-20261118.md [--json]

检查项（E = 错误阻断；W = 告警人工确认）：
  E1 未找到 report_meta 摘要块，或出现多个
  E2 必填字段缺失：project / date / source_run / summary{total,passed,failed,blocked,missing}
     / exit_code_hint / triage / bugs
  E3 枚举越界：source_run ∈ {pr-smoke,nightly-full,release-gate,manual-full}；
     exit_code_hint ∈ {0,1,2,3}；triage 键 ⊆ {A,B1,B2,C,D,U}；bugs.severity ∈ {S0,S1,S2}；
     bugs.status ∈ {新建,已修复待验证,已验证关闭,不予修复}
  E4 计数自洽：summary 各值为非负整数；passed+failed+blocked+missing = total；
     failed = triage 各类之和；bugs 每条须有 id/severity/status
  W1 摘要块不是全文最后一个代码块（契约要求文末）
  W2 date 不是 YYYY-MM-DD
exit 0 = 通过；1 = 有错误；2 = 用法/读取错误。
"""

import argparse
import json
import re
import sys
from pathlib import Path

FENCE_RE = re.compile(r"```ya?ml[ \t]*\n(.*?)```", re.S)
MARKER = "report_meta:"
SOURCE_RUNS = {"pr-smoke", "nightly-full", "release-gate", "manual-full"}
TRIAGE_KEYS = {"A", "B1", "B2", "C", "D", "U"}
SEVERITIES = {"S0", "S1", "S2"}
BUG_STATUSES = {"新建", "已修复待验证", "已验证关闭", "不予修复"}
SUMMARY_KEYS = ("total", "passed", "failed", "blocked", "missing")


def strip_comment(line: str) -> str:
    """去掉行尾 # 注释（不处理引号内的 #，本契约值不含 #）。"""
    out, in_quote = [], None
    for ch in line:
        if in_quote:
            if ch == in_quote:
                in_quote = None
        elif ch in "\"'":
            in_quote = ch
        elif ch == "#":
            break
        out.append(ch)
    return "".join(out)


def scalar(text: str):
    t = text.strip()
    if len(t) >= 2 and t[0] == t[-1] and t[0] in "\"'":
        return t[1:-1]
    if re.fullmatch(r"-?\d+", t):
        return int(t)
    if t in ("true", "false"):
        return t == "true"
    return t


def split_flow(text: str):
    """拆 flow 映射/列表的一层（值里带引号逗号也安全）。"""
    items, buf, in_quote, depth = [], [], None, 0
    for ch in text:
        if in_quote:
            buf.append(ch)
            if ch == in_quote:
                in_quote = None
            continue
        if ch in "\"'":
            in_quote = ch
        elif ch in "{[(":
            depth += 1
        elif ch in "}])":
            depth -= 1
        elif ch == "," and depth == 0:
            items.append("".join(buf))
            buf = []
            continue
        buf.append(ch)
    if "".join(buf).strip():
        items.append("".join(buf))
    return [i.strip() for i in items if i.strip()]


def parse_value(rest: str):
    if rest.startswith("{") and rest.endswith("}"):
        out = {}
        for part in split_flow(rest[1:-1]):
            if ":" not in part:
                continue
            k, v = part.split(":", 1)
            out[scalar(k)] = scalar(v)
        return out
    if rest.startswith("[") and rest.endswith("]"):
        return [scalar(p) for p in split_flow(rest[1:-1])]
    return scalar(rest)


def indent_of(line: str) -> int:
    return len(line) - len(line.lstrip())


def parse_map(lines, i, indent, errors):
    out = {}
    while i < len(lines):
        line = lines[i]
        cur = indent_of(line)
        s = line.strip()
        if cur < indent or s.startswith("- "):
            break
        if cur > indent:
            i += 1  # 容错：多余缩进，跳过
            continue
        if ":" not in s:
            errors.append(f"E4 无法解析的行: {s}")
            i += 1
            continue
        key, rest = s.split(":", 1)
        key, rest = key.strip(), rest.strip()
        if rest:
            out[key] = parse_value(rest)
            i += 1
            continue
        if i + 1 < len(lines) and lines[i + 1].strip().startswith("- "):
            val, i = parse_list(lines, i + 1, indent_of(lines[i + 1]), errors)
        elif i + 1 < len(lines) and indent_of(lines[i + 1]) > cur:
            val, i = parse_map(lines, i + 1, indent_of(lines[i + 1]), errors)
        else:
            out[key] = None
            i += 1
        out[key] = val
    return out, i


def parse_list(lines, i, indent, errors):
    out = []
    while i < len(lines) and lines[i].strip().startswith("- ") and indent_of(lines[i]) == indent:
        item_text = lines[i].strip()[2:].strip()
        item = {}
        j = i
        if ":" in item_text and not item_text.startswith(("{", "[")):
            k, v = item_text.split(":", 1)
            item[k.strip()] = parse_value(v.strip())
            # 后续更深缩进的键值属于同一条目
            j = i + 1
            while j < len(lines) and indent_of(lines[j]) > indent:
                s = lines[j].strip()
                if ":" in s and not s.startswith("- "):
                    kk, vv = s.split(":", 1)
                    item[kk.strip()] = parse_value(vv.strip())
                j += 1
        else:
            item = parse_value(item_text)
            j = i + 1
        out.append(item)
        i = j
    return out, i


def check_block(block_text, errors, warnings):
    raw = [strip_comment(l.rstrip()) for l in block_text.splitlines()]
    lines = [l for l in raw if l.strip()]
    if not lines or not lines[0].strip().startswith(MARKER):
        errors.append("E1 摘要块首个非空行不是 report_meta:")
        return {}
    rest = lines[1:]
    if not rest:
        errors.append("E2 report_meta 下无内容")
        return {}
    doc, _ = parse_map(rest, 0, indent_of(rest[0]), errors)
    return doc


def check(md_text, errors, warnings):
    blocks = [b for b in FENCE_RE.findall(md_text) if MARKER in b]
    if not blocks:
        errors.append("E1 未找到 report_meta 摘要块（```yaml 围栏 + report_meta:）")
        return
    if len(blocks) > 1:
        errors.append(f"E1 出现 {len(blocks)} 个 report_meta 摘要块，应唯一")
        return
    all_blocks = FENCE_RE.findall(md_text)
    if all_blocks and all_blocks[-1] != blocks[0]:
        warnings.append("W1 摘要块不是文末最后一个代码块（契约要求放报告末节）")

    doc = check_block(blocks[0], errors, warnings)
    if not doc:
        return

    for key in ("project", "date", "source_run", "summary", "exit_code_hint", "triage", "bugs"):
        if key not in doc:
            errors.append(f"E2 缺少必填字段 {key}")
    summary = doc.get("summary") or {}
    if not isinstance(summary, dict):
        errors.append("E2 summary 必须是映射")
        summary = {}
    for key in SUMMARY_KEYS:
        if key not in summary:
            errors.append(f"E2 summary 缺少 {key}")

    src = doc.get("source_run")
    if src is not None and src not in SOURCE_RUNS:
        errors.append(f"E3 source_run={src} 越界（{'/'.join(sorted(SOURCE_RUNS))}）")
    hint = doc.get("exit_code_hint")
    if hint is not None and hint not in (0, 1, 2, 3):
        errors.append(f"E3 exit_code_hint={hint} 越界（0/1/2/3）")

    counts = {}
    for key in SUMMARY_KEYS:
        v = summary.get(key)
        if key in summary and not (isinstance(v, int) and not isinstance(v, bool) and v >= 0):
            errors.append(f"E4 summary.{key}={v!r} 不是非负整数")
        elif isinstance(v, int):
            counts[key] = v

    triage = doc.get("triage") or {}
    if not isinstance(triage, dict):
        errors.append("E2 triage 必须是映射")
        triage = {}
    for k, v in triage.items():
        if k not in TRIAGE_KEYS:
            errors.append(f"E3 triage 键 {k} 越界（A/B1/B2/C/D/U）")
        if not (isinstance(v, int) and not isinstance(v, bool) and v >= 0):
            errors.append(f"E4 triage.{k}={v!r} 不是非负整数")

    if len(counts) == len(SUMMARY_KEYS):
        s = sum(counts[k] for k in ("passed", "failed", "blocked", "missing"))
        if s != counts["total"]:
            errors.append(f"E4 计数不自洽：passed+failed+blocked+missing={s} ≠ total={counts['total']}")
    if isinstance(triage, dict) and all(isinstance(v, int) and not isinstance(v, bool) for v in triage.values()):
        if "failed" in counts and sum(triage.values()) != counts["failed"]:
            errors.append(f"E4 triage 合计 {sum(triage.values())} ≠ summary.failed={counts['failed']}")

    bugs = doc.get("bugs")
    if bugs is not None and not isinstance(bugs, list):
        errors.append("E2 bugs 必须是列表（无缺陷写 bugs: []）")
        bugs = []
    for idx, bug in enumerate(bugs or [], 1):
        if not isinstance(bug, dict):
            errors.append(f"E4 bugs 第 {idx} 条不是映射")
            continue
        for key in ("id", "severity", "status"):
            if key not in bug:
                errors.append(f"E4 bugs 第 {idx} 条缺 {key}")
        if bug.get("severity") is not None and bug.get("severity") not in SEVERITIES:
            errors.append(f"E3 bugs[{bug.get('id')}] severity={bug.get('severity')} 越界（S0/S1/S2）")
        if bug.get("status") is not None and bug.get("status") not in BUG_STATUSES:
            errors.append(f"E3 bugs[{bug.get('id')}] status={bug.get('status')} 越界")

    date = doc.get("date")
    if date is not None and not re.fullmatch(r"\d{4}-\d{2}-\d{2}", str(date)):
        warnings.append(f"W2 date={date} 不是 YYYY-MM-DD")


def main() -> int:
    ap = argparse.ArgumentParser(description="测试报告机读摘要段校验（契约 test-artifacts.md §2）")
    ap.add_argument("file", help="测试报告 markdown（docs/testing/*.md）")
    ap.add_argument("--json", action="store_true", help="输出 JSON")
    args = ap.parse_args()

    path = Path(args.file)
    if not path.exists():
        print(f"文件不存在: {path}", file=sys.stderr)
        return 2
    text = path.read_text(encoding="utf-8", errors="replace")
    errors, warnings = [], []
    check(text, errors, warnings)

    if args.json:
        print(json.dumps({"file": str(path), "errors": errors, "warnings": warnings},
                         ensure_ascii=False, indent=2))
    else:
        for e in errors:
            print(f"ERROR  {e}")
        for w in warnings:
            print(f"WARN   {w}")
        print(f"\n机读摘要段：ERROR {len(errors)} | WARN {len(warnings)}"
              + ("  → 修复后重跑" if errors else "  → 通过"))
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())