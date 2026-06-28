use crate::types::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// New-format session stored as JSON under `~/.dijiang/mem/`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DijiangSession {
    pub session_id: String,
    pub project_id: String,
    pub workspace_key: Option<String>,
    pub workspace_path: Option<String>,
    pub status: SessionStatus,
    pub task: Option<String>,
    pub phase: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub action_count: u32,
    pub summary: Option<String>,
    pub provider: String,
}

impl From<DijiangSession> for SessionRecord {
    fn from(s: DijiangSession) -> Self {
        SessionRecord {
            session_id: s.session_id,
            project_id: s.project_id,
            workspace_key: s.workspace_key,
            workspace_path: s.workspace_path,
            status: s.status,
            task: s.task,
            phase: s.phase,
            created_at: s.created_at,
            updated_at: s.updated_at,
            action_count: s.action_count,
            summary: s.summary,
            provider: s.provider,
            source_path: None,
        }
    }
}

/// Storage for the new `~/.dijiang/mem/` JSON session format.
pub struct SessionStore {
    base: PathBuf,
}

impl SessionStore {
    /// Create a store rooted at `~/.dijiang/mem/`.
    pub fn new() -> Self {
        let home = dirs::home_dir().expect("HOME must be set");
        Self {
            base: home.join(".dijiang").join("mem"),
        }
    }

    /// Create a store at a custom path (for testing).
    pub fn new_at(path: PathBuf) -> Self {
        Self { base: path }
    }

    // -- Path helpers --

    fn active_dir(&self) -> PathBuf {
        self.base.join("sessions")
    }

    fn archive_dir(&self) -> PathBuf {
        self.base.join("archives")
    }

    fn session_path(&self, session_id: &str, archived: bool) -> PathBuf {
        if archived {
            self.archive_dir().join(format!("{session_id}.json"))
        } else {
            self.active_dir().join(format!("{session_id}.json"))
        }
    }

    // -- Read --

    /// Read a single session from the new format.
    pub fn read_session(&self, session_id: &str) -> Result<SessionRecord, MemError> {
        let path = self.session_path(session_id, false);
        if path.exists() {
            return self.read_json(&path);
        }
        let path = self.session_path(session_id, true);
        if path.exists() {
            return self.read_json(&path);
        }
        Err(MemError::NotFound(session_id.to_string()))
    }

    fn read_json(&self, path: &Path) -> Result<SessionRecord, MemError> {
        let content = std::fs::read_to_string(path)?;
        let session: DijiangSession = serde_json::from_str(&content)
            .map_err(|e| MemError::Parse(format!("invalid session JSON: {e}")))?;
        Ok(session.into())
    }

    /// List all sessions in the new format.
    pub fn list_sessions(&self) -> Result<Vec<SessionRecord>, MemError> {
        let mut sessions = Vec::new();

        // Active
        if self.active_dir().exists() {
            for entry in std::fs::read_dir(self.active_dir())? {
                let entry = entry?;
                if entry.path().extension().is_some_and(|e| e == "json") {
                    if let Ok(record) = self.read_json(&entry.path()) {
                        sessions.push(record);
                    }
                }
            }
        }

        // Archived
        if self.archive_dir().exists() {
            for entry in std::fs::read_dir(self.archive_dir())? {
                let entry = entry?;
                if entry.path().extension().is_some_and(|e| e == "json") {
                    if let Ok(mut record) = self.read_json(&entry.path()) {
                        record.status = SessionStatus::Archived;
                        sessions.push(record);
                    }
                }
            }
        }

        sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(sessions)
    }

    // -- Write --

    /// Save a session (active).
    pub fn save_session(&self, record: &SessionRecord) -> Result<(), MemError> {
        let session = DijiangSession {
            session_id: record.session_id.clone(),
            project_id: record.project_id.clone(),
            workspace_key: record.workspace_key.clone(),
            workspace_path: record.workspace_path.clone(),
            status: SessionStatus::Active,
            task: record.task.clone(),
            phase: record.phase.clone(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            action_count: record.action_count,
            summary: record.summary.clone(),
            provider: "pi".to_string(),
        };

        let dir = self.active_dir();
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.json", record.session_id));
        let content = serde_json::to_string_pretty(&session)
            .map_err(|e| MemError::Parse(format!("serialization error: {e}")))?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Archive a session (move from sessions/ to archives/).
    pub fn archive_session(&self, session_id: &str) -> Result<(), MemError> {
        let active_path = self.session_path(session_id, false);
        if !active_path.exists() {
            return Err(MemError::NotFound(session_id.to_string()));
        }

        let archive_path = self.session_path(session_id, true);
        std::fs::create_dir_all(self.archive_dir())?;
        std::fs::rename(&active_path, &archive_path)?;
        Ok(())
    }

    /// Delete a session.
    pub fn delete_session(&self, session_id: &str) -> Result<(), MemError> {
        for archived in [false, true] {
            let path = self.session_path(session_id, archived);
            if path.exists() {
                std::fs::remove_file(&path)?;
                return Ok(());
            }
        }
        Err(MemError::NotFound(session_id.to_string()))
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_read_session() {
        let tmp = TempDir::new().unwrap();
        let store = SessionStore::new_at(tmp.path().join("mem"));

        let record = SessionRecord {
            session_id: "sess-test-001".to_string(),
            project_id: "test-project".to_string(),
            workspace_key: Some("wk1".to_string()),
            workspace_path: Some("/tmp/test".to_string()),
            status: SessionStatus::Active,
            task: Some("Test session".to_string()),
            phase: Some("implement".to_string()),
            created_at: "2026-06-28T12:00:00Z".to_string(),
            updated_at: Some("2026-06-28T12:30:00Z".to_string()),
            action_count: 3,
            summary: Some("Test summary".to_string()),
            provider: "pi".to_string(),
            source_path: None,
        };

        store.save_session(&record).unwrap();
        let loaded = store.read_session("sess-test-001").unwrap();
        assert_eq!(loaded.session_id, "sess-test-001");
        assert_eq!(loaded.task.unwrap(), "Test session");
        assert_eq!(loaded.status, SessionStatus::Active);
    }

    #[test]
    fn test_write_and_archive() {
        let tmp = TempDir::new().unwrap();
        let store = SessionStore::new_at(tmp.path().join("mem"));

        let record = SessionRecord {
            session_id: "sess-test-002".to_string(),
            project_id: "test-project".to_string(),
            workspace_key: None,
            workspace_path: None,
            status: SessionStatus::Active,
            task: None,
            phase: None,
            created_at: "2026-06-28T13:00:00Z".to_string(),
            updated_at: None,
            action_count: 1,
            summary: None,
            provider: "pi".to_string(),
            source_path: None,
        };

        store.save_session(&record).unwrap();
        assert!(store.session_path("sess-test-002", false).exists());

        store.archive_session("sess-test-002").unwrap();
        assert!(!store.session_path("sess-test-002", false).exists());
        assert!(store.session_path("sess-test-002", true).exists());

        let loaded = store.read_session("sess-test-002").unwrap();
        assert_eq!(loaded.session_id, "sess-test-002");
    }

    #[test]
    fn test_list_sessions() {
        let tmp = TempDir::new().unwrap();
        let store = SessionStore::new_at(tmp.path().join("mem"));

        assert!(store.list_sessions().unwrap().is_empty());

        let r1 = SessionRecord {
            session_id: "sess-001".to_string(),
            project_id: "proj-a".to_string(),
            status: SessionStatus::Active,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            ..Default::default()
        };
        let r2 = SessionRecord {
            session_id: "sess-002".to_string(),
            project_id: "proj-b".to_string(),
            status: SessionStatus::Active,
            created_at: "2026-02-01T00:00:00Z".to_string(),
            ..Default::default()
        };
        store.save_session(&r1).unwrap();
        store.save_session(&r2).unwrap();

        let list = store.list_sessions().unwrap();
        assert_eq!(list.len(), 2);
    }
}
