# /case Skill

Behavioral case discovery through structured developer conversation. Runs before plan-phase to surface success, failure, and edge cases for each operation in a phase.

## Why It Exists

`gsd:discuss` produces design decisions (HOW — technology choices, architecture patterns), but not operation-level behavioral specifications (WHAT — inputs, outputs, error conditions, side effects). /case fills that gap. It identifies concrete operations from abstract requirements, then discovers their behavioral cases through conversation.

Without /case, the planner works from CONTEXT.md alone and behavioral coverage depends on whoever writes the plan remembering edge cases. With /case, the planner gets explicit S/F/E case tables and maps must-priority cases directly to test acceptance criteria.

## Components

The skill is a 3-agent system:

```
case-briefer (sonnet)        /case orchestrator (opus)        case-validator (sonnet)
    extracts operations  ──→    drives conversation with    ←──  cross-checks cases
    from planning docs          the developer, depth-first       against planning artifacts
         │                              │                              │
         ▼                              ▼                              ▼
    CASE-BRIEFING.md              CASE-SCRATCH.md                structured findings
                                        │                       (returned inline)
                                        ▼
                                  XX-CASES.md (final)
```

| File | Role |
|------|------|
| `.claude/commands/case.md` | Orchestrator — the main skill that runs the conversation |
| `.claude/agents/case-briefer.md` | Subagent — reads CONTEXT.md/ROADMAP.md, extracts operations |
| `.claude/agents/case-validator.md` | Subagent — finds requirement gaps, decision gaps, consistency issues |

## Flow

1. **Init** — Load phase context via gsd-tools. Dispatch case-briefer to extract operations from planning documents. Check for existing CASES.md or CASE-SCRATCH.md (resume support).

2. **Select** — Present discovered operations grouped by category. Developer picks which to discuss.

3. **Discuss** — Depth-first per operation: anchor shared understanding, propose success cases, then systematically probe failures (input validation, auth, resource state, boundaries, concurrency, side effects, infrastructure). Each operation review is presented as an ASCII flow diagram showing decision paths with S/F/E cases at their logical positions. Each completed operation is saved to CASE-SCRATCH.md (table format) to survive context compression.

4. **Cross-Operation** — Check consistency across operations: error formats, auth patterns, event emission, cascade behavior.

5. **Validate** — Dispatch case-validator to cross-check discovered cases against CONTEXT.md decisions, ROADMAP.md requirements, and the briefing. Developer reviews findings and incorporates confirmed gaps.

6. **Write** — Produce `{padded_phase}-CASES.md` with structured case tables, priority levels, and open questions.

## Artifacts

All artifacts live in `.planning/phases/{padded_phase}-{name}/`:

| Artifact | Lifetime | Description |
|----------|----------|-------------|
| `CASE-BRIEFING.md` | Internal | Operation extraction from planning docs. Read by orchestrator, not edited manually. |
| `CASE-SCRATCH.md` | Internal | Per-operation case data saved during discussion. Enables resume after interruption. |
| `{padded}-CASES.md` | Downstream | Final output consumed by plan-phase and test-gen. |

## Design Principles

**Technology-neutral.** Cases describe observable behavior ("validation error", "success"), not protocol specifics ("400 Bad Request", "201 Created"). The same CASES.md works whether the operation is exposed via REST, gRPC, CLI, or event handler.

**WHAT, not HOW.** Cases specify what the caller should observe, never implementation details. "What error does the caller see?" is in scope. "Should we use middleware for this?" is not.

**Propose, don't interrogate.** The orchestrator batches related probes and proposes expected cases for confirmation rather than asking twenty individual questions.

**Questions are first-class output.** When the developer says "I don't know," the answer is captured as an open question — never silently embedded as a guess. Open questions block readiness in the summary table.

## Pipeline Position

```
gsd:discuss → /case → (gsd:ui) → gsd:plan → /test-gen → gsd:execute
```

/case reads CONTEXT.md and ROADMAP.md. No implementation code, proto files, or tests exist at this point. The planner reads CASES.md downstream and maps must-priority cases to task acceptance criteria using `OperationName.S1` format.

Not every phase needs /case. Infrastructure phases, simple refactors, or time-constrained phases can skip it — the planner works from CONTEXT.md alone in that case.

## Usage

```
/case 03          # Run case discovery for phase 03
/case 12          # Run case discovery for phase 12
```

The phase number maps to a directory in `.planning/phases/`. The skill handles resume automatically if a previous session was interrupted.
