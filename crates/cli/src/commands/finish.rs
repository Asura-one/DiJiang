use crate::util::{require_dijiang_dir, run_git, read_developer, current_session_key, git_current_branch, git_worktree_root};
use dijiang_task::store;
use dijiang_task::types::{TaskRecord, TaskStatus};
use std::path::{Path, PathBuf};
use std::io::Write;
use serde_json::Value;

#[derive(Debug, Clone, Copy)]
pub struct FinishWorkOptions<'a> {
    pub summary: Option<&'a str>,
    pub verification: Option<&'a str>,
    pub docs_sync: Option<&'a str>,
    pub version_impact: &'a str,
    pub commit: bool,
    pub commit_message: Option<&'a str>,
    pub push: bool,
    pub integrate: bool,
    pub approve_integrate: bool,
    pub approve_cleanup: bool,
    pub main_branch: &'a str,
    pub remote: &'a str,
    pub allow_dirty: bool,
    pub keep_worktree: bool,
}

fn git_dirty_entries(project_root: &Path) -> anyhow::Result<Vec<String>> {
    if !project_root.join(".git").exists() {
        return Ok(Vec::new());
    }
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(project_root)
        .output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "git status failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

fn find_dijiang_dir_in_git_worktrees(project_root: &Path) -> anyhow::Result<Option<PathBuf>> {
    let output = std::process::Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(project_root)
        .output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "git worktree list failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            let candidate = PathBuf::from(path).join(".dijiang");
            if candidate.is_dir() {
                return Ok(Some(candidate));
            }
        }
    }
    Ok(None)
}

fn recover_finish_task_from_branch(
    tasks_dir: &Path,
    branch: &str,
) -> Option<(String, TaskRecord)> {
    let branch = branch.trim();
    if branch.is_empty() { return None; }
    let tasks = store::list_tasks(tasks_dir).ok()?;
    tasks.into_iter()
        .find(|task| task.branch.as_deref() == Some(branch) || task.name == branch)
        .map(|task| (task.name.clone(), task))
}

#[derive(Debug, Clone)]
struct ResolvedFinishTarget {
    task_name: String,
    task: TaskRecord,
}

fn resolve_finish_target(
    tasks_dir: &Path,
    active_task: Option<&str>,
    current_branch: Option<&str>,
    worktree_hint: Option<&str>,
) -> anyhow::Result<Option<ResolvedFinishTarget>> {
    let recover = |hint: Option<&str>| {
        hint.and_then(|value| recover_finish_task_from_branch(tasks_dir, value))
            .map(|(task_name, task)| ResolvedFinishTarget { task_name, task })
    };
    match active_task {
        Some(active_task) => match store::load_task(tasks_dir, active_task) {
            Ok(task) => Ok(Some(ResolvedFinishTarget { task_name: active_task.to_string(), task })),
            Err(store::TaskError::NotFound(_)) => recover(current_branch)
                .or_else(|| recover(worktree_hint))
                .map(Some)
                .ok_or_else(|| anyhow::anyhow!(
                    "finish-work 的 active task 指向 `{active_task}`，但 `.dijiang/tasks/{active_task}/task.json` 不存在。这通常表示 task state 已陈旧或 task artifact 被清理。请用 `dijiang task current` / `dijiang task list` 检查状态；若当前工作仍需归档，请重新 `dijiang start <name>`，否则清理 stale active task 后再继续。"
                )),
            Err(e) => Err(e.into()),
        },
        None => Ok(recover(current_branch).or_else(|| recover(worktree_hint))),
    }
}

fn append_finish_journal(
    dijiang_dir: &Path,
    developer: &str,
    task_name: &str,
    summary: Option<&str>,
    verification: &str,
    dirty_allowed: bool,
) -> anyhow::Result<PathBuf> {
    let workspace = dijiang_dir.join("workspace").join(developer);
    std::fs::create_dir_all(&workspace)?;
    let journal = workspace.join("journal.md");
    let summary = summary.unwrap_or("工作已完成。");
    let status = if task_name == "no-active-task" { "completed-no-task" } else { "archived" };
    let entry = format!(
        "\n## {} — finish-work\n- 任务：`{}`\n- 摘要：{}\n- 验证：{}\n- 允许脏改：{}\n- 状态：{}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M"),
        task_name, summary, verification, dirty_allowed, status
    );
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&journal)?
        .write_all(entry.as_bytes())?;
    Ok(journal)
}

fn append_session_closure(
    dijiang_dir: &Path,
    developer: &str,
    session_key: &str,
    source: &str,
    task_name: &str,
    summary: Option<&str>,
    verification: &str,
    dirty_allowed: bool,
) -> anyhow::Result<PathBuf> {
    let closed_at = chrono::Utc::now().to_rfc3339();
    let sessions_dir = dijiang_dir.join("workspace").join(developer).join("sessions");
    std::fs::create_dir_all(&sessions_dir)?;
    let journal = sessions_dir.join(format!("{session_key}.jsonl"));
    let event = serde_json::json!({
        "event": "session_closed",
        "session_key": session_key,
        "source": source,
        "task": task_name,
        "summary": summary.unwrap_or("Work finished and task archived."),
        "verification": verification,
        "dirty_allowed": dirty_allowed,
        "closed_at": closed_at,
    });
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&journal)?
        .write_all(format!("{}\n", serde_json::to_string(&event)?).as_bytes())?;

    let runtime_path = dijiang_dir.join(".runtime").join("sessions").join(format!("{session_key}.json"));
    if let Some(parent) = runtime_path.parent() { std::fs::create_dir_all(parent)?; }
    let mut value: Value = if runtime_path.exists() {
        let content = std::fs::read_to_string(&runtime_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({ "session_key": session_key, "source": source })
    };
    value["closed_at"] = serde_json::json!(closed_at);
    value["closed_task"] = serde_json::json!(task_name);
    value["closed_verification"] = serde_json::json!(verification);
    value["closed_dirty_allowed"] = serde_json::json!(dirty_allowed);
    value["current_task"] = serde_json::Value::Null;
    std::fs::write(runtime_path, serde_json::to_string_pretty(&value)?)?;
    Ok(journal)
}

fn default_commit_message(project_root: &Path, task_name: &str, summary: Option<&str>) -> String {
    let diff_stat = std::process::Command::new("git")
        .args(["diff", "--cached", "--stat"])
        .current_dir(project_root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() { None } else { Some(s) }
        });
    if let Some(stats) = diff_stat {
        let last_line = stats.lines().last().unwrap_or("").to_string();
        let summary = summary.map(str::trim).filter(|v| !v.is_empty()).unwrap_or(task_name);
        let chinese_stat = last_line
            .replace(" files changed", " 个文件变更")
            .replace(" file changed", " 个文件变更")
            .replace(" insertions(+)", " 处新增")
            .replace(" insertion(+)", " 处新增")
            .replace(" deletions(-)", " 处删除")
            .replace(" deletion(-)", " 处删除");
        format!("{}: {}", summary, chinese_stat)
    } else {
        summary.map(str::trim).filter(|v| !v.is_empty()).unwrap_or(task_name).to_string()
    }
}

fn bump_semver(version: &str, impact: &str) -> anyhow::Result<String> {
    let parts = version.split('.').map(str::parse::<u64>).collect::<Result<Vec<_>, _>>()?;
    if parts.len() != 3 { anyhow::bail!("unsupported version format: {version}"); }
    let (major, minor, patch) = (parts[0], parts[1], parts[2]);
    Ok(match impact {
        "major" => format!("{}.0.0", major + 1),
        "minor" => format!("{major}.{}.0", minor + 1),
        "patch" => format!("{major}.{minor}.{}", patch + 1),
        "none" => version.to_string(),
        _ => anyhow::bail!("unsupported version impact: {impact}"),
    })
}

fn update_workspace_version(project_root: &Path, impact: &str) -> anyhow::Result<Option<String>> {
    if impact == "none" { return Ok(None); }
    let cargo_toml = project_root.join("Cargo.toml");
    if !cargo_toml.exists() { return Ok(None); }
    let content = std::fs::read_to_string(&cargo_toml)?;
    let mut in_workspace_package = false;
    let mut changed = false;
    let mut old_version = String::new();
    let mut new_lines = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') { in_workspace_package = trimmed == "[workspace.package]"; }
        if in_workspace_package && trimmed.starts_with("version") && trimmed.contains('=') {
            let indent = line.chars().take_while(|ch| ch.is_whitespace()).collect::<String>();
            let value = trimmed.split_once('=').map(|(_, v)| v.trim().trim_matches('"'))
                .ok_or_else(|| anyhow::anyhow!("invalid version line in Cargo.toml"))?;
            let next = bump_semver(value, impact)?;
            old_version = value.to_string();
            new_lines.push(format!("{indent}version = \"{next}\""));
            changed = true;
            continue;
        }
        new_lines.push(line.to_string());
    }
    if changed {
        std::fs::write(&cargo_toml, format!("{}\n", new_lines.join("\n")))?;
        Ok(Some(format!("{old_version} -> {}", bump_semver(&old_version, impact)?)))
    } else { Ok(None) }
}

fn ensure_finish_preconditions(
    project_root: &Path,
    task: Option<&TaskRecord>,
    options: FinishWorkOptions<'_>,
) -> anyhow::Result<(String, String)> {
    if let Some(task) = task {
        if matches!(task.status, TaskStatus::Archived) {
            anyhow::bail!("Task '{}' is already archived.", task.name);
        }
    }
    if options.commit && options.allow_dirty {
        anyhow::bail!("finish-work 不能同时使用 --commit 和 --allow-dirty");
    }
    if (options.push || options.integrate) && !options.commit {
        anyhow::bail!("--push/--integrate 需要同时使用 --commit");
    }
    if !matches!(options.version_impact, "major" | "minor" | "patch" | "none") {
        anyhow::bail!("--version-impact must be one of: major, minor, patch, none");
    }
    let verification = crate::util::trim_required(
        options.verification,
        "finish-work requires --verification",
    )?;
    let dirty = git_dirty_entries(project_root)?;
    if (options.commit || !dirty.is_empty()) && !options.allow_dirty {
        let docs_sync = crate::util::trim_required(
            options.docs_sync,
            "finish-work requires --docs-sync when code/artifacts changed",
        )?;
        if options.commit { return Ok((verification, docs_sync)); }
    }
    if !dirty.is_empty() && !options.allow_dirty {
        let preview = dirty.iter().take(12).cloned().collect::<Vec<_>>().join("\n  ");
        anyhow::bail!(
            "finish-work 被阻止：git worktree 存在未提交修改。\n  {}\n",
            preview
        );
    }
    Ok((verification, options.docs_sync
        .map(str::trim).filter(|v| !v.is_empty())
        .unwrap_or("none: no code or docs change").to_string()))
}

fn git_common_dir(project_root: &Path) -> anyhow::Result<PathBuf> {
    let path = run_git(project_root, &["rev-parse", "--git-common-dir"])?;
    let path = PathBuf::from(path);
    Ok(if path.is_absolute() { path } else { project_root.join(path) })
}

pub(crate) fn git_main_worktree(project_root: &Path, main_branch: &str) -> anyhow::Result<PathBuf> {
    let _common_dir = git_common_dir(project_root)?;
    let output = std::process::Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(project_root)
        .output()?;
    if !output.status.success() {
        anyhow::bail!("git worktree list failed: {}", String::from_utf8_lossy(&output.stderr).trim());
    }
    let mut current_path: Option<PathBuf> = None;
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if let Some(path) = line.strip_prefix("worktree ") { current_path = Some(PathBuf::from(path)); continue; }
        if line == format!("branch refs/heads/{main_branch}") {
            return current_path.ok_or_else(|| anyhow::anyhow!("invalid git worktree output"));
        }
    }
    anyhow::bail!("未找到主分支 worktree：{main_branch}");
}

fn cleanup_current_worktree(project_root: &Path, main_branch: &str) -> anyhow::Result<()> {
    let branch_name = git_current_branch(project_root).unwrap_or_else(|_| "detached".to_string());
    if branch_name == main_branch {
        println!("  ✓ 当前位于主分支 worktree，不执行自动清理");
        return Ok(());
    }
    println!("  → 清理当前任务 worktree：{} ({})", project_root.display(), branch_name);
    println!("    ✓ 跳过自动删除：当前仍在该 worktree 内运行 finish-work");
    Ok(())
}

fn auto_cleanup_worktree(project_root: &Path, main_branch: &str) -> anyhow::Result<()> {
    let output = std::process::Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(project_root)
        .output()?;
    if !output.status.success() { return Ok(()); }
    let main_path = std::fs::canonicalize(project_root).unwrap_or_else(|_| project_root.to_path_buf());
    let entries: Vec<(String, String)> = {
        let mut entries = Vec::new();
        let mut current_path = String::new();
        let mut current_branch = String::new();
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if let Some(path) = line.strip_prefix("worktree ") { current_path = path.to_string(); }
            else if let Some(br) = line.strip_prefix("branch refs/heads/") { current_branch = br.to_string(); }
            else if line.trim().is_empty() && !current_path.is_empty() {
                entries.push((current_path.clone(), current_branch.clone()));
                current_path.clear(); current_branch.clear();
            }
        }
        if !current_path.is_empty() { entries.push((current_path, current_branch)); }
        entries
    };
    let mut cleaned = false;
    for (path, branch_name) in &entries {
        if path.is_empty() { continue; }
        let canonical = std::fs::canonicalize(path.as_str()).unwrap_or_else(|_| PathBuf::from(path.as_str()));
        if canonical == main_path || branch_name == main_branch { continue; }
        println!("  → 清理 worktree：{} ({})", path, if branch_name.is_empty() { "detached" } else { branch_name });
        if std::process::Command::new("git").args(["worktree", "remove", path.as_str()])
            .current_dir(project_root).status().ok().map_or(false, |s| s.success())
        {
            cleaned = true;
            println!("    ✓ 已删除 worktree");
            if !branch_name.is_empty() {
                let _ = std::process::Command::new("git").args(["branch", "-d", branch_name.as_str()]).current_dir(project_root).status();
            }
        } else {
            if std::process::Command::new("git").args(["worktree", "remove", "--force", path.as_str()])
                .current_dir(project_root).status().ok().map_or(false, |s| s.success())
            {
                cleaned = true;
                println!("    ✓ 已强制删除 worktree");
                if !branch_name.is_empty() {
                    let _ = std::process::Command::new("git").args(["branch", "-D", branch_name.as_str()]).current_dir(project_root).status();
                }
            } else { eprintln!("    ⚠  删除失败，请手动处理"); }
        }
    }
    if !cleaned { println!("  ✓ 无可清理的 worktree"); }
    Ok(())
}

fn perform_finish_commit(project_root: &Path, task_name: &str, summary: Option<&str>, message: Option<&str>) -> anyhow::Result<Option<String>> {
    let dirty = git_dirty_entries(project_root)?;
    if dirty.is_empty() { return Ok(None); }
    run_git(project_root, &["add", "--all"])?;
    let commit_message = message.map(str::trim).filter(|v| !v.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| default_commit_message(project_root, task_name, summary));
    run_git(project_root, &["commit", "-m", &commit_message])?;
    let commit = run_git(project_root, &["rev-parse", "--short", "HEAD"])?;
    Ok(Some(commit))
}

fn perform_finish_integration(project_root: &Path, options: FinishWorkOptions<'_>, approved: bool) -> anyhow::Result<()> {
    let decision = dijiang_task::evaluate_capability(
        dijiang_task::WorkflowCapsule::Finish, dijiang_task::CapabilityTarget::FinishIntegrate, approved,
    );
    if options.integrate && matches!(decision.action, dijiang_task::CapabilityAction::Block) {
        anyhow::bail!("finish-work integration blocked: {}", decision.reason);
    }
    if options.push {
        let push_dec = dijiang_task::evaluate_capability(
            dijiang_task::WorkflowCapsule::Finish, dijiang_task::CapabilityTarget::FinishPush, approved,
        );
        if matches!(push_dec.action, dijiang_task::CapabilityAction::Block) {
            anyhow::bail!("finish-work push blocked: {}", push_dec.reason);
        }
    }
    let branch = git_current_branch(project_root)?;
    if branch.is_empty() { anyhow::bail!("finish-work 无法在 detached HEAD 上执行集成"); }
    if branch == options.main_branch { anyhow::bail!("finish-work 不在主分支上执行 --integrate"); }
    if options.push { run_git(project_root, &["push", "-u", options.remote, &branch])?; }
    if options.integrate {
        let main_worktree = git_main_worktree(project_root, options.main_branch)?;
        let cleanup_decision = dijiang_task::evaluate_capability(
            dijiang_task::WorkflowCapsule::Finish, dijiang_task::CapabilityTarget::FinishCleanup, options.approve_cleanup,
        );
        if matches!(cleanup_decision.action, dijiang_task::CapabilityAction::Block) {
            anyhow::bail!("finish-work cleanup blocked: {}; nextAction: {}", cleanup_decision.reason, cleanup_decision.next_action);
        }
        let project_root_str = project_root.display().to_string();
        run_git(&main_worktree, &["merge", "--no-ff", &branch])?;
        if options.push { run_git(&main_worktree, &["push", options.remote, options.main_branch])?; }
        run_git(&main_worktree, &["worktree", "remove", &project_root_str])?;
        run_git(&main_worktree, &["branch", "-d", &branch])?;
    }
    Ok(())
}

fn current_project_memory(dijiang_dir: &Path) -> anyhow::Result<dijiang_mem::ProjectMemory> {
    Ok(dijiang_mem::ProjectMemory::from_dijiang_dir(dijiang_dir)?)
}

pub fn cmd_finish_work(options: FinishWorkOptions<'_>) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let project_root = git_worktree_root(&cwd)?.unwrap_or(cwd);
    let local_dijiang_dir = project_root.join(".dijiang");
    let uses_local_dijiang_state = local_dijiang_dir.is_dir();
    let dijiang_dir = if uses_local_dijiang_state {
        local_dijiang_dir
    } else {
        find_dijiang_dir_in_git_worktrees(&project_root)?
            .ok_or_else(|| anyhow::anyhow!("未找到 .dijiang/ 目录。请先运行 `dijiang init`。"))?
    };
    let tasks_dir = dijiang_dir.join("tasks");
    let active_task = store::read_active_task(&dijiang_dir)?;
    let current_branch = git_current_branch(&project_root).ok();
    let worktree_hint = project_root.file_name().and_then(|v| v.to_str()).map(str::to_string);
    let resolved_target = resolve_finish_target(&tasks_dir, active_task.as_deref(), current_branch.as_deref(), worktree_hint.as_deref())?;
    let task_before_archive = resolved_target.as_ref().map(|target| &target.task);
    let (verification, docs_sync) = ensure_finish_preconditions(&project_root, task_before_archive, options)?;
    let version_update = update_workspace_version(&project_root, options.version_impact)?;
    let developer = read_developer(&dijiang_dir)?;
    let (session_key, source) = current_session_key();
    let task_label = resolved_target.as_ref().map(|t| t.task_name.as_str()).unwrap_or("no-active-task");
    let journal = append_finish_journal(&dijiang_dir, &developer, task_label, options.summary, &verification, options.allow_dirty)?;
    let archive_status = if let Some(target) = resolved_target.as_ref() {
        let task = store::archive_task(&tasks_dir, &target.task_name)?;
        store::clear_active_task(&dijiang_dir)?;
        format!("archived task `{}` (status: {}), journal: {}", target.task_name, task.status.as_str(), journal.display())
    } else { "skipped: no active task".to_string() };
    let session_journal = append_session_closure(&dijiang_dir, &developer, &session_key, &source, task_label, options.summary, &verification, options.allow_dirty)?;
    let project_memory = dijiang_mem::ProjectMemory::from_dijiang_dir(&dijiang_dir)?;
    let memory_closure = dijiang_mem::SessionClosure {
        timestamp: chrono::Utc::now().to_rfc3339(),
        session_key: session_key.clone(),
        source: source.clone(),
        task: task_label.to_string(),
        summary: options.summary.unwrap_or("Work finished and task archived.").to_string(),
        verification: verification.clone(),
        docs_sync: docs_sync.clone(),
        version_impact: options.version_impact.to_string(),
        status: "completed".to_string(),
        confidence: "medium".to_string(),
        outcome: None,
        next_tactic: None,
        next_pattern: None,
        loop_signal: None,
        attempts: vec![],
    };
    if options.commit { project_memory.append_session_closure(&memory_closure)?; }
    let commit = if options.commit { perform_finish_commit(&project_root, task_label, options.summary, options.commit_message)? } else { None };
    if options.commit && !options.integrate && !options.keep_worktree { cleanup_current_worktree(&project_root, options.main_branch)?; }
    if options.push || options.integrate { perform_finish_integration(&project_root, options, options.approve_integrate)?; }
    if !options.commit { project_memory.append_session_closure(&memory_closure)?; }
    if let Some(target) = resolved_target.as_ref() { println!("✓ 已完成任务 '{}'", target.task_name); } else { println!("✓ 已完成工作（无 active task）"); }
    println!("  验证：{verification}");
    println!("  版本更新：{}", options.version_impact);
    if let Ok(mem) = current_project_memory(&dijiang_dir) {
        if let Ok(findings) = mem.load_findings() {
            if !findings.is_empty() {
                println!("  💡 提示：运行 `dijiang mem recall --query \"<关键词>\"` 回顾");
            }
        }
    }
    if let Some(version_update) = version_update { println!("  版本更新：{version_update}"); }
    if let Some(commit) = commit { println!("  Commit：{commit}"); } else { println!("  Commit：none"); }
    println!("  Push：{}", if options.push { "done" } else { "skipped" });
    println!("  Integration：{}", if options.integrate { "done" } else { "skipped" });
    println!("  Task archive：{archive_status}");
    let memory_closure_path = project_memory.root().join("sessions.jsonl");
    println!("  Memory closure：written ({})", memory_closure_path.display());
    println!("  Session 已关闭：{}", session_journal.display());
    if resolved_target.is_some() {
        println!("  当前 session 的 active task 已清理");
    } else {
        println!("  当前 session 没有 active task 需要清理");
    }
    Ok(())
}
