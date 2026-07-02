/// Registry of platform configurators for auto-discovery and bulk operations.
use crate::types::*;
use std::path::Path;

/// Holds all registered platform configurators, ordered by priority.
pub struct ConfiguratorRegistry {
    configurators: Vec<Box<dyn Configurator>>,
}

impl ConfiguratorRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            configurators: Vec::new(),
        }
    }

    /// Create a registry pre-populated with all built-in configurators.
    pub fn with_all() -> Self {
        let mut reg = Self::new();
        reg.register(Box::new(crate::PiConfigurator::new()));
        reg.register(Box::new(crate::CursorConfigurator::new()));
        reg.register(Box::new(crate::ClaudeConfigurator::new()));
        reg.register(Box::new(crate::CodexConfigurator::new()));
        reg.register(Box::new(crate::OpenCodeConfigurator::new()));
        reg.register(Box::new(crate::HermesConfigurator::new()));
        reg.sort();
        reg
    }

    /// Register a configurator. Duplicate platforms replace the previous entry.
    pub fn register(&mut self, configurator: Box<dyn Configurator>) {
        let platform = configurator.platform();
        // Remove any existing configurator for the same platform
        self.configurators.retain(|c| c.platform() != platform);
        self.configurators.push(configurator);
    }

    /// Sort by priority (ascending — lower number = higher priority).
    pub fn sort(&mut self) {
        self.configurators.sort_by_key(|c| c.priority());
    }

    /// All registered platforms.
    pub fn platforms(&self) -> Vec<PlatformKind> {
        self.configurators.iter().map(|c| c.platform()).collect()
    }

    /// Number of registered configurators.
    pub fn len(&self) -> usize {
        self.configurators.len()
    }

    /// Check if registry is empty.
    pub fn is_empty(&self) -> bool {
        self.configurators.is_empty()
    }

    /// Get a configurator by platform.
    pub fn get(&self, platform: PlatformKind) -> Option<&dyn Configurator> {
        self.configurators
            .iter()
            .find(|c| c.platform() == platform)
            .map(|c| c.as_ref())
    }

    /// Detect which installed platforms are available on this system.
    pub fn auto_detect(&self) -> Vec<PlatformKind> {
        let mut detected: Vec<PlatformKind> = self
            .configurators
            .iter()
            .filter(|c| c.is_installed())
            .map(|c| c.platform())
            .collect();
        detected.sort_by_key(|p| p.priority());
        detected
    }

    /// Configure all registered platforms at `cwd`.
    /// Returns a Vec of (platform, result) for each configurator run.
    pub fn configure_all(&self, cwd: &Path) -> Vec<(PlatformKind, Result<(), ConfigError>)> {
        self.configurators
            .iter()
            .map(|c| (c.platform(), c.configure(cwd)))
            .collect()
    }

    /// Configure specific platforms at `cwd`.
    pub fn configure(
        &self,
        cwd: &Path,
        platforms: &[PlatformKind],
    ) -> Vec<(PlatformKind, Result<(), ConfigError>)> {
        platforms
            .iter()
            .map(|p| {
                let result = self.get(*p).map(|c| c.configure(cwd)).unwrap_or(Err(
                    ConfigError::InvalidPath(format!("No configurator for platform '{:?}'", p)),
                ));
                (*p, result)
            })
            .collect()
    }
}

impl Default for ConfiguratorRegistry {
    fn default() -> Self {
        Self::with_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_has_all_platforms() {
        let reg = ConfiguratorRegistry::with_all();
        assert_eq!(reg.len(), 6);
        assert!(reg.get(PlatformKind::Pi).is_some());
        assert!(reg.get(PlatformKind::Cursor).is_some());
        assert!(reg.get(PlatformKind::Claude).is_some());
        assert!(reg.get(PlatformKind::Codex).is_some());
        assert!(reg.get(PlatformKind::OpenCode).is_some());
        assert!(reg.get(PlatformKind::Hermes).is_some());
    }

    #[test]
    fn test_registry_auto_detect() {
        let reg = ConfiguratorRegistry::with_all();
        let detected = reg.auto_detect();
        // Pi is always detected (home dir check passes in CI)
        assert!(detected.contains(&PlatformKind::Pi));
    }

    #[test]
    fn test_registry_configure_all() {
        let tmp = tempfile::TempDir::new().unwrap();
        let reg = ConfiguratorRegistry::with_all();
        let results = reg.configure_all(tmp.path());
        assert_eq!(results.len(), 6);
        // At minimum Pi should succeed
        let pi_result = results.iter().find(|(p, _)| *p == PlatformKind::Pi);
        assert!(pi_result.is_some());
        assert!(pi_result.unwrap().1.is_ok());
    }

    #[test]
    fn test_registry_configure_specific() {
        let tmp = tempfile::TempDir::new().unwrap();
        let reg = ConfiguratorRegistry::with_all();
        let results = reg.configure(tmp.path(), &[PlatformKind::Pi, PlatformKind::Cursor]);
        assert_eq!(results.len(), 2);
        assert!(results[0].1.is_ok());
        assert!(results[1].1.is_ok());
    }

    #[test]
    fn test_runtime_hooks_surface_errors_and_session_lifecycle() {
        let tmp = tempfile::TempDir::new().unwrap();
        let reg = ConfiguratorRegistry::with_all();
        let results = reg.configure(
            tmp.path(),
            &[
                PlatformKind::Pi,
                PlatformKind::Cursor,
                PlatformKind::Claude,
                PlatformKind::Codex,
                PlatformKind::OpenCode,
            ],
        );
        assert!(results.iter().all(|(_, result)| result.is_ok()));

        let codex_hook =
            std::fs::read_to_string(tmp.path().join(".codex/hooks/inject-workflow-state.py"))
                .unwrap();
        assert!(codex_hook.contains("Hook 错误:"));
        assert!(codex_hook.contains("CODEX_SESSION_ID"));

        let claude_hook =
            std::fs::read_to_string(tmp.path().join(".claude/hooks/inject-workflow-state.py"))
                .unwrap();
        assert!(claude_hook.contains("Hook 错误:"));
        assert!(claude_hook.contains("CLAUDE_SESSION_ID"));

        let cursor_hooks = std::fs::read_to_string(tmp.path().join(".cursor/hooks.json")).unwrap();
        assert!(cursor_hooks.contains(".cursor/hooks/inject-workflow-state.py"));
        let cursor_hook =
            std::fs::read_to_string(tmp.path().join(".cursor/hooks/inject-workflow-state.py"))
                .unwrap();
        assert!(cursor_hook.contains("Hook 错误:"));
        assert!(cursor_hook.contains("CURSOR_SESSION_ID"));

        let pi_extension =
            std::fs::read_to_string(tmp.path().join(".pi/extensions/dijiang/index.ts")).unwrap();
        assert!(pi_extension.contains("session_start"));
        assert!(pi_extension.contains("session_shutdown"));
        assert!(pi_extension.contains("Hook 错误:"));
        assert!(pi_extension.contains("PI_SESSION_ID"));

        let opencode_plugin =
            std::fs::read_to_string(tmp.path().join(".opencode/plugins/session-start.js")).unwrap();
        assert!(opencode_plugin.contains("--hook-event"));
        assert!(opencode_plugin.contains("OPENCODE_SESSION_ID"));
        assert!(opencode_plugin.contains("Hook 错误:"));
    }

    #[test]
    fn test_registry_unknown_platform() {
        let tmp = tempfile::TempDir::new().unwrap();
        let reg = ConfiguratorRegistry::new();
        let results = reg.configure(tmp.path(), &[PlatformKind::Pi]);
        assert_eq!(results.len(), 1);
        assert!(results[0].1.is_err());
    }
}
