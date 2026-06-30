use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// All dj-* skill names managed by DiJiang.
const DJ_SKILL_NAMES: &[&str] = &[
    "dj-audit",
    "dj-check",
    "dj-debt",
    "dj-design",
    "dj-dispatch",
    "dj-grill",
    "dj-handoff",
    "dj-health",
    "dj-hunt",
    "dj-implement",
    "dj-karpathy",
    "dj-output",
    "dj-pattern",
    "dj-ponytail",
    "dj-prototype",
    "dj-review",
    "dj-script",
    "dj-tdd",
    "dj-write",
];

/// Embedded skill content. Each entry is (skill_name, SKILL.md content).
/// These are populated at compile time via include_str!.
fn embedded_skills() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "dj-audit",
            include_str!("../templates/skills/dj-audit/SKILL.md"),
        ),
        (
            "dj-check",
            include_str!("../templates/skills/dj-check/SKILL.md"),
        ),
        (
            "dj-debt",
            include_str!("../templates/skills/dj-debt/SKILL.md"),
        ),
        (
            "dj-design",
            include_str!("../templates/skills/dj-design/SKILL.md"),
        ),
        (
            "dj-dispatch",
            include_str!("../templates/skills/dj-dispatch/SKILL.md"),
        ),
        (
            "dj-grill",
            include_str!("../templates/skills/dj-grill/SKILL.md"),
        ),
        (
            "dj-handoff",
            include_str!("../templates/skills/dj-handoff/SKILL.md"),
        ),
        (
            "dj-health",
            include_str!("../templates/skills/dj-health/SKILL.md"),
        ),
        (
            "dj-hunt",
            include_str!("../templates/skills/dj-hunt/SKILL.md"),
        ),
        (
            "dj-implement",
            include_str!("../templates/skills/dj-implement/SKILL.md"),
        ),
        (
            "dj-karpathy",
            include_str!("../templates/skills/dj-karpathy/SKILL.md"),
        ),
        (
            "dj-output",
            include_str!("../templates/skills/dj-output/SKILL.md"),
        ),
        (
            "dj-pattern",
            include_str!("../templates/skills/dj-pattern/SKILL.md"),
        ),
        (
            "dj-ponytail",
            include_str!("../templates/skills/dj-ponytail/SKILL.md"),
        ),
        (
            "dj-prototype",
            include_str!("../templates/skills/dj-prototype/SKILL.md"),
        ),
        (
            "dj-review",
            include_str!("../templates/skills/dj-review/SKILL.md"),
        ),
        (
            "dj-script",
            include_str!("../templates/skills/dj-script/SKILL.md"),
        ),
        (
            "dj-tdd",
            include_str!("../templates/skills/dj-tdd/SKILL.md"),
        ),
        (
            "dj-write",
            include_str!("../templates/skills/dj-write/SKILL.md"),
        ),
    ]
}

/// Return the global skill template directory: `~/.dijiang/skills/`.
fn global_skills_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("cannot determine home directory")?;
    Ok(home.join(".dijiang").join("skills"))
}

/// Get embedded skill content by name.
pub fn get_skill_content(name: &str) -> Option<&'static str> {
    embedded_skills()
        .into_iter()
        .find(|(n, _)| *n == name)
        .map(|(_, content)| content)
}

/// Ensure the global skill template directory exists and is populated
/// from embedded resources. Only writes if the directory is missing or empty.
pub fn ensure_global_skills() -> Result<PathBuf> {
    let dir = global_skills_dir()?;

    // Already populated — skip.
    if dir.exists() && dir.read_dir()?.next().is_some() {
        return Ok(dir);
    }

    fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    for (name, content) in embedded_skills() {
        let skill_dir = dir.join(name);
        fs::create_dir_all(&skill_dir)?;
        fs::write(skill_dir.join("SKILL.md"), content)?;
    }

    Ok(dir)
}

/// Write dj-* skills from the global template directory into a project's
/// `.pi/skills/` directory. Skips skills that already exist in the project.
pub fn write_project_skills(project_dir: &Path) -> Result<usize> {
    let global_dir = ensure_global_skills()?;
    let pi_skills = project_dir.join(".pi").join("skills");

    let mut written = 0usize;
    for name in DJ_SKILL_NAMES {
        let src = global_dir.join(name).join("SKILL.md");
        if !src.exists() {
            continue;
        }

        let dst_dir = pi_skills.join(name);
        let dst = dst_dir.join("SKILL.md");

        // Don't overwrite existing project skills.
        if dst.exists() {
            continue;
        }

        fs::create_dir_all(&dst_dir)?;
        fs::copy(&src, &dst)?;
        written += 1;
    }

    Ok(written)
}

/// List all managed dj-* skill names.
pub fn list_skill_names() -> &'static [&'static str] {
    DJ_SKILL_NAMES
}
