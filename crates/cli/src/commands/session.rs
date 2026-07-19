use crate::util::require_dijiang_dir;
use dijiang_task::developer::DeveloperContext;
use std::io::Read;
use std::path::Path;
use std::fs;

/// Add a session entry to the journal and update index.md.
pub fn cmd_session_add(
    title: &str,
    summary: &str,
    branch: Option<&str>,
    stdin_content: bool,
) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    let dev = DeveloperContext::new(&dijiang_dir);

    // Ensure workspace dir exists
    fs::create_dir_all(&dev.workspace)?;

    // Resolve branch: CLI arg → active task → "(none)"
    let branch_str = match branch {
        Some(b) => b.to_string(),
        None => {
            match dijiang_task::store::read_active_task(&dijiang_dir)? {
                Some(task_name) => {
                    let tasks_dir = dijiang_dir.join("tasks");
                    match dijiang_task::store::load_task(&tasks_dir, &task_name) {
                        Ok(task) => task.branch.unwrap_or_else(|| "(none)".to_string()),
                        Err(_) => "(none)".to_string(),
                    }
                }
                None => "(none)".to_string(),
            }
        }
    };

    // Read stdin content if --stdin
    let extra_content = if stdin_content {
        let mut buf = String::new();
        std::io::stdin()
            .lock()
            .read_to_string(&mut buf)
            .map(|_| buf.trim().to_string())
            .unwrap_or_default()
    } else {
        String::new()
    };

    // ── Append to journal.md ────────────────────────────────────────
    let today = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    let journal_entry = format!(
        "\n## Session: {title}\n\n"
    );
    let mut journal_content = journal_entry.clone();
    journal_content.push_str(&format!("**Date**: {today}\n"));
    journal_content.push_str(&format!("**Branch**: `{branch_str}`\n\n"));
    journal_content.push_str(&format!("{summary}\n"));
    if !extra_content.is_empty() {
        journal_content.push_str(&format!("\n### Details\n\n{extra_content}\n"));
    }
    journal_content.push_str("\n---\n");

    // Append to journal.md
    let journal_path = &dev.journal;
    fs::write(journal_path, {
        let mut content = String::new();
        if journal_path.exists() {
            content = fs::read_to_string(journal_path)?;
        }
        content.push_str(&journal_content);
        content
    })?;
    println!("  ✓ Appended to {}", journal_path.display());

    // ── Update index.md ─────────────────────────────────────────────
    let index_path = dijiang_dir.join("workspace").join("index.md");
    update_index(&index_path, &dev.name, title, &today, &branch_str)?;

    println!("  ✓ Session saved: {title}");
    Ok(())
}

fn update_index(
    index_path: &Path,
    developer: &str,
    title: &str,
    date: &str,
    branch: &str,
) -> anyhow::Result<()> {
    let mut content = if index_path.exists() {
        fs::read_to_string(index_path)?
    } else {
        String::new()
    };

    // Simple append-only history table
    let entry = format!("| {date} | {title} | `{branch}` | {developer} |\n");

    // Find or create the history table
    if content.contains("| Date | Title | Branch | Developer |") {
        // Append after the header separator
        if let Some(pos) = content.find("|---|---|") {
            let insert_at = content[pos..].find('\n').map(|p| pos + p + 1).unwrap_or(content.len());
            content.insert_str(insert_at, &entry);
        } else {
            content.push_str(&entry);
        }
    } else {
        // Create the table
        content.push_str("\n## Session History\n\n");
        content.push_str("| Date | Title | Branch | Developer |\n");
        content.push_str("|---|---|---|---|\n");
        content.push_str(&entry);
    }

    fs::write(index_path, content)?;
    println!("  ✓ Updated {}", index_path.display());
    Ok(())
}
