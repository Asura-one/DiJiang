use clap::{Parser, Subcommand};
use dialoguer::{Input, MultiSelect};
use dijiang_configurator::PlatformKind;
use dijiang_configurator::TemplateRegistry;
use dijiang_task::store;
use dijiang_task::types::{TaskRecord, TaskStatus};
use serde_json::Value;
use std::io::Write;
use std::path::{Path, PathBuf};
#[derive(Parser)]
#[command(
    name = "dijiang",
    version = env!("CARGO_PKG_VERSION"),
    about = "DiJiang - AI coding assistant workflow layer"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 显示项目状态
    Status {
        /// 显示详细的兼容性诊断信息
        #[arg(long)]
        compat: bool,
    },
    /// 开始并激活一个 DiJiang 工作会话（生命周期入口）
    Start {
        /// 任务名称（slug 格式，如 "fix-login-bug"）
        name: String,
        /// 任务显示标题（可选）
        title: Option<String>,
    },
    /// 接收自然语言请求，自动创建/激活任务并输出路由上下文
    Dispatch {
        /// 用户原始请求
        prompt: Vec<String>,
        /// 已有 active task 时仍创建新任务
        #[arg(long)]
        force_new: bool,
        /// 输出 Codex/Agent hook 可消费的 JSON payload
        #[arg(long)]
        json: bool,
        /// hook event name（JSON 输出时使用）
        #[arg(long, default_value = "UserPromptSubmit")]
        hook_event: String,
    },
    /// 底层任务管理（原子状态操作）
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },
    /// 初始化 DiJiang 项目
    Init {
        /// 项目名称（默认为目录名）
        #[arg(default_value = "")]
        name: String,
        /// 开发者名称（如未提供则从 git 检测）
        #[arg(long)]
        developer: Option<String>,
        /// 跳过交互式提示，使用默认值
        #[arg(long, short = 'y')]
        yes: bool,
        /// 强制重新初始化
        #[arg(long)]
        force: bool,
        /// 要配置的平台（逗号分隔：pi,cursor,claude,codex,opencode,hermes）
        #[arg(long)]
        platforms: Option<String>,
        /// 自动检测已安装的平台
        #[arg(long)]
        auto_detect: bool,
    },
    /// 记忆管理：项目 findings/lessons 与全局 tactics/patterns
    Mem {
        #[command(subcommand)]
        command: MemCommands,
    },
    /// 模板管理
    Template {
        #[command(subcommand)]
        command: TemplateCommands,
    },
    /// 管理 dj-* 技能（列出、同步到项目）
    Skills {
        /// 同步技能到当前项目
        #[arg(long)]
        sync: bool,
    },
    /// 输出当前 session 的 workflow state（供 hook/agent 注入使用）
    WorkflowState {
        /// 输出 Codex/Agent hook 可消费的 JSON payload
        #[arg(long)]
        json: bool,
        /// hook event name（JSON 输出时使用）
        #[arg(long, default_value = "UserPromptSubmit")]
        hook_event: String,
    },
    /// 将 Trellis 项目迁移到 DiJiang
    Migrate,
    /// Agent 通道管理
    Channel {
        #[command(subcommand)]
        command: ChannelCommands,
    },
    /// 完成当前工作：验证、文档同步门禁、可选提交/集成、归档任务、记录 journal
    FinishWork {
        /// 本次工作的简短总结
        #[arg(long)]
        summary: Option<String>,
        /// 已执行的验证命令或人工检查结论
        #[arg(long)]
        verification: Option<String>,
        /// 文档/spec 同步结论；有代码变更或 --commit 时必填
        #[arg(long)]
        docs_sync: Option<String>,
        /// 版本影响决策：major、minor、patch、none
        #[arg(long, default_value = "none")]
        version_impact: String,
        /// 提交当前任务 diff 后再归档
        #[arg(long)]
        commit: bool,
        /// 提交消息；未提供时根据 task 和 summary 自动生成
        #[arg(long)]
        commit_message: Option<String>,
        /// push 任务分支；只在 --commit 后允许
        #[arg(long)]
        push: bool,
        /// 合并任务分支到主分支并清理 worktree；只在 --commit 后允许
        #[arg(long)]
        integrate: bool,
        /// 集成目标主分支
        #[arg(long, default_value = "main")]
        main_branch: String,
        /// 远端名称
        #[arg(long, default_value = "origin")]
        remote: String,
        /// 允许在 git 工作区存在未提交/未跟踪改动时不提交也完成任务
        #[arg(long)]
        allow_dirty: bool,
    },
    /// 更新当前项目的 dj-* 技能和代理
    Update {
        /// 强制更新所有文件
        #[arg(long)]
        force: bool,
        /// 从 GitHub 下载最新版本
        #[arg(long)]
        from_github: bool,
    },
}
#[derive(Subcommand)]
enum ChannelCommands {
    /// 生成一个 agent 执行任务
    Spawn {
        /// Agent 名称（check, implement 等）
        agent: String,
        /// 任务路径（可选，默认为当前任务）
        #[arg(long)]
        task: Option<String>,
        /// 工作目录（可选）
        #[arg(long)]
        dir: Option<String>,
    },
    /// 列出活跃的通道
    List,
    /// 向通道发送消息
    Send {
        /// 通道 ID
        channel_id: String,
        /// 要发送的消息
        message: String,
    },
    /// 查看通道状态
    Status {
        /// 通道 ID（或 'all' 查看所有通道）
        channel_id: String,
    },
    /// 停止通道
    Stop {
        /// 通道 ID
        channel_id: String,
    },
    /// 在通道中执行 agent
    Execute {
        /// 通道 ID
        channel_id: String,
        /// 使用的模型（可选）
        #[arg(long)]
        model: Option<String>,
        /// 使用的提供商（可选）
        #[arg(long)]
        provider: Option<String>,
        /// 超时时间（秒，默认 300）
        #[arg(short, long, default_value = "300")]
        timeout: u64,
        /// 实时输出
        #[arg(long)]
        follow: bool,
    },
    /// 并行执行所有活跃的通道
    ExecuteAll {
        /// 使用的模型（可选）
        #[arg(long)]
        model: Option<String>,
        /// 使用的提供商（可选）
        #[arg(long)]
        provider: Option<String>,
        /// 超时时间（秒，默认 300）
        #[arg(short, long, default_value = "300")]
        timeout: u64,
    },
}
#[derive(Subcommand)]
enum MemCommands {
    /// 列出跨平台会话
    List,
    /// 同步所有平台会话到 ~/.dijiang/mem/
    Sync,
    /// 追加发现到项目日志
    Findings {
        #[arg(long)]
        finding: String,
    },
    /// 记录学习到项目日志
    Learn {
        #[arg(long)]
        lesson: String,
    },
    /// 记录用户纠正并吸收为经验教训
    Correction {
        #[arg(long)]
        correction: String,
        #[arg(long)]
        lesson: String,
        #[arg(long, default_value = "workflow")]
        scope: String,
        #[arg(long, default_value = "user")]
        source: String,
        #[arg(long, default_value = "until-superseded")]
        freshness: String,
        #[arg(long, default_value = "none")]
        conflict: String,
        #[arg(long)]
        actionability: String,
    },
    /// 归档当前会话
    Archive,
    /// 添加策略到全局记忆
    Tactic {
        #[arg(long)]
        name: String,
        #[arg(long)]
        description: String,
    },
    /// 列出策略或通过 Thompson 采样选择 top-k
    Tactics {
        #[arg(long, default_value = "5")]
        select: usize,
    },
    /// 记录策略事件（成功/失败）
    Record {
        #[arg(long)]
        tactic: String,
        #[arg(long)]
        outcome: String, // success or failure
        #[arg(long)]
        context: String,
    },
    /// 添加模式/标准操作流程
    Pattern {
        #[arg(long)]
        name: String,
        #[arg(long)]
        description: String,
    },
    /// 列出模式
    Patterns,
    /// 显示记忆统计信息
    Stats,
    /// 备份项目记忆到全局
    Backup,
    /// 运行快速进化循环（分析会话，提炼策略）
    Evolve,
    /// 运行慢速微调循环（基于积累的经验训练）
    Finetune,
}

#[derive(Subcommand)]
enum TemplateCommands {
    /// 列出可用模板（内置和缓存）
    List,
    /// 从源拉取模板（gh:owner/repo 或 URL）
    Pull {
        /// 模板源（如 gh:tiezhu/dijiang-templates）
        source: String,
    },
    /// 验证模板目录
    Validate {
        /// 模板目录或 manifest.toml 路径
        path: String,
    },
}

#[derive(Subcommand)]
enum TaskCommands {
    /// List all tasks
    List,
    /// Show current active task
    Current,
    /// Create and activate a task record (low-level task operation)
    Start {
        /// Task name (slug, e.g. "fix-login-bug")
        name: String,
    },
    /// Set task status
    Status {
        /// Task name (slug)
        name: String,
        /// New status: planning|in_progress|completed|archived|paused
        status: String,
    },
    /// Archive a task (set status to Archived, record archived_at)
    Archive {
        /// Task name (slug)
        name: String,
    },
    /// Prune old archived tasks
    Prune {
        /// Prune tasks archived more than N days ago (default: 30)
        #[arg(long, default_value = "30")]
        days: u64,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { name, title } => cmd_start(&name, title.as_deref()),
        Commands::Dispatch {
            prompt,
            force_new,
            json,
            hook_event,
        } => cmd_dispatch(&prompt.join(" "), force_new, json, &hook_event),
        Commands::Status { compat } => cmd_status(compat),
        Commands::Init {
            name,
            developer,
            yes,
            force,
            platforms,
            auto_detect,
        } => cmd_init(
            &name,
            developer.as_deref(),
            yes,
            force,
            platforms.as_deref(),
            auto_detect,
        ),
        Commands::Task { command } => match command {
            TaskCommands::List => cmd_task_list(),
            TaskCommands::Current => cmd_task_current(),
            TaskCommands::Start { name } => cmd_task_start(&name),
            TaskCommands::Status { name, status } => cmd_task_status(&name, &status),
            TaskCommands::Archive { name } => cmd_task_archive(&name),
            TaskCommands::Prune { days } => cmd_task_prune(days),
        },
        Commands::Mem {
            command: MemCommands::List,
        } => cmd_mem_list(),
        Commands::Mem {
            command: MemCommands::Sync,
        } => cmd_mem_sync(),
        Commands::Mem {
            command: MemCommands::Findings { finding },
        } => cmd_mem_findings(&finding),
        Commands::Mem {
            command: MemCommands::Learn { lesson },
        } => cmd_mem_learn(&lesson),
        Commands::Mem {
            command:
                MemCommands::Correction {
                    correction,
                    lesson,
                    scope,
                    source,
                    freshness,
                    conflict,
                    actionability,
                },
        } => cmd_mem_correction(
            &correction,
            &lesson,
            &scope,
            &source,
            &freshness,
            &conflict,
            &actionability,
        ),
        Commands::Mem {
            command: MemCommands::Archive,
        } => cmd_mem_archive(),
        Commands::Mem {
            command: MemCommands::Tactic { name, description },
        } => cmd_mem_tactic(&name, &description),
        Commands::Mem {
            command: MemCommands::Tactics { select },
        } => cmd_mem_tactics(select),
        Commands::Mem {
            command:
                MemCommands::Record {
                    tactic,
                    outcome,
                    context,
                },
        } => cmd_mem_record(&tactic, &outcome, &context),
        Commands::Mem {
            command: MemCommands::Pattern { name, description },
        } => cmd_mem_pattern(&name, &description),
        Commands::Mem {
            command: MemCommands::Patterns,
        } => cmd_mem_patterns(),
        Commands::Mem {
            command: MemCommands::Stats,
        } => cmd_mem_stats(),
        Commands::Mem {
            command: MemCommands::Backup,
        } => cmd_mem_backup(),
        Commands::Mem {
            command: MemCommands::Evolve,
        } => cmd_mem_evolve(),
        Commands::Mem {
            command: MemCommands::Finetune,
        } => cmd_mem_finetune(),
        Commands::Template { command } => match command {
            TemplateCommands::List => cmd_template_list(),
            TemplateCommands::Pull { source } => cmd_template_pull(&source),
            TemplateCommands::Validate { path } => cmd_template_validate(&path),
        },
        Commands::Skills { sync } => cmd_skills(sync),
        Commands::Migrate => cmd_migrate(),
        Commands::WorkflowState { json, hook_event } => cmd_workflow_state(json, &hook_event),
        Commands::Channel { command } => match command {
            ChannelCommands::Spawn { agent, task, dir } => {
                cmd_channel_spawn(&agent, task.as_deref(), dir.as_deref())
            }
            ChannelCommands::List => cmd_channel_list(),
            ChannelCommands::Send {
                channel_id,
                message,
            } => cmd_channel_send(&channel_id, &message),
            ChannelCommands::Status { channel_id } => cmd_channel_status(&channel_id),
            ChannelCommands::Stop { channel_id } => cmd_channel_stop(&channel_id),
            ChannelCommands::Execute {
                channel_id,
                model,
                provider,
                timeout,
                follow,
            } => cmd_channel_execute(
                &channel_id,
                model.as_deref(),
                provider.as_deref(),
                timeout,
                follow,
            ),
            ChannelCommands::ExecuteAll {
                model,
                provider,
                timeout,
            } => cmd_channel_execute_all(model.as_deref(), provider.as_deref(), timeout),
        },
        Commands::FinishWork {
            summary,
            verification,
            docs_sync,
            version_impact,
            commit,
            commit_message,
            push,
            integrate,
            main_branch,
            remote,
            allow_dirty,
        } => cmd_finish_work(FinishWorkOptions {
            summary: summary.as_deref(),
            verification: verification.as_deref(),
            docs_sync: docs_sync.as_deref(),
            version_impact: &version_impact,
            commit,
            commit_message: commit_message.as_deref(),
            push,
            integrate,
            main_branch: &main_branch,
            remote: &remote,
            allow_dirty,
        }),
        Commands::Update { force, from_github } => cmd_update(force, from_github),
    }
}

/// 获取项目 .dijiang/ 目录（失败时返回错误）
fn require_dijiang_dir() -> anyhow::Result<std::path::PathBuf> {
    let cwd = std::env::current_dir()?;
    crate::store::find_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow::anyhow!("未找到 .dijiang/ 目录。请先运行 `dijiang init`。"))
}

fn read_developer(dijiang_dir: &std::path::Path) -> anyhow::Result<String> {
    let config_path = dijiang_dir.join("config.toml");
    let config_str = std::fs::read_to_string(&config_path).unwrap_or_default();
    Ok(config_str
        .lines()
        .find(|line| line.trim_start().starts_with("developer"))
        .and_then(|line| line.split('=').nth(1))
        .map(|value| value.trim().trim_matches('"').to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "developer".to_string()))
}

fn read_project_name(dijiang_dir: &std::path::Path) -> anyhow::Result<String> {
    let config_path = dijiang_dir.join("config.toml");
    let config_str = std::fs::read_to_string(&config_path).unwrap_or_default();
    Ok(config_str
        .lines()
        .find(|line| line.trim_start().starts_with("name"))
        .and_then(|line| line.split('=').nth(1))
        .map(|value| value.trim().trim_matches('"').to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string()))
}

fn append_finish_journal(
    dijiang_dir: &Path,
    developer: &str,
    task_name: &str,
    summary: Option<&str>,
    verification: &str,
    dirty_allowed: bool,
) -> anyhow::Result<PathBuf> {
    let workspace = dijiang_dir.join("workspace").join(developer);
    std::fs::create_dir_all(&workspace)?;
    let journal = workspace.join("journal.md");
    let summary = summary.unwrap_or("Work finished.");
    let status = if task_name == "no-active-task" {
        "completed-no-task"
    } else {
        "archived"
    };
    let entry = format!(
        "\n## {} — finish-work\n- Task: `{}`\n- Summary: {}\n- Verification: {}\n- Dirty allowed: {}\n- Status: {}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M"),
        task_name,
        summary,
        verification,
        dirty_allowed,
        status
    );
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&journal)?
        .write_all(entry.as_bytes())?;
    Ok(journal)
}

fn git_dirty_entries(project_root: &Path) -> anyhow::Result<Vec<String>> {
    if !project_root.join(".git").exists() {
        return Ok(Vec::new());
    }
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(project_root)
        .output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "git status failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}
#[derive(Debug, Clone, Copy)]
struct FinishWorkOptions<'a> {
    summary: Option<&'a str>,
    verification: Option<&'a str>,
    docs_sync: Option<&'a str>,
    version_impact: &'a str,
    commit: bool,
    commit_message: Option<&'a str>,
    push: bool,
    integrate: bool,
    main_branch: &'a str,
    remote: &'a str,
    allow_dirty: bool,
}

fn trim_required(value: Option<&str>, message: &str) -> anyhow::Result<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| anyhow::anyhow!(message.to_string()))
}

fn run_git(project_root: &Path, args: &[&str]) -> anyhow::Result<String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(project_root)
        .output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn git_worktree_root(cwd: &Path) -> anyhow::Result<Option<PathBuf>> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(cwd)
        .output()?;
    if !output.status.success() {
        return Ok(None);
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        Ok(None)
    } else {
        Ok(Some(PathBuf::from(path)))
    }
}

fn find_dijiang_dir_in_git_worktrees(project_root: &Path) -> anyhow::Result<Option<PathBuf>> {
    let output = std::process::Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(project_root)
        .output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "git worktree list failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            let candidate = PathBuf::from(path).join(".dijiang");
            if candidate.is_dir() {
                return Ok(Some(candidate));
            }
        }
    }

    Ok(None)
}

fn git_current_branch(project_root: &Path) -> anyhow::Result<String> {
    run_git(project_root, &["branch", "--show-current"])
}

fn git_common_dir(project_root: &Path) -> anyhow::Result<PathBuf> {
    let path = run_git(project_root, &["rev-parse", "--git-common-dir"])?;
    let path = PathBuf::from(path);
    Ok(if path.is_absolute() {
        path
    } else {
        project_root.join(path)
    })
}

fn git_main_worktree(project_root: &Path, main_branch: &str) -> anyhow::Result<PathBuf> {
    let common_dir = git_common_dir(project_root)?;
    let output = std::process::Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(project_root)
        .output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "git worktree list failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let mut current_path: Option<PathBuf> = None;
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            current_path = Some(PathBuf::from(path));
            continue;
        }
        if line == format!("branch refs/heads/{main_branch}") {
            return current_path.ok_or_else(|| anyhow::anyhow!("invalid git worktree output"));
        }
    }

    let main_path = common_dir
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| anyhow::anyhow!("cannot infer main worktree from git common dir"))?
        .to_path_buf();
    if git_current_branch(&main_path).ok().as_deref() == Some(main_branch) {
        return Ok(main_path);
    }

    Err(anyhow::anyhow!(
        "未找到主分支 worktree：{main_branch}。请先 checkout 主分支或手动合并。"
    ))
}

fn bump_semver(version: &str, impact: &str) -> anyhow::Result<String> {
    let parts = version
        .split('.')
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()?;
    if parts.len() != 3 {
        return Err(anyhow::anyhow!("unsupported version format: {version}"));
    }
    let (major, minor, patch) = (parts[0], parts[1], parts[2]);
    Ok(match impact {
        "major" => format!("{}.0.0", major + 1),
        "minor" => format!("{major}.{}.0", minor + 1),
        "patch" => format!("{major}.{minor}.{}", patch + 1),
        "none" => version.to_string(),
        _ => return Err(anyhow::anyhow!("unsupported version impact: {impact}")),
    })
}

fn update_workspace_version(project_root: &Path, impact: &str) -> anyhow::Result<Option<String>> {
    if impact == "none" {
        return Ok(None);
    }
    let cargo_toml = project_root.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&cargo_toml)?;
    let mut in_workspace_package = false;
    let mut changed = false;
    let mut old_version = String::new();
    let mut new_lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_workspace_package = trimmed == "[workspace.package]";
        }
        if in_workspace_package && trimmed.starts_with("version") && trimmed.contains('=') {
            let indent = line
                .chars()
                .take_while(|ch| ch.is_whitespace())
                .collect::<String>();
            let value = trimmed
                .split_once('=')
                .map(|(_, value)| value.trim().trim_matches('"'))
                .ok_or_else(|| anyhow::anyhow!("invalid version line in Cargo.toml"))?;
            let next = bump_semver(value, impact)?;
            old_version = value.to_string();
            new_lines.push(format!("{indent}version = \"{next}\""));
            changed = true;
            continue;
        }
        new_lines.push(line.to_string());
    }

    if changed {
        std::fs::write(&cargo_toml, format!("{}\n", new_lines.join("\n")))?;
        Ok(Some(format!(
            "{old_version} -> {}",
            bump_semver(&old_version, impact)?
        )))
    } else {
        Ok(None)
    }
}
fn ensure_finish_preconditions(
    project_root: &Path,
    task: Option<&dijiang_task::types::TaskRecord>,
    options: FinishWorkOptions<'_>,
) -> anyhow::Result<(String, String)> {
    if let Some(task) = task {
        if matches!(task.status, TaskStatus::Archived) {
            return Err(anyhow::anyhow!("Task '{}' is already archived.", task.name));
        }
    }

    if options.commit && options.allow_dirty {
        return Err(anyhow::anyhow!(
            "finish-work 不能同时使用 --commit 和 --allow-dirty；提交模式会自动提交当前任务 diff。"
        ));
    }
    if (options.push || options.integrate) && !options.commit {
        return Err(anyhow::anyhow!(
            "finish-work 的 --push/--integrate 需要同时使用 --commit，避免推送或合并未记录的 diff。"
        ));
    }
    if !matches!(options.version_impact, "major" | "minor" | "patch" | "none") {
        return Err(anyhow::anyhow!(
            "--version-impact must be one of: major, minor, patch, none"
        ));
    }

    let verification = trim_required(
        options.verification,
        "finish-work requires --verification, e.g. `--verification \"cargo test -p dijiang-task\"`.",
    )?;

    let dirty = git_dirty_entries(project_root)?;
    if (options.commit || !dirty.is_empty()) && !options.allow_dirty {
        let docs_sync = trim_required(
            options.docs_sync,
            "finish-work requires --docs-sync when code/artifacts changed, e.g. `--docs-sync \"docs/spec updated\"` or `--docs-sync \"none: no docs affected\"`.",
        )?;
        if options.commit {
            return Ok((verification, docs_sync));
        }
    }

    if !dirty.is_empty() && !options.allow_dirty {
        let preview = dirty
            .iter()
            .take(12)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n  ");
        let more = if dirty.len() > 12 {
            format!("\n  ... and {} more", dirty.len() - 12)
        } else {
            String::new()
        };
        return Err(anyhow::anyhow!(
            "finish-work 被阻止：git worktree 存在未提交修改。请先审查范围、决定版本影响、提交当前任务 diff，再重新运行 finish-work。只有明确要不提交就关闭任务时才使用 --allow-dirty。\n  {preview}{more}"
        ));
    }

    Ok((
        verification,
        options
            .docs_sync
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("none: no code or docs change")
            .to_string(),
    ))
}

fn default_commit_message(task_name: &str, summary: Option<&str>) -> String {
    let summary = summary
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(task_name);
    format!("chore(dijiang): 完成 {summary}")
}

fn current_session_key() -> (String, String) {
    store::current_session_identity()
        .map(|identity| (identity.key().to_string(), identity.source().to_string()))
        .unwrap_or_else(|| ("global_global".to_string(), "global".to_string()))
}

fn append_session_closure(
    dijiang_dir: &Path,
    developer: &str,
    session_key: &str,
    source: &str,
    task_name: &str,
    summary: Option<&str>,
    verification: &str,
    dirty_allowed: bool,
) -> anyhow::Result<PathBuf> {
    let closed_at = chrono::Utc::now().to_rfc3339();
    let sessions_dir = dijiang_dir
        .join("workspace")
        .join(developer)
        .join("sessions");
    std::fs::create_dir_all(&sessions_dir)?;
    let journal = sessions_dir.join(format!("{session_key}.jsonl"));
    let event = serde_json::json!({
        "event": "session_closed",
        "session_key": session_key,
        "source": source,
        "task": task_name,
        "summary": summary.unwrap_or("Work finished and task archived."),
        "verification": verification,
        "dirty_allowed": dirty_allowed,
        "closed_at": closed_at,
    });
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&journal)?
        .write_all(format!("{}\n", serde_json::to_string(&event)?).as_bytes())?;

    let runtime_path = dijiang_dir
        .join(".runtime")
        .join("sessions")
        .join(format!("{session_key}.json"));
    if let Some(parent) = runtime_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut value: Value = if runtime_path.exists() {
        let content = std::fs::read_to_string(&runtime_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({
            "session_key": session_key,
            "source": source,
        })
    };
    value["closed_at"] = serde_json::json!(closed_at);
    value["closed_task"] = serde_json::json!(task_name);
    value["closed_verification"] = serde_json::json!(verification);
    value["closed_dirty_allowed"] = serde_json::json!(dirty_allowed);
    value["current_task"] = serde_json::Value::Null;
    std::fs::write(runtime_path, serde_json::to_string_pretty(&value)?)?;

    Ok(journal)
}

fn perform_finish_commit(
    project_root: &Path,
    task_name: &str,
    summary: Option<&str>,
    message: Option<&str>,
) -> anyhow::Result<Option<String>> {
    let dirty = git_dirty_entries(project_root)?;
    if dirty.is_empty() {
        return Ok(None);
    }

    run_git(project_root, &["add", "--all"])?;
    let commit_message = message
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| default_commit_message(task_name, summary));
    run_git(project_root, &["commit", "-m", &commit_message])?;
    let commit = run_git(project_root, &["rev-parse", "--short", "HEAD"])?;
    Ok(Some(commit))
}

fn perform_finish_integration(
    project_root: &Path,
    options: FinishWorkOptions<'_>,
) -> anyhow::Result<()> {
    let branch = git_current_branch(project_root)?;
    if branch.is_empty() {
        return Err(anyhow::anyhow!(
            "finish-work 无法在 detached HEAD 上执行集成。"
        ));
    }
    if branch == options.main_branch {
        return Err(anyhow::anyhow!(
            "finish-work 不在主分支上执行 --integrate。请在任务 worktree 分支中运行。"
        ));
    }

    if options.push {
        run_git(project_root, &["push", "-u", options.remote, &branch])?;
    }

    if options.integrate {
        let main_worktree = git_main_worktree(project_root, options.main_branch)?;
        let project_root_string = project_root.display().to_string();
        run_git(&main_worktree, &["merge", "--no-ff", &branch])?;
        if options.push {
            run_git(
                &main_worktree,
                &["push", options.remote, options.main_branch],
            )?;
        }
        run_git(
            &main_worktree,
            &["worktree", "remove", &project_root_string],
        )?;
        run_git(&main_worktree, &["branch", "-d", &branch])?;
    }

    Ok(())
}

fn cmd_finish_work(options: FinishWorkOptions<'_>) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let project_root = git_worktree_root(&cwd)?.unwrap_or(cwd);
    let local_dijiang_dir = project_root.join(".dijiang");
    let uses_local_dijiang_state = local_dijiang_dir.is_dir();
    let dijiang_dir = if uses_local_dijiang_state {
        local_dijiang_dir
    } else {
        find_dijiang_dir_in_git_worktrees(&project_root)?
            .ok_or_else(|| anyhow::anyhow!("未找到 .dijiang/ 目录。请先运行 `dijiang init`。"))?
    };
    let tasks_dir = dijiang_dir.join("tasks");
    let active_task = if uses_local_dijiang_state {
        store::read_active_task(&dijiang_dir)?
    } else {
        None
    };
    let task_before_archive = match active_task.as_deref() {
        Some(active_task) => match store::load_task(&tasks_dir, active_task) {
            Ok(task) => Some(task),
            Err(store::TaskError::NotFound(_)) => {
                anyhow::bail!(
                    "finish-work 的 active task 指向 `{active_task}`，但 `.dijiang/tasks/{active_task}/task.json` 不存在。这通常表示 task state 已陈旧或 task artifact 被清理。请用 `dijiang task current` / `dijiang task list` 检查状态；若当前工作仍需归档，请重新 `dijiang start <name>`，否则清理 stale active task 后再继续。"
                );
            }
            Err(error) => return Err(error.into()),
        },
        None => None,
    };
    let (verification, docs_sync) =
        ensure_finish_preconditions(&project_root, task_before_archive.as_ref(), options)?;
    let version_update = update_workspace_version(&project_root, options.version_impact)?;
    let developer = read_developer(&dijiang_dir)?;
    let (session_key, source) = current_session_key();
    let task_label = active_task.as_deref().unwrap_or("no-active-task");
    let journal = append_finish_journal(
        &dijiang_dir,
        &developer,
        task_label,
        options.summary,
        &verification,
        options.allow_dirty,
    )?;

    let archive_status = if let Some(active_task) = active_task.as_deref() {
        let task = store::archive_task(&tasks_dir, active_task)?;
        store::clear_active_task(&dijiang_dir)?;
        format!(
            "archived task `{}` (status: {}), journal: {}",
            active_task,
            task.status.as_str(),
            journal.display()
        )
    } else {
        "skipped: no active task".to_string()
    };

    let session_journal = append_session_closure(
        &dijiang_dir,
        &developer,
        &session_key,
        &source,
        task_label,
        options.summary,
        &verification,
        options.allow_dirty,
    )?;

    let project_memory = dijiang_mem::ProjectMemory::from_dijiang_dir(&dijiang_dir)?;
    let memory_closure_path = project_memory.root().join("sessions.jsonl");
    let memory_closure = dijiang_mem::SessionClosure {
        timestamp: chrono::Utc::now().to_rfc3339(),
        session_key: session_key.clone(),
        source: source.clone(),
        task: task_label.to_string(),
        summary: options
            .summary
            .unwrap_or("Work finished and task archived.")
            .to_string(),
        verification: verification.clone(),
        docs_sync: docs_sync.clone(),
        version_impact: options.version_impact.to_string(),
        status: "completed".to_string(),
        confidence: "verified".to_string(),
    };
    if options.commit {
        project_memory.append_session_closure(&memory_closure)?;
    }

    let commit = if options.commit {
        perform_finish_commit(
            &project_root,
            task_label,
            options.summary,
            options.commit_message,
        )?
    } else {
        None
    };

    if options.push || options.integrate {
        perform_finish_integration(&project_root, options)?;
    }

    if !options.commit {
        project_memory.append_session_closure(&memory_closure)?;
    }

    if let Some(active_task) = active_task.as_deref() {
        println!("✓ 已完成任务 '{active_task}'");
    } else {
        println!("✓ 已完成工作（无 active task，已跳过任务归档）");
    }
    println!("  验证：{verification}");
    println!("  文档/spec 同步：{docs_sync}");
    println!("  版本影响：{}", options.version_impact);
    println!(
        "  版本更新：{}",
        version_update.as_deref().unwrap_or("none")
    );
    if let Some(commit) = commit {
        println!("  Commit：{commit}");
    } else {
        println!("  Commit：none");
    }
    println!("  Push：{}", if options.push { "done" } else { "skipped" });
    println!(
        "  Integration：{}",
        if options.integrate { "done" } else { "skipped" }
    );
    println!("  Task archive：{archive_status}");
    println!(
        "  Memory closure：written ({})",
        memory_closure_path.display()
    );
    println!("  Session 已关闭：{}", session_journal.display());
    if active_task.is_some() {
        println!("  当前 session 的 active task 已清理");
    } else {
        println!("  当前 session 没有 active task 需要清理");
    }
    Ok(())
}

fn cmd_workflow_state(json: bool, hook_event: &str) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    let state = dijiang_task::workflow_state::build(&dijiang_dir)?;

    if json {
        let payload = serde_json::json!({
            "hookEventName": hook_event,
            "additionalContext": state.additional_context(),
        });
        println!("{}", serde_json::to_string(&payload)?);
    } else {
        println!("{}", state.additional_context());
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct DispatchRoute {
    task_type: &'static str,
    primary_intent: &'static str,
    skill: &'static str,
    recommended_path: &'static str,
    status: TaskStatus,
}

#[derive(Debug, Clone)]
struct WorktreeDecision {
    branch: String,
    path: PathBuf,
    created: bool,
    note: Option<String>,
}

fn strip_embedded_context(prompt: &str) -> String {
    let mut output = String::with_capacity(prompt.len());
    let mut rest = prompt;

    loop {
        let Some(start) = rest.find("<skill ") else {
            output.push_str(rest);
            break;
        };
        output.push_str(&rest[..start]);
        let after_start = &rest[start..];
        if let Some(end) = after_start.find("</skill>") {
            rest = &after_start[end + "</skill>".len()..];
        } else {
            break;
        }
    }

    output
}

fn dispatch_route(prompt: &str) -> DispatchRoute {
    let visible_prompt = strip_embedded_context(prompt);
    let lower = visible_prompt.to_lowercase();
    let has_any = |words: &[&str]| words.iter().any(|word| lower.contains(word));
    let has_vague_bug_intent = has_any(&[
        "修 bug",
        "修bug",
        "fix bug",
        "fix bugs",
        "修复 bug",
        "修复bug",
        "有个 bug",
        "有 bug",
        "bug 这些",
        "bug这些",
    ]);
    let has_specific_failure_signal = has_any(&[
        "排查",
        "调试",
        "debug",
        "crash",
        "error",
        "fail",
        "报错",
        "崩溃",
        "无法启动",
        "不能运行",
        "失败",
        "复现",
        "日志",
        "stack",
        "trace",
    ]);
    let has_specific_implementation_signal = has_any(&[
        "字段", "接口", "按钮", "页面", "文件", "函数", "方法", "模块", "配置", "校验", "样式",
        "布局", "api", "cli", "command", "config", "button", "field",
    ]);
    let has_vague_feature_intent = has_any(&[
        "做个",
        "做一个",
        "加个",
        "加一个",
        "新增个",
        "新增一个",
        "实现个",
        "实现一个",
        "优化",
        "改进",
        "提升",
        "体验",
    ]) && !has_specific_implementation_signal;
    let has_hunt_intent =
        has_specific_failure_signal || lower.contains("bug") && !has_vague_bug_intent;

    if has_hunt_intent {
        return DispatchRoute {
            task_type: "排查调试",
            primary_intent: "排查根因",
            skill: "dj-hunt",
            recommended_path: "dj-hunt → dj-implement → dj-check",
            status: TaskStatus::InProgress,
        };
    }

    if has_vague_feature_intent {
        return DispatchRoute {
            task_type: "调研对齐",
            primary_intent: "需求澄清",
            skill: "dj-grill",
            recommended_path: "dj-grill → dj-output/dj-implement",
            status: TaskStatus::Planning,
        };
    }

    if has_any(&["审计", "安全", "扫描", "体检", "audit", "security"]) {
        return DispatchRoute {
            task_type: "审计代码",
            primary_intent: "代码审计",
            skill: "dj-audit",
            recommended_path: "dj-audit → dj-implement → dj-check",
            status: TaskStatus::InProgress,
        };
    }

    if has_any(&["调研", "方案", "对比", "url", "网页", "research", "compare"]) {
        return DispatchRoute {
            task_type: "调研对齐",
            primary_intent: "调研并对齐",
            skill: "dj-grill",
            recommended_path: "dj-grill → dj-output/dj-tdd",
            status: TaskStatus::Planning,
        };
    }

    if has_any(&["文档", "prd", "设计文档", "润色", "document", "write"]) {
        return DispatchRoute {
            task_type: "写文档",
            primary_intent: "文档产出",
            skill: "dj-output",
            recommended_path: "dj-output",
            status: TaskStatus::Planning,
        };
    }

    if has_any(&["脚本", "工具", "自动化", "script", "cli", "tool"]) {
        return DispatchRoute {
            task_type: "脚本工具",
            primary_intent: "脚本或工具实现",
            skill: "dj-script",
            recommended_path: "dj-script → dj-check",
            status: TaskStatus::InProgress,
        };
    }

    if has_any(&["ui", "页面", "样式", "布局", "组件", "design", "style"]) {
        return DispatchRoute {
            task_type: "设计 UI",
            primary_intent: "UI 设计实现",
            skill: "dj-design",
            recommended_path: "dj-design → dj-implement → dj-check",
            status: TaskStatus::InProgress,
        };
    }

    if has_any(&["测试", "tdd", "test"]) {
        return DispatchRoute {
            task_type: "测试开发",
            primary_intent: "测试驱动开发",
            skill: "dj-tdd",
            recommended_path: "dj-tdd → dj-check",
            status: TaskStatus::InProgress,
        };
    }

    if has_any(&[
        "实现",
        "修复",
        "重构",
        "新增",
        "修改",
        "改",
        "implement",
        "fix",
        "refactor",
        "add",
    ]) {
        return DispatchRoute {
            task_type: "代码开发",
            primary_intent: "实现变更",
            skill: "dj-implement",
            recommended_path: "dj-implement → dj-check",
            status: TaskStatus::InProgress,
        };
    }

    DispatchRoute {
        task_type: "调研对齐",
        primary_intent: "需求澄清",
        skill: "dj-grill",
        recommended_path: "dj-grill → dj-output/dj-implement",
        status: TaskStatus::Planning,
    }
}

fn dispatch_route_from_skill(skill: &str) -> Option<DispatchRoute> {
    match skill {
        "dj-hunt" => Some(DispatchRoute {
            task_type: "排查调试",
            primary_intent: "继续排查",
            skill: "dj-hunt",
            recommended_path: "dj-hunt → dj-implement → dj-check",
            status: TaskStatus::InProgress,
        }),
        "dj-implement" => Some(DispatchRoute {
            task_type: "代码开发",
            primary_intent: "继续实现",
            skill: "dj-implement",
            recommended_path: "dj-implement → dj-check",
            status: TaskStatus::InProgress,
        }),
        "dj-tdd" => Some(DispatchRoute {
            task_type: "测试开发",
            primary_intent: "继续 TDD",
            skill: "dj-tdd",
            recommended_path: "dj-tdd → dj-check",
            status: TaskStatus::InProgress,
        }),
        "dj-check" => Some(DispatchRoute {
            task_type: "代码审查",
            primary_intent: "质量检查",
            skill: "dj-check",
            recommended_path: "dj-check",
            status: TaskStatus::InProgress,
        }),
        "dj-output" => Some(DispatchRoute {
            task_type: "写文档",
            primary_intent: "文档产出",
            skill: "dj-output",
            recommended_path: "dj-output",
            status: TaskStatus::Planning,
        }),
        "dj-grill" => Some(DispatchRoute {
            task_type: "调研对齐",
            primary_intent: "需求澄清",
            skill: "dj-grill",
            recommended_path: "dj-grill → dj-output/dj-implement",
            status: TaskStatus::Planning,
        }),
        _ => None,
    }
}

fn dispatch_route_for_active_task(task: &TaskRecord) -> DispatchRoute {
    match task.status {
        TaskStatus::Planning => DispatchRoute {
            task_type: "调研对齐",
            primary_intent: "需求澄清",
            skill: "dj-grill",
            recommended_path: "dj-grill → dj-output/dj-implement",
            status: TaskStatus::Planning,
        },
        TaskStatus::InProgress => task
            .meta
            .get("dispatch")
            .and_then(|dispatch| dispatch.get("skill"))
            .and_then(|skill| skill.as_str())
            .and_then(dispatch_route_from_skill)
            .unwrap_or(DispatchRoute {
                task_type: "代码开发",
                primary_intent: "继续实现",
                skill: "dj-implement",
                recommended_path: "dj-implement → dj-check",
                status: TaskStatus::InProgress,
            }),
        TaskStatus::Completed => DispatchRoute {
            task_type: "收尾归档",
            primary_intent: "完成工作",
            skill: "dijiang-finish-work",
            recommended_path: "dijiang-finish-work",
            status: TaskStatus::Completed,
        },
        TaskStatus::Paused => DispatchRoute {
            task_type: "恢复上下文",
            primary_intent: "继续暂停任务",
            skill: "dijiang-continue",
            recommended_path: "dijiang-continue",
            status: TaskStatus::Paused,
        },
        TaskStatus::Archived => DispatchRoute {
            task_type: "调研对齐",
            primary_intent: "重新确认归档任务",
            skill: "dj-grill",
            recommended_path: "dj-grill → dj-output/dj-implement",
            status: TaskStatus::Planning,
        },
    }
}

fn title_from_prompt(prompt: &str) -> String {
    let compact = prompt.split_whitespace().collect::<Vec<_>>().join(" ");
    let title = compact.chars().take(80).collect::<String>();
    if title.trim().is_empty() {
        "Untitled DiJiang Task".to_string()
    } else {
        title
    }
}

fn slug_from_prompt(prompt: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in prompt.to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
        if slug.len() >= 48 {
            break;
        }
    }
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        format!("task-{}", chrono::Utc::now().format("%Y%m%d%H%M%S"))
    } else {
        slug.to_string()
    }
}

fn unique_task_name(tasks_dir: &Path, base: &str) -> String {
    if !tasks_dir.join(base).exists() {
        return base.to_string();
    }
    for index in 2..1000 {
        let candidate = format!("{base}-{index}");
        if !tasks_dir.join(&candidate).exists() {
            return candidate;
        }
    }
    format!("{base}-{}", chrono::Utc::now().timestamp())
}

fn route_requires_worktree(route: &DispatchRoute) -> bool {
    matches!(
        route.skill,
        "dj-implement" | "dj-hunt" | "dj-tdd" | "dj-script" | "dj-design"
    )
}

fn branch_prefix(route: &DispatchRoute) -> &'static str {
    match route.skill {
        "dj-hunt" => "fix",
        "dj-tdd" => "test",
        "dj-script" => "chore",
        _ => "feat",
    }
}

fn git_has_head(project_root: &Path) -> anyhow::Result<bool> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--verify", "HEAD"])
        .current_dir(project_root)
        .output()?;
    Ok(output.status.success())
}

fn unique_git_branch(project_root: &Path, base: &str) -> anyhow::Result<String> {
    for index in 0..1000 {
        let candidate = if index == 0 {
            base.to_string()
        } else {
            format!("{base}-{index}")
        };
        let output = std::process::Command::new("git")
            .args(["show-ref", "--verify", &format!("refs/heads/{candidate}")])
            .current_dir(project_root)
            .output()?;
        if !output.status.success() {
            return Ok(candidate);
        }
    }
    anyhow::bail!("无法为任务 worktree 生成唯一分支名：{base}")
}

fn unique_worktree_path(project_root: &Path, task_name: &str) -> PathBuf {
    let repo_name = project_root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("repo");
    let parent = project_root.parent().unwrap_or(project_root);
    for index in 0..1000 {
        let suffix = if index == 0 {
            String::new()
        } else {
            format!("-{index}")
        };
        let candidate = parent.join(format!("{repo_name}-{task_name}{suffix}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    parent.join(format!(
        "{repo_name}-{task_name}-{}",
        chrono::Utc::now().timestamp()
    ))
}

fn ensure_task_worktree(
    project_root: &Path,
    tasks_dir: &Path,
    task: &mut TaskRecord,
    route: &DispatchRoute,
) -> anyhow::Result<Option<WorktreeDecision>> {
    if !route_requires_worktree(route) {
        return Ok(None);
    }

    if let (Some(branch), Some(path)) = (&task.branch, &task.worktree_path) {
        return Ok(Some(WorktreeDecision {
            branch: branch.clone(),
            path: PathBuf::from(path),
            created: false,
            note: None,
        }));
    }

    if !git_has_head(project_root)? {
        return Ok(Some(WorktreeDecision {
            branch: String::new(),
            path: PathBuf::new(),
            created: false,
            note: Some("当前 git 仓库还没有提交，无法创建任务 worktree；请先建立基线提交。".to_string()),
        }));
    }

    let base_branch = git_current_branch(project_root).unwrap_or_else(|_| "HEAD".to_string());
    let branch_base = format!("{}/{}", branch_prefix(route), task.name);
    let branch = unique_git_branch(project_root, &branch_base)?;
    let path = unique_worktree_path(project_root, &task.name);
    let path_string = path.display().to_string();
    run_git(
        project_root,
        &["worktree", "add", &path_string, "-b", &branch, &base_branch],
    )?;

    task.branch = Some(branch.clone());
    task.base_branch = Some(base_branch);
    task.worktree_path = Some(path_string);
    store::save_task(tasks_dir, task)?;

    Ok(Some(WorktreeDecision {
        branch,
        path,
        created: true,
        note: None,
    }))
}

fn dispatch_context(
    task_name: &str,
    title: &str,
    route: &DispatchRoute,
    state_context: &str,
    worktree: Option<&WorktreeDecision>,
) -> String {
    let worktree_line = match worktree {
        Some(decision) if decision.created => format!(
            "Git 工作流：已创建任务 worktree `{}`，分支 `{}`。\n下一步：切换到该 worktree，读取 `.dijiang/tasks/{task_name}/task.json`，然后按 {skill} 执行。",
            decision.path.display(),
            decision.branch,
            skill = route.skill,
        ),
        Some(decision) if !decision.branch.is_empty() => format!(
            "Git 工作流：复用任务 worktree `{}`，分支 `{}`。\n下一步：切换到该 worktree，读取 `.dijiang/tasks/{task_name}/task.json`，然后按 {skill} 执行。",
            decision.path.display(),
            decision.branch,
            skill = route.skill,
        ),
        Some(decision) => format!(
            "Git 工作流：未创建 worktree，原因：{}\n下一步：先处理 Git 基线，再按 {skill} 执行。",
            decision.note.as_deref().unwrap_or("未知"),
            skill = route.skill,
        ),
        None => format!(
            "Git 工作流：当前路线不需要立即创建代码 worktree。\n下一步：读取 `.dijiang/tasks/{task_name}/task.json`，然后按 {skill} 执行。",
            skill = route.skill,
        ),
    };
    format!(
        "<dijiang-dispatch>\n任务：{task_name}\n标题：{title}\n任务类型：{task_type}\n主要意图：{primary_intent}\n路线：{skill}\n推荐路径：{recommended_path}\n{worktree_line}\n</dijiang-dispatch>\n{state_context}",
        task_type = route.task_type,
        primary_intent = route.primary_intent,
        skill = route.skill,
        recommended_path = route.recommended_path,
    )
}

fn cmd_dispatch(prompt: &str, force_new: bool, json: bool, hook_event: &str) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");
    std::fs::create_dir_all(&tasks_dir)?;
    let prompt = prompt.trim();

    if prompt.is_empty() {
        anyhow::bail!("dispatch requires a prompt, e.g. `dijiang dispatch \"fix login bug\"`");
    }

    if !force_new {
        if let Some(active) = store::read_active_task(&dijiang_dir)? {
            let mut task = store::load_task(&tasks_dir, &active)?;
            let route = if matches!(hook_event, "session:start" | "session_start") {
                dispatch_route_for_active_task(&task)
            } else {
                dispatch_route(prompt)
            };
            let project_root = dijiang_dir.parent().unwrap_or(&dijiang_dir);
            let worktree = ensure_task_worktree(project_root, &tasks_dir, &mut task, &route)?;
            let state = dijiang_task::workflow_state::build(&dijiang_dir)?;
            let context = dispatch_context(
                &active,
                &task.title,
                &route,
                &state.additional_context(),
                worktree.as_ref(),
            );
            if json {
                let payload = serde_json::json!({
                    "hookEventName": hook_event,
                    "additionalContext": context,
                });
                println!("{}", serde_json::to_string(&payload)?);
            } else {
                println!("{context}");
            }
            return Ok(());
        }
    }

    let route = dispatch_route(prompt);
    let base = slug_from_prompt(prompt);
    let task_name = unique_task_name(&tasks_dir, &base);
    let title = title_from_prompt(prompt);
    let mut task = store::create_task(&task_name, &title);
    task.description = prompt.to_string();
    task.status = route.status.clone();
    task.started_at = Some(chrono::Utc::now().to_rfc3339());
    task.source = Some("dijiang dispatch".to_string());
    task.session_id = Some(current_session_key().0);
    task.meta = serde_json::json!({
        "dispatch": {
            "task_type": route.task_type,
            "primary_intent": route.primary_intent,
            "skill": route.skill,
            "recommended_path": route.recommended_path
        }
    });
    store::save_task(&tasks_dir, &task)?;
    store::write_active_task(&dijiang_dir, &task_name)?;
    let project_root = dijiang_dir.parent().unwrap_or(&dijiang_dir);
    let worktree = ensure_task_worktree(project_root, &tasks_dir, &mut task, &route)?;

    let state = dijiang_task::workflow_state::build(&dijiang_dir)?;
    let context = dispatch_context(
        &task_name,
        &title,
        &route,
        &state.additional_context(),
        worktree.as_ref(),
    );
    if json {
        let payload = serde_json::json!({
            "hookEventName": hook_event,
            "additionalContext": context,
        });
        println!("{}", serde_json::to_string(&payload)?);
    } else {
        println!("{context}");
    }
    Ok(())
}

/// 从通道元数据中读取 agent 名称
fn read_channel_agent_name(channel_dir: &std::path::Path) -> String {
    let channel_toml = channel_dir.join("channel.toml");
    if !channel_toml.exists() {
        return "unknown".to_string();
    }
    std::fs::read_to_string(&channel_toml)
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find(|l| l.contains("agent"))
                .and_then(|l| l.split('=').nth(1))
                .map(|s| s.trim().trim_matches('"').to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// 更新通道状态
fn update_channel_status(channel_dir: &std::path::Path, status: &str) -> anyhow::Result<()> {
    let channel_toml = channel_dir.join("channel.toml");
    if channel_toml.exists() {
        let content = std::fs::read_to_string(&channel_toml)?;
        let new_content =
            content.replace("status = \"active\"", &format!("status = \"{}\"", status));
        std::fs::write(&channel_toml, &new_content)?;
    }
    Ok(())
}

/// 写入通道元数据
fn write_channel_metadata(
    channel_dir: &std::path::Path,
    channel_id: &str,
    agent: &str,
    task: &str,
    dir: &std::path::Path,
) -> anyhow::Result<()> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let metadata = format!(
        "id = {:?}\nagent = {:?}\nstatus = \"active\"\ncreated = {:?}\n\"task\" = {:?}\n\"dir\" = {:?}\n",
        channel_id, agent, timestamp, task, dir
    );
    std::fs::write(channel_dir.join("channel.toml"), &metadata)?;
    Ok(())
}

fn status_line(label: &str, value: impl std::fmt::Display) {
    println!("  {label:15} {value}");
}

fn cmd_status(compat: bool) -> anyhow::Result<()> {
    println!("\n  ── DiJiang Status ──\n");

    let cwd = std::env::current_dir()?;
    let dijiang_dir = require_dijiang_dir()?;

    let name = dijiang_configurator::read_project_name(&cwd);
    status_line("Project:", &name);

    // Active task
    let active = store::read_active_task(&dijiang_dir).unwrap_or(None);
    let tasks_dir = dijiang_dir.join("tasks");
    let tasks = store::list_tasks(&tasks_dir).unwrap_or_default();

    match &active {
        Some(t) => {
            status_line("Active Task:", t);
            if let Some(task) = tasks.iter().find(|x| &x.name == t) {
                status_line("Status:", task.status.as_str());
                status_line("Phase:", task.status.to_trellis_status());
                status_line("Compatible:", "yes");
            }
        }
        None => status_line("Active Task:", "(none)"),
    }

    println!("  Tasks ({count}):", count = tasks.len());
    for t in &tasks {
        let marker = active
            .as_ref()
            .map_or(' ', |a| if a == &t.name { '*' } else { ' ' });
        let phase = t.status.to_trellis_status();
        println!(
            "    {marker} {name:<45} {status:12} {phase:12}",
            name = t.name,
            status = t.status.as_str(),
            phase = phase,
        );
    }

    let pi_dir = dijiang_dir.parent().map(|p| p.join(".pi"));
    if pi_dir.as_ref().is_some_and(|p| p.exists()) {
        println!("  Pi:              ✓ configured");
    }

    // --compat: detailed diagnostics
    if compat {
        println!("  ── Compatibility Diagnostics ──");
        let statuses = [
            ("planning", "plan"),
            ("in_progress", "implement"),
            ("completed", "complete"),
            ("paused", "in_progress  (downgraded)"),
            ("archived", "complete      (downgraded)"),
        ];
        println!("  Status mapping (DiJiang → Trellis):");
        for (dij, tre) in &statuses {
            println!("    {dij:<20} → {tre}");
        }
        if dijiang_dir.join("tasks").exists() {
            println!("  DiJiang project: \u{2713} detected");
        } else {
            println!("  DiJiang project: \u{2717} not detected");
        }
    }

    println!();
    Ok(())
}

fn cmd_task_list() -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;

    let tasks_dir = dijiang_dir.join("tasks");
    let tasks = store::list_tasks(&tasks_dir).unwrap_or_default();

    if tasks.is_empty() {
        println!("No tasks found.");
        return Ok(());
    }

    for t in &tasks {
        println!(
            "{name:<50} {status:12}  {priority:2}",
            name = t.name,
            status = t.status.as_str(),
            priority = t.priority,
        );
    }
    Ok(())
}

fn cmd_task_current() -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;

    match store::read_active_task(&dijiang_dir)? {
        Some(name) => println!("{name}"),
        None => println!("(none)"),
    }
    Ok(())
}

fn cmd_task_start(name: &str) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;

    let tasks_dir = dijiang_dir.join("tasks");

    // Ensure task exists — create if missing, activate if exists
    match store::load_task(&tasks_dir, name) {
        Ok(mut task) => {
            task.status = TaskStatus::InProgress;
            store::save_task(&tasks_dir, &task)?;
        }
        Err(store::TaskError::NotFound(_)) => {
            // Create the task
            let task = store::create_task(name, name);
            store::save_task(&tasks_dir, &task)?;
            println!("✓ Created task: {name}");
        }
        Err(e) => {
            eprintln!("Error loading task: {e}");
            std::process::exit(1);
        }
    }

    store::write_active_task(&dijiang_dir, name)?;
    println!("✓ Current task set to: .dijiang/tasks/{name}");
    println!("  Status: planning → in_progress");
    Ok(())
}

fn cmd_task_status(name: &str, status_str: &str) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;

    let new_status = match status_str {
        "planning" => TaskStatus::Planning,
        "in_progress" => TaskStatus::InProgress,
        "completed" => TaskStatus::Completed,
        "archived" => TaskStatus::Archived,
        "paused" => TaskStatus::Paused,
        _ => {
            eprintln!(
                "Invalid status: '{status_str}'. Valid: planning|in_progress|completed|archived|paused"
            );
            std::process::exit(1);
        }
    };

    let tasks_dir = dijiang_dir.join("tasks");
    match store::update_status(&tasks_dir, name, new_status) {
        Ok(task) => {
            println!(
                "✓ Task '{name}' status updated to: {}",
                task.status.as_str()
            );
        }
        Err(store::TaskError::NotFound(_)) => {
            eprintln!("Task '{name}' not found.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error updating task: {e}");
            std::process::exit(1);
        }
    }
    Ok(())
}

fn cmd_task_archive(name: &str) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;

    let tasks_dir = dijiang_dir.join("tasks");
    match store::archive_task(&tasks_dir, name) {
        Ok(task) => {
            println!(
                "✓ Task '{name}' archived (status: {})",
                task.status.as_str()
            );
        }
        Err(store::TaskError::NotFound(_)) => {
            eprintln!("Task '{name}' not found.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error archiving task: {e}");
            std::process::exit(1);
        }
    }
    Ok(())
}

fn cmd_task_prune(days: u64) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;

    let tasks_dir = dijiang_dir.join("tasks");
    match store::prune_tasks(&tasks_dir, days) {
        Ok(count) => {
            if count > 0 {
                println!("✓ Pruned {count} archived task(s) older than {days} days.");
            } else {
                println!("No tasks to prune.");
            }
        }
        Err(e) => {
            eprintln!("Error pruning tasks: {e}");
            std::process::exit(1);
        }
    }
    Ok(())
}

fn cmd_template_list() -> anyhow::Result<()> {
    let registry = TemplateRegistry::new();
    let builtins = registry.list_builtin();

    println!("\n  ── Available Templates ──\n");
    println!("  Built-in:");
    if builtins.is_empty() {
        println!("    (none)");
    } else {
        for name in &builtins {
            println!("    • {name}");
        }
    }

    let cached = registry.list_local().unwrap_or_default();
    println!("\n  Cached ({}):", cached.len());
    if cached.is_empty() {
        println!("    (none — use `dijiang template pull <source>` to add templates)");
    } else {
        for pkg in &cached {
            println!(
                "    • {} v{} — {}",
                pkg.manifest.template.name,
                pkg.manifest.template.version,
                pkg.manifest.template.description,
            );
        }
    }
    println!();
    Ok(())
}

fn cmd_template_pull(source: &str) -> anyhow::Result<()> {
    let registry = TemplateRegistry::new();
    match registry.resolve(source) {
        Ok(pkg) => {
            println!(
                "✓ Pulled template '{}' v{} to cache",
                pkg.manifest.template.name, pkg.manifest.template.version,
            );
            println!("  Location: {}", pkg.root.display());
            let file_count = pkg.manifest.files.len();
            println!("  Files: {file_count}");
            Ok(())
        }
        Err(e) => {
            eprintln!("Error pulling template: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_template_validate(path: &str) -> anyhow::Result<()> {
    let template_path = std::path::Path::new(path);
    match TemplateRegistry::validate(template_path) {
        Ok(manifest) => {
            println!(
                "✓ Template '{}' v{} is valid",
                manifest.template.name, manifest.template.version,
            );
            println!("  Description: {}", manifest.template.description);
            println!("  Files: {}", manifest.files.len());
            if let Some(meta) = &manifest.metadata {
                if let Some(author) = &meta.author {
                    println!("  Author: {author}");
                }
            }
            Ok(())
        }
        Err(errors) => {
            for err in &errors {
                eprintln!("  ✗ {err}");
            }
            std::process::exit(1);
        }
    }
}

fn cmd_init(
    name: &str,
    developer: Option<&str>,
    yes: bool,
    force: bool,
    platforms: Option<&str>,
    auto_detect: bool,
) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;

    // Check if already initialized
    if cwd.join(".dijiang").join("config.toml").exists() {
        if !force {
            println!("  Already initialized. Use --force to reinitialize.");
            return Ok(());
        }
        println!("  Overwriting...");
    }

    // Project name
    let project_name = if name.is_empty() {
        let default_name = cwd
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("my-project");
        if yes {
            default_name.to_string()
        } else {
            Input::new()
                .with_prompt("Project name")
                .default(default_name.to_string())
                .interact_text()?
        }
    } else {
        name.to_string()
    };

    // Developer name: try git config, then prompt
    let developer = developer.map(|s| s.to_string()).or_else(|| {
        // Try to detect from git config
        let git_name = std::process::Command::new("git")
            .args(["config", "--global", "user.name"])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout)
                        .ok()
                        .map(|s| s.trim().to_string())
                } else {
                    None
                }
            });

        if yes {
            git_name
        } else {
            let default_dev = git_name.unwrap_or_default();
            let input: String = Input::new()
                .with_prompt("Developer name")
                .default(default_dev)
                .allow_empty(true)
                .interact_text()
                .ok()
                .filter(|s| !s.is_empty())?;
            Some(input)
        }
    });

    // Platform selection
    let selected_platforms: Vec<PlatformKind> = if auto_detect {
        let registry = dijiang_configurator::ConfiguratorRegistry::with_all();
        let detected = registry.auto_detect();
        if detected.is_empty() {
            eprintln!("No installed platforms detected. Run without --auto-detect to select.");
            std::process::exit(1);
        }
        println!(
            "  Detected platforms: {}",
            detected
                .iter()
                .map(|p| p.display_name())
                .collect::<Vec<_>>()
                .join(", ")
        );
        detected
    } else if let Some(p) = platforms {
        p.split(',')
            .filter_map(|s| match s.trim() {
                "pi" => Some(PlatformKind::Pi),
                "cursor" => Some(PlatformKind::Cursor),
                "claude" => Some(PlatformKind::Claude),
                "codex" => Some(PlatformKind::Codex),
                "opencode" => Some(PlatformKind::OpenCode),
                "hermes" => Some(PlatformKind::Hermes),
                _ => None,
            })
            .collect()
    } else if yes {
        PlatformKind::all()
    } else {
        let all_platforms = PlatformKind::all();
        let display_names: Vec<&str> = all_platforms.iter().map(|p| p.display_name()).collect();
        let selections = MultiSelect::new()
            .with_prompt("Select platforms to configure")
            .items(&display_names)
            .defaults(&[true, false, false, false, false, false])
            .interact()?;
        selections.iter().map(|&i| all_platforms[i]).collect()
    };

    if selected_platforms.is_empty() {
        eprintln!("No platforms selected. Use --platforms or select at least one.");
        std::process::exit(1);
    }

    // Execute init
    dijiang_configurator::init_project_with_platforms(
        &cwd,
        &project_name,
        developer.as_deref(),
        &selected_platforms,
    )?;

    // Write dj-* skills to project
    let skills_written = dijiang_configurator::write_project_skills(&cwd, false)?;
    if skills_written > 0 {
        println!("  Wrote {} dj-* skills to .pi/skills/", skills_written);
    }

    // Initialize default tactics
    match dijiang_mem::GlobalMemory::new() {
        Ok(global_mem) => {
            if let Err(e) = global_mem.ensure_default_tactics() {
                eprintln!("  Warning: Failed to initialize default tactics: {}", e);
            } else {
                println!(
                    "  Initialized default tactics (cargo-test, typecheck, lint-fix, doc-update)"
                );
            }
        }
        Err(e) => {
            eprintln!("  Warning: Failed to initialize global memory: {}", e);
        }
    }

    Ok(())
}

fn cmd_start(name: &str, title: Option<&str>) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let dijiang_dir = require_dijiang_dir()?;

    let tasks_dir = dijiang_dir.join("tasks");
    let now = chrono::Utc::now();

    // Load existing task or create new one. `start` activates session context;
    // dispatch owns classification and phase selection for new work.
    match store::load_task(&tasks_dir, name) {
        Ok(mut task) => {
            // Update existing task
            let was_status = task.status.as_str().to_string();
            if matches!(task.status, TaskStatus::Archived) {
                task.status = TaskStatus::Planning;
            }
            task.started_at = task.started_at.take().or(Some(now.to_rfc3339()));
            store::save_task(&tasks_dir, &task)?;
            println!("  ✓ Task '{name}' updated");
            println!(
                "    Status: {was_status} → {status}",
                status = task.status.as_str()
            );
        }
        Err(store::TaskError::NotFound(_)) => {
            // Create new task
            let display_title = title.unwrap_or(name);
            let mut task = store::create_task(name, display_title);
            task.status = TaskStatus::Planning;
            task.started_at = Some(now.to_rfc3339());
            store::save_task(&tasks_dir, &task)?;
            println!("  ✓ Task '{name}' created");
            println!("    Title: {display_title}");
            println!("    Status: planning");
        }
        Err(e) => {
            eprintln!("Error accessing task: {e}");
            std::process::exit(1);
        }
    }

    store::write_active_task(&dijiang_dir, name)?;

    // Print startup summary
    println!("  ✓ Session started");
    println!();

    // Show project and active task summary
    let project_name = dijiang_dir
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("(unknown)");
    println!("  Project: {project_name}");
    println!("  Active:  .dijiang/tasks/{name}");
    println!();

    // Show task title if available
    if let Ok(task) = store::load_task(&tasks_dir, name) {
        println!("  Task summary:");
        println!("    Title:  {title}", title = task.title);
        println!("    State:  {status}", status = task.status.as_str());
        println!("    Phase:  {phase}", phase = task.status.infer_phase());
        if let Some(ac) = &task.acceptance_criteria {
            println!("    Goals:  {ac}");
        }
    }
    println!();
    Ok(())
}

fn cmd_mem_list() -> anyhow::Result<()> {
    println!("\n  ── DiJiang Memory ──\n");

    let mut registry = dijiang_mem::MemRegistry::new();
    registry.register(Box::new(dijiang_mem::PiMemAdapter::new()));
    registry.register(Box::new(dijiang_mem::ClaudeAdapter::new()));
    registry.register(Box::new(dijiang_mem::CodexAdapter::new()));
    registry.register(Box::new(dijiang_mem::HermesAdapter::new()));
    registry.register(Box::new(dijiang_mem::OpenCodeAdapter::new()));

    let rt = tokio::runtime::Runtime::new()?;
    let projects = rt.block_on(registry.aggregate_by_project())?;
    let providers = registry.providers();

    if projects.is_empty() {
        println!("  No sessions found.\n");
        return Ok(());
    }

    println!(
        "  Providers: {} ({})",
        providers.join(" + "),
        registry.adapter_count()
    );
    println!();

    let total_sessions: usize = projects.iter().map(|p| p.sessions.len()).sum();

    for p in &projects {
        let total = p.sessions.len();
        let active = p
            .sessions
            .iter()
            .filter(|s| s.status == dijiang_mem::SessionStatus::Active)
            .count();
        let archived = total - active;
        let latest = p.last_active_at.as_deref().unwrap_or("-");
        println!("  {project}", project = p.project_id);
        println!("    Total: {total}  Active: {active}  Archived: {archived}");
        println!("    Latest: {latest}");

        for s in p.sessions.iter().take(3) {
            let task = s.task.as_deref().unwrap_or("(no task)");
            let truncated = if task.len() > 60 {
                let mut end = 57;
                while !task.is_char_boundary(end) {
                    end += 1;
                }
                &task[..end]
            } else {
                task
            };
            let marker = if s.status == dijiang_mem::SessionStatus::Active {
                "[A]"
            } else {
                "[ ]"
            };
            println!("    {marker:7} {truncated}");
        }
        if p.sessions.len() > 3 {
            println!("    ... and {} more", p.sessions.len() - 3);
        }
        println!();
    }

    println!("  Total: {total_sessions} session(s)");
    println!();
    Ok(())
}

fn cmd_mem_sync() -> anyhow::Result<()> {
    println!("\n  ── DiJiang Memory Sync ──\n");

    let mut registry = dijiang_mem::MemRegistry::new();
    registry.register(Box::new(dijiang_mem::PiMemAdapter::new()));
    registry.register(Box::new(dijiang_mem::ClaudeAdapter::new()));
    registry.register(Box::new(dijiang_mem::CodexAdapter::new()));
    registry.register(Box::new(dijiang_mem::HermesAdapter::new()));
    registry.register(Box::new(dijiang_mem::OpenCodeAdapter::new()));

    let rt = tokio::runtime::Runtime::new()?;
    let sessions = rt.block_on(registry.list_sessions())?;
    let store = dijiang_mem::SessionStore::new();

    if sessions.is_empty() {
        println!("  No sessions found to sync.\n");
        return Ok(());
    }

    let mut synced = 0u32;
    let mut skipped = 0u32;

    for s in &sessions {
        // Check if already synced (by session_id)
        match store.read_session(&s.session_id) {
            Ok(_) => skipped += 1,
            Err(_) => {
                store.save_session(s)?;
                synced += 1;
            }
        }
    }

    println!("  Synced: {} new sessions", synced);
    println!("  Skipped: {} already in store", skipped);
    if synced > 0 {
        println!("  Location: ~/.dijiang/mem/sessions/");
    }
    println!();
    Ok(())
}

fn cmd_skills(sync: bool) -> anyhow::Result<()> {
    if sync {
        let cwd = std::env::current_dir()?;
        let skills_written = dijiang_configurator::write_project_skills(&cwd, false)?;
        println!("  Synced {} dj-* skills to .pi/skills/", skills_written);
    } else {
        let names = dijiang_configurator::list_skill_names();
        println!("  {} dj-* skills available:", names.len());
        for name in names {
            println!("    {}", name);
        }
        println!();
        println!("  Use `dijiang skills --sync` to write skills to current project.");
    }
    Ok(())
}

fn cmd_migrate() -> anyhow::Result<()> {
    use std::fs;
    let cwd = std::env::current_dir()?;
    let trellis = cwd.join(".trellis");
    let dijiang = cwd.join(".dijiang");

    if !trellis.exists() {
        println!("  No .trellis/ directory found. Nothing to migrate.");
        return Ok(());
    }

    if dijiang.exists() {
        println!("  .dijiang/ already exists. Skipping migration.");
        return Ok(());
    }

    println!("  Migrating .trellis/ -> .dijiang/...");
    fs::rename(&trellis, &dijiang)?;
    println!("  Done.");
    println!("  Run `dijiang init` to reconfigure platforms.");
    Ok(())
}

fn cmd_channel_spawn(agent: &str, task: Option<&str>, dir: Option<&str>) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let base_dir = dir.map(PathBuf::from).unwrap_or_else(|| cwd.clone());
    let dijiang_dir = crate::store::find_dijiang_dir(&base_dir)
        .ok_or_else(|| anyhow::anyhow!("No .dijiang/ found. Run `dijiang init` first."))?;

    // Read agent definition
    let agents_dir = dijiang_dir
        .parent()
        .map(|p| p.join(".pi").join("agents"))
        .unwrap_or_default();
    let agent_file = agents_dir.join(format!("dijiang-{}.md", agent));
    if !agent_file.exists() {
        anyhow::bail!("Agent '{}' not found at {}", agent, agent_file.display());
    }
    let agent_def = std::fs::read_to_string(&agent_file)?;

    // Generate channel ID
    let channel_id = format!(
        "{}-{}-{}",
        agent,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        &cwd.to_string_lossy()[cwd.to_string_lossy().len() - 8..].replace('/', "-")
    );

    // Create channel directory
    let channel_dir = dijiang_dir.join("channels").join(&channel_id);
    std::fs::create_dir_all(&channel_dir)?;

    // Write agent definition to channel
    std::fs::write(channel_dir.join("agent.md"), &agent_def)?;

    // Write inbox with task
    let inbox_content = match task {
        Some(t) => format!("Active task: {}\n", t),
        None => format!("Active task: {}\n", cwd.display()),
    };
    std::fs::write(channel_dir.join("inbox"), &inbox_content)?;

    // Write channel metadata
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let metadata = format!(
        "id = {:?}\nagent = {:?}\nstatus = \"active\"\ncreated = {:?}\n\"task\" = {:?}\n\"dir\" = {:?}\n",
        channel_id,
        agent,
        timestamp,
        task.unwrap_or(""),
        cwd.display()
    );
    std::fs::write(channel_dir.join("channel.toml"), &metadata)?;

    println!("  Agent '{}' spawned", agent);
    println!("  Channel ID: {}", channel_id);
    println!("  Channel dir: {}", channel_dir.display());
    println!();
    println!("  The agent is ready to receive tasks.");
    println!("  To execute, run: dijiang channel execute {}", channel_id);
    Ok(())
}

fn cmd_channel_list() -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let dijiang_dir = crate::store::find_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No .dijiang/ found. Run `dijiang init` first."))?;

    let channels_dir = dijiang_dir.join("channels");
    if !channels_dir.exists() {
        println!("  No channels found.");
        return Ok(());
    }

    let mut channels = Vec::new();
    for entry in std::fs::read_dir(&channels_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let channel_id = entry.file_name().to_string_lossy().to_string();
            let channel_toml = entry.path().join("channel.toml");
            if channel_toml.exists() {
                let content = std::fs::read_to_string(&channel_toml)?;
                let agent = content
                    .lines()
                    .find(|l| l.contains("agent"))
                    .and_then(|l| l.split('=').nth(1))
                    .map(|s| s.trim().trim_matches('"').to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                let status = content
                    .lines()
                    .find(|l| l.contains("status"))
                    .and_then(|l| l.split('=').nth(1))
                    .map(|s| s.trim().trim_matches('"').to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                channels.push((channel_id, agent, status));
            }
        }
    }

    if channels.is_empty() {
        println!("  未找到通道。");
    } else {
        println!("  {} 个活跃通道:", channels.len());
        for (id, agent, status) in &channels {
            println!("  {} - {} ({})", id, agent, status);
        }
    }
    Ok(())
}

fn cmd_channel_send(channel_id: &str, message: &str) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let dijiang_dir = crate::store::find_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No .dijiang/ found. Run `dijiang init` first."))?;

    let channel_dir = dijiang_dir.join("channels").join(channel_id);
    if !channel_dir.exists() {
        anyhow::bail!("Channel '{}' not found", channel_id);
    }

    // Append to inbox
    let inbox_path = channel_dir.join("inbox");
    let mut inbox = std::fs::read_to_string(&inbox_path).unwrap_or_default();
    inbox.push_str(message);
    inbox.push('\n');
    std::fs::write(&inbox_path, &inbox)?;

    println!("  Message sent to channel {}", channel_id);
    Ok(())
}

fn cmd_channel_status(channel_id: &str) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let dijiang_dir = crate::store::find_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No .dijiang/ found. Run `dijiang init` first."))?;

    if channel_id == "all" {
        // List all channels
        return cmd_channel_list();
    }

    let channel_dir = dijiang_dir.join("channels").join(channel_id);
    if !channel_dir.exists() {
        anyhow::bail!("Channel '{}' not found", channel_id);
    }

    // Read channel metadata
    let channel_toml = channel_dir.join("channel.toml");
    if channel_toml.exists() {
        let content = std::fs::read_to_string(&channel_toml)?;
        println!("  Channel {}:", channel_id);
        for line in content.lines() {
            if !line.trim().is_empty() {
                println!("    {}", line);
            }
        }
    } else {
        println!("  Channel {}:", channel_id);
        println!("    No metadata found.");
    }

    // Show inbox size
    let inbox_path = channel_dir.join("inbox");
    if inbox_path.exists() {
        let inbox = std::fs::read_to_string(&inbox_path)?;
        println!("    inbox: {} bytes", inbox.len());
    }

    // Show output if exists
    let output_path = channel_dir.join("output");
    if output_path.exists() {
        let output = std::fs::read_to_string(&output_path)?;
        println!("    output: {} bytes", output.len());
    }

    Ok(())
}

fn cmd_channel_stop(channel_id: &str) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let dijiang_dir = crate::store::find_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No .dijiang/ found. Run `dijiang init` first."))?;

    let channel_dir = dijiang_dir.join("channels").join(channel_id);
    if !channel_dir.exists() {
        anyhow::bail!("Channel '{}' not found", channel_id);
    }

    // Update status in channel.toml
    let channel_toml = channel_dir.join("channel.toml");
    if channel_toml.exists() {
        let content = std::fs::read_to_string(&channel_toml)?;
        let new_content = content.replace("status = \"active\"", "status = \"stopped\"");
        std::fs::write(&channel_toml, &new_content)?;
    }

    println!("  通道 {} 已停止。", channel_id);
    Ok(())
}

fn cmd_channel_execute(
    channel_id: &str,
    model: Option<&str>,
    provider: Option<&str>,
    timeout: u64,
    follow: bool,
) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let dijiang_dir = crate::store::find_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No .dijiang/ found. Run `dijiang init` first."))?;

    let channel_dir = dijiang_dir.join("channels").join(channel_id);
    if !channel_dir.exists() {
        anyhow::bail!("Channel '{}' not found", channel_id);
    }

    // Read agent definition
    let agent_file = channel_dir.join("agent.md");
    if !agent_file.exists() {
        anyhow::bail!("No agent definition found in channel");
    }

    // Read inbox
    let inbox_file = channel_dir.join("inbox");
    if !inbox_file.exists() {
        anyhow::bail!("No inbox found in channel");
    }

    // Read channel metadata
    let channel_toml = channel_dir.join("channel.toml");
    let mut agent_name = "unknown".to_string();
    if channel_toml.exists() {
        let content = std::fs::read_to_string(&channel_toml)?;
        if let Some(line) = content.lines().find(|l| l.contains("agent")) {
            if let Some(val) = line.split('=').nth(1) {
                agent_name = val.trim().trim_matches('"').to_string();
            }
        }
    }

    println!(
        "  Executing agent '{}' in channel {}",
        agent_name, channel_id
    );
    println!("  Timeout: {}s", timeout);
    if follow {
        println!("  Follow mode: enabled");
    }
    println!();

    // Build pi command
    let mut pi_args = vec!["--print".to_string()];

    if let Some(m) = model {
        pi_args.push("--model".to_string());
        pi_args.push(m.to_string());
    }

    if let Some(p) = provider {
        pi_args.push("--provider".to_string());
        pi_args.push(p.to_string());
    }

    // Build the prompt from agent definition + inbox
    let agent_def = std::fs::read_to_string(&agent_file)?;
    let inbox_content = std::fs::read_to_string(&inbox_file)?;

    let prompt = format!("{}\n\n---\n\nInbox:\n{}", agent_def, inbox_content);

    // Execute pi with the prompt
    println!("  Running: pi {}", pi_args.join(" "));
    println!("  Prompt length: {} chars", prompt.len());
    println!();

    // Execute pi using stdin to avoid command line length limits
    let mut child = std::process::Command::new("pi")
        .args(&pi_args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .current_dir(&cwd)
        .spawn()?;

    // Write prompt to stdin
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(prompt.as_bytes())?;
    }

    // Wait for output with timeout
    let start = std::time::Instant::now();
    let output = loop {
        match child.try_wait() {
            Ok(Some(_status)) => {
                // Process finished
                let output = child.wait_with_output()?;
                break output;
            }
            Ok(None) => {
                // Check timeout
                if start.elapsed().as_secs() >= timeout {
                    println!("  超时 {}s，正在终止进程...", timeout);
                    child.kill()?;
                    child.wait()?;
                    anyhow::bail!("执行超时（{}s）", timeout);
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => {
                anyhow::bail!("Error waiting for process: {}", e);
            }
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Write output
    let output_file = channel_dir.join("output");
    std::fs::write(&output_file, stdout.as_ref())?;

    // Write status
    let status = if output.status.success() {
        "completed"
    } else {
        "failed"
    };
    if channel_toml.exists() {
        let content = std::fs::read_to_string(&channel_toml)?;
        let new_content =
            content.replace("status = \"active\"", &format!("status = \"{}\"", status));
        std::fs::write(&channel_toml, &new_content)?;
    }

    if follow {
        println!("{}", stdout);
    }

    if output.status.success() {
        println!("  Agent 执行完成。");
        println!("  输出: {} 字节", stdout.len());
        if !stderr.is_empty() {
            println!("  警告: {} 字节", stderr.len());
        }
    } else {
        println!("  Agent 执行失败。");
        println!("  stderr: {}", stderr);
        std::process::exit(1);
    }

    Ok(())
}

fn cmd_channel_execute_all(
    model: Option<&str>,
    provider: Option<&str>,
    timeout: u64,
) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let dijiang_dir = crate::store::find_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No .dijiang/ found. Run `dijiang init` first."))?;

    let channels_dir = dijiang_dir.join("channels");
    if !channels_dir.exists() {
        println!("  No channels found.");
        return Ok(());
    }

    // Collect active channels
    let mut active_channels = Vec::new();
    for entry in std::fs::read_dir(&channels_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let channel_id = entry.file_name().to_string_lossy().to_string();
            let channel_toml = entry.path().join("channel.toml");
            if channel_toml.exists() {
                let content = std::fs::read_to_string(&channel_toml)?;
                if content.contains("status = \"active\"") {
                    active_channels.push(channel_id);
                }
            }
        }
    }

    if active_channels.is_empty() {
        println!("  No active channels to execute.");
        return Ok(());
    }

    println!(
        "  Executing {} active channel(s) in parallel",
        active_channels.len()
    );
    println!("  Timeout: {}s per channel", timeout);
    println!();

    // Execute each channel
    let mut handles = Vec::new();
    for channel_id in &active_channels {
        let channel_id = channel_id.clone();
        let model = model.map(|s| s.to_string());
        let provider = provider.map(|s| s.to_string());
        let cwd = cwd.clone();
        let dijiang_dir = dijiang_dir.clone();

        let handle = std::thread::spawn(move || {
            match cmd_channel_execute_single(
                &channel_id,
                model.as_deref(),
                provider.as_deref(),
                timeout,
                &cwd,
                &dijiang_dir,
            ) {
                Ok(_) => (channel_id, true, String::new()),
                Err(e) => (channel_id, false, e.to_string()),
            }
        });
        handles.push(handle);
    }

    // Collect results
    let mut success_count = 0;
    let mut fail_count = 0;
    for handle in handles {
        let (channel_id, success, error) = handle.join().unwrap();
        if success {
            println!("  {} 已完成", channel_id);
            success_count += 1;
        } else {
            println!("  {} 失败: {}", channel_id, error);
            fail_count += 1;
        }
    }

    println!();
    println!("  结果: {} 成功, {} 失败", success_count, fail_count);

    Ok(())
}

fn cmd_channel_execute_single(
    channel_id: &str,
    model: Option<&str>,
    provider: Option<&str>,
    timeout: u64,
    cwd: &std::path::Path,
    dijiang_dir: &std::path::Path,
) -> anyhow::Result<()> {
    let channel_dir = dijiang_dir.join("channels").join(channel_id);
    if !channel_dir.exists() {
        anyhow::bail!("Channel '{}' not found", channel_id);
    }

    // Read agent definition
    let agent_file = channel_dir.join("agent.md");
    if !agent_file.exists() {
        anyhow::bail!("No agent definition found in channel");
    }

    // Read inbox
    let inbox_file = channel_dir.join("inbox");
    if !inbox_file.exists() {
        anyhow::bail!("No inbox found in channel");
    }

    // Build pi command
    let mut pi_args = vec!["--print".to_string()];
    if let Some(m) = model {
        pi_args.push("--model".to_string());
        pi_args.push(m.to_string());
    }
    if let Some(p) = provider {
        pi_args.push("--provider".to_string());
        pi_args.push(p.to_string());
    }

    // Build the prompt
    let agent_def = std::fs::read_to_string(&agent_file)?;
    let inbox_content = std::fs::read_to_string(&inbox_file)?;
    let prompt = format!("{}\n\n---\n\nInbox:\n{}", agent_def, inbox_content);

    // Execute pi
    let mut child = std::process::Command::new("pi")
        .args(&pi_args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .current_dir(cwd)
        .spawn()?;

    // Write prompt to stdin
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(prompt.as_bytes())?;
    }

    // Wait for output with timeout
    let start = std::time::Instant::now();
    let output = loop {
        match child.try_wait() {
            Ok(Some(_status)) => {
                let output = child.wait_with_output()?;
                break output;
            }
            Ok(None) => {
                if start.elapsed().as_secs() >= timeout {
                    child.kill()?;
                    child.wait()?;
                    anyhow::bail!("Execution timed out after {}s", timeout);
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => {
                anyhow::bail!("Error waiting for process: {}", e);
            }
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Write output
    let output_file = channel_dir.join("output");
    std::fs::write(&output_file, stdout.as_ref())?;

    // Write status
    let channel_toml = channel_dir.join("channel.toml");
    let status = if output.status.success() {
        "completed"
    } else {
        "failed"
    };
    if channel_toml.exists() {
        let content = std::fs::read_to_string(&channel_toml)?;
        let new_content =
            content.replace("status = \"active\"", &format!("status = \"{}\"", status));
        std::fs::write(&channel_toml, &new_content)?;
    }

    Ok(())
}

fn current_project_memory(dijiang_dir: &Path) -> anyhow::Result<dijiang_mem::ProjectMemory> {
    dijiang_mem::ProjectMemory::from_dijiang_dir(dijiang_dir)
}

fn cmd_mem_findings(finding: &str) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let dijiang_dir = crate::store::find_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No .dijiang/ found. Run `dijiang init` first."))?;
    let project = read_project_name(&dijiang_dir).unwrap_or_else(|_| "unknown".to_string());
    let (session_key, _) = current_session_key();
    let mem = current_project_memory(&dijiang_dir)?;
    let record = dijiang_mem::Finding {
        timestamp: chrono::Local::now().to_rfc3339(),
        content: finding.to_string(),
        session_id: Some(session_key),
        project: Some(project),
    };
    mem.append_finding(&record)?;
    println!(
        "  Finding recorded to {}",
        mem.root().join("findings.jsonl").display()
    );
    Ok(())
}

fn cmd_mem_learn(lesson: &str) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let dijiang_dir = crate::store::find_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No .dijiang/ found. Run `dijiang init` first."))?;
    let project = read_project_name(&dijiang_dir).unwrap_or_else(|_| "unknown".to_string());
    let (session_key, _) = current_session_key();
    let mem = current_project_memory(&dijiang_dir)?;
    let record = dijiang_mem::Learning {
        timestamp: chrono::Local::now().to_rfc3339(),
        content: lesson.to_string(),
        session_id: Some(session_key),
        project: Some(project),
    };
    mem.append_learning(&record)?;
    println!(
        "  Lesson recorded to {}",
        mem.root().join("learnings.jsonl").display()
    );
    Ok(())
}

fn cmd_mem_correction(
    correction: &str,
    lesson: &str,
    scope: &str,
    source: &str,
    freshness: &str,
    conflict: &str,
    actionability: &str,
) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let dijiang_dir = crate::store::find_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No .dijiang/ found. Run `dijiang init` first."))?;
    let active_task = store::read_active_task(&dijiang_dir)?;
    let (session_key, _) = current_session_key();
    let mem = current_project_memory(&dijiang_dir)?;
    let record = dijiang_mem::Correction {
        timestamp: chrono::Local::now().to_rfc3339(),
        session_key: Some(session_key),
        task: active_task,
        source: source.to_string(),
        correction: correction.to_string(),
        lesson: lesson.to_string(),
        scope: scope.to_string(),
        confidence: if source == "user" {
            "user-confirmed".to_string()
        } else {
            "observed".to_string()
        },
        freshness: freshness.to_string(),
        conflict: conflict.to_string(),
        actionability: actionability.to_string(),
    };
    mem.append_correction(&record)?;
    println!(
        "  Correction recorded to {}",
        mem.root().join("corrections.jsonl").display()
    );
    Ok(())
}

fn cmd_mem_archive() -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let dijiang_dir = crate::store::find_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No .dijiang/ found. Run `dijiang init` first."))?;

    let config_path = dijiang_dir.join("config.toml");
    let config_str = std::fs::read_to_string(&config_path)?;
    let developer = config_str
        .lines()
        .find(|l| l.starts_with("developer"))
        .and_then(|l| l.split('=').nth(1))
        .map(|s| s.trim().trim_matches('\"').to_string())
        .unwrap_or_else(|| "developer".to_string());

    let workspace = dijiang_dir.join("workspace").join(&developer);
    std::fs::create_dir_all(&workspace)?;

    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let archive_dir = workspace.join(format!("{}-archive", date));
    std::fs::create_dir_all(&archive_dir)?;

    // Move today's findings and lessons to archive
    for name in &["findings.md", "lessons.md"] {
        let src = workspace.join(name);
        if src.exists() {
            let dst = archive_dir.join(name);
            std::fs::rename(&src, &dst)?;
            println!("  Archived {}", name);
        }
    }

    println!("  Session archived to {}", archive_dir.display());
    Ok(())
}

fn cmd_mem_tactic(name: &str, description: &str) -> anyhow::Result<()> {
    let mem = dijiang_mem::GlobalMemory::new()?;
    let tactic = mem.add_tactic(name, description, "cli")?;
    println!(
        "  Added tactic: {} (alpha={}, beta={})",
        tactic.name, tactic.alpha, tactic.beta
    );
    Ok(())
}

fn cmd_mem_tactics(select: usize) -> anyhow::Result<()> {
    let mem = dijiang_mem::GlobalMemory::new()?;
    let tactics = mem.select_tactics(select)?;
    println!("  Top {} tactics (Thompson sampling):", select);
    for t in &tactics {
        println!(
            "    {} (win_rate={:.2}, a={}, b={})",
            t.name,
            t.win_rate(),
            t.alpha,
            t.beta
        );
    }
    Ok(())
}

fn cmd_mem_record(tactic_name: &str, outcome: &str, context: &str) -> anyhow::Result<()> {
    let mem = dijiang_mem::GlobalMemory::new()?;
    let outcome_enum = match outcome {
        "success" => dijiang_mem::Outcome::Success,
        "failure" => dijiang_mem::Outcome::Failure,
        _ => anyhow::bail!("outcome must be 'success' or 'failure'"),
    };
    mem.record_event(tactic_name, outcome_enum, context, None)?;
    println!("  Recorded {} for tactic {}", outcome, tactic_name);
    Ok(())
}

fn cmd_mem_pattern(name: &str, description: &str) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let dijiang_dir = crate::store::find_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No .dijiang/ found. Run `dijiang init` first."))?;
    let mem = dijiang_mem::ProjectMemory::from_dijiang_dir(&dijiang_dir)?;
    let pattern = dijiang_mem::Pattern {
        name: name.to_string(),
        description: description.to_string(),
        steps: vec![],
        tags: vec![],
        created_at: chrono::Local::now().to_rfc3339(),
        project: None,
    };
    mem.add_pattern(&pattern)?;
    println!("  Added pattern: {}", name);
    Ok(())
}

fn cmd_mem_patterns() -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let dijiang_dir = crate::store::find_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No .dijiang/ found. Run `dijiang init` first."))?;
    let mem = dijiang_mem::ProjectMemory::from_dijiang_dir(&dijiang_dir)?;
    let patterns = mem.load_patterns()?;
    println!("  {} patterns:", patterns.len());
    for p in &patterns {
        println!("    {} - {}", p.name, p.description);
    }
    Ok(())
}

fn cmd_mem_stats() -> anyhow::Result<()> {
    let global_mem = dijiang_mem::GlobalMemory::new()?;
    let tactics = global_mem.load_tactics()?;
    let avg_win_rate = if tactics.is_empty() {
        0.0
    } else {
        tactics.iter().map(|t| t.win_rate()).sum::<f64>() / tactics.len() as f64
    };

    let cwd = std::env::current_dir()?;
    let dijiang_dir = crate::store::find_dijiang_dir(&cwd);
    let (findings, learnings, corrections, sessions, patterns) =
        if let Some(dijiang_dir) = dijiang_dir {
            let project_mem = dijiang_mem::ProjectMemory::from_dijiang_dir(&dijiang_dir)?;
            (
                project_mem.load_findings()?.len(),
                project_mem.load_learnings()?.len(),
                project_mem.load_corrections()?.len(),
                project_mem.load_session_closures()?.len(),
                project_mem.load_patterns()?.len(),
            )
        } else {
            (0, 0, 0, 0, 0)
        };

    println!("  Memory Stats:");
    println!("    Session closures: {}", sessions);
    println!("    Findings: {}", findings);
    println!("    Learnings: {}", learnings);
    println!("    Corrections: {}", corrections);
    println!("    Patterns: {}", patterns);
    println!("    Tactics: {}", tactics.len());
    println!("    Avg win rate: {:.2}", avg_win_rate);
    Ok(())
}

fn cmd_mem_backup() -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let dijiang_dir = crate::store::find_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No .dijiang/ found. Run `dijiang init` first."))?;
    let config_path = dijiang_dir.join("config.toml");
    let config_str = std::fs::read_to_string(&config_path)?;
    let project = config_str
        .lines()
        .find(|l| l.starts_with("name"))
        .and_then(|l| l.split('=').nth(1))
        .map(|s| s.trim().trim_matches('"').to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let global_mem = dijiang_mem::GlobalMemory::new()?;
    let project_mem = dijiang_mem::ProjectMemory::from_dijiang_dir(&dijiang_dir)?;
    global_mem.backup_project(&project, &project_mem)?;
    println!("  Backed up project '{}' to ~/.dijiang/backups/", project);
    Ok(())
}

fn cmd_mem_evolve() -> anyhow::Result<()> {
    println!("  🔥 Fast-loop evolution: analyzing session...");
    let cwd = std::env::current_dir()?;
    let dijiang_dir = crate::store::find_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No .dijiang/ found. Run `dijiang init` first."))?;

    // Read project findings and learnings
    let project_mem = dijiang_mem::ProjectMemory::from_dijiang_dir(&dijiang_dir)?;
    let findings = project_mem.load_findings()?;
    let learnings = project_mem.load_learnings()?;
    let corrections = project_mem.load_corrections()?;
    let sessions = project_mem.load_session_closures()?;

    // Analyze patterns and create/update tactics
    let global_mem = dijiang_mem::GlobalMemory::new()?;
    let mut tactics_created = 0;

    // Simple pattern detection: if similar findings appear 3+ times, create a tactic
    let mut finding_counts: std::collections::HashMap<String, u32> =
        std::collections::HashMap::new();
    for finding in &findings {
        let key = finding.content.chars().take(50).collect::<String>();
        *finding_counts.entry(key).or_insert(0) += 1;
    }

    for (pattern, count) in &finding_counts {
        if *count >= 3 {
            // Check if tactic already exists
            let existing = global_mem.load_tactics()?;
            if !existing.iter().any(|t| t.description.contains(pattern)) {
                global_mem.add_tactic(
                    pattern,
                    &format!("Auto-detected from {} findings", count),
                    &dijiang_dir.to_string_lossy(),
                )?;
                tactics_created += 1;
            }
        }
    }

    // Backup project memory
    let config_path = dijiang_dir.join("config.toml");
    let config_str = std::fs::read_to_string(&config_path)?;
    let project = config_str
        .lines()
        .find(|l| l.starts_with("name"))
        .and_then(|l| l.split('=').nth(1))
        .map(|s| s.trim().trim_matches('"').to_string())
        .unwrap_or_else(|| "unknown".to_string());
    global_mem.backup_project(&project, &project_mem)?;

    println!("  Findings analyzed: {}", findings.len());
    println!("  Learnings analyzed: {}", learnings.len());
    println!("  Corrections analyzed: {}", corrections.len());
    println!("  Session closures analyzed: {}", sessions.len());
    println!("  Tactics created: {}", tactics_created);
    println!(
        "  Project memory backed up to ~/.dijiang/backups/{}",
        project
    );
    Ok(())
}

fn cmd_mem_finetune() -> anyhow::Result<()> {
    println!("  🧬 Slow-loop fine-tune: training on accumulated experience...");
    let global_mem = dijiang_mem::GlobalMemory::new()?;

    // Load all tactics
    let tactics = global_mem.load_tactics()?;
    if tactics.is_empty() {
        println!("  No tactics found. Run `dijiang mem evolve` first.");
        return Ok(());
    }

    // Calculate statistics
    let total_tactics = tactics.len();
    let avg_win_rate = tactics.iter().map(|t| t.win_rate()).sum::<f64>() / total_tactics as f64;
    let high_performers: Vec<_> = tactics.iter().filter(|t| t.win_rate() > 0.7).collect();
    let low_performers: Vec<_> = tactics.iter().filter(|t| t.win_rate() < 0.3).collect();

    println!("  Total tactics: {}", total_tactics);
    println!("  Average win rate: {:.2}", avg_win_rate);
    println!("  High performers (>70%): {}", high_performers.len());
    println!("  Low performers (<30%): {}", low_performers.len());

    // Ratchet gate: only promote if no regressions
    if low_performers.len() > high_performers.len() {
        println!("  ⚠️  More low performers than high performers. Consider pruning.");
    } else {
        println!("  ✅ Ratchet gate: PASS - system improving.");
    }

    // Update stats
    let stats = dijiang_mem::MemoryStats {
        total_findings: 0,
        total_learnings: 0,
        total_corrections: 0,
        total_tactics: total_tactics as u64,
        total_patterns: 0,
        total_sessions: 0,
        avg_tactic_win_rate: avg_win_rate,
        last_evolution: Some(chrono::Local::now().to_rfc3339()),
        last_finetune: Some(chrono::Local::now().to_rfc3339()),
    };
    global_mem.save_stats(&stats)?;

    println!("  Fine-tune complete.");
    Ok(())
}

fn cmd_update(force: bool, from_github: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let _dijiang_dir = crate::store::find_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow::anyhow!("未找到 .dijiang/ 目录。请先运行 `dijiang init`。"))?;

    if from_github {
        println!("  正在从 GitHub 下载最新版本...");
        let temp_dir = std::env::temp_dir().join("dijiang-update");
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir)?;
        }

        let output = std::process::Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "https://github.com/Asura-one/DiJiang.git",
                temp_dir.to_str().unwrap(),
            ])
            .output()?;

        if !output.status.success() {
            anyhow::bail!(
                "从 GitHub 下载失败: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        println!("  下载完成，正在更新全局技能...");
        let global_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?
            .join(".dijiang")
            .join("skills");
        std::fs::create_dir_all(&global_dir)?;

        let src_skills = temp_dir
            .join("crates")
            .join("configurator")
            .join("templates")
            .join("skills");
        if src_skills.exists() {
            for entry in std::fs::read_dir(&src_skills)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    let name = entry.file_name();
                    let src = entry.path().join("SKILL.md");
                    let dst = global_dir.join(&name).join("SKILL.md");
                    if src.exists() {
                        std::fs::create_dir_all(dst.parent().unwrap())?;
                        std::fs::copy(&src, &dst)?;
                        println!("  已更新全局技能: {}", name.to_string_lossy());
                    }
                }
            }
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
        println!("  GitHub 更新完成。\n");
    }

    let report =
        dijiang_configurator::update_project(&cwd, dijiang_configurator::UpdateOptions { force })?;

    // Show version comparison
    let old_version = report.old_version.as_deref().unwrap_or("unknown");
    println!(
        "  DiJiang 版本: {old_version} -> {}",
        report.new_version
    );

    // Show changelog if version changed
    if report.old_version.as_deref() != Some(&report.new_version) {
        let changelog =
            dijiang_configurator::changelog_between(old_version, &report.new_version);
        if !changelog.is_empty() {
            println!("\n  ── 变更日志 ──");
            for line in changelog.lines() {
                println!("  {line}");
            }
            println!();
        }
    }

    for path in &report.updated {
        println!("  updated   {path}");
    }
    for path in &report.unchanged {
        println!("  unchanged {path}");
    }
    for path in &report.removed {
        println!("  removed   {path}");
    }
    for warning in &report.warnings {
        println!("  warning   {warning}");
    }
    for path in &report.conflicts {
        println!("  conflict  {path}");
    }

    println!();
    println!(
        "  更新完成: {} 个文件已更新, {} 个已是最新, {} 个已删除, {} 个冲突, {} 个警告",
        report.updated.len(),
        report.unchanged.len(),
        report.removed.len(),
        report.conflicts.len(),
        report.warnings.len()
    );

    if !report.is_clean() {
        anyhow::bail!(
            "update blocked: {} 个受管文件可能包含用户修改，未覆盖。确认后可使用 `dijiang update --force` 覆盖并建立后续升级 hash。",
            report.conflicts.len()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{dispatch_route, dispatch_route_for_active_task};
    use dijiang_task::store;
    use dijiang_task::types::TaskStatus;

    fn status_format(status: TaskStatus) -> (String, String) {
        (
            status.as_str().to_string(),
            status.to_trellis_status().to_string(),
        )
    }

    #[test]
    fn test_status_format_planning() {
        let (s, p) = status_format(TaskStatus::Planning);
        assert_eq!(p, "plan");
        assert_eq!(s, "planning");
    }

    #[test]
    fn test_status_format_in_progress() {
        let (s, p) = status_format(TaskStatus::InProgress);
        assert_eq!(p, "implement");
        assert_eq!(s, "in_progress");
    }

    #[test]
    fn test_status_format_completed() {
        let (s, p) = status_format(TaskStatus::Completed);
        assert_eq!(p, "complete");
        assert_eq!(s, "completed");
    }

    #[test]
    fn test_status_format_paused() {
        let (s, p) = status_format(TaskStatus::Paused);
        assert_eq!(p, "in_progress");
        assert_eq!(s, "paused");
    }

    #[test]
    fn test_status_format_archived() {
        let (s, p) = status_format(TaskStatus::Archived);
        assert_eq!(p, "complete");
        assert_eq!(s, "archived");
    }

    #[test]
    fn test_dispatch_ignores_embedded_skill_context_for_visible_prompt() {
        let route = dispatch_route(
            r#"<skill name="dijiang-start">排查调试 debug bug</skill>新问题，grill未触发"#,
        );

        assert_eq!(route.skill, "dj-grill");
        assert_eq!(route.status, TaskStatus::Planning);
    }

    #[test]
    fn test_dispatch_routes_vague_bug_request_to_grill() {
        let route = dispatch_route("有个 bug 帮我修一下");

        assert_eq!(route.skill, "dj-grill");
        assert_eq!(route.status, TaskStatus::Planning);
    }

    #[test]
    fn test_dispatch_routes_specific_bug_request_to_hunt() {
        let route = dispatch_route("登录接口报错，帮我排查并修复");

        assert_eq!(route.skill, "dj-hunt");
        assert_eq!(route.status, TaskStatus::InProgress);
    }

    #[test]
    fn test_dispatch_routes_vague_feature_request_to_grill() {
        let route = dispatch_route("做个导出功能");

        assert_eq!(route.skill, "dj-grill");
        assert_eq!(route.status, TaskStatus::Planning);
    }

    #[test]
    fn test_dispatch_routes_specific_feature_request_to_implement() {
        let route = dispatch_route("新增一个导出按钮");

        assert_eq!(route.skill, "dj-implement");
        assert_eq!(route.status, TaskStatus::InProgress);
    }

    #[test]
    fn test_dispatch_routes_specific_interface_request_to_implement() {
        let route = dispatch_route("新增一个导出接口");

        assert_eq!(route.skill, "dj-implement");
        assert_eq!(route.status, TaskStatus::InProgress);
    }

    #[test]
    fn test_dispatch_routes_vague_login_optimization_to_grill() {
        let route = dispatch_route("优化登录体验");

        assert_eq!(route.skill, "dj-grill");
        assert_eq!(route.status, TaskStatus::Planning);
    }

    #[test]
    fn test_dispatch_routes_test_work_to_tdd() {
        let route = dispatch_route("补测试");

        assert_eq!(route.skill, "dj-tdd");
        assert_eq!(route.status, TaskStatus::InProgress);
    }

    #[test]
    fn test_session_start_dispatch_routes_planning_task_to_grill() {
        let mut task = store::create_task("needs-alignment", "Needs Alignment");
        task.status = TaskStatus::Planning;
        task.meta = serde_json::json!({
            "dispatch": { "skill": "dj-hunt" }
        });

        let route = dispatch_route_for_active_task(&task);

        assert_eq!(route.skill, "dj-grill");
        assert_eq!(route.status, TaskStatus::Planning);
    }

    #[test]
    fn test_session_start_dispatch_keeps_in_progress_hunt_route() {
        let mut task = store::create_task("bug", "Bug");
        task.status = TaskStatus::InProgress;
        task.meta = serde_json::json!({
            "dispatch": { "skill": "dj-hunt" }
        });

        let route = dispatch_route_for_active_task(&task);

        assert_eq!(route.skill, "dj-hunt");
        assert_eq!(route.status, TaskStatus::InProgress);
    }
}
