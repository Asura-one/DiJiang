use super::{ApiItemKind, ChangeKind, DepChange, DiffReport, ModuleChange, PubApiChange};
use std::path::Path;

/// Parse git diff output and extract structured change information.
///
/// Phase 1: analyzes changed files by path patterns.
/// Future phases: full diff content parsing for API-level detection.
pub struct DiffAnalyzer;

impl DiffAnalyzer {
    /// Run `git diff <base>..HEAD` and produce a structured [`DiffReport`].
    ///
    /// `base` defaults to `main`, `project_root` is the git repository root.
    pub fn analyze(project_root: &Path, base: &str) -> Result<DiffReport, String> {
        let changed_files = Self::get_changed_files(project_root, base)?;
        let commit_summaries = Self::get_commit_summaries(project_root, base)?;

        // Read full diff content for API-level analysis
        let diff_content = Self::get_diff_content(project_root, base);

        Ok(DiffReport {
            pub_api_changes: Self::detect_pub_api_changes(&diff_content),
            module_changes: Self::detect_module_changes(&changed_files),
            dep_changes: Self::detect_dep_changes(&diff_content),
            changed_files,
            commit_summaries,
        })
    }

    /// Get list of files changed between base..HEAD.
    fn get_changed_files(root: &Path, base: &str) -> Result<Vec<String>, String> {
        let output = std::process::Command::new("git")
            .args(["-C", &root.to_string_lossy()])
            .args(["diff", &format!("{base}..HEAD"), "--name-only"])
            .output()
            .map_err(|e| format!("git diff failed: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git diff error: {stderr}"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let files: Vec<String> = stdout
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
        Ok(files)
    }

    /// Get full diff content for deeper analysis.
    fn get_diff_content(root: &Path, base: &str) -> String {
        if let Ok(output) = std::process::Command::new("git")
            .args(["-C", &root.to_string_lossy()])
            .args(["diff", &format!("{base}..HEAD")])
            .output()
        {
            if output.status.success() {
                return String::from_utf8_lossy(&output.stdout).to_string();
            }
        }
        String::new()
    }

    /// Get commit message summaries between base..HEAD.
    fn get_commit_summaries(root: &Path, base: &str) -> Result<Vec<String>, String> {
        let output = std::process::Command::new("git")
            .args(["-C", &root.to_string_lossy()])
            .args([
                "log",
                &format!("{base}..HEAD"),
                "--oneline",
                "--no-decorate",
            ])
            .output()
            .map_err(|e| format!("git log failed: {e}"))?;

        if !output.status.success() {
            return Ok(Vec::new()); // non-fatal
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|l| l.to_string()).collect())
    }

    /// Detect pub API changes from diff content.
    ///
    /// Phase 1: heuristic-based detection from diff lines containing `pub fn`, `pub struct`, etc.
    fn detect_pub_api_changes(diff: &str) -> Vec<PubApiChange> {
        if diff.is_empty() {
            return Vec::new();
        }

        let mut changes = Vec::new();
        let mut current_file = String::new();

        for line in diff.lines() {
            if let Some(file) = line.strip_prefix("+++ b/") {
                current_file = file.to_string();
                continue;
            }
            if current_file.is_empty() {
                continue;
            }

            let is_added = line.starts_with('+') && !line.starts_with("+++");
            let is_removed = line.starts_with('-') && !line.starts_with("---");
            if !is_added && !is_removed {
                continue;
            }

            let content = if is_added || is_removed {
                &line[1..]
            } else {
                line
            };

            let change_kind = if is_added {
                ChangeKind::Added
            } else {
                ChangeKind::Removed
            };

            // Detect pub items
            if let Some(name) = Self::extract_pub_item(content) {
                changes.push(PubApiChange {
                    file: current_file.clone(),
                    name,
                    kind: ApiItemKind::Function,
                    change: change_kind,
                });
            }
        }

        changes
    }

    /// Extract a pub item name from a diff line.
    ///
    /// Matches patterns like `pub fn foo`, `pub struct Foo`, `pub trait Foo`, etc.
    fn extract_pub_item(line: &str) -> Option<String> {
        let trimmed = line.trim();

        // Skip non-pub lines
        if !trimmed.starts_with("pub ") {
            return None;
        }

        // Skip attribute lines and comments
        if trimmed.starts_with("pub(crate)")
            || trimmed.starts_with("pub(self)")
            || trimmed.starts_with("pub(super)")
        {
            return None;
        }

        let keywords = ["fn ", "struct ", "trait ", "enum ", "mod ", "type ", "const "];
        for kw in &keywords {
            if let Some(rest) = trimmed.strip_prefix("pub ") {
                if let Some(name_start) = rest.strip_prefix(kw) {
                    let name = name_start
                        .split(|c: char| c.is_whitespace() || c == '(' || c == '<' || c == '!')
                        .next()
                        .unwrap_or(name_start);
                    let name = name.trim_end_matches(';');
                    if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        return Some(name.to_string());
                    }
                }
            }
        }

        None
    }

    /// Detect module/directory changes from the changed file list.
    fn detect_module_changes(changed_files: &[String]) -> Vec<ModuleChange> {
        let mut modules: std::collections::BTreeMap<String, ChangeKind> =
            std::collections::BTreeMap::new();

        for file in changed_files {
            // Look for mod.rs files — these indicate module structure changes
            if file.ends_with("/mod.rs") || file == "mod.rs" || file.ends_with("/mod") {
                let dir = if file.ends_with("/mod.rs") {
                    file.trim_end_matches("/mod.rs")
                } else if file == "mod.rs" {
                    "."
                } else {
                    file.trim_end_matches("/mod")
                };
                modules.insert(dir.to_string(), ChangeKind::Modified);
            }

            // Detect newly added directories
            let parts: Vec<&str> = file.split('/').collect();
            if parts.len() >= 2 {
                // Top-level directory
                let top_dir = parts[0].to_string();
                modules.entry(top_dir).or_insert(ChangeKind::Modified);
            }
        }

        modules
            .into_iter()
            .map(|(path, kind)| ModuleChange { path, kind })
            .collect()
    }

    /// Detect dependency changes from diff content (Cargo.toml, etc.)
    fn detect_dep_changes(diff: &str) -> Vec<DepChange> {
        if diff.is_empty() {
            return Vec::new();
        }

        let mut changes = Vec::new();

        // Check if any Cargo.toml was changed
        let has_cargo_changes = diff.lines().any(|l| l.contains("Cargo.toml"));
        if !has_cargo_changes {
            return changes;
        }

        // Simple heuristic: look for `name = "..."` lines in added/removed context
        let mut in_deps = false;
        for line in diff.lines() {
            let content = if line.starts_with('+') {
                &line[1..]
            } else if line.starts_with('-') {
                &line[1..]
            } else {
                line
            };

            if content.trim().starts_with("[dependencies]") {
                in_deps = true;
                continue;
            }
            if content.trim().starts_with('[') {
                in_deps = false;
                continue;
            }
            if !in_deps {
                continue;
            }

            if let Some(name) = content
                .split('=')
                .next()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty() && !s.starts_with('#'))
            {
                if line.starts_with('+') {
                    changes.push(DepChange {
                        name: name.to_string(),
                        old_version: None,
                        new_version: Some("added".to_string()),
                    });
                } else if line.starts_with('-') {
                    changes.push(DepChange {
                        name: name.to_string(),
                        old_version: Some("removed".to_string()),
                        new_version: None,
                    });
                }
            }
        }

        changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_git_repo() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_path_buf();

        // Init git repo with explicit main branch
        std::process::Command::new("git")
            .args([
"-C",
&root.to_string_lossy(),
"init",
"--initial-branch=main",
])
.output()
    .expect("git init");

        std::process::Command::new("git")
            .args([
                "-C",
                &root.to_string_lossy(),
                "config",
                "user.email",
                "test@test.com",
            ])
            .output()
            .expect("git config email");

        std::process::Command::new("git")
            .args([
                "-C",
                &root.to_string_lossy(),
                "config",
                "user.name",
                "Test",
            ])
            .output()
            .expect("git config name");

        (dir, root)
    }

    fn git_commit(root: &Path, msg: &str) {
        std::process::Command::new("git")
            .args(["-C", &root.to_string_lossy(), "add", "-A"])
            .output()
            .expect("git add");
        std::process::Command::new("git")
            .args(["-C", &root.to_string_lossy(), "commit", "-m", msg])
            .output()
            .expect("git commit");
    }

    fn git_branch(root: &Path, name: &str) {
        std::process::Command::new("git")
            .args(["-C", &root.to_string_lossy(), "branch", name])
            .output()
            .expect("git branch");
    }

    #[test]
    fn analyze_empty_repo() {
        let (_tmp, root) = temp_git_repo();
        fs::write(root.join("main.rs"), "fn main() {}").unwrap();
        git_commit(&root, "initial");
    }

    #[test]
    fn analyze_detects_changed_files() {
        let (_tmp, root) = temp_git_repo();
        fs::write(root.join("main.rs"), "fn main() {}").unwrap();
        git_commit(&root, "initial");
    }

    #[test]
    fn detect_pub_api_changes_ignores_crate_visibility() {
        let diff = "\
--- a/lib.rs
+++ b/lib.rs
@@ -1 +1,2 @@
+pub(crate) fn internal() {}
+pub fn external() {}
";
        let changes = DiffAnalyzer::detect_pub_api_changes(diff);
        assert_eq!(changes.len(), 1, "only pub (non-crate) items");
        assert_eq!(changes[0].name, "external");
    }

    #[test]
    fn detect_module_changes_mod_rs() {
        let files = vec![
            "src/main.rs".to_string(),
            "src/modules/mod.rs".to_string(),
        ];
        let changes = DiffAnalyzer::detect_module_changes(&files);
        assert!(
            changes.iter().any(|m| m.path == "src/modules"),
            "should detect src/modules as a module change"
        );
    }

    #[test]
    fn detect_module_changes_top_level_dir() {
        let files = vec![
            "src/main.rs".to_string(),
            "tests/test_a.rs".to_string(),
        ];
        let changes = DiffAnalyzer::detect_module_changes(&files);
        assert!(
            changes.iter().any(|m| m.path == "src"),
            "should detect src/ directory"
        );
        assert!(
            changes.iter().any(|m| m.path == "tests"),
            "should detect tests/ directory"
        );
    }

    #[test]
    fn extract_pub_item_various_keywords() {
        assert_eq!(
            DiffAnalyzer::extract_pub_item("pub fn hello()"),
            Some("hello".into())
        );
        assert_eq!(
            DiffAnalyzer::extract_pub_item("pub struct User"),
            Some("User".into())
        );
        assert_eq!(
            DiffAnalyzer::extract_pub_item("pub trait Display"),
            Some("Display".into())
        );
        assert_eq!(
            DiffAnalyzer::extract_pub_item("pub enum Color"),
            Some("Color".into())
        );
        assert_eq!(
            DiffAnalyzer::extract_pub_item("pub mod utils"),
            Some("utils".into())
        );
        assert_eq!(
            DiffAnalyzer::extract_pub_item("pub type Id = u64"),
            Some("Id".into())
        );
    }

    #[test]
    fn extract_pub_item_non_pub() {
        assert_eq!(DiffAnalyzer::extract_pub_item("fn private()"), None);
        assert_eq!(DiffAnalyzer::extract_pub_item("struct Foo"), None);
        assert_eq!(
            DiffAnalyzer::extract_pub_item("pub(crate) fn internal()"),
            None
        );
        assert_eq!(DiffAnalyzer::extract_pub_item("// pub fn commented"), None);
    }

    #[test]
    fn get_changed_files_error() {
        // Non-git directory should give an error
        let tmp = tempfile::tempdir().expect("tempdir");
        let result = DiffAnalyzer::get_changed_files(tmp.path(), "main");
        assert!(result.is_err(), "non-git dir should error");
    }
}
