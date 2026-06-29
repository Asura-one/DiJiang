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

/// Find the `.trellis/` directory by walking up from `cwd`.
pub fn find_trellis_dir(cwd: &Path) -> Option<PathBuf> {
    let mut dir = Some(cwd.to_path_buf());
    while let Some(d) = dir {
        let candidate = d.join(".trellis");
        if candidate.is_dir() {
            return Some(candidate);
        }
        dir = d.parent().map(|p| p.to_path_buf());
    }
    None
}

/// Find the active task by reading runtime session files.
/// Falls back to `.trellis/active_task.txt` for Rust-only usage.
pub fn read_active_task(trellis_dir: &Path) -> Result<Option<String>, TaskError> {
    // First try runtime sessions (Python task.py format)
    let sessions_dir = trellis_dir.join(".runtime").join("sessions");
    if sessions_dir.is_dir() {
        let mut latest: Option<(String, String)> = None; // (task_name, last_seen)
        for entry in fs::read_dir(&sessions_dir)? {
            let entry = entry?;
            if !entry.path().extension().is_some_and(|e| e == "json") {
                continue;
            }
            let content = fs::read_to_string(entry.path())?;
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(task) = data.get("current_task").and_then(|v| v.as_str()) {
                    let seen = data
                        .get("last_seen_at")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    match &latest {
                        Some((_, last_seen)) if seen > last_seen.as_str() => {
                            latest = Some((task.to_string(), seen.to_string()));
                        }
                        None => {
                            latest = Some((task.to_string(), seen.to_string()));
                        }
                        _ => {}
                    }
                }
            }
        }
        if let Some((task, _)) = latest {
            return Ok(Some(task));
        }
    }

    // Fallback: Rust active_task.txt
    let path = trellis_dir.join("active_task.txt");
    if path.exists() {
        let content = fs::read_to_string(&path)?;
        return Ok(Some(content.trim().to_string()));
    }

    Ok(None)
}

/// Write the active task name to `.trellis/active_task.txt`.
/// Write the active task name to `.trellis/active_task.txt`,
/// and dual-write to `.runtime/sessions/<task_name>.json` so that
/// `read_active_task` finds the active task via either path.
///
/// Also creates a `.runtime/.trellis_owned` marker to declare DiJiang's
/// ownership of the `.runtime/` subtree.
pub fn write_active_task(trellis_dir: &Path, task_name: &str) -> Result<(), TaskError> {
    // Primary: `.trellis/active_task.txt` (simple format, Trellis-compat)
    let path = trellis_dir.join("active_task.txt");
    fs::write(&path, task_name)?;

    // Dual-write: `.runtime/sessions/<task_name>.json`
    let runtime_dir = trellis_dir.join(".runtime");
    let sessions_dir = runtime_dir.join("sessions");
    fs::create_dir_all(&sessions_dir)?;
    let session = serde_json::json!({
        "current_task": task_name,
        "last_seen_at": chrono::Utc::now().to_rfc3339(),
    });
    let session_path = sessions_dir.join(format!("{task_name}.json"));
    fs::write(&session_path, serde_json::to_string_pretty(&session)?)?;

    // Ownership marker: `.runtime/.trellis_owned`
    fs::write(runtime_dir.join(".trellis_owned"), "")?;

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
pub fn archive_task(
    tasks_dir: &Path,
    task_name: &str,
) -> Result<TaskRecord, TaskError> {
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
                if let Ok(archived_date) = chrono::NaiveDate::parse_from_str(archived_at, "%Y-%m-%d")
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
    fn test_find_trellis_dir() {
        let dir = tempfile::tempdir().unwrap();
        let trellis = dir.path().join(".trellis");
        fs::create_dir(&trellis).unwrap();
        let found = find_trellis_dir(dir.path());
        assert!(found.is_some());
        assert_eq!(found.unwrap(), trellis);
    }

    #[test]
    fn test_active_task_dual_write() {
        let dir = tempfile::tempdir().unwrap();
        let trellis_dir = dir.path().join(".trellis");
        fs::create_dir(&trellis_dir).unwrap();

        // Before any write — no active task.
        assert!(read_active_task(&trellis_dir).unwrap().is_none());

        write_active_task(&trellis_dir, "my-task").unwrap();

        // Primary path: `.trellis/active_task.txt`
        let primary = trellis_dir.join("active_task.txt");
        assert!(primary.exists());
        assert_eq!(fs::read_to_string(&primary).unwrap(), "my-task");

        // Dual-write path: `.runtime/sessions/my-task.json`
        let session = trellis_dir.join(".runtime").join("sessions").join("my-task.json");
        assert!(session.exists());
        let session_data: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&session).unwrap()).unwrap();
        assert_eq!(session_data["current_task"], "my-task");
        assert!(session_data["last_seen_at"].is_string());

        // Owned marker
        let marker = trellis_dir.join(".runtime").join(".trellis_owned");
        assert!(marker.exists());

        // read_active_task returns the task (prefers sessions, falls back to active_task.txt)
        let active = read_active_task(&trellis_dir).unwrap();
        assert_eq!(active, Some("my-task".to_string()));
    }

    #[test]
    fn test_active_task_fallback_from_primary() {
        // When sessions path is removed, read should fall back to active_task.txt.
        let dir = tempfile::tempdir().unwrap();
        let trellis_dir = dir.path().join(".trellis");
        fs::create_dir(&trellis_dir).unwrap();

        write_active_task(&trellis_dir, "fallback-task").unwrap();

        // Remove sessions dir to simulate Trellis-only write
        let sessions_dir = trellis_dir.join(".runtime").join("sessions");
        fs::remove_dir_all(sessions_dir).unwrap();

        // Should still read from active_task.txt
        let active = read_active_task(&trellis_dir).unwrap();
        assert_eq!(active, Some("fallback-task".to_string()));
    }
}
