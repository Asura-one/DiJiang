use std::fs;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::store::{self, SessionIdentity, TaskError};
use crate::types::{TaskRecord, TaskStatus};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowState {
    pub session: Option<WorkflowSession>,
    pub active_task: Option<WorkflowTask>,
    pub guidance: String,
    pub runtime: WorkflowRuntime,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowSession {
    pub key: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRuntime {
    pub injection_count: u64,
    pub active_task_changed: bool,
    pub previous_active_task: Option<String>,
    pub last_seen_at: String,
    pub log_path: String,
    pub journal_path: String,
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

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
struct RuntimeSessionRecord {
    session_key: String,
    source: String,
    current_task: Option<String>,
    last_active_task: Option<String>,
    injection_count: u64,
    last_seen_at: String,
}

impl WorkflowState {
    pub fn additional_context(&self) -> String {
        let session_line = self
            .session
            .as_ref()
            .map(|session| format!("Session: {} ({})", session.key, session.source))
            .unwrap_or_else(|| "Session: global fallback".to_string());
        let previous = self
            .runtime
            .previous_active_task
            .as_deref()
            .unwrap_or("none");
        let runtime_line = format!(
            "Injection: #{} at {}\nActive task changed: {}\nPrevious active task: {}\nRuntime log: {}\nSession journal: {}",
            self.runtime.injection_count,
            self.runtime.last_seen_at,
            self.runtime.active_task_changed,
            previous,
            self.runtime.log_path,
            self.runtime.journal_path
        );

        let Some(task) = &self.active_task else {
            return format!(
                "<dijiang-workflow-state>\n{session_line}\n{runtime_line}\nActive task: none\nNext: {}\n</dijiang-workflow-state>",
                self.guidance
            );
        };

        format!(
            "<dijiang-workflow-state>\n{session_line}\n{runtime_line}\nActive task: {}\nTitle: {}\nStatus: {}\nTask path: {}\nGuidance: {}\nLoad context: read task.json plus prd.md/design.md/implement.md/check artifacts when present.\n</dijiang-workflow-state>",
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
    let active_task_name = store::read_active_task_for_session(dijiang_dir, identity)?;
    let task = match active_task_name {
        Some(active_task_name) => Some(store::load_task(
            &dijiang_dir.join("tasks"),
            &active_task_name,
        )?),
        None => None,
    };
    let runtime = record_runtime_injection(dijiang_dir, identity, task.as_ref())?;

    let Some(task) = task else {
        return Ok(WorkflowState {
            session,
            active_task: None,
            guidance: "run `dijiang start <name>` or read `.dijiang/workflow.md` before coding."
                .to_string(),
            runtime,
        });
    };

    let guidance = status_guidance(&task.status).to_string();
    let active_task = Some(workflow_task(dijiang_dir, &task));
    Ok(WorkflowState {
        session,
        active_task,
        guidance,
        runtime,
    })
}

fn record_runtime_injection(
    dijiang_dir: &Path,
    identity: Option<&SessionIdentity>,
    active_task: Option<&TaskRecord>,
) -> Result<WorkflowRuntime, TaskError> {
    let runtime_dir = dijiang_dir.join(".runtime");
    let sessions_dir = runtime_dir.join("sessions");
    fs::create_dir_all(&sessions_dir)?;
    fs::write(runtime_dir.join(".dijiang_owned"), "")?;

    let fallback_identity;
    let identity = match identity {
        Some(identity) => identity,
        None => {
            fallback_identity = SessionIdentity::new("global", "global")
                .expect("literal global session key is valid");
            &fallback_identity
        }
    };

    let session_path = sessions_dir.join(format!("{}.json", identity.key()));
    let mut record = if session_path.exists() {
        let content = fs::read_to_string(&session_path)?;
        serde_json::from_str::<RuntimeSessionRecord>(&content).unwrap_or_default()
    } else {
        RuntimeSessionRecord::default()
    };
    let previous_active_task = record.last_active_task.clone();
    let current_active_task = active_task.map(|task| task.name.clone());
    let active_task_changed = previous_active_task != current_active_task;
    let last_seen_at = chrono::Utc::now().to_rfc3339();

    record.session_key = identity.key().to_string();
    record.source = identity.source().to_string();
    record.current_task = current_active_task.clone();
    record.last_active_task = current_active_task.clone();
    record.injection_count = record.injection_count.saturating_add(1);
    record.last_seen_at = last_seen_at.clone();

    fs::write(&session_path, serde_json::to_string_pretty(&record)?)?;

    let log_path = runtime_dir.join("workflow-state.log");
    let active_task_event = active_task.map(|task| {
        serde_json::json!({
            "id": task.id,
            "name": task.name,
            "title": task.title,
            "status": task.status.as_str(),
            "task_path": relative_path(dijiang_dir, &dijiang_dir.join("tasks").join(&task.name)),
        })
    });
    let event = serde_json::json!({
        "event": "workflow_state_injected",
        "session_key": identity.key(),
        "source": identity.source(),
        "injection_count": record.injection_count,
        "active_task": current_active_task,
        "active_task_detail": active_task_event,
        "previous_active_task": previous_active_task,
        "active_task_changed": active_task_changed,
        "at": last_seen_at,
    });
    let mut log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    writeln!(log, "{}", serde_json::to_string(&event)?)?;
    let journal_path = append_session_journal(dijiang_dir, identity, &event)?;

    Ok(WorkflowRuntime {
        injection_count: record.injection_count,
        active_task_changed,
        previous_active_task,
        last_seen_at,
        log_path: relative_path(dijiang_dir, &log_path),
        journal_path: relative_path(dijiang_dir, &journal_path),
    })
}

fn append_session_journal(
    dijiang_dir: &Path,
    identity: &SessionIdentity,
    event: &serde_json::Value,
) -> Result<std::path::PathBuf, TaskError> {
    let developer = read_developer(dijiang_dir);
    let sessions_dir = dijiang_dir
        .join("workspace")
        .join(developer)
        .join("sessions");
    fs::create_dir_all(&sessions_dir)?;
    let journal_path = sessions_dir.join(format!("{}.jsonl", identity.key()));
    let mut journal = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&journal_path)?;
    writeln!(journal, "{}", serde_json::to_string(event)?)?;
    Ok(journal_path)
}

fn read_developer(dijiang_dir: &Path) -> String {
    let config_path = dijiang_dir.join("config.toml");
    let Ok(config_str) = fs::read_to_string(config_path) else {
        return "developer".to_string();
    };
    config_str
        .lines()
        .find(|line| line.trim_start().starts_with("developer"))
        .and_then(|line| line.split('=').nth(1))
        .map(|value| value.trim().trim_matches('"').to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "developer".to_string())
}

fn relative_path(dijiang_dir: &Path, path: &Path) -> String {
    path.strip_prefix(dijiang_dir.parent().unwrap_or(dijiang_dir))
        .unwrap_or(path)
        .display()
        .to_string()
}

fn workflow_task(dijiang_dir: &Path, task: &TaskRecord) -> WorkflowTask {
    let task_path = dijiang_dir.join("tasks").join(&task.name);
    let task_path = relative_path(dijiang_dir, &task_path);

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
        std::fs::create_dir_all(&dijiang_dir).unwrap();
        std::fs::write(
            dijiang_dir.join("config.toml"),
            "[project]\ndeveloper = \"tester\"\n",
        )
        .unwrap();
        let tasks_dir = dijiang_dir.join("tasks");
        std::fs::create_dir_all(&tasks_dir).unwrap();

        store::save_task(&tasks_dir, &task("task-a", "Task A")).unwrap();
        store::save_task(&tasks_dir, &task("task-b", "Task B")).unwrap();
        let window_a = store::SessionIdentity::new("dijiang", "window-a").unwrap();
        let window_b = store::SessionIdentity::new("dijiang", "window-b").unwrap();
        store::write_active_task_for_session(&dijiang_dir, "task-a", Some(&window_a)).unwrap();
        store::write_active_task_for_session(&dijiang_dir, "task-b", Some(&window_b)).unwrap();

        let state_a = build_for_session(&dijiang_dir, Some(&window_a)).unwrap();
        let context_a = state_a.additional_context();
        let state_b = build_for_session(&dijiang_dir, Some(&window_b)).unwrap();
        let context_b = state_b.additional_context();

        assert!(context_a.contains("Session: dijiang_window-a (dijiang)"));
        assert!(context_a.contains("Injection: #1"));
        assert!(context_a.contains("Active task changed: true"));
        assert!(context_a.contains("Active task: task-a"));
        assert!(context_a.contains("Title: Task A"));
        assert!(context_a.contains(
            "Session journal: .dijiang/workspace/tester/sessions/dijiang_window-a.jsonl"
        ));
        assert!(context_b.contains("Session: dijiang_window-b (dijiang)"));
        assert!(context_b.contains("Active task: task-b"));
        assert!(context_b.contains("Title: Task B"));

        let next_a = build_for_session(&dijiang_dir, Some(&window_a))
            .unwrap()
            .additional_context();
        assert!(next_a.contains("Injection: #2"));
        assert!(next_a.contains("Active task changed: false"));

        let log = std::fs::read_to_string(dijiang_dir.join(".runtime/workflow-state.log")).unwrap();
        assert!(log.contains("workflow_state_injected"));
        assert!(log.contains("dijiang_window-a"));
        assert!(log.contains("dijiang_window-b"));

        let journal_a = std::fs::read_to_string(
            dijiang_dir.join("workspace/tester/sessions/dijiang_window-a.jsonl"),
        )
        .unwrap();
        assert_eq!(journal_a.lines().count(), 2);
        assert!(journal_a.contains("workflow_state_injected"));
        assert!(journal_a.contains("\"title\":\"Task A\""));
        assert!(journal_a.contains("\"injection_count\":2"));
    }
}
