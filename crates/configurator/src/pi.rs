use crate::templates;
use crate::types::*;
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
        let content = templates::render("extensions/dijiang/index.ts", &[])
            .map_err(|e| ConfigError::Serialize(e))?;
        let path = cwd.join(".pi").join("extensions").join("dijiang").join("index.ts");
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Write prompt templates to `.pi/prompts/`.
    fn write_prompts(cwd: &Path) -> Result<(), ConfigError> {
        let prompts_dir = cwd.join(".pi").join("prompts");

        let start = templates::render("prompts/dijiang-start.md", &[])
            .map_err(|e| ConfigError::Serialize(e))?;
        std::fs::write(prompts_dir.join("dijiang-start.md"), start)?;

        let finish = templates::render("prompts/dijiang-finish-work.md", &[])
            .map_err(|e| ConfigError::Serialize(e))?;
        std::fs::write(prompts_dir.join("dijiang-finish-work.md"), finish)?;

        Ok(())
    }

    /// Write AGENTS.md with DiJiang instructions block (safe replace).
    pub(crate) fn write_agents_md(cwd: &Path) -> Result<(), ConfigError> {
        let content = templates::render("config/agents.md", &[])
            .map_err(|e| ConfigError::Serialize(e))?;

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
        let skill_templates = [
            ("dijiang-start", "skills/dijiang-start/SKILL.md"),
            ("dijiang-continue", "skills/dijiang-continue/SKILL.md"),
            ("dijiang-finish-work", "skills/dijiang-finish-work/SKILL.md"),
        ];
        for (name, template_path) in &skill_templates {
            let dir = skills_dir.join(name);
            std::fs::create_dir_all(&dir)?;
            let content = templates::render(template_path, &[("developer", "")])
                .map_err(|e| ConfigError::Serialize(e))?;
            std::fs::write(dir.join("SKILL.md"), content)?;
        }
        Ok(())
    }

    /// Write DiJiang agent definitions to `.pi/agents/dijiang-*.md`.
    fn write_agents(cwd: &Path) -> Result<(), ConfigError> {
        let agents_dir = cwd.join(".pi").join("agents");
        let agent_templates = [
            ("dijiang-implement", "agents/dijiang-implement.md"),
            ("dijiang-check", "agents/dijiang-check.md"),
            ("dijiang-research", "agents/dijiang-research.md"),
        ];
        for (name, template_path) in &agent_templates {
            let content = templates::render(template_path, &[])
                .map_err(|e| ConfigError::Serialize(e))?;
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
    fn platform(&self) -> PlatformKind {
        PlatformKind::Pi
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

    fn is_installed(&self) -> bool {
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
        let existing = "existing content\n<!-- DIJIANG:START -->\nreplace-me\n<!-- DIJIANG:END -->\nmore content";
        std::fs::write(tmp.path().join("AGENTS.md"), existing).unwrap();
        PiConfigurator::write_agents_md(tmp.path()).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
        assert!(content.starts_with("existing content"), "bad start: {content:?}");
        assert!(content.contains("<!-- DIJIANG:START -->"), "no start marker");
        assert!(content.contains("<!-- DIJIANG:END -->"), "no end marker");
        assert!(!content.contains("replace-me"), "old content still present");
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
