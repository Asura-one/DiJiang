use std::collections::BTreeMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

use crate::store::TaskError;

/// Relative spec file path → content hash (SipHash via std).
///
/// Not cryptographically secure — adequate for change detection.
pub type SpecChecksums = BTreeMap<String, u64>;

/// Result of comparing current spec files against stored checksums.
#[derive(Debug, Clone)]
pub struct SpecDiff {
    pub new: Vec<String>,
    pub changed: Vec<String>,
    pub deleted: Vec<String>,
}

impl SpecDiff {
    /// Returns `true` when any spec file has been added, modified, or removed.
    pub fn has_changes(&self) -> bool {
        !self.new.is_empty() || !self.changed.is_empty() || !self.deleted.is_empty()
    }
}

/// Recursively collect all readable file paths under `dir` (with known extensions).
fn collect_spec_files(dir: &Path, base_dir: &Path) -> Vec<(String, String)> {
    let mut files = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_spec_files(&path, base_dir));
        } else {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if matches!(ext, "md" | "json" | "yaml" | "toml" | "rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    let relative = path
                        .strip_prefix(base_dir)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();
                    files.push((relative, content));
                }
            }
        }
    }
    files
}

/// Compute SipHash checksums for all files under `.dijiang/spec/`.
/// Walks the spec directory recursively and hashes every recognized file.
pub fn compute_spec_checksums(dijiang_dir: &Path) -> SpecChecksums {
    let spec_dir = dijiang_dir.join("spec");
    if !spec_dir.is_dir() {
        return SpecChecksums::new();
    }
    let project_root = dijiang_dir.parent().unwrap_or(dijiang_dir);
    let mut checksums = SpecChecksums::new();
    for (relative, content) in collect_spec_files(&spec_dir, project_root) {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        content.hash(&mut hasher);
        checksums.insert(relative, hasher.finish());
    }
    checksums
}

/// Path to the persisted checksum file
fn checksums_path(dijiang_dir: &Path) -> std::path::PathBuf {
    dijiang_dir.join(".runtime").join("spec-checksums.json")
}

/// Read previously stored spec checksums from `.dijiang/.runtime/spec-checksums.json`.
///
/// Returns an empty map when the file does not exist (first run).
pub fn read_stored_checksums(dijiang_dir: &Path) -> Result<SpecChecksums, TaskError> {
    let path = checksums_path(dijiang_dir);
    if !path.exists() {
        return Ok(SpecChecksums::new());
    }
    let content = fs::read_to_string(&path)?;
    let checksums: SpecChecksums = serde_json::from_str(&content)?;
    Ok(checksums)
}

/// Persist current spec checksums to `.dijiang/.runtime/spec-checksums.json`.
///
/// Creates `.dijiang/.runtime/` if it does not exist.
pub fn write_stored_checksums(
    dijiang_dir: &Path,
    checksums: &SpecChecksums,
) -> Result<(), TaskError> {
    let path = checksums_path(dijiang_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(checksums)?;
    fs::write(&path, content)?;
    Ok(())
}

/// Compare current spec file checksums against the stored baseline.
///
/// Returns a [`SpecDiff`] describing files that are new, changed, or deleted.
pub fn check_spec_changes(dijiang_dir: &Path) -> Result<SpecDiff, TaskError> {
    let current = compute_spec_checksums(dijiang_dir);
    let stored = read_stored_checksums(dijiang_dir)?;

    let mut diff = SpecDiff {
        new: Vec::new(),
        changed: Vec::new(),
        deleted: Vec::new(),
    };

    for (path, hash) in &current {
        match stored.get(path) {
            None => diff.new.push(path.clone()),
            Some(old) if old != hash => diff.changed.push(path.clone()),
            _ => {}
        }
    }

    for path in stored.keys() {
        if !current.contains_key(path) {
            diff.deleted.push(path.clone());
        }
    }

    diff.new.sort();
    diff.changed.sort();
    diff.deleted.sort();

    Ok(diff)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_dijiang_dir() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let dijiang_dir = dir.path().join(".dijiang");
        fs::create_dir_all(dijiang_dir.join("spec/guides")).expect("create spec dir");
        (dir, dijiang_dir)
    }

    fn write_spec(dijiang_dir: &Path, rel: &str, content: &str) {
        let path = dijiang_dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, content).unwrap();
    }

    #[test]
    fn compute_checksums_empty_dir() {
        let (_tmp, dijiang_dir) = temp_dijiang_dir();
        let cs = compute_spec_checksums(&dijiang_dir);
        assert!(cs.is_empty(), "no spec files → empty checksums");
    }

    #[test]
    fn compute_checksums_single_file() {
        let (_tmp, dijiang_dir) = temp_dijiang_dir();
        write_spec(&dijiang_dir, "spec/guides/test.md", "hello");
        let cs = compute_spec_checksums(&dijiang_dir);
        assert_eq!(cs.len(), 1, "one .md file found");
        assert!(cs.contains_key(".dijiang/spec/guides/test.md"));
    }

    #[test]
    fn compute_checksums_ignores_unknown_extensions() {
        let (_tmp, dijiang_dir) = temp_dijiang_dir();
        write_spec(&dijiang_dir, "spec/readme.txt", "ignore me");
        write_spec(&dijiang_dir, "spec/logo.png", "");
        let cs = compute_spec_checksums(&dijiang_dir);
        assert!(cs.is_empty(), "only .txt and .png → no recognized files");
    }

    #[test]
    fn compute_checksums_stable_hash() {
        let (_tmp, dijiang_dir) = temp_dijiang_dir();
        write_spec(&dijiang_dir, "spec/test.md", "stable content");
        let cs1 = compute_spec_checksums(&dijiang_dir);
        let cs2 = compute_spec_checksums(&dijiang_dir);
        assert_eq!(cs1, cs2, "same content → same hash");
    }

    #[test]
    fn compute_checksums_hash_on_content_change() {
        let (_tmp, dijiang_dir) = temp_dijiang_dir();
        write_spec(&dijiang_dir, "spec/test.md", "version 1");
        let cs1 = compute_spec_checksums(&dijiang_dir);
        write_spec(&dijiang_dir, "spec/test.md", "version 2");
        let cs2 = compute_spec_checksums(&dijiang_dir);
        assert_ne!(cs1, cs2, "different content → different hash");
    }

    #[test]
    fn read_stored_empty_when_file_missing() {
        let (_tmp, dijiang_dir) = temp_dijiang_dir();
        let stored = read_stored_checksums(&dijiang_dir).expect("read ok");
        assert!(stored.is_empty(), "no file yet → empty map");
    }

    #[test]
    fn round_trip_write_and_read() {
        let (_tmp, dijiang_dir) = temp_dijiang_dir();
        let mut cs = SpecChecksums::new();
        cs.insert("spec/a.md".into(), 42);
        cs.insert("spec/b.md".into(), 99);
        write_stored_checksums(&dijiang_dir, &cs).expect("write ok");
        let read_back = read_stored_checksums(&dijiang_dir).expect("read ok");
        assert_eq!(cs, read_back, "round-trip preserves all entries");
    }

    #[test]
    fn check_spec_changes_all_new() {
        let (_tmp, dijiang_dir) = temp_dijiang_dir();
        write_spec(&dijiang_dir, "spec/new.md", "content");
        let diff = check_spec_changes(&dijiang_dir).expect("check ok");
        assert!(diff.has_changes());
        assert_eq!(diff.new, vec![".dijiang/spec/new.md"]);
        assert!(diff.changed.is_empty());
        assert!(diff.deleted.is_empty());
    }

    #[test]
    fn check_spec_changes_detects_modification() {
        let (_tmp, dijiang_dir) = temp_dijiang_dir();
        write_spec(&dijiang_dir, "spec/stable.md", "original");
        let cs = compute_spec_checksums(&dijiang_dir);
        write_stored_checksums(&dijiang_dir, &cs).expect("write stored");

        write_spec(&dijiang_dir, "spec/stable.md", "modified");
        let diff = check_spec_changes(&dijiang_dir).expect("check ok");
        assert!(diff.has_changes());
        assert!(diff.new.is_empty());
        assert_eq!(diff.changed, vec![".dijiang/spec/stable.md"]);
        assert!(diff.deleted.is_empty());
    }

    #[test]
    fn check_spec_changes_detects_deletion() {
        let (_tmp, dijiang_dir) = temp_dijiang_dir();
        write_spec(&dijiang_dir, "spec/to-delete.md", "original");
        let cs = compute_spec_checksums(&dijiang_dir);
        write_stored_checksums(&dijiang_dir, &cs).expect("write stored");

        // Remove the file
        fs::remove_file(dijiang_dir.join("spec/to-delete.md")).unwrap();
        let diff = check_spec_changes(&dijiang_dir).expect("check ok");
        assert!(diff.has_changes());
        assert_eq!(diff.deleted, vec![".dijiang/spec/to-delete.md"]);
    }

    #[test]
    fn check_spec_changes_no_change() {
        let (_tmp, dijiang_dir) = temp_dijiang_dir();
        write_spec(&dijiang_dir, "spec/stable.md", "unchanged");
        let cs = compute_spec_checksums(&dijiang_dir);
        write_stored_checksums(&dijiang_dir, &cs).expect("write stored");
        let diff = check_spec_changes(&dijiang_dir).expect("check ok");
        assert!(!diff.has_changes(), "no changes expected");
    }

    #[test]
    fn check_spec_changes_mixed() {
        let (_tmp, dijiang_dir) = temp_dijiang_dir();
        write_spec(&dijiang_dir, "spec/unchanged.md", "same");
        write_spec(&dijiang_dir, "spec/to-change.md", "old");
        write_spec(&dijiang_dir, "spec/to-delete.md", "delete me");

        let cs = compute_spec_checksums(&dijiang_dir);
        write_stored_checksums(&dijiang_dir, &cs).expect("write stored");

        // Now: add one, change one, delete one
        write_spec(&dijiang_dir, "spec/new.md", "new file");
        write_spec(&dijiang_dir, "spec/to-change.md", "new content");
        fs::remove_file(dijiang_dir.join("spec/to-delete.md")).unwrap();

        let diff = check_spec_changes(&dijiang_dir).expect("check ok");
        assert!(diff.has_changes());
        assert_eq!(diff.new, vec![".dijiang/spec/new.md"]);
        assert_eq!(diff.changed, vec![".dijiang/spec/to-change.md"]);
        assert_eq!(diff.deleted, vec![".dijiang/spec/to-delete.md"]);
    }
}
