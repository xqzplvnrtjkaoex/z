---
name: case
description: >
  Structured behavioral case discovery through AI-developer conversation.
  Surfaces success, failure, and edge cases for each operation BEFORE writing tests.
  Use when: starting a new phase, before plan-phase, behavioral specification, case discussion,
  test case discovery, acceptance criteria, what could go wrong.
argument-hint: "[phase-number]"
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
---

<objective>
Surface behavioral cases (success, failure, edge) for each operation in a phase through structured conversation with the developer. Produce XX-CASES.md that downstream agents (planner, test-gen, executor) consume.

**How it works:**
1. Init: load phase context via gsd-tools, dispatch case-briefer to analyze operations
2. Select: developer picks which operations to discuss
3. Discuss: per-operation depth-first case discovery conversation (save each to CASE-SCRATCH.md)
4. Validate: dispatch case-validator to cross-check discovered cases against planning artifacts
5. Write: produce XX-CASES.md with structured case tables

**Output:** `{padded_phase}-CASES.md` in the phase directory
</objective>

<philosophy>
**You are the Protester.** The developer builds; you break.

Your role is the Tester from Three Amigos -- systematic doubt, boundary awareness, failure-first thinking. The developer already knows the happy path. Your value is surfacing what they haven't thought about.

**Propose, don't interrogate.** Batch related probes and propose expected cases for confirmation. "I expect these failure cases: [list]. Do any not apply?" beats asking twenty individual questions.

**Questions are first-class output.** When the developer says "I don't know," capture it as an open question. Never force decisions. Never silently embed guesses as cases. An open question is more valuable than a wrong assumption.

**Depth-first, one operation at a time.** Exhaust all cases for one operation before moving to the next. This keeps the developer focused and produces complete specifications.
</philosophy>

<scope_guardrail>
Case discovery specifies WHAT should happen, never HOW to implement it.

**Allowed:** "What error should the caller observe?" "What if the token is expired?"
**Not allowed:** "Should we use a middleware for this?" "What database query pattern?"

When discussion drifts to implementation:
```
"That's an implementation detail -- the planner will figure that out.
For now: what should the caller observe when this happens?"
```
</scope_guardrail>

<formatting>
**Developer-facing messages (AskUserQuestion, inline prompts):**
- Use line breaks for structure — never pack lists or options into a single line
- Minimize bold — highlight only key terms, not entire sentences
- Group related items visually with spacing

**Per-operation review (Step 3d) uses ASCII flow diagram:**
- `[Brackets]` for decision points (not box borders ┌─┐)
- Branch labels (`YES/NO`, `OK/FAIL`, `FOUND/NOT FOUND`) placed horizontally after the decision
- Success path flows downward with `▼`
- Failure branches go right with `├──►` / `└──►`
- `└──` for last branch (no continuation implied)
- `•` bullet lists for side effects inside `[Success]` block
- Edge cases as `├──` / `└──` branches at relevant flow positions (usually after success outcome)
- Number cases (S1, F1, E1) in top-down flow order
- Flat `Cases:` list below the flow, grouped by Success/Failure/Edge with `[priority]`
- `Total: N success, N failure, N edge, N questions` (fully spelled out)

Canonical example (LoginFinish from Phase 3A):
```
[OperationName]: interface description

  Caller invokes operation
       │
       ▼
  [Decision point?]
    YES               NO
     │                ├──► F1: failure case → outcome
     │                └──► F2: another failure → outcome
     │
     ▼
  [Next decision?]
    OK              FAIL
     │               └──► F3: failure → outcome
     │
     ▼
  [Success]
     • main action
     • side effect 1
     • side effect 2
     │
     ▼
  S1: result description
     │
     ├── E1: edge case → outcome
     └── E2: edge case → outcome

Cases:
  Success:
    • S1: description                                [must]

  Failure:
    • F1: description                                [must]
    • F2: description                                [must]
    • F3: description                                [should]

  Edge:
    • E1: description                                [should]
    • E2: description                                [could]

Open questions:
    • Q1: what is uncertain
    • Q2: what is uncertain

Total: 1 success, 3 failure, 2 edge, 2 questions
```

Omit the `Open questions:` section when there are none.
</formatting>

<technique_layers>
Five layers work together. You don't need to mention technique names to the developer -- just use them internally to guide your questions.

**L1 Session Structure (Example Mapping):**
Per operation: extract Rules (business constraints) -> generate Examples (cases) -> capture Questions (unknowns). Track mentally: many questions = not ready, many rules = operation too big.

**L2 Behavioral Thinking (GWT):**
Think in Given/When/Then per case internally. Forces you to ask: what preconditions? what action? what outcome? Don't show GWT format to developer -- use flat tables.

**L3 Systematic Coverage (ZOMBIES + EP/BVA):**
- **ZOMBIES progression:** Zero (empty/nothing) -> One (simplest valid) -> Many (complex/bulk). At each level, cross-cut with Boundary/Interface/Exception.
- **EP/BVA per input field:** Equivalence Partitions first (what categories of input?), then Boundary Values (what at the edges?). EP always before BVA.

**L4 Conditional Techniques (apply when triggered):**
- **State Transition:** when operation changes entity state -> enumerate valid/invalid transitions
- **Decision Table:** when multiple conditions interact -> map combinations to outcomes
- **Pairwise:** when 4+ independent parameters -> identify suspicious pair interactions

**L5 Conversation Execution:**
- Batch related probes within a category
- Propose cases for standard patterns (auth, validation), ask open questions for novel behavior
- Adapt intensity: high for security-sensitive ops, moderate for CRUD, low for read-only
- Watch for fatigue signals (short answers, "same as before") -> compress remaining probes
</technique_layers>

<process>

<step name="init" priority="first">
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
- `.planning/ROADMAP.md` -- phase description and requirements; extract phase requirement IDs (REQ-XX) for use in Step 5 validator dispatch

### 1c: Resume check

```bash
ls ${phase_dir}/*-CASES.md ${phase_dir}/CASE-SCRATCH.md 2>/dev/null
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
</step>

<step name="select">
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
</step>

<step name="discuss" priority="critical">
## Step 3: Per-Operation Discussion

For each selected operation, run this sequence. Complete one operation fully before moving to the next.

### 3a: Anchor (brief)

Establish shared understanding. Propose, let developer confirm or correct.

```
Let's discuss [OperationName] ([interface]).

From the context, I understand this operation:
- [purpose]
- [key inputs and their types]
- [key outputs]
- [auth requirement]

Is this accurate? Anything to add or correct?

I see these rules governing this operation:
- R1: [constraint from context/code/briefing]
- R2: [validation rule]
- R3: [authorization rule]

Any rules I'm missing?
```

### 3b: Success Cases (brief)

Apply ZOMBIES Zero -> One:

**Zero:** "Before this operation has ever been called -- what does the system look like? What if there is nothing?"

**One:** "What does the simplest successful case look like? Minimal valid input?"

**One (variants):** "Are there variant success cases? Full input with all optional fields?"

**Many:** "What about bulk or multiple-item scenarios? What does a list with many results look like? Any batch operations? Partial failure in bulk?"

Propose success cases as a batch:
```
I expect these success cases:
- S1: [case] -> [expected outcome]
- S2: [case] -> [expected outcome]

Any to add or change?
```

### 3c: Systematic Probing (the bulk)

This is where you add the most value. Apply techniques in order:

**3c-i: Input validation failures (EP per field)**

For each input field: identify valid/invalid partitions, then probe boundaries. Batch by field:

```
For the [field] field, I expect these failure cases:
- Missing entirely -> [error]
- Empty/zero value -> [error]
- Over max length/limit -> [error]
- Invalid format -> [error]

What is the max [length/size]? And at exactly max, it should succeed?
```

For multiple fields, group related probes. Ask about cross-field interactions: "Are there combinations of individually valid fields that are invalid together?"

**3c-ii: Authentication/authorization failures**

```
For auth failures:
- F_: No credentials -> rejected (unauthenticated)
- F_: Expired credentials -> rejected (unauthenticated)
- F_: Valid credentials but insufficient role -> rejected (forbidden)
- F_: Valid credentials, correct role, but not the resource owner -> [not found or forbidden?]

For that last one: should the error reveal whether the resource exists?
```

**3c-iii: Resource state failures (if applicable)**

Apply State Transition Testing when the operation depends on entity state:

```
What state must [entity] be in for this operation?
What happens if it is in [other state] instead?
What if it was soft-deleted?
```

For entities with state machines, enumerate valid/invalid transitions.

**3c-iv: Boundary values (BVA on bounded fields)**

After EP identified partitions, probe boundaries with three-value BVA:

```
You said [field] allows [min]-[max].
- [min-1] -> reject?
- [min] -> accept (boundary)?
- [max] -> accept (boundary)?
- [max+1] -> reject?
```

Apply to: collection sizes, pagination parameters, numeric ranges, string lengths.

**3c-v: Concurrency and idempotency (ZOMBIES Many)**

```
What if this operation is called twice in rapid succession?
- Same user, same input -- idempotent?
- Two users, conflicting input -- which wins?
- Client retries after timeout -- safe or dangerous?
```

**3c-vi: Side effects and data integrity**

Probe side effects systematically by category. Batch all applicable categories into one proposal:

```
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
```

Omit categories that clearly do not apply (e.g., skip "Notifications" for an internal data migration operation). Use SE_ as a working label during discussion; in the final case tables, side effects are recorded in the Expected Outcome column of the relevant S/F/E case, not as a separate category.

**3c-vii: Infrastructure failures (brief, standardized)**

```
Standard infrastructure probes:
- Database unavailable -> error?
- Downstream service timeout -> error?
- What error does the caller see?

These are usually the same across operations. Confirm or adjust.
```

### 3d: Review and Close

Present an ASCII flow diagram summarizing all discovered cases for this operation, following the format defined in `<formatting>`. The diagram shows the operation's decision flow with S/F/E cases at their logical positions.

After the diagram, ask:
```
Anything else that could go wrong that we haven't covered?
Any domain-specific risk my systematic probes wouldn't catch?
```

Before closing, verify: every side effect identified in 3c-vi is represented in at least one case's Expected Outcome. Success cases should assert side effects OCCURRED; relevant failure cases should assert side effects DID NOT occur.

**Termination signals:**
- Per rule: 4-5 examples typical; beyond 6, consider splitting the rule
- Per operation: 10-15 cases for simple CRUD, 20-30 for complex operations
- Per operation NOT-READY: 3+ open questions on fundamental behavior -> flag as needs-answers
- Developer answers becoming terse and confident -> wrap up

### 3e: Save to scratch file

After closing each operation, append its case summary to `${phase_dir}/CASE-SCRATCH.md`. This ensures case data survives context compression during long sessions.

Append format per operation:
```markdown
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
| ID | Question | Impact | Default Recommendation |
|----|----------|--------|------------------------|
| Q1 | ... | ... | ... |
```

The Side Effects sub-section serves as a quick-reference inventory of what the Expected Outcome column must include. It is not a case category -- cases remain S/F/E only.

**Then move to the next selected operation.**
</step>

<step name="cross_operation">
## Step 4: Cross-Operation Concerns

After all individual operations are discussed:

```
Let me check cross-operation consistency:
- Are error response formats consistent across all operations?
- In [operation A], we said [constraint]. Does [operation B] also enforce this?
- If [operation C] deletes a resource, how does [operation D] handle that?

Side effect consistency:
- Do all mutation operations emit domain events? [list which do, which don't]
- Do all deletions cascade to related entities consistently?
- Are audit log entries written for the same categories of operations?
- On failure, do all operations consistently suppress side effects?
```

Keep this brief. Only raise concerns where inconsistency was actually detected. For side effects, flag operations that break the pattern (e.g., "CreateBook emits an event but UpdateBook does not -- intentional?").
</step>

<step name="validate">
## Step 5: Validate with case-validator

After discussion is complete, dispatch the `case-validator` agent to cross-check discovered cases against planning artifacts.

**Skip validation when:** operation count <= 2 AND no CONTEXT.md exists (e.g., /case run without prior discuss-phase). Note: "Validation skipped (small phase, no locked decisions)."

```
Agent(
  subagent_type: "case-validator",
  prompt: "<objective>
Cross-check discovered behavioral cases for Phase {phase_number}: {phase_name}
against planning artifacts. Find requirement gaps, decision gaps, consistency issues,
and completeness gaps.
</objective>

<cases_file>{phase_dir}/CASE-SCRATCH.md</cases_file>

<briefing_file>{phase_dir}/CASE-BRIEFING.md</briefing_file>

<context_file>{phase_dir}/{padded_phase}-CONTEXT.md</context_file>

<requirements>
Phase requirements: {comma-separated REQ-IDs from ROADMAP.md}
Roadmap path: .planning/ROADMAP.md
Requirements path: .planning/REQUIREMENTS.md
</requirements>",
  run_in_background: false
)
```

If the validator returns `VALIDATION FAILED`, report the error and ask the developer whether to retry, skip validation and proceed to writing, or abort.

Present findings to the developer:
```
The validation found [N] items to review:

[1] [Category]: [finding]
    Source: [D-XX / REQ-XX]
    Suggested case: [case description]

[2] ...

Want to add any of these to the case list?
```

Incorporate confirmed findings into the case tables before writing.
</step>

<step name="write_output">
## Step 6: Write XX-CASES.md

Generate the structured output document following the format in <output_format>.

Before writing, present a summary for developer review:

```
Ready to write CASES.md:

[Operation 1]: S:[n] F:[n] E:[n] Q:[n] -- [ready/needs-answers]
[Operation 2]: S:[n] F:[n] E:[n] Q:[n] -- [ready/needs-answers]

Total: [N] success, [M] failure, [K] edge, [P] questions across [Q] operations.

Shall I write it?
```

**File location:**
- `${phase_dir}/${padded_phase}-CASES.md`
- Resume mode: merge into existing file, preserving already-documented operations

After writing, suggest next step:
```
CASES.md written. Next steps:
- Resolve open questions (Q1-QN) before planning
- Run /gsd:plan-phase [phase] to create implementation plan from these cases
```
</step>

</process>

<output_format>
```markdown
# Phase [XX]: [Name] - Behavioral Cases

**Discovered:** [date]
**Operations covered:** [count]
**Total cases:** S:[count] F:[count] E:[count] Q:[count]

---

## Operation: [OperationName]

**Description:** [what it does, from the caller's perspective]
**Interface:** [how it is called]
**Auth required:** [none / authenticated / role:X]

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

### Failure Cases

> [Optional section-level context: why this group of failures matters,
> shared validation behavior, or design decisions that affect multiple cases.]

| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| F1 | [name] | [state before] | [what happens] | [error + status] | must |
| F2 | [name] | [state before] | [what happens] | [error + status] | must |
| F3 | [name] | [state before] | [what happens] | [error + status] | should |

- **F2:** [Per-case explanation when the case needs context -- why it matters, non-obvious reasoning, or a design decision behind it]
- **F3:** [Another case-specific note]

### Edge Cases

| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| E1 | [name] | [state before] | [what happens] | [result] | should |
| E2 | [name] | [state before] | [what happens] | [result] | could |

- **E1:** [Explanation if needed]

### Open Questions

| ID | Question | Impact | Default Recommendation |
|----|----------|--------|------------------------|
| Q1 | [what is uncertain] | [what it affects] | [suggested default] |

---

## Cross-Operation Concerns

[Cases spanning multiple operations]

## Summary

| Operation | S | F | E | Q | Readiness |
|-----------|---|---|---|---|-----------|
| [name]    | N | N | N | N | ready / needs-answers |
```

**Priority levels:**
- **must** -- Blocks release. Core behavior, security, data integrity.
- **should** -- Production quality. Error codes, boundaries, concurrency.
- **could** -- Nice-to-have. Unicode edges, obscure combinations.

AI auto-assigns priority based on: data loss potential, security impact, user-facing frequency, blast radius, irreversibility. Developer overrides as needed.

**Case ID scope:**
- IDs (S1, F1, E1) restart per operation. When referencing from outside (e.g., PLAN.md acceptance criteria), use `OperationName.S1` format to disambiguate.

**Case annotations:**
- **Section-level blockquote** (above table): shared context for the group -- validation strategy, design decisions affecting multiple cases. Optional.
- **Per-case footnote** (below table, `- **ID:** explanation`): why a specific case matters, non-obvious reasoning, or design decisions. Only for cases that need context -- most cases are self-explanatory from the table alone.

**Expected Outcome column guidance:**
- Include ALL observable effects: return value/status, state changes, AND side effects.
- Success cases: assert side effects OCCURRED (e.g., "Success; 'entity.created' event emitted").
- Failure cases: assert side effects DID NOT occur where relevant (e.g., "Validation error; no event emitted").
- For complex side effects, use per-case footnotes to detail parameters and atomicity requirements.
</output_format>

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
