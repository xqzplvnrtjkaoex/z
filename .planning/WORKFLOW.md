# Development Workflow

Extended GSD workflow with custom skills (/case, /test-gen) inserted into the official GSD pipeline.

## Pipeline

```
         ┌──────────────────────┐
         v                      |
    gsd:discuss <───────────────┤
         |                      |
       /case <──────────┐       |
         |              |       |
    (gsd:ui) <- optional|       |
         |              |       |
    gsd:plan ───────────┘───────┘
         |
     /test-gen
         |
    gsd:execute --> gsd:verify --> gsd:ship
```

**Iterative, not linear.** Any step can return to an earlier step and redo from there. Common loops:
- plan → discuss: discovered that a design decision was wrong or missing
- plan → case: realized more operations or cases need discussion
- case → discuss: case discussion surfaced gray areas that need design decisions first

## Step Responsibilities

| Step | Produces | Answers | Key Input |
|------|----------|---------|-----------|
| **gsd:discuss** | XX-CONTEXT.md | **HOW** — technology choices, patterns, architecture decisions | ROADMAP.md, PROJECT.md, prior CONTEXT.md |
| **/case** | XX-CASES.md | **WHAT exactly** — operations, interfaces, rules, S/F/E cases, side effects, open questions | CONTEXT.md, ROADMAP.md |
| **gsd:ui** | UI-SPEC.md | **UI design** — layout, components, interactions (frontend phases only, otherwise skipped) | CONTEXT.md, CASES.md |
| **gsd:plan** | PLAN.md | **HOW to build** — task decomposition, dependencies, execution order | CONTEXT.md, CASES.md, RESEARCH.md |
| **/test-gen** | Test skeletons | **Failing tests** — compilation-error test files from CASES.md (the "red" in TDD) | CASES.md |
| **gsd:execute** | Implementation code | **Code** — make tests pass, implement all planned tasks | PLAN.md, test skeletons |

## /case Dual Role

/case serves as BOTH functional spec and behavioral spec:

1. **Functional spec:** Identify concrete operations from abstract requirements. "Implement passkey authentication" → {register_passkey, authenticate_with_passkey, list_passkeys, delete_passkey}. Define interfaces (inputs, outputs, auth).

2. **Behavioral spec:** For each operation, discover success/failure/edge cases through structured conversation with the developer. Surface rules, side effects, and open questions.

This dual role exists because gsd:discuss produces design decisions (HOW), not operation definitions (WHAT). /case fills the WHAT gap.

## Artifact Availability

At each step, what artifacts exist:

| Step | ROADMAP | CONTEXT | CASES | RESEARCH | PLAN | Proto/Code |
|------|---------|---------|-------|----------|------|------------|
| gsd:discuss | Y | prior phases only | - | - | - | - |
| /case | Y | Y | - | - | - | - |
| gsd:ui | Y | Y | Y | - | - | - |
| gsd:plan | Y | Y | Y | maybe | - | - |
| /test-gen | Y | Y | Y | Y | Y | - |
| gsd:execute | Y | Y | Y | Y | Y | being created |

**Key constraint:** No implementation code, proto files, or tests exist until gsd:execute. All preceding steps must work from planning documents only.

## Artifact File Locations

All phase artifacts live in `.planning/phases/{padded_phase}-{name}/`:

| Artifact | Filename | Producer |
|----------|----------|----------|
| Context | `{padded}-CONTEXT.md` | discuss-phase |
| Cases | `{padded}-CASES.md` | /case |
| Case Briefing | `CASE-BRIEFING.md` | case-briefer (internal to /case) |
| Case Scratch | `CASE-SCRATCH.md` | /case (internal, resume support) |
| Research | `{padded}-RESEARCH.md` | plan-phase |
| Plan | `{padded}-{N}-PLAN.md` | plan-phase |
| Validation | `{padded}-VALIDATION.md` | plan-phase (Nyquist) |
| UI Spec | `{padded}-UI-SPEC.md` | ui-phase |

## Iterative Return Paths

| From | To | Trigger | Effect |
|------|----|---------|--------|
| plan | case | Missing behavioral cases discovered | Re-run /case, then re-plan |
| plan | discuss | Unresolved design decisions | Re-run discuss, then /case + plan |
| case | discuss | Case discussion surfaces gray areas | Re-run discuss, then redo /case |
| execute | plan | Implementation reveals plan flaws | Generate fix plans or re-plan |
| verify | execute | UAT failures | Generate fix plans, re-execute |

**Redo semantics:** All downstream artifacts from the return point forward are regenerated.

## CASES.md Optionality

The planner must work with or without CASES.md:

- **With CASES.md:** must-priority cases become acceptance criteria. Case IDs (S1, F3, E2) referenced in PLAN.md tasks. Open questions flag unresolved items.
- **Without CASES.md:** plans derive from CONTEXT.md + REQUIREMENTS.md alone (standard GSD).

Not every phase needs /case — simple phases, infrastructure phases, or time-constrained phases may skip it.

## Integration Notes

- **gsd:plan must read CASES.md:** The GSD planner does not know about CASES.md by default. CLAUDE.md instruction ensures it is included as required context for task decomposition.
- **/test-gen consumes CASES.md:** Designed as primary input — each S/F/E case maps to a test skeleton.
- **Iteration invalidates downstream:** If you redo gsd:discuss, CONTEXT.md changes may invalidate CASES.md. If you redo /case, CASES.md changes may invalidate PLAN.md. Downstream artifacts should be regenerated.
