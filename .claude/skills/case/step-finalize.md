# Step: Finalize (Cross-Operation + Validate + Write)

## Step 4: Cross-Operation Concerns

After all individual operations are discussed, check consistency one category at a time. Present each category's findings before moving to the next.

### 4a: Error response consistency

Build a comparison table categorizing each operation's errors by type. Column headers are not fixed -- derive error categories from the actual cases discussed (e.g., Auth, Validation, Infra, Ceremony, Conflict, NotFound, etc.):

```
| Operation | [Category A] | [Category B] | ... | Infra |
|-----------|-------------|-------------|-----|-------|
| OpA       | 400         | 401         | ... | 500   |
| OpB       | --          | 401         | ... | 500   |
```

Flag rows where the same error category has different status codes across operations. Present inconsistencies to developer for resolution.

### 4b: Cross-phase consistency

Compare discovered cases against existing operations from previous phases (read existing code/cases if available). Flag divergences in error formats, auth patterns, or response conventions between new and existing operations.

### 4c: Additional concerns

Context-dependent checks -- apply based on what was discovered:
- Side effect consistency (do all mutations emit events? do failures suppress side effects?)
- Naming and response format consistency
- Resource lifecycle interactions (if one operation deletes, how do others handle that?)

Keep each category brief. Only raise concerns where inconsistency was actually detected.

---

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

Present findings to the developer, distinguishing Decision Gaps from Constraint Forwarding Gaps:
```
The validation found [N] items to review:

[1] Decision Gap: D-XX in [OperationName]
    "[decision text]" — no failure case for [scenario]
    Suggested case: F_ [description] -> [expected outcome]
    -> Add this case?

[2] Constraint Forwarding: D-XX (Phase-wide)
    "[constraint text]" — not documented as Phase Rule or operation Rule
    Suggested action: Add as PR[N] in Phase Rules
    -> Add this rule?

[3] ...

Want to add any of these?
```

Incorporate confirmed findings into the case tables before writing.

---

## Step 6: Write XX-CASES.md

Generate the structured output document following the output format below.

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

---

<output_format>
## Output Format

```markdown
# Phase [XX]: [Name] - Behavioral Cases

**Discovered:** [date]
**Operations covered:** [count]
**Total cases:** S:[count] F:[count] E:[count] Q:[count]

---

## Phase Rules

> Constraints that apply to ALL operations in this phase.
> Referenced by ID (PR1, PR2...) in operation Rules sections.

- PR1: [constraint description] (D-XX)
- PR2: [constraint description] (D-XX, D-YY)
- PR3: [constraint description]. Per SR-01: [system rule it derives from]

**System Rules (from PROJECT.md) applicable to this phase:**
- SR-01: [brief description] — [which operations]
- SR-02: [brief description] — [which operations]

---

## Operation: [OperationName]

**Description:** [what it does, from the caller's perspective]
**Interface:** [how it is called]
**Auth required:** [none / authenticated / role:X]

### Rules

- R1: [business rule or constraint]
- R2: [validation rule]
- R3: [authorization rule]
- Inherits: PR1, PR2, SR-01

### Side Effects

- Domain event: "[entity].[action]" emitted on success
- [category]: [description]
> If the operation has no side effects, write: `None (read-only)` or `None (query operation)`.

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

## SR Candidates

> Constraints discovered during discussion that may warrant PROJECT.md promotion.

| Constraint | Source | Rationale |
|-----------|--------|-----------|
| (none for this phase) | | |

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
- Failure cases: specify concrete error type/status with domain error name in parentheses (e.g., "400 bad request (ValidationError)", "401 unauthorized (TokenExpired)", "500 internal error (DatabaseUnavailable)"), not generic "error". Omit the parenthetical only when the error is intentionally opaque by design. Assert side effects DID NOT occur where relevant.
- For complex side effects, use per-case footnotes to detail parameters and atomicity requirements.
</output_format>

---

## Transition

CASES.md written -> done. No further step files to load.
