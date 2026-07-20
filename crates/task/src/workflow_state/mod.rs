use std::fs;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::circuit_breaker::{
    BreakerDecision, CircuitBreakerConfig, Ledger, PruneConfig, check_circuit_breaker,
    error_signature, prune_ledger,
};
use crate::git_gate::summarize_git_gate;
use crate::route_gate::summarize_route_gate;
use crate::skill_manifest::manifests_for_capsule;
use crate::store::{self, SessionIdentity, TaskError};
use crate::types::{TaskRecord, TaskStatus};
use dijiang_mem::{GlobalMemory, ProjectMemory};

mod types;
pub use types::*;

mod tag_parser;


pub fn build(dijiang_dir: &Path) -> Result<WorkflowState, TaskError> {
    build_for_session(dijiang_dir, store::current_session_identity().as_ref())
}

pub fn build_for_session(
    dijiang_dir: &Path,
    identity: Option<&SessionIdentity>,
) -> Result<WorkflowState, TaskError> {
    let session = identity.map(|identity| WorkflowSession {
        key: identity.key().to_string(),
        source: identity.source().to_string(),
    });
    let active_task_state = match store::read_active_task_for_session(dijiang_dir, identity)? {
        Some(active_task_name) => {
            match store::load_task(&dijiang_dir.join("tasks"), &active_task_name) {
                Ok(task) => ActiveTaskState::Present(task),
                Err(TaskError::NotFound(_)) => ActiveTaskState::Missing(active_task_name),
                Err(error) => return Err(error),
            }
        }
        None => ActiveTaskState::None,
    };
    let task = active_task_state.task();
    let runtime = record_runtime_injection(dijiang_dir, identity, task)?;
    let memory = load_recent_memory(dijiang_dir, &runtime.journal_path, 5);
    let peers = load_peer_sessions(dijiang_dir, identity, 8)?;
    let learned_memory = load_learned_memory(dijiang_dir);
    let circuit_breaker_status = load_circuit_breaker_status(dijiang_dir);
    let workflow_tags = load_workflow_tags(dijiang_dir);
    let Some(task) = task else {
        let guidance = match active_task_state.missing_name() {
            Some(missing) => format!(
                "active task `{missing}` 指向缺失的 `.dijiang/tasks/{missing}/task.json`，task state 已陈旧。按 dj-hunt 排查；用 `dijiang task current` / `dijiang task list` 对比状态，确认后运行 `dijiang start <name>` 恢复有效任务，或清理 stale active task。"
            ),
            None => "run `dijiang start <name>` or read `.dijiang/workflow.md` before coding."
                .to_string(),
        };
        return Ok(WorkflowState {
            session,
            active_task: None,
            guidance,
            runtime,
            loop_state: None,
            memory,
            peers,
            route_gate: None,
            git_gate: None,
            skill_manifests: vec![],
            workflow_tags,
            learned_memory,
            circuit_breaker_status,
        });
    };

    let project_root = dijiang_dir.parent().unwrap_or(dijiang_dir);
    let guidance = status_guidance(&task.status).to_string();
    let task_context = workflow_task_context(&task);
    let active_task = Some(workflow_task(dijiang_dir, task_context.task));
    let route_gate = Some(workflow_route_gate(
        task_context.recommended_path,
        &task_context,
    ));
    let git_gate = Some(workflow_git_gate(project_root, task_context.task));
    let skill_manifests = workflow_skill_manifests(&task_context);
    let loop_state = Some(workflow_loop_state(&task_context));
    Ok(WorkflowState {
        session,
        active_task,
        guidance,
        runtime,
        loop_state,
        memory,
        peers,
        route_gate,
        git_gate,
        skill_manifests,
        workflow_tags,
        learned_memory,
        circuit_breaker_status,
    })
}

fn record_runtime_injection(
    dijiang_dir: &Path,
    identity: Option<&SessionIdentity>,
    active_task: Option<&TaskRecord>,
) -> Result<WorkflowRuntime, TaskError> {
    let runtime_dir = dijiang_dir.join(".runtime");
    let sessions_dir = runtime_dir.join("sessions");
    fs::create_dir_all(&sessions_dir)?;
    fs::write(runtime_dir.join(".dijiang_owned"), "")?;

    let fallback_identity;
    let identity = match identity {
        Some(identity) => identity,
        None => {
            fallback_identity = SessionIdentity::new("global", "global")
                .expect("literal global session key is valid");
            &fallback_identity
        }
    };

    let session_path = sessions_dir.join(format!("{}.json", identity.key()));
    let mut record = if session_path.exists() {
        let content = fs::read_to_string(&session_path)?;
        serde_json::from_str::<RuntimeSessionRecord>(&content).unwrap_or_default()
    } else {
        RuntimeSessionRecord::default()
    };
    let previous_active_task = record.last_active_task.clone();
    let current_active_task = active_task.map(|task| task.name.clone());
    let active_task_changed = previous_active_task != current_active_task;
    let last_seen_at = chrono::Utc::now().to_rfc3339();

    record.session_key = identity.key().to_string();
    record.source = identity.source().to_string();
    record.current_task = current_active_task.clone();
    record.last_active_task = current_active_task.clone();
    record.injection_count = record.injection_count.saturating_add(1);
    record.last_seen_at = last_seen_at.clone();
    record.closed_task = None;

    fs::write(&session_path, serde_json::to_string_pretty(&record)?)?;

    let log_path = runtime_dir.join("workflow-state.log");
    let active_task_event = active_task.map(|task| {
        serde_json::json!({
            "id": task.id,
            "name": task.name,
            "title": task.title,
            "status": task.status.as_str(),
            "task_path": relative_path(dijiang_dir, &dijiang_dir.join("tasks").join(&task.name)),
        })
    });
    let route_gate_event = active_task.map(|task| {
        let task_context = workflow_task_context(task);
        let gate = workflow_route_gate(task_context.recommended_path, &task_context);
        serde_json::json!({
            "capsule": gate.capsule,
            "default_skill": gate.default_skill,
            "allowed_skills": gate.allowed_skills,
            "blocked_skills": gate.blocked_skills,
        })
    });
    let git_gate_event = active_task.map(|task| {
        let gate = workflow_git_gate(dijiang_dir.parent().unwrap_or(dijiang_dir), task);
        serde_json::json!({
            "state": gate.state,
            "branch": gate.branch,
            "base_branch": gate.base_branch,
            "worktree_path": gate.worktree_path,
            "note": gate.note,
        })
    });
    let skill_manifests_event = active_task.map(|task| {
        let task_context = workflow_task_context(task);
        workflow_skill_manifests(&task_context)
            .into_iter()
            .map(|manifest| {
                serde_json::json!({
                    "name": manifest.name,
                    "risk": manifest.risk,
                    "phases": manifest.phases,
                })
            })
            .collect::<Vec<_>>()
    });
    let event = serde_json::json!({
        "event": "workflow_state_injected",
        "session_key": identity.key(),
        "source": identity.source(),
        "injection_count": record.injection_count,
        "active_task": current_active_task,
        "active_task_detail": active_task_event,
        "route_gate": route_gate_event,
        "git_gate": git_gate_event,
        "skill_manifests": skill_manifests_event,
        "loop_state": active_task.map(|task| {
            let task_context = workflow_task_context(task);
            serde_json::to_value(workflow_loop_state(&task_context))
                .expect("workflow loop state serializes")
        }),
        "previous_active_task": previous_active_task,
        "active_task_changed": active_task_changed,
        "at": last_seen_at,
    });
    let mut log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    writeln!(log, "{}", serde_json::to_string(&event)?)?;
    let journal_path = append_session_journal(dijiang_dir, identity, &event)?;

    Ok(WorkflowRuntime {
        injection_count: record.injection_count,
        active_task_changed,
        previous_active_task,
        last_seen_at,
        log_path: relative_path(dijiang_dir, &log_path),
        journal_path: relative_path(dijiang_dir, &journal_path),
    })
}

fn append_session_journal(
    dijiang_dir: &Path,
    identity: &SessionIdentity,
    event: &serde_json::Value,
) -> Result<std::path::PathBuf, TaskError> {
    let developer = read_developer(dijiang_dir);
    let sessions_dir = dijiang_dir
        .join("workspace")
        .join(developer)
        .join("sessions");
    fs::create_dir_all(&sessions_dir)?;
    let journal_path = sessions_dir.join(format!("{}.jsonl", identity.key()));
    let mut journal = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&journal_path)?;
    writeln!(journal, "{}", serde_json::to_string(event)?)?;
    Ok(journal_path)
}

fn load_recent_memory(dijiang_dir: &Path, journal_path: &str, limit: usize) -> WorkflowMemory {
    let path = dijiang_dir
        .parent()
        .unwrap_or(dijiang_dir)
        .join(journal_path);
    let Ok(content) = fs::read_to_string(path) else {
        return WorkflowMemory {
            summary: "当前窗口没有上一轮会话记忆。".to_string(),
            events: Vec::new(),
        };
    };

    let mut events = content
        .lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter_map(memory_event_from_json)
        .take(limit)
        .collect::<Vec<_>>();
    events.reverse();

    let summary = if events.is_empty() {
        "当前窗口没有上一轮会话记忆。".to_string()
    } else {
        format!("当前窗口已加载 {} 条最近会话事件。", events.len())
    };

    WorkflowMemory { summary, events }
}

fn memory_event_from_json(value: serde_json::Value) -> Option<WorkflowMemoryEvent> {
    let kind = value.get("event")?.as_str()?.to_string();
    let at = value
        .get("at")
        .or_else(|| value.get("closed_at"))
        .and_then(|value| value.as_str())
        .map(str::to_string);
    let detail = match kind.as_str() {
        "workflow_state_injected" => {
            let count = value
                .get("injection_count")
                .and_then(|value| value.as_u64())
                .unwrap_or_default();
            let active = value
                .get("active_task")
                .and_then(|value| value.as_str())
                .unwrap_or("none");
            let previous = value
                .get("previous_active_task")
                .and_then(|value| value.as_str())
                .unwrap_or("none");
            let changed = value
                .get("active_task_changed")
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            let progress = value
                .get("loop_state")
                .and_then(|value| value.get("progress"))
                .and_then(|value| value.get("signal"))
                .and_then(|value| value.as_str())
                .unwrap_or("no loop signal");
            let next_action = value
                .get("loop_state")
                .and_then(|value| value.get("nextAction"))
                .or_else(|| {
                    value
                        .get("loop_state")
                        .and_then(|value| value.get("next_action"))
                })
                .and_then(|value| value.as_str())
                .unwrap_or("no next action");
            format!(
                "注入 #{count}：活跃任务={active}，上一个任务={previous}，变化={changed}，进展={progress}，下一步={next_action}",
            )
        }
        "session_closed" => {
            let task = value
                .get("task")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown");
            let verification = value
                .get("verification")
                .and_then(|value| value.as_str())
                .unwrap_or("not recorded");
            format!("会话已关闭：任务={task}，验证={verification}")
        }
        _ => return None,
    };

    Some(WorkflowMemoryEvent { kind, detail, at })
}

fn format_memory(memory: &WorkflowMemory) -> String {
    if memory.events.is_empty() {
        return format!("最近记忆：{}", memory.summary);
    }

    // Fold consecutive events with identical detail text into a single
    // line with a repeat count, keeping the context bounded.
    let mut folded_events: Vec<String> = Vec::new();
    let mut repeat_count: usize = 0;
    let mut last_detail: Option<String> = None;

    for event in &memory.events {
        let detail_line = match &event.at {
            Some(at) => format!("- [{}] {}", at, event.detail),
            None => format!("- {}", event.detail),
        };
        // Normalize detail for dedup comparison (strip timestamps from
        // injection counts to avoid false uniqueness)
        let normalized = event.detail.replace("注入 #", "注入 #");

        if let Some(last) = &last_detail {
            if normalized == *last {
                repeat_count += 1;
                continue;
            } else {
                if repeat_count > 0 {
                    if let Some(entry) = folded_events.last_mut() {
                        *entry = format!("{} (×{})", entry, repeat_count + 1);
                    }
                }
                folded_events.push(detail_line.clone());
                last_detail = Some(normalized);
                repeat_count = 0;
            }
        } else {
            folded_events.push(detail_line.clone());
            last_detail = Some(normalized);
            repeat_count = 0;
        }
    }
    // Flush last entry
    if repeat_count > 0 {
        if let Some(entry) = folded_events.last_mut() {
            *entry = format!("{} (×{})", entry, repeat_count + 1);
        }
    }

    format!("最近记忆：{}\n{}", memory.summary, folded_events.join("\n"))
}

/// Best-effort read-back of learned tactics (global) and patterns (project).
///
/// Reads from the global tactic store (`~/.dijiang/memory/tactics.json`) and
/// the project pattern store (`.dijiang/memory/patterns.jsonl`). Any read
/// failure degrades to an empty list with an explanatory summary rather than
/// failing the whole workflow-state build — the agent still gets a usable
/// context even when memory is unavailable.
fn load_learned_memory(dijiang_dir: &Path) -> WorkflowLearnedMemory {
    let global_mem = GlobalMemory::new();
    let project_mem = ProjectMemory::from_dijiang_dir(dijiang_dir);
    load_learned_memory_from(project_mem.as_ref().ok(), global_mem.as_ref().ok())
}

/// Best-effort load of circuit breaker status from the attempt ledger.
///
/// Reads `.dijiang/.runtime/ledger.json` if it exists, checks the breaker
/// with default config, and returns the decision. If the ledger file doesn't
/// exist or can't be parsed, returns `None` — the loop has no recorded
/// attempts to evaluate, so the breaker stays in "none" state.
fn load_circuit_breaker_status(dijiang_dir: &Path) -> Option<BreakerDecision> {
    let ledger_path = dijiang_dir.join(".runtime").join("ledger.json");
    if !ledger_path.exists() {
        return None;
    }
    let content = fs::read_to_string(&ledger_path).ok()?;
    let ledger: Ledger = serde_json::from_str(&content).ok()?;
    if ledger.attempts.is_empty() {
        return None;
    }
    let config = CircuitBreakerConfig::default();
    Some(check_circuit_breaker(&ledger, &config))
}

/// Load workflow-state tag blocks from `.dijiang/workflow.md`.
///
/// Reads `[workflow-state:TAG]...[/workflow-state:TAG]` blocks from the
/// workflow markdown file. Returns an empty map (graceful degradation) when
/// the file is missing or unparseable — the caller falls back to hardcoded
/// guidance.
fn load_workflow_tags(dijiang_dir: &Path) -> WorkflowTagMap {
    let workflow_path = dijiang_dir.join("workflow.md");
    tag_parser::parse_workflow_tags(&workflow_path)
}

/// Testable core: read back from explicit memory handles so unit tests can
/// inject isolated stores without touching the real HOME.
fn load_learned_memory_from(
    project_mem: Option<&ProjectMemory>,
    global_mem: Option<&GlobalMemory>,
) -> WorkflowLearnedMemory {
    const TACTIC_LIMIT: usize = 5;
    const PATTERN_LIMIT: usize = 5;

    let tactics = global_mem
        .and_then(|global| global.load_tactics().ok())
        .map(|mut loaded| {
            loaded.sort_by(|left, right| {
                right
                    .win_rate()
                    .partial_cmp(&left.win_rate())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            loaded.truncate(TACTIC_LIMIT);
            loaded
                .into_iter()
                .map(|tactic| {
                    let win_rate = tactic.win_rate();
                    WorkflowLearnedTactic {
                        name: tactic.name,
                        description: tactic.description,
                        win_rate,
                        source: tactic.source,
                        last_used: tactic.last_used,
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let patterns = project_mem
        .and_then(|project| project.recent_patterns(PATTERN_LIMIT).ok())
        .map(|loaded| {
            loaded
                .into_iter()
                .map(|pattern| WorkflowLearnedPattern {
                    name: pattern.name,
                    description: pattern.description,
                    steps: pattern.steps,
                    tags: pattern.tags,
                    cadence: pattern.cadence,
                    risk: pattern.risk,
                    week_one_mode: pattern.week_one_mode,
                    token_cost: pattern.token_cost,
                    human_gates: pattern.human_gates,
                    phases: pattern.phases,
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let summary = if tactics.is_empty() && patterns.is_empty() {
        "暂无可读回的历史 tactic / pattern。".to_string()
    } else {
        format!(
            "已读回 {} 条 tactic / {} 条 pattern；当前 loop 应参考这些历史沉淀的策略。",
            tactics.len(),
            patterns.len()
        )
    };

    WorkflowLearnedMemory {
        summary,
        tactics,
        patterns,
    }
}

fn format_learned_memory(learned: &WorkflowLearnedMemory) -> String {
    if learned.tactics.is_empty() && learned.patterns.is_empty() {
        return format!("Learned Memory (read-back)：{}", learned.summary);
    }

    let mut lines = vec![format!("Learned Memory (read-back)：{}", learned.summary)];
    for tactic in &learned.tactics {
        lines.push(format!(
            "- tactic: {} (win={:.2}, source={}) — {}",
            tactic.name, tactic.win_rate, tactic.source, tactic.description
        ));
    }
    for pattern in &learned.patterns {
        let meta = pattern.tags.join(",");
        let extra = {
            let mut parts = Vec::new();
            if let Some(c) = &pattern.cadence {
                parts.push(format!("cadence={}", c));
            }
            if let Some(r) = &pattern.risk {
                parts.push(format!("risk={}", r));
            }
            if let Some(m) = &pattern.week_one_mode {
                parts.push(format!("mode={}", m));
            }
            if let Some(t) = &pattern.token_cost {
                parts.push(format!("cost={}", t));
            }
            if !pattern.phases.is_empty() {
                parts.push(format!("phases={}", pattern.phases.join(",")));
            }
            if parts.is_empty() {
                String::new()
            } else {
                format!(" [{}]", parts.join(", "))
            }
        };
        lines.push(format!(
            "- pattern: {} [{}]{} — {}",
            pattern.name, meta, extra, pattern.description
        ));
    }
    lines.join("\n")
}

fn load_peer_sessions(
    dijiang_dir: &Path,
    identity: Option<&SessionIdentity>,
    limit: usize,
) -> Result<Vec<WorkflowPeerSession>, TaskError> {
    let sessions_dir = dijiang_dir.join(".runtime").join("sessions");
    if !sessions_dir.exists() {
        return Ok(Vec::new());
    }
    let current_key = identity
        .map(|identity| identity.key().to_string())
        .unwrap_or_else(|| {
            SessionIdentity::new("global", "global")
                .expect("literal global session key is valid")
                .key()
                .to_string()
        });

    let mut peers = Vec::new();
    for entry in fs::read_dir(sessions_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let content = fs::read_to_string(path)?;
        let record = serde_json::from_str::<RuntimeSessionRecord>(&content).unwrap_or_default();
        if record.session_key.is_empty() || record.session_key == current_key {
            continue;
        }
        peers.push(WorkflowPeerSession {
            key: record.session_key,
            source: record.source,
            current_task: record.current_task,
            injection_count: record.injection_count,
            last_seen_at: record.last_seen_at,
            closed_task: record.closed_task,
        });
    }

    peers.sort_by(|left, right| right.last_seen_at.cmp(&left.last_seen_at));
    peers.truncate(limit);
    Ok(peers)
}

fn format_peer_sessions(peers: &[WorkflowPeerSession]) -> String {
    if peers.is_empty() {
        return "其他活跃窗口：none".to_string();
    }

    let sessions = peers
        .iter()
        .map(|peer| {
            let task = peer
                .current_task
                .as_deref()
                .or(peer.closed_task.as_deref())
                .unwrap_or("none");
            let state = if peer.current_task.is_some() {
                "active"
            } else if peer.closed_task.is_some() {
                "closed"
            } else {
                "idle"
            };
            format!(
                "- {} ({}) 任务={} 状态={} 注入={} 最近活跃={}",
                peer.key, peer.source, task, state, peer.injection_count, peer.last_seen_at,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("其他活跃窗口：{}\n{}", peers.len(), sessions)
}

fn read_developer(dijiang_dir: &Path) -> String {
    let config_path = dijiang_dir.join("config.toml");
    let Ok(config_str) = fs::read_to_string(config_path) else {
        return "developer".to_string();
    };
    config_str
        .lines()
        .find(|line| line.trim_start().starts_with("developer"))
        .and_then(|line| line.split('=').nth(1))
        .map(|value| value.trim().trim_matches('"').to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "developer".to_string())
}

fn relative_path(dijiang_dir: &Path, path: &Path) -> String {
    path.strip_prefix(dijiang_dir.parent().unwrap_or(dijiang_dir))
        .unwrap_or(path)
        .display()
        .to_string()
}

fn workflow_task(dijiang_dir: &Path, task: &TaskRecord) -> WorkflowTask {
    let task_path = dijiang_dir.join("tasks").join(&task.name);
    let task_path = relative_path(dijiang_dir, &task_path);

    WorkflowTask {
        id: task.id.clone(),
        name: task.name.clone(),
        title: task.title.clone(),
        status: task.status.as_str().to_string(),
        task_path,
    }
}

fn infer_post_restart_status(status: &TaskStatus) -> TaskStatus {
    match status {
        TaskStatus::Archived => TaskStatus::Planning,
        TaskStatus::Paused => TaskStatus::InProgress,
        other => other.clone(),
    }
}

fn workflow_task_context(task: &TaskRecord) -> WorkflowTaskContext<'_> {
    let dispatch = dispatch_meta(task);
    WorkflowTaskContext {
        task,
        effective_status: infer_post_restart_status(&task.status),
        recommended_path: dispatch.recommended_path.unwrap_or(""),
    }
}

fn dispatch_meta(task: &TaskRecord) -> DispatchMeta<'_> {
    let dispatch = task.meta.get("dispatch");
    DispatchMeta {
        task_type: dispatch
            .and_then(|value| value.get("task_type"))
            .and_then(|value| value.as_str()),
        primary_intent: dispatch
            .and_then(|value| value.get("primary_intent"))
            .and_then(|value| value.as_str()),
        skill: dispatch
            .and_then(|value| value.get("skill"))
            .and_then(|value| value.as_str()),
        recommended_path: dispatch
            .and_then(|value| value.get("recommended_path"))
            .and_then(|value| value.as_str()),
        action: dispatch
            .and_then(|value| value.get("action"))
            .and_then(|value| value.as_str()),
        reason: dispatch
            .and_then(|value| value.get("reason"))
            .and_then(|value| value.as_str()),
        next_action: dispatch
            .and_then(|value| value.get("next_action"))
            .and_then(|value| value.as_str()),
        attempt: dispatch
            .and_then(|value| value.get("attempt"))
            .and_then(|value| value.as_u64()),
        max_attempts: dispatch
            .and_then(|value| value.get("max_attempts"))
            .and_then(|value| value.as_u64()),
        last_failure: dispatch
            .and_then(|value| value.get("last_failure"))
            .and_then(|value| value.as_str()),
    }
}

fn workflow_route_gate(
    recommended_path: &str,
    task_context: &WorkflowTaskContext<'_>,
) -> WorkflowRouteGate {
    let summary = summarize_route_gate(&task_context.effective_status, None);
    WorkflowRouteGate {
        capsule: summary.capsule.as_str().to_string(),
        allowed_skills: summary
            .allowed_skills
            .into_iter()
            .map(str::to_string)
            .collect(),
        default_skill: summary.default_skill.to_string(),
        blocked_skills: summary
            .blocked_skills
            .into_iter()
            .map(str::to_string)
            .collect(),
        recommended_path: recommended_path.to_string(),
        note: summary.note,
    }
}

fn format_route_gate(route_gate: &WorkflowRouteGate) -> String {
    format!(
        "Route Gate：capsule={}；default_skill={}；recommended_path={}；allowed={}；blocked={}；note={}",
        route_gate.capsule,
        route_gate.default_skill,
        route_gate.recommended_path,
        route_gate.allowed_skills.join(", "),
        route_gate.blocked_skills.join(", "),
        route_gate.note,
    )
}

fn workflow_git_gate(project_root: &Path, task: &TaskRecord) -> WorkflowGitGate {
    let summary = summarize_git_gate(task, project_root);
    WorkflowGitGate {
        state: summary.state.as_str().to_string(),
        branch: summary.branch,
        base_branch: summary.base_branch,
        worktree_path: summary.worktree_path,
        note: summary.note,
    }
}

fn workflow_skill_manifests(task_context: &WorkflowTaskContext<'_>) -> Vec<WorkflowSkillManifest> {
    let capsule = summarize_route_gate(&task_context.effective_status, None).capsule;
    manifests_for_capsule(capsule)
        .into_iter()
        .map(|entry| WorkflowSkillManifest {
            name: entry.name.to_string(),
            summary: entry.summary.to_string(),
            phases: entry
                .phases
                .iter()
                .map(|phase| (*phase).to_string())
                .collect(),
            risk: entry.risk.to_string(),
        })
        .collect()
}

fn format_target_skill_bodies(
    route_gate: Option<&WorkflowRouteGate>,
    skill_manifests: &[WorkflowSkillManifest],
    primary_skill: &str,
    recommended_path: &str,
) -> String {
    let Some(route_gate) = route_gate else {
        return String::new();
    };
    let skill_info = skill_manifests
        .iter()
        .find(|s| s.name == primary_skill)
        .map(|s| format!("{}（{}）", s.name, s.summary))
        .unwrap_or_else(|| primary_skill.to_string());
    format!(
        "Target Skill：[{}] capsule={}；recommended_path={}",
        skill_info, route_gate.capsule, recommended_path
    )
}

fn format_git_gate(git_gate: &WorkflowGitGate) -> String {
    format!(
        "Git Gate：state={}；branch={}；baseBranch={}；worktreePath={}；note={}",
        git_gate.state,
        git_gate.branch.as_deref().unwrap_or("none"),
        git_gate.base_branch.as_deref().unwrap_or("none"),
        git_gate.worktree_path.as_deref().unwrap_or("none"),
        git_gate.note,
    )
}

fn format_skill_manifests(skill_manifests: &[WorkflowSkillManifest]) -> String {
    if skill_manifests.is_empty() {
        return "Skill Manifests：none".to_string();
    }

    let entries = skill_manifests
        .iter()
        .map(|manifest| {
                format!(
                    "{}({})",
                    manifest.name, manifest.summary
                )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("Skill Manifests：{}", entries)
}

fn workflow_loop_state(task_context: &WorkflowTaskContext<'_>) -> WorkflowLoopState {
    let task = task_context.task;
    let dispatch = dispatch_meta(task);
    let goal = task
        .acceptance_criteria
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            let title = task.title.trim();
            if title.is_empty() {
                format!("推进任务 {} 到可验证完成", task.name)
            } else {
                title.to_string()
            }
        });
    let mode = summarize_route_gate(&task_context.effective_status, None)
        .capsule
        .as_str()
        .to_string();
    let attempt = dispatch.attempt.unwrap_or(1);
    let max_attempts = dispatch.max_attempts;
    let last_failure = dispatch.last_failure.map(str::to_string);
    let progress_status = match task.status {
        TaskStatus::Planning => "aligning",
        TaskStatus::InProgress => "executing",
        TaskStatus::Completed => "verified",
        TaskStatus::Archived | TaskStatus::Paused => match &task_context.effective_status {
            TaskStatus::Planning => "ready_to_restart",
            TaskStatus::InProgress => "ready_to_resume",
            TaskStatus::Completed => "verified",
            TaskStatus::Archived => "closed",
            TaskStatus::Paused => "paused",
        },
    }
    .to_string();
    let progress_signal = match task.status {
        TaskStatus::Planning => "需求与验收标准仍需对齐",
        TaskStatus::InProgress => "实现与验证正在推进",
        TaskStatus::Completed => "实现已完成，等待 finish-work 收口",
        TaskStatus::Archived => "任务已归档；若 restart，将从 planning 重新进入 workflow",
        TaskStatus::Paused => "任务已暂停；若 continue，将回到 in_progress workflow",
    }
    .to_string();
    let stop_conditions = match task.status {
        TaskStatus::Planning => vec![
            "需求、范围和验收标准已经确认".to_string(),
            "实现入口已切换到 dj-implement 或 dj-tdd".to_string(),
        ],
        TaskStatus::InProgress => vec![
            "目标行为已实现".to_string(),
            "GREEN command 通过".to_string(),
            "相关回归验证通过后切入 dj-check".to_string(),
        ],
        TaskStatus::Completed => vec![
            "dj-check 完成并记录验证证据".to_string(),
            "finish-work 已归档任务与会话".to_string(),
        ],
        TaskStatus::Archived => vec![
            "如需继续，先 restart 任务并重新进入 planning 路径".to_string(),
            "确认归档前结论仍然有效，避免在旧上下文上直接实现".to_string(),
        ],
        TaskStatus::Paused => vec![
            "恢复上下文并重新进入实现或检查路径".to_string(),
            "确认暂停原因已解除".to_string(),
        ],
    };
    let next_skill =
        next_skill_from_recommended_path(task_context.recommended_path).or_else(|| {
            Some(
                summarize_route_gate(&task_context.effective_status, None)
                    .default_skill
                    .to_string(),
            )
        });
    let next_action = match task.status {
        TaskStatus::Planning => "继续对齐需求并收敛 acceptance".to_string(),
        TaskStatus::InProgress => {
            if let Some(skill) = next_skill.as_deref() {
                format!("继续当前 loop，并按 {skill} 推进下一轮最小验证闭环")
            } else {
                "继续当前 loop，并推进下一轮最小验证闭环".to_string()
            }
        }
        TaskStatus::Completed => "汇总验证证据并准备 finish-work".to_string(),
        TaskStatus::Archived => "若要继续，先 restart 任务并按 planning 路径重新对齐".to_string(),
        TaskStatus::Paused => "恢复任务上下文后继续 loop".to_string(),
    };
    let agent_focus = agent_focus(task, &dispatch, task_context, next_skill.as_deref());
    let resolved_agent = {
        let capsule = summarize_route_gate(&task_context.effective_status, None)
            .capsule;
        let agent_name = crate::agent_manifest::resolve_agent(
            dispatch.task_type,
            dispatch.primary_intent,
            capsule.as_str(),
        );
        Some(agent_name.to_string())
    };
    let memory_writeback =
        workflow_memory_writeback(task, &dispatch, task_context, next_skill.as_deref());
    let remaining_attempts = max_attempts.map(|max| max.saturating_sub(attempt));
    let can_retry = match task.status {
        TaskStatus::Completed => false,
        _ => remaining_attempts
            .map(|remaining| remaining > 0)
            .unwrap_or(true),
    };

    WorkflowLoopState {
        goal,
        mode,
        progress: WorkflowLoopProgress {
            status: progress_status,
            signal: progress_signal,
        },
        stop_conditions,
        next_action,
        next_skill,
        agent_focus,
        resolved_agent,
        memory_writeback,
        retry: WorkflowLoopRetry {
            attempt,
            max_attempts,
            remaining_attempts,
            can_retry,
            last_failure,
        },
    }
}

fn agent_focus(
    task: &TaskRecord,
    dispatch: &DispatchMeta<'_>,
    task_context: &WorkflowTaskContext<'_>,
    next_skill: Option<&str>,
) -> String {
    let task_type = dispatch.task_type.unwrap_or("未分类任务");
    let intent = dispatch.primary_intent.unwrap_or("推进当前目标");
    let route_action = dispatch.action.unwrap_or("allow");
    let route_reason = dispatch.reason.unwrap_or("无额外路由说明");
    let dispatch_next = dispatch.next_action.unwrap_or("继续当前闭环");
    let recommended = dispatch
        .recommended_path
        .unwrap_or(task_context.recommended_path);
    let route_skill = dispatch.skill.or(next_skill).unwrap_or("none");
    let target = task
        .acceptance_criteria
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(task.title.as_str());
    format!(
        "任务类型={task_type}；意图={intent}；目标={target}；当前推荐 skill={route_skill}；recommended_path={recommended}；route_action={route_action}；route_reason={route_reason}；dispatch_next={dispatch_next}",
    )
}

fn workflow_memory_writeback(
    task: &TaskRecord,
    dispatch: &DispatchMeta<'_>,
    task_context: &WorkflowTaskContext<'_>,
    next_skill: Option<&str>,
) -> WorkflowMemoryWriteback {
    let outcome = match task.status {
        TaskStatus::Planning => "alignment_pending",
        TaskStatus::InProgress => {
            if task_context.effective_status == TaskStatus::Completed {
                "ready_for_finish"
            } else {
                "execution_in_progress"
            }
        }
        TaskStatus::Completed => "verified_waiting_finish",
        TaskStatus::Archived => "archived_restart_required",
        TaskStatus::Paused => "paused_resume_required",
    }
    .to_string();
    let next_tactic = format!(
        "{} | {}",
        dispatch.skill.or(next_skill).unwrap_or("none"),
        dispatch.next_action.unwrap_or("continue current loop")
    );
    let next_pattern = format!(
        "{} | {}",
        dispatch
            .recommended_path
            .unwrap_or(task_context.recommended_path),
        dispatch.reason.unwrap_or("follow route gate")
    );

    WorkflowMemoryWriteback {
        outcome,
        next_tactic,
        next_pattern,
    }
}

fn next_skill_from_recommended_path(recommended_path: &str) -> Option<String> {
    recommended_path
        .split("->")
        .map(str::trim)
        .find(|segment| !segment.is_empty())
        .and_then(|segment| segment.split_whitespace().next())
        .map(str::to_string)
}

fn format_loop_state(loop_state: &WorkflowLoopState) -> String {
    let stop_conditions = if loop_state.stop_conditions.is_empty() {
        "none".to_string()
    } else {
        loop_state.stop_conditions.join(" | ")
    };
    let retry = format!(
        "attempt={}; max={}; remaining={}; can_retry={}; last_failure={}",
        loop_state.retry.attempt,
        loop_state
            .retry
            .max_attempts
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unbounded".to_string()),
        loop_state
            .retry
            .remaining_attempts
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        loop_state.retry.can_retry,
        loop_state.retry.last_failure.as_deref().unwrap_or("none"),
    );
    format!(
        "Loop：goal={}；mode={}；progress={} ({})；next_skill={}；next_action={}；agent_focus={}；resolved_agent={}；memory_writeback=outcome:{}|next_tactic:{}|next_pattern:{}；stop_conditions={}；retry={}",
        loop_state.goal,
        loop_state.mode,
        loop_state.progress.status,
        loop_state.progress.signal,
        loop_state.next_skill.as_deref().unwrap_or("none"),
        loop_state.next_action,
        loop_state.agent_focus,
        loop_state.resolved_agent.as_deref().unwrap_or("none"),
        loop_state.memory_writeback.outcome,
        loop_state.memory_writeback.next_tactic,
        loop_state.memory_writeback.next_pattern,
        stop_conditions,
        retry,
    )
}

fn status_guidance(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Planning => {
            "当前任务处于 planning。按 DiJiang 流程先用 dj-grill 对齐需求，再产出必要设计。"
        }
        TaskStatus::InProgress => {
            "当前任务处于 in_progress。继续实现；完成后运行相关验证，再进入 dj-check。"
        }
        TaskStatus::Completed => {
            "当前任务已 completed。检查是否需要运行 /dijiang-finish-work 归档和记录 journal。"
        }
        TaskStatus::Archived => {
            "当前任务已 archived。若继续工作，请先运行 dijiang start <task> 激活新任务。"
        }
        TaskStatus::Paused => "当前任务已 paused。恢复前先读取任务上下文和最近 workspace journal。",
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::store;
    use crate::types::TaskRecord;

    fn task(name: &str, title: &str) -> TaskRecord {
        TaskRecord {
            id: name.to_string(),
            name: name.to_string(),
            title: title.to_string(),
            description: String::new(),
            status: TaskStatus::InProgress,
            dev_type: None,
            scope: None,
            package: None,
            priority: "medium".to_string(),
            creator: "test".to_string(),
            assignee: "test".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            completed_at: None,
            branch: None,
            base_branch: None,
            worktree_path: None,
            commit: None,
            pr_url: None,
            subtasks: vec![],
            children: vec![],
            parent: None,
            related_files: vec![],
            depends_on: None,
            notes: String::new(),
            meta: serde_json::json!({}),
            started_at: None,
            archived_at: None,
            acceptance_criteria: None,
            key_deliverables: None,
            source: None,
            session_id: None,
            session_summary: None,
            estimated_effort: None,
            actual_effort: None,
            review_status: None,
            review_comments: None,
            tags: None,
            hooks: None,
        }
    }

    #[test]
    fn builds_session_scoped_context() {
        let dir = tempfile::tempdir().unwrap();
        let dijiang_dir = dir.path().join(".dijiang");
        std::fs::create_dir_all(&dijiang_dir).unwrap();
        std::fs::write(
            dijiang_dir.join("config.toml"),
            "[project]\ndeveloper = \"tester\"\n",
        )
        .unwrap();
        let tasks_dir = dijiang_dir.join("tasks");
        std::fs::create_dir_all(&tasks_dir).unwrap();

        store::save_task(&tasks_dir, &task("task-a", "Task A")).unwrap();
        store::save_task(&tasks_dir, &task("task-b", "Task B")).unwrap();
        let window_a = store::SessionIdentity::new("dijiang", "window-a").unwrap();
        let window_b = store::SessionIdentity::new("dijiang", "window-b").unwrap();
        store::write_active_task_for_session(&dijiang_dir, "task-a", Some(&window_a)).unwrap();
        store::write_active_task_for_session(&dijiang_dir, "task-b", Some(&window_b)).unwrap();

        let state_a = build_for_session(&dijiang_dir, Some(&window_a)).unwrap();
        let context_a = state_a.additional_context();
        let state_b = build_for_session(&dijiang_dir, Some(&window_b)).unwrap();
        let context_b = state_b.additional_context();

        assert!(context_a.contains("会话：dijiang_window-a（dijiang）"));
        assert!(context_a.contains("注入：#1"));
        assert!(context_a.contains("活跃任务是否变化：true"));
        assert!(context_a.contains("活跃任务：task-a"));
        assert!(context_a.contains("标题：Task A"));
        assert!(context_a.contains("Route Gate：capsule=implement"));
        assert!(context_a.contains("default_skill=dj-implement"));
        assert!(context_a.contains("Git Gate：state=ready"));
        assert!(context_a.contains("worktreePath=none"));
        assert!(context_a.contains("Skill Manifests："));
        assert!(context_a.contains("dj-implement("));
        assert!(context_a.contains("dj-tdd("));
        assert!(
            context_a.contains("<dijiang-target-skill role=\"primary\" name=\"dj-implement\">")
        );
        assert!(context_a.contains("Loop：goal=Task A"));
        assert!(context_a.contains("progress=executing (实现与验证正在推进)"));
        assert!(context_a.contains("next_skill=dj-implement"));
        assert!(context_a.contains("agent_focus=任务类型=未分类任务"));
        assert!(context_a.contains("memory_writeback=outcome:execution_in_progress|next_tactic:dj-implement | continue current loop|next_pattern: | follow route gate"));
        assert!(context_a.contains(
            "retry=attempt=1; max=unbounded; remaining=unknown; can_retry=true; last_failure=none"
        ));
        assert!(context_b.contains("会话：dijiang_window-b（dijiang）"));
        assert!(context_b.contains("活跃任务：task-b"));
        assert!(context_b.contains("标题：Task B"));

        let next_a = build_for_session(&dijiang_dir, Some(&window_a))
            .unwrap()
            .additional_context();
        assert!(next_a.contains("注入：#2"));
        assert!(next_a.contains("活跃任务是否变化：false"));
        assert!(
            next_a.contains("下一步=继续当前 loop，并按 dj-implement 推进下一轮最小验证闭环")
                || next_a.contains(
                    "next_action=继续当前 loop，并按 dj-implement 推进下一轮最小验证闭环"
                )
        );

        let log = std::fs::read_to_string(dijiang_dir.join(".runtime/workflow-state.log")).unwrap();
        assert!(log.contains("workflow_state_injected"));
        assert!(log.contains("dijiang_window-a"));
        assert!(log.contains("dijiang_window-b"));
        assert!(log.contains("route_gate"));
        assert!(log.contains("skill_manifests"));
        assert!(log.contains("loop_state"));

        let journal_a = std::fs::read_to_string(
            dijiang_dir.join("workspace/tester/sessions/dijiang_window-a.jsonl"),
        )
        .unwrap();
        assert_eq!(journal_a.lines().count(), 2);
        assert!(journal_a.contains("workflow_state_injected"));
        assert!(journal_a.contains("\"title\":\"Task A\""));
        assert!(journal_a.contains("\"injection_count\":2"));
        assert!(journal_a.contains("\"loop_state\""));
    }

    #[test]
    fn builds_recoverable_context_for_stale_active_task_pointer() {
        let dir = tempfile::tempdir().unwrap();
        let dijiang_dir = dir.path().join(".dijiang");
        std::fs::create_dir_all(&dijiang_dir).unwrap();
        std::fs::write(
            dijiang_dir.join("config.toml"),
            "[project]\ndeveloper = \"tester\"\n",
        )
        .unwrap();

        let window = store::SessionIdentity::new("dijiang", "stale-window").unwrap();
        store::write_active_task_for_session(&dijiang_dir, "missing-task", Some(&window)).unwrap();

        let state = build_for_session(&dijiang_dir, Some(&window)).unwrap();
        let context = state.additional_context();

        assert!(state.active_task.is_none());
        assert!(context.contains("会话：dijiang_stale-window（dijiang）"));
        assert!(context.contains("活跃任务：none"));
        assert!(context.contains("missing-task"));
        assert!(context.contains("task state 已陈旧"));
        assert!(context.contains("dj-hunt"));

        let session_runtime = std::fs::read_to_string(
            dijiang_dir.join(".runtime/sessions/dijiang_stale-window.json"),
        )
        .unwrap();
        assert!(session_runtime.contains("\"current_task\": null"));
        assert!(session_runtime.contains("\"last_active_task\": null"));

        let log = std::fs::read_to_string(dijiang_dir.join(".runtime/workflow-state.log")).unwrap();
        assert!(log.contains("workflow_state_injected"));
        assert!(log.contains("\"active_task\":null"));
    }

    #[test]
    fn planning_state_exposes_align_capsule() {
        let dir = tempfile::tempdir().unwrap();
        let dijiang_dir = dir.path().join(".dijiang");
        std::fs::create_dir_all(&dijiang_dir).unwrap();
        std::fs::write(
            dijiang_dir.join("config.toml"),
            "[project]\ndeveloper = \"tester\"\n",
        )
        .unwrap();
        let tasks_dir = dijiang_dir.join("tasks");
        std::fs::create_dir_all(&tasks_dir).unwrap();

        let mut planning = task("align-task", "Align Task");
        planning.status = TaskStatus::Planning;
        store::save_task(&tasks_dir, &planning).unwrap();
        let window = store::SessionIdentity::new("dijiang", "align-window").unwrap();
        store::write_active_task_for_session(&dijiang_dir, "align-task", Some(&window)).unwrap();

        let context = build_for_session(&dijiang_dir, Some(&window))
            .unwrap()
            .additional_context();
        assert!(context.contains("Route Gate：capsule=align"));
        assert!(context.contains("default_skill=dj-grill"));
        assert!(context.contains("Git Gate：state=ready"));
        assert!(context.contains("Skill Manifests：dj-grill"));
        assert!(context.contains("dj-output"));
        assert!(context.contains("<dijiang-target-skill role=\"primary\" name=\"dj-grill\">"));
        assert!(context.contains("Loop：goal=Align Task"));
        assert!(context.contains("progress=aligning (需求与验收标准仍需对齐)"));
        assert!(context.contains("next_skill=dj-grill"));
        assert!(context.contains("agent_focus=任务类型=未分类任务"));
    }

    #[test]
    fn paused_state_exposes_continue_route() {
        let dir = tempfile::tempdir().unwrap();
        let dijiang_dir = dir.path().join(".dijiang");
        std::fs::create_dir_all(&dijiang_dir).unwrap();
        std::fs::write(
            dijiang_dir.join("config.toml"),
            "[project]\ndeveloper = \"tester\"\n",
        )
        .unwrap();
        let tasks_dir = dijiang_dir.join("tasks");
        std::fs::create_dir_all(&tasks_dir).unwrap();

        let mut paused = task("paused-task", "Paused Task");
        paused.status = TaskStatus::Paused;
        store::save_task(&tasks_dir, &paused).unwrap();
        let window = store::SessionIdentity::new("dijiang", "paused-window").unwrap();
        store::write_active_task_for_session(&dijiang_dir, "paused-task", Some(&window)).unwrap();

        let context = build_for_session(&dijiang_dir, Some(&window))
            .unwrap()
            .additional_context();
        assert!(context.contains("Route Gate：capsule=implement"));
        assert!(context.contains("default_skill=dj-implement"));
        assert!(context.contains("Git Gate：state=ready"));
        assert!(context.contains("Skill Manifests："));
        assert!(context.contains("Loop：goal=Paused Task"));
    }

    #[test]
    fn archived_state_exposes_restart_requirement() {
        let dir = tempfile::tempdir().unwrap();
        let dijiang_dir = dir.path().join(".dijiang");
        std::fs::create_dir_all(&dijiang_dir).unwrap();
        std::fs::write(
            dijiang_dir.join("config.toml"),
            "[project]\ndeveloper = \"tester\"\n",
        )
        .unwrap();
        let tasks_dir = dijiang_dir.join("tasks");
        std::fs::create_dir_all(&tasks_dir).unwrap();

        let mut archived = task("archived-task", "Archived Task");
        archived.status = TaskStatus::Archived;
        store::save_task(&tasks_dir, &archived).unwrap();
        let window = store::SessionIdentity::new("dijiang", "archived-window").unwrap();
        store::write_active_task_for_session(&dijiang_dir, "archived-task", Some(&window)).unwrap();

        let context = build_for_session(&dijiang_dir, Some(&window))
            .unwrap()
            .additional_context();
        assert!(context.contains("Route Gate：capsule=align"));
        assert!(context.contains("default_skill=dj-grill"));
        assert!(context.contains("Git Gate：state=ready"));
        assert!(context.contains("Skill Manifests：dj-grill"));
        assert!(context.contains("<dijiang-target-skill role=\"primary\" name=\"dj-grill\">"));
        assert!(context.contains("progress=ready_to_restart"));
        assert!(context.contains("先 restart 任务并按 planning 路径重新对齐"));
    }

    #[test]
    fn read_back_surfaces_learned_tactics_and_patterns() {
        let global_dir = tempfile::tempdir().unwrap();
        let project_mem_dir = tempfile::tempdir().unwrap();

        let global_mem = GlobalMemory::new_at(global_dir.path()).unwrap();
        global_mem
            .add_tactic(
                "read-back-tactic",
                "tactic promoted by mem evolve",
                "mem_evolve_writeback",
            )
            .unwrap();

        let project_mem = ProjectMemory::new_at(project_mem_dir.path()).unwrap();
        project_mem
            .add_pattern(&dijiang_mem::Pattern {
                name: "loop-read-back-pattern".to_string(),
                description: "pattern promoted by mem evolve".to_string(),
                steps: vec!["verify then archive".to_string()],
                tags: vec!["loop-writeback".to_string()],
                created_at: "2026-01-01T00:00:00Z".to_string(),
                project: Some("test".to_string()),
                cadence: None,
                risk: None,
                week_one_mode: None,
                token_cost: None,
                human_gates: vec![],
                phases: vec![],
            })
            .unwrap();

        let learned = load_learned_memory_from(Some(&project_mem), Some(&global_mem));
        assert!(
            learned
                .tactics
                .iter()
                .any(|tactic| tactic.name == "read-back-tactic"),
            "tactics: {:?}",
            learned.tactics
        );
        assert!(
            learned
                .patterns
                .iter()
                .any(|pattern| pattern.name == "loop-read-back-pattern"),
            "patterns: {:?}",
            learned.patterns
        );

        let rendered = format_learned_memory(&learned);
        assert!(rendered.contains("Learned Memory (read-back)：已读回 1 条 tactic / 1 条 pattern"));
        assert!(rendered.contains("- tactic: read-back-tactic"));
        assert!(rendered.contains("- pattern: loop-read-back-pattern [loop-writeback]"));
    }

    #[test]
    fn read_back_is_empty_when_memory_unavailable() {
        let learned = load_learned_memory_from(None, None);
        assert!(learned.tactics.is_empty());
        assert!(learned.patterns.is_empty());
        assert!(learned.summary.contains("暂无可读回"));

        let rendered = format_learned_memory(&learned);
        assert!(rendered.contains("Learned Memory (read-back)：暂无可读回"));
    }

    #[test]
    fn build_for_session_surfaces_project_patterns_in_read_back() {
        let dir = tempfile::tempdir().unwrap();
        let dijiang_dir = dir.path().join(".dijiang");
        std::fs::create_dir_all(&dijiang_dir).unwrap();
        std::fs::write(
            dijiang_dir.join("config.toml"),
            "[project]\ndeveloper = \"tester\"\n",
        )
        .unwrap();
        let tasks_dir = dijiang_dir.join("tasks");
        std::fs::create_dir_all(&tasks_dir).unwrap();

        // Seed the project memory store with a promoted pattern.
        let project_mem = ProjectMemory::from_dijiang_dir(&dijiang_dir).unwrap();
        project_mem
            .add_pattern(&dijiang_mem::Pattern {
                name: "loop-verified-and-archived".to_string(),
                description: "pattern promoted back by session loop".to_string(),
                steps: vec![],
                tags: vec!["loop-writeback".to_string()],
                created_at: "2026-06-01T00:00:00Z".to_string(),
                project: Some("test".to_string()),
                cadence: None,
                risk: None,
                week_one_mode: None,
                token_cost: None,
                human_gates: vec![],
                phases: vec![],
            })
            .unwrap();

        store::save_task(&tasks_dir, &task("read-back-task", "Read Back Task")).unwrap();
        let window = store::SessionIdentity::new("dijiang", "read-back-window").unwrap();
        store::write_active_task_for_session(&dijiang_dir, "read-back-task", Some(&window))
            .unwrap();

        let context = build_for_session(&dijiang_dir, Some(&window))
            .unwrap()
            .additional_context();

        assert!(
            context.contains("Learned Memory (read-back)："),
            "context should surface read-back section: {context}"
        );
        assert!(
            context.contains("loop-verified-and-archived"),
            "context should read back the project pattern: {context}"
        );
    }
}
