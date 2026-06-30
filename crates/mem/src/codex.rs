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
    id: String,
    #[serde(default)]
    timestamp: String,
    #[serde(default)]
    cwd: String,
    #[serde(default)]
    originator: Option<String>,
    #[serde(default)]
    cli_version: Option<String>,
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
}
