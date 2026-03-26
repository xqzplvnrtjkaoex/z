# Step: Init + Select

## Step 1: Initialize

### 1a: Phase setup via gsd-tools

```bash
INIT=$(node "$HOME/.claude/get-shit-done/bin/gsd-tools.cjs" init phase-op "${PHASE}")
if [[ "$INIT" == @file:* ]]; then INIT=$(cat "${INIT#@file:}"); fi
```

Parse JSON for: `phase_dir`, `phase_number`, `phase_name`, `padded_phase`, `has_context`.

If `phase_found` is false: exit with error.

### 1b: Load phase context

Read these files for locked decisions and phase scope:
- `${phase_dir}/*-CONTEXT.md` -- locked decisions, discretion areas
- `${phase_dir}/*-RESEARCH.md` -- technical patterns if available
- `.planning/ROADMAP.md` -- phase description and requirements; extract phase requirement IDs (REQ-XX) for use in finalize step validator dispatch

### 1c: Resume check

```bash
find ${phase_dir} -maxdepth 1 \( -name "*-CASES.md" -o -name "CASE-SCRATCH.md" \) 2>/dev/null
```

If CASES.md exists:
- Read it and identify already-documented operations
- Ask developer: "Update existing" / "Resume (add more operations)" / "Start fresh"
- Resume mode: present only undocumented operations in Step 2

If only CASE-SCRATCH.md exists (no CASES.md -- interrupted session):
- Read it and identify operations already discussed
- Ask developer: "Resume from scratch file (continue where we left off)" / "Start fresh"
- Resume mode: load scratch data as already-discussed operations, skip to next undiscussed operation
- The scratch file's cases will be included in the final CASES.md without re-discussion

### 1d: Dispatch case-briefer for operation extraction

Dispatch the `case-briefer` agent to extract operations from planning documents:

```
Agent(
  subagent_type: "case-briefer",
  prompt: "<objective>
Analyze planning documents for Phase {phase_number}: {phase_name}.
Extract all operations, constraints, and decision context.
</objective>

<phase_context>
Phase: {phase_number} - {phase_name}
Description: {phase_description from ROADMAP.md}
Locked decisions: {key decisions from CONTEXT.md}
</phase_context>

<files_to_read>
- .planning/ROADMAP.md -- phase description, success criteria, requirement IDs
- {phase_dir}/*-CONTEXT.md -- locked decisions, constraints
- .planning/REQUIREMENTS.md -- requirement ID descriptions
- .planning/PROJECT.md -- architecture reference (service topology, patterns)
</files_to_read>

<output>
Write to: {phase_dir}/CASE-BRIEFING.md
</output>",
  run_in_background: false
)
```

Read the produced `CASE-BRIEFING.md` to prepare for Step 2.

If the briefer returns `BRIEFING FAILED` or `CASE-BRIEFING.md` is not produced, report the error and ask the developer whether to retry, manually define operations, or abort.

### 1e: Initialize CASE-SCRATCH.md with Phase Rules header

If CASE-SCRATCH.md does not already exist (fresh start), create it with a Phase Rules placeholder:

```markdown
# Case Scratch: Phase [XX] - [Name]

## Phase Rules
(populated after briefing review in Step 2.5)

---
```

This ensures Phase Rules have a home from the start. The placeholder is populated during Step 2.5 in step-discuss.md.

---

## Step 2: Select Operations

Present discovered operations grouped by category:

```
I found these operations for Phase [X]: [Name]

[Category]:
  1. OperationA -- [interface from briefing]
  2. OperationB -- [interface from briefing]

[Category]:
  3. OperationC -- [interface from briefing]

Which operations would you like to discuss?
Enter numbers, 'all', or a category name.
```

After selection, reorder for logical discussion flow:
- Dependencies first (create before read/update/delete)
- Simple before complex
- Auth/validation before business logic

If resuming, show already-documented operations marked as `(documented)`.

---

## Transition

When init + select is complete:
-> Read [step-discuss.md](step-discuss.md) and begin per-operation case discovery.

Pass forward to discuss step: `phase_dir`, `padded_phase`, selected operations list, operation order.
