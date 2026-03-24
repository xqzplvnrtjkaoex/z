# GSD Extended Workflow Research

**Researched:** 2026-03-24
**Domain:** GSD workflow stages, custom skill integration points, artifact flow
**Confidence:** HIGH (GSD docs fetched from official repo; project artifacts read directly)

---

## Summary

GSD (Get Shit Done) is a meta-prompting, context engineering, and spec-driven development framework for AI coding agents. It organizes software development into a staged pipeline where each stage produces artifacts that feed into downstream stages. The official workflow is: **init -> discuss -> plan -> execute -> verify -> ship**. This project extends it with two custom skills: **/case** (behavioral case discovery, between discuss and plan) and **/test-gen** (test skeleton generation, between plan and execute, planned but not yet built).

The extended workflow is iterative, not linear. Any step can trigger a return to an earlier step. The key constraint is that at the /case step, only planning documents exist -- no implementation code, no proto files for the current phase's scope. This shapes what /case can analyze: it works from ROADMAP.md requirements, CONTEXT.md decisions, and any *existing* code from prior phases, not from code that the current phase will create.

**Primary recommendation:** Document the extended workflow as a first-class extension of GSD, with explicit artifact dependency chains. The CLAUDE.md file should instruct GSD agents to look for XX-CASES.md as an additional input artifact during plan-phase, and for test skeletons during execute-phase.

---

## 1. Official GSD Workflow

### 1.1 Core Stages

The official GSD workflow has 8 stages, with 5 forming the core development cycle:

| Stage | Command | Purpose | Key Agent(s) |
|-------|---------|---------|--------------|
| **Initialize** | `/gsd:new-project` | Create foundational docs | 4 parallel researchers, synthesizer, roadmapper |
| **Discuss** | `/gsd:discuss-phase N` | Lock implementation preferences | Main conversation (interactive) |
| **UI Design** | `/gsd:ui-phase N` | Visual design contract (frontend only) | Main conversation + component audit |
| **Plan** | `/gsd:plan-phase N` | Research domain + create atomic task plans | Phase researcher, planner, plan checker (max 3 iterations) |
| **Execute** | `/gsd:execute-phase N` | Implement tasks in parallel waves | Wave-parallel executors (fresh 200K context each) |
| **Verify** | `/gsd:verify-work N` | User acceptance testing + auto-diagnosis | Main conversation + debugger agents |
| **Ship** | `/gsd:ship N` | Create PR from completed work | Main conversation + gh CLI |
| **Milestone** | `/gsd:complete-milestone` | Archive release, tag, advance | Auditor agent |

### 1.2 Official Artifact Flow

```
/gsd:new-project
  -> PROJECT.md, REQUIREMENTS.md, ROADMAP.md, STATE.md, config.json

/gsd:discuss-phase N
  reads: PROJECT.md, REQUIREMENTS.md, ROADMAP.md
  writes: {phase}-CONTEXT.md

/gsd:ui-phase N (frontend only)
  reads: CONTEXT.md, PROJECT.md
  writes: {phase}-UI-SPEC.md

/gsd:plan-phase N
  reads: PROJECT.md, REQUIREMENTS.md, CONTEXT.md, STATE.md
  writes: {phase}-RESEARCH.md, {phase}-{N}-PLAN.md (1+ plans), {phase}-VALIDATION.md

/gsd:execute-phase N
  reads: PLAN.md files, PROJECT.md, STATE.md, CONTEXT.md, RESEARCH.md
  writes: per-plan SUMMARY.md, VERIFICATION.md, atomic git commits

/gsd:verify-work N
  reads: VERIFICATION.md, PLAN.md files, codebase
  writes: {phase}-UAT.md (+ fix plans if failures)

/gsd:ship N
  reads: completed phase state
  writes: PR with auto-generated body
```

### 1.3 Plan Structure (XML Format)

Each PLAN.md contains atomic tasks in XML structure with mandatory fields:

| Field | Purpose |
|-------|---------|
| `name` | Human-readable task identifier |
| `files_modified` | Explicit list of files this task creates/edits |
| `read_first` | Files the executor must read before starting (MUST include files being modified) |
| `action` | Step-by-step implementation instructions |
| `acceptance_criteria` | Grep-verifiable conditions that prove the task is done |
| `verify` | Commands to run after implementation |
| `done` | What "complete" means for this specific task |

Frontmatter includes: `wave` (dependency ordering), `depends_on` (prior plans that must complete first), `autonomous` (can run without user interaction).

### 1.4 Wave-Based Execution Model

Plans are grouped into dependency-ordered waves:

```
Wave 1: Plans with no dependencies -> run in parallel (fresh 200K context each)
Wave 2: Plans depending on Wave 1 -> run after Wave 1 completes
Wave 3: Plans depending on Wave 2 -> ...
```

Each executor receives: specific PLAN.md, PROJECT.md, STATE.md, CONTEXT.md, RESEARCH.md. Executors create atomic git commits per task. Post-wave verification runs after each wave completes.

### 1.5 Quality Assurance Mechanisms

| Mechanism | When | What |
|-----------|------|------|
| **Plan Checking** | After planning, before execution | Verifies plans against 8 dimensions (requirement coverage, task atomicity, dependency ordering, etc.). Up to 3 correction loops. |
| **Nyquist Validation** | During planning | Maps automated test coverage to requirements. Ensures feedback mechanisms exist within seconds of task completion. |
| **Post-Execution Verification** | After each wave | Automated checks that codebase delivers what plans promised. |
| **Node Repair** | On task verification failure | Auto-diagnoses and generates fix plans. |
| **UAT** | After all waves | User acceptance testing with AI diagnosis support. |

---

## 2. Extended Workflow

### 2.1 Full Pipeline

```
              OFFICIAL GSD                         CUSTOM EXTENSIONS
         +-----------------+                  +---------------------+
         |                 |                  |                     |
init --> discuss --> /case --> (ui) --> plan --> /test-gen --> execute --> verify --> ship
         |            |                |                        |
         |   behavioral cases     research +                   |
         |   (what to test)       atomic plans           implementation
         |                        (how to build)         + tests
         |                                                     |
         +<-- iterative returns (any step can loop back) ------+
```

### 2.2 ASCII Workflow Diagram

```
                                     ITERATIVE RETURN PATHS
                               +----------------------------------+
                               |                                  |
  +--------+   +---------+   +------+   +----+   +------+   +---------+   +--------+   +------+
  |  init  |-->| discuss |-->| /case|-->| ui |-->| plan |-->|/test-gen|-->| execute |-->|verify|-->ship
  +--------+   +---------+   +------+   +----+   +------+   +---------+   +--------+   +------+
                   ^            ^                    |            |             |
                   |            |                    |            |             |
                   |            +--------------------+            |             |
                   |            plan -> case (redo from case)     |             |
                   |                                              |             |
                   +----------------------------------------------+             |
                   discuss -> (redo from discuss)                               |
                                                                                |
                   +------------------------------------------------------------+
                   execute -> discuss (redo from discuss)
```

### 2.3 Stage-by-Stage Breakdown

#### Stage 1: Discuss (`/gsd:discuss-phase N`)

| Property | Value |
|----------|-------|
| **Purpose** | Capture implementation decisions and preferences before any technical work |
| **Input artifacts** | PROJECT.md, REQUIREMENTS.md, ROADMAP.md |
| **Output artifact** | `{phase_dir}/{padded_phase}-CONTEXT.md` |
| **Output format** | Sections: `## Decisions` (locked), `## Claude's Discretion` (AI decides), `## Deferred Ideas` (out of scope) |
| **Agent** | Main conversation (interactive with user) |
| **Modes** | Standard (open-ended questions), Assumptions (Claude surfaces assumptions for correction) |

**Key decisions captured:** Technology choices, API design, error handling strategy, auth requirements, data model preferences.

#### Stage 2: /case (Behavioral Case Discovery)

| Property | Value |
|----------|-------|
| **Purpose** | Surface success, failure, and edge cases for each operation BEFORE writing tests |
| **Input artifacts** | CONTEXT.md (locked decisions), ROADMAP.md (phase requirements), RESEARCH.md (if available from prior research), existing codebase (from prior phases) |
| **Output artifact** | `{phase_dir}/{padded_phase}-CASES.md` |
| **Output format** | Per-operation sections: Rules, Success/Failure/Edge case tables (S/F/E), Open Questions, Cross-Operation Concerns, Summary |
| **Agent architecture** | Main conversation (Protester persona) + 2 subagents: case-briefer (codebase mapping), case-validator (cross-checking) |
| **Human interaction** | High -- depth-first conversation per operation |

**Key constraint:** At the /case step for a new phase, the phase's implementation code does not yet exist. The case-briefer analyzes:
- Proto definitions and service handlers from *prior completed phases*
- ROADMAP.md requirements for the current phase
- CONTEXT.md decisions for the current phase
- Any existing code that the current phase will *extend* (not create from scratch)

For greenfield phases (Phase 1, or phases creating entirely new services), the case-briefer has less to scan. Case discovery relies more heavily on the developer's domain knowledge surfaced through conversation.

**Output feeds into:** plan-phase (planner maps must-priority cases to required test tasks, references case IDs in acceptance criteria).

#### Stage 3: UI Design (`/gsd:ui-phase N`) -- Optional

| Property | Value |
|----------|-------|
| **Purpose** | Visual design contract for frontend phases |
| **Input artifacts** | CONTEXT.md, PROJECT.md |
| **Output artifact** | `{phase_dir}/{padded_phase}-UI-SPEC.md` |
| **When used** | Frontend phases only (not applicable to madome's current backend-only scope) |

#### Stage 4: Plan (`/gsd:plan-phase N`)

| Property | Value |
|----------|-------|
| **Purpose** | Research the technical domain and create atomic, executable task plans |
| **Input artifacts** | PROJECT.md, REQUIREMENTS.md, CONTEXT.md, STATE.md, **CASES.md (extended)** |
| **Output artifacts** | `{padded_phase}-RESEARCH.md`, `{padded_phase}-{N}-PLAN.md` (1+ plans), `{padded_phase}-VALIDATION.md` |
| **Agent architecture** | Phase researcher (or 4 parallel researchers), planner, plan checker (max 3 iteration loops) |
| **Human interaction** | Low -- automated research + planning, user reviews output |

**How CASES.md feeds into planning:**
1. Planner reads CASES.md as additional input alongside CONTEXT.md and RESEARCH.md
2. Must-priority cases become required test tasks or acceptance criteria in PLAN.md
3. Case IDs (S1, F3, E2) are referenced in task acceptance criteria
4. Open questions (Q1, Q2) flag items that may need resolution before implementation
5. The Nyquist validation layer maps phase requirements to test coverage, using CASES.md as the behavioral specification source

#### Stage 5: /test-gen (Test Skeleton Generation) -- Planned, Not Yet Built

| Property | Value |
|----------|-------|
| **Purpose** | Generate test skeletons from CASES.md before implementation, enabling TDD workflow |
| **Input artifacts** | CASES.md, PLAN.md files, RESEARCH.md (for stack/pattern knowledge) |
| **Output artifact** | Test skeleton files (e.g., `tests/test_*.rs` with `#[test]` stubs) |
| **When dispatched** | After plan-phase, before execute-phase |
| **Agent architecture** | TBD |

**Key design considerations:**
- Test skeletons should compile but fail (red phase of TDD)
- Each skeleton maps to one or more case IDs from CASES.md
- Skeletons include the case description as a comment and the expected assertion pattern
- The execute-phase executor implements the code that makes these tests pass
- Must account for test infrastructure from Nyquist validation (VALIDATION.md)

#### Stage 6: Execute (`/gsd:execute-phase N`)

| Property | Value |
|----------|-------|
| **Purpose** | Implement tasks from PLAN.md files in parallel waves |
| **Input artifacts** | PLAN.md files, PROJECT.md, STATE.md, CONTEXT.md, RESEARCH.md, **test skeletons (extended, when /test-gen is built)** |
| **Output artifacts** | Per-plan SUMMARY.md, VERIFICATION.md, atomic git commits |
| **Agent architecture** | Wave-parallel executors (fresh 200K context each) |
| **Human interaction** | None during execution; review after |

**How test skeletons feed into execution:**
- When /test-gen produces test skeleton files, executors find failing tests already in the codebase
- Executors write implementation code that makes tests pass (TDD green phase)
- Acceptance criteria in PLAN.md reference both case IDs and test file paths
- Without /test-gen: executors write both implementation and tests (current behavior)

#### Stage 7-8: Verify + Ship

Standard GSD verify-work and ship stages. No custom extensions needed.

### 2.4 Iterative Return Paths

The workflow is iterative. Any stage can trigger a return to an earlier stage:

| From | To | Trigger | What happens |
|------|----|---------|--------------|
| plan | case | Planner discovers missing behavioral cases | Re-run /case for specific operations, then re-plan |
| plan | discuss | Planner identifies unresolved design decisions | Re-run discuss to lock new decisions, then case + plan |
| case | discuss | Case discovery surfaces fundamental design questions | Re-run discuss, then redo case |
| execute | plan | Implementation reveals plan flaws | Generate fix plans or re-plan |
| execute | discuss | Implementation reveals fundamental design issues | Rare; redo from discuss |
| verify | execute | UAT failures | Generate fix plans, re-execute |
| verify | plan | Systemic failures requiring re-planning | Re-plan + re-execute |

**Redo semantics:** When returning to an earlier stage, all downstream artifacts from that point forward are regenerated. For example, `plan -> case` means: re-run /case, then re-plan (with updated CASES.md), then re-execute.

---

## 3. Artifact Dependency Chain

### 3.1 Complete Artifact Graph

```
PROJECT.md ─────────────────────────────────────────────────────┐
REQUIREMENTS.md ────────────────────────────────────────────────┤
ROADMAP.md ─────────────────────────────────────────────────────┤
                                                                |
/gsd:discuss-phase N                                            |
  reads: PROJECT.md, REQUIREMENTS.md, ROADMAP.md                |
  writes: CONTEXT.md                                            |
              |                                                 |
              v                                                 |
/case N                                                         |
  reads: CONTEXT.md, ROADMAP.md, existing codebase              |
  writes: CASES.md                                              |
              |                                                 |
              v                                                 |
/gsd:plan-phase N                                               |
  reads: PROJECT.md, REQUIREMENTS.md, CONTEXT.md,               |
         STATE.md, CASES.md (if exists)                         |
  writes: RESEARCH.md, PLAN.md files, VALIDATION.md             |
              |                                                 |
              v                                                 |
/test-gen N (planned)                                           |
  reads: CASES.md, PLAN.md files, RESEARCH.md                   |
  writes: test skeleton files                                   |
              |                                                 |
              v                                                 |
/gsd:execute-phase N                                            |
  reads: PLAN.md files, PROJECT.md, STATE.md, CONTEXT.md,       |
         RESEARCH.md, test skeletons (if exist)                 |
  writes: SUMMARY.md, VERIFICATION.md, git commits              |
```

### 3.2 Artifact File Locations

All phase artifacts live in `.planning/phases/{padded_phase}-{name}/`:

| Artifact | Filename Pattern | Producer | Consumers |
|----------|-----------------|----------|-----------|
| Context | `{padded}-CONTEXT.md` | discuss-phase | /case, plan-phase, execute-phase |
| Cases | `{padded}-CASES.md` | /case skill | plan-phase, /test-gen |
| Case Briefing | `CASE-BRIEFING.md` | case-briefer subagent | /case main conversation |
| Case Scratch | `CASE-SCRATCH.md` | /case main conversation | /case main conversation (resume) |
| Research | `{padded}-RESEARCH.md` | plan-phase researcher | planner, execute-phase |
| Plan | `{padded}-{N}-PLAN.md` | planner | execute-phase, /test-gen |
| Validation | `{padded}-VALIDATION.md` | plan-phase | execute-phase (Nyquist) |
| UI Spec | `{padded}-UI-SPEC.md` | ui-phase | plan-phase, execute-phase |
| Summary | `{padded}-{N}-SUMMARY.md` | execute-phase | verify-work |
| Verification | `VERIFICATION.md` | execute-phase | verify-work |
| UAT | `{padded}-UAT.md` | verify-work | (terminal) |

### 3.3 Critical Dependency: CASES.md Must Be Optional for Planner

The planner must work both with and without CASES.md:

- **With CASES.md:** Planner uses cases to create test-aware task plans. Must-priority cases become acceptance criteria. Case IDs are referenced.
- **Without CASES.md:** Planner creates plans from CONTEXT.md + REQUIREMENTS.md alone. Test tasks are generated from requirements rather than behavioral cases. This is the standard GSD behavior.

This optionality is important because:
1. Not every phase needs /case (simple phases, infra phases)
2. The user may choose to skip /case for time reasons
3. GSD should degrade gracefully when custom artifacts are absent

---

## 4. Integration Points with GSD

### 4.1 How /case Output Feeds into plan-phase

The GSD planner (gsd-planner agent) reads files listed in `<files_to_read>` during dispatch. To integrate CASES.md:

**Option A: CLAUDE.md instruction (recommended)**

Add to CLAUDE.md a section that instructs the planner to check for CASES.md:

```markdown
## Extended Workflow

When running `/gsd:plan-phase`, the planner should check for `{phase_dir}/*-CASES.md`.
If present:
- Read it as an additional input alongside CONTEXT.md
- Map must-priority cases (S/F/E with priority "must") to required test tasks
- Reference case IDs (S1, F3, E2) in task acceptance_criteria
- Flag open questions (Q1-QN) as items requiring resolution
```

**Option B: Workflow hook (if GSD supports it)**

GSD has a hook system. A pre-planning hook could inject CASES.md into the planner's file list. However, the hook system appears oriented toward custom scripts, not artifact injection.

**Recommendation:** Option A. CLAUDE.md instructions are read by all GSD agents (the executor, planner, and researcher all read CLAUDE.md). This is the simplest and most reliable integration mechanism.

### 4.2 How /test-gen Output Feeds into execute-phase

When /test-gen is built, it will produce test skeleton files in the source tree. These become part of the codebase that executors see during execute-phase. No special integration is needed -- executors already read the full codebase.

The integration point is in PLAN.md task definitions:
- `read_first` should include the test skeleton file
- `acceptance_criteria` should include "test passes" conditions
- `action` should reference "make the existing failing test pass"

### 4.3 CLAUDE.md Requirements for GSD Agent Awareness

For GSD agents to be aware of custom artifacts, CLAUDE.md needs:

```markdown
## Extended Workflow (Custom Skills)

This project uses custom skills that extend the standard GSD workflow:

### /case -- Behavioral Case Discovery
- Run after `/gsd:discuss-phase`, before `/gsd:plan-phase`
- Produces: `{padded_phase}-CASES.md` in the phase directory
- Contains: per-operation behavioral cases (Success/Failure/Edge), rules, open questions
- Plan-phase MUST check for this file and use it when present
- Case IDs (S1, F3, E2) should be referenced in PLAN.md acceptance criteria

### /test-gen -- Test Skeleton Generation (planned)
- Run after `/gsd:plan-phase`, before `/gsd:execute-phase`
- Produces: test skeleton files (compile but fail)
- Execute-phase executors should make failing tests pass (TDD green phase)

### Workflow Order
discuss -> /case -> plan -> /test-gen -> execute -> verify -> ship
(ui-phase inserted after /case for frontend phases, not applicable to this project)
```

---

## 5. The /case Constraint: No Implementation Code at Case Time

### 5.1 The Problem

When /case runs for a new phase, the phase's implementation does not yet exist. For Phase 3 (Authentication):

- Auth service source code does not exist
- Auth proto definitions may not exist (unless created in an earlier phase)
- Auth-related gateway middleware does not exist

The case-briefer subagent has limited codebase to scan.

### 5.2 What the Case-Briefer CAN Analyze

| Source | What It Provides | Available at /case Time? |
|--------|-----------------|------------------------|
| ROADMAP.md | Phase requirements (AUTH-01 through AUTH-05) | Yes |
| CONTEXT.md | Locked design decisions | Yes |
| PROJECT.md | Architecture, domain model, patterns | Yes |
| `proto/*.proto` | Service definitions from *prior phases* | Partially (user.proto exists from Phase 2; auth.proto may not) |
| Existing service code | Patterns from prior phases (user service) | Yes -- establishes conventions |
| Prior CASES.md | Case patterns from prior phases | Yes |

### 5.3 How This Shapes Case Discovery

For early-phase operations (where code does not exist):
- Case discovery relies heavily on **developer domain knowledge** surfaced through conversation
- The case-briefer produces a lighter briefing based on ROADMAP.md requirements
- The Protester persona asks more open questions and proposes fewer pre-filled cases
- Proto message types and field names come from the developer's answers, not from code scanning

For later-phase operations (where prior code establishes patterns):
- The case-briefer can extract existing validation patterns, error handling conventions, and auth requirements from prior phases
- These patterns become defaults that the Protester proposes: "In Phase 2, we used INVALID_ARGUMENT for validation errors. Same here?"
- More pre-filled proposals, fewer open questions

### 5.4 Implication for /test-gen

/test-gen runs after plan-phase, so it has access to PLAN.md files which specify the file structure, dependencies, and implementation approach. It also has CASES.md with explicit behavioral expectations. This is sufficient to generate test skeletons even without implementation code, because:

- Test function signatures come from PLAN.md (what files/modules will exist)
- Test assertions come from CASES.md (what behavior is expected)
- Test infrastructure comes from VALIDATION.md (what test framework, shared fixtures)

---

## 6. GSD Configuration Relevant to This Project

From `.planning/config.json`:

| Setting | Value | Impact on Extended Workflow |
|---------|-------|---------------------------|
| `workflow.nyquist_validation` | `true` | Plan-phase produces VALIDATION.md with test-to-requirement mapping. /case CASES.md provides the behavioral specification source for this mapping. |
| `workflow.plan_check` | `true` | Plan checker verifies plans against 8 dimensions. When CASES.md exists, checker should verify case coverage in plan acceptance criteria. |
| `workflow.research` | `true` | Research runs during plan-phase. Researcher does not need CASES.md -- it researches technical domain, not behavioral cases. |
| `git.branching_strategy` | `phase` | Phase branches created for execution. /case and /test-gen run before the phase branch is created (they produce planning artifacts, not code). |
| `commit_docs` | `true` | Planning artifacts (including CASES.md) are committed to git. |
| `model_overrides.gsd-executor` | `sonnet` | Executors use Sonnet. Case-briefer and case-validator also use Sonnet (configured in agent definition). |

---

## 7. Comparison: Standard GSD vs Extended Workflow

| Aspect | Standard GSD | Extended (with /case + /test-gen) |
|--------|-------------|----------------------------------|
| **Test source** | Requirements -> planner infers tests | Requirements -> /case discovers cases -> planner maps cases to tests |
| **Test timing** | Tests written during execute-phase | Test skeletons before execute (/test-gen), implementation during execute |
| **Behavioral spec** | Implicit in PLAN.md tasks | Explicit in CASES.md with S/F/E tables |
| **Developer input** | discuss-phase only | discuss-phase + /case (deeper behavioral discussion) |
| **Planning input** | CONTEXT.md + REQUIREMENTS.md | CONTEXT.md + REQUIREMENTS.md + CASES.md |
| **TDD support** | Planner may include TDD tasks | /test-gen produces failing tests that executors make pass |
| **Iteration triggers** | Plan/execute failures | Case discovery can trigger return to discuss |

---

## 8. Recommendations

### 8.1 CLAUDE.md Updates Needed

Add an "Extended Workflow" section to CLAUDE.md documenting:
1. The extended pipeline order: discuss -> /case -> plan -> /test-gen -> execute
2. CASES.md as an additional input for plan-phase
3. Case ID referencing convention in PLAN.md acceptance criteria
4. /test-gen output as pre-existing test files for executors

### 8.2 GSD Agent Instructions

For plan-phase integration without modifying GSD source code:
- CLAUDE.md instructions are the integration mechanism (all GSD agents read CLAUDE.md)
- The planner's `<files_to_read>` will include CLAUDE.md, which tells it to also check for CASES.md
- No GSD plugin modifications needed

### 8.3 /test-gen Design Considerations

When designing /test-gen:
1. Read CASES.md for behavioral expectations
2. Read PLAN.md files for file structure and module layout
3. Read RESEARCH.md for testing patterns and libraries
4. Read VALIDATION.md for test infrastructure and framework config
5. Produce test files that compile but fail
6. Map each test function to a case ID from CASES.md
7. Include case description as a doc comment on each test

### 8.4 Workflow Documentation Location

This document captures the workflow for reference. The actionable instructions should be:
- CLAUDE.md: operational instructions for GSD agents (what to read, how to use CASES.md)
- ROADMAP.md: no changes needed (already references /case in phase workflow)
- `.claude/commands/case.md`: already documents /case workflow (complete)

---

## Sources

### Primary (HIGH confidence -- official GSD documentation)

- [GSD GitHub Repository](https://github.com/gsd-build/get-shit-done) -- README.md (fetched via WebFetch)
- [GSD docs/USER-GUIDE.md](https://github.com/gsd-build/get-shit-done/blob/main/docs/USER-GUIDE.md) -- Complete workflow stages, artifact flow, security features (fetched via WebFetch)
- [GSD docs/COMMANDS.md](https://github.com/gsd-build/get-shit-done/blob/main/docs/COMMANDS.md) -- Full command reference with syntax and flags (fetched via WebFetch)
- [GSD docs/ARCHITECTURE.md](https://github.com/gsd-build/get-shit-done/blob/main/docs/ARCHITECTURE.md) -- Multi-agent system, wave execution model, artifact dependencies (fetched via WebFetch)
- [GSD docs/FEATURES.md](https://github.com/gsd-build/get-shit-done/blob/main/docs/FEATURES.md) -- Feature set, quality assurance mechanisms, extensibility (fetched via WebFetch)

### Primary (HIGH confidence -- project artifacts, direct reading)

- `.claude/commands/case.md` -- /case skill definition with full process, output format, subagent dispatch patterns
- `.planning/research/GSD-AGENT-PATTERNS.md` -- 10 design patterns extracted from GSD source files
- `.planning/research/CASE-SKILL-SYNTHESIS.md` -- Unified case discovery methodology (5 technique layers)
- `.planning/research/CASE-AGENT-SYNTHESIS.md` -- Agent architecture for /case (2 subagents, dispatch patterns)
- `.planning/research/CASE-AGENT-NEEDS.md` -- Per-step subagent analysis, context budget
- `.planning/research/CASE-CATEGORY-COMPLETENESS.md` -- S/F/E category sufficiency audit
- `.planning/ROADMAP.md` -- Phase plan with dependencies
- `.planning/PROJECT.md` -- Architecture, domain decisions, service topology
- `.planning/config.json` -- GSD configuration (nyquist_validation, plan_check, etc.)
- `CLAUDE.md` -- Project conventions, build commands, testing strategy

---

*Research completed: 2026-03-24*
*Valid until: 2026-06-24 (GSD workflow is stable; custom skills are project-specific)*
