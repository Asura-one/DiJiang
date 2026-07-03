use crate::route_gate::WorkflowCapsule;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityAction {
    Allow,
    Block,
}

impl CapabilityAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Block => "block",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityTarget {
    FinishIntegrate,
    FinishPush,
    FinishCleanup,
}

impl CapabilityTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FinishIntegrate => "finish_integrate",
            Self::FinishPush => "finish_push",
            Self::FinishCleanup => "finish_cleanup",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityDecision {
    pub capsule: WorkflowCapsule,
    pub target: CapabilityTarget,
    pub action: CapabilityAction,
    pub reason: String,
    pub next_action: String,
}

pub fn evaluate_capability(
    capsule: WorkflowCapsule,
    target: CapabilityTarget,
    approved: bool,
) -> CapabilityDecision {
    match target {
        CapabilityTarget::FinishIntegrate => {
            if approved {
                CapabilityDecision {
                    capsule,
                    target,
                    action: CapabilityAction::Allow,
                    reason: "explicit approval present for finish-work integration".to_string(),
                    next_action: "continue with finish-work integration".to_string(),
                }
            } else {
                CapabilityDecision {
                    capsule,
                    target,
                    action: CapabilityAction::Block,
                    reason: "finish-work integration is high-risk and requires explicit approval".to_string(),
                    next_action: "re-run with --approve-integrate once merge/worktree cleanup is intended".to_string(),
                }
            }
        }
        CapabilityTarget::FinishPush => {
            if approved {
                CapabilityDecision {
                    capsule,
                    target,
                    action: CapabilityAction::Allow,
                    reason: "explicit approval present for finish-work push".to_string(),
                    next_action: "continue with finish-work push".to_string(),
                }
            } else {
                CapabilityDecision {
                    capsule,
                    target,
                    action: CapabilityAction::Block,
                    reason: "finish-work push is high-risk and requires explicit approval".to_string(),
                    next_action: "re-run with --approve-integrate once pushing the task branch is intended".to_string(),
                }
            }
        }
        CapabilityTarget::FinishCleanup => {
            if approved {
                CapabilityDecision {
                    capsule,
                    target,
                    action: CapabilityAction::Allow,
                    reason: "explicit approval present for finish-work cleanup".to_string(),
                    next_action: "continue with worktree/branch cleanup".to_string(),
                }
            } else {
                CapabilityDecision {
                    capsule,
                    target,
                    action: CapabilityAction::Block,
                    reason: "finish-work cleanup is high-risk and requires explicit approval".to_string(),
                    next_action: "re-run with --approve-cleanup once deleting worktree/branch is intended".to_string(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integrate_requires_explicit_approval() {
        let decision = evaluate_capability(
            WorkflowCapsule::Finish,
            CapabilityTarget::FinishIntegrate,
            false,
        );
        assert_eq!(decision.action, CapabilityAction::Block);
        assert!(decision.reason.contains("requires explicit approval"));
    }

    #[test]
    fn integrate_allows_with_explicit_approval() {
        let decision = evaluate_capability(
            WorkflowCapsule::Finish,
            CapabilityTarget::FinishIntegrate,
            true,
        );
        assert_eq!(decision.action, CapabilityAction::Allow);
        assert!(decision.reason.contains("explicit approval present"));
    }

    #[test]
    fn push_requires_explicit_approval() {
        let decision = evaluate_capability(
            WorkflowCapsule::Finish,
            CapabilityTarget::FinishPush,
            false,
        );
        assert_eq!(decision.action, CapabilityAction::Block);
        assert!(decision.reason.contains("push is high-risk"));
    }

    #[test]
    fn cleanup_requires_explicit_approval() {
        let decision = evaluate_capability(
            WorkflowCapsule::Finish,
            CapabilityTarget::FinishCleanup,
            false,
        );
        assert_eq!(decision.action, CapabilityAction::Block);
        assert!(decision.reason.contains("cleanup is high-risk"));
    }
}
