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
- `.dijiang/tasks/` — Task records (JSON, Trellis-compatible format)
- `.dijiang/spec/` — Project specifications and coding guidelines
- `.dijiang/workspace/` — Developer journals and session traces
- `.pi/` — Pi agent platform configuration
- `crates/` — Rust workspace crates (core, cli, task, mem, configurator)

## Task Workflow

Tasks live in `.dijiang/tasks/<name>/` with these artifacts:
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

1. **plan**: Start by reading `dijiang workflow-state --json`, and treat injected `Skill Manifests` plus `<dijiang-target-skill ...>` as the primary runtime routing context before reading `prd.md`, `design.md`, `implement.md`.
2. **implement**: Write code following the runtime context and specs. Run `cargo build` to verify.
3. **check**: Run `cargo test`, verify types, lint.
4. **archive**: Commit changes when done.

## CLI Commands

- `dijiang status` — 项目概览
- `dijiang task list` — 所有任务
- `dijiang task current` — 当前任务
- `dijiang start <name>` — Start task
- `dijiang workflow-state --json` — Load injected runtime route + target skill context
- `cargo build -p dijiang-cli` — Build CLI
- `cargo test` — Run all tests
"#,
            project_name = project_name
        )
    }
    fn hook_script_content() -> &'static str {
        r#"#!/usr/bin/env python3
"""Proxies to `.dijiang/scripts/workflow_state.py` — no dijiang CLI dependency."""
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def find_dijiang_root(start: Path) -> Path | None:
    current = start.resolve()
    while True:
        if (current / ".dijiang").is_dir():
            return current
        if current == current.parent:
            return None
        current = current.parent


def main() -> int:
    root = find_dijiang_root(Path.cwd())
    if root is None:
        return 0

    script = root / ".dijiang" / "scripts" / "workflow_state.py"
    if not script.is_file():
        return 0

    try:
        stdin_data = sys.stdin.read()
    except OSError:
        stdin_data = ""

    try:
        result = subprocess.run(
            ["python3", str(script)],
            input=stdin_data,
            text=True,
            cwd=root,
            check=False,
            capture_output=True,
            timeout=10,
        )
    except subprocess.TimeoutExpired:
        return 0
    except (subprocess.CalledProcessError, FileNotFoundError):
        return 0

    if result.returncode == 0 and result.stdout.strip():
        print(result.stdout.strip())
    return 0


if __name__ == "__main__":
    sys.exit(main())
"#
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

        // Write .claude/settings.json
        let claude_dir = cwd.join(".claude");
        fs::create_dir_all(&claude_dir)?;

        let settings = r#"{
  "skills": {
    "enabled": true
  },
  "hooks": {
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "python3 -X utf8 .claude/hooks/inject-workflow-state.py",
            "timeout": 15
          }
        ]
      }
    ]
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
        let hooks_dir = claude_dir.join("hooks");
        fs::create_dir_all(&hooks_dir)?;
        let hook_path = hooks_dir.join("inject-workflow-state.py");
        fs::write(&hook_path, Self::hook_script_content())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&hook_path)?.permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&hook_path, permissions)?;
        }
        Ok(())
    }

    fn has_hooks(&self) -> bool {
        true
    }
}
