# Verification Loop Guide

## Purpose

A verification loop turns a goal into an observable pass/fail signal. DiJiang work should converge through evidence instead of confidence. Use this guide when a task is vague, AI-generated code needs validation, or a change crosses user-visible behavior.

## Rules

- Define the smallest key proposition before coding: what must be true for the task to be done?
- Choose the cheapest reliable loop that can prove it: unit test, CLI fixture, HTTP call, browser/OCR check, trace replay, or manual checklist.
- Keep the loop close to the user-visible contract. Avoid tests that still pass if the requested behavior is deleted.
- Record expected input, action, output, and evidence. A future agent should be able to rerun or audit it.
- Grow coverage only after the first key proposition is proven.
- When automation is impossible, write an explicit human-verifiable checklist with screenshots, logs, or command output paths.
- After an AI-caused bug, capture why the AI made the mistake and update prompt, skill, spec, or memory.

## Choosing a Loop

| Situation | Preferred loop |
|-----------|----------------|
| Pure function or parser | Unit test with good/base/bad cases |
| CLI behavior | Fixture input plus stdout/stderr/exit-code assertion |
| API behavior | HTTP script or integration test |
| UI behavior | Browser script, screenshot, or OCR assertion |
| Hardware/external runtime | Flash/run script, device log, or reproducible manual checklist |
| Regression across commits | `git bisect run` harness |
| Intermittent behavior | Trace replay, deterministic seed, or logging around the suspected boundary |

## AI Bug Prevention Checklist

- [ ] Did the implementation preserve source facts from the old system?
- [ ] Did any file fail to decode as UTF-8? If yes, retry with the likely legacy encoding before inferring missing text.
- [ ] Did the AI rename fields, labels, or columns because they looked wrong? Require explicit evidence.
- [ ] Does the regression test fail against the buggy behavior and pass against the fix?
- [ ] Was the prevention written back into the durable instruction surface that caused the mistake?

## Wrong vs Correct

### Wrong

The agent changes table labels during migration because the original names look inconsistent, then only verifies that the new UI renders.

### Correct

The agent compares against the source UI/API, notices encoding issues if labels are unreadable, preserves the original labels unless a decision artifact says otherwise, and adds a verification step that checks the migrated labels against the source facts.
