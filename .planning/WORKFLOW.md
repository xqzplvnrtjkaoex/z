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
    gsd:execute
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

## Integration Notes

- **gsd:plan must read CASES.md:** The GSD planner does not know about CASES.md by default. CLAUDE.md instruction ensures it is included as required context for task decomposition.
- **/test-gen consumes CASES.md:** Designed as primary input — each S/F/E case maps to a test skeleton.
- **Iteration invalidates downstream:** If you redo gsd:discuss, CONTEXT.md changes may invalidate CASES.md. If you redo /case, CASES.md changes may invalidate PLAN.md. Downstream artifacts should be regenerated.
