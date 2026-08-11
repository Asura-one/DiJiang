pub fn cmd_skills(sync: bool, validate: bool) -> anyhow::Result<()> {
    if validate {
        dijiang_configurator::validate_skill_templates().map_err(anyhow::Error::msg)?;
        dijiang_configurator::validate_skill_registry_names(dijiang_task::all_skill_names())
            .map_err(anyhow::Error::msg)?;
        println!("  Managed dj-* skill registry is valid.");
    } else if sync {
        let cwd = std::env::current_dir()?;
        let skills_written = dijiang_configurator::write_project_skills(&cwd, false)?;
        println!("  Synced {} dj-* skills to .pi/skills/", skills_written);
    } else {
        let names = dijiang_configurator::list_skill_names();
        println!("  {} dj-* skills available:", names.len());
        for name in names {
            println!("    {}", name);
        }
        println!();
        println!("  Use `dijiang skills --sync` to write skills to current project.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_skill_validation_connects_runtime_and_templates() {
        cmd_skills(false, true).unwrap();
    }
}
