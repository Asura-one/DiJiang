use crate::util::{require_dijiang_dir, run_git, read_developer, current_session_key, git_current_branch, git_worktree_root};
use dijiang_task::hooks::{self, HookEvent};
use dijiang_task::store;
use dijiang_task::types::{TaskRecord, TaskStatus};
use std::path::{Path, PathBuf};
use std::io::Write;
use serde_json::Value;
use std::collections::HashMap;

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

/// 检查字符串是否包含中文（CJK 统一表意文字）
fn has_chinese(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(c,
            '\u{4e00}'..='\u{9fff}' |
            '\u{3400}'..='\u{4dbf}' |
            '\u{f900}'..='\u{faff}' |
            '\u{2f800}'..='\u{2fa1f}'
        )
    })
}

fn default_commit_message(project_root: &Path, task_name: &str, summary: Option<&str>) -> String {
    let user_summary = summary.and_then(|s| {
        let t = s.trim();
        if t.is_empty() { None } else { Some(t.to_string()) }
    });

    let name_status = std::process::Command::new("git")
        .args(["diff", "--cached", "--name-status"])
        .current_dir(project_root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() { None } else { Some(s) }
        });

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
    let numstat = std::process::Command::new("git")
.args(["diff", "--cached", "--numstat"])
.current_dir(project_root)
.output()
.ok()
.filter(|o| o.status.success())
.map(|o| {
        let s = String::from_utf8_lossy(&o.stdout).to_string();
        let mut map: HashMap<String, (usize, usize)> = HashMap::new();
        for line in s.lines() {
            if line.trim().is_empty() { continue; }
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() == 3 {
                let added: usize = parts[0].parse().unwrap_or(0);
                let deleted: usize = parts[1].parse().unwrap_or(0);
                map.insert(parts[2].trim().to_string(), (added, deleted));
            }
        }
        map
    });

    let (mut added, mut modified, mut deleted) = (Vec::new(), Vec::new(), Vec::new());
    if let Some(ns) = &name_status {
        for line in ns.lines() {
            if line.trim().is_empty() { continue; }
            if let Some((status, path)) = line.split_once('\t') {
                let path = path.trim();
                match status.chars().next() {
                    Some('A') => added.push(path.to_string()),
                    Some('D') => deleted.push(path.to_string()),
                    _ => modified.push(path.to_string()),
                }
            }
        }
    }

    let total = added.len() + modified.len() + deleted.len();
    if total == 0 {
        return user_summary.unwrap_or_else(|| task_name.to_string());
    }

    // === 确定 type (Conventional Commits) ===
    let change_type = if let Some(s) = &user_summary {
        let lower = s.to_lowercase();
        if lower.contains("修复") || lower.contains("fix") { "fix" }
        else if lower.contains("feat") || lower.contains("新增") || lower.contains("添加") { "feat" }
        else if lower.contains("docs") || lower.contains("文档") { "docs" }
        else if lower.contains("refactor") || lower.contains("重构") { "refactor" }
        else if lower.contains("test") || lower.contains("测试") { "test" }
        else if lower.contains("perf") || lower.contains("性能") || lower.contains("优化") { "perf" }
        else if lower.contains("ci") || lower.starts_with("ci") { "ci" }
        else if lower.contains("style") || lower.starts_with("style") { "style" }
        else if lower.contains("chore") || lower.starts_with("chore") || lower.contains("配置") { "chore" }
        else { "refactor" }
    } else {
        // Pure file-based detection without user summary
        let is_test = |f: &&str| {
            f.contains("/tests/") || f.starts_with("tests/")
                || f.ends_with("_test.rs") || f.ends_with("_test.go")
                || f.ends_with(".spec.js") || f.ends_with(".spec.ts")
        };
        let is_doc = |f: &&str| f.ends_with(".md") || f.starts_with("docs/") || f.contains("/docs/");
        let is_ci = |f: &&str| {
            f.contains("/.github/") || f.contains("/.gitlab/") || f.starts_with(".github/")
                || f.ends_with("Dockerfile") || f.ends_with("Jenkinsfile")
        };
        let is_config = |f: &&str| {
            f.ends_with(".toml") || f.ends_with(".lock") || f.ends_with(".env")
                || f.ends_with(".editorconfig") || f.ends_with(".gitignore")
        };

        let all_files: Vec<&str> = added.iter()
.map(|s| s.as_str())
.chain(modified.iter().map(|s| s.as_str()))
.chain(deleted.iter().map(|s| s.as_str()))
.collect();
        // Test changes
        if all_files.iter().all(|f| is_test(f)) {
            "test"
        } else if all_files.iter().all(|f| is_doc(f)) {
            "docs"
        } else if all_files.iter().all(|f| is_ci(f)) {
            "ci"
        } else if all_files.iter().all(|f| is_config(f)) {
            "chore"
        } else if !added.is_empty() && total == added.len() {
            "feat"
        } else {
            "refactor"
        }
    };

    // === 确定 scope (最常见的 crate/模块前缀) ===
    let all_files: Vec<&str> = added.iter()
        .chain(modified.iter())
        .chain(deleted.iter())
        .map(|s| s.as_str())
        .collect();
    let scope = detect_scope(&all_files);

    // === 描述 ===
    let description = if let Some(s) = user_summary {
        s
    } else {
        generate_description(task_name, &added, &modified, &deleted)
    };

    // === 标题行 ===
    let title = format!("{}: {}", change_type, description);

    // === 正文：按目录分组 + 统计行 ===
    let body = build_body(&added, &modified, &deleted, &diff_stat, &numstat);

    format!("{}\n\n{}", title, body)
}

fn detect_scope(files: &[&str]) -> Option<String> {
    // 只考虑"真正"的文件：跳过内部文件和根级配置文件
    let significant: Vec<&str> = files.iter()
.filter(|f| !f.starts_with(".dijiang/") && !f.starts_with(".pi/"))
.filter(|f| {
            let name = f.split('/').last().unwrap_or("");
            !matches!(name, "Cargo.toml" | "Cargo.lock" | "package.json" | "package-lock.json"
)
        })
.copied()
.collect();

    if significant.is_empty() { return None; }

    // 统计 crate 级目录（优先）和顶层目录频率
    let mut top_counts: HashMap<&str, usize> = HashMap::new();
    let mut crate_counts: HashMap<String, usize> = HashMap::new();

    for path in &significant {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() < 2 { continue; }

        // 收集顶层目录
        *top_counts.entry(parts[0]).or_insert(0) += 1;

        // crates/PKG/... → 使用 PKG 作为 scope
        if parts[0] == "crates" && parts.len() >= 3 {
            *crate_counts.entry(parts[1].to_string()).or_insert(0) += 1;
        }
        // packages/NAME/... → 使用 NAME
        if parts[0] == "packages" && parts.len() >= 3 {
            *crate_counts.entry(parts[1].to_string()).or_insert(0) += 1;
        }
    }

    let n = significant.len();

    // 优先使用 crate 名（当 crate 级目录有明确多数时）
    if let Some((scope, count)) = crate_counts.into_iter().max_by_key(|(_, c)| *c) {
        if count * 2 >= n {
            return Some(scope);
        }
    }

    // 回退到顶层目录（需要超过 50% 才能确定 scope）
    top_counts.into_iter()
.filter(|(_, count)| *count * 2 > n)  // 严格 >50%
.max_by_key(|(_, count)| *count)
.map(|(dir, _)| dir.to_string())
}

fn generate_description(_task_name: &str, added: &[String], modified: &[String], deleted: &[String]) -> String {
    let total = added.len() + modified.len() + deleted.len();
    if total == 0 {
        return _task_name.to_string();
    }

    let is_internal = |s: &str| s.starts_with(".dijiang/") || s.starts_with(".pi/");

    let fmt_paths = |files: &[String], max: usize| -> Option<String> {
        let real: Vec<&str> = files.iter()
.filter(|f| !is_internal(f.as_str()))
.map(|s| s.as_str())
.collect();
        if real.is_empty() { return None; }
        let shown: Vec<&str> = real.iter().take(max).map(|s| *s).collect();
        let joined = shown.join("、");
        let suffix = if real.len() > max { format!(" 等 {} 个", real.len()) } else { String::new() };
        Some(format!("{}{}", joined, suffix))
    };

    let mut parts = Vec::new();
    if !added.is_empty() {
        if let Some(paths) = fmt_paths(added, 3) {
            parts.push(format!("添加 {}", paths));
        }
    }
    if !modified.is_empty() {
        if let Some(paths) = fmt_paths(modified, 3) {
            parts.push(format!("修改 {}", paths));
        }
    }
    if !deleted.is_empty() {
        if let Some(paths) = fmt_paths(deleted, 3) {
            parts.push(format!("删除 {}", paths));
        }
    }
    if parts.is_empty() {
        return _task_name.to_string();
    }
    parts.join("，")
}

fn build_body(
    added: &[String],
    modified: &[String],
    deleted: &[String],
    _diff_stat: &Option<String>,
    numstat: &Option<HashMap<String, (usize, usize)>>,
) -> String {
    let is_internal = |s: &&str| s.starts_with(".dijiang/") || s.starts_with(".pi/");

    let fmt_stats = |path: &&str| -> String {
        numstat.as_ref()
.and_then(|ns| ns.get(*path))
.map(|(add, del)| {
            if *del > 0 && *add > 0 {
                format!("（+{} 行，-{} 行）", add, del)
            } else if *del > 0 {
                format!("（-{} 行）", del)
            } else {
                format!("（+{} 行）", add)
            }
        })
.unwrap_or_default()
    };

    let mut body = String::new();

    let filtered_added: Vec<&str> = added.iter().map(|s| s.as_str()).filter(|s| !is_internal(s)).collect();
    let filtered_modified: Vec<&str> = modified.iter().map(|s| s.as_str()).filter(|s| !is_internal(s)).collect();
    let filtered_deleted: Vec<&str> = deleted.iter().map(|s| s.as_str()).filter(|s| !is_internal(s)).collect();

    let total = filtered_added.len() + filtered_modified.len() + filtered_deleted.len();
    if total == 0 {
        return String::new();
    }

    body.push_str(&format!("变更 {} 个文件：\n\n", total));

    let push_files = |body: &mut String, files: &[&str], prefix: &str| {
        for path in files {
            body.push_str(&format!("- {} {}{}\n", prefix, path, fmt_stats(path)));
        }
    };

    if !filtered_added.is_empty() {
        push_files(&mut body, &filtered_added, "新增");
    }
    if !filtered_modified.is_empty() {
        if !filtered_added.is_empty() { body.push('\n'); }
        push_files(&mut body, &filtered_modified, "修改");
    }
    if !filtered_deleted.is_empty() {
        if !filtered_added.is_empty() || !filtered_modified.is_empty() { body.push('\n'); }
        push_files(&mut body, &filtered_deleted, "删除");
    }

    body.trim().to_string()
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
    if !has_chinese(&commit_message) {
        anyhow::bail!("commit message 不含中文字符，已拒绝：{}

所有 commit message 必须使用中文编写，描述实际变更内容。", commit_message);
    }
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
    let developer = dijiang_task::developer::resolve_developer(&dijiang_dir);
    let (session_key, source) = current_session_key();
    let task_label = resolved_target.as_ref().map(|t| t.task_name.as_str()).unwrap_or("no-active-task");
    let journal = append_finish_journal(&dijiang_dir, &developer, task_label, options.summary, &verification, options.allow_dirty)?;
    let archive_status = if let Some(target) = resolved_target.as_ref() {
        let task = store::archive_task(&tasks_dir, &target.task_name)?;
        hooks::run_task_hooks(&dijiang_dir, HookEvent::AfterTaskFinish, &target.task_name);
        hooks::run_task_hooks(&dijiang_dir, HookEvent::AfterTaskArchive, &target.task_name);
        store::clear_active_task(&dijiang_dir)?;
        format!("archived task `{}` (status: {}), journal: {}", target.task_name, task.status.as_str(), journal.display())
    } else { "skipped: no active task".to_string() };

    // Auto-archive orphan planning tasks that were created but never started.
    // These are tasks in planning state with no started_at —
    // they were abandoned when another task became active.
    let orphan_archived = if resolved_target.is_some() {
        if let Ok(all_tasks) = store::list_tasks(&tasks_dir) {
            let mut count = 0usize;
            for task in &all_tasks {
                if task.status == TaskStatus::Planning
                    && task.started_at.is_none()
                    && Some(&task.name) != resolved_target.as_ref().map(|t| &t.task_name)
                {
                    store::archive_task(&tasks_dir, &task.name)?;
                    count += 1;
                }
            }
            count
        } else { 0 }
    } else { 0 };
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
    if orphan_archived > 0 {
        println!("  Orphan tasks archived：{orphan_archived}（从未开始的 planning 任务）");
    }
    println!("  Session 已关闭：{}", session_journal.display());
    if resolved_target.is_some() {
        println!("  当前 session 的 active task 已清理");
    } else {
        println!("  当前 session 没有 active task 需要清理");
    }
    // #16: Auto-commit workspace journal (best-effort)
    let _ = auto_commit_journal(&project_root, task_label);
    Ok(())
}

/// Auto-commit workspace journal changes (best-effort).
fn auto_commit_journal(project_root: &Path, task_label: &str) -> anyhow::Result<()> {
    let workspace_dir = project_root.join(".dijiang").join("workspace");
    if !workspace_dir.exists() {
        return Ok(());
    }
    let status = run_git(project_root, &["status", "--porcelain", ".dijiang/workspace/"]);
    match status {
        Ok(stdout) if stdout.trim().is_empty() => return Ok(()),
        Err(_) => return Ok(()), // not a git repo or git not available
        _ => {}
    }
    run_git(project_root, &["add", ".dijiang/workspace/"])?;
    let msg = format!("journal: {}", task_label);
    let _ = run_git(project_root, &["commit", "-m", &msg]);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_chinese_with_pure_english() {
        assert!(!has_chinese("test: this is an english message"));
        assert!(!has_chinese("fix: update skill name"));
        assert!(!has_chinese("chore: sync code"));
    }

    #[test]
    fn test_has_chinese_with_pure_chinese() {
        assert!(has_chinese("feat(test): 测试中文 commit message"));
        assert!(has_chinese("fix(cli): 修复登录 session 过期问题"));
        assert!(has_chinese("全部是中文"));
    }

    #[test]
    fn test_has_chinese_with_mixed() {
        assert!(has_chinese("fix(test): 修复 bug with english"));
        assert!(has_chinese("feat(cli): 添加 utf-8 support"));
    }

    #[test]
    fn test_has_chinese_with_empty_and_special() {
        assert!(!has_chinese(""));
        assert!(!has_chinese("12345"));
        assert!(!has_chinese("!@#$%"));
        assert!(!has_chinese("test: "));
    }

    #[test]
    fn test_has_chinese_with_english_only_commit() {
        // 这是历史上出现过的真实英文 commit
        assert!(!has_chinese("task-20260703155147"));
        assert!(!has_chinese("fix(extension): live refresh status bar and widget on session events"));
        // 这也是历史上出现过的真实英文 commit（不含中文）
        assert!(!has_chinese("task-20260703155147"));
        assert!(!has_chinese("fix(extension): live refresh status bar and widget on session events"));
    }
}
