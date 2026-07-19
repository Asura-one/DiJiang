use crate::util::require_dijiang_dir;
use dijiang_task::hooks::{self, HookEvent};
use dijiang_task::store;
use dijiang_task::types::TaskStatus;
use chrono;

pub fn cmd_start(name: &str, title: Option<&str>) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    let tasks_dir = dijiang_dir.join("tasks");
    let now = chrono::Utc::now();

    let mut task = match store::load_task(&tasks_dir, name) {
        Ok(mut task) => {
            let was_status = task.status.as_str().to_string();
            if matches!(task.status, TaskStatus::Archived) {
                task.status = TaskStatus::Planning;
            }
            task.started_at = task.started_at.take().or(Some(now.to_rfc3339()));
            println!("  ✓ Task '{name}' updated");
            println!("    Status: {was_status} → {status}", status = task.status.as_str());
            task
        }
        Err(store::TaskError::NotFound(_)) => {
            let display_title = title.unwrap_or(name);
            let mut task = store::create_task(name, display_title);
            task.status = TaskStatus::Planning;
            task.started_at = Some(now.to_rfc3339());
            println!("  ✓ Task '{name}' created");
            println!("    Title: {display_title}");
            println!("    Status: planning");
            task
        }
        Err(e) => {
            eprintln!("Error accessing task: {e}");
            std::process::exit(1);
        }
    };

    store::activate_new_task(&dijiang_dir, &task)?;
    hooks::run_task_hooks(&dijiang_dir, HookEvent::AfterTaskStart, name);
    println!("  ✓ Session started\n");

    let project_name = dijiang_dir
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("(unknown)");
    println!("  Project: {project_name}");
    println!("  Active:  .dijiang/tasks/{name}\n");

    println!("  Task summary:");
    println!("    Title:  {title}", title = task.title);
    println!("    State:  {status}", status = task.status.as_str());
    println!("    Phase:  {phase}", phase = task.status.infer_phase());
    if let Some(ac) = &task.acceptance_criteria {
        println!("    Goals:  {ac}");
    }
    println!();
    Ok(())
}
