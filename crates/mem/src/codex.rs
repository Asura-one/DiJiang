use crate::MemAdapter;
/// Codex platform memory adapter.
///
/// Session storage layout:
///   `~/.codex/sessions/YYYY/MM/DD/rollout-<ts>-<id>.jsonl`
///
/// The first event of each file is `session_meta` with:
///   payload.id, payload.timestamp, payload.cwd, payload.originator
use crate::jsonl;
use crate::types::*;
use async_trait::async_trait;
use serde::Deserialize;
use std::path::{Path, PathBuf};

// -- Loose shapes matching Codex's schema --

#[derive(Debug, Deserialize)]
struct CodexPayload {
    #[serde(default)]
    timestamp: String,
    #[serde(default)]
    cwd: String,
}

#[derive(Debug, Deserialize)]
struct CodexEvent {
    #[serde(default)]
    r#type: String,
    #[serde(default)]
    timestamp: String,
    #[serde(default)]
    payload: Option<CodexPayload>,
}

/// Codex platform adapter.
pub struct CodexAdapter {
    sessions_dir: PathBuf,
}

impl CodexAdapter {
    /// Create a new Codex adapter.
    ///
    /// `sessions_dir` defaults to `~/.codex/sessions/`.
    pub fn new() -> Self {
        let home = dirs::home_dir().expect("HOME must be set");
        Self {
            sessions_dir: home.join(".codex").join("sessions"),
        }
    }

    /// Create at a custom path (for testing).
    pub fn new_at(sessions_dir: PathBuf) -> Self {
        Self { sessions_dir }
    }

    /// Parse a session record from a Codex JSONL file.
    fn record_from_file(file_path: &Path) -> Option<SessionRecord> {
        let file_name = file_path.file_name()?.to_str()?;

        // Extract session_id from filename: rollout-<ts>-<id>.jsonl
        // Extract session_id from filename: rollout-<ts>-<id>.jsonl
        // Timestamp part is exactly 19 chars (YYYY-MM-DDTHH-MM-SS)
        let session_id = file_name
            .strip_prefix("rollout-")
            .and_then(|s| {
                if s.len() <= 20 {
                    return None;
                }
                // Skip 19-char timestamp + 1-char separator
                Some(s[20..].trim_end_matches(".jsonl").to_string())
            })
            .unwrap_or_else(|| file_name.to_string());

        // Read first event for metadata
        let meta: CodexEvent = jsonl::read_jsonl_first(file_path)?;
        if meta.r#type != "session_meta" {
            return None; // not a valid Codex session
        }

        let payload = meta.payload?;
        let created = if !payload.timestamp.is_empty() {
            payload.timestamp.clone()
        } else {
            meta.timestamp.clone()
        };

        let metadata = std::fs::metadata(file_path).ok();
        let updated = metadata
            .and_then(|m| m.modified().ok())
            .map(|t| {
                let duration = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
                chrono::DateTime::from_timestamp(duration.as_secs() as i64, duration.subsec_nanos())
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        Some(SessionRecord {
            session_id,
            project_id: payload.cwd.clone(),
            workspace_key: None,
            workspace_path: Some(payload.cwd),
            status: SessionStatus::Archived,
            task: None,
            phase: None,
            created_at: created,
            updated_at: Some(updated),
            action_count: 0,
            summary: None,
            provider: "codex".to_string(),
            source_path: Some(file_path.to_string_lossy().to_string()),
        })
    }
}

impl Default for CodexAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MemAdapter for CodexAdapter {
    fn provider(&self) -> &str {
        "codex"
    }

    async fn list_sessions(&self) -> Result<Vec<SessionRecord>, MemError> {
        let mut sessions = Vec::new();

        if !self.sessions_dir.exists() {
            return Ok(sessions);
        }

        // Walk all JSONL files recursively
        let files = jsonl::walk_dir(&self.sessions_dir, |p| {
            p.extension().is_some_and(|e| e == "jsonl")
        });

        for file_path in &files {
            if let Some(record) = Self::record_from_file(file_path) {
                sessions.push(record);
            }
        }

        sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(sessions)
    }

    async fn get_session(&self, session_id: &str) -> Result<SessionRecord, MemError> {
        // Search for a file with the given session_id in its path
        let files = jsonl::walk_dir(&self.sessions_dir, |p| {
            p.extension().is_some_and(|e| e == "jsonl") && p.to_string_lossy().contains(session_id)
        });

        for file_path in &files {
            if let Some(record) = Self::record_from_file(file_path) {
                if record.session_id == session_id {
                    return Ok(record);
                }
            }
        }

        Err(MemError::NotFound(session_id.to_string()))
    }

    async fn get_dialogue(&self, session_id: &str) -> Result<Vec<DialogueEntry>, MemError> {
        let record = self.get_session(session_id).await?;
        let path = record
            .source_path
            .ok_or_else(|| MemError::NotFound(session_id.to_string()))?;
        let mut entries = Vec::new();
        let mut replacement_mirrors = std::collections::HashMap::<(String, String), usize>::new();
        let mut mirror_window_open = false;
        for event in crate::dialogue::read_events(Path::new(&path))? {
            let timestamp = event
                .get("timestamp")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            let kind = event
                .get("type")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            let payload = event.get("payload").unwrap_or(&event);
            if kind == "response_item"
                && payload.get("type").and_then(serde_json::Value::as_str) == Some("message")
            {
                let mirrored = mirror_window_open
                    && crate::dialogue::message_key(payload)
                        .and_then(|key| replacement_mirrors.get_mut(&key))
                        .is_some_and(|remaining| {
                            if *remaining > 0 {
                                *remaining -= 1;
                                true
                            } else {
                                false
                            }
                        });
                if !mirrored {
                    mirror_window_open = false;
                    replacement_mirrors.clear();
                    crate::dialogue::push_message(&mut entries, session_id, timestamp, payload);
                }
            }
            if kind == "compacted"
                && let Some(history) = payload
                    .get("replacement_history")
                    .and_then(serde_json::Value::as_array)
            {
                entries.clear();
                replacement_mirrors.clear();
                mirror_window_open = true;
                for item in history {
                    let message = item.get("payload").unwrap_or(item);
                    if message.get("type").and_then(serde_json::Value::as_str) == Some("message")
                        && let Some(key) = crate::dialogue::push_message(
                            &mut entries,
                            session_id,
                            timestamp,
                            message,
                        )
                    {
                        *replacement_mirrors.entry(key).or_default() += 1;
                    }
                }
            }
        }
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_list_sessions_empty() {
        let tmp = TempDir::new().unwrap();
        let adapter = CodexAdapter::new_at(tmp.path().join("sessions"));
        let sessions = futures::executor::block_on(adapter.list_sessions()).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_list_sessions_with_data() {
        let tmp = TempDir::new().unwrap();
        let session_dir = tmp
            .path()
            .join("sessions")
            .join("2026")
            .join("06")
            .join("28");
        std::fs::create_dir_all(&session_dir).unwrap();

        // Write a Codex session file
        let session_id = "abc123-def456";
        let file_path = session_dir.join(format!("rollout-2026-06-28T12-00-00-{session_id}.jsonl"));
        std::fs::write(
            &file_path,
            r#"{"type":"session_meta","timestamp":"2026-06-28T12:00:00Z","payload":{"id":"abc123-def456","timestamp":"2026-06-28T12:00:00Z","cwd":"/tmp/test-project","originator":"codex_cli_rs","cli_version":"0.107.0"}}
{"type":"user","timestamp":"2026-06-28T12:01:00Z","payload":{"role":"user","content":"hello"}}
"#,
        )
        .unwrap();

        let adapter = CodexAdapter::new_at(tmp.path().join("sessions"));
        let sessions = futures::executor::block_on(adapter.list_sessions()).unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session_id, session_id);
        assert_eq!(sessions[0].provider, "codex");
        assert_eq!(sessions[0].project_id, "/tmp/test-project");
    }

    #[test]
    fn dialogue_reads_replacement_history_and_deduplicates_stream_events() {
        let tmp = TempDir::new().unwrap();
        let sessions = tmp.path().join("sessions/2026/01/01");
        std::fs::create_dir_all(&sessions).unwrap();
        let path = sessions.join("rollout-2026-01-01T00-00-00-compact.jsonl");
        std::fs::write(&path, concat!(
            "{\"type\":\"session_meta\",\"timestamp\":\"t0\",\"payload\":{\"timestamp\":\"t0\",\"cwd\":\"/tmp/project\"}}\n",
            "{\"type\":\"compacted\",\"timestamp\":\"t1\",\"payload\":{\"replacement_history\":[{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"before\"}]},{\"type\":\"summary\",\"text\":\"ignore\"}]}}\n",
            "{\"type\":\"response_item\",\"timestamp\":\"t2\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"before\"}]}}\n",
            "{\"type\":\"response_item\",\"timestamp\":\"t3\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"after\"}]}}\n",
        )).unwrap();
        let adapter = CodexAdapter::new_at(tmp.path().join("sessions"));
        let turns = futures::executor::block_on(adapter.get_dialogue("compact")).unwrap();
        assert_eq!(
            turns
                .iter()
                .map(|turn| turn.content.as_str())
                .collect::<Vec<_>>(),
            ["before", "after"]
        );
    }

    #[test]
    fn dialogue_replaces_preceding_stream_history_without_duplicates() {
        let turns = dialogue_from_events(concat!(
            "{\"type\":\"response_item\",\"timestamp\":\"t1\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":\"same\"}}\n",
            "{\"type\":\"compacted\",\"timestamp\":\"t2\",\"payload\":{\"replacement_history\":[{\"type\":\"message\",\"role\":\"user\",\"content\":\"same\"}]}}\n",
        ));

        assert_eq!(turns, ["same"]);
    }

    #[test]
    fn dialogue_keeps_same_content_after_compaction_mirror_window() {
        let turns = dialogue_from_events(concat!(
            "{\"type\":\"compacted\",\"timestamp\":\"t1\",\"payload\":{\"replacement_history\":[{\"type\":\"message\",\"role\":\"user\",\"content\":\"same\"}]}}\n",
            "{\"type\":\"response_item\",\"timestamp\":\"t2\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":\"same\"}}\n",
            "{\"type\":\"response_item\",\"timestamp\":\"t3\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":\"boundary\"}}\n",
            "{\"type\":\"response_item\",\"timestamp\":\"t4\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":\"same\"}}\n",
        ));

        assert_eq!(turns, ["same", "boundary", "same"]);
    }

    fn dialogue_from_events(events: &str) -> Vec<String> {
        let tmp = TempDir::new().unwrap();
        let sessions = tmp.path().join("sessions/2026/01/01");
        std::fs::create_dir_all(&sessions).unwrap();
        let path = sessions.join("rollout-2026-01-01T00-00-00-ordering.jsonl");
        std::fs::write(
            path,
            format!(
                "{{\"type\":\"session_meta\",\"timestamp\":\"t0\",\"payload\":{{\"timestamp\":\"t0\",\"cwd\":\"/tmp/project\"}}}}\n{events}"
            ),
        )
        .unwrap();
        let adapter = CodexAdapter::new_at(tmp.path().join("sessions"));
        futures::executor::block_on(adapter.get_dialogue("ordering"))
            .unwrap()
            .into_iter()
            .map(|turn| turn.content)
            .collect()
    }
}
