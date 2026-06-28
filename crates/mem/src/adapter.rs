use crate::types::*;
use async_trait::async_trait;

/// Platform-specific memory adapter.
///
/// Each provider (Pi, Claude, Codex, Cursor) implements this trait
/// to expose their session data in a uniform format.
#[async_trait]
pub trait MemAdapter: Send + Sync {
    /// Provider name (e.g. "pi", "claude", "codex", "cursor").
    fn provider(&self) -> &str;

    /// List all sessions from this provider.
    async fn list_sessions(&self) -> Result<Vec<SessionRecord>, MemError>;

    /// Get a single session by ID.
    async fn get_session(&self, session_id: &str) -> Result<SessionRecord, MemError>;

    /// Get dialogue entries for a given session.
    ///
    /// Default implementation returns empty — override if the provider
    /// stores conversational history.
    async fn get_dialogue(&self, session_id: &str) -> Result<Vec<DialogueEntry>, MemError> {
        let _ = session_id;
        Ok(Vec::new())
    }
}
