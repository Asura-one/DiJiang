use super::{ChangeKind, DiffReport, ModuleChange, PubApiChange};

/// Types of documents that doc-sync can generate/update.
#[derive(Debug, Clone, PartialEq)]
pub enum DocType {
    /// API reference docs (`docs/api/`)
    Api,
    /// Changelog (`CHANGELOG.md`)
    Changelog,
    /// Project readme (`README.md`)
    Readme,
    /// Design notes (`docs/DESIGN_NOTES.md`)
    DesignNotes,
    /// RCA postmortem (`docs/rca/`)
    Rca,
    /// User guide (`docs/guide/`)
    UserGuide,
    /// Deploy docs (`docs/deploy/`)
    Deploy,
    /// Test criteria (`docs/test-criteria.md`)
    TestCriteria,
    /// Requirements (`docs/requirements.md`)
    Requirements,
    /// Design doc (`docs/design.md`)
    Design,
    /// Architecture doc (`docs/architecture/`)
    Architecture,
}

impl DocType {
    /// The default target path for this document type.
    pub fn target_path(&self) -> &str {
        match self {
            DocType::Api => "docs/api/",
            DocType::Changelog => "CHANGELOG.md",
            DocType::Readme => "README.md",
            DocType::DesignNotes => "docs/DESIGN_NOTES.md",
            DocType::Rca => "docs/rca/",
            DocType::UserGuide => "docs/guide/",
            DocType::Deploy => "docs/deploy/",
            DocType::TestCriteria => "docs/test-criteria.md",
            DocType::Requirements => "docs/requirements.md",
            DocType::Design => "docs/design.md",
            DocType::Architecture => "docs/architecture/",
        }
    }

    /// Human-readable label for display.
    pub fn label(&self) -> &str {
        match self {
            DocType::Api => "API 参考文档",
            DocType::Changelog => "变更日志",
            DocType::Readme => "项目说明",
            DocType::DesignNotes => "设计决策记录",
            DocType::Rca => "事故复盘",
            DocType::UserGuide => "用户手册",
            DocType::Deploy => "部署说明",
            DocType::TestCriteria => "测试标准",
            DocType::Requirements => "需求规格说明",
            DocType::Design => "技术设计文档",
            DocType::Architecture => "架构说明文档",
        }
    }
}

/// The kind of impact a code change has on a document.
#[derive(Debug, Clone, PartialEq)]
pub enum ImpactKind {
    /// Document needs to be created (new module/feature)
    NeedsCreation,
    /// Existing document needs updating
    NeedsUpdate,
    /// Existing document should be reviewed for relevance
    NeedsReview,
}

/// Describes how a code change impacts a specific document.
#[derive(Debug, Clone)]
pub struct DocImpact {
    /// Which document type is affected
    pub doc_type: DocType,
    /// Target file path for the affected document
    pub target_path: String,
    /// What kind of update is needed
    pub impact_kind: ImpactKind,
    /// Confidence level (0.0 = guess, 1.0 = certain)
    pub confidence: f64,
    /// Human-readable explanation
    pub reason: String,
}

/// Apply mapping rules: code changes → affected documents.
///
/// Phase 1 rules are heuristic-based (file path patterns).
/// Future phases will incorporate full diff content analysis.
pub fn map_changes_to_docs(report: &DiffReport) -> Vec<DocImpact> {
    let mut impacts: Vec<DocImpact> = Vec::new();
    let mut seen_docs = std::collections::HashSet::new();

    // Helper to add unique impacts
    let mut add = |doc_type: DocType, kind: ImpactKind, confidence: f64, reason: &str| {
        let key = doc_type.target_path().to_string();
        if seen_docs.insert(key) {
            impacts.push(DocImpact {
                target_path: doc_type.target_path().to_string(),
                doc_type,
                impact_kind: kind,
                confidence,
                reason: reason.to_string(),
            });
        }
    };

    // --- Rule 1: Pub API changes → API docs ---
    if !report.pub_api_changes.is_empty() {
        let count = report.pub_api_changes.len();
        let files: Vec<&str> = report
            .pub_api_changes
            .iter()
            .map(|a| a.file.as_str())
            .collect();
        let files_str = files.join(", ");
        let api_reason = format!("{count} 个 API 变更（{files_str}）");
        add(DocType::Api, ImpactKind::NeedsUpdate, 0.9, &api_reason);
    }

    // --- Rule 2: Commit messages → Changelog ---
    if !report.commit_summaries.is_empty() {
        let count = report.commit_summaries.len();
        let changelog_reason = format!("{count} 个 commit 未记录");
        add(
            DocType::Changelog,
            ImpactKind::NeedsUpdate,
            0.8,
            &changelog_reason,
        );
    }

    // --- Rule 3: Core file changes → README ---
    let readme_triggers = ["Cargo.toml", "README.md", "Makefile", "package.json"];
    let has_readme_trigger = report
        .changed_files
        .iter()
        .any(|f| readme_triggers.iter().any(|t| f == t || f.ends_with(t)));
    if has_readme_trigger {
        add(
            DocType::Readme,
            ImpactKind::NeedsUpdate,
            0.7,
            "核心配置文件变更，README 可能需要同步",
        );
    }

    // --- Rule 4: Cargo.toml changes → Deploy docs ---
    let has_cargo_change = report
        .changed_files
        .iter()
        .any(|f| f == "Cargo.toml" || f.ends_with("/Cargo.toml"));
    if has_cargo_change {
        add(
            DocType::Deploy,
            ImpactKind::NeedsUpdate,
            0.8,
            "Cargo.toml 变更影响部署配置",
        );
    }

    // --- Rule 5: Module structure changes → Architecture docs ---
    if !report.module_changes.is_empty() {
        let modules: Vec<&str> = report
            .module_changes
            .iter()
            .map(|m| m.path.as_str())
            .collect();
        let arch_reason = format!("模块结构变更（{}）", modules.join(", "));
        add(
            DocType::Architecture,
            ImpactKind::NeedsUpdate,
            0.7,
            &arch_reason,
        );
    }

    // --- Rule 6: Test file changes → Test Criteria ---
    let has_test_changes = report
        .changed_files
        .iter()
        .any(|f| f.contains("tests/") || f.contains("/test_") || f.ends_with("_test.rs"));
    if has_test_changes {
        add(
            DocType::TestCriteria,
            ImpactKind::NeedsUpdate,
            0.6,
            "测试文件变更，可能需要同步测试标准",
        );
    }

    // --- Rule 7: Dependency changes → Design doc ---
    if !report.dep_changes.is_empty() {
        let deps: Vec<&str> = report.dep_changes.iter().map(|d| d.name.as_str()).collect();
        let dep_reason = format!("依赖变更（{}），可能影响技术设计", deps.join(", "));
        add(DocType::Design, ImpactKind::NeedsReview, 0.5, &dep_reason);
    }

    // --- Rule 9: Docs/ directory changes → Design notes ---
    let has_doc_changes = report.changed_files.iter().any(|f| f.starts_with("docs/"));
    if has_doc_changes {
        add(
            DocType::DesignNotes,
            ImpactKind::NeedsReview,
            0.4,
            "文档目录变更，可能需要更新设计决策记录",
        );
    }

    // --- Rule 10: Any changes → always recommend changelog if commits exist ---
    // Already handled by Rule 2.

    impacts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_report() -> DiffReport {
        DiffReport {
            changed_files: vec![
                "src/lib.rs".into(),
                "src/modules/mod.rs".into(),
                "Cargo.toml".into(),
            ],
            pub_api_changes: vec![super::super::PubApiChange {
                file: "src/lib.rs".into(),
                name: "UserService".into(),
                kind: super::super::ApiItemKind::Struct,
                change: ChangeKind::Added,
            }],
            module_changes: vec![ModuleChange {
                path: "src/modules".into(),
                kind: ChangeKind::Added,
            }],
            dep_changes: vec![],
            commit_summaries: vec![
                "feat: add UserService struct".into(),
                "chore: update Cargo.toml".into(),
            ],
        }
    }

    #[test]
    fn map_pub_api_changes_to_api_docs() {
        let report = sample_report();
        let impacts = map_changes_to_docs(&report);
        assert!(
            impacts.iter().any(|i| i.doc_type == DocType::Api),
            "API changes should map to API docs"
        );
    }

    #[test]
    fn map_changelog_when_commits_exist() {
        let report = sample_report();
        let impacts = map_changes_to_docs(&report);
        assert!(
            impacts.iter().any(|i| i.doc_type == DocType::Changelog),
            "commits should map to changelog"
        );
    }

    #[test]
    fn map_deploy_when_cargo_toml_changes() {
        let report = sample_report();
        let impacts = map_changes_to_docs(&report);
        assert!(
            impacts.iter().any(|i| i.doc_type == DocType::Deploy),
            "Cargo.toml change should map to deploy docs"
        );
    }

    #[test]
    fn map_architecture_when_module_changes() {
        let report = sample_report();
        let impacts = map_changes_to_docs(&report);
        assert!(
            impacts.iter().any(|i| i.doc_type == DocType::Architecture),
            "module changes should map to architecture docs"
        );
    }

    #[test]
    fn no_impacts_on_empty_report() {
        let report = DiffReport::default();
        let impacts = map_changes_to_docs(&report);
        assert!(impacts.is_empty(), "empty diff → no impacts");
    }

    #[test]
    fn no_duplicate_impacts() {
        let report = sample_report();
        let impacts = map_changes_to_docs(&report);
        let mut seen = std::collections::HashSet::new();
        for impact in &impacts {
            assert!(
                seen.insert(impact.doc_type.target_path()),
                "duplicate impact for {}",
                impact.target_path
            );
        }
    }

    #[test]
    fn test_changes_map_to_test_criteria() {
        let report = DiffReport {
            changed_files: vec!["tests/test_a.rs".into()],
            ..DiffReport::default()
        };
        let impacts = map_changes_to_docs(&report);
        assert!(
            impacts.iter().any(|i| i.doc_type == DocType::TestCriteria),
            "test changes should map to test criteria"
        );
    }

    #[test]
    fn doc_changes_map_to_design_notes() {
        let report = DiffReport {
            changed_files: vec!["docs/architecture/design.md".into()],
            ..DiffReport::default()
        };
        let impacts = map_changes_to_docs(&report);
        assert!(
            impacts.iter().any(|i| i.doc_type == DocType::DesignNotes),
            "doc changes should map to design notes"
        );
    }

    #[test]
    fn dep_changes_map_to_design() {
        let report = DiffReport {
            changed_files: vec!["Cargo.toml".into()],
            dep_changes: vec![super::super::DepChange {
                name: "serde".into(),
                old_version: None,
                new_version: Some("1.0".into()),
            }],
            ..DiffReport::default()
        };
        let impacts = map_changes_to_docs(&report);
        assert!(
            impacts.iter().any(|i| i.doc_type == DocType::Design),
            "dep changes should map to design doc"
        );
    }

    #[test]
    fn readme_triggered_by_cargo_toml() {
        let report = DiffReport {
            changed_files: vec!["Cargo.toml".into()],
            ..DiffReport::default()
        };
        let impacts = map_changes_to_docs(&report);
        assert!(
            impacts.iter().any(|i| i.doc_type == DocType::Readme),
            "Cargo.toml change should trigger README"
        );
    }
}
