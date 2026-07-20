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
    /// Previous DiJiang version (before update)
    pub old_version: Option<String>,
    /// New DiJiang version (after update)
    pub new_version: String,
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
    crate::write_project_skills(temp.path(), options.force)
        .map_err(|e| ConfigError::Serialize(e.to_string()))?;

    // Override embedded skills and per-skill references with filesystem template sources
    // if the project has a crates/configurator/templates/ directory.
    let templates_skills_dir = cwd.join("crates/configurator/templates/skills");
    if templates_skills_dir.exists() {
        copy_template_skills(&templates_skills_dir, temp.path())?;
    }

    // Sync shared references (templates/references/) if available
    let templates_references_dir = cwd.join("crates/configurator/templates/references");
    if templates_references_dir.exists() {
        let dst_references = temp.path().join(".dijiang/references");
        fs::create_dir_all(&dst_references)?;
        copy_dir_contents(&templates_references_dir, &dst_references)?;
    }

    // Sync spec guides and meta (templates/spec/) if available
    let templates_spec_dir = cwd.join("crates/configurator/templates/spec");
    if templates_spec_dir.exists() {
        let dst_spec = temp.path().join(".dijiang/spec");
        fs::create_dir_all(&dst_spec)?;
        copy_dir_contents(&templates_spec_dir, &dst_spec)?;
    }

    let mut managed_files = managed_files_for_platforms(&platforms);
    // Enumerate ALL files under .pi/skills/ in the temp dir (SKILL.md + references)
    // instead of using a static list, so new template files are picked up automatically.
    let pi_skills_dir = temp.path().join(".pi/skills");
    if pi_skills_dir.exists() {
        collect_managed_files(&pi_skills_dir, ".pi/skills", &mut managed_files);
    }
    // Enumerate ALL files under .dijiang/references/ in the temp dir
    let dijiang_references = temp.path().join(".dijiang/references");
    if dijiang_references.exists() {
        collect_managed_files(&dijiang_references, ".dijiang/references", &mut managed_files);
    }
    // Enumerate ALL files under .dijiang/spec/ in the temp dir
    let dijiang_spec = temp.path().join(".dijiang/spec");
    if dijiang_spec.exists() {
        collect_managed_files(&dijiang_spec, ".dijiang/spec", &mut managed_files);
    }
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
    // Remove stale files from fully managed directories (.pi/skills/, .dijiang/references/)
    // that exist in the project dir but no longer in the template source.
    // .dijiang/spec/ is excluded because it also holds local-only files not tracked in templates.
    remove_stale_files(cwd, temp.path(), &[".pi/skills", ".dijiang/references"], &mut report)?;

    let config_updated =
        update_config(cwd, &platforms, config.as_ref(), &mut hashes, options.force)?;
    match config_updated {
        FileUpdate::Updated(path) => report.updated.push(path),
        FileUpdate::Unchanged(path) => report.unchanged.push(path),
        FileUpdate::Conflict(path) => report.conflicts.push(path),
        FileUpdate::Missing => {}
    }

    save_hashes(&dijiang_dir, &hashes)?;

    // Install/refresh git commit-msg hook from template
    install_git_hook(cwd)?;

    let new_version = env!("CARGO_PKG_VERSION").to_string();
    let old_version = config.as_ref().map(|c| c.dijiang_version.clone());
    report.old_version = old_version;
    report.new_version = new_version;

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
                managed(".pi/prompts/dijiang-reason.md"),
                managed(".pi/extensions/dijiang/index.ts"),
                protected(".pi/agents/dijiang-implementer.md"),
                protected(".pi/agents/dijiang-checker.md"),
                protected(".pi/agents/dijiang-researcher.md"),
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
                protected(".codex/agents/dijiang-implementer.toml"),
                protected(".codex/agents/dijiang-checker.toml"),
                managed(".codex/hooks/inject-workflow-state.py"),
                managed(".codex/hooks.json"),
                managed(".codex/config.toml"),
            ]),
            PlatformKind::OpenCode => files.extend([
                protected(".opencode/agents/dijiang-implementer.md"),
                protected(".opencode/agents/dijiang-checker.md"),
                managed(".opencode/plugins/session-start.js"),
                managed(".opencode/lib/dijiang-context.js"),
                managed(".opencode/lib/session-utils.js"),
                managed(".opencode/package.json"),
            ]),
            PlatformKind::Hermes => files.extend([
                protected(".hermes/agents/dijiang-implementer.md"),
                protected(".hermes/agents/dijiang-checker.md"),
                managed(".hermes/hooks.json"),
            ]),
        }
    }
    files
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

    next.dijiang_version = env!("CARGO_PKG_VERSION").to_string();

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

/// embedded versions. Also copies per-skill reference files (e.g. references/*.md).
fn copy_template_skills(src: &Path, temp_dir: &Path) -> Result<(), ConfigError> {
    let dst_base = temp_dir.join(".pi/skills");
    for entry in fs::read_dir(src).map_err(|e| {
        ConfigError::Serialize(format!("failed to read template skills dir: {e}"))
    })? {
        let entry = entry.map_err(|e| {
            ConfigError::Serialize(format!("failed to read entry in template skills dir: {e}"))
        })?;
        let skill_name = entry.file_name();
        let skill_name = skill_name.to_string_lossy();
        if skill_name.starts_with('.') { continue; }
        let src_dir = entry.path();
        if !src_dir.is_dir() { continue; }
        let dst_dir = dst_base.join(&*skill_name);
        copy_dir_contents(&src_dir, &dst_dir)?;
    }
    Ok(())
}

/// Recursively copy all files from src_dir to dst_dir, preserving relative paths.
fn copy_dir_contents(src_dir: &Path, dst_dir: &Path) -> Result<(), ConfigError> {
    fs::create_dir_all(dst_dir).map_err(|e| {
        ConfigError::Serialize(format!("failed to create dir {}: {e}", dst_dir.display()))
    })?;
    for entry in fs::read_dir(src_dir).map_err(|e| {
        ConfigError::Serialize(format!("failed to read dir {}: {e}", src_dir.display()))
    })? {
        let entry = entry.map_err(|e| {
            ConfigError::Serialize(format!("failed to read entry in {}: {e}", src_dir.display()))
        })?;
        let file_type = entry.file_type().map_err(|e| {
            ConfigError::Serialize(format!("failed to get file type: {e}"))
        })?;
        if file_type.is_dir() {
            let sub_src = entry.path();
            let sub_name = entry.file_name();
            let sub_dst = dst_dir.join(&sub_name);
            copy_dir_contents(&sub_src, &sub_dst)?;
        } else if file_type.is_file() {
            let src_file = entry.path();
            let dst_file = dst_dir.join(entry.file_name());
            fs::copy(&src_file, &dst_file).map_err(|e| {
                ConfigError::Serialize(format!(
                    "failed to copy {} -> {}: {e}",
                    src_file.display(),
                    dst_file.display()
                ))
            })?;
        }
    }
    Ok(())
}

/// Walk a directory tree and add every file as a HashProtected ManagedFile,
/// using the given prefix (e.g. ".pi/skills") for the managed path.
fn collect_managed_files(dir: &Path, prefix: &str, managed_files: &mut Vec<ManagedFile>) {
    collect_managed_files_inner(dir, prefix, "", managed_files);
}

fn collect_managed_files_inner(
    dir: &Path,
    prefix: &str,
    subpath: &str,
    managed_files: &mut Vec<ManagedFile>,
) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with('.') && name_str != ".gitkeep" { continue; }
        let relative = if subpath.is_empty() {
            name_str.to_string()
        } else {
            format!("{subpath}/{name_str}")
        };
        if file_type.is_dir() {
            collect_managed_files_inner(&entry.path(), prefix, &relative, managed_files);
        } else if file_type.is_file() {
            managed_files.push(ManagedFile {
                path: format!("{prefix}/{relative}"),
                policy: UpdatePolicy::HashProtected,
            });
        }
    }
}

/// Remove files from managed subdirectories in the project dir that no longer exist
/// in the temp dir. This implements delete-sync for fully-managed directories.
fn remove_stale_files(
    project_dir: &Path,
    temp_dir: &Path,
    managed_roots: &[&str],
    report: &mut UpdateReport,
) -> Result<(), ConfigError> {
    for root in managed_roots {
        let project_root = project_dir.join(root);
        let temp_root = temp_dir.join(root);
        if !project_root.exists() || !temp_root.exists() {
            continue;
        }
        let relative_from = if root.starts_with('.') { root } else { root };
        remove_stale_inner(&project_root, &temp_root, relative_from, report)?;
    }
    Ok(())
}

/// Recursively walk `dir` and remove files/subdirs that don't exist in `reference_dir`.
fn remove_stale_inner(
    dir: &Path,
    reference_dir: &Path,
    prefix: &str,
    report: &mut UpdateReport,
) -> Result<(), ConfigError> {
    let entries: Vec<_> = fs::read_dir(dir)
        .map_err(|e| ConfigError::Serialize(format!("failed to read stale dir {dir:?}: {e}")))?
        .filter_map(|e| e.ok())
        .collect();
    for entry in entries {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with('.') && name_str != ".gitkeep" { continue; }
        let file_type = entry.file_type().map_err(|e| {
            ConfigError::Serialize(format!("failed to get file type: {e}"))
        })?;
        let ref_path = reference_dir.join(&name);
        if !ref_path.exists() {
            // Delete: file/subdir exists in project but not in reference
            let full_path = entry.path();
            let relative = format!("{prefix}/{name_str}");
            if file_type.is_dir() {
                fs::remove_dir_all(&full_path).map_err(|e| {
                    ConfigError::Serialize(format!("failed to remove stale dir {relative}: {e}"))
                })?;
            } else {
                fs::remove_file(&full_path).map_err(|e| {
                    ConfigError::Serialize(format!("failed to remove stale file {relative}: {e}"))
                })?;
            }
            report.removed.push(relative);
        } else if file_type.is_dir() {
            // Recurse into subdirectories that still exist
            remove_stale_inner(&entry.path(), &ref_path, &format!("{prefix}/{name_str}"), report)?;
        }
    }
    // Clean up now-empty direcories in the project dir
    let remaining: Vec<_> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .collect();
    if remaining.is_empty() {
        let _ = fs::remove_dir(dir);
    }
    Ok(())
}

/// 安装 git commit-msg hook，确保所有 commit message 含中文字符
fn install_git_hook(project_root: &Path) -> Result<(), ConfigError> {
    let hook_path = project_root.join(".git/hooks/commit-msg");
    let template_path = project_root.join("crates/configurator/templates/.git/hooks/commit-msg");

    // 如果模板文件不存在，跳过安装
    if !template_path.exists() {
        return Ok(());
    }

    let template_content = fs::read_to_string(&template_path)
        .map_err(|e| ConfigError::Serialize(format!("读取 hook 模板失败: {e}")))?;

    // 读取现有 hook 文件（如果存在）
    let existing_content = fs::read_to_string(&hook_path).ok();

    // 如果内容相同，不写入
    if existing_content.as_deref() == Some(&template_content) {
        return Ok(());
    }

    // 确保 .git/hooks 目录存在
    if let Some(parent) = hook_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| ConfigError::Serialize(format!("创建 .git/hooks 目录失败: {e}")))?;
    }

    fs::write(&hook_path, &template_content)
        .map_err(|e| ConfigError::Serialize(format!("写入 hook 文件失败: {e}")))?;

    // 设置可执行权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = hook_path.metadata()
            .map_err(|e| ConfigError::Serialize(format!("读取 hook 权限失败: {e}")))?
            .permissions();
        perms.set_mode(perms.mode() | 0o111);
        fs::set_permissions(&hook_path, perms)
            .map_err(|e| ConfigError::Serialize(format!("设置 hook 可执行权限失败: {e}")))?;
    }

    println!("  ✓ 已安装/更新 git hook: {}", hook_path.display());
    Ok(())
}

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
        assert!(hook.contains("workflow_state.py"));
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
            report
                .removed
                .contains(&".pi/skills/dj-dj-check".to_string()),
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
