---
name: dijiang-continue
description: "Resume work on the current task: find active task and phase, load artifacts, route to the appropriate dj-* skill."
triggers:
  - session:start
---

# Continue Session

Resume work on the current DiJiang task.

## Steps

1. **Load state**:
   ```bash
   dijiang status
   ```

2. **Find active task and phase** from the output.

3. **Read task artifacts**: `prd.md`, `design.md` (if present), `implement.md` (if present).

4. **Read journal** at `.trellis/workspace/<developer>/` for context from prior sessions.

5. **Route to the phase-appropriate dj-* skill**:

   | Phase | Skill |
   |---|---|
   | requirements alignment | `dj-grill` |
   | document creation | `dj-output` |
   | implementation | `dj-implement` or `dj-tdd` |
   | investigation / debugging | `dj-hunt` |
   | review / verification | `dj-check` |

   If no active task exists, load `dj-dispatch` to classify the request.
