use crate::{ConfigError, Configurator, PlatformKind};
use std::fs;
use std::path::Path;

/// Claude Code configurator — writes `CLAUDE.md` and `.claude/settings.json`.
///
/// Claude is a class-1 (hasHooks=true) platform. `CLAUDE.md` is auto-read on
/// session start; `.claude/settings.json` configures slash commands.
pub struct ClaudeConfigurator;

impl ClaudeConfigurator {
    pub fn new() -> Self {
        Self
    }

    fn claude_md_content(project_name: &str) -> String {
        format!(
            r#"# {project_name} — DiJiang Project

This project uses DiJiang, an AI-native development workflow framework.

## Project Structure

- `.dijiang/` — DiJiang project configuration
- `.trellis/tasks/` — Task records (JSON, Trellis-compatible format)
- `.trellis/spec/` — Project specifications and coding guidelines
- `.trellis/workspace/` — Developer journals and session traces
- `.pi/` — Pi agent platform configuration
- `crates/` — Rust workspace crates (core, cli, task, mem, configurator)

## Task Workflow

Tasks live in `.trellis/tasks/<name>/` with these artifacts:
- `prd.md` — Requirements (mandatory)
- `design.md` — Technical design (complex tasks)
- `implement.md` — Execution plan (complex tasks)
- `task.json` — Structured task record (24 Trellis-compatible fields)

### Phases

| Status | Phase | Action |
|--------|-------|--------|
| planning | plan | Read PRD, align requirements |
| in_progress | implement | Write code |
| completed | complete | Verify quality gate |
| archived | archive | Move to archive |

## Core Workflow

1. **plan**: Start by reading `prd.md`, `design.md`, `implement.md`. Load specs from `spec/`.
2. **implement**: Write code following the specs. Run `cargo build` to verify.
3. **check**: Run `cargo test`, verify types, lint.
4. **archive**: Commit changes when done.

## CLI Commands

- `dijiang status` — Project overview
- `dijiang task list` — All tasks
- `dijiang task current` — Active task
- `dijiang start <name>` — Start task
- `cargo build -p dijiang-cli` — Build CLI
- `cargo test` — Run all tests
"#,
            project_name = project_name
        )
    }
}

impl Configurator for ClaudeConfigurator {
    fn platform(&self) -> PlatformKind {
        PlatformKind::Claude
    }

    fn is_installed(&self) -> bool {
        std::process::Command::new("claude")
            .arg("--version")
            .output()
            .ok()
            .is_some_and(|o| o.status.success())
    }

    fn configure(&self, cwd: &Path) -> Result<(), ConfigError> {
        // Read project name from DiJiang config, fallback to directory name
        let project_name = crate::init::read_project_name(cwd);

        // Write CLAUDE.md
        let claude_md = cwd.join("CLAUDE.md");
        fs::write(&claude_md, Self::claude_md_content(&project_name))?;
        eprintln!("  ├── CLAUDE.md");

        // Write .claude/settings.json
        let claude_dir = cwd.join(".claude");
        fs::create_dir_all(&claude_dir)?;

        let settings = r#"{
  "skills": {
    "enabled": true
  },
  "slash_commands": [
    {
      "name": "dijiang-status",
      "description": "Show DiJiang project status",
      "command": "dijiang status"
    },
    {
      "name": "dijiang-task-list",
      "description": "List all tasks",
      "command": "dijiang task list"
    },
    {
      "name": "dijiang-start",
      "description": "Start a task",
      "command": "dijiang start"
    }
  ]
}
"#;
        let settings_path = claude_dir.join("settings.json");
        fs::write(&settings_path, settings)?;
        eprintln!("  ├── .claude/settings.json");

        Ok(())
    }

    fn has_hooks(&self) -> bool {
        true
    }
}
