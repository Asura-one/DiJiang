/// Parse `[workflow-state:TAG]...[/workflow-state:TAG]` blocks from a
/// Markdown file (typically `.dijiang/workflow.md`).
///
/// This implements the Trellis "workflow.md is the runtime state machine"
/// pattern: the workflow document is the single source of truth for
/// per-turn breadcrumb text. When the file cannot be read or parsed,
/// lookups return `None` so the caller can fall back to hardcoded guidance.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Map from status tag (e.g. "planning", "in_progress") to extracted
/// tag body text (the trimmed content between opening and closing tags).
pub type WorkflowTagMap = HashMap<String, String>;

/// Extract all `[workflow-state:STATUS]...[/workflow-state:STATUS]`
/// blocks from a markdown file and return them as a map.
///
/// The opening marker is `[workflow-state:<tag>]` (case-sensitive).
/// The closing marker is `[/workflow-state:<tag>]` (same tag).
/// Inner content is the text between them, trimmed, with leading `//`
/// comments stripped.
///
/// If the file cannot be read, returns an empty map (graceful
/// degradation).
pub fn parse_workflow_tags(path: &Path) -> WorkflowTagMap {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return WorkflowTagMap::new(),
    };

    parse_tags_from_str(&content)
}

/// Parse tags from a string (testable core).
fn parse_tags_from_str(content: &str) -> WorkflowTagMap {
    let mut map = WorkflowTagMap::new();

    // Pattern: [workflow-state:<tag>]...[/workflow-state:<tag>]
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(tag) = trimmed
            .strip_prefix("[workflow-state:")
            .and_then(|rest| rest.strip_suffix(']'))
        {
            // Skip empty tags (they're just markers)
            if !tag.is_empty() && !tag.contains(' ') && !tag.contains('/') {
                let body = extract_tag_body(content, tag);
                if !body.is_empty() {
                    map.insert(tag.to_string(), body);
                }
            }
        }
    }

    map
}

/// Extract the body between `[workflow-state:<tag>]` and
/// `[/workflow-state:<tag>]`.
fn extract_tag_body(full_text: &str, tag: &str) -> String {
    let open_marker = format!("[workflow-state:{}]", tag);
    let close_marker = format!("[/workflow-state:{}]", tag);

    let start = match full_text.find(&open_marker) {
        Some(pos) => pos + open_marker.len(),
        None => return String::new(),
    };

    let end = match full_text[start..].find(&close_marker) {
        Some(pos) => start + pos,
        None => return String::new(),
    };

    let raw = &full_text[start..end];
    // Strip leading/trailing whitespace and HTML comments
    let mut lines: Vec<&str> = raw
        .lines()
        .map(|l| {
            // Strip <!-- ... --> style comments (full line or trailing)
            let l = l.trim();
            if l.starts_with("<!--") && l.ends_with("-->") {
                "" // entire line is a comment
            } else {
                l
            }
        })
        .filter(|l| !l.is_empty())
        .collect();

    // Trim leading blank lines
    while lines.first().map_or(false, |l| l.trim().is_empty()) {
        lines.remove(0);
    }

    lines.join("\n")
}

/// Look up the tag body for a given task status.
pub fn tag_for_status<'a>(tags: &'a WorkflowTagMap, status: &str) -> Option<&'a str> {
    // Try exact match first
    if let Some(body) = tags.get(status) {
        return Some(body.as_str());
    }

    // Fallback: try common aliases
    let alias = match status {
        "planning" => "planning",
        "in_progress" => "in_progress",
        "completed" => "completed",
        "archived" => "archived",
        "paused" => "paused",
        _ => return None,
    };
    tags.get(alias).map(|s| s.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_tag() {
        let md = "\
# Test

[workflow-state:planning]
Stay in planning. Write prd.md first.
[/workflow-state:planning]
";
        let map = parse_tags_from_str(md);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("planning"));
        assert!(map["planning"].contains("Stay in planning"));
    }

    #[test]
    fn test_parse_multiple_tags() {
        let md = "\
[workflow-state:no_task]
No active task.
[/workflow-state:no_task]

[workflow-state:planning]
Planning phase.
[/workflow-state:planning]

[workflow-state:in_progress]
Implementation phase.
[/workflow-state:in_progress]
";
        let map = parse_tags_from_str(md);
        assert_eq!(map.len(), 3);
        assert!(map.contains_key("no_task"));
        assert!(map.contains_key("planning"));
        assert!(map.contains_key("in_progress"));
    }

    #[test]
    fn test_empty_file_returns_empty() {
        let map = parse_tags_from_str("");
        assert!(map.is_empty());
    }

    #[test]
    fn test_no_tags_returns_empty() {
        let map = parse_tags_from_str("# Just a regular markdown file\nwith no tags");
        assert!(map.is_empty());
    }

    #[test]
    fn test_tag_for_status() {
        let mut map = WorkflowTagMap::new();
        map.insert("planning".to_string(), "Plan phase body".to_string());

        assert_eq!(tag_for_status(&map, "planning"), Some("Plan phase body"));
        assert_eq!(tag_for_status(&map, "in_progress"), None);
    }

    #[test]
    fn test_extract_body_strips_html_comments() {
        let md = "\
[workflow-state:planning]
<!-- Per-turn breadcrumb: planning -->
Plan phase.
[/workflow-state:planning]
";
        let map = parse_tags_from_str(md);
        assert_eq!(map["planning"], "Plan phase.");
    }

    #[test]
    fn test_missing_closing_tag() {
        let md = "\
[workflow-state:planning]
Some text without closing tag
";
        let map = parse_tags_from_str(md);
        assert!(map.is_empty());
    }

    #[test]
    fn test_invalid_tag_format_skipped() {
        let md = "\
[workflow-state:]
Empty tag.
[/workflow-state:]

[workflow-state:tag with spaces]
Spaces not allowed.
[/workflow-state:tag with spaces]
";
        let map = parse_tags_from_str(md);
        assert!(map.is_empty());
    }
}
