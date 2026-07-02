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
        self.append_jsonl("findings.jsonl", finding)
    }

    pub fn append_learning(&self, learning: &Learning) -> Result<()> {
        self.append_jsonl("learnings.jsonl", learning)
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
        self.append_jsonl("patterns.jsonl", pattern)
    }

    pub fn load_patterns(&self) -> Result<Vec<Pattern>> {
        self.load_jsonl("patterns.jsonl")
    }
}
