use crate::MemAdapter;
/// Claude Code platform memory adapter.
///
/// Session storage layout:
///   `~/.claude/projects/<sanitized-cwd>/<sessionId>.jsonl`
/// Metadata index:
///   `~/.claude/projects/<sanitized-cwd>/sessions-index.json`
///
/// The index contains an `entries` array with fields:
///   sessionId, fullPath, created, modified, projectPath, messageCount
use crate::jsonl;
use crate::types::*;
use async_trait::async_trait;
use serde::Deserialize;
use std::path::{Path, PathBuf};

// -- Loose shapes matching Claude's schema --

#[derive(Debug, Deserialize)]
struct ClaudeIndexEntry {
    #[serde(default)]
    sessionId: String,
    #[serde(default)]
    fullPath: String,
    #[serde(default)]
    projectPath: String,
    #[serde(default)]
    created: String,
    #[serde(default)]
    modified: String,
    #[serde(default)]
    firstPrompt: Option<String>,
    #[serde(default)]
    messageCount: u64,
}

#[derive(Debug, Deserialize)]
struct ClaudeIndex {
    #[serde(default)]
    entries: Vec<ClaudeIndexEntry>,
}

#[derive(Debug, Deserialize)]
struct ClaudeMessage {
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    content: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ClaudeEvent {
    #[serde(default)]
    r#type: String,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(default)]
    message: Option<ClaudeMessage>,
}

/// Claude Code platform adapter.
pub struct ClaudeAdapter {
    projects_dir: PathBuf,
}

impl ClaudeAdapter {
    /// Create a new Claude adapter.
    ///
    /// `projects_dir` defaults to `~/.claude/projects/`.
    pub fn new() -> Self {
        let home = dirs::home_dir().expect("HOME must be set");
        Self {
            projects_dir: home.join(".claude").join("projects"),
        }
    }

    /// Create at a custom path (for testing).
    pub fn new_at(projects_dir: PathBuf) -> Self {
        Self { projects_dir }
    }

    /// Parse a single session from an index entry + file path.
    fn record_from_index(
        &self,
        entry: &ClaudeIndexEntry,
        file_path: &Path,
    ) -> Option<SessionRecord> {
        let session_id = if !entry.sessionId.is_empty() {
            entry.sessionId.clone()
        } else {
            file_path
                .file_stem()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())?
        };

        Some(SessionRecord {
            session_id,
            project_id: entry.projectPath.clone(),
            workspace_key: None,
            workspace_path: Some(entry.projectPath.clone()),
            status: SessionStatus::Archived,
            task: entry.firstPrompt.clone(),
            phase: None,
            created_at: entry.created.clone(),
            updated_at: Some(entry.modified.clone()),
            action_count: entry.messageCount as u32,
            summary: None,
            provider: "claude".to_string(),
            source_path: Some(file_path.to_string_lossy().to_string()),
        })
    }

    /// Fallback: read first event from a JSONL file to get cwd/timestamp.
    fn read_first_event_cwd(file_path: &Path) -> Option<(String, String)> {
        let event: ClaudeEvent = jsonl::read_jsonl_first(file_path)?;
        let cwd = event.cwd?;
        let ts = event.timestamp.unwrap_or_default();
        Some((cwd, ts))
    }
}

impl Default for ClaudeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MemAdapter for ClaudeAdapter {
    fn provider(&self) -> &str {
        "claude"
    }

    async fn list_sessions(&self) -> Result<Vec<SessionRecord>, MemError> {
        let mut sessions = Vec::new();

        if !self.projects_dir.exists() {
            return Ok(sessions);
        }

        for entry in std::fs::read_dir(&self.projects_dir)? {
            let entry = entry?;
            let project_dir = entry.path();
            if !project_dir.is_dir() {
                continue;
            }

            // Read the index file first (fast path)
            let index_path = project_dir.join("sessions-index.json");
            if let Ok(content) = std::fs::read_to_string(&index_path) {
                if let Ok(idx) = serde_json::from_str::<ClaudeIndex>(&content) {
                    for entry in &idx.entries {
                        let file_path = PathBuf::from(&entry.fullPath);
                        if file_path.exists() {
                            if let Some(record) = self.record_from_index(entry, &file_path) {
                                sessions.push(record);
                            }
                        }
                    }
                    continue; // index gives us everything
                }
            }

            // Fallback: scan .jsonl files directly
            for file in std::fs::read_dir(&project_dir)? {
                let file = file?;
                let path = file.path();
                if !path.is_file() || path.extension().is_some_and(|e| e != "jsonl") {
                    continue;
                }

                let session_id = path
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                let (cwd, created) = Self::read_first_event_cwd(&path).unwrap_or_default();

                let metadata = std::fs::metadata(&path).ok();
                let updated = metadata
                    .and_then(|m| m.modified().ok())
                    .map(|t| {
                        let duration = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
                        chrono::DateTime::from_timestamp(
                            duration.as_secs() as i64,
                            duration.subsec_nanos(),
                        )
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_default()
                    })
                    .unwrap_or_default();

                sessions.push(SessionRecord {
                    session_id,
                    project_id: cwd.clone(),
                    workspace_key: None,
                    workspace_path: Some(cwd),
                    status: SessionStatus::Archived,
                    task: None,
                    phase: None,
                    created_at: created,
                    updated_at: Some(updated),
                    action_count: 0,
                    summary: None,
                    provider: "claude".to_string(),
                    source_path: Some(path.to_string_lossy().to_string()),
                });
            }
        }

        sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(sessions)
    }

    async fn get_session(&self, session_id: &str) -> Result<SessionRecord, MemError> {
        for entry in std::fs::read_dir(&self.projects_dir)? {
            let entry = entry?;
            let project_dir = entry.path();
            if !project_dir.is_dir() {
                continue;
            }

            let file_path = project_dir.join(format!("{session_id}.jsonl"));
            if file_path.exists() {
                let (cwd, created) = Self::read_first_event_cwd(&file_path).unwrap_or_default();
                let metadata = std::fs::metadata(&file_path).ok();
                let updated = metadata
                    .and_then(|m| m.modified().ok())
                    .map(|t| {
                        let duration = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
                        chrono::DateTime::from_timestamp(
                            duration.as_secs() as i64,
                            duration.subsec_nanos(),
                        )
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_default()
                    })
                    .unwrap_or_default();

                return Ok(SessionRecord {
                    session_id: session_id.to_string(),
                    project_id: cwd.clone(),
                    workspace_key: None,
                    workspace_path: Some(cwd),
                    status: SessionStatus::Archived,
                    task: None,
                    phase: None,
                    created_at: created,
                    updated_at: Some(updated),
                    action_count: 0,
                    summary: None,
                    provider: "claude".to_string(),
                    source_path: Some(file_path.to_string_lossy().to_string()),
                });
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
        let adapter = ClaudeAdapter::new_at(tmp.path().join("projects"));
        let sessions = futures::executor::block_on(adapter.list_sessions()).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_list_sessions_with_index() {
        let tmp = TempDir::new().unwrap();
        let proj_dir = tmp.path().join("projects").join("test-project");
        std::fs::create_dir_all(&proj_dir).unwrap();

        // Write a session file
        let session_id = "test-session-001";
        let session_path = proj_dir.join(format!("{session_id}.jsonl"));
        std::fs::write(
            &session_path,
            r#"{"type":"user","cwd":"/tmp/test","timestamp":"2026-01-01T00:00:00Z","message":{"role":"user","content":"hello"}}
{"type":"assistant","timestamp":"2026-01-01T00:01:00Z","message":{"role":"assistant","content":"hi"}}
"#,
        )
        .unwrap();

        // Write index
        let index = serde_json::json!({
            "entries": [{
                "sessionId": session_id,
                "fullPath": session_path.to_string_lossy(),
                "created": "2026-01-01T00:00:00Z",
                "modified": "2026-01-01T00:01:00Z",
                "projectPath": "/tmp/test",
                "firstPrompt": "hello",
                "messageCount": 2
            }]
        });
        std::fs::write(proj_dir.join("sessions-index.json"), index.to_string()).unwrap();

        let adapter = ClaudeAdapter::new_at(tmp.path().join("projects"));
        let sessions = futures::executor::block_on(adapter.list_sessions()).unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session_id, session_id);
        assert_eq!(sessions[0].provider, "claude");
        assert_eq!(sessions[0].project_id, "/tmp/test");
        assert_eq!(sessions[0].task.as_deref(), Some("hello"));
    }
}
