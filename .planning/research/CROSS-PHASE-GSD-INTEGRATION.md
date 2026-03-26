# Cross-Phase Forwarding Integration with GSD Workflow

**Researched:** 2026-03-26
**Domain:** GSD workflow extension, cross-phase concern forwarding, intra-milestone artifact flow
**Confidence:** HIGH (direct analysis of project artifacts + GSD documentation)

---

## Summary

Cross-phase concern forwarding addresses the gap where Phase A's `/case` discovers concerns relevant to Phase B, but no mechanism guarantees Phase B's agents see them. The concrete motivating example: 3A's RefreshToken R4 (JWT claims from session data -- freshness maintained by update operations) must be verified in 3B when user info modification is introduced. Currently, this concern exists only in 3A's CASE-SCRATCH.md and in a memory file -- neither is reliably consumed by 3B's pipeline stages.

This research analyzes WHERE in the GSD workflow forwarding should happen and HOW existing agents should consume forwarded concerns. The analysis evaluates five integration points: discuss-phase, case-briefer, step-init, plan-phase, and verify-work.

**Primary recommendation:** Implement forwarding at TWO points (Option C from Research Question 2):
1. **case-briefer** reads dependency phases' CASES.md and extracts open questions + deferred concerns into a new "Inherited Concerns" section of CASE-BRIEFING.md
2. **discuss-phase pre-flight** (via CLAUDE.md instruction) prompts the human to review dependency concerns before locking decisions

This dual approach handles both concern types: behavioral concerns that need /case discussion (briefer path) and design decisions that need discuss-phase resolution (discuss path). The forwarding mechanism is fully contained within existing project-local files -- no GSD source modification required.

---

## 1. GSD Workflow Anatomy

### Relevant Stages and Agent Capabilities

| Stage | Agent | Reads | Writes | Extension Mechanism |
|-------|-------|-------|--------|---------------------|
| discuss-phase | Main conversation (interactive) | PROJECT.md, REQUIREMENTS.md, ROADMAP.md, prior CONTEXT.md | CONTEXT.md | CLAUDE.md instructions |
| /case (briefer) | case-briefer (sonnet subagent) | CONTEXT.md, ROADMAP.md, REQUIREMENTS.md, PROJECT.md | CASE-BRIEFING.md | Agent definition in `.claude/agents/` |
| /case (discuss) | Main conversation (orchestrator) | CASE-BRIEFING.md, CONTEXT.md | CASE-SCRATCH.md | Skill definition in `.claude/skills/case/` |
| /case (validator) | case-validator (opus subagent) | CASE-SCRATCH.md, CASE-BRIEFING.md, CONTEXT.md, REQUIREMENTS.md | Inline findings | Agent definition in `.claude/agents/` |
| plan-phase (research) | gsd-phase-researcher (subagent) | PROJECT.md, CONTEXT.md, ROADMAP.md | RESEARCH.md | CLAUDE.md instructions |
| plan-phase (planner) | gsd-planner (subagent) | PROJECT.md, CONTEXT.md, CASES.md, RESEARCH.md, REQUIREMENTS.md | PLAN.md files | CLAUDE.md instructions |
| verify-work | Main conversation | CONTEXT.md, CASES.md, code | UAT results | CLAUDE.md instructions |

### Extension Points

GSD agents discover project context through two mechanisms:

1. **CLAUDE.md instructions** -- All GSD agents read CLAUDE.md. Project-specific instructions here are authoritative. This is already how CASES.md integration works (the "Extended Workflow" section in CLAUDE.md tells planner/executor to check for CASES.md).

2. **Project-local agent/skill files** -- The `/case` skill is entirely project-local (`.claude/skills/case/`, `.claude/agents/case-*.md`). Changes to these files are versioned with the project and take effect immediately.

GSD does NOT expose plugin-level hooks for injecting custom artifacts into its dispatch prompts. The `hooks` config key in config.json controls simple boolean flags (`context_warnings`, `research_questions`), not extensible artifact injection.

**Key insight:** All cross-phase forwarding MUST work through either CLAUDE.md instructions (for GSD agents) or project-local skill/agent modifications (for /case agents). There is no third mechanism.

---

## 2. ROADMAP Dependency Resolution

### Current State

ROADMAP.md has a `Depends on` field per phase:

```
Phase 3A: Depends on: Phase 1, Phase 2
Phase 3B: Depends on: Phase 3A
Phase 6:  Depends on: Phase 3, Phase 4
```

This field is currently human-readable only. No agent parses it programmatically.

### Parsing Approach

The `Depends on` field follows a consistent format: `Phase N` or `Phase NA, Phase NB`. Parsing is straightforward:

```
# Extract dependency phase numbers from ROADMAP.md
Pattern: "**Depends on**: Phase {N}[, Phase {M}]*"
```

The case-briefer already reads ROADMAP.md (it is in `<files_to_read>`). Adding dependency resolution requires:
1. Parse `Depends on` for the current phase
2. Resolve each dependency to a phase directory name
3. Check for `*-CASES.md` in each dependency phase directory
4. Read and extract relevant sections

### Direct vs Transitive Dependencies

**Recommendation: DIRECT dependencies only.**

| Approach | Pro | Con |
|----------|-----|-----|
| Direct only | Simple to implement; clear responsibility chain; each phase only forwards concerns from its immediate predecessors | Misses concerns from Phase 1 that transit through Phase 2 to Phase 3B |
| Transitive (all ancestors) | Complete coverage | Exponential scan as phases accumulate; most transitive concerns are either resolved by intermediate phases or irrelevant; creates noise |
| One-level transitive | Moderate coverage | Ambiguous cutoff rule |

**Rationale for direct-only:** If Phase 1 has a concern relevant to Phase 3B, it should have been addressed when Phase 2 or Phase 3A ran (since they depend on Phase 1). If it was NOT addressed, that is a gap in Phase 2/3A's handling -- not something Phase 3B should compensate for. The forwarding chain is: 1 -> 2 -> 3A -> 3B, with each link handling its immediate predecessor's concerns.

**Exception:** If ROADMAP.md lists explicit transitive dependencies (e.g., Phase 6 says `Depends on: Phase 3, Phase 4`), those are direct dependencies even if Phase 3 transitively depends on Phase 1. The briefer should read Phase 3 and Phase 4 CASES.md, not Phase 1.

### What to Extract from Dependency CASES.md

Not everything in a dependency's CASES.md is relevant. The briefer should extract:

| Section | Relevance | Action |
|---------|-----------|--------|
| Open Questions (Q1-QN) | HIGH -- explicitly unresolved | Forward as "Inherited Open Questions" |
| Operation Rules with "3B" or "{downstream_phase}" mentions | HIGH -- explicitly deferred | Forward as "Deferred Concerns" |
| Phase Rules (PR) | MEDIUM -- may establish conventions | Note if they create expectations for downstream |
| SR Candidates | MEDIUM -- may need promotion | Flag if relevant |
| Individual S/F/E cases | LOW -- already the dependency's concern | Do NOT forward |

The concrete 3A concern that motivated this research is RefreshToken R4: "JWT claims from session data -- freshness maintained by update operations, not RefreshToken." The term "update operations" refers to functionality that does not exist in 3A but WILL exist in 3B (user info modification). This is a textual signal: a rule that references future-phase functionality should be forwarded.

**Signal patterns for forwarding:**
- Explicit phase references: "3B", "Phase 4", "future phase", "later"
- Future-tense references: "when X is introduced", "once Y exists"
- Open Questions marked as "deferred to next phase"
- Rules with "not RefreshToken's responsibility" or similar delegation language

---

## 3. Integration Point Analysis

### Option A: discuss-phase surfaces concerns -> CONTEXT.md includes them

**How it would work:**
1. CLAUDE.md instructs discuss-phase to scan dependency phases' CASES.md
2. Discuss-phase presents inherited concerns as discussion topics
3. Resolved concerns become decisions in CONTEXT.md
4. /case reads CONTEXT.md naturally -- no change needed

**Pros:**
- Design-level concerns (like session staleness policy) belong in discuss-phase
- Concerns are resolved before /case starts, reducing /case scope
- CONTEXT.md is the canonical decision document; concerns resolved here are authoritative

**Cons:**
- discuss-phase is a GSD agent (main conversation with GSD prompting) -- custom file scanning must go through CLAUDE.md instructions, which are advisory, not mandatory
- Discuss-phase already has a lot to do; adding dependency scanning may cause it to skip or superficially handle inherited concerns
- The human may not understand the concern without the full CASES.md context

**Confidence: MEDIUM** -- the mechanism works (CLAUDE.md instructions are read), but reliability depends on the GSD discuss-phase agent faithfully executing the instruction.

### Option B: /case surfaces concerns directly via case-briefer

**How it would work:**
1. case-briefer reads dependency phases' CASES.md (added to `<files_to_read>`)
2. Briefer extracts open questions + deferred concerns into CASE-BRIEFING.md
3. step-init or step-discuss presents inherited concerns to the developer
4. Developer decides: discuss now, mark as open question, or dismiss

**Pros:**
- case-briefer is a project-local agent -- full control over its behavior
- Inherited concerns appear in the behavioral specification context where they are most actionable
- The developer sees both the concern and the operation it affects

**Cons:**
- Behavioral concerns are well-handled, but design decisions (architecture choices) may not fit naturally in /case's "WHAT, not HOW" scope
- case-briefer is a Sonnet model -- adding dependency scanning increases its cognitive load

**Confidence: HIGH** -- project-local agent, clear implementation path, natural fit for behavioral concerns.

### Option C: Both stages surface different concern types (RECOMMENDED)

**How it would work:**
1. **discuss-phase pre-flight** (CLAUDE.md instruction): Before asking new questions, scan dependency CASES.md open questions. Present design-level concerns (architecture, policy, configuration) for resolution. These become CONTEXT.md decisions.
2. **case-briefer enrichment** (agent definition change): Read dependency CASES.md. Extract behavioral concerns (operation rules referencing future phases, open questions about behavioral expectations) into an "Inherited Concerns" section of CASE-BRIEFING.md. The /case orchestrator presents these during discussion.

**Why both:**
- The 3A RefreshToken R4 concern has TWO aspects:
  - **Design decision:** "Should update operations proactively refresh session data?" -- this belongs in discuss-phase
  - **Behavioral case:** "When user info changes, does the next refreshed JWT reflect the change?" -- this belongs in /case
- Forwarding only at one stage misses one aspect

**Deduplication:** If discuss-phase resolves a concern and records it in CONTEXT.md, the case-briefer will see it as a locked decision (it already reads CONTEXT.md) and will NOT re-surface it as an inherited concern. Natural deduplication through existing artifact flow.

**Confidence: HIGH** -- leverages existing artifact dependencies, clear separation of concern types, natural deduplication.

### Why NOT step-init or plan-phase

**step-init** could scan dependency CASES.md, but step-init runs AFTER the briefer and is an orchestrator step (main conversation), not a subagent. Adding file scanning to the orchestrator bloats its responsibilities. Better to keep it in the briefer (dedicated subagent for extraction).

**plan-phase** is too late. By plan-phase, behavioral decisions should already be made. The planner creates tasks from decided behavior -- it should not be discovering new concerns. If a forwarded concern reaches plan-phase unresolved, it should appear as a flagged open question, not a new discovery.

---

## 4. Agent Consumption Patterns

### gsd:plan-phase (planner)

**Current:** Planner reads CASES.md and maps must-priority cases to tasks. Some cases may have open questions flagged.

**With forwarding:** Inherited concerns that were discussed and resolved appear as normal decisions in CONTEXT.md and normal cases in CASES.md. The planner does NOT need to know they were inherited -- they are just decisions and cases like any others.

**Unresolved inherited concerns:** If an inherited concern reaches CASES.md as an open question (Q), the planner handles it the same way it handles any open question: flag it, use default recommendation, or ask for resolution. No special treatment needed.

**Recommendation: No change to planner.** The forwarding mechanism ensures concerns are resolved BEFORE plan-phase. The planner sees only resolved artifacts.

### gsd:discuss-phase

**Current:** Reads PROJECT.md, REQUIREMENTS.md, ROADMAP.md, and prior CONTEXT.md.

**With forwarding:** CLAUDE.md instruction tells discuss-phase to ALSO check dependency phases' CASES.md for open questions and deferred concerns. This is an advisory scan, not a mandatory gate.

**Proposed CLAUDE.md addition:**

```markdown
### Cross-Phase Concern Forwarding

When running `/gsd:discuss-phase` for a phase with dependencies:
1. Check ROADMAP.md for the phase's `Depends on` field
2. For each direct dependency phase, check for `*-CASES.md` in that phase's directory
3. If found, scan for:
   - Open Questions (Q sections) that reference the current phase or future functionality
   - Rules that delegate responsibility to downstream phases
4. Present relevant concerns to the developer as "Inherited from Phase X" discussion topics
5. Resolve them as CONTEXT.md decisions, defer them explicitly, or dismiss with justification
```

**Confidence: MEDIUM** -- the instruction is clear but GSD discuss-phase compliance depends on the agent reading and faithfully executing CLAUDE.md instructions. This is the same mechanism used for CASES.md integration with plan-phase, which has worked.

### gsd:verify-work

**Current:** Uses must-priority S/F cases from CASES.md as UAT scenarios.

**With forwarding:** If an inherited concern was converted to a case during /case, it appears in CASES.md like any other case and is verified normally. If it was resolved as a CONTEXT.md decision without a corresponding case (e.g., "session data is refreshed proactively"), the verifier checks CONTEXT.md decisions against implementation.

**Recommendation: No change to verifier.** Forwarded concerns that produce cases are verified through existing CASES.md verification. Concerns that produce design decisions are verified through existing CONTEXT.md verification.

### case-validator

**Current:** Cross-checks CASE-SCRATCH.md against CONTEXT.md, REQUIREMENTS.md, and CASE-BRIEFING.md. Five checks (A through E).

**With forwarding:** The validator could gain a **Check F: Inherited Concern Coverage** that verifies all inherited concerns from the briefing are addressed in CASE-SCRATCH.md (as cases, rules, or explicit dismissals).

**Recommendation: Defer.** Adding a sixth check adds complexity. The case-briefer's inherited concerns section will be visible to the orchestrator, who presents them to the developer. If the developer dismisses a concern, that is recorded. The validator's existing Check E (Briefing Gaps -- briefed operations not discussed) partially covers this by checking if briefer-identified items are addressed. For now, rely on the orchestrator presenting inherited concerns during discussion.

---

## 5. Seeds vs Intra-Milestone Forwarding

### GSD Seed Mechanism

Seeds (`gsd:plant-seed`) are forward-looking ideas with trigger conditions stored in `.planning/seeds/SEED-NNN-slug.md`. They surface automatically at milestone boundaries when `gsd:new-milestone` scans all seeds and presents matches. Seeds are designed for cross-MILESTONE idea forwarding -- "when the next milestone starts, consider X."

### Intra-Milestone Forwarding

Cross-phase concern forwarding is for concerns WITHIN the same milestone. The concern from 3A RefreshToken R4 must surface in 3B, which is in the same milestone. Seeds would not help because:

1. Seeds surface at milestone START, not between phases
2. The trigger condition mechanism matches milestone descriptions, not phase-level concerns
3. Seeds are ideas/suggestions; forwarded concerns are mandatory review items

### Relationship

| Dimension | Seeds | Intra-Milestone Forwarding |
|-----------|-------|---------------------------|
| Scope | Cross-milestone | Within same milestone |
| Trigger | Milestone start | Phase start (discuss or /case) |
| Content | Ideas, suggestions, future considerations | Behavioral concerns, deferred decisions, open questions |
| Obligation | Advisory (developer may dismiss) | Review mandatory (developer may resolve, defer, or dismiss with justification) |
| Storage | `.planning/seeds/SEED-NNN-slug.md` | Inline in dependency CASES.md (open questions, rules with future references) |
| Surfacing | `gsd:new-milestone` scans seeds | case-briefer scans dependency CASES.md |

**They are complementary, not overlapping.** Seeds handle cross-milestone forwarding. Intra-milestone forwarding handles within-milestone concerns. There is no deconfliction needed -- they operate at different timescales and granularities.

**Could seeds be repurposed for intra-milestone forwarding?** No. Seeds are file-based (`SEED-NNN-slug.md`) and surface at milestone boundaries. Using them for intra-phase concerns would require creating a seed file per concern, with trigger conditions matching phase numbers rather than milestone descriptions. This misuses the abstraction and creates file proliferation for concerns that already have a natural home (CASES.md open questions).

---

## 6. GSD Coupling Analysis

### Where Changes Live

| Change | Location | GSD Coupling |
|--------|----------|-------------|
| case-briefer reads dependency CASES.md | `.claude/agents/case-briefer.md` | NONE -- project-local agent |
| case-briefer output adds "Inherited Concerns" section | `.claude/agents/case-briefer.md` | NONE -- project-local agent |
| step-discuss presents inherited concerns | `.claude/skills/case/step-discuss.md` | NONE -- project-local skill |
| discuss-phase scans dependency CASES.md | `CLAUDE.md` instruction | INDIRECT -- relies on GSD agent reading CLAUDE.md |
| plan-phase handling | No change | NONE |
| verify-work handling | No change | NONE |

### Risk Assessment

**CLAUDE.md instruction compliance (MEDIUM risk):** GSD agents read CLAUDE.md but treat it as project guidance, not hard constraints. The discuss-phase agent may skip the dependency scanning instruction if its prompt processing prioritizes other tasks. Mitigation: keep the instruction concise and high-priority (place it early in the Extended Workflow section).

**case-briefer cognitive load (LOW risk):** Adding dependency CASES.md scanning increases the briefer's workload. Mitigation: the briefer already reads 4 files (ROADMAP, CONTEXT, REQUIREMENTS, PROJECT). Adding 1-2 dependency CASES.md files is incremental. The extraction targets (open questions, rules with future references) are specific and bounded.

**GSD update compatibility (NO risk):** All changes are in project-local files. GSD updates do not touch `.claude/agents/`, `.claude/skills/`, or project-level `CLAUDE.md`. The forwarding mechanism is entirely decoupled from GSD internals.

### Should Forwarding Be Fully Contained in /case?

**Mostly yes, with one CLAUDE.md instruction for discuss-phase.**

The case-briefer handles the bulk of forwarding (scanning dependency CASES.md, extracting concerns, presenting in briefing). This is fully project-local. The one GSD-touching piece is the discuss-phase instruction in CLAUDE.md, which enables design-level concerns to be resolved before /case runs. This instruction is low-coupling (advisory, not structural) and follows the same pattern already used for CASES.md integration with plan-phase.

---

## 7. Implementation Design

### 7.1 case-briefer Changes

**New Step 1.5: Scan Dependency CASES.md**

After understanding phase scope (Step 1) and before discovering operations (Step 2):

1. Read ROADMAP.md `Depends on` field for the current phase
2. Resolve each dependency to its phase directory
3. For each dependency, check for `*-CASES.md`
4. If found, extract:
   - Open Questions where Impact or Recommendation references the current phase, future functionality, or downstream phases
   - Operation Rules that delegate responsibility (keywords: "3B", future-phase name, "not X's responsibility", "when Y is introduced")
   - SR Candidates not yet promoted
5. Write extracted items to CASE-BRIEFING.md as "Inherited Concerns" section

**CASE-BRIEFING.md addition:**

```markdown
## Inherited Concerns (from dependency phases)

### From Phase 3A: Authentication Core

| Source | Type | Concern | Relevance to This Phase |
|--------|------|---------|------------------------|
| RefreshToken R4 | Deferred Rule | JWT claims from session data -- freshness maintained by update operations | 3B introduces user info modification; refreshed JWT must reflect changes |
| RegisterBegin Q2 | Open Question | Design ceremony logic to be reusable for 3B passkey addition | 3B adds passkey addition flow |

### From Phase 2: User Profile

(None found)
```

**Dispatch prompt change:** Add dependency CASES.md paths to `<files_to_read>`:

```xml
<files_to_read>
- .planning/ROADMAP.md -- phase description, success criteria, requirement IDs, dependency resolution
- {phase_dir}/*-CONTEXT.md -- locked decisions, constraints
- .planning/REQUIREMENTS.md -- requirement ID descriptions
- .planning/PROJECT.md -- architecture reference
- {dep_phase_dir}/*-CASES.md -- dependency phase behavioral cases (for inherited concerns)
</files_to_read>
```

### 7.2 step-discuss Changes

**Step 2.5 extension: Present Inherited Concerns**

After Phase Rules confirmation and before first operation discussion, present inherited concerns from the briefing:

```
Inherited concerns from dependency phases:

[1] From 3A RefreshToken R4: JWT claims come from session data.
    When this phase introduces user info modification, verify that
    refreshed JWTs reflect the updated values.
    -> Should we add this as a case to a relevant operation?

[2] From 3A RegisterBegin Q2: Ceremony logic should be reusable
    for passkey addition.
    -> Already covered by 3B passkey addition operations? Dismiss?
```

Developer actions per concern:
- **Add as case** to a specific operation -> becomes part of that operation's discussion
- **Add as Phase Rule** -> goes into Phase Rules section
- **Defer** -> stays as open question in CASES.md
- **Dismiss** -> recorded with justification (e.g., "already handled by existing operation X")

### 7.3 CLAUDE.md Addition

Add to the "Extended Workflow" section:

```markdown
### Cross-Phase Concern Forwarding

When running `/gsd:discuss-phase` for a phase with ROADMAP.md dependencies:
1. Check each direct dependency phase's directory for `*-CASES.md`
2. If found, scan Open Questions and Rules for items referencing the current
   phase or future functionality
3. Present relevant items as "Inherited from Phase X" discussion topics
4. Resolve as CONTEXT.md decisions, explicitly defer, or dismiss with justification

The `/case` skill handles this automatically via case-briefer dependency scanning.
```

### 7.4 Updated Pipeline Diagram

The pipeline itself does not change -- no new stage or arrow is added. The change is within existing stages:

```
                     CROSS-PHASE FORWARDING (within existing stages)
                     ================================================

         ┌─────────────────────────────────────┐
         v                                     |
    gsd:discuss <──────────────┐───────────────┤
         |                     |               |
         | [reads dep CASES.md |               |
         |  open questions     |               |
         |  for design-level   |               |
         |  concerns]          |               |
         |                     |               |
       /case <─────────────┐───┘               |
         |                 |                   |
         | [briefer reads  |                   |
         |  dep CASES.md   |                   |
         |  for behavioral |                   |
         |  concerns]      |                   |
         |                 |                   |
   (gsd:research) ─────────┘───────────────────┘
         |
    gsd:plan
         |
    (no change -- resolved concerns appear as normal decisions/cases)
```

The forwarding is invisible to downstream stages. By the time plan-phase runs, inherited concerns have been resolved into CONTEXT.md decisions or CASES.md cases/rules/questions. The planner sees only the resolved state.

---

## 8. Concrete Example: 3A -> 3B Forwarding

Walking through the motivating example end-to-end:

### Source (3A CASE-SCRATCH.md, RefreshToken)

```
R4: JWT claims from session data -- freshness maintained by
update operations, not RefreshToken. Design intent: when user
info changes, the update operation writes to session data in
Redis; RefreshToken trusts session data as-is.
```

### Step 1: case-briefer scans 3A CASES.md for 3B

Briefer reads ROADMAP.md: "Phase 3B depends on Phase 3A."
Briefer reads 3A CASES.md. Finds RefreshToken R4 containing "update operations" -- this references functionality that 3B introduces (user role change, user deactivation).

Outputs to CASE-BRIEFING.md:

```markdown
## Inherited Concerns (from dependency phases)

### From Phase 3A: Authentication Core

| Source | Type | Concern | Relevance to This Phase |
|--------|------|---------|------------------------|
| RefreshToken R4 | Deferred Rule | JWT claims from session data; freshness maintained by update operations | 3B introduces role change and deactivation; must verify session data is updated and refreshed JWT reflects changes |
```

### Step 2: step-discuss presents to developer

```
Inherited concern from 3A:
  RefreshToken R4 says JWT claims come from session data,
  and update operations maintain freshness.
  Phase 3B introduces role change -- should RoleChange operation
  update session data in Redis so next JWT refresh reflects new role?
```

Developer confirms: "Yes, role change must update session data."

This becomes a case in the RoleChange operation:
```
E1: Role change + subsequent JWT refresh | User role changed from user to admin |
    RoleChange, then JWT expires and refreshes | Refreshed JWT has role=admin |
    (must)
```

### Step 3: Downstream stages see resolved state

- CONTEXT.md gains a decision: "Role change updates session data in Redis"
- CASES.md gains an edge case for RoleChange
- Planner sees these as normal inputs and creates tasks accordingly
- Verifier tests the edge case as part of UAT

---

## 9. Scope and Limitations

### What This Mechanism Does NOT Handle

1. **Cross-milestone forwarding** -- Use seeds (`gsd:plant-seed`) for that
2. **Concerns with no textual signal** -- If 3A's CASES.md does not mention "3B" or "future" or "update operations," the briefer cannot detect the concern. The developer's memory is the fallback.
3. **Transitive dependency concerns** -- Only direct dependencies are scanned. If Phase 1 has a concern relevant to Phase 3B but not to Phase 2 or 3A, it will be missed. This is acceptable because the concern should have been addressed by the intermediate phases.
4. **Automatic resolution** -- Forwarding surfaces concerns; it does not resolve them. The developer must still make decisions.

### What About Memory Files?

The `project_cross_phase_forwarding.md` memory file captures the problem statement and candidate solutions. Memory files are NOT a reliable forwarding mechanism because:
- They are session-scoped (loaded into context at session start)
- They may be pruned or summarized over time
- They are not consumed by subagents (case-briefer, case-validator)
- They have no structured extraction mechanism

Memory files serve as reminders to the HUMAN and the main conversation, not as a forwarding pipeline. The case-briefer dependency scanning replaces the need for memory-based forwarding.

---

## Confidence Assessment

| Area | Level | Reason |
|------|-------|--------|
| Integration point selection (Option C) | HIGH | Evidence-based analysis of 5 candidates with clear pro/con |
| case-briefer changes | HIGH | Project-local agent, clear implementation path, bounded scope |
| ROADMAP dependency parsing | HIGH | Consistent format, simple extraction |
| Direct-only dependencies | HIGH | Clear rationale, exception handling for explicit transitive deps |
| discuss-phase CLAUDE.md instruction | MEDIUM | Depends on GSD agent compliance, same mechanism as existing CASES.md integration |
| Seeds vs forwarding deconfliction | HIGH | Different timescales, no overlap |
| GSD coupling risk | HIGH (low risk) | All changes project-local except one advisory instruction |
| Inherited concern extraction signals | MEDIUM | Textual signals may miss implicit concerns; developer memory is fallback |

---

## Implementation Order

1. **case-briefer.md** -- Add Step 1.5 (dependency CASES.md scanning) and "Inherited Concerns" output section. This is the core change.
2. **step-discuss.md** -- Add inherited concern presentation to Step 2.5 (Phase Rules confirmation). Natural insertion point.
3. **CLAUDE.md** -- Add cross-phase concern forwarding instruction to Extended Workflow section.
4. **step-init.md** -- Update case-briefer dispatch prompt to include dependency CASES.md paths in `<files_to_read>`.

Steps 1-2 are the essential changes. Steps 3-4 are refinements.

### Files to Modify

| File | Change | Impact |
|------|--------|--------|
| `.claude/agents/case-briefer.md` | Add Step 1.5 (dependency scanning) + Inherited Concerns output section | Core mechanism |
| `.claude/skills/case/step-discuss.md` | Add inherited concern presentation to Step 2.5 | Discussion flow |
| `.claude/skills/case/step-init.md` | Update briefer dispatch to include dep CASES.md paths | Dispatch contract |
| `CLAUDE.md` | Add cross-phase forwarding instruction to Extended Workflow | GSD agent awareness |

### What Does NOT Change

- case-validator (no new check -- defer to future)
- step-finalize.md (inherited concerns that become cases/rules appear in normal output)
- plan-phase (sees resolved artifacts)
- verify-work (sees resolved artifacts)
- GSD source code (no plugin modification)

---

## Open Questions

### OQ-1: Briefer Model Capacity for Dependency Scanning

The case-briefer runs on Sonnet. Adding dependency CASES.md scanning increases its input. For large dependency CASES.md files (3A has ~350 lines), this is manageable. For phases with many dependencies, total input could grow. Monitor briefer quality on first use.

**Recommendation:** Proceed. Sonnet handles multi-file analysis well. The extraction task is specific (open questions, rules with future references) -- not open-ended analysis.

### OQ-2: Signal Detection Accuracy

The briefer must detect which rules/questions in dependency CASES.md are relevant to the current phase. The textual signals ("3B", "future", "update operations") work for explicit references but may miss implicit ones.

**Recommendation:** Accept the limitation. Explicit signals catch the most critical cases (like 3A RefreshToken R4). Implicit concerns that lack textual signals are, by definition, not well-articulated -- the developer's memory is the appropriate fallback for those.

### OQ-3: Retrospective Application to 3B

3B has not yet run /case. When it does, the case-briefer will scan 3A's CASE-SCRATCH.md (or CASES.md once finalized). This is the natural first test of the forwarding mechanism.

**Recommendation:** Implement the changes before 3B's /case session. The 3A -> 3B forwarding is the motivating example and the ideal first validation.

---

## Sources

### Primary (HIGH confidence)
- `.claude/agents/case-briefer.md` -- current briefer agent definition
- `.claude/agents/case-validator.md` -- current validator agent definition
- `.claude/skills/case/SKILL.md` -- main skill definition
- `.claude/skills/case/step-init.md` -- init step with briefer dispatch
- `.claude/skills/case/step-discuss.md` -- discussion step with Step 2.5
- `.claude/skills/case/step-finalize.md` -- finalize step with output format
- `.planning/WORKFLOW.md` -- complete GSD workflow with stages and artifacts
- `.planning/ROADMAP.md` -- phase dependency structure
- `.planning/PROJECT.md` -- system-wide rules, architecture
- `.planning/phases/03a-authentication-core/CASE-SCRATCH.md` -- 3A cases with RefreshToken R4
- `.planning/phases/03b-authentication-operations/03b-CONTEXT.md` -- 3B scope
- `.planning/research/CASE-CONSTRAINT-FORWARDING-SUMMARY.md` -- prior constraint forwarding research
- `.planning/research/GSD-AGENT-PATTERNS.md` -- GSD agent design patterns
- `.planning/research/WORKFLOW-RESEARCH.md` -- extended workflow research

### Secondary (MEDIUM confidence)
- [GSD GitHub Repository](https://github.com/gsd-build/get-shit-done) -- GSD architecture, seed mechanism
- [GSD USER-GUIDE.md](https://github.com/gsd-build/get-shit-done/blob/main/docs/USER-GUIDE.md) -- discuss-phase workflow, seed details
- Memory: `project_cross_phase_forwarding.md` -- original problem statement
- Memory: `feedback_case_constraint_forwarding.md` -- constraint forwarding implementation

---

*Research completed: 2026-03-26*
*Valid until: 2026-06-26 (stable domain -- workflow architecture, not fast-moving libraries)*
