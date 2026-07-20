use serde::{Deserialize, Serialize};

pub use crate::circuit_breaker::BreakerDecision;
use crate::types::{TaskRecord, TaskStatus};
pub use crate::workflow_state::tag_parser::WorkflowTagMap;

// ── Core types ───────────────────────────────────────────────────────

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
    pub workflow_tags: WorkflowTagMap,
    pub learned_memory: WorkflowLearnedMemory,
    pub circuit_breaker_status: Option<BreakerDecision>,
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
    pub agent_focus: String,
    pub resolved_agent: Option<String>,
    pub memory_writeback: WorkflowMemoryWriteback,
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
pub struct WorkflowMemoryWriteback {
    pub outcome: String,
    pub next_tactic: String,
    pub next_pattern: String,
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
pub struct WorkflowLearnedMemory {
    pub summary: String,
    pub tactics: Vec<WorkflowLearnedTactic>,
    pub patterns: Vec<WorkflowLearnedPattern>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowLearnedTactic {
    pub name: String,
    pub description: String,
    pub win_rate: f64,
    pub source: String,
    pub last_used: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowLearnedPattern {
    pub name: String,
    pub description: String,
    pub steps: Vec<String>,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cadence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub week_one_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_cost: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub human_gates: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub phases: Vec<String>,
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

// ── Private internal types ──────────────────────────────────────────

pub(super) struct WorkflowTaskContext<'a> {
    pub task: &'a TaskRecord,
    pub effective_status: TaskStatus,
    pub recommended_path: &'a str,
}

pub(super) struct DispatchMeta<'a> {
    pub task_type: Option<&'a str>,
    pub primary_intent: Option<&'a str>,
    pub skill: Option<&'a str>,
    pub recommended_path: Option<&'a str>,
    pub action: Option<&'a str>,
    pub reason: Option<&'a str>,
    pub next_action: Option<&'a str>,
    pub attempt: Option<u64>,
    pub max_attempts: Option<u64>,
    pub last_failure: Option<&'a str>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) struct RuntimeSessionRecord {
    pub session_key: String,
    pub source: String,
    pub current_task: Option<String>,
    pub last_active_task: Option<String>,
    pub injection_count: u64,
    pub last_seen_at: String,
    pub closed_task: Option<String>,
}

pub(super) enum ActiveTaskState {
    Present(TaskRecord),
    Missing(String),
    None,
}

impl ActiveTaskState {
    pub fn task(&self) -> Option<&TaskRecord> {
        match self {
            Self::Present(task) => Some(task),
            Self::Missing(_) | Self::None => None,
        }
    }

    pub fn missing_name(&self) -> Option<&str> {
        match self {
            Self::Missing(name) => Some(name),
            Self::Present(_) | Self::None => None,
        }
    }
}

// ── WorkflowState impl ───────────────────────────────────────────────

impl WorkflowState {
    pub fn additional_context(&self) -> String {
        use crate::workflow_state::{
            format_learned_memory, format_loop_state, format_memory,
            format_peer_sessions, format_route_gate, format_git_gate,
            format_skill_manifests, format_target_skill_bodies,
        };

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
        let learned_line = format_learned_memory(&self.learned_memory);
        let breaker_line = self
            .circuit_breaker_status
            .as_ref()
            .map(|d| format!("Circuit Breaker：{}", d))
            .unwrap_or_else(|| "Circuit Breaker：none".to_string());

        let tag_line = match &self.active_task {
            Some(task) => {
                let status = &task.status;
                match crate::workflow_state::tag_parser::tag_for_status(&self.workflow_tags, status) {
                    Some(text) => format!("Workflow 标签 [{}]:\n{}", status, text),
                    None => format!("Workflow 标签 [{}]: （无）", status),
                }
            }
            None => {
                match crate::workflow_state::tag_parser::tag_for_status(&self.workflow_tags, "no_task") {
                    Some(text) => format!("Workflow 标签 [no_task]:\n{}", text),
                    None => String::new(),
                }
            }
        };

        let Some(task) = &self.active_task else {
            return format!(
                "<dijiang-workflow-state>\n{session_line}\n{runtime_line}\n{loop_line}\n{learned_line}\n{breaker_line}\n{memory_line}\n{peers_line}\n{tag_line}\n活跃任务：none\n下一步：{}\n</dijiang-workflow-state>",
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

        let agent_line = self.loop_state
            .as_ref()
            .and_then(|ls| ls.resolved_agent.as_ref())
            .and_then(|name| {
                let entry = crate::agent_manifest::agent_by_name(name)?;
                Some(format!("<dijiang-agent name=\"{}\" summary=\"{}\" />", entry.name, entry.summary))
            })
            .unwrap_or_default();

        format!(
            "<dijiang-workflow-state>\n{session_line}\n{runtime_line}\n{loop_line}\n{learned_line}\n{breaker_line}\n{memory_line}\n{peers_line}\n{tag_line}\n{agent_line}\n活跃任务：{}\n标题：{}\n状态：{}\n任务路径：{}\n指引：{}\n{}\n{}\n{}\n{}\n加载上下文：读取 task.json；如果存在，也读取 prd.md/design.md/implement.md/check 产物。\n</dijiang-workflow-state>",
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

