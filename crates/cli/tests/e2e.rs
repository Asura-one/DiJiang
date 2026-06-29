//! End-to-end integration tests for dijiang CLI.
//!
//! These tests build and run the `dijiang` binary as a subprocess,
//! simulating real user workflows: init, task lifecycle, template commands.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Locate the `dijiang` binary.
///
/// `cargo test` sets `CARGO_BIN_EXE_dijiang` when the test is in the
/// same package as the binary target.  Fall back to the build directory
/// relative to the workspace root.
fn dijiang_bin() -> PathBuf {
    if let Some(path) = std::env::var_os("CARGO_BIN_EXE_dijiang") {
        return PathBuf::from(path);
    }
    // When running `cargo test -p dijiang`, the test binary lives under
    // target/debug/ alongside the main binary.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("../../target/debug/dijiang")
}

/// Run `dijiang <args>` in `cwd`, returning stdout.
fn dijang(args: &[&str], cwd: &Path) -> Result<String, String> {
    let bin = dijiang_bin();
    if !bin.exists() {
        return Err(format!(
            "Binary not found at {}. Run `cargo build -p dijiang` first.",
            bin.display()
        ));
    }
    let output = Command::new(&bin)
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("Failed to execute {}: {e}", bin.display()))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !output.status.success() {
        return Err(format!(
            "dijiang {} exited with {}:\n  stdout: {stdout}\n  stderr: {stderr}",
            args.join(" "),
            output.status,
        ));
    }
    Ok(stdout)
}

/// Initialize a temporary dijiang project and return its path.
fn init_project() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let project_dir = tmp.path().join("e2e-test");
    std::fs::create_dir_all(&project_dir).unwrap();

    // Init git repo (required for developer detection)
    Command::new("git")
        .args(["init"])
        .current_dir(&project_dir)
        .output()
        .expect("git init");
    Command::new("git")
        .args(["config", "user.email", "e2e@test.local"])
        .current_dir(&project_dir)
        .output()
        .expect("git config email");
    Command::new("git")
        .args(["config", "user.name", "E2E Test"])
        .current_dir(&project_dir)
        .output()
        .expect("git config name");

    let _ = dijang(
        &[
            "init",
            "--yes",
            "--developer",
            "e2e",
            "--platforms",
            "pi",
            "e2e-test",
        ],
        &project_dir,
    )
    .expect("dijiang init should succeed");

    (tmp, project_dir)
}

// ─────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────

#[test]
fn test_e2e_init_creates_project_structure() {
    let (_tmp, project_dir) = init_project();

    // .dijiang/config.toml
    assert!(
        project_dir.join(".dijiang").join("config.toml").exists(),
        ".dijiang/config.toml should exist"
    );

    // .dijiang/ + .trellis/ infrastructure
    assert!(project_dir.join(".trellis").join("workflow.md").exists());
    assert!(project_dir.join(".trellis").join("tasks").exists());
    assert!(project_dir.join(".trellis").join("workspace").join("e2e").exists());
    assert!(project_dir.join(".trellis").join("spec").exists());

    // Pi platform files
    assert!(project_dir.join(".pi").join("settings.json").exists());
    assert!(project_dir.join("AGENTS.md").exists());
}

#[test]
fn test_e2e_task_lifecycle() {
    let (_tmp, project_dir) = init_project();

    // 1. List tasks (should be empty-ish, not an error)
    let list_out = dijang(&["task", "list"], &project_dir).unwrap();
    assert!(
        list_out.contains("Tasks") || list_out.contains("task") || list_out.is_empty(),
        "task list should not error: {list_out}"
    );

    // 2. Start a task
    dijang(&["task", "start", "e2e-task"], &project_dir).unwrap();

    // 3. Check current task
    let current_out = dijang(&["task", "current"], &project_dir).unwrap();
    assert!(
        current_out.contains("e2e-task"),
        "current task should be 'e2e-task': {current_out}"
    );

    // 4. Update status
    dijang(
        &["task", "status", "e2e-task", "in_progress"],
        &project_dir,
    )
    .unwrap();

    // 5. Archive the task
    dijang(&["task", "archive", "e2e-task"], &project_dir).unwrap();

    // 6. Prune archived tasks older than 0 days
    let _prune_out = dijang(&["task", "prune", "--days", "0"], &project_dir).unwrap();
    // After pruning, the task directory should be gone
    assert!(
        !project_dir.join(".dijiang").join("tasks").join("e2e-task").exists(),
        "pruned task directory should be removed"
    );
}

#[test]
fn test_e2e_template_list() {
    let (_tmp, project_dir) = init_project();

    let list_out = dijang(&["template", "list"], &project_dir).unwrap();
    assert!(
        list_out.contains("default-rust"),
        "template list should show 'default-rust': {list_out}"
    );
}

#[test]
fn test_e2e_template_validate() {
    // Validate the built-in template package directly from the source
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cwd = std::env::temp_dir();
    let package_path = manifest_dir
        .join("../../crates/configurator/templates/packages/default-rust")
        .canonicalize()
        .unwrap();

    let package_str = package_path.to_string_lossy().to_string();
    let validate_out = dijang(&["template", "validate", &package_str], &cwd).unwrap();
    assert!(
        validate_out.contains("valid"),
        "template validate should report valid: {validate_out}"
    );
    assert!(validate_out.contains("default-rust"));
}

#[test]
fn test_e2e_binary_exists() {
    let bin = dijiang_bin();
    assert!(bin.exists(), "dijiang binary should exist at {}", bin.display());
}
