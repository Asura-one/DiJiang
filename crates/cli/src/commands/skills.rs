pub fn cmd_skills(sync: bool) -> anyhow::Result<()> {
    if sync {
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
