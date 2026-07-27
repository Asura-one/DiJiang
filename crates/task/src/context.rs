use crate::store::TaskError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

// ── Types ──────────────────────────────────────────────────────────

/// A context entry stored in a JSONL manifest.
///
/// Each entry documents why a specific spec file or context was attached
/// to a task during a given phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEntry {
    /// Which sub-agent this entry is for: "implement" or "check"
    pub action: String,
    /// Path to the spec file (relative to project root)
    pub file: String,
    /// Why this spec is needed for this task
    pub reason: String,
}

// ── Manifest file path ─────────────────────────────────────────────

const CONTEXT_ACTIONS: &[&str] = &["implement", "check"];

fn validate_context_action(action: &str) -> Result<&str, TaskError> {
    if CONTEXT_ACTIONS.contains(&action) {
        Ok(action)
    } else {
        Err(TaskError::InvalidContextAction(action.to_string()))
    }
}

/// Resolve a context path only when it remains inside the project root.
pub fn resolve_context_file(repo_root: &Path, file: &str) -> Result<PathBuf, TaskError> {
    let path = Path::new(file);
    if path.is_absolute()
        || file.is_empty()
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(TaskError::InvalidContextPath(file.to_string()));
    }

    let canonical_root = repo_root
        .canonicalize()
        .map_err(|_| TaskError::InvalidContextPath(file.to_string()))?;
    let candidate = repo_root.join(path);
    let canonical_candidate = candidate
        .canonicalize()
        .map_err(|_| TaskError::InvalidContextPath(file.to_string()))?;
    if !canonical_candidate.starts_with(&canonical_root) {
        return Err(TaskError::InvalidContextPath(file.to_string()));
    }
    Ok(canonical_candidate)
}

/// Path to the context manifest file for a given action.
pub fn context_manifest_path(
    tasks_dir: &Path,
    task_name: &str,
    action: &str,
) -> Result<PathBuf, TaskError> {
    let action = validate_context_action(action)?;
    Ok(tasks_dir.join(task_name).join(format!("{action}.jsonl")))
}

// ── Add entry ──────────────────────────────────────────────────────

/// Add a context entry to a task's JSONL manifest.
/// Creates the file if it doesn't exist.
pub fn add_context_entry(
    tasks_dir: &Path,
    task_name: &str,
    entry: &ContextEntry,
) -> Result<(), TaskError> {
    let path = context_manifest_path(tasks_dir, task_name, &entry.action)?;
    let repo_root = tasks_dir
        .parent()
        .and_then(Path::parent)
        .unwrap_or(tasks_dir);
    let context_file = resolve_context_file(repo_root, &entry.file)?;
    if !context_file.is_file() {
        return Err(TaskError::InvalidContextPath(entry.file.clone()));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let line = serde_json::to_string(entry)? + "\n";
    fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&path)?
        .write_all(line.as_bytes())?;
    Ok(())
}

// ── List entries ───────────────────────────────────────────────────

/// List context entries from a task's JSONL manifest.
pub fn list_context_entries(
    tasks_dir: &Path,
    task_name: &str,
    action: &str,
) -> Result<Vec<ContextEntry>, TaskError> {
    let path = context_manifest_path(tasks_dir, task_name, action)?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(&path)?;
    let entries: Vec<ContextEntry> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_entries_stay_inside_the_project_and_known_actions() {
        let root = tempfile::tempdir().unwrap();
        let dijiang_dir = root.path().join(".dijiang");
        let tasks_dir = dijiang_dir.join("tasks");
        fs::create_dir_all(tasks_dir.join("task")).unwrap();
        fs::write(root.path().join("spec.md"), "spec").unwrap();

        let valid = ContextEntry {
            action: "implement".to_string(),
            file: "spec.md".to_string(),
            reason: "required by task".to_string(),
        };
        add_context_entry(&tasks_dir, "task", &valid).unwrap();
        assert_eq!(
            list_context_entries(&tasks_dir, "task", "implement")
                .unwrap()
                .len(),
            1
        );

        let escaped = ContextEntry {
            file: "../outside.md".to_string(),
            ..valid.clone()
        };
        assert!(matches!(
            add_context_entry(&tasks_dir, "task", &escaped),
            Err(TaskError::InvalidContextPath(_))
        ));
        let outside = root.path().parent().unwrap().join("outside.md");
        fs::write(&outside, "outside").unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&outside, root.path().join("linked.md")).unwrap();
            let symlinked = ContextEntry {
                file: "linked.md".to_string(),
                ..valid.clone()
            };
            assert!(matches!(
                add_context_entry(&tasks_dir, "task", &symlinked),
                Err(TaskError::InvalidContextPath(_))
            ));
        }
        assert!(matches!(
            context_manifest_path(&tasks_dir, "task", "other"),
            Err(TaskError::InvalidContextAction(_))
        ));
    }
}
