use crate::util::{require_dijiang_dir, read_project_name, current_session_key};
use dijiang_task::store;
use std::path::Path;

fn current_project_memory(dijiang_dir: &Path) -> anyhow::Result<dijiang_mem::ProjectMemory> {
    dijiang_mem::ProjectMemory::from_dijiang_dir(dijiang_dir)
}

pub fn cmd_mem_findings(finding: &str) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    let project = read_project_name(&dijiang_dir).unwrap_or_else(|_| "unknown".to_string());
    let (session_key, _) = current_session_key();
    let mem = current_project_memory(&dijiang_dir)?;
    let record = dijiang_mem::Finding {
        timestamp: chrono::Local::now().to_rfc3339(),
        content: finding.to_string(),
        session_id: Some(session_key),
        project: Some(project),
        tags: vec![],
        scope: dijiang_mem::MemoryScope::Project,
    };
    mem.append_finding(&record)?;
    println!("  Finding recorded to {}", mem.root().join("findings.jsonl").display());
    Ok(())
}

pub fn cmd_mem_learn(lesson: &str) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    let project = read_project_name(&dijiang_dir).unwrap_or_else(|_| "unknown".to_string());
    let (session_key, _) = current_session_key();
    let mem = current_project_memory(&dijiang_dir)?;
    let record = dijiang_mem::Learning {
        timestamp: chrono::Local::now().to_rfc3339(),
        content: lesson.to_string(),
        session_id: Some(session_key),
        project: Some(project),
        tags: vec![],
        scope: dijiang_mem::MemoryScope::Project,
    };
    mem.append_learning(&record)?;
    println!("  Lesson recorded to {}", mem.root().join("learnings.jsonl").display());
    Ok(())
}

pub fn cmd_mem_correction(correction: &str, lesson: &str, scope: &str, source: &str, freshness: &str, conflict: &str, actionability: &str) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
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
        confidence: if source == "user" { "user-confirmed".to_string() } else { "observed".to_string() },
        freshness: freshness.to_string(),
        conflict: conflict.to_string(),
        actionability: actionability.to_string(),
    };
    mem.append_correction(&record)?;
    println!("  Correction recorded to {}", mem.root().join("corrections.jsonl").display());
    Ok(())
}

pub fn cmd_mem_archive() -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    let config_str = std::fs::read_to_string(dijiang_dir.join("config.toml"))?;
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
    for name in &["findings.md", "lessons.md"] {
        let src = workspace.join(name);
        if src.exists() {
            std::fs::rename(&src, &archive_dir.join(name))?;
            println!("  Archived {}", name);
        }
    }
    println!("  Session archived to {}", archive_dir.display());
    Ok(())
}

pub fn cmd_mem_tactic(name: &str, description: &str) -> anyhow::Result<()> {
    let mem = dijiang_mem::GlobalMemory::new()?;
    let tactic = mem.add_tactic(name, description, "cli")?;
    println!("  Added tactic: {} (alpha={}, beta={})", tactic.name, tactic.alpha, tactic.beta);
    Ok(())
}

pub fn cmd_mem_tactics(select: usize) -> anyhow::Result<()> {
    let mem = dijiang_mem::GlobalMemory::new()?;
    let tactics = mem.select_tactics(select)?;
    println!("  Top {} tactics (Thompson sampling):", select);
    for t in &tactics {
        println!("    {} (win_rate={:.2}, a={}, b={})", t.name, t.win_rate(), t.alpha, t.beta);
    }
    Ok(())
}

pub fn cmd_mem_record(tactic_name: &str, outcome: &str, context: &str) -> anyhow::Result<()> {
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

pub fn cmd_mem_pattern(name: &str, description: &str) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    let mem = current_project_memory(&dijiang_dir)?;
    let pattern = dijiang_mem::Pattern {
        name: name.to_string(),
        description: description.to_string(),
        steps: vec![], tags: vec![],
        created_at: chrono::Local::now().to_rfc3339(),
        project: None, cadence: None, risk: None,
        token_cost: None, week_one_mode: None,
        human_gates: vec![], phases: vec![],
    };
    mem.add_pattern(&pattern)?;
    println!("  Added pattern: {}", name);
    Ok(())
}

pub fn cmd_mem_patterns() -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    let mem = current_project_memory(&dijiang_dir)?;
    let patterns = mem.load_patterns()?;
    println!("  {} patterns:", patterns.len());
    for p in &patterns { println!("    {} - {}", p.name, p.description); }
    Ok(())
}

pub fn cmd_mem_stats() -> anyhow::Result<()> {
    let global_mem = dijiang_mem::GlobalMemory::new()?;
    let tactics = global_mem.load_tactics()?;
    let avg_win_rate = if tactics.is_empty() { 0.0 } else { tactics.iter().map(|t| t.win_rate()).sum::<f64>() / tactics.len() as f64 };
    let dijiang_dir = require_dijiang_dir().ok();
    let (findings, learnings, corrections, sessions, patterns) = if let Some(dir) = dijiang_dir.as_ref() {
        let pm = current_project_memory(dir)?;
        (pm.load_findings()?.len(), pm.load_learnings()?.len(), pm.load_corrections()?.len(), pm.load_session_closures()?.len(), pm.load_patterns()?.len())
    } else { (0, 0, 0, 0, 0) };
    println!("  Memory Stats:");
    println!("    Session closures: {}", sessions);
    println!("    Findings: {}", findings);
    println!("    Learnings: {}", learnings);
    println!("    Corrections: {}", corrections);
    println!("    Patterns: {}", patterns);
    println!("    Tactics: {}", tactics.len());
    println!("    Avg win rate: {:.2}", avg_win_rate);
    if let Some(dir) = dijiang_dir.as_ref() {
        if let Ok(pm) = current_project_memory(dir) {
            if let Ok(all_findings) = pm.load_findings() {
                let mut tag_counts = std::collections::BTreeMap::<String, usize>::new();
                for f in &all_findings { for tag in &f.tags { *tag_counts.entry(tag.clone()).or_insert(0) += 1; } }
                if !tag_counts.is_empty() { println!("\n    Tags distribution:"); for (tag, count) in &tag_counts { println!("      {}: {}", tag, count); } }
            }
        }
    }
    Ok(())
}

pub fn cmd_mem_backup() -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    let config_str = std::fs::read_to_string(dijiang_dir.join("config.toml"))?;
    let project = config_str.lines().find(|l| l.starts_with("name"))
        .and_then(|l| l.split('=').nth(1)).map(|s| s.trim().trim_matches('"').to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let global_mem = dijiang_mem::GlobalMemory::new()?;
    let project_mem = current_project_memory(&dijiang_dir)?;
    global_mem.backup_project(&project, &project_mem)?;
    println!("  Backed up project '{}' to ~/.dijiang/backups/", project);
    Ok(())
}

pub fn cmd_mem_evolve() -> anyhow::Result<()> {
    println!("  🔥 Fast-loop evolution: analyzing session...");
    let dijiang_dir = require_dijiang_dir()?;
    let project_mem = current_project_memory(&dijiang_dir)?;
    let findings = project_mem.load_findings()?;
    let learnings = project_mem.load_learnings()?;
    let corrections = project_mem.load_corrections()?;
    let sessions = project_mem.load_session_closures()?;
    let global_mem = dijiang_mem::GlobalMemory::new()?;
    let mut tactics_created = 0;
    let mut finding_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    for finding in &findings { *finding_counts.entry(finding.content.chars().take(50).collect()).or_insert(0) += 1; }
    for (pattern, count) in &finding_counts {
        if *count >= 3 {
            let existing = global_mem.load_tactics()?;
            if !existing.iter().any(|t| t.description.contains(pattern)) {
                global_mem.add_tactic(pattern, &format!("Auto-detected from {} findings", count), &dijiang_dir.to_string_lossy())?;
                tactics_created += 1;
            }
        }
    }
    let config_str = std::fs::read_to_string(dijiang_dir.join("config.toml"))?;
    let project = config_str.lines().find(|l| l.starts_with("name"))
        .and_then(|l| l.split('=').nth(1)).map(|s| s.trim().trim_matches('"').to_string())
        .unwrap_or_else(|| "unknown".to_string());
    global_mem.backup_project(&project, &project_mem)?;
    println!("  Findings analyzed: {}", findings.len());
    println!("  Learnings analyzed: {}", learnings.len());
    println!("  Corrections analyzed: {}", corrections.len());
    println!("  Session closures analyzed: {}", sessions.len());
    println!("  Tactics created: {}", tactics_created);
    Ok(())
}

pub fn cmd_mem_finetune() -> anyhow::Result<()> {
    println!("  🧬 Slow-loop fine-tune: training on accumulated experience...");
    let global_mem = dijiang_mem::GlobalMemory::new()?;
    let tactics = global_mem.load_tactics()?;
    if tactics.is_empty() { println!("  No tactics found. Run `dijiang mem evolve` first."); return Ok(()); }
    let total_tactics = tactics.len();
    let avg_win_rate = tactics.iter().map(|t| t.win_rate()).sum::<f64>() / total_tactics as f64;
    let high_performers: Vec<_> = tactics.iter().filter(|t| t.win_rate() > 0.7).collect();
    let low_performers: Vec<_> = tactics.iter().filter(|t| t.win_rate() < 0.3).collect();
    println!("  Total tactics: {}", total_tactics);
    println!("  Average win rate: {:.2}", avg_win_rate);
    println!("  High performers (>70%): {}", high_performers.len());
    println!("  Low performers (<30%): {}", low_performers.len());
    if low_performers.len() > high_performers.len() { println!("  ⚠️  More low performers than high. Consider pruning."); }
    else { println!("  ✅ Ratchet gate: PASS - system improving."); }
    let stats = dijiang_mem::MemoryStats {
        total_findings: 0, total_learnings: 0, total_corrections: 0,
        total_tactics: total_tactics as u64, total_patterns: 0, total_sessions: 0,
        avg_tactic_win_rate: avg_win_rate,
        last_evolution: Some(chrono::Local::now().to_rfc3339()),
        last_finetune: Some(chrono::Local::now().to_rfc3339()),
    };
    global_mem.save_stats(&stats)?;
    println!("  Fine-tune complete.");
    Ok(())
}

pub fn cmd_mem_recall(query: &str, limit: usize, project: Option<&str>) -> anyhow::Result<()> {
    let mem = if let Some(p) = project {
        let path = std::path::Path::new(p);
        if !path.join(".dijiang").exists() { anyhow::bail!("指定路径没有 .dijiang/ 目录: {p}"); }
        let dijiang_dir = store::find_dijiang_dir(path)
            .ok_or_else(|| anyhow::anyhow!("未找到 .dijiang/ 目录"))?;
        current_project_memory(&dijiang_dir)?
    } else {
        current_project_memory(&require_dijiang_dir()?)?
    };
    let results = mem.recall(query, limit)?;
    if results.is_empty() { println!("  No matching memories found."); }
    else {
        println!("  Found {} result(s):\n", results.len());
        for (i, r) in results.iter().enumerate() {
            let pct = (r.score * 100.0) as u8;
            println!("  [{}.] [{}] ({}%)", i + 1, r.source, pct);
            println!("       {}", r.content);
            println!();
        }
    }
    Ok(())
}

pub fn cmd_mem_prune(days: u64) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    let mem = current_project_memory(&dijiang_dir)?;
    let report = mem.prune(days)?;
    println!("  ✅ Pruned entries older than {days} days.");
    println!("     Findings kept: {}", report.findings_before);
    println!("     Learnings kept: {}", report.learnings_before);
    Ok(())
}

pub fn cmd_mem_index() -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    let mem = current_project_memory(&dijiang_dir)?;
    mem.build_index()?;
    println!("  ✅ Search index rebuilt.");
    Ok(())
}

pub fn cmd_mem_list() -> anyhow::Result<()> {
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

    println!("  Providers: {} ({})", providers.join(" + "), registry.adapter_count());
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
        if p.sessions.len() > 3 { println!("    ... and {} more", p.sessions.len() - 3); }
        println!();
    }

    println!("  Total: {total_sessions} session(s)", total_sessions = total_sessions);
    println!();
    Ok(())
}

pub fn cmd_mem_sync() -> anyhow::Result<()> {
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
    if synced > 0 { println!("  Location: ~/.dijiang/mem/sessions/"); }
    println!();
    Ok(())
}
