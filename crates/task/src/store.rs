use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::route_gate::{RouteAction, RouteDecision, WorkflowCapsule};
use crate::types::{TaskRecord, TaskStatus};
use serde::{Deserialize, Serialize};
/// Error type for task operations.
#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Task not found: {0}")]
    NotFound(String),

    #[error("Invalid status: {0}")]
    InvalidStatus(String),

    #[error("Invalid transition: {from} → {to}")]
    InvalidTransition { from: TaskStatus, to: TaskStatus },

    #[error("Task is not eligible for archive: {0}")]
    ArchiveIneligible(String),

    #[error("Invalid context action: {0}")]
    InvalidContextAction(String),

    #[error("Invalid context path: {0}")]
    InvalidContextPath(String),

    #[error("Invalid task reference: {0}")]
    InvalidTaskReference(String),
}

fn validate_task_reference(task_name: &str) -> Result<&str, TaskError> {
    let trimmed = task_name.trim();
    let mut components = Path::new(trimmed).components();
    if task_name != trimmed
        || trimmed.is_empty()
        || components.next().is_none()
        || components.next().is_some()
        || trimmed == "."
        || trimmed == ".."
        || trimmed.contains(['/', '\\'])
    {
        return Err(TaskError::InvalidTaskReference(task_name.to_string()));
    }
    Ok(trimmed)
}

/// Find the `.dijiang/` directory by walking up from `cwd`.
/// Falls back to `.trellis/` for backward compatibility with existing projects.
pub fn find_dijiang_dir(cwd: &Path) -> Option<PathBuf> {
    let mut dir = Some(cwd.to_path_buf());
    while let Some(d) = dir {
        // Prefer `.dijiang/`
        let candidate = d.join(".dijiang");
        if candidate.is_dir() {
            return Some(candidate);
        }
        // Fallback: `.trellis/` (legacy)
        let legacy = d.join(".trellis");
        if legacy.is_dir() {
            return Some(legacy);
        }
        dir = d.parent().map(|p| p.to_path_buf());
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionIdentity {
    key: String,
    source: String,
}

impl SessionIdentity {
    pub fn new(source: impl Into<String>, value: impl AsRef<str>) -> Option<Self> {
        let source = source.into();
        let source_key = sanitize_session_key(&source);
        let value_key = sanitize_session_key(value.as_ref());
        if source_key.is_empty() || value_key.is_empty() {
            return None;
        }
        Some(Self {
            key: format!("{source_key}_{value_key}"),
            source,
        })
    }

    pub fn explicit(source: impl Into<String>, key: impl AsRef<str>) -> Option<Self> {
        let source = source.into();
        let key = sanitize_session_key(key.as_ref());
        if source.trim().is_empty() || key.is_empty() {
            return None;
        }
        Some(Self { key, source })
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn source(&self) -> &str {
        &self.source
    }
}

fn sanitize_session_key(raw: &str) -> String {
    let mut safe = String::with_capacity(raw.len().min(160));
    for ch in raw.trim().chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
            safe.push(ch);
        } else if !safe.ends_with('_') {
            safe.push('_');
        }
        if safe.len() >= 160 {
            break;
        }
    }
    safe.trim_matches(&['.', '_', '-'][..]).to_string()
}

pub fn current_session_identity() -> Option<SessionIdentity> {
    if let Ok(value) = env::var("DIJIANG_CONTEXT_ID") {
        if let Some(identity) = SessionIdentity::explicit("dijiang", value) {
            return Some(identity);
        }
    }

    const ENV_KEYS: &[(&str, &[&str])] = &[
        ("dijiang", &["DIJIANG_SESSION_ID"]),
        ("codex", &["CODEX_SESSION_ID", "CODEX_THREAD_ID"]),
        ("claude", &["CLAUDE_SESSION_ID", "CLAUDE_CODE_SESSION_ID"]),
        ("pi", &["PI_SESSION_ID", "PI_SESSIONID"]),
        ("cursor", &["CURSOR_SESSION_ID", "CURSOR_CONVERSATION_ID"]),
        ("opencode", &["OPENCODE_SESSION_ID", "OPENCODE_RUN_ID"]),
        ("hermes", &["HERMES_SESSION_ID"]),
    ];

    for (source, keys) in ENV_KEYS {
        for key in *keys {
            if let Ok(value) = env::var(key) {
                if let Some(identity) = SessionIdentity::new(*source, value) {
                    return Some(identity);
                }
            }
        }
    }
    None
}

fn global_session_identity() -> SessionIdentity {
    SessionIdentity::new("global", "global").expect("literal global session key is valid")
}

fn session_path(dijiang_dir: &Path, identity: &SessionIdentity) -> PathBuf {
    dijiang_dir
        .join(".runtime")
        .join("sessions")
        .join(format!("{}.json", identity.key()))
}

fn read_session_task(
    dijiang_dir: &Path,
    identity: &SessionIdentity,
) -> Result<Option<String>, TaskError> {
    let path = session_path(dijiang_dir, identity);
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path)?;
    let data: serde_json::Value = serde_json::from_str(&content)?;
    let task = data
        .get("current_task")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());
    task.map(validate_task_reference)
        .transpose()
        .map(|task| task.map(str::to_string))
}

/// Find the active task for the current session, using global state only when no session identity exists.
pub fn read_active_task(dijiang_dir: &Path) -> Result<Option<String>, TaskError> {
    match current_session_identity() {
        Some(identity) => read_active_task_for_session(dijiang_dir, Some(&identity)),
        None => read_global_active_task(dijiang_dir),
    }
}

fn read_global_active_task(dijiang_dir: &Path) -> Result<Option<String>, TaskError> {
    let global_identity = global_session_identity();
    if let Some(task) = read_session_task(dijiang_dir, &global_identity)? {
        return Ok(Some(task));
    }

    let path = dijiang_dir.join("active_task.txt");
    if path.exists() {
        let content = fs::read_to_string(&path)?;
        let active = content.trim();
        if !active.is_empty() {
            return Ok(Some(validate_task_reference(active)?.to_string()));
        }
    }

    Ok(None)
}

pub fn read_active_task_for_session(
    dijiang_dir: &Path,
    identity: Option<&SessionIdentity>,
) -> Result<Option<String>, TaskError> {
    if let Some(identity) = identity {
        if let Some(task) = read_session_task(dijiang_dir, identity)? {
            return Ok(Some(task));
        }
        if let Some(task) = read_global_active_task(dijiang_dir)? {
            write_active_task_for_session(dijiang_dir, &task, Some(identity))?;
            return Ok(Some(task));
        }
        return Ok(None);
    }

    Ok(None)
}

/// Write the active task for the current session, using a global pointer only without session identity.
pub fn write_active_task(dijiang_dir: &Path, task_name: &str) -> Result<(), TaskError> {
    write_active_task_for_session(dijiang_dir, task_name, current_session_identity().as_ref())
}

pub fn write_active_task_for_session(
    dijiang_dir: &Path,
    task_name: &str,
    identity: Option<&SessionIdentity>,
) -> Result<(), TaskError> {
    let task_name = validate_task_reference(task_name)?;
    if identity.is_none() {
        fs::write(dijiang_dir.join("active_task.txt"), task_name)?;
    }
    let runtime_dir = dijiang_dir.join(".runtime");
    let sessions_dir = runtime_dir.join("sessions");
    fs::create_dir_all(&sessions_dir)?;

    let fallback_identity;
    let identity = match identity {
        Some(identity) => identity,
        None => {
            fallback_identity = global_session_identity();
            &fallback_identity
        }
    };

    let path = session_path(dijiang_dir, identity);
    let mut session = if path.exists() {
        let content = fs::read_to_string(&path)?;
        serde_json::from_str::<serde_json::Value>(&content)
            .unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    session["current_task"] = serde_json::json!(task_name);
    session["last_seen_at"] = serde_json::json!(chrono::Utc::now().to_rfc3339());
    session["session_key"] = serde_json::json!(identity.key());
    session["source"] = serde_json::json!(identity.source());
    if session.get("closed_task").is_some() {
        session["closed_task"] = serde_json::Value::Null;
    }
    fs::write(path, serde_json::to_string_pretty(&session)?)?;
    fs::write(runtime_dir.join(".dijiang_owned"), "")?;

    Ok(())
}

pub fn clear_active_task(dijiang_dir: &Path) -> Result<(), TaskError> {
    clear_active_task_for_session(dijiang_dir, current_session_identity().as_ref())
}

/// Save a task and set it as the active task.
/// Convenience wrapper around `save_task + write_active_task`.
pub fn activate_new_task(dijiang_dir: &Path, task: &TaskRecord) -> Result<(), TaskError> {
    let tasks_dir = dijiang_dir.join("tasks");
    save_task(&tasks_dir, task)?;
    write_active_task(dijiang_dir, &task.name)?;
    if let Err(e) = update_workspace_index(dijiang_dir, task) {
        eprintln!("Warning: failed to update workspace index: {}", e);
    }
    Ok(())
}

fn update_workspace_index(dijiang_dir: &Path, task: &TaskRecord) -> Result<(), TaskError> {
    let index_path = dijiang_dir.join("workspace").join("index.md");
    if !index_path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(&index_path)?;

    // Update Active Developers table
    let dev_name = if task.creator.is_empty() {
        "none"
    } else {
        &task.creator
    };
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let dev_row = format!("| {} | {} | - | - |\n", dev_name, today);

    let mut new_content = String::new();
    let mut dev_updated = false;

    for line in content.lines() {
        if !dev_updated && line.starts_with("| (none yet)") {
            new_content.push_str(&dev_row);
            dev_updated = true;
            continue;
        }
        if !dev_updated
            && line.starts_with("| ")
            && line.contains("|")
            && !line.contains("---")
            && !line.contains("Developer")
        {
            new_content.push_str(&dev_row);
            dev_updated = true;
            continue;
        }
        new_content.push_str(line);
        new_content.push('\n');
    }

    // Append Task History section if not present
    if !new_content.contains("## Task History") {
        new_content.push_str("\n## Task History\n\n");
        new_content.push_str("| Date Created | Task | Title | Status | Creator |\n");
        new_content.push_str("|---|---|---|---|---|\n");
    }

    // Append task entry
    let entry = format!(
        "| {} | `{}` | {} | {} | {} |\n",
        task.created_at,
        task.name,
        task.title,
        task.status.as_str(),
        dev_name
    );

    if let Some(pos) = new_content.rfind("## Task History") {
        let after_header = &new_content[pos..];
        if let Some(table_end) = after_header.find("\n## ") {
            let insert_at = pos + table_end;
            new_content.insert_str(insert_at, &entry);
        } else {
            new_content.push_str(&entry);
        }
    } else {
        new_content.push_str(&entry);
    }

    fs::write(&index_path, new_content)?;
    Ok(())
}

pub fn clear_active_task_for_session(
    dijiang_dir: &Path,
    identity: Option<&SessionIdentity>,
) -> Result<(), TaskError> {
    // 1. Clear the identity-specific session file
    if let Some(identity) = identity {
        let path = session_path(dijiang_dir, identity);
        if path.exists() {
            fs::remove_file(path)?;
        }
    } else {
        let path = session_path(dijiang_dir, &global_session_identity());
        if path.exists() {
            fs::remove_file(path)?;
        }
    }
    // 2. Clear legacy active_task.txt
    let active_path = dijiang_dir.join("active_task.txt");
    if active_path.exists() {
        fs::remove_file(active_path)?;
    }
    // 3. BUGFIX: Also clear current_task from the global session file,
    //    so a new session doesn't resurrect it from the global fallback.
    let global_path = session_path(dijiang_dir, &global_session_identity());
    if global_path.exists() {
        let content = fs::read_to_string(&global_path)?;
        if let Ok(mut session) = serde_json::from_str::<serde_json::Value>(&content) {
            if session.get("current_task").is_some()
                && session["current_task"].is_string()
                && !session["current_task"].as_str().unwrap_or("").is_empty()
            {
                session["last_active_task"] = session["current_task"].clone();
                session["current_task"] = serde_json::Value::Null;
                fs::write(&global_path, serde_json::to_string_pretty(&session)?)?;
            }
        }
    }
    Ok(())
}

/// Load a single task from its task.json file.
pub fn load_task(tasks_dir: &Path, task_name: &str) -> Result<TaskRecord, TaskError> {
    let task_name = validate_task_reference(task_name)?;
    let task_dir = tasks_dir.join(task_name);
    if !task_dir.exists() {
        return Err(TaskError::NotFound(task_name.to_string()));
    }
    let tasks_root = fs::canonicalize(tasks_dir)?;
    let canonical_task_dir = fs::canonicalize(&task_dir)?;
    if canonical_task_dir.parent() != Some(tasks_root.as_path()) {
        return Err(TaskError::InvalidTaskReference(task_name.to_string()));
    }
    let content = fs::read_to_string(canonical_task_dir.join("task.json"))?;
    let record = serde_json::from_str(&content)?;
    Ok(record)
}

/// Save a task record to its task.json file.
pub fn save_task(tasks_dir: &Path, task: &TaskRecord) -> Result<(), TaskError> {
    let task_name = validate_task_reference(&task.name)?;
    let task_dir = tasks_dir.join(task_name);
    fs::create_dir_all(&task_dir)?;
    let tasks_root = fs::canonicalize(tasks_dir)?;
    let canonical_task_dir = fs::canonicalize(&task_dir)?;
    if canonical_task_dir.parent() != Some(tasks_root.as_path()) {
        return Err(TaskError::InvalidTaskReference(task_name.to_string()));
    }
    let path = task_dir.join("task.json");
    let content = serde_json::to_string_pretty(task)?;
    fs::write(&path, content)?;
    scaffold_task_docs(tasks_dir, &task.name, &task.title)?;
    Ok(())
}

/// List all tasks in the tasks directory.
pub fn list_tasks(tasks_dir: &Path) -> Result<Vec<TaskRecord>, TaskError> {
    let mut tasks = Vec::new();
    if !tasks_dir.exists() {
        return Ok(tasks);
    }

    for entry in fs::read_dir(tasks_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let task_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        match load_task(tasks_dir, &task_name) {
            Ok(task) => tasks.push(task),
            Err(_) => continue, // skip malformed
        }
    }

    // Sort by name
    tasks.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(tasks)
}

/// Update the status of a task.
pub fn update_status(
    tasks_dir: &Path,
    task_name: &str,
    new_status: TaskStatus,
) -> Result<TaskRecord, TaskError> {
    let mut task = load_task(tasks_dir, task_name)?;
    let old_status = task.status.clone();
    if old_status == new_status {
        // Same status — no-op, return as-is
        return Ok(task);
    }
    if !old_status.is_valid_transition(&new_status) {
        return Err(TaskError::InvalidTransition {
            from: old_status,
            to: new_status,
        });
    }
    task.status = new_status;
    save_task(tasks_dir, &task)?;
    Ok(task)
}

/// Archive a task: set status to Archived and record archived_at timestamp.
pub fn archive_task(tasks_dir: &Path, task_name: &str) -> Result<TaskRecord, TaskError> {
    let mut task = load_task(tasks_dir, task_name)?;
    // Planning → Archived is an explicit abandonment path. A completed task
    // must use the same checklist gate as finish-work before it can close.
    if task.status == TaskStatus::Completed {
        ensure_finish_eligible(tasks_dir, task_name).map_err(TaskError::ArchiveIneligible)?;
    }
    let old_status = task.status.clone();
    if !old_status.is_valid_transition(&TaskStatus::Archived) {
        return Err(TaskError::InvalidTransition {
            from: old_status,
            to: TaskStatus::Archived,
        });
    }
    task.status = TaskStatus::Archived;
    task.archived_at = Some(chrono::Utc::now().format("%Y-%m-%d").to_string());
    save_task(tasks_dir, &task)?;
    Ok(task)
}

/// Prune old archived tasks: remove task directories whose `archived_at`
/// timestamp is older than `older_than_days` days. Returns count of pruned tasks.
pub fn prune_tasks(tasks_dir: &Path, older_than_days: u64) -> Result<usize, TaskError> {
    let now = chrono::Utc::now();
    let cutoff = chrono::Duration::days(older_than_days as i64);
    let mut pruned = 0;

    if !tasks_dir.exists() {
        return Ok(0);
    }

    for entry in fs::read_dir(tasks_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let task_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Load task to check archived_at
        if let Ok(task) = load_task(tasks_dir, &task_name) {
            if task.status != TaskStatus::Archived {
                continue;
            }
            if let Some(archived_at) = &task.archived_at {
                if let Ok(archived_date) =
                    chrono::NaiveDate::parse_from_str(archived_at, "%Y-%m-%d")
                        .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
                        .map(|dt| dt.and_utc())
                {
                    let age = now - archived_date;
                    if age > cutoff {
                        // Remove the entire task directory
                        let _ = fs::remove_dir_all(&path);
                        pruned += 1;
                    }
                }
            }
        }
    }

    Ok(pruned)
}

// ── Task hierarchy (parent/child) ────────────────────────────────

/// Link a child task under a parent task. Updates both task.json records:
/// sets `parent` on the child and appends the child's id to the parent's `children` list.
pub fn link_tasks(tasks_dir: &Path, parent_name: &str, child_name: &str) -> Result<(), TaskError> {
    let mut parent = load_task(tasks_dir, parent_name)?;
    let mut child = load_task(tasks_dir, child_name)?;

    if parent_name == child_name {
        return Err(TaskError::InvalidStatus(
            "parent and child must be different tasks".into(),
        ));
    }

    // Set parent on child
    child.parent = Some(parent.id.clone());
    if !parent.children.contains(&child.id) {
        parent.children.push(child.id.clone());
    }

    save_task(tasks_dir, &parent)?;
    save_task(tasks_dir, &child)?;
    Ok(())
}

/// Unlink a child from its parent. Clears the child's `parent` field and
/// removes the child's id from the parent's `children` list.
pub fn unlink_task(tasks_dir: &Path, child_name: &str) -> Result<(), TaskError> {
    let mut child = load_task(tasks_dir, child_name)?;
    let parent_id = match child.parent.take() {
        Some(id) => id,
        None => return Ok(()),
    };

    // Find the parent task by id
    let tasks = list_tasks(tasks_dir)?;
    if let Some(parent_task) = tasks
        .iter()
        .find(|t| t.id == parent_id)
        .map(|t| t.name.clone())
    {
        let mut parent = load_task(tasks_dir, &parent_task)?;
        parent.children.retain(|c| c != &child.id);
        save_task(tasks_dir, &parent)?;
    }

    save_task(tasks_dir, &child)?;
    Ok(())
}
/// Create a new task record with default values.
pub fn create_task(name: &str, title: &str) -> TaskRecord {
    let now = chrono::Utc::now();
    TaskRecord {
        // ── Identity ──
        id: name.to_string(),
        name: name.to_string(),
        title: title.to_string(),
        description: String::new(),
        status: TaskStatus::Planning,
        dev_type: None,
        scope: None,
        package: None,
        priority: "P2".to_string(),
        // ── People ──
        creator: String::new(),
        assignee: String::new(),
        // ── Timestamps ──
        created_at: now.format("%Y-%m-%d").to_string(),
        completed_at: None,
        branch: None,
        base_branch: None,
        worktree_path: None,
        commit: None,
        pr_url: None,
        // ── Relations ──
        subtasks: Vec::new(),
        children: Vec::new(),
        parent: None,
        related_files: Vec::new(),
        depends_on: None,
        // ── Metadata ──
        notes: String::new(),
        meta: serde_json::Value::Null,
        // ── DiJiang extensions ──
        started_at: None,
        archived_at: None,
        acceptance_criteria: None,
        key_deliverables: None,
        source: None,
        session_id: None,
        session_summary: None,
        estimated_effort: None,
        actual_effort: None,
        review_status: None,
        review_comments: None,
        tags: None,
        hooks: None,
    }
}

// ── Task document scaffolding ────────────────────────────────────

const PRD_TEMPLATE: &str = "# {title}\n\n## Goal\n\nTBD.\n\n## Requirements\n\n- TBD\n\n## Acceptance Criteria\n\n- [ ] TBD\n\n## Notes\n\n- Keep `prd.md` focused on requirements, constraints, and acceptance criteria.\n";

const DESIGN_TEMPLATE: &str = "# {title} — Technical Design\n\n## Background\n\n<why this design is needed>\n\n## Solution\n\n<design decisions and trade-offs>\n\n## Impact Scope\n\n<affected modules/interfaces/data models>\n";

const IMPLEMENT_TEMPLATE: &str =
    "# {title} — Implementation Plan\n\n## Steps\n\n- [ ] TBD\n\n## Verification\n\n- [ ] TBD\n";

/// Create scaffolding documentation (prd.md, design.md, implement.md)
/// for a task, if they don't already exist.
pub fn scaffold_task_docs(tasks_dir: &Path, task_name: &str, title: &str) -> Result<(), TaskError> {
    let task_name = validate_task_reference(task_name)?;
    let task_dir = tasks_dir.join(task_name);
    fs::create_dir_all(&task_dir)?;
    let tasks_root = fs::canonicalize(tasks_dir)?;
    let canonical_task_dir = fs::canonicalize(&task_dir)?;
    if canonical_task_dir.parent() != Some(tasks_root.as_path()) {
        return Err(TaskError::InvalidTaskReference(task_name.to_string()));
    }
    let research_dir = task_dir.join("research");
    fs::create_dir_all(&research_dir)?;

    let prd_path = task_dir.join("prd.md");
    if !prd_path.exists() {
        let content = PRD_TEMPLATE.replace("{title}", title);
        fs::write(&prd_path, content)?;
    }

    let design_path = task_dir.join("design.md");
    if !design_path.exists() {
        let content = DESIGN_TEMPLATE.replace("{title}", title);
        fs::write(&design_path, content)?;
    }

    let implement_path = task_dir.join("implement.md");
    if !implement_path.exists() {
        let content = IMPLEMENT_TEMPLATE.replace("{title}", title);
        fs::write(&implement_path, content)?;
    }

    Ok(())
}
// ── Context manifest (JSONL) ────────────────────────────────────────

// Context manifest management moved to `crate::context`.
// Re-exports for backward compatibility.
pub use crate::context::{ContextEntry, add_context_entry, list_context_entries};

// ── Lifecycle Hooks ────────────────────────────────────────────────

/// Get hooks for a task by name. Returns None if no hooks are configured.
pub fn get_task_hooks(
    tasks_dir: &Path,
    task_name: &str,
) -> Result<Option<HashMap<String, Vec<String>>>, TaskError> {
    let task = load_task(tasks_dir, task_name)?;
    Ok(task.hooks)
}

/// Set hooks for a task. The existing hooks are replaced entirely.
pub fn set_task_hooks(
    tasks_dir: &Path,
    task_name: &str,
    hooks: HashMap<String, Vec<String>>,
) -> Result<(), TaskError> {
    let mut task = load_task(tasks_dir, task_name)?;
    task.hooks = Some(hooks);
    save_task(tasks_dir, &task)
}

/// Run hooks for a given event on a task. Each hook command is executed
/// via the shell (sh -c). Non-zero exit codes are logged to stderr but
/// do not abort remaining hooks or return an error.
pub fn run_task_hooks(tasks_dir: &Path, task_name: &str, event: &str) -> Result<(), TaskError> {
    let hooks = match get_task_hooks(tasks_dir, task_name)? {
        Some(h) => h,
        None => return Ok(()),
    };
    let cmds = match hooks.get(event) {
        Some(c) => c.clone(),
        None => return Ok(()),
    };
    for cmd in &cmds {
        eprintln!("⚡ Hook [{}] {}: {}", task_name, event, cmd);
        let output = std::process::Command::new("sh").arg("-c").arg(cmd).output();
        match output {
            Ok(out) => {
                if !out.status.success() {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    eprintln!("⚠ Hook failed [{}]: {}", task_name, stderr.trim());
                }
            }
            Err(e) => {
                eprintln!("⚠ Hook error [{}]: {}", task_name, e);
            }
        }
    }
    Ok(())
}
/// Verifies that a task has a substantive PRD and that `.dijiang/spec/` contains
/// at least one directory before allowing implementation skills.
pub fn apply_readiness_gate(
    dijiang_dir: &Path,
    tasks_dir: &Path,
    task_name: &str,
    decision: &RouteDecision,
) -> RouteDecision {
    if decision.action != RouteAction::Allow {
        return decision.clone();
    }
    let requires_readiness = matches!(
        decision.requested_intent,
        crate::route_gate::RouteIntent::Implement
            | crate::route_gate::RouteIntent::Debug
            | crate::route_gate::RouteIntent::Check
    );
    if !requires_readiness {
        return decision.clone();
    }
    let spec_dir = dijiang_dir.join("spec");
    let has_specs = spec_dir.is_dir()
        && std::fs::read_dir(&spec_dir)
            .map(|entries| entries.flatten().any(|entry| entry.path().is_dir()))
            .unwrap_or(false);
    if !has_specs {
        return readiness_redirect(
            decision,
            "dj-spec-bootstrap",
            "spec directory (.dijiang/spec/) is missing or empty -- run dj-spec-bootstrap first",
            "run dj-spec-bootstrap to initialize project specs",
        );
    }
    if !has_task_or_parent_prd(tasks_dir, task_name) {
        return readiness_redirect(
            decision,
            "dj-output",
            "task PRD is missing or incomplete -- complete its goal, requirements, and acceptance criteria first",
            "run dj-output to produce a substantive prd.md before implementation",
        );
    }
    if has_confirmed_grilling(tasks_dir, task_name) {
        return decision.clone();
    }
    readiness_redirect(
        decision,
        "dj-grill",
        "task grilling is incomplete -- answer at least one decision question and confirm shared understanding first",
        "run dj-grill, answer one decision question, and explicitly confirm shared understanding before implementation",
    )
}

fn readiness_redirect(
    decision: &RouteDecision,
    resolved_skill: &'static str,
    reason: &str,
    next_action: &str,
) -> RouteDecision {
    RouteDecision {
        task_status: decision.task_status.clone(),
        capsule: decision.capsule,
        requested_intent: decision.requested_intent.clone(),
        requested_skill: decision.requested_skill.clone(),
        resolved_skill,
        action: RouteAction::Redirect,
        reason: reason.to_string(),
        next_action: next_action.to_string(),
        requires_alignment_artifact: true,
        complexity: decision.complexity,
    }
}
/// Returns whether a task has the complete local grilling record required by
/// the compatibility gate. This checks record shape only; it does not verify
/// that a Pi UI or another protected issuer obtained the confirmation.
pub fn has_confirmed_grilling(tasks_dir: &Path, task_name: &str) -> bool {
    load_task(tasks_dir, task_name).ok().is_some_and(|task| {
        let Some(grilling) = task.meta.get("grilling") else {
            return false;
        };
        let started_at = grilling
            .get("startedAt")
            .and_then(|value| value.as_str())
            .is_some_and(|value| !value.trim().is_empty());
        let confirmed_at = grilling
            .get("confirmedAt")
            .and_then(|value| value.as_str())
            .is_some_and(|value| !value.trim().is_empty());
        let confirmation = grilling
            .get("confirmation")
            .and_then(|value| value.as_str())
            .is_some_and(|value| !value.trim().is_empty());
        let answered_question = grilling
            .get("questions")
            .and_then(|value| value.as_array())
            .is_some_and(|questions| {
                questions.iter().any(|question| {
                    question
                        .get("prompt")
                        .and_then(|value| value.as_str())
                        .is_some_and(|value| !value.trim().is_empty())
                        && question
                            .get("recommendation")
                            .and_then(|value| value.as_str())
                            .is_some_and(|value| !value.trim().is_empty())
                        && question
                            .get("answer")
                            .and_then(|value| value.as_str())
                            .is_some_and(|value| !value.trim().is_empty())
                })
            });
        started_at && answered_question && confirmed_at && confirmation
    })
}

fn has_task_or_parent_prd(tasks_dir: &Path, task_name: &str) -> bool {
    let task_prd = tasks_dir.join(task_name).join("prd.md");
    if has_substantive_prd(&task_prd) {
        return true;
    }
    load_task(tasks_dir, task_name)
        .ok()
        .and_then(|task| task.parent)
        .is_some_and(|parent| has_substantive_prd(&tasks_dir.join(parent).join("prd.md")))
}

fn has_substantive_prd(path: &Path) -> bool {
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };
    has_section_content(&content, "## Goal", |line| {
        !matches!(line, "TBD." | "TBD" | "<why this design is needed>")
    }) && has_section_content(&content, "## Requirements", |line| {
        line.starts_with("- ") && !matches!(line, "- TBD" | "- [ ] TBD")
    }) && has_section_content(&content, "## Acceptance Criteria", |line| {
        (line.starts_with("- [ ] ") || line.starts_with("- [x] "))
            && !matches!(line, "- [ ] TBD" | "- [x] TBD")
    })
}

fn has_section_content(content: &str, heading: &str, predicate: impl Fn(&str) -> bool) -> bool {
    let mut in_section = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == heading {
            in_section = true;
            continue;
        }
        if in_section && trimmed.starts_with("## ") {
            return false;
        }
        if in_section && !trimmed.is_empty() && predicate(trimmed) {
            return true;
        }
    }
    false
}

// ── Task hierarchy (parent / children) ────────────────────────────────────

/// Get the parent task record, if one exists.
pub fn get_parent_task(tasks_dir: &Path, task_name: &str) -> Result<Option<TaskRecord>, TaskError> {
    let task = load_task(tasks_dir, task_name)?;
    match task.parent {
        Some(parent_name) => load_task(tasks_dir, &parent_name).map(Some),
        None => Ok(None),
    }
}

/// Get all child task records.
pub fn get_child_tasks(tasks_dir: &Path, task_name: &str) -> Result<Vec<TaskRecord>, TaskError> {
    let task = load_task(tasks_dir, task_name)?;
    task.children
        .iter()
        .map(|name| load_task(tasks_dir, name))
        .collect()
}

/// Check if a task has any children.
pub fn has_children(tasks_dir: &Path, task_name: &str) -> Result<bool, TaskError> {
    let task = load_task(tasks_dir, task_name)?;
    Ok(!task.children.is_empty())
}

/// Recursively collect all descendant task IDs (DFS order).
pub fn subtree_ids(tasks_dir: &Path, task_name: &str) -> Result<Vec<String>, TaskError> {
    let mut ids = Vec::new();
    subtree_ids_recursive(tasks_dir, task_name, &mut ids)?;
    Ok(ids)
}

fn subtree_ids_recursive(
    tasks_dir: &Path,
    task_name: &str,
    ids: &mut Vec<String>,
) -> Result<(), TaskError> {
    let task = load_task(tasks_dir, task_name)?;
    for child in &task.children {
        ids.push(child.clone());
        subtree_ids_recursive(tasks_dir, child, ids)?;
    }
    Ok(())
}

// ── Completion criteria checklist ────────────────────────────────────────────

/// A single checklist item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub description: String,
    pub met: bool,
}

/// A completion checklist for a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionChecklist {
    pub criteria: Vec<ChecklistItem>,
}

impl Default for CompletionChecklist {
    fn default() -> Self {
        Self {
            criteria: Vec::new(),
        }
    }
}

const CHECKLIST_FILE: &str = "checklist.json";

fn checklist_path(tasks_dir: &Path, task_name: &str) -> PathBuf {
    tasks_dir.join(task_name).join(CHECKLIST_FILE)
}

/// Get the completion checklist for a task. Returns default (empty) if no checklist exists.
pub fn get_checklist(tasks_dir: &Path, task_name: &str) -> Result<CompletionChecklist, TaskError> {
    let path = checklist_path(tasks_dir, task_name);
    if path.exists() {
        let content = fs::read_to_string(&path)?;
        serde_json::from_str(&content).map_err(TaskError::Json)
    } else {
        Ok(CompletionChecklist::default())
    }
}

fn save_checklist(
    tasks_dir: &Path,
    task_name: &str,
    checklist: &CompletionChecklist,
) -> Result<(), TaskError> {
    let path = checklist_path(tasks_dir, task_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(checklist)?;
    fs::write(&path, content.as_bytes())?;
    Ok(())
}

/// Add a new checklist item (unmet by default).
pub fn add_checklist_item(
    tasks_dir: &Path,
    task_name: &str,
    description: &str,
) -> Result<(), TaskError> {
    let mut checklist = get_checklist(tasks_dir, task_name)?;
    checklist.criteria.push(ChecklistItem {
        description: description.to_string(),
        met: false,
    });
    save_checklist(tasks_dir, task_name, &checklist)
}

/// Mark a checklist item as met or unmet by index.
pub fn set_checklist_item(
    tasks_dir: &Path,
    task_name: &str,
    index: usize,
    met: bool,
) -> Result<(), TaskError> {
    let mut checklist = get_checklist(tasks_dir, task_name)?;
    if index >= checklist.criteria.len() {
        return Err(TaskError::NotFound(format!(
            "Checklist item at index {} not found (max: {})",
            index,
            checklist.criteria.len().saturating_sub(1)
        )));
    }
    checklist.criteria[index].met = met;
    save_checklist(tasks_dir, task_name, &checklist)
}

/// Remove a checklist item by index.
pub fn remove_checklist_item(
    tasks_dir: &Path,
    task_name: &str,
    index: usize,
) -> Result<(), TaskError> {
    let mut checklist = get_checklist(tasks_dir, task_name)?;
    if index >= checklist.criteria.len() {
        return Err(TaskError::NotFound(format!(
            "Checklist item at index {} not found (max: {})",
            index,
            checklist.criteria.len().saturating_sub(1)
        )));
    }
    checklist.criteria.remove(index);
    save_checklist(tasks_dir, task_name, &checklist)
}

/// Check whether all checklist items are marked met (must have at least one item).
pub fn is_checklist_complete(tasks_dir: &Path, task_name: &str) -> Result<bool, TaskError> {
    let checklist = get_checklist(tasks_dir, task_name)?;
    Ok(!checklist.criteria.is_empty() && checklist.criteria.iter().all(|c| c.met))
}

/// Completion gate: block Finish action unless the task is eligible to archive.
pub fn apply_completion_gate(
    tasks_dir: &Path,
    task_name: Option<&str>,
    decision: &RouteDecision,
) -> RouteDecision {
    let Some(task_name) = task_name else {
        return decision.clone();
    };
    if decision.capsule != WorkflowCapsule::Finish {
        return decision.clone();
    }
    match ensure_finish_eligible(tasks_dir, task_name) {
        Ok(()) => decision.clone(),
        Err(reason) => RouteDecision {
            task_status: decision.task_status.clone(),
            capsule: WorkflowCapsule::Finish,
            requested_intent: decision.requested_intent,
            requested_skill: decision.requested_skill.clone(),
            resolved_skill: "",
            action: RouteAction::Block,
            reason,
            next_action: String::new(),
            requires_alignment_artifact: false,
            complexity: decision.complexity,
        },
    }
}

/// Verifies the status and checklist required before `finish-work` archives a task.
pub fn ensure_finish_eligible(tasks_dir: &Path, task_name: &str) -> Result<(), String> {
    let task = load_task(tasks_dir, task_name).map_err(|error| error.to_string())?;
    if task.status != TaskStatus::Completed {
        return Err(format!(
            "Task '{}' is {}. Mark it completed through the verified workflow before finish-work.",
            task_name,
            task.status.as_str()
        ));
    }
    if !is_checklist_complete(tasks_dir, task_name).map_err(|error| error.to_string())? {
        return Err(format!(
            "Task '{}' has an incomplete or empty completion checklist.",
            task_name
        ));
    }
    Ok(())
}

// ── Task Queue ────────────────────────────────────────────────────

/// Read task queue from `.dijiang/queue.toml`.
/// Each non-empty, non-comment line is a task name.
pub fn read_task_queue(dijiang_dir: &Path) -> Vec<String> {
    let queue_path = dijiang_dir.join("queue.toml");
    let content = match std::fs::read_to_string(&queue_path) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

/// Write task queue to `.dijiang/queue.toml`.
pub fn write_task_queue(dijiang_dir: &Path, queue: &[String]) -> Result<(), TaskError> {
    let queue_path = dijiang_dir.join("queue.toml");
    let content = queue.join("\n");
    std::fs::write(&queue_path, content)?;
    Ok(())
}

/// Add a task to the end of the queue. Returns `false` if already present.
pub fn queue_add(dijiang_dir: &Path, task_name: &str) -> bool {
    let mut queue = read_task_queue(dijiang_dir);
    if queue.contains(&task_name.to_string()) {
        return false;
    }
    queue.push(task_name.to_string());
    write_task_queue(dijiang_dir, &queue).is_ok()
}

/// Remove a task from the queue. Returns `false` if not found.
pub fn queue_remove(dijiang_dir: &Path, task_name: &str) -> bool {
    let mut queue = read_task_queue(dijiang_dir);
    let before = queue.len();
    queue.retain(|t| t != task_name);
    if queue.len() == before {
        return false;
    }
    write_task_queue(dijiang_dir, &queue).is_ok()
}

/// Pop the first task from the queue (removes it from the queue).
pub fn queue_pop(dijiang_dir: &Path) -> Option<String> {
    let mut queue = read_task_queue(dijiang_dir);
    if queue.is_empty() {
        return None;
    }
    let task = queue.remove(0);
    write_task_queue(dijiang_dir, &queue).ok()?;
    Some(task)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_temp_tasks() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let tasks_dir = dir.path().join("tasks");
        fs::create_dir_all(&tasks_dir).unwrap();
        (dir, tasks_dir)
    }

    #[test]
    fn test_create_and_load_task() {
        let (_dir, tasks_dir) = setup_temp_tasks();
        let task = create_task("test-1", "Test Task");
        save_task(&tasks_dir, &task).unwrap();
        let loaded = load_task(&tasks_dir, "test-1").unwrap();
        assert_eq!(loaded.name, "test-1");
        assert_eq!(loaded.title, "Test Task");
        assert_eq!(loaded.status, TaskStatus::Planning);
    }

    #[test]
    fn test_list_tasks() {
        let (_dir, tasks_dir) = setup_temp_tasks();
        let t1 = create_task("a-task", "Alpha");
        let t2 = create_task("b-task", "Beta");
        save_task(&tasks_dir, &t1).unwrap();
        save_task(&tasks_dir, &t2).unwrap();
        let tasks = list_tasks(&tasks_dir).unwrap();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].name, "a-task");
        assert_eq!(tasks[1].name, "b-task");
    }

    #[test]
    fn test_update_status() {
        let (_dir, tasks_dir) = setup_temp_tasks();
        let task = create_task("test-update", "Update Test");
        save_task(&tasks_dir, &task).unwrap();
        let updated = update_status(&tasks_dir, "test-update", TaskStatus::InProgress).unwrap();
        assert_eq!(updated.status, TaskStatus::InProgress);
        let reloaded = load_task(&tasks_dir, "test-update").unwrap();
        assert_eq!(reloaded.status, TaskStatus::InProgress);
    }

    #[test]
    fn test_archive_task_sets_archived() {
        let (_dir, tasks_dir) = setup_temp_tasks();
        let task = create_task("test-archive-status", "Archive Status Test");
        save_task(&tasks_dir, &task).unwrap();
        let archived = archive_task(&tasks_dir, "test-archive-status").unwrap();
        assert_eq!(archived.status, TaskStatus::Archived);
        assert!(archived.archived_at.is_some(), "archived_at should be set");
        let reloaded = load_task(&tasks_dir, "test-archive-status").unwrap();
        assert_eq!(reloaded.status, TaskStatus::Archived);
    }

    fn allowed_implementation_decision() -> RouteDecision {
        RouteDecision {
            task_status: TaskStatus::InProgress,
            capsule: WorkflowCapsule::Implement,
            requested_intent: crate::route_gate::RouteIntent::Implement,
            requested_skill: Some("dj-implement".to_string()),
            resolved_skill: "dj-implement",
            action: RouteAction::Allow,
            reason: String::new(),
            next_action: String::new(),
            requires_alignment_artifact: false,
            complexity: crate::route_gate::TaskComplexity::Complex,
        }
    }

    fn write_substantive_prd(tasks_dir: &Path, task_name: &str) {
        let task_dir = tasks_dir.join(task_name);
        fs::create_dir_all(&task_dir).unwrap();
        fs::write(
            task_dir.join("prd.md"),
            "# Task\n\n## Goal\n\nPrevent invalid task completion.\n\n## Requirements\n\n- Require verified task artifacts.\n\n## Acceptance Criteria\n\n- [ ] Invalid tasks are blocked.\n",
        )
        .unwrap();
    }

    fn confirmed_grilling() -> serde_json::Value {
        serde_json::json!({
            "startedAt": "2026-07-30T08:00:00Z",
            "questions": [{
                "prompt": "Which acceptance criterion is highest risk?",
                "recommendation": "Define one observable outcome.",
                "answer": "Exported files must preserve column order."
            }],
            "confirmedAt": "2026-07-30T08:01:00Z",
            "confirmation": "I confirm we share this understanding."
        })
    }
    #[test]
    fn readiness_redirects_when_prd_is_missing_or_template() {
        let (dir, tasks_dir) = setup_temp_tasks();
        let dijiang_dir = dir.path().join(".dijiang");
        fs::create_dir_all(dijiang_dir.join("spec").join("task")).unwrap();
        let task = create_task("task", "Task");
        save_task(&tasks_dir, &task).unwrap();
        let decision = allowed_implementation_decision();
        assert_eq!(
            apply_readiness_gate(&dijiang_dir, &tasks_dir, "task", &decision).resolved_skill,
            "dj-output"
        );
        fs::write(
            tasks_dir.join("task").join("prd.md"),
            "# Task\n\n## Goal\n\nTBD.\n\n## Requirements\n\n- TBD\n\n## Acceptance Criteria\n\n- [ ] TBD\n",
        )
        .unwrap();
        assert_eq!(
            apply_readiness_gate(&dijiang_dir, &tasks_dir, "task", &decision).resolved_skill,
            "dj-output"
        );
    }

    #[test]
    fn readiness_requires_answered_and_confirmed_grilling() {
        let (dir, tasks_dir) = setup_temp_tasks();
        let dijiang_dir = dir.path().join(".dijiang");
        fs::create_dir_all(dijiang_dir.join("spec").join("task")).unwrap();
        let task = create_task("task", "Task");
        save_task(&tasks_dir, &task).unwrap();
        write_substantive_prd(&tasks_dir, "task");
        let decision = allowed_implementation_decision();

        let blocked = apply_readiness_gate(&dijiang_dir, &tasks_dir, "task", &decision);
        assert_eq!(blocked.action, RouteAction::Redirect);
        assert_eq!(blocked.resolved_skill, "dj-grill");

        let mut answered = load_task(&tasks_dir, "task").unwrap();
        answered.meta = serde_json::json!({
            "grilling": {
                "startedAt": "2026-07-30T08:00:00Z",
                "questions": [{
                    "prompt": "Which acceptance criterion is highest risk?",
                    "recommendation": "Define one observable outcome.",
                    "answer": "Exported files must preserve column order."
                }]
            }
        });
        save_task(&tasks_dir, &answered).unwrap();
        let still_blocked = apply_readiness_gate(&dijiang_dir, &tasks_dir, "task", &decision);
        assert_eq!(still_blocked.action, RouteAction::Redirect);
        assert_eq!(still_blocked.resolved_skill, "dj-grill");

        answered.meta = serde_json::json!({ "grilling": confirmed_grilling() });
        save_task(&tasks_dir, &answered).unwrap();
        assert_eq!(
            apply_readiness_gate(&dijiang_dir, &tasks_dir, "task", &decision).action,
            RouteAction::Allow
        );
    }

    #[test]
    fn readiness_applies_to_every_implementation_intent() {
        let (dir, tasks_dir) = setup_temp_tasks();
        let dijiang_dir = dir.path().join(".dijiang");
        fs::create_dir_all(dijiang_dir.join("spec").join("task")).unwrap();
        let mut task = create_task("task", "Task");
        task.meta = serde_json::json!({ "grilling": confirmed_grilling() });
        save_task(&tasks_dir, &task).unwrap();
        write_substantive_prd(&tasks_dir, "task");

        for intent in [
            crate::route_gate::RouteIntent::Implement,
            crate::route_gate::RouteIntent::Debug,
            crate::route_gate::RouteIntent::Check,
        ] {
            let mut decision = allowed_implementation_decision();
            decision.requested_intent = intent;
            decision.resolved_skill = "dj-channel";
            assert_eq!(
                apply_readiness_gate(&dijiang_dir, &tasks_dir, "task", &decision).action,
                RouteAction::Allow
            );
        }
    }

    #[test]
    fn readiness_allows_parent_prd_after_task_grilling_is_confirmed() {
        let (dir, tasks_dir) = setup_temp_tasks();
        let dijiang_dir = dir.path().join(".dijiang");
        fs::create_dir_all(dijiang_dir.join("spec").join("task")).unwrap();
        let mut task = create_task("task", "Task");
        task.meta = serde_json::json!({ "grilling": confirmed_grilling() });
        save_task(&tasks_dir, &task).unwrap();
        write_substantive_prd(&tasks_dir, "task");
        let decision = allowed_implementation_decision();
        assert_eq!(
            apply_readiness_gate(&dijiang_dir, &tasks_dir, "task", &decision).action,
            RouteAction::Allow
        );
        let mut child = create_task("child", "Child");
        child.parent = Some("task".to_string());
        child.meta = serde_json::json!({ "grilling": confirmed_grilling() });
        save_task(&tasks_dir, &child).unwrap();
        assert_eq!(
            apply_readiness_gate(&dijiang_dir, &tasks_dir, "child", &decision).action,
            RouteAction::Allow
        );
    }

    #[test]
    fn finish_eligibility_requires_completed_task_and_complete_checklist() {
        let (_dir, tasks_dir) = setup_temp_tasks();
        let task = create_task("task", "Task");
        save_task(&tasks_dir, &task).unwrap();
        assert!(ensure_finish_eligible(&tasks_dir, "task").is_err());
        update_status(&tasks_dir, "task", TaskStatus::InProgress).unwrap();
        update_status(&tasks_dir, "task", TaskStatus::Completed).unwrap();
        assert!(ensure_finish_eligible(&tasks_dir, "task").is_err());
        add_checklist_item(&tasks_dir, "task", "Verified").unwrap();
        assert!(ensure_finish_eligible(&tasks_dir, "task").is_err());
        set_checklist_item(&tasks_dir, "task", 0, true).unwrap();
        assert!(ensure_finish_eligible(&tasks_dir, "task").is_ok());
    }

    #[test]
    fn completion_gate_blocks_ineligible_finish() {
        let (_dir, tasks_dir) = setup_temp_tasks();
        let task = create_task("task", "Task");
        save_task(&tasks_dir, &task).unwrap();
        let decision = RouteDecision {
            task_status: TaskStatus::Planning,
            capsule: WorkflowCapsule::Finish,
            requested_intent: crate::route_gate::RouteIntent::Finish,
            requested_skill: Some("dijiang-finish-work".to_string()),
            resolved_skill: "dijiang-finish-work",
            action: RouteAction::Allow,
            reason: String::new(),
            next_action: String::new(),
            requires_alignment_artifact: false,
            complexity: crate::route_gate::TaskComplexity::Complex,
        };
        assert_eq!(
            apply_completion_gate(&tasks_dir, Some("task"), &decision).action,
            RouteAction::Block
        );
    }

    #[test]
    fn test_completed_then_archive_transition() {
        // Simulate the finish-work flow: save as Completed, then archive
        let (_dir, tasks_dir) = setup_temp_tasks();
        let mut task = create_task("test-complete-archive", "Complete → Archive");
        save_task(&tasks_dir, &task).unwrap();

        // Step 1: set to InProgress (like start would)
        let in_progress =
            update_status(&tasks_dir, "test-complete-archive", TaskStatus::InProgress).unwrap();
        assert_eq!(in_progress.status, TaskStatus::InProgress);

        // Step 2: set to Completed (like finish-work now does before archiving)
        task.status = TaskStatus::Completed;
        save_task(&tasks_dir, &task).unwrap();
        let loaded = load_task(&tasks_dir, "test-complete-archive").unwrap();
        assert_eq!(
            loaded.status,
            TaskStatus::Completed,
            "should pass through Completed"
        );

        add_checklist_item(&tasks_dir, "test-complete-archive", "verification complete").unwrap();
        set_checklist_item(&tasks_dir, "test-complete-archive", 0, true).unwrap();

        // Step 3: archive only after the shared finish eligibility gate passes.
        let archived = archive_task(&tasks_dir, "test-complete-archive").unwrap();
        assert_eq!(archived.status, TaskStatus::Archived);
        let reloaded = load_task(&tasks_dir, "test-complete-archive").unwrap();
        assert_eq!(reloaded.status, TaskStatus::Archived);
    }
    fn test_find_dijiang_dir() {
        let dir = tempfile::tempdir().unwrap();
        let dijiang = dir.path().join(".dijiang");
        fs::create_dir(&dijiang).unwrap();
        let found = find_dijiang_dir(dir.path());
        assert!(found.is_some());
        assert_eq!(found.unwrap(), dijiang);
    }

    #[test]
    fn test_active_task_write_with_session_does_not_touch_global_pointer() {
        let dir = tempfile::tempdir().unwrap();
        let dijiang_dir = dir.path().join(".dijiang");
        fs::create_dir(&dijiang_dir).unwrap();

        assert!(
            read_active_task_for_session(&dijiang_dir, None)
                .unwrap()
                .is_none()
        );

        let identity = SessionIdentity::new("codex", "window-a").unwrap();
        write_active_task_for_session(&dijiang_dir, "my-task", Some(&identity)).unwrap();

        let primary = dijiang_dir.join("active_task.txt");
        assert!(!primary.exists());

        let session = dijiang_dir
            .join(".runtime")
            .join("sessions")
            .join("codex_window-a.json");
        assert!(session.exists());
        let session_data: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&session).unwrap()).unwrap();
        assert_eq!(session_data["current_task"], "my-task");
        assert_eq!(session_data["session_key"], "codex_window-a");
        assert_eq!(session_data["source"], "codex");
        assert!(session_data["last_seen_at"].is_string());

        let marker = dijiang_dir.join(".runtime").join(".dijiang_owned");
        assert!(marker.exists());

        let active = read_active_task_for_session(&dijiang_dir, Some(&identity)).unwrap();
        assert_eq!(active, Some("my-task".to_string()));
        assert_eq!(
            read_active_task_for_session(&dijiang_dir, None).unwrap(),
            None
        );
    }

    #[test]
    fn test_active_task_is_session_scoped() {
        let dir = tempfile::tempdir().unwrap();
        let dijiang_dir = dir.path().join(".dijiang");
        fs::create_dir(&dijiang_dir).unwrap();

        let window_a = SessionIdentity::new("codex", "window-a").unwrap();
        let window_b = SessionIdentity::new("codex", "window-b").unwrap();
        write_active_task_for_session(&dijiang_dir, "task-a", Some(&window_a)).unwrap();
        write_active_task_for_session(&dijiang_dir, "task-b", Some(&window_b)).unwrap();

        assert_eq!(
            read_active_task_for_session(&dijiang_dir, Some(&window_a)).unwrap(),
            Some("task-a".to_string())
        );
        assert_eq!(
            read_active_task_for_session(&dijiang_dir, Some(&window_b)).unwrap(),
            Some("task-b".to_string())
        );
        assert_eq!(
            read_active_task_for_session(&dijiang_dir, None).unwrap(),
            None
        );
    }

    #[test]
    fn active_task_rejects_invalid_references_on_write_and_read() {
        let dir = tempfile::tempdir().unwrap();
        let dijiang_dir = dir.path().join(".dijiang");
        fs::create_dir(&dijiang_dir).unwrap();
        let identity = SessionIdentity::new("codex", "window-a").unwrap();

        assert!(
            write_active_task_for_session(&dijiang_dir, "../outside", Some(&identity)).is_err()
        );
        assert!(write_active_task_for_session(&dijiang_dir, "/tmp/outside", None).is_err());
        assert!(write_active_task_for_session(&dijiang_dir, " task-a", None).is_err());
        assert!(write_active_task_for_session(&dijiang_dir, "nested\\task", None).is_err());

        let session = session_path(&dijiang_dir, &identity);
        fs::create_dir_all(session.parent().unwrap()).unwrap();
        fs::write(&session, r#"{"current_task":"../outside"}"#).unwrap();
        assert!(read_active_task_for_session(&dijiang_dir, Some(&identity)).is_err());

        fs::write(&session, r#"{"current_task":" task-a"}"#).unwrap();
        assert!(read_active_task_for_session(&dijiang_dir, Some(&identity)).is_err());

        fs::remove_file(session).unwrap();
        fs::write(dijiang_dir.join("active_task.txt"), "/tmp/outside\n").unwrap();
        assert!(read_global_active_task(&dijiang_dir).is_err());
    }

    #[test]
    fn load_task_rejects_paths_outside_tasks_directory() {
        let (dir, tasks_dir) = setup_temp_tasks();
        let outside = dir.path().join("outside");
        fs::create_dir_all(&outside).unwrap();
        fs::write(
            outside.join("task.json"),
            serde_json::to_string(&create_task("outside", "Outside")).unwrap(),
        )
        .unwrap();

        assert!(load_task(&tasks_dir, "../outside").is_err());
        assert!(load_task(&tasks_dir, outside.to_str().unwrap()).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn load_task_rejects_symlink_outside_tasks_directory() {
        use std::os::unix::fs::symlink;

        let (dir, tasks_dir) = setup_temp_tasks();
        let outside = dir.path().join("outside");
        fs::create_dir_all(&outside).unwrap();
        fs::write(
            outside.join("task.json"),
            serde_json::to_string(&create_task("linked", "Linked")).unwrap(),
        )
        .unwrap();
        symlink(&outside, tasks_dir.join("linked")).unwrap();

        assert!(load_task(&tasks_dir, "linked").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn save_task_rejects_symlink_outside_tasks_directory() {
        use std::os::unix::fs::symlink;

        let (dir, tasks_dir) = setup_temp_tasks();
        let outside = dir.path().join("outside");
        fs::create_dir_all(&outside).unwrap();
        symlink(&outside, tasks_dir.join("linked")).unwrap();

        let task = create_task("linked", "Linked");
        assert!(save_task(&tasks_dir, &task).is_err());
        assert!(!outside.join("task.json").exists());
    }

    #[test]
    fn scaffold_task_docs_rejects_paths_outside_tasks_directory() {
        let (dir, tasks_dir) = setup_temp_tasks();
        assert!(scaffold_task_docs(&tasks_dir, "../outside", "Outside").is_err());
        assert!(!dir.path().join("outside/prd.md").exists());
    }

    #[cfg(unix)]
    #[test]
    fn scaffold_task_docs_rejects_symlink_outside_tasks_directory() {
        use std::os::unix::fs::symlink;

        let (dir, tasks_dir) = setup_temp_tasks();
        let outside = dir.path().join("outside");
        fs::create_dir_all(&outside).unwrap();
        symlink(&outside, tasks_dir.join("linked")).unwrap();

        assert!(scaffold_task_docs(&tasks_dir, "linked", "Linked").is_err());
        assert!(!outside.join("prd.md").exists());
    }

    #[test]
    fn test_active_task_falls_back_to_global_when_session_task_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let dijiang_dir = dir.path().join(".dijiang");
        fs::create_dir(&dijiang_dir).unwrap();
        fs::write(dijiang_dir.join("active_task.txt"), "fallback-task\n").unwrap();

        let identity = SessionIdentity::new("codex", "window-a").unwrap();
        let active = read_active_task_for_session(&dijiang_dir, Some(&identity)).unwrap();

        assert_eq!(active, Some("fallback-task".to_string()));
    }

    #[test]
    fn test_clear_active_task_with_session_also_clears_global_pointer() {
        let dir = tempfile::tempdir().unwrap();
        let dijiang_dir = dir.path().join(".dijiang");
        fs::create_dir(&dijiang_dir).unwrap();
        fs::write(dijiang_dir.join("active_task.txt"), "fallback-task\n").unwrap();

        let identity = SessionIdentity::new("codex", "window-a").unwrap();
        write_active_task_for_session(&dijiang_dir, "task-a", Some(&identity)).unwrap();
        clear_active_task_for_session(&dijiang_dir, Some(&identity)).unwrap();

        assert!(!dijiang_dir.join("active_task.txt").exists());
        assert_eq!(
            read_active_task_for_session(&dijiang_dir, Some(&identity)).unwrap(),
            None
        );
    }

    #[test]
    fn test_active_task_fallback_from_primary() {
        let dir = tempfile::tempdir().unwrap();
        let dijiang_dir = dir.path().join(".trellis");
        fs::create_dir(&dijiang_dir).unwrap();

        write_active_task_for_session(&dijiang_dir, "fallback-task", None).unwrap();

        let sessions_dir = dijiang_dir.join(".runtime").join("sessions");
        fs::remove_dir_all(sessions_dir).unwrap();

        let active = read_active_task_for_session(&dijiang_dir, None).unwrap();
        assert_eq!(active, None);

        let active = read_active_task(&dijiang_dir).unwrap();
        assert_eq!(active, Some("fallback-task".to_string()));
    }
}
