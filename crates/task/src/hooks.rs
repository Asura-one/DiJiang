use std::collections::HashMap;
use std::path::Path;

/// Hook events supported by the DiJiang lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookEvent {
    AfterTaskCreate,
    AfterTaskStart,
    AfterTaskFinish,
    AfterTaskArchive,
}

impl HookEvent {
    /// The TOML key in the `[hooks]` section.
    pub fn config_key(&self) -> &'static str {
        match self {
            HookEvent::AfterTaskCreate => "after_task_create",
            HookEvent::AfterTaskStart => "after_task_start",
            HookEvent::AfterTaskFinish => "after_task_finish",
            HookEvent::AfterTaskArchive => "after_task_archive",
        }
    }

    /// All known hook events for discovery.
    pub fn all() -> &'static [HookEvent] {
        &[
            HookEvent::AfterTaskCreate,
            HookEvent::AfterTaskStart,
            HookEvent::AfterTaskFinish,
            HookEvent::AfterTaskArchive,
        ]
    }
}

/// Parsed hook configuration for a project.
#[derive(Debug, Clone)]
pub struct HooksConfig {
    /// Per-event shell commands. Empty string means the hook is disabled.
    commands: HashMap<String, String>,
}

impl HooksConfig {
    /// Parse hooks from TOML config content.
    pub fn parse(toml_content: &str) -> Self {
        let mut commands = HashMap::new();
        let mut in_hooks = false;

        for line in toml_content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("[hooks]") {
                in_hooks = true;
                continue;
            }
            // Exit hooks section when another section starts
            if in_hooks && trimmed.starts_with('[') {
                in_hooks = false;
                continue;
            }
            if in_hooks {
                if let Some((key, value)) = parse_key_value(trimmed) {
                    let value = value.trim().trim_matches('"').to_string();
                    commands.insert(key, value);
                }
            }
        }

        HooksConfig { commands }
    }

    /// Load hooks from a DiJiang config directory using the structured config module.
    pub fn load(dijiang_dir: &Path) -> Self {
        use crate::config;
        match config::read_hooks_config(dijiang_dir) {
            Some(section) => {
                let mut commands = HashMap::new();
                if let Some(cmd) = section.after_task_create {
                    commands.insert("after_task_create".to_string(), cmd);
                }
                if let Some(cmd) = section.after_task_start {
                    commands.insert("after_task_start".to_string(), cmd);
                }
                if let Some(cmd) = section.after_task_finish {
                    commands.insert("after_task_finish".to_string(), cmd);
                }
                if let Some(cmd) = section.after_task_archive {
                    commands.insert("after_task_archive".to_string(), cmd);
                }
                HooksConfig { commands }
            }
            None => HooksConfig::default(),
        }
    }

    /// Get the command for a hook event, if configured and non-empty.
    pub fn command_for(&self, event: HookEvent) -> Option<&str> {
        let cmd = self.commands.get(event.config_key())?;
        if cmd.is_empty() { None } else { Some(cmd.as_str()) }
    }

    /// Returns true if any hooks are configured.
    pub fn is_enabled(&self) -> bool {
        self.commands.values().any(|c| !c.is_empty())
    }

    /// Iterate over configured (event_key, command) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        let all_events = HookEvent::all();
        let mut pairs = Vec::new();
        for event in all_events {
            if let Some(cmd) = self.command_for(*event) {
                pairs.push((event.config_key(), cmd));
            }
        }
        pairs.into_iter()
    }
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self { commands: HashMap::new() }
    }
}

fn parse_key_value(s: &str) -> Option<(String, String)> {
    let eq_idx = s.find('=')?;
    let key = s[..eq_idx].trim().to_string();
    let value = s[eq_idx + 1..].trim().to_string();
    if key.is_empty() || value.is_empty() { None } else { Some((key, value)) }
}

/// Load hooks configuration from the project's config.toml.
pub fn load_hooks_config(dijiang_dir: &Path) -> HooksConfig {
    let config_path = dijiang_dir.join("config.toml");
    let content = std::fs::read_to_string(config_path).unwrap_or_default();
    HooksConfig::parse(&content)
}

/// Run all hooks for the given event.
///
/// Each configured hook command is executed as a shell command.
/// The environment variable `DIJIANG_TASK_JSON_PATH` is set to the task directory.
///
/// Non-zero exit codes from hook commands are logged as warnings, not errors.
pub fn run_task_hooks(
    dijiang_dir: &Path,
    event: HookEvent,
    task_name: &str,
) {
    let hooks = load_hooks_config(dijiang_dir);
    let Some(cmd) = hooks.command_for(event) else { return };

    let task_json_path = dijiang_dir
        .join("tasks")
        .join(task_name)
        .join("task.json");

    // Execute the hook as a shell command
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .env("DIJIANG_TASK_JSON_PATH", task_json_path.to_string_lossy().as_ref())
        .output();

    match output {
        Ok(out) => {
            if !out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr);
                if !stderr.trim().is_empty() {
                    eprintln!("[hook:{}] stderr: {}", event.config_key(), stderr.trim());
                }
                eprintln!(
                    "[hook:{}] exited with code {}",
                    event.config_key(),
                    out.status.code().unwrap_or(-1)
                );
            }
        }
        Err(e) => {
            eprintln!("[hook:{}] failed to execute: {}", event.config_key(), e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_config() {
        let config = HooksConfig::parse("");
        assert!(!config.is_enabled());
        assert!(config.command_for(HookEvent::AfterTaskCreate).is_none());
    }

    #[test]
    fn test_parse_no_hooks_section() {
        let config = HooksConfig::parse("[project]\nname = \"test\"\n");
        assert!(!config.is_enabled());
    }

    #[test]
    fn test_parse_hooks_section() {
        let toml = r#"
[project]
name = "test"

[hooks]
after_task_create = "echo hello"
after_task_start = ""
"#;
        let config = HooksConfig::parse(toml);
        assert!(config.is_enabled());
        assert_eq!(config.command_for(HookEvent::AfterTaskCreate), Some("echo hello"));
        // Empty command should be treated as disabled
        assert!(config.command_for(HookEvent::AfterTaskStart).is_none());
        // Unconfigured events should be None
        assert!(config.command_for(HookEvent::AfterTaskFinish).is_none());
    }

    #[test]
    fn test_iter() {
        let toml = r#"
[hooks]
after_task_create = "echo a"
after_task_finish = "echo b"
"#;
        let config = HooksConfig::parse(toml);
        let pairs: Vec<_> = config.iter().collect();
        assert_eq!(pairs.len(), 2);
    }

    #[test]
    fn test_run_hooks_disabled() {
        // Should not crash when no hooks are configured
        let dir = tempfile::tempdir().unwrap();
        run_task_hooks(dir.path(), HookEvent::AfterTaskCreate, "test-task");
    }

    #[test]
    fn test_parse_key_value() {
        assert_eq!(
            parse_key_value("key = \"value\""),
            Some(("key".to_string(), "\"value\"".to_string()))
        );
        assert_eq!(parse_key_value("invalid"), None);
        assert_eq!(parse_key_value("= \"only value\""), None);
    }

    #[test]
    fn test_command_for_unknown_event() {
        let config = HooksConfig::default();
        assert!(config.command_for(HookEvent::AfterTaskCreate).is_none());
        assert!(config.command_for(HookEvent::AfterTaskStart).is_none());
        assert!(config.command_for(HookEvent::AfterTaskFinish).is_none());
        assert!(config.command_for(HookEvent::AfterTaskArchive).is_none());
    }
}
