---
name: state
disable-model-invocation: true
description: Lightweight STATE.md update and commit without HANDOFF/continue-here overhead.
---

# Save State

Lightweight session state save. Updates STATE.md via gsd-tools and commits changed planning files.

## When to Use

- User says "state", "save state", "session end", or similar
- NOT for mid-execution pauses with complex context (use `/gsd:pause-work` instead)

## Process

1. **Read** `.planning/STATE.md` to understand current state
2. **Get timestamp** via gsd-tools:
   ```bash
   TIMESTAMP=$(node "$HOME/.claude/get-shit-done/bin/gsd-tools.cjs" current-timestamp --pick timestamp)
   ```
3. **Update frontmatter** via gsd-tools (these commands update STATE.md directly):
   ```bash
   node "$HOME/.claude/get-shit-done/bin/gsd-tools.cjs" state update stopped_at "<brief description>"
   node "$HOME/.claude/get-shit-done/bin/gsd-tools.cjs" state update last_activity "<summary of session>"
   node "$HOME/.claude/get-shit-done/bin/gsd-tools.cjs" state update last_updated "$TIMESTAMP"
   ```
   - Only update `progress` fields (total_plans, completed_plans, etc.) if phase/plan progress actually changed
4. **Update Session Continuity section** (manual edit — gsd-tools does not manage this section):
   - `Last session:` ISO timestamp from step 2 (e.g., `2026-03-25T11:48:52.678Z`)
   - `Last activity:` same as frontmatter `last_activity`
   - `Stopped at:` same as frontmatter `stopped_at`
   - `Resume file:` most relevant file for next session
   - `Next action:` specific next step
5. **Update Pending Todos** if any were completed or added
6. **Commit** all changed `.planning/` files with message: `docs(state): {brief description}`

## Rules

- Use `gsd-tools state update` for frontmatter — never edit frontmatter YAML manually
- Use `gsd-tools current-timestamp` for all timestamps — never hardcode or approximate
- Session Continuity `Last session:` must use full ISO timestamp (not date-only)
- Only update STATE.md and commit planning files
- Do NOT create HANDOFF.json or .continue-here.md
- Do NOT create WIP commits — use proper conventional commit
- If no planning files changed beyond STATE.md, commit STATE.md alone
