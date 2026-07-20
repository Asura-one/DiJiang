use serde::Serialize;

// ── Agent Manifest ───────────────────────────────────────────────────
//
// Agents are persona-level definitions ("who you are + how you think")
// that complement skills ("what to do + rules"). The active agent is
// resolved at workflow-build time from task metadata + route gate and
// injected into the AI session as a persona overlay.

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentManifestEntry {
    pub name: &'static str,
    pub summary: &'static str,
    pub body: &'static str,
}

include!("agent_manifest.gen.rs");

/// Look up an agent manifest entry by name.
pub fn agent_by_name(name: &str) -> Option<&'static AgentManifestEntry> {
    AGENT_MANIFESTS.iter().find(|entry| entry.name == name)
}

/// Get the agent body Markdown by agent name.
pub fn agent_body_by_name(name: &str) -> Option<&'static str> {
    agent_by_name(name).map(|entry| entry.body)
}

/// Return the sorted list of all agent names.
pub fn all_agent_names() -> Vec<&'static str> {
    let mut names: Vec<&str> = AGENT_MANIFESTS.iter().map(|e| e.name).collect();
    names.sort();
    names
}

// ── Agent Resolution ─────────────────────────────────────────────────
//
/// Resolve the active agent from task-type intent and current capsule.
///
/// Heuristic (ordered):
/// 1. `task_type` == "research" / "调研" → researcher
/// 2. `primary_intent` == "architecture" / "架构评审" → architect
/// 3. capsule == "align" → planner
/// 4. capsule == "implement" → implementer
/// 5. capsule == "check" | "finish" → checker
/// 6. fallback → implementer (the most common persona)
pub fn resolve_agent(
    task_type: Option<&str>,
    primary_intent: Option<&str>,
    capsule: &str,
) -> &'static str {
    // 1. Research tasks → researcher
    if let Some(tt) = task_type {
        let tt = tt.trim().to_lowercase();
        if tt == "research" || tt == "调研" || tt == "调研对齐" || tt == "调研/设计" {
            return "researcher";
        }
    }

    // 2. Architecture intent → architect
    if let Some(intent) = primary_intent {
        let intent_lower = intent.trim().to_lowercase();
        if intent_lower.contains("architecture")
            || intent_lower.contains("架构")
            || intent_lower.contains("arch")
        {
            return "architect";
        }
    }

    // 3-6. Capsule-based routing
    match capsule {
        "align" => "planner",
        "implement" | "idle" => "implementer",
        "check" => "checker",
        "finish" => "checker",
        "resume" => "planner",
        _ => "implementer",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_lookup_returns_known_agents() {
        let planner = agent_by_name("planner").expect("planner agent");
        assert_eq!(planner.name, "planner");
        assert!(planner.body.contains("Planner"));
        assert!(planner.body.contains("Persona"));

        let implementer = agent_by_name("implementer").expect("implementer agent");
        assert_eq!(implementer.name, "implementer");
        assert!(implementer.body.contains("Implementer"));

        let checker = agent_by_name("checker").expect("checker agent");
        assert_eq!(checker.name, "checker");
        assert!(checker.body.contains("Checker"));

        let architect = agent_by_name("architect").expect("architect agent");
        assert_eq!(architect.name, "architect");
        assert!(architect.body.contains("Architect"));

        let researcher = agent_by_name("researcher").expect("researcher agent");
        assert_eq!(researcher.name, "researcher");
        assert!(researcher.body.contains("Researcher"));
    }

    #[test]
    fn agent_body_by_name_returns_body() {
        let body = agent_body_by_name("planner").expect("body");
        assert!(body.contains("# Planner"));
        assert!(body.contains("## Operating Persona"));
    }

    #[test]
    fn all_agent_names_returns_sorted() {
        let names = all_agent_names();
        assert_eq!(names.len(), 5);
        assert_eq!(names[0], "architect");
        assert_eq!(names[1], "checker");
        assert_eq!(names[2], "implementer");
        assert_eq!(names[3], "planner");
        assert_eq!(names[4], "researcher");
    }

    #[test]
    fn resolve_agent_returns_researcher_for_research_task() {
        assert_eq!(
            resolve_agent(Some("research"), None, "align"),
            "researcher"
        );
        assert_eq!(
            resolve_agent(Some("调研"), None, "align"),
            "researcher"
        );
        assert_eq!(
            resolve_agent(Some("调研对齐"), None, "align"),
            "researcher"
        );
    }

    #[test]
    fn resolve_agent_returns_architect_for_architecture_intent() {
        assert_eq!(
            resolve_agent(None, Some("architecture review"), "implement"),
            "architect"
        );
        assert_eq!(
            resolve_agent(None, Some("架构评审"), "implement"),
            "architect"
        );
    }

    #[test]
    fn resolve_agent_returns_planner_for_align_capsule() {
        assert_eq!(resolve_agent(None, None, "align"), "planner");
    }

    #[test]
    fn resolve_agent_returns_implementer_for_implement_capsule() {
        assert_eq!(resolve_agent(None, None, "implement"), "implementer");
    }

    #[test]
    fn resolve_agent_returns_checker_for_check_and_finish() {
        assert_eq!(resolve_agent(None, None, "check"), "checker");
        assert_eq!(resolve_agent(None, None, "finish"), "checker");
    }

    #[test]
    fn resolve_agent_returns_implementer_for_unknown_capsule() {
        assert_eq!(resolve_agent(None, None, "unknown"), "implementer");
    }
}
