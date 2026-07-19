use crate::types::TaskStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteIntent {
    Align,
    Document,
    Implement,
    Debug,
    Check,
    Research,
    Finish,
    Resume,
    Unknown,
}

impl RouteIntent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Align => "align",
            Self::Document => "document",
            Self::Implement => "implement",
            Self::Debug => "debug",
            Self::Check => "check",
            Self::Finish => "finish",
            Self::Resume => "resume",
            Self::Research => "research",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteAction {
    Allow,
    Redirect,
    Block,
}

impl RouteAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Redirect => "redirect",
            Self::Block => "block",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowCapsule {
    Align,
    Implement,
    Check,
    Finish,
    Resume,
    Idle,
}

impl WorkflowCapsule {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Align => "align",
            Self::Implement => "implement",
            Self::Check => "check",
            Self::Finish => "finish",
            Self::Resume => "resume",
            Self::Idle => "idle",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskComplexity {
    Lightweight,
    Complex,
}

impl TaskComplexity {
    pub fn is_lightweight(self) -> bool {
        matches!(self, Self::Lightweight)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteDecision {
    pub task_status: TaskStatus,
    pub capsule: WorkflowCapsule,
    pub requested_intent: RouteIntent,
    pub requested_skill: Option<String>,
    pub resolved_skill: &'static str,
    pub action: RouteAction,
    pub reason: String,
    pub next_action: String,
    pub requires_alignment_artifact: bool,
    pub complexity: TaskComplexity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteGateSummary {
    pub capsule: WorkflowCapsule,
    pub complexity: TaskComplexity,
    pub allowed_skills: Vec<&'static str>,
    pub default_skill: &'static str,
    pub blocked_skills: Vec<&'static str>,
    pub note: String,
}

pub fn evaluate_route(
    task_status: &TaskStatus,
    requested_intent: RouteIntent,
    requested_skill: Option<&str>,
    complexity: Option<TaskComplexity>,
) -> RouteDecision {
    let effective_complexity = complexity.unwrap_or(TaskComplexity::Complex);
    let requested_skill = requested_skill.map(str::to_string);
    let mut result = match task_status {
        TaskStatus::Planning => match requested_intent {
            RouteIntent::Align => {
                let resolved =
                    requested_skill_name(requested_skill.as_deref()).unwrap_or("dj-grill");
                decision(
                    task_status.clone(),
                    WorkflowCapsule::Align,
                    requested_intent,
                    requested_skill,
                    resolved,
                    RouteAction::Allow,
                    "planning tasks must align before implementation",
                    "continue with dj-grill to confirm scope and acceptance",
                    false,
                )
            }
            RouteIntent::Document => decision(
                task_status.clone(),
                WorkflowCapsule::Align,
                requested_intent,
                requested_skill,
                "dj-output",
                RouteAction::Allow,
                "planning tasks may produce task artifacts before implementation",
                "continue with dj-output and keep the work scoped to task artifacts",
                false,
            ),
            RouteIntent::Finish => decision(
                task_status.clone(),
                WorkflowCapsule::Align,
                requested_intent,
                requested_skill,
                "dj-grill",
                RouteAction::Block,
                "planning tasks cannot finish before alignment and implementation complete",
                "run dj-grill first, then progress through output/implementation before finish-work",
                true,
            ),
            RouteIntent::Research => decision(
                task_status.clone(),
                WorkflowCapsule::Align,
                requested_intent,
                requested_skill,
                "dj-research",
                RouteAction::Allow,
                "planning tasks may perform research to support requirement decisions",
                "continue with dj-research to investigate technical questions",
                false,
            ),
RouteIntent::Implement
| RouteIntent::Debug
| RouteIntent::Check
| RouteIntent::Unknown
| RouteIntent::Resume => {
    if effective_complexity.is_lightweight() {
        let resolved = requested_skill_name(requested_skill.as_deref())
            .unwrap_or(match requested_intent {
                RouteIntent::Debug => "dj-hunt",
                RouteIntent::Check => "dj-check",
                _ => "dj-implement",
            });
        decision(
            task_status.clone(),
            WorkflowCapsule::Implement,
            requested_intent,
            requested_skill,
            resolved,
            RouteAction::Allow,
            "lightweight task: prd.md is sufficient before implementation",
            "continue with the requested implementation skill",
            true,
        )
    } else {
        decision(
            task_status.clone(),
            WorkflowCapsule::Align,
            requested_intent,
            requested_skill,
            "dj-grill",
            RouteAction::Redirect,
            "planning tasks are hard-gated to alignment before implementation-oriented work",
            "continue with dj-grill to produce a confirmed requirement summary",
            true,
        )
    }
},
        },
        TaskStatus::InProgress => match requested_intent {
            RouteIntent::Debug => decision(
                task_status.clone(),
                WorkflowCapsule::Implement,
                requested_intent,
                requested_skill,
                "dj-hunt",
                RouteAction::Allow,
                "in_progress tasks may investigate regressions and failures directly",
                "continue with dj-hunt and keep RED/REPRO evidence",
                true,
            ),
            RouteIntent::Check => decision(
                task_status.clone(),
                WorkflowCapsule::Check,
                requested_intent,
                requested_skill,
                "dj-check",
                RouteAction::Allow,
                "in_progress tasks may enter verification directly",
                "continue with dj-check and record validation scope",
                false,
            ),
            RouteIntent::Document => decision(
                task_status.clone(),
                WorkflowCapsule::Implement,
                requested_intent,
                requested_skill,
                "dj-output",
                RouteAction::Allow,
                "in_progress tasks may sync task artifacts while implementation is active",
                "continue with dj-output and sync implementation notes",
                false,
            ),
            RouteIntent::Finish => decision(
                task_status.clone(),
                WorkflowCapsule::Check,
                requested_intent,
                requested_skill,
                "dj-check",
                RouteAction::Redirect,
                "finish-work is gated behind verification for in_progress tasks",
                "run dj-check first, then finish-work once validation is complete",
                false,
            ),
            RouteIntent::Align => {
                let resolved =
                    requested_skill_name(requested_skill.as_deref()).unwrap_or("dj-grill");
                decision(
                    task_status.clone(),
                    WorkflowCapsule::Align,
                    requested_intent,
                    requested_skill,
                    resolved,
                    RouteAction::Allow,
                    "in_progress tasks may re-open alignment when scope changes",
                    "continue with dj-grill to refresh scope and assumptions",
                    false,
                )
            }
            RouteIntent::Research => decision(
                task_status.clone(),
                WorkflowCapsule::Implement,
                requested_intent,
                requested_skill,
                "dj-research",
                RouteAction::Allow,
                "in_progress tasks may research technical questions during implementation",
                "continue with dj-research and record findings in task research/",
                false,
            ),
            RouteIntent::Implement | RouteIntent::Unknown | RouteIntent::Resume => {
                let resolved =
                    requested_skill_name(requested_skill.as_deref()).unwrap_or("dj-implement");
                decision(
                    task_status.clone(),
                    WorkflowCapsule::Implement,
                    requested_intent,
                    requested_skill,
                    resolved,
                    RouteAction::Allow,
                    "in_progress tasks may continue implementation-oriented work",
                    "continue in the implementation lane and keep validation commands current",
                    true,
                )
            }
        },
        TaskStatus::Completed => match requested_intent {
            RouteIntent::Finish => decision(
                task_status.clone(),
                WorkflowCapsule::Finish,
                requested_intent,
                requested_skill,
                "dijiang-finish-work",
                RouteAction::Allow,
                "completed tasks may enter finish-work directly",
                "continue with dijiang-finish-work and archive the session once verified",
                false,
            ),
            RouteIntent::Check => decision(
                task_status.clone(),
                WorkflowCapsule::Check,
                requested_intent,
                requested_skill,
                "dj-check",
                RouteAction::Allow,
                "completed tasks may still run verification or review",
                "continue with dj-check if more validation evidence is needed",
                false,
            ),
            RouteIntent::Document => decision(
                task_status.clone(),
                WorkflowCapsule::Finish,
                requested_intent,
                requested_skill,
                "dj-output",
                RouteAction::Allow,
                "completed tasks may still sync docs before finish-work",
                "continue with dj-output and finalize task artifacts",
                false,
            ),
            RouteIntent::Align => {
                let resolved =
                    requested_skill_name(requested_skill.as_deref()).unwrap_or("dj-grill");
                decision(
                    task_status.clone(),
                    WorkflowCapsule::Align,
                    requested_intent,
                    requested_skill,
                    resolved,
                    RouteAction::Allow,
                    "completed tasks may re-open alignment to decide whether follow-up work belongs here",
                    "continue with dj-grill to decide whether to restart or split follow-up work",
                    false,
                )
            }
            RouteIntent::Implement
            | RouteIntent::Debug
            | RouteIntent::Resume
            | RouteIntent::Unknown => decision(
                task_status.clone(),
                WorkflowCapsule::Align,
                requested_intent,
                requested_skill,
                "dj-grill",
                RouteAction::Redirect,
                "completed tasks should not silently resume implementation without re-alignment",
                "continue with dj-grill to decide whether to reopen or create a follow-up task",
                false,
            ),
            RouteIntent::Research => decision(
                task_status.clone(),
                WorkflowCapsule::Finish,
                requested_intent,
                requested_skill,
                "dj-research",
                RouteAction::Allow,
                "completed tasks may research follow-up questions before archiving",
                "continue with dj-research and capture findings before finish-work",
                false,
            ),
        },
        TaskStatus::Archived => decision(
            task_status.clone(),
            WorkflowCapsule::Idle,
            requested_intent,
            requested_skill,
            "dijiang-start",
            RouteAction::Block,
            "archived tasks are closed and must be explicitly restarted before more work",
            "run dijiang start <task> or create a new task before continuing",
            false,
        ),
        TaskStatus::Paused => match requested_intent {
            RouteIntent::Resume => decision(
                task_status.clone(),
                WorkflowCapsule::Resume,
                requested_intent,
                requested_skill,
                "dijiang-continue",
                RouteAction::Allow,
                "paused tasks must restore context before other work resumes",
                "continue with dijiang-continue to recover task context",
                false,
            ),
            _ => decision(
                task_status.clone(),
                WorkflowCapsule::Resume,
                requested_intent,
                requested_skill,
                "dijiang-continue",
                RouteAction::Redirect,
                "paused tasks must restore context before routing into other skills",
                "continue with dijiang-continue, then re-evaluate the next skill",
                false,
            ),
        },
    };
    result.complexity = effective_complexity;
    result
}

pub fn summarize_route_gate(
    task_status: &TaskStatus,
    complexity: Option<TaskComplexity>,
) -> RouteGateSummary {
    let effective_complexity = complexity.unwrap_or(TaskComplexity::Complex);
    match task_status {
        TaskStatus::Planning => {
            if effective_complexity.is_lightweight() {
                RouteGateSummary {
                    capsule: WorkflowCapsule::Implement,
                    complexity: TaskComplexity::Lightweight,
                    allowed_skills: vec!["dj-grill", "dj-output", "dj-reason", "dj-research", "dj-implement", "dj-script", "dj-tdd", "dj-hunt", "dj-check"],
                    default_skill: "dj-implement",
                    blocked_skills: vec![],
                    note: "lightweight task: prd.md is sufficient before implementation. Implementation skills are directly accessible.".to_string(),
                }
            } else {
                RouteGateSummary {
                    capsule: WorkflowCapsule::Align,
                    complexity: TaskComplexity::Complex,
                    allowed_skills: vec!["dj-grill", "dj-output", "dj-reason", "dj-research"],
                    default_skill: "dj-grill",
                    blocked_skills: vec!["dj-implement", "dj-script", "dj-tdd", "dj-hunt", "dj-check"],
                    note: "complex task: finish prd.md, design.md, and implement.md before implementation. Focus on alignment first.".to_string(),
                }
            }
        },
        TaskStatus::InProgress => RouteGateSummary {
            capsule: WorkflowCapsule::Implement,
            complexity: effective_complexity,
            allowed_skills: vec!["dj-implement", "dj-script", "dj-tdd", "dj-hunt", "dj-check", "dj-output", "dj-grill", "dj-reason", "dj-research"],
            default_skill: "dj-implement",
            blocked_skills: vec!["dijiang-finish-work"],
            note: "in_progress tasks stay in the implementation lane, but finish-work remains gated behind verification.".to_string(),
        },
        TaskStatus::Completed => RouteGateSummary {
            capsule: WorkflowCapsule::Finish,
            complexity: effective_complexity,
            allowed_skills: vec!["dijiang-finish-work", "dj-check", "dj-output", "dj-grill", "dj-reason", "dj-research"],
            default_skill: "dijiang-finish-work",
            blocked_skills: vec!["dj-implement", "dj-script", "dj-tdd", "dj-hunt"],
            note: "completed tasks may finish or document, but implementation requests must re-open alignment first.".to_string(),
        },
        TaskStatus::Archived => RouteGateSummary {
            capsule: WorkflowCapsule::Idle,
            complexity: effective_complexity,
            allowed_skills: vec!["dijiang-start"],
            default_skill: "dijiang-start",
            blocked_skills: vec!["dj-grill", "dj-output", "dj-reason", "dj-implement", "dj-script", "dj-tdd", "dj-hunt", "dj-check", "dijiang-finish-work"],
            note: "archived tasks are closed; the harness blocks further work until the task is explicitly restarted.".to_string(),
        },
        TaskStatus::Paused => RouteGateSummary {
            capsule: WorkflowCapsule::Resume,
            complexity: effective_complexity,
            allowed_skills: vec!["dijiang-continue"],
            default_skill: "dijiang-continue",
            blocked_skills: vec!["dj-grill", "dj-output", "dj-reason", "dj-implement", "dj-script", "dj-tdd", "dj-hunt", "dj-check", "dijiang-finish-work"],
            note: "paused tasks restore context first, then route again once the session has been recovered.".to_string(),
        },
    }
}

fn requested_skill_name(skill: Option<&str>) -> Option<&'static str> {
    match skill {
        Some("dj-implement") => Some("dj-implement"),
        Some("dj-script") => Some("dj-script"),
        Some("dj-tdd") => Some("dj-tdd"),
        Some("dj-hunt") => Some("dj-hunt"),
        Some("dj-check") => Some("dj-check"),
        Some("dj-output") => Some("dj-output"),
        Some("dj-grill") => Some("dj-grill"),
        Some("dj-reason") => Some("dj-reason"),
        Some("dj-research") => Some("dj-research"),
        Some("dijiang-finish-work") => Some("dijiang-finish-work"),
        Some("dijiang-continue") => Some("dijiang-continue"),
        _ => None,
    }
}

fn decision(
    task_status: TaskStatus,
    capsule: WorkflowCapsule,
    requested_intent: RouteIntent,
    requested_skill: Option<String>,
    resolved_skill: &'static str,
    action: RouteAction,
    reason: &str,
    next_action: &str,
    requires_alignment_artifact: bool,
) -> RouteDecision {
    RouteDecision {
        task_status,
        capsule,
        requested_intent,
        requested_skill,
        resolved_skill,
        action,
        reason: reason.to_string(),
        next_action: next_action.to_string(),
        requires_alignment_artifact,
        complexity: TaskComplexity::Complex,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planning_implement_redirects_to_grill() {
        let decision = evaluate_route(
            &TaskStatus::Planning,
            RouteIntent::Implement,
            Some("dj-implement"),
            None,
        );
        assert_eq!(decision.action, RouteAction::Redirect);
        assert_eq!(decision.resolved_skill, "dj-grill");
        assert_eq!(decision.capsule, WorkflowCapsule::Align);
        assert!(decision.requires_alignment_artifact);
        assert!(!decision.reason.is_empty());
        assert!(!decision.next_action.is_empty());
    }

    #[test]
    fn planning_reason_is_allowed() {
        let decision = evaluate_route(&TaskStatus::Planning, RouteIntent::Align, Some("dj-reason"), None);
        assert_eq!(decision.action, RouteAction::Allow);
        assert_eq!(decision.resolved_skill, "dj-reason");
        assert_eq!(decision.capsule, WorkflowCapsule::Align);
    }
    #[test]
    fn planning_script_redirects_to_grill() {
        let decision = evaluate_route(
            &TaskStatus::Planning,
            RouteIntent::Implement,
            Some("dj-script"),
            None,
        );
        assert_eq!(decision.action, RouteAction::Redirect);
        assert_eq!(decision.resolved_skill, "dj-grill");
    }

    #[test]
    fn planning_output_is_allowed() {
        let decision = evaluate_route(
            &TaskStatus::Planning,
            RouteIntent::Document,
            Some("dj-output"),
            None,
        );
        assert_eq!(decision.action, RouteAction::Allow);
        assert_eq!(decision.resolved_skill, "dj-output");
    }

    #[test]
    fn planning_finish_is_blocked() {
        let decision = evaluate_route(
            &TaskStatus::Planning,
            RouteIntent::Finish,
            Some("dijiang-finish-work"),
            None,
        );
        assert_eq!(decision.action, RouteAction::Block);
        assert_eq!(decision.resolved_skill, "dj-grill");
    }

    #[test]
    fn in_progress_implement_is_allowed() {
        let decision = evaluate_route(
            &TaskStatus::InProgress,
            RouteIntent::Implement,
            Some("dj-implement"),
            None,
        );
        assert_eq!(decision.action, RouteAction::Allow);
        assert_eq!(decision.resolved_skill, "dj-implement");
    }

    #[test]
    fn in_progress_finish_redirects_to_check() {
        let decision = evaluate_route(
            &TaskStatus::InProgress,
            RouteIntent::Finish,
            Some("dijiang-finish-work"),
            None,
        );
        assert_eq!(decision.action, RouteAction::Redirect);
        assert_eq!(decision.resolved_skill, "dj-check");
        assert_eq!(decision.capsule, WorkflowCapsule::Check);
    }

    #[test]
    fn completed_finish_is_allowed() {
        let decision = evaluate_route(
            &TaskStatus::Completed,
            RouteIntent::Finish,
            Some("dijiang-finish-work"),
            None,
        );
        assert_eq!(decision.action, RouteAction::Allow);
        assert_eq!(decision.resolved_skill, "dijiang-finish-work");
    }

    #[test]
    fn completed_implement_redirects_to_grill() {
        let decision = evaluate_route(
            &TaskStatus::Completed,
            RouteIntent::Implement,
            Some("dj-implement"),
            None,
        );
        assert_eq!(decision.action, RouteAction::Redirect);
        assert_eq!(decision.resolved_skill, "dj-grill");
    }

    #[test]
    fn archived_blocks_everything() {
        let decision = evaluate_route(
            &TaskStatus::Archived,
            RouteIntent::Implement,
            Some("dj-implement"),
            None,
        );
        assert_eq!(decision.action, RouteAction::Block);
        assert_eq!(decision.resolved_skill, "dijiang-start");
        assert_eq!(decision.capsule, WorkflowCapsule::Idle);
    }

    #[test]
    fn paused_implement_redirects_to_continue() {
        let decision = evaluate_route(
            &TaskStatus::Paused,
            RouteIntent::Implement,
            Some("dj-implement"),
            None,
        );
        assert_eq!(decision.action, RouteAction::Redirect);
        assert_eq!(decision.resolved_skill, "dijiang-continue");
    }

    #[test]
    fn paused_resume_is_allowed() {
        let decision = evaluate_route(
            &TaskStatus::Paused,
            RouteIntent::Resume,
            Some("dijiang-continue"),
            None,
        );
        assert_eq!(decision.action, RouteAction::Allow);
        assert_eq!(decision.resolved_skill, "dijiang-continue");
    }

    #[test]
    fn planning_unknown_redirects_to_grill() {
        let decision = evaluate_route(&TaskStatus::Planning, RouteIntent::Unknown, None, None);
        assert_eq!(decision.action, RouteAction::Redirect);
        assert_eq!(decision.resolved_skill, "dj-grill");
    }

    #[test]
    fn planning_lightweight_implement_is_allowed() {
        let decision = evaluate_route(
            &TaskStatus::Planning,
            RouteIntent::Implement,
            Some("dj-implement"),
            Some(TaskComplexity::Lightweight),
        );
        assert_eq!(decision.action, RouteAction::Allow);
        assert_eq!(decision.resolved_skill, "dj-implement");
        assert_eq!(decision.capsule, WorkflowCapsule::Implement);
    }

    #[test]
    fn planning_lightweight_debug_is_allowed() {
        let decision = evaluate_route(
            &TaskStatus::Planning,
            RouteIntent::Debug,
            None,
            Some(TaskComplexity::Lightweight),
        );
        assert_eq!(decision.action, RouteAction::Allow);
        assert_eq!(decision.resolved_skill, "dj-hunt");
        assert_eq!(decision.capsule, WorkflowCapsule::Implement);
    }

    #[test]
    fn planning_lightweight_check_is_allowed() {
        let decision = evaluate_route(
            &TaskStatus::Planning,
            RouteIntent::Check,
            None,
            Some(TaskComplexity::Lightweight),
        );
        assert_eq!(decision.action, RouteAction::Allow);
        assert_eq!(decision.resolved_skill, "dj-check");
        assert_eq!(decision.capsule, WorkflowCapsule::Implement);
    }

    #[test]
    fn planning_complex_implement_still_redirects() {
        let decision = evaluate_route(
            &TaskStatus::Planning,
            RouteIntent::Implement,
            Some("dj-implement"),
            Some(TaskComplexity::Complex),
        );
        assert_eq!(decision.action, RouteAction::Redirect);
        assert_eq!(decision.resolved_skill, "dj-grill");
        assert_eq!(decision.capsule, WorkflowCapsule::Align);
        assert!(decision.requires_alignment_artifact);
    }
}
