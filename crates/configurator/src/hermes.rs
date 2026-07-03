use crate::{ConfigError, Configurator, PlatformKind};
use std::fs;
use std::path::Path;

/// Hermes configurator — writes `.hermes/agents/` with DiJiang workflow instructions.
///
/// Hermes is a class-1 (hasHooks=true) CLI-based platform. Agent instructions
/// auto-inject on session start via `.hermes/agents/` configuration.
pub struct HermesConfigurator;

impl HermesConfigurator {
    pub fn new() -> Self {
        Self
    }

    fn implement_agent_content() -> &'static str {
        r#"---
name: trellis-implement
description: DiJiang implementation agent
mode: subagent
---

# Implement Agent

You are the Implement Agent in the DiJiang workflow.

## Recursion Guard

You are already the `dijiang-implement` sub-agent. Do the implementation work directly.

## Protocol

1. Read `dijiang workflow-state --json` and treat injected `Skill Manifests` plus `<dijiang-target-skill ...>` as the primary runtime routing context.
2. Find active task: `dijiang task current`
3. Read `prd.md`, `design.md`, `implement.md` from the task directory
4. Load implement.jsonl for spec references
5. Implement the changes
6. Verify: `cargo build && cargo test`

## Forbidden

- `git commit`, `git push`, `git merge`
"#
    }

    fn check_agent_content() -> &'static str {
        r#"---
name: trellis-check
description: DiJiang quality check agent
mode: subagent
---

# Check Agent

You are the Check Agent in the DiJiang workflow.

## Protocol

1. Read `dijiang workflow-state --json` and treat injected `Skill Manifests` plus `<dijiang-target-skill ...>` as the primary runtime routing context.
2. Find active task: `dijiang task current`
3. Read `prd.md` for acceptance criteria
4. Load check.jsonl for spec references
5. Get changes: `git diff`
6. Check against specs in `.dijiang/spec/`
7. Self-fix any issues
8. Verify: `cargo build && cargo test`
"#
    }

    fn hooks_content() -> &'static str {
        r#"{
  "hooks": [
    {
      "type": "session:start",
      "command": "dijiang status",
      "description": "Load DiJiang project context on session start"
    }
  ]
}
"#
    }
}

impl Configurator for HermesConfigurator {
    fn platform(&self) -> PlatformKind {
        PlatformKind::Hermes
    }

    fn is_installed(&self) -> bool {
        std::process::Command::new("hermes")
            .arg("--version")
            .output()
            .ok()
            .is_some_and(|o| o.status.success())
    }

    fn configure(&self, cwd: &Path) -> Result<(), ConfigError> {
        let hermes_dir = cwd.join(".hermes");

        // ── agents/ ──
        let agents_dir = hermes_dir.join("agents");
        fs::create_dir_all(&agents_dir)?;
        fs::write(
            agents_dir.join("dijiang-implement.md"),
            Self::implement_agent_content(),
        )?;
        fs::write(
            agents_dir.join("dijiang-check.md"),
            Self::check_agent_content(),
        )?;
        eprintln!("  ├── .hermes/agents/dijiang-implement.md");
        eprintln!("  ├── .hermes/agents/dijiang-check.md");

        // ── hooks.json ──
        fs::write(hermes_dir.join("hooks.json"), Self::hooks_content())?;
        eprintln!("  ├── .hermes/hooks.json");

        Ok(())
    }

    fn has_hooks(&self) -> bool {
        true
    }
}
