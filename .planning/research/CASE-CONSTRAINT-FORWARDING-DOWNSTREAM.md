# Constraint Classification: case-validator + Output Format Research

**Researched:** 2026-03-26
**Domain:** Case-validator agent methodology, CASES.md output format
**Confidence:** HIGH

---

## Summary

The case-validator currently treats all CONTEXT.md decisions with no covering case as "Decision Gaps." But the 3A re-validation surfaced that some decisions are architectural invariants that should appear as Rules in CASES.md, not as behavioral cases in S/F/E tables. These two gap types require different developer actions ("add a case" vs "add a Rule") and different severity classification.

This research analyzes the 3A CONTEXT.md decisions to derive a concrete classification heuristic, evaluates integration options with the existing 5-check validator structure, designs the Phase Rules section for CASES.md, and proposes output formats for the new finding category.

**Primary recommendation:** Extend Check B with two sub-categories (Decision Gaps + Constraint Forwarding Gaps) rather than adding a new Check F. Add a Phase Rules (PR) section to CASES.md output format before individual operations. Do not verify SR coverage in the validator -- SRs are inherited by default and checking would produce noise.

---

## Q1: Constraint Classification Heuristic

### Evidence from 3A CONTEXT.md

Analyzing all 3A decisions against the question "does this need a behavioral case, a Rule, or neither?"

**Category 1: Behavioral decisions (need a case)**

These answer "what should the caller observe when X happens?" -- they describe a triggerable event with an observable outcome.

| Decision | Content | Why behavioral |
|----------|---------|----------------|
| D-03 | Single-use, 30-minute expiry | Caller observes rejection on reuse/expiry |
| D-48 | Invalid JWT signature -> immediate 401, no session fallback | Caller observes specific error path |
| D-50 | Generic `unauthorized` for all auth failures | Caller observes specific error message |
| D-52 | Invite token errors: unified `invite_invalid` | Caller observes specific error message |
| D-53 | Deactivated user login: generic `unauthorized` | Caller observes hidden failure reason |
| D-34 | JWT grace period: 1 minute | Caller observes automatic refresh behavior |
| D-35 | Concurrent sessions: unlimited | Caller observes both sessions work |
| D-36 | Logout current session only | Caller observes scope of invalidation |
| D-111 | Login success: 204 No Content | Caller observes response format |
| D-112 | Registration success: 201 + specific body | Caller observes response format |
| D-15 | Last passkey cannot be deleted | Caller observes rejection |

**Pattern:** These decisions contain action verbs with observable results: "returns," "rejected," "observes," "receives," explicit status codes, explicit error names.

**Category 2: Architectural constraints (need a Rule)**

These impose invariants on implementation that are NOT triggered by a specific caller action. They constrain HOW things work, but the constraint itself is not directly testable via a request/response exchange. They are "always true" properties, not "when X then Y" behaviors.

| Decision | Content | Why Rule, not case |
|----------|---------|-------------------|
| D-98 | Ceremony state MUST NOT appear in response body or error detail | Negative invariant -- absence cannot be triggered. No caller action produces ceremony state in response; the constraint is that it NEVER appears. This is verified by code review / test assertion on all ceremony-related responses, not by a specific input that triggers a specific output |
| D-44 | Cookie: HttpOnly=true, Secure=true, SameSite=Strict | Implementation invariant on all cookie-setting responses. Not triggered by caller action. Verified by inspecting cookie attributes in any Set-Cookie response |
| D-135 | Per-endpoint gRPC timeout: 5 seconds | Cross-cutting operational parameter. Not a caller-triggered behavior -- the caller never "requests a timeout." It manifests as a failure case (F: "service timeout >5s -> 500") but the Rule itself is the 5-second threshold |
| D-136 | User service unavailable = auth operation failure | Operational policy. The failure case tests infra unavailability; the Rule documents the design decision that there is no fallback/retry |
| D-100 | Ceremony ID transport: HttpOnly cookie | Implementation mechanism. The case tests "ceremony cookie missing -> error" but the cookie-as-transport-mechanism is a Rule |
| D-101 | Credential storage: JSONB column with specific schema | Storage implementation invariant |
| D-99 | Challenge state Redis key: `ceremony:{type}:{random_id}` | Key naming convention. Implementation detail surfacing as a Rule |
| D-132 | Seed invite: fixed well-known dev token | Bootstrap mechanism, not caller-triggered |

**Pattern:** These decisions contain: "MUST NOT" (negative), "always" (invariant), "stored as" (implementation mechanism), configuration parameters (TTL values, key patterns), transport mechanisms. They describe properties that hold across all operations or constrain implementation without a triggering event.

**Category 3: Structural decisions (skip entirely)**

These affect code organization, not behavior or constraints visible to any consumer (caller or implementor).

| Decision | Content | Why skip |
|----------|---------|----------|
| D-68 | 4-layer architecture | Code organization |
| D-75 | mod.rs convention | File naming |
| D-76 | adapter/ directory structure | Directory layout |
| D-82 | Prefer trait_variant over async_trait | Implementation choice |
| D-69 | AuthPorts and AuthConfig separate traits | Code design pattern |
| D-77 | Directory-based organization | File organization |

**Pattern:** These mention "architecture," "convention," "pattern," "structure," "organization." They are invisible to both caller and test assertions.

### The Ambiguous Zone

Some decisions straddle the boundary between behavioral and architectural constraint:

| Decision | Content | Ambiguity | Resolution |
|----------|---------|-----------|------------|
| D-37 | JWT caching: in-memory, ~10sec TTL | The caching mechanism is architectural, but "concurrent requests receive same JWT within window" is observable | **Behavioral** -- the observable effect (same JWT) maps to an edge case |
| D-42 | Validation: iss + aud claim verification enabled | The configuration is architectural, but "wrong iss/aud -> 401" is observable | **Behavioral** -- the rejection is a testable failure case |
| D-102 | Concurrent ceremony policy: TTL natural expiry only | The policy is architectural, but "no ceremony limit" is observable (multiple ceremonies succeed) | **Both** -- edge case for observable behavior, Rule for the design rationale |
| D-139 | Compensating User.DeleteUser on registration failure | The compensation mechanism is architectural, but the double-failure outcome is observable | **Both** -- F case for double-failure outcome, Rule for the compensation strategy |

### Proposed Decision Tree

```
For each CONTEXT.md decision (D-XX):

1. Does it describe code organization, file structure, naming convention, or trait design?
   YES -> SKIP (structural)
   NO  -> continue

2. Does it describe infrastructure setup, dev tooling, or deployment config?
   YES -> SKIP (structural/infra)
   NO  -> continue

3. Does it describe a specific caller-triggered event with an observable outcome?
   (e.g., "when X happens, caller sees Y", explicit status code, explicit error name)
   YES -> BEHAVIORAL (needs a case)
   NO  -> continue

4. Does it impose a "MUST/MUST NOT/always/never" constraint on implementation
   that is NOT triggered by a specific caller action?
   (e.g., "state MUST NOT appear in response", "timeout is 5 seconds",
   "cookie MUST be HttpOnly", "no retry logic")
   YES -> ARCHITECTURAL CONSTRAINT (needs a Rule)
   NO  -> continue

5. Does it set a configuration parameter, storage format, or transport mechanism
   that manifests only through other behavioral decisions?
   (e.g., "Redis key pattern", "JSONB column", "PEM format keys")
   YES -> ARCHITECTURAL CONSTRAINT (needs a Rule) or SKIP if purely internal
   NO  -> SKIP (informational context)
```

**Handling ambiguous cases:** When a decision has both an architectural constraint aspect AND an observable behavioral aspect, classify it as BEHAVIORAL (the case covers the observable part) and note the constraint aspect in the case's Rules section. The validator should not report these as Constraint Forwarding Gaps if a behavioral case exists, even if no explicit Rule row exists.

### Language Pattern Signals

**Behavioral signals (high probability of needing a case):**
- Explicit status codes: "returns 401", "responds with 204"
- Explicit error names: "`unauthorized`", "`invite_invalid`"
- Conditional outcomes: "if X then Y", "when X -> Y"
- Observable verbs: "returns", "rejects", "accepts", "receives", "shows", "discloses"
- Caller perspective: "client sees", "caller observes"

**Architectural constraint signals (high probability of needing a Rule):**
- Negative universals: "MUST NOT", "never exposed", "not disclosed"
- Positive universals: "always", "all endpoints", "every response"
- Configuration values: "5 seconds", "15 minutes", "10 codes"
- Implementation mechanisms: "stored as", "transported via", "serialized with"
- Design policies: "no retry", "no fallback", "fail-fast"

**Structural signals (skip):**
- Organization terms: "architecture", "convention", "pattern", "structure"
- File/module terms: "directory", "module", "mod.rs", "folder"
- Trait/type terms: "trait design", "generic over", "type alias"
- Tool choices: "prefer X over Y" (for implementation tools, not behavior)

**Confidence:** HIGH -- derived from systematic analysis of 40+ real decisions in 3A CONTEXT.md.

---

## Q2: Check B Modification vs New Check F

### Option Analysis

**Option A: Extend Check B with two sub-categories** (RECOMMENDED)

Check B currently: "Cross-reference CONTEXT.md behavioral decisions against CASE-SCRATCH.md. Find: Behavioral decisions with no exercising case."

Proposed change: Check B becomes "Decision and Constraint Coverage." It applies the classification heuristic to each unmatched decision and produces two sub-lists:

```
Check B output:
  - Decision Gaps: [behavioral decisions with no exercising case]
  - Constraint Forwarding Gaps: [architectural constraints with no covering Rule]
```

Trade-offs:
| Pro | Con |
|-----|-----|
| No new check -- keeps the 5-check structure stable | Check B becomes more complex internally |
| Both sub-categories share the same source artifact (CONTEXT.md) and same scan logic | Output format changes from flat list to two sub-lists |
| Preserves the finding cap (15) across both sub-categories naturally | Validator agent needs the classification heuristic |
| Minimal change to dispatch/return contracts | |

**Option B: Add new Check F**

Add a sixth check specifically for constraint forwarding.

Trade-offs:
| Pro | Con |
|-----|-----|
| Clean separation of concerns | Breaks the established 5-check structure |
| Each check does exactly one thing | Two checks scan CONTEXT.md (B and F) -- redundant traversal |
| Easier to skip independently | Finding cap (15) now shared across 6 categories instead of 5 |
| | More changes to output format, dispatch, return protocol |

**Option C: Modify Check B's filtering heuristic only**

Instead of producing two sub-categories, modify Check B to filter OUT architectural constraints silently (as it currently filters structural decisions). The constraint forwarding responsibility moves entirely to step-discuss.

Trade-offs:
| Pro | Con |
|-----|-----|
| Zero change to validator output | Validator silently ignores constraints -- no safety net |
| Simplest implementation | If step-discuss misses a constraint, nobody catches it |
| | Contradicts the original problem: constraints DO need forwarding |

### Recommendation: Option A

Option A is the clear winner. The validator already scans CONTEXT.md decisions in Check B. The classification heuristic adds one branching decision per unmatched decision. The output contract gains one new section heading but the overall structure is unchanged.

The key insight is that Check B's current behavioral filtering heuristic already does half the work -- it filters OUT structural decisions. Extending it to distinguish "behavioral (needs case)" from "architectural constraint (needs Rule)" is a natural refinement, not a new capability.

### Impact on Finding Cap

The cap of 15 should remain unchanged and span all categories. If 15 findings are reached, priority ordering becomes:

```
Requirement Gaps > Decision Gaps > Constraint Forwarding Gaps > Consistency > Completeness > Briefing
```

Constraint Forwarding Gaps rank below Decision Gaps because a missing case has higher impact than a missing Rule -- a missing case means untested behavior, while a missing Rule means undocumented-but-still-implemented constraint.

### Output Format Impact

The return protocol gains one counter:

```
## VALIDATION COMPLETE
Requirement Gaps: [count] | Decision Gaps: [count] | Constraint Gaps: [count] | Consistency: [count] | Completeness: [count] | Briefing: [count]
```

The structured output gains one section (see Q5 for full format).

### Coverage Scope Expansion for Check B

Currently Check B checks if a decision appears in "Rules, Side Effects, OR case table Expected Outcome." For the Constraint Forwarding sub-category, coverage scope should additionally include Phase Rules (PR-XX) once that section exists. A constraint covered by a Phase Rule should NOT be reported as a gap.

**Confidence:** HIGH -- minimal structural change, natural extension of existing logic.

---

## Q3: SR Coverage Verification

### Analysis

System Rules (SR) in PROJECT.md are project-wide invariants. Example: "SR-01: 5s gRPC timeout for all services."

**Should the validator check that CASES.md references each applicable SR?**

Arguments for:
- Ensures completeness -- every phase acknowledges system-wide constraints
- Catches phases that violate system rules unknowingly

Arguments against:
- SRs are inherited by default -- requiring explicit acknowledgment is bureaucratic overhead
- The validator would need to determine which SRs apply to which phase (not all SRs apply everywhere)
- PROJECT.md is a living document -- SR numbering may change, creating stale references
- The planner and executor already read PROJECT.md directly; CASES.md does not need to duplicate it
- Risk of noise: a phase with 5 SRs would always show 5 "SR not referenced" findings until every CASES.md is updated

**What happens if you DON'T verify?**

SRs live in PROJECT.md, which is a canonical reference that all downstream agents (planner, executor) must read per CLAUDE.md. A missed SR in CASES.md does not mean a missed SR in implementation -- the executor reads PROJECT.md independently. The gap is only in CASES.md completeness, not in implementation safety.

**What if an SR is violated in CASES.md?**

This is a consistency issue, not a coverage issue. If a phase's cases specify a 10-second timeout but SR-01 says 5 seconds, that is a contradiction. Check C (Consistency) already catches cross-cutting inconsistencies. The validator could check CASES.md values against PROJECT.md SRs as part of Check C without requiring explicit SR references.

### Recommendation: Do NOT verify SR coverage

SRs are inherited by default. Requiring explicit acknowledgment adds noise without safety benefit. Instead:

1. **Phase Rules (PR) section references applicable SRs by inheritance:** The CASES.md Phase Rules section may optionally note "Per SR-XX" when a Phase Rule derives from a System Rule. This is documentation, not a validator-checked requirement.

2. **Check C gains SR contradiction detection:** When checking consistency, the validator cross-references any numerical values in CASES.md (timeouts, TTLs, limits) against PROJECT.md System Rules to detect contradictions. This is a lightweight addition to the existing consistency check.

3. **The /case discussion step references SRs proactively:** When step-discuss encounters an operation with gRPC calls, it should remind the developer "Per SR-01, gRPC timeout is 5 seconds" and propose it as a Rule. This is a step-discuss improvement, not a validator check.

**Confidence:** HIGH -- clear analysis of cost vs benefit. The validator checking SR references would produce noise without preventing real issues.

---

## Q4: CASES.md Phase Rules Section

### Placement

Phase Rules go AFTER the document header and BEFORE the first operation. This is the natural position because:
- Phase Rules apply to all operations -- reading them first establishes context
- They parallel how CONTEXT.md structures its decisions (phase-wide before operation-specific)
- The developer reads them as "ground rules" before diving into individual operations

### Format

```markdown
# Phase [XX]: [Name] - Behavioral Cases

**Discovered:** [date]
**Operations covered:** [count]
**Total cases:** S:[count] F:[count] E:[count] Q:[count]

## Phase Rules

> Constraints that apply across ALL operations in this phase. Individual operation Rules
> (R1, R2...) document operation-specific constraints.

- PR-01: [constraint description] (D-XX)
- PR-02: [constraint description] (D-XX, D-YY)
- PR-03: [constraint description]. Per SR-XX: [system rule it derives from]

---

## Operation: [OperationName]
...
```

**Numbering:** PR-01, PR-02, etc. The "PR-" prefix distinguishes from operation-level R1, R2 and (future) system-level SR-01.

**Source references:** Each PR cites its source decision (D-XX) in parentheses. If it derives from a System Rule, append "Per SR-XX" with a brief note.

**No table format:** Rules are constraints expressed in natural language. A numbered list is more readable than a table for prose constraints. This matches the existing operation-level Rules format (also a list).

### What Qualifies as a Phase Rule?

A decision becomes a Phase Rule when:
1. It applies to ALL or MOST operations in the phase (not just one)
2. It is an architectural constraint (per Q1 heuristic), not a behavioral decision
3. It cannot be attributed to a single operation's Rules section

Examples from 3A:
- PR-01: Ceremony state content MUST NOT appear in any response body or error detail (D-98) -- applies to RegisterBegin, RegisterFinish, LoginBegin, LoginFinish
- PR-02: All auth failures return generic error messages; no specific failure reason disclosed to clients (D-50, D-51, D-52, D-53) -- applies to all operations
- PR-03: Per-endpoint gRPC timeout: 5 seconds for all Auth -> User service calls (D-135) -- applies to RegisterFinish, LoginFinish, GetCurrentUser

Counter-examples (remain as operation-level Rules):
- Token consume-first ordering (D-03 in RegisterBegin) -- specific to one operation
- Handle case-insensitive uniqueness (D-57 in RegisterBegin) -- specific to registration
- Per-session lock on refresh (RefreshToken R2) -- specific to one operation

### Impact on CASE-SCRATCH.md

CASE-SCRATCH.md is the intermediate file used during discussion. It should also gain a Phase Rules section, placed at the top before individual operations. The /case orchestrator creates this section:

1. After the briefer runs and before discussion begins, the orchestrator scans CONTEXT.md for cross-cutting architectural constraints
2. Proposes an initial set of Phase Rules to the developer
3. Stores confirmed Phase Rules at the top of CASE-SCRATCH.md
4. During per-operation discussion, any constraint discovered that applies to multiple operations gets promoted to Phase Rules

**CASE-SCRATCH.md header with Phase Rules:**
```markdown
# Case Scratch: Phase [XX] - [Name]

## Phase Rules
- PR-01: [constraint] (D-XX)
- PR-02: [constraint] (D-XX)

---

## Operation: [OperationName]
...
```

### Impact on step-discuss

Step-discuss currently builds Rules per operation in step 3a (flow diagram). With Phase Rules:

1. **Before first operation discussion:** The orchestrator presents proposed Phase Rules from CONTEXT.md scan. Developer confirms, modifies, or defers.
2. **During operation discussion:** When proposing Rules, the AI checks if a constraint already exists as a Phase Rule. If so, it references "Per PR-01" instead of duplicating. If a new constraint is discovered that applies to multiple operations, the AI proposes promoting it to a Phase Rule.
3. **During finalize (step-finalize):** Cross-operation analysis may surface additional Phase Rules from patterns detected across operations.

### Impact on step-finalize output_format

The output_format in step-finalize.md gains the Phase Rules section. The change is additive:

```markdown
## Phase Rules

> Constraints that apply across ALL operations in this phase.

- PR-01: [constraint] (D-XX)
- PR-02: [constraint] (D-XX)
```

Inserted between the document header and the first `## Operation:` section.

### How Phase Rules interact with the validator

When the validator (Check B) encounters an unmatched architectural constraint:
1. Check if it appears in any operation's Rules -> covered
2. Check if it appears in Phase Rules -> covered
3. Neither -> Constraint Forwarding Gap

This means adding Phase Rules reduces Constraint Forwarding Gap findings by providing a correct home for cross-cutting constraints.

**Confidence:** HIGH -- natural extension of existing format, follows established R1/R2 precedent.

---

## Q5: Validator Output Format for New Category

### Finding Format

```markdown
## Constraint Forwarding Gaps (CONTEXT.md constraint with no covering Rule)

1. **D-XX: [constraint summary]**
   Source: CONTEXT.md
   Quote: "[relevant text from decision]"
   Scope: [Phase-wide / specific operations: Op1, Op2, Op3]
   No Rule in CASES.md documents this constraint.
   Suggested action: Add as [PR-XX in Phase Rules / R-N in OperationName Rules]

2. **D-XX, D-YY: [constraint cluster summary]**
   Source: CONTEXT.md
   Quote: "[relevant text]"
   Scope: Phase-wide
   These related constraints are not reflected in any Phase Rule or operation Rule.
   Suggested action: Add as PR-XX: "[proposed rule text]"
```

If no findings: include heading with "None found."

### Key Differences from Decision Gaps

| Aspect | Decision Gap | Constraint Forwarding Gap |
|--------|-------------|--------------------------|
| Source | Behavioral decision | Architectural constraint |
| What's missing | A case (S/F/E row) | A Rule (R-N or PR-XX) |
| Developer action | Add a case to the table | Add a Rule to the list |
| Suggested format | `F3 [description] -> [expected outcome]` | `PR-XX: [constraint text] (D-XX)` |
| Severity | High (security/auth) / Medium (other) | Medium (all) |
| Urgency | High -- untested behavior | Medium -- undocumented but implementable constraint |

### Severity Classification

Constraint Forwarding Gaps are uniformly **Medium** severity. Rationale:
- They document constraints that will likely be implemented correctly anyway (the developer made the decision in CONTEXT.md and the executor reads CONTEXT.md)
- The risk is not "wrong behavior" but "invisible constraint" -- if CASES.md is the sole reference for test generation, the constraint might not be verified
- This is less urgent than a Decision Gap (missing case = untested behavior)

Exception: Security constraints (like D-98 "state MUST NOT leak") should be **High** severity even as Constraint Forwarding Gaps, because security invariants must be explicitly documented and tested. The validator should detect security-related keywords ("MUST NOT", "never exposed", "not disclosed", "security") and elevate those to High.

### How the /case orchestrator presents these

The orchestrator already presents validator findings one-by-one for developer confirmation. For Constraint Forwarding Gaps, the presentation differs:

```
The validation found [N] items to review:

[1] Decision Gap: D-03 in RegisterFinish
    "single-use, 30-minute expiry" -- no failure case for reused token
    Suggested case: F_ Reused invite token -> invite_invalid
    -> Add this case?

[2] Constraint Forwarding: D-98 (Phase-wide)
    "Ceremony state MUST NOT appear in any response body or error detail"
    Not documented as a Phase Rule or operation Rule.
    Suggested action: Add as PR-01 in Phase Rules
    -> Add this rule?

[3] Constraint Forwarding: D-135 (Phase-wide)
    "Per-endpoint gRPC timeout: 5 seconds"
    Not documented as a Phase Rule.
    Suggested action: Add as PR-XX in Phase Rules
    -> Add this rule?
```

The key UX difference: for Decision Gaps, the developer confirms "add this case" (modifies a table). For Constraint Forwarding Gaps, the developer confirms "add this rule" (adds a line to a list). Both are quick confirmation actions.

### Finding Cap Interaction

The 15-finding cap spans all categories with this priority ordering:

1. Requirement Gaps (highest)
2. Decision Gaps
3. Constraint Forwarding Gaps (security-elevated ones rank with Decision Gaps)
4. Consistency Issues
5. Completeness Gaps
6. Briefing Gaps (lowest)

In practice, Constraint Forwarding Gaps should be few per phase (5-10 cross-cutting constraints typical), so they are unlikely to consume a large share of the cap.

**Confidence:** HIGH -- follows existing patterns, clear differentiation from Decision Gaps.

---

## Integration Summary: All Changes Required

### Files to modify

| File | Change | Impact |
|------|--------|--------|
| `.claude/agents/case-validator.md` | Extend Check B with classification heuristic and two sub-categories; add Constraint Forwarding Gaps output section; update severity table; update return protocol counter; add Phase Rules to coverage scope; update quality gate | Primary change |
| `.claude/skills/case/step-finalize.md` | Add Phase Rules section to output_format template; update validator finding presentation to distinguish Decision Gaps from Constraint Forwarding Gaps | Output format |
| `.claude/skills/case/SKILL.md` | Add Phase Rules section to output_format example; note PR-XX numbering convention | Output format |
| `.claude/skills/case/step-discuss.md` | Add Phase Rules proposal before first operation discussion (scan CONTEXT.md for cross-cutting constraints); add "Per PR-XX" reference pattern for operation Rules | Discussion flow |
| `.claude/skills/case/step-init.md` | Add Phase Rules initialization to CASE-SCRATCH.md header | Scratch format |
| `.planning/PROJECT.md` | Add "System-Wide Operational Rules" section with SR-XX numbering (future, not required for validator change) | System Rules |

### Implementation Order

1. **case-validator.md** -- core logic change (Check B extension + new output section)
2. **step-finalize.md** -- output format (Phase Rules section + updated validator presentation)
3. **SKILL.md** -- output format consistency
4. **step-discuss.md** -- Phase Rules proposal during discussion
5. **step-init.md** -- CASE-SCRATCH.md header update
6. **PROJECT.md SR section** -- deferred (not required for validator to work, but establishes the top tier of the hierarchy)

### What NOT to change

- **Finding cap:** Remains 15
- **5-check structure:** Remains 5 checks (A through E)
- **Dispatch pattern in step-finalize.md:** No change to Agent call -- the validator reads the same files
- **Return protocol structure:** Same format, one new counter
- **Check A, C, D, E:** No changes

---

## Open Questions

### OQ-1: When to introduce SR numbering in PROJECT.md

System Rules (SR-XX) in PROJECT.md are the top tier of the hierarchy but PROJECT.md currently has no numbered rules section. Adding it is valuable for long-term consistency but not strictly required for the validator changes to work. Phase Rules can reference decisions directly (D-XX) without an SR intermediary.

**Recommendation:** Defer PROJECT.md SR section to a separate, small change. The validator and output format changes work without it. When SRs are added, Phase Rules can be updated to reference them.

### OQ-2: Retroactive application to 3A CASE-SCRATCH.md

The 3A CASE-SCRATCH.md was written before Phase Rules existed. Should it be updated to extract cross-cutting constraints into a Phase Rules section?

**Recommendation:** Yes, but as part of the next /case session or plan-phase for 3A, not as a standalone task. The constraints are already in operation-level Rules (e.g., D-98 appears in RegisterBegin R12, RegisterFinish R10, LoginBegin R5, LoginFinish R8). Promoting them to Phase Rules removes duplication.

### OQ-3: Validator model complexity

The classification heuristic adds branching logic to Check B. The validator agent (currently opus model) should handle this without difficulty, but the heuristic should be expressed as a clear decision tree in the agent definition, not as prose that the model must interpret.

**Recommendation:** Include the decision tree from Q1 verbatim in the agent definition. Concrete examples from 3A (both behavioral and architectural) serve as few-shot guidance.

---

## Sources

### Primary (HIGH confidence)
- `.claude/agents/case-validator.md` -- current validator agent definition
- `.claude/skills/case/step-finalize.md` -- finalize step with output format
- `.claude/skills/case/SKILL.md` -- main skill definition with formatting rules
- `.claude/skills/case/step-discuss.md` -- per-operation discussion flow
- `.planning/phases/03a-authentication-core/CASE-SCRATCH.md` -- real 3A scratch output (40+ decisions classified)
- `.planning/phases/03a-authentication-core/03a-CONTEXT.md` -- real 3A decisions (source for classification analysis)
- `.planning/phases/03a-authentication-core/CASE-BRIEFING.md` -- real 3A briefer output

### Secondary (HIGH confidence)
- `.planning/research/CASE-VALIDATOR-REDESIGN.md` -- prior validator redesign research
- Memory: `feedback_case_constraint_forwarding.md` -- original problem statement
- Memory: `project_system_rules_promote.md` -- 3-tier hierarchy proposal

---

*Research completed: 2026-03-26*
*Valid until: 2026-06-26 (stable domain, methodology-focused)*
