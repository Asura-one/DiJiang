/// Shared JSONL reading utilities.
use std::fs;
use std::path::Path;

/// Read the first line of a JSONL file and deserialize.
pub fn read_jsonl_first<T: serde::de::DeserializeOwned>(path: &Path) -> Option<T> {
    let content = fs::read_to_string(path).ok()?;
    let first_line = content.lines().next()?;
    serde_json::from_str(first_line).ok()
}

/// Read all lines of a JSONL file and deserialize each.
pub fn read_jsonl_all<T: serde::de::DeserializeOwned>(path: &Path) -> Vec<T> {
    let content = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                None
            } else {
                serde_json::from_str(trimmed).ok()
            }
        })
        .collect()
}

/// Walk a directory recursively, yielding file paths matching a predicate.
pub fn walk_dir<F>(dir: &Path, pred: F) -> Vec<std::path::PathBuf>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut results = Vec::new();
    if !dir.exists() {
        return results;
    }
    walk_dir_inner(dir, &pred, &mut results);
    results
}

fn walk_dir_inner(
    dir: &Path,
    pred: &dyn Fn(&std::path::Path) -> bool,
    results: &mut Vec<std::path::PathBuf>,
) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk_dir_inner(&path, pred, results);
            } else if pred(&path) {
                results.push(path);
            }
        }
    }
}
