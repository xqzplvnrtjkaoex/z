# Upstream Constraint Forwarding Pipeline - Research

**Researched:** 2026-03-26
**Domain:** case skill pipeline (case-briefer, step-discuss, step-finalize)
**Confidence:** HIGH (analysis of existing artifacts, no external dependencies)

## Summary

The `/case` pipeline currently has no systematic mechanism for forwarding architectural/security constraints from CONTEXT.md through to CASES.md when those constraints have no behavioral case. Constraints like D-98 ("ceremony state MUST NOT appear in response bodies") are invariants that apply across multiple operations but only get captured if the AI or developer manually notices them during discussion.

Analysis of the 3A CASE-SCRATCH.md reveals clear evidence of this problem: the rule "Ceremony state content MUST NOT appear in any response body or error detail (D-98)" was manually repeated as a rule in 4 separate operations (RegisterBegin R12, RegisterFinish R10, LoginBegin R5, LoginFinish R8). Similarly, "gRPC timeout 5 seconds (D-135)" appears as a separate rule in 5 operations. This is exactly the duplication that a tiered rule hierarchy would eliminate.

**Primary recommendation:** Implement a 3-tier rule hierarchy (SR/PR/R) with lightweight changes to case-briefer output, step-discuss consumption, and step-finalize output format. The briefer classifies constraints; step-discuss confirms them; step-finalize outputs them in the appropriate tier.

---

## Q1: SR/PR/R Classification Criteria

### Evidence from 3A CASE-SCRATCH.md

Analyzing all rules across 9 operations reveals three natural groupings:

**System-wide (would be SR) -- same rule in most/all phases:**

| Rule Pattern | Operations | Decision |
|-------------|------------|----------|
| gRPC timeout 5 seconds | RegisterFinish, LoginFinish, ValidateSession, RefreshToken, GetCurrentUser | D-135 |
| User service unavailable = operation failure | RegisterFinish, LoginFinish | D-136 |

These are project-level infrastructure policies. D-135 says "5 seconds via `tonic::Request::set_timeout`. Applies to all Auth -> User calls" -- the wording itself is universal. Future phases (catalog, scraper) will have similar timeout rules. These belong in PROJECT.md as system-wide invariants.

**Phase-wide (would be PR) -- same rule across multiple ops in this phase:**

| Rule Pattern | Operations | Decision |
|-------------|------------|----------|
| Ceremony state MUST NOT appear in any response body | RegisterBegin, RegisterFinish, LoginBegin, LoginFinish | D-98 |
| Ceremony state stored in Redis with 5min TTL | RegisterBegin, LoginBegin | D-97 |
| Ceremony ID delivered via HttpOnly cookie | RegisterBegin, LoginBegin | D-100 |
| All auth failures return generic "unauthorized" | LoginFinish, VerifyJwt | D-50 |
| Invite token errors return generic "invite_invalid" | RegisterBegin | D-52 |
| JWT: ES256, 15min TTL, HttpOnly cookie | RegisterFinish, VerifyJwt | D-40/D-43/D-44 |

D-98 is the canonical example: it is a security invariant that governs all ceremony-based operations in Phase 3A but has no meaning outside this phase's auth domain. D-50 similarly governs auth failure responses across all auth operations.

**Operation-specific (stays as R) -- unique to one operation:**

| Rule Pattern | Operation | Decision |
|-------------|-----------|----------|
| Processing order: validate -> consume -> reserve -> ceremony | RegisterBegin | D-16 |
| Compensating User.DeleteUser on failure | RegisterFinish | D-139 |
| Verify-only middleware (no refresh) | Logout | Design intent |
| Per-session refresh lock | RefreshToken | Design intent |

These are truly operation-specific -- they describe behavior unique to that operation's flow.

### Proposed Classification Heuristic

**Confidence: HIGH** -- derived directly from observed patterns.

```
Decision from CONTEXT.md (D-XX)
  |
  ├── Contains "all" / "every" / "always" language?
  |     YES: Candidate for SR or PR
  |     NO: Likely R
  |
  ├── Applies across phases (timeout, error format, auth policy)?
  |     YES: SR candidate
  |     NO: ...
  |
  ├── Applies to multiple operations within THIS phase?
  |     YES: PR candidate
  |     NO: R (operation-specific)
  |
  └── Is it an invariant (negation: "MUST NOT", "never")?
        YES: Strong SR/PR candidate (invariants rarely apply to single operations)
        NO: Classification by scope above
```

**Concrete criteria:**

| Tier | Trigger | Language Patterns | Examples |
|------|---------|-------------------|----------|
| SR | Applies to all services or all phases | "all gRPC calls", "every endpoint", project-wide policy | Timeout policy, error response format standards, UUIDv4/v7 policy |
| PR | Applies to multiple operations in one phase, but not universally | "all auth operations", "ceremony-based ops", phase-specific security | Ceremony state no-leak, auth failure opacity, session TTL parameters |
| R | Unique to one operation's flow | Operation-specific ordering, compensation logic, specific field validation | Token consume-first ordering, handle reservation, orphan logging |

### Edge Cases

**PR that looks like SR:** "All auth failures return generic unauthorized (D-50)" looks system-wide but is actually auth-phase-specific. Future phases (catalog) will have their own error policies. Keep as PR unless PROJECT.md explicitly promotes it.

**R that repeats:** If a rule appears in exactly 2 operations and those operations are tightly coupled (e.g., RegisterBegin + RegisterFinish), it may still be R -- the coupling is structural, not a cross-cutting concern. But if it appears in 3+ unrelated operations, promote to PR.

**Design intent rules:** Always R. Design intent captures a developer's implementation approach for a specific operation. Never promote to PR/SR.

---

## Q2: PROJECT.md SR Section Format

### Placement

**Recommendation: After "Constraints" section, before "Key Decisions".**

**Confidence: HIGH** -- this is a natural position because:
1. Constraints are fixed external facts (tech stack, deployment); SRs are the project's own invariants derived from those constraints.
2. Key Decisions is a historical record; SRs are active enforcement rules.
3. The Constraints section is the "what we must work with"; SRs are "what we require of all code."

### Format

```markdown
## System-Wide Rules

Rules that apply to ALL services and ALL phases. Lower-level rules (PR, R) may reference these.
Override requires explicit justification in the phase's CASES.md Phase Rules section.

| ID | Rule | Scope | Source |
|----|------|-------|--------|
| SR-01 | All inter-service gRPC calls use 5-second timeout via `tonic::Request::set_timeout` | All gRPC callers | D-135 (3A) |
| SR-02 | Service unavailability in synchronous call chain = operation failure. No retry, no partial recovery | All cross-service calls | D-136 (3A) |
| SR-03 | Detailed error info logged server-side with request_id correlation; generic errors to clients | All services | D-54 (3A) |
| SR-04 | UUIDv4 for security-sensitive entities; UUIDv7 for time-sortable entities | All services | PROJECT.md ID Design |
```

### Field Definitions

| Field | Description |
|-------|-------------|
| ID | `SR-{NN}`, sequential, never reused |
| Rule | One-sentence invariant. Imperative ("use", "must", "never") |
| Scope | Which services/layers this applies to |
| Source | Decision ID + phase where first established. Traceability |

### Numbering Convention

`SR-01` through `SR-99`. Two digits. No sub-numbering. If the list exceeds 20 items, the project has too many system rules and should consolidate.

### How Lower Levels Reference SRs

**In CASES.md Phase Rules:**
```markdown
## Phase Rules
- PR1: Ceremony state content MUST NOT appear in any response body or error detail (D-98)
- PR2: All auth failures return generic "unauthorized" to client (D-50)
- PR3: Per SR-01 — gRPC timeout applies to all User service calls in this phase
```

**In operation Rules, when overriding:**
```markdown
### Rules
- R1: Processing order — validate -> consume -> reserve -> ceremony (D-16)
- R2: Overrides SR-01: 10-second timeout for bulk operations (justified: batch creates take longer)
```

Override format: `Overrides SR-XX: [what changes] ([justification])`.

### Initial SR Population

For the first implementation, I recommend extracting these from existing PROJECT.md + CONTEXT.md decisions:

1. **SR-01:** 5-second gRPC timeout (D-135)
2. **SR-02:** Service unavailability = operation failure (D-136)
3. **SR-03:** Server-side detailed logging with request_id; generic client errors (D-54)
4. **SR-04:** UUIDv4 for unpredictable entities, UUIDv7 for time-sortable (already in PROJECT.md ID Design)

**Confidence: MEDIUM** -- items 1-3 are clearly system-wide from 3A evidence; SR-04 is already implicit in PROJECT.md but not formalized as a rule. The full SR list will grow as more phases are implemented.

---

## Q3: case-briefer Constraint Classification

### Current State

The briefer currently outputs constraints in two places:
1. **Per-operation "Decided constraints"** -- attached to specific operations
2. **Observations** -- cross-cutting patterns noted at the bottom

Looking at the 3A CASE-BRIEFING.md, the Observations section already captures some cross-cutting patterns informally:
- "All auth failures return generic error strings" (D-50-53)
- "All inter-service gRPC calls use a 5-second timeout" (D-135)
- "User service unavailability propagates as auth operation failure" (D-136)

These are exactly the constraints that should become SR/PR. The Observations section is the right place -- it just needs structure.

### Proposed Changes to Briefing Output

**Confidence: HIGH** -- minimal format change, backward-compatible.

#### Option A: Structured Observations with Tags (Recommended)

Replace the freeform Observations section with a structured constraint classification:

```markdown
## Cross-Cutting Constraints

### System-Wide Candidates (may belong in PROJECT.md SR)

| Constraint | Scope | Source | Existing SR? |
|-----------|-------|--------|--------------|
| 5-second gRPC timeout on all inter-service calls | All gRPC callers | D-135 | SR-01 (already in PROJECT.md) |
| Service unavailability = operation failure | All cross-service calls | D-136 | SR-02 |

### Phase-Wide Constraints (PR candidates)

| Constraint | Applies To | Source | Behavioral? |
|-----------|------------|--------|-------------|
| Ceremony state MUST NOT appear in response bodies | All ceremony ops (Register*, Login*) | D-98 | No (invariant) |
| All auth failures return generic "unauthorized" | All auth endpoints | D-50 | No (invariant) |
| Invite token errors return generic "invite_invalid" | Registration endpoints | D-52 | No (invariant) |
| JWT: ES256, 15min, HttpOnly cookie | Registration + refresh | D-40/43/44 | No (configuration) |

### Observations

[Remaining notes that don't fit the above categories]
```

**Behavioral? column:** Distinguishes constraints that will produce cases (YES -- e.g., "max 100 items per page" produces boundary cases) from constraints that are invariants (NO -- e.g., "never expose ceremony state"). The Protester uses this to decide whether to generate cases or propose Rules.

#### Option B: Tags on Existing Constraints

Add `[SR]`, `[PR]`, `[R]` tags to the per-operation constraints:

```markdown
- **Decided constraints:**
  - [PR] Ceremony state MUST NOT appear in any response body (D-98)
  - [SR] gRPC timeout 5 seconds (D-135)
  - [R] Token consume-first ordering (D-16)
```

**Tradeoff:** Option B is simpler but scatters the classification. Option A consolidates cross-cutting constraints in one place, making them visible to the Protester before operation-by-operation discussion begins.

**Recommendation: Option A.** The whole point is to surface constraints that span operations. Scattering them across operations defeats the purpose.

### Should the Briefer Read PROJECT.md for Existing SRs?

**Yes.** The briefer already reads PROJECT.md (it's in the `<files_to_read>` list). The change is:
1. Check PROJECT.md for existing `## System-Wide Rules` section
2. For each existing SR, note it in the "System-Wide Candidates" table with `Existing SR?` = yes
3. This prevents the Protester from re-discovering known system rules

If PROJECT.md has no SR section yet, the briefer outputs candidates as "New -- needs PROJECT.md addition."

### Briefer Agent Changes Summary

1. **New Step 4.5** (between current Step 4 "Extract decided constraints" and Step 5 "Map requirements"):
   - Classify each constraint as SR-candidate, PR-candidate, or operation-specific
   - SR: matches existing PROJECT.md SR or uses "all services/phases" language
   - PR: applies to 2+ operations in this phase
   - R: operation-specific (default, no change needed)

2. **Output format:** Replace `## Observations` with `## Cross-Cutting Constraints` (structured) + `## Observations` (remaining notes)

3. **Quality gate addition:** "Cross-cutting constraints section included with SR/PR classification"

---

## Q4: step-discuss Consumption Mechanism

### Where in the Discussion Flow?

**Confidence: HIGH** -- there are exactly two natural insertion points.

#### Point 1: Before First Operation (Phase Rules Confirmation)

After step-init presents operations for selection (Step 2) and before entering per-operation discussion (Step 3a), add a **Step 2.5: Phase Rules**:

```
From the briefing, I identified these phase-wide constraints:

Phase Rules (apply to all operations below):
  - PR1: Ceremony state MUST NOT appear in any response body or error detail (D-98)
  - PR2: All auth failures return generic "unauthorized" to client (D-50)
  - PR3: Invite token errors return generic "invite_invalid" (D-52)

System Rules (already in PROJECT.md, apply to this phase too):
  - SR-01: 5-second gRPC timeout
  - SR-02: Service unavailability = operation failure

Confirm these phase rules? Any to add, remove, or adjust?
```

This is **confirmatory, not automatic.** The developer sees the classification and can:
- Confirm all
- Demote a PR to R ("that only applies to registration, not login")
- Promote an R to PR ("actually, that applies to all ops")
- Add new PRs the briefer missed

#### Point 2: Per-Operation (Abbreviated Reference)

During 3a anchor, instead of repeating the full rule text, reference the PR:

```
[RegisterBegin]: Passkey registration start

Interface:
  - Inputs: token (String), name (String), handle (String)
  - Output: WebAuthn challenge + ceremony cookie
  - Auth: none (public)
  - Phase Rules: PR1, PR2 apply

  ...flow diagram...

Rules:
  - R1: Processing order — validate -> consume -> reserve -> ceremony (D-16)
  - R2: Token consume-first is fail-safe ordering (D-16)
  - (PR1, PR2, SR-01 apply — see Phase Rules above)
```

This eliminates the current duplication where D-98 is written out 4 times. The per-operation Rules section only contains operation-specific rules.

### Automatic vs. Confirmatory

**Recommendation: Confirmatory at Point 1, abbreviated at Point 2.**

- Phase Rules are confirmed once, before discussion begins
- Per-operation, they're referenced by ID (not re-confirmed)
- New constraints discovered mid-discussion are handled at Point 2 (see below)

### Handling Constraints That Span Multiple Operations

Three patterns:

1. **Known at briefing time (most common):** Briefer classifies as PR. Step 2.5 confirms. Per-operation references by ID.

2. **Discovered during discussion:** The developer says something like "actually, that same error handling applies to all operations." The Protester:
   ```
   That sounds like a phase-wide rule. I'll add it as PR4: [description].
   It will apply to the remaining operations too. Confirm?
   ```
   Then adds to the Phase Rules list and references in subsequent operations.

3. **Demoted during discussion:** A confirmed PR turns out to not apply to one operation. The Protester notes the exception:
   ```
   Rules:
     - R1: ...
     - PR1 does NOT apply here (ceremony state not involved)
   ```

### Discovering New SR Candidates During Discussion

**Trigger:** When the developer describes a constraint and uses language like "this should be the same everywhere" or "this is a project policy."

**Mechanism:**
```
That sounds like it should be a system-wide rule (all services, all phases).
I'll flag it as SR-candidate: "[description]"
This will need to be added to PROJECT.md after this session.
For now, I'll treat it as a Phase Rule (PR).
```

The SR-candidate is recorded in CASES.md in a new `## SR Candidates` section at the bottom. The developer or a follow-up session adds it to PROJECT.md.

**Rationale:** The `/case` skill should not modify PROJECT.md directly. That's a separate concern. But it should capture the discovery and make it visible.

---

## Q5: Interaction Between Tiers

### If a SR Already Covers a Concern

**Recommendation: Mention once in Phase Rules, not per-operation.**

In Step 2.5:
```
System Rules (from PROJECT.md):
  - SR-01: 5-second gRPC timeout — applies to: RegisterFinish, LoginFinish,
    ValidateSession, RefreshToken, GetCurrentUser
  - SR-02: Service unavailability = operation failure — applies to: RegisterFinish, LoginFinish
```

In per-operation Rules, do NOT repeat the SR content. Instead:
```
Rules:
  - R1: [operation-specific rule]
  - SR-01, SR-02 apply (see Phase Rules)
```

This prevents the current problem where "gRPC timeout 5 seconds" is written 5 times.

### Can a Phase Override a System Rule?

**Yes, with documented justification.**

**Format in CASES.md Phase Rules:**
```
## Phase Rules
- PR1: ...
- PR-override-SR-01: Bulk import operations use 30-second timeout
  (justification: batch processing needs longer window than default 5s)
```

**Format in operation Rules:**
```
Rules:
- R1: Overrides SR-01 — 10-second timeout for this operation
  (justification: downstream service has known slow path)
```

The override is always explicit, always justified, and always references the SR being overridden. The planner and executor see both the default (SR) and the exception (override) and can implement accordingly.

### SR Promotion Trigger

During discussion, when the Protester notices a pattern:

| Signal | Action |
|--------|--------|
| Same rule appears in 3+ operations across 2+ categories | Propose as PR |
| Developer says "this is a project policy" or "same everywhere" | Flag as SR-candidate |
| Rule already exists in PROJECT.md | Reference as SR, no re-discussion |
| Rule contradicts existing SR | Flag conflict, ask developer to resolve |

---

## Proposed Changes Summary

### Files to Modify

| File | Change | Scope |
|------|--------|-------|
| `.planning/PROJECT.md` | Add `## System-Wide Rules` section | One-time, then maintained |
| `.claude/agents/case-briefer.md` | Add Step 4.5 (constraint classification), restructure Observations | Agent definition |
| `.claude/skills/case/step-discuss.md` | Add Step 2.5 (Phase Rules confirmation), update 3a/3e for PR references | Skill step |
| `.claude/skills/case/step-finalize.md` | Add Phase Rules section to output format | Skill step |
| `.claude/skills/case/SKILL.md` | Update formatting section for PR/SR references in flow diagrams | Skill index |

### CASES.md Output Format Changes

```markdown
# Phase [XX]: [Name] - Behavioral Cases

**Discovered:** [date]
**Operations covered:** [count]
**Total cases:** S:[count] F:[count] E:[count] Q:[count]

---

## Phase Rules

> Constraints that apply to ALL operations in this phase.
> Referenced by ID (PR1, PR2...) in operation Rules sections.

- PR1: Ceremony state content MUST NOT appear in any response body or error detail (D-98)
- PR2: All auth failures return generic "unauthorized" to client (D-50)
- PR3: Invite token errors return generic "invite_invalid" (D-52)
- PR4: JWT configuration: ES256, 15min TTL, HttpOnly/Secure/SameSite=Strict cookie (D-40, D-43, D-44)

**System Rules (from PROJECT.md) applicable to this phase:**
- SR-01: 5-second gRPC timeout — RegisterFinish, LoginFinish, ValidateSession, RefreshToken, GetCurrentUser
- SR-02: Service unavailability = operation failure — RegisterFinish, LoginFinish

---

## Operation: [OperationName]

...

### Rules
- R1: [operation-specific rule]
- R2: [operation-specific rule]
- Inherits: PR1, PR2, SR-01

### Cases
...

---

## SR Candidates

> Constraints discovered during discussion that may warrant PROJECT.md promotion.

| Constraint | Proposed ID | Source | Rationale |
|-----------|-------------|--------|-----------|
| (none for this phase) | | | |

---

## Cross-Operation Concerns
...
```

### Key Design Decisions

| Decision | Rationale | Alternative Considered |
|----------|-----------|----------------------|
| Confirmatory Phase Rules (not automatic) | Developer must validate AI classification; prevents silent wrong assumptions | Automatic insertion -- rejected because misclassification would propagate silently |
| PR referenced by ID in operation Rules | Eliminates duplication; current 3A has D-98 written 4 times | Full text repeated -- rejected, the whole problem we're solving |
| SR-candidates captured but not written to PROJECT.md | `/case` skill shouldn't modify project-level docs mid-session | Auto-write to PROJECT.md -- rejected, scope creep |
| Briefer uses structured Cross-Cutting Constraints section | Consolidates classification for Protester; freeform Observations was already doing this informally | Tags on per-operation constraints -- rejected, scatters information |
| Override requires explicit justification | Prevents accidental SR violations; makes exceptions visible in the plan | Silent override -- rejected, defeats purpose of system rules |

---

## Risk Analysis

### Low Risk
- **Output format changes** (CASES.md Phase Rules section): Additive, doesn't break existing consumer parsing. Planner and executor just get more structured input.
- **PROJECT.md SR section**: Additive, no existing content modified.

### Medium Risk
- **Briefer Step 4.5 classification accuracy**: The briefer (Sonnet model) may misclassify constraints. Mitigation: Step 2.5 confirmatory check by developer.
- **Protester context budget**: Phase Rules confirmation adds ~10 lines to conversation. For phases with many PRs, this could add up. Mitigation: Group related PRs.

### Addressed by Design
- **Constraint missed by briefer**: Step-discuss still allows ad-hoc rule discovery during 3c probing. The briefer provides a starting point, not a complete set.
- **SR drift (PROJECT.md out of date)**: SR-candidates section in CASES.md creates a backlog. Periodic PROJECT.md updates address this.

---

## Implementation Sequence

Recommended order:

1. **PROJECT.md SR section** -- Add initial SRs extracted from 3A decisions. Independent of other changes.
2. **case-briefer.md** -- Add Step 4.5 + restructure output. Can test independently by re-running briefer on 3A.
3. **step-discuss.md** -- Add Step 2.5 + update 3a/3e. Requires briefer changes to be in place.
4. **step-finalize.md** -- Add Phase Rules section to output format. Requires discuss changes.
5. **SKILL.md** -- Update formatting section. Last, as it's documentation.

Steps 1-2 can be done in parallel. Steps 3-5 are sequential.

---

## Confidence Assessment

| Area | Level | Reason |
|------|-------|--------|
| SR/PR/R classification criteria | HIGH | Derived directly from 3A evidence -- 60+ rules analyzed across 9 operations |
| PROJECT.md format | HIGH | Simple additive table, follows existing PROJECT.md conventions |
| Briefer changes | HIGH | Minimal format change, extends existing Observations pattern |
| Step-discuss consumption | HIGH | Clear insertion points; confirmatory approach is safe |
| Tier interaction | MEDIUM | Override semantics are well-defined but untested in practice |
| Full pipeline integration | MEDIUM | Each piece is simple but end-to-end flow has not been exercised |

---

## Open Questions

1. **Should the case-validator also check PR/SR coverage?**
   - Currently validates operation-level Rules against cases
   - Could additionally verify: "PR1 references D-98 -- is there at least one case per ceremony operation where this constraint is testable?"
   - Recommendation: Defer. PR invariants are not always testable as cases (that's the whole point -- they're non-behavioral). The validator should check that PRs don't contradict cases, not that cases exercise PRs.

2. **Retroactive application to existing CASES.md?**
   - 3A CASE-SCRATCH.md already exists with duplicated rules
   - Should it be rewritten with the new format?
   - Recommendation: No. The 3A planning is complete. Apply to future phases (3B onward). The 3A scratch file serves as the evidence base for this research.

3. **Planner consumption of Phase Rules**
   - How does plan-phase use PR and SR references?
   - Recommendation: Planner should include PRs as cross-cutting verification items, not per-task items. E.g., "Verify PR1: ceremony state not in any response" as a phase-level acceptance criterion, not duplicated per task.
