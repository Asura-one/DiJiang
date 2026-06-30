use crate::{ConfigError, Configurator, PlatformKind};
use std::fs;
use std::path::Path;

/// OpenCode configurator — writes `.opencode/agents/` and `.opencode/plugins/`.
///
/// OpenCode is a class-2 (hasHooks=false) platform. Context injection uses
/// plugin hooks (chat.message intercept) rather than push-based injection.
pub struct OpenCodeConfigurator;

impl OpenCodeConfigurator {
    pub fn new() -> Self {
        Self
    }

    fn implement_agent_content() -> &'static str {
        r#"---
description: |
  Code implementation expert. Understands specs and requirements, then implements features. No git commit allowed.
mode: subagent
permission:
  read: allow
  write: allow
  edit: allow
  bash: allow
  glob: allow
  grep: allow
---
# Implement Agent

You are the Implement Agent in the DiJiang workflow.

## Recursion Guard

You are already the `dijiang-implement` sub-agent. Do the implementation work directly.
- Do NOT spawn another `dijiang-implement` or `dijiang-check` sub-agent.
- Only the main session may dispatch implement/check agents.

## Context

Before implementing, read:
- `.dijiang/workflow.md` — Project workflow
- `.dijiang/spec/` — Development guidelines
- Task `prd.md` — Requirements document
- Task `design.md` — Technical design (if exists)
- Task `implement.md` — Execution plan (if exists)

## Core Responsibilities

1. **Understand specs** — Read relevant spec files in `.dijiang/spec/`
2. **Understand task artifacts** — Read prd.md, design.md, implement.md
3. **Implement features** — Write code following specs and task artifacts
4. **Self-check** — Run `cargo build` / `cargo test` to verify
5. **Report results**

## Forbidden Operations

- `git commit`
- `git push`
- `git merge`

## Report Format

```markdown
## Implementation Complete

### Files Modified
- `src/feature.rs` — New component

### Verification Results
- Build: Passed
- Test: Passed
```
"#
    }

    fn check_agent_content() -> &'static str {
        r#"---
description: |
  Code quality check expert. Reviews code changes against specs and self-fixes issues.
mode: subagent
permission:
  read: allow
  write: allow
  edit: allow
  bash: allow
  glob: allow
  grep: allow
---
# Check Agent

You are the Check Agent in the DiJiang workflow.

## Recursion Guard

You are already the `dijiang-check` sub-agent. Do the review and fixes directly.
- Do NOT spawn another `dijiang-check` or `dijiang-implement` sub-agent.
- Only the main session may dispatch implement/check agents.

## Core Responsibilities

1. **Get code changes** — Use `git diff` to get uncommitted code
2. **Review against specs** — Check changes against prd.md and spec files
3. **Self-fix** — Fix issues yourself, not just report them
4. **Run verification** — `cargo build` / `cargo test`

## Workflow

### Step 1: Get Changes
```bash
git diff --name-only
git diff
```

### Step 2: Check Against Specs
Read task prd.md, design.md, implement.md, and specs in `.dijiang/spec/`.

### Step 3: Self-Fix
Fix issues directly, then re-run verification.

### Step 4: Run Verification
```bash
cargo build
cargo test
```

## Report Format

```markdown
## Self-Check Complete
### Files Checked
- src/feature.rs
### Issues Found and Fixed
1. `<file>:<line>` — <what was fixed>
### Verification Results
- Build: Passed
- Test: Passed
```
"#
    }

    fn session_start_plugin_content() -> &'static str {
        r#"/* global process */
import { execFileSync } from "node:child_process";

function errorContext(message) {
  const session =
    process.env.DIJIANG_CONTEXT_ID ||
    process.env.OPENCODE_SESSION_ID ||
    process.env.OPENCODE_RUN_ID ||
    "unknown";
  return [
    "<dijiang-workflow-state>",
    "Platform: opencode",
    `Session hint: ${session}`,
    `Hook error: ${message}`,
    "Active task: unknown",
    "Next: run `dijiang workflow-state` from the project root and check that `dijiang` is on PATH.",
    "</dijiang-workflow-state>",
  ].join("\n");
}

export default async ({ directory }) => {
  return {
    "chat.message": async (_input, output) => {
      const parts = output?.parts || [];
      let context = "";

      try {
        context = execFileSync(
          "dijiang",
          ["workflow-state", "--hook-event", "chat.message"],
          {
            cwd: directory,
            encoding: "utf8",
            timeout: 10000,
            env: {
              ...process.env,
              DIJIANG_CONTEXT_ID:
                process.env.DIJIANG_CONTEXT_ID ||
                process.env.OPENCODE_SESSION_ID ||
                process.env.OPENCODE_RUN_ID ||
                "opencode",
            },
          }
        ).trim();
      } catch (error) {
        context = errorContext(error instanceof Error ? error.message : String(error));
      }

      if (!context) {
        return;
      }

      const textPartIndex = parts.findIndex(
        (part) => part.type === "text" && part.text !== undefined
      );

      if (textPartIndex !== -1) {
        parts[textPartIndex].text = context + "\n\n---\n\n" + (parts[textPartIndex].text || "");
      } else {
        parts.unshift({ type: "text", text: context });
      }
    },
  };
};
"#
    }

    fn package_json_content() -> &'static str {
        r#"{
  "dependencies": {
    "@opencode-ai/plugin": "^1.14.39"
  }
}
"#
    }

    fn dijiang_context_content() -> &'static str {
        r#"/**
 * DiJiang Trellis Context utilities.
 * Simplified version for .opencode plugin context management.
 */

export class TrellisContext {
  constructor(directory) {
    this.directory = directory;
  }
}
"#
    }

    fn session_utils_content() -> &'static str {
        r#"/**
 * DiJiang session utilities for OpenCode plugins.
 */
export function buildSessionContext(ctx, input) {
  return `DiJiang project at ${ctx.directory}`;
}
"#
    }
}

impl Configurator for OpenCodeConfigurator {
    fn platform(&self) -> PlatformKind {
        PlatformKind::OpenCode
    }

    fn is_installed(&self) -> bool {
        std::process::Command::new("opencode")
            .arg("--version")
            .output()
            .ok()
            .is_some_and(|o| o.status.success())
    }

    fn configure(&self, cwd: &Path) -> Result<(), ConfigError> {
        let opencode_dir = cwd.join(".opencode");

        // ── agents/ ──
        let agents_dir = opencode_dir.join("agents");
        fs::create_dir_all(&agents_dir)?;
        fs::write(
            agents_dir.join("dijiang-implement.md"),
            Self::implement_agent_content(),
        )?;
        fs::write(
            agents_dir.join("dijiang-check.md"),
            Self::check_agent_content(),
        )?;
        eprintln!("  ├── .opencode/agents/dijiang-implement.md");
        eprintln!("  ├── .opencode/agents/dijiang-check.md");

        // ── plugins/ ──
        let plugins_dir = opencode_dir.join("plugins");
        fs::create_dir_all(&plugins_dir)?;
        fs::write(
            plugins_dir.join("session-start.js"),
            Self::session_start_plugin_content(),
        )?;
        eprintln!("  ├── .opencode/plugins/session-start.js");

        // ── lib/ ──
        let lib_dir = opencode_dir.join("lib");
        fs::create_dir_all(&lib_dir)?;
        fs::write(
            lib_dir.join("dijiang-context.js"),
            Self::dijiang_context_content(),
        )?;
        fs::write(
            lib_dir.join("session-utils.js"),
            Self::session_utils_content(),
        )?;
        eprintln!("  ├── .opencode/lib/dijiang-context.js");
        eprintln!("  ├── .opencode/lib/session-utils.js");

        // ── package.json ──
        fs::write(
            opencode_dir.join("package.json"),
            Self::package_json_content(),
        )?;
        eprintln!("  ├── .opencode/package.json");

        Ok(())
    }

    fn has_hooks(&self) -> bool {
        false
    }
}
