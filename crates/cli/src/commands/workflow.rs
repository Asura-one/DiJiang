use crate::util::require_dijiang_dir;

pub fn cmd_workflow_state(json: bool, hook_event: &str) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    let state = dijiang_task::workflow_state::build(&dijiang_dir)?;

    if json {
        let payload = serde_json::json!({
            "hookEventName": hook_event,
            "additionalContext": state.additional_context(),
        });
        println!("{}", serde_json::to_string(&payload)?);
    } else {
        println!("{}", state.additional_context());
    }

    Ok(())
}

pub fn cmd_skill_body(name: &str, json: bool) -> anyhow::Result<()> {
    let mut cache = dijiang_task::SkillBodyCache::default();
    let body = cache
        .body(name)
        .ok_or_else(|| anyhow::anyhow!("unknown skill body: {name}"))?;
    let summary = dijiang_task::manifest_by_name(name)
        .map(|manifest| manifest.summary)
        .unwrap_or("target skill body not registered");
    if json {
        let payload = serde_json::json!({
            "name": name,
            "summary": summary,
            "body": body,
        });
        println!("{}", serde_json::to_string(&payload)?);
    } else {
        println!(
            "<dijiang-target-skill name=\"{}\">\nsummary: {}\n\n{}\n</dijiang-target-skill>",
            name, summary, body
        );
    }
    Ok(())
}
