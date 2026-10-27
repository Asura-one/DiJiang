#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""verify_skills.py — DiJiang 技能库机械校验（stdlib-only）

检查项：
  1. glob 自动发现 skills/*/*/SKILL.md（不维护手工清单，防清单漂移）
  2. frontmatter 完整性（--- 开头 / 闭合 / name 字段）
  3. description 契约（长度 40-500 / 含 "Use when" / 含 "触发词" / 不以 The/A/This 起）
  4. 引用文件存在性（references/ 与 docs/references/ 相对路径真实存在）
  5. 触发词两两 Jaccard 重叠（>= 0.5 报 FAIL，防两技能抢同一触发面）
  6. 反模式表机械完整性（编号连续 / 无重名 / 解析 0 条目即 FAIL）
  7. canary 承重短语守卫（核心纪律短语在 AGENTS.md 与 README.md 双存活）
  8. STOM 三纯净（禁平台斜杠命令 / 禁绝对路径 / 工具用能力描述）+ 3000B 体量上限
  9. docs 防腐（历史变迁词 / 单文档限长）
 10. TTL 复核挂载（review_before frontmatter 过期报 WARN，缺失统计）

用法：
  python scripts/verify_skills.py           # 全量检查，存量 FAIL 以告警清单输出，exit 0
  python scripts/verify_skills.py --strict  # 任何 FAIL 退出码 1（CI/门禁用）

设计来源：Waza checks_content.py（description 契约 + Jaccard）、ponytail check-rule-copies.js（canary）、
琅嬛 skill-token-optimization（STOM）、dsh-agent-notes-maintenance（docs 防腐）、knowledge-base-lifecycle-governance（TTL）

门禁自愈语义（DJ-004）：检查器自身的"数据缺失/解析失败"路径一律 FAIL 而非 WARN——
"检查器抓不到东西"不等于"没有问题"；--strict 下解析失配必须阻断。
突变测试见 tests/test_verify_skills.py：故意注入损坏夹具断言门禁咬合。
"""
import os
import re
import sys
from datetime import date

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# canary 承重短语：DiJiang 核心纪律，要求在两个入口文档中逐字存活。
# 改动这些措辞前必须同步两处，否则守卫报警。
CANARY = {
    "worktree": ["AGENTS.md", "README.md"],          # Worktree-First 纪律
    "Conventional Commits": ["AGENTS.md", "README.md"],  # 提交格式约定
    "使用中文编写": ["AGENTS.md", "README.md"],        # 中文提交信息
}

FM_RE = re.compile(r"\n?---\s*\n(.*?)\n---\s*\n", re.S)
REF_RE = re.compile(r"`?([\w\-./]*references/[\w\-./]+\.(?:md|sh|py|fish|json))`?")
TRIGGER_RE = re.compile(r"触发词[：:]\s*(.+)")
# STOM：平台斜杠命令（换行后紧跟 /command 形式）；绝对路径；具体工具名而非能力描述
SLASH_CMD_RE = re.compile(r"\n/[a-z][\w-]+[\s\n]")
ABS_PATH_RE = re.compile(r"[A-Z]:\\\\|/Users/|/home/")
TOOL_NAME_RE = re.compile(r"\b(Edit|Write|Read|Grep|Glob|Bash)\s*(?:工具|tool|Tool)\b")
# docs 防腐：历史变迁词（历史应进 CHANGELOG，正文只写当前事实）
ROT_WORDS_RE = re.compile(r"不再|以前|曾经|现在改为|已废弃|原先")

BODY_MAX = 3000      # dj-meta 硬上限：SKILL.md 正文 3000B
DOC_MAX = 2000       # 单篇 docs 文档字数参考上限
TTL_DIRS = ("docs/references", "docs/adr")


def resolve_ref(skill_dir, ref):
    """解析引用真实路径：docs/ 前缀 -> 仓库根；<技能名>/references/ -> 跨技能引用；references/ -> 本技能目录。"""
    if ref.startswith("docs/"):
        return os.path.join(ROOT, ref)
    parts = ref.split("/")
    if len(parts) >= 3 and parts[0] != "references" and parts[1] == "references":
        for bucket in ("engineering", "productivity"):
            cand = os.path.join(ROOT, "skills", bucket, parts[0], *parts[1:])
            if os.path.exists(cand):
                return cand
        return os.path.join(ROOT, "skills", "__missing__", ref)
    return os.path.join(skill_dir, ref)


class Report:
    def __init__(self):
        self.passes, self.warns, self.fails = [], [], []

    def ok(self, msg):
        self.passes.append(msg)

    def warn(self, msg):
        self.warns.append(msg)

    def fail(self, msg):
        self.fails.append(msg)


def discover():
    """glob 自动发现全部 SKILL.md，不维护手工清单。"""
    out = []
    for bucket in sorted(os.listdir(os.path.join(ROOT, "skills"))):
        bdir = os.path.join(ROOT, "skills", bucket)
        if not os.path.isdir(bdir):
            continue
        for name in sorted(os.listdir(bdir)):
            p = os.path.join(bdir, name, "SKILL.md")
            if os.path.isfile(p):
                out.append((bucket, name, p))
    return out


def parse_frontmatter(text):
    """返回 (frontmatter_dict, body)。frontmatter 损坏返回 (None, None)。"""
    if not text.startswith("---"):
        return None, None
    m = FM_RE.match(text)
    if not m:
        return None, None
    fm_text = m.group(1)
    props = {}
    for line in fm_text.splitlines():
        if ":" in line and not line.startswith((" ", "-", "AIGC")):
            k, _, v = line.partition(":")
            props[k.strip()] = v.strip().strip("'\"")
    body = text[m.end():]
    return props, body


def check_frontmatter(rep, bucket, name, path):
    text = open(path, encoding="utf-8").read()
    props, body = parse_frontmatter(text)
    ident = f"{bucket}/{name}"
    if props is None:
        rep.fail(f"[frontmatter] {ident}: frontmatter 缺失或未闭合")
        return None, None
    if not props.get("name"):
        rep.fail(f"[frontmatter] {ident}: 缺 name 字段")
    elif props["name"] != name:
        rep.fail(f"[frontmatter] {ident}: name 字段 ({props['name']}) 与目录名不一致")
    else:
        rep.ok(f"frontmatter {ident}")
    if body is not None and len(body.strip()) < 50:
        rep.warn(f"[body] {ident}: 正文过短（<50 字符）")
    return props, body


def check_description(rep, bucket, name, props):
    desc = (props or {}).get("description", "")
    ident = f"{bucket}/{name}"
    if not desc:
        rep.fail(f"[description] {ident}: description 缺失")
        return ""
    n = len(desc)
    if not (40 <= n <= 500):
        rep.warn(f"[description] {ident}: 长度 {n} 超出 40-500 参考")
    if "Use when" not in desc and "use when" not in desc.lower():
        rep.warn(f"[description] {ident}: 缺 'Use when' 触发短语")
    if "触发词" not in desc:
        rep.warn(f"[description] {ident}: 缺 '触发词' 段（dj-dispatch 路由锚点）")
    if re.match(r"^(The|A|This)\b", desc):
        rep.fail(f"[description] {ident}: 以 The/A/This 起头，应动词开头")
    return desc


def check_refs(rep, bucket, name, path):
    """SKILL.md 引用的 references/ 与 docs/references/ 文件必须真实存在。"""
    text = open(path, encoding="utf-8").read()
    ident = f"{bucket}/{name}"
    skill_dir = os.path.dirname(path)
    for ref in set(REF_RE.findall(text)):
        target = resolve_ref(skill_dir, ref)
        if not os.path.exists(target):
            rep.fail(f"[ref] {ident}: 引用不存在 -> {ref}")
    rep.ok(f"refs {ident}")


def check_stom(rep, bucket, name, body):
    """STOM 三纯净 + 体量上限：平台可移植性 + 跨环境可读性。"""
    ident = f"{bucket}/{name}"
    text = body or ""
    if SLASH_CMD_RE.search("\n" + text):
        rep.warn(f"[stom] {ident}: 疑似平台斜杠命令（/xxx）——纯 skill 库应写能力描述而非平台触发符")
    hits = ABS_PATH_RE.findall(text)
    if hits:
        rep.warn(f"[stom] {ident}: 含绝对路径 {sorted(set(hits))[:3]}——改用仓库相对路径")
    if TOOL_NAME_RE.search(text):
        rep.warn(f"[stom] {ident}: 以具体工具名指代操作——改用能力描述（'修改文件'而非'Edit 工具'）")
    if len(text) > BODY_MAX:
        rep.warn(f"[stom] {ident}: 正文 {len(text)}B 超过 dj-meta 硬上限 {BODY_MAX}B——把深度内容外溢到 references/")
    rep.ok(f"stom {ident}")


def trigger_tokens(desc):
    m = TRIGGER_RE.search(desc or "")
    if not m:
        return []
    return [t for t in re.split(r"[、，,；;/\s]+", m.group(1)) if t]


def check_jaccard(rep, descs):
    """两两技能触发词 Jaccard >= 0.5 报 FAIL（防两技能抢同一触发面）。"""
    keys = sorted(descs)
    reported = 0
    for i, a in enumerate(keys):
        for b in keys[i + 1:]:
            ta, tb = trigger_tokens(descs[a]), trigger_tokens(descs[b])
            if not ta or not tb:
                continue
            inter = len(set(ta) & set(tb))
            union = len(set(ta) | set(tb))
            j = inter / union if union else 0.0
            if j >= 0.5:
                rep.fail(f"[jaccard] {a} vs {b}: 触发词重叠度 {j:.2f} >= 0.5（{sorted(set(ta) & set(tb))}）")
                reported += 1
    if not reported:
        rep.ok("jaccard 全库无高重叠技能对")


def check_antipatterns(rep):
    p = os.path.join(ROOT, "docs", "references", "anti-patterns.md")
    if not os.path.exists(p):
        rep.fail("[anti-patterns] 文件不存在——承重文档丢失，门禁拒绝放行")
        return
    rows = re.findall(r"^\|\s*(\d+)\s*\|\s*\*{2}(.+?)\*{2}\s*\|", open(p, encoding="utf-8").read(), re.M)
    nums = [int(n) for n, _ in rows]
    names = [nm for _, nm in rows]
    if not nums:
        # 门禁自愈语义（DJ-004）：解析器抓不到东西 ≠ 没有问题
        rep.fail("[anti-patterns] 解析到 0 条目——表格格式漂移导致检查器失明，strict 下必须阻断")
        return
    if nums != list(range(1, len(nums) + 1)):
        rep.fail(f"[anti-patterns] 编号不连续: {nums}")
    else:
        rep.ok(f"anti-patterns 编号连续 1-{len(nums)}")
    dup = {n for n in names if names.count(n) > 1}
    if dup:
        rep.fail(f"[anti-patterns] 条目重名: {sorted(dup)}")
    banned = re.compile(r"DiJiang|Forgejo|pentest")
    for nm in names:
        if banned.search(nm):
            rep.fail(f"[anti-patterns] 条目含项目专名: {nm}")


def check_canary(rep):
    for phrase, files in CANARY.items():
        for fn in files:
            p = os.path.join(ROOT, fn)
            if not os.path.exists(p):
                # 门禁自愈语义（DJ-004）：入口文档丢失必须 FAIL，不能静默跳过
                rep.fail(f"[canary] {fn} 不存在——承重文档丢失，无法核实短语 '{phrase}'")
                continue
            if phrase not in open(p, encoding="utf-8").read():
                rep.fail(f"[canary] 承重短语丢失: '{phrase}' 不在 {fn} —— 核心纪律措辞被改动，检查是否需要同步两处")
    rep.ok("canary 承重短语")


def _iter_docs():
    for fn in ("AGENTS.md", "README.md", "CONTEXT.md", "SCOPE.md"):
        p = os.path.join(ROOT, fn)
        if os.path.isfile(p):
            yield p
    for dirpath, _dirs, files in os.walk(os.path.join(ROOT, "docs")):
        for f in files:
            if f.endswith(".md"):
                yield os.path.join(dirpath, f)


def check_docs_freshness(rep):
    """docs 防腐：历史变迁词（历史进 CHANGELOG，正文写当前事实）+ 单文档限长。"""
    rot_hits = 0
    for p in _iter_docs():
        rel = os.path.relpath(p, ROOT)
        with open(p, encoding="utf-8") as f:
            text = f.read()
        if os.sep + "adr" + os.sep in p:
            # ADR 是决策历史记录，变迁词是其正当用法，只查限长
            words = set()
        else:
            words = {w for w in ROT_WORDS_RE.findall(text)}
        if words:
            rot_hits += 1
            rep.warn(f"[docs-fresh] {rel}: 含历史变迁词 {sorted(words)}——历史应进 CHANGELOG，正文只写当前事实")
        if len(text) > DOC_MAX:
            rep.warn(f"[docs-fresh] {rel}: {len(text)} 字超过 {DOC_MAX} 参考上限——考虑拆分或外溢 references/")
    if rot_hits == 0:
        rep.ok("docs 无历史变迁词")


def check_ttl(rep):
    """TTL 复核挂载：review_before 过期报 WARN；缺失统计（存量逐步补齐，不强制）。"""
    today = date.today()
    missing, expired = [], []
    for d in TTL_DIRS:
        base = os.path.join(ROOT, d)
        if not os.path.isdir(base):
            continue
        for fn in sorted(os.listdir(base)):
            if not fn.endswith(".md"):
                continue
            p = os.path.join(base, fn)
            text = open(p, encoding="utf-8").read()
            props, _ = parse_frontmatter(text) if text.startswith("---") else ({}, None)
            rb = (props or {}).get("review_before", "")
            rel = os.path.relpath(p, ROOT)
            if not rb:
                missing.append(rel)
                continue
            try:
                if date.fromisoformat(rb.strip()) < today:
                    expired.append(rel)
                    rep.warn(f"[ttl] {rel}: review_before={rb} 已过期——复核后四选一留痕（顺延/修订/归档/废止）")
            except ValueError:
                rep.warn(f"[ttl] {rel}: review_before={rb!r} 非 ISO 日期")
    if expired or missing:
        rep.ok(f"ttl 扫描完成：过期 {len(expired)}，缺 review_before {len(missing)}（存量补齐项，不阻塞）")
    else:
        rep.ok("ttl 全部在册且未过期")


def main():
    strict = "--strict" in sys.argv
    rep = Report()
    skills = discover()
    if not skills:
        print("FATAL: 未发现任何 SKILL.md")
        return 2
    rep.ok(f"发现 {len(skills)} 个技能（glob 自动发现）")

    descs = {}
    for bucket, name, path in skills:
        props, body = check_frontmatter(rep, bucket, name, path)
        desc = check_description(rep, bucket, name, props)
        check_refs(rep, bucket, name, path)
        check_stom(rep, bucket, name, body)
        descs[f"{bucket}/{name}"] = desc

    check_jaccard(rep, descs)
    check_antipatterns(rep)
    check_canary(rep)
    check_docs_freshness(rep)
    check_ttl(rep)

    print(f"PASS {len(rep.passes)}  |  WARN {len(rep.warns)}  |  FAIL {len(rep.fails)}")
    for w in rep.warns:
        print("WARN ", w)
    for f in rep.fails:
        print("FAIL ", f)
    if not rep.warns and not rep.fails:
        print("ALL CLEAN")
    if strict and rep.fails:
        print("\n--strict 模式：FAIL 阻断（exit 1）")
        return 1
    if strict and rep.warns:
        print("\n--strict 模式：WARN 不阻断，但请清理存量")
    return 0


if __name__ == "__main__":
    sys.exit(main())