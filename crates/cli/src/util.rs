use anyhow::anyhow;
use std::path::{Path, PathBuf};

/// 获取项目 .dijiang/ 目录（失败时返回错误）
pub fn require_dijiang_dir() -> anyhow::Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    dijiang_task::store::find_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow!("未找到 .dijiang/ 目录。请先运行 `dijiang init`。"))
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
