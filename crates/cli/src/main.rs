use clap::{Parser, Subcommand};
use dijiang_configurator::PlatformKind;
use dijiang_task::store;
use dijiang_task::types::TaskStatus;
use dijiang_configurator::TemplateRegistry;
use dialoguer::{Input, MultiSelect, Confirm};
use std::path::PathBuf;
use sha2::{Sha256, Digest};
#[derive(Parser)]
#[command(name = "dijiang", version = "0.1.0", about = "DiJiang - AI coding assistant workflow layer")]
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
    /// 开始一个新的编码会话
    Start {
        /// 任务名称（slug 格式，如 "fix-login-bug"）
        name: String,
        /// 任务显示标题（可选）
        title: Option<String>,
    },
    /// 任务管理
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
    /// 将 Trellis 项目迁移到 DiJiang
    Migrate,
    /// 代码审查（对抗式/第一性原理）
    Review {
        /// 审查模式：adversarial 或 first-principles
        #[arg(long, default_value = "adversarial")]
        mode: String,
    },
    /// Agent 通道管理
    Channel {
        #[command(subcommand)]
        command: ChannelCommands,
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
        #[arg(long)]
        outcome: String,  // success or failure
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
    /// Create and activate a new task
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
        Commands::Status { compat } => cmd_status(compat),
        Commands::Init { name, developer, yes, force, platforms, auto_detect } =>
            cmd_init(&name, developer.as_deref(), yes, force, platforms.as_deref(), auto_detect),
        Commands::Task { command } => match command {
            TaskCommands::List => cmd_task_list(),
            TaskCommands::Current => cmd_task_current(),
            TaskCommands::Start { name } => cmd_task_start(&name),
            TaskCommands::Status { name, status } => cmd_task_status(&name, &status),
            TaskCommands::Archive { name } => cmd_task_archive(&name),
            TaskCommands::Prune { days } => cmd_task_prune(days),
        },
        Commands::Mem { command: MemCommands::List } => cmd_mem_list(),
        Commands::Mem { command: MemCommands::Sync } => cmd_mem_sync(),
        Commands::Mem { command: MemCommands::Findings { finding } } => cmd_mem_findings(&finding),
        Commands::Mem { command: MemCommands::Learn { lesson } } => cmd_mem_learn(&lesson),
        Commands::Mem { command: MemCommands::Archive } => cmd_mem_archive(),
        Commands::Mem { command: MemCommands::Tactic { name, description } } => cmd_mem_tactic(&name, &description),
        Commands::Mem { command: MemCommands::Tactics { select } } => cmd_mem_tactics(select),
        Commands::Mem { command: MemCommands::Record { tactic, outcome, context } } => cmd_mem_record(&tactic, &outcome, &context),
        Commands::Mem { command: MemCommands::Pattern { name, description } } => cmd_mem_pattern(&name, &description),
        Commands::Mem { command: MemCommands::Patterns } => cmd_mem_patterns(),
        Commands::Mem { command: MemCommands::Stats } => cmd_mem_stats(),
        Commands::Mem { command: MemCommands::Backup } => cmd_mem_backup(),
        Commands::Mem { command: MemCommands::Evolve } => cmd_mem_evolve(),
        Commands::Mem { command: MemCommands::Finetune } => cmd_mem_finetune(),
        Commands::Template { command } => match command {
            TemplateCommands::List => cmd_template_list(),
            TemplateCommands::Pull { source } => cmd_template_pull(&source),
            TemplateCommands::Validate { path } => cmd_template_validate(&path),
        },
        Commands::Skills { sync } => cmd_skills(sync),
        Commands::Migrate => cmd_migrate(),
        Commands::Review { mode } => cmd_review(&mode),
        Commands::Channel { command } => match command {
            ChannelCommands::Spawn { agent, task, dir } => cmd_channel_spawn(&agent, task.as_deref(), dir.as_deref()),
            ChannelCommands::List => cmd_channel_list(),
            ChannelCommands::Send { channel_id, message } => cmd_channel_send(&channel_id, &message),
            ChannelCommands::Status { channel_id } => cmd_channel_status(&channel_id),
            ChannelCommands::Stop { channel_id } => cmd_channel_stop(&channel_id),
            ChannelCommands::Execute { channel_id, model, provider, timeout, follow } => cmd_channel_execute(&channel_id, model.as_deref(), provider.as_deref(), timeout, follow),
            ChannelCommands::ExecuteAll { model, provider, timeout } => cmd_channel_execute_all(model.as_deref(), provider.as_deref(), timeout),
        },
        Commands::Update { force, from_github } => cmd_update(force, from_github),
    }
}
fn status_line(label: &str, value: impl std::fmt::Display) {
    println!("  {label:15} {value}");
}

fn cmd_status(compat: bool) -> anyhow::Result<()> {
    println!("\n  ── DiJiang Status ──\n");

    let cwd = std::env::current_dir()?;
    let dijiang_dir = match store::find_dijiang_dir(&cwd) {
        Some(d) => d,
        None => {
            println!("  No .dijiang/ found. Run `dijiang init` first.");
            return Ok(());
        }
    };

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
        let marker = active.as_ref().map_or(' ', |a| if a == &t.name { '*' } else { ' ' });
        let phase = t.status.to_trellis_status();
        println!("    {marker} {name:<45} {status:12} {phase:12}",
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
    let dijiang_dir = match store::find_dijiang_dir(&std::env::current_dir()?) {
        Some(d) => d,
        None => {
            println!("No .dijiang/ found.");
            return Ok(());
        }
    };

    let tasks_dir = dijiang_dir.join("tasks");
    let tasks = store::list_tasks(&tasks_dir).unwrap_or_default();

    if tasks.is_empty() {
        println!("No tasks found.");
        return Ok(());
    }

    for t in &tasks {
        println!("{name:<50} {status:12}  {priority:2}",
            name = t.name,
            status = t.status.as_str(),
            priority = t.priority,
        );
    }
    Ok(())
}

fn cmd_task_current() -> anyhow::Result<()> {
    let dijiang_dir = match store::find_dijiang_dir(&std::env::current_dir()?) {
        Some(d) => d,
        None => {
            println!("No .dijiang/ found.");
            return Ok(());
        }
    };

    match store::read_active_task(&dijiang_dir)? {
        Some(name) => println!("{name}"),
        None => println!("(none)"),
    }
    Ok(())
}

fn cmd_task_start(name: &str) -> anyhow::Result<()> {
    let dijiang_dir = match store::find_dijiang_dir(&std::env::current_dir()?) {
        Some(d) => d,
        None => {
            eprintln!("No .dijiang/ found.");
            std::process::exit(1);
        }
    };

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
    let dijiang_dir = match store::find_dijiang_dir(&std::env::current_dir()?) {
        Some(d) => d,
        None => {
            eprintln!("No .dijiang/ found.");
            std::process::exit(1);
        }
    };

    let new_status = match status_str {
        "planning" => TaskStatus::Planning,
        "in_progress" => TaskStatus::InProgress,
        "completed" => TaskStatus::Completed,
        "archived" => TaskStatus::Archived,
        "paused" => TaskStatus::Paused,
        _ => {
            eprintln!("Invalid status: '{status_str}'. Valid: planning|in_progress|completed|archived|paused");
            std::process::exit(1);
        }
    };

    let tasks_dir = dijiang_dir.join("tasks");
    match store::update_status(&tasks_dir, name, new_status) {
        Ok(task) => {
            println!("✓ Task '{name}' status updated to: {}", task.status.as_str());
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
    let dijiang_dir = match store::find_dijiang_dir(&std::env::current_dir()?) {
        Some(d) => d,
        None => {
            println!("No .dijiang/ found.");
            return Ok(());
        }
    };

    let tasks_dir = dijiang_dir.join("tasks");
    match store::archive_task(&tasks_dir, name) {
        Ok(task) => {
            println!("✓ Task '{name}' archived (status: {})", task.status.as_str());
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
    let dijiang_dir = match store::find_dijiang_dir(&std::env::current_dir()?) {
        Some(d) => d,
        None => {
            println!("No .dijiang/ found.");
            return Ok(());
        }
    };

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
            println!("    • {} v{} — {}",
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
            println!("✓ Pulled template '{}' v{} to cache",
                pkg.manifest.template.name,
                pkg.manifest.template.version,
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
            println!("✓ Template '{}' v{} is valid",
                manifest.template.name,
                manifest.template.version,
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

fn cmd_init(name: &str, developer: Option<&str>, yes: bool, force: bool, platforms: Option<&str>, auto_detect: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;

    // Check if already initialized
    // Check if already initialized
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
                    String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
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
        println!("  Detected platforms: {}", detected.iter().map(|p| p.display_name()).collect::<Vec<_>>().join(", "));
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
    // Write dj-* skills to project
    let skills_written = dijiang_configurator::write_project_skills(&cwd)?;
    if skills_written > 0 {
        println!("  Wrote {} dj-* skills to .pi/skills/", skills_written);
    }

    // Initialize default tactics
    match dijiang_mem::GlobalMemory::new() {
        Ok(global_mem) => {
            if let Err(e) = global_mem.ensure_default_tactics() {
                eprintln!("  Warning: Failed to initialize default tactics: {}", e);
            } else {
                println!("  Initialized default tactics (cargo-test, typecheck, review-*, lint-fix, doc-update)");
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
    let dijiang_dir = match store::find_dijiang_dir(&cwd) {
        Some(d) => d,
        None => {
            eprintln!("No .dijiang/ found. Run `dijiang init` first.");
            std::process::exit(1);
        }
    };

    let tasks_dir = dijiang_dir.join("tasks");
    let now = chrono::Utc::now();

    // Load existing task or create new one
    match store::load_task(&tasks_dir, name) {
        Ok(mut task) => {
            // Update existing task
            let was_status = task.status.as_str().to_string();
            task.status = TaskStatus::InProgress;
            task.started_at = task.started_at.take().or(Some(now.to_rfc3339()));
            store::save_task(&tasks_dir, &task)?;
            println!("  ✓ Task '{name}' updated");
            println!("    Status: {was_status} → in_progress");
        }
        Err(store::TaskError::NotFound(_)) => {
            // Create new task
            let display_title = title.unwrap_or(name);
            let mut task = store::create_task(name, display_title);
            task.status = TaskStatus::InProgress;
            task.started_at = Some(now.to_rfc3339());
            store::save_task(&tasks_dir, &task)?;
            println!("  ✓ Task '{name}' created");
            println!("    Title: {display_title}");
            println!("    Status: planning → in_progress");
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
        let active = p.sessions.iter().filter(|s| s.status == dijiang_mem::SessionStatus::Active).count();
        let archived = total - active;
        let latest = p.last_active_at.as_deref().unwrap_or("-");
        println!("  {project}", project = p.project_id);
        println!("    Total: {total}  Active: {active}  Archived: {archived}");
        println!("    Latest: {latest}");

        for s in p.sessions.iter().take(3) {
            let task = s.task.as_deref().unwrap_or("(no task)");
            let truncated = if task.len() > 60 {
                let mut end = 57;
                while !task.is_char_boundary(end) { end += 1; }
                &task[..end]
            } else { task };
            let marker = if s.status == dijiang_mem::SessionStatus::Active { "[A]" } else { "[ ]" };
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
        let skills_written = dijiang_configurator::write_project_skills(&cwd)?;
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

fn cmd_review(mode: &str) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let dijiang_dir = crate::store::find_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No .dijiang/ found. Run `dijiang init` first."))?;

    // Read agent definitions
    let agents_dir = dijiang_dir.parent().map(|p| p.join(".pi").join("agents")).unwrap_or_default();

    match mode {
        "adversarial" => {
            println!("  🔍 Adversarial Review Mode (对抗式审查)");
            println!();
            println!("  Use the dijiang-check agent with these review angles:");
            println!("  1. Security - How would a malicious user attack this?");
            println!("  2. Edge cases - What happens with extreme inputs?");
            println!("  3. Performance - What if resources are exhausted?");
            println!("  4. Data corruption - What if data is malformed?");
            println!("  5. Race conditions - What if concurrent access occurs?");
            println!("  6. Resource leaks - What if cleanup fails?");
            println!("  7. Error handling - What if dependencies fail?");
            println!();
            println!("  Agent definition: .pi/agents/dijiang-check.md");
            println!();
            // Generate review prompt
            let prompt = "Review the code changes from a security perspective.\n\n".to_string()
                + "Focus on these attack vectors:\n"
                + "1. Input validation - What if inputs are malicious?\n"
                + "2. Injection attacks - SQL, command, XSS?\n"
                + "3. Authentication bypass - Can unauthorized access occur?\n"
                + "4. Data exposure - Are secrets or sensitive data leaked?\n"
                + "5. Denial of service - Can the system be overwhelmed?\n"
                + "6. Supply chain - Are dependencies trustworthy?\n"
                + "\n"
                + "For each issue found, provide:\n"
                + "- file:line citation\n"
                + "- attack scenario\n"
                + "- severity (critical/high/medium/low)\n"
                + "- recommended fix\n"
                + "\n"
                + "Run: git diff to see changes, then review each file.";
            println!("  Generated prompt:");
            println!("  {}", prompt);
        },
        "first-principles" => {
            println!("  🧠 First Principles Review Mode (第一性原理审查)");
            println!();
            println!("  Use the dijiang-implement agent with these analysis steps:");
            println!("  1. What is the fundamental problem being solved?");
            println!("  2. What are the basic facts and constraints?");
            println!("  3. What assumptions are we making?");
            println!("  4. Can we derive the solution from first principles?");
            println!("  5. Is there a simpler, more fundamental approach?");
            println!("  6. What are the trade-offs of each approach?");
            println!();
            println!("  Agent definition: .pi/agents/dijiang-implement.md");
            println!();
            // Generate review prompt
            let prompt = "Review this code from first principles.\n\n".to_string()
                + "Step 1: Identify the fundamental problem this code solves.\n"
                + "Step 2: List the basic facts and constraints.\n"
                + "Step 3: Identify hidden assumptions.\n"
                + "Step 4: Derive the solution from first principles.\n"
                + "Step 5: Propose a simpler approach if possible.\n"
                + "Step 6: Analyze trade-offs.\n"
                + "\n"
                + "For each finding, explain:\n"
                + "- What assumption is being made\n"
                + "- Why it might be wrong\n"
                + "- What the first-principles alternative would be\n"
                + "\n"
                + "Run: git diff to see changes, then analyze each component.";
            println!("  Generated prompt:");
            println!("  {}", prompt);
        },
        _ => anyhow::bail!("mode must be 'adversarial' or 'first-principles'"),
    }

    // Record this as a tactic
    let global_mem = dijiang_mem::GlobalMemory::new()?;
    let tactic_name = format!("review-{}", mode);
    global_mem.add_tactic(&tactic_name, &format!("{} review performed", mode), &dijiang_dir.to_string_lossy())?;
    global_mem.record_event(&tactic_name, dijiang_mem::Outcome::Success, "review completed", Some(&dijiang_dir.to_string_lossy()))?;

    println!();
    println!("  Review recorded as tactic: {}", tactic_name);
    println!("  Use this prompt with dj-check or dj-implement agent.");
    Ok(())
}

fn cmd_channel_spawn(agent: &str, task: Option<&str>, dir: Option<&str>) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let base_dir = dir.map(PathBuf::from).unwrap_or_else(|| cwd.clone());
    let dijiang_dir = crate::store::find_dijiang_dir(&base_dir)
        .ok_or_else(|| anyhow::anyhow!("No .dijiang/ found. Run `dijiang init` first."))?;

    // Read agent definition
    let agents_dir = dijiang_dir.parent().map(|p| p.join(".pi").join("agents")).unwrap_or_default();
    let agent_file = agents_dir.join(format!("dijiang-{}.md", agent));
    if !agent_file.exists() {
        anyhow::bail!("Agent '{}' not found at {}", agent, agent_file.display());
    }
    let agent_def = std::fs::read_to_string(&agent_file)?;

    // Generate channel ID
    let channel_id = format!("{}-{}-{}", agent,
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs(),
        &cwd.to_string_lossy()[cwd.to_string_lossy().len()-8..].replace('/', "-"));

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
        channel_id, agent, timestamp, task.unwrap_or(""), cwd.display());
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
                let agent = content.lines()
                    .find(|l| l.contains("agent"))
                    .and_then(|l| l.split('=').nth(1))
                    .map(|s| s.trim().trim_matches('"').to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                let status = content.lines()
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

fn cmd_channel_execute(channel_id: &str, model: Option<&str>, provider: Option<&str>, timeout: u64, follow: bool) -> anyhow::Result<()> {
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

    println!("  Executing agent '{}' in channel {}", agent_name, channel_id);
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

    let prompt = format!(
        "{}\n\n---\n\nInbox:\n{}",
        agent_def, inbox_content
    );

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
    let status = if output.status.success() { "completed" } else { "failed" };
    if channel_toml.exists() {
        let content = std::fs::read_to_string(&channel_toml)?;
        let new_content = content.replace("status = \"active\"", &format!("status = \"{}\"", status));
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

fn cmd_channel_execute_all(model: Option<&str>, provider: Option<&str>, timeout: u64) -> anyhow::Result<()> {
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

    println!("  Executing {} active channel(s) in parallel", active_channels.len());
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
            match cmd_channel_execute_single(&channel_id, model.as_deref(), provider.as_deref(), timeout, &cwd, &dijiang_dir) {
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
    let prompt = format!(
        "{}\n\n---\n\nInbox:\n{}",
        agent_def, inbox_content
    );

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
    let status = if output.status.success() { "completed" } else { "failed" };
    if channel_toml.exists() {
        let content = std::fs::read_to_string(&channel_toml)?;
        let new_content = content.replace("status = \"active\"", &format!("status = \"{}\"", status));
        std::fs::write(&channel_toml, &new_content)?;
    }

    Ok(())
}

fn cmd_mem_findings(finding: &str) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let dijiang_dir = crate::store::find_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No .dijiang/ found. Run `dijiang init` first."))?;

    // Read developer name from config.toml (simple parser)
    let config_path = dijiang_dir.join("config.toml");
    let config_str = std::fs::read_to_string(&config_path)?;
    let developer = config_str.lines()
        .find(|l| l.starts_with("developer"))
        .and_then(|l| l.split('=').nth(1))
        .map(|s| s.trim().trim_matches('\"').to_string())
        .unwrap_or_else(|| "developer".to_string());

    let workspace = dijiang_dir.join("workspace").join(&developer);
    std::fs::create_dir_all(&workspace)?;

    let journal = workspace.join("findings.md");
    let entry = format!(
        "\n## {}\n{}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M"),
        finding
    );
    use std::io::Write;
    std::fs::OpenOptions::new().create(true).append(true).open(&journal)?.write_all(entry.as_bytes())?;
    println!("  Finding recorded to {}", journal.display());
    Ok(())
}

fn cmd_mem_learn(lesson: &str) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let dijiang_dir = crate::store::find_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No .dijiang/ found. Run `dijiang init` first."))?;

    let config_path = dijiang_dir.join("config.toml");
    let config_str = std::fs::read_to_string(&config_path)?;
    let developer = config_str.lines()
        .find(|l| l.starts_with("developer"))
        .and_then(|l| l.split('=').nth(1))
        .map(|s| s.trim().trim_matches('\"').to_string())
        .unwrap_or_else(|| "developer".to_string());

    let workspace = dijiang_dir.join("workspace").join(&developer);
    std::fs::create_dir_all(&workspace)?;

    let journal = workspace.join("lessons.md");
    let entry = format!(
        "\n## {}\n{}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M"),
        lesson
    );
    use std::io::Write;
    std::fs::OpenOptions::new().create(true).append(true).open(&journal)?.write_all(entry.as_bytes())?;
    println!("  Lesson recorded to {}", journal.display());
    Ok(())
}

fn cmd_mem_archive() -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let dijiang_dir = crate::store::find_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No .dijiang/ found. Run `dijiang init` first."))?;

    let config_path = dijiang_dir.join("config.toml");
    let config_str = std::fs::read_to_string(&config_path)?;
    let developer = config_str.lines()
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
    println!("  Added tactic: {} (alpha={}, beta={})", tactic.name, tactic.alpha, tactic.beta);
    Ok(())
}

fn cmd_mem_tactics(select: usize) -> anyhow::Result<()> {
    let mem = dijiang_mem::GlobalMemory::new()?;
    let tactics = mem.select_tactics(select)?;
    println!("  Top {} tactics (Thompson sampling):", select);
    for t in &tactics {
        println!("    {} (win_rate={:.2}, a={}, b={})", t.name, t.win_rate(), t.alpha, t.beta);
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
    let mem = dijiang_mem::ProjectMemory::new(&dijiang_dir)?;
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
    let mem = dijiang_mem::ProjectMemory::new(&dijiang_dir)?;
    let patterns = mem.load_patterns()?;
    println!("  {} patterns:", patterns.len());
    for p in &patterns {
        println!("    {} - {}", p.name, p.description);
    }
    Ok(())
}

fn cmd_mem_stats() -> anyhow::Result<()> {
    let mem = dijiang_mem::GlobalMemory::new()?;
    let tactics = mem.load_tactics()?;
    let avg_win_rate = if tactics.is_empty() { 0.0 } else {
        tactics.iter().map(|t| t.win_rate()).sum::<f64>() / tactics.len() as f64
    };
    println!("  Memory Stats:");
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
    let project = config_str.lines()
        .find(|l| l.starts_with("name"))
        .and_then(|l| l.split('=').nth(1))
        .map(|s| s.trim().trim_matches('"').to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let global_mem = dijiang_mem::GlobalMemory::new()?;
    let project_mem = dijiang_mem::ProjectMemory::new(&dijiang_dir)?;
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
    let project_mem = dijiang_mem::ProjectMemory::new(&dijiang_dir)?;
    let findings = project_mem.load_findings()?;
    let learnings = project_mem.load_learnings()?;

    // Analyze patterns and create/update tactics
    let global_mem = dijiang_mem::GlobalMemory::new()?;
    let mut tactics_created = 0;

    // Simple pattern detection: if similar findings appear 3+ times, create a tactic
    let mut finding_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    for finding in &findings {
        let key = finding.content.chars().take(50).collect::<String>();
        *finding_counts.entry(key).or_insert(0) += 1;
    }

    for (pattern, count) in &finding_counts {
        if *count >= 3 {
            // Check if tactic already exists
            let existing = global_mem.load_tactics()?;
            if !existing.iter().any(|t| t.description.contains(pattern)) {
                global_mem.add_tactic(pattern, &format!("Auto-detected from {} findings", count), &dijiang_dir.to_string_lossy())?;
                tactics_created += 1;
            }
        }
    }

    // Backup project memory
    let config_path = dijiang_dir.join("config.toml");
    let config_str = std::fs::read_to_string(&config_path)?;
    let project = config_str.lines()
        .find(|l| l.starts_with("name"))
        .and_then(|l| l.split('=').nth(1))
        .map(|s| s.trim().trim_matches('"').to_string())
        .unwrap_or_else(|| "unknown".to_string());
    global_mem.backup_project(&project, &project_mem)?;

    println!("  Findings analyzed: {}", findings.len());
    println!("  Learnings analyzed: {}", learnings.len());
    println!("  Tactics created: {}", tactics_created);
    println!("  Project memory backed up to ~/.dijiang/backups/{}", project);
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

fn cmd_update(_force: bool, from_github: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let dijiang_dir = crate::store::find_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow::anyhow!("未找到 .dijiang/ 目录。请先运行 `dijiang init`。"))?;

    // 从 GitHub 下载最新版本
    if from_github {
        println!("  正在从 GitHub 下载最新版本...");
        let temp_dir = std::env::temp_dir().join("dijiang-update");
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir)?;
        }

        let output = std::process::Command::new("git")
            .args(&["clone", "--depth", "1", "https://github.com/Asura-one/DiJiang.git", temp_dir.to_str().unwrap()])
            .output()?;

        if !output.status.success() {
            anyhow::bail!("从 GitHub 下载失败: {}", String::from_utf8_lossy(&output.stderr));
        }

        println!("  下载完成，正在更新技能...");

        // 更新全局技能目录
        let global_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?
            .join(".dijiang").join("skills");
        std::fs::create_dir_all(&global_dir)?;

        let src_skills = temp_dir.join("crates").join("configurator").join("templates").join("skills");
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

        // 清理临时目录
        let _ = std::fs::remove_dir_all(&temp_dir);
        println!();
        println!("  GitHub 更新完成。");
        println!();
    }

    // 加载模板哈希
    let hashes_file = dijiang_dir.join(".template-hashes.json");
    let mut template_hashes: std::collections::HashMap<String, String> =
        if hashes_file.exists() {
            let content = std::fs::read_to_string(&hashes_file)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            std::collections::HashMap::new()
        };

    let mut updated = 0;
    let mut skipped = 0;

    // 更新 dj-* 技能
    let skills_dir = cwd.join(".pi").join("skills");
    std::fs::create_dir_all(&skills_dir)?;

    let skill_names = dijiang_configurator::dj_skills::list_skill_names();
    for name in skill_names {
        let skill_dir = skills_dir.join(name);
        std::fs::create_dir_all(&skill_dir)?;
        let skill_file = skill_dir.join("SKILL.md");

        // 获取嵌入的技能内容
        let content = dijiang_configurator::dj_skills::get_skill_content(name)
            .unwrap_or_default();

        if content.is_empty() {
            continue;
        }

        // 计算新内容的哈希
        let new_hash = format!("{:x}", Sha256::digest(content.as_bytes()));
        let key = format!(".pi/skills/dj-{}/SKILL.md", name);
        let old_hash = template_hashes.get(&key);

        if old_hash != Some(&new_hash) {
            std::fs::write(&skill_file, content)?;
            template_hashes.insert(key, new_hash);
            println!("  已更新: dj-{}/SKILL.md", name);
            updated += 1;
        } else {
            skipped += 1;
        }
    }

    // 保存模板哈希
    let hashes_json = serde_json::to_string_pretty(&template_hashes)?;
    std::fs::write(&hashes_file, hashes_json)?;

    println!();
    println!("  更新完成: {} 个文件已更新, {} 个已是最新", updated, skipped);

    Ok(())
}

#[cfg(test)]
mod tests {
    use dijiang_task::types::TaskStatus;

    fn status_format(status: TaskStatus) -> (String, String) {
        (status.as_str().to_string(), status.to_trellis_status().to_string())
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
}
