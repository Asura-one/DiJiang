//! DiJiang MCP handler — bridges MCP protocol to DiJiang domain logic.

use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_json::{Value, json};

use crate::protocol::*;

/// Handles MCP requests by dispatching to DiJiang internals.
pub struct DiJiangMcpHandler {
    /// Current working directory (project root).
    cwd: PathBuf,
}

impl DiJiangMcpHandler {
    pub fn new(cwd: PathBuf) -> Self {
        Self { cwd }
    }

    /// Dispatch an incoming MCP message. Returns `None` for notifications.
    pub fn handle_request(&self, msg: &JsonRpcMessage) -> Result<Option<JsonRpcResponse>> {
        if msg.is_notification() {
            match msg.method() {
                "notifications/initialized" => {
                    // Acknowledge silently; no response needed for notifications.
                }
                other => {
                    // Unknown notification — ignore per JSON-RPC spec.
                    eprintln!("[mcp] unknown notification: {}", other);
                }
            }
            return Ok(None);
        }

        let id = msg.id();
        let result = match msg.method() {
            "initialize" => self.handle_initialize(msg),
            "resources/list" => self.handle_resources_list(),
            "resources/read" => self.handle_resources_read(msg),
            "tools/list" => self.handle_tools_list(),
            "tools/call" => self.handle_tools_call(msg),
            other => {
                return Ok(Some(JsonRpcResponse::error(
                    id,
                    -32601,
                    format!("Method not found: {}", other),
                )));
            }
        };

        match result {
            Ok(val) => Ok(Some(JsonRpcResponse::success(id, val))),
            Err(e) => Ok(Some(JsonRpcResponse::error(
                id,
                -32603,
                format!("Internal error: {}", e),
            ))),
        }
    }

    // ─── Initialize ───────────────────────────────────────────

    fn handle_initialize(&self, _msg: &JsonRpcMessage) -> Result<Value> {
        Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "resources": {},
                "tools": {}
            },
            "serverInfo": {
                "name": "dijiang-mcp",
                "version": env!("CARGO_PKG_VERSION")
            }
        }))
    }

    // ─── Resources ────────────────────────────────────────────

    fn handle_resources_list(&self) -> Result<Value> {
        let resources = vec![
            McpResource {
                uri: "dijiang://workflow-state".to_string(),
                name: "Workflow State".to_string(),
                description: "Current DiJiang workflow state, active task, and loop signals"
                    .to_string(),
                mime_type: Some("application/json".to_string()),
            },
            McpResource {
                uri: "dijiang://patterns".to_string(),
                name: "Pattern Registry".to_string(),
                description:
                    "All registered workflow patterns with metadata (cadence, risk, phases)"
                        .to_string(),
                mime_type: Some("application/json".to_string()),
            },
            McpResource {
                uri: "dijiang://tactics".to_string(),
                name: "Tactics".to_string(),
                description: "All Bayesian tactics with win rates and Thompson sampling data"
                    .to_string(),
                mime_type: Some("application/json".to_string()),
            },
            McpResource {
                uri: "dijiang://audit".to_string(),
                name: "Audit Report".to_string(),
                description: "Loop Readiness Score and signal breakdown".to_string(),
                mime_type: Some("application/json".to_string()),
            },
        ];
        Ok(json!({ "resources": resources }))
    }

    fn handle_resources_read(&self, msg: &JsonRpcMessage) -> Result<Value> {
        let uri = msg
            .params()
            .and_then(|p| p.get("uri"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let content = match uri.as_str() {
            "dijiang://workflow-state" => self.read_workflow_state()?,
            "dijiang://patterns" => self.read_patterns()?,
            "dijiang://tactics" => self.read_tactics()?,
            "dijiang://audit" => self.read_audit()?,
            other => {
                return Ok(json!({
                    "contents": [{
                        "uri": uri,
                        "mimeType": "text/plain",
                        "text": format!("Resource not found: {}", other)
                    }]
                }));
            }
        };

        Ok(json!({
            "contents": [content]
        }))
    }

    fn read_workflow_state(&self) -> Result<Value> {
        let dijiang_dir = self.find_dijiang_dir();
        let state_json = match dijiang_dir {
            Some(dir) => match dijiang_task::workflow_state::build_for_session(&dir, None) {
                Ok(state) => serde_json::to_value(&state)?,
                Err(e) => json!({ "error": format!("Failed to build state: {}", e) }),
            },
            None => json!({ "error": "No .dijiang/ found" }),
        };
        Ok(json!({
            "uri": "dijiang://workflow-state",
            "mimeType": "application/json",
            "text": serde_json::to_string_pretty(&state_json)?
        }))
    }

    fn read_patterns(&self) -> Result<Value> {
        let patterns = match self.find_dijiang_dir() {
            Some(dir) => match dijiang_mem::ProjectMemory::from_dijiang_dir(&dir) {
                Ok(mem) => mem.load_patterns().unwrap_or_default(),
                Err(_) => vec![],
            },
            None => vec![],
        };
        Ok(json!({
            "uri": "dijiang://patterns",
            "mimeType": "application/json",
            "text": serde_json::to_string_pretty(&patterns)?
        }))
    }

    fn read_tactics(&self) -> Result<Value> {
        let tactics = match dijiang_mem::GlobalMemory::new() {
            Ok(mem) => mem.load_tactics().unwrap_or_default(),
            Err(_) => vec![],
        };
        Ok(json!({
            "uri": "dijiang://tactics",
            "mimeType": "application/json",
            "text": serde_json::to_string_pretty(&tactics)?
        }))
    }

    fn read_audit(&self) -> Result<Value> {
        // Reuse the scoring logic from cmd_audit — inline lightweight audit.
        let mut signals: Vec<Value> = Vec::new();

        let dijiang_dir = self.find_dijiang_dir();
        let project_root = dijiang_dir
            .as_ref()
            .and_then(|d| d.parent())
            .map(|p| p.to_path_buf());

        // Signals (same weights as dijiang audit)
        let checks: Vec<(&str, u64, Box<dyn Fn() -> bool>)> = vec![
            (
                ".dijiang/",
                10,
                Box::new(|| dijiang_dir.as_ref().map_or(false, |d| d.exists())),
            ),
            (
                "active_task",
                18,
                Box::new(|| {
                    dijiang_dir
                        .as_ref()
                        .and_then(|d| dijiang_task::store::read_active_task(d).ok().flatten())
                        .is_some()
                }),
            ),
            (
                "route_gate",
                14,
                Box::new(|| self.cwd.join("crates/task/src/route_gate.rs").exists()),
            ),
            (
                "git_gate",
                14,
                Box::new(|| self.cwd.join("crates/task/src/git_gate.rs").exists()),
            ),
            (
                "skills_2plus",
                14,
                Box::new(|| {
                    dijiang_dir
                        .as_ref()
                        .map(|d| {
                            let skills_dir = d.join("skills");
                            if skills_dir.exists() {
                                std::fs::read_dir(&skills_dir)
                                    .map(|e| e.count())
                                    .unwrap_or(0)
                                    >= 2
                            } else {
                                false
                            }
                        })
                        .unwrap_or(false)
                }),
            ),
            (
                "verifier_skill",
                14,
                Box::new(|| {
                    dijiang_dir
                        .as_ref()
                        .map(|d| {
                            let sd = d.join("skills");
                            sd.join("dj-check").exists() || sd.join("dj-check.skill").exists()
                        })
                        .unwrap_or(false)
                }),
            ),
            (
                "AGENTS.md",
                9,
                Box::new(|| {
                    project_root
                        .as_ref()
                        .map_or(false, |r| r.join("AGENTS.md").exists())
                }),
            ),
            (
                "workflow.md",
                9,
                Box::new(|| {
                    dijiang_dir
                        .as_ref()
                        .map_or(false, |d| d.join("workflow.md").exists())
                }),
            ),
            (
                "tactics",
                6,
                Box::new(|| {
                    dijiang_mem::GlobalMemory::new()
                        .ok()
                        .and_then(|m| m.load_tactics().ok())
                        .map_or(false, |t| !t.is_empty())
                }),
            ),
            (
                "patterns",
                6,
                Box::new(|| {
                    dijiang_dir
                        .as_ref()
                        .and_then(|d| dijiang_mem::ProjectMemory::from_dijiang_dir(d).ok())
                        .and_then(|m| m.load_patterns().ok())
                        .map_or(false, |p| !p.is_empty())
                }),
            ),
            (
                "circuit_breaker",
                6,
                Box::new(|| self.cwd.join("crates/task/src/circuit_breaker.rs").exists()),
            ),
            (
                "run_log",
                3,
                Box::new(|| {
                    dijiang_dir
                        .as_ref()
                        .map_or(false, |d| d.join(".runtime/loop-run-log.json").exists())
                }),
            ),
            (
                "budget",
                3,
                Box::new(|| {
                    dijiang_dir.as_ref().map_or(false, |d| {
                        d.join("budget.md").exists() || d.join("budget.json").exists()
                    })
                }),
            ),
        ];

        let total_possible: u64 = checks.iter().map(|(_, w, _)| w).sum();
        let mut total_earned: u64 = 0;

        for (name, weight, check) in checks {
            let earned = if check() { weight } else { 0 };
            total_earned += earned;
            signals.push(json!({
                "name": name,
                "weight": weight,
                "earned": earned
            }));
        }

        let score = total_earned as f64 / total_possible as f64 * 100.0;
        let level = if score >= 78.0 {
            "L3"
        } else if score >= 58.0 {
            "L2"
        } else if score >= 38.0 {
            "L1"
        } else {
            "L0"
        };

        let report = json!({
            "score": format!("{:.0}", score),
            "level": level,
            "totalEarned": total_earned,
            "totalPossible": total_possible,
            "signals": signals
        });

        Ok(json!({
            "uri": "dijiang://audit",
            "mimeType": "application/json",
            "text": serde_json::to_string_pretty(&report)?
        }))
    }

    // ─── Tools ────────────────────────────────────────────────

    fn handle_tools_list(&self) -> Result<Value> {
        let tools = vec![
            McpTool {
                name: "list_patterns".to_string(),
                description: "List all registered patterns".to_string(),
                input_schema: None,
            },
            McpTool {
                name: "get_pattern".to_string(),
                description: "Get a single pattern by name".to_string(),
                input_schema: Some(json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Pattern name"
                        }
                    },
                    "required": ["name"]
                })),
            },
            McpTool {
                name: "recommend_pattern".to_string(),
                description: "Recommend patterns matching a use case".to_string(),
                input_schema: Some(json!({
                    "type": "object",
                    "properties": {
                        "use_case": {
                            "type": "string",
                            "description": "Use case description (e.g. 'watch CI', 'monitor PR')"
                        }
                    },
                    "required": ["use_case"]
                })),
            },
            McpTool {
                name: "estimate_cost".to_string(),
                description: "Estimate daily/monthly token cost for a pattern".to_string(),
                input_schema: Some(json!({
                    "type": "object",
                    "properties": {
                        "pattern": {
                            "type": "string",
                            "description": "Pattern name (optional, omit for all)"
                        },
                        "level": {
                            "type": "string",
                            "description": "L1, L2, or L3",
                            "default": "L1"
                        }
                    }
                })),
            },
            McpTool {
                name: "run_audit".to_string(),
                description: "Run a Loop Readiness Audit".to_string(),
                input_schema: None,
            },
        ];
        Ok(json!({ "tools": tools }))
    }

    fn handle_tools_call(&self, msg: &JsonRpcMessage) -> Result<Value> {
        let params = msg.params().unwrap_or(&Value::Null);
        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let arguments = params.get("arguments").unwrap_or(&Value::Null);

        match name {
            "list_patterns" => self.tool_list_patterns(),
            "get_pattern" => self.tool_get_pattern(arguments),
            "recommend_pattern" => self.tool_recommend_pattern(arguments),
            "estimate_cost" => self.tool_estimate_cost(arguments),
            "run_audit" => self.read_audit().map(|v| {
                json!({
                    "content": [{
                        "type": "resource",
                        "resource": v
                    }]
                })
            }),
            other => Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Unknown tool: {}", other)
                }]
            })),
        }
    }

    fn tool_list_patterns(&self) -> Result<Value> {
        let dijiang_dir = match self.find_dijiang_dir() {
            Some(d) => d,
            None => {
                return Ok(json!({
                    "content": [{"type": "text", "text": "No .dijiang/ directory found"}]
                }));
            }
        };
        let mem = dijiang_mem::ProjectMemory::from_dijiang_dir(&dijiang_dir)?;
        let patterns = mem.load_patterns()?;
        let text = if patterns.is_empty() {
            "No patterns registered.".to_string()
        } else {
            patterns
                .iter()
                .map(|p| {
                    let meta = {
                        let mut parts = Vec::new();
                        if let Some(c) = &p.cadence {
                            parts.push(format!("cad={}", c));
                        }
                        if let Some(r) = &p.risk {
                            parts.push(format!("risk={}", r));
                        }
                        if parts.is_empty() {
                            String::new()
                        } else {
                            format!(" [{}]", parts.join(", "))
                        }
                    };
                    format!("  - {}{}: {}", p.name, meta, p.description)
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        Ok(json!({
            "content": [{"type": "text", "text": text}]
        }))
    }

    fn tool_get_pattern(&self, args: &Value) -> Result<Value> {
        let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
        if name.is_empty() {
            return Ok(json!({
                "content": [{"type": "text", "text": "Missing required argument: name"}]
            }));
        }
        let dijiang_dir = match self.find_dijiang_dir() {
            Some(d) => d,
            None => {
                return Ok(json!({
                    "content": [{"type": "text", "text": "No .dijiang/ directory found"}]
                }));
            }
        };
        let mem = dijiang_mem::ProjectMemory::from_dijiang_dir(&dijiang_dir)?;
        let patterns = mem.load_patterns()?;
        let found = patterns.into_iter().find(|p| p.name == name);
        match found {
            Some(p) => Ok(json!({
                "content": [{
                    "type": "resource",
                    "resource": {
                        "uri": format!("dijiang://patterns/{}", p.name),
                        "mimeType": "application/json",
                        "text": serde_json::to_string_pretty(&p)?
                    }
                }]
            })),
            None => Ok(json!({
                "content": [{"type": "text", "text": format!("Pattern not found: {}", name)}]
            })),
        }
    }

    fn tool_recommend_pattern(&self, args: &Value) -> Result<Value> {
        let use_case = args.get("use_case").and_then(|v| v.as_str()).unwrap_or("");
        let dijiang_dir = match self.find_dijiang_dir() {
            Some(d) => d,
            None => {
                return Ok(json!({
                    "content": [{"type": "text", "text": "No .dijiang/ directory found"}]
                }));
            }
        };
        let mem = dijiang_mem::ProjectMemory::from_dijiang_dir(&dijiang_dir)?;
        let patterns = mem.load_patterns()?;

        if patterns.is_empty() {
            return Ok(json!({
                "content": [{"type": "text", "text": "No patterns registered."}]
            }));
        }

        let query = use_case.to_lowercase();
        let keyword_map = [
            ("watch CI", &["ci", "sweeper", "build", "test"] as &[&str]),
            (
                "monitor PR",
                &["pr", "babysitter", "pull_request", "review"],
            ),
            ("daily triage", &["triage", "daily", "issue", "prioritize"]),
            (
                "dependency check",
                &["dependency", "deps", "update", "outdated"],
            ),
            ("changelog", &["changelog", "release", "log", "draft"]),
            ("post-merge", &["merge", "cleanup", "post_merge"]),
        ];

        let mut scored: Vec<(f64, &dijiang_mem::Pattern)> = patterns
            .iter()
            .map(|p| {
                let mut score = 0.0;
                if !query.is_empty() {
                    if p.name.to_lowercase().contains(&query) {
                        score += 10.0;
                    }
                    for kw in &p.tags {
                        if kw.to_lowercase().contains(&query) {
                            score += 5.0;
                        }
                    }
                    for (_, keywords) in &keyword_map {
                        for kw in *keywords {
                            if query.contains(kw) || kw.contains(&query) {
                                score += 3.0;
                            }
                        }
                    }
                }
                if p.cadence.is_some() {
                    score += 1.0;
                }
                if p.risk.is_some() {
                    score += 1.0;
                }
                if !p.phases.is_empty() {
                    score += 1.0;
                }
                (score, p)
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let top: Vec<_> = scored.iter().take(5).collect();

        let text = if top.is_empty() || top[0].0 <= 0.0 {
            "No matching patterns found.".to_string()
        } else {
            let mut lines = vec![format!("Top patterns for '{}':", use_case)];
            for (score, p) in &top {
                lines.push(format!("  {} (match: {:.0}%)", p.name, score.min(100.0)));
                lines.push(format!("    {}", p.description));
                if let Some(c) = &p.cadence {
                    lines.push(format!("    cadence: {}", c));
                }
                if let Some(r) = &p.risk {
                    lines.push(format!("    risk: {}", r));
                }
            }
            lines.join("\n")
        };

        Ok(json!({
            "content": [{"type": "text", "text": text}]
        }))
    }

    fn tool_estimate_cost(&self, args: &Value) -> Result<Value> {
        let pattern_name = args.get("pattern").and_then(|v| v.as_str());
        let level = args.get("level").and_then(|v| v.as_str()).unwrap_or("L1");

        let level_mult = match level {
            "L3" => 5.0,
            "L2" => 3.0,
            _ => 1.0,
        };

        let token_per_run = |cost: &Option<String>| -> f64 {
            match cost.as_deref() {
                Some("high") => 80_000.0,
                Some("medium") => 20_000.0,
                _ => 5_000.0,
            }
        };

        let runs_per_day = |cadence: &Option<String>| -> f64 {
            match cadence.as_deref() {
                Some(c) if c == "15m" || c == "5m" => 96.0,
                Some(c) if c == "30m" => 48.0,
                Some(c) if c == "1h" => 24.0,
                Some(c) if c == "2h" => 12.0,
                Some(c) if c == "6h" => 4.0,
                Some(c) if c == "12h" => 2.0,
                Some(c) if c == "1d" => 1.0,
                _ => 1.0,
            }
        };

        let dijiang_dir = match self.find_dijiang_dir() {
            Some(d) => d,
            None => {
                return Ok(json!({
                    "content": [{"type": "text", "text": "No .dijiang/ directory found"}]
                }));
            }
        };
        let mem = dijiang_mem::ProjectMemory::from_dijiang_dir(&dijiang_dir)?;
        let patterns = mem.load_patterns()?;

        let filtered: Vec<_> = match pattern_name {
            Some(name) => patterns.iter().filter(|p| p.name == name).collect(),
            None => patterns.iter().collect(),
        };

        if filtered.is_empty() {
            return Ok(json!({
                "content": [{"type": "text", "text": "No patterns found."}]
            }));
        }

        let mut lines = vec![format!("Cost estimate (level: {}):", level)];
        for p in &filtered {
            let tpr = token_per_run(&p.token_cost) * level_mult;
            let rpd = runs_per_day(&p.cadence);
            let daily = tpr * rpd;
            let monthly = daily * 30.0;
            lines.push(format!(
                "{}: {} tokens/run * {:.1} runs/day = {:.0}/day, {:.0}/month",
                p.name, tpr, rpd, daily, monthly
            ));
        }

        Ok(json!({
            "content": [{"type": "text", "text": lines.join("\n")}]
        }))
    }

    // ─── Helpers ──────────────────────────────────────────────

    fn find_dijiang_dir(&self) -> Option<PathBuf> {
        dijiang_task::store::find_dijiang_dir(&self.cwd)
    }
}
