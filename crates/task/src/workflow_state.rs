use std::fs;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::route_gate::{summarize_route_gate};
use crate::store::{self, SessionIdentity, TaskError};
use crate::types::{TaskRecord, TaskStatus};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowState {
    pub session: Option<WorkflowSession>,
    pub active_task: Option<WorkflowTask>,
    pub guidance: String,
    pub runtime: WorkflowRuntime,
    pub memory: WorkflowMemory,
    pub peers: Vec<WorkflowPeerSession>,
    pub route_gate: Option<WorkflowRouteGate>,
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowMemory {
    pub summary: String,
    pub events: Vec<WorkflowMemoryEvent>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowMemoryEvent {
    pub kind: String,
    pub detail: String,
    pub at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowPeerSession {
    pub key: String,
    pub source: String,
    pub current_task: Option<String>,
    pub injection_count: u64,
    pub last_seen_at: String,
    pub closed_task: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRouteGate {
    pub capsule: String,
    pub allowed_skills: Vec<String>,
    pub default_skill: String,
    pub blocked_skills: Vec<String>,
    pub note: String,
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
    closed_task: Option<String>,
}

enum ActiveTaskState {
    Present(TaskRecord),
    Missing(String),
    None,
}

impl ActiveTaskState {
    fn task(&self) -> Option<&TaskRecord> {
        match self {
            Self::Present(task) => Some(task),
            Self::Missing(_) | Self::None => None,
        }
    }

    fn missing_name(&self) -> Option<&str> {
        match self {
            Self::Missing(name) => Some(name),
            Self::Present(_) | Self::None => None,
        }
    }
}

impl WorkflowState {
    pub fn additional_context(&self) -> String {
        let session_line = self
            .session
            .as_ref()
            .map(|session| format!("会话：{}（{}）", session.key, session.source))
            .unwrap_or_else(|| "会话：global fallback".to_string());
        let previous = self
            .runtime
            .previous_active_task
            .as_deref()
            .unwrap_or("none");
        let runtime_line = format!(
            "注入：#{}，时间：{}\n活跃任务是否变化：{}\n上一个活跃任务：{}\n运行日志：{}\n会话日志：{}",
            self.runtime.injection_count,
            self.runtime.last_seen_at,
            self.runtime.active_task_changed,
            previous,
            self.runtime.log_path,
            self.runtime.journal_path
        );
        let memory_line = format_memory(&self.memory);
        let peers_line = format_peer_sessions(&self.peers);

        let Some(task) = &self.active_task else {
            return format!(
                "<dijiang-workflow-state>\n{session_line}\n{runtime_line}\n{memory_line}\n{peers_line}\n活跃任务：none\n下一步：{}\n</dijiang-workflow-state>",
                self.guidance
            );
        };

        let route_gate_line = self
            .route_gate
            .as_ref()
            .map(format_route_gate)
            .unwrap_or_default();

        format!(
            "<dijiang-workflow-state>\n{session_line}\n{runtime_line}\n{memory_line}\n{peers_line}\n活跃任务：{}\n标题：{}\n状态：{}\n任务路径：{}\n指引：{}\n{}\n加载上下文：读取 task.json；如果存在，也读取 prd.md/design.md/implement.md/check 产物。\n</dijiang-workflow-state>",
            task.id, task.title, task.status, task.task_path, self.guidance, route_gate_line
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
    let active_task_state = match store::read_active_task_for_session(dijiang_dir, identity)? {
        Some(active_task_name) => match store::load_task(&dijiang_dir.join("tasks"), &active_task_name)
        {
            Ok(task) => ActiveTaskState::Present(task),
            Err(TaskError::NotFound(_)) => ActiveTaskState::Missing(active_task_name),
            Err(error) => return Err(error),
        },
        None => ActiveTaskState::None,
    };
    let task = active_task_state.task();
    let runtime = record_runtime_injection(dijiang_dir, identity, task)?;
    let memory = load_recent_memory(dijiang_dir, &runtime.journal_path, 5);
    let peers = load_peer_sessions(dijiang_dir, identity, 8)?;

    let Some(task) = task else {
        let guidance = match active_task_state.missing_name() {
            Some(missing) => format!(
                "active task `{missing}` 指向缺失的 `.dijiang/tasks/{missing}/task.json`，task state 已陈旧。按 dj-hunt 排查；用 `dijiang task current` / `dijiang task list` 对比状态，确认后运行 `dijiang start <name>` 恢复有效任务，或清理 stale active task。"
            ),
            None => "run `dijiang start <name>` or read `.dijiang/workflow.md` before coding."
                .to_string(),
        };
        return Ok(WorkflowState {
            session,
            active_task: None,
            guidance,
            runtime,
            memory,
            peers,
            route_gate: None,
        });
    };

    let guidance = status_guidance(&task.status).to_string();
    let active_task = Some(workflow_task(dijiang_dir, &task));
    let route_gate = Some(workflow_route_gate(&task.status));
    Ok(WorkflowState {
        session,
        active_task,
        guidance,
        runtime,
        memory,
        peers,
        route_gate,
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
    record.closed_task = None;

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
    let route_gate_event = active_task.map(|task| {
        let gate = workflow_route_gate(&task.status);
        serde_json::json!({
            "capsule": gate.capsule,
            "default_skill": gate.default_skill,
            "allowed_skills": gate.allowed_skills,
            "blocked_skills": gate.blocked_skills,
        })
    });
    let event = serde_json::json!({
        "event": "workflow_state_injected",
        "session_key": identity.key(),
        "source": identity.source(),
        "injection_count": record.injection_count,
        "active_task": current_active_task,
        "active_task_detail": active_task_event,
        "route_gate": route_gate_event,
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
    let sessions_dir = dijiang_dir.join("workspace").join(developer).join("sessions");
    fs::create_dir_all(&sessions_dir)?;
    let journal_path = sessions_dir.join(format!("{}.jsonl", identity.key()));
    let mut journal = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&journal_path)?;
    writeln!(journal, "{}", serde_json::to_string(event)?)?;
    Ok(journal_path)
}

fn load_recent_memory(dijiang_dir: &Path, journal_path: &str, limit: usize) -> WorkflowMemory {
    let path = dijiang_dir.parent().unwrap_or(dijiang_dir).join(journal_path);
    let Ok(content) = fs::read_to_string(path) else {
        return WorkflowMemory {
            summary: "当前窗口没有上一轮会话记忆。".to_string(),
            events: Vec::new(),
        };
    };

    let mut events = content
        .lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter_map(memory_event_from_json)
        .take(limit)
        .collect::<Vec<_>>();
    events.reverse();

    let summary = if events.is_empty() {
        "当前窗口没有上一轮会话记忆。".to_string()
    } else {
        format!("当前窗口已加载 {} 条最近会话事件。", events.len())
    };

    WorkflowMemory { summary, events }
}

fn memory_event_from_json(value: serde_json::Value) -> Option<WorkflowMemoryEvent> {
    let kind = value.get("event")?.as_str()?.to_string();
    let at = value
        .get("at")
        .or_else(|| value.get("closed_at"))
        .and_then(|value| value.as_str())
        .map(str::to_string);
    let detail = match kind.as_str() {
        "workflow_state_injected" => {
            let count = value
                .get("injection_count")
                .and_then(|value| value.as_u64())
                .unwrap_or_default();
            let active = value
                .get("active_task")
                .and_then(|value| value.as_str())
                .unwrap_or("none");
            let previous = value
                .get("previous_active_task")
                .and_then(|value| value.as_str())
                .unwrap_or("none");
            let changed = value
                .get("active_task_changed")
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            format!("注入 #{count}：活跃任务={active}，上一个任务={previous}，变化={changed}")
        }
        "session_closed" => {
            let task = value
                .get("task")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown");
            let verification = value
                .get("verification")
                .and_then(|value| value.as_str())
                .unwrap_or("not recorded");
            format!("会话已关闭：任务={task}，验证={verification}")
        }
        _ => return None,
    };

    Some(WorkflowMemoryEvent { kind, detail, at })
}

fn format_memory(memory: &WorkflowMemory) -> String {
    if memory.events.is_empty() {
        return format!("最近记忆：{}", memory.summary);
    }

    let events = memory
        .events
        .iter()
        .map(|event| match &event.at {
            Some(at) => format!("- [{}] {}", at, event.detail),
            None => format!("- {}", event.detail),
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("最近记忆：{}\n{}", memory.summary, events)
}

fn load_peer_sessions(
    dijiang_dir: &Path,
    identity: Option<&SessionIdentity>,
    limit: usize,
) -> Result<Vec<WorkflowPeerSession>, TaskError> {
    let sessions_dir = dijiang_dir.join(".runtime").join("sessions");
    if !sessions_dir.exists() {
        return Ok(Vec::new());
    }
    let current_key = identity
        .map(|identity| identity.key().to_string())
        .unwrap_or_else(|| {
            SessionIdentity::new("global", "global")
                .expect("literal global session key is valid")
                .key()
                .to_string()
        });

    let mut peers = Vec::new();
    for entry in fs::read_dir(sessions_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let content = fs::read_to_string(path)?;
        let record = serde_json::from_str::<RuntimeSessionRecord>(&content).unwrap_or_default();
        if record.session_key.is_empty() || record.session_key == current_key {
            continue;
        }
        peers.push(WorkflowPeerSession {
            key: record.session_key,
            source: record.source,
            current_task: record.current_task,
            injection_count: record.injection_count,
            last_seen_at: record.last_seen_at,
            closed_task: record.closed_task,
        });
    }

    peers.sort_by(|left, right| right.last_seen_at.cmp(&left.last_seen_at));
    peers.truncate(limit);
    Ok(peers)
}

fn format_peer_sessions(peers: &[WorkflowPeerSession]) -> String {
    if peers.is_empty() {
        return "其他活跃窗口：none".to_string();
    }

    let sessions = peers
        .iter()
        .map(|peer| {
            let task = peer
                .current_task
                .as_deref()
                .or(peer.closed_task.as_deref())
                .unwrap_or("none");
            let state = if peer.current_task.is_some() {
                "active"
            } else if peer.closed_task.is_some() {
                "closed"
            } else {
                "idle"
            };
            format!(
                "- {} ({}) 任务={} 状态={} 注入={} 最近活跃={}",
                peer.key, peer.source, task, state, peer.injection_count, peer.last_seen_at,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("其他活跃窗口：{}\n{}", peers.len(), sessions)
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

fn workflow_route_gate(status: &TaskStatus) -> WorkflowRouteGate {
    let summary = summarize_route_gate(status);
    WorkflowRouteGate {
        capsule: summary.capsule.as_str().to_string(),
        allowed_skills: summary
            .allowed_skills
            .into_iter()
            .map(str::to_string)
            .collect(),
        default_skill: summary.default_skill.to_string(),
        blocked_skills: summary
            .blocked_skills
            .into_iter()
            .map(str::to_string)
            .collect(),
        note: summary.note,
    }
}

fn format_route_gate(route_gate: &WorkflowRouteGate) -> String {
    format!(
        "Route Gate：capsule={}；default_skill={}；allowed={}；blocked={}；note={}",
        route_gate.capsule,
        route_gate.default_skill,
        route_gate.allowed_skills.join(", "),
        route_gate.blocked_skills.join(", "),
        route_gate.note,
    )
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

        assert!(context_a.contains("会话：dijiang_window-a（dijiang）"));
        assert!(context_a.contains("注入：#1"));
        assert!(context_a.contains("活跃任务是否变化：true"));
        assert!(context_a.contains("活跃任务：task-a"));
        assert!(context_a.contains("标题：Task A"));
        assert!(context_a.contains("Route Gate：capsule=implement"));
        assert!(context_a.contains("default_skill=dj-implement"));
        assert!(context_a.contains(
            "会话日志：.dijiang/workspace/tester/sessions/dijiang_window-a.jsonl"
        ));
        assert!(context_b.contains("会话：dijiang_window-b（dijiang）"));
        assert!(context_b.contains("活跃任务：task-b"));
        assert!(context_b.contains("标题：Task B"));

        let next_a = build_for_session(&dijiang_dir, Some(&window_a))
            .unwrap()
            .additional_context();
        assert!(next_a.contains("注入：#2"));
        assert!(next_a.contains("活跃任务是否变化：false"));

        let log = std::fs::read_to_string(dijiang_dir.join(".runtime/workflow-state.log")).unwrap();
        assert!(log.contains("workflow_state_injected"));
        assert!(log.contains("dijiang_window-a"));
        assert!(log.contains("dijiang_window-b"));
        assert!(log.contains("route_gate"));

        let journal_a = std::fs::read_to_string(
            dijiang_dir.join("workspace/tester/sessions/dijiang_window-a.jsonl"),
        )
        .unwrap();
        assert_eq!(journal_a.lines().count(), 2);
        assert!(journal_a.contains("workflow_state_injected"));
        assert!(journal_a.contains("\"title\":\"Task A\""));
        assert!(journal_a.contains("\"injection_count\":2"));
    }

    #[test]
    fn builds_recoverable_context_for_stale_active_task_pointer() {
        let dir = tempfile::tempdir().unwrap();
        let dijiang_dir = dir.path().join(".dijiang");
        std::fs::create_dir_all(&dijiang_dir).unwrap();
        std::fs::write(
            dijiang_dir.join("config.toml"),
            "[project]\ndeveloper = \"tester\"\n",
        )
        .unwrap();

        let window = store::SessionIdentity::new("dijiang", "stale-window").unwrap();
        store::write_active_task_for_session(&dijiang_dir, "missing-task", Some(&window)).unwrap();

        let state = build_for_session(&dijiang_dir, Some(&window)).unwrap();
        let context = state.additional_context();

        assert!(state.active_task.is_none());
        assert!(context.contains("会话：dijiang_stale-window（dijiang）"));
        assert!(context.contains("活跃任务：none"));
        assert!(context.contains("missing-task"));
        assert!(context.contains("task state 已陈旧"));
        assert!(context.contains("dj-hunt"));

        let session_runtime = std::fs::read_to_string(
            dijiang_dir.join(".runtime/sessions/dijiang_stale-window.json"),
        )
        .unwrap();
        assert!(session_runtime.contains("\"current_task\": null"));
        assert!(session_runtime.contains("\"last_active_task\": null"));

        let log = std::fs::read_to_string(dijiang_dir.join(".runtime/workflow-state.log")).unwrap();
        assert!(log.contains("workflow_state_injected"));
        assert!(log.contains("\"active_task\":null"));
    }

    #[test]
    fn planning_state_exposes_align_capsule() {
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

        let mut planning = task("align-task", "Align Task");
        planning.status = TaskStatus::Planning;
        store::save_task(&tasks_dir, &planning).unwrap();
        let window = store::SessionIdentity::new("dijiang", "align-window").unwrap();
        store::write_active_task_for_session(&dijiang_dir, "align-task", Some(&window)).unwrap();

        let context = build_for_session(&dijiang_dir, Some(&window))
            .unwrap()
            .additional_context();
        assert!(context.contains("Route Gate：capsule=align"));
        assert!(context.contains("default_skill=dj-grill"));
    }

    #[test]
    fn paused_state_exposes_continue_route() {
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

        let mut paused = task("paused-task", "Paused Task");
        paused.status = TaskStatus::Paused;
        store::save_task(&tasks_dir, &paused).unwrap();
        let window = store::SessionIdentity::new("dijiang", "paused-window").unwrap();
        store::write_active_task_for_session(&dijiang_dir, "paused-task", Some(&window)).unwrap();

        let context = build_for_session(&dijiang_dir, Some(&window))
            .unwrap()
            .additional_context();
        assert!(context.contains("Route Gate：capsule=resume"));
        assert!(context.contains("default_skill=dijiang-continue"));
    }

    #[test]
    fn archived_state_exposes_restart_requirement() {
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

        let mut archived = task("archived-task", "Archived Task");
        archived.status = TaskStatus::Archived;
        store::save_task(&tasks_dir, &archived).unwrap();
        let window = store::SessionIdentity::new("dijiang", "archived-window").unwrap();
        store::write_active_task_for_session(&dijiang_dir, "archived-task", Some(&window)).unwrap();

        let context = build_for_session(&dijiang_dir, Some(&window))
            .unwrap()
            .additional_context();
        assert!(context.contains("Route Gate：capsule=idle"));
        assert!(context.contains("default_skill=dijiang-start"));
    }
}
