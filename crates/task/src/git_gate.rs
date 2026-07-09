use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::types::TaskRecord;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GitGateState {
    Ready,
    Provisioned,
    Blocked,
}

impl GitGateState {
    pub fn as_str(&self) -> &'static str {
        match self {
            GitGateState::Ready => "ready",
            GitGateState::Provisioned => "provisioned",
            GitGateState::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitRuntimeLocation {
    Unknown,
    MainWorktree,
    TaskWorktree,
    OtherWorktree,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitGateInput {
    pub current_location: PathBuf,
    pub current_worktree_root: Option<PathBuf>,
    pub main_worktree_root: Option<PathBuf>,
    pub route_requires_worktree: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeReadiness {
    pub task_name: String,
    pub branch: Option<String>,
    pub base_branch: Option<String>,
    pub worktree_path: Option<String>,
    pub state: GitGateState,
    pub current_location: String,
    pub current_worktree_root: Option<String>,
    pub expected_worktree_root: Option<String>,
    pub location_kind: String,
    pub message: String,
    pub fix_applied: bool,
    pub needs_provision: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitGateSummary {
    pub state: GitGateState,
    pub branch: Option<String>,
    pub base_branch: Option<String>,
    pub worktree_path: Option<String>,
    pub note: String,
}

pub fn summarize_git_gate(task: &TaskRecord, current_location: &Path) -> GitGateSummary {
    let readiness = evaluate_worktree_readiness(
        task,
        &GitGateInput {
            current_location: current_location.to_path_buf(),
            current_worktree_root: None,
            main_worktree_root: None,
            route_requires_worktree: false,
        },
    );
    GitGateSummary {
        state: readiness.state,
        branch: readiness.branch,
        base_branch: readiness.base_branch,
        worktree_path: readiness.worktree_path,
        note: readiness.message,
    }
}

pub fn evaluate_worktree_readiness(task: &TaskRecord, input: &GitGateInput) -> WorktreeReadiness {
    let worktree_path = task.worktree_path.clone();
    let branch = task.branch.clone();
    let base_branch = task.base_branch.clone();
    let current_location = input.current_location.display().to_string();
    let current_worktree_root = input
        .current_worktree_root
        .as_ref()
        .map(|path| path.display().to_string());
    let expected_worktree_root = worktree_path.clone();
    let location_kind = classify_location(input, worktree_path.as_deref());

    if !input.route_requires_worktree {
        return WorktreeReadiness {
            task_name: task.name.clone(),
            branch,
            base_branch,
            worktree_path,
            state: GitGateState::Ready,
            current_location,
            current_worktree_root,
            expected_worktree_root,
            location_kind: location_kind.as_str().to_string(),
            message: "current route does not require a task worktree yet".to_string(),
            fix_applied: false,
            needs_provision: false,
        };
    }

    if task.branch.is_none() && task.worktree_path.is_none() {
        return WorktreeReadiness {
            task_name: task.name.clone(),
            branch,
            base_branch,
            worktree_path,
            state: GitGateState::Blocked,
            current_location,
            current_worktree_root,
            expected_worktree_root,
            location_kind: location_kind.as_str().to_string(),
            message: "task has no provisioned worktree metadata yet".to_string(),
            fix_applied: false,
            needs_provision: true,
        };
    }

    if task.branch.is_some() ^ task.worktree_path.is_some() {
        return WorktreeReadiness {
            task_name: task.name.clone(),
            branch,
            base_branch,
            worktree_path,
            state: GitGateState::Blocked,
            current_location,
            current_worktree_root,
            expected_worktree_root,
            location_kind: location_kind.as_str().to_string(),
            message:
                "task worktree metadata is incomplete; branch and worktreePath must both exist"
                    .to_string(),
            fix_applied: false,
            needs_provision: false,
        };
    }

    let Some(expected_root) = task.worktree_path.as_deref() else {
        unreachable!();
    };
    let expected_path = Path::new(expected_root);

    if !expected_path.exists() {
        return WorktreeReadiness {
            task_name: task.name.clone(),
            branch,
            base_branch,
            worktree_path,
            state: GitGateState::Blocked,
            current_location,
            current_worktree_root,
            expected_worktree_root,
            location_kind: location_kind.as_str().to_string(),
            message: format!(
                "task worktree path does not exist on disk: {}",
                expected_path.display()
            ),
            fix_applied: false,
            needs_provision: false,
        };
    }

    match location_kind {
        GitRuntimeLocation::TaskWorktree | GitRuntimeLocation::Unknown => WorktreeReadiness {
            task_name: task.name.clone(),
            branch,
            base_branch,
            worktree_path,
            state: GitGateState::Ready,
            current_location,
            current_worktree_root,
            expected_worktree_root,
            location_kind: location_kind.as_str().to_string(),
            message: default_message(
                GitGateState::Ready,
                task.branch.as_deref(),
                task.base_branch.as_deref(),
                task.worktree_path.as_deref(),
            ),
            fix_applied: false,
            needs_provision: false,
        },
        GitRuntimeLocation::MainWorktree => WorktreeReadiness {
            task_name: task.name.clone(),
            branch,
            base_branch,
            worktree_path,
            state: GitGateState::Blocked,
            current_location,
            current_worktree_root,
            expected_worktree_root,
            location_kind: location_kind.as_str().to_string(),
            message: format!(
                "task worktree exists but the current runtime is still in the main checkout; switch to {} before implementation work",
                expected_path.display()
            ),
            fix_applied: false,
            needs_provision: false,
        },
        GitRuntimeLocation::OtherWorktree => WorktreeReadiness {
            task_name: task.name.clone(),
            branch,
            base_branch,
            worktree_path,
            state: GitGateState::Blocked,
            current_location,
            current_worktree_root,
            expected_worktree_root,
            location_kind: location_kind.as_str().to_string(),
            message: format!(
                "current runtime is attached to a different worktree; expected {}",
                expected_path.display()
            ),
            fix_applied: false,
            needs_provision: false,
        },
    }
}

pub fn worktree_readiness(
    task: &TaskRecord,
    state: GitGateState,
    current_location: &Path,
    fix_applied: bool,
    message_override: Option<String>,
) -> WorktreeReadiness {
    let worktree_path = task.worktree_path.clone();
    let branch = task.branch.clone();
    let base_branch = task.base_branch.clone();
    let message = message_override.unwrap_or_else(|| {
        default_message(
            state,
            branch.as_deref(),
            base_branch.as_deref(),
            worktree_path.as_deref(),
        )
    });

    WorktreeReadiness {
        task_name: task.name.clone(),
        branch,
        base_branch,
        worktree_path: worktree_path.clone(),
        state,
        current_location: current_location.display().to_string(),
        current_worktree_root: None,
        expected_worktree_root: worktree_path,
        location_kind: GitRuntimeLocation::Unknown.as_str().to_string(),
        message,
        fix_applied,
        needs_provision: false,
    }
}

fn classify_location(
    input: &GitGateInput,
    expected_worktree_path: Option<&str>,
) -> GitRuntimeLocation {
    let Some(current_root) = input.current_worktree_root.as_ref() else {
        return GitRuntimeLocation::Unknown;
    };

    if let Some(main_root) = input.main_worktree_root.as_ref()
        && current_root == main_root
    {
        return GitRuntimeLocation::MainWorktree;
    }

    if let Some(expected) = expected_worktree_path
        && current_root == Path::new(expected)
    {
        return GitRuntimeLocation::TaskWorktree;
    }

    GitRuntimeLocation::OtherWorktree
}

fn default_message(
    state: GitGateState,
    branch: Option<&str>,
    base_branch: Option<&str>,
    worktree_path: Option<&str>,
) -> String {
    match state {
        GitGateState::Ready => format!(
            "task worktree is ready; branch={}; baseBranch={}; worktreePath={}",
            branch.unwrap_or("none"),
            base_branch.unwrap_or("none"),
            worktree_path.unwrap_or("none")
        ),
        GitGateState::Provisioned => format!(
            "task worktree was provisioned; branch={}; baseBranch={}; worktreePath={}",
            branch.unwrap_or("none"),
            base_branch.unwrap_or("none"),
            worktree_path.unwrap_or("none")
        ),
        GitGateState::Blocked => format!(
            "task worktree is blocked; branch={}; baseBranch={}; worktreePath={}",
            branch.unwrap_or("none"),
            base_branch.unwrap_or("none"),
            worktree_path.unwrap_or("none")
        ),
    }
}

impl GitRuntimeLocation {
    fn as_str(&self) -> &'static str {
        match self {
            GitRuntimeLocation::Unknown => "unknown",
            GitRuntimeLocation::MainWorktree => "main_worktree",
            GitRuntimeLocation::TaskWorktree => "task_worktree",
            GitRuntimeLocation::OtherWorktree => "other_worktree",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TaskStatus;

    fn task() -> TaskRecord {
        TaskRecord {
            id: "task-1".to_string(),
            name: "task-1".to_string(),
            title: "Task 1".to_string(),
            description: String::new(),
            status: TaskStatus::Planning,
            dev_type: None,
            scope: None,
            package: None,
            priority: "medium".to_string(),
            creator: String::new(),
            assignee: String::new(),
            created_at: "2026-01-01".to_string(),
            completed_at: None,
            branch: Some("feat/task-1".to_string()),
            base_branch: Some("main".to_string()),
            worktree_path: Some("/tmp/task-1".to_string()),
            commit: None,
            pr_url: None,
            subtasks: vec![],
            children: vec![],
            parent: None,
            related_files: vec![],
            notes: String::new(),
            meta: serde_json::json!({}),
            started_at: None,
            archived_at: None,
            acceptance_criteria: None,
            key_deliverables: None,
            source: None,
            session_id: None,
            session_summary: None,
            estimated_effort: None,
            actual_effort: None,
            review_status: None,
            review_comments: None,
            tags: None,
        }
    }

    #[test]
    fn summarize_ready_git_gate_uses_task_record() {
        let summary = summarize_git_gate(&task(), Path::new("/repo"));
        assert_eq!(summary.state, GitGateState::Ready);
        assert_eq!(summary.branch.as_deref(), Some("feat/task-1"));
        assert_eq!(summary.base_branch.as_deref(), Some("main"));
        assert_eq!(summary.worktree_path.as_deref(), Some("/tmp/task-1"));
    }

    #[test]
    fn evaluator_marks_missing_metadata_as_needing_provision() {
        let mut record = task();
        record.branch = None;
        record.worktree_path = None;
        let readiness = evaluate_worktree_readiness(
            &record,
            &GitGateInput {
                current_location: PathBuf::from("/repo"),
                current_worktree_root: Some(PathBuf::from("/repo")),
                main_worktree_root: Some(PathBuf::from("/repo")),
                route_requires_worktree: true,
            },
        );
        assert_eq!(readiness.state, GitGateState::Blocked);
        assert!(readiness.needs_provision);
        assert_eq!(readiness.location_kind, "main_worktree");
    }

    #[test]
    fn evaluator_blocks_when_runtime_stays_on_main_checkout() {
        let temp = tempfile::tempdir().unwrap();
        let task_dir = temp.path().join("task-1");
        std::fs::create_dir_all(&task_dir).unwrap();
        let mut record = task();
        record.worktree_path = Some(task_dir.display().to_string());
        let readiness = evaluate_worktree_readiness(
            &record,
            &GitGateInput {
                current_location: temp.path().to_path_buf(),
                current_worktree_root: Some(temp.path().to_path_buf()),
                main_worktree_root: Some(temp.path().to_path_buf()),
                route_requires_worktree: true,
            },
        );
        assert_eq!(readiness.state, GitGateState::Blocked);
        assert!(!readiness.needs_provision);
        assert_eq!(readiness.location_kind, "main_worktree");
        assert!(readiness.message.contains("main checkout"));
    }

    #[test]
    fn evaluator_accepts_matching_task_worktree() {
        let temp = tempfile::tempdir().unwrap();
        let task_dir = temp.path().join("task-1");
        std::fs::create_dir_all(&task_dir).unwrap();
        let mut record = task();
        record.worktree_path = Some(task_dir.display().to_string());
        let readiness = evaluate_worktree_readiness(
            &record,
            &GitGateInput {
                current_location: task_dir.clone(),
                current_worktree_root: Some(task_dir.clone()),
                main_worktree_root: Some(temp.path().to_path_buf()),
                route_requires_worktree: true,
            },
        );
        assert_eq!(readiness.state, GitGateState::Ready);
        assert!(!readiness.needs_provision);
        assert_eq!(readiness.location_kind, "task_worktree");
    }
}
