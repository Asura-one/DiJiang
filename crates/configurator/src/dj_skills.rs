use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::templates::TemplateAssets;

/// List all dj-* skill names by scanning the embedded templates directory.
/// Only matches top-level SKILL.md files under `skills/<name>/SKILL.md`.
pub fn list_skill_names() -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for path in TemplateAssets::iter() {
        let path = path.as_ref();
        if let Some(name) = path
            .strip_prefix("skills/")
            .and_then(|p| p.strip_suffix("/SKILL.md"))
        {
            if !name.contains('/') && !name.starts_with('.') {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    names
}

/// Return the global skill template directory: `~/.dijiang/skills/`.
fn global_skills_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("cannot determine home directory")?;
    Ok(home.join(".dijiang").join("skills"))
}

/// Ensure the global skill template directory exists and is populated
/// from embedded resources. Force refresh rewrites managed global dj-* skills.
pub fn ensure_global_skills(force: bool) -> Result<PathBuf> {
    let dir = global_skills_dir()?;

    if !force && dir.exists() && dir.read_dir()?.next().is_some() {
        return Ok(dir);
    }

    fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    // Discover and copy all skills from the embedded template directory.
    // Each skill is under `skills/<name>/SKILL.md` in the templates.
    for path in TemplateAssets::iter() {
        let path = path.as_ref();
        if let Some(name) = path
            .strip_prefix("skills/")
            .and_then(|p| p.strip_suffix("/SKILL.md"))
        {
            if name.contains('/') || name.starts_with('.') {
                continue; // Skip nested files and retired/hidden skills
            }
            let asset = TemplateAssets::get(path)
                .expect("Embedded skill SKILL.md should exist after iter() returned it");
            let content = std::str::from_utf8(asset.data.as_ref())
                .context("Skill SKILL.md is not valid UTF-8")?;
            let skill_dir = dir.join(name);
            fs::create_dir_all(&skill_dir)?;
            fs::write(skill_dir.join("SKILL.md"), content)?;
        }
    }

    Ok(dir)
}

/// Write dj-* skills from the global template directory into a project's
/// `.pi/skills/` directory. Force refresh rewrites managed global templates
/// and overwrites project copies of managed `dj-*` skills.
pub fn write_project_skills(project_dir: &Path, force: bool) -> Result<usize> {
    let global_dir = ensure_global_skills(force)?;
    let pi_skills = project_dir.join(".pi").join("skills");

    let mut written = 0usize;

    if global_dir.exists() {
        for entry in fs::read_dir(&global_dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy().to_string();
            if !entry.file_type()?.is_dir() || name_str.starts_with('.') {
                continue;
            }
            let src = entry.path().join("SKILL.md");
            if !src.exists() {
                continue;
            }
            let dst_dir = pi_skills.join(&name_str);
            let dst = dst_dir.join("SKILL.md");

            if dst.exists() && !force {
                continue;
            }

            fs::create_dir_all(&dst_dir)?;
            fs::copy(&src, &dst)?;
            written += 1;
        }
    }

    Ok(written)
}

pub fn get_skill_content(name: &str) -> Option<String> {
    let asset_path = format!("skills/{name}/SKILL.md");
    let asset = TemplateAssets::get(&asset_path)?;
    let content = std::str::from_utf8(asset.data.as_ref()).ok()?;
    Some(content.to_string())
}
