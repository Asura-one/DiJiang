# 2026-06-29 DiJiang Independence Refactor

## What Was Accomplished

Completed the full migration of DiJiang from Trellis dependency to independent operation:

### Data Layer
- Migrated from `.trellis/` to `.dijiang/` for all project data
- `find_trellis_dir()` → `find_dijiang_dir()` with backward-compatible fallback
- Updated `config.toml` paths to point to `.dijiang/` subdirectories

### Skill Layer
- Created `dj_skills.rs` module with 19 embedded dj-* skill templates
- Skills written to `.pi/skills/dj-*/` during `dijiang init`
- Global template directory at `~/.dijiang/skills/`
- Rebranded `trellis-meta` → `dijiang-meta`

### CLI
- Added `dijiang skills` command (list/sync)
- Added `dijiang migrate` command for legacy `.trellis/` projects
- All output messages updated to reference `.dijiang/`

## Key Decisions

1. **Backward compatibility preserved**: `find_dijiang_dir()` falls back to `.trellis/` for existing projects
2. **Skills embedded in binary**: 19 dj-* skills are compile-time resources, not runtime dependencies
3. **Global template directory**: Skills live at `~/.dijiang/skills/` and are copied to projects during init
4. **trellis-meta rebranded**: Kept functionality but renamed to dijiang-meta

## Files Changed

- 84 files changed in commit `b420eb5`
- +3211/-4527 lines
- 19 new skill templates in `crates/configurator/templates/skills/`

## Remaining Work

- Test `dijiang init` on a fresh project to verify skill writing
- Test `dijiang migrate` on an existing Trellis project
- Consider adding `dijiang skill update` command for updating global templates
- Update documentation and examples
