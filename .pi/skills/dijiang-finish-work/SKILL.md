---
name: dijiang-finish-work
description: "Wrap up the current session: verify quality with dj-check, write journal entry, remind to commit."
triggers:
  - session:start
---

# Finish Work

Wrap up the current DiJiang session.

## Steps

1. **Verify quality**: Run `dj-check` if code was written/changed.

2. **Review changes**:
   ```bash
   git diff --stat HEAD
   git status
   ```

3. **Update task status** if needed:
   ```bash
   dijiang task status <name> completed
   ```

4. **Write journal entry** at `.trellis/workspace/<developer>/`:
   - What was accomplished
   - Key decisions made
   - Remaining work / next steps

5. **Update specs** if learned anything new — use `dj-output` for spec documents.

6. **Remind user** to commit with descriptive message.
