use crate::store::SessionStore;
use crate::types::*;
use crate::MemAdapter;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// Pi platform memory adapter.
///
/// Reads sessions from three locations:
/// 1. `~/.config/muse/` — legacy Go dj-muse format (context.md + front-matter)
/// 2. `~/.dijiang/mem/` — new DiJiang JSON format
/// 3. `~/.pi/agent/sessions/` — Trellis Pi JSONL format
pub struct PiMemAdapter {
    muse_dir: PathBuf,
    dijiang_dir: PathBuf,
    pi_agent_dir: PathBuf,
}

impl PiMemAdapter {
/// Create a new PiMemAdapter.
///
/// Default paths: `~/.config/muse/`, `~/.dijiang/mem/`, `~/.pi/agent/sessions/`.
pub fn new() -> Self {
        let home = dirs::home_dir().expect("HOME must be set");
        Self {
            muse_dir: home.join(".config").join("muse"),
            dijiang_dir: home.join(".dijiang").join("mem"),
            pi_agent_dir: home.join(".pi").join("agent").join("sessions"),
        }
        }

    /// Parse a session from a dj-muse directory (context.md + optional session_summary.md).
    fn parse_muse_session(
        session_id: &str,
        dir: &Path,
        status: SessionStatus,
    ) -> Option<SessionRecord> {
        let context_path = dir.join("context.md");
        let context_text = std::fs::read_to_string(context_path).ok()?;
        let front_matter = parse_front_matter(&context_text)?;

        let project_id = front_matter
            .get("project_id")
            .cloned()
            .unwrap_or_else(|| "default".to_string());
        let task = front_matter.get("task").cloned();
        let phase = front_matter.get("phase").cloned();
        let created_at = front_matter
            .get("created_at")
            .cloned()
            .unwrap_or_else(|| String::new());
        let updated_at = front_matter.get("updated_at").cloned();
        let workspace_key = front_matter.get("workspace_key").cloned();
        let workspace_path = front_matter.get("workspace_path").cloned();
        let action_count: u32 = front_matter
            .get("action_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        // Read session_summary.md if it exists
        let summary = std::fs::read_to_string(dir.join("session_summary.md")).ok();

        Some(SessionRecord {
            session_id: session_id.to_string(),
            project_id,
            workspace_key,
            workspace_path,
            status,
            task,
            phase,
            created_at,
            updated_at,
            action_count,
            summary,
            provider: "pi".to_string(),
            source_path: Some(dir.to_string_lossy().to_string()),
        })
    }

    /// Scan a directory of session dirs (active or archived).
    fn scan_muse_dir(
        dir: &Path,
        status: SessionStatus,
    ) -> Result<Vec<SessionRecord>, MemError> {
        let mut sessions = Vec::new();
        if !dir.exists() {
            return Ok(sessions);
        }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let session_id = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if let Some(record) = Self::parse_muse_session(session_id, &path, status.clone()) {
                sessions.push(record);
            }
        }
        Ok(sessions)
    }

    /// Scan Pi agent sessions directory (Trellis JSONL format).
    fn scan_pi_agent_dir(dir: &Path) -> Vec<SessionRecord> {
        let mut sessions = Vec::new();
        if !dir.exists() {
            return sessions;
        }
        // Pi stores sessions in subdirectories named --<encoded-cwd>--
        for entry in std::fs::read_dir(dir).ok().into_iter().flatten() {
            if let Ok(entry) = entry {
                let proj_dir = entry.path();
                if !proj_dir.is_dir() {
                    continue;
                }
                // Scan JSONL files in the project directory
                for file in std::fs::read_dir(&proj_dir).ok().into_iter().flatten() {
                    if let Ok(file) = file {
                        let path = file.path();
                        if !path.is_file() || path.extension().is_some_and(|e| e != "jsonl") {
                            continue;
                        }
                        // Read first event for metadata
                        if let Some(record) = Self::parse_pi_jsonl_session(&path) {
                            sessions.push(record);
                        }
                    }
                }
            }
        }
        sessions
    }

    /// Parse a Pi agent session from a JSONL file.
    fn parse_pi_jsonl_session(file_path: &Path) -> Option<SessionRecord> {
        let file_name = file_path.file_name()?.to_str()?;
        // Extract session_id from filename: <timestamp>_<id>.jsonl
        let session_id = file_name
            .strip_suffix(".jsonl")
            .and_then(|s| s.split('_').nth(1))
            .map(|s| s.to_string())
            .unwrap_or_else(|| file_name.to_string());

        // Get project_id from parent directory name (--<encoded-cwd>--)
        let project_id = file_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|s| {
                // Decode --<encoded-cwd>-- back to path
                let s = s.trim_matches('-');
                s.replace('-', "/")
            })
            .unwrap_or_default();

        // Read created timestamp from first event
        #[derive(serde::Deserialize)]
        struct PiEvent {
            #[serde(default)]
            timestamp: Option<String>,
        }
        let created = crate::jsonl::read_jsonl_first::<PiEvent>(file_path)
            .and_then(|e| e.timestamp)
            .unwrap_or_default();

        let metadata = std::fs::metadata(file_path).ok();
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

        Some(SessionRecord {
            session_id,
            project_id,
            workspace_key: None,
            workspace_path: None,
            status: SessionStatus::Archived,
            task: None,
            phase: None,
            created_at: created,
            updated_at: Some(updated),
            action_count: 0,
            summary: None,
            provider: "pi".to_string(),
            source_path: Some(file_path.to_string_lossy().to_string()),
        })
    }
}

impl Default for PiMemAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MemAdapter for PiMemAdapter {
    fn provider(&self) -> &str {
        "pi"
    }

    async fn list_sessions(&self) -> Result<Vec<SessionRecord>, MemError> {
        let mut sessions = Vec::new();

        // Legacy dj-muse format
        sessions.append(&mut Self::scan_muse_dir(
            &self.muse_dir.join("sessions"),
            SessionStatus::Active,
        )?);
        sessions.append(&mut Self::scan_muse_dir(
            &self.muse_dir.join("archives"),
            SessionStatus::Archived,
        )?);

        // New DiJiang JSON format
        let store = SessionStore::new_at(self.dijiang_dir.clone());
        if let Ok(dijiang_sessions) = store.list_sessions() {
            sessions.extend(dijiang_sessions);
        }

        // Trellis Pi JSONL format (~/.pi/agent/sessions/)
        let pi_sessions = Self::scan_pi_agent_dir(&self.pi_agent_dir);
        sessions.extend(pi_sessions);

        sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(sessions)
    }

    async fn get_session(&self, session_id: &str) -> Result<SessionRecord, MemError> {
        // Check legacy dj-muse format
        for dir in [
            self.muse_dir.join("sessions").join(session_id),
            self.muse_dir.join("archives").join(session_id),
        ] {
            if dir.is_dir() {
                let status = if dir.parent().and_then(|p| p.file_name().and_then(|n| n.to_str()))
                    == Some("sessions")
                {
                    SessionStatus::Active
                } else {
                    SessionStatus::Archived
                };
                if let Some(record) = Self::parse_muse_session(session_id, &dir, status) {
                    return Ok(record);
                }
            }
        }

        // Check new DiJiang JSON format
        let store = SessionStore::new_at(self.dijiang_dir.clone());
        if let Ok(record) = store.read_session(session_id) {
            return Ok(record);
        }

        Err(MemError::NotFound(session_id.to_string()))
    }
}

/// Parse YAML front matter from a markdown file.
///
/// Returns a map of key-value pairs from the YAML front matter block
/// (between `---` markers).
fn parse_front_matter(content: &str) -> Option<std::collections::HashMap<String, String>> {
    let mut map = std::collections::HashMap::new();
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() || lines[0].trim() != "---" {
        return None;
    }

    // Find closing ---
    let end = lines[1..].iter().position(|l| l.trim() == "---")? + 1;
    for line in &lines[1..end] {
        let trimmed = line.trim();
        if !trimmed.contains(':') {
            continue;
        }
        let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
        if parts.len() == 2 {
            let key = parts[0].trim().to_string();
            let value = parts[1].trim().trim_matches('"').to_string();
            map.insert(key, value);
        }
    }
    Some(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_front_matter() {
        let content = r#"---
session_id: "sess-test-123"
project_id: "my-project"
task: "Implement feature X"
status: active
created_at: "2026-06-25T01:00:49Z"
---
Some content"#;
        let fm = parse_front_matter(content).unwrap();
        assert_eq!(fm.get("session_id").unwrap(), "sess-test-123");
        assert_eq!(fm.get("project_id").unwrap(), "my-project");
        assert_eq!(fm.get("task").unwrap(), "Implement feature X");
        assert_eq!(fm.get("status").unwrap(), "active");
        assert_eq!(fm.get("created_at").unwrap(), "2026-06-25T01:00:49Z");
    }

    #[test]
    fn test_parse_front_matter_no_markers() {
        let content = "no front matter here";
        assert!(parse_front_matter(content).is_none());
    }

    #[test]
    fn test_parse_front_matter_empty() {
        let content = "---\n---\nbody";
        let fm = parse_front_matter(content).unwrap();
        assert!(fm.is_empty());
    }
}
