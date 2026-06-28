use serde::{Deserialize, Serialize};
use std::fmt;

/// Session status: active or archived.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    #[default]
    Active,
    Archived,
}

impl fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionStatus::Active => write!(f, "active"),
            SessionStatus::Archived => write!(f, "archived"),
        }
    }
}

/// A session record from any provider.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionRecord {
    pub session_id: String,
    pub project_id: String,
    pub workspace_key: Option<String>,
    pub workspace_path: Option<String>,
    pub status: SessionStatus,
    pub task: Option<String>,
    pub phase: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub action_count: u32,
    pub summary: Option<String>,
    pub provider: String,
    pub source_path: Option<String>,
}

/// Dialogue entry for conversational context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueEntry {
    pub session_id: String,
    pub timestamp: String,
    pub role: String,
    pub content: String,
}

/// A project aggregation — sessions grouped by project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSessions {
    pub project_id: String,
    pub sessions: Vec<SessionRecord>,
    pub last_active_at: Option<String>,
}

/// Aggregation result from all adapters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMem {
    pub projects: Vec<ProjectSessions>,
    pub total_sessions: usize,
    pub providers: Vec<String>,
}

/// Memory errors.
#[derive(Debug, thiserror::Error)]
pub enum MemError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Session not found: {0}")]
    NotFound(String),

    #[error("Provider error: {0}")]
    Provider(String),
}
