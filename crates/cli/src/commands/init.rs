use crate::util::require_dijiang_dir;
use dialoguer::{Input, MultiSelect};
use dijiang_configurator::PlatformKind;

fn ensure_project_root() -> anyhow::Result<std::path::PathBuf> {
    let cwd = std::env::current_dir()?;
    // If .dijiang/ exists, use its parent; otherwise use cwd
    if let Ok(dir) = require_dijiang_dir() {
        Ok(dir.parent()
            .ok_or_else(|| anyhow::anyhow!("无法确定项目根目录"))?
            .to_path_buf())
    } else {
        Ok(cwd)
    }
}

pub fn cmd_init(
    name: &str,
    developer: Option<&str>,
    yes: bool,
    force: bool,
    platforms: Option<&str>,
    auto_detect: bool,
) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;

    // Check if already initialized
    if cwd.join(".dijiang").join("config.toml").exists() {
        if !force {
            println!("  Already initialized. Use --force to reinitialize.");
            return Ok(());
        }
        println!("  Overwriting...");
    }

    // Project name
    let project_name = if name.is_empty() {
        let default_name = cwd
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("my-project");
        if yes {
            default_name.to_string()
        } else {
            Input::new()
                .with_prompt("Project name")
                .default(default_name.to_string())
                .interact_text()?
        }
    } else {
        name.to_string()
    };

    // Developer name
    let developer = developer.map(|s| s.to_string()).or_else(|| {
        let git_name = std::process::Command::new("git")
            .args(["config", "--global", "user.name"])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
                } else {
                    None
                }
            });
        if yes {
            git_name
        } else {
            let default_dev = git_name.unwrap_or_default();
            Input::new()
                .with_prompt("Developer name")
                .default(default_dev)
                .allow_empty(true)
                .interact_text()
                .ok()
                .filter(|s| !s.is_empty())
        }
    });

    // Platform selection
    let selected_platforms: Vec<PlatformKind> = if auto_detect {
        let registry = dijiang_configurator::ConfiguratorRegistry::with_all();
        let detected = registry.auto_detect();
        if detected.is_empty() {
            eprintln!("No installed platforms detected. Run without --auto-detect to select.");
            std::process::exit(1);
        }
        println!("  Detected platforms: {}",
            detected.iter().map(|p| p.display_name()).collect::<Vec<_>>().join(", "));
        detected
    } else if let Some(p) = platforms {
        p.split(',')
            .filter_map(|s| match s.trim() {
                "pi" => Some(PlatformKind::Pi),
                "cursor" => Some(PlatformKind::Cursor),
                "claude" => Some(PlatformKind::Claude),
                "codex" => Some(PlatformKind::Codex),
                "opencode" => Some(PlatformKind::OpenCode),
                "hermes" => Some(PlatformKind::Hermes),
                _ => None,
            })
            .collect()
    } else if yes {
        PlatformKind::all()
    } else {
        let all_platforms = PlatformKind::all();
        let display_names: Vec<&str> = all_platforms.iter().map(|p| p.display_name()).collect();
        let selections = MultiSelect::new()
            .with_prompt("Select platforms to configure")
            .items(&display_names)
            .defaults(&[true, false, false, false, false, false])
            .interact()?;
        selections.iter().map(|&i| all_platforms[i]).collect()
    };

    if selected_platforms.is_empty() {
        eprintln!("No platforms selected. Use --platforms or select at least one.");
        std::process::exit(1);
    }

    dijiang_configurator::init_project_with_platforms(
        &cwd, &project_name, developer.as_deref(), &selected_platforms,
    )?;

    let skills_written = dijiang_configurator::write_project_skills(&cwd, false)?;
    if skills_written > 0 {
        println!("  Wrote {} dj-* skills to .pi/skills/", skills_written);
    }

    match dijiang_mem::GlobalMemory::new() {
        Ok(global_mem) => {
            if let Err(e) = global_mem.ensure_default_tactics() {
                eprintln!("  Warning: Failed to initialize default tactics: {}", e);
            } else {
                println!("  Initialized default tactics (cargo-test, typecheck, lint-fix, doc-update)");
            }
        }
        Err(e) => {
            eprintln!("  Warning: Failed to initialize global memory: {}", e);
        }
    }

    Ok(())
}
