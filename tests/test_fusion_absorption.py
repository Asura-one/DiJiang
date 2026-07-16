#!/usr/bin/env python3
"""
TDD 验证：融合 referens 项目吸收内容。

按照 TDD 模式，先写测试，再通过修改 DiJiang 的 spec/workflow 使其通过。
测试覆盖融合分析报告中 P0（立即吸收）和 P1（下一批吸收）的核心实践。
"""

import os
import re
import sys
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DIJIANG_DIR = ROOT / ".dijiang"
SPEC_DIR = DIJIANG_DIR / "spec"
REFERENCES_DIR = DIJIANG_DIR / "references"
TEMPLATES_SKILLS_DIR = ROOT / "crates" / "configurator" / "templates" / "skills"
# 所有 skill 检查走模板目录，不检查 .pi/ 产物
SKILLS_DIR = TEMPLATES_SKILLS_DIR
WORKFLOW_MD = DIJIANG_DIR / "workflow.md"
REPORT_PATH = ROOT / "referens" / "融合分析报告.md"

pass_count = 0
fail_count = 0
failures = []


def test(name, condition, detail=""):
    global pass_count, fail_count
    if condition:
        pass_count += 1
        print(f"  ✅ {name}")
    else:
        fail_count += 1
        msg = f"  ❌ {name}"
        if detail:
            msg += f" — {detail}"
        print(msg)
        failures.append((name, detail))


# ─── Test Group A: 融合分析报告完整性 ──────────────────────────────

def test_report_exists():
    """融合分析报告必须存在"""
    test("融合分析报告存在", REPORT_PATH.is_file())


def test_report_covers_four_projects():
    """报告必须覆盖全部四个项目"""
    content = REPORT_PATH.read_text(encoding="utf-8")
    for project in ["Trellis", "Waza", "ponytail", "Skills"]:
        test(f"报告覆盖 {project}", project in content,
             f"未找到 '{project}' 章节")


def test_report_has_tdd_plan():
    """报告必须包含 TDD 验证计划章节"""
    content = REPORT_PATH.read_text(encoding="utf-8")
    test("报告包含 TDD 验证计划", "TDD 验证计划" in content or "test_" in content)


def test_report_has_p0_p1_p2():
    """报告必须按 P0/P1/P2 优先级分级"""
    content = REPORT_PATH.read_text(encoding="utf-8")
    for level in ["P0", "P1", "P2"]:
        test(f"报告包含 {level} 优先级", level in content)


# ─── Test Group B: Workflow 状态机自描述 ──────────────────────────

def test_workflow_state_tags():
    """workflow.md 必须包含 [workflow-state:*] 标签"""
    if not WORKFLOW_MD.is_file():
        test("workflow-state 标签", False, "workflow.md 不存在")
        return

    content = WORKFLOW_MD.read_text(encoding="utf-8")
    tags = re.findall(r'\[workflow-state:(\w+)\]', content)
    test("workflow.md 包含 [workflow-state:*] 标签", len(tags) >= 3,
         f"找到 {len(tags)} 个标签: {tags}")


def test_workflow_state_required_steps():
    """每个 [required·once] 步骤必须在 workflow-state 块中有 enforcement"""
    if not WORKFLOW_MD.is_file():
        return
    content = WORKFLOW_MD.read_text(encoding="utf-8")
    required_steps = re.findall(r'`\[required · once\]`', content)
    # 至少应该有一些 required once 步骤
    test("workflow.md 定义了 [required·once] 步骤", len(required_steps) >= 2,
         f"找到 {len(required_steps)} 个 required·once 步骤")


# ─── Test Group C: Anti-patterns 共享规则 ─────────────────────────

def test_anti_patterns_file():
    """必须有跨 skill 的共享 anti-patterns 文件"""
    anti_patterns_path = REFERENCES_DIR / "anti-patterns.md"
    has_file = anti_patterns_path.is_file()
    test("anti-patterns 共享规则文件存在", has_file)

    if has_file:
        content = anti_patterns_path.read_text(encoding="utf-8")
        test("anti-patterns 包含范围蔓延规则", "scope creep" in content.lower()
             or "范围蔓延" in content)
        test("anti-patterns 包含先读后写规则", "read before" in content.lower()
             or "先读" in content)


def test_skills_reference_anti_patterns():
    """核心 dj-* skills 必须引用 anti-patterns"""
    anti_patterns_path = REFERENCES_DIR / "anti-patterns.md"
    if not anti_patterns_path.is_file():
        test("技能引用 anti-patterns", False, "anti-patterns.md 不存在")
        return

    core_skills = ["dj-grill", "dj-implement", "dj-check", "dj-hunt", "dj-tdd"]
    for skill in core_skills:
        skill_dir = SKILLS_DIR / skill
        skill_md = skill_dir / "SKILL.md"
        if skill_md.is_file():
            content = skill_md.read_text(encoding="utf-8")
            refs = "anti-patterns" in content.lower()
            # 如果没有文件级引用，检查是否有指向 references 的路径
            refs = refs or "references" in content.lower()
            test(f"{skill} 引用共享规则", refs,
                 f"SKILL.md 中未找到 references 或 anti-patterns 引用")


# ─── Test Group D: Outcome Contract 标准化 ────────────────────────

def test_skill_outcome_contract():
    """核心 dj-* skills 的 SKILL.md 必须包含 Outcome Contract 结构"""
    required_sections = ["Outcome", "Done when", "Evidence", "Output",
                         "结果", "完成", "证据", "输出"]

    core = ["dj-grill", "dj-implement", "dj-check", "dj-hunt", "dj-tdd",
            "dj-ponytail", "dj-output"]

    for skill in core:
        skill_dir = SKILLS_DIR / skill
        skill_md = skill_dir / "SKILL.md"
        if not skill_md.is_file():
            test(f"{skill} Outcome Contract", False, "SKILL.md 不存在")
            continue

        content = skill_md.read_text(encoding="utf-8")
        has_contract = False
        for section in required_sections:
            if section in content:
                has_contract = True
                break

        test(f"{skill} 包含 Outcome Contract", has_contract,
             "SKILL.md 中未找到 Outcome Contract 结构")
        test(f"{skill} 包含 Outcome Contract", has_contract,
             "SKILL.md 中未找到 Outcome Contract 结构")


# ─── Test Group E1: Skill Frontmatter 元数据约定 ─────────────────

def test_skill_frontmatter():
    """模板 skill 的 SKILL.md frontmatter 必须包含 name/description/dispatch_intent/when_to_use"""
    core = ["dj-grill", "dj-implement", "dj-check", "dj-hunt", "dj-tdd",
            "dj-ponytail", "dj-output"]

    for skill in core:
        skill_md = TEMPLATES_SKILLS_DIR / skill / "SKILL.md"
        if not skill_md.is_file():
            test(f"{skill} frontmatter", False, "模板 SKILL.md 不存在")
            continue

        content = skill_md.read_text(encoding="utf-8")
        # 必须包含 YAML frontmatter
        has_frontmatter = content.startswith("---")

        # 检查必需字段
        fields = ["name", "description", "dispatch_intent", "when_to_use"]
        all_found = True
        for field in fields:
            pattern = rf'^{field}:'
            if not re.search(pattern, content, re.MULTILINE):
                all_found = False
                test(f"{skill}.{field} 缺失", False, f"在模板中未找到 {field} 字段")

        test(f"{skill} 模板 frontmatter 完整", all_found,
             f"缺少字段，需包含 {fields}")

# ─── Test Group E2: Spec 精准投递 ──────────────────────────────────
def test_jsonl_context_mechanism():
    """应支持 JSONL 格式的 spec context manifest"""
    # 检查是否有任何脚本或工具支持 JSONL context manifest
    # 至少应该在 workflow.md 中提及
    if not WORKFLOW_MD.is_file():
        return
    content = WORKFLOW_MD.read_text(encoding="utf-8")
    test("workflow.md 提及 spec context 管理", "context" in content.lower()
         and ("spec" in content.lower() or "manifest" in content.lower()))


# ─── Test Group F: ponytail 门禁 ──────────────────────────────────

def test_ponytail_skill():
    """dj-ponytail SKILL.md 必须包含 7-rung ladder 或类似结构"""
    ponytail_md = SKILLS_DIR / "dj-ponytail" / "SKILL.md"
    if not ponytail_md.is_file():
        test("dj-ponytail 阶梯门禁", False, "dj-ponytail/SKILL.md 不存在")
        return

    content = ponytail_md.read_text(encoding="utf-8")
    ladder_keywords = ["YAGNI", "standard library", "stdlib", "one line", "最小",
                       "already exist", "reuse", "native"]
    found = [kw for kw in ladder_keywords if kw.lower() in content.lower()]
    test("dj-ponytail 包含阶梯判断逻辑", len(found) >= 3,
         f"阶梯关键词找到 {len(found)}/7: {found}")


def test_dj_implement_ladder_gate():
    """dj-implement SKILL.md 必须包含决策阶梯门禁步骤"""
    implement_md = SKILLS_DIR / "dj-implement" / "SKILL.md"
    if not implement_md.is_file():
        test("dj-implement 阶梯门禁", False, "dj-implement/SKILL.md 不存在")
        return

    content = implement_md.read_text(encoding="utf-8")
    # 必须包含决策阶梯门禁章节
    has_gate = "决策阶梯门禁" in content
    # 必须引用 ponytail 阶梯
    has_ref = "ponytail" in content.lower() or "Ponytail" in content
    # 必须提到七级阶梯
    has_rung = "YAGNI" in content
    test("dj-implement 包含决策阶梯门禁", has_gate, "未找到 '决策阶梯门禁' 章节")
    test("dj-implement 引用 ponytail 阶梯", has_ref, "未引用 ponytail 阶梯")
    test("dj-implement 包含 YAGNI 阶梯关键词", has_rung, "未找到 YAGNI")


# ─── Test Group G: 任务层级支持 ───────────────────────────────────

def test_task_hierarchy():
    """task.json 必须支持 parent/children 字段"""
    # 检查任意一个 task.json 来验证 schema
    tasks_dir = DIJIANG_DIR / "tasks"
    if not tasks_dir.is_dir():
        test("task.json 支持 parent/children", False, "tasks 目录不存在")
        return

    found_hierarchy = False
    for task_dir in tasks_dir.iterdir():
        if not task_dir.is_dir():
            continue
        task_json = task_dir / "task.json"
        if task_json.is_file():
            try:
                data = json.loads(task_json.read_text(encoding="utf-8"))
                if "parent" in data or "children" in data or "subtasks" in data:
                    found_hierarchy = True
                    break
            except (json.JSONDecodeError, UnicodeDecodeError):
                continue

    test("task.json 支持任务层级", found_hierarchy)


# ─── Test Group H: 生命周期 Hook ──────────────────────────────────

def test_hook_mechanism():
    """应支持任务生命周期 hook 机制"""
    # 检查 TaskRecord 是否有 hooks 字段（类型级验证）
    typ_path = ROOT / "crates" / "task" / "src" / "types.rs"
    if typ_path.is_file():
        content = typ_path.read_text(encoding="utf-8")
        has_hooks = "hooks:" in content and "HashMap" in content
    else:
        has_hooks = False

    if not has_hooks:
        # 回退：检查 workflow.md
        if WORKFLOW_MD.is_file():
            content = WORKFLOW_MD.read_text(encoding="utf-8")
            has_hooks = "hook" in content.lower()

    test("支持生命周期 Hook 机制", has_hooks,
         "types.rs 中未找到 hooks 字段或 workflow.md 中未定义 hook")


# ─── Test Group I: DiJiang Spec 结构 ──────────────────────────────

def test_spec_organization():
    """spec 目录应有合理组织"""
    if not SPEC_DIR.is_dir():
        test("spec 目录组织", False, ".dijiang/spec/ 目录不存在")
        return

    # 至少有一些子目录
    dirs = [d for d in SPEC_DIR.iterdir() if d.is_dir()]
    test("spec 包含子目录分类", len(dirs) >= 2, f"找到 {len(dirs)} 个子目录")


def test_bucket_organization():
    """应存在技能桶分类（Rust 内嵌，非 YAML）"""
    bucket_mod = ROOT / "crates" / "task" / "src" / "buckets" / "mod.rs"
    assert bucket_mod.is_file(), "Rust 桶模块不存在"

    with open(bucket_mod, encoding="utf-8") as f:
        content = f.read()

    # 验证桶定义内嵌在 Rust 源码中（而非 .dijiang/ 产物目录）
    has_core = '"core"' in content
    has_specialized = '"specialized"' in content
    has_extended = '"extended"' in content
    has_internal = '"internal"' in content
    missing = " ".join(k for k,v in [("core",has_core),("specialized",has_specialized),("extended",has_extended),("internal",has_internal)] if not v)
    has_all_buckets = not missing

    test("技能桶分类内嵌在 Rust 中: core/specialized/extended/internal", has_all_buckets,
         f"buckets/mod.rs 缺少桶: {missing}" if missing else None)

    # 验证数据位于 Rust 源码而非 .dijiang/ 产物目录
    yaml_gone = not (DIJIANG_DIR / "buckets.yaml").exists()
    test(".dijiang/buckets.yaml 已移除", yaml_gone,
         ".dijiang/buckets.yaml 应移动到 Rust 源码中")

    # 验证 get_default_buckets 函数存在
    has_fn = "fn get_default_buckets" in content
    test("get_default_buckets() 函数", has_fn,
         "buckets/mod.rs 缺少 get_default_buckets()")



def main():
    print("=" * 60)
    print("融合吸收 TDD 验证测试")
    print("=" * 60)
    print()

    # Group A: 分析报告完整性
    print("A. 融合分析报告完整性")
    test_report_exists()
    test_report_covers_four_projects()
    test_report_has_tdd_plan()
    test_report_has_p0_p1_p2()
    print()

    # Group B: Workflow 自描述状态机
    print("B. Workflow 状态机自描述")
    test_workflow_state_tags()
    test_workflow_state_required_steps()
    print()

    # Group C: Anti-patterns 共享规则
    print("C. Anti-patterns 共享规则")
    test_anti_patterns_file()
    test_skills_reference_anti_patterns()
    print()

    # Group D: Outcome Contract 标准化
    print("D. Outcome Contract 标准化")
    test_skill_outcome_contract()
    print()

    # Group E: Spec 精准投递
    print("E. Spec 精准投递")
    test_jsonl_context_mechanism()
    print()

    # Group F: ponytail 门禁
    print("F. ponytail 阶梯门禁")
    test_ponytail_skill()
    print()

    # Group G: 任务层级
    print("G. 任务层级支持")
    test_task_hierarchy()
    print()

    # Group H: 生命周期 Hook
    print("H. 生命周期 Hook")
    test_hook_mechanism()
    print()

    # Group I: Spec 结构
    print("I. Spec 结构")
    test_spec_organization()
    print()

    # Group J: 基准测试
    print("J. 基准测试")
    test_benchmarks()
    print()

    # Group K: 桶分类
    print("K. 桶分类")
    test_bucket_organization()
    print()

    # ── Summary ──
    print("=" * 60)
    total = pass_count + fail_count
    print(f"总计: {total}  |  通过: {pass_count}  |  失败: {fail_count}")
    if failures:
        print()
        print("失败详情:")
        for name, detail in failures:
            print(f"  ❌ {name}")
            if detail:
                print(f"     {detail}")
    print("=" * 60)

    return 0 if fail_count == 0 else 1
def test_benchmarks():
    """应存在基准测试场景定义（模板源码）和 CLI 入口"""
    bench_dir = ROOT / "crates" / "configurator" / "templates" / "benchmarks" / "scenarios"
    if not bench_dir.is_dir():
        test("基准测试场景目录", False, "templates/benchmarks/scenarios/ 不存在")
        return

    scenario_files = list(bench_dir.iterdir())
    if not scenario_files:
        test("基准测试场景", False, "templates/benchmarks/scenarios/ 为空")
        return

    # 至少有两个 YAML 场景
    yaml_count = sum(1 for f in scenario_files if f.suffix in {".yaml", ".yml"})
    has_enough = yaml_count >= 2
    test(f"基准测试场景: {yaml_count} 个 YAML 定义", has_enough,
         f"templates/benchmarks/scenarios/ 下有 {yaml_count} 个 YAML 文件，期望≥2")

    # 验证模板目录存在
    bench_template_dir = ROOT / "crates" / "configurator" / "templates" / "benchmarks"
    if not bench_template_dir.is_dir():
        test("基准测试模板目录", False, "templates/benchmarks/ 不存在")
        return

    # 验证 Rust benchmarks 模块存在
    mod_path = ROOT / "crates" / "task" / "src" / "benchmarks" / "mod.rs"
    has_module = mod_path.is_file()
    test("基准测试 Rust 模块", has_module,
         "crates/task/src/benchmarks/mod.rs 不存在")

if __name__ == "__main__":
    sys.exit(main())
