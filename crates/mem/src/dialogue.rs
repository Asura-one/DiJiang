use crate::{DialogueEntry, MemError};
use serde_json::Value;
use std::path::Path;

pub(crate) fn read_events(path: &Path) -> Result<Vec<Value>, MemError> {
    let content = std::fs::read_to_string(path)?;
    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).map_err(MemError::from))
        .collect()
}

pub(crate) fn text_content(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => nonempty(text.clone()),
        Value::Array(blocks) => nonempty(
            blocks
                .iter()
                .filter_map(|block| {
                    let kind = block.get("type").and_then(Value::as_str).unwrap_or("text");
                    matches!(kind, "text" | "input_text" | "output_text")
                        .then(|| block.get("text").and_then(Value::as_str))
                        .flatten()
                })
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        _ => None,
    }
}

pub(crate) fn message_key(message: &Value) -> Option<(String, String)> {
    let role = message.get("role").and_then(Value::as_str)?;
    if !matches!(role, "user" | "assistant") {
        return None;
    }
    let content = message.get("content").and_then(text_content)?;
    Some((role.to_string(), content))
}

pub(crate) fn push_message(
    entries: &mut Vec<DialogueEntry>,
    session_id: &str,
    timestamp: &str,
    message: &Value,
) -> Option<(String, String)> {
    let (role, content) = message_key(message)?;
    let key = (role.clone(), content.clone());
    entries.push(DialogueEntry {
        session_id: session_id.to_string(),
        timestamp: timestamp.to_string(),
        role,
        content,
    });
    Some(key)
}

fn nonempty(text: String) -> Option<String> {
    (!text.trim().is_empty()).then_some(text)
}
