use serde::Deserialize;
use std::path::Path;

// ── Types ──────────────────────────────────────────────────────────

/// DiJiang project configuration, deserialized from `.dijiang/config.toml`.
///
/// All fields are optional because config sections may be absent or partial.
/// Top-level keys in TOML (platforms, tasks_dir, spec_dir, workspace_dir,
/// dijiang_version) are deserialized directly here, while `[project]` and
/// `[hooks]` sections use nested structs.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Platforms to generate agent files for (e.g., `["pi", "claude", "codex"]`)
    pub platforms: Vec<String>,

    /// Path to task directory (default: `.dijiang/tasks`)
    pub tasks_dir: Option<String>,

    /// Path to spec directory (default: `.dijiang/spec`)
    pub spec_dir: Option<String>,

    /// Path to workspace directory (default: `.dijiang/workspace`)
    pub workspace_dir: Option<String>,

    /// Version of DiJiang that created this project
    pub dijiang_version: Option<String>,

    /// `[project]` section
    pub project: Option<ProjectConfig>,

    /// `[hooks]` section (lifecycle event → shell command)
    pub hooks: Option<HooksSection>,
}

/// `[project]` section.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct ProjectConfig {
    /// Human-readable project name
    pub name: Option<String>,

    /// Developer identifier (e.g., username)
    pub developer: Option<String>,

    /// Project version string
    pub version: Option<String>,
}

/// `[hooks]` section — lifecycle event shell commands.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct HooksSection {
    /// Shell command to run after a task is created.
    pub after_task_create: Option<String>,

    /// Shell command to run after a task is started.
    pub after_task_start: Option<String>,

    /// Shell command to run after a task is finished.
    pub after_task_finish: Option<String>,

    /// Shell command to run after a task is archived.
    pub after_task_archive: Option<String>,
}

// ── Loading ────────────────────────────────────────────────────────

/// Load and deserialize `.dijiang/config.toml` from the given DiJiang dir.
///
/// Returns a `Config::default()` if the config file does not exist or
/// cannot be parsed.
pub fn load_config(dijiang_dir: &Path) -> Config {
    let config_path = dijiang_dir.join("config.toml");
    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return Config::default(),
    };
    toml::from_str(&content).unwrap_or_default()
}

// ── Convenience accessors ──────────────────────────────────────────

/// Read the `[project].developer` value.
pub fn read_developer(dijiang_dir: &Path) -> Option<String> {
    load_config(dijiang_dir).project?.developer
}

/// Read the `[project].name` value.
pub fn read_project_name(dijiang_dir: &Path) -> Option<String> {
    load_config(dijiang_dir).project?.name
}

/// Read the `[project].version` value.
pub fn read_project_version(dijiang_dir: &Path) -> Option<String> {
    load_config(dijiang_dir).project?.version
}

/// Read the `[hooks]` section.
pub fn read_hooks_config(dijiang_dir: &Path) -> Option<HooksSection> {
    load_config(dijiang_dir).hooks
}
