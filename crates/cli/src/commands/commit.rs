use crate::util::{require_dijiang_dir, run_git};
use anyhow::anyhow;
use std::path::{Path, PathBuf};

/// Options for the `dijiang commit` command.
#[derive(Debug, Clone)]
pub struct CommitOptions {
    pub dry_run: bool,
    pub message: Option<String>,
    pub force: bool,
    pub allow_empty: bool,
    /// Path to project root
    pub project_root: PathBuf,
}

/// Execute `dijiang commit` — pre-commit quality check + safe commit.
pub fn cmd_commit(opts: CommitOptions) -> anyhow::Result<()> {
    let project_root = &opts.project_root;

    // ── Step 1: Inspect dirty state ─────────────────────────────

    let dirty_raw = run_git(project_root, &["status", "--porcelain"])?;
    let dirty_files: Vec<&str> = dirty_raw.lines().map(|l| l.trim()).collect();

    if dirty_files.is_empty() {
        if opts.allow_empty {
            println!("Working tree is clean (--allow-empty). Nothing to commit.");
            return Ok(());
        }
        anyhow::bail!("Working tree is clean. Nothing to commit.");
    }

    println!("── Dirty files ({}): ─────────────────", dirty_files.len());
    for f in &dirty_files {
        println!("  {f}");
    }
    println!();

    // ── Step 2: Pre-commit quality gate ─────────────────────────

    let cargo_toml = project_root.join("Cargo.toml");
    if cargo_toml.exists() && !opts.force {
        println!("── Pre-commit check: cargo check ──────────────");
        let check_result = std::process::Command::new("cargo")
            .args(["check"])
            .current_dir(project_root)
            .output()
            .map_err(|e| anyhow!("Failed to run cargo check: {e}"))?;

        if !check_result.status.success() {
            let stderr = String::from_utf8_lossy(&check_result.stderr);
            eprintln!("cargo check FAILED. Use --force to bypass.");
            eprintln!("--- stderr (last 20 lines) ---");
            let lines: Vec<&str> = stderr.lines().collect();
            let start = lines.len().saturating_sub(20);
            for line in &lines[start..] {
                eprintln!("{line}");
            }
            std::process::exit(1);
        }
        println!("  Passed.\n");
    }

    // ── Step 3: Dry run (skip message prompt in dry-run mode) ─────

    if opts.dry_run {
        // Show recent commits for reference anyway
        let recent = run_git(project_root, &["log", "--oneline", "-5"])
            .unwrap_or_default();
        if !recent.is_empty() {
            println!("── Recent commits (for style reference): ────");
            for line in recent.lines() {
                println!("  {line}");
            }
            println!();
        }
        println!("── Dry run ─────────────────────────────────────");
        println!("Files to commit:");
        for f in &dirty_files {
            println!("  {f}");
        }
        println!("Would commit (not executed). Use -m to specify message.");
        return Ok(());
    }

    // ── Step 4: Determine commit message ────────────────────────

    let commit_msg = match &opts.message {
        Some(msg) => msg.clone(),
        None => {
            // Auto-detect commit style from recent history
            let recent = run_git(project_root, &["log", "--oneline", "-5"])
                .unwrap_or_default();
            if !recent.is_empty() {
                println!("── Recent commits (for style reference): ────");
                for line in recent.lines() {
                    println!("  {line}");
                }
                println!();
            }

            eprint!("Enter commit message: ");
            use std::io::{self, Write};
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let msg = input.trim().to_string();
            if msg.is_empty() {
                anyhow::bail!("Commit message cannot be empty.");
            }
            msg
        }
    };

    // ── Step 5: Execute commit ──────────────────────────────────

    // Separate file paths from status codes: "M src/main.rs" → "src/main.rs"
    let paths: Vec<&str> = dirty_files
        .iter()
        .filter_map(|line| {
            // `git status --porcelain` output: XY <path>
            // X = staging area, Y = working tree
            // A typical line: " M src/main.rs" or "M  src/main.rs"
            // Extract path after the two status chars + space
            line.split_whitespace()
                .nth(1)
                .or_else(|| {
                    // For renamed files: "R  old -> new"
                    line.split(" -> ").nth(1)
                })
        })
        .collect();

    if paths.is_empty() {
        anyhow::bail!("No files to add (status parsing returned empty paths).");
    }

    // Git add
    println!("── Staging files ────────────────────────────");
    for p in &paths {
        println!("  {p}");
    }

    let add_result = std::process::Command::new("git")
        .args(["add", "--"])
        .args(&paths)
        .current_dir(project_root)
        .output()
        .map_err(|e| anyhow!("git add failed: {e}"))?;

    if !add_result.status.success() {
        let stderr = String::from_utf8_lossy(&add_result.stderr);
        anyhow::bail!("git add failed:\n{stderr}");
    }

    // Git commit
    let commit_result = std::process::Command::new("git")
        .args(["commit", "-m", &commit_msg])
        .current_dir(project_root)
        .output()
        .map_err(|e| anyhow!("git commit failed: {e}"))?;

    if !commit_result.status.success() {
        let stderr = String::from_utf8_lossy(&commit_result.stderr);
        anyhow::bail!("git commit failed:\n{stderr}");
    }

    println!();
    println!("Committed: {commit_msg}");

    // Show the resulting commit
    let short = run_git(project_root, &["log", "--oneline", "-1"]).unwrap_or_default();
    if !short.is_empty() {
        println!("  {short}");
    }

    Ok(())
}
