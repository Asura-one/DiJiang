use clap::{Parser, Subcommand};
use dijiang_configurator::PlatformKind;
use dijiang_task::store;
use dijiang_task::types::TaskStatus;
use dijiang_configurator::TemplateRegistry;
use dialoguer::{Input, MultiSelect, Confirm};
#[derive(Parser)]
#[command(name = "dijiang", version = "0.1.0", about = "DiJiang - AI coding assistant workflow layer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show project status
    /// Show project status
    Status {
        /// Show detailed compatibility diagnostics
        #[arg(long)]
        compat: bool,
    },
    /// Start a new coding session with a task
    Start {
        /// Task name (slug, e.g. "fix-login-bug")
        name: String,
        /// Task display title (optional)
        title: Option<String>,
    },
    /// Task management
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },
    /// Initialize a DiJiang project
    Init {
        /// Project name (defaults to directory name)
        #[arg(default_value = "")]
        name: String,
        /// Developer name (detected from git if not provided)
        #[arg(long)]
        developer: Option<String>,
        /// Skip interactive prompts, use defaults
        /// Skip interactive prompts, use defaults
        #[arg(long, short = 'y')]
        yes: bool,
        /// Force re-initialization even if already initialized
        #[arg(long)]
        force: bool,
        /// Comma-separated platforms to configure (pi,cursor,claude,codex,opencode,hermes)
        #[arg(long)]
        platforms: Option<String>,
        /// Auto-detect installed platforms
        #[arg(long)]
        auto_detect: bool,
    },
    Mem {
        #[command(subcommand)]
        command: MemCommands,
    },
    /// Template management
    Template {
        #[command(subcommand)]
        command: TemplateCommands,
    },
    /// Manage dj-* skills (list, sync to project)
    Skills {
        /// Sync skills to current project
        #[arg(long)]
        sync: bool,
    },
    /// Migrate a Trellis project to DiJiang
    Migrate,
}
#[derive(Subcommand)]
enum MemCommands {
    /// List sessions across platforms
    List,
    /// Sync all platform sessions into ~/.dijiang/mem/
    Sync,
    /// Append a finding to the current session journal
    Findings {
        /// The finding to record
        #[arg(long)]
        finding: String,
    },
    /// Write a lesson learned to the current session journal
    Learn {
        /// The lesson to record
        #[arg(long)]
        lesson: String,
    },
    /// Archive the current session (write summary to journal)
    Archive,
}

#[derive(Subcommand)]
enum TemplateCommands {
    /// List available templates (built-in and cached)
    List,
    /// Pull a template from a source (gh:owner/repo or URL)
    Pull {
        /// Template source (e.g. gh:tiezhu/dijiang-templates)
        source: String,
    },
    /// Validate a template directory
    Validate {
        /// Path to template directory or manifest.toml
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
        Commands::Template { command } => match command {
            TemplateCommands::List => cmd_template_list(),
            TemplateCommands::Pull { source } => cmd_template_pull(&source),
            TemplateCommands::Validate { path } => cmd_template_validate(&path),
        },
        Commands::Skills { sync } => cmd_skills(sync),
        Commands::Migrate => cmd_migrate(),
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
    let skills_written = dijiang_configurator::write_project_skills(&cwd)?;
    if skills_written > 0 {
        println!("  Wrote {} dj-* skills to .pi/skills/", skills_written);
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
