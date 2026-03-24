# Development Workflow

Extended GSD workflow with custom skill (/case) and additional GSD stages for maximum coverage. TDD is enforced during execute via CLAUDE.md protocol (Stub-Red-Green-Refactor per task).

## Main Pipeline

```
         ┌─────────────────────────────────────┐
         v                                     |
    gsd:discuss <──────────────┐───────────────┤
         |                     |               |
       /case <─────────────┐───┘               |
         |                 |                   |
   (gsd:ui-phase)          |                   |
         |                 |                   |
   (gsd:research) ─────────┘───────────────────┘
         |
    gsd:plan
         |
   (gsd:review) ──(reject)──> gsd:plan
         |
  (gsd:assumptions)
         |
    gsd:execute ──(stuck)──> gsd:debug    [TDD via CLAUDE.md protocol]
         |
  (gsd:ui-review)
         |
   (gsd:validate)
         |
    gsd:verify
         |
   (gsd:simplify)
         |
    gsd:ship
```

**Iterative, not linear.** Any step can return to an earlier step and redo from there. Common loops:
- plan -> discuss: discovered that a design decision was wrong or missing
- plan -> case: realized more operations or cases need discussion
- case -> discuss: case discussion surfaced gray areas that need design decisions first
- review -> plan: peer review found flaws in the plan

Steps in `(parentheses)` are optional — see each step's "When to skip" note below.

## Step Responsibilities

| Step | Produces | Answers | Key Input |
|------|----------|---------|-----------|
| **gsd:discuss** | XX-CONTEXT.md | **HOW** — technology choices, patterns, architecture decisions | ROADMAP.md, PROJECT.md, prior CONTEXT.md |
| **/case** | XX-CASES.md | **WHAT exactly** — operations, interfaces, rules, S/F/E cases, side effects, open questions | CONTEXT.md, ROADMAP.md |
| **gsd:ui-phase** | UI-SPEC.md | **UI design** — layout, components, interactions | CONTEXT.md, CASES.md |
| **gsd:research** | XX-RESEARCH.md | **What exists** — library APIs, protocol details, external constraints | CONTEXT.md, CASES.md |
| **gsd:plan** | PLAN.md | **HOW to build** — task decomposition, dependencies, execution order | CONTEXT.md, CASES.md, RESEARCH.md |
| **gsd:review** | Review comments | **Is the plan sound** — gaps, wrong dependencies, overcomplexity | PLAN.md, CASES.md |
| **gsd:assumptions** | Assumptions list | **What did we assume** — implicit dependencies, preconditions | PLAN.md |
| **gsd:execute** | Implementation code + tests | **Code** — implement tasks with TDD (Stub-Red-Green-Refactor per `tdd="true"` task, guided by CLAUDE.md protocol) | PLAN.md, CASES.md (via `<behavior>` items) |
| **gsd:debug** | Debug state | **Root cause** — systematic diagnosis with persistent state | Error context |
| **gsd:ui-review** | UI audit report | **Visual quality** — 6-pillar audit (a11y, responsiveness, etc.) | Implemented UI |
| **gsd:validate** | Validation report | **Plan fulfilled** — PLAN.md tasks vs actual implementation | PLAN.md, code |
| **gsd:verify** | UAT results | **Requirements met** — user acceptance testing against original goals | CONTEXT.md, CASES.md, code |
| **gsd:simplify** | Cleaned code + commit | **Code quality** — deduplication, efficiency, reuse opportunities | git diff, codebase |
| **gsd:ship** | PR | **Ready to merge** — PR creation, review, merge preparation | Verified code |

## When to Use Optional Steps

| Step | Use when | Skip when |
|------|----------|-----------|
| **gsd:ui-phase** | Frontend phases with user-facing UI | Backend-only phases (current scope) |
| **gsd:research** | Unfamiliar libraries, complex protocols, external service integration | Well-known tech stack, simple CRUD |
| **gsd:review** | Complex/risky phases (auth, data migration), large PLAN.md | Simple phases, tight timelines |
| **gsd:assumptions** | Many implicit dependencies, cross-service phases | Self-contained phases with clear scope |
| **gsd:debug** | Stuck during execute, same error recurring, unclear root cause | Obvious fixes, simple compile errors |
| **gsd:ui-review** | Frontend phases after implementation | Backend-only phases |
| **gsd:validate** | Large phases with many tasks, easy to miss items | Small phases where verify covers everything |
| **gsd:simplify** | Large implementation phases, significant new code | Small changes, trivial fixes |

## /case Dual Role

/case serves as BOTH functional spec and behavioral spec:

1. **Functional spec:** Identify concrete operations from abstract requirements. "Implement passkey authentication" -> {register_passkey, authenticate_with_passkey, list_passkeys, delete_passkey}. Define interfaces (inputs, outputs, auth).

2. **Behavioral spec:** For each operation, discover success/failure/edge cases through structured conversation with the developer. Surface rules, side effects, and open questions.

This dual role exists because gsd:discuss produces design decisions (HOW), not operation definitions (WHAT). /case fills the WHAT gap.

## Artifact Availability

At each step, what artifacts exist:

| Step | ROADMAP | CONTEXT | CASES | RESEARCH | PLAN | Proto/Code |
|------|---------|---------|-------|----------|------|------------|
| gsd:discuss | Y | prior phases only | - | - | - | - |
| /case | Y | Y | - | - | - | - |
| gsd:ui-phase | Y | Y | Y | - | - | - |
| gsd:research | Y | Y | Y | - | - | - |
| gsd:plan | Y | Y | Y | maybe | - | - |
| gsd:review | Y | Y | Y | maybe | Y | - |
| gsd:assumptions | Y | Y | Y | maybe | Y | - |
| gsd:execute | Y | Y | Y | Y | Y | being created (tests + impl via TDD) |
| gsd:ui-review | Y | Y | Y | Y | Y | Y |
| gsd:validate | Y | Y | Y | Y | Y | Y |
| gsd:verify | Y | Y | Y | Y | Y | Y |
| gsd:simplify | Y | Y | Y | Y | Y | Y |
| gsd:ship | Y | Y | Y | Y | Y | Y |

**Key constraint:** No implementation code, proto files, or tests exist until gsd:execute. All preceding steps must work from planning documents only.

## Artifact File Locations

All phase artifacts live in `.planning/phases/{padded_phase}-{name}/`:

| Artifact | Filename | Producer |
|----------|----------|----------|
| Context | `{padded}-CONTEXT.md` | discuss-phase |
| Cases | `{padded}-CASES.md` | /case |
| Case Briefing | `CASE-BRIEFING.md` | case-briefer (internal to /case) |
| Case Scratch | `CASE-SCRATCH.md` | /case (internal, resume support) |
| Research | `{padded}-RESEARCH.md` | research-phase / plan-phase |
| Plan | `{padded}-{N}-PLAN.md` | plan-phase |
| Validation | `{padded}-VALIDATION.md` | plan-phase (Nyquist) / validate-phase |
| UI Spec | `{padded}-UI-SPEC.md` | ui-phase |

## Iterative Return Paths

| From | To | Trigger | Effect |
|------|----|---------|--------|
| plan | case | Missing behavioral cases discovered | Re-run /case, then re-plan |
| plan | discuss | Unresolved design decisions | Re-run discuss, then /case + plan |
| case | discuss | Case discussion surfaces gray areas | Re-run discuss, then redo /case |
| review | plan | Plan has gaps, wrong deps, overcomplexity | Re-plan with review feedback |
| review | discuss | Fundamental design issue found | Re-discuss, cascade forward |
| execute | plan | Implementation reveals plan flaws | Generate fix plans or re-plan |
| verify | execute | UAT failures | Generate fix plans, re-execute |
| validate | execute | Plan tasks not fully implemented | Fix missing implementations |
| simplify | execute | Significant refactoring beyond cleanup scope | Re-execute affected tasks |

**Redo semantics:** All downstream artifacts from the return point forward are regenerated.

## CASES.md Optionality

The planner must work with or without CASES.md:

- **With CASES.md:** must-priority cases become acceptance criteria. Case IDs (S1, F3, E2) referenced in PLAN.md tasks. Open questions flag unresolved items.
- **Without CASES.md:** plans derive from CONTEXT.md + REQUIREMENTS.md alone (standard GSD).

Not every phase needs /case — simple phases, infrastructure phases, or time-constrained phases may skip it.

## Integration Notes

- **gsd:plan must read CASES.md:** The GSD planner does not know about CASES.md by default. CLAUDE.md instruction ensures it is included as required context for task decomposition.
- **gsd:execute uses CASES.md via PLAN.md:** For `tdd="true"` tasks, the executor reads `<behavior>` items (annotated with case IDs) and follows the TDD protocol in CLAUDE.md to write tests before implementation.
- **Iteration invalidates downstream:** If you redo gsd:discuss, CONTEXT.md changes may invalidate CASES.md. If you redo /case, CASES.md changes may invalidate PLAN.md. Downstream artifacts should be regenerated.

---

## Session Management

These are not pipeline steps — they support work continuity across sessions.

| Skill | When | What it does |
|-------|------|--------------|
| **gsd:pause-work** | Stopping mid-phase | Saves current progress, next steps, open issues as a context handoff document |
| **gsd:resume-work** | Starting a new session | Reads the handoff document and fully restores previous session context |
| **gsd:session-report** | End of session | Generates session report: token usage, tasks completed, remaining work |

**Typical flow:**
```
[session start] --> gsd:resume-work --> (do work) --> gsd:pause-work --> [session end]
                                                  --> gsd:session-report (optional)
```

## Milestone Lifecycle

Milestones group multiple phases. These skills manage the milestone as a whole.

| Skill | When | What it does |
|-------|------|--------------|
| **gsd:audit-milestone** | After last phase verify | Cross-phase audit: original goals vs what was actually built |
| **gsd:audit-uat** | With/after audit-milestone | Cross-phase audit of all outstanding UAT items |
| **gsd:complete-milestone** | After audits pass | Archives the milestone, prepares for next |
| **gsd:milestone-summary** | At completion | Generates comprehensive milestone summary report |

**Typical flow:**
```
[last phase shipped] --> gsd:audit-milestone --> gsd:audit-uat
                              |                      |
                              v                      v
                         (fix gaps)            (fix gaps)
                              |                      |
                              +----------+-----------+
                                         |
                                  gsd:complete-milestone
                                         |
                                  gsd:milestone-summary
```

## Idea Management

Available anytime during development. Not pipeline steps — lightweight capture tools.

| Skill | Weight | Use for | Example |
|-------|--------|---------|---------|
| **gsd:note** | Lightest | Fleeting thoughts, quick memos | "Rate limiting might be needed later" |
| **gsd:add-backlog** | Light | Deferred features (parked at 999.x) | "OAuth social login for next milestone" |
| **gsd:plant-seed** | Medium | Ideas with trigger conditions | "When catalog is done, consider full-text search" |
| **gsd:review-backlog** | Periodic | Promote backlog items to active milestone | Review at new milestone start |

**Typical flow:**
```
(anytime) --> gsd:note / gsd:add-backlog / gsd:plant-seed
                                |
[new milestone start] --> gsd:review-backlog --> promote to active phases
```
