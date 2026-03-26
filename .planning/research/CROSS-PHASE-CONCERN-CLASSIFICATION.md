# Cross-Phase Concern Identification and Classification Criteria

**Researched:** 2026-03-26
**Domain:** /case skill pipeline, GSD workflow cross-phase coordination
**Confidence:** HIGH (analysis of real artifacts from 3A/3B, existing pipeline, memory records)
**Inputs:** 3A CASE-SCRATCH.md, 3A/3B CONTEXT.md, ROADMAP.md, memory records, existing constraint forwarding research

---

## Executive Summary

This research analyzes the problem of cross-phase concern identification -- how items discovered during one phase's `/case` session should be forwarded to downstream phases. The problem has four distinct sources: CONTEXT.md deferred items, CASES.md Open Questions targeting downstream phases, Phase Rules that downstream phases should inherit, and implicit behavioral concerns not explicitly tagged but logically affecting downstream phases.

Analysis of 3A/3B artifacts reveals that CONTEXT.md deferred items and CASES.md Open Questions are complementary (not redundant): CONTEXT.md deferred items are design decisions deferred BEFORE case discovery, while Open Questions emerge DURING case discovery. Both are valid forwarding sources. The most dangerous category is implicit concerns -- behavioral implications discoverable during case discussion but not tagged for forwarding (e.g., "JWT carries session-cached data" in 3A RefreshToken R4 affects 3B's user modification operations).

**Primary recommendation:** Implement a 5-category concern taxonomy with a classification decision tree, a `Forward` column in the Open Questions table, an `## Inherited Concerns` section in the receiving phase's CASE-BRIEFING.md, and developer-confirmed resolution during the receiving phase's `/case` session.

---

## 1. Concern Taxonomy

Five categories of cross-phase concerns, derived from real 3A/3B evidence.

### Category 1: Explicit Deferred Items (CONTEXT.md)

**Source:** CONTEXT.md `<deferred>` section, produced during `gsd:discuss`.
**Timing:** Exists BEFORE `/case` runs.
**Direction:** Always forward (they exist specifically because they were deferred).

**3A/3B evidence:**
- D-02: "Role-selectable invite creation and admin invite management deferred to 3B"
- D-09: "Admin query/cancel endpoints deferred to 3B"
- D-36: "Logout all sessions deferred to 3B (verified tier)"
- D-55: "Rate limiting: deferred (out of scope)"
- Recovery flow decisions (D-22, D-23, D-104, D-105) explicitly deferred to 3B

**Characteristics:**
- Explicitly tagged with phase target ("deferred to 3B", "Phase 3B", "v2+")
- Already documented in a well-defined location
- High visibility -- anyone reading CONTEXT.md sees them

**Current forwarding mechanism:** None systematic. The 3B CONTEXT.md was written by the same human during the same discuss session, so the deferred items were manually carried forward. For phases discussed in different sessions, there is no guarantee the receiving phase's discuss session will see the originating phase's deferred items.

### Category 2: Targeted Open Questions (CASES.md)

**Source:** CASES.md Open Questions, produced during `/case`.
**Timing:** Discovered DURING case discussion.
**Direction:** Forward when the question targets a specific downstream phase.

**3A evidence:**
- Q2 (RegisterBegin): "Design Register ceremony logic to be reusable for 3B passkey addition" -- explicitly targets 3B's architecture
- RefreshToken Q1: "Cache/lock storage: Redis vs in-memory with swappable abstraction" -- self-contained, resolve before 3A's own planning

**Characteristics:**
- Contain a direct reference to a downstream phase or feature ("3B", "when X is added")
- The question cannot be resolved in the current phase because the downstream phase's scope is needed
- Distinguished from self-contained questions by the presence of a phase/feature reference

### Category 3: Phase Rules with Downstream Relevance

**Source:** CASES.md Phase Rules (PR section), produced during `/case`.
**Timing:** Confirmed during Step 2.5 of case discussion.
**Direction:** Forward when a downstream phase shares the same operational domain.

**3A evidence (hypothetical -- PR system was implemented after 3A):**
- PR1: "Ceremony state content MUST NOT appear in any response body or error detail (D-98)" -- applies to 3B's verify ceremony and additional passkey registration ceremony
- PR2: "All auth failures return generic 'unauthorized' to client (D-50)" -- applies to 3B's recovery flow, API key auth
- SR-01: "5-second gRPC timeout" -- applies everywhere

**Characteristics:**
- The receiving phase operates in the same domain (both are auth phases)
- The rule constrains behavior that the receiving phase will implement
- Some rules (SRs) are already in PROJECT.md and inherited automatically
- Phase Rules are NOT in PROJECT.md and NOT automatically inherited

### Category 4: Implicit Behavioral Concerns

**Source:** Discovered during `/case` but NOT tagged for forwarding. Encoded in case details (Rules, Edge Cases, Side Effects) that have cross-phase implications.
**Timing:** Present in completed CASES.md but not identified as cross-phase.
**Direction:** Forward when a downstream phase introduces behavior that interacts with the concern.

**3A evidence (the canonical example):**
- RefreshToken R4: "JWT claims from session data -- freshness maintained by update operations, not RefreshToken. Design intent: when user info changes, the update operation writes to session data in Redis; RefreshToken trusts session data as-is."
- GetCurrentUser E1: "JWT claims stale -- name/role changed since login -> DB has latest values, not JWT claims"

**Cross-phase implication:** When 3B introduces user info modification (name change, role change), the modifying operation MUST update session data in Redis. If it doesn't, RefreshToken will issue JWTs with stale claims. This concern is NOT recorded anywhere as targeting 3B -- it is implicit in R4's design intent.

**Characteristics:**
- No explicit phase reference in the case text
- Requires understanding the interaction between phases to identify
- Most dangerous category because it is invisible to any automated scan
- Often embedded in "Design intent:" notes or Edge Case Expected Outcomes

### Category 5: Deferred-by-Scope Items (discovered during /case but out of phase scope)

**Source:** During `/case` discussion, the developer or AI identifies a concern that falls outside the current phase's boundary but is not explicitly deferred in CONTEXT.md.
**Timing:** Discovered DURING case discussion.
**Direction:** Forward to the appropriate phase.

**3A evidence:**
- During CreateInvite discussion, E1 notes "no rate limit (D-55 deferred)" -- this is actually a Category 1 item that was re-surfaced during case discussion
- RegisterBegin Q2 could also be viewed this way: the question emerged because case discussion revealed a code reuse opportunity spanning phases

**Characteristics:**
- Overlap with Categories 1 and 2 -- this is the "discovery moment" version of an explicit deferral
- Distinguishable from Category 4 because the developer explicitly acknowledges the cross-phase relevance during discussion

---

## 2. Classification Decision Tree

For any concern identified during or after `/case`, apply this decision tree to determine disposition.

```
Concern identified
    |
    v
[Does it reference a specific downstream phase or feature?]
  YES --> [Is it resolvable in the current phase?]
    |         YES --> Resolve before own planning (self-contained OQ)
    |         NO  --> FORWARD: Tag with target phase
  NO  --> continue
    |
    v
[Is it a deferred CONTEXT.md item?]
  YES --> FORWARD: Already tagged in <deferred> section
  NO  --> continue
    |
    v
[Is it a Phase Rule (PR) or System Rule (SR)?]
  YES --> [Does the downstream phase share the same domain?]
    |         YES --> INHERIT: Include in downstream briefing
    |         NO  --> SKIP: Not relevant to downstream phase
  NO  --> continue
    |
    v
[Does the concern describe behavior that interacts with
 operations a downstream phase will introduce?]
  YES --> FORWARD: Implicit concern (Category 4)
          Tag with: target phase, interacting operation, risk
  NO  --> SKIP: Self-contained to current phase
```

### Classification Signals for Open Questions

| Signal | Classification | Example |
|--------|---------------|---------|
| Contains "Phase X", "when X is added", "3B" | Targeted OQ (Category 2) | Q2: "reusable for 3B passkey addition" |
| Asks about a value/parameter within current scope | Self-contained OQ | Q1: "max length for handle and name" |
| Contains "Design intent:" note | Check for Category 4 implicit concern | R4: "update operations write to session data" |
| Developer says "that's for later" | Category 5 deferred-by-scope | Rate limiting during CreateInvite discussion |
| Uses "project policy" language | SR-candidate (not cross-phase forward) | "same timeout everywhere" |

### Who Classifies?

**During `/case` discussion (primary):** The Protester (AI) proposes classification during Step 3e (save to scratch). After completing an operation's cases, the AI reviews Open Questions and explicitly asks:

```
Q2 targets 3B (passkey addition). I'll mark it [->3B] for forwarding.
Q1 is self-contained for this phase's planning.
Correct?
```

**During cross-operation analysis (Step 4):** The Protester reviews all operations' Rules and Edge Cases for implicit concerns (Category 4). This is the hardest classification and benefits from the AI's ability to cross-reference operation interactions.

**Developer override:** The developer can always reclassify. If the AI marks something self-contained, the developer can say "actually, that affects 3B" and vice versa.

---

## 3. CONTEXT.md Deferred Items vs CASES.md Open Questions

### Relationship Analysis

| Property | CONTEXT.md Deferred | CASES.md Open Questions |
|----------|-------------------|----------------------|
| **When produced** | During `gsd:discuss` (before /case) | During `/case` (before plan) |
| **Granularity** | Feature/decision level ("recovery flow deferred to 3B") | Behavioral/technical level ("should ceremony logic be reusable?") |
| **Audience** | Future discuss session (design decisions) | Future plan session (implementation details) |
| **Certainty** | Definite deferral (explicit decision to not address now) | Open question (may resolve here or forward) |
| **Location** | `<deferred>` section, highly visible | Per-operation table, may be buried |

### Are They Redundant?

**No.** They are complementary and operate at different levels of abstraction.

CONTEXT.md deferred items answer: "What design decisions are we NOT making in this phase?"
CASES.md Open Questions answer: "What behavioral details remain unclear for the operations we ARE building?"

A single concern may appear in both:
- CONTEXT.md defers "recovery flow" to 3B (feature-level)
- CASES.md Q2 asks "should registration ceremony be reusable for 3B passkey addition?" (architecture-level, discovered during case discussion for a 3A operation)

These are different facets of the same concern -- the CONTEXT.md item is broader (entire flow), while the OQ is specific (one code architecture question about reuse).

### Should Both Be Forwarding Sources?

**Yes.** Each captures concerns at different points in the pipeline:

```
gsd:discuss         /case               gsd:plan
    |                  |                    |
    v                  v                    v
CONTEXT.md       CASES.md              PLAN.md
deferred items   Open Questions        (consumes both)
(Category 1)     (Category 2, 4, 5)
```

A concern NOT in CONTEXT.md deferred may still be discovered during /case. A concern in CONTEXT.md deferred may need refinement during /case (from "recovery flow" to specific Q about ceremony reuse). Both forwarding paths are needed.

### What About Deferred Items That /case Didn't Discover?

This is a real gap. CONTEXT.md may have deferred items that /case never encounters because:
- The deferred feature has no interaction with current phase operations
- The AI didn't probe deep enough during case discussion

**Example:** CONTEXT.md defers "JWT key rotation with kid claim" -- this has zero interaction with 3A operations and would never surface during case discussion. But when a future phase implements key rotation, it needs to know that 3A's JWT implementation assumed a single key pair (D-45).

**Mitigation:** The case-briefer already reads CONTEXT.md. Adding a "Deferred Items with Downstream Impact" section to the briefing would surface these for the Protester to consider. The Protester can then ask: "CONTEXT.md defers X. Do any current operations have assumptions that X would invalidate?"

---

## 4. Forwarded Concern Lifecycle

### 4a: When a Concern Arrives at the Receiving Phase

**Recommended lifecycle: Surface -> Review -> Classify -> Integrate or Dismiss**

```
Forwarded concern arrives (via briefer scan)
    |
    v
[case-briefer reads dependency phase's artifacts]
    |
    v
Briefer produces "## Inherited Concerns" section in CASE-BRIEFING.md
    |
    v
[/case Step 2.5: Protester presents inherited concerns alongside Phase Rules]
    |
    v
Developer reviews each concern:
    |
    ├── "Relevant, needs a case" --> Convert to Open Question or Case in target operation
    ├── "Relevant, needs a rule" --> Add as Phase Rule (PR) or operation Rule (R)
    ├── "Already handled" --> Mark as resolved with reference
    └── "Not relevant" --> Dismiss with justification
```

**What NOT to do:**
- **Auto-convert to a case** -- context from the originating phase may be wrong or stale. The receiving phase's developer must validate.
- **Auto-insert as a Rule** -- same risk. The constraint may not apply in the receiving phase's context.
- **Silently inherit** -- invisible concerns are exactly the problem. Every forwarded concern must be explicitly acknowledged.

### 4b: Integration Formats

When a developer confirms a forwarded concern:

**As an Open Question:**
```markdown
### Open Questions
| ID | Question | Impact | Default Recommendation | Source |
|----|----------|--------|------------------------|--------|
| Q1 | How should user info modification update session data for RefreshToken? | JWT claim freshness | Update Redis session on each modification | Inherited: 3A RefreshToken R4 |
```

**As a Phase Rule:**
```markdown
## Phase Rules
- PR1: Session data in Redis MUST be updated when user info changes, ensuring RefreshToken issues fresh JWTs. Inherited from 3A RefreshToken R4 (D-37)
```

**As an operation Rule:**
```markdown
### Rules
- R3: After updating user name/role, write updated values to user's active sessions in Redis. Inherited: 3A RefreshToken R4
```

### 4c: Back-Propagation

**When the receiving phase resolves a forwarded concern, should the originating phase's CASES.md be updated?**

**No. Treat completed phase artifacts as read-only archives.**

Rationale:
1. The originating phase has already been planned, executed, verified, and shipped (or is in progress). Modifying its CASES.md creates confusion about what was actually implemented vs what was retroactively documented.
2. The resolution belongs to the receiving phase -- it is that phase's decision and implementation.
3. The "living spec" idea (consolidating per-service specs across phases) is deferred per `project_unified_spec_idea.md`. Cross-referencing via "Inherited from 3A" annotations is sufficient for traceability.

**Exception:** If the originating phase is NOT yet complete (parallel development or re-planning), updating its artifacts is appropriate. But this is a normal iterative workflow, not a back-propagation mechanism.

### 4d: Handling Irrelevant Forwarded Concerns

If a forwarded concern turns out to be irrelevant in the receiving phase:

```markdown
## Inherited Concerns (from dependency phases)

| # | Source | Concern | Disposition |
|---|--------|---------|-------------|
| 1 | 3A RefreshToken R4 | Session data freshness when user info changes | Integrated as PR3 |
| 2 | 3A RegisterBegin Q2 | Ceremony logic reuse for passkey addition | Integrated as design constraint in AddPasskey operation |
| 3 | 3A CONTEXT.md deferred | Rate limiting on auth endpoints | Dismissed: out of 3B scope, deferred to infrastructure phase |
```

Document the dismissal -- it shows the concern was considered, not overlooked.

---

## 5. Priority/Severity Propagation Rules

### Does Priority Transfer?

**No. Priority resets in the receiving phase.** The receiving phase's developer evaluates the concern in their own context.

Rationale:
- A `must` case in Phase A may have an Open Question that, in Phase B's context, is merely `should` -- the behavior is desirable but not security-critical in B's scope.
- Conversely, a `should` edge case in Phase A may become `must` in Phase B if it represents a security boundary in B's operations.

### Priority Guidance

| Originating Priority | Arrives As | Receiving Phase Re-evaluates |
|---------------------|-----------|------------------------------|
| must (security concern) | Flagged as security-elevated | Developer confirms must or downgrades with justification |
| must (data integrity) | Normal forwarded concern | Developer evaluates impact in own context |
| should/could | Normal forwarded concern | Developer evaluates independently |

### Security Elevation

When a forwarded concern originated from a security-sensitive context (auth failure, credential handling, session integrity), it arrives with a security flag:

```markdown
| # | Source | Concern | Security | Disposition |
|---|--------|---------|----------|-------------|
| 1 | 3A RefreshToken R4 | Session data freshness | YES (JWT claim integrity) | ... |
| 2 | 3A RegisterBegin Q2 | Ceremony reuse | NO (architecture question) | ... |
```

The security flag means the developer must explicitly acknowledge the concern -- it cannot be silently dismissed. Dismissal requires a justification note.

---

## 6. Proposed Tagging Format

### Open Questions Table Enhancement

Current format:
```markdown
| ID | Question | Impact | Default Recommendation |
```

Proposed format:
```markdown
| ID | Question | Impact | Default Recommendation | Forward |
```

The `Forward` column values:
- Empty (blank) -- self-contained, resolve before own planning
- `->3B` -- forward to Phase 3B
- `->3B:AddPasskey` -- forward to Phase 3B, specifically relevant to AddPasskey operation
- `->catalog` -- forward to catalog phase (when exact number unknown)
- `->PROJECT` -- promote to PROJECT.md (system-wide concern, not phase-specific)

### Examples from 3A

```markdown
### Open Questions
| ID | Question | Impact | Default Recommendation | Forward |
|----|----------|--------|------------------------|---------|
| Q1 | Max length for handle and name | Boundary tests, DB column sizing | Planner discretion (handle: 32, name: 64) | |
| Q2 | Register ceremony reusable for 3B passkey addition | Architecture, code reuse | Separate ceremony logic from registration orchestration | ->3B:AddPasskey |
```

### Implicit Concerns Section

For Category 4 implicit concerns (not Open Questions, but behavioral implications), add a section after Open Questions in CASE-SCRATCH.md and CASES.md:

```markdown
### Cross-Phase Implications
| ID | Concern | Affects | When Triggered | Forward |
|----|---------|---------|----------------|---------|
| X1 | RefreshToken trusts session data as-is (R4). Modifying user info without updating session data produces stale JWT claims | User info modification operations | When 3B adds name/role change | ->3B:UpdateUser |
| X2 | GetCurrentUser reads from DB, not JWT claims (E1). DB and JWT can be inconsistent between refresh cycles | Any operation that modifies user data | When 3B adds user modification | ->3B |
```

The `X` prefix distinguishes cross-phase implications from Open Questions (`Q`), Cases (`S/F/E`), and Rules (`R/PR/SR`).

### Where to Store Forward Items

**In the originating phase:** Open Questions get a Forward column. Implicit concerns get a Cross-Phase Implications section.

**In the receiving phase:** The case-briefer scans dependency phases' CASES.md for items with forward tags. These appear in the briefing's `## Inherited Concerns` section.

---

## 7. Implementation: Pipeline Changes

### 7a: case-briefer Enhancement

**Current:** Reads CONTEXT.md and produces per-operation briefing.
**Proposed addition:** Also read dependency phase's CASES.md (identified via ROADMAP.md `Depends on`). Extract:
1. Open Questions with `Forward` column targeting this phase
2. Cross-Phase Implications (`X` items) targeting this phase
3. Phase Rules from dependency phase (for inheritance consideration)

**New briefing section:**
```markdown
## Inherited Concerns

### From Phase 3A (Authentication Core)

| # | Type | Source | Concern | Target |
|---|------|--------|---------|--------|
| 1 | Open Question | RegisterBegin Q2 | Ceremony logic reusable for passkey addition | AddPasskey operation |
| 2 | Implicit | RefreshToken X1 | Session data freshness on user info modification | UpdateUser, RoleChange |
| 3 | Phase Rule | PR1 (D-98) | Ceremony state MUST NOT appear in response bodies | Verify ceremony, AddPasskey ceremony |
| 4 | Deferred | CONTEXT.md | Admin invite management with role selection | CreateInvite (extended) |

### From CONTEXT.md Deferred Items

| # | Decision | Deferred Feature | Relevant Operations |
|---|----------|-----------------|---------------------|
| 1 | D-02 | Role-selectable invite creation | CreateInvite |
| 2 | D-09 | Admin invite query/cancel | ListInvites, CancelInvite |
| 3 | D-36 | Logout all sessions | LogoutAll |
```

### 7b: step-discuss Enhancement (Step 2.5)

**Current Step 2.5:** Presents Phase Rules for confirmation.
**Proposed extension:** Also present Inherited Concerns from the briefing.

```
From the briefing, I identified these inherited concerns from Phase 3A:

Inherited Concerns:
  1. [OQ] Ceremony logic reuse for passkey addition (3A RegisterBegin Q2)
     -> Relevant to AddPasskey operation in this phase
  2. [Implicit] Session data freshness on user info modification (3A RefreshToken R4)
     -> Relevant to any operation that modifies user name/role
  3. [PR] Ceremony state no-leak (3A PR1/D-98)
     -> Applies to verify and additional passkey registration ceremonies
  4. [Deferred] Role-selectable invite creation (3A D-02)
     -> Already captured in this phase's CONTEXT.md decisions

Confirm which concerns to integrate? Any to dismiss?
```

### 7c: step-discuss Enhancement (Step 3e)

**Current Step 3e:** Save operation cases to CASE-SCRATCH.md.
**Proposed extension:** After saving Open Questions, the AI reviews for cross-phase implications.

```
Looking at the cases we just discussed for UpdateUser:

The session data freshness concern from 3A (RefreshToken R4) means UpdateUser
must write updated name/role to the user's active sessions in Redis.

I'll add this as R3 in UpdateUser's Rules:
  R3: After successful update, write new user data to active sessions in Redis.
      Inherited: 3A RefreshToken R4 (D-37)

Confirm?
```

### 7d: Open Questions Format Change

Add `Forward` column to the Open Questions table in both CASE-SCRATCH.md and CASES.md output format.

**step-finalize.md output format update:**
```markdown
### Open Questions

| ID | Question | Impact | Default Recommendation | Forward |
|----|----------|--------|------------------------|---------|
| Q1 | [question] | [impact] | [recommendation] | |
| Q2 | [question] | [impact] | [recommendation] | ->4:BookCreate |
```

### 7e: Cross-Phase Implications Section

Add to CASES.md output format, after the last operation and before SR Candidates:

```markdown
## Cross-Phase Implications

> Behavioral concerns in this phase that affect downstream phases.
> Tagged with target phase. Case-briefer scans this section for dependency phases.

| ID | Concern | Source Rule/Case | Affects Phase | Trigger |
|----|---------|-----------------|---------------|---------|
| X1 | [description] | [Rule ID] | [target] | [when the concern activates] |

> If none: "None identified."
```

---

## 8. Discovery Mechanism: How the Briefer Finds Dependency Artifacts

### ROADMAP.md as the Dependency Graph

Each phase in ROADMAP.md has a `Depends on` field:
- Phase 3A: "Requires Phase 1, Phase 2"
- Phase 3B: "Requires Phase 3A"
- Phase 4: "Requires Phase 1"

The briefer reads ROADMAP.md to identify dependency phases, then reads each dependency phase's CASES.md (if it exists) for forwarded concerns.

### Scan Algorithm

```
For each dependency phase in ROADMAP.md:
  1. Read {dep_phase_dir}/*-CASES.md
  2. Extract Open Questions with Forward column matching this phase
  3. Extract Cross-Phase Implications (X items) matching this phase
  4. Extract Phase Rules (PR) for inheritance consideration
  5. Read {dep_phase_dir}/*-CONTEXT.md <deferred> section
  6. Extract deferred items that match this phase's scope
```

### What If CASES.md Doesn't Exist for a Dependency?

If a dependency phase was completed without /case (e.g., Phase 1, which was done before /case was implemented), the briefer falls back to reading CONTEXT.md only. No forwarded concerns from CASES.md, but deferred items from CONTEXT.md are still available.

---

## 9. Edge Cases and Risks

### Risk: Over-Forwarding

If every edge case and design intent note gets forwarded, receiving phases drown in inherited concerns.

**Mitigation:** Only forward items that meet BOTH criteria:
1. The concern describes behavior that a downstream phase's operations will directly interact with
2. The concern cannot be inferred from the downstream phase's own CONTEXT.md decisions alone

### Risk: Stale Forwards

An originating phase forwards a concern, but by the time the receiving phase runs /case, the concern has been resolved through implementation.

**Mitigation:** The receiving phase's developer dismisses resolved concerns during Step 2.5 review. The lifecycle model (Section 4) handles this via "Already handled" disposition.

### Risk: Implicit Concern Discovery Failure

Category 4 concerns are the hardest to identify. The AI may miss them.

**Mitigation:** Three layers of defense:
1. The Protester asks "Do any of this operation's design assumptions depend on downstream phases maintaining certain behaviors?" at the end of each operation discussion
2. Cross-operation analysis (Step 4) reviews all operations for cross-phase implications
3. The receiving phase's briefer can flag "Design intent:" notes in dependency CASES.md that mention concepts relevant to the receiving phase's scope

### Risk: Circular Forwarding

Phase A forwards to Phase B, which discovers a concern that forwards back to Phase A.

**Mitigation:** Phase A is already completed (or further along in the pipeline). Back-propagation is handled through the iterative workflow ("any step can return to an earlier step"), not through the forwarding mechanism. If the concern is critical enough to modify Phase A, that requires a deliberate re-planning decision by the developer.

---

## 10. Concrete 3A -> 3B Forward Items

Applying the taxonomy to real 3A artifacts, these items should forward to 3B:

### From Open Questions (Category 2)

| Source | Concern | Forward Target |
|--------|---------|---------------|
| RegisterBegin Q2 | Ceremony logic reusable for passkey addition | ->3B:AddPasskey |

### From Implicit Concerns (Category 4)

| Source | Concern | Forward Target |
|--------|---------|---------------|
| RefreshToken R4 | Session data freshness: modifying user info must update Redis session data or JWT claims go stale | ->3B:UpdateUser, RoleChange |
| GetCurrentUser E1 | DB vs JWT claim inconsistency during refresh window | ->3B:UpdateUser (informational, not actionable -- DB is already authoritative) |
| Logout R1 | "logout-all is 3B" -- explicitly scoped deferral | ->3B:LogoutAll |

### From Phase Rules (Category 3)

| Source | Concern | Forward Target |
|--------|---------|---------------|
| PR1 (D-98) | Ceremony state no-leak | ->3B:VerifyBegin, VerifyFinish, AddPasskeyBegin, AddPasskeyFinish |
| PR2 (D-50) | Generic auth failure errors | ->3B:Recover, all auth operations |
| SR-01 | 5s gRPC timeout | ->3B:all cross-service operations |

### From CONTEXT.md Deferred (Category 1)

| Source | Concern | Forward Target |
|--------|---------|---------------|
| D-02 | Role-selectable invite creation | ->3B:CreateInvite (admin) |
| D-09 | Admin invite query/cancel | ->3B:ListInvites, CancelInvite |
| D-36 | Logout all sessions | ->3B:LogoutAll |
| D-55 | Rate limiting | ->infrastructure (not 3B) |
| D-22/23/104/105 | Recovery flow | ->3B:Recover |

---

## 11. Summary of Proposed Changes

| File | Change | Purpose |
|------|--------|---------|
| CASES.md output format (step-finalize.md) | Add `Forward` column to Open Questions table | Tag targeted questions |
| CASES.md output format (step-finalize.md) | Add `## Cross-Phase Implications` section | Capture implicit concerns |
| case-briefer agent | Read dependency phases' CASES.md via ROADMAP dependency graph | Discover forwarded concerns |
| case-briefer output | Add `## Inherited Concerns` section to CASE-BRIEFING.md | Surface concerns for Protester |
| step-discuss (Step 2.5) | Present inherited concerns alongside Phase Rules | Developer review and integration |
| step-discuss (Step 3e) | AI reviews for cross-phase implications after each operation | Capture implicit concerns |
| step-discuss (Step 4) | Cross-operation analysis includes cross-phase implication scan | Catch remaining implicit concerns |

### What Does NOT Change

- CONTEXT.md format (deferred items remain as-is)
- Existing Open Questions format (Forward column is additive)
- Phase Rules / SR system (already implemented)
- Validator (does not check cross-phase forwarding -- too noisy)
- Back-propagation to originating phases (explicitly rejected)

---

## Confidence Assessment

| Area | Level | Reason |
|------|-------|--------|
| Concern taxonomy (5 categories) | HIGH | Derived from real 3A/3B artifacts with concrete examples for each category |
| Classification decision tree | HIGH | Tested against all 3A Open Questions and edge cases |
| CONTEXT.md vs CASES.md relationship | HIGH | Clear distinction: timing, granularity, audience |
| Lifecycle model (receive -> review -> integrate/dismiss) | HIGH | Follows existing confirmatory pattern from constraint forwarding |
| Priority propagation (reset, not transfer) | HIGH | Consistent with existing Phase Rule confirmation model |
| Tagging format (Forward column, X-items) | MEDIUM | Reasonable proposal but untested in practice |
| Briefer dependency scan | MEDIUM | Relies on ROADMAP dependency graph being accurate and CASES.md existing |
| Implicit concern discovery (Category 4) | LOW | Hardest category; relies on AI cross-referencing quality |

**Overall: HIGH** for the taxonomy and lifecycle. MEDIUM for the tagging format and pipeline changes. LOW for implicit concern discovery, which is fundamentally a hard problem that no automated system fully solves -- the developer remains the ultimate safety net.

---

## Open Questions (for this research)

### OQ-1: Briefer Model Capability

The case-briefer uses Sonnet. Can Sonnet reliably identify implicit cross-phase concerns (Category 4) by cross-referencing Design intent notes against the receiving phase's operation list? This may require a more capable model for the dependency scan step, or the dependency scan could be a separate agent call using Opus.

**Recommendation:** Start with Sonnet for explicit items (Categories 1-3, 5). For Category 4, add a heuristic: scan for "Design intent:" notes in dependency CASES.md and flag any that mention concepts appearing in the receiving phase's CONTEXT.md. If precision is too low, consider a targeted Opus scan.

### OQ-2: Scaling to Non-Adjacent Dependencies

Phase 3B depends on 3A (direct). But Phase 6 (User Preferences) depends on Phase 3 (auth) AND Phase 4 (catalog). Should Phase 6's briefer scan Phase 3A, 3B, 4, AND 5? Transitive dependencies could create a large scan surface.

**Recommendation:** Scan only direct dependencies (ROADMAP `Depends on` field). Transitive concerns should have been forwarded through the chain: if 3A forwards to 3B, and 3B has a concern for Phase 6, that concern should appear in 3B's Cross-Phase Implications, which Phase 6's briefer would pick up when scanning 3B.

### OQ-3: First Real Test

The 3A artifacts were written before this forwarding system was designed. The first real test will be when Phase 3B runs `/case`. At that point, 3A's artifacts are in the old format (no Forward column, no X-items). The briefer will need to fall back to heuristic scanning of 3A's Open Questions and Rules rather than structured forward tags.

**Recommendation:** Accept the bootstrapping problem. The briefer's heuristic scan (looking for phase references in Q text, Design intent notes, and explicit "3B" mentions) should catch the most important items. Future phases will have structured forward tags from the start.

---

## Sources

### Primary (HIGH confidence)
- `.planning/phases/03a-authentication-core/CASE-SCRATCH.md` -- 9 operations, 60+ rules, 3 Open Questions
- `.planning/phases/03a-authentication-core/03a-CONTEXT.md` -- 70+ decisions with deferred items
- `.planning/phases/03b-authentication-operations/03b-CONTEXT.md` -- downstream phase scope and decisions
- `.planning/ROADMAP.md` -- phase dependency structure
- `.planning/research/CASE-CONSTRAINT-FORWARDING-SUMMARY.md` -- existing constraint forwarding research
- `.planning/research/CASE-CONSTRAINT-FORWARDING-UPSTREAM.md` -- SR/PR/R tier design
- `.planning/research/CASE-CONSTRAINT-FORWARDING-DOWNSTREAM.md` -- validator integration design

### Secondary (HIGH confidence)
- `.claude/skills/case/step-discuss.md` -- current discussion flow with Step 2.5
- `.claude/skills/case/step-finalize.md` -- current output format
- `.claude/agents/case-briefer.md` -- current briefer agent
- Memory: `project_cross_phase_forwarding.md` -- original problem statement
- Memory: `project_unified_spec_idea.md` -- related living spec concept
- Memory: `feedback_case_constraint_forwarding.md` -- implemented constraint forwarding

---

*Research completed: 2026-03-26*
*Valid until: 2026-06-26 (methodology-focused, stable domain)*
