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

    /// `[workflow]` section (project-specific route defaults)
    pub workflow: Option<WorkflowConfig>,
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

fn default_skill_is_allowed(status: &str, skill: &str) -> bool {
    matches!(
        (status, skill),
        (
            "planning",
            "dj-grill" | "dj-output" | "dj-reason" | "dj-research"
        ) | (
            "in_progress",
            "dj-implement"
                | "dj-script"
                | "dj-tdd"
                | "dj-hunt"
                | "dj-check"
                | "dj-review"
                | "dj-audit"
                | "dj-output"
                | "dj-grill"
                | "dj-gov"
                | "dj-reason"
                | "dj-research"
        ) | (
            "completed",
            "dijiang-finish-work"
                | "dj-gov"
                | "dj-check"
                | "dj-review"
                | "dj-audit"
                | "dj-output"
                | "dj-grill"
                | "dj-reason"
                | "dj-research"
        )
    )
}

/// `[workflow]` section — optional default skills by task status.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WorkflowConfig {
    /// Skill to use when a planning task is resumed.
    pub planning_default_skill: Option<String>,

    /// Skill to use when an in-progress task is resumed.
    pub in_progress_default_skill: Option<String>,

    /// Skill to use when a completed task is resumed.
    pub completed_default_skill: Option<String>,

    /// How `dj-grill` should converge: `adaptive`, `grill-me`, or `grill-with-doc`.
    pub grill_mode: Option<String>,
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

/// Return a configured default skill for the task status when it names a registered skill.
pub fn workflow_default_skill(dijiang_dir: &Path, status: &str) -> Option<String> {
    let workflow = load_config(dijiang_dir).workflow?;
    let skill = match status {
        "planning" => workflow.planning_default_skill,
        "in_progress" => workflow.in_progress_default_skill,
        "completed" => workflow.completed_default_skill,
        _ => None,
    }?;
    let route = crate::skill_route(&skill)?;
    if default_skill_is_allowed(status, route.name) {
        Some(route.name.to_string())
    } else {
        None
    }
}

/// Return the configured grill convergence mode, falling back to `adaptive`.
pub fn grill_mode(dijiang_dir: &Path) -> String {
    match load_config(dijiang_dir)
        .workflow
        .and_then(|workflow| workflow.grill_mode)
    {
        Some(mode) if matches!(mode.as_str(), "adaptive" | "grill-me" | "grill-with-doc") => mode,
        _ => "adaptive".to_string(),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_workflow_defaults_accept_registered_skills() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("config.toml"),
            "[workflow]\nplanning_default_skill = \"dj-reason\"\ngrill_mode = \"grill-with-doc\"\n",
        )
        .unwrap();

        assert_eq!(
            workflow_default_skill(dir.path(), "planning").as_deref(),
            Some("dj-reason")
        );
        assert_eq!(grill_mode(dir.path()), "grill-with-doc");
    }

    #[test]
    fn registered_skill_outside_status_allowlist_falls_back() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("config.toml"),
            "[workflow]\nplanning_default_skill = \"dj-implement\"\n",
        )
        .unwrap();

        assert_eq!(workflow_default_skill(dir.path(), "planning"), None);
    }

    #[test]
    fn invalid_workflow_values_fall_back_to_safe_defaults() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("config.toml"),
            "[workflow]\nplanning_default_skill = \"shell\"\ngrill_mode = \"questionnaire\"\n",
        )
        .unwrap();

        assert_eq!(workflow_default_skill(dir.path(), "planning"), None);
        assert_eq!(grill_mode(dir.path()), "adaptive");
    }
}
