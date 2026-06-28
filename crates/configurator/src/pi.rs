use crate::types::*;
use crate::skills::skill_content;
use std::path::Path;

/// Pi platform configurator.
///
/// Generates Pi-specific config files:
/// - `.pi/settings.json` — registers skills, prompts, extensions
/// - `.pi/prompts/dijiang-*.md` — command prompt templates
/// - `.pi/extensions/dijiang/index.ts` — extension hook
/// - `.dijiang/config.toml` — DiJiang project config
/// - `AGENTS.md` — DiJiang instructions block (inject/update)
/// - `AGENTS.md` — Trellis-style instructions block (inject/update)
pub struct PiConfigurator;

impl PiConfigurator {
    pub fn new() -> Self {
        Self
    }

    /// Ensure `.pi/` subdirectory exists.
    fn ensure_pi_dir(cwd: &Path) -> Result<(), ConfigError> {
        for sub in &["prompts", "agents", "skills", "extensions/dijiang"] {
            std::fs::create_dir_all(cwd.join(".pi").join(sub))?;
        }
        Ok(())
    }

    /// Write `.pi/settings.json`.
    fn write_settings(cwd: &Path) -> Result<(), ConfigError> {
        let settings = PiSettings {
            enable_skill_commands: true,
            extensions: vec!["./extensions/dijiang/index.ts".to_string()],
            skills: vec!["./skills".to_string()],
            prompts: vec!["./prompts".to_string()],
            agents: vec![],
        };
        let json = serde_json::to_string_pretty(&settings)
            .map_err(|e| ConfigError::Serialize(e.to_string()))?;
        std::fs::write(cwd.join(".pi").join("settings.json"), json)?;
        Ok(())
    }

    /// Write `.pi/extensions/dijiang/index.ts`.
    fn write_extension(cwd: &Path) -> Result<(), ConfigError> {
        let content = r##"import { defineExtension } from "pi";
import { readFileSync } from "fs";
import { join } from "path";

export default defineExtension({
  name: "dijiang",
  "session:start": async (ctx) => {
    const cwd = process.cwd();

    // Read active task context via dijiang CLI
    try {
      const { execSync } = require("child_process");
      const result = execSync(`dijiang task current`, {});
      const taskPath = result.toString().trim();
      if (taskPath && taskPath !== "No active task") {
        ctx.setVar("activeTask", taskPath);
      }
    } catch {}

    // Read spec index
    try {
      const specIndex = join(cwd, ".trellis/spec/index.md");
      const content = readFileSync(specIndex, "utf-8");
      ctx.setVar("specIndex", content);
    } catch {}
  },
});"##;
        std::fs::write(
            cwd.join(".pi").join("extensions").join("dijiang").join("index.ts"),
            content.trim_start(),
        )?;
        Ok(())
    }

    /// Write prompt templates to `.pi/prompts/`.
    fn write_prompts(cwd: &Path) -> Result<(), ConfigError> {
        let prompts_dir = cwd.join(".pi").join("prompts");

        // start.md
        std::fs::write(
            prompts_dir.join("dijiang-start.md"),
            vec![
                "## Active Task Context",
                "",
                "Active Task: {{activeTask}}",
                "",
                "## Spec Index",
                "",
                "{{specIndex}}",
                "",
                "---",
                "",
                "Read `dijiang task current` to load current task.",
                "Read `.trellis/workflow.md` for development workflow.",
            ]
            .join("\n"),
        )?;

        // finish-work.md
        std::fs::write(
            prompts_dir.join("dijiang-finish-work.md"),
            vec![
                "## Finish Work",
                "",
                "Complete your current task and prepare for review.",
                "",
                "Steps:",
                "1. Run relevant tests",
                "2. Verify type checks pass",
                "3. Update task status if needed",
                "4. Write workspace journal entry",
            ]
            .join("\n"),
        )?;

        Ok(())
    }

    /// Write AGENTS.md Trellis-style instructions.
    pub(crate) fn write_agents_md(cwd: &Path) -> Result<(), ConfigError> {
        let content = vec![
            "<!-- DIJIANG:START -->",
            "# DiJiang Project Instructions",
            "",
            "This project uses DiJiang for task management and workflow.",
            "",
            "## Project Structure",
            "",
            "- `.dijiang/` — DiJiang project configuration",
            "- `.trellis/tasks/` — active and archived tasks",
            "- `.trellis/spec/` — coding guidelines",
            "- `.trellis/workspace/` — developer journals",
            "- `.pi/` — Pi platform configuration",
            "",
            "## Available Commands",
            "",
            "| Command | Description |",
            "|---------|-------------|",
            "| `/dj-dispatch` | Classify and route a task |",
            "| `/dj-grill` | Requirements alignment |",
            "| `/dj-implement` | Code implementation |",
            "| `/dj-hunt` | Debug and investigate |",
            "| `/dj-check` | Code review and verification |",
            "| `/dj-muse` | Memory management |",
            "| `/dj-output` | Document generation |",
            "| `/dj-tdd` | Test-driven development |",
            "| `/dj-design` | Design documentation |",
            "| `dijiang status` | Show project status |",
            "| `dijiang task list` | List active tasks |",
            "| `dijiang task current` | Show active task |",
            "| `dijiang mem list` | Show memory sessions |",
            "",
            "## Workflow",
            "",
            "1. Read this file and `.trellis/workflow.md` at session start",
            "2. Check active task with `dijiang task current`",
            "3. Read task artifacts (task.json, prd.md, design.md, implement.md)",
            "4. Read relevant spec files from `.trellis/spec/`",
            "5. Follow the task status phase to determine workflow:",
            "   - `planning` → dj-grill (requirements alignment)",
            "   - `in_progress` → dj-implement (implementation)",
            "   - `review` → dj-check (verification)",
            "",
            "Managed by DiJiang. Edits outside this block are preserved.",
            "<!-- DIJIANG:END -->",
        ]
        .join("\n");

        let agents_path = cwd.join("AGENTS.md");
        let existing = std::fs::read_to_string(&agents_path).unwrap_or_default();

        // Replace existing DIJIANG block or append
        if let Some(start) = existing.find("<!-- DIJIANG:START -->") {
            if let Some(end) = existing[start..].find("<!-- DIJIANG:END -->") {
                let new = format!(
                    "{}{}{}",
                    &existing[..start],
                    content,
                    &existing[start + end + "<!-- DIJIANG:END -->".len()..]
                );
                std::fs::write(&agents_path, new)?;
                return Ok(());
            }
        }

        // Append if no existing block
        let new = if existing.is_empty() {
            content
        } else {
            format!("{existing}\n\n{content}")
        };
        std::fs::write(&agents_path, new)?;
        Ok(())
    }

    /// Write DiJiang skill files to `.pi/skills/dijiang-*/SKILL.md`.
    fn write_skills(cwd: &Path) -> Result<(), ConfigError> {
        let skills_dir = cwd.join(".pi").join("skills");
        for (name, desc, content) in skill_content::all_session_skills() {
            let dir = skills_dir.join(name);
            std::fs::create_dir_all(&dir)?;
            std::fs::write(
                dir.join("SKILL.md"),
                skill_content::wrap_frontmatter(name, desc, content),
            )?;
        }
        Ok(())
    }

    /// Write DiJiang agent definitions to `.pi/agents/dijiang-*.md`.
    fn write_agents(cwd: &Path) -> Result<(), ConfigError> {
        let agents_dir = cwd.join(".pi").join("agents");
        let agents: [(&str, &str); 3] = [
            ("dijiang-implement",
                "---\nname: dijiang-implement\ntype: sub-agent\n---\n\n# DiJiang Implement\n\nYou are the implementation sub-agent in the DiJiang ecosystem.\n\n## Context Loading\n\n1. Find active task: `dijiang task current`\n2. Read prd.md, design.md, implement.md from task directory\n3. Read relevant specs from `.trellis/spec/`\n4. Load context from implement.jsonl\n\n## Workflow\n\nAfter loading context, delegate to the appropriate dj-* skill:\n- Feature work → `dj-implement`\n- Test-driven → `dj-tdd`\n- Prototyping → `dj-prototype`\n- Refactoring → `dj-ponytail`\n- Scripting → `dj-script`\n\nUse `dj-karpathy` (LLM coding guidelines) alongside any implementation skill.\nRun `cargo build && cargo test` to verify after changes.\n"),
            ("dijiang-check",
                "---\nname: dijiang-check\ntype: sub-agent\n---\n\n# DiJiang Check\n\nYou are the quality check sub-agent in the DiJiang ecosystem.\n\n## Context Loading\n\n1. Find active task: `dijiang task current`\n2. Read prd.md for acceptance criteria\n3. Read relevant specs from `.trellis/spec/`\n4. Load context from check.jsonl\n\n## Workflow\n\nAfter loading context, delegate to `dj-check` for:\n- Diff quality review\n- Functional completeness check\n- Safety verification\n- git-safety compliance\n\nAlso use:\n- `dj-audit` for whole-codebase over-engineering scans\n- `dj-debt` for tech debt tracking\n- `dj-health` for agent configuration health\n\nRun `cargo test && cargo build` to verify.\n"),
("dijiang-research",
                "---\nname: dijiang-research\ntype: sub-agent\n---\n\n# DiJiang Research\n\nYou are the research sub-agent in the DiJiang ecosystem for technical investigation.\n\n## Workflow\n\n1. Read relevant specs from `.trellis/spec/`\n2. Research code patterns and dependencies\n3. Summarize findings for the implement agent\n\nUse `dj-hunt` for systematic bug investigation when needed.\nUse `dj-dispatch` to classify ambiguous research requests.\n"),
        ];
        for (name, content) in &agents {
            std::fs::write(agents_dir.join(format!("{name}.md")), content)?;
        }
        Ok(())
    }
    /// Write `.dijiang/config.toml`.
    pub(crate) fn write_dijiang_config(cwd: &Path, name: &str, dev: Option<&str>) -> Result<(), ConfigError> {
        let config = DijiangConfig {
            project: ProjectConfig {
                name: name.to_string(),
                description: None,
                developer: dev.map(|s| s.to_string()),
                version: "0.1.0".to_string(),
            },
            platforms: Some(vec!["pi".to_string()]),
            ..Default::default()
        };
        let toml = toml::to_string_pretty(&config)
            .map_err(|e| ConfigError::Serialize(e.to_string()))?;
        let config_dir = cwd.join(".dijiang");
        std::fs::create_dir_all(&config_dir)?;
        std::fs::write(config_dir.join("config.toml"), toml)?;
        Ok(())
    }
}

impl Default for PiConfigurator {
    fn default() -> Self {
        Self::new()
    }
}

impl Configurator for PiConfigurator {
    fn platform(&self) -> &str {
        "pi"
    }

    fn configure(&self, cwd: &Path) -> Result<(), ConfigError> {
        Self::ensure_pi_dir(cwd)?;
        Self::write_settings(cwd)?;
        Self::write_extension(cwd)?;
        Self::write_prompts(cwd)?;
        Self::write_skills(cwd)?;
        Self::write_agents(cwd)?;
        Ok(())
    }

    fn has_hooks(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pi_configure_creates_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        let pi = PiConfigurator::new();
        pi.configure(tmp.path()).unwrap();

        assert!(tmp.path().join(".pi").join("settings.json").exists());
        assert!(tmp.path().join(".pi").join("prompts").join("dijiang-start.md").exists());
        assert!(tmp.path().join(".pi").join("prompts").join("dijiang-finish-work.md").exists());
        assert!(tmp.path().join(".pi").join("extensions").join("dijiang").join("index.ts").exists());
    }
    #[test]
    fn test_write_agents_md_new() {
        let tmp = tempfile::TempDir::new().unwrap();
        PiConfigurator::write_agents_md(tmp.path()).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
        assert!(content.contains("<!-- DIJIANG:START -->"));
        assert!(content.contains("<!-- DIJIANG:END -->"));
        assert!(content.contains("DiJiang Project Instructions"));
    }

    #[test]
    fn test_write_agents_md_replace_block() {
        let tmp = tempfile::TempDir::new().unwrap();
        let existing = "existing content\n<!-- DIJIANG:START -->\nold\n<!-- DIJIANG:END -->\nmore content";
        std::fs::write(tmp.path().join("AGENTS.md"), existing).unwrap();
        PiConfigurator::write_agents_md(tmp.path()).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
        assert!(content.starts_with("existing content"), "bad start: {content:?}");
        assert!(content.contains("<!-- DIJIANG:START -->"), "no start marker");
        assert!(content.contains("<!-- DIJIANG:END -->"), "no end marker");
        assert!(!content.contains("old"), "old content still present");
        // content after block should be preserved
        assert!(
            content.ends_with("more content") || content.ends_with("more content\n") || content.ends_with("more content\n\n"),
            "did not preserve content after block: ...{content:?}"
        );
    }

    #[test]
    fn test_write_dijiang_config() {
        let tmp = tempfile::TempDir::new().unwrap();
        PiConfigurator::write_dijiang_config(tmp.path(), "test-project", Some("tiezhu")).unwrap();
        let toml = std::fs::read_to_string(tmp.path().join(".dijiang").join("config.toml")).unwrap();
        assert!(toml.contains("test-project"));
        assert!(toml.contains("tiezhu"));
        assert!(toml.contains("pi"));
    }
}
