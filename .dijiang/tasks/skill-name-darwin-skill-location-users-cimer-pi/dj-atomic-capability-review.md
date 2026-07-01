# DJ Atomic Capability Review

Generated: 2026-07-01T02:04:49+00:00

## Scope

Reviewed all built-in `dj-*` template skills under `crates/configurator/templates/skills/*/SKILL.md` for atomicity, overlap, and redundancy.

## Summary

The `dj-*` set is broadly atomic. No skill is redundant enough to delete. Several skills intentionally share cross-cutting lenses such as first-principles reasoning, adversarial review, simplicity, and verification, but their input/output contracts remain distinct.

One real boundary conflict was found and fixed: `dj-check` frontmatter described itself as code review and merge-oriented, while its body correctly defines it as a delivery quality gate that must not commit, push, merge, tag, or clean worktrees. The frontmatter now says `dj-check` is a delivery quality gate and finish-work readiness review.

## Atomic Capability Matrix

| Skill | Atomic capability | Boundary |
|---|---|---|
| `dj-dispatch` | Route new requests to the correct workflow | Does not implement, debug, audit, or write docs |
| `dj-grill` | Align fuzzy intent into confirmed requirements | Does not implement or write design docs |
| `dj-output` | Create/update project docs and code-doc alignment | Does not invent requirements or modify code |
| `dj-implement` | Implement verified code diff from confirmed requirements | Does not commit/push/merge or do unrelated refactors |
| `dj-tdd` | Test-first implementation via red-green-refactor slices | Does not batch tests or test private internals |
| `dj-hunt` | Diagnose bugs from symptoms to root cause, then fix | Does not patch before root cause |
| `dj-check` | Delivery quality gate and finish-work readiness evidence | Does not modify code or perform release actions |
| `dj-review` | Lightweight read-only diff/PR review | Does not run full quality gate or replace `dj-check` |
| `dj-audit` | Whole-repo read-only bloat/security scan | Does not fix findings or chase correctness bugs |
| `dj-health` | Agent/tooling/config health check | Does not inspect application code quality or fix config |
| `dj-debt` | Read `ponytail:` markers into a debt ledger | Does not create docs or fix debt by default |
| `dj-pattern` | Identify recurring code patterns and anti-patterns | Does not edit code or run broad audits |
| `dj-ponytail` | Overlay lens for minimal/YAGNI coding | Does not replace the active workflow skill |
| `dj-karpathy` | Overlay lens for disciplined LLM coding behavior | Does not replace the active workflow skill |
| `dj-prototype` | Disposable runnable prototype to answer one design question | Does not ship prototype code |
| `dj-script` | Standalone script/tool creation outside app features | Does not become app feature implementation |
| `dj-design` | Opinionated UI/page/component design and implementation surface | Does not produce generic marketing pages by default |
| `dj-write` | Natural writing/polish without adding claims | Does not invent facts or inflate tone |
| `dj-handoff` | Session context compression for transfer | Does not solve remaining work or commit |

## Overlap Assessment

### `dj-review` vs `dj-check`

This is a valid layered overlap. `dj-review` is lightweight read-only diff/PR review. `dj-check` is the canonical delivery quality gate with validation evidence, version conclusion, and finish-work handoff. The `dj-check` frontmatter was corrected to avoid claiming generic review/merge responsibility.

### `dj-audit` vs `dj-check` vs `dj-review`

This is a scope difference, not redundancy. `dj-audit` scans a whole repo or subsystem and only reports. `dj-review` inspects a specific diff. `dj-check` gates delivery for a task and uses audit/review lenses only as sub-checks.

### `dj-hunt` vs `dj-implement`

This is a sequence difference. `dj-hunt` starts with a failing symptom and must establish root cause before code changes. `dj-implement` starts from confirmed requirements or a plan. Bug fixes can flow `dj-hunt -> dj-implement` when the fix is no longer investigative.

### `dj-tdd` vs `dj-implement`

This is a process variant. `dj-tdd` is implementation with a strict test-first loop and one behavior slice. `dj-implement` allows direct implementation when TDD is not appropriate or no test framework exists.

### `dj-prototype` vs `dj-implement` vs `dj-script`

This is lifecycle separation. `dj-prototype` writes disposable code to answer a design question. `dj-script` writes standalone automation outside app features. `dj-implement` modifies product/application behavior.

### `dj-pattern` vs `dj-audit` vs `dj-debt`

This is evidence-source separation. `dj-pattern` discovers repeated code or recurring history. `dj-audit` scans for bloat/security. `dj-debt` only reads explicit shortcut markers.

### `dj-karpathy` vs `dj-ponytail`

Both are overlay lenses, not standalone lifecycle skills. `dj-karpathy` covers broader LLM coding discipline: assumptions, verifiable success, surgical changes, first-principles reasoning. `dj-ponytail` is narrower and stronger: do less, avoid complexity, ladder decisions. They can stack without conflict if the active workflow skill remains primary.

### `dj-output` vs `dj-write`

This is artifact type separation. `dj-output` owns project docs, task artifacts, PRDs, design docs, and code-doc consistency. `dj-write` owns prose quality and natural language style. A project document may route `dj-output -> dj-write` only when content exists and the remaining work is text polish.

### `dj-health` vs `dj-audit`

This is target separation. `dj-health` checks agent/runtime/tooling/config health. `dj-audit` checks application repository code health, bloat, and security.

## Redundancy Verdict

No `dj-*` skill should be deleted or merged at this point.

`dj-karpathy` and `dj-ponytail` are intentionally non-atomic overlays, but they are not redundant because they do not own task lifecycle. Their frontmatter already says they are stackable and must not replace active workflow skills.

## Change Made

Updated `crates/configurator/templates/skills/dj-check/SKILL.md` frontmatter:

- Removed wording that implied `dj-check` performs generic code review and merge actions.
- Reframed it as delivery quality gate, completion verification, release-blocking check, and finish-work readiness review.
- Reduced trigger overlap with `dj-review` by making `check` / quality gate / finish-work readiness primary.

## Recommendation

Keep all current `dj-*` skills. Treat overlap as a routing problem, not a deletion problem. The core rule should be:

- New or ambiguous task -> `dj-dispatch`
- Need clarity -> `dj-grill`
- Need product/app code -> `dj-implement` or `dj-tdd`
- Symptom/failure -> `dj-hunt`
- Need quick diff opinion -> `dj-review`
- Need delivery gate -> `dj-check`
- Need whole-repo scan -> `dj-audit`
- Need docs -> `dj-output`
- Need prose polish -> `dj-write`
- Need context transfer -> `dj-handoff`
- Need minimalism/discipline -> overlay `dj-ponytail` / `dj-karpathy`
