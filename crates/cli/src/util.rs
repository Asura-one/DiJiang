use anyhow::anyhow;
use std::path::{Path, PathBuf};

/// Resolve `.dijiang/` for the current working tree.
///
/// Order matters for task worktrees (no local `.dijiang`, gitignored):
/// 1. Walk up from `cwd` but **stop at the git worktree root** (do not climb into
///    `$HOME/.dijiang`, which would steal discovery from the project).
/// 2. Scan sibling git worktrees; prefer one that has `.dijiang/`.
/// 3. Last resort: unrestricted upward walk (legacy non-git layouts).
pub fn resolve_dijiang_dir(cwd: &Path) -> Option<PathBuf> {
    if let Some(root) = git_worktree_root(cwd).ok().flatten() {
        if let Some(dir) = find_dijiang_under(cwd, Some(&root)) {
            return Some(dir);
        }
        if let Some(dir) = find_dijiang_across_git_worktrees(cwd).ok().flatten() {
            return Some(dir);
        }
        // Unrelated clone path: fall through.
    }
    dijiang_task::store::find_dijiang_dir(cwd)
}

fn find_dijiang_under(start: &Path, stop_at: Option<&Path>) -> Option<PathBuf> {
    let stop = stop_at.and_then(|p| std::fs::canonicalize(p).ok());
    let mut dir = Some(start.to_path_buf());
    while let Some(d) = dir {
        let candidate = d.join(".dijiang");
        if candidate.is_dir() {
            return Some(candidate);
        }
        let legacy = d.join(".trellis");
        if legacy.is_dir() {
            return Some(legacy);
        }
        if let Some(ref stop) = stop {
            if let Ok(canon) = std::fs::canonicalize(&d) {
                if canon == *stop {
                    break;
                }
            }
        }
        dir = d.parent().map(|p| p.to_path_buf());
    }
    None
}

/// 获取项目 .dijiang/ 目录（失败时返回错误）
pub fn require_dijiang_dir() -> anyhow::Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    resolve_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow!("未找到 .dijiang/ 目录。请先运行 `dijiang init`。"))
}

/// Scan `git worktree list` for a checkout that contains `.dijiang/`.
pub fn find_dijiang_across_git_worktrees(project_root: &Path) -> anyhow::Result<Option<PathBuf>> {
    let output = std::process::Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(project_root)
        .output()?;
    if !output.status.success() {
        return Ok(None);
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

pub fn run_git(project_root: &Path, args: &[&str]) -> anyhow::Result<String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(project_root)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git {} failed: {}", args.join(" "), stderr.trim());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

pub fn read_developer(dijiang_dir: &Path) -> anyhow::Result<String> {
    Ok(dijiang_task::config::read_developer(dijiang_dir)
        .unwrap_or_else(|| "developer".to_string()))
}

pub fn read_project_name(dijiang_dir: &Path) -> anyhow::Result<String> {
    Ok(dijiang_task::config::read_project_name(dijiang_dir)
        .unwrap_or_else(|| "unknown".to_string()))
}

pub fn current_session_key() -> (String, String) {
    dijiang_task::store::current_session_identity()
        .map(|identity| (identity.key().to_string(), identity.source().to_string()))
        .unwrap_or_else(|| ("global_global".to_string(), "global".to_string()))
}

pub fn trim_required(value: Option<&str>, message: &str) -> anyhow::Result<String> {
    value
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("{}", message))
}

pub fn git_worktree_root(cwd: &Path) -> anyhow::Result<Option<PathBuf>> {
    let output = run_git(cwd, &["rev-parse", "--show-toplevel"])?;
    let path = output.trim();
    if path.is_empty() {
        Ok(None)
    } else {
        Ok(Some(PathBuf::from(path)))
    }
}

pub fn git_current_branch(project_root: &Path) -> anyhow::Result<String> {
    run_git(project_root, &["rev-parse", "--abbrev-ref", "HEAD"])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;

    fn git(cwd: &Path, args: &[&str]) {
        let st = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .status()
            .expect("git");
        assert!(st.success(), "git {args:?} failed");
    }

    #[test]
    fn resolve_dijiang_dir_finds_main_worktree_from_task_worktree() {
        let tmp = tempfile::TempDir::new().unwrap();
        let main = tmp.path().join("main");
        fs::create_dir_all(&main).unwrap();
        git(&main, &["init"]);
        git(&main, &["config", "user.email", "t@t.local"]);
        git(&main, &["config", "user.name", "T"]);
        let _ = Command::new("git")
            .args(["branch", "-m", "main"])
            .current_dir(&main)
            .status();
        fs::create_dir_all(main.join(".dijiang")).unwrap();
        fs::write(main.join(".dijiang/config.toml"), "project = \"p\"\n").unwrap();
        fs::write(main.join("base.txt"), "b").unwrap();
        git(&main, &["add", "base.txt"]);
        git(&main, &["commit", "-m", "base"]);

        let linked = tmp.path().join("task-wt");
        git(
            &main,
            &[
                "worktree",
                "add",
                "-b",
                "feat/task",
                linked.to_str().unwrap(),
            ],
        );
        assert!(!linked.join(".dijiang").exists());

        let found = resolve_dijiang_dir(&linked).expect("should find main .dijiang");
        let expected = main.join(".dijiang");
        assert_eq!(
            fs::canonicalize(&found).unwrap(),
            fs::canonicalize(&expected).unwrap()
        );
    }

    #[test]
    fn resolve_dijiang_dir_does_not_prefer_home_dot_dijiang_over_project() {
        // Simulate: task worktree has no .dijiang; parent-of-repo style home dir has one.
        let tmp = tempfile::TempDir::new().unwrap();
        let homeish = tmp.path().join("home");
        let main = homeish.join("proj");
        fs::create_dir_all(&main).unwrap();
        fs::create_dir_all(homeish.join(".dijiang")).unwrap();
        fs::write(homeish.join(".dijiang/active_task.txt"), "home-stolen\n").unwrap();

        git(&main, &["init"]);
        git(&main, &["config", "user.email", "t@t.local"]);
        git(&main, &["config", "user.name", "T"]);
        let _ = Command::new("git")
            .args(["branch", "-m", "main"])
            .current_dir(&main)
            .status();
        fs::create_dir_all(main.join(".dijiang")).unwrap();
        fs::write(main.join(".dijiang/active_task.txt"), "project-task\n").unwrap();
        fs::write(main.join("base.txt"), "b").unwrap();
        git(&main, &["add", "base.txt"]);
        git(&main, &["commit", "-m", "base"]);

        let linked = homeish.join("proj-task-wt");
        git(
            &main,
            &[
                "worktree",
                "add",
                "-b",
                "feat/task",
                linked.to_str().unwrap(),
            ],
        );

        let found = resolve_dijiang_dir(&linked).expect("project .dijiang");
        let active = fs::read_to_string(found.join("active_task.txt")).unwrap();
        assert!(
            active.contains("project-task"),
            "must not pick homeish .dijiang: found={}, active={}",
            found.display(),
            active
        );
    }
}
