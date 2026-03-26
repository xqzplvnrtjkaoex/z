# Constraint Forwarding Research Summary

**Domain:** /case skill pipeline -- constraint classification and forwarding
**Researched:** 2026-03-26
**Confidence:** HIGH
**Inputs:** CASE-CONSTRAINT-FORWARDING-UPSTREAM.md (R1), CASE-CONSTRAINT-FORWARDING-DOWNSTREAM.md (R2)

## Executive Summary

The `/case` pipeline currently has no systematic mechanism for forwarding architectural constraints through from CONTEXT.md to CASES.md. Constraints like D-98 ("ceremony state MUST NOT appear in response bodies") are invariants that apply across multiple operations but only get captured if manually noticed during discussion. When they are captured, they get duplicated -- in 3A, D-98 appears as a separate rule in 4 operations, D-135 in 5 operations.

Both researchers independently converge on a 3-tier rule hierarchy: System Rules (SR) in PROJECT.md for project-wide invariants, Phase Rules (PR) in CASES.md for phase-wide constraints, and operation-level Rules (R) for operation-specific logic. They agree on classification criteria, output placement, and the principle that constraints should be confirmed by the developer rather than automatically inserted. The researchers diverge on minor formatting (numbering convention), implementation sequencing (SR section timing), and retroactive application to 3A.

The combined change touches 7 files across the case pipeline. The changes are additive -- no existing functionality is removed, the 5-check validator structure is preserved, and the 15-finding cap is unchanged. The primary risk is classification accuracy in the briefer (Sonnet model), which is mitigated by the confirmatory step in discussion.

---

## Reconciliation: Where Researchers Agree and Diverge

### Full Agreement

| Topic | Shared Position |
|-------|----------------|
| 3-tier hierarchy (SR/PR/R) | Both propose identical tiers with identical scoping |
| Classification criteria | Both derive the same categories from 3A evidence: SR = cross-phase, PR = cross-operation within phase, R = single-operation |
| Phase Rules placement | Before first operation in CASES.md, after document header |
| Confirmatory approach | Developer confirms PR classification before discussion proceeds; not automatic |
| PR reference in operation Rules | Reference by ID ("Inherits: PR1, SR-01") instead of duplicating text |
| SR candidates in CASES.md | New section at bottom of CASES.md for constraints needing PROJECT.md promotion |
| Override semantics | Explicit, justified, references the rule being overridden |
| No SR verification in validator | SRs are inherited by default; checking produces noise without safety benefit |

### Divergences Requiring Resolution

#### 1. PR Numbering Convention

| R1 (Upstream) | R2 (Downstream) | Resolution |
|---------------|-----------------|------------|
| `PR1, PR2, PR3` (no dash, no padding) | `PR-01, PR-02, PR-03` (dash, zero-padded) | **Use R1's format: `PR1, PR2, PR3`** |

Rationale: Operation-level rules already use `R1, R2, R3` (no dash, no padding). Phase Rules should follow the same convention for consistency within CASES.md. The dash/padding in `PR-01` is borrowed from `SR-01` convention, which makes sense for PROJECT.md (a more formal document) but is overly formal for CASES.md rules that are confirmed conversationally. System Rules keep their format: `SR-01, SR-02`.

#### 2. SR Section Timing

| R1 (Upstream) | R2 (Downstream) | Resolution |
|---------------|-----------------|------------|
| Do PROJECT.md SR section first, in parallel with briefer changes | Defer PROJECT.md SR section; validator works without it | **R1's approach: do SR section early** |

Rationale: The SR section is small (4 initial rules), independent of all other changes, and enables the briefer to cross-reference existing SRs immediately. Deferring it means the briefer's "Existing SR?" column has nothing to check against. Doing it first is ~10 minutes of work with clear value.

#### 3. Retroactive Application to 3A

| R1 (Upstream) | R2 (Downstream) | Resolution |
|---------------|-----------------|------------|
| No retroactive update to 3A CASE-SCRATCH | Yes, at next /case session or plan-phase | **Neither -- address only if 3A is reopened** |

Rationale: 3A planning is complete. Retroactively rewriting CASE-SCRATCH.md for a finished phase has no downstream consumer. If 3A is ever revisited (e.g., re-planning after scope change), the new format would apply naturally. Do not create work that produces no benefit.

#### 4. Briefer Output: "Behavioral?" Column

| R1 (Upstream) | R2 (Downstream) | Resolution |
|---------------|-----------------|------------|
| Adds "Behavioral?" column to cross-cutting constraints table | Provides the classification heuristic but doesn't specify this column | **Include the column -- it bridges R1's format with R2's heuristic** |

The briefer's "Behavioral?" column answers exactly the question R2's decision tree addresses. The column value tells the Protester: behavioral constraint = will produce cases, non-behavioral = propose as Rule.

---

## Unified Classification Heuristic

Both researchers derive compatible classification logic. The unified decision tree (combining R1's scope-based approach with R2's content-based signals):

```
For each CONTEXT.md decision (D-XX):

1. Structural? (code organization, file structure, trait design, tooling choice)
   YES -> SKIP
   NO  -> continue

2. Caller-triggered event with observable outcome?
   (explicit status code, error name, "when X -> Y", "returns/rejects/accepts")
   YES -> BEHAVIORAL (needs case in S/F/E table)
   NO  -> continue

3. "MUST/MUST NOT/always/never" constraint on implementation?
   YES -> ARCHITECTURAL CONSTRAINT -> classify scope:
         a. "All services/phases" language, or matches existing SR?  -> SR
         b. Applies to 2+ operations in this phase?                  -> PR
         c. Single operation?                                        -> R
   NO  -> continue

4. Configuration parameter, storage format, or transport mechanism?
   YES -> ARCHITECTURAL CONSTRAINT (scope as above) or SKIP if purely internal
   NO  -> SKIP (informational context)
```

**Ambiguous cases:** When a decision has both behavioral and constraint aspects, classify as BEHAVIORAL (the case covers the observable part) and note the constraint in the operation's Rules section.

**Language signals:**
- Behavioral: explicit status codes, error names, conditional outcomes, observable verbs
- Constraint: negative universals (MUST NOT), positive universals (always/every), config values, mechanisms (stored as, via), design policies (no retry, fail-fast)
- Structural: architecture, convention, pattern, directory, module, trait design

---

## Unified CASES.md Output Format

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
- SR-01: [brief description] -- [which operations]
- SR-02: [brief description] -- [which operations]

---

## Operation: [OperationName]

[anchor, flow diagram, etc.]

### Rules
- R1: [operation-specific rule]
- R2: [operation-specific rule]
- Inherits: PR1, PR2, SR-01

### Side Effects
...

### Cases
| # | Scenario | Priority | Expected Outcome |
...

---

## SR Candidates

> Constraints discovered during discussion that may warrant PROJECT.md promotion.

| Constraint | Source | Rationale |
|-----------|--------|-----------|
| (none for this phase) | | |

---

## Cross-Operation Concerns
...
```

Key format decisions:
- PR numbering: `PR1, PR2, PR3` (matches R-numbering style)
- SR numbering: `SR-01, SR-02` (dash + zero-padded, formal PROJECT.md style)
- SR references in Phase Rules use "Per SR-XX" prefix
- Operation Rules use "Inherits: PR1, PR2, SR-01" line (not repeated text)
- Override format: "Overrides SR-XX: [what changes] ([justification])"

---

## Unified Change Plan

### Files to Modify (7 total)

| # | File | Primary Changes | Owner Research |
|---|------|----------------|----------------|
| 1 | `.planning/PROJECT.md` | Add `## System-Wide Rules` section with SR-01 through SR-04 | R1 |
| 2 | `.claude/agents/case-briefer.md` | Add Step 4.5 (constraint classification); replace Observations with structured Cross-Cutting Constraints section | R1 |
| 3 | `.claude/agents/case-validator.md` | Extend Check B with classification heuristic + two sub-categories (Decision Gaps, Constraint Forwarding Gaps); add Phase Rules to coverage scope; update severity table; update return protocol counter | R2 |
| 4 | `.claude/skills/case/step-init.md` | Add Phase Rules section to CASE-SCRATCH.md header template | R2 |
| 5 | `.claude/skills/case/step-discuss.md` | Add Step 2.5 (Phase Rules confirmation before first operation); update 3a for PR references; add mid-discussion PR promotion pattern; add SR-candidate discovery | R1 + R2 |
| 6 | `.claude/skills/case/step-finalize.md` | Add Phase Rules section to output format; update validator finding presentation for Constraint Forwarding Gaps | R1 + R2 |
| 7 | `.claude/skills/case/SKILL.md` | Update formatting section for PR/SR references; add PR numbering convention | R1 + R2 |

### Implementation Order

**Batch 1 (independent, parallel):**
- `PROJECT.md` -- Add SR section. 4 initial rules from 3A evidence.
- `case-validator.md` -- Extend Check B. Include decision tree verbatim.

**Batch 2 (depends on Batch 1 for SR references + classification heuristic):**
- `case-briefer.md` -- Add Step 4.5 + Cross-Cutting Constraints output. Reads PROJECT.md SRs.
- `step-init.md` -- Add Phase Rules to CASE-SCRATCH.md template. Simple additive change.

**Batch 3 (depends on Batch 2 for briefer output format):**
- `step-discuss.md` -- Add Step 2.5 + PR reference pattern. Consumes briefer output.

**Batch 4 (depends on Batch 3 for discussion output):**
- `step-finalize.md` -- Add Phase Rules to output format + validator presentation changes.
- `SKILL.md` -- Update formatting section. Documentation of the above.

### What Does NOT Change

- 5-check validator structure (A through E)
- 15-finding cap
- Validator dispatch/Agent call pattern in step-finalize
- Check A, C, D, E logic (except: Check C gains SR contradiction detection as a lightweight addition)
- CASE-BRIEFING.md core format (only Observations section is restructured)

---

## Detailed Changes Per File

### 1. PROJECT.md -- System-Wide Rules Section

**Placement:** After "Constraints" section, before "Key Decisions."

**Content:**
```markdown
## System-Wide Rules

Rules that apply to ALL services and ALL phases. Phase Rules (PR) and
operation Rules (R) may reference these. Override requires explicit
justification in the phase's CASES.md.

| ID | Rule | Scope | Source |
|----|------|-------|--------|
| SR-01 | All inter-service gRPC calls use 5-second timeout via tonic::Request::set_timeout | All gRPC callers | D-135 (3A) |
| SR-02 | Service unavailability in synchronous call chain = operation failure; no retry, no partial recovery | All cross-service calls | D-136 (3A) |
| SR-03 | Detailed error info logged server-side with request_id; generic errors to clients | All services | D-54 (3A) |
| SR-04 | UUIDv4 for security-sensitive entities; UUIDv7 for time-sortable entities | All services | ID Design |
```

### 2. case-briefer.md -- Constraint Classification

**New Step 4.5** (between "Extract decided constraints" and "Map requirements"):
- Apply classification heuristic to each constraint
- SR: matches existing PROJECT.md SR or uses "all services/phases" language
- PR: applies to 2+ operations in this phase
- R: operation-specific (default)

**Output format change:** Replace `## Observations` with:
```markdown
## Cross-Cutting Constraints

### System-Wide Candidates (may belong in PROJECT.md SR)
| Constraint | Scope | Source | Existing SR? |
|-----------|-------|--------|--------------|

### Phase-Wide Constraints (PR candidates)
| Constraint | Applies To | Source | Behavioral? |
|-----------|------------|--------|-------------|

### Observations
[remaining notes]
```

**Quality gate addition:** "Cross-cutting constraints section included with SR/PR classification."

### 3. case-validator.md -- Check B Extension

**Check B rename:** "Decision and Constraint Coverage"

**Classification logic:** Include decision tree verbatim with 3A examples as few-shot guidance.

**Output sub-categories:**
- Decision Gaps: behavioral decisions with no exercising case (existing)
- Constraint Forwarding Gaps: architectural constraints with no covering Rule or Phase Rule (new)

**Coverage scope expansion:** Check if unmatched constraint appears in Phase Rules (PR section) in addition to operation Rules.

**Severity:** Constraint Forwarding Gaps default to Medium. Security constraints ("MUST NOT", "never exposed", "security") elevated to High.

**Return protocol:** Add one counter:
```
Requirement Gaps: [N] | Decision Gaps: [N] | Constraint Gaps: [N] | Consistency: [N] | Completeness: [N] | Briefing: [N]
```

**Priority ordering within cap:**
Requirement Gaps > Decision Gaps > Constraint Forwarding Gaps (security-elevated rank with Decision Gaps) > Consistency > Completeness > Briefing

### 4. step-init.md -- CASE-SCRATCH.md Header

Add Phase Rules section to scratch template:
```markdown
## Phase Rules
(populated after briefing review)

---
```

### 5. step-discuss.md -- Phase Rules Confirmation

**New Step 2.5** (after operation selection, before first operation discussion):
- Present Phase Rules from briefer's Cross-Cutting Constraints
- Present applicable System Rules from PROJECT.md
- Developer confirms, modifies, promotes, or demotes

**Step 3a update:** Operation Rules reference PRs by ID ("Inherits: PR1, SR-01") instead of duplicating text.

**Mid-discussion promotion:** When a constraint discovered during operation discussion applies to multiple operations, propose promoting to PR.

**SR-candidate discovery:** When developer uses "project policy" or "same everywhere" language, flag as SR-candidate. Record in CASES.md SR Candidates section. Do not modify PROJECT.md.

### 6. step-finalize.md -- Output Format + Validator Presentation

**Output format:** Add Phase Rules section between header and first operation (see unified format above).

**Validator finding presentation:** Distinguish Constraint Forwarding Gaps from Decision Gaps:
- Decision Gap: "no failure case for X -- suggested case: F_ [description]"
- Constraint Forwarding Gap: "not documented as Phase Rule or operation Rule -- suggested action: Add as PR-XX"

### 7. SKILL.md -- Formatting Update

- Add Phase Rules section to output format example
- Document PR numbering convention (PR1, PR2 -- no dash, no padding)
- Document "Inherits:" line convention for operation Rules
- Note SR reference format ("Per SR-XX")

---

## Risk Analysis

| Risk | Severity | Mitigation |
|------|----------|------------|
| Briefer (Sonnet) misclassifies constraints | Medium | Step 2.5 confirmatory check by developer before discussion proceeds |
| Phase Rules add context overhead to discussion | Low | Group related PRs; ~10 lines added to conversation start |
| Constraint Forwarding Gaps produce noise in validator | Low | Medium default severity; security keywords elevate to High |
| End-to-end flow untested | Medium | Each component is simple; first real test will be next /case session |
| Classification heuristic too complex for agent prompt | Low | Express as concrete decision tree with 3A examples, not prose |

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Classification heuristic | HIGH | Both researchers derived same categories independently from 3A evidence (40+ decisions analyzed) |
| CASES.md output format | HIGH | Additive change following existing R1/R2 conventions |
| Briefer changes | HIGH | Minimal format change extending existing Observations pattern |
| Validator changes | HIGH | Natural extension of Check B's existing filtering logic |
| Step-discuss flow | HIGH | Clear insertion points; confirmatory approach is safe |
| PROJECT.md SR section | HIGH | Simple additive table with 4 initial entries |
| Tier interaction (overrides) | MEDIUM | Well-defined semantics but untested in practice |
| Full pipeline integration | MEDIUM | Individual pieces are simple; end-to-end has not been exercised |

**Overall: HIGH** -- both researchers converge on the same design with minor formatting differences. All changes are additive. First real validation will be next /case session.

---

## Open Questions

### OQ-1: Check C SR Contradiction Detection

R2 proposes adding SR contradiction detection to Check C (consistency check): "cross-reference numerical values in CASES.md against PROJECT.md System Rules." R1 does not mention this.

**Recommendation:** Defer. This is a nice-to-have that adds complexity to a check that is not being modified in this change. When a PR override contradicts an SR, the explicit "Overrides SR-XX" format makes it visible without automated detection. Revisit if contradictions actually occur in practice.

### OQ-2: Planner Consumption of Phase Rules

R1 raises how plan-phase uses PR and SR references. Recommendation from R1: "Planner should include PRs as cross-cutting verification items, not per-task items."

**Recommendation:** Document this in the synthesis but do not implement now. The planner already reads CASES.md; the new Phase Rules section will be naturally visible. If the planner needs explicit instructions, add them when the planner skill is next modified.

### OQ-3: Validator Model Complexity

R2 notes the classification heuristic adds branching logic to Check B and recommends including the decision tree verbatim (not as prose). This is a strong recommendation -- the validator agent definition should contain the concrete decision tree with examples, not a paragraph the model must interpret.

---

## Sources

### Primary (HIGH confidence)
- `.planning/phases/03a-authentication-core/CASE-SCRATCH.md` -- 60+ rules analyzed across 9 operations
- `.planning/phases/03a-authentication-core/03a-CONTEXT.md` -- 40+ decisions classified
- `.planning/phases/03a-authentication-core/CASE-BRIEFING.md` -- existing briefer output pattern
- `.claude/agents/case-validator.md` -- current validator structure
- `.claude/agents/case-briefer.md` -- current briefer agent
- `.claude/skills/case/step-discuss.md` -- current discussion flow
- `.claude/skills/case/step-finalize.md` -- current output format
- `.claude/skills/case/SKILL.md` -- main skill definition

### Secondary (HIGH confidence)
- `.planning/research/CASE-VALIDATOR-REDESIGN.md` -- prior validator research
- Memory: `feedback_case_constraint_forwarding.md` -- original problem statement
- Memory: `project_system_rules_promote.md` -- 3-tier hierarchy proposal

---

*Research synthesized: 2026-03-26*
*Implementation ready: yes -- all changes specified at file + section level*
