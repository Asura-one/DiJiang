use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::types::{TaskRecord, TaskStatus};

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
    Ok(data
        .get("current_task")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string))
}

/// Find the active task for the current session, using global state only when no session identity exists.
pub fn read_active_task(dijiang_dir: &Path) -> Result<Option<String>, TaskError> {
    read_active_task_for_session(dijiang_dir, current_session_identity().as_ref())
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
            return Ok(Some(active.to_string()));
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

    read_global_active_task(dijiang_dir)
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


pub fn clear_active_task_for_session(
    dijiang_dir: &Path,
    identity: Option<&SessionIdentity>,
) -> Result<(), TaskError> {
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
    let active_path = dijiang_dir.join("active_task.txt");
    if active_path.exists() {
        fs::remove_file(active_path)?;
    }

    Ok(())
}

/// Load a single task from its task.json file.
pub fn load_task(tasks_dir: &Path, task_name: &str) -> Result<TaskRecord, TaskError> {
    let path = tasks_dir.join(task_name).join("task.json");
    if !path.exists() {
        return Err(TaskError::NotFound(task_name.to_string()));
    }
    let content = fs::read_to_string(&path)?;
    let record = serde_json::from_str(&content)?;
    Ok(record)
}

/// Save a task record to its task.json file.
pub fn save_task(tasks_dir: &Path, task: &TaskRecord) -> Result<(), TaskError> {
    let task_dir = tasks_dir.join(&task.name);
    fs::create_dir_all(&task_dir)?;
    let path = task_dir.join("task.json");
    let content = serde_json::to_string_pretty(task)?;
    fs::write(&path, content)?;
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
    task.status = new_status;
    save_task(tasks_dir, &task)?;
    Ok(task)
}

/// Archive a task: set status to Archived and record archived_at timestamp.
pub fn archive_task(tasks_dir: &Path, task_name: &str) -> Result<TaskRecord, TaskError> {
    let mut task = load_task(tasks_dir, task_name)?;
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
    }
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
        // Verify persisted
        let reloaded = load_task(&tasks_dir, "test-update").unwrap();
        assert_eq!(reloaded.status, TaskStatus::InProgress);
    }

    #[test]
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
        // When sessions path is removed, read should fall back to active_task.txt.
        let dir = tempfile::tempdir().unwrap();
        let dijiang_dir = dir.path().join(".trellis");
        fs::create_dir(&dijiang_dir).unwrap();

        write_active_task_for_session(&dijiang_dir, "fallback-task", None).unwrap();

        // Remove sessions dir to simulate Trellis-only write
        let sessions_dir = dijiang_dir.join(".runtime").join("sessions");
        fs::remove_dir_all(sessions_dir).unwrap();

        // Should still read from active_task.txt
        let active = read_active_task_for_session(&dijiang_dir, None).unwrap();
        assert_eq!(active, Some("fallback-task".to_string()));
    }
}
