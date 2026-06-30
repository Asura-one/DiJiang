use std::path::Path;

use serde::Serialize;

use crate::store::{self, SessionIdentity, TaskError};
use crate::types::{TaskRecord, TaskStatus};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowState {
    pub session: Option<WorkflowSession>,
    pub active_task: Option<WorkflowTask>,
    pub guidance: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowSession {
    pub key: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowTask {
    pub id: String,
    pub name: String,
    pub title: String,
    pub status: String,
    pub task_path: String,
}

impl WorkflowState {
    pub fn additional_context(&self) -> String {
        let session_line = self
            .session
            .as_ref()
            .map(|session| format!("Session: {} ({})", session.key, session.source))
            .unwrap_or_else(|| "Session: global fallback".to_string());

        let Some(task) = &self.active_task else {
            return format!(
                "<dijiang-workflow-state>\n{session_line}\nActive task: none\nNext: {}\n</dijiang-workflow-state>",
                self.guidance
            );
        };

        format!(
            "<dijiang-workflow-state>\n{session_line}\nActive task: {}\nTitle: {}\nStatus: {}\nTask path: {}\nGuidance: {}\nLoad context: read task.json plus prd.md/design.md/implement.md/check artifacts when present.\n</dijiang-workflow-state>",
            task.id, task.title, task.status, task.task_path, self.guidance
        )
    }
}

pub fn build(dijiang_dir: &Path) -> Result<WorkflowState, TaskError> {
    build_for_session(dijiang_dir, store::current_session_identity().as_ref())
}

pub fn build_for_session(
    dijiang_dir: &Path,
    identity: Option<&SessionIdentity>,
) -> Result<WorkflowState, TaskError> {
    let session = identity.map(|identity| WorkflowSession {
        key: identity.key().to_string(),
        source: identity.source().to_string(),
    });

    let Some(active_task_name) = store::read_active_task_for_session(dijiang_dir, identity)? else {
        return Ok(WorkflowState {
            session,
            active_task: None,
            guidance: "run `dijiang start <name>` or read `.dijiang/workflow.md` before coding."
                .to_string(),
        });
    };

    let tasks_dir = dijiang_dir.join("tasks");
    let task = store::load_task(&tasks_dir, &active_task_name)?;
    let guidance = status_guidance(&task.status).to_string();
    let active_task = Some(workflow_task(dijiang_dir, &task));

    Ok(WorkflowState {
        session,
        active_task,
        guidance,
    })
}

fn workflow_task(dijiang_dir: &Path, task: &TaskRecord) -> WorkflowTask {
    let task_path = dijiang_dir.join("tasks").join(&task.name);
    let task_path = task_path
        .strip_prefix(dijiang_dir.parent().unwrap_or(dijiang_dir))
        .unwrap_or(&task_path)
        .display()
        .to_string();

    WorkflowTask {
        id: task.id.clone(),
        name: task.name.clone(),
        title: task.title.clone(),
        status: task.status.as_str().to_string(),
        task_path,
    }
}

fn status_guidance(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Planning => {
            "当前任务处于 planning。按 DiJiang 流程先用 dj-grill 对齐需求，再产出必要设计。"
        }
        TaskStatus::InProgress => {
            "当前任务处于 in_progress。继续实现；完成后运行相关验证，再进入 dj-check。"
        }
        TaskStatus::Completed => {
            "当前任务已 completed。检查是否需要运行 /dijiang-finish-work 归档和记录 journal。"
        }
        TaskStatus::Archived => {
            "当前任务已 archived。若继续工作，请先运行 dijiang start <task> 激活新任务。"
        }
        TaskStatus::Paused => "当前任务已 paused。恢复前先读取任务上下文和最近 workspace journal。",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store;
    use crate::types::TaskRecord;

    fn task(name: &str, title: &str) -> TaskRecord {
        TaskRecord {
            id: name.to_string(),
            name: name.to_string(),
            title: title.to_string(),
            description: String::new(),
            status: TaskStatus::InProgress,
            dev_type: None,
            scope: None,
            package: None,
            priority: "medium".to_string(),
            creator: "test".to_string(),
            assignee: "test".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            completed_at: None,
            branch: None,
            base_branch: None,
            worktree_path: None,
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
    fn builds_session_scoped_context() {
        let dir = tempfile::tempdir().unwrap();
        let dijiang_dir = dir.path().join(".dijiang");
        let tasks_dir = dijiang_dir.join("tasks");
        std::fs::create_dir_all(&tasks_dir).unwrap();

        store::save_task(&tasks_dir, &task("task-a", "Task A")).unwrap();
        store::save_task(&tasks_dir, &task("task-b", "Task B")).unwrap();
        let window_a = store::SessionIdentity::new("dijiang", "window-a").unwrap();
        let window_b = store::SessionIdentity::new("dijiang", "window-b").unwrap();
        store::write_active_task_for_session(&dijiang_dir, "task-a", Some(&window_a)).unwrap();
        store::write_active_task_for_session(&dijiang_dir, "task-b", Some(&window_b)).unwrap();

        let context_a = build_for_session(&dijiang_dir, Some(&window_a))
            .unwrap()
            .additional_context();
        let context_b = build_for_session(&dijiang_dir, Some(&window_b))
            .unwrap()
            .additional_context();

        assert!(context_a.contains("Session: dijiang_window-a (dijiang)"));
        assert!(context_a.contains("Active task: task-a"));
        assert!(context_a.contains("Title: Task A"));
        assert!(context_b.contains("Session: dijiang_window-b (dijiang)"));
        assert!(context_b.contains("Active task: task-b"));
        assert!(context_b.contains("Title: Task B"));
    }
}
