use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rand::Rng;

use crate::types::*;

/// Global memory store (tactics, ledger, meta, backups).
pub struct GlobalMemory {
    root: PathBuf,
}

/// Project-level memory store (sessions, findings, learnings, patterns).
pub struct ProjectMemory {
    root: PathBuf,
}

impl GlobalMemory {
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().context("cannot determine home directory")?;
        let root = home.join(".dijiang").join("memory");
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    pub fn new_at(root: &Path) -> Result<Self> {
        fs::create_dir_all(root)?;
        Ok(Self {
            root: root.to_path_buf(),
        })
    }

    pub fn tactics_path(&self) -> PathBuf {
        self.root.join("tactics.json")
    }
    pub fn ledger_path(&self) -> PathBuf {
        self.root.join("ledger.jsonl")
    }
    pub fn stats_path(&self) -> PathBuf {
        self.root.join("meta").join("stats.json")
    }
    pub fn baseline_path(&self) -> PathBuf {
        self.root.join("meta").join("baseline.json")
    }
    pub fn evolution_path(&self) -> PathBuf {
        self.root.join("meta").join("evolution.json")
    }
    pub fn backup_dir(&self, project: &str) -> PathBuf {
        self.root.join("backups").join(project)
    }

    // ─── Default Tactics ────────────────────────────────────────

    pub fn ensure_default_tactics(&self) -> Result<()> {
        let mut tactics = self.load_tactics()?;
        let existing: Vec<String> = tactics.iter().map(|t| t.name.clone()).collect();

        let defaults = vec![
            ("cargo-test", "Run cargo test before committing"),
            ("typecheck", "Run typecheck before committing"),
            ("lint-fix", "Run lint and auto-fix before committing"),
            ("doc-update", "Update docs when changing public API"),
        ];

        for (name, desc) in defaults {
            if !existing.contains(&name.to_string()) {
                tactics.push(Tactic {
                    name: name.to_string(),
                    description: desc.to_string(),
                    alpha: 1,
                    beta: 1,
                    source: "builtin".to_string(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                    last_used: Some(chrono::Utc::now().to_rfc3339()),
                });
            }
        }

        self.save_tactics(&tactics)?;
        Ok(())
    }

    // ─── Tactics (L3: Semantic Memory) ─────────────────────────

    pub fn load_tactics(&self) -> Result<Vec<Tactic>> {
        let path = self.tactics_path();
        if !path.exists() {
            return Ok(vec![]);
        }
        let data = fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&data)?)
    }

    pub fn save_tactics(&self, tactics: &[Tactic]) -> Result<()> {
        let data = serde_json::to_string_pretty(tactics)?;
        fs::write(self.tactics_path(), data)?;
        Ok(())
    }

    pub fn add_tactic(&self, name: &str, description: &str, source: &str) -> Result<Tactic> {
        let mut tactics = self.load_tactics()?;
        let tactic = Tactic::new(name, description, source);
        tactics.push(tactic.clone());
        self.save_tactics(&tactics)?;
        Ok(tactic)
    }

    pub fn record_event(
        &self,
        tactic_name: &str,
        outcome: Outcome,
        context: &str,
        project: Option<&str>,
    ) -> Result<()> {
        let entry = LedgerEntry {
            timestamp: chrono::Local::now().to_rfc3339(),
            tactic_name: tactic_name.to_string(),
            outcome: outcome.clone(),
            context: context.to_string(),
            project: project.map(|s| s.to_string()),
        };

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.ledger_path())?;
        serde_json::to_writer(&mut file, &entry)?;
        writeln!(file)?;

        let mut tactics = self.load_tactics()?;
        if let Some(tactic) = tactics.iter_mut().find(|t| t.name == tactic_name) {
            match outcome {
                Outcome::Success => tactic.record_success(),
                Outcome::Failure => tactic.record_failure(),
            }
            self.save_tactics(&tactics)?;
        }
        Ok(())
    }

    /// Thompson sampling: select top-k tactics by random sampling.
    pub fn select_tactics(&self, k: usize) -> Result<Vec<Tactic>> {
        let tactics = self.load_tactics()?;
        let mut rng = rand::thread_rng();
        let mut sampled: Vec<(f64, Tactic)> = tactics
            .into_iter()
            .map(|t| {
                // Simple Thompson sampling: sample ~ Beta(alpha, beta)
                // Using approximation: sample from uniform, then transform
                let u: f64 = rng.gen_range(0.0..1.0);
                let sample = t.alpha as f64 / (t.alpha + t.beta) as f64 + (u - 0.5) * 0.2;
                (sample.clamp(0.0, 1.0), t)
            })
            .collect();
        sampled.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        Ok(sampled.into_iter().take(k).map(|(_, t)| t).collect())
    }

    // ─── Backup ────────────────────────────────────────────────

    pub fn backup_project(&self, project: &str, project_memory: &ProjectMemory) -> Result<()> {
        let backup_dir = self.backup_dir(project);
        fs::create_dir_all(&backup_dir)?;

        for name in &[
            "sessions.jsonl",
            "findings.jsonl",
            "learnings.jsonl",
            "corrections.jsonl",
            "patterns.jsonl",
        ] {
            let src = project_memory.root.join(name);
            if src.exists() {
                fs::copy(&src, backup_dir.join(name))?;
            }
        }
        Ok(())
    }

    // ─── Meta (L5) ─────────────────────────────────────────────

    pub fn load_stats(&self) -> Result<MemoryStats> {
        let path = self.stats_path();
        if !path.exists() {
            return Ok(MemoryStats::default());
        }
        let data = fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&data)?)
    }

    pub fn save_stats(&self, stats: &MemoryStats) -> Result<()> {
        let dir = self.root.join("meta");
        fs::create_dir_all(&dir)?;
        let data = serde_json::to_string_pretty(stats)?;
        fs::write(self.stats_path(), data)?;
        Ok(())
    }
}

impl ProjectMemory {
    pub fn new(project_dir: &Path) -> Result<Self> {
        Self::from_project_root(project_dir)
    }

    pub fn from_project_root(project_dir: &Path) -> Result<Self> {
        let root = project_dir.join(".dijiang").join("memory");
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    pub fn from_dijiang_dir(dijiang_dir: &Path) -> Result<Self> {
        let root = dijiang_dir.join("memory");
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn new_at(root: &Path) -> Result<Self> {
        fs::create_dir_all(root)?;
        Ok(Self {
            root: root.to_path_buf(),
        })
    }

    // ─── L2: Episodic Memory ──────────────────────────────────

    fn append_jsonl<T: serde::Serialize>(&self, name: &str, value: &T) -> Result<()> {
        let path = self.root.join(name);
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        serde_json::to_writer(&mut file, value)?;
        writeln!(file)?;
        Ok(())
    }

    fn load_jsonl<T: for<'de> serde::Deserialize<'de>>(&self, name: &str) -> Result<Vec<T>> {
        let path = self.root.join(name);
        if !path.exists() {
            return Ok(vec![]);
        }
        let data = fs::read_to_string(&path)?;
        let mut results = Vec::new();
        for line in data.lines() {
            if !line.is_empty() {
                results.push(serde_json::from_str::<T>(line)?);
            }
        }
        Ok(results)
    }

    pub fn append_session_closure(&self, closure: &SessionClosure) -> Result<()> {
        self.append_jsonl("sessions.jsonl", closure)
    }

    pub fn load_session_closures(&self) -> Result<Vec<SessionClosure>> {
        self.load_jsonl("sessions.jsonl")
    }

    pub fn append_finding(&self, finding: &Finding) -> Result<()> {
        self.append_jsonl("findings.jsonl", finding)?;
        self.append_to_index("findings", &finding.content, &finding.timestamp)
    }

    pub fn append_learning(&self, learning: &Learning) -> Result<()> {
        self.append_jsonl("learnings.jsonl", learning)?;
        self.append_to_index("learnings", &learning.content, &learning.timestamp)
    }

    pub fn append_correction(&self, correction: &Correction) -> Result<()> {
        self.append_jsonl("corrections.jsonl", correction)
    }

    pub fn load_corrections(&self) -> Result<Vec<Correction>> {
        self.load_jsonl("corrections.jsonl")
    }

    pub fn load_findings(&self) -> Result<Vec<Finding>> {
        self.load_jsonl("findings.jsonl")
    }

    pub fn load_learnings(&self) -> Result<Vec<Learning>> {
        self.load_jsonl("learnings.jsonl")
    }

    // ─── L4: Procedural Memory ────────────────────────────────

    pub fn add_pattern(&self, pattern: &Pattern) -> Result<()> {
        self.append_jsonl("patterns.jsonl", pattern)?;
        let text = format!("{} {} {:?}", pattern.name, pattern.description, pattern.tags);
        self.append_to_index("patterns", &text, &pattern.created_at)
    }

    pub fn load_patterns(&self) -> Result<Vec<Pattern>> {
        self.load_jsonl("patterns.jsonl")
    }

    pub fn recent_patterns(&self, limit: usize) -> Result<Vec<Pattern>> {
        let mut patterns = self.load_patterns()?;
        patterns.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        patterns.truncate(limit);
        Ok(patterns)
    }

    /// Remove entries older than `days` from findings, learnings, and corrections.
    pub fn prune(&self, days: u64) -> Result<PruneReport> {
        let cutoff = chrono::Local::now() - chrono::Duration::days(days as i64);
        let cutoff_str = cutoff.to_rfc3339();
        let mut report = PruneReport::default();

        // Prune findings
        let all: Vec<Finding> = self.load_findings()?;
        let keep: Vec<Finding> = all.into_iter().filter(|f| f.timestamp >= cutoff_str).collect();
        report.findings_before = keep.len();
        let _ = std::fs::remove_file(self.root.join("findings.jsonl"));

        for f in &keep { self.append_finding(f)?; }

        // Prune learnings
        let all_l: Vec<Learning> = self.load_learnings()?;
        let keep_l: Vec<Learning> = all_l.into_iter().filter(|l| l.timestamp >= cutoff_str).collect();
        report.learnings_before = keep_l.len();
        let _ = std::fs::remove_file(self.root.join("learnings.jsonl"));
        for l in &keep_l { self.append_learning(l)?; }

        // Rebuild index
        let _ = self.build_index();
        Ok(report)
    }

    /// Recall: search findings, learnings, patterns for keyword matches.
    /// Returns scored results sorted by relevance (keyword hit count with time decay).
    pub fn recall(&self, query: &str, limit: usize) -> Result<Vec<ScoredMemory>> {
        // Try index first
        if let Ok(results) = self.search_index(query, limit) {
            if !results.is_empty() {
                return Ok(results);
            }
        }
        // Fall back to linear scan
        let results = self.recall_linear(query, limit)?;
        Ok(results)
    }

    // ─── Inverted Index ─────────────────────────────────────

    /// Append a single entry to the search index (incremental update).
    fn append_to_index(&self, source: &str, content: &str, timestamp: &str) -> Result<()> {
        for token in tokenize(content) {
            self.append_jsonl("index.jsonl", &IndexEntry {
                term: token,
                source: source.into(),
                content: content.into(),
                timestamp: timestamp.into(),
                line: 0,
            })?;
        }
        Ok(())
    }

    /// Path to the search index file.
    fn index_path(&self) -> PathBuf {
        self.root.join("index.jsonl")
    }

    /// Build the inverted search index from all memory sources.
    pub fn build_index(&self) -> Result<()> {
        // Remove old index file
        let path = self.index_path();
        let _ = std::fs::remove_file(&path);

        let findings = self.load_findings()?;
        for f in &findings {
            for token in tokenize(&f.content) {
                self.append_jsonl("index.jsonl", &IndexEntry {
                    term: token,
                    source: "findings".into(),
                    content: f.content.clone(),
                    timestamp: f.timestamp.clone(),
                    line: 0,
                })?;
            }
        }
        for l in self.load_learnings()? {
            for token in tokenize(&l.content) {
                self.append_jsonl("index.jsonl", &IndexEntry {
                    term: token,
                    source: "learnings".into(),
                    content: l.content.clone(),
                    timestamp: l.timestamp.clone(),
                    line: 0,
                })?;
            }
        }
        for p in self.load_patterns()? {
            let text = format!("{} {} {:?}", p.name, p.description, p.tags);
            for token in tokenize(&text) {
                self.append_jsonl("index.jsonl", &IndexEntry {
                    term: token,
                    source: "patterns".into(),
                    content: format!("{}: {}", p.name, p.description),
                    timestamp: p.created_at.clone(),
                    line: 0,
                })?;
            }
        }
        Ok(())
    }

    /// Search the inverted index for a query.
    pub fn search_index(&self, query: &str, limit: usize) -> Result<Vec<ScoredMemory>> {
        let path = self.index_path();
        if !path.exists() {
            return Ok(vec![]);
        }

        let entries: Vec<IndexEntry> = self.load_jsonl("index.jsonl")?;
        let q = query.to_lowercase();
        let terms: Vec<&str> = q.split_whitespace().collect();
        if terms.is_empty() {
            return Ok(vec![]);
        }

        let now = chrono::Local::now().timestamp();
        let mut results: Vec<ScoredMemory> = Vec::new();
        let mut seen: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();

        for entry in &entries {
            let hits: usize = terms.iter().filter(|&&t| entry.term.contains(t)).count();
            if hits == 0 { continue; }
            let key = (entry.source.clone(), entry.content.clone());
            if seen.contains(&key) { continue; }
            seen.insert(key.clone());

            let age_days = (now - parse_timestamp(&entry.timestamp)) as f64 / 86400.0;
            let time_decay = (-age_days * 0.01).exp();
            let score = (hits as f64) * time_decay;

            results.push(ScoredMemory {
                source: entry.source.clone(),
                content: entry.content.clone(),
                score,
                timestamp: entry.timestamp.clone(),
            });
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        Ok(results)
    }

    /// Linear scan recall (original implementation, used as fallback).
    fn recall_linear(&self, query: &str, limit: usize) -> Result<Vec<ScoredMemory>> {
        let q = query.to_lowercase();
        let terms: Vec<&str> = q.split_whitespace().collect();
        if terms.is_empty() {
            return Ok(vec![]);
        }

        let mut results: Vec<ScoredMemory> = Vec::new();
        let now = chrono::Local::now().timestamp();

        // Search findings
        for f in self.load_findings()? {
            let t = f.content.to_lowercase();
            let hits: usize = terms.iter().filter(|&&term| t.contains(term)).count();
            if hits > 0 {
                let age_days = (now - parse_timestamp(&f.timestamp)) as f64 / 86400.0;
                let time_decay = (-age_days * 0.01).exp(); // ~50% decay after 70 days
                let score = (hits as f64 / terms.len() as f64) * time_decay;
                results.push(ScoredMemory {
                    source: "findings".into(),
                    content: f.content,
                    score,
                    timestamp: f.timestamp,
                });
            }
        }

        // Search learnings
        for l in self.load_learnings()? {
            let t = l.content.to_lowercase();
            let hits: usize = terms.iter().filter(|&&term| t.contains(term)).count();
            if hits > 0 {
                let age_days = (now - parse_timestamp(&l.timestamp)) as f64 / 86400.0;
                let time_decay = (-age_days * 0.01).exp();
                let score = (hits as f64 / terms.len() as f64) * time_decay;
                results.push(ScoredMemory {
                    source: "learnings".into(),
                    content: l.content,
                    score,
                    timestamp: l.timestamp,
                });
            }
        }

        // Search patterns
        for p in self.load_patterns()? {
            let t = format!("{} {} {:?}", p.name, p.description, p.tags).to_lowercase();
            let hits: usize = terms.iter().filter(|&&term| t.contains(term)).count();
            if hits > 0 {
                let age_days = (now - parse_timestamp(&p.created_at)) as f64 / 86400.0;
                let time_decay = (-age_days * 0.01).exp();
                let score = (hits as f64 / terms.len() as f64) * time_decay;
                results.push(ScoredMemory {
                    source: "patterns".into(),
                    content: format!("{}: {}", p.name, p.description),
                    score,
                    timestamp: p.created_at,
                });
            }
        }

        // Sort by score descending, take top N
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);
        Ok(results)
    }
}

fn parse_timestamp(ts: &str) -> i64 {
    chrono::DateTime::parse_from_rfc3339(ts)
        .map(|dt| dt.timestamp())
        .unwrap_or(0)
}

/// Tokenize text into lowercase terms, splitting on non-alphanumeric characters.
fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 2)
        .map(|t| t.to_string())
        .collect()
}
