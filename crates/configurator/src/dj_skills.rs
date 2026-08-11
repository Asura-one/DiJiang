use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

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
    fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    for path in TemplateAssets::iter() {
        let path = path.as_ref();
        let Some(relative_path) = path.strip_prefix("skills/") else {
            continue;
        };
        let Some((name, _)) = relative_path.split_once('/') else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        let destination = dir.join(relative_path);
        if destination.exists() && !force {
            continue;
        }
        let asset = TemplateAssets::get(path)
            .expect("Embedded skill asset should exist after iter() returned it");
        let parent = destination
            .parent()
            .expect("Skill asset path must have a parent directory");
        fs::create_dir_all(parent)?;
        fs::write(destination, asset.data.as_ref())?;
    }

    Ok(dir)
}

/// Write managed skill directories from the global template directory into a
/// project's `.pi/skills/` directory. Each directory, including references and
/// scripts, is copied as one unit so init and update have identical assets.
pub fn write_project_skills(project_dir: &Path, force: bool) -> Result<usize> {
    let global_dir = ensure_global_skills(force)?;
    let pi_skills = project_dir.join(".pi").join("skills");
    let mut written = 0usize;

    for entry in fs::read_dir(&global_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !entry.file_type()?.is_dir() || name.starts_with('.') {
            continue;
        }
        let source = entry.path();
        if !source.join("SKILL.md").exists() {
            continue;
        }
        let destination = pi_skills.join(name.as_ref());
        if destination.exists() && !force {
            continue;
        }
        copy_skill_dir(&source, &destination)?;
        written += 1;
    }

    Ok(written)
}

fn copy_skill_dir(source: &Path, destination: &Path) -> Result<()> {
    if destination.exists() {
        fs::remove_dir_all(destination)?;
    }
    copy_dir_contents(source, destination)
}

fn copy_dir_contents(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') {
            continue;
        }
        let source_path = entry.path();
        let destination_path = destination.join(name);
        if entry.file_type()?.is_dir() {
            copy_dir_contents(&source_path, &destination_path)?;
        } else {
            fs::copy(&source_path, &destination_path)?;
        }
    }
    Ok(())
}

pub fn get_skill_content(name: &str) -> Option<String> {
    let asset_path = format!("skills/{name}/SKILL.md");
    let asset = TemplateAssets::get(&asset_path)?;
    let content = std::str::from_utf8(asset.data.as_ref()).ok()?;
    Some(content.to_string())
}

/// Verify the supplied skill registry contains exactly the managed skill templates.
pub fn validate_skill_registry_names<I, S>(names: I) -> Result<(), String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut errors = Vec::new();
    let expected = list_skill_names();
    let actual: std::collections::BTreeSet<_> = names
        .into_iter()
        .map(|name| name.as_ref().to_string())
        .collect();
    let expected_set: std::collections::BTreeSet<_> = expected.iter().cloned().collect();
    for name in expected_set.difference(&actual) {
        errors.push(format!("registry missing {name}"));
    }
    for name in actual.difference(&expected_set) {
        errors.push(format!("registry has unknown {name}"));
    }
    for name in expected {
        let content = get_skill_content(&name)
            .ok_or_else(|| format!("missing embedded template for {name}"))?;
        if let Err(error) = validate_skill_frontmatter(&name, &content) {
            errors.push(error);
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

#[derive(Deserialize)]
struct SkillFrontmatter {
    name: String,
    description: String,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    phases: Vec<String>,
    #[serde(default)]
    risk: Option<String>,
}

/// Validate one managed skill's YAML frontmatter against its directory name.
pub fn validate_skill_frontmatter(name: &str, content: &str) -> Result<(), String> {
    let content = content
        .strip_prefix("---\n")
        .ok_or_else(|| format!("template {name} has invalid frontmatter boundaries"))?;
    let closing = content
        .lines()
        .position(|line| line == "---")
        .ok_or_else(|| format!("template {name} has invalid frontmatter boundaries"))?;
    let body = content.lines().take(closing).collect::<Vec<_>>().join("\n");
    let frontmatter: SkillFrontmatter = serde_yaml::from_str(&body)
        .map_err(|error| format!("template {name} has invalid frontmatter: {error}"))?;
    if frontmatter.name != name {
        return Err(format!(
            "template {name} declares skill name {}",
            frontmatter.name
        ));
    }
    if frontmatter.description.trim().is_empty()
        || frontmatter
            .summary
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        || frontmatter
            .risk
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
    {
        return Err(format!("template {name} has an empty metadata field"));
    }
    let _ = frontmatter.phases;
    Ok(())
}

/// Validate every embedded managed skill template and its frontmatter.
pub fn validate_skill_templates() -> Result<(), String> {
    for name in list_skill_names() {
        let content = get_skill_content(&name)
            .ok_or_else(|| format!("missing embedded template for {name}"))?;
        validate_skill_frontmatter(&name, &content)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_skill_registry_matches_template_frontmatter() {
        validate_skill_templates().unwrap();
    }

    #[test]
    fn skill_registry_rejects_name_drift() {
        let error = validate_skill_registry_names(["dj-grill", "unknown-skill"])
            .expect_err("name drift must fail validation");
        assert!(error.contains("registry missing"));
        assert!(error.contains("registry has unknown unknown-skill"));
    }

    #[test]
    fn skill_frontmatter_requires_valid_yaml_and_fields() {
        for content in [
            "# missing frontmatter",
            "---\nname: dj-test\ndescription: \nsummary: Test\nphases: [check]\nrisk: low\n---\n",
            "---\nname: other\ndescription: Test\nsummary: Test\nphases: [check]\nrisk: low\n---\n",
            "---\nname: dj-test\ndescription: Test\n---junk\nbody\n",
        ] {
            assert!(validate_skill_frontmatter("dj-test", content).is_err());
        }
    }

    #[test]
    fn skill_frontmatter_accepts_managed_schema() {
        let content = "---\nname: dj-test\ndescription: Test skill\nsummary: Test\nphases: [check]\nrisk: low\n---\n";
        validate_skill_frontmatter("dj-test", content).unwrap();
    }
}
