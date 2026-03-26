# Spec-Driven Workflow Research Summary

**Project:** madome
**Domain:** Workflow infrastructure -- spec lifecycle management, cross-phase discovery, briefer intelligence
**Researched:** 2026-03-26
**Confidence:** MEDIUM-HIGH

## Executive Summary

The spec-driven workflow idea emerged from a real incident: Phase 3A's /case session produced an Open Question about handle length (4-15 chars) that was already decided and implemented in Phase 2. The case-briefer could not surface this because it does not cross-reference operation names from CONTEXT.md against existing phase specs. The broader question was whether madome needs a full spec-driven workflow (living specs, service-organized spec folders, spec-first discipline) or just a smarter briefer.

After synthesizing four research tracks -- methodology patterns, tool landscape, folder structure design, and briefer intelligence -- the answer is clear: **the briefer is the bottleneck, not the spec format or organization**. The current CASES.md format is already richer than what any commercial SDD tool produces. The GSD pipeline already follows spec-first ordering for new phases. The missing piece is that completed phase specs become cold storage instead of being discoverable by later phases. This is primarily a briefer discovery problem, not a spec lifecycle problem.

The full spec-driven workflow (living specs with spec-first discipline for ALL changes, service-organized spec folders, drift detection) is well-designed in the research but **disproportionate to the actual problem**. The project has 7 phases, 6 services, one developer + AI agents. The Q1 handle-length incident was one case across 3 completed phases. Implementing the full workflow adds ongoing maintenance burden (spec extraction scripts, folder sync, drift monitoring) for a problem that occurs infrequently. The recommendation is: **improve the briefer now, defer everything else until the problem recurs or the project grows**.

## Cross-Cutting Analysis

### Where All 4 Research Tracks Agree

1. **No external tool worth adopting.** 01-METHODOLOGY confirms CASES.md is already superior to Gherkin/BDD formats. 02-TOOLS confirms no SDD tool (Spec Kit, Kiro, Tessl, OpenSpec) solves our exact problem. The project's existing workflow is more sophisticated than all surveyed alternatives.

2. **Spec rot is the #1 risk.** All four files flag this independently. 01-METHODOLOGY cites BDD abandonment patterns. 02-TOOLS identifies it as anti-pattern #1. 03-FOLDER-STRUCTURE warns about dual source-of-truth risk. 04-BRIEFER notes that adding more artifacts to maintain increases staleness risk. This convergence is significant -- any solution that adds maintenance burden must justify itself against this risk.

3. **Briefer improvements are independent of everything else.** 04-BRIEFER explicitly states this: "The briefer improvements are independent of the spec folder and should be implemented first. They will work equally well with or without a spec folder." This means the highest-value work has zero dependency on the larger spec-driven workflow decision.

4. **Incremental approach over big-bang.** 01-METHODOLOGY recommends "start with process discipline + enhanced gsd:validate." 02-TOOLS recommends "spec density grows naturally per phase." 03-FOLDER-STRUCTURE proposes migration phases. 04-BRIEFER proposes minimal viable improvement first.

### Where Research Tracks Disagree or Create Tension

1. **Folder structure vs briefer intelligence.** 03-FOLDER-STRUCTURE recommends building `specs/` with automated extraction at ship time. 04-BRIEFER recommends deferring the spec folder entirely and solving discovery through operation name cross-referencing within existing phase directories. These are competing solutions to the same problem. 04-BRIEFER's approach is lower cost and sufficient for the current project scale. 03-FOLDER-STRUCTURE's approach becomes necessary only if the project accumulates many more phases where directory scanning becomes unwieldy.

2. **Spec-anchored lifecycle vs frozen archives.** 01-METHODOLOGY makes a strong case for elevating CASES.md from "planning artifact" to "living specification" with an AUTHORITATIVE state and spec-first discipline for all changes. But 04-BRIEFER and 02-TOOLS both warn that the maintenance cost is real and the benefit is theoretical until the project actually needs to modify shipped specs. The spec-anchored model is the right long-term direction but implementing it now (adding state markers, changelog sections, spec-first discipline for bug fixes) adds overhead before the project has evidence it is needed.

3. **INDEX.md auto-generation vs heuristic discovery.** 03-FOLDER-STRUCTURE designs an auto-generated INDEX.md with metadata headers. 04-BRIEFER proposes regex-based operation name matching against existing files. The INDEX.md approach is more structured but requires extraction tooling. The regex approach is scrappy but works today. Neither approach has been tested in a real /case session.

### Dependencies Between Research Tracks

```
04-BRIEFER (quick wins)  <-- independent, implement first
    |
    v [validates whether cross-referencing works]
    |
01-METHODOLOGY (spec lifecycle) <-- needs decision: is spec-anchored worth the cost?
    |                                                            |
    v [if yes]                                                   v [if no, stop here]
03-FOLDER-STRUCTURE (spec organization) <-- needs decision: is extraction tooling worth building?
    |
    v [extraction tooling built]
04-BRIEFER (spec folder integration) <-- reads from specs/ instead of phase dirs
```

The key insight: each layer only becomes necessary if the previous layer proves insufficient.

## Cost/Benefit Analysis

### What Is the Actual Problem Size?

Evidence from 3 completed phases with /case usage (3A has one, 1 and 2 predated /case):

- **Confirmed incidents:** 1 (Q1 handle-length in 3A, resolved during re-validation)
- **Potential incidents avoided by ROADMAP deps:** Unknown but ROADMAP `Depends on` already captures the major cross-phase dependencies
- **Estimated future incident rate:** Low for Milestone 1 (remaining phases 3B, 4, 5, 6, 7 each build on known dependencies). Potentially higher for Milestone 2+ if earlier specs are forgotten.

This is a **small, infrequent problem** today. The question is whether it gets worse.

### What Is the Cost of Doing Nothing?

- **Linear growth:** Each new phase adds more completed specs to scan. With 7 phases, the briefer must scan at most 6 prior phases. With 15 phases (Milestone 2+), it must scan 14. The ROADMAP `Depends on` field already narrows this.
- **Not exponential:** Cross-phase references are bounded by the number of services (6) and the operation count per service (5-20). The combinatorial explosion that makes enterprise spec management hard does not apply here.
- **Mitigation already in place:** Cross-phase forwarding (implemented) catches forward-tagged concerns. The problem is only untagged implicit references.

Verdict: **the cost of doing nothing grows linearly and slowly**. The project will not collapse without spec organization. But the briefer gap (not surfacing already-decided constraints) will recur.

### Implementation Cost by Tier

| Work Item | Effort | Ongoing Maintenance | Risk |
|-----------|--------|---------------------|------|
| Briefer: PROJECT.md extraction | ~15 lines to agent def | None | LOW |
| Briefer: Operation name cross-referencing | ~25 lines to agent def | None | MEDIUM (untested heuristic) |
| Briefer: Auto-detect auth dependency | ~10 lines | None | LOW |
| CASES.md: Add state marker + last-verified | ~5 min per phase | LOW (update at ship) | LOW |
| Spec folder: Directory structure + templates | ~30 min one-time | None | LOW |
| Spec folder: Extraction justfile recipe | 2-4 hours | MEDIUM (must maintain when CASES.md format evolves) | MEDIUM |
| Spec folder: INDEX.md auto-generation | Included above | Same | Same |
| Briefer: Spec folder integration | 1-2 hours | LOW | LOW |
| Spec-anchored discipline | Zero (process change) | ONGOING (every behavioral change starts at spec) | HIGH (spec rot) |
| Historical backfill (Phase 1-2) | 30-60 min | None | LOW |

### Spec Rot Risk Assessment

The 01-METHODOLOGY research documents BDD teams abandoning Cucumber specs within 2 quarters. The common pattern: specs are written enthusiastically, then maintenance is skipped under delivery pressure, then specs become misleading, then they are abandoned.

For madome specifically:
- **Mitigating factor:** Solo developer + AI workflow. No coordination overhead. The developer IS the only person who can let specs rot.
- **Aggravating factor:** AI agents cannot currently detect spec drift automatically. If a behavioral change happens without a spec update, nothing catches it until the next validate/verify cycle.
- **Honest assessment:** The project has already produced CASES.md for only 1 of 3 phases that needed it (Phase 1, 2 predated /case). The convention is new enough that the developer's spec discipline is untested across many phases. Adding more spec maintenance before the existing convention is proven increases the risk of spec abandonment.

## Recommendation Tiers

### Tier 1 -- Do Now (high value, low cost)

These are briefer improvements. No new files, no new conventions, no maintenance burden. Implement before Phase 3B's /case session.

**1A. Enhanced PROJECT.md extraction in case-briefer**
- Read service topology, auth policy, cross-service patterns, API conventions
- Output as `## Architectural Context` section in CASE-BRIEFING.md
- Effort: ~15 lines added to `case-briefer.md`

**1B. Operation name cross-referencing**
- When CONTEXT.md mentions `User.CreateUser`, find that operation's spec in dependency phase CASES.md
- Extract constraints and failure modes, include as `Referenced Operations` subsection
- Directly prevents the Q1 handle-length type of Open Question
- Effort: ~25 lines added to `case-briefer.md`

**1C. Auto-detect authentication dependency**
- Any phase with REST endpoints gets auth CASES.md as implicit dependency source
- Based on PROJECT.md "all endpoints require authentication" policy
- Effort: ~10 lines

**Total Tier 1 effort:** ~50 lines of agent definition changes. No tooling. No folder restructuring. Immediate value.

### Tier 2 -- Do If Committed (medium value, medium cost)

These make sense if the developer decides the briefer improvements alone are insufficient after testing them on 3B.

**2A. CASES.md metadata header**
- Add spec state (DRAFT/VALIDATED/AUTHORITATIVE), last-verified date
- Trivial per-file overhead, but establishes the foundation for spec-anchored lifecycle
- Effort: ~5 min per phase at ship time

**2B. Service-organized spec folder (Candidate B structure)**
```
.planning/specs/
  INDEX.md
  auth/operations.md, decisions.md, errors.md
  gateway/operations.md, interactions.md
  user/operations.md, decisions.md
  catalog/operations.md, decisions.md
```
- Automated extraction via `just extract-specs <phase>` at ship time
- Phase directories remain untouched (historical record)
- specs/ becomes the curated, maintained view
- Effort: 2-4 hours for extraction recipe, 30 min for initial structure

**2C. Briefer spec folder integration**
- Read INDEX.md instead of ROADMAP dependency traversal
- Operation-centric lookup instead of phase-centric scanning
- Only valuable AFTER 2B is populated with real data
- Effort: 1-2 hours

**Total Tier 2 effort:** ~5-8 hours. Moderate ongoing maintenance (extraction script must track CASES.md format changes).

### Tier 3 -- Defer or Skip (uncertain value, high cost)

These ideas are well-designed in the research but are overengineering for the current project state.

**3A. Full spec-anchored discipline**
- Every behavioral change starts at CASES.md, not code
- Bug fixes update spec first, then code
- Cross-phase spec modification rules
- Skip for now. The discipline has never been tested. Start with the convention on a natural occasion (when Phase 3B actually modifies Phase 3A behavior) and evaluate whether the overhead is worth it.

**3B. Drift detection automation**
- CI check mapping case IDs to test functions
- Automated behavioral audit (AI-assisted)
- Skip entirely. The project has 1 developer. Manual discipline + gsd:validate is sufficient. CI automation for spec drift is enterprise-scale tooling for a community project.

**3C. Historical backfill (Phase 1-2 specs)**
- Retroactively creating CASES.md-equivalent specs for Phase 1 and Phase 2
- Do demand-driven only (when a later phase actually needs to reference Phase 2 behavior)
- Phase 2 user operations are the most likely candidate (referenced by auth service)

**3D. Spec versioning/changelog**
- Inline changelog section in CASES.md
- Git history is sufficient. Do not add another section to maintain.

## Decision Framework

### Option A: Full Spec-Driven Workflow

Implement all tiers. Living specs, service folders, extraction tooling, spec-first discipline.

| Dimension | Assessment |
|-----------|-----------|
| Effort | ~10-15 hours upfront + ongoing spec maintenance |
| Payoff | Complete spec organization, discoverable contracts, drift protection |
| Risk | High -- spec rot if discipline lapses; extraction tooling maintenance |
| When it makes sense | If the project grows to 15+ phases, multiple contributors, or Milestone 2+ scope |

### Option B: Briefer Improvements Only (Recommended)

Implement Tier 1 only. Smarter briefer, no new artifacts, no new conventions.

| Dimension | Assessment |
|-----------|-----------|
| Effort | ~1-2 hours (agent definition changes) |
| Payoff | Prevents the Q1-type gap; surfaces cross-phase constraints; zero maintenance burden |
| Risk | Low -- worst case, the heuristics produce some false positives that the developer ignores |
| When it makes sense | Now. The problem is small, the fix is small, the cost of being wrong is negligible. |

### Option C: Briefer + Lightweight Spec Index

Implement Tier 1 + Tier 2A + partial Tier 2B (folder structure and manual extraction, no automated tooling).

| Dimension | Assessment |
|-----------|-----------|
| Effort | ~3-4 hours upfront + manual spec extraction at ship time |
| Payoff | Organized spec discovery + smarter briefer |
| Risk | Medium -- manual extraction may be skipped under pressure (spec rot at the folder level) |
| When it makes sense | If Tier 1 proves insufficient during 3B and the developer wants better organization |

### Option D: Do Nothing

Keep the current workflow unchanged.

| Dimension | Assessment |
|-----------|-----------|
| Effort | Zero |
| Payoff | None -- the Q1-type gap will recur |
| Risk | Low near-term, slowly growing as phases accumulate |
| When it makes sense | If the Q1 incident is genuinely not a problem worth solving |

**Recommendation: Option B now. Re-evaluate for Option C after Phase 3B's /case session validates whether the briefer improvements actually prevent the Q1-type gap.**

## Open Questions Requiring Developer Input

**OQ-1: Is the Q1 handle-length incident representative or anomalous?**
If this was a one-off caused by Phase 2 predating the /case convention (so no Phase 2 CASES.md existed for the briefer to find), then even Tier 1 improvements may not help -- the constraint was only in code, not in any spec. If the problem is expected to recur with future phases that DO have CASES.md, then Tier 1 directly addresses it.

**OQ-2: How much do you actually read completed CASES.md when starting a new phase?**
The research assumes the briefer is the primary consumer of cross-phase specs. If the developer manually reads prior CASES.md during the discuss phase, the briefer gap matters less. If the developer relies entirely on the briefer and /case process, the gap matters more.

**OQ-3: Is spec-first discipline for bug fixes realistic?**
01-METHODOLOGY recommends: "bug fix changes behavior -> update CASES.md first, then fix code+tests." In practice, will you actually do this? If not, the spec-anchored model collapses. Honest self-assessment needed before investing in Tier 2+.

**OQ-4: Should the spec folder wait until after Milestone 1?**
03-FOLDER-STRUCTURE proposes building it now. But Milestone 1 is the natural consolidation point (all phases shipped, audit complete). Building the spec folder at milestone completion is a batch operation that avoids the incremental maintenance during active development.

**OQ-5: What is the minimum that makes the briefer "good enough"?**
The briefer currently handles ~90% of what it needs to. The Q1 gap was caught during re-validation. Is "catches most things, misses some, developer catches the rest" acceptable? Or is "never misses an already-decided constraint" the bar?

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Methodology (01) | MEDIUM-HIGH | Well-researched patterns; applicability to solo+AI workflow is judgment-based |
| Tools (02) | HIGH | Comprehensive landscape survey; clear conclusion (no tool to adopt) |
| Folder Structure (03) | HIGH | Thorough candidate analysis; recommendation is well-reasoned |
| Briefer Intelligence (04) | HIGH | Based on real artifacts and confirmed gaps from 3A experience |
| Cost/Benefit judgment | MEDIUM | Limited data points (1 confirmed incident across 3 phases) |
| Spec rot risk | HIGH | Extensively documented in BDD/SDD literature, directly applicable |

**Overall: MEDIUM-HIGH.** The individual research tracks are solid. The uncertainty is in the cost/benefit trade-off -- whether the problem justifies the solution -- which only the developer can judge.

### Gaps to Address

- **No real-world test of briefer cross-referencing.** Strategy A (operation name matching) is untested. Phase 3B's /case session is the first opportunity to validate.
- **Phase 2 has no CASES.md.** The most-referenced dependency (user service operations) has no structured spec. Briefer cross-referencing only works against phases WITH CASES.md. This is the biggest gap: the fix for Q1's problem (operation cross-referencing) would not have actually helped Q1 (Phase 2 had no CASES.md). The briefer improvement prevents FUTURE instances of this gap, not past ones.
- **Spec-anchored discipline is entirely untested.** No phase has ever modified a shipped CASES.md. The workflow change is theoretical.

## Sources

Aggregated from all 4 research files. See individual files for full citations.

### Primary (HIGH confidence)
- Martin Fowler's SDD analysis (spec-first/anchored/as-source taxonomy)
- Thoughtworks Technology Radar (SDD emerging practice, failure modes)
- Project-internal artifacts: case-briefer.md, CASE-BRIEFING.md (3A), CASE-SCRATCH.md (3A), CONTEXT.md files, PROJECT.md, ROADMAP.md, WORKFLOW.md
- MADR ADR categorization patterns

### Secondary (MEDIUM confidence)
- GitHub Spec Kit, Kiro, Tessl, OpenSpec -- tool-specific analysis
- Scott Logic critical review of Spec Kit
- BDD/Cucumber failure mode literature
- DependEval (LLM dependency recognition), Graph-RAG requirements traceability

### Tertiary (LOW confidence)
- Various Medium articles on SDD tool comparisons
- Community blog posts on brownfield SDD patterns

---
*Research completed: 2026-03-26*
*Decision needed: Option B (briefer improvements only) vs Option C (briefer + spec folder) -- developer input required on OQ-1 through OQ-5*
