---
name: dijiang-finish-work
description: "Wrap up the current session: verify quality with dj-check, write journal entry, remind to commit."
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

4. **Write journal entry** at `.dijiang/workspace//`:
   - What was accomplished
   - Key decisions made
   - Remaining work / next steps

5. **Update specs** if learned anything new — use `dj-output` for spec documents.

6. **Write learnings**: Capture what was learned for cross-session recall:
   ```bash
   dijiang mem findings --finding "<key decisions and learnings>"
   dijiang mem learn --lesson "<specific lesson learned>"
   dijiang mem archive
   ```

7. **Record outcomes**: Track what worked and what didn't:
   ```bash
   dijiang mem record --tactic "<tactic-name>" --outcome "success" --context "<what happened>"
   dijiang mem record --tactic "<tactic-name>" --outcome "failure" --context "<what happened>"
   ```

8. **Evolve**: Analyze session and update tactics:
   ```bash
   dijiang mem evolve
   ```

9. **Backup**: Save project memory to global:
   ```bash
   dijiang mem backup
   ```

7. **Remind user** to commit with descriptive message.
