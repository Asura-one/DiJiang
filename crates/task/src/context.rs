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

fn is_sensitive_context_path(path: &Path) -> bool {
    path.components().any(|component| {
        let component = component.as_os_str().to_string_lossy().to_ascii_lowercase();
        component.starts_with(".env")
            || matches!(
                component.as_str(),
                "secrets" | ".ssh" | ".aws" | ".gnupg" | ".config"
            )
            || component.contains("credential")
    })
}

/// Resolve a context path only when it remains inside the project root.
pub fn resolve_context_file(repo_root: &Path, file: &str) -> Result<PathBuf, TaskError> {
    let path = Path::new(file);
    if path.is_absolute()
        || file.is_empty()
        || is_sensitive_context_path(path)
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
    let canonical_relative = canonical_candidate
        .strip_prefix(&canonical_root)
        .map_err(|_| TaskError::InvalidContextPath(file.to_string()))?;
    if is_sensitive_context_path(canonical_relative) {
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
    let mut entries = Vec::new();
    for (index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let entry =
            serde_json::from_str(line).map_err(|source| TaskError::InvalidContextManifest {
                path: path.display().to_string(),
                line: index + 1,
                source,
            })?;
        entries.push(entry);
    }
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
        fs::create_dir_all(root.path().join("docs/gh")).unwrap();
        fs::write(root.path().join("docs/gh/guide.md"), "guide").unwrap();
        resolve_context_file(root.path(), "docs/gh/guide.md").unwrap();

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
            fs::write(root.path().join(".env"), "placeholder").unwrap();
            std::os::unix::fs::symlink(root.path().join(".env"), root.path().join("public.md"))
                .unwrap();
            let sensitive_symlink = ContextEntry {
                file: "public.md".to_string(),
                ..valid.clone()
            };
            assert!(matches!(
                add_context_entry(&tasks_dir, "task", &sensitive_symlink),
                Err(TaskError::InvalidContextPath(_))
            ));
        }
        assert!(matches!(
            context_manifest_path(&tasks_dir, "task", "other"),
            Err(TaskError::InvalidContextAction(_))
        ));
    }

    #[test]
    fn context_entries_reject_sensitive_paths() {
        let root = tempfile::tempdir().unwrap();
        let dijiang_dir = root.path().join(".dijiang");
        let tasks_dir = dijiang_dir.join("tasks");
        fs::create_dir_all(tasks_dir.join("task")).unwrap();

        for file in [
            ".env",
            ".env.local",
            ".envrc",
            "config/credentials.json",
            "secrets/key.txt",
            ".ssh/id_ed25519",
            ".aws/config",
            ".gnupg/pubring.kbx",
            ".config/gh/hosts.yml",
        ] {
            let path = root.path().join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, "placeholder").unwrap();
            let entry = ContextEntry {
                action: "implement".to_string(),
                file: file.to_string(),
                reason: "must not be attached".to_string(),
            };
            assert!(matches!(
                add_context_entry(&tasks_dir, "task", &entry),
                Err(TaskError::InvalidContextPath(_))
            ));
        }
    }

    #[test]
    fn context_manifest_reports_invalid_json_line() {
        let root = tempfile::tempdir().unwrap();
        let tasks_dir = root.path().join(".dijiang/tasks");
        let task_dir = tasks_dir.join("task");
        fs::create_dir_all(&task_dir).unwrap();
        fs::write(
            task_dir.join("implement.jsonl"),
            "{\"action\":\"implement\",\"file\":\"spec.md\",\"reason\":\"ok\"}\n{broken\n",
        )
        .unwrap();

        let error = list_context_entries(&tasks_dir, "task", "implement")
            .expect_err("invalid JSONL must not be silently discarded");
        assert!(error.to_string().contains("implement.jsonl:2"));
    }
}
