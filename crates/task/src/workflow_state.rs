use std::fs;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::git_gate::summarize_git_gate;
use crate::route_gate::summarize_route_gate;
use crate::skill_manifest::manifests_for_capsule;
use crate::store::{self, SessionIdentity, TaskError};
use crate::types::{TaskRecord, TaskStatus};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowState {
    pub session: Option<WorkflowSession>,
    pub active_task: Option<WorkflowTask>,
    pub guidance: String,
    pub runtime: WorkflowRuntime,
    pub loop_state: Option<WorkflowLoopState>,
    pub memory: WorkflowMemory,
    pub peers: Vec<WorkflowPeerSession>,
    pub route_gate: Option<WorkflowRouteGate>,
    pub git_gate: Option<WorkflowGitGate>,
    pub skill_manifests: Vec<WorkflowSkillManifest>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowLoopState {
    pub goal: String,
    pub mode: String,
    pub progress: WorkflowLoopProgress,
    pub stop_conditions: Vec<String>,
    pub next_action: String,
    pub next_skill: Option<String>,
    pub retry: WorkflowLoopRetry,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowLoopProgress {
    pub status: String,
    pub signal: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowLoopRetry {
    pub attempt: u64,
    pub max_attempts: Option<u64>,
    pub remaining_attempts: Option<u64>,
    pub can_retry: bool,
    pub last_failure: Option<String>,
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
    pub recommended_path: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowGitGate {
    pub state: String,
    pub branch: Option<String>,
    pub base_branch: Option<String>,
    pub worktree_path: Option<String>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowSkillManifest {
    pub name: String,
    pub summary: String,
    pub phases: Vec<String>,
    pub risk: String,
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
        let loop_line = self
            .loop_state
            .as_ref()
            .map(format_loop_state)
            .unwrap_or_else(|| "Loop：none".to_string());
        let memory_line = format_memory(&self.memory);
        let peers_line = format_peer_sessions(&self.peers);

        let Some(task) = &self.active_task else {
            return format!(
                "<dijiang-workflow-state>\n{session_line}\n{runtime_line}\n{loop_line}\n{memory_line}\n{peers_line}\n活跃任务：none\n下一步：{}\n</dijiang-workflow-state>",
                self.guidance
            );
        };

        let route_gate_line = self
            .route_gate
            .as_ref()
            .map(format_route_gate)
            .unwrap_or_default();
        let git_gate_line = self
            .git_gate
            .as_ref()
            .map(format_git_gate)
            .unwrap_or_default();
        let skill_manifest_line = format_skill_manifests(&self.skill_manifests);
        let target_skill_line = format_target_skill_bodies(
            self.route_gate.as_ref(),
            &self.skill_manifests,
            self.route_gate
                .as_ref()
                .map(|route_gate| route_gate.default_skill.as_str())
                .unwrap_or_default(),
            self.route_gate
                .as_ref()
                .map(|route_gate| route_gate.recommended_path.as_str())
                .unwrap_or_default(),
        );

        format!(
            "<dijiang-workflow-state>\n{session_line}\n{runtime_line}\n{loop_line}\n{memory_line}\n{peers_line}\n活跃任务：{}\n标题：{}\n状态：{}\n任务路径：{}\n指引：{}\n{}\n{}\n{}\n{}\n加载上下文：读取 task.json；如果存在，也读取 prd.md/design.md/implement.md/check 产物。\n</dijiang-workflow-state>",
            task.id,
            task.title,
            task.status,
            task.task_path,
            self.guidance,
            route_gate_line,
            git_gate_line,
            skill_manifest_line,
            target_skill_line,
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
            loop_state: None,
            memory,
            peers,
            route_gate: None,
            git_gate: None,
            skill_manifests: vec![],
        });
    };

    let guidance = status_guidance(&task.status).to_string();
    let active_task = Some(workflow_task(dijiang_dir, &task));
    let recommended_path = task.meta.get("dispatch").and_then(|d| d.get("recommended_path")).and_then(|v| v.as_str()).unwrap_or("");
    let route_gate = Some(workflow_route_gate(&task.status, &recommended_path));
    let git_gate = Some(workflow_git_gate(dijiang_dir.parent().unwrap_or(dijiang_dir), &task));
    let skill_manifests = workflow_skill_manifests(&task.status);
    let loop_state = Some(workflow_loop_state(&task, recommended_path));
    Ok(WorkflowState {
        session,
        active_task,
        guidance,
        runtime,
        loop_state,
        memory,
        peers,
        route_gate,
        git_gate,
        skill_manifests,
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
        let recommended_path = task.meta.get("dispatch").and_then(|d| d.get("recommended_path")).and_then(|v| v.as_str()).unwrap_or("");
        let gate = workflow_route_gate(&task.status, &recommended_path);
        serde_json::json!({
            "capsule": gate.capsule,
            "default_skill": gate.default_skill,
            "allowed_skills": gate.allowed_skills,
            "blocked_skills": gate.blocked_skills,
        })
    });
    let git_gate_event = active_task.map(|task| {
        let gate = workflow_git_gate(dijiang_dir.parent().unwrap_or(dijiang_dir), task);
        serde_json::json!({
            "state": gate.state,
            "branch": gate.branch,
            "base_branch": gate.base_branch,
            "worktree_path": gate.worktree_path,
            "note": gate.note,
        })
    });
    let skill_manifests_event = active_task.map(|task| {
        workflow_skill_manifests(&task.status)
            .into_iter()
            .map(|manifest| {
                serde_json::json!({
                    "name": manifest.name,
                    "risk": manifest.risk,
                    "phases": manifest.phases,
                })
            })
            .collect::<Vec<_>>()
    });
    let event = serde_json::json!({
        "event": "workflow_state_injected",
        "session_key": identity.key(),
        "source": identity.source(),
        "injection_count": record.injection_count,
        "active_task": current_active_task,
        "active_task_detail": active_task_event,
        "route_gate": route_gate_event,
        "git_gate": git_gate_event,
        "skill_manifests": skill_manifests_event,
        "loop_state": active_task.map(|task| {
            let recommended_path = task
                .meta
                .get("dispatch")
                .and_then(|d| d.get("recommended_path"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            serde_json::to_value(workflow_loop_state(task, recommended_path))
                .expect("workflow loop state serializes")
        }),
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
            let progress = value
                .get("loop_state")
                .and_then(|value| value.get("progress"))
                .and_then(|value| value.get("signal"))
                .and_then(|value| value.as_str())
                .unwrap_or("no loop signal");
            let next_action = value
                .get("loop_state")
                .and_then(|value| value.get("nextAction"))
                .or_else(|| value.get("loop_state").and_then(|value| value.get("next_action")))
                .and_then(|value| value.as_str())
                .unwrap_or("no next action");
            format!(
                "注入 #{count}：活跃任务={active}，上一个任务={previous}，变化={changed}，进展={progress}，下一步={next_action}",
            )
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

fn workflow_route_gate(status: &TaskStatus, recommended_path: &str) -> WorkflowRouteGate {
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
        recommended_path: recommended_path.to_string(),
        note: summary.note,
    }
}

fn format_route_gate(route_gate: &WorkflowRouteGate) -> String {
    format!(
        "Route Gate：capsule={}；default_skill={}；recommended_path={}；allowed={}；blocked={}；note={}",
        route_gate.capsule,
        route_gate.default_skill,
        route_gate.recommended_path,
        route_gate.allowed_skills.join(", "),
        route_gate.blocked_skills.join(", "),
        route_gate.note,
    )
}

fn workflow_git_gate(project_root: &Path, task: &TaskRecord) -> WorkflowGitGate {
    let summary = summarize_git_gate(task, project_root);
    WorkflowGitGate {
        state: summary.state.as_str().to_string(),
        branch: summary.branch,
        base_branch: summary.base_branch,
        worktree_path: summary.worktree_path,
        note: summary.note,
    }
}

fn workflow_skill_manifests(status: &TaskStatus) -> Vec<WorkflowSkillManifest> {
    let capsule = summarize_route_gate(status).capsule;
    manifests_for_capsule(capsule)
        .into_iter()
        .map(|entry| WorkflowSkillManifest {
            name: entry.name.to_string(),
            summary: entry.summary.to_string(),
            phases: entry.phases.iter().map(|phase| (*phase).to_string()).collect(),
            risk: entry.risk.to_string(),
        })
        .collect()
}

fn format_target_skill_bodies(
    route_gate: Option<&WorkflowRouteGate>,
    _skill_manifests: &[WorkflowSkillManifest],
    primary_skill: &str,
    recommended_path: &str,
) -> String {
    let Some(route_gate) = route_gate else {
        return String::new();
    };
    let selected = crate::skill_manifest::select_skill_bodies(
        match route_gate.capsule.as_str() {
            "align" => crate::route_gate::WorkflowCapsule::Align,
            "implement" => crate::route_gate::WorkflowCapsule::Implement,
            "check" => crate::route_gate::WorkflowCapsule::Check,
            "finish" => crate::route_gate::WorkflowCapsule::Finish,
            "resume" => crate::route_gate::WorkflowCapsule::Resume,
            _ => crate::route_gate::WorkflowCapsule::Idle,
        },
        primary_skill,
        recommended_path,
    );
    if selected.is_empty() {
        return String::new();
    }
    let mut cache = crate::skill_manifest::SkillBodyCache::default();
    let rendered = crate::skill_manifest::render_selected_skill_bodies(&selected, &mut cache);
    if rendered.is_empty() {
        return String::new();
    }
    rendered
}

fn format_git_gate(git_gate: &WorkflowGitGate) -> String {
    format!(
        "Git Gate：state={}；branch={}；baseBranch={}；worktreePath={}；note={}",
        git_gate.state,
        git_gate.branch.as_deref().unwrap_or("none"),
        git_gate.base_branch.as_deref().unwrap_or("none"),
        git_gate.worktree_path.as_deref().unwrap_or("none"),
        git_gate.note,
    )
}

fn format_skill_manifests(skill_manifests: &[WorkflowSkillManifest]) -> String {
    if skill_manifests.is_empty() {
        return "Skill Manifests：none".to_string();
    }

    let entries = skill_manifests
        .iter()
        .map(|manifest| format!("{}({}; risk={})", manifest.name, manifest.summary, manifest.risk))
        .collect::<Vec<_>>()
        .join(", ");
    format!("Skill Manifests：{}", entries)
}

fn workflow_loop_state(task: &TaskRecord, recommended_path: &str) -> WorkflowLoopState {
    let dispatch = task.meta.get("dispatch");
    let goal = task
        .acceptance_criteria
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            let title = task.title.trim();
            if title.is_empty() {
                format!("推进任务 {} 到可验证完成", task.name)
            } else {
                title.to_string()
            }
        });
    let mode = summarize_route_gate(&task.status).capsule.as_str().to_string();
    let attempt = dispatch
        .and_then(|value| value.get("attempt"))
        .and_then(|value| value.as_u64())
        .unwrap_or(1);
    let max_attempts = dispatch
        .and_then(|value| value.get("max_attempts"))
        .and_then(|value| value.as_u64());
    let last_failure = dispatch
        .and_then(|value| value.get("last_failure"))
        .and_then(|value| value.as_str())
        .map(str::to_string);
    let progress_status = match task.status {
        TaskStatus::Planning => "aligning",
        TaskStatus::InProgress => "executing",
        TaskStatus::Completed => "verified",
        TaskStatus::Archived => "closed",
        TaskStatus::Paused => "paused",
    }
    .to_string();
    let progress_signal = match task.status {
        TaskStatus::Planning => "需求与验收标准仍需对齐",
        TaskStatus::InProgress => "实现与验证正在推进",
        TaskStatus::Completed => "实现已完成，等待 finish-work 收口",
        TaskStatus::Archived => "任务已归档，loop 已关闭",
        TaskStatus::Paused => "任务已暂停，等待恢复",
    }
    .to_string();
    let stop_conditions = match task.status {
        TaskStatus::Planning => vec![
            "需求、范围和验收标准已经确认".to_string(),
            "实现入口已切换到 dj-implement 或 dj-tdd".to_string(),
        ],
        TaskStatus::InProgress => vec![
            "目标行为已实现".to_string(),
            "GREEN command 通过".to_string(),
            "相关回归验证通过后切入 dj-check".to_string(),
        ],
        TaskStatus::Completed => vec![
            "dj-check 完成并记录验证证据".to_string(),
            "finish-work 已归档任务与会话".to_string(),
        ],
        TaskStatus::Archived => vec!["任务已经归档，无需继续 loop".to_string()],
        TaskStatus::Paused => vec![
            "恢复上下文并重新进入实现或检查路径".to_string(),
            "确认暂停原因已解除".to_string(),
        ],
    };
    let next_skill = next_skill_from_recommended_path(recommended_path)
        .or_else(|| Some(summarize_route_gate(&task.status).default_skill.to_string()));
    let next_action = match task.status {
        TaskStatus::Planning => "继续对齐需求并收敛 acceptance".to_string(),
        TaskStatus::InProgress => {
            if let Some(skill) = next_skill.as_deref() {
                format!("继续当前 loop，并按 {skill} 推进下一轮最小验证闭环")
            } else {
                "继续当前 loop，并推进下一轮最小验证闭环".to_string()
            }
        }
        TaskStatus::Completed => "汇总验证证据并准备 finish-work".to_string(),
        TaskStatus::Archived => "任务已结束，无需继续执行".to_string(),
        TaskStatus::Paused => "恢复任务上下文后继续 loop".to_string(),
    };
    let remaining_attempts = max_attempts.map(|max| max.saturating_sub(attempt));
    let can_retry = match task.status {
        TaskStatus::Archived | TaskStatus::Completed => false,
        _ => remaining_attempts.map(|remaining| remaining > 0).unwrap_or(true),
    };

    WorkflowLoopState {
        goal,
        mode,
        progress: WorkflowLoopProgress {
            status: progress_status,
            signal: progress_signal,
        },
        stop_conditions,
        next_action,
        next_skill,
        retry: WorkflowLoopRetry {
            attempt,
            max_attempts,
            remaining_attempts,
            can_retry,
            last_failure,
        },
    }
}

fn next_skill_from_recommended_path(recommended_path: &str) -> Option<String> {
    recommended_path
        .split("->")
        .map(str::trim)
        .find(|segment| !segment.is_empty())
        .and_then(|segment| segment.split_whitespace().next())
        .map(str::to_string)
}

fn format_loop_state(loop_state: &WorkflowLoopState) -> String {
    let stop_conditions = if loop_state.stop_conditions.is_empty() {
        "none".to_string()
    } else {
        loop_state.stop_conditions.join(" | ")
    };
    let retry = format!(
        "attempt={}; max={}; remaining={}; can_retry={}; last_failure={}",
        loop_state.retry.attempt,
        loop_state
            .retry
            .max_attempts
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unbounded".to_string()),
        loop_state
            .retry
            .remaining_attempts
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        loop_state.retry.can_retry,
        loop_state
            .retry
            .last_failure
            .as_deref()
            .unwrap_or("none"),
    );
    format!(
        "Loop：goal={}；mode={}；progress={} ({})；next_skill={}；next_action={}；stop_conditions={}；retry={}",
        loop_state.goal,
        loop_state.mode,
        loop_state.progress.status,
        loop_state.progress.signal,
        loop_state.next_skill.as_deref().unwrap_or("none"),
        loop_state.next_action,
        stop_conditions,
        retry,
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
        assert!(context_a.contains("Git Gate：state=ready"));
        assert!(context_a.contains("worktreePath=none"));
        assert!(context_a.contains("Skill Manifests："));
        assert!(context_a.contains("dj-implement("));
        assert!(context_a.contains("dj-tdd("));
        assert!(context_a.contains("<dijiang-target-skill role=\"primary\" name=\"dj-implement\">"));
        assert!(context_a.contains("Loop：goal=Task A"));
        assert!(context_a.contains("progress=executing (实现与验证正在推进)"));
        assert!(context_a.contains("next_skill=dj-implement"));
        assert!(context_a.contains("retry=attempt=1; max=unbounded; remaining=unknown; can_retry=true; last_failure=none"));
        assert!(context_b.contains("会话：dijiang_window-b（dijiang）"));
        assert!(context_b.contains("活跃任务：task-b"));
        assert!(context_b.contains("标题：Task B"));

        let next_a = build_for_session(&dijiang_dir, Some(&window_a))
            .unwrap()
            .additional_context();
        assert!(next_a.contains("注入：#2"));
        assert!(next_a.contains("活跃任务是否变化：false"));
        assert!(next_a.contains("下一步=继续当前 loop，并按 dj-implement 推进下一轮最小验证闭环") || next_a.contains("next_action=继续当前 loop，并按 dj-implement 推进下一轮最小验证闭环"));

        let log = std::fs::read_to_string(dijiang_dir.join(".runtime/workflow-state.log")).unwrap();
        assert!(log.contains("workflow_state_injected"));
        assert!(log.contains("dijiang_window-a"));
        assert!(log.contains("dijiang_window-b"));
        assert!(log.contains("route_gate"));
        assert!(log.contains("skill_manifests"));
        assert!(log.contains("loop_state"));

        let journal_a = std::fs::read_to_string(
            dijiang_dir.join("workspace/tester/sessions/dijiang_window-a.jsonl"),
        )
        .unwrap();
        assert_eq!(journal_a.lines().count(), 2);
        assert!(journal_a.contains("workflow_state_injected"));
        assert!(journal_a.contains("\"title\":\"Task A\""));
        assert!(journal_a.contains("\"injection_count\":2"));
        assert!(journal_a.contains("\"loop_state\""));
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
        assert!(context.contains("Git Gate：state=ready"));
        assert!(context.contains("Skill Manifests：dj-grill"));
        assert!(context.contains("dj-output"));
        assert!(context.contains("<dijiang-target-skill role=\"primary\" name=\"dj-grill\">"));
        assert!(context.contains("Loop：goal=Align Task"));
        assert!(context.contains("progress=aligning (需求与验收标准仍需对齐)"));
        assert!(context.contains("next_skill=dj-grill"));
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
        assert!(context.contains("Git Gate：state=ready"));
        assert!(context.contains("Skill Manifests：dijiang-continue"));
        assert!(context.contains("<dijiang-target-skill role=\"primary\" name=\"dijiang-continue\">"));
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
        assert!(context.contains("Git Gate：state=ready"));
        assert!(context.contains("Skill Manifests：dijiang-start"));
        assert!(context.contains("<dijiang-target-skill role=\"primary\" name=\"dijiang-start\">"));
    }
}
