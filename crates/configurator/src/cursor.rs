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
- `.dijiang/tasks/` — Task records (JSON, Trellis-compatible format)
- `.dijiang/spec/` — Project specifications and coding guidelines
- `.dijiang/workspace/` — Developer journals and session traces
- `.pi/` — Pi agent platform configuration

## Core Workflow (dj-*)

Task lifecycle follows `plan → implement → check → archive`:

1. **plan** — Read `dijiang workflow-state --json` first, and treat injected `Skill Manifests` plus `<dijiang-target-skill ...>` as the primary runtime routing context before reading `prd.md` (requirements) + `design.md` (design) + `implement.md` (execution plan)
2. **implement** — Write code, following runtime context and specs in `.dijiang/spec/`
3. **check** — Run `cargo test`, verify types, lint, verify cross-layer consistency
4. **archive** — Commit changes

## Commands

- `dijiang status` — Project overview + active task
- `dijiang task list` — List all tasks
- `dijiang task current` — Show active task
- `dijiang mem list` — Cross-platform session memory
- `dijiang start <name> [title]` — Start a new session/task
- `dijiang workflow-state --json` — Load injected runtime route + target skill context
- `cargo build` / `cargo test` — Build and test Rust code

## Hooks

Hooks call `dijiang workflow-state` to load session-scoped task context, including `Skill Manifests` and `<dijiang-target-skill ...>`.
"#
    }

    fn hooks_json_content() -> &'static str {
        r#"{
  "hooks": {
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "python3 -X utf8 .cursor/hooks/inject-workflow-state.py",
            "timeout": 15
          }
        ]
      }
    ]
  }
}"#
    }
    fn hook_script_content() -> &'static str {
        r#"#!/usr/bin/env python3
from __future__ import annotations

import json
import os
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


def visible_error(message: str) -> str:
    session = (
        os.environ.get("DIJIANG_CONTEXT_ID")
        or os.environ.get("CURSOR_SESSION_ID")
        or os.environ.get("CURSOR_CONVERSATION_ID")
        or "unknown"
    )
    context = "\n".join(
        [
            "<dijiang-workflow-state>",
            "平台: cursor",
            f"会话: {session}",
            f"Hook 错误: {message}",
            "当前任务: unknown",
            "下一步: 在项目根目录运行 `dijiang workflow-state`，并确认 `dijiang` 已在 PATH 中。",
            "</dijiang-workflow-state>",
        ]
    )
    return json.dumps({"hookEventName": "UserPromptSubmit", "additionalContext": context})


def main() -> int:
    root = find_dijiang_root(Path.cwd())
    if root is None:
        return 0

    try:
        stdin_data = sys.stdin.read()
    except OSError:
        stdin_data = ""

    try:
        result = subprocess.run(
            ["dijiang", "workflow-state", "--json", "--hook-event", "UserPromptSubmit"],
            input=stdin_data,
            text=True,
            cwd=root,
            check=False,
            capture_output=True,
            timeout=10,
        )
    except FileNotFoundError:
        print(visible_error("dijiang executable not found"))
        return 0
    except subprocess.TimeoutExpired:
        print(visible_error("dijiang workflow-state timed out"))
        return 0
    except subprocess.SubprocessError as exc:
        print(visible_error(str(exc)))
        return 0

    if result.returncode == 0 and result.stdout.strip():
        print(result.stdout.strip())
    elif result.returncode != 0:
        detail = (result.stderr or result.stdout or f"exit code {result.returncode}").strip()
        print(visible_error(detail))
    return 0


if __name__ == "__main__":
    sys.exit(main())
"#
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
        // Write hooks.json + wrapper script
        let cursor_dir = cwd.join(".cursor");
        fs::create_dir_all(cursor_dir.join("hooks"))?;
        let hooks_path = cursor_dir.join("hooks.json");
        fs::write(&hooks_path, Self::hooks_json_content())?;
        let hook_path = cursor_dir.join("hooks").join("inject-workflow-state.py");
        fs::write(&hook_path, Self::hook_script_content())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&hook_path)?.permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&hook_path, permissions)?;
        }

        eprintln!("  ├── .cursor/rules/dijiang.mdc");
        eprintln!("  ├── .cursor/hooks.json");
        eprintln!("  ├── .cursor/hooks/inject-workflow-state.py");

        Ok(())
    }

    fn has_hooks(&self) -> bool {
        true
    }
}
