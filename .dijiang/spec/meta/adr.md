# Architecture Decision Records

## Purpose

ADR records why an important decision was made. It is not a design document and should not describe the whole current system. DiJiang uses ADRs to keep decision history durable while allowing design docs, specs, and code to evolve.

## Rules

- Write an ADR for decisions that change architecture, workflow, public contracts, long-lived conventions, or default agent behavior.
- Keep one ADR to one decision. If a topic needs multiple independent choices, split it.
- Keep ADR numbers stable forever. Do not rename accepted ADRs to fill gaps.
- Track status explicitly: `proposed`, `accepted`, `rejected`, `deprecated`, or `superseded`.
- Supersede instead of rewriting history. The old ADR points to the new ADR; the new ADR names what it supersedes.
- Record context, decision drivers, considered alternatives, final decision, consequences, and references.
- Let design docs describe current structure. They may cite ADRs for reasons, but should not duplicate ADR history.
- Let implementation plans cite ADRs when a decision constrains the work.

## Template

```markdown
---
id: ADR-0001
title: Use <decision>
date: YYYY-MM-DD
status: proposed
supersedes: []
supersededBy: null
---

# ADR-0001: Use <decision>

## Context

What problem or pressure forced a decision now?

## Decision Drivers

- Driver 1
- Driver 2

## Considered Options

| Option | Pros | Cons | Outcome |
|--------|------|------|---------|
| Option A | ... | ... | accepted/rejected |

## Decision

The chosen option and the reason it wins.

## Consequences

- Positive consequence
- Negative or operational cost
- Follow-up constraint for specs, skills, or code

## References

- Related PRD, design doc, issue, commit, or external source
```

## Examples

Good ADR topics:

- `ADR-0004: Use dj-check as the canonical quality gate`
- `ADR-0008: Store task state under .dijiang/tasks instead of legacy .trellis`
- `ADR-0012: Require task worktrees for code modification`

Poor ADR topics:

- `System architecture overview`
- `How the task crate is implemented`
- `Current CLI command list`

## Anti-patterns

- Writing a large design document and calling it an ADR.
- Editing an accepted ADR until old references no longer explain the original decision.
- Recording what changed without recording the rejected alternatives.
- Using ADRs for small reversible implementation details that belong in code or task notes.
