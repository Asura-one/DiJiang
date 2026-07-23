use crate::util::{require_dijiang_dir, run_git, git_current_branch, git_worktree_root, trim_required};
use dijiang_task::hooks::{self, HookEvent};
use dijiang_task::store;
use dijiang_task::types::{TaskRecord, TaskStatus};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DispatchRoute {
    pub task_type: &'static str,
    pub primary_intent: &'static str,
    pub skill: &'static str,
    pub recommended_path: &'static str,
    pub status: TaskStatus,
    pub intent: dijiang_task::RouteIntent,
    pub complexity: dijiang_task::TaskComplexity,
}

#[derive(Debug, Clone)]
pub struct WorktreeDecision {
    pub readiness: dijiang_task::WorktreeReadiness,
}

#[derive(Debug, Clone)]
pub struct DispatchDecision {
    pub route: DispatchRoute,
    pub decision: dijiang_task::RouteDecision,
}

fn strip_embedded_context(prompt: &str) -> String {
    let mut output = String::with_capacity(prompt.len());
    let mut rest = prompt;
    loop {
        let Some(start) = rest.find("<skill ") else {
            output.push_str(rest);
            break;
        };
        output.push_str(&rest[..start]);
        let after_start = &rest[start..];
        if let Some(end) = after_start.find("</skill>") {
            rest = &after_start[end + "</skill>".len()..];
        } else {
            break;
        }
    }
    output
}

pub fn dispatch_route(prompt: &str) -> DispatchRoute {
    let visible_prompt = strip_embedded_context(prompt);
    let lower = visible_prompt.to_lowercase();
    let has_any = |words: &[&str]| words.iter().any(|word| lower.contains(word));
    let has_vague_bug_intent = has_any(&[
        "修 bug", "修bug", "fix bug", "fix bugs", "修复 bug", "修复bug",
        "有个 bug", "有 bug", "bug 这些", "bug这些",
    ]);
    let has_specific_failure_signal = has_any(&[
        "排查", "调试", "debug", "crash", "error", "fail", "报错",
        "崩溃", "无法启动", "不能运行", "失败", "复现", "日志", "stack", "trace",
    ]);
    let has_specific_implementation_signal = has_any(&[
        "字段", "接口", "按钮", "页面", "文件", "函数", "方法", "模块", "配置",
        "校验", "样式", "布局", "api", "cli", "command", "config", "button", "field",
    ]);
    let has_vague_feature_intent = has_any(&[
        "做个", "做一个", "加个", "加一个", "新增个", "新增一个",
        "实现个", "实现一个", "优化", "改进", "提升", "体验",
    ]) && !has_specific_implementation_signal;
    let has_hunt_intent = has_specific_failure_signal
        || lower.contains("bug") && !has_vague_bug_intent;

    if has_hunt_intent {
        return DispatchRoute {
            task_type: "排查调试",
            primary_intent: "排查根因",
            skill: "dj-hunt",
            recommended_path: "dj-hunt → dj-implement → dj-check",
            status: TaskStatus::InProgress,
            intent: dijiang_task::RouteIntent::Debug,
            complexity: dijiang_task::TaskComplexity::Complex,
        };
    }
    if has_vague_feature_intent {
        return DispatchRoute {
            task_type: "调研对齐",
            primary_intent: "需求澄清",
            skill: "dj-grill",
            recommended_path: "dj-grill → dj-output/dj-implement",
            status: TaskStatus::Planning,
            intent: dijiang_task::RouteIntent::Align,
            complexity: dijiang_task::TaskComplexity::Complex,
        };
    }
    if has_any(&["审计", "安全", "扫描", "体检", "audit", "security"]) {
        return DispatchRoute {
            task_type: "审计代码",
            primary_intent: "代码审计",
            skill: "dj-audit",
            recommended_path: "dj-audit → dj-implement → dj-check",
            status: TaskStatus::InProgress,
            intent: dijiang_task::RouteIntent::Check,
            complexity: dijiang_task::TaskComplexity::Complex,
        };
    }
    if has_any(&["调研", "research", "资料", "技术方案对比"]) {
        return DispatchRoute {
            task_type: "技术调研",
            primary_intent: "调研收集信息",
            skill: "dj-research",
            recommended_path: "dj-research → dj-output/dj-implement",
            status: TaskStatus::Planning,
            intent: dijiang_task::RouteIntent::Research,
            complexity: dijiang_task::TaskComplexity::Complex,
        };
    }
    if has_any(&["方案", "对比", "url", "网页", "compare"]) {
        return DispatchRoute {
            task_type: "调研对齐",
            primary_intent: "调研并对齐",
            skill: "dj-grill",
            recommended_path: "dj-grill → dj-output/dj-tdd",
            status: TaskStatus::Planning,
            intent: dijiang_task::RouteIntent::Align,
            complexity: dijiang_task::TaskComplexity::Complex,
        };
    }
    if has_any(&["文档", "prd", "设计文档", "润色", "document", "write"]) {
        return DispatchRoute {
            task_type: "写文档",
            primary_intent: "文档产出",
            skill: "dj-output",
            recommended_path: "dj-output",
            status: TaskStatus::Planning,
            intent: dijiang_task::RouteIntent::Document,
            complexity: dijiang_task::TaskComplexity::Complex,
        };
    }
    if has_any(&["脚本", "工具", "自动化", "script", "cli", "tool"]) {
        return DispatchRoute {
            task_type: "脚本工具",
            primary_intent: "脚本或工具实现",
            skill: "dj-script",
            recommended_path: "dj-script → dj-check",
            status: TaskStatus::InProgress,
            intent: dijiang_task::RouteIntent::Implement,
            complexity: dijiang_task::TaskComplexity::Complex,
        };
    }
    if has_any(&["ui", "页面", "样式", "布局", "组件", "design", "style"]) {
        return DispatchRoute {
            task_type: "设计 UI",
            primary_intent: "UI 设计实现",
            skill: "dj-design",
            recommended_path: "dj-design → dj-implement → dj-check",
            status: TaskStatus::InProgress,
            intent: dijiang_task::RouteIntent::Implement,
            complexity: dijiang_task::TaskComplexity::Complex,
        };
    }
    if has_any(&["测试", "tdd", "test"]) {
        return DispatchRoute {
            task_type: "测试开发",
            primary_intent: "测试驱动开发",
            skill: "dj-tdd",
            recommended_path: "dj-tdd → dj-check",
            status: TaskStatus::InProgress,
            intent: dijiang_task::RouteIntent::Implement,
            complexity: dijiang_task::TaskComplexity::Complex,
        };
    }
    if has_any(&["实现", "修复", "重构", "新增", "修改", "改",
        "implement", "fix", "refactor", "add"])
    {
        return DispatchRoute {
            task_type: "代码开发",
            primary_intent: "实现变更",
            skill: "dj-implement",
            recommended_path: "dj-implement → dj-check",
            status: TaskStatus::InProgress,
            intent: dijiang_task::RouteIntent::Implement,
            complexity: dijiang_task::TaskComplexity::Complex,
        };
    }
    DispatchRoute {
        task_type: "调研对齐",
        primary_intent: "需求澄清",
        skill: "dj-grill",
        recommended_path: "dj-grill → dj-output/dj-implement",
        status: TaskStatus::Planning,
        intent: dijiang_task::RouteIntent::Unknown,
            complexity: dijiang_task::TaskComplexity::Complex,
    }
}

pub fn dispatch_route_from_skill(skill: &str) -> Option<DispatchRoute> {
    match skill {
        "dj-hunt" => Some(DispatchRoute {
            task_type: "排查调试", primary_intent: "继续排查",
            skill: "dj-hunt", recommended_path: "dj-hunt → dj-implement → dj-check",
            status: TaskStatus::InProgress, intent: dijiang_task::RouteIntent::Debug,
            complexity: dijiang_task::TaskComplexity::Complex,
        }),
        "dj-implement" => Some(DispatchRoute {
            task_type: "代码开发", primary_intent: "继续实现",
            skill: "dj-implement", recommended_path: "dj-implement → dj-check",
            status: TaskStatus::InProgress, intent: dijiang_task::RouteIntent::Implement,
            complexity: dijiang_task::TaskComplexity::Complex,
        }),
        "dj-script" => Some(DispatchRoute {
            task_type: "脚本工具", primary_intent: "继续实现脚本或工具",
            skill: "dj-script", recommended_path: "dj-script → dj-check",
            status: TaskStatus::InProgress, intent: dijiang_task::RouteIntent::Implement,
            complexity: dijiang_task::TaskComplexity::Complex,
        }),
        "dj-tdd" => Some(DispatchRoute {
            task_type: "测试开发", primary_intent: "继续 TDD",
            skill: "dj-tdd", recommended_path: "dj-tdd → dj-check",
            status: TaskStatus::InProgress, intent: dijiang_task::RouteIntent::Implement,
            complexity: dijiang_task::TaskComplexity::Complex,
        }),
        "dj-check" => Some(DispatchRoute {
            task_type: "代码审查", primary_intent: "质量检查",
            skill: "dj-check", recommended_path: "dj-check",
            status: TaskStatus::InProgress, intent: dijiang_task::RouteIntent::Check,
            complexity: dijiang_task::TaskComplexity::Complex,
        }),
        "dj-output" => Some(DispatchRoute {
            task_type: "写文档", primary_intent: "文档产出",
            skill: "dj-output", recommended_path: "dj-output",
            status: TaskStatus::Planning, intent: dijiang_task::RouteIntent::Document,
            complexity: dijiang_task::TaskComplexity::Complex,
        }),
        "dj-grill" => Some(DispatchRoute {
            task_type: "调研对齐", primary_intent: "需求澄清",
            skill: "dj-grill", recommended_path: "dj-grill → dj-output/dj-implement",
            status: TaskStatus::Planning, intent: dijiang_task::RouteIntent::Align,
            complexity: dijiang_task::TaskComplexity::Complex,
        }),
        "dj-research" => Some(DispatchRoute {
            task_type: "调研对齐", primary_intent: "需求澄清",
            skill: "dj-grill", recommended_path: "dj-grill → dj-output/dj-implement",
            status: TaskStatus::Planning, intent: dijiang_task::RouteIntent::Align,
            complexity: dijiang_task::TaskComplexity::Complex,
        }),
        "dijiang-finish-work" => Some(DispatchRoute {
            task_type: "收尾归档", primary_intent: "完成工作",
            skill: "dijiang-finish-work", recommended_path: "dijiang-finish-work",
            status: TaskStatus::Completed, intent: dijiang_task::RouteIntent::Finish,
            complexity: dijiang_task::TaskComplexity::Complex,
        }),
        "dijiang-continue" => Some(DispatchRoute {
            task_type: "恢复上下文", primary_intent: "继续暂停任务",
            skill: "dijiang-continue", recommended_path: "dijiang-continue",
            status: TaskStatus::Paused, intent: dijiang_task::RouteIntent::Resume,
            complexity: dijiang_task::TaskComplexity::Complex,
        }),
        "dijiang-start" => Some(DispatchRoute {
            task_type: "恢复上下文", primary_intent: "重新激活归档任务",
            skill: "dijiang-start", recommended_path: "dijiang-start",
            status: TaskStatus::Archived, intent: dijiang_task::RouteIntent::Resume,
            complexity: dijiang_task::TaskComplexity::Complex,
        }),
        _ => None,
    }
}

pub fn dispatch_route_for_active_task(task: &TaskRecord) -> DispatchRoute {
    match task.status {
        TaskStatus::Planning => DispatchRoute {
            task_type: "调研对齐", primary_intent: "需求澄清",
            skill: "dj-grill", recommended_path: "dj-grill → dj-output/dj-implement",
            status: TaskStatus::Planning, intent: dijiang_task::RouteIntent::Align,
            complexity: dijiang_task::TaskComplexity::Complex,
        },
        TaskStatus::InProgress => task
            .meta
            .get("dispatch")
            .and_then(|dispatch| dispatch.get("skill"))
            .and_then(|skill| skill.as_str())
            .and_then(dispatch_route_from_skill)
            .unwrap_or(DispatchRoute {
                task_type: "代码开发", primary_intent: "继续实现",
                skill: "dj-implement", recommended_path: "dj-implement → dj-check",
                status: TaskStatus::InProgress, intent: dijiang_task::RouteIntent::Implement,
            complexity: dijiang_task::TaskComplexity::Complex,
            }),
        TaskStatus::Completed => DispatchRoute {
            task_type: "收尾归档", primary_intent: "完成工作",
            skill: "dijiang-finish-work", recommended_path: "dijiang-finish-work",
            status: TaskStatus::Completed, intent: dijiang_task::RouteIntent::Finish,
            complexity: dijiang_task::TaskComplexity::Complex,
        },
        TaskStatus::Paused => DispatchRoute {
            task_type: "恢复上下文", primary_intent: "继续暂停任务",
            skill: "dijiang-continue", recommended_path: "dijiang-continue",
            status: TaskStatus::Paused, intent: dijiang_task::RouteIntent::Resume,
            complexity: dijiang_task::TaskComplexity::Complex,
        },
        TaskStatus::Archived => DispatchRoute {
            task_type: "恢复上下文", primary_intent: "重新激活归档任务",
            skill: "dijiang-start", recommended_path: "dijiang-start",
            status: TaskStatus::Archived, intent: dijiang_task::RouteIntent::Resume,
            complexity: dijiang_task::TaskComplexity::Complex,
        },
    }
}

pub fn apply_route_gate(
    status: &TaskStatus,
    route: DispatchRoute,
    requested_skill: Option<&str>,
    dijiang_dir: &Path,
    tasks_dir: &Path,
    task_name: Option<&str>,
) -> DispatchDecision {
    let mut decision = dijiang_task::evaluate_route(
        status,
        route.intent,
        requested_skill.or(Some(route.skill)),
        Some(route.complexity),
    );
    // Apply readiness gate: verify prerequisites before implementation skills
    if let Some(name) = task_name {
        decision = store::apply_readiness_gate(dijiang_dir, tasks_dir, name, &decision);
    }
    // Apply completion gate: block Finish action if checklist is incomplete
    if let Some(name) = task_name {
        decision = store::apply_completion_gate(tasks_dir, Some(name), &decision);
    }
    let resolved_skill = decision.resolved_skill;
    let gated_route = dispatch_route_from_skill(resolved_skill).unwrap_or(DispatchRoute {
        task_type: route.task_type,
        primary_intent: route.primary_intent,
        skill: resolved_skill,
        recommended_path: route.recommended_path,
        status: route.status,
        intent: route.intent,
        complexity: route.complexity,
    });
    DispatchDecision {
        route: gated_route,
        decision,
    }
}

pub fn title_from_prompt(prompt: &str) -> String {
    let compact = prompt.split_whitespace().collect::<Vec<_>>().join(" ");
    let title = compact.chars().take(80).collect::<String>();
    if title.trim().is_empty() {
        "Untitled DiJiang Task".to_string()
    } else {
        title
    }
}

pub fn slug_from_prompt(prompt: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in prompt.to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
        if slug.len() >= 48 { break; }
    }
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        format!("task-{}", chrono::Utc::now().format("%Y%m%d%H%M%S"))
    } else {
        slug.to_string()
    }
}

pub fn unique_task_name(tasks_dir: &Path, base: &str) -> String {
    if !tasks_dir.join(base).exists() {
        return base.to_string();
    }
    for index in 2..1000 {
        let candidate = format!("{base}-{index}");
        if !tasks_dir.join(&candidate).exists() {
            return candidate;
        }
    }
    format!("{base}-{}", chrono::Utc::now().timestamp())
}

pub fn route_requires_worktree(route: &DispatchRoute) -> bool {
    matches!(route.skill, "dj-implement" | "dj-hunt" | "dj-tdd" | "dj-script" | "dj-design")
}

pub fn branch_prefix(route: &DispatchRoute) -> &'static str {
    match route.skill {
        "dj-hunt" => "fix",
        "dj-tdd" => "test",
        "dj-script" => "chore",
        _ => "feat",
    }
}

fn git_has_head(project_root: &Path) -> anyhow::Result<bool> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--verify", "HEAD"])
        .current_dir(project_root)
        .output()?;
    Ok(output.status.success())
}

fn unique_git_branch(project_root: &Path, base: &str) -> anyhow::Result<String> {
    for index in 0..1000 {
        let candidate = if index == 0 { base.to_string() } else { format!("{base}-{index}") };
        let output = std::process::Command::new("git")
            .args(["show-ref", "--verify", &format!("refs/heads/{candidate}")])
            .current_dir(project_root)
            .output()?;
        if !output.status.success() {
            return Ok(candidate);
        }
    }
    anyhow::bail!("无法为任务 worktree 生成唯一分支名：{base}")
}

fn unique_worktree_path(project_root: &Path, task_name: &str) -> PathBuf {
    let repo_name = project_root.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("repo");
    let parent = project_root.parent().unwrap_or(project_root);
    for index in 0..1000 {
        let suffix = if index == 0 { String::new() } else { format!("-{index}") };
        let candidate = parent.join(format!("{repo_name}-{task_name}{suffix}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    parent.join(format!("{repo_name}-{task_name}-{}", chrono::Utc::now().timestamp()))
}

pub fn ensure_task_worktree(
    project_root: &Path,
    tasks_dir: &Path,
    task: &mut TaskRecord,
    route: &DispatchRoute,
    current_location: &Path,
    current_worktree_root: Option<&Path>,
    main_worktree_root: Option<&Path>,
) -> anyhow::Result<Option<WorktreeDecision>> {
    if !route_requires_worktree(route) {
        return Ok(None);
    }

    let input = dijiang_task::GitGateInput {
        current_location: current_location.to_path_buf(),
        current_worktree_root: current_worktree_root.map(Path::to_path_buf),
        main_worktree_root: main_worktree_root.map(Path::to_path_buf),
        route_requires_worktree: true,
    };
    let readiness = dijiang_task::evaluate_worktree_readiness(task, &input);

    if readiness.state == dijiang_task::GitGateState::Ready || !readiness.needs_provision {
        return Ok(Some(WorktreeDecision { readiness }));
    }

    if !git_has_head(project_root)? {
        return Ok(Some(WorktreeDecision {
            readiness: dijiang_task::worktree_readiness(
                task,
                dijiang_task::GitGateState::Blocked,
                current_location,
                false,
                Some("当前 git 仓库还没有提交，无法创建任务 worktree；请先建立基线提交。".to_string()),
            ),
        }));
    }

    let base_branch = git_current_branch(project_root).unwrap_or_else(|_| "HEAD".to_string());
    let branch_base = format!("{}/{}", branch_prefix(route), task.name);
    let branch = unique_git_branch(project_root, &branch_base)?;
    let path = unique_worktree_path(project_root, &task.name);
    let path_string = path.display().to_string();
    run_git(project_root, &["worktree", "add", &path_string, "-b", &branch, &base_branch])?;

    task.branch = Some(branch);
    task.base_branch = Some(base_branch);
    task.worktree_path = Some(path_string);
    store::save_task(tasks_dir, task)?;

    Ok(Some(WorktreeDecision {
        readiness: dijiang_task::worktree_readiness(
            task,
            dijiang_task::GitGateState::Provisioned,
            current_location,
            true,
            None,
        ),
    }))
}

pub fn dispatch_skill_manifests_text(capsule: dijiang_task::WorkflowCapsule) -> String {
    let manifests = dijiang_task::manifests_for_capsule(capsule);
    if manifests.is_empty() {
        return "<dijiang-skill-manifests>\nnone\n</dijiang-skill-manifests>".to_string();
    }
    let lines = manifests.into_iter()
        .map(|manifest| format!("- {} | {} | phases={} | risk={}",
            manifest.name, manifest.summary, manifest.phases.join(","), manifest.risk))
        .collect::<Vec<_>>().join("\n");
    format!("<dijiang-skill-manifests>\n{}\n</dijiang-skill-manifests>", lines)
}

pub fn dispatch_target_skill_bodies(
    capsule: dijiang_task::WorkflowCapsule,
    primary_skill: &str,
    recommended_path: &str,
) -> String {
    let selected = dijiang_task::select_skill_bodies(capsule, primary_skill, recommended_path);
    if selected.is_empty() { return String::new(); }
    let mut cache = dijiang_task::SkillBodyCache::default();
    dijiang_task::render_selected_skill_bodies(&selected, &mut cache)
}

pub fn dispatch_runtime_skill_context(dispatch: &DispatchDecision) -> String {
    let manifests = dispatch_skill_manifests_text(dispatch.decision.capsule.clone());
    let targets = dispatch_target_skill_bodies(
        dispatch.decision.capsule,
        dispatch.route.skill,
        dispatch.route.recommended_path,
    );
    format!("{}\n{}", manifests, targets)
}

pub fn dispatch_context(
    task_name: &str,
    title: &str,
    dispatch: &DispatchDecision,
    state_context: &str,
    worktree: Option<&WorktreeDecision>,
    original_status: Option<&TaskStatus>,
) -> String {
    let route = &dispatch.route;
    let decision = &dispatch.decision;
    let worktree_line = match worktree {
        Some(wd) => match wd.readiness.state {
            dijiang_task::GitGateState::Provisioned => format!(
                "Git 工作流：Git Gate=provisioned；已创建任务 worktree `{}`，分支 `{}`。\n下一步：切换到该 worktree，读取 `.dijiang/tasks/{task_name}/task.json`，然后按 {skill} 执行。",
                wd.readiness.worktree_path.as_deref().unwrap_or("unknown"),
                wd.readiness.branch.as_deref().unwrap_or("unknown"),
                skill = route.skill,
            ),
            dijiang_task::GitGateState::Ready => format!(
                "Git 工作流：Git Gate=ready；任务 worktree 已就绪 `{}`，分支 `{}`。\n下一步：在该 worktree 中读取 `.dijiang/tasks/{task_name}/task.json`，然后按 {skill} 执行。",
                wd.readiness.worktree_path.as_deref().unwrap_or("unknown"),
                wd.readiness.branch.as_deref().unwrap_or("unknown"),
                skill = route.skill,
            ),
            dijiang_task::GitGateState::Blocked if wd.readiness.needs_provision => format!(
                "Git 工作流：Git Gate=blocked；当前还没有可用的任务 worktree。原因：{}\n下一步：先完成 Git 基线或创建 task worktree，再按 {skill} 执行。",
                wd.readiness.message,
                skill = route.skill,
            ),
            dijiang_task::GitGateState::Blocked => format!(
                "Git 工作流：Git Gate=blocked；任务 worktree 已记录，但当前 runtime 尚未进入正确位置。原因：{}\n下一步：切换到记录的 task worktree 后，再按 {skill} 执行。",
                wd.readiness.message,
                skill = route.skill,
            ),
        },
        None => format!(
            "Git 工作流：当前路线不需要立即创建代码 worktree。\n下一步：读取 `.dijiang/tasks/{task_name}/task.json`，然后按 {skill} 执行。",
            skill = route.skill,
        ),
    };
    let skill_context = dispatch_runtime_skill_context(dispatch);
    let status_hint = if let Some(status) = original_status {
        let legal: Vec<&str> = status.legal_transitions()
.iter()
.map(|t| t.as_str())
.collect();
        format!("\n状态推进：当前状态={}，合法推进方向：{}",
            status.as_str(), legal.join(", "))
    } else {
        String::new()
    };
    format!(
        "<dijiang-dispatch>\n任务：{task_name}\n标题：{title}\n任务类型：{task_type}\n主要意图：{primary_intent}\n路线：{skill}\n推荐路径：{recommended_path}\naction：{action}\nreason：{reason}\nnextAction：{next_action}{status_hint}\n{worktree_line}\n</dijiang-dispatch>\n{skill_context}\n{state_context}",
        task_type = route.task_type,
        primary_intent = route.primary_intent,
        skill = route.skill,
        recommended_path = route.recommended_path,
        action = decision.action.as_str(),
        reason = decision.reason,
        next_action = decision.next_action,
        status_hint = status_hint,
    )
}

pub fn cmd_dispatch(prompt: &str, force_new: bool, json: bool, hook_event: &str) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");

    // Read active task
    let existing_task = store::read_active_task(&dijiang_dir)?
        .and_then(|name| store::load_task(&tasks_dir, &name).ok());

    // Route the prompt
    let dispatch = match &existing_task {
        Some(task) if matches!(hook_event, "session:start" | "session_start") => {
            let route = dispatch_route_for_active_task(task);
            apply_route_gate(&task.status, route, Some(prompt), &dijiang_dir, &tasks_dir, Some(&task.name))
        }
        Some(task) => {
            let route = dispatch_route(prompt);
            apply_route_gate(&task.status, route, Some(prompt), &dijiang_dir, &tasks_dir, Some(&task.name))
        }
        None => {
            let route = dispatch_route(prompt);
            let gate_status = &route.status.clone();
            let mut dispatch = apply_route_gate(gate_status, route, None, &dijiang_dir, &tasks_dir, None);
            dispatch.decision.next_action = "continue with the requested skill for the new task".to_string();
            dispatch
        }
    };

    let cwd = std::env::current_dir()?;
    let worktree_root = git_worktree_root(&cwd)?;
    let project_root = dijiang_dir.parent()
        .ok_or_else(|| anyhow::anyhow!("无法确定项目根目录"))?;

    // Gather task context — create new task if none exists
    let (task_name, title, mut task) = if let Some(task) = existing_task {
        (task.name.clone(), task.title.clone(), task.clone())
    } else {
        let clean_prompt = strip_embedded_context(prompt);
        let title = title_from_prompt(&clean_prompt);
        let slug = slug_from_prompt(&clean_prompt);
        let unique_name = unique_task_name(&tasks_dir, &slug);
        let mut new_task = store::create_task(&unique_name, &title);
        new_task.status = dispatch.route.status.clone();
        new_task.meta = serde_json::json!({
            "hookEventName": hook_event,
            "route": {
                "skill": dispatch.route.skill,
                "recommended_path": dispatch.route.recommended_path,
                "action": dispatch.decision.action.as_str(),
                "reason": dispatch.decision.reason,
                "nextAction": dispatch.decision.next_action,
            },
        });
        store::activate_new_task(&dijiang_dir, &new_task)?;
        hooks::run_task_hooks(&dijiang_dir, HookEvent::AfterTaskCreate, &unique_name);
        (unique_name, title, new_task)
    };
    // Sync task status with route status via transition validation
    // Capture original status for transition hints
    let original_status = task.status.clone();
    if task.status != dispatch.route.status {
        task = match store::update_status(&tasks_dir, &task_name, dispatch.route.status.clone()) {
            Ok(updated) => updated,
            Err(store::TaskError::InvalidTransition { from, to }) => {
                let legal: Vec<&str> = from.legal_transitions()
                    .iter()
                    .map(|s| s.as_str())
                    .collect();
                anyhow::bail!("✗ 无法从 {} 状态推进到 {}。\n  合法推进方向：{}",
                    from.as_str(), to.as_str(), legal.join(", "));
            }
            Err(e) => return Err(e.into()),
        };
    }

    // Worktree decision (for both existing and new tasks)
    let main_worktree_root = crate::commands::finish::git_main_worktree(project_root, "main").ok();
    let worktree_decision = ensure_task_worktree(
        project_root, &tasks_dir, &mut task, &dispatch.route,
        &cwd, worktree_root.as_deref(), main_worktree_root.as_deref(),
    )?;

    // No fallback implement-route provision: worktrees are only created when the
    // active route itself requires one (route_requires_worktree). Status
    // planning→in_progress alone must not invent a worktree for check/docs routes.

    // Build state_context from workflow state
    let state_context = match dijiang_task::workflow_state::build(&dijiang_dir) {
        Ok(state) => state.additional_context(),
        Err(_) => String::new(),
    };

    // Build dispatch context
    let context = dispatch_context(&task_name, &title, &dispatch, &state_context, worktree_decision.as_ref(), Some(&original_status));

    if json {
        let payload = serde_json::json!({
            "hookEventName": hook_event,
            "additionalContext": context,
            "route": {
                "skill": dispatch.route.skill,
                "recommended_path": dispatch.route.recommended_path,
                "action": dispatch.decision.action.as_str(),
                "reason": dispatch.decision.reason,
                "nextAction": dispatch.decision.next_action,
            },
            "gitGate": worktree_decision.as_ref().map(|wd| {
                serde_json::json!({
                    "state": wd.readiness.state.as_str(),
                    "branch": wd.readiness.branch,
                    "worktreePath": wd.readiness.worktree_path,
                    "needsProvision": wd.readiness.needs_provision,
                    "locationKind": wd.readiness.location_kind.as_str(),
                })
            }),
        });
        println!("{}", serde_json::to_string(&payload)?);
    } else {
        println!("{}", context);
    }

    Ok(())
}
