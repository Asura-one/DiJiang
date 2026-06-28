use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Task status values matching Trellis format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Planning,
    InProgress,
    Completed,
    Archived,
    Paused,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Planning => "planning",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Completed => "completed",
            TaskStatus::Archived => "archived",
            TaskStatus::Paused => "paused",
        }
    }

    /// Infer the development phase from status.
    pub fn infer_phase(&self) -> &'static str {
        match self {
            TaskStatus::Planning => "plan",
            TaskStatus::InProgress | TaskStatus::Paused => "implement",
            TaskStatus::Completed => "complete",
            TaskStatus::Archived => "archive",
        }
    }
}

/// Full task record matching Trellis's `TrellisTaskRecord` exactly (24 fields
/// in `TASK_RECORD_FIELD_ORDER`), plus DiJiang extensions.
///
/// Field order below is deliberate — serde serialises in declaration order
/// when `#[serde(rename_all = "camelCase")]` is used on the struct.
/// Trellis-standard Optional fields always serialize (even as `null`).
/// DiJiang extensions use `skip_serializing_if` to stay out of standard files.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRecord {
    // ── Identity (Trellis, in TASK_RECORD_FIELD_ORDER) ──
    pub id: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    #[serde(default)]
    pub dev_type: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub package: Option<String>,
    pub priority: String,
    // ── People ──
    pub creator: String,
    #[serde(default)]
    pub assignee: String,
    // ── Timestamps ──
    pub created_at: String,
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub base_branch: Option<String>,
    #[serde(default)]
    pub worktree_path: Option<String>,
    #[serde(default)]
    pub commit: Option<String>,
    #[serde(default)]
    pub pr_url: Option<String>,
    // ── Relations ──
    #[serde(default)]
    pub subtasks: Vec<String>,
    #[serde(default)]
    pub children: Vec<String>,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub related_files: Vec<String>,
    // ── Metadata ──
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub meta: Value,

    // ── DiJiang extensions (skipped when None) ──
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acceptance_criteria: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_deliverables: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual_effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_comments: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
}
