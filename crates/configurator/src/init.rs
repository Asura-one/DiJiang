/// Initialize `.trellis/` infrastructure (task storage, workflow, workspace).
use crate::templates;
use crate::PlatformKind;
use std::path::Path;

/// Create `.trellis/` infrastructure: workflow.md, tasks/, workspace/, spec/.
pub(crate) fn write_trellis_infrastructure(
    cwd: &Path,
    developer: Option<&str>,
) -> Result<(), crate::ConfigError> {
    let trellis_dir = cwd.join(".trellis");

    // tasks/ — task storage
    // tasks/ — task storage
    std::fs::create_dir_all(trellis_dir.join("tasks"))?;

    // workspace/ — developer journals
    std::fs::create_dir_all(trellis_dir.join("workspace"))?;
    if let Some(dev) = developer {
        std::fs::create_dir_all(trellis_dir.join("workspace").join(dev))?;
    }

    // spec/ — coding guidelines (placeholder)
    std::fs::create_dir_all(trellis_dir.join("spec"))?;

    // workflow.md — from embedded template
    let workflow = templates::render("config/workflow.md", &[])
        .map_err(|e| crate::ConfigError::Serialize(e))?;
    std::fs::write(trellis_dir.join("workflow.md"), workflow)?;

    Ok(())
}

/// Read project name from `.dijiang/config.toml`.
/// Returns the project name or the directory name as fallback.
pub fn read_project_name(cwd: &Path) -> String {
    let config_path = cwd.join(".dijiang").join("config.toml");
    if let Ok(content) = std::fs::read_to_string(config_path) {
        if let Ok(config) = toml::from_str::<crate::DijiangConfig>(&content) {
            return config.project.name;
        }
    }
    cwd.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}
/// Initialize a DiJiang project at the given path (all platforms).
pub fn init_project(
    cwd: &Path,
    name: &str,
    developer: Option<&str>,
) -> Result<(), crate::ConfigError> {
    init_project_with_platforms(cwd, name, developer, &PlatformKind::all())
}

/// Like `init_project`, but with explicit platform selection.
pub fn init_project_with_platforms(
    cwd: &Path,
    name: &str,
    developer: Option<&str>,
    platforms: &[PlatformKind],
) -> Result<(), crate::ConfigError> {
    // Always write DiJiang config and .trellis/ infrastructure
    crate::PiConfigurator::write_dijiang_config(cwd, name, developer)?;
    write_trellis_infrastructure(cwd, developer)?;

    // Always write AGENTS.md
    crate::PiConfigurator::write_agents_md(cwd)?;

    // Use registry for platform-specific config
    let registry = crate::ConfiguratorRegistry::with_all();
    let results = registry.configure(cwd, platforms);

    let mut files_created: Vec<&str> = Vec::new();
    for (platform, result) in &results {
        if let Err(e) = result {
            eprintln!("  ⚠ {platform:?} config error: {e}");
            continue;
        }
        match platform {
            PlatformKind::Pi => {
                files_created.extend([
                    ".pi/settings.json",
                    ".pi/prompts/",
                    ".pi/extensions/dijiang/index.ts",
                ]);
            }
            PlatformKind::Cursor => {
                files_created.extend([".cursor/rules/", ".cursor/hooks.json"]);
            }
            PlatformKind::Claude => {
                files_created.extend(["CLAUDE.md", ".claude/"]);
            }
            PlatformKind::Codex => {
                files_created.extend([".codex/agents/", ".codex/hooks/"]);
            }
            PlatformKind::OpenCode => {
                files_created.extend([".opencode/"]);
            }
            PlatformKind::Hermes => {
                files_created.extend([".hermes/agents/", ".hermes/hooks.json"]);
            }
        }
    }

    println!("  ✓ Initialized DiJiang project '{name}'");
    println!("  ├── .dijiang/config.toml");
    println!("  ├── .trellis/workflow.md");
    println!("  ├── .trellis/tasks/");
    println!("  ├── .trellis/workspace/");
    println!("  ├── .trellis/spec/");
    for f in &files_created {
        println!("  ├── {f}");
    }
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
