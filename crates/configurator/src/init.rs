use crate::PlatformKind;
use crate::templates;
use std::path::Path;

/// Policy for handling pre-existing `.dijiang/` content when `init` runs.
///
/// DiJiang is a Rust-native independent harness that borrows best practices
/// from Trellis. When the user already has a Trellis project (or a previous
/// DiJiang init), we must not silently overwrite their data. This enum lets
/// callers pick the right trade-off.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ConflictPolicy {
    /// Refuse to write anything that would clobber existing content. Returns
    /// `ConfigError::Io` with a descriptive message listing the conflicts.
    /// This is the safest default and matches the principle "do not destroy
    /// user data".
    #[default]
    Error,
    /// Coexist with existing content: skip `tasks/` and `spec/`, insert
    /// DiJiang blocks into `workflow.md` (or skip if already present). This
    /// is what the CLI default uses — most common case for "Trellis user
    /// wants to try DiJiang skills".
    Merge,
    /// Unconditionally overwrite everything. Reserved for explicit `--force`
    /// from the CLI; never the default.
    Overwrite,
}

/// Report of conflicts detected between an existing `.dijiang/` directory
/// and what DiJiang would write.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct ConflictReport {
    pub has_dijiang_dir: bool,
    pub has_workflow_md: bool,
    pub has_tasks_dir: bool,
    pub has_spec_dir: bool,
    pub has_workspace_dir: bool,
    pub has_dijiang_block_in_workflow: bool,
}

impl ConflictReport {
    /// True if any conflict is present. An existing-but-empty `.dijiang/`
    /// does not count as a conflict (we are allowed to populate it).
    pub fn has_conflict(&self) -> bool {
        self.has_workflow_md || self.has_tasks_dir || self.has_spec_dir
    }
}

/// Marker strings used to mark DiJiang-managed regions inside `workflow.md`.
/// Exposed as constants so tests and downstream code can reference them.
pub(crate) const DIJIANG_BLOCK_BEGIN: &str =
    "<!-- BEGIN DIJIANG-MANAGED BLOCK: do not edit between these markers -->";
pub(crate) const DIJIANG_BLOCK_END: &str = "<!-- END DIJIANG-MANAGED BLOCK -->";

/// Detect what already exists under `.dijiang/` and whether `workflow.md`
/// already contains a DiJiang-managed block. Pure read-only — does not
/// mutate the filesystem.
pub(crate) fn detect_dijiang_conflict(cwd: &Path) -> ConflictReport {
    let dijiang_dir = cwd.join(".dijiang");
    let mut report = ConflictReport::default();

    if !dijiang_dir.exists() {
        return report;
    }
    report.has_dijiang_dir = true;
    report.has_tasks_dir = dijiang_dir.join("tasks").exists();
    report.has_spec_dir = dijiang_dir.join("spec").exists();
    report.has_workspace_dir = dijiang_dir.join("workspace").exists();

    let workflow_path = dijiang_dir.join("workflow.md");
    if workflow_path.exists() {
        report.has_workflow_md = true;
        if let Ok(content) = std::fs::read_to_string(&workflow_path) {
            report.has_dijiang_block_in_workflow = content.contains(DIJIANG_BLOCK_BEGIN);
        }
    }

    report
}

/// Create `.dijiang/` infrastructure: workflow.md, tasks/, workspace/, spec/.
///
/// Under the default `Merge` policy this coexists with any pre-existing
/// Trellis content:
/// - `tasks/` and `spec/` are never overwritten (and their files are preserved)
/// - `workflow.md` either gets a DiJiang block appended, or is skipped if
/// Decide whether `write_dijiang_infrastructure` should append a DiJiang
/// block to an existing `workflow.md` or write a fresh file.
///
/// Returns `true` only under the `Merge` policy when `workflow.md` exists
/// but does not already contain a DiJiang block. In every other case
/// (Overwrite, no existing file, block already present, or Error policy
/// that should never reach here because of the early return) we write
/// fresh or leave the file alone.
pub(crate) fn should_append_dijiang_block(
    policy: ConflictPolicy,
    has_workflow_md: bool,
    has_dijiang_block: bool,
) -> bool {
    matches!(
        (policy, has_workflow_md, has_dijiang_block),
        (ConflictPolicy::Merge, true, false)
    )
}

/// - `workspace/` is always ensured (empty journal dirs are safe to create)
pub(crate) fn write_dijiang_infrastructure(
    cwd: &Path,
    developer: Option<&str>,
    policy: ConflictPolicy,
) -> Result<(), crate::ConfigError> {
    let dijiang_dir = cwd.join(".dijiang");
    let report = detect_dijiang_conflict(cwd);

    if policy == ConflictPolicy::Error && report.has_conflict() {
        let mut conflicts = Vec::new();
        if report.has_workflow_md {
            conflicts.push("workflow.md");
        }
        if report.has_tasks_dir {
            conflicts.push("tasks/");
        }
        if report.has_spec_dir {
            conflicts.push("spec/");
        }
        return Err(crate::ConfigError::Io(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!(
                ".dijiang/ already exists with content that would be overwritten: {}. \
                 Re-run with --merge to coexist, or --force to overwrite.",
                conflicts.join(", ")
            ),
        )));
    }

    // tasks/ — task storage (never overwrite under Merge; safe to create
    // under Overwrite on a fresh dir; safe to skip on a populated one)
    if !report.has_tasks_dir {
        std::fs::create_dir_all(dijiang_dir.join("tasks"))?;
    }

    // workspace/ — developer journals (always ensure; empty dirs are safe)
    std::fs::create_dir_all(dijiang_dir.join("workspace"))?;
    if let Some(dev) = developer {
        std::fs::create_dir_all(dijiang_dir.join("workspace").join(dev))?;
    }

    // spec/ — coding guidelines with Chinese-first templates
    if !report.has_spec_dir {
        let spec_root = dijiang_dir.join("spec");
        std::fs::create_dir_all(&spec_root)?;
        // Populate spec directory from embedded Chinese templates
        let spec_templates: &[(&str, &str)] = &[
            ("guides/index.md", "spec/guides/index.md"),
            ("guides/verification-loop-guide.md", "spec/guides/verification-loop-guide.md"),
            ("guides/cross-layer-thinking-guide.md", "spec/guides/cross-layer-thinking-guide.md"),
            ("guides/code-reuse-thinking-guide.md", "spec/guides/code-reuse-thinking-guide.md"),
            ("guides/memory-lifecycle-guide.md", "spec/guides/memory-lifecycle-guide.md"),
            ("guides/tool-preferences.md", "spec/guides/tool-preferences.md"),
            ("backend/index.md", "spec/backend/index.md"),
            ("backend/quality-guidelines.md", "spec/backend/quality-guidelines.md"),
            ("backend/error-handling.md", "spec/backend/error-handling.md"),
            ("backend/logging-guidelines.md", "spec/backend/logging-guidelines.md"),
            ("backend/directory-structure.md", "spec/backend/directory-structure.md"),
            ("backend/database-guidelines.md", "spec/backend/database-guidelines.md"),
            ("frontend/index.md", "spec/frontend/index.md"),
            ("frontend/quality-guidelines.md", "spec/frontend/quality-guidelines.md"),
            ("frontend/type-safety.md", "spec/frontend/type-safety.md"),
            ("frontend/state-management.md", "spec/frontend/state-management.md"),
            ("frontend/component-guidelines.md", "spec/frontend/component-guidelines.md"),
            ("frontend/directory-structure.md", "spec/frontend/directory-structure.md"),
            ("frontend/hook-guidelines.md", "spec/frontend/hook-guidelines.md"),
            ("meta/index.md", "spec/meta/index.md"),
            ("meta/adr.md", "spec/meta/adr.md"),
            ("meta/contributing.md", "spec/meta/contributing.md"),
        ];
        for (rel_path, tmpl_name) in spec_templates {
            let content = templates::render(tmpl_name, &[])
                .map_err(crate::ConfigError::Serialize)?;
            let file_path = spec_root.join(rel_path);
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(file_path, content)?;
        }
    }

    // agents/ — persona definitions for AI agent roles
    let agents_root = dijiang_dir.join("agents");
    std::fs::create_dir_all(&agents_root)?;
    let agent_templates: &[(&str, &str)] = &[
        ("architect.md", "agents/dijiang-architect.md"),
        ("planner.md", "agents/dijiang-planner.md"),
        ("implementer.md", "agents/dijiang-implementer.md"),
        ("checker.md", "agents/dijiang-checker.md"),
        ("researcher.md", "agents/dijiang-researcher.md"),
    ];
    for (filename, tmpl_name) in agent_templates {
        let file_path = agents_root.join(filename);
        if !file_path.exists() {
            if let Ok(content) = templates::render(tmpl_name, &[]) {
                std::fs::write(&file_path, content)?;
            }
        }
    }


    // workflow.md — from embedded template (block-insert under Merge)
    let workflow =
        templates::render("config/workflow.md", &[]).map_err(crate::ConfigError::Serialize)?;
    let workflow_path = dijiang_dir.join("workflow.md");

    if should_append_dijiang_block(
        policy,
        report.has_workflow_md,
        report.has_dijiang_block_in_workflow,
    ) {
        let existing = std::fs::read_to_string(&workflow_path)?;
        let block = format!(
            "\n\n{}\n{}\n{}\n",
            DIJIANG_BLOCK_BEGIN, workflow, DIJIANG_BLOCK_END
        );
        std::fs::write(workflow_path, format!("{existing}{block}"))?;
    } else if !report.has_workflow_md || policy == ConflictPolicy::Overwrite {
        // No existing file, or Overwrite policy: write fresh.
        std::fs::write(workflow_path, workflow)?;
    }

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
        .unwrap_or("unnamed-project")
        .to_string()
}

/// Initialize a project with all supported platforms.
pub fn init_project(
    cwd: &Path,
    name: &str,
    developer: Option<&str>,
) -> Result<(), crate::ConfigError> {
    init_project_with_platforms(cwd, name, developer, &PlatformKind::all())
}

/// Like `init_project`, but with explicit platform selection.
///
/// The default conflict policy is `Merge` — safe for users who already have
/// a Trellis project and want to try DiJiang alongside it. Power users can
/// pass `ConflictPolicy::Overwrite` via a `--force` CLI flag.
pub fn init_project_with_platforms(
    cwd: &Path,
    name: &str,
    developer: Option<&str>,
    platforms: &[PlatformKind],
) -> Result<(), crate::ConfigError> {
    // Always write DiJiang config and .dijiang/ infrastructure (under Merge
    // policy — safe coexistence with Trellis).
    crate::PiConfigurator::write_dijiang_config(cwd, name, developer)?;
    write_dijiang_infrastructure(cwd, developer, ConflictPolicy::Merge)?;

    // Always write AGENTS.md
    crate::PiConfigurator::write_agents_md(cwd)?;

    // Use registry for platform-specific config
    let registry = crate::ConfiguratorRegistry::with_all();
    let results = registry.configure(cwd, platforms);

    for (platform, result) in results {
        if let Err(e) = result {
            eprintln!("  warning: platform config failed for {platform:?}: {e}");
        }
    }
    println!("\n[OK] Initialized DiJiang project: {name}");
    println!("  ├── .dijiang/config.toml");
    println!("  ├── .dijiang/workflow.md");
    println!("  ├── .dijiang/agents/");
    println!("  ├── .dijiang/tasks/");
    println!("  ├── .dijiang/workspace/");
    println!("  └── .dijiang/spec/");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fresh_tmpdir(label: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("dijiang-test-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    /// §1.2: detect_dijiang_conflict on a clean directory must report zero
    /// conflicts and no dijiang dir.
    #[test]
    fn detect_conflict_clean_dir() {
        let dir = fresh_tmpdir("clean");
        let report = detect_dijiang_conflict(&dir);
        assert!(!report.has_dijiang_dir);
        assert!(!report.has_conflict());
    }

    /// §1.2: detect_dijiang_conflict must surface an existing workflow.md
    /// and tasks/ as conflicts under the Error policy.
    #[test]
    fn detect_conflict_populated_dijiang() {
        let dir = fresh_tmpdir("populated");
        let dijiang_dir = dir.join(".dijiang");
        fs::create_dir_all(dijiang_dir.join("tasks")).unwrap();
        fs::write(dijiang_dir.join("workflow.md"), "# existing\n").unwrap();
        fs::create_dir_all(dijiang_dir.join("spec")).unwrap();

        let report = detect_dijiang_conflict(&dir);
        assert!(report.has_dijiang_dir);
        assert!(report.has_workflow_md);
        assert!(report.has_tasks_dir);
        assert!(report.has_spec_dir);
        assert!(report.has_conflict());
    }

    /// §1.2: write_dijiang_infrastructure under Error policy must refuse
    /// to overwrite an existing workflow.md and return an error.
    #[test]
    fn write_infrastructure_error_policy_blocks_overwrite() {
        let dir = fresh_tmpdir("error-policy");
        let dijiang_dir = dir.join(".dijiang");
        fs::create_dir_all(&dijiang_dir).unwrap();
        fs::write(dijiang_dir.join("workflow.md"), "# original\n").unwrap();

        let result = write_dijiang_infrastructure(&dir, None, ConflictPolicy::Error);
        assert!(
            result.is_err(),
            "Error policy must refuse to clobber workflow.md"
        );
        let original = fs::read_to_string(dijiang_dir.join("workflow.md")).unwrap();
        assert_eq!(original, "# original\n", "workflow.md must be untouched");
    }

    /// §1.2: write_dijiang_infrastructure under Merge policy must coexist
    /// with existing content: tasks/ and spec/ are NOT overwritten, but
    /// workflow.md gets a DiJiang-managed block appended.
    #[test]
    fn write_infrastructure_merge_coexists() {
        let dir = fresh_tmpdir("merge-coexist");
        let dijiang_dir = dir.join(".dijiang");
        fs::create_dir_all(dijiang_dir.join("00-existing")).unwrap();
        fs::write(dijiang_dir.join("00-existing/task.json"), "{}").unwrap();
        fs::create_dir_all(dijiang_dir.join("spec")).unwrap();
        fs::write(dijiang_dir.join("spec/note.md"), "user note").unwrap();
        fs::write(dijiang_dir.join("workflow.md"), "# Trellis workflow\n").unwrap();

        write_dijiang_infrastructure(&dir, Some("tiezhu"), ConflictPolicy::Merge).unwrap();

        // tasks/ contents preserved
        assert!(dijiang_dir.join("00-existing/task.json").exists());
        // spec/ contents preserved
        assert_eq!(
            fs::read_to_string(dijiang_dir.join("spec/note.md")).unwrap(),
            "user note"
        );
        // workflow.md has the original line plus a DiJiang block
        let workflow = fs::read_to_string(dijiang_dir.join("workflow.md")).unwrap();
        assert!(
            workflow.starts_with("# Trellis workflow\n"),
            "original line preserved"
        );
        assert!(
            workflow.contains(DIJIANG_BLOCK_BEGIN),
            "DiJiang block inserted"
        );
        assert!(workflow.contains(DIJIANG_BLOCK_END));
        // workspace/ created
        assert!(dijiang_dir.join("workspace/tiezhu").exists());
    }

    /// §1.2: Re-running Merge must be idempotent — the DiJiang block must
    /// not be duplicated.
    #[test]
    fn write_infrastructure_merge_is_idempotent() {
        let dir = fresh_tmpdir("merge-idem");
        let dijiang_dir = dir.join(".dijiang");
        fs::create_dir_all(&dijiang_dir).unwrap();
        fs::write(dijiang_dir.join("workflow.md"), "# existing\n").unwrap();

        write_dijiang_infrastructure(&dir, None, ConflictPolicy::Merge).unwrap();
        let after_first = fs::read_to_string(dijiang_dir.join("workflow.md")).unwrap();

        write_dijiang_infrastructure(&dir, None, ConflictPolicy::Merge).unwrap();
        let after_second = fs::read_to_string(dijiang_dir.join("workflow.md")).unwrap();

        assert_eq!(
            after_first, after_second,
            "Merge policy must be idempotent — running it twice must not change the file"
        );
        // Exactly one DiJiang block, not two
        assert_eq!(after_second.matches(DIJIANG_BLOCK_BEGIN).count(), 1);
    }
}
