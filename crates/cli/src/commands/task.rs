use std::collections::HashMap;
use crate::util::require_dijiang_dir;
use dijiang_task::store;
use dijiang_task::types::TaskStatus;
use dijiang_task::TaskRecord;

pub fn cmd_task_list() -> anyhow::Result<()> {
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

pub fn cmd_task_current() -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    match store::read_active_task(&dijiang_dir)? {
        Some(name) => println!("{name}"),
        None => println!("(none)"),
    }
    Ok(())
}

pub fn cmd_task_start(name: &str) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");

    match store::load_task(&tasks_dir, name) {
        Ok(mut task) => {
            task.status = TaskStatus::InProgress;
            store::save_task(&tasks_dir, &task)?;
        }
        Err(store::TaskError::NotFound(_)) => {
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

pub fn cmd_task_status(name: &str, status_str: &str) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
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

pub fn cmd_task_archive(name: &str) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
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

pub fn cmd_task_prune(days: u64) -> anyhow::Result<()> {
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

pub fn cmd_task_context_add(file: &str, action: &str, reason: &str) -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");

    // Get the active task name
    let task_name = match store::read_active_task(&dijiang_dir)? {
        Some(name) => name,
        None => {
            eprintln!("No active task. Set one with 'dijiang task start <name>'.");
            std::process::exit(1);
        }
    };

    let entry = store::ContextEntry {
        action: action.to_string(),
        file: file.to_string(),
        reason: reason.to_string(),
    };

    store::add_context_entry(&tasks_dir, &task_name, &entry)?;
    println!("✓ Added context entry: {} ({}) — {}", file, action, reason);
    Ok(())
}

pub fn cmd_task_context_list(action: Option<&str>) -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");

    let task_name = match store::read_active_task(&dijiang_dir)? {
        Some(name) => name,
        None => {
            eprintln!("No active task.");
            std::process::exit(1);
        }
    };

    let actions: &[&str] = match action {
        Some(a) => &[a],
        None => &["implement", "check"],
    };

    for a in actions {
        let entries = store::list_context_entries(&tasks_dir, &task_name, a)?;
        if entries.is_empty() {
            println!("[{}] (no entries)", a);
        } else {
            println!("[{}]", a);
            for entry in &entries {
                println!("  {} — {}", entry.file, entry.reason);
            }
        }
    }
    Ok(())
}

pub fn cmd_task_link(parent: &str, child: &str) -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");

    match store::link_tasks(&tasks_dir, parent, child) {
        Ok(()) => {
            println!("✓ Linked {} as child of {}", child, parent);
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to link tasks: {}", e);
            std::process::exit(1);
        }
    }
}

pub fn cmd_task_unlink(child: &str) -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");

    match store::unlink_task(&tasks_dir, child) {
        Ok(()) => {
            println!("✓ Unlinked {}", child);
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to unlink task: {}", e);
            std::process::exit(1);
        }
    }
}

pub fn cmd_task_tree() -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");
    let tasks = store::list_tasks(&tasks_dir).unwrap_or_default();
    if tasks.is_empty() {
        println!("(no tasks)");
        return Ok(());
    }

    // Find root tasks (no parent, or parent not in our task list)
    let roots: Vec<_> = tasks
        .iter()
        .filter(|t| {
            t.parent.is_none()
                || !tasks.iter().any(|other| Some(other.id.as_str()) == t.parent.as_deref())
        })
        .collect();

    fn print_tree(tasks: &[TaskRecord], task_name: &str, indent: usize) {
        let indent_str = "  ".repeat(indent);
        if let Some(task) = tasks.iter().find(|t| t.name == task_name) {
            println!(
                "{}- {} [{}]",
                indent_str, task.title, task.status.as_str()
            );
            for child_id in &task.children {
                if let Some(child) = tasks.iter().find(|t| t.id == *child_id) {
                    print_tree(tasks, &child.name, indent + 1);
                }
            }
        }
    }

    for root in &roots {
        print_tree(&tasks, &root.name, 0);
    }

    Ok(())
}

// ── Lifecycle Hooks ────────────────────────────────────────────────

pub fn cmd_task_hook_list(task_name: &str, event: Option<&str>) -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");

    match store::get_task_hooks(&tasks_dir, task_name) {
        Ok(Some(hooks)) => {
            for (ev, cmds) in &hooks {
                if event.map_or(true, |e| e == ev) {
                    for (i, cmd) in cmds.iter().enumerate() {
                        println!("  [{}] {} #{}: {}", task_name, ev, i, cmd);
                    }
                }
            }
            Ok(())
        }
        Ok(None) => {
            println!("No hooks configured for task '{}'", task_name);
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!("Failed to read hooks: {}", e)),
    }
}

pub fn cmd_task_hook_add(task_name: &str, event: &str, cmd: &str) -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");

    let mut hooks = match store::get_task_hooks(&tasks_dir, task_name) {
        Ok(Some(h)) => h,
        Ok(None) => HashMap::new(),
        Err(e) => return Err(anyhow::anyhow!("Failed to read hooks: {}", e)),
    };

    hooks.entry(event.to_string()).or_default().push(cmd.to_string());
    store::set_task_hooks(&tasks_dir, task_name, hooks)?;
    println!("✓ Added hook [{}] {}: {}", task_name, event, cmd);
    Ok(())
}

pub fn cmd_task_hook_remove(task_name: &str, event: &str, index: usize) -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");

    let mut hooks = match store::get_task_hooks(&tasks_dir, task_name) {
        Ok(Some(h)) => h,
        Ok(None) => return Err(anyhow::anyhow!("No hooks for task '{}'", task_name)),
        Err(e) => return Err(anyhow::anyhow!("Failed to read hooks: {}", e)),
    };

    let cmds = hooks.get_mut(event).ok_or_else(|| {
        anyhow::anyhow!("No hooks for event '{}' on task '{}'", event, task_name)
    })?;

    if index >= cmds.len() {
        return Err(anyhow::anyhow!("Index {} out of range (0..{})", index, cmds.len()));
    }
    cmds.remove(index);

    if cmds.is_empty() {
        hooks.remove(event);
    }

    store::set_task_hooks(&tasks_dir, task_name, hooks)?;
    println!("✓ Removed hook [{}] {} #{} ", task_name, event, index);
    Ok(())
}

pub fn cmd_task_hook_run(task_name: &str, event: &str) -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");

    store::run_task_hooks(&tasks_dir, task_name, event)
        .map_err(|e| anyhow::anyhow!("Hook execution failed: {}", e))
}
