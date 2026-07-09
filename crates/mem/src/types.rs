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
/// Memory errors.
///
/// See module-level documentation for guidance on when to use each variant.
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

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

// ─── L2: Episodic Memory ───────────────────────────────────────────

/// Scope of a memory entry — controls visibility and promotion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryScope {
    /// Visible only within the current project (default).
    Project,
    /// Suitable for promotion to global memory.
    Global,
    /// Sensitive — never promoted, never cross-project.
    Sensitive,
}

impl Default for MemoryScope {
    fn default() -> Self {
        MemoryScope::Project
    }
}

/// A finding recorded during a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub timestamp: String,
    pub content: String,
    pub session_id: Option<String>,
    pub project: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub scope: MemoryScope,
}

/// A session closure written when work finishes successfully.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionClosure {
    pub timestamp: String,
    pub session_key: String,
    pub source: String,
    pub task: String,
    pub summary: String,
    pub verification: String,
    pub docs_sync: String,
    pub version_impact: String,
    pub status: String,
    pub confidence: String,
    pub outcome: Option<String>,
    pub next_tactic: Option<String>,
    pub next_pattern: Option<String>,
    pub loop_signal: Option<String>,
    /// Attempt ledger from circuit breaker tracking.
    /// Empty by default for backward compatibility with older closures.
    #[serde(default)]
    pub attempts: Vec<AttemptEntry>,
}

/// A single attempt recorded in the session closure ledger.
///
/// This is the L2 episodic record of what was tried during a loop,
/// enabling `mem evolve` to consume precise iteration data instead
/// of relying on the 50-char finding truncation method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttemptEntry {
    /// Iteration number (1-based).
    pub iteration: u64,
    /// What action was attempted.
    pub action: String,
    /// Outcome of this attempt.
    pub outcome: AttemptOutcome,
    /// Error message (if failure).
    #[serde(default)]
    pub error: Option<String>,
    /// Tokens consumed (if tracked).
    #[serde(default)]
    pub tokens_used: Option<u64>,
}

/// Outcome of a single attempt in the session ledger.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AttemptOutcome {
    Success,
    Failure,
    Noop,
}

/// A user correction that should change future agent behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Correction {
    pub timestamp: String,
    pub session_key: Option<String>,
    pub task: Option<String>,
    pub source: String,
    pub correction: String,
    pub lesson: String,
    pub scope: String,
    pub confidence: String,
    pub freshness: String,
    pub conflict: String,
    pub actionability: String,
}

/// A lesson learned during a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Learning {
    pub timestamp: String,
    pub content: String,
    pub session_id: Option<String>,
    pub project: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub scope: MemoryScope,
}

// ─── L3: Semantic Memory (Tactics) ─────────────────────────────────

/// A tactic with Bayesian tracking (Beta distribution).
///
/// Each tactic tracks success/failure counts and computes win rate.
/// Used by Thompson sampling for strategy selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tactic {
    pub name: String,
    pub description: String,
    pub alpha: u32,     // wins + 1
    pub beta: u32,      // losses + 1
    pub source: String, // project or session that created this
    pub created_at: String,
    pub last_used: Option<String>,
}

impl Tactic {
    pub fn new(name: &str, description: &str, source: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            alpha: 1,
            beta: 1,
            source: source.to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            last_used: None,
        }
    }

    pub fn win_rate(&self) -> f64 {
        self.alpha as f64 / (self.alpha + self.beta) as f64
    }

    pub fn record_success(&mut self) {
        self.alpha += 1;
        self.last_used = Some(chrono::Local::now().to_rfc3339());
    }

    pub fn record_failure(&mut self) {
        self.beta += 1;
        self.last_used = Some(chrono::Local::now().to_rfc3339());
    }
}

/// An event in the ledger (success or failure).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub timestamp: String,
    pub tactic_name: String,
    pub outcome: Outcome,
    pub context: String,
    pub project: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Outcome {
    Success,
    Failure,
}

// ─── L4: Procedural Memory (Patterns/SOPs) ─────────────────────────

/// A standard operating procedure or workflow pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub name: String,
    pub description: String,
    pub steps: Vec<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub project: Option<String>, // None = global
    /// How often this pattern runs (e.g. "1d", "15m", "6h")
    #[serde(default)]
    pub cadence: Option<String>,
    /// Risk level: "low", "medium", "high"
    #[serde(default)]
    pub risk: Option<String>,
    /// Week-1 rollout mode: "L1", "L2"
    #[serde(default)]
    pub week_one_mode: Option<String>,
    /// Estimated token cost: "low", "medium", "high"
    #[serde(default)]
    pub token_cost: Option<String>,
    /// Scenarios requiring human approval
    #[serde(default)]
    pub human_gates: Vec<String>,
    /// Phases in this pattern: discover, triage, fix, verify, notify
    #[serde(default)]
    pub phases: Vec<String>,
}

// ─── L5: Meta Memory ──────────────────────────────────────────────

/// Memory statistics.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryStats {
    pub total_findings: u64,
    pub total_learnings: u64,
    pub total_corrections: u64,
    pub total_tactics: u64,
    pub total_patterns: u64,
    pub total_sessions: u64,
    pub avg_tactic_win_rate: f64,
    pub last_evolution: Option<String>,
    pub last_finetune: Option<String>,
}

/// Evolution state tracking.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvolutionState {
    pub tactic_count: u64,
    pub events_since_last_evolve: u64,
    pub evolve_threshold: u64, // default: 3
    pub last_evolution: Option<String>,
}

/// Baseline evaluation for ratchet gate.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Baseline {
    pub pass_rate: f64,
    pub total_tests: u64,
    pub regressions: u64,
    pub timestamp: String,
}

/// A scored memory result from recall.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredMemory {
    pub source: String,       // "findings" / "learnings" / "patterns"
    pub content: String,
    pub score: f64,
    pub timestamp: String,
}

/// An entry in the inverted search index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub term: String,
    pub source: String,
    pub content: String,
    pub timestamp: String,
    pub line: usize,
}

/// The inverted index structure stored as JSON.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchIndex {
    pub entries: Vec<IndexEntry>,
}

/// Report from a prune operation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PruneReport {
    pub findings_before: usize,
    pub learnings_before: usize,
}
