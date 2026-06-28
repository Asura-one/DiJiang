use clap::{Parser, Subcommand};
use dijiang_task::store;
use dijiang_task::types::TaskStatus;

#[derive(Parser)]
#[command(name = "dijiang", version = "0.1.0", about = "DiJiang - AI coding assistant workflow layer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show project status
    Status,
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
        /// Developer name
        #[arg(long)]
        developer: Option<String>,
    },
    /// Memory commands
    Mem {
        #[command(subcommand)]
        command: MemCommands,
    },
}

#[derive(Subcommand)]
enum TaskCommands {
    /// List tasks
    List,
    /// Show current task
    Current,
    /// Start a task
    Start { name: String },
    /// Update task status
    Status {
        /// Task name (slug)
        name: String,
        /// New status: planning|in_progress|completed|archived|paused
        status: String,
    },
}

#[derive(Subcommand)]
enum MemCommands {
    /// List sessions across platforms
    List,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { name, title } => cmd_start(&name, title.as_deref()),
        Commands::Status => cmd_status(),
        Commands::Init { name, developer } => cmd_init(&name, developer.as_deref()),
        Commands::Task { command } => match command {
            TaskCommands::List => cmd_task_list(),
            TaskCommands::Current => cmd_task_current(),
            TaskCommands::Start { name } => cmd_task_start(&name),
            TaskCommands::Status { name, status } => cmd_task_status(&name, &status),
        },
        Commands::Mem { command: MemCommands::List } => cmd_mem_list(),
    }
}

fn status_line(label: &str, value: impl std::fmt::Display) {
    println!("  {label:15} {value}");
}

fn cmd_status() -> anyhow::Result<()> {
    println!("\n  ── DiJiang Status ──\n");

    let cwd = std::env::current_dir()?;
    let trellis_dir = match store::find_trellis_dir(&cwd) {
        Some(d) => d,
        None => {
            println!("  No .trellis/ found. Run `dijiang init` first.");
            return Ok(());
        }
    };

    // Project name — read from .dijiang/config.toml, then fall back to directory
    let name = dijiang_configurator::read_project_name(&cwd)
        .or_else(|| {
            trellis_dir.parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "(unknown)".to_string());
    status_line("Project:", &name);
    // Active task
    let active = store::read_active_task(&trellis_dir).unwrap_or(None);
    match active {
        Some(ref t) => status_line("Active Task:", t),
        None => status_line("Active Task:", "(none)"),
    }

    // Task list
    let tasks_dir = trellis_dir.join("tasks");
    let tasks = store::list_tasks(&tasks_dir).unwrap_or_default();
    println!("  Tasks ({count}):", count = tasks.len());
    for t in &tasks {
        let marker = active.as_ref().map_or(' ', |a| if a == &t.name { '*' } else { ' ' });
        println!("    {marker} {name:<45} {status:12}",
            name = t.name,
            status = t.status.as_str(),
        );
    }

    // Platform check
    let pi_dir = trellis_dir.parent().map(|p| p.join(".pi"));
    if pi_dir.as_ref().is_some_and(|p| p.exists()) {
        println!("  Pi:              ✓ configured");
    }

    println!();
    Ok(())
}

fn cmd_task_list() -> anyhow::Result<()> {
    let trellis_dir = match store::find_trellis_dir(&std::env::current_dir()?) {
        Some(d) => d,
        None => {
            println!("No .trellis/ found.");
            return Ok(());
        }
    };

    let tasks_dir = trellis_dir.join("tasks");
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
    let trellis_dir = match store::find_trellis_dir(&std::env::current_dir()?) {
        Some(d) => d,
        None => {
            println!("No .trellis/ found.");
            return Ok(());
        }
    };

    match store::read_active_task(&trellis_dir)? {
        Some(name) => println!("{name}"),
        None => println!("(none)"),
    }
    Ok(())
}

fn cmd_task_start(name: &str) -> anyhow::Result<()> {
    let trellis_dir = match store::find_trellis_dir(&std::env::current_dir()?) {
        Some(d) => d,
        None => {
            eprintln!("No .trellis/ found.");
            std::process::exit(1);
        }
    };

    let tasks_dir = trellis_dir.join("tasks");

    // Ensure task exists
    match store::load_task(&tasks_dir, name) {
        Ok(mut task) => {
            task.status = TaskStatus::InProgress;
            store::save_task(&tasks_dir, &task)?;
        }
        Err(store::TaskError::NotFound(_)) => {
            eprintln!("Task '{name}' not found.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error loading task: {e}");
            std::process::exit(1);
        }
    }

    store::write_active_task(&trellis_dir, name)?;
    println!("✓ Current task set to: .trellis/tasks/{name}");
    println!("  Status: planning → in_progress");
    Ok(())
}

fn cmd_task_status(name: &str, status_str: &str) -> anyhow::Result<()> {
    let trellis_dir = match store::find_trellis_dir(&std::env::current_dir()?) {
        Some(d) => d,
        None => {
            eprintln!("No .trellis/ found.");
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

    let tasks_dir = trellis_dir.join("tasks");
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


fn cmd_init(name: &str, developer: Option<&str>) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let project_name = if name.is_empty() {
        cwd.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("my-project")
            .to_string()
    } else {
        name.to_string()
    };

    dijiang_configurator::init_project(&cwd, &project_name, developer)?;
    Ok(())
}

fn cmd_start(name: &str, title: Option<&str>) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let trellis_dir = match store::find_trellis_dir(&cwd) {
        Some(d) => d,
        None => {
            eprintln!("No .trellis/ found. Run `dijiang init` first.");
            std::process::exit(1);
        }
    };

    let tasks_dir = trellis_dir.join("tasks");
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

    store::write_active_task(&trellis_dir, name)?;

    // Print startup summary
    println!("  ✓ Session started");
    println!();

    // Show project and active task summary
    let project_name = trellis_dir
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("(unknown)");
    println!("  Project: {project_name}");
    println!("  Active:  .trellis/tasks/{name}");
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
