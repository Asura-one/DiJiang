use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use std::path::Path;
use std::fs;
use crate::store::TaskError;

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

/// Path to the context manifest file for a given action.
pub fn context_manifest_path(tasks_dir: &Path, task_name: &str, action: &str) -> PathBuf {
    tasks_dir.join(task_name).join(format!("{action}.jsonl"))
}

// ── Add entry ──────────────────────────────────────────────────────

/// Add a context entry to a task's JSONL manifest.
/// Creates the file if it doesn't exist.
pub fn add_context_entry(
    tasks_dir: &Path,
    task_name: &str,
    entry: &ContextEntry,
) -> Result<(), TaskError> {
    let path = context_manifest_path(tasks_dir, task_name, &entry.action);
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
    let path = context_manifest_path(tasks_dir, task_name, action);
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
