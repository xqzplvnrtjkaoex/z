# /case Skill Redesign: Unified Specification

**Synthesized:** 2026-03-24
**Sources:** WORKFLOW-RESEARCH.md, CASE-BRIEFER-REDESIGN.md, CASE-VALIDATOR-REDESIGN.md, CASE-SIDE-EFFECTS-DRAFT.md
**Purpose:** Actionable specification for editing case.md, case-briefer.md, and case-validator.md

---

## 1. Executive Summary

The /case skill's two subagents (case-briefer, case-validator) are entirely code-dependent, but /case runs after discuss-phase and before plan-phase -- a point in the pipeline where no implementation code exists for new phases. Both agents are non-functional for their intended purpose. This redesign replaces code scanning with planning-document-based methodologies: the briefer extracts operations from CONTEXT.md endpoint tables and proto RPC lists; the validator cross-references discovered cases against CONTEXT.md decisions and ROADMAP.md requirements. Additionally, the orchestrator (case.md) receives side effect enhancements that systematically surface domain events, cache mutations, and related entity updates through the existing S/F/E case structure, plus three minor fixes for resume handling, dispatch guidance, and tool permissions.

---

## 2. Workflow Integration

### Where /case fits

```
discuss --> /case --> plan --> execute --> verify --> ship
```

/case reads the output of discuss-phase and produces input for plan-phase. It is the behavioral specification layer between "what decisions have we locked?" and "what atomic tasks do we plan?"

### Artifact flow

| Reads | Produces | Consumed by |
|-------|----------|-------------|
| CONTEXT.md (locked decisions, endpoint tables, proto RPC lists, constraints) | CASE-BRIEFING.md (operation inventory with interfaces, inputs, outputs, auth, constraints) | /case main conversation (Protester) |
| ROADMAP.md (phase requirements, success criteria) | CASE-SCRATCH.md (incremental case data per operation) | /case main conversation (resume) |
| REQUIREMENTS.md (requirement ID descriptions) | XX-CASES.md (final deliverable: S/F/E case tables per operation) | plan-phase planner, /test-gen (planned) |
| PROJECT.md (architecture reference) | | |

### CLAUDE.md instructions needed

Add an "Extended Workflow" section to CLAUDE.md:

1. Pipeline order: discuss -> /case -> plan -> /test-gen -> execute
2. CASES.md as additional input for plan-phase (optional, graceful degradation)
3. Case ID referencing convention (S1, F3, E2) in PLAN.md acceptance criteria
4. When CASES.md exists, planner maps must-priority cases to required test tasks

Integration mechanism: CLAUDE.md instructions (all GSD agents read CLAUDE.md). No GSD plugin modifications needed.

---

## 3. Orchestrator Changes (case.md)

### 3.1 Side Effect Enhancements (6 changes)

All changes come from CASE-SIDE-EFFECTS-DRAFT.md. They integrate side effect discovery into the existing S/F/E framework without adding a new case category.

| ID | Location | Change |
|----|----------|--------|
| C1 | Step 3c-vi | Replace vague 4-bullet probe with systematic category-based batch proposal (domain events, related entity updates, cache mutations, audit/logging, notifications, external system calls). Use SE_ working labels during discussion. |
| C2 | Step 3d | Add "Side effects identified: N" to review template. Add verification instruction: every side effect from 3c-vi must appear in at least one case's Expected Outcome. |
| C3 | Step 3e | Add optional "### Side Effects" sub-section to CASE-SCRATCH.md format between Rules and Cases. Quick-reference inventory, not a case category. |
| C4 | output_format | Add optional "### Side Effects" sub-section to CASES.md format. Add "Expected Outcome column guidance" -- success cases assert side effects OCCURRED, failure cases assert side effects DID NOT occur. |
| C5 | Step 4 | Add side effect consistency probes to cross-operation checks: consistent event emission, cascade behavior, audit logging patterns, failure suppression. |
| C6 | success_criteria | Add explicit criterion: "Side effects reflected in Expected Outcome for all relevant cases." |

### 3.2 Minor Fixes (3 changes)

| ID | Location | Change |
|----|----------|--------|
| F1 | Step 1d | Replace vague `{relevant source paths}` placeholder with concrete derivation rules tied to 4-layer architecture (proto files, handlers, domain, usecases, adapters, tests) with annotation format. **NOTE:** This fix applies to the code-based dispatch path. The text-based dispatch (see 3.3) replaces this entirely for pre-implementation phases. |
| F2 | Step 1c | Extend resume check to also look for CASE-SCRATCH.md (not just CASES.md). Handle interrupted sessions where scratch exists but final output does not. |
| F3 | Front matter | Add TaskCreate and TaskUpdate to allowed-tools list. |

### 3.3 Briefer Dispatch Changes

Replace the current `<files_to_read>` dispatch in Step 1d with planning document paths for pre-implementation phases:

**New dispatch for text-based briefer:**
```xml
<files_to_read>
- .planning/ROADMAP.md -- phase description, success criteria, requirement IDs
- {phase_dir}/*-CONTEXT.md -- locked decisions, endpoint tables, constraints
- .planning/REQUIREMENTS.md -- requirement ID descriptions
- .planning/PROJECT.md -- architecture reference (service topology, patterns)
</files_to_read>
```

The orchestrator determines mode by checking: does relevant service code exist beyond stubs? If no substantive code exists, use text-based dispatch. CONTEXT.md's `<code_context>` section already summarizes relevant existing code, so the briefer does not need to scan code directly even for incremental phases.

**Design decision override applied:** Single mode only -- discuss-output based (CONTEXT.md + ROADMAP.md). No code scanning at all. Not as primary, not as fallback. The briefer agent receives only planning documents in `<files_to_read>`, never source code paths.

### 3.4 Validator Dispatch Changes

Replace the current `<files_to_read>` dispatch in Step 5 with artifact paths:

**New dispatch:**
```xml
<objective>
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
</requirements>
```

Key change: `<files_to_read>` (code paths) replaced with `<context_file>` and `<requirements>`. No source code paths passed.

---

## 4. Briefer Redesign (case-briefer.md)

### Methodology: Text-Based Extraction Only

**Design decision override:** The briefer research recommended dual-mode (text-based + code-based). This has been REJECTED. Single mode only: extract operations from CONTEXT.md + ROADMAP.md. No code scanning -- not as primary, not as fallback.

Rationale: discuss phase resolves all gray areas; CONTEXT.md contains all needed decisions; code does not exist at /case step for new phases; for existing code phases, discuss still captures the target behavior.

**7-step extraction process:**

1. **Parse endpoint tables** from CONTEXT.md `<decisions>` section. Each table row with HTTP method becomes one operation.
2. **Parse proto RPC lists** from CONTEXT.md decisions. Cross-reference with endpoint tables to classify as REST-exposed vs internal-only.
3. **Extract inputs/outputs** from decision details. Classify each field as EXPLICIT (source: D-XX), INFERRED (derived from decisions + domain knowledge), or UNKNOWN (not mentioned).
4. **Extract auth requirements** from route tier tables (public, protected, verified, admin, scraper, recovery).
5. **Extract domain constraints** from decisions. Attach each constraint to the operation(s) it governs.
6. **Identify cross-service operations** from orchestration decisions (e.g., D-120 deactivation flow).
7. **Group and write** by natural category (CONTEXT.md sections, endpoint path prefixes).

**Decision tree for operation inclusion:**
1. In CONTEXT.md endpoint table? -> EXPLICIT operation
2. In CONTEXT.md proto RPC list? -> Check if REST-exposed (already captured), internal cross-service (include as INTERNAL), or infrastructure-only (skip unless phase targets it)
3. In ROADMAP success criterion but not detailed? -> PARTIAL confidence
4. Logically required by other operations? -> Note as side-effect, not separate operation
5. None of the above? -> Do not include

### Input Contract Changes

**Remove:** All source code paths (proto/*.proto, services/*/src/)
**Replace with:** Planning document paths only:
- `.planning/ROADMAP.md`
- `{phase_dir}/*-CONTEXT.md`
- `.planning/REQUIREMENTS.md`
- `.planning/PROJECT.md` (reference only)

### Output Contract Changes (CASE-BRIEFING.md)

**Removed fields:**
- "Existing tests" -- meaningless pre-implementation

**Renamed fields:**
- "Existing validation" -> "Decided constraints" (source: D-XX refs instead of file:line)
- "Domain constraints" -> merged into "Decided constraints"

**New fields:**
- "Open decisions" -- items from CONTEXT.md Claude's Discretion section. Tells Protester which areas are still flexible.
- "Requirements" -- REQ-ID list linking operations to requirements for traceability.
- "Extraction Confidence" -- per-operation table: EXPLICIT / INFERRED / PARTIAL.

**Unchanged:**
- CASE-BRIEFING.md overall structure (grouped operations)
- Per-operation fields: Name, Interface, Auth, Inputs, Outputs
- Category grouping by domain cluster
- Return protocol: `## BRIEFING COMPLETE` with counts

### Quality Gate Changes

**Remove:** "Every operation's source code was read," "Each gap references file:line"
**Add:**
- All endpoint table rows from CONTEXT.md captured as operations
- All proto RPCs accounted for (as operations or noted as infrastructure-skip)
- Each ROADMAP success criterion maps to at least one briefed operation
- Decided constraints reference decision IDs (D-XX)
- Open decisions reference Claude's Discretion items
- Inferred fields marked as `[Inferred: ...]`
- Unknown fields marked as `[Not specified]`
- No operations invented beyond what CONTEXT.md describes
- No implementation recommendations
- Extraction confidence table included

### Agent Definition Changes

- Remove code-based extraction methodology entirely (not dual-mode)
- Update description: "Extracts operations, constraints, and decision context from phase planning documents"
- Tool set: Read, Grep, Glob, Write (Glob retained for finding CONTEXT.md in phase directories)
- Model: sonnet (unchanged)

---

## 5. Validator Redesign (case-validator.md)

### Methodology: 5 Artifact Cross-Reference Gap Checks

The validator performs exactly 5 checks, ordered from highest-value to lowest:

| Check | Cross-References | Finds |
|-------|-----------------|-------|
| **A: Requirement Coverage** | ROADMAP.md reqs vs CASE-SCRATCH.md | Requirement with no covering operation or no success case |
| **B: Decision Coverage** | CONTEXT.md behavioral decisions vs CASE-SCRATCH.md | Behavioral decision with no exercising case |
| **C: Completeness** | CASE-SCRATCH.md internal consistency | Operation missing S or F cases, auth cases for protected endpoints, unexercised rules |
| **D: Consistency** | CASE-SCRATCH.md operations against each other | Inconsistent error formats, auth enforcement, pagination, not-found behavior across operations |
| **E: Briefing Coverage** | CASE-BRIEFING.md operations vs CASE-SCRATCH.md | Operation identified by briefer but absent from scratch (accidentally forgotten) |

**Behavioral decision filtering heuristic:** Only flag decisions that answer "what should the caller observe?" (error codes, boundary values, auth tiers, observable behavior). Skip structural decisions (architecture patterns, file organization, naming conventions) and informational decisions (background context).

**Decision grouping:** Related decisions (e.g., D-21 through D-23 all about recovery codes) are checked as a cluster, not individually. Coverage at the cluster level suffices.

**Finding cap:** Maximum 15 findings. If more are generated, rank by severity and present top 15 with a note about remaining items.

### Input Contract Changes

**Remove:** `<files_to_read>` with source code paths
**Replace with:**
- `<context_file>` -- path to XX-CONTEXT.md
- `<requirements>` -- phase REQ-IDs + path to ROADMAP.md and REQUIREMENTS.md

**Existing tags kept:** `<objective>`, `<cases_file>`, `<briefing_file>`

**Tool set change:** Read, Grep (remove Glob -- agent reads specific named files, not searching filesystem)

### Output Contract Changes

**Old categories:** Gaps, Conflicts, Suggestions
**New categories:**

| Category | What It Catches | Severity |
|----------|----------------|----------|
| **Requirement Gaps** | ROADMAP.md requirement with no covering case | High |
| **Decision Gaps** | CONTEXT.md behavioral decision with no exercising case | High/Medium |
| **Consistency Issues** | Operations handling cross-cutting concerns differently, or cases contradicting locked decisions | Medium |
| **Completeness Gaps** | Operation missing expected case categories (no S, no F, no auth cases, unexercised rules) | Low/Medium |
| **Briefing Gaps** | Operation from briefer not discussed in scratch | Medium |

Each finding includes: category, severity, source reference (CONTEXT.md D-XX / ROADMAP.md REQ-ID), quoted text from source artifact, suggested action with concrete case proposal.

**Return protocol:** `## VALIDATION COMPLETE` with per-category counts (unchanged structure, new categories).

### Quality Gate Changes

**Remove:** "Every operation's source code was read," "Each gap finding references file:line," "Distinguish code gaps from case gaps"
**Add:**
- All five gap checks executed
- Each finding references specific artifact location (D-XX, REQ-ID, operation name)
- Each finding quotes relevant text from source artifact
- Each finding includes suggested action with S/F/E case proposal
- No finding duplicates existing case in CASE-SCRATCH.md
- Structural/non-behavioral decisions filtered out
- Public-tier operations not flagged for missing auth failure cases
- Findings prioritized (requirement and security gaps before completeness)

### Agent Name

Keep `case-validator`. Update description in frontmatter to: "Cross-checks discovered behavioral cases against planning artifacts (CONTEXT.md, ROADMAP.md, CASE-BRIEFING.md) to find requirement gaps, decision gaps, and consistency issues."

### When to Skip Validation

The orchestrator should skip validator dispatch when:
- Operation count <= 2 AND no CONTEXT.md exists
- Note: "Validation skipped (small phase, no locked decisions)"

---

## 6. WORKFLOW.md Deliverable

The WORKFLOW-RESEARCH.md contains comprehensive documentation of the extended GSD workflow. A distilled `.planning/WORKFLOW.md` should be created (separate task from this redesign) containing:

**Key sections:**
1. Extended pipeline diagram (discuss -> /case -> plan -> /test-gen -> execute -> verify -> ship)
2. Stage-by-stage breakdown with artifact inputs/outputs per stage
3. Artifact dependency chain (complete graph showing what reads what)
4. Artifact file location conventions ({phase_dir}/{padded}-{artifact}.md)
5. Iterative return paths (which stages can loop back to which, with triggers)
6. CASES.md optionality for planner (works with or without)
7. /test-gen design considerations (planned, captures future integration point)

**From the workflow research, the critical constraint to document:** At /case time for new phases, no implementation code exists. The briefer works from planning documents only. For later phases with existing code from prior phases, CONTEXT.md's `<code_context>` section already summarizes integration points.

---

## 7. Open Questions

### Must resolve before implementation

| # | Question | Source | Resolution needed for |
|---|----------|--------|-----------------------|
| 1 | Should the orchestrator detect mode (text-based vs code-based) or always use text-based? | Briefer research Section 11 | **RESOLVED by design override: always text-based. No mode detection needed.** |
| 2 | Does the validator need the full REQUIREMENTS.md file path, or just the REQ-IDs extracted by the orchestrator? | Validator research Section 6 | Validator dispatch template. Recommend: pass both the IDs and the file path so the validator can look up requirement descriptions. |
| 3 | Should Fix 1 (files_to_read guidance) still be applied given the design override to text-based only? | Side effects draft, Fix 1 | **Yes, but adapt it:** replace code-based derivation rules with the text-based file list. The fix's structure (annotated paths with purpose) is valuable; the content changes to planning document paths. |

### Can resolve during implementation

| # | Question | Source | Notes |
|---|----------|--------|-------|
| 4 | What is the right threshold for "skip validation" -- operation count <= 2 or some other number? | Validator research Section 8 | Judgment call. Start with <= 2, adjust after use. |
| 5 | Should the finding cap be 15 or some other number? | Validator research Section 10 | Start with 15, observe developer response. |
| 6 | Should the briefer include RESEARCH.md from prior plan-phase runs if it exists? | Briefer research Section 8 | Low priority. RESEARCH.md is technology-focused, not behavior-focused. Include as optional reference if it exists. |
| 7 | Exact format for the "Extraction Confidence" table in CASE-BRIEFING.md -- per-operation or per-field? | Briefer research Section 4 | Per-operation is sufficient. Per-field would be too verbose. |

---

## 8. Implementation Order

Changes have dependencies. Recommended sequence:

### Phase 1: Orchestrator Minor Fixes (independent, low risk)

1. **F3:** Add TaskCreate/TaskUpdate to case.md allowed-tools (front matter edit)
2. **F2:** Extend resume check to handle CASE-SCRATCH.md (Step 1c edit)

These are standalone fixes that improve the current skill regardless of the redesign.

### Phase 2: Briefer Redesign (largest change, unblocks Phase 3)

3. **Rewrite case-briefer.md** with text-based extraction methodology only (no dual-mode):
   - New 7-step extraction process
   - New input contract (planning documents only)
   - New output contract (Decided constraints, Open decisions, Requirements, Extraction Confidence)
   - New quality gate
4. **Update case.md Step 1d dispatch** to pass planning document paths instead of source code paths. Replace F1's code-based derivation rules with the text-based file list.

The briefer must be updated before the validator because the validator reads CASE-BRIEFING.md, and the validator's Briefing Coverage check (Check E) depends on the briefer's output format.

### Phase 3: Validator Redesign (depends on briefer output format)

5. **Rewrite case-validator.md** with 5 artifact cross-reference gap checks:
   - New methodology (5 checks: requirement, decision, completeness, consistency, briefing)
   - New input contract (context_file, requirements tags)
   - New output contract (5 gap categories)
   - New quality gate
   - Behavioral decision filtering heuristic
6. **Update case.md Step 5 dispatch** to use new validator prompt template with artifact paths.

### Phase 4: Side Effect Enhancements (independent of subagent redesigns)

7. **C1:** Step 3c-vi systematic side effect probing
8. **C2:** Step 3d review with side effect verification
9. **C3:** Step 3e scratch format with Side Effects sub-section
10. **C4:** output_format with Side Effects sub-section and Expected Outcome guidance
11. **C5:** Step 4 cross-operation side effect consistency probes
12. **C6:** success_criteria side effect coverage criterion

Side effect changes are to case.md only and do not depend on the subagent rewrites. They can be applied in Phase 4 or interleaved with Phase 2/3 at discretion.

### Phase 5: CLAUDE.md + WORKFLOW.md Updates (documentation)

13. **Add "Extended Workflow" section to CLAUDE.md** documenting /case and /test-gen integration with plan-phase
14. **Create .planning/WORKFLOW.md** from distilled workflow research

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Workflow integration | HIGH | GSD official docs fetched; artifact flow verified against project structure |
| Briefer text-based methodology | HIGH | Validated against all 3 existing CONTEXT.md files; endpoint tables and RPC lists are consistent patterns |
| Validator gap check design | HIGH | 5 checks are orthogonal, exhaustive, and each cross-references exactly 2 artifacts |
| Behavioral decision filtering | MEDIUM | Heuristic is sound but untested against all 120+ Phase 3 decisions; may need iteration |
| Side effect enhancements | HIGH | Changes are text edits to case.md with clear rationale; no external dependencies |
| Design override (text-only, no code) | HIGH | Owner decision, well-reasoned; simplifies implementation significantly |

**Overall: HIGH.** All research converges on the same diagnosis (code-dependent agents fail pre-implementation) and the same solution (planning-document-based extraction). The design override to single-mode text-based simplifies the briefer considerably.

---

## Sources

Aggregated from all 4 research files:

### Primary (HIGH confidence)
- GSD GitHub Repository -- README.md, USER-GUIDE.md, COMMANDS.md, ARCHITECTURE.md, FEATURES.md (fetched via WebFetch)
- `.claude/agents/case-briefer.md` -- current briefer agent definition
- `.claude/agents/case-validator.md` -- current validator agent definition
- `.claude/commands/case.md` -- /case skill orchestrator
- `.planning/phases/03-authentication/03-CONTEXT.md` -- most detailed CONTEXT.md (120+ decisions)
- `.planning/phases/02-user-profile/02-CONTEXT.md` -- second CONTEXT.md
- `.planning/phases/01-foundation-and-gateway-infrastructure/01-CONTEXT.md` -- first CONTEXT.md
- `.planning/ROADMAP.md` -- phase structure and requirement IDs
- `.planning/REQUIREMENTS.md` -- requirement definitions

### Secondary (HIGH confidence)
- `.planning/research/CASE-SKILL-SYNTHESIS.md` -- /case skill design and technique layers
- `.planning/research/CASE-AGENT-SYNTHESIS.md` -- agent architecture decisions
- `.planning/research/CASE-AGENT-NEEDS.md` -- per-step subagent analysis
- `.planning/research/CASE-CATEGORY-COMPLETENESS.md` -- S/F/E category system audit
- `.planning/research/GSD-AGENT-PATTERNS.md` -- 10 design patterns from GSD source

---

*Synthesis completed: 2026-03-24*
*Applies design decision override: single mode text-based briefer (no code scanning)*
