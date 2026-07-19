use crate::config;
use std::path::{Path, PathBuf};

// ── Resolve developer identity ─────────────────────────────────────

/// Resolve the active developer name.
///
/// Priority order:
/// 1. `[project].developer` in `.dijiang/config.toml`
/// 2. `DIJIANG_DEVELOPER` environment variable
/// 3. `USER` / `USERNAME` environment variable
/// 4. Fallback to `"developer"`
pub fn resolve_developer(dijiang_dir: &Path) -> String {
    // 1. Config
    if let Some(dev) = config::read_developer(dijiang_dir) {
        return dev;
    }
    // 2. Env
    if let Ok(dev) = std::env::var("DIJIANG_DEVELOPER") {
        if !dev.is_empty() {
            return dev;
        }
    }
    // 3. OS user
    if let Ok(user) = std::env::var("USER") {
        if !user.is_empty() {
            return user;
        }
    }
    if let Ok(user) = std::env::var("USERNAME") {
        if !user.is_empty() {
            return user;
        }
    }
    // 4. Fallback
    "developer".to_string()
}

// ── Path helpers ───────────────────────────────────────────────────

/// Path to the developer's workspace directory (`<dijiang_dir>/workspace/<developer>/`).
pub fn workspace_dir(dijiang_dir: &Path, developer: &str) -> PathBuf {
    dijiang_dir.join("workspace").join(developer)
}

/// Path to the developer's sessions directory (`<dijiang_dir>/workspace/<developer>/sessions/`).
pub fn sessions_dir(dijiang_dir: &Path, developer: &str) -> PathBuf {
    workspace_dir(dijiang_dir, developer).join("sessions")
}

/// Path to the developer's journal file (`<dijiang_dir>/workspace/<developer>/journal.md`).
pub fn journal_path(dijiang_dir: &Path, developer: &str) -> PathBuf {
    workspace_dir(dijiang_dir, developer).join("journal.md")
}

/// Resolve the developer identity **and** return the associated workspace paths.
pub struct DeveloperContext {
    pub name: String,
    pub workspace: PathBuf,
    pub sessions: PathBuf,
    pub journal: PathBuf,
}

impl DeveloperContext {
    /// Build a `DeveloperContext` for the given DiJiang directory.
    ///
    /// Resolves the developer identity automatically via `resolve_developer`.
    pub fn new(dijiang_dir: &Path) -> Self {
        let name = resolve_developer(dijiang_dir);
        let workspace = workspace_dir(dijiang_dir, &name);
        let sessions = sessions_dir(dijiang_dir, &name);
        let journal = journal_path(dijiang_dir, &name);
        DeveloperContext { name, workspace, sessions, journal }
    }
}
