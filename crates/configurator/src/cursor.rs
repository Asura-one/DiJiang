use crate::{ConfigError, Configurator, PlatformKind};
use std::fs;
use std::path::Path;

/// Cursor configurator — writes `.cursor/rules/` and `hooks.json`.
///
/// Cursor is a class-1 (hasHooks=true) platform. Rules auto-inject into
/// agent context on session start.
pub struct CursorConfigurator;

impl CursorConfigurator {
    pub fn new() -> Self {
        Self
    }

    fn rules_content() -> &'static str {
        r#"---
description: DiJiang project workflow and conventions
globs: ["*"]
---
# DiJiang Project

This project uses DiJiang, an AI-native development workflow framework.

## Structure

- `.dijiang/` — DiJiang project configuration
- `.trellis/tasks/` — Task records (JSON, Trellis-compatible format)
- `.trellis/spec/` — Project specifications and coding guidelines
- `.trellis/workspace/` — Developer journals and session traces
- `.pi/` — Pi agent platform configuration

## Core Workflow (dj-*)

Task lifecycle follows `plan → implement → check → archive`:

1. **plan** — Read `prd.md` (requirements) + `design.md` (design) + `implement.md` (execution plan)
2. **implement** — Write code, following specs in `.trellis/spec/`
3. **check** — Run `cargo test`, verify types, lint, verify cross-layer consistency
4. **archive** — Commit changes

## Commands

- `dijiang status` — Project overview + active task
- `dijiang task list` — List all tasks
- `dijiang task current` — Show active task
- `dijiang mem list` — Cross-platform session memory
- `dijiang start <name> [title]` — Start a new session/task
- `cargo build` / `cargo test` — Build and test Rust code

## Hooks

Hooks fire automatically on session start to load task context.
"#
    }

    fn hooks_json_content() -> &'static str {
        r#"{
  "hooks": [
    {
      "type": "session:start",
      "command": "dijiang status",
      "description": "Load DiJiang project context on session start"
    }
  ]
}"#
    }
}

impl Configurator for CursorConfigurator {
    fn platform(&self) -> PlatformKind {
        PlatformKind::Cursor
    }

    fn is_installed(&self) -> bool {
        std::env::var_os("HOME")
            .map(|h| std::path::Path::new(&h).join(".cursor").exists())
            .unwrap_or(false)
    }

    fn configure(&self, cwd: &Path) -> Result<(), ConfigError> {
        let rules_dir = cwd.join(".cursor").join("rules");

        // Write rule file
        fs::create_dir_all(&rules_dir)?;
        let rule_path = rules_dir.join("dijiang.mdc");
        fs::write(&rule_path, Self::rules_content())?;

        // Write hooks.json
        let hooks_path = cwd.join(".cursor").join("hooks.json");
        fs::write(&hooks_path, Self::hooks_json_content())?;

        eprintln!("  ├── .cursor/rules/dijiang.mdc");
        eprintln!("  ├── .cursor/hooks.json");

        Ok(())
    }

    fn has_hooks(&self) -> bool {
        true
    }
}
