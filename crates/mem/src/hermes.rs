use crate::MemAdapter;
/// Hermes Agent platform memory adapter.
///
/// Session storage layout:
///   `~/.hermes/sessions/`
///     - `sessions.json`         — quick index (keyed by session_key)
///     - `session_YYYYMMDD_HHMMSS_<id>.json`  — per-session metadata + message log
///     - `YYYYMMDD_HHMMSS_<id>.jsonl`          — JSONL message log (optional)
///
/// The `session_*.json` files contain the richest metadata:
///   session_id, model, platform, session_start, last_updated, message_count, messages[]
use crate::jsonl;
use crate::types::*;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// -- Index shape from sessions.json --

#[derive(Debug, Deserialize, Default)]
struct HermesIndexEntry {
    #[serde(default)]
    session_id: String,
    #[serde(default)]
    created_at: String,
    #[serde(default)]
    updated_at: String,
    #[serde(default)]
    display_name: String,
    #[serde(default)]
    platform: String,
    #[serde(default)]
    chat_type: String,
}

type HermesIndex = HashMap<String, HermesIndexEntry>;

// -- Per-session metadata from session_*.json --

#[derive(Debug, Deserialize)]
struct HermesSessionMeta {
    #[serde(default)]
    session_id: String,
    #[serde(default)]
    model: String,
    #[serde(default)]
    platform: String,
    #[serde(default)]
    session_start: String,
    #[serde(default)]
    last_updated: String,
    #[serde(default)]
    message_count: u32,
}

// -- First event from JSONL (session_meta) --

#[derive(Debug, Deserialize)]
struct HermesJsonlMeta {
    #[serde(default)]
    platform: String,
    #[serde(default)]
    timestamp: String,
    #[serde(default)]
    model: String,
}

/// Hermes Agent platform adapter.
pub struct HermesAdapter {
    sessions_dir: PathBuf,
}

impl HermesAdapter {
    /// Create a new Hermes adapter pointing at `~/.hermes/sessions/`.
    pub fn new() -> Self {
        let home = dirs::home_dir().expect("HOME must be set");
        Self {
            sessions_dir: home.join(".hermes").join("sessions"),
        }
    }

    /// Create at a custom path (for testing).
    pub fn new_at(sessions_dir: PathBuf) -> Self {
        Self { sessions_dir }
    }

    /// Build a SessionRecord from a session JSON file path.
    fn record_from_session_file(&self, meta_path: &Path) -> Option<SessionRecord> {
        let content = std::fs::read_to_string(meta_path).ok()?;
        let meta: HermesSessionMeta = serde_json::from_str(&content).ok()?;

        let session_id = if !meta.session_id.is_empty() {
            meta.session_id
        } else {
            meta_path
                .file_stem()
                .and_then(|n| n.to_str())
                .map(|s| s.strip_prefix("session_").unwrap_or(s).to_string())?
        };

        // Derive project_id from platform
        let project_id = if !meta.platform.is_empty() {
            format!("hermes/{}", meta.platform)
        } else {
            "hermes".to_string()
        };

        let created_at = if !meta.session_start.is_empty() {
            // Normalize: Hermes uses space-separated format "2026-05-19 16:12:07.648299"
            meta.session_start.replace(' ', "T")
        } else {
            String::new()
        };

        let updated_at = if !meta.last_updated.is_empty() {
            Some(meta.last_updated.replace(' ', "T"))
        } else {
            None
        };

        // For first prompt / task, read the first user message from `messages` field
        let task = extract_first_user_message(&content);

        Some(SessionRecord {
            session_id,
            project_id,
            workspace_key: None,
            workspace_path: None,
            status: SessionStatus::Archived,
            task,
            phase: None,
            created_at,
            updated_at,
            action_count: meta.message_count,
            summary: None,
            provider: "hermes".to_string(),
            source_path: Some(meta_path.to_string_lossy().to_string()),
        })
    }

    /// Build a SessionRecord from a JSONL file path (fallback for sessions without a session_*.json).
    fn record_from_jsonl(&self, file_path: &Path) -> Option<SessionRecord> {
        let first: HermesJsonlMeta = jsonl::read_jsonl_first(file_path)?;

        let session_id = file_path
            .file_stem()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())?;

        let project_id = if !first.platform.is_empty() {
            format!("hermes/{}", first.platform)
        } else {
            "hermes".to_string()
        };

        Some(SessionRecord {
            session_id,
            project_id,
            workspace_key: None,
            workspace_path: None,
            status: SessionStatus::Archived,
            task: None,
            phase: None,
            created_at: first.timestamp,
            updated_at: None,
            action_count: 0,
            summary: None,
            provider: "hermes".to_string(),
            source_path: Some(file_path.to_string_lossy().to_string()),
        })
    }

    /// Walk the sessions dir and find all `session_*.json` files.
    fn scan_session_json_files(&self) -> Vec<PathBuf> {
        jsonl::walk_dir(&self.sessions_dir, |p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("session_") && n.ends_with(".json"))
        })
    }

    /// Walk the sessions dir and find all `*.jsonl` files that are NOT session_*.
    fn scan_jsonl_files(&self) -> Vec<PathBuf> {
        jsonl::walk_dir(&self.sessions_dir, |p| {
            p.extension().is_some_and(|e| e == "jsonl")
        })
    }
}

impl Default for HermesAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MemAdapter for HermesAdapter {
    fn provider(&self) -> &str {
        "hermes"
    }

    async fn list_sessions(&self) -> Result<Vec<SessionRecord>, MemError> {
        let mut sessions: Vec<SessionRecord> = Vec::new();

        if !self.sessions_dir.exists() {
            return Ok(sessions);
        }

        // --- Method 1: try sessions.json index first (fast path) ---
        let index_path = self.sessions_dir.join("sessions.json");
        if let Ok(content) = std::fs::read_to_string(&index_path) {
            if let Ok(index) = serde_json::from_str::<HermesIndex>(&content) {
                for (_key, entry) in &index {
                    // Try to find the matching session_*.json file
                    let stem = format!("session_{}.json", entry.session_id);
                    let meta_path = self.sessions_dir.join(&stem);

                    if meta_path.exists() {
                        if let Some(record) = self.record_from_session_file(&meta_path) {
                            sessions.push(record);
                            continue;
                        }
                    }

                    // Fallback: create a minimal record from the index
                    sessions.push(SessionRecord {
                        session_id: entry.session_id.clone(),
                        project_id: format!("hermes/{}", entry.platform),
                        workspace_key: None,
                        workspace_path: None,
                        status: SessionStatus::Archived,
                        task: Some(entry.display_name.clone()),
                        phase: None,
                        created_at: entry.created_at.replace(' ', "T"),
                        updated_at: Some(entry.updated_at.replace(' ', "T")),
                        action_count: 0,
                        summary: None,
                        provider: "hermes".to_string(),
                        source_path: None,
                    });
                }
            }
        }

        // --- Method 2: scan for orphan JSON files not covered by index ---
        for meta_path in self.scan_session_json_files() {
            if let Some(record) = self.record_from_session_file(&meta_path) {
                if !sessions.iter().any(|s| s.session_id == record.session_id) {
                    sessions.push(record);
                }
            }
        }

        // --- Method 3: scan for orphan JSONL files not covered by index ---
        for jsonl_path in self.scan_jsonl_files() {
            if let Some(record) = self.record_from_jsonl(&jsonl_path) {
                if !sessions.iter().any(|s| s.session_id == record.session_id) {
                    sessions.push(record);
                }
            }
        }

        sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(sessions)
    }

    async fn get_session(&self, session_id: &str) -> Result<SessionRecord, MemError> {
        if !self.sessions_dir.exists() {
            return Err(MemError::NotFound(session_id.to_string()));
        }

        // Try session_<id>.json first
        let meta_path = self.sessions_dir.join(format!("session_{session_id}.json"));
        if meta_path.exists() {
            if let Some(record) = self.record_from_session_file(&meta_path) {
                return Ok(record);
            }
        }

        // Fallback: try JSONL
        let jsonl_path = self.sessions_dir.join(format!("{session_id}.jsonl"));
        if jsonl_path.exists() {
            if let Some(record) = self.record_from_jsonl(&jsonl_path) {
                return Ok(record);
            }
        }

        // Last resort: check sessions.json index
        let index_path = self.sessions_dir.join("sessions.json");
        if let Ok(content) = std::fs::read_to_string(&index_path) {
            if let Ok(index) = serde_json::from_str::<HermesIndex>(&content) {
                for (_key, entry) in &index {
                    if entry.session_id == session_id {
                        return Ok(SessionRecord {
                            session_id: entry.session_id.clone(),
                            project_id: format!("hermes/{}", entry.platform),
                            workspace_key: None,
                            workspace_path: None,
                            status: SessionStatus::Archived,
                            task: Some(entry.display_name.clone()),
                            phase: None,
                            created_at: entry.created_at.replace(' ', "T"),
                            updated_at: Some(entry.updated_at.replace(' ', "T")),
                            action_count: 0,
                            summary: None,
                            provider: "hermes".to_string(),
                            source_path: None,
                        });
                    }
                }
            }
        }

        Err(MemError::NotFound(session_id.to_string()))
    }
}

/// Extract the first user message from a session JSON file's `messages` array.
fn extract_first_user_message(content: &str) -> Option<String> {
    // Quick parse: find first message with role "user"
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(messages) = val.get("messages").and_then(|m| m.as_array()) {
            for msg in messages {
                if msg.get("role").and_then(|r| r.as_str()) == Some("user") {
                    let text = msg.get("content").and_then(|c| c.as_str())?;
                    // Truncate very long messages
                    // Truncate very long messages at char boundary
                    if text.len() > 120 {
                        let truncated: String = text.chars().take(117).collect();
                        return Some(format!("{truncated}..."));
                    }
                    return Some(text.to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_list_sessions_empty() {
        let tmp = TempDir::new().unwrap();
        let adapter = HermesAdapter::new_at(tmp.path().join("sessions"));
        let sessions = futures::executor::block_on(adapter.list_sessions()).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_list_sessions_from_json_files() {
        let tmp = TempDir::new().unwrap();
        let sess_dir = tmp.path().join("sessions");
        std::fs::create_dir_all(&sess_dir).unwrap();

        // Write a session_*.json file
        let session_json = serde_json::json!({
            "session_id": "20260519_180207_26acd8",
            "model": "mimo-v2.5-pro",
            "platform": "cli",
            "session_start": "2026-05-19T16:12:07.648299",
            "last_updated": "2026-05-19T23:34:41.152149",
            "message_count": 20,
            "messages": [
                {"role": "user", "content": "fix the bug in parser"},
                {"role": "assistant", "content": "Looking at the code..."}
            ]
        });
        std::fs::write(
            sess_dir.join("session_20260519_180207_26acd8.json"),
            session_json.to_string(),
        )
        .unwrap();

        let adapter = HermesAdapter::new_at(sess_dir);
        let sessions = futures::executor::block_on(adapter.list_sessions()).unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session_id, "20260519_180207_26acd8");
        assert_eq!(sessions[0].provider, "hermes");
        assert_eq!(sessions[0].project_id, "hermes/cli");
        assert_eq!(sessions[0].task.as_deref(), Some("fix the bug in parser"));
        assert_eq!(sessions[0].action_count, 20);
    }

    #[test]
    fn test_list_sessions_from_index() {
        let tmp = TempDir::new().unwrap();
        let sess_dir = tmp.path().join("sessions");
        std::fs::create_dir_all(&sess_dir).unwrap();

        // Write a session_*.json file
        let session_json = serde_json::json!({
            "session_id": "20260416_083335_0792d3ba",
            "model": "gpt-4o",
            "platform": "telegram",
            "session_start": "2026-04-16T08:33:35.480710",
            "last_updated": "2026-04-16T08:35:35.663744",
            "message_count": 5,
            "messages": [
                {"role": "user", "content": "hello"}
            ]
        });
        std::fs::write(
            sess_dir.join("session_20260416_083335_0792d3ba.json"),
            session_json.to_string(),
        )
        .unwrap();

        // Write sessions.json index
        let index = serde_json::json!({
            "agent:main:telegram:dm:test:123": {
                "session_key": "agent:main:telegram:dm:test:123",
                "session_id": "20260416_083335_0792d3ba",
                "created_at": "2026-04-16T08:33:35.480710",
                "updated_at": "2026-04-16T08:35:35.663744",
                "display_name": "Test User",
                "platform": "telegram",
                "chat_type": "dm"
            }
        });
        std::fs::write(sess_dir.join("sessions.json"), index.to_string()).unwrap();

        let adapter = HermesAdapter::new_at(sess_dir);
        let sessions = futures::executor::block_on(adapter.list_sessions()).unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session_id, "20260416_083335_0792d3ba");
        assert_eq!(sessions[0].provider, "hermes");
        assert_eq!(sessions[0].project_id, "hermes/telegram");
    }

    #[test]
    fn test_get_session_not_found() {
        let tmp = TempDir::new().unwrap();
        let adapter = HermesAdapter::new_at(tmp.path().join("sessions"));
        let result = futures::executor::block_on(adapter.get_session("nonexistent"));
        assert!(matches!(result, Err(MemError::NotFound(_))));
    }
}
