use crate::{ConfigError, ConfiguratorRegistry, DijiangConfig, PlatformKind};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const HASHES_FILE: &str = ".template-hashes.json";

#[derive(Debug, Clone, Copy, Default)]
pub struct UpdateOptions {
    pub force: bool,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct UpdateReport {
    pub updated: Vec<String>,
    pub unchanged: Vec<String>,
    pub conflicts: Vec<String>,
    pub warnings: Vec<String>,
    pub removed: Vec<String>,
}

impl UpdateReport {
    pub fn is_clean(&self) -> bool {
        self.conflicts.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UpdatePolicy {
    Managed,
    HashProtected,
}

#[derive(Debug)]
struct ManagedFile {
    path: String,
    policy: UpdatePolicy,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct TemplateHashes(BTreeMap<String, String>);

pub fn update_project(cwd: &Path, options: UpdateOptions) -> Result<UpdateReport, ConfigError> {
    let dijiang_dir = cwd.join(".dijiang");
    if !dijiang_dir.exists() {
        return Err(ConfigError::InvalidPath(
            "未找到 .dijiang/ 目录。请先运行 `dijiang init`。".to_string(),
        ));
    }

    let config = read_existing_config(cwd);
    let platforms = configured_platforms(cwd, config.as_ref());
    let temp = GeneratedProject::new()?;
    let project_name = config
        .as_ref()
        .map(|cfg| cfg.project.name.as_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| {
            cwd.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("dijiang-project")
        });
    let developer = config
        .as_ref()
        .and_then(|cfg| cfg.project.developer.as_deref());

    crate::PiConfigurator::write_dijiang_config(temp.path(), project_name, developer)?;
    crate::write_dijiang_infrastructure(temp.path(), developer, crate::ConflictPolicy::Merge)?;
    crate::PiConfigurator::write_agents_md(temp.path())?;
    let registry = ConfiguratorRegistry::with_all();
    for (_, result) in registry.configure(temp.path(), &platforms) {
        result?;
    }
    crate::write_project_skills(temp.path()).map_err(|e| ConfigError::Serialize(e.to_string()))?;
    let mut managed_files = managed_files_for_platforms(&platforms);
    managed_files.extend(skill_files());
    managed_files.extend(project_skill_files());
    managed_files.push(ManagedFile {
        path: "AGENTS.md".to_string(),
        policy: UpdatePolicy::Managed,
    });
    managed_files.push(ManagedFile {
        path: ".dijiang/workflow.md".to_string(),
        policy: UpdatePolicy::HashProtected,
    });

    let mut hashes = load_hashes(&dijiang_dir)?;
    let mut report = UpdateReport::default();
    record_duplicate_skill_dirs(cwd, &mut report, options.force)?;
    let mut seen = BTreeSet::new();

    for managed in managed_files {
        if !seen.insert(managed.path.clone()) {
            continue;
        }
        let src = temp.path().join(&managed.path);
        if !src.exists() {
            continue;
        }
        apply_managed_file(cwd, &managed, &src, &mut hashes, &mut report, options.force)?;
    }

    let config_updated =
        update_config(cwd, &platforms, config.as_ref(), &mut hashes, options.force)?;
    match config_updated {
        FileUpdate::Updated(path) => report.updated.push(path),
        FileUpdate::Unchanged(path) => report.unchanged.push(path),
        FileUpdate::Conflict(path) => report.conflicts.push(path),
        FileUpdate::Missing => {}
    }

    save_hashes(&dijiang_dir, &hashes)?;
    Ok(report)
}

fn read_existing_config(cwd: &Path) -> Option<DijiangConfig> {
    fs::read_to_string(cwd.join(".dijiang/config.toml"))
        .ok()
        .and_then(|content| toml::from_str(&content).ok())
}

fn configured_platforms(cwd: &Path, config: Option<&DijiangConfig>) -> Vec<PlatformKind> {
    let mut platforms = Vec::new();

    if let Some(configured) = config.and_then(|cfg| cfg.platforms.as_ref()) {
        for platform in configured
            .iter()
            .filter_map(|name| platform_from_name(name))
        {
            push_unique(&mut platforms, platform);
        }
    }

    if cwd.join(".pi").exists() {
        push_unique(&mut platforms, PlatformKind::Pi);
    }
    if cwd.join(".cursor").exists() {
        push_unique(&mut platforms, PlatformKind::Cursor);
    }
    if cwd.join(".claude").exists() || cwd.join("CLAUDE.md").exists() {
        push_unique(&mut platforms, PlatformKind::Claude);
    }
    if cwd.join(".codex").exists() {
        push_unique(&mut platforms, PlatformKind::Codex);
    }
    if cwd.join(".opencode").exists() {
        push_unique(&mut platforms, PlatformKind::OpenCode);
    }
    if cwd.join(".hermes").exists() {
        push_unique(&mut platforms, PlatformKind::Hermes);
    }

    if platforms.is_empty() {
        platforms.push(PlatformKind::Pi);
    }
    platforms
}

fn push_unique(platforms: &mut Vec<PlatformKind>, platform: PlatformKind) {
    if !platforms.contains(&platform) {
        platforms.push(platform);
    }
}

fn platform_from_name(name: &str) -> Option<PlatformKind> {
    match name.trim().to_ascii_lowercase().as_str() {
        "pi" => Some(PlatformKind::Pi),
        "cursor" => Some(PlatformKind::Cursor),
        "claude" | "claude_code" | "claude-code" => Some(PlatformKind::Claude),
        "codex" | "codex_cli" | "codex-cli" => Some(PlatformKind::Codex),
        "opencode" | "open_code" | "open-code" => Some(PlatformKind::OpenCode),
        "hermes" => Some(PlatformKind::Hermes),
        _ => None,
    }
}

fn platform_name(platform: PlatformKind) -> &'static str {
    match platform {
        PlatformKind::Pi => "pi",
        PlatformKind::Cursor => "cursor",
        PlatformKind::Claude => "claude",
        PlatformKind::Codex => "codex",
        PlatformKind::OpenCode => "opencode",
        PlatformKind::Hermes => "hermes",
    }
}

fn managed_files_for_platforms(platforms: &[PlatformKind]) -> Vec<ManagedFile> {
    let mut files = Vec::new();
    for platform in platforms {
        match platform {
            PlatformKind::Pi => files.extend([
                managed(".pi/settings.json"),
                managed(".pi/prompts/dijiang-start.md"),
                managed(".pi/prompts/dijiang-finish-work.md"),
                managed(".pi/extensions/dijiang/index.ts"),
                protected(".pi/agents/dijiang-implement.md"),
                protected(".pi/agents/dijiang-check.md"),
                protected(".pi/agents/dijiang-research.md"),
            ]),
            PlatformKind::Cursor => files.extend([
                managed(".cursor/rules/dijiang.mdc"),
                managed(".cursor/hooks.json"),
                managed(".cursor/hooks/inject-workflow-state.py"),
            ]),
            PlatformKind::Claude => files.extend([
                protected("CLAUDE.md"),
                managed(".claude/settings.json"),
                managed(".claude/hooks/inject-workflow-state.py"),
            ]),
            PlatformKind::Codex => files.extend([
                protected(".codex/agents/dijiang-implement.toml"),
                protected(".codex/agents/dijiang-check.toml"),
                managed(".codex/hooks/inject-workflow-state.py"),
                managed(".codex/hooks.json"),
                managed(".codex/config.toml"),
            ]),
            PlatformKind::OpenCode => files.extend([
                protected(".opencode/agents/dijiang-implement.md"),
                protected(".opencode/agents/dijiang-check.md"),
                managed(".opencode/plugins/session-start.js"),
                managed(".opencode/lib/dijiang-context.js"),
                managed(".opencode/lib/session-utils.js"),
                managed(".opencode/package.json"),
            ]),
            PlatformKind::Hermes => files.extend([
                protected(".hermes/agents/dijiang-implement.md"),
                protected(".hermes/agents/dijiang-check.md"),
                managed(".hermes/hooks.json"),
            ]),
        }
    }
    files
}

fn skill_files() -> Vec<ManagedFile> {
    crate::dj_skills::list_skill_names()
        .iter()
        .map(|name| protected(&format!(".pi/skills/{name}/SKILL.md")))
        .collect()
}

fn project_skill_files() -> Vec<ManagedFile> {
    ["dijiang-start", "dijiang-continue", "dijiang-finish-work"]
        .iter()
        .map(|name| protected(&format!(".pi/skills/{name}/SKILL.md")))
        .collect()
}

fn managed(path: &str) -> ManagedFile {
    ManagedFile {
        path: path.to_string(),
        policy: UpdatePolicy::Managed,
    }
}

fn protected(path: &str) -> ManagedFile {
    ManagedFile {
        path: path.to_string(),
        policy: UpdatePolicy::HashProtected,
    }
}

fn record_duplicate_skill_dirs(
    cwd: &Path,
    report: &mut UpdateReport,
    force: bool,
) -> Result<(), ConfigError> {
    for name in crate::detect_skills_conflict(cwd).duplicate_dj_skill_dirs {
        let path = format!(".pi/skills/{name}");
        if force {
            fs::remove_dir_all(cwd.join(&path))?;
            report.removed.push(path);
        } else {
            report.warnings.push(format!(
                "stale duplicate generated skill directory: {path}; rerun with --force to remove"
            ));
        }
    }
    Ok(())
}

fn apply_managed_file(
    cwd: &Path,
    managed: &ManagedFile,
    src: &Path,
    hashes: &mut TemplateHashes,
    report: &mut UpdateReport,
    force: bool,
) -> Result<(), ConfigError> {
    let dst = cwd.join(&managed.path);
    let latest = fs::read(src)?;
    let latest_hash = hash_bytes(&latest);

    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }

    if !dst.exists() {
        fs::write(&dst, latest)?;
        hashes.0.insert(managed.path.clone(), latest_hash);
        set_executable_if_script(&dst)?;
        report.updated.push(managed.path.clone());
        return Ok(());
    }

    let current = fs::read(&dst)?;
    let current_hash = hash_bytes(&current);
    if current_hash == latest_hash {
        hashes.0.insert(managed.path.clone(), latest_hash);
        report.unchanged.push(managed.path.clone());
        return Ok(());
    }

    let previous_hash = hashes.0.get(&managed.path);
    let can_update = force
        || managed.policy == UpdatePolicy::Managed
        || previous_hash.is_some_and(|hash| hash == &current_hash);

    if can_update {
        fs::write(&dst, latest)?;
        hashes.0.insert(managed.path.clone(), latest_hash);
        set_executable_if_script(&dst)?;
        report.updated.push(managed.path.clone());
    } else {
        report.conflicts.push(managed.path.clone());
    }

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
enum FileUpdate {
    Updated(String),
    Unchanged(String),
    Conflict(String),
    Missing,
}

fn update_config(
    cwd: &Path,
    platforms: &[PlatformKind],
    config: Option<&DijiangConfig>,
    hashes: &mut TemplateHashes,
    force: bool,
) -> Result<FileUpdate, ConfigError> {
    let path = cwd.join(".dijiang/config.toml");
    let Some(mut next) = config.cloned() else {
        return Ok(FileUpdate::Missing);
    };

    next.platforms = Some(
        platforms
            .iter()
            .map(|platform| platform_name(*platform).to_string())
            .collect(),
    );
    if next.tasks_dir.trim().is_empty() {
        next.tasks_dir = ".dijiang/tasks".to_string();
    }
    if next.spec_dir.trim().is_empty() {
        next.spec_dir = ".dijiang/spec".to_string();
    }
    if next.workspace_dir.trim().is_empty() {
        next.workspace_dir = ".dijiang/workspace".to_string();
    }

    let latest =
        toml::to_string_pretty(&next).map_err(|e| ConfigError::Serialize(e.to_string()))?;
    let latest_hash = hash_bytes(latest.as_bytes());
    let key = ".dijiang/config.toml".to_string();
    let current = fs::read(&path)?;
    let current_hash = hash_bytes(&current);

    if current_hash == latest_hash {
        hashes.0.insert(key.clone(), latest_hash);
        return Ok(FileUpdate::Unchanged(key));
    }

    let previous_hash = hashes.0.get(&key);
    if force || previous_hash.is_some_and(|hash| hash == &current_hash) || previous_hash.is_none() {
        fs::write(&path, latest)?;
        hashes.0.insert(key.clone(), latest_hash);
        Ok(FileUpdate::Updated(key))
    } else {
        Ok(FileUpdate::Conflict(key))
    }
}

fn load_hashes(dijiang_dir: &Path) -> Result<TemplateHashes, ConfigError> {
    let path = dijiang_dir.join(HASHES_FILE);
    if !path.exists() {
        return Ok(TemplateHashes::default());
    }
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content).unwrap_or_default())
}

fn save_hashes(dijiang_dir: &Path, hashes: &TemplateHashes) -> Result<(), ConfigError> {
    fs::create_dir_all(dijiang_dir)?;
    let content =
        serde_json::to_string_pretty(hashes).map_err(|e| ConfigError::Serialize(e.to_string()))?;
    fs::write(dijiang_dir.join(HASHES_FILE), content)?;
    Ok(())
}

fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn set_executable_if_script(path: &Path) -> Result<(), ConfigError> {
    if path.extension().and_then(|ext| ext.to_str()) != Some("py") {
        return Ok(());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions)?;
    }
    Ok(())
}

struct GeneratedProject {
    path: PathBuf,
}

impl GeneratedProject {
    fn new() -> Result<Self, ConfigError> {
        let path = std::env::temp_dir().join(format!(
            "dijiang-update-{}-{}",
            std::process::id(),
            chrono_like_timestamp()
        ));
        if path.exists() {
            fs::remove_dir_all(&path)?;
        }
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for GeneratedProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn chrono_like_timestamp() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_project_refreshes_hooks_and_preserves_skill_conflicts() {
        let tmp = tempfile::TempDir::new().unwrap();
        crate::init_project_with_platforms(
            tmp.path(),
            "update-test",
            Some("dev"),
            &[PlatformKind::Pi, PlatformKind::Codex, PlatformKind::Cursor],
        )
        .unwrap();

        fs::write(
            tmp.path().join(".codex/hooks/inject-workflow-state.py"),
            "old hook",
        )
        .unwrap();
        let edited_skill = tmp.path().join(".pi/skills/dj-implement/SKILL.md");
        fs::create_dir_all(edited_skill.parent().unwrap()).unwrap();
        fs::write(&edited_skill, "# user edited skill").unwrap();
        fs::create_dir_all(tmp.path().join(".pi/skills/dj-dj-implement")).unwrap();

        let report = update_project(tmp.path(), UpdateOptions { force: false }).unwrap();
        assert!(
            report
                .updated
                .contains(&".codex/hooks/inject-workflow-state.py".to_string()),
            "managed hook should be refreshed: {report:?}"
        );
        assert!(
            report
                .conflicts
                .contains(&".pi/skills/dj-implement/SKILL.md".to_string()),
            "edited skill without previous hash should conflict: {report:?}"
        );
        assert!(
            report.warnings.iter().any(|warning| warning
                .contains("stale duplicate generated skill directory: .pi/skills/dj-dj-implement")),
            "duplicate generated skill dir should be reported without blocking update: {report:?}"
        );
        assert!(
            !report
                .conflicts
                .contains(&".pi/skills/dj-dj-implement".to_string()),
            "duplicate generated skill dir must not block update: {report:?}"
        );
        assert_eq!(
            fs::read_to_string(tmp.path().join(".pi/skills/dj-implement/SKILL.md")).unwrap(),
            "# user edited skill"
        );
        let hook =
            fs::read_to_string(tmp.path().join(".codex/hooks/inject-workflow-state.py")).unwrap();
        assert!(hook.contains("Hook error:"));
    }

    #[test]
    fn update_project_force_overwrites_conflicts_and_records_hashes() {
        let tmp = tempfile::TempDir::new().unwrap();
        crate::init_project_with_platforms(tmp.path(), "update-test", None, &[PlatformKind::Pi])
            .unwrap();
        let edited_skill = tmp.path().join(".pi/skills/dj-check/SKILL.md");
        fs::create_dir_all(edited_skill.parent().unwrap()).unwrap();
        fs::write(&edited_skill, "# local edit").unwrap();
        fs::create_dir_all(tmp.path().join(".pi/skills/dj-dj-check")).unwrap();

        let report = update_project(tmp.path(), UpdateOptions { force: true }).unwrap();
        assert!(report.conflicts.is_empty(), "force should not conflict");
        assert!(
            report
                .updated
                .contains(&".pi/skills/dj-check/SKILL.md".to_string()),
            "force should update edited skill: {report:?}"
        );
        assert!(
            report.removed.contains(&".pi/skills/dj-dj-check".to_string()),
            "force should remove duplicate generated skill dir: {report:?}"
        );
        assert!(!tmp.path().join(".pi/skills/dj-dj-check").exists());
        assert!(tmp.path().join(".dijiang/.template-hashes.json").exists());
        let skill = fs::read_to_string(tmp.path().join(".pi/skills/dj-check/SKILL.md")).unwrap();
        assert_ne!(skill, "# local edit");
    }

    #[test]
    fn update_project_infers_platforms_when_config_is_old() {
        let tmp = tempfile::TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".dijiang")).unwrap();
        fs::write(
            tmp.path().join(".dijiang/config.toml"),
            r#"[project]
name = "legacy"
version = "0.1.0"
"#,
        )
        .unwrap();
        fs::create_dir_all(tmp.path().join(".codex")).unwrap();
        fs::create_dir_all(tmp.path().join(".cursor")).unwrap();

        let report = update_project(tmp.path(), UpdateOptions { force: false }).unwrap();
        assert!(
            report.conflicts.is_empty(),
            "unexpected conflicts: {report:?}"
        );
        assert!(
            tmp.path()
                .join(".codex/hooks/inject-workflow-state.py")
                .exists()
        );
        assert!(
            tmp.path()
                .join(".cursor/hooks/inject-workflow-state.py")
                .exists()
        );
        let config = fs::read_to_string(tmp.path().join(".dijiang/config.toml")).unwrap();
        assert!(config.contains("codex"));
        assert!(config.contains("cursor"));
    }
}
