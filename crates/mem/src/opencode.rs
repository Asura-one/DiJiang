use crate::MemAdapter;
/// OpenCode platform memory adapter — currently a no-op.
///
/// OpenCode 1.2+ stores sessions in a SQLite database at
/// `~/.local/share/opencode/opencode.db`. A native SQLite reader would require
/// bundling libsqlite3 — deferred until a non-native backend ships.
///
/// The adapter exists so callers can iterate all known platforms without
/// special-casing. `list_sessions` and `get_session` return empty/NotFound.
use crate::types::*;
use async_trait::async_trait;
use std::path::PathBuf;

/// OpenCode platform adapter (no-op until SQLite support is added).
pub struct OpenCodeAdapter {
    _data_dir: PathBuf,
}

impl OpenCodeAdapter {
    /// Create a new OpenCode adapter at `~/.local/share/opencode/`.
    pub fn new() -> Self {
        let home = dirs::home_dir().expect("HOME must be set");
        Self {
            _data_dir: home.join(".local").join("share").join("opencode"),
        }
    }

    #[allow(dead_code)]
    fn new_at(data_dir: PathBuf) -> Self {
        Self {
            _data_dir: data_dir,
        }
    }
}

impl Default for OpenCodeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MemAdapter for OpenCodeAdapter {
    fn provider(&self) -> &str {
        "opencode"
    }

    async fn list_sessions(&self) -> Result<Vec<SessionRecord>, MemError> {
        Ok(Vec::new())
    }

    async fn get_session(&self, session_id: &str) -> Result<SessionRecord, MemError> {
        Err(MemError::NotFound(session_id.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opencode_empty() {
        let adapter = OpenCodeAdapter::new();
        let sessions = futures::executor::block_on(adapter.list_sessions()).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_opencode_get_session_not_found() {
        let adapter = OpenCodeAdapter::new();
        let result = futures::executor::block_on(adapter.get_session("any"));
        assert!(matches!(result, Err(MemError::NotFound(_))));
    }
}
