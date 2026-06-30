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
fn dijang_with_env(args: &[&str], cwd: &Path, envs: &[(&str, &str)]) -> Result<String, String> {
    let bin = dijiang_bin();
    if !bin.exists() {
        return Err(format!(
            "Binary not found at {}. Run `cargo build -p dijiang` first.",
            bin.display()
        ));
    }
    let mut command = Command::new(&bin);
    command.args(args).current_dir(cwd);
    for (key, value) in envs {
        command.env(key, value);
    }
    let output = command
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

    // .dijiang/ infrastructure
    assert!(project_dir.join(".dijiang").join("workflow.md").exists());
    assert!(project_dir.join(".dijiang").join("tasks").exists());
    assert!(
        project_dir
            .join(".dijiang")
            .join("workspace")
            .join("e2e")
            .exists()
    );
    assert!(project_dir.join(".dijiang").join("spec").exists());

    // Pi platform files
    assert!(project_dir.join(".pi").join("settings.json").exists());
    assert!(project_dir.join("AGENTS.md").exists());
}

#[test]
fn test_e2e_update_refreshes_existing_platform_hooks() {
    let (_tmp, project_dir) = init_project();
    std::fs::create_dir_all(project_dir.join(".codex/hooks")).unwrap();
    std::fs::write(
        project_dir.join(".codex/hooks/inject-workflow-state.py"),
        "old hook",
    )
    .unwrap();

    let out = dijang(&["update"], &project_dir).unwrap();
    assert!(
        out.contains(".codex/hooks/inject-workflow-state.py"),
        "update should report refreshed codex hook: {out}"
    );
    let hook =
        std::fs::read_to_string(project_dir.join(".codex/hooks/inject-workflow-state.py")).unwrap();
    assert!(hook.contains("workflow-state"));
    let config = std::fs::read_to_string(project_dir.join(".dijiang/config.toml")).unwrap();
    assert!(config.contains("codex"));
}

#[test]
fn test_e2e_update_blocks_and_force_overwrites_skill_conflicts() {
    let (_tmp, project_dir) = init_project();
    let skill = project_dir.join(".pi/skills/dj-implement/SKILL.md");
    std::fs::create_dir_all(skill.parent().unwrap()).unwrap();
    std::fs::write(&skill, "# local skill edit").unwrap();

    let bin = dijiang_bin();
    let output = Command::new(&bin)
        .args(["update"])
        .current_dir(&project_dir)
        .output()
        .expect("run update");
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains("conflict  .pi/skills/dj-implement/SKILL.md")
            || stderr.contains("update blocked"),
        "update should expose conflict; stdout={stdout:?} stderr={stderr:?}"
    );
    assert_eq!(
        std::fs::read_to_string(&skill).unwrap(),
        "# local skill edit"
    );

    let force_out = dijang(&["update", "--force"], &project_dir).unwrap();
    assert!(
        force_out.contains(".pi/skills/dj-implement/SKILL.md"),
        "force update should report skill update: {force_out}"
    );
    assert_ne!(
        std::fs::read_to_string(&skill).unwrap(),
        "# local skill edit"
    );
    assert!(project_dir.join(".dijiang/.template-hashes.json").exists());
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
    dijang(&["task", "status", "e2e-task", "in_progress"], &project_dir).unwrap();

    // 5. Archive the task
    dijang(&["task", "archive", "e2e-task"], &project_dir).unwrap();

    // 6. Prune archived tasks older than 0 days
    let _prune_out = dijang(&["task", "prune", "--days", "0"], &project_dir).unwrap();
    // After pruning, the task directory should be gone
    assert!(
        !project_dir
            .join(".dijiang")
            .join("tasks")
            .join("e2e-task")
            .exists(),
        "pruned task directory should be removed"
    );
}
#[test]
fn test_e2e_finish_work_archives_and_clears_active_task() {
    let (_tmp, project_dir) = init_project();

    dijang(&["start", "finish-e2e", "Finish E2E"], &project_dir).unwrap();
    let finish_out = dijang(
        &[
            "finish-work",
            "--summary",
            "implemented finish-work flow",
            "--verification",
            "cargo test -p dijiang-task",
            "--allow-dirty",
        ],
        &project_dir,
    )
    .unwrap();
    assert!(finish_out.contains("Finished task 'finish-e2e'"));
    assert!(finish_out.contains("Active task cleared"));
    assert!(finish_out.contains("Verification: cargo test -p dijiang-task"));
    assert!(finish_out.contains("Session closed:"));

    let current = dijang(&["task", "current"], &project_dir).unwrap();
    assert!(current.contains("(none)"), "current output: {current}");

    let task_json = std::fs::read_to_string(
        project_dir
            .join(".dijiang")
            .join("tasks")
            .join("finish-e2e")
            .join("task.json"),
    )
    .unwrap();
    assert!(task_json.contains("\"status\": \"archived\""));

    let journal = std::fs::read_to_string(
        project_dir
            .join(".dijiang")
            .join("workspace")
            .join("e2e")
            .join("journal.md"),
    )
    .unwrap();
    assert!(journal.contains("finish-e2e"));
    assert!(journal.contains("implemented finish-work flow"));
    assert!(journal.contains("cargo test -p dijiang-task"));
}

#[test]
fn test_e2e_finish_work_requires_verification() {
    let (_tmp, project_dir) = init_project();

    dijang(
        &["start", "needs-verification", "Needs Verification"],
        &project_dir,
    )
    .unwrap();
    let err = dijang(&["finish-work", "--allow-dirty"], &project_dir).unwrap_err();
    assert!(err.contains("requires --verification"), "error: {err}");
}

#[test]
fn test_e2e_finish_work_blocks_dirty_git_worktree() {
    let (_tmp, project_dir) = init_project();

    dijang(&["start", "dirty-task", "Dirty Task"], &project_dir).unwrap();
    std::fs::write(project_dir.join("dirty.txt"), "dirty").unwrap();

    let err = dijang(
        &[
            "finish-work",
            "--verification",
            "cargo test -p dijiang-task",
        ],
        &project_dir,
    )
    .unwrap_err();
    assert!(
        err.contains("git worktree has uncommitted changes"),
        "error: {err}"
    );
    assert!(err.contains("dirty.txt"), "error: {err}");

    let current = dijang(&["task", "current"], &project_dir).unwrap();
    assert!(current.contains("dirty-task"), "current output: {current}");
}

#[test]
fn test_e2e_finish_work_closes_session_on_clean_git_worktree() {
    let (_tmp, project_dir) = init_project();

    dijang_with_env(
        &["start", "clean-task", "Clean Task"],
        &project_dir,
        &[("DIJIANG_CONTEXT_ID", "finish-window")],
    )
    .unwrap();
    dijang_with_env(
        &["workflow-state"],
        &project_dir,
        &[("DIJIANG_CONTEXT_ID", "finish-window")],
    )
    .unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(&project_dir)
        .output()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-m", "baseline"])
        .current_dir(&project_dir)
        .output()
        .expect("git commit");

    let finish_out = dijang_with_env(
        &[
            "finish-work",
            "--summary",
            "clean close",
            "--verification",
            "cargo test -p dijiang-task",
        ],
        &project_dir,
        &[("DIJIANG_CONTEXT_ID", "finish-window")],
    )
    .unwrap();
    assert!(finish_out.contains("Finished task 'clean-task'"));
    assert!(finish_out.contains("Session closed:"));

    let session_journal = std::fs::read_to_string(
        project_dir
            .join(".dijiang")
            .join("workspace")
            .join("e2e")
            .join("sessions")
            .join("dijiang_finish-window.jsonl"),
    )
    .unwrap();
    assert!(session_journal.contains("workflow_state_injected"));
    assert!(session_journal.contains("session_closed"));
    assert!(session_journal.contains("clean close"));
    assert!(session_journal.contains("cargo test -p dijiang-task"));

    let session_runtime = std::fs::read_to_string(
        project_dir
            .join(".dijiang")
            .join(".runtime")
            .join("sessions")
            .join("dijiang_finish-window.json"),
    )
    .unwrap();
    assert!(session_runtime.contains("closed_at"));
    assert!(session_runtime.contains("clean-task"));

    let current = dijang_with_env(
        &["task", "current"],
        &project_dir,
        &[("DIJIANG_CONTEXT_ID", "finish-window")],
    )
    .unwrap();
    assert!(current.contains("(none)"), "current output: {current}");
}

#[test]
fn test_e2e_workflow_state_records_multi_turn_session_changes() {
    let (_tmp, project_dir) = init_project();

    dijang_with_env(
        &["start", "window-a-task", "Window A Task"],
        &project_dir,
        &[("DIJIANG_CONTEXT_ID", "window-a")],
    )
    .unwrap();
    let first_a = dijang_with_env(
        &["workflow-state"],
        &project_dir,
        &[("DIJIANG_CONTEXT_ID", "window-a")],
    )
    .unwrap();
    assert!(first_a.contains("Session: dijiang_window-a (dijiang)"));
    assert!(first_a.contains("Injection: #1"));
    assert!(first_a.contains("Active task changed: true"));
    assert!(first_a.contains("Active task: window-a-task"));
    assert!(
        first_a.contains("Session journal: .dijiang/workspace/e2e/sessions/dijiang_window-a.jsonl")
    );
    assert!(first_a.contains("Recent memory: 1 recent session event(s) loaded for this window."));
    assert!(first_a.contains("injection #1: active=window-a-task, previous=none, changed=true"));

    let second_a = dijang_with_env(
        &["workflow-state"],
        &project_dir,
        &[("DIJIANG_CONTEXT_ID", "window-a")],
    )
    .unwrap();
    assert!(second_a.contains("Injection: #2"));
    assert!(second_a.contains("Active task changed: false"));
    assert!(second_a.contains("Recent memory: 2 recent session event(s) loaded for this window."));
    assert!(second_a.contains("injection #1: active=window-a-task, previous=none, changed=true"));
    assert!(
        second_a
            .contains("injection #2: active=window-a-task, previous=window-a-task, changed=false")
    );
    assert!(second_a.contains("Other active windows: none"));

    dijang_with_env(
        &["start", "window-b-task", "Window B Task"],
        &project_dir,
        &[("DIJIANG_CONTEXT_ID", "window-b")],
    )
    .unwrap();
    let first_b = dijang_with_env(
        &["workflow-state"],
        &project_dir,
        &[("DIJIANG_CONTEXT_ID", "window-b")],
    )
    .unwrap();
    assert!(first_b.contains("Session: dijiang_window-b (dijiang)"));
    assert!(first_b.contains("Injection: #1"));
    assert!(first_b.contains("Active task: window-b-task"));
    assert!(first_b.contains("Recent memory: 1 recent session event(s) loaded for this window."));
    assert!(first_b.contains("Other active windows: 1"));
    assert!(
        first_b.contains("dijiang_window-a (dijiang) task=window-a-task state=active injections=2")
    );
    assert!(!first_b.contains("injection #1: active=window-a-task"));

    dijang_with_env(
        &["start", "window-a-next", "Window A Next"],
        &project_dir,
        &[("DIJIANG_CONTEXT_ID", "window-a")],
    )
    .unwrap();
    let changed_a = dijang_with_env(
        &["workflow-state"],
        &project_dir,
        &[("DIJIANG_CONTEXT_ID", "window-a")],
    )
    .unwrap();
    assert!(changed_a.contains("Injection: #3"));
    assert!(changed_a.contains("Active task changed: true"));
    assert!(changed_a.contains("Previous active task: window-a-task"));
    assert!(changed_a.contains("Active task: window-a-next"));
    assert!(changed_a.contains("Recent memory: 3 recent session event(s) loaded for this window."));
    assert!(
        changed_a
            .contains("injection #3: active=window-a-next, previous=window-a-task, changed=true")
    );
    assert!(changed_a.contains("Other active windows: 1"));
    assert!(
        changed_a
            .contains("dijiang_window-b (dijiang) task=window-b-task state=active injections=1")
    );

    let session_a = std::fs::read_to_string(
        project_dir
            .join(".dijiang")
            .join(".runtime")
            .join("sessions")
            .join("dijiang_window-a.json"),
    )
    .unwrap();
    assert!(session_a.contains("\"injection_count\": 3"));
    assert!(session_a.contains("window-a-next"));

    let log = std::fs::read_to_string(
        project_dir
            .join(".dijiang")
            .join(".runtime")
            .join("workflow-state.log"),
    )
    .unwrap();
    assert!(log.contains("workflow_state_injected"));
    assert!(log.contains("dijiang_window-a"));
    assert!(log.contains("dijiang_window-b"));

    let journal_a = std::fs::read_to_string(
        project_dir
            .join(".dijiang")
            .join("workspace")
            .join("e2e")
            .join("sessions")
            .join("dijiang_window-a.jsonl"),
    )
    .unwrap();
    assert_eq!(journal_a.lines().count(), 3);
    assert!(journal_a.contains("window-a-task"));
    assert!(journal_a.contains("window-a-next"));
    assert!(journal_a.contains("\"active_task_changed\":true"));

    let journal_b = std::fs::read_to_string(
        project_dir
            .join(".dijiang")
            .join("workspace")
            .join("e2e")
            .join("sessions")
            .join("dijiang_window-b.jsonl"),
    )
    .unwrap();
    assert_eq!(journal_b.lines().count(), 1);
    assert!(journal_b.contains("window-b-task"));
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
    assert!(
        bin.exists(),
        "dijiang binary should exist at {}",
        bin.display()
    );
}

#[test]
fn test_e2e_init_detects_reinit() {
    let (_tmp, project_dir) = init_project();

    // Trying to init again without --force should print Already initialized
    let out = dijang(&["init", "--yes", "e2e-test"], &project_dir).unwrap();
    assert!(out.contains("Already initialized"), "stdout: {out}");

    // With --force it should succeed
    dijang(&["init", "--yes", "--force", "e2e-test"], &project_dir).unwrap();

    // Verify config exists after re-init
    assert!(project_dir.join(".dijiang").join("config.toml").exists());
}

// ─── Review ───────────────────────────────────────────────────

#[test]
fn test_e2e_review_adversarial() {
    let (_tmp, project_dir) = init_project();
    let out = dijang(&["review", "--mode", "adversarial"], &project_dir);
    assert!(
        out.is_ok(),
        "review adversarial should succeed: {:?}",
        out.err()
    );
    let stdout = out.unwrap();
    assert!(
        stdout.contains("Adversarial"),
        "should show adversarial mode"
    );
}

#[test]
fn test_e2e_review_first_principles() {
    let (_tmp, project_dir) = init_project();
    let out = dijang(&["review", "--mode", "first-principles"], &project_dir);
    assert!(
        out.is_ok(),
        "review first-principles should succeed: {:?}",
        out.err()
    );
    let stdout = out.unwrap();
    assert!(
        stdout.contains("First Principles"),
        "should show first-principles mode"
    );
}

#[test]
fn test_e2e_review_invalid_mode() {
    let (_tmp, project_dir) = init_project();
    let out = dijang(&["review", "--mode", "invalid"], &project_dir);
    assert!(out.is_err(), "review with invalid mode should fail");
}

// ─── Channel ──────────────────────────────────────────────────

#[test]
fn test_e2e_channel_lifecycle() {
    let (_tmp, project_dir) = init_project();

    // List (empty)
    let out = dijang(&["channel", "list"], &project_dir);
    assert!(out.is_ok(), "channel list should succeed");

    // Spawn
    let out = dijang(&["channel", "spawn", "check", "--task", "."], &project_dir);
    assert!(out.is_ok(), "channel spawn should succeed: {:?}", out.err());
    let stdout = out.unwrap();
    assert!(
        stdout.contains("Agent 'check' spawned"),
        "should confirm spawn"
    );
    assert!(stdout.contains("Channel ID:"), "should output channel ID");

    // List (one active)
    let out = dijang(&["channel", "list"], &project_dir);
    assert!(out.is_ok());
    let stdout = out.unwrap();
    assert!(stdout.contains("check"), "should list check agent");
    assert!(stdout.contains("active"), "should show active status");

    // Extract channel ID
    let channel_id = stdout
        .lines()
        .find(|l| l.contains("check") && l.contains("active"))
        .and_then(|l| l.split_whitespace().next())
        .expect("should find channel ID");

    // Status
    let out = dijang(&["channel", "status", channel_id], &project_dir);
    assert!(
        out.is_ok(),
        "channel status should succeed: {:?}",
        out.err()
    );
    let stdout = out.unwrap();
    assert!(stdout.contains("id"), "should show channel id");

    // Send
    let out = dijang(
        &["channel", "send", channel_id, "test message"],
        &project_dir,
    );
    assert!(out.is_ok(), "channel send should succeed: {:?}", out.err());

    // Stop
    let out = dijang(&["channel", "stop", channel_id], &project_dir);
    assert!(out.is_ok(), "channel stop should succeed: {:?}", out.err());
    let stdout = out.unwrap();
    assert!(stdout.contains("已停止"), "should confirm stop");

    // Verify stopped
    let out = dijang(&["channel", "list"], &project_dir);
    let stdout = out.unwrap();
    assert!(stdout.contains("stopped"), "should show stopped status");
}

#[test]
fn test_e2e_channel_spawn_nonexistent_agent() {
    let (_tmp, project_dir) = init_project();
    let out = dijang(&["channel", "spawn", "nonexistent-agent"], &project_dir);
    assert!(out.is_err(), "spawning nonexistent agent should fail");
}

#[test]
fn test_e2e_channel_status_nonexistent() {
    let (_tmp, project_dir) = init_project();
    let out = dijang(&["channel", "status", "nonexistent-id"], &project_dir);
    assert!(out.is_err(), "status of nonexistent channel should fail");
}

#[test]
fn test_e2e_channel_stop_nonexistent() {
    let (_tmp, project_dir) = init_project();
    let out = dijang(&["channel", "stop", "nonexistent-id"], &project_dir);
    assert!(out.is_err(), "stopping nonexistent channel should fail");
}

#[test]
fn test_e2e_execute_all_no_active() {
    let (_tmp, project_dir) = init_project();
    let out = dijang(&["channel", "execute-all"], &project_dir);
    // Should succeed even with no active channels
    assert!(out.is_ok(), "execute-all should not error: {:?}", out.err());
    let stdout = out.unwrap();
    // Either no channels or no active channels
    assert!(
        stdout.contains("No active channels")
            || stdout.contains("No channels")
            || stdout.contains("0 active"),
        "should report no active channels, got: {}",
        stdout
    );
}

// ─── Mem ──────────────────────────────────────────────────────

#[test]
fn test_e2e_mem_record_and_findings() {
    let (_tmp, project_dir) = init_project();

    // Record learning
    let out = dijang(&["mem", "learn", "--lesson", "test-learning"], &project_dir);
    assert!(out.is_ok(), "mem learn should succeed: {:?}", out.err());

    // Record finding
    let out = dijang(
        &["mem", "findings", "--finding", "Found something important"],
        &project_dir,
    );
    assert!(out.is_ok(), "mem findings should succeed: {:?}", out.err());

    // List sessions
    let out = dijang(&["mem", "list"], &project_dir);
    assert!(out.is_ok(), "mem list should succeed: {:?}", out.err());
    let stdout = out.unwrap();
    // Should show something (sessions or empty message)
    assert!(!stdout.is_empty(), "should have some output");
}

#[test]
fn test_e2e_mem_tactics() {
    let (_tmp, project_dir) = init_project();
    let out = dijang(&["mem", "tactics"], &project_dir);
    assert!(out.is_ok(), "mem tactics should succeed: {:?}", out.err());
    let stdout = out.unwrap();
    // Should show at least cargo-test or review tactics
    assert!(
        stdout.contains("cargo-test") || stdout.contains("review") || stdout.contains("No tactics"),
        "should show tactics or empty message"
    );
}
