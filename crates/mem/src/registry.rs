use crate::adapter::MemAdapter;
use crate::types::*;
use std::collections::BTreeMap;

/// Registry of memory adapters.
///
/// Aggregates sessions from all registered platform adapters
/// and provides cross-platform queries.
pub struct MemRegistry {
    adapters: Vec<Box<dyn MemAdapter>>,
}

impl MemRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            adapters: Vec::new(),
        }
    }

    /// Register a memory adapter.
    pub fn register(&mut self, adapter: Box<dyn MemAdapter>) {
        self.adapters.push(adapter);
    }

    /// List sessions from all adapters.
    pub async fn list_sessions(&self) -> Result<Vec<SessionRecord>, MemError> {
        let mut all = Vec::new();
        for adapter in &self.adapters {
            match adapter.list_sessions().await {
                Ok(sessions) => all.extend(sessions),
                Err(e) => {
                    eprintln!("  [mem] {}/list_sessions: {e}", adapter.provider());
                }
            }
        }
        // Sort by created_at descending
        all.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(all)
    }

    /// Aggregate sessions by project.
    pub async fn aggregate_by_project(&self) -> Result<Vec<ProjectSessions>, MemError> {
        let sessions = self.list_sessions().await?;
        let mut projects: BTreeMap<String, Vec<SessionRecord>> = BTreeMap::new();
        for s in sessions {
            projects
                .entry(s.project_id.clone())
                .or_default()
                .push(s);
        }

        let result: Vec<ProjectSessions> = projects
            .into_iter()
            .map(|(project_id, sessions)| {
                let last_active_at = sessions
                    .iter()
                    .filter_map(|s| Some(s.created_at.as_str()))
                    .max()
                    .map(|s| s.to_string());
                ProjectSessions {
                    project_id,
                    sessions,
                    last_active_at,
                }
            })
            .collect();

        Ok(result)
    }

    /// Get active providers.
    pub fn providers(&self) -> Vec<String> {
        self.adapters.iter().map(|a| a.provider().to_string()).collect()
    }

    /// Total number of adapters.
    pub fn adapter_count(&self) -> usize {
        self.adapters.len()
    }
}

impl Default for MemRegistry {
    fn default() -> Self {
        Self::new()
    }
}
