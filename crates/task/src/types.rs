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

    /// Map DiJiang task status to a Trellis-readable status string.
    ///
    /// DiJiang has 5 status variants; Trellis's `inferTaskPhase` recognises
    /// only 4 (`planning`/`in_progress`/`review`/`completed`). The two extra
    /// states (`Archived`, `Paused`) are downgraded so JSON emitted by DiJiang
    /// remains interpretable by Trellis tools. The original status is preserved
    /// in `meta.original_status` by the caller.
    pub fn to_trellis_status(&self) -> &'static str {
        match self {
            TaskStatus::Planning => "plan",
            TaskStatus::InProgress => "implement",
            TaskStatus::Completed => "complete",
            TaskStatus::Archived => "complete",
            TaskStatus::Paused => "in_progress",
        }
    }
    /// Parse a status string. Unknown values fall back to [`TaskStatus::Paused`]
    /// (the most conservative DiJiang state) and the raw input is returned via
    /// the second tuple element. This keeps reads of Trellis task.json files
    /// forward-compatible with Trellis status values DiJiang does not know about.
    pub fn from_str_lossy(s: &str) -> (Self, Option<String>) {
        match s {
            "planning" => (Self::Planning, None),
            "in_progress" => (Self::InProgress, None),
            "completed" | "done" => (Self::Completed, None),
            "review" => (Self::Completed, None),
            "archived" => (Self::Archived, None),
            "paused" => (Self::Paused, None),
            other => (Self::Paused, Some(other.to_string())),
        }
    }

    pub fn infer_phase(&self) -> &'static str {
        match self {
            TaskStatus::Planning => "plan",
            TaskStatus::InProgress | TaskStatus::Paused => "implement",
            TaskStatus::Completed => "complete",
            TaskStatus::Archived => "archive",
        }
    }
}

/// Canonical field order for the `task.json` file. Must match the 24-field
/// `TASK_RECORD_FIELD_ORDER` defined in Trellis's `packages/core/src/task/schema.ts`
/// and the `TaskData` TypedDict in Trellis's `scripts/common/types.py`. Field order is
/// load-bearing for Trellis interop: any reordering or insertion will break consumers.
pub const TASK_RECORD_FIELD_ORDER: &[&str] = &[
    "id",
    "name",
    "title",
    "description",
    "status",
    "devType",
    "scope",
    "package",
    "priority",
    "creator",
    "assignee",
    "createdAt",
    "completedAt",
    "branch",
    "baseBranch",
    "worktreePath",
    "commit",
    "prUrl",
    "subtasks",
    "children",
    "parent",
    "relatedFiles",
    "notes",
    "meta",
];

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
