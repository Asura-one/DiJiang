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

#[test]
fn test_e2e_help_describes_canonical_boundaries() {
    let (_tmp, project_dir) = init_project();

    let help = dijang(&["--help"], &project_dir).unwrap();
    assert!(help.contains("生命周期入口"));
    assert!(help.contains("原子状态操作"));
    assert!(help.contains("完成当前工作"));

    let task_help = dijang(&["task", "--help"], &project_dir).unwrap();
    assert!(task_help.contains("low-level task operation"));
}

/// Initialize a temporary dijiang project and return its path.
fn init_project() -> (tempfile::TempDir, PathBuf) {
    init_project_with_env(&[])
}

fn init_project_with_env(envs: &[(&str, &str)]) -> (tempfile::TempDir, PathBuf) {
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

    let _ = dijang_with_env(
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
        envs,
    )
    .expect("dijiang init should succeed");
    std::fs::write(project_dir.join("baseline.txt"), "baseline").unwrap();
    Command::new("git")
        .args(["add", "baseline.txt"])
        .current_dir(&project_dir)
        .output()
        .expect("git add baseline");
    Command::new("git")
        .args(["commit", "-m", "test: init baseline"])
        .current_dir(&project_dir)
        .output()
        .expect("git commit baseline");

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
fn test_e2e_init_projects_include_code_task_tdd_contract() {
    let tmp = tempfile::tempdir().expect("home tempdir");
    let home = tmp.path().join("home");
    let home_str = home.to_str().unwrap();
    let (_project_tmp, project_dir) = init_project_with_env(&[("HOME", home_str)]);

    let workflow = std::fs::read_to_string(project_dir.join(".dijiang/workflow.md")).unwrap();
    assert_code_task_tdd_contract(".dijiang/workflow.md", &workflow);

    for skill in [
        "dj-dispatch",
        "dj-implement",
        "dj-hunt",
        "dj-check",
        "dijiang-finish-work",
    ] {
        let path = format!(".pi/skills/{skill}/SKILL.md");
        let content = std::fs::read_to_string(project_dir.join(&path)).unwrap();
        assert_code_task_tdd_contract(&path, &content);
    }
}

fn assert_code_task_tdd_contract(artifact: &str, content: &str) {
    for required in [
        "Code Task TDD Contract",
        "RED/Repro evidence",
        "GREEN command",
        "Regression scope",
        "Exception",
    ] {
        assert!(
            content.contains(required),
            "{artifact} missing TDD contract marker: {required}"
        );
    }
}

#[test]
fn test_e2e_update_refreshes_existing_platform_hooks() {
    let tmp = tempfile::tempdir().expect("home tempdir");
    let home = tmp.path().join("home");
    let home_str = home.to_str().unwrap();
    let (_project_tmp, project_dir) = init_project_with_env(&[("HOME", home_str)]);
    std::fs::create_dir_all(project_dir.join(".codex/hooks")).unwrap();
    std::fs::write(
        project_dir.join(".codex/hooks/inject-workflow-state.py"),
        "old hook",
    )
    .unwrap();

    let out = dijang_with_env(&["update"], &project_dir, &[("HOME", home_str)]).unwrap();
    assert!(
        out.contains(".codex/hooks/inject-workflow-state.py"),
        "update should report refreshed codex hook: {out}"
    );
    let hook =
        std::fs::read_to_string(project_dir.join(".codex/hooks/inject-workflow-state.py")).unwrap();
    assert!(hook.contains("workflow_state.py"));
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
fn test_e2e_update_force_reports_duplicate_skill_dirs_without_blocking() {
    let (_tmp, project_dir) = init_project();
    let duplicate_dir = project_dir.join(".pi/skills/dj-dj-hunt");
    std::fs::create_dir_all(&duplicate_dir).unwrap();
    std::fs::write(duplicate_dir.join("SKILL.md"), "# stale duplicate").unwrap();

    let out = dijang(&["update", "--force"], &project_dir).unwrap();
    assert!(
        out.contains(".pi/skills/dj-dj-hunt"),
        "force update should remove duplicate skill dir: {out}"
    );
    assert!(
        out.contains("0 个冲突") || !out.contains("update blocked"),
        "duplicate skill dir should not block force update: {out}"
    );
    assert!(!duplicate_dir.exists());
}

#[test]
fn test_e2e_update_force_refreshes_global_skill_cache() {
    let (tmp, project_dir) = init_project();
    let home = tmp.path().join("home");
    let global_skill = home.join(".dijiang/skills/dj-audit/SKILL.md");
    std::fs::create_dir_all(global_skill.parent().unwrap()).unwrap();
    std::fs::write(&global_skill, "# Audit\n# 测试\n").unwrap();

    let project_skill = project_dir.join(".pi/skills/dj-audit/SKILL.md");
    std::fs::write(&project_skill, "# Audit\n# 测试\n").unwrap();

    let out = dijang_with_env(
        &["update", "--force"],
        &project_dir,
        &[("HOME", home.to_str().unwrap())],
    )
    .unwrap();
    assert!(
        out.contains(".pi/skills/dj-audit/SKILL.md"),
        "force update should refresh project skill from embedded template: {out}"
    );

    let refreshed_global = std::fs::read_to_string(&global_skill).unwrap();
    assert!(!refreshed_global.contains("# 测试"));
    let refreshed_project = std::fs::read_to_string(&project_skill).unwrap();
    assert!(!refreshed_project.contains("# 测试"));
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

    // 5. Complete the task
    dijang(&["task", "status", "e2e-task", "completed"], &project_dir).unwrap();

    // 6. Archive the task
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
fn test_e2e_dispatch_creates_task_from_natural_language() {
    let (_tmp, project_dir) = init_project();

    let out = dijang(
        &[
            "dispatch",
            "排查登录接口报错并修复",
            "--json",
            "--hook-event",
            "input",
        ],
        &project_dir,
    )
    .unwrap();
    assert!(out.contains("dijiang-dispatch"), "dispatch output: {out}");
    assert!(out.contains("路线：dj-hunt"), "dispatch output: {out}");

    let current = dijang(&["task", "current"], &project_dir).unwrap();
    assert!(!current.contains("(none)"), "current output: {current}");
    let task_name = current.trim();
    let task_json = std::fs::read_to_string(
        project_dir
            .join(".dijiang")
            .join("tasks")
            .join(task_name)
            .join("task.json"),
    )
    .unwrap();
    assert!(task_json.contains("排查登录接口报错并修复"));
    assert!(task_json.contains("dj-hunt"));
    assert!(task_json.contains("in_progress"));
}

#[test]
fn test_e2e_dispatch_discussion_of_exception_states_does_not_route_to_hunt() {
    let (_tmp, project_dir) = init_project();

    let out = dijang(
        &[
            "dispatch",
            "解释一下这些异常情况为什么全部走 hunt",
            "--force-new",
        ],
        &project_dir,
    )
    .unwrap();

    assert!(out.contains("路线：dj-grill"), "dispatch output: {out}");
    assert!(!out.contains("路线：dj-hunt"), "dispatch output: {out}");
}

#[test]
fn test_e2e_dispatch_vague_feature_routes_to_grill() {
    let (_tmp, project_dir) = init_project();

    let out = dijang(&["dispatch", "做个导出功能", "--force-new"], &project_dir).unwrap();

    assert!(out.contains("路线：dj-grill"), "dispatch output: {out}");
    assert!(out.contains("状态：planning"), "dispatch output: {out}");
}
#[test]
fn test_e2e_dispatch_specific_feature_routes_to_implement() {
    let (_tmp, project_dir) = init_project();

    let out = dijang(
        &["dispatch", "新增一个导出按钮", "--force-new"],
        &project_dir,
    )
    .unwrap();

    assert!(out.contains("路线：dj-implement"), "dispatch output: {out}");
    assert!(out.contains("action：allow"), "dispatch output: {out}");
    assert!(
        out.contains("nextAction：continue with the requested skill for the new task"),
        "dispatch output: {out}"
    );
    assert!(
        out.contains("Git 工作流：Git Gate=provisioned；已创建任务 worktree"),
        "dispatch output: {out}"
    );
    assert!(out.contains("状态：in_progress"), "dispatch output: {out}");
}

#[test]
fn test_e2e_dispatch_redirects_planning_implement_to_grill_for_active_task() {
    let (_tmp, project_dir) = init_project();

    dijang(&["start", "route-gate", "Route Gate"], &project_dir).unwrap();
    let out = dijang(&["dispatch", "新增一个导出按钮"], &project_dir).unwrap();

    assert!(out.contains("路线：dj-grill"), "dispatch output: {out}");
    assert!(out.contains("action：redirect"), "dispatch output: {out}");
    assert!(
        out.contains(
            "reason：planning tasks are hard-gated to alignment before implementation-oriented work"
        ),
        "dispatch output: {out}"
    );
    assert!(
        out.contains(
            "nextAction：continue with dj-grill to produce a confirmed requirement summary"
        ),
        "dispatch output: {out}"
    );
    assert!(
        out.contains("Git 工作流：当前路线不需要立即创建代码 worktree。"),
        "dispatch output: {out}"
    );
}

#[test]
fn test_e2e_dispatch_paused_task_redirects_to_continue() {
    let (_tmp, project_dir) = init_project();

    dijang(&["start", "paused-task", "Paused Task"], &project_dir).unwrap();
    dijang(&["task", "status", "paused-task", "in_progress"], &project_dir).unwrap();
    dijang(&["task", "status", "paused-task", "paused"], &project_dir).unwrap();
    let out = dijang(&["dispatch", "新增一个导出按钮"], &project_dir).unwrap();

    assert!(
        out.contains("路线：dijiang-continue"),
        "dispatch output: {out}"
    );
    assert!(out.contains("action：redirect"), "dispatch output: {out}");
    assert!(
        out.contains("nextAction：continue with dijiang-continue, then re-evaluate the next skill"),
        "dispatch output: {out}"
    );
}

#[test]
fn test_e2e_dispatch_archived_task_creates_new_task() {
    let (_tmp, project_dir) = init_project();

    dijang(&["start", "archived-task", "Archived Task"], &project_dir).unwrap();
    dijang(
        &["task", "status", "archived-task", "archived"],
        &project_dir,
    )
    .unwrap();
    let out = dijang(&["dispatch", "新增一个导出按钮"], &project_dir).unwrap();

    // After archiving, active task pointer is cleared.
    // Dispatch creates a new in_progress task instead of blocking.
    assert!(
        out.contains("action：allow"),
        "dispatch output: {out}"
    );
    assert!(
        out.contains("路线：dj-implement"),
        "dispatch output: {out}"
    );
}

#[test]
fn test_e2e_dispatch_blocks_implement_route_from_main_checkout_when_task_worktree_exists() {
    let (_tmp, project_dir) = init_project();

    std::fs::write(project_dir.join("base.txt"), "base").unwrap();
    Command::new("git")
        .args(["add", "base.txt"])
        .current_dir(&project_dir)
        .output()
        .expect("git add base");
    Command::new("git")
        .args(["commit", "-m", "test: base"])
        .current_dir(&project_dir)
        .output()
        .expect("git commit base");

    dijang(
        &["dispatch", "新增一个导出按钮", "--force-new"],
        &project_dir,
    )
    .unwrap();
    let task_name = dijang(&["task", "current"], &project_dir).unwrap();
    let task_name = task_name.trim();
    let task_json = std::fs::read_to_string(
        project_dir
            .join(".dijiang")
            .join("tasks")
            .join(task_name)
            .join("task.json"),
    )
    .unwrap();
    let task: serde_json::Value = serde_json::from_str(&task_json).unwrap();
    let worktree_path = task["worktreePath"].as_str().unwrap();
    assert!(Path::new(worktree_path).exists());

    let out = dijang(&["dispatch", "实现一个导出按钮"], &project_dir).unwrap();
    assert!(out.contains("Git Gate=blocked"), "dispatch output: {out}");
    assert!(
        out.contains("当前 runtime 尚未进入正确位置"),
        "dispatch output: {out}"
    );
    assert!(out.contains("expected"), "dispatch output: {out}");

    let json_out = dijang(&["dispatch", "实现一个导出按钮", "--json"], &project_dir).unwrap();
    let payload: serde_json::Value = serde_json::from_str(&json_out).unwrap();
    assert_eq!(payload["gitGate"]["state"], "blocked");
    assert_eq!(payload["gitGate"]["needsProvision"], false);
    assert!(payload["gitGate"]["locationKind"].as_str().is_some());

    let worktree_dir = PathBuf::from(worktree_path);
    assert!(worktree_dir.exists());
}
#[test]
fn test_e2e_start_keeps_new_task_in_planning_for_dispatch() {
    let (_tmp, project_dir) = init_project();

    let out = dijang(&["start", "new-session", "New Session"], &project_dir).unwrap();

    assert!(out.contains("Status: planning"), "start output: {out}");
    assert!(out.contains("State:  planning"), "start output: {out}");
    assert!(out.contains("Phase:  plan"), "start output: {out}");

    let task_json = std::fs::read_to_string(
        project_dir
            .join(".dijiang")
            .join("tasks")
            .join("new-session")
            .join("task.json"),
    )
    .unwrap();
    assert!(task_json.contains(r#""status": "planning""#));
}
#[test]
fn test_e2e_dispatch_reuses_active_task_by_default() {
    let (_tmp, project_dir) = init_project();

    dijang(&["dispatch", "实现一个导出按钮"], &project_dir).unwrap();
    let first = dijang(&["task", "current"], &project_dir).unwrap();
    let second_out = dijang(&["dispatch", "再补充一个细节"], &project_dir).unwrap();
    let second = dijang(&["task", "current"], &project_dir).unwrap();

    assert_eq!(first.trim(), second.trim());
    assert!(second_out.contains("任务："));
}

#[test]
fn test_e2e_dispatch_active_task_is_session_scoped() {
    let (_tmp, project_dir) = init_project();

    dijang_with_env(
        &["dispatch", "实现窗口 A 功能"],
        &project_dir,
        &[("DIJIANG_CONTEXT_ID", "window-a")],
    )
    .unwrap();
    let current_a = dijang_with_env(
        &["task", "current"],
        &project_dir,
        &[("DIJIANG_CONTEXT_ID", "window-a")],
    )
    .unwrap();

    dijang_with_env(
        &["dispatch", "实现窗口 B 功能"],
        &project_dir,
        &[("DIJIANG_CONTEXT_ID", "window-b")],
    )
    .unwrap();
    let current_b = dijang_with_env(
        &["task", "current"],
        &project_dir,
        &[("DIJIANG_CONTEXT_ID", "window-b")],
    )
    .unwrap();
    let current_a_after_b = dijang_with_env(
        &["task", "current"],
        &project_dir,
        &[("DIJIANG_CONTEXT_ID", "window-a")],
    )
    .unwrap();

    assert_ne!(current_a.trim(), current_b.trim());
    assert_eq!(current_a.trim(), current_a_after_b.trim());

    let no_session_current = dijang(&["task", "current"], &project_dir).unwrap();
    assert!(
        no_session_current.contains("(none)"),
        "current output: {no_session_current}"
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
    assert!(finish_out.contains("已完成任务 'finish-e2e'"));
    assert!(finish_out.contains("当前 session 的 active task 已清理"));
    assert!(finish_out.contains("验证：cargo test -p dijiang-task"));
    assert!(finish_out.contains("Task archive：archived task `finish-e2e`"), "finish_out: {finish_out}");
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

    let closures = std::fs::read_to_string(
        project_dir
            .join(".dijiang")
            .join("memory")
            .join("sessions.jsonl"),
    )
    .unwrap();
    assert!(closures.contains("finish-e2e"), "closures: {closures}");
    assert!(
        closures.contains("cargo test -p dijiang-task"),
        "closures: {closures}"
    );
    assert!(closures.contains(r#""verification":"cargo test -p dijiang-task""#), "closures: {closures}");
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
fn test_e2e_finish_work_allows_missing_active_task() {
    let (_tmp, project_dir) = init_project();
    let out = dijang(
        &[
            "finish-work",
            "--verification",
            "manual check",
            "--docs-sync",
            "none: no task work",
            "--version-impact",
            "none",
            "--allow-dirty",
        ],
        &project_dir,
    )
    .unwrap();

    assert!(out.contains("已完成工作（无 active task"), "output: {out}");
    assert!(
        out.contains("Task archive：skipped: no active task"),
        "output: {out}"
    );
    assert!(
        out.contains("当前 session 没有 active task 需要清理"),
        "output: {out}"
    );
    let journal = std::fs::read_to_string(
        project_dir
            .join(".dijiang")
            .join("workspace")
            .join("e2e")
            .join("journal.md"),
    )
    .unwrap();
    assert!(journal.contains("no-active-task"));
    assert!(journal.contains("completed-no-task"));

    let closures = std::fs::read_to_string(
        project_dir
            .join(".dijiang")
            .join("memory")
            .join("sessions.jsonl"),
    )
    .unwrap();
    assert!(closures.contains("no-active-task"), "closures: {closures}");
    assert!(closures.contains("manual check"), "closures: {closures}");

    let current = dijang(&["task", "current"], &project_dir).unwrap();
    assert!(current.contains("(none)"), "current output: {current}");
}

#[test]
fn test_e2e_finish_work_commit_without_active_task_skips_archive() {
    let (_tmp, project_dir) = init_project();
    std::fs::write(project_dir.join("change.txt"), "changed").unwrap();

    let out = dijang(
        &[
            "finish-work",
            "--summary",
            "standalone finish",
            "--verification",
            "manual check",
            "--docs-sync",
            "none: standalone change",
            "--version-impact",
            "none",
            "--commit",
            "--commit-message",
            "test(cli): 测试独立 finish 功能",
        ],
        &project_dir,
    )
    .unwrap();

    assert!(out.contains("已完成工作（无 active task"), "output: {out}");
    assert!(
        out.contains("Task archive：skipped: no active task"),
        "output: {out}"
    );
    assert!(out.contains("Commit："), "output: {out}");

    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&project_dir)
        .output()
        .expect("git status");
    assert_eq!(String::from_utf8_lossy(&status.stdout).trim(), "");

    let log = Command::new("git")
        .args(["log", "--oneline", "-1"])
        .current_dir(&project_dir)
        .output()
        .expect("git log");
    assert!(
        String::from_utf8_lossy(&log.stdout).contains("test(cli): 测试独立 finish 功能"),
        "log: {}",
        String::from_utf8_lossy(&log.stdout)
    );
}

#[test]
fn test_e2e_finish_work_commit_from_git_worktree_without_local_dijiang() {
    let (tmp, project_dir) = init_project();
    std::fs::write(project_dir.join("base.txt"), "base").unwrap();
    Command::new("git")
        .args(["add", "base.txt"])
        .current_dir(&project_dir)
        .output()
        .expect("git add base");
    Command::new("git")
        .args(["commit", "-m", "test: base"])
        .current_dir(&project_dir)
        .output()
        .expect("git commit base");

    let worktree_dir = tmp.path().join("external-worktree");
    Command::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            "finish-work-external",
            worktree_dir.to_str().unwrap(),
        ])
        .current_dir(&project_dir)
        .output()
        .expect("git worktree add");
    assert!(!worktree_dir.join(".dijiang").exists());
    dijang(
        &["start", "finish-work-external", "Finish Work External"],
        &project_dir,
    )
    .unwrap();
    let task_json = project_dir.join(".dijiang/tasks/finish-work-external/task.json");
    let mut task = std::fs::read_to_string(&task_json).expect("task json exists");
    task = task.replace("\"branch\": null", "\"branch\": \"finish-work-external\"");
    task = task.replace(
        "\"worktreePath\": null",
        &format!("\"worktreePath\": \"{}\"", worktree_dir.display()),
    );
    std::fs::write(&task_json, task).unwrap();
    std::fs::write(
        project_dir.join(".dijiang/active_task.txt"),
        "stale-main-worktree-task",
    )
    .unwrap();
    let parent_dijiang = tmp.path().join(".dijiang");
    std::fs::create_dir_all(&parent_dijiang).unwrap();
    std::fs::write(
        parent_dijiang.join("config.toml"),
        "[project]\ndeveloper = \"wrong-root\"\n",
    )
    .unwrap();
    std::fs::write(parent_dijiang.join("active_task.txt"), "stale-parent-task").unwrap();
    std::fs::write(worktree_dir.join("change.txt"), "changed").unwrap();
    let out = dijang(
        &[
            "finish-work",
            "--summary",
            "external worktree finish",
            "--verification",
            "manual check",
            "--docs-sync",
            "none: standalone external worktree change",
            "--version-impact",
            "none",
            "--commit",
            "--commit-message",
            "test(cli): 测试外部 worktree finish 功能",
        ],
        &worktree_dir,
    )
    .unwrap();

    assert!(!out.contains("已完成工作（无 active task"), "output: {out}");
    assert!(
        !out.contains("Task archive：skipped: no active task"),
        "output: {out}"
    );
    assert!(
        out.contains("Task archive：archived task `"),
        "output: {out}"
    );
    assert!(!out.contains("Commit：none"), "output: {out}");

    let task_json = project_dir.join(".dijiang/tasks/finish-work-external/task.json");
    let task = std::fs::read_to_string(&task_json).expect("archived task json");

    let active_task_path = project_dir.join(".dijiang/active_task.txt");
    assert!(
        !active_task_path.exists(),
        "active task pointer should be cleared"
    );

    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&worktree_dir)
        .output()
        .expect("git status");
    assert_eq!(String::from_utf8_lossy(&status.stdout).trim(), "");

    let log = Command::new("git")
        .args(["log", "--oneline", "-1"])
        .current_dir(&worktree_dir)
        .output()
        .expect("git log");
    assert!(
        String::from_utf8_lossy(&log.stdout).contains("test(cli): 测试外部 worktree finish 功能"),
        "log: {}",
        String::from_utf8_lossy(&log.stdout)
    );
}

#[test]
fn test_e2e_finish_work_explains_stale_active_task() {
    let (_tmp, project_dir) = init_project();
    let session_dir = project_dir
        .join(".dijiang")
        .join(".runtime")
        .join("sessions");
    std::fs::create_dir_all(&session_dir).unwrap();
    std::fs::write(
        session_dir.join("stale-window.json"),
        r#"{"current_task":"missing-task","session_key":"stale-window","source":"env"}"#,
    )
    .unwrap();

    let err = dijang_with_env(
        &[
            "finish-work",
            "--verification",
            "manual check",
            "--docs-sync",
            "none: stale task",
            "--version-impact",
            "none",
        ],
        &project_dir,
        &[("DIJIANG_CONTEXT_ID", "stale-window")],
    )
    .unwrap_err();

    assert!(
        err.contains("active task 指向 `missing-task`"),
        "error: {err}"
    );
    assert!(err.contains("task state 已陈旧"), "error: {err}");
    assert!(err.contains("dijiang task current"), "error: {err}");
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
            "--docs-sync",
            "none: dirty guard test",
        ],
        &project_dir,
    )
    .unwrap_err();
    assert!(err.contains("git worktree 存在未提交修改"), "error: {err}");
    assert!(err.contains("dirty.txt"), "error: {err}");

    let current = dijang(&["task", "current"], &project_dir).unwrap();
    assert!(current.contains("dirty-task"), "current output: {current}");
}
#[test]
fn test_e2e_skill_body_returns_json_payload() {
    let (_tmp, project_dir) = init_project();
    let out = dijang(&["skill-body", "dj-tdd", "--json"], &project_dir).unwrap();
    let payload: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(payload["name"], "dj-tdd");
    assert!(payload["summary"].as_str().unwrap().contains("测试驱动"));
    assert!(payload["body"].as_str().unwrap().contains("TDD"));
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
    assert!(finish_out.contains("已完成任务 'clean-task'"));
    assert!(finish_out.contains("Session 已关闭："));

    let session_journal = std::fs::read_to_string(
        project_dir
            .join(".dijiang")
            .join("workspace")
            .join("e2e")
            .join("sessions")
            .join("finish-window.jsonl"),
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
            .join("finish-window.json"),
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
fn test_e2e_finish_work_commit_requires_docs_sync() {
    let (_tmp, project_dir) = init_project();

    dijang(
        &["start", "commit-docs-gate", "Commit Docs Gate"],
        &project_dir,
    )
    .unwrap();
    std::fs::write(project_dir.join("change.txt"), "changed").unwrap();

    let err = dijang(
        &[
            "finish-work",
            "--summary",
            "commit docs gate",
            "--verification",
            "manual check",
            "--commit",
        ],
        &project_dir,
    )
    .unwrap_err();
    assert!(err.contains("requires --docs-sync"), "error: {err}");

    let current = dijang(&["task", "current"], &project_dir).unwrap();
    assert!(
        current.contains("commit-docs-gate"),
        "current output: {current}"
    );
}
#[test]
fn test_e2e_finish_work_blocks_integrate_without_approval() {
    let (_tmp, project_dir) = init_project();
    dijang(
        &["start", "integrate-blocked", "Integrate Blocked"],
        &project_dir,
    )
    .unwrap();
    std::fs::write(project_dir.join("change.txt"), "changed").unwrap();
    let err = dijang(
        &[
            "finish-work",
            "--summary",
            "integrate blocked",
            "--verification",
            "manual check",
            "--docs-sync",
            "none: integrate gate test",
            "--version-impact",
            "none",
            "--commit",
            "--commit-message",
            "test(cli): 测试 integrate 被阻断",
            "--integrate",
        ],
        &project_dir,
    )
    .unwrap_err();
    assert!(
        err.contains("finish-work integration blocked"),
        "error: {err}"
    );
    assert!(err.contains("requires explicit approval"), "error: {err}");
}
#[test]
fn test_e2e_finish_work_blocks_push_without_approval() {
    let (_tmp, project_dir) = init_project();
    dijang(&["start", "push-blocked", "Push Blocked"], &project_dir).unwrap();
    std::fs::write(project_dir.join("change.txt"), "changed").unwrap();
    let err = dijang(
        &[
            "finish-work",
            "--summary",
            "push blocked",
            "--verification",
            "manual check",
            "--docs-sync",
            "none: push gate test",
            "--version-impact",
            "none",
            "--commit",
            "--commit-message",
            "test(cli): 测试 push 被阻断",
            "--push",
        ],
        &project_dir,
    )
    .unwrap_err();
    assert!(err.contains("finish-work push blocked"), "error: {err}");
    assert!(err.contains("requires explicit approval"), "error: {err}");
}
#[test]
fn test_e2e_finish_work_blocks_cleanup_without_approval() {
    let (tmp, project_dir) = init_project();
    // git init creates 'master' but git_main_worktree expects 'main'
    Command::new("git")
        .args(["branch", "-m", "master", "main"])
        .current_dir(&project_dir)
        .output()
        .expect("git branch -m master main");
    std::fs::write(project_dir.join("base.txt"), "base").unwrap();
    Command::new("git")
        .args(["add", "base.txt"])
        .current_dir(&project_dir)
        .output()
        .expect("git add base");
    Command::new("git")
        .args(["commit", "-m", "test: cleanup base"])
        .current_dir(&project_dir)
        .output()
        .expect("git commit base");

    let worktree_dir = tmp.path().join("cleanup-worktree");
    Command::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            "cleanup-blocked",
            worktree_dir.to_str().unwrap(),
        ])
        .current_dir(&project_dir)
        .output()
        .expect("git worktree add");

    std::fs::write(worktree_dir.join("change.txt"), "changed").unwrap();
    let err = dijang(
        &[
            "finish-work",
            "--summary",
            "cleanup blocked",
            "--verification",
            "manual check",
            "--docs-sync",
            "none: cleanup gate test",
            "--version-impact",
            "none",
            "--commit",
            "--commit-message",
            "test(cli): 测试 cleanup 被阻断",
            "--integrate",
            "--approve-integrate",
        ],
        &worktree_dir,
    )
    .unwrap_err();
    assert!(err.contains("finish-work cleanup blocked"), "error: {err}");
    assert!(err.contains("requires explicit approval"), "error: {err}");
}

#[test]
fn test_e2e_finish_work_commit_archives_and_commits_diff() {
    let (_tmp, project_dir) = init_project();

    dijang(&["start", "commit-finish", "Commit Finish"], &project_dir).unwrap();
    std::fs::write(
        project_dir.join("Cargo.toml"),
        "[workspace.package]\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(project_dir.join("change.txt"), "changed").unwrap();

    let finish_out = dijang(
        &[
            "finish-work",
            "--summary",
            "commit finish",
            "--verification",
            "manual check",
            "--docs-sync",
            "none: no docs affected",
            "--version-impact",
            "patch",
            "--commit",
            "--commit-message",
            "test(cli): 测试 finish work 提交",
        ],
        &project_dir,
    )
    .unwrap();

    assert!(finish_out.contains("已完成任务 'commit-finish'"));
    assert!(finish_out.contains("版本更新：patch"), "finish_out: {finish_out}");
    assert!(finish_out.contains("版本更新：0.1.0 -> 0.1.1"));
    assert!(finish_out.contains("Commit："));
    assert!(finish_out.contains("Task archive：archived task `commit-finish`"));
    assert!(finish_out.contains("当前 session 的 active task 已清理"));
    assert!(finish_out.contains("Session 已关闭："));

    let cargo_toml = std::fs::read_to_string(project_dir.join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("version = \"0.1.1\""));

    let current = dijang(&["task", "current"], &project_dir).unwrap();
    assert!(current.contains("(none)"), "current output: {current}");

    let session_runtime = std::fs::read_to_string(
        project_dir
            .join(".dijiang")
            .join(".runtime")
            .join("sessions")
            .join("global_global.json"),
    )
    .unwrap();
    assert!(
        session_runtime.contains("\"closed_task\": \"commit-finish\""),
        "session runtime: {session_runtime}"
    );
    assert!(
        session_runtime.contains("\"current_task\": null"),
        "session runtime: {session_runtime}"
    );

    let closures = std::fs::read_to_string(
        project_dir
            .join(".dijiang")
            .join("memory")
            .join("sessions.jsonl"),
    )
    .unwrap();
    assert!(closures.contains("commit-finish"), "closures: {closures}");
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&project_dir)
        .output()
        .expect("git status");
    assert_eq!(String::from_utf8_lossy(&status.stdout).trim(), "");

    let log = Command::new("git")
        .args(["log", "--oneline", "-1"])
        .current_dir(&project_dir)
        .output()
        .expect("git log");
    assert!(
        String::from_utf8_lossy(&log.stdout).contains("test(cli): 测试 finish work 提交"),
        "log: {}",
        String::from_utf8_lossy(&log.stdout)
    );
}
#[test]
fn test_e2e_workflow_state_reports_stale_active_task_without_failing() {
    let (_tmp, project_dir) = init_project();
    let session_dir = project_dir
        .join(".dijiang")
        .join(".runtime")
        .join("sessions");
    std::fs::create_dir_all(&session_dir).unwrap();
    std::fs::write(
        session_dir.join("stale-window.json"),
        r#"{"current_task":"missing-task","session_key":"stale-window","source":"dijiang"}"#,
    )
    .unwrap();

    let out = dijang_with_env(
        &["workflow-state"],
        &project_dir,
        &[("DIJIANG_CONTEXT_ID", "stale-window")],
    )
    .unwrap();

    assert!(
        out.contains("会话：stale-window（dijiang）"),
        "output: {out}"
    );
    assert!(out.contains("活跃任务：none"), "output: {out}");
    assert!(out.contains("missing-task"), "output: {out}");
    assert!(out.contains("task state 已陈旧"), "output: {out}");
    assert!(out.contains("dj-hunt"), "output: {out}");
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
    assert!(first_a.contains("会话：window-a（dijiang）"));
    assert!(first_a.contains("注入：#1"));
    assert!(first_a.contains("活跃任务是否变化：true"));
    assert!(first_a.contains("活跃任务：window-a-task"));
    assert!(first_a.contains("会话日志：.dijiang/workspace/e2e/sessions/window-a.jsonl"));
    assert!(first_a.contains("最近记忆：当前窗口已加载 1 条最近会话事件。"));
    assert!(first_a.contains("注入 #1：活跃任务=window-a-task，上一个任务=none，变化=true"));
    assert!(first_a.contains("Loop：goal=Window A Task"));
    assert!(
        first_a.contains("progress=executing (实现与验证正在推进)")
            || first_a.contains("progress=aligning (需求与验收标准仍需对齐)")
    );
    assert!(first_a.contains("next_skill=dj-implement") || first_a.contains("next_skill=dj-grill"));

    let second_a = dijang_with_env(
        &["workflow-state"],
        &project_dir,
        &[("DIJIANG_CONTEXT_ID", "window-a")],
    )
    .unwrap();
    assert!(second_a.contains("注入：#2"));
    assert!(second_a.contains("活跃任务是否变化：false"));
    assert!(second_a.contains("最近记忆：当前窗口已加载 2 条最近会话事件。"));
    assert!(second_a.contains("注入 #1：活跃任务=window-a-task，上一个任务=none，变化=true"));
    assert!(
        second_a.contains("注入 #2：活跃任务=window-a-task，上一个任务=window-a-task，变化=false")
    );
    assert!(second_a.contains("Loop：goal=Window A Task"));
    assert!(
        second_a.contains("progress=executing (实现与验证正在推进)")
            || second_a.contains("progress=aligning (需求与验收标准仍需对齐)")
    );
    assert!(
        second_a.contains("next_skill=dj-implement") || second_a.contains("next_skill=dj-grill")
    );
    assert!(second_a.contains("其他活跃窗口：none"));

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
    assert!(first_b.contains("会话：window-b（dijiang）"));
    assert!(first_b.contains("注入：#1"));
    assert!(first_b.contains("活跃任务：window-b-task"));
    assert!(first_b.contains("最近记忆：当前窗口已加载 1 条最近会话事件。"));
    assert!(first_b.contains("其他活跃窗口：1"));
    assert!(first_b.contains("window-a (dijiang) 任务=window-a-task 状态=active 注入=2"));
    assert!(!first_b.contains("注入 #1：活跃任务=window-a-task"));

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
    assert!(changed_a.contains("注入：#3"));
    assert!(changed_a.contains("活跃任务是否变化：true"));
    assert!(changed_a.contains("上一个活跃任务：window-a-task"));
    assert!(changed_a.contains("活跃任务：window-a-next"));
    assert!(changed_a.contains("最近记忆：当前窗口已加载 3 条最近会话事件。"));
    assert!(
        changed_a.contains("注入 #3：活跃任务=window-a-next，上一个任务=window-a-task，变化=true")
    );
    assert!(changed_a.contains("Loop：goal=Window A Next"));
    assert!(
        changed_a.contains("progress=executing (实现与验证正在推进)")
            || changed_a.contains("progress=aligning (需求与验收标准仍需对齐)")
    );
    assert!(changed_a.contains("其他活跃窗口：1"));
    assert!(changed_a.contains("window-b (dijiang) 任务=window-b-task 状态=active 注入=1"));
    let session_a = std::fs::read_to_string(
        project_dir
            .join(".dijiang")
            .join(".runtime")
            .join("sessions")
            .join("window-a.json"),
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
    assert!(log.contains("window-a"));
    assert!(log.contains("window-b"));
    assert!(log.contains("\"loop_state\""));

    let journal_a = std::fs::read_to_string(
        project_dir
            .join(".dijiang")
            .join("workspace")
            .join("e2e")
            .join("sessions")
            .join("window-a.jsonl"),
    )
    .unwrap();
    assert_eq!(journal_a.lines().count(), 3);
    assert!(journal_a.contains("window-a-task"));
    assert!(journal_a.contains("window-a-next"));
    assert!(journal_a.contains("\"active_task_changed\":true"));
    assert!(journal_a.contains("\"loop_state\""));

    let journal_b = std::fs::read_to_string(
        project_dir
            .join(".dijiang")
            .join("workspace")
            .join("e2e")
            .join("sessions")
            .join("window-b.jsonl"),
    )
    .unwrap();
    assert_eq!(journal_b.lines().count(), 1);
    assert!(journal_b.contains("window-b-task"));
}

#[test]
fn test_e2e_workflow_state_json_exposes_loop_runtime() {
    let (_tmp, project_dir) = init_project();

    dijang_with_env(
        &["start", "loop-task", "Loop Task"],
        &project_dir,
        &[
            ("DIJIANG_CONTEXT_ID", "loop-json"),
            ("DIJIANG_SESSION_ID", "loop-json"),
        ],
    )
    .unwrap();

    let payload = dijang_with_env(
        &[
            "workflow-state",
            "--json",
            "--hook-event",
            "UserPromptSubmit",
        ],
        &project_dir,
        &[
            ("DIJIANG_CONTEXT_ID", "loop-json"),
            ("DIJIANG_SESSION_ID", "loop-json"),
        ],
    )
    .unwrap();

    assert!(
        payload.contains("\"hookEventName\":\"UserPromptSubmit\""),
        "payload: {payload}"
    );
    assert!(
        payload.contains(
            "Loop：goal=Loop Task；mode=align；progress=aligning (需求与验收标准仍需对齐)"
        ),
        "payload: {payload}"
    );
    assert!(
        payload.contains("next_skill=dj-grill"),
        "payload: {payload}"
    );
    assert!(
        payload.contains(
            "retry=attempt=1; max=unbounded; remaining=unknown; can_retry=true; last_failure=none"
        ),
        "payload: {payload}"
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

// ─── Channel ──────────────────────────────────────────────────

#[test]
fn test_e2e_channel_lifecycle() {
    let (_tmp, project_dir) = init_project();

    // List (empty)
    let out = dijang(&["channel", "list"], &project_dir);
    assert!(out.is_ok(), "channel list should succeed");

    // Spawn
    let out = dijang(&["channel", "spawn", "checker", "--task", "."], &project_dir);
    assert!(out.is_ok(), "channel spawn should succeed: {:?}", out.err());
    let stdout = out.unwrap();
    assert!(
        stdout.contains("Agent 'checker' spawned"),
        "should confirm spawn"
    );
    assert!(stdout.contains("Channel ID:"), "should output channel ID");

    // List (one active)
    let out = dijang(&["channel", "list"], &project_dir);
    assert!(out.is_ok());
    let stdout = out.unwrap();
    assert!(stdout.contains("checker"), "should list check agent");
    assert!(stdout.contains("active"), "should show active status");

    // Extract channel ID
    let channel_id = stdout
        .lines()
        .find(|l| l.contains("checker") && l.contains("active"))
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

    let out = dijang(&["mem", "learn", "--lesson", "test-learning"], &project_dir);
    assert!(out.is_ok(), "mem learn should succeed: {:?}", out.err());

    let out = dijang(
        &["mem", "findings", "--finding", "Found something important"],
        &project_dir,
    );
    assert!(out.is_ok(), "mem findings should succeed: {:?}", out.err());

    let memory_dir = project_dir.join(".dijiang").join("memory");
    let learnings = std::fs::read_to_string(memory_dir.join("learnings.jsonl")).unwrap();
    assert!(
        learnings.contains("test-learning"),
        "learnings: {learnings}"
    );
    let findings = std::fs::read_to_string(memory_dir.join("findings.jsonl")).unwrap();
    assert!(
        findings.contains("Found something important"),
        "findings: {findings}"
    );

    let stats = dijang(&["mem", "stats"], &project_dir).unwrap();
    assert!(stats.contains("Findings: 1"), "stats: {stats}");
    assert!(stats.contains("Learnings: 1"), "stats: {stats}");
    assert!(stats.contains("Corrections: 0"), "stats: {stats}");

    let out = dijang(&["mem", "list"], &project_dir);
    assert!(out.is_ok(), "mem list should succeed: {:?}", out.err());
    let stdout = out.unwrap();
    assert!(!stdout.is_empty(), "should have some output");
}

#[test]
fn test_e2e_mem_correction_writes_structured_record() {
    let (_tmp, project_dir) = init_project();

    let out = dijang(
        &[
            "mem",
            "correction",
            "--correction",
            "missed full mem scope",
            "--lesson",
            "include all existing mem commands",
            "--scope",
            "workflow",
            "--actionability",
            "future implementation must preserve all mem subcommands",
        ],
        &project_dir,
    );
    assert!(
        out.is_ok(),
        "mem correction should succeed: {:?}",
        out.err()
    );

    let corrections = std::fs::read_to_string(
        project_dir
            .join(".dijiang")
            .join("memory")
            .join("corrections.jsonl"),
    )
    .unwrap();
    assert!(
        corrections.contains("missed full mem scope"),
        "corrections: {corrections}"
    );
    assert!(
        corrections.contains("user-confirmed"),
        "corrections: {corrections}"
    );

    let stats = dijang(&["mem", "stats"], &project_dir).unwrap();
    assert!(stats.contains("Corrections: 1"), "stats: {stats}");
}

#[test]
fn test_e2e_mem_tactics() {
    let (_tmp, project_dir) = init_project();
    let out = dijang(&["mem", "tactics"], &project_dir);
    assert!(out.is_ok(), "mem tactics should succeed: {:?}", out.err());
    let stdout = out.unwrap();
    // Should show default tactics or an empty message
    assert!(
        stdout.contains("cargo-test")
            || stdout.contains("typecheck")
            || stdout.contains("No tactics"),
        "should show tactics or empty message"
    );
}
