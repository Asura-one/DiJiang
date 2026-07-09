pub mod capability_gate;
pub mod circuit_breaker;
pub mod doc_sync;
pub mod git_gate;
pub mod route_gate;
pub mod skill_manifest;
pub mod spec_sync;
pub mod store;
pub mod types;
pub mod workflow_state;

pub use capability_gate::{
    CapabilityAction, CapabilityDecision, CapabilityTarget, evaluate_capability,
};
pub use circuit_breaker::{
    Attempt, AttemptOutcome, BreakerDecision, BreakerTrigger, CircuitBreakerConfig, Ledger,
    PruneConfig, build_context_injection, check_circuit_breaker, error_signature, prune_ledger,
    summarize_attempts,
};
pub use git_gate::{
    GitGateInput, GitGateState, GitGateSummary, GitRuntimeLocation, WorktreeReadiness,
    evaluate_worktree_readiness, summarize_git_gate, worktree_readiness,
};
pub use route_gate::{
    RouteAction, RouteDecision, RouteGateSummary, RouteIntent, WorkflowCapsule, evaluate_route,
    summarize_route_gate,
};
pub use skill_manifest::{
    SelectedSkillBody, SkillBodyCache, SkillManifestEntry, manifest_by_name, manifests_for_capsule,
    render_selected_skill_bodies, select_skill_bodies, skill_body_by_name,
};
pub use types::{TASK_RECORD_FIELD_ORDER, TaskRecord, TaskStatus};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    /// §1.1: Pin the JSON key order of a fully-populated TaskRecord so it
    /// matches Trellis's `TASK_RECORD_FIELD_ORDER`. Any reordering of
    /// `TaskRecord`'s field declarations will fail this test.
    #[test]
    fn task_record_field_order_matches_trellis() {
        let record = TaskRecord {
            // 24 Trellis fields in fixed order
            id: "id".into(),
            name: "name".into(),
            title: "title".into(),
            description: "description".into(),
            status: TaskStatus::Planning,
            dev_type: Some("dev_type".to_string()),
            scope: Some("scope".to_string()),
            package: Some("package".to_string()),
            priority: "priority".into(),
            creator: "creator".into(),
            assignee: "assignee".into(),
            created_at: "createdAt".into(),
            completed_at: Some("completedAt".to_string()),
            branch: Some("branch".to_string()),
            base_branch: Some("base_branch".to_string()),
            worktree_path: Some("worktree_path".to_string()),
            commit: Some("commit".to_string()),
            pr_url: Some("pr_url".to_string()),
            subtasks: vec!["subtasks".into()],
            children: vec!["children".into()],
            parent: Some("parent".to_string()),
            related_files: vec!["relatedFiles".into()],
            notes: "notes".to_string(),
            meta: serde_json::json!({}),
            // DiJiang extension fields (skip_serializing_if = None -> omitted)
            started_at: None,
            archived_at: None,
            acceptance_criteria: None,
            key_deliverables: None,
            source: None,
            session_id: None,
            session_summary: None,
            // DiJiang extension fields (skip_serializing_if = None -> omitted)
            estimated_effort: None,
            actual_effort: None,
            review_status: None,
            review_comments: None,
            tags: None,
        };
        let json = serde_json::to_string(&record).expect("serialize");
        let value: Value = serde_json::from_str(&json).expect("parse");
        let obj = value.as_object().expect("object");
        let actual: Vec<&str> = obj.keys().map(String::as_str).collect();
        assert_eq!(
            actual, TASK_RECORD_FIELD_ORDER,
            "TaskRecord field order diverged from TASK_RECORD_FIELD_ORDER; \
             update TASK_RECORD_FIELD_ORDER or reorder the struct fields"
        );
    }

    /// §1.3: Every DiJiang status variant must map to a Trellis-readable status
    /// string. Archived and Paused are downgraded to the closest Trellis state.
    #[test]
    fn to_trellis_status_covers_all_variants() {
        assert_eq!(TaskStatus::Planning.to_trellis_status(), "plan");
        assert_eq!(TaskStatus::InProgress.to_trellis_status(), "implement");
        assert_eq!(TaskStatus::Completed.to_trellis_status(), "complete");
        assert_eq!(TaskStatus::Archived.to_trellis_status(), "complete");
        assert_eq!(TaskStatus::Paused.to_trellis_status(), "in_progress");
    }

    /// §3.1: Unknown status strings (forward-compat with future Trellis statuses)
    /// fall back to Paused, with the raw value returned for caller-side recording
    /// in `meta.unmapped_status`.
    #[test]
    fn from_str_lossy_handles_known_and_unknown() {
        let (s, raw) = TaskStatus::from_str_lossy("in_progress");
        assert_eq!(s, TaskStatus::InProgress);
        assert!(raw.is_none());

        let (s, raw) = TaskStatus::from_str_lossy("blocked");
        assert_eq!(s, TaskStatus::Paused);
        assert_eq!(raw.as_deref(), Some("blocked"));

        let (s, raw) = TaskStatus::from_str_lossy("cancelled");
        assert_eq!(s, TaskStatus::Paused);
        assert_eq!(raw.as_deref(), Some("cancelled"));
    }
}
