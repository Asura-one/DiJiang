use serde::{Deserialize, Serialize};
use std::path::Path;

/// Errors from configurator operations.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialize error: {0}")]
    Serialize(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

/// Platform configurator trait.
///
/// Each platform has its own configurator that generates
/// the necessary configuration files for that platform.
pub trait Configurator: Send + Sync {
    /// Platform name (e.g. "pi", "cursor", "claude", "codex").
    fn platform(&self) -> &str;

    /// Configure a DiJiang project at `cwd`.
    ///
    /// Creates/updates platform-specific config files.
    fn configure(&self, cwd: &Path) -> Result<(), ConfigError>;

    /// Returns true if the platform supports auto-injection (class-1).
    fn has_hooks(&self) -> bool {
        true
    }
}

/// DiJiang project config (`~/.dijiang/config.toml` on user level,
/// `.dijiang/config.toml` on project level).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DijiangConfig {
    /// Project metadata
    pub project: ProjectConfig,

    /// Enabled platforms (generated config files for these)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platforms: Option<Vec<String>>,

    /// Tasks directory (relative to project root)
    #[serde(default = "default_tasks_dir")]
    pub tasks_dir: String,

    /// Spec directory
    #[serde(default = "default_spec_dir")]
    pub spec_dir: String,

    /// Workspace directory
    #[serde(default = "default_workspace_dir")]
    pub workspace_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project name
    pub name: String,

    /// Description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Default developer name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub developer: Option<String>,

    /// Version
    #[serde(default = "default_version")]
    pub version: String,
}

fn default_tasks_dir() -> String {
    ".trellis/tasks".to_string()
}

fn default_spec_dir() -> String {
    ".trellis/spec".to_string()
}

fn default_workspace_dir() -> String {
    ".trellis/workspace".to_string()
}

fn default_version() -> String {
    "0.1.0".to_string()
}

impl Default for DijiangConfig {
    fn default() -> Self {
        Self {
            project: ProjectConfig::default(),
            platforms: None,
            tasks_dir: default_tasks_dir(),
            spec_dir: default_spec_dir(),
            workspace_dir: default_workspace_dir(),
        }
    }
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: "my-project".to_string(),
            description: None,
            developer: None,
            version: default_version(),
        }
    }
}

/// Pi platform settings (`.pi/settings.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiSettings {
    #[serde(default = "default_true")]
    pub enable_skill_commands: bool,

    #[serde(default)]
    pub extensions: Vec<String>,

    #[serde(default)]
    pub skills: Vec<String>,

    #[serde(default)]
    pub prompts: Vec<String>,

    #[serde(default)]
    pub agents: Vec<String>,
}

fn default_true() -> bool {
    true
}
