use crate::{ConfigError, Configurator};
use std::fs;
use std::path::Path;

/// Codex configurator — writes `.codex/agents/`, `.codex/hooks/`, `.codex/config.toml`.
///
/// Codex is a class-2 (hasHooks=false) platform. Context is pull-based —
/// sub-agents load task context via prelude instructions rather than hook injection.
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
  "hooks": [
    {
      "type": "UserPromptSubmit",
      "command": ".codex/hooks/inject-workflow-state.sh",
      "description": "Auto-inject DiJiang workflow state"
    }
  ]
}"#
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
    fn platform(&self) -> &str {
        "codex"
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
        // Write shared hook script (shell calling dijiang CLI)
        let hook_script = "#!/bin/sh
dijiang status 2>/dev/null || true";
        fs::write(hooks_dir.join("inject-workflow-state.sh"), hook_script)?;
        eprintln!("  ├── .codex/hooks/inject-workflow-state.sh");

        // ── hooks.json ──
        fs::write(codex_dir.join("hooks.json"), Self::hooks_json_content())?;
        eprintln!("  ├── .codex/hooks.json");

        // ── config.toml ──
        fs::write(codex_dir.join("config.toml"), Self::config_toml_content())?;
        eprintln!("  ├── .codex/config.toml");

        Ok(())
    }

    fn has_hooks(&self) -> bool {
        false
    }
}
