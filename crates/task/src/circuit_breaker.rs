//! Circuit breaker for DiJiang agent loops.
//!
//! Ported from loop-engineering's `loop-context` TypeScript implementation,
//! rewritten in Rust with the same deterministic, zero-dependency philosophy.
//!
//! The circuit breaker prevents agent loops from running indefinitely by
//! detecting four conditions in priority order:
//!
//! 1. **Stagnation**: the same error signature appears N consecutive times
//! 2. **No-progress**: N consecutive failures with no success
//! 3. **Token budget**: cumulative tokens exceed a configured budget
//! 4. **Max iterations**: iteration count exceeds a configured limit
//!
//! When any condition is met, the loop should stop or escalate rather than
//! continue trying the same approach.

use serde::{Deserialize, Serialize};
use std::fmt;

// ─── Error Signature ──────────────────────────────────────────

/// Normalize a runtime error / stack trace into a stable signature.
///
/// Strips volatile details (timestamps, memory addresses, line numbers,
/// temporary paths, port numbers) so that "the same underlying error"
/// produces the same signature across iterations.
///
/// Algorithm (ported from loop-engineering's `errorSignature`):
/// - ISO timestamps → `<ts>`
/// - Hex addresses → `<addr>`
/// - File paths → basename only
/// - `:line:col` annotations → stripped
/// - Standalone digits → `#`
pub fn error_signature(error: &str) -> String {
    compute_error_signature(error)
}

/// Specialized replacer for ISO timestamps.
fn replace_iso_timestamps(input: &str) -> String {
    let mut result = String::new();
    let chars = input.chars().collect::<Vec<_>>();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Try to match YYYY-MM-DD[T ]HH:MM:SS
        if i + 19 <= len
            && is_digit(chars[i])
            && is_digit(chars[i + 1])
            && is_digit(chars[i + 2])
            && is_digit(chars[i + 3])
            && chars[i + 4] == '-'
            && is_digit(chars[i + 5])
            && is_digit(chars[i + 6])
            && chars[i + 7] == '-'
            && is_digit(chars[i + 8])
            && is_digit(chars[i + 9])
        {
            let sep = chars[i + 10];
            if sep == 'T' || sep == ' ' {
                if i + 19 <= len
                    && is_digit(chars[i + 11])
                    && is_digit(chars[i + 12])
                    && chars[i + 13] == ':'
                    && is_digit(chars[i + 14])
                    && is_digit(chars[i + 15])
                    && chars[i + 16] == ':'
                    && is_digit(chars[i + 17])
                    && is_digit(chars[i + 18])
                {
                    // Matched core timestamp; check for fractional seconds
                    let mut end = i + 19;
                    if end < len && chars[end] == '.' {
                        end += 1;
                        while end < len && is_digit(chars[end]) {
                            end += 1;
                        }
                    }
                    // Check for timezone: Z or ±HH:MM
                    if end < len && chars[end] == 'Z' {
                        end += 1;
                    } else if end + 2 < len
                        && (chars[end] == '+' || chars[end] == '-')
                        && is_digit(chars[end + 1])
                        && is_digit(chars[end + 2])
                    {
                        end += 3;
                        if end < len && chars[end] == ':' {
                            end += 1;
                            if end + 1 < len && is_digit(chars[end]) && is_digit(chars[end + 1]) {
                                end += 2;
                            }
                        }
                    }
                    result.push_str("<ts>");
                    i = end;
                    continue;
                }
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

/// Specialized replacer for hex addresses (0x followed by 4+ hex digits).
fn replace_hex_addresses(input: &str) -> String {
    let mut result = String::new();
    let chars = input.chars().collect::<Vec<_>>();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if i + 2 < len && chars[i] == '0' && chars[i + 1] == 'x' {
            let mut hex_len = 0;
            let mut j = i + 2;
            while j < len && is_hex_digit(chars[j]) {
                hex_len += 1;
                j += 1;
            }
            if hex_len >= 4 {
                result.push_str("<addr>");
                i = j;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

/// Specialized replacer for port-like patterns: :3000, :8080 (2-5 digits after colon).
fn replace_port_numbers(input: &str) -> String {
    let mut result = String::new();
    let chars = input.chars().collect::<Vec<_>>();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == ':' && i + 1 < len && is_digit(chars[i + 1]) {
            // Count digits
            let mut digit_count = 0;
            let mut j = i + 1;
            while j < len && is_digit(chars[j]) {
                digit_count += 1;
                j += 1;
            }
            if digit_count >= 2 && digit_count <= 5 {
                // Check that next char is a separator (space, end, comma, paren)
                if j >= len
                    || chars[j] == ' '
                    || chars[j] == ','
                    || chars[j] == ')'
                    || chars[j] == '\n'
                {
                    result.push(':');
                    i = j;
                    continue;
                }
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

/// Specialized replacer for line/column annotations: :42:3, :42
/// Only match when preceded by a non-digit and followed by a separator.
fn replace_line_col_annotations(input: &str) -> String {
    let mut result = String::new();
    let chars = input.chars().collect::<Vec<_>>();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Match :digits[:digits] when the colon is preceded by a non-digit
        // and the digits are followed by a separator
        if chars[i] == ':'
            && i > 0
            && !is_digit(chars[i - 1])
            && i + 1 < len
            && is_digit(chars[i + 1])
        {
            let mut j = i + 1;
            while j < len && is_digit(chars[j]) {
                j += 1;
            }
            // Optional second :digits
            if j < len && chars[j] == ':' {
                j += 1;
                while j < len && is_digit(chars[j]) {
                    j += 1;
                }
            }
            // Must be followed by a separator
            if j >= len || chars[j] == ' ' || chars[j] == ',' || chars[j] == ')' || chars[j] == '\n'
            {
                // Skip the entire match (remove it)
                i = j;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

/// Keep only basename of file paths.
/// Matches /path/to/file.ext and replaces with file.ext.
fn regex_replace_path_basenames(input: &str) -> String {
    let mut result = String::new();
    let mut current_token = String::new();
    let mut in_path = false;

    for ch in input.chars() {
        if ch == '/' {
            in_path = true;
            current_token.clear();
        } else if in_path {
            if ch == ' ' || ch == '\n' || ch == ',' || ch == ')' || ch == '(' {
                // End of path token — emit basename
                if !current_token.is_empty() {
                    result.push_str(&current_token);
                }
                result.push(ch);
                in_path = false;
                current_token.clear();
            } else {
                current_token.push(ch);
            }
        } else {
            result.push(ch);
        }
    }
    // Flush remaining
    if in_path && !current_token.is_empty() {
        result.push_str(&current_token);
    }
    result
}

/// Replace standalone digit sequences (2+ digits) with #.
/// Only when the digits form a standalone word (bounded by non-digit chars).
fn replace_standalone_digits(input: &str) -> String {
    let mut result = String::new();
    let chars = input.chars().collect::<Vec<_>>();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if is_digit(chars[i]) && (i == 0 || !is_digit(chars[i - 1])) {
            let mut digit_count = 0;
            let mut j = i;
            while j < len && is_digit(chars[j]) {
                digit_count += 1;
                j += 1;
            }
            // Check word boundary after digits
            if j >= len || !is_alphanumeric(chars[j]) {
                if digit_count >= 2 {
                    result.push('#');
                    i = j;
                    continue;
                }
            }
            // Single digit or part of alphanumeric — keep original
            for k in i..j {
                result.push(chars[k]);
            }
            i = j;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

/// Full error_signature pipeline using the specialized replacers.
pub fn compute_error_signature(error: &str) -> String {
    let mut sig = error.to_string();

    sig = replace_iso_timestamps(&sig);
    sig = replace_hex_addresses(&sig);
    sig = replace_port_numbers(&sig);
    sig = replace_line_col_annotations(&sig);
    sig = regex_replace_path_basenames(&sig);
    sig = replace_standalone_digits(&sig);

    // Trim and collapse whitespace
    sig = sig
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let mut collapsed = String::new();
    let mut prev_char = '\0';
    for ch in sig.chars() {
        if ch == ' ' && prev_char == ' ' {
            continue;
        }
        collapsed.push(ch);
        prev_char = ch;
    }
    collapsed.trim().to_string()
}

fn is_digit(ch: char) -> bool {
    ch.is_ascii_digit()
}

fn is_hex_digit(ch: char) -> bool {
    ch.is_ascii_hexdigit()
}

fn is_alphanumeric(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

// ─── Data Structures ─────────────────────────────────────────

/// Configuration for the circuit breaker.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CircuitBreakerConfig {
    /// Maximum number of iterations before triggering.
    pub max_iterations: u64,
    /// Number of consecutive same-signature failures before stagnation trigger.
    pub stagnation_threshold: u64,
    /// Number of consecutive failures (any signature) before no-progress trigger.
    pub no_progress_threshold: u64,
    /// Optional cumulative token budget.
    pub token_budget: Option<u64>,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            stagnation_threshold: 3,
            no_progress_threshold: 5,
            token_budget: None,
        }
    }
}

/// The outcome of a single attempt in the ledger.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AttemptOutcome {
    Success,
    Failure,
    Noop,
}

impl fmt::Display for AttemptOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttemptOutcome::Success => write!(f, "success"),
            AttemptOutcome::Failure => write!(f, "failure"),
            AttemptOutcome::Noop => write!(f, "noop"),
        }
    }
}

/// A single attempt recorded in the ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attempt {
    /// Iteration number (1-based).
    pub iteration: u64,
    /// What action was attempted.
    pub action: String,
    /// Outcome of this attempt.
    pub outcome: AttemptOutcome,
    /// Error message (if failure).
    pub error: Option<String>,
    /// Tokens consumed (if tracked).
    pub tokens_used: Option<u64>,
    /// Number of consecutive repeats of the same error signature.
    /// Used by prune to fold repeated failures.
    #[serde(default)]
    pub repeated: Option<u64>,
}

/// The ledger tracking all attempts for a goal.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Ledger {
    /// The goal this ledger is tracking.
    pub goal: String,
    /// All recorded attempts.
    pub attempts: Vec<Attempt>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BreakerTrigger {
    Stagnation,
    NoProgress,
    TokenBudget,
    MaxIterations,
}

impl fmt::Display for BreakerTrigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BreakerTrigger::Stagnation => write!(f, "stagnation"),
            BreakerTrigger::NoProgress => write!(f, "no-progress"),
            BreakerTrigger::TokenBudget => write!(f, "token-budget"),
            BreakerTrigger::MaxIterations => write!(f, "max-iterations"),
        }
    }
}

/// Decision returned by the circuit breaker check.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BreakerDecision {
    /// Whether the loop should continue.
    pub should_continue: bool,
    /// Whether the situation should be escalated to a human.
    pub escalate: bool,
    /// Which trigger condition fired (if any).
    pub trigger: Option<BreakerTrigger>,
    /// Human-readable reason for the decision.
    pub reason: String,
}

impl fmt::Display for BreakerDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.should_continue {
            write!(f, "continue")
        } else {
            write!(
                f,
                "STOP ({}) — {}",
                self.trigger
                    .as_ref()
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                self.reason
            )
        }
    }
}

// ─── Circuit Breaker Check ────────────────────────────────────

/// Check whether the circuit breaker should stop the loop.
///
/// Evaluates four conditions in priority order (most specific/cheapest first):
/// 1. Stagnation: same error signature repeated N consecutive times
/// 2. No-progress: N consecutive failures regardless of error
/// 3. Token budget: cumulative tokens exceed configured budget
/// 4. Max iterations: iteration count exceeds configured limit
///
/// When multiple conditions are met, the most actionable trigger is reported.
pub fn check_circuit_breaker(ledger: &Ledger, config: &CircuitBreakerConfig) -> BreakerDecision {
    let failures = ledger
        .attempts
        .iter()
        .filter(|a| a.outcome == AttemptOutcome::Failure)
        .count();

    // 1. Stagnation: same error signature repeated consecutively
    let last_n_signatures: Vec<String> = ledger
        .attempts
        .iter()
        .rev()
        .take(config.stagnation_threshold as usize)
        .filter(|a| a.outcome == AttemptOutcome::Failure)
        .map(|a| {
            a.error
                .as_deref()
                .map(compute_error_signature)
                .unwrap_or_default()
        })
        .collect();

    if !last_n_signatures.is_empty()
        && last_n_signatures.len() >= config.stagnation_threshold as usize
        && last_n_signatures
            .iter()
            .all(|sig| sig == &last_n_signatures[0])
        && !last_n_signatures[0].is_empty()
    {
        return BreakerDecision {
            should_continue: false,
            escalate: true,
            trigger: Some(BreakerTrigger::Stagnation),
            reason: format!(
                "Loop stagnated: same error '{}' repeated {} consecutive times",
                last_n_signatures[0],
                last_n_signatures.len()
            ),
        };
    }

    // 2. No-progress: consecutive failures without any success
    let consecutive_failures = ledger
        .attempts
        .iter()
        .rev()
        .take_while(|a| a.outcome != AttemptOutcome::Success)
        .filter(|a| a.outcome == AttemptOutcome::Failure)
        .count();

    if consecutive_failures >= config.no_progress_threshold as usize {
        return BreakerDecision {
            should_continue: false,
            escalate: true,
            trigger: Some(BreakerTrigger::NoProgress),
            reason: format!(
                "No progress: {} consecutive failures without success",
                consecutive_failures
            ),
        };
    }

    // 3. Token budget exceeded
    if let Some(budget) = config.token_budget {
        let total_tokens: u64 = ledger.attempts.iter().filter_map(|a| a.tokens_used).sum();
        if total_tokens > budget {
            return BreakerDecision {
                should_continue: false,
                escalate: false,
                trigger: Some(BreakerTrigger::TokenBudget),
                reason: format!(
                    "Token budget exceeded: {} tokens used vs {} budget",
                    total_tokens, budget
                ),
            };
        }
    }

    // 4. Max iterations exceeded
    let max_iteration = ledger
        .attempts
        .iter()
        .map(|a| a.iteration)
        .max()
        .unwrap_or(0);
    if max_iteration >= config.max_iterations {
        return BreakerDecision {
            should_continue: false,
            escalate: false,
            trigger: Some(BreakerTrigger::MaxIterations),
            reason: format!(
                "Max iterations reached: {} attempts vs {} limit",
                max_iteration, config.max_iterations
            ),
        };
    }

    // All checks passed — continue
    BreakerDecision {
        should_continue: true,
        escalate: false,
        trigger: None,
        reason: format!(
            "Loop healthy: {} attempts, {} failures, still within limits",
            ledger.attempts.len(),
            failures
        ),
    }
}

// ─── Ledger Pruning ───────────────────────────────────────────

/// Configuration for ledger pruning.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PruneConfig {
    /// How many recent attempts to keep in the window.
    pub window_size: usize,
    /// Maximum lines of error/stack trace to retain per attempt.
    pub max_trace_lines: usize,
}

impl Default for PruneConfig {
    fn default() -> Self {
        Self {
            window_size: 5,
            max_trace_lines: 8,
        }
    }
}

/// Prune the ledger to a bounded window, folding consecutive same-signature
/// failures and truncating long error traces.
///
/// Returns a pruned ledger suitable for injecting into agent context
/// without bloating the prompt.
pub fn prune_ledger(ledger: &Ledger, config: &PruneConfig) -> Ledger {
    let mut windowed: Vec<Attempt> = ledger
        .attempts
        .iter()
        .rev()
        .take(config.window_size)
        .cloned()
        .collect();

    // Truncate error traces
    for attempt in &mut windowed {
        if let Some(error) = &attempt.error {
            let lines = error.lines().collect::<Vec<_>>();
            if lines.len() > config.max_trace_lines {
                let pruned_trace = lines[..config.max_trace_lines].join("\n");
                attempt.error = Some(format!(
                    "{}\n… {} more lines pruned",
                    pruned_trace,
                    lines.len() - config.max_trace_lines
                ));
            }
        }
    }

    // Fold consecutive same-signature failures
    let mut folded: Vec<Attempt> = Vec::new();
    for attempt in windowed {
        let sig = attempt
            .error
            .as_deref()
            .map(compute_error_signature)
            .unwrap_or_default();
        if let Some(last) = folded.last_mut() {
            let last_sig = last
                .error
                .as_deref()
                .map(compute_error_signature)
                .unwrap_or_default();
            if !sig.is_empty() && sig == last_sig && attempt.outcome == AttemptOutcome::Failure {
                // Fold: increment repeat count instead of adding a new entry
                last.repeated = Some(last.repeated.unwrap_or(1) + 1);
                continue;
            }
        }
        folded.push(attempt);
    }

    // Reverse back to chronological order
    folded.reverse();

    Ledger {
        goal: ledger.goal.clone(),
        attempts: folded,
    }
}

// ─── Attempt Summary ──────────────────────────────────────────

/// Produce a deterministic fact summary of the ledger for context injection.
///
/// Lists: total attempts, success/failure counts, distinct error groups
/// (sorted by frequency), actions already tried.
pub fn summarize_attempts(ledger: &Ledger) -> String {
    let total = ledger.attempts.len();
    let successes = ledger
        .attempts
        .iter()
        .filter(|a| a.outcome == AttemptOutcome::Success)
        .count();
    let failures = ledger
        .attempts
        .iter()
        .filter(|a| a.outcome == AttemptOutcome::Failure)
        .count();

    // Distinct error groups by signature, sorted by frequency
    let mut error_groups: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for attempt in &ledger.attempts {
        if attempt.outcome == AttemptOutcome::Failure {
            if let Some(error) = &attempt.error {
                let sig = compute_error_signature(error);
                if !sig.is_empty() {
                    *error_groups.entry(sig).or_insert(0) += 1;
                }
            }
        }
    }
    let mut sorted_errors: Vec<(String, usize)> = error_groups.into_iter().collect();
    sorted_errors.sort_by(|a, b| b.1.cmp(&a.1));

    // Actions already tried
    let actions: Vec<String> = ledger.attempts.iter().map(|a| a.action.clone()).collect();

    let mut lines = vec![format!(
        "Progress: {} attempts, {} successes, {} failures",
        total, successes, failures
    )];

    if !sorted_errors.is_empty() {
        lines.push("Distinct errors:".to_string());
        for (sig, count) in &sorted_errors {
            lines.push(format!("  - {} (×{})", sig, count));
        }
    }

    if !actions.is_empty() {
        lines.push(format!("Actions tried: {}", actions.join(", ")));
    }

    lines.join("\n")
}

// ─── Context Injection Builder ────────────────────────────────

/// Build a compact context block for injecting into the next prompt.
///
/// Combines goal, progress, attempted actions, failure patterns,
/// and circuit breaker status into a single block.
pub fn build_context_injection(
    ledger: &Ledger,
    breaker_decision: &BreakerDecision,
    prune_config: &PruneConfig,
) -> String {
    let pruned = prune_ledger(ledger, &prune_config);
    let summary = summarize_attempts(&pruned);

    let mut lines = vec![format!("Goal: {}", ledger.goal)];
    lines.push(summary);

    // Recent error (from pruned ledger)
    if let Some(last_failure) = pruned
        .attempts
        .iter()
        .rev()
        .find(|a| a.outcome == AttemptOutcome::Failure)
    {
        if let Some(error) = &last_failure.error {
            let error_preview = error.lines().take(3).collect::<Vec<_>>().join("\n");
            lines.push(format!("Last error:\n{}", error_preview));
        }
    }

    // Circuit breaker status
    lines.push(format!("Circuit breaker: {}", breaker_decision));

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_signature_normalizes_timestamps() {
        let error = "error at 2026-07-07T14:39:55Z: connection refused";
        let sig = compute_error_signature(error);
        assert_eq!(sig, "error at <ts>: connection refused");
    }

    #[test]
    fn error_signature_normalizes_hex_addresses() {
        let error = "panic at 0x7f8a3b2c1d00: null pointer";
        let sig = compute_error_signature(error);
        assert_eq!(sig, "panic at <addr>: null pointer");
    }

    #[test]
    fn error_signature_normalizes_line_numbers() {
        let error = "error in src/main.rs:42:3: type mismatch";
        let sig = compute_error_signature(error);
        assert!(
            sig.contains("main.rs") && !sig.contains(":42"),
            "sig should strip line numbers: {sig}"
        );
    }

    #[test]
    fn error_signature_normalizes_paths_to_basenames() {
        let error = "cannot open /home/user/project/src/main.rs";
        let sig = compute_error_signature(error);
        assert!(
            sig.contains("main.rs") && !sig.contains("/home/user/project/src/"),
            "sig should keep only basename: {sig}"
        );
    }

    #[test]
    fn error_signature_normalizes_digit_sequences() {
        // 30000 is adjacent to "ms" so it's NOT standalone — it stays.
        // But standalone numbers like 42 after # should be replaced.
        let error = "timeout after 30000ms retry count 42";
        let sig = compute_error_signature(error);
        // "30000ms" stays because 30000 is bounded by "ms" (alphanumeric).
        // "42" at end of string IS standalone → replaced with #
        assert!(
            sig.contains("30000ms") || sig.contains("#ms"),
            "adjacent-to-word digits may or may not be replaced: {sig}"
        );
        assert!(
            !sig.contains(" 42") && sig.contains("#"),
            "standalone 42 should be replaced: {sig}"
        );
    }

    #[test]
    fn error_signature_same_error_produces_same_signature() {
        let error_a =
            "error at 2026-07-07T14:39:55Z in /path/to/main.rs:42: type mismatch at 0x7f8a3b2c1d00";
        let error_b = "error at 2026-07-08T09:12:30Z in /other/path/to/main.rs:99: type mismatch at 0xdeadbeef0042";
        let sig_a = compute_error_signature(error_a);
        let sig_b = compute_error_signature(error_b);
        assert_eq!(
            sig_a, sig_b,
            "same underlying error should produce same signature: a={sig_a} b={sig_b}"
        );
    }

    #[test]
    fn circuit_breaker_detects_stagnation() {
        let config = CircuitBreakerConfig {
            stagnation_threshold: 3,
            ..Default::default()
        };
        let ledger = Ledger {
            goal: "fix login bug".to_string(),
            attempts: vec![
                Attempt {
                    iteration: 1,
                    action: "run test".into(),
                    outcome: AttemptOutcome::Failure,
                    error: Some("TypeError: Cannot read property 'user' at src/auth.rs:42".into()),
                    tokens_used: None,
                    repeated: None,
                },
                Attempt {
                    iteration: 2,
                    action: "run test".into(),
                    outcome: AttemptOutcome::Failure,
                    error: Some("TypeError: Cannot read property 'user' at src/auth.rs:99".into()),
                    tokens_used: None,
                    repeated: None,
                },
                Attempt {
                    iteration: 3,
                    action: "run test".into(),
                    outcome: AttemptOutcome::Failure,
                    error: Some("TypeError: Cannot read property 'user' at src/auth.rs:150".into()),
                    tokens_used: None,
                    repeated: None,
                },
            ],
        };
        let decision = check_circuit_breaker(&ledger, &config);
        assert!(!decision.should_continue);
        assert!(decision.escalate);
        assert_eq!(decision.trigger, Some(BreakerTrigger::Stagnation));
    }

    #[test]
    fn circuit_breaker_detects_no_progress() {
        let config = CircuitBreakerConfig {
            no_progress_threshold: 5,
            stagnation_threshold: 10, // high enough to not trigger first
            ..Default::default()
        };
        let ledger = Ledger {
            goal: "fix build".to_string(),
            attempts: (1..=6)
                .map(|i| Attempt {
                    iteration: i,
                    action: "cargo build".into(),
                    outcome: AttemptOutcome::Failure,
                    error: Some(format!("different error {}", i)),
                    tokens_used: None,
                    repeated: None,
                })
                .collect(),
        };
        let decision = check_circuit_breaker(&ledger, &config);
        assert!(!decision.should_continue);
        assert_eq!(decision.trigger, Some(BreakerTrigger::NoProgress));
    }

    #[test]
    fn circuit_breaker_detects_token_budget() {
        let config = CircuitBreakerConfig {
            stagnation_threshold: 100,
            no_progress_threshold: 100,
            token_budget: Some(1000),
            max_iterations: 100,
        };
        let ledger = Ledger {
            goal: "fix bug".to_string(),
            attempts: vec![
                Attempt {
                    iteration: 1,
                    action: "analyze".into(),
                    outcome: AttemptOutcome::Failure,
                    error: Some("err".into()),
                    tokens_used: Some(600),
                    repeated: None,
                },
                Attempt {
                    iteration: 2,
                    action: "fix".into(),
                    outcome: AttemptOutcome::Failure,
                    error: Some("err".into()),
                    tokens_used: Some(500),
                    repeated: None,
                },
            ],
        };
        let decision = check_circuit_breaker(&ledger, &config);
        assert!(!decision.should_continue);
        assert_eq!(decision.trigger, Some(BreakerTrigger::TokenBudget));
    }

    #[test]
    fn circuit_breaker_detects_max_iterations() {
        let config = CircuitBreakerConfig {
            stagnation_threshold: 100,
            no_progress_threshold: 100,
            max_iterations: 3,
            token_budget: None,
        };
        let ledger = Ledger {
            goal: "fix bug".to_string(),
            attempts: vec![
                Attempt {
                    iteration: 1,
                    action: "try A".into(),
                    outcome: AttemptOutcome::Success,
                    error: None,
                    tokens_used: None,
                    repeated: None,
                },
                Attempt {
                    iteration: 2,
                    action: "try B".into(),
                    outcome: AttemptOutcome::Success,
                    error: None,
                    tokens_used: None,
                    repeated: None,
                },
                Attempt {
                    iteration: 3,
                    action: "try C".into(),
                    outcome: AttemptOutcome::Success,
                    error: None,
                    tokens_used: None,
                    repeated: None,
                },
            ],
        };
        let decision = check_circuit_breaker(&ledger, &config);
        assert!(!decision.should_continue);
        assert_eq!(decision.trigger, Some(BreakerTrigger::MaxIterations));
    }

    #[test]
    fn circuit_breaker_continues_when_healthy() {
        let config = CircuitBreakerConfig::default();
        let ledger = Ledger {
            goal: "fix bug".to_string(),
            attempts: vec![
                Attempt {
                    iteration: 1,
                    action: "try A".into(),
                    outcome: AttemptOutcome::Success,
                    error: None,
                    tokens_used: None,
                    repeated: None,
                },
                Attempt {
                    iteration: 2,
                    action: "try B".into(),
                    outcome: AttemptOutcome::Failure,
                    error: Some("different error".into()),
                    tokens_used: None,
                    repeated: None,
                },
            ],
        };
        let decision = check_circuit_breaker(&ledger, &config);
        assert!(decision.should_continue);
        assert!(decision.trigger.is_none());
    }

    #[test]
    fn prune_ledger_truncates_window() {
        let config = PruneConfig {
            window_size: 3,
            max_trace_lines: 100,
        };
        let ledger = Ledger {
            goal: "test".to_string(),
            attempts: (1..=10)
                .map(|i| Attempt {
                    iteration: i,
                    action: format!("action {}", i),
                    outcome: AttemptOutcome::Failure,
                    error: Some(format!("error {}", i)),
                    tokens_used: None,
                    repeated: None,
                })
                .collect(),
        };
        let pruned = prune_ledger(&ledger, &config);
        assert!(
            pruned.attempts.len() <= 3,
            "pruned should have at most 3 attempts"
        );
    }

    #[test]
    fn prune_ledger_folds_repeated_errors() {
        let config = PruneConfig {
            window_size: 10,
            max_trace_lines: 100,
        };
        let ledger = Ledger {
            goal: "test".to_string(),
            attempts: vec![
                Attempt {
                    iteration: 1,
                    action: "run".into(),
                    outcome: AttemptOutcome::Failure,
                    error: Some("TypeError at auth.rs:42: bad".into()),
                    tokens_used: None,
                    repeated: None,
                },
                Attempt {
                    iteration: 2,
                    action: "run".into(),
                    outcome: AttemptOutcome::Failure,
                    error: Some("TypeError at auth.rs:99: bad".into()),
                    tokens_used: None,
                    repeated: None,
                },
                Attempt {
                    iteration: 3,
                    action: "run".into(),
                    outcome: AttemptOutcome::Failure,
                    error: Some("TypeError at auth.rs:150: bad".into()),
                    tokens_used: None,
                    repeated: None,
                },
                Attempt {
                    iteration: 4,
                    action: "fix".into(),
                    outcome: AttemptOutcome::Success,
                    error: None,
                    tokens_used: None,
                    repeated: None,
                },
            ],
        };
        let pruned = prune_ledger(&ledger, &config);
        // Same-signature failures should be folded into one entry with repeated count
        let failure_entries = pruned
            .attempts
            .iter()
            .filter(|a| a.outcome == AttemptOutcome::Failure)
            .count();
        assert!(
            failure_entries <= 2,
            "same-signature failures should be folded: got {failure_entries} entries"
        );
    }

    #[test]
    fn prune_ledger_truncates_long_traces() {
        let config = PruneConfig {
            window_size: 10,
            max_trace_lines: 3,
        };
        let long_trace = (1..=20)
            .map(|i| format!("trace line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let ledger = Ledger {
            goal: "test".to_string(),
            attempts: vec![Attempt {
                iteration: 1,
                action: "run".into(),
                outcome: AttemptOutcome::Failure,
                error: Some(long_trace),
                tokens_used: None,
                repeated: None,
            }],
        };
        let pruned = prune_ledger(&ledger, &config);
        let error = pruned.attempts[0].error.as_ref().unwrap();
        assert!(
            error.contains("… 17 more lines pruned"),
            "long trace should be truncated: {error}"
        );
        assert!(
            error.lines().count() <= 5,
            "should have at most ~5 lines (3 trace + pruned msg)"
        );
    }

    #[test]
    fn summarize_attempts_lists_distinct_errors() {
        let ledger = Ledger {
            goal: "fix bug".to_string(),
            attempts: vec![
                Attempt {
                    iteration: 1,
                    action: "test".into(),
                    outcome: AttemptOutcome::Failure,
                    error: Some("TypeError".into()),
                    tokens_used: None,
                    repeated: None,
                },
                Attempt {
                    iteration: 2,
                    action: "test".into(),
                    outcome: AttemptOutcome::Failure,
                    error: Some("TypeError".into()),
                    tokens_used: None,
                    repeated: None,
                },
                Attempt {
                    iteration: 3,
                    action: "test".into(),
                    outcome: AttemptOutcome::Failure,
                    error: Some("NetworkError".into()),
                    tokens_used: None,
                    repeated: None,
                },
                Attempt {
                    iteration: 4,
                    action: "fix".into(),
                    outcome: AttemptOutcome::Success,
                    error: None,
                    tokens_used: None,
                    repeated: None,
                },
            ],
        };
        let summary = summarize_attempts(&ledger);
        assert!(summary.contains("4 attempts, 1 successes, 3 failures"));
        assert!(summary.contains("Distinct errors:"));
        // TypeError should appear with count 2 (most frequent)
        assert!(summary.contains("×2"));
    }

    #[test]
    fn build_context_injection_compact_block() {
        let ledger = Ledger {
            goal: "fix login bug".to_string(),
            attempts: vec![Attempt {
                iteration: 1,
                action: "run test".into(),
                outcome: AttemptOutcome::Failure,
                error: Some("TypeError".into()),
                tokens_used: None,
                repeated: None,
            }],
        };
        let breaker = BreakerDecision {
            should_continue: true,
            escalate: false,
            trigger: None,
            reason: "healthy".to_string(),
        };
        let config = PruneConfig::default();
        let injection = build_context_injection(&ledger, &breaker, &config);
        assert!(injection.contains("Goal: fix login bug"));
        assert!(injection.contains("Circuit breaker: continue"));
    }

    #[test]
    fn breaker_decision_display_format() {
        let stop = BreakerDecision {
            should_continue: false,
            escalate: true,
            trigger: Some(BreakerTrigger::Stagnation),
            reason: "stuck".to_string(),
        };
        assert_eq!(stop.to_string(), "STOP (stagnation) — stuck");

        let continue_ = BreakerDecision {
            should_continue: true,
            escalate: false,
            trigger: None,
            reason: "ok".to_string(),
        };
        assert_eq!(continue_.to_string(), "continue");
    }
}
