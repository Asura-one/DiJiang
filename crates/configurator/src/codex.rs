use crate::{ConfigError, Configurator, PlatformKind};
use std::fs;
use std::path::Path;

/// Codex configurator — writes `.codex/agents/`, `.codex/hooks/`, `.codex/config.toml`.
///
/// Codex uses a UserPromptSubmit hook to inject DiJiang workflow state every turn.
pub struct CodexConfigurator;

impl CodexConfigurator {
    pub fn new() -> Self {
        Self
    }

    fn agent_content() -> &'static str {
        r#"[agent]
name = "dijiang-implement"
version = "1.0"
description = "DiJiang implementation agent"

[agent.system_prompt]
content = """
You are a DiJiang implementation sub-agent. Your job is to implement code changes based on task context.

Required: Load task context first
1. Find active task from dispatch prompt or `dijiang task current`
2. Read prd.md, design.md, implement.md from the task directory
3. Load implement.jsonl for spec references
4. Implement the changes, run cargo build/cargo test to verify
"""

[agent.tools]
enabled = ["Read", "Write", "Edit", "Bash", "Glob", "Grep"]
"#
    }

    fn check_agent_content() -> &'static str {
        r#"[agent]
name = "dijiang-check"
version = "1.0"
description = "DiJiang quality check agent"

[agent.system_prompt]
content = """
You are a DiJiang quality check sub-agent. Your job is to verify code quality.

Required: Load task context first
1. Find active task from dispatch prompt or `dijiang task current`
2. Read prd.md for acceptance criteria
3. Load check.jsonl for spec references
4. Verify: cargo test, cargo build, spec compliance
"""

[agent.tools]
enabled = ["Read", "Bash", "Glob", "Grep"]
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
            "command": "python3 -X utf8 .codex/hooks/inject-workflow-state.py",
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
    except (OSError, subprocess.SubprocessError):
        return 0

    if result.returncode == 0 and result.stdout.strip():
        print(result.stdout.strip())
    return 0


if __name__ == "__main__":
    sys.exit(main())
"#
    }

    fn config_toml_content() -> &'static str {
        r#"[agent]
default_agent = "dijiang-implement"

[features]
hooks = true
"#
    }
}

impl Configurator for CodexConfigurator {
    fn platform(&self) -> PlatformKind {
        PlatformKind::Codex
    }

    fn is_installed(&self) -> bool {
        std::process::Command::new("codex")
            .arg("--version")
            .output()
            .ok()
            .is_some_and(|o| o.status.success())
    }

    fn configure(&self, cwd: &Path) -> Result<(), ConfigError> {
        let codex_dir = cwd.join(".codex");

        // ── agents/ ──
        let agents_dir = codex_dir.join("agents");
        fs::create_dir_all(&agents_dir)?;

        fs::write(
            agents_dir.join("dijiang-implement.toml"),
            Self::agent_content(),
        )?;
        fs::write(
            agents_dir.join("dijiang-check.toml"),
            Self::check_agent_content(),
        )?;
        eprintln!("  ├── .codex/agents/dijiang-implement.toml");
        eprintln!("  ├── .codex/agents/dijiang-check.toml");

        // ── hooks/ ──
        let hooks_dir = codex_dir.join("hooks");
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
        eprintln!("  ├── .codex/hooks/inject-workflow-state.py");

        // ── hooks.json ──
        fs::write(codex_dir.join("hooks.json"), Self::hooks_json_content())?;
        eprintln!("  ├── .codex/hooks.json");

        // ── config.toml ──
        fs::write(codex_dir.join("config.toml"), Self::config_toml_content())?;
        eprintln!("  ├── .codex/config.toml");

        Ok(())
    }

    fn has_hooks(&self) -> bool {
        true
    }
}
