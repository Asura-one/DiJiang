/// Initialize `.trellis/` infrastructure (task storage, workflow, workspace).
use crate::Configurator;
use std::path::Path;

/// Read project name from `.dijiang/config.toml`, or fall back to directory name.
pub fn read_project_name(cwd: &Path) -> Option<String> {
    let config_path = cwd.join(".dijiang").join("config.toml");
    if !config_path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(config_path).ok()?;
    // Parse TOML to extract project.name
    for line in content.lines() {
        if let Some(val) = line.strip_prefix("name = \"") {
            if let Some(end) = val.find('\"') {
                return Some(val[..end].to_string());
            }
        }
    }
    None
}

/// Create `.trellis/` infrastructure: workflow.md, tasks/, workspace/, spec/.
pub(crate) fn write_trellis_infrastructure(cwd: &Path, developer: Option<&str>) -> std::io::Result<()> {
    let trellis_dir = cwd.join(".trellis");

    // tasks/ — task storage
    std::fs::create_dir_all(trellis_dir.join("tasks"))?;

    // workspace/ — developer journals
    std::fs::create_dir_all(trellis_dir.join("workspace"))?;
    if let Some(dev) = developer {
        std::fs::create_dir_all(trellis_dir.join("workspace").join(dev))?;
    }

    // spec/ — coding guidelines (placeholder)
    std::fs::create_dir_all(trellis_dir.join("spec"))?;

    // workflow.md — DiJiang workflow guide
    let workflow = r#"# Development Workflow

---

## Core Principles

1. **Plan before code** — figure out what to do before you start
2. **Specs injected, not remembered** — guidelines are injected via hook/skill, not recalled from memory
3. **Persist everything** — research, decisions, and lessons all go to files
4. **Incremental development** — one task at a time
5. **Capture learnings** — after each task, review and write new knowledge back to spec

---

## DiJiang Workflow

DiJiang uses a **dispatch → grill → output → implement/tdd → hunt ↔ check** workflow:

### Phase 1: Requirements (grill)
- Use `/dj-grill` to align on requirements
- One question at a time, with recommended answers
- Output: `prd.md`

### Phase 2: Document (output)
- Use `/dj-output` to create design docs, implementation plans
- Output: `design.md`, `implement.md`

### Phase 3: Implement
- Use `/dj-implement` for feature coding
- Use `/dj-tdd` for test-driven development
- Use `/dj-ponytail` for minimal, focused changes
- Output: working code + tests

### Phase 4: Investigate (hunt)
- Use `/dj-hunt` for bug investigation
- Systematic root cause analysis
- Output: fix + spec update

### Phase 5: Check
- Use `/dj-check` for quality review
- Use `/dj-audit` for whole-codebase over-engineering scans
- Output: verified changes

---

## Project Structure

```
.trellis/            # Task management + specs
├── tasks/           # Task directories (task.json, prd.md, design.md, …)
├── spec/            # Coding guidelines by package/layer
├── workspace/       # Developer journals
└── workflow.md      # This file

.dijiang/            # DiJiang configuration
└── config.toml

.pi/                 # Pi platform configuration
├── skills/          # DiJiang workflow skills
├── agents/          # Sub-agent definitions
├── prompts/         # Prompt templates
└── settings.json
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `dijiang status` | Show project and active task status |
| `dijiang start <name>` | Create and activate a new task |
| `dijiang task list` | List all tasks |
| `dijiang task current` | Show active task |
| `dijiang task status <name> <status>` | Update task status |
| `dijiang mem list` | List memory sessions |

## Skill Routing

| Request type | Use |
|---|---|
| New feature / unclear requirements | `/dj-dispatch` → `/dj-grill` |
| Feature implementation | `/dj-implement` or `/dj-tdd` |
| Bug / regression | `/dj-hunt` |
| Code review | `/dj-check` |
| Documentation | `/dj-output` |
| Refactoring | `/dj-ponytail` |
| Prototype | `/dj-prototype` |
| UI design | `/dj-design` |
| Script / tool | `/dj-script` |
"#;
    std::fs::write(trellis_dir.join("workflow.md"), workflow)?;

    Ok(())
}

/// Initialize a DiJiang project at the given path.
///
/// Steps:
/// 1. Create `.dijiang/` with `config.toml`
/// 2. Create `.trellis/` (workflow.md, tasks/, workspace/, spec/)
/// 3. Write AGENTS.md with DiJiang instructions
/// 4. Run Pi configurator — `.pi/` setup
/// 5. Run Cursor configurator — `.cursor/rules/`
/// 6. Run Claude configurator — `CLAUDE.md` + `.claude/`
/// 7. Run Codex configurator — `.codex/agents/`
/// 8. Run OpenCode configurator — `.opencode/`
/// 9. Run Hermes configurator — `.hermes/`
pub fn init_project(
    cwd: &Path,
    name: &str,
    developer: Option<&str>,
) -> Result<(), crate::ConfigError> {
    let pi = crate::PiConfigurator::new();
    let cursor = crate::CursorConfigurator::new();
    let claude = crate::ClaudeConfigurator::new();
    let codex = crate::CodexConfigurator::new();
    let opencode = crate::OpenCodeConfigurator::new();
    let hermes = crate::HermesConfigurator::new();

    // Write DiJiang config
    crate::PiConfigurator::write_dijiang_config(cwd, name, developer)?;

    // Write .trellis/ infrastructure
    write_trellis_infrastructure(cwd, developer)?;

    // Write AGENTS.md
    crate::PiConfigurator::write_agents_md(cwd)?;

    // Run Pi configurator
    pi.configure(cwd)?;

    // Run multi-platform configurators
    cursor.configure(cwd)?;
    claude.configure(cwd)?;
    codex.configure(cwd)?;
    opencode.configure(cwd)?;
    hermes.configure(cwd)?;

    println!("  ✓ Initialized DiJiang project '{name}'");
    println!("  ├── .dijiang/config.toml");
    println!("  ├── .trellis/workflow.md");
    println!("  ├── .trellis/tasks/");
    println!("  ├── .trellis/workspace/");
    println!("  ├── .trellis/spec/");
    println!("  ├── .pi/settings.json");
    println!("  ├── .pi/prompts/");
    println!("  ├── .pi/extensions/dijiang/index.ts");
    println!("  ├── .cursor/rules/dijiang.mdc");
    println!("  ├── .cursor/hooks.json");
    println!("  ├── CLAUDE.md");
    println!("  ├── .claude/settings.json");
    println!("  ├── .codex/agents/");
    println!("  ├── .codex/hooks/");
    println!("  ├── .codex/hooks.json");
    println!("  ├── .codex/config.toml");
    println!("  ├── .opencode/agents/");
    println!("  ├── .opencode/plugins/");
    println!("  ├── .opencode/lib/");
    println!("  ├── .opencode/package.json");
    println!("  ├── .hermes/agents/");
    println!("  ├── .hermes/hooks.json");
    println!("  └── AGENTS.md");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_init_project_creates_all_platform_files() {
        let dir = std::env::temp_dir().join("dijiang_init_test_all");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let result = init_project(&dir, "test-project", Some("tiezhu"));
        assert!(result.is_ok(), "init_project failed: {:?}", result.err());

        // Verify .dijiang/ config exists
        assert!(dir.join(".dijiang").join("config.toml").exists());

        // Verify .trellis/ infrastructure exists
        assert!(dir.join(".trellis").join("workflow.md").exists());
        assert!(dir.join(".trellis").join("tasks").exists());
        assert!(dir.join(".trellis").join("workspace").join("tiezhu").exists());
        assert!(dir.join(".trellis").join("spec").exists());

        // Verify platform files exist
        assert!(dir.join(".pi").join("settings.json").exists());
        assert!(dir.join("AGENTS.md").exists());
        assert!(dir.join(".cursor").join("rules").join("dijiang.mdc").exists());
        assert!(dir.join("CLAUDE.md").exists());
        assert!(dir.join(".claude").join("settings.json").exists());
        assert!(dir.join(".codex").join("agents").join("dijiang-implement.toml").exists());
        assert!(dir.join(".opencode").join("agents").join("dijiang-implement.md").exists());
        assert!(dir.join(".hermes").join("agents").join("dijiang-implement.md").exists());

        // Verify workflow.md content is DiJiang-specific
        let workflow = fs::read_to_string(dir.join(".trellis").join("workflow.md")).unwrap();
        assert!(workflow.contains("DiJiang Workflow"));
        assert!(workflow.contains("dijiang status"));
        assert!(workflow.contains("dj-grill"));
        assert!(workflow.contains("dj-hunt"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_init_project_no_developer() {
        let dir = std::env::temp_dir().join("dijiang_init_test_nodev");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let result = init_project(&dir, "test-project", None);
        assert!(result.is_ok(), "init_project failed: {:?}", result.err());

        // workspace/ should exist but without developer subdir
        assert!(dir.join(".trellis").join("workspace").exists());

        let _ = fs::remove_dir_all(&dir);
    }
}
