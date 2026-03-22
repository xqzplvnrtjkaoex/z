---
name: state
disable-model-invocation: true
description: Lightweight STATE.md update and commit without HANDOFF/continue-here overhead.
---

# Save State

Lightweight session state save. Updates STATE.md and commits changed planning files.

## When to Use

- User says "state", "save state", "session end", or similar
- NOT for mid-execution pauses with complex context (use `/gsd:pause-work` instead)

## Process

1. **Read** `.planning/STATE.md`
2. **Update Session Continuity section:**
   - `Last session:` current date
   - `Last activity:` summary of what was done this session
   - `Stopped at:` current position description
   - `Resume file:` most relevant file for next session
   - `Next action:` specific next step
3. **Update frontmatter** if phase/plan progress changed:
   - `stopped_at:` brief description
   - `last_updated:` current ISO timestamp
4. **Update Pending Todos** if any were completed or added
5. **Commit** all changed `.planning/` files with message: `docs(planning): update state — {brief description}`

## Rules

- Only update STATE.md and commit planning files
- Do NOT create HANDOFF.json or .continue-here.md
- Do NOT create WIP commits — use proper conventional commit
- If no planning files changed beyond STATE.md, commit STATE.md alone
