# DJ Cross-Skill Boundary Review

Generated: 2026-07-01T02:13:57+00:00

## Trigger

User observed that direct use of `dj-output` without extra input could be interpreted as a reason to invoke `dj-hunt`. That is a boundary bug: lack of explicit input for a documentation skill should default to documentation sync, not bug investigation.

## Decision

Cross-skill orchestration is workflow responsibility. Atomic `dj-*` skills may:

- report that another type of work is needed;
- mark a follow-up type such as implementation, hunt, docs, or check;
- stop when their own input contract is not satisfiable.

Atomic `dj-*` skills must not:

- switch to another skill by themselves;
- treat missing input as a reason to invoke another skill;
- perform another skill's core work inside their own workflow;
- turn a read-only skill into implementation, repair, or release work.

`dj-dispatch` remains the place where route decisions and skill chains are produced. Canonical workflow remains intact; the change only prevents skill templates from encoding self-directed cross-skill calls.

## Root Cause

Several skill templates used wording such as "route to", "go back to", "call", or "hand to" another skill. In runtime, that language can be interpreted as an execution instruction rather than a follow-up recommendation. This blurs atomic skill boundaries and can make direct invocation surprising.

## Changes

### `dj-output`

`dj-output` now has an explicit default for no extra input:

- direct use with no additional input runs documentation sync mode;
- it reads active task, task artifacts, existing docs/spec, and current diff;
- it updates or fills current task-related documentation;
- it must not treat missing explicit input as a bug investigation signal;
- it only stops for requirement alignment when there is no source material and the user is asking for a new document that would invent product requirements.

### Read-only and gate skills

Updated cross-skill wording in:

- `dj-review`
- `dj-check`
- `dj-audit`
- `dj-pattern`
- `dj-health`
- `dj-debt`
- `dj-prototype`
- `dj-hunt`
- `dj-grill`
- `dj-implement`

The new wording uses "follow-up type", "后续项", "需要对齐", or "workflow/user decides" instead of directly routing or switching.

## Boundary Rule

Use this rule when editing future skills:

```text
Inside a dj-* skill:
- Do the current atomic capability.
- If another capability is needed, report it as follow-up.
- Do not switch skills internally.

Inside workflow / dj-dispatch:
- Decide the next skill.
- Chain skills when the task requires a sequence.
```

## Expected Behavior

Direct `dj-output` invocation with no input now means:

```text
Mode: documentation sync
Inputs: active task + existing artifacts + docs/spec + diff
Action: sync related documentation or report that no document update is needed
No action: no automatic `dj-hunt`, no automatic route switch
```

## Validation Result

- Non-dispatch `dj-*` strong routing scan: `cross-skill strong routing ok`.
- Markdown fence check: `fence ok`.
- Generated `test-prompts.json` parse check: `prompt json ok`.
- Old `dijiang review` reference scan: no matches under `crates`.
- `cargo check -p dijiang`: passed; only pre-existing warnings remain in mem parser structs and unused CLI channel helpers.
- `cargo test -p dijiang --test e2e`: passed, 26 tests.
