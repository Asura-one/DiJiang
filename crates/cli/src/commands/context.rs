use crate::util;
use clap::ValueEnum;
use dijiang_task::store;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Context modes — what to include in the output.
#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum ContextMode {
    Status,
    Git,
    Tasks,
    Packages,
    Spec,
    All,
    Record,
}

/// Top-level context bundle (used for JSON serialization).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextBundle {
    pub developer: Option<String>,
    pub project: Option<String>,
    pub git: Option<GitContext>,
    pub active_task: Option<TaskContext>,
    pub packages: Vec<PackageContext>,
    pub spec_files: Vec<String>,
    pub workflow_state: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitContext {
    pub branch: String,
    pub status: String,
    pub recent_commits: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskContext {
    pub id: String,
    pub title: String,
    pub status: String,
    pub task_type: Option<String>,
    pub intent: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageContext {
    pub path: String,
    pub name: Option<String>,
}

// ── Main entry point ────────────────────────────────────────────────

/// Execute `dijiang context` with the given mode and optional JSON flag.
pub fn cmd_context(
    mode: Option<ContextMode>,
    json: bool,
) -> anyhow::Result<()> {
    let mode = mode.unwrap_or(ContextMode::Status);
    let project_root = &std::env::current_dir()?;

    // ── Gather raw data ─────────────────────────────────────────

    // Developer / project name
    let dijiang_dir = project_root.join(".dijiang");
    let developer = dijiang_dir.exists().then(|| {
        util::read_developer(&dijiang_dir).unwrap_or_else(|_| "developer".into())
    });
    let project = dijiang_dir.exists().then(|| {
        util::read_project_name(&dijiang_dir).unwrap_or_else(|_| "unknown".into())
    });

    // Git context
    let show_git = matches!(mode, ContextMode::Git | ContextMode::All | ContextMode::Status | ContextMode::Record);
    let git = if show_git {
        Some(collect_git_context(project_root))
    } else {
        None
    };

    // Tasks context
    let show_tasks = matches!(mode, ContextMode::Tasks | ContextMode::All | ContextMode::Status | ContextMode::Record);
    let active_task = if show_tasks {
        collect_active_task(project_root)
    } else {
        None
    };

    // Packages context
    let show_packages = matches!(mode, ContextMode::Packages | ContextMode::All | ContextMode::Record);
    let packages = if show_packages {
        collect_packages(project_root)
    } else {
        vec![]
    };

    // Spec context
    let show_spec = matches!(mode, ContextMode::Spec | ContextMode::All);
    let spec_files = if show_spec {
        collect_spec_files(project_root)
    } else {
        vec![]
    };

    // Workflow state (only in Status/All modes)
    let show_ws = matches!(mode, ContextMode::Status | ContextMode::All);
    let workflow_state = if show_ws {
        dijiang_dir.exists().then(|| {
            let state_bytes = std::process::Command::new(std::env::args().next().unwrap_or_else(|| "dijiang".into()))
                .args(&["workflow-state"])
                .current_dir(project_root)
                .output()
                .ok()
                .and_then(|o| if o.status.success() { Some(o.stdout) } else { None })
                .and_then(|b| String::from_utf8(b).ok())
                .unwrap_or_else(|| "".into());
            state_bytes
        })
    } else {
        None
    };

    let bundle = ContextBundle {
        developer,
        project,
        git,
        active_task,
        packages,
        spec_files,
        workflow_state,
    };

    // ── Output ──────────────────────────────────────────────────

    if json {
        println!("{}", serde_json::to_string_pretty(&bundle)?);
    } else {
        print_text_context(&bundle, mode);
    }

    Ok(())
}

// ── Git context ──────────────────────────────────────────────────────

fn collect_git_context(project_root: &Path) -> GitContext {
    let branch = util::git_current_branch(project_root).unwrap_or_else(|_| "unknown".into());
    let status = util::run_git(project_root, &["status", "--short"])
        .map(|s| {
            if s.is_empty() { "clean".into() } else { s }
        })
        .unwrap_or_else(|_| "unknown".into());

    let recent_commits = util::run_git(project_root, &["log", "--oneline", "-5"])
        .map(|s| s.lines().map(|l| l.to_string()).collect())
        .unwrap_or_default();

    GitContext { branch, status, recent_commits }
}

// ── Task context ─────────────────────────────────────────────────────

fn collect_active_task(project_root: &Path) -> Option<TaskContext> {
    let dijiang_dir = project_root.join(".dijiang");
    if !dijiang_dir.exists() {
        return None;
    }
    let active_task_path = dijiang_dir.join("active_task");
    let task_id = std::fs::read_to_string(&active_task_path).ok()?;
    let task_id = task_id.trim().to_string();
    if task_id.is_empty() {
        return None;
    }
    let tasks_dir = dijiang_dir.join("tasks");
    let task = store::load_task(&tasks_dir, &task_id).ok()?;

    Some(TaskContext {
        id: task.name,
        title: task.title,
        status: task.status.as_str().to_string(),
        task_type: task.dev_type,
        intent: None,
    })
}

// ── Packages context ─────────────────────────────────────────────────

fn collect_packages(project_root: &Path) -> Vec<PackageContext> {
    let crates_dir = project_root.join("crates");
    if !crates_dir.exists() {
        // Also check for Cargo.toml at root (single-crate projects)
        let root_toml = project_root.join("Cargo.toml");
        if root_toml.exists() {
            return vec![PackageContext {
                path: ".".into(),
                name: extract_crate_name(&root_toml),
            }];
        }
        return vec![];
    }

    let mut packages = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&crates_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let cargo_toml = path.join("Cargo.toml");
                if cargo_toml.exists() {
                    packages.push(PackageContext {
                        path: format!("crates/{}", path.file_name().unwrap_or_default().to_string_lossy()),
                        name: extract_crate_name(&cargo_toml),
                    });
                }
            }
        }
    }
    // Also include workspace root Cargo.toml
    let root_toml = project_root.join("Cargo.toml");
    if root_toml.exists() {
        packages.insert(0, PackageContext {
            path: ".".into(),
            name: extract_crate_name(&root_toml),
        });
    }
    packages.sort_by(|a, b| a.path.cmp(&b.path));
    packages
}

fn extract_crate_name(cargo_toml: &Path) -> Option<String> {
    let content = std::fs::read_to_string(cargo_toml).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_prefix("name = ") {
            return Some(name.trim_matches('"').to_string());
        }
    }
    None
}

// ── Spec context ─────────────────────────────────────────────────────

fn collect_spec_files(project_root: &Path) -> Vec<String> {
    let spec_dir = project_root.join(".dijiang").join("spec");
    if !spec_dir.exists() {
        return vec![];
    }
    let mut files = Vec::new();
    collect_markdown_files(&spec_dir, &spec_dir, &mut files);
    files.sort();
    files
}

fn collect_markdown_files(base: &Path, dir: &Path, files: &mut Vec<String>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_markdown_files(base, &path, files);
            } else if path.extension().map_or(false, |e| e == "md") {
                if let Ok(rel) = path.strip_prefix(base) {
                    files.push(rel.to_string_lossy().to_string());
                }
            }
        }
    }
}

// ── Text output ──────────────────────────────────────────────────────

fn print_text_context(bundle: &ContextBundle, mode: ContextMode) {
    if let Some(ref dev) = bundle.developer {
        println!("Developer: {dev}");
    }
    if let Some(ref proj) = bundle.project {
        println!("Project: {proj}");
    }

    if let Some(ref git) = bundle.git {
        println!();
        println!("── Git ──────────────────────────────────────");
        println!("Branch: {}", git.branch);
        if mode == ContextMode::Record {
            println!("Status:");
            for line in git.status.lines() {
                println!("  {line}");
            }
        } else {
            let status_preview: Vec<&str> = git.status.lines().take(5).collect();
            if status_preview.is_empty() {
                println!("Status: clean");
            } else {
                println!("Status:");
                for line in &status_preview {
                    println!("  {line}");
                }
                if git.status.lines().count() > 5 {
                    println!("  ... ({} more)", git.status.lines().count() - 5);
                }
            }
        }
        if !git.recent_commits.is_empty() {
            println!("Recent commits:");
            for c in &git.recent_commits {
                println!("  {c}");
            }
        }
    }

    if let Some(ref task) = bundle.active_task {
        println!();
        println!("── Active Task ───────────────────────────────");
        println!("ID:     {}", task.id);
        println!("Title:  {}", task.title);
        println!("Status: {}", task.status);
        if let Some(ref tt) = task.task_type {
            println!("Type:   {tt}");
        }
        if let Some(ref intent) = task.intent {
            println!("Intent: {intent}");
        }
    }

    if !bundle.packages.is_empty() {
        println!();
        println!("── Packages ──────────────────────────────────");
        for pkg in &bundle.packages {
            if let Some(ref name) = pkg.name {
                println!("  {} — {name}", pkg.path);
            } else {
                println!("  {}", pkg.path);
            }
        }
    }

    if !bundle.spec_files.is_empty() {
        println!();
        println!("── Spec Files ────────────────────────────────");
        for f in &bundle.spec_files {
            println!("  {f}");
        }
    }

    if let Some(ref ws) = bundle.workflow_state {
        println!();
        println!("── Workflow State ────────────────────────────");
        println!("{ws}");
    }
}
