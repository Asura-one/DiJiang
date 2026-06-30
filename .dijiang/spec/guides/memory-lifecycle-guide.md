# Memory Lifecycle Guide

## Purpose

DiJiang memory is not a notebook. It is a controlled learning loop that helps future agents understand tasks, reuse proven tactics, and avoid repeated mistakes without letting stale or unverified claims pollute later work.

## Memory Layers

| Layer | What It Stores | When To Write |
|-------|----------------|---------------|
| Immediate | Current task, constraints, user preferences, active context | Session start, task routing, major scope changes |
| Working | Findings, hypotheses, decisions, blockers, verification evidence | During implementation, debugging, and review |
| Durable | Stable rules, reusable tactics, architectural decisions, repeated user preferences | Finish-work, after bugs, after repeated success |
| Offline consolidation | Condensed lessons from raw sessions, conflicts, and aging knowledge | Periodic maintenance, handoff, or explicit research tasks |

## Rules

- Write memory only when it can influence future action. Raw transcripts, temporary guesses, and one-off details belong in task artifacts, not durable memory.
- Every durable memory item needs source, scope, confidence, and freshness. If those cannot be named, keep it as a task note.
- Prefer structured records: entity, event, cause, decision, consequence, and verification. Facts without relationships are hard to recall correctly.
- Record what not to remember. Outdated assumptions, failed tactics, and rejected hypotheses should be retired instead of silently kept.
- Resolve conflicts before injection. When two memories disagree, surface both with source and date; do not pick the convenient one without evidence.
- Promote lessons from repeated signals, not single anecdotes. One failure can create a caution; repeated failures can become a rule.
- Use memory to change behavior: routing, checklist choice, validation strength, source lookup, or implementation strategy.

## Memory Quality Gate

Before writing durable memory, check:

- [ ] Source: Where did this come from?
- [ ] Scope: Which project, package, workflow, user, or task type does it apply to?
- [ ] Confidence: Is it observed, inferred, or user-stated?
- [ ] Freshness: When should this be reviewed or deleted?
- [ ] Conflict: Does it contradict existing spec, ADR, task artifact, or memory?
- [ ] Actionability: What future decision should change because of it?

If an item fails actionability, keep it out of durable memory.

## Consolidation Loop

1. Capture raw evidence in the task: commands, outputs, links, findings, decisions.
2. Distill the stable lesson: what should happen differently next time?
3. Classify the destination: task artifact, spec, skill, ADR, or memory.
4. Gate the lesson with source/scope/confidence/freshness/conflict/actionability.
5. Inject only the relevant memory in future tasks; do not flood the agent with unrelated history.
6. Review old memory when it causes a wrong action, conflicts with code, or no longer matches the project.

## Anti-patterns

- Treating memory as a dump folder for every conversation.
- Writing preferences or tactics without scope, then applying them globally.
- Trusting memory over current code, current docs, or user correction.
- Keeping failed hypotheses because they were eloquent.
- Adding memory after every task without asking whether it will change future behavior.
