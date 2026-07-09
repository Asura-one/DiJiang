pub mod analyzer;
pub mod mapper;

pub use mapper::{DocImpact, DocType, ImpactKind, map_changes_to_docs};

use std::fmt;

/// Types of changes the analyzer can detect.
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeKind {
    Added,
    Modified,
    Removed,
}

impl fmt::Display for ChangeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChangeKind::Added => write!(f, "新增"),
            ChangeKind::Modified => write!(f, "修改"),
            ChangeKind::Removed => write!(f, "删除"),
        }
    }
}

/// Changes to a pub API item (function, struct, trait, etc.)
#[derive(Debug, Clone)]
pub struct PubApiChange {
    pub file: String,
    pub name: String,
    pub kind: ApiItemKind,
    pub change: ChangeKind,
}

/// Categories of Rust API items
#[derive(Debug, Clone, PartialEq)]
pub enum ApiItemKind {
    Function,
    Struct,
    Trait,
    Enum,
    Module,
    Constant,
    TypeAlias,
}

impl fmt::Display for ApiItemKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiItemKind::Function => write!(f, "函数"),
            ApiItemKind::Struct => write!(f, "结构体"),
            ApiItemKind::Trait => write!(f, "Trait"),
            ApiItemKind::Enum => write!(f, "枚举"),
            ApiItemKind::Module => write!(f, "模块"),
            ApiItemKind::Constant => write!(f, "常量"),
            ApiItemKind::TypeAlias => write!(f, "类型别名"),
        }
    }
}

/// Changes to a module/directory structure.
#[derive(Debug, Clone)]
pub struct ModuleChange {
    pub path: String,
    pub kind: ChangeKind,
}

/// Changes to project dependencies (Cargo.toml, etc.)
#[derive(Debug, Clone)]
pub struct DepChange {
    pub name: String,
    pub old_version: Option<String>,
    pub new_version: Option<String>,
}

/// Complete diff analysis result.
#[derive(Debug, Clone, Default)]
pub struct DiffReport {
    /// All files touched by the diff (relative paths)
    pub changed_files: Vec<String>,
    /// Pub API items that changed (added/modified/removed)
    pub pub_api_changes: Vec<PubApiChange>,
    /// Module/directory structure changes
    pub module_changes: Vec<ModuleChange>,
    /// Changes to dependencies
    pub dep_changes: Vec<DepChange>,
    /// Parsed commit message summaries
    pub commit_summaries: Vec<String>,
}

impl DiffReport {
    pub fn is_empty(&self) -> bool {
        self.changed_files.is_empty()
    }
}

/// Display the report in a human-readable format.
pub fn format_report(report: &DiffReport, impacts: &[DocImpact]) -> String {
    let mut out = String::new();

    if report.is_empty() {
        out.push_str("  当前分支与 main 无差异。\n");
        return out;
    }

    // Changed files summary
    out.push_str(&format!("  涉及 {} 个文件变更", report.changed_files.len()));
    if !report.pub_api_changes.is_empty() {
        out.push_str(&format!(
            "，其中 {} 个 pub API 变更",
            report.pub_api_changes.len()
        ));
    }
    out.push('\n');

    // Pub API changes detail
    for api in &report.pub_api_changes {
        out.push_str(&format!(
            "    {} {}: {} ({})\n",
            match api.change {
                ChangeKind::Added => "➕",
                ChangeKind::Modified => "✏️",
                ChangeKind::Removed => "🗑️",
            },
            api.kind,
            api.name,
            api.file,
        ));
    }

    // Module changes
    for mc in &report.module_changes {
        out.push_str(&format!(
            "    {} {} ({})\n",
            match mc.kind {
                ChangeKind::Added => "📁",
                ChangeKind::Modified => "📂",
                ChangeKind::Removed => "📭",
            },
            &mc.path.trim_end_matches('/'),
            mc.kind,
        ));
    }

    // Dependency changes
    for dc in &report.dep_changes {
        let version_info = match (&dc.old_version, &dc.new_version) {
            (Some(old), Some(new)) if old != new => format!(": {old} → {new}"),
            (None, Some(new)) => format!(": {new}"),
            (Some(old), None) => format!(": {old} → (removed)"),
            _ => String::new(),
        };
        let mut status = String::new();
        if dc.old_version.is_none() {
            status.push_str("新增依赖");
        } else if dc.new_version.is_none() {
            status.push_str("删除依赖");
        } else {
            status.push_str("依赖更新");
            status.push_str(&version_info);
        }
        out.push_str(&format!("    📦 {} {}\n", dc.name, status));
    }

    // Commit summaries
    for msg in &report.commit_summaries {
        out.push_str(&format!("    💬 {}\n", msg));
    }

    out.push('\n');

    // Document impacts
    if impacts.is_empty() {
        out.push_str("  📋 当前变更不影响任何已注册文档。\n");
    } else {
        out.push_str("📋 以下文档可能因代码变更需要更新：\n\n");
        for impact in impacts {
            let icon = match impact.impact_kind {
                ImpactKind::NeedsCreation => "🆕",
                ImpactKind::NeedsUpdate => "⚠️",
                ImpactKind::NeedsReview => "🔍",
            };
            out.push_str(&format!(
                "  {}  {} — {}\n",
                icon, impact.target_path, impact.reason,
            ));
        }
        out.push('\n');
    }

    out
}
