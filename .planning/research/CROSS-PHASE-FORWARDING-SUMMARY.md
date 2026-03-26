# Cross-Phase Concern Forwarding: Research Summary

**Project:** madome
**Domain:** GSD workflow extension -- intra-milestone cross-phase concern propagation
**Researched:** 2026-03-26
**Confidence:** HIGH

## Decision Summary

| Question | Answer | Source |
|----------|--------|--------|
| What mechanism? | Hybrid F: automatic briefer scan + explicit Forward Concerns section in CASES.md | R1 Section 6 |
| What gets forwarded? | 5 concern categories (explicit deferred, targeted OQ, phase rules, implicit behavioral, deferred-by-scope) | R2 Section 1 |
| Where in the pipeline? | Two points: case-briefer (behavioral) + discuss-phase pre-flight via CLAUDE.md (design-level) | R3 Section 3 Option C |
| Which files change? | 4 project-local files (~80 lines total) | R1 + R3 merged |
| GSD coupling risk? | None. All changes are project-local. No GSD-owned files modified. | R4 Section 4 |
| Relationship to living spec? | Complementary. Forwarding solves immediate agent blindness; living spec solves long-term staleness. Implement forwarding now; defer living spec. | R1 Section 5 |

## Executive Summary

The GSD pipeline has a gap: when Phase A's `/case` session discovers concerns relevant to Phase B, no mechanism guarantees Phase B's agents see them. The case-briefer reads only the current phase's CONTEXT.md, ROADMAP.md, REQUIREMENTS.md, and PROJECT.md. It does not read dependency phases' CASES.md, CASE-SCRATCH.md, or CONTEXT.md deferred items. This was concretely demonstrated by the 3A->3B scenario where RefreshToken R4 (JWT claims from session data -- freshness maintained by update operations) must surface in 3B's case discussion when user info modification is introduced, but currently has no forwarding path.

Four parallel research tracks converge on a single coherent design: **Hybrid F** (R1) combines automatic briefer scanning of dependency CASES.md with an explicit Forward Concerns section, consuming a **5-category concern taxonomy** (R2) that classifies concerns by source and timing, integrated at **two pipeline points** (R3) -- the case-briefer for behavioral concerns and discuss-phase for design-level concerns -- all implemented through **project-local files only** (R4), requiring zero modification to GSD-owned code. The 3A->3B walkthrough (R1 Section 4) scored Hybrid F at 2-3 automatic catches + 0-2 explicit catches + 2 already-covered items, the strongest coverage of all 6 candidates evaluated while maintaining minimal developer burden.

The key risk is Category 4 implicit concerns -- behavioral implications not explicitly tagged for forwarding. No automated system fully solves this; the developer remains the ultimate safety net. The mitigation is three-layered: the Protester asks about cross-phase assumptions during each operation discussion, cross-operation analysis scans for implications, and the explicit Forward Concerns section lets developers manually register concerns they recognize.

## Key Findings

### Mechanism Design (R1: CROSS-PHASE-FORWARDING-MECHANISM.md)

Six candidates were evaluated against 5 concrete 3A->3B concerns. Hybrid F scored highest by combining automatic extraction (catches structurally-marked items like Open Questions and rules with phase references) with explicit forwarding (catches developer-recognized items the automation would miss). The key differentiators vs alternatives:

- **vs Briefer-only (A):** F adds the explicit channel for non-structural concerns
- **vs Pre-flight (B):** F avoids the developer round-trip friction before every /case session
- **vs FORWARD.md (C) / Registry (E):** F has an automatic component; C and E depend entirely on developer discipline
- **vs Deferred parsing (D):** D scored weakest (0 direct catches) because the most valuable concerns are discovered DURING case analysis, not deferred BEFORE it

Living spec relationship: forwarding and living spec are complementary (R1 Section 5). Forwarding ensures the right concerns surface at the right time. Living spec ensures all behavioral decisions live in one place. The extraction logic built for forwarding is reusable when/if living spec is implemented later.

### Concern Classification (R2: CROSS-PHASE-CONCERN-CLASSIFICATION.md)

Five concern categories derived from real 3A/3B artifacts:

| Category | Source | Timing | Forwarding Path |
|----------|--------|--------|-----------------|
| 1. Explicit Deferred | CONTEXT.md `<deferred>` | Before /case | Already tagged; briefer reads dep CONTEXT.md |
| 2. Targeted Open Questions | CASES.md Open Questions | During /case | Forward column in OQ table (e.g., `->3B:AddPasskey`) |
| 3. Phase Rules (PR) | CASES.md Phase Rules | During /case | Inherited when downstream phase shares domain |
| 4. Implicit Behavioral | Case Rules/Edge Cases | During /case | Cross-Phase Implications section (`X` items) |
| 5. Deferred-by-Scope | Discussion discovery | During /case | Overlaps with Cat 1/2; captured at discovery moment |

CONTEXT.md deferred items and CASES.md Open Questions are complementary, not redundant (R2 Section 3). Deferred items are design decisions deferred BEFORE case discovery; OQs emerge DURING case discovery at a different abstraction level.

Priority does NOT transfer across phases (R2 Section 5). The receiving phase re-evaluates each concern in its own context. Security-elevated concerns require explicit acknowledgment (cannot be silently dismissed).

### GSD Integration (R3: CROSS-PHASE-GSD-INTEGRATION.md)

Five integration points evaluated. Two selected (Option C):

1. **case-briefer** (primary): Reads dependency CASES.md via ROADMAP `Depends on` resolution. Extracts OQs with forward tags, rules with phase references, Cross-Phase Implications. Outputs `## Inherited Concerns` section in CASE-BRIEFING.md. Project-local agent -- full control.

2. **discuss-phase pre-flight** (secondary): CLAUDE.md instruction tells discuss-phase to scan dependency CASES.md for design-level concerns. Resolved concerns become CONTEXT.md decisions before /case runs. Natural deduplication: briefer reads CONTEXT.md, so concerns resolved in discuss don't re-surface.

**Why not other points:** step-init is the wrong abstraction (orchestrator, not extractor). plan-phase is too late (should not discover new concerns). case-validator deferred (would add noise for marginal benefit).

Direct-only dependency scanning recommended (R3 Section 2). Transitive concerns should propagate through the forwarding chain: 1->2->3A->3B. ROADMAP.md explicit dependencies (even if logically transitive) are treated as direct.

### GSD Safety (R4: GSD-PATCH-SYSTEM.md)

All proposed changes live in project-local files that GSD never touches:

| File | Location | GSD-Safe? |
|------|----------|-----------|
| `.claude/agents/case-briefer.md` | Project-local | YES |
| `.claude/skills/case/step-discuss.md` | Project-local | YES |
| `.claude/skills/case/step-finalize.md` | Project-local | YES |
| `CLAUDE.md` | Project-local | YES |

No GSD-owned files need modification. No patch system usage required. The CLAUDE.md instruction for discuss-phase is the only GSD-adjacent change, and it follows the same advisory mechanism already used for CASES.md integration with plan-phase.

Known limitation: Claude Code bug #10061 (sub-agent skill resolution from global instead of project-local) does not affect this design because the case-briefer is invoked directly, not through a GSD agent spawning a sub-agent.

## Implementation Scope

### Files to Modify (4 files, ~80 lines total)

| # | File | Change | Lines | Depends On |
|---|------|--------|-------|------------|
| 1 | `.claude/agents/case-briefer.md` | Add Step 1.5 (dependency CASES.md scan) + `## Inherited Concerns` output section + update `<files_to_read>` guidance | ~40 | None |
| 2 | `.claude/skills/case/step-finalize.md` | Add `Forward` column to OQ table + `## Forward Concerns` section (or `## Cross-Phase Implications` with `X` items) to output format + finalize prompt for developer review | ~25 | None |
| 3 | `.claude/skills/case/step-discuss.md` | Add inherited concern presentation in Step 2.5 + implicit concern check prompt in Step 3e | ~15 | #1 (consumes briefer output) |
| 4 | `CLAUDE.md` | Add cross-phase forwarding instruction to Extended Workflow section | ~8 | None |

Changes 1 and 2 are independent (parallelizable). Change 3 depends on 1. Change 4 is independent.

### What Does NOT Change

- step-init.md (briefer handles scanning, not orchestrator)
- case-validator.md (no new validation check -- deferred)
- SKILL.md (no formatting changes)
- WORKFLOW.md (pipeline order unchanged)
- No new artifacts, no new agent types
- No GSD-owned files
- Existing CASES.md format gains optional additive sections

## Decisions Requiring Developer Input

### D1: Forward Concerns section placement in CASES.md

R1 proposes `## Forward Concerns` as a simple table (target phase, concern, source). R2 proposes `## Cross-Phase Implications` with richer structure (`X` items with ID, concern, source rule, affected phase, trigger). These are complementary but could be merged into one section or kept separate.

**R1's Forward Concerns:** Developer-authored during finalize. Simple. Explicit.
**R2's Cross-Phase Implications:** AI-authored during cross-operation analysis. Structured. Catches implicit concerns.

**Recommendation:** Merge into one section called `## Forward Concerns` with two subsections -- explicit (developer) and inferred (AI from cross-operation analysis). But this is a formatting decision the developer should confirm.

### D2: Open Questions `Forward` column vs separate section

R2 proposes adding a `Forward` column to the existing OQ table. This is a minor format change to an existing structure. The alternative (from R1) is to keep OQs as-is and rely on the briefer's heuristic scan to detect forward-relevant OQs by their text content.

**Recommendation:** Add the `Forward` column. It is explicit, low-cost, and the briefer can use structured tags (`->3B:AddPasskey`) rather than heuristic text scanning. The heuristic scan serves as a fallback for legacy CASES.md files that predate the column.

### D3: Discuss-phase pre-flight scope

R3 recommends CLAUDE.md instruction for discuss-phase to scan dependency CASES.md. This is the weakest link (MEDIUM confidence on GSD agent compliance). The developer should decide whether this is worth adding vs relying solely on the case-briefer path.

**Recommendation:** Add it. The instruction is 8 lines, follows an established pattern, and catches design-level concerns that /case's behavioral focus might not probe. If discuss-phase ignores it, the case-briefer path still catches most items.

### D4: Bootstrapping (3A artifacts lack forward tags)

3A was written before this system was designed. Its CASES.md has no `Forward` column and no `## Cross-Phase Implications`. The briefer must fall back to heuristic scanning (R2 OQ-3, R3 OQ-3).

**Recommendation:** Accept the bootstrapping gap. The briefer's heuristic scan (phase references in text, "Design intent:" notes, explicit "3B" mentions) catches the most critical items. Do not retroactively modify 3A's completed artifacts.

## Risk Assessment

| Risk | Severity | Likelihood | Mitigation |
|------|----------|------------|------------|
| Briefer (Sonnet) misses implicit concerns during scan | Medium | Medium | Forward Concerns section provides explicit fallback; developer can add concerns manually |
| Discuss-phase agent ignores CLAUDE.md instruction | Low | Medium | case-briefer path catches behavioral concerns independently |
| Inherited Concerns section adds noise to briefing | Low | Low | Section is empty/skippable when no concerns exist; typically 2-5 items |
| Over-forwarding drowns receiving phase in concerns | Low | Low | Dual criteria: must affect downstream operations AND not inferable from CONTEXT.md alone |
| Long dependency chains accumulate many concerns | Low | Low | Direct-only scanning; concerns spanning 3+ phases should be promoted to PROJECT.md SRs |
| Forward Concerns section forgotten during finalize | Low | Medium | Automatic scan catches structural items; explicit section is additive safety |
| Category 4 implicit concern goes undetected | Medium | Medium | Three-layer defense: Protester prompt, cross-op analysis, explicit developer channel |

**Fallback:** If the entire forwarding system fails to surface a concern, the developer's memory and existing CONTEXT.md overlap provide a partial safety net. The system improves reliability, not creates a hard dependency.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Mechanism (R1) | HIGH | 6 candidates scored against 5 concrete concerns with clear walkthrough |
| Classification (R2) | HIGH | 5-category taxonomy with real 3A/3B evidence; classification decision tree tested |
| GSD Integration (R3) | HIGH | Direct analysis of existing agents/skills; clear extension points identified |
| GSD Safety (R4) | HIGH | Project-local strategy confirmed safe; patch system understood but not needed |
| Implicit concern detection (Cat 4) | LOW | Fundamentally hard problem; no automated system fully solves it |
| Discuss-phase compliance | MEDIUM | CLAUDE.md instruction is advisory; same mechanism used for CASES.md integration |

**Overall: HIGH** for the design. The mechanism, classification, integration points, and safety are well-grounded in real artifacts and concrete evidence. The only LOW area (implicit concern detection) is acknowledged as inherently unsolvable by automation -- the design provides the best available mitigation.

### Gaps to Address

- **First real test:** The 3A->3B forwarding is untested. Implement before 3B's /case session and validate.
- **Sonnet extraction quality:** Unknown whether Sonnet reliably extracts implicit concerns from dependency CASES.md. Monitor on first use; consider targeted Opus scan for Category 4 if Sonnet's precision is too low.
- **Format convergence (D1):** R1 and R2 propose slightly different output section structures. Needs developer decision before implementation.

## Sources

### Primary (HIGH confidence)
- `.planning/phases/03a-authentication-core/CASE-SCRATCH.md` -- real 3A case data
- `.planning/phases/03a-authentication-core/03a-CONTEXT.md` -- real 3A deferred items
- `.planning/phases/03b-authentication-operations/03b-CONTEXT.md` -- real 3B scope
- `.claude/agents/case-briefer.md` -- current briefer agent definition
- `.claude/skills/case/step-init.md` -- current init step
- `.claude/skills/case/step-discuss.md` -- current discuss step
- `.claude/skills/case/step-finalize.md` -- current finalize step
- `.planning/ROADMAP.md` -- phase dependency structure
- `.planning/WORKFLOW.md` -- artifact availability and pipeline flow
- `.planning/research/CASE-CONSTRAINT-FORWARDING-SUMMARY.md` -- within-phase constraint forwarding

### Secondary (HIGH confidence)
- Memory: `project_cross_phase_forwarding.md` -- original problem statement
- Memory: `project_unified_spec_idea.md` -- living spec relationship
- Claude Code sub-agents and skills documentation -- precedence rules

### Tertiary (MEDIUM confidence)
- GSD GitHub repository -- patch system mechanics, update flow
- Community sources -- GSD update behavior, local modification handling

---

### Research Files

| File | Researcher | Focus |
|------|-----------|-------|
| `CROSS-PHASE-FORWARDING-MECHANISM.md` | R1 | 6 mechanism candidates, Hybrid F recommendation, living spec analysis |
| `CROSS-PHASE-CONCERN-CLASSIFICATION.md` | R2 | 5-category taxonomy, classification decision tree, lifecycle model, tagging format |
| `CROSS-PHASE-GSD-INTEGRATION.md` | R3 | Pipeline integration points, agent consumption patterns, dependency resolution |
| `GSD-PATCH-SYSTEM.md` | R4 | GSD update mechanics, file ownership map, project-local safety confirmation |

---
*Research synthesized: 2026-03-26*
*Ready for implementation: yes (pending D1-D4 decisions)*
