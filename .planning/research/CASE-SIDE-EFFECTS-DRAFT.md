# /case Skill: Side Effect Enhancements and Minor Fixes -- Draft Changes

**Drafted:** 2026-03-24
**Target file:** `.claude/commands/case.md`
**Based on:** CASE-CATEGORY-COMPLETENESS.md Section 4 (Side Effect Problem: Solved Without SE) and Section 6 (Recommendation)
**Confidence:** HIGH -- all changes are text edits to a skill file; no external dependencies to verify.

---

## Side Effect Enhancement Changes (6)

---

### Change 1 -- Step 3c-vi Expansion: Systematic Category-Based Side Effect Probing

**Location:** Lines 289-297, inside `<step name="discuss">`, sub-step 3c-vi

**Current text:**
```
**3c-vi: Side effects and data integrity**

\`\`\`
What other state changes when this operation succeeds?
- Events emitted?
- Related entities updated?
- Cache invalidated?
What if the operation partially succeeds then fails?
\`\`\`
```

**Replacement text:**
```
**3c-vi: Side effects and data integrity**

Probe side effects systematically by category. Batch all applicable categories into one proposal:

\`\`\`
When this operation succeeds, I expect these side effects:

Domain events:
- SE_: "[entity].[action]" event emitted with [key fields]

Related entity updates:
- SE_: [related entity] [created/updated/deleted] as consequence

Cache mutations:
- SE_: [cache key/region] invalidated

Audit / logging:
- SE_: Audit log entry with [action, actor, target]

Notifications:
- SE_: [notification type] sent to [recipient]

External system calls:
- SE_: [system] called with [payload summary]

Which of these apply? Any I'm missing?
What if the operation partially succeeds then fails -- are side effects rolled back or left in place?
\`\`\`

Omit categories that clearly do not apply (e.g., skip "Notifications" for an internal data migration operation). Use SE_ as a working label during discussion; in the final case tables, side effects are recorded in the Expected Outcome column of the relevant S/F/E case, not as a separate category.
```

**Rationale:** The current probe is four bullet points that can be answered with "yes/no" without surfacing specifics. The replacement proposes concrete side effects for the developer to confirm or correct, consistent with the skill's "propose, don't interrogate" philosophy. The category list comes from the completeness research's recommendation (Section 6: domain events, cache mutations, related entity updates, audit/logging, notifications, external system calls).

---

### Change 2 -- Step 3d Review Enhancement: Side Effect Capture Verification

**Location:** Lines 310-324, inside `<step name="discuss">`, sub-step 3d

**Current text:**
```
### 3d: Review and Close

\`\`\`
Here is what we have for [OperationName]:

Rules: N confirmed
Success cases: M
Failure cases: K
Edge cases: J
Open questions: P
  Q1: [question]

Anything else that could go wrong that we haven't covered?
Any domain-specific risk my systematic probes wouldn't catch?
\`\`\`
```

**Replacement text:**
```
### 3d: Review and Close

\`\`\`
Here is what we have for [OperationName]:

Rules: N confirmed
Success cases: M
Failure cases: K
Edge cases: J
Side effects identified: N (all reflected in Expected Outcome column)
Open questions: P
  Q1: [question]

Anything else that could go wrong that we haven't covered?
Any domain-specific risk my systematic probes wouldn't catch?
\`\`\`

Before closing, verify: every side effect identified in 3c-vi is represented in at least one case's Expected Outcome. Success cases should assert side effects OCCURRED; relevant failure cases should assert side effects DID NOT occur.
```

**Rationale:** Makes side effect capture an explicit checkpoint in the review. Without this, side effects discovered in 3c-vi can be discussed but never recorded in the case tables. The verification instruction ensures side effects flow into the output.

---

### Change 3 -- Step 3e CASE-SCRATCH Format: Optional Side Effects Sub-Section

**Location:** Lines 332-353, inside `<step name="discuss">`, sub-step 3e

**Current text:**
```
### 3e: Save to scratch file

After closing each operation, append its case summary to `${phase_dir}/CASE-SCRATCH.md`. This ensures case data survives context compression during long sessions.

Append format per operation:
\`\`\`markdown
## Operation: [OperationName]

### Rules
- R1: [rule]

### Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | ... | ... | ... | ... | must |
| F1 | ... | ... | ... | ... | must |

### Open Questions
| ID | Question | Impact |
|----|----------|--------|
| Q1 | ... | ... |
\`\`\`
```

**Replacement text:**
```
### 3e: Save to scratch file

After closing each operation, append its case summary to `${phase_dir}/CASE-SCRATCH.md`. This ensures case data survives context compression during long sessions.

Append format per operation:
\`\`\`markdown
## Operation: [OperationName]

### Rules
- R1: [rule]

### Side Effects
> Optional. Include only when side effects were identified in 3c-vi.

- Domain event: "[entity].[action]" with [key fields]
- [other side effects by category]

### Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | ... | ... | ... | ... | must |
| F1 | ... | ... | ... | ... | must |

### Open Questions
| ID | Question | Impact |
|----|----------|--------|
| Q1 | ... | ... |
\`\`\`

The Side Effects sub-section serves as a quick-reference inventory of what the Expected Outcome column must include. It is not a case category -- cases remain S/F/E only.
```

**Rationale:** Provides a structured place to record discovered side effects before they are distributed into individual case rows. During long sessions with context compression, this sub-section ensures side effects are not lost between the 3c-vi discussion and the final output generation.

---

### Change 4 -- output_format Enhancement: Side Effects Sub-Section and Outcome Column Guidance

**Location:** Lines 447-528, inside `<output_format>`

**Current text (relevant portion, lines 462-474):**
```
### Rules

- R1: [business rule or constraint]
- R2: [validation rule]
- R3: [authorization rule]

### Success Cases

| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | [name] | [state before] | [what happens] | [result] | must |
| S2 | [name] | [state before] | [what happens] | [result] | should |
```

**Replacement text:**
```
### Rules

- R1: [business rule or constraint]
- R2: [validation rule]
- R3: [authorization rule]

### Side Effects
> Optional. Include only when the operation has side effects beyond the primary response.

- Domain event: "[entity].[action]" emitted on success
- [category]: [description]

### Success Cases

| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | [name] | [state before] | [what happens] | [result; side effects] | must |
| S2 | [name] | [state before] | [what happens] | [result] | should |
```

Additionally, append the following guidance after the existing "Case annotations" paragraph (after line 527):

**Current text (lines 525-528):**
```
**Case annotations:**
- **Section-level blockquote** (above table): shared context for the group -- validation strategy, design decisions affecting multiple cases. Optional.
- **Per-case footnote** (below table, `- **ID:** explanation`): why a specific case matters, non-obvious reasoning, or design decisions. Only for cases that need context -- most cases are self-explanatory from the table alone.
</output_format>
```

**Replacement text:**
```
**Case annotations:**
- **Section-level blockquote** (above table): shared context for the group -- validation strategy, design decisions affecting multiple cases. Optional.
- **Per-case footnote** (below table, `- **ID:** explanation`): why a specific case matters, non-obvious reasoning, or design decisions. Only for cases that need context -- most cases are self-explanatory from the table alone.

**Expected Outcome column guidance:**
- Include ALL observable effects: return value/status, state changes, AND side effects.
- Success cases: assert side effects OCCURRED (e.g., "201 Created; 'book.created' event emitted").
- Failure cases: assert side effects DID NOT occur where relevant (e.g., "400 Bad Request; no event emitted").
- For complex side effects, use per-case footnotes to detail parameters and atomicity requirements.
</output_format>
```

**Rationale:** Two sub-changes here. (1) The Side Effects sub-section between Rules and Success Cases serves as an inventory -- a quick reference for which side effects exist so readers can verify they appear in the Expected Outcome column. (2) The Outcome column guidance makes explicit what the completeness research identified as the root cause of side effect loss: Expected Outcome tends to capture only the response, not consequences. This guidance prevents that.

---

### Change 5 -- Step 4 Cross-Operation Enhancement: Side Effect Consistency Probes

**Location:** Lines 358-371, inside `<step name="cross_operation">`

**Current text:**
```
<step name="cross_operation">
## Step 4: Cross-Operation Concerns

After all individual operations are discussed:

\`\`\`
Let me check cross-operation consistency:
- Are error response formats consistent across all operations?
- In [operation A], we said [constraint]. Does [operation B] also enforce this?
- If [operation C] deletes a resource, how does [operation D] handle that?
\`\`\`

Keep this brief. Only raise concerns where inconsistency was actually detected.
</step>
```

**Replacement text:**
```
<step name="cross_operation">
## Step 4: Cross-Operation Concerns

After all individual operations are discussed:

\`\`\`
Let me check cross-operation consistency:
- Are error response formats consistent across all operations?
- In [operation A], we said [constraint]. Does [operation B] also enforce this?
- If [operation C] deletes a resource, how does [operation D] handle that?

Side effect consistency:
- Do all mutation operations emit domain events? [list which do, which don't]
- Do all deletions cascade to related entities consistently?
- Are audit log entries written for the same categories of operations?
- On failure, do all operations consistently suppress side effects?
\`\`\`

Keep this brief. Only raise concerns where inconsistency was actually detected. For side effects, flag operations that break the pattern (e.g., "CreateBook emits an event but UpdateBook does not -- intentional?").
</step>
```

**Rationale:** The completeness research (Section 4.2, point 3) recommends: "Flagged in cross-operation concerns. 'All mutation operations emit domain events. Verify: CreateBook (S1), UpdateBook (S1), DeleteBook (S1).'" This change implements that recommendation by adding side-effect-specific consistency probes alongside the existing general consistency probes.

---

### Change 6 -- success_criteria Enhancement: Side Effect Coverage Criterion

**Location:** Lines 530-539, inside `<success_criteria>`

**Current text:**
```
<success_criteria>
- Operations extracted from phase context (or ad-hoc description)
- Developer selected which operations to discuss
- Each selected operation discussed depth-first: anchor, success, systematic probing, review
- Cases organized as S/F/E with priority levels
- Open questions captured (not glossed over or guessed)
- Cross-operation consistency checked
- XX-CASES.md written and confirmed by developer
- Next step (plan-phase or resolve questions) communicated
</success_criteria>
```

**Replacement text:**
```
<success_criteria>
- Operations extracted from phase context (or ad-hoc description)
- Developer selected which operations to discuss
- Each selected operation discussed depth-first: anchor, success, systematic probing, review
- Cases organized as S/F/E with priority levels
- Side effects reflected in Expected Outcome for all relevant cases (success: occurred; failure: did not occur)
- Open questions captured (not glossed over or guessed)
- Cross-operation consistency checked (including side effect consistency)
- XX-CASES.md written and confirmed by developer
- Next step (plan-phase or resolve questions) communicated
</success_criteria>
```

**Rationale:** Without an explicit success criterion, side effect coverage is aspirational. This ensures the skill self-checks that side effects made it from the 3c-vi discussion into the final case tables.

---

## Minor Fixes (3)

---

### Fix 1 -- `<files_to_read>` Guidance in Briefer Dispatch

**Location:** Lines 116-143, inside `<step name="init">`, sub-step 1d

**Current text (lines 134-136):**
```
<files_to_read>
{relevant source paths with annotations, derived from phase scope}
</files_to_read>
```

**Replacement text:**
```
<files_to_read>
{Construct from phase scope. Include paths that contain the operations this phase implements or modifies.

Derivation rules:
- Proto/interface files: proto/*.proto or OpenAPI specs that define the phase's API surface
- Service handlers: services/{service}/src/app/ -- tonic handlers or route handlers
- Domain types: services/{service}/src/domain/ -- entities, ports, errors
- Use cases: services/{service}/src/usecase/ -- business logic
- Adapters: services/{service}/src/adapter/ -- concrete implementations
- Existing tests: services/{service}/tests/ -- integration test files

Annotate each path with its purpose:
  services/auth/src/app/handler.rs  # tonic gRPC handlers (operation interfaces)
  services/auth/src/domain/mod.rs   # domain types, ports, errors
  proto/auth.proto                  # proto definitions (canonical interface)

For new phases where code does not exist yet: include the proto/interface files and any existing skeleton files. The briefer will note "not yet implemented" for missing code.}
</files_to_read>
```

**Rationale:** The current placeholder is too vague to be actionable. The /case orchestrator must construct this annotation at dispatch time, but "relevant source paths derived from phase scope" gives no guidance on what to include or how to annotate. The replacement provides concrete derivation rules tied to the project's 4-layer architecture and proto workflow, plus an annotation format. This is especially important because the same `<files_to_read>` block is reused for the case-validator dispatch (line 393-394 says "same as briefer dispatch").

---

### Fix 2 -- Resume Check for CASE-SCRATCH.md

**Location:** Lines 105-114, inside `<step name="init">`, sub-step 1c

**Current text:**
```
### 1c: Resume check

\`\`\`bash
ls ${phase_dir}/*-CASES.md 2>/dev/null
\`\`\`

If CASES.md exists:
- Read it and identify already-documented operations
- Ask developer: "Update existing" / "Resume (add more operations)" / "Start fresh"
- Resume mode: present only undocumented operations in Step 2
```

**Replacement text:**
```
### 1c: Resume check

\`\`\`bash
ls ${phase_dir}/*-CASES.md ${phase_dir}/CASE-SCRATCH.md 2>/dev/null
\`\`\`

If CASES.md exists:
- Read it and identify already-documented operations
- Ask developer: "Update existing" / "Resume (add more operations)" / "Start fresh"
- Resume mode: present only undocumented operations in Step 2

If only CASE-SCRATCH.md exists (no CASES.md -- interrupted session):
- Read it and identify operations already discussed
- Ask developer: "Resume from scratch file (continue where we left off)" / "Start fresh"
- Resume mode: load scratch data as already-discussed operations, skip to next undiscussed operation
- The scratch file's cases will be included in the final CASES.md without re-discussion
```

**Rationale:** CASE-SCRATCH.md is written incrementally during Step 3e as each operation completes. If a session is interrupted (context limit, crash, user stops), CASE-SCRATCH.md may contain completed operations while CASES.md does not exist yet (it is written in Step 6). The current resume check only looks for CASES.md, so a leftover CASE-SCRATCH.md is silently ignored and all work is repeated.

---

### Fix 3 -- allowed-tools Update

**Location:** Lines 9-16, in the YAML front matter

**Current text:**
```
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - Grep
  - AskUserQuestion
  - Agent
```

**Replacement text:**
```
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - Grep
  - AskUserQuestion
  - Agent
  - TaskCreate
  - TaskUpdate
```

**Rationale:** TaskCreate and TaskUpdate enable optional progress tracking during long case discovery sessions (e.g., creating a task per operation, updating status as each completes). These tools are non-disruptive -- they do nothing if the task system is not in use. Adding them keeps the option open without requiring changes to the process steps.

---

## Application Order

These changes are independent and can be applied in any order. However, for readability during review, the recommended application order is:

1. Fix 3 (front matter -- smallest, most isolated)
2. Fix 2 (Step 1c -- early in the process flow)
3. Fix 1 (Step 1d -- early in the process flow)
4. Change 1 (Step 3c-vi -- core enhancement)
5. Change 2 (Step 3d -- depends on Change 1 being understood)
6. Change 3 (Step 3e -- scratch format matches discussion output)
7. Change 4 (output_format -- final output matches scratch format)
8. Change 5 (Step 4 -- cross-operation, after individual operations)
9. Change 6 (success_criteria -- verification of all above)

---

## Summary

| # | Type | Location | What It Does |
|---|------|----------|-------------|
| C1 | Enhancement | Step 3c-vi | Replaces vague 4-bullet probe with systematic category-based batch proposal |
| C2 | Enhancement | Step 3d | Adds side effect count and capture verification to review template |
| C3 | Enhancement | Step 3e | Adds optional Side Effects sub-section to CASE-SCRATCH format |
| C4 | Enhancement | output_format | Adds Side Effects sub-section to CASES.md format + Expected Outcome column guidance |
| C5 | Enhancement | Step 4 | Adds side effect consistency probes to cross-operation checks |
| C6 | Enhancement | success_criteria | Adds side effect coverage as an explicit success criterion |
| F1 | Fix | Step 1d | Provides concrete derivation rules for files_to_read annotation |
| F2 | Fix | Step 1c | Handles leftover CASE-SCRATCH.md from interrupted sessions |
| F3 | Fix | Front matter | Adds TaskCreate and TaskUpdate to allowed-tools |
