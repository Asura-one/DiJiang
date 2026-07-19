use std::collections::HashMap;
use crate::util::require_dijiang_dir;
use dijiang_task::hooks::{self, HookEvent};
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

pub fn cmd_task_start(name: &str, parent: Option<&str>) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");

    let mut task = match store::load_task(&tasks_dir, name) {
        Ok(task) => {
            if let Some(parent_name) = parent {
                store::link_tasks(&tasks_dir, parent_name, name)?;
                println!("✓ Linked {} as child of {}", name, parent_name);
            }
            task
        }
        Err(store::TaskError::NotFound(_)) => {
            let t = store::create_task(name, name);
            println!("✓ Created task: {name}");
            t
        }
        Err(e) => {
            eprintln!("Error loading task: {e}");
            std::process::exit(1);
        }
    };
    task.status = TaskStatus::InProgress;
    store::activate_new_task(&dijiang_dir, &task)?;
    hooks::run_task_hooks(&dijiang_dir, HookEvent::AfterTaskStart, name);
    // If parent specified and task already existed, link was handled above
    if let Some(parent_name) = parent {
        // For newly created tasks, link now
        store::link_tasks(&tasks_dir, parent_name, name)?;
        println!("✓ Linked {} as child of {}", name, parent_name);
    }
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
            hooks::run_task_hooks(&dijiang_dir, HookEvent::AfterTaskArchive, name);
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

pub fn cmd_task_set_scope(name: &str, scope: &str) -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");
    let mut task = match store::load_task(&tasks_dir, name) {
        Ok(task) => task,
        Err(e) => {
            eprintln!("Error loading task '{name}': {e}");
            std::process::exit(1);
        }
    };
    task.scope = Some(scope.to_string());
    store::save_task(&tasks_dir, &task)?;
    println!("✓ Scope set for task '{name}': {scope}");
    Ok(())
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

pub fn cmd_task_checklist_list() -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");
    let active = store::read_active_task(&dijiang_dir)?;
    let task_name = active.as_deref().unwrap_or("<no active task>");
    let checklist = store::get_checklist(&tasks_dir, task_name)?;
    if checklist.criteria.is_empty() {
        println!("Checklist is empty. Use `dijiang task checklist add <description>` to add items.");
    }
    for (i, item) in checklist.criteria.iter().enumerate() {
        let status = if item.met { "[x]" } else { "[ ]" };
        println!("{} {}: {}", status, i, item.description);
    }
    Ok(())
}

pub fn cmd_task_checklist_add(description: &str) -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");
    let active = store::read_active_task(&dijiang_dir)?;
    let task_name = active.as_deref().unwrap_or("<no active task>");
    store::add_checklist_item(&tasks_dir, task_name, description)?;
    println!("Added checklist item: {}", description);
    Ok(())
}

pub fn cmd_task_checklist_check(index: usize) -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");
    let active = store::read_active_task(&dijiang_dir)?;
    let task_name = active.as_deref().unwrap_or("<no active task>");
    store::set_checklist_item(&tasks_dir, task_name, index, true)?;
    println!("Checklist item {} marked as done.", index);
    Ok(())
}

pub fn cmd_task_checklist_uncheck(index: usize) -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");
    let active = store::read_active_task(&dijiang_dir)?;
    let task_name = active.as_deref().unwrap_or("<no active task>");
    store::set_checklist_item(&tasks_dir, task_name, index, false)?;
    println!("Checklist item {} marked as not done.", index);
    Ok(())
}

pub fn cmd_task_checklist_remove(index: usize) -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");
    let active = store::read_active_task(&dijiang_dir)?;
    let task_name = active.as_deref().unwrap_or("<no active task>");
    store::remove_checklist_item(&tasks_dir, task_name, index)?;
    println!("Removed checklist item {}.", index);
    Ok(())
}

// ── Queue commands ──────────────────────────────────────────────────

pub fn cmd_task_queue_list() -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    let queue = store::read_task_queue(&dijiang_dir);
    if queue.is_empty() {
        println!("Task queue is empty.");
    } else {
        println!("── Task Queue ────────────────────────────────");
        for (i, task) in queue.iter().enumerate() {
            println!("  {}. {task}", i + 1);
        }
    }
    Ok(())
}

pub fn cmd_task_queue_add(name: &str) -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    if store::queue_add(&dijiang_dir, name) {
        println!("Added {name} to queue.");
    } else {
        println!("{name} is already in the queue.");
    }
    Ok(())
}

pub fn cmd_task_queue_remove(name: &str) -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    if store::queue_remove(&dijiang_dir, name) {
        println!("Removed {name} from queue.");
    } else {
        println!("{name} not found in queue.");
    }
    Ok(())
}

pub fn cmd_task_queue_next() -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    match store::queue_pop(&dijiang_dir) {
        Some(task) => {
            println!("Next task: {task}");
            // Optionally start the task
            let tasks_dir = dijiang_dir.join("tasks");
            if let Ok(mut record) = store::load_task(&tasks_dir, &task) {
                record.status = dijiang_task::types::TaskStatus::InProgress;
                if store::activate_new_task(&dijiang_dir, &record).is_ok() {
                    println!("  Activated: {task}");
                }
            }
        }
        None => {
            println!("Queue is empty.");
        }
    }
    Ok(())
}

pub fn cmd_task_queue_clear() -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    store::write_task_queue(&dijiang_dir, &[])?;
    println!("Queue cleared.");
    Ok(())
}

// ── Validate ──────────────────────────────────────────────────────

/// Validate a task's structure and context files.
/// Checks: task.json integrity, JSONL context file validity, link consistency.
pub fn cmd_task_validate(name: Option<&str>) -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");
    let task_name = match name {
        Some(n) => n.to_string(),
        None => match store::read_active_task(&dijiang_dir)? {
            Some(n) => n,
            None => {
                eprintln!("Error: no active task and no task name provided");
                std::process::exit(1);
            }
        },
    };
    let task_dir = tasks_dir.join(&task_name);
    let mut total_errors = 0;

    println!("Validating task: {}", task_name);
    println!("  Dir: {}", task_dir.display());
    println!();

    // 1. Validate task.json
    total_errors += validate_task_json(&task_dir, &task_name);

    // 2. Validate context JSONL files (implement.jsonl, check.jsonl)
    total_errors += validate_context_files(&task_dir);

    // 3. Validate link consistency
    total_errors += validate_links(&tasks_dir, &task_name);

    println!();
    if total_errors == 0 {
        println!("✓ All validations passed");
    } else {
        eprintln!("✗ {} validation error(s) found", total_errors);
        std::process::exit(1);
    }
    Ok(())
}

fn validate_task_json(task_dir: &std::path::Path, task_name: &str) -> usize {
    let json_path = task_dir.join("task.json");
    let mut errors = 0;
    if !json_path.exists() {
        eprintln!("  ✗ task.json not found");
        return 1;
    }
    let content = match std::fs::read_to_string(&json_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("  ✗ task.json: read error: {}", e);
            return 1;
        }
    };
    let task: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("  ✗ task.json: invalid JSON: {}", e);
            return 1;
        }
    };
    // Required fields
    let required = ["id", "name", "title", "status", "createdAt"];
    for field in &required {
        if !task.get(*field).is_some() {
            eprintln!("  ✗ task.json: missing required field '{}'", field);
            errors += 1;
        }
    }
    // Validate status
    if let Some(status) = task.get("status").and_then(|s| s.as_str()) {
        let valid = ["planning", "in_progress", "completed", "archived", "paused"];
        if !valid.contains(&status) {
            eprintln!("  ✗ task.json: unknown status '{}'", status);
            errors += 1;
        }
    }
    // Validate name matches directory
    if let Some(n) = task.get("name").and_then(|n| n.as_str()) {
        if n != task_name {
            eprintln!("  ✗ task.json: name '{}' does not match directory '{}'", n, task_name);
            errors += 1;
        }
    }
    if errors == 0 {
        println!("  ✓ task.json: structure OK");
    }
    errors
}

fn validate_context_files(task_dir: &std::path::Path) -> usize {
    let mut errors = 0;
    let repo_root = std::env::current_dir().unwrap_or_else(|_| task_dir.to_path_buf());
    for jsonl_name in ["implement.jsonl", "check.jsonl"] {
        let jsonl_path = task_dir.join(jsonl_name);
        if !jsonl_path.exists() {
            println!("    {}: not found (skipped)", jsonl_name);
            continue;
        }
        let content = match std::fs::read_to_string(&jsonl_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("  ✗ {}: read error: {}", jsonl_name, e);
                errors += 1;
                continue;
            }
        };
        let mut real_entries = 0;
        let mut file_errors = 0;
        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let entry: serde_json::Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("  ✗ {}:{} invalid JSON: {}", jsonl_name, line_num + 1, e);
                    errors += 1;
                    continue;
                }
            };
            let file_path = entry.get("file").and_then(|f| f.as_str());
            let file_path = match file_path {
                Some(f) => f,
                None => continue, // seed row, skip
            };
            real_entries += 1;
            let entry_type = entry.get("type").and_then(|t| t.as_str()).unwrap_or("file");
            let full_path = repo_root.join(file_path);
            if entry_type == "directory" {
                if !full_path.is_dir() {
                    eprintln!("  ✗ {}:{} directory not found: {}", jsonl_name, line_num + 1, file_path);
                    file_errors += 1;
                }
            } else if !full_path.is_file() {
                eprintln!("  ✗ {}:{} file not found: {}", jsonl_name, line_num + 1, file_path);
                file_errors += 1;
            }
        }
        if file_errors == 0 {
            println!("    {}: ✓ ({} entries)", jsonl_name, real_entries);
        } else {
            println!("    {}: ✗ ({} errors)", jsonl_name, file_errors);
            errors += file_errors;
        }
    }
    errors
}

fn validate_links(tasks_dir: &std::path::Path, task_name: &str) -> usize {
    let mut errors = 0;
    let all_tasks = store::list_tasks(tasks_dir).unwrap_or_default();
    let current = match all_tasks.iter().find(|t| t.name == *task_name) {
        Some(t) => t,
        None => return 0,
    };
    // Check parent exists
    if let Some(parent_id) = &current.parent {
        if !all_tasks.iter().any(|t| t.id == *parent_id) {
            eprintln!("  ✗ child links to parent '{}' which does not exist", parent_id);
            errors += 1;
        }
    }
    // Check children exist
    for child_id in &current.children {
        if !all_tasks.iter().any(|t| t.id == *child_id) {
            eprintln!("  ✗ parent lists child '{}' which does not exist", child_id);
            errors += 1;
        }
    }
    if errors == 0 {
        println!("  ✓ links: consistent");
    }
    errors
}

// ── Create PR ───────────────────────────────────────────────────────

/// Create a GitHub Pull Request from a task.
/// Uses `gh` CLI. Saves PR URL back to task.json.
pub fn cmd_task_create_pr(
    name: Option<&str>,
    title_override: Option<&str>,
    body: Option<&str>,
    branch_override: Option<&str>,
    base_override: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");
    let task_name = match name {
        Some(n) => n.to_string(),
        None => match store::read_active_task(&dijiang_dir)? {
            Some(n) => n,
            None => {
                eprintln!("Error: no active task and no task name provided");
                std::process::exit(1);
            }
        },
    };
    let mut task = match store::load_task(&tasks_dir, &task_name) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error: cannot load task '{}': {}", task_name, e);
            std::process::exit(1);
        }
    };

    // Resolve branch
    let branch = branch_override
        .map(|s| s.to_string())
        .or_else(|| task.branch.clone())
        .or_else(|| crate::util::git_current_branch(&dijiang_dir).ok());

    // Resolve base branch
    let base = base_override
        .map(|s| s.to_string())
        .or_else(|| task.base_branch.clone())
        .unwrap_or_else(|| "main".to_string());

    let branch = match branch {
        Some(b) => b,
        None => {
            eprintln!("Error: cannot determine branch. Set with --branch or `task set-branch`.");
            std::process::exit(1);
        }
    };

    // Resolve title
    let pr_title = title_override
        .map(|s| s.to_string())
        .unwrap_or_else(|| task.title.clone());

    // Resolve body
    let pr_body = body
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("Automated PR for task: {}\n\n**Task**: {}\n**Scope**: {}",
            task.title, task_name, task.scope.as_deref().unwrap_or("(none)")));

    println!("── Creating PR ────────────────────────────────");
    println!("  Task:   {}", task_name);
    println!("  Title:  {}", pr_title);
    println!("  Branch: {}", branch);
    println!("  Base:   {}", base);

    if dry_run {
        println!("  (dry-run) Would execute: gh pr create --title \"{}\" --head {} --base {} --body \"<body>\"",
            pr_title, branch, base);
        println!("  (dry-run) Would save pr_url to task.json");
        println!("  ✓ DRY-RUN: no changes made");
        return Ok(());
    }

    // Run gh pr create
    let output = std::process::Command::new("gh")
        .args(["pr","create","--title", &pr_title, "--head", &branch, "--base", &base])
        .arg("--body")
        .arg(&pr_body)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Error: gh pr create failed:");
        eprintln!("{}", stderr);
        if stderr.contains("could not find any remote") {
            eprintln!("Hint: ensure your branch is pushed: git push origin {}", branch);
        }
        std::process::exit(1);
    }

    let pr_url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    println!("  PR URL: {}", pr_url);

    // Save pr_url and branch/base to task.json
    task.pr_url = Some(pr_url.clone());
    task.branch = Some(branch.clone());
    task.base_branch = Some(base);
    store::save_task(&tasks_dir, &task)?;
    println!("  ✓ PR URL saved to task.json");

    Ok(())
}

// ── Set branch / base-branch ────────────────────────────────────────

/// Set the git branch for a task.
pub fn cmd_task_set_branch(name: Option<&str>, branch: &str) -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");
    let task_name = match name {
        Some(n) => n.to_string(),
        None => match store::read_active_task(&dijiang_dir)? {
            Some(n) => n,
            None => {
                eprintln!("Error: no active task and no task name provided");
                std::process::exit(1);
            }
        },
    };
    let mut task = store::load_task(&tasks_dir, &task_name)?;
    task.branch = Some(branch.to_string());
    store::save_task(&tasks_dir, &task)?;
    println!("✓ Set branch for '{}' to '{}'", task_name, branch);
    Ok(())
}

/// Set the base (PR target) branch for a task.
pub fn cmd_task_set_base_branch(name: Option<&str>, branch: &str) -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");
    let task_name = match name {
        Some(n) => n.to_string(),
        None => match store::read_active_task(&dijiang_dir)? {
            Some(n) => n,
            None => {
                eprintln!("Error: no active task and no task name provided");
                std::process::exit(1);
            }
        },
    };
    let mut task = store::load_task(&tasks_dir, &task_name)?;
    task.base_branch = Some(branch.to_string());
    store::save_task(&tasks_dir, &task)?;
    println!("✓ Set base branch for '{}' to '{}'", task_name, branch);
    Ok(())
}

/// Add a dependency (another task this task depends on).
pub fn cmd_task_add_dep(name: Option<&str>, depends_on: &[String]) -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");
    let task_name = match name {
        Some(n) => n.to_string(),
        None => match store::read_active_task(&dijiang_dir)? {
            Some(n) => n,
            None => {
                eprintln!("Error: no active task and no task name provided");
                std::process::exit(1);
            }
        },
    };
    let mut task = store::load_task(&tasks_dir, &task_name)?;
    let deps = task.depends_on.get_or_insert_with(Vec::new);
    for dep in depends_on {
        if !deps.contains(dep) {
            deps.push(dep.clone());
            println!("✓ Added dependency: {} depends on {}", task_name, dep);
        } else {
            println!("  Dependency already exists: {} -> {}", task_name, dep);
        }
    }
    store::save_task(&tasks_dir, &task)?;
    Ok(())
}

/// Remove a dependency.
pub fn cmd_task_remove_dep(name: Option<&str>, depends_on: &[String]) -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");
    let task_name = match name {
        Some(n) => n.to_string(),
        None => match store::read_active_task(&dijiang_dir)? {
            Some(n) => n,
            None => {
                eprintln!("Error: no active task and no task name provided");
                std::process::exit(1);
            }
        },
    };
    let mut task = store::load_task(&tasks_dir, &task_name)?;
    if let Some(deps) = &mut task.depends_on {
        for dep in depends_on {
            if deps.contains(dep) {
                deps.retain(|d| d != dep);
                println!("✓ Removed dependency: {} -> {}", task_name, dep);
            } else {
                println!("  No such dependency: {} -> {}", task_name, dep);
            }
        }
        if deps.is_empty() {
            task.depends_on = None;
        }
    } else {
        println!("  No dependencies defined for '{}'", task_name);
    }
    store::save_task(&tasks_dir, &task)?;
    Ok(())
}

/// List dependencies for a task.
pub fn cmd_task_list_deps(name: Option<&str>) -> anyhow::Result<()> {
    let dijiang_dir = crate::util::require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");
    let task_name = match name {
        Some(n) => n.to_string(),
        None => match store::read_active_task(&dijiang_dir)? {
            Some(n) => n,
            None => {
                eprintln!("Error: no active task and no task name provided");
                std::process::exit(1);
            }
        },
    };
    let task = store::load_task(&tasks_dir, &task_name)?;
    match &task.depends_on {
        Some(deps) if !deps.is_empty() => {
            println!("Dependencies for '{}':", task_name);
            for dep in deps {
                println!("  - {}", dep);
            }
        }
        _ => {
            println!("No dependencies for '{}'", task_name);
        }
    }
    Ok(())
}
