/// Template engine for DiJiang configurator.
///
/// Templates are embedded at compile time via `rust-embed` and
/// support variable substitution with `{{key}}` syntax.
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "templates/"]
#[include = "*.md"]
#[include = "*.ts"]
#[include = "*.json"]
#[include = "*.toml"]
#[include = "*.yml"]
#[include = "*.yaml"]
struct TemplateAssets;

/// Simple variable substitution: replaces `{{key}}` with `value`.
fn substitute(template: &str, vars: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}

/// Render optional (if) blocks: `{{#if var == "value"}}...{{/if}}`
/// and truthy checks: `{{#if var}}...{{/if}}`.
/// Supports `{{else}}` inside if-blocks.
fn render_if_blocks(template: &str, vars: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    // Build a lookup from the vars slice
    let lookup: std::collections::HashMap<&str, &str> = vars.iter().cloned().collect();

    // Process {{#if ...}}...{{/if}} blocks
    // We iterate until no more matches (handles nested blocks from outside in)
    loop {
        let start = match result.find("{{#if ") {
            Some(p) => p,
            None => break,
        };

        let end = match result[start..].find("{{/if}}") {
            Some(p) => start + p,
            None => break,
        };
        let end_tag_len = 7; // {{/if}}
        let block = &result[start..end + end_tag_len].to_string();

        // Extract condition: "{{#if var == \"value\"}}" or "{{#if var}}"
        let cond_start = "{{#if ".len();
        let cond_end = match block[cond_start..].find("}}") {
            Some(p) => cond_start + p,
            None => continue,
        };
        let condition = &block[cond_start..cond_end];

        // Parse condition: "var" or "var == value" or "var != value"
        let (var_name, expected_value, negated) = parse_condition(condition);

        // Find the {{else}} separator if it exists
        let block_body = &block[cond_end + 2..block.len() - end_tag_len]; // after }} and before {{/if}}
        let else_block = "{{else}}";
        let (if_content, else_content) = if let Some(else_pos) = block_body.find(else_block) {
            (
                &block_body[..else_pos],
                &block_body[else_pos + else_block.len()..],
            )
        } else {
            (block_body, "")
        };

        // Evaluate condition
        let actual_value = lookup.get(var_name).copied().unwrap_or("");
        let condition_met = if let Some(expected) = expected_value {
            if negated {
                actual_value != expected
            } else {
                actual_value == expected
            }
        } else {
            // Truthy check
            let met = !actual_value.is_empty();
            if negated {
                !met
            } else {
                met
            }
        };

        let replacement = if condition_met {
            if_content
        } else {
            else_content
        };
        // Recursively process nested if-blocks
        let processed = render_if_blocks(replacement, vars);
        result.replace_range(start..end + end_tag_len, &processed);
    }
    result
}

/// Parse an if-condition string like `"var == value"` or `"var"` or `"var != value"`.
/// Returns (variable_name, expected_value, is_negated).
fn parse_condition(s: &str) -> (&str, Option<&str>, bool) {
    let s = s.trim();
    if let Some(pos) = s.find(" != ") {
        let var = s[..pos].trim();
        let val = s[pos + 4..].trim();
        let val = val
            .strip_prefix('\"')
            .and_then(|v| v.strip_suffix('\"'))
            .unwrap_or(val);
        (var, Some(val), true)
    } else if let Some(pos) = s.find(" == ") {
        let var = s[..pos].trim();
        let val = s[pos + 4..].trim();
        let val = val
            .strip_prefix('\"')
            .and_then(|v| v.strip_suffix('\"'))
            .unwrap_or(val);
        (var, Some(val), false)
    } else {
        // Truthy check — if starts with !, negate
        if let Some(var) = s.strip_prefix('!') {
            (var.trim(), None, true)
        } else {
            (s, None, false)
        }
    }
}

/// Load a template file and substitute variables.
pub fn render(path: &str, vars: &[(&str, &str)]) -> Result<String, String> {
    let asset = TemplateAssets::get(path).ok_or_else(|| format!("Template not found: {path}"))?;
    let content = std::str::from_utf8(asset.data.as_ref())
        .map_err(|e| format!("Template {path} is not valid UTF-8: {e}"))?;
    let rendered = render_if_blocks(content, vars);
    Ok(substitute(&rendered, vars))
}

/// Check if a template exists.
pub fn exists(path: &str) -> bool {
    TemplateAssets::get(path).is_some()
}

/// List available built-in template packages.
pub fn list_builtin_packages() -> Vec<String> {
    let mut packages: Vec<String> = Vec::new();
    for path in TemplateAssets::iter() {
        if let Some(rest) = path.strip_prefix("packages/") {
            if let Some(pkg_name) = rest.split('/').next() {
                if !packages.contains(&pkg_name.to_string()) {
                    packages.push(pkg_name.to_string());
                }
            }
        }
    }
    packages.sort();
    packages
}

/// Get a built-in template package file by package name and relative path.
pub fn get_builtin_package_file(package: &str, file_path: &str) -> Option<String> {
    let full_path = format!("packages/{}/{}", package, file_path);
    let asset = TemplateAssets::get(&full_path)?;
    let content = std::str::from_utf8(asset.data.as_ref()).ok()?;
    Some(content.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_skill_template() {
        let content = render("skills/dijiang-start/SKILL.md", &[]).unwrap();
        assert!(content.contains("dj-dispatch"));
        assert!(content.contains("交接给 dj-dispatch"));
    }

    #[test]
    fn test_load_agent_template() {
        let content = render("agents/dijiang-implement.md", &[]).unwrap();
        assert!(content.contains("dj-implement"));
        assert!(content.contains("dijiang workflow-state --json"));
        assert!(content.contains("<dijiang-target-skill ...>"));
    }

    #[test]
    fn test_substitution() {
        let content = render(
            "skills/dijiang-continue/SKILL.md",
            &[("developer", "tiezhu")],
        )
        .unwrap();
        assert!(content.contains("tiezhu"));
    }

    #[test]
    fn test_template_not_found() {
        let result = render("nonexistent.md", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_workflow_template() {
        let content = render("config/workflow.md", &[]).unwrap();
        assert!(content.contains("DiJiang 规范工作流"));
        assert!(content.contains("dj-dispatch"));
        assert!(!content.contains("dj-muse"));
    }

    #[test]
    fn test_code_task_tdd_contract_templates() {
        let workflow = render("config/workflow.md", &[]).unwrap();
        assert_code_task_tdd_contract(&workflow);

        for template in [
            "skills/dj-dispatch/SKILL.md",
            "skills/dj-implement/SKILL.md",
            "skills/dj-hunt/SKILL.md",
            "skills/dj-check/SKILL.md",
            "skills/dijiang-finish-work/SKILL.md",
        ] {
            let content = render(template, &[]).unwrap();
            assert_code_task_tdd_contract(&content);
        }
    }
    #[test]
    fn test_finish_work_requires_worktree_residue_decision() {
        let content = render("skills/dijiang-finish-work/SKILL.md", &[]).unwrap();
        for required in [
            "worktree 残留",
            "默认使用 `--integrate`",
            "残留 worktree 检查",
            "git worktree list",
        ] {
            assert!(
                content.contains(required),
                "finish-work template missing worktree residue guard: {required}"
            );
        }
    }

    fn assert_code_task_tdd_contract(content: &str) {
        for required in [
            "Code Task TDD Contract",
            "RED/Repro evidence",
            "GREEN command",
            "Regression scope",
            "Exception",
        ] {
            assert!(
                content.contains(required),
                "template missing TDD contract marker: {required}"
            );
        }
    }

    #[test]
    fn test_conditional_true() {
        let template = "Hello {{#if show}}World{{/if}}!";
        let result = render_if_blocks(template, &[("show", "yes")]);
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_conditional_false() {
        let template = "Hello {{#if show}}World{{/if}}!";
        let result = render_if_blocks(template, &[("show", "")]);
        assert_eq!(result, "Hello !");
    }

    #[test]
    fn test_conditional_eq() {
        let template = "{{#if lang == \"rust\"}}Rust lang{{/if}}";
        let result = render_if_blocks(template, &[("lang", "rust")]);
        assert_eq!(result, "Rust lang");
    }

    #[test]
    fn test_conditional_neq() {
        let template = "{{#if lang != \"python\"}}Not Python{{/if}}";
        let result = render_if_blocks(template, &[("lang", "rust")]);
        assert_eq!(result, "Not Python");
    }

    #[test]
    fn test_conditional_else() {
        let template = "{{#if show}}A{{else}}B{{/if}}";
        let result = render_if_blocks(template, &[("show", "")]);
        assert_eq!(result, "B");
    }

    #[test]
    fn test_conditional_substitution() {
        let template = "{{#if lang == \"rust\"}}Selected: {{lang}}{{/if}}";
        let mut result = render_if_blocks(template, &[("lang", "rust")]);
        result = substitute(&result, &[("lang", "rust")]);
        assert_eq!(result, "Selected: rust");
    }

    #[test]
    fn test_parse_condition_eq() {
        let (var, val, neg) = parse_condition("lang == \"rust\"");
        assert_eq!(var, "lang");
        assert_eq!(val.unwrap(), "rust");
        assert!(!neg);
    }

    #[test]
    fn test_parse_condition_neq() {
        let (var, val, neg) = parse_condition("lang != \"python\"");
        assert_eq!(var, "lang");
        assert_eq!(val.unwrap(), "python");
        assert!(neg);
    }

    #[test]
    fn test_parse_condition_truthy() {
        let (var, val, neg) = parse_condition("show");
        assert_eq!(var, "show");
        assert!(val.is_none());
        assert!(!neg);
    }

    #[test]
    fn test_parse_condition_negated() {
        let (var, val, neg) = parse_condition("!show");
        assert_eq!(var, "show");
        assert!(val.is_none());
        assert!(neg);
    }
}
