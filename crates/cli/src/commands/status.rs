use crate::util::require_dijiang_dir;
use dijiang_task::store;
use std::path::Path;

fn status_line(label: &str, value: impl std::fmt::Display) {
    println!("  {label:15} {value}");
}

pub fn cmd_status(compat: bool) -> anyhow::Result<()> {
    println!("\n  ── DiJiang Status ──\n");

    let cwd = std::env::current_dir()?;
    let dijiang_dir = require_dijiang_dir()?;

    let name = dijiang_configurator::read_project_name(&cwd);
    status_line("项目:", &name);

    let active = store::read_active_task(&dijiang_dir).unwrap_or(None);
    let tasks_dir = dijiang_dir.join("tasks");
    let tasks = store::list_tasks(&tasks_dir).unwrap_or_default();

    match &active {
        Some(t) => {
            status_line("当前任务:", t);
            if let Some(task) = tasks.iter().find(|x| &x.name == t) {
                status_line("状态:", task.status.as_str());
                status_line("阶段:", task.status.to_trellis_status());
                status_line("兼容:", "yes");
            }
        }
        None => status_line("当前任务:", "(none)"),
    }

    println!("  任务 ({count}):", count = tasks.len());
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
            println!("  DiJiang project: ✓ detected");
        } else {
            println!("  DiJiang project: ✗ not detected");
        }
    }

    println!();
    Ok(())
}
