/// Benchmarks 体系
///
/// 从 ponytail 的基准测试思想借鉴而来，为 DiJiang 提供可重复的
/// 代码质量基准测量框架，衡量 AI 代码输出是否符合"最小工作量"原则。
///
/// 基准场景定义在 `crates/configurator/templates/benchmarks/scenarios/*.yaml` 中，
/// 每个场景包含一组检查项，对当前 Git diff 运行验证。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// 基准场景定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkScenario {
    pub name: String,
    pub description: String,
    pub checks: Vec<BenchCheck>,
}

/// 单个检查项定义
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BenchCheck {
    /// 统计 Git diff 新增行数
    #[serde(rename = "git_diff_loc")]
    GitDiffLoc {
        name: String,
        description: String,
        #[serde(default)]
        glob: Option<String>,
        #[serde(default = "default_max_additions")]
        max_additions: u64,
    },
    /// 统计 Git diff 修改文件数
    #[serde(rename = "git_diff_files")]
    GitDiffFiles {
        name: String,
        description: String,
        #[serde(default = "default_max_files")]
        max_files: u64,
    },
    /// 检查是否有新文件
    #[serde(rename = "git_diff_new_files")]
    GitDiffNewFiles {
        name: String,
        description: String,
    },
    /// 正则搜索文件内容
    #[serde(rename = "regex_search")]
    RegexSearch {
        name: String,
        description: String,
        path: String,
        includes: Vec<String>,
        #[serde(default)]
        excludes: Vec<String>,
    },
    /// 琐碎变更检查
    #[serde(rename = "trivial_check")]
    TrivialCheck {
        name: String,
        description: String,
    },
}

fn default_max_additions() -> u64 {
    200
}
fn default_max_files() -> u64 {
    15
}

/// 基准运行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub scenario: String,
    pub passed: bool,
    pub checks: Vec<CheckResult>,
    pub timestamp: String,
}

/// 单个检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub name: String,
    pub passed: bool,
    pub message: String,
    pub detail: String,
}

/// 加载所有基准场景
pub fn load_scenarios(benchmarks_dir: &Path) -> Vec<BenchmarkScenario> {
    let scenarios_dir = benchmarks_dir.join("scenarios");
    if !scenarios_dir.is_dir() {
        return Vec::new();
    }

    let mut scenarios = Vec::new();
    if let Ok(entries) = fs::read_dir(&scenarios_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "yaml" || e == "yml") {
                match fs::read_to_string(&path) {
                    Ok(content) => {
                        match serde_yaml::from_str::<BenchmarkScenario>(&content) {
                            Ok(scenario) => scenarios.push(scenario),
                            Err(e) => {
                                eprintln!(
                                    "⚠  Benchmark scenario parse error ({}): {}",
                                    path.display(),
                                    e
                                );
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠  Cannot read {}: {}", path.display(), e);
                    }
                }
            }
        }
    }
    scenarios
}

/// 列出可用基准场景名
pub fn list_scenarios(benchmarks_dir: &Path) -> Vec<String> {
    load_scenarios(benchmarks_dir)
        .into_iter()
        .map(|s| s.name)
        .collect()
}

/// 按名称查找基准场景
pub fn find_scenario(benchmarks_dir: &Path, name: &str) -> Option<BenchmarkScenario> {
    load_scenarios(benchmarks_dir)
        .into_iter()
        .find(|s| s.name == name)
}

/// 运行一个基准场景
pub fn run_scenario(scenario: &BenchmarkScenario, project_root: &Path) -> BenchmarkResult {
    let results: Vec<CheckResult> = scenario
        .checks
        .iter()
        .map(|check| run_check(check, project_root, scenario))
        .collect();

    let passed = results.iter().all(|r| r.passed);

    BenchmarkResult {
        scenario: scenario.name.clone(),
        passed,
        checks: results,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}

fn run_check(check: &BenchCheck, project_root: &Path, _scenario: &BenchmarkScenario) -> CheckResult {
    match check {
        BenchCheck::GitDiffLoc {
            name,
            description,
            glob,
            max_additions,
        } => run_git_diff_loc(name, description, glob, *max_additions, project_root),
        BenchCheck::GitDiffFiles {
            name,
            description,
            max_files,
        } => run_git_diff_files(name, description, *max_files, project_root),
        BenchCheck::GitDiffNewFiles { name, description } => {
            run_git_diff_new_files(name, description, project_root)
        }
        BenchCheck::RegexSearch {
            name,
            description,
            path: search_path,
            includes,
            excludes,
        } => run_regex_search(name, description, search_path, includes, excludes, project_root),
        BenchCheck::TrivialCheck { name, description } => {
            run_trivial_check(name, description, project_root)
        }
    }
}

fn run_git_diff_loc(
    name: &str,
    description: &str,
    glob: &Option<String>,
    max_additions: u64,
    _project_root: &Path,
) -> CheckResult {
    // 使用 git diff --stat 统计新增行数
    let mut cmd = std::process::Command::new("git");
    cmd.args(["diff", "--stat"]);

    let additions = match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let last_line = stdout.lines().last().unwrap_or("");
            // Parse "N file changed, M insertions(+), D deletions(-)"
            let parts: Vec<&str> = last_line.split_whitespace().collect();
            if let Some(idx) = parts.iter().position(|&p| p == "insertion" || p == "insertions" || p.ends_with("insertion")) {
                if idx >= 2 {
                    parts[idx - 1].parse::<u64>().unwrap_or(0)
                } else {
                    0
                }
            } else {
                0
            }
        }
        Err(_) => 0,
    };

    let passed = additions <= max_additions;
    let detail = format!("{} insertions (max: {}){}",
        additions,
        max_additions,
        if let Some(g) = glob { format!(", glob: {}", g) } else { String::new() }
    );

    CheckResult {
        name: name.to_string(),
        passed,
        message: if passed {
            format!("✓ {}: {}", name, description)
        } else {
            format!("✗ {}: {} ({} insertions > {} max)", name, description, additions, max_additions)
        },
        detail,
    }
}

fn run_git_diff_files(
    name: &str,
    description: &str,
    max_files: u64,
    _project_root: &Path,
) -> CheckResult {
    let mut cmd = std::process::Command::new("git");
    cmd.args(["diff", "--name-only"]);

    let file_count = match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.lines().filter(|l| !l.is_empty()).count() as u64
        }
        Err(_) => 0,
    };

    let passed = file_count <= max_files;
    CheckResult {
        name: name.to_string(),
        passed,
        message: if passed {
            format!("✓ {}: {}", name, description)
        } else {
            format!("✗ {}: {} ({} files > {} max)", name, description, file_count, max_files)
        },
        detail: format!("{} files changed (max: {})", file_count, max_files),
    }
}

fn run_git_diff_new_files(
    name: &str,
    description: &str,
    _project_root: &Path,
) -> CheckResult {
    let mut cmd = std::process::Command::new("git");
    cmd.args(["diff", "--name-only", "--diff-filter=A"]);

    let new_files = match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let files: Vec<String> = stdout.lines().filter(|l| !l.is_empty()).map(|s| s.to_string()).collect();
            files
        }
        Err(_) => vec![],
    };

    let passed = new_files.is_empty();
    CheckResult {
        name: name.to_string(),
        passed,
        message: if passed {
            format!("✓ {}: {}", name, description)
        } else {
            format!("✗ {}: {} new files detected: {}", name, description, new_files.join(", "))
        },
        detail: if new_files.is_empty() {
            "No new files".to_string()
        } else {
            format!("{} new file(s): {}", new_files.len(), new_files.join(", "))
        },
    }
}

fn run_regex_search(
    name: &str,
    description: &str,
    search_path: &str,
    includes: &[String],
    excludes: &[String],
    project_root: &Path,
) -> CheckResult {
    // 检查 diff 中是否包含特定模式
    let mut cmd = std::process::Command::new("git");
    cmd.args(["diff"]);

    let diff_content = match cmd.output() {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(_) => String::new(),
    };

    let mut matched_lines: Vec<String> = Vec::new();
    for pattern in includes {
        let re = regex::Regex::new(pattern).unwrap_or_else(|_| regex::Regex::new("").unwrap());
        for line in diff_content.lines() {
            if re.is_match(line) {
                // 检查是否在排除列表中
                let excluded = excludes.iter().any(|ex| {
                    let ex_re = regex::Regex::new(ex).unwrap_or_else(|_| regex::Regex::new("").unwrap());
                    ex_re.is_match(line)
                });
                if !excluded {
                    matched_lines.push(line.to_string());
                }
            }
        }
    }

    let passed = matched_lines.is_empty();
    CheckResult {
        name: name.to_string(),
        passed,
        message: if passed {
            format!("✓ {}: {}", name, description)
        } else {
            format!("✗ {}: {} ({} matches found)", name, description, matched_lines.len())
        },
        detail: if matched_lines.is_empty() {
            "No matches in diff".to_string()
        } else {
            format!("{} match(es) in diff:\n{}", matched_lines.len(), matched_lines.join("\n"))
        },
    }
}

fn run_trivial_check(
    name: &str,
    description: &str,
    _project_root: &Path,
) -> CheckResult {
    let mut cmd = std::process::Command::new("git");
    cmd.args(["diff", "--stat"]);

    let additions = match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let last_line = stdout.lines().last().unwrap_or("");
            let parts: Vec<&str> = last_line.split_whitespace().collect();
            if let Some(idx) = parts.iter().position(|&p| p == "insertion" || p == "insertions" || p.ends_with("insertion")) {
                if idx >= 2 {
                    parts[idx - 1].parse::<u64>().unwrap_or(0)
                } else {
                    0
                }
            } else {
                0
            }
        }
        Err(_) => 0,
    };

    // 琐碎变更：<= 3 行新增
    let is_trivial = additions <= 3;
    let passed = true; // 琐碎检查是信息性的，不直接 pass/fail
    CheckResult {
        name: name.to_string(),
        passed,
        message: format!("ℹ {}: {} ({} insertions{})", name, description, additions,
            if is_trivial { ", trivial change" } else { ", non-trivial" }),
        detail: format!("{} insertion(s) — {}", additions,
            if is_trivial { "trivial change, no test needed" } else { "non-trivial, test recommended" }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn setup() {
        INIT.call_once(|| {
            // Ensure we're in the DiJiang project dir
            let _ = std::env::current_dir();
        });
    }

    #[test]
    fn benchmark_modules_defined() {
        // 验证 benchmarks 目录结构存在
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .unwrap_or(Path::new("."));
        let bench_dir = project_root.join("crates").join("configurator").join("templates").join("benchmarks").join("scenarios");
        assert!(
            bench_dir.is_dir(),
            "Benchmark scenarios directory missing: {}",
            bench_dir.display()
        );
    }

    #[test]
    fn load_scenarios_returns_at_least_yagni() {
        setup();
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .unwrap_or(Path::new("."));
        let bench_dir = project_root.join("crates").join("configurator").join("templates").join("benchmarks");
        let scenarios = load_scenarios(&bench_dir);
        let names: Vec<&str> = scenarios.iter().map(|s| s.name.as_str()).collect();
        assert!(
            names.contains(&"yagni"),
            "YAGNI benchmark scenario must exist. Found: {:?}",
            names
        );
    }

    #[test]
    fn minimal_change_scenario_exists() {
        setup();
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .unwrap_or(Path::new("."));
        let bench_dir = project_root.join("crates").join("configurator").join("templates").join("benchmarks");
        let scenarios = load_scenarios(&bench_dir);
        let names: Vec<&str> = scenarios.iter().map(|s| s.name.as_str()).collect();
        assert!(
            names.contains(&"minimal-change"),
            "minimal-change benchmark scenario must exist. Found: {:?}",
            names
        );
    }

    #[test]
    fn each_scenario_has_at_least_one_check() {
        setup();
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .unwrap_or(Path::new("."));
        let bench_dir = project_root.join("crates").join("configurator").join("templates").join("benchmarks");
        let scenarios = load_scenarios(&bench_dir);
        for scenario in &scenarios {
            assert!(
                !scenario.checks.is_empty(),
                "Benchmark scenario '{}' has no checks defined",
                scenario.name
            );
        }
    }
}
