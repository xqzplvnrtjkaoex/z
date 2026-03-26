# Spec-First Workflow Methodology Patterns

**Researched:** 2026-03-26
**Domain:** Spec-driven development, living specifications, behavioral specification lifecycle
**Confidence:** MEDIUM-HIGH (blended: methodology landscape is well-documented; applicability to this project's custom workflow requires judgment)

---

## Executive Summary

Spec-first development inverts the traditional flow: specification changes precede code changes, and the spec is the authoritative source of behavioral truth. The landscape has three levels of commitment: **spec-first** (write spec before code, discard after), **spec-anchored** (spec persists as living document, evolves with code), and **spec-as-source** (spec IS the source, code is generated). This project's CASES.md is currently spec-first -- created before implementation, consumed by planner/executor, then effectively frozen. The question is whether to elevate it to spec-anchored.

The research finds that **spec-anchored is the right target** for this project, but the current CASES.md format requires only light modifications, not a rewrite. The primary cost is not format change but **process discipline** -- ensuring spec-then-code ordering on every behavioral change. The primary risk is **spec rot** -- the moment the team starts changing code without updating specs, the entire system's value collapses. Automated drift detection (comparing spec cases against test coverage) is the critical success factor.

---

## 1. Spec-First vs Current Workflow

### Current Pipeline (Status Quo)

```
discuss -> /case -> plan -> execute -> verify -> ship
            |                  |
        CASES.md            Tests + Code
        (created)           (derived from CASES)
            |
        (effectively frozen after ship)
```

CASES.md is currently a **planning artifact**: created once, consumed by the planner to generate PLAN.md tasks with `<behavior>` items and `tdd="true"` flags, consumed by the executor to write tests, then never touched again. When a later phase needs to change behavior defined in an earlier phase's CASES.md, the earlier spec is not updated.

### Spec-First (What Changes)

```
           CASES.md (living spec)
               |
     +---------+---------+
     |         |         |
  PLAN.md   Tests    Code
  (derived)  (derived) (derived)
```

In a spec-first workflow, the sequence for ANY behavioral change becomes:

1. **Spec change first:** Update CASES.md to reflect the new/changed behavior
2. **Plan change:** If the change requires new tasks, update or create PLAN.md
3. **Code change:** Tests and implementation follow from updated spec

This applies to:
- New features in a new phase (already works -- /case produces CASES.md before plan)
- Bug fixes that change expected behavior (currently: fix code + tests. Spec-first: update CASES.md, then fix)
- Cross-phase behavioral changes (currently: only the new phase's CASES.md exists. Spec-first: update the original operation's spec too)
- Refactoring that changes observable behavior (rare, but when it happens: spec first)

### What Specifically Changes in the Pipeline

| Situation | Current Behavior | Spec-First Behavior |
|-----------|-----------------|---------------------|
| New phase with /case | /case creates CASES.md (already spec-first) | No change |
| Bug fix changes behavior | Fix code+tests, CASES.md untouched | Update CASES.md first, then fix code+tests |
| Phase B changes behavior from Phase A | Phase B's CASES.md describes its own ops; Phase A's spec stale | Phase A's spec updated (or superseded) to reflect new behavior |
| verify/validate finds behavior gap | Fix code+tests | Update CASES.md first, then fix |
| Post-ship maintenance | CASES.md is archived | CASES.md remains authoritative |

**The pipeline does NOT change structurally.** The discuss/case/plan/execute/verify flow stays identical. What changes is the **lifecycle of CASES.md** -- from "planning artifact" to "living specification" -- and the **discipline** that behavioral changes start at the spec, not the code.

### Confidence: HIGH
This distinction is well-established in literature. Martin Fowler's team (martinfowler.com/articles/exploring-gen-ai/sdd-3-tools.html) explicitly distinguishes spec-first (use and discard) from spec-anchored (persistent, evolving).

---

## 2. Spec Lifecycle

### Lifecycle States

Based on synthesis of BDD living documentation, SDD literature, and this project's workflow:

```
DRAFT -----> VALIDATED -----> AUTHORITATIVE -----> SUPERSEDED
  |              |                  |                    |
  |         /case complete     ship complete         New spec
  |         + developer OK     + tests pass          replaces
  |                                |                 this one
  |                           EVOLVING
  |                           (spec changes
  |                            trigger code)
  |
  v
ABANDONED (never completed)
```

| State | Meaning | Transition Trigger | Who Can Modify |
|-------|---------|-------------------|----------------|
| **DRAFT** | Under active discussion in /case | /case session started | /case agent + developer |
| **VALIDATED** | Developer has reviewed and approved all cases | /case session complete, scratch finalized to CASES.md | Developer (via re-running /case) |
| **AUTHORITATIVE** | Implementation matches spec, tests pass | Phase shipped, verify passed | Developer (must update spec before code) |
| **EVOLVING** | Authoritative spec being modified for new changes | Behavioral change needed | Developer (back through spec-first flow) |
| **SUPERSEDED** | Replaced by a newer version of the same operation's spec | New phase redefines the operation | Nobody (archived) |

### Key Insight: "Authoritative" vs "Frozen"

The critical difference from current workflow is the AUTHORITATIVE state. Currently, after ship, CASES.md enters an implicit "frozen/archived" state. In spec-anchored, AUTHORITATIVE means "this is the truth, and changes to truth start here."

An authoritative spec is NOT immutable. It is **the starting point for changes**. When behavior needs to change, the spec transitions to EVOLVING, is updated, then returns to AUTHORITATIVE after the change is shipped.

### When Is a Spec Authoritative?

A spec becomes authoritative when ALL of:
1. The /case discussion is complete (all operations have S/F/E cases)
2. Tests derived from the spec pass
3. The phase has been verified (gsd:verify passed)

A spec REMAINS authoritative as long as:
- No conflicting spec exists for the same operation
- The implementation still matches (no untracked behavioral drift)

### Confidence: MEDIUM-HIGH
The state model is synthesized from BDD and SDD literature, not copied from a single source. The EVOLVING state is this project's addition to handle the iterative return paths already in the workflow.

---

## 3. Spec-Code Synchronization (Spec Drift)

### The Problem

Spec drift occurs when the spec says behavior X but the code does behavior Y. This is the #1 failure mode of living specification systems. BDD teams that abandon Cucumber typically cite "stale feature files" as the primary reason.

### Detection Mechanisms

#### 3a. Structural: Spec-to-Test Traceability

CASES.md cases already map to test functions via `<behavior>` items in PLAN.md and `tdd="true"` enforcement. This creates a structural chain:

```
CASES.md case [SignupBegin.F2]
  -> PLAN.md <behavior> item [SignupBegin.F2]
    -> test function: should_reject_when_handle_empty
```

**Drift detection:** If a test is deleted, renamed, or its assertion changed such that it no longer covers the case's Expected Outcome, structural drift has occurred.

**Automation potential:** A CI check could parse CASES.md case tables, extract case IDs, verify that each must-priority case has a corresponding test function (via naming convention or annotation), and verify the test passes. This is analogous to BDD's Cucumber runner but without requiring Gherkin format.

#### 3b. Behavioral: Test Coverage vs Spec Coverage

CASES.md defines the behavioral contract. Tests implement that contract. If code changes cause a test to pass with different behavior than the spec describes, the spec has drifted.

**Detection:** This is harder to automate. It requires either:
1. **Manual review discipline** -- when changing code, review CASES.md to ensure spec still matches
2. **AI-assisted audit** -- gsd:validate already cross-references CASES.md against implementation. Expanding this to flag divergences between spec Expected Outcome and actual test assertions

#### 3c. Process: Spec-First Ordering Enforcement

The strongest drift prevention is **process discipline**: never change behavior without updating the spec first.

**Enforcement options:**
- **Honor system** -- developer discipline (lowest cost, highest drift risk)
- **Commit message convention** -- behavioral changes require spec update in same commit or preceding commit
- **CI gate** -- if a test file changes and its corresponding CASES.md hasn't been modified in the same PR, flag for review (requires mapping infrastructure)

### Cost of Maintaining Sync

| Mechanism | Setup Cost | Ongoing Cost | Drift Prevention |
|-----------|-----------|--------------|-----------------|
| Honor system (process discipline) | None | Low (mental overhead) | LOW -- degrades under pressure |
| gsd:validate integration | Low (already exists) | Low (run per phase) | MEDIUM -- catches gaps at validate time |
| Structural CI check (case ID -> test mapping) | Medium (needs parser) | Low (automated) | MEDIUM-HIGH -- catches missing/renamed tests |
| Full behavioral audit (spec Expected Outcome vs test assertions) | High (needs AI analysis) | Medium (compute per PR) | HIGH -- catches semantic drift |

### Recommendation for This Project

**Start with process discipline + enhanced gsd:validate.** The validate step already cross-references CASES.md against PLAN.md behavior items. Extending it to check that spec Expected Outcomes match test assertions is a natural evolution.

Do NOT build CI infrastructure for drift detection until spec-anchored workflow has been practiced for at least 2-3 phases and the team has calibrated whether manual discipline is sufficient.

### Confidence: HIGH
BDD failure modes are extensively documented. The structural traceability already exists in this project's workflow (case IDs in behavior items). The recommendation is conservative by design.

---

## 4. Existing Methodologies: Patterns to Borrow

### 4a. BDD (Behavior-Driven Development)

**What it does:** Structured collaboration (Three Amigos) producing Gherkin scenarios that become automated tests and living documentation.

**Patterns to borrow:**
- **Declarative over imperative specifications.** CASES.md already does this well -- cases describe behavior ("Handle already taken -> 409 conflict") not implementation ("INSERT INTO users fails with unique violation").
- **The scenario-per-rule pattern.** Each rule in CASES.md generates 1+ cases. This is Example Mapping, already adopted in /case.
- **Living documentation concept.** Specs are not write-once artifacts; they evolve with the system.

**Patterns to avoid:**
- **Gherkin format itself.** The Given/When/Then natural language format adds ceremony without benefit for AI-developer workflow. CASES.md tabular format (Preconditions | Action | Expected Outcome) carries the same information more densely.
- **Step definition layer.** BDD's glue code (step definitions mapping Gherkin to test code) is a maintenance burden. CASES.md -> PLAN.md behavior items -> test functions is a simpler chain.
- **Stakeholder collaboration theater.** BDD's Three Amigos assumes non-technical stakeholders. This project has one developer using AI agents. The /case discussion IS the Three Amigos equivalent.

**Known failure modes:**
- Teams write 20+ scenarios per feature, creating maintenance burden that collapses
- Step definitions multiply into hundreds of reusable steps harder to maintain than plain test code
- Feature files become stale when no one outside QA reads them
- Teams push BDD down to unit tests where syntax overhead outweighs communication benefit (typically abandoned within 2 quarters)

**Relevance:** HIGH. CASES.md is already a declarative behavioral spec. The failure modes are the exact risks to watch for.

### 4b. ATDD (Acceptance Test-Driven Development)

**What it does:** Collaborative definition of acceptance criteria before development. Tests derived from criteria serve as verification AND documentation.

**Patterns to borrow:**
- **Acceptance criteria as contract.** CASES.md must-priority cases ARE acceptance criteria. The current verify step already uses them for UAT. Making this explicit (spec = acceptance contract) strengthens the model.
- **Tests derived from spec, not spec derived from tests.** ATDD enforces that tests come from acceptance criteria, never the reverse. This is the same ordering as spec-first.

**Patterns to avoid:**
- **Heavyweight tooling.** ATDD tools (FitNesse, Robot Framework) add infrastructure overhead without proportional benefit for a solo-developer + AI workflow.

**Known failure modes:**
- Acceptance criteria written too abstractly to be testable
- Gap between "acceptance test" and "unit test" levels -- developers write unit tests that pass but acceptance tests fail due to integration issues

**Relevance:** MEDIUM. The principles are already embedded in the workflow. No new patterns needed.

### 4c. Design by Contract (DbC)

**What it does:** Functions specify preconditions, postconditions, and invariants. Violations are detected at runtime.

**Patterns to borrow:**
- **Precondition/postcondition thinking.** CASES.md already captures this: Preconditions column + Expected Outcome = precondition/postcondition pair. Rules section captures invariants.
- **Invariant tracking.** Phase Rules (PR) and System-Wide Rules (SR) in CASES.md are behavioral invariants that must hold across all operations. This is directly analogous to DbC class invariants.

**Patterns to avoid:**
- **Runtime assertion enforcement.** DbC's value proposition is compile-time/runtime checking. In Rust, the type system + tests already provide this. Adding a separate assertion layer would be redundant.

**Known failure modes:**
- Contracts become stale when not maintained (same as spec drift)
- Overhead of maintaining contracts for simple functions

**Relevance:** LOW for adoption. The concepts are already in CASES.md under different names.

### 4d. Spec-Driven Development (SDD, 2025 emergence)

**What it does:** Treats specifications as the authoritative source from which code is derived. Three levels: spec-first, spec-anchored, spec-as-source.

**Patterns to borrow:**
- **Spec-anchored as target level.** Code is not generated from spec (too rigid), but spec persists and governs evolution. This is exactly the target for CASES.md.
- **Spec change triggers downstream regeneration.** When a spec changes, derived artifacts (plan, tests) must be updated. This aligns with the existing iterative return paths.
- **Living spec over static spec.** Static specs require manual reconciliation. Living specs evolve and trigger downstream updates.

**Patterns to avoid:**
- **Spec-as-source.** Treating the spec as the ONLY source (code is generated and never hand-edited) requires deterministic code generation. LLMs are non-deterministic. Not viable for this project.
- **Heavy tooling (Kiro, spec-kit, Tessl).** These tools assume greenfield projects with AI-generated code. This project has an established workflow with custom skills. Adopting an external SDD tool would conflict with the GSD pipeline.

**Known failure modes:**
- "Most spec-driven development initiatives fail at workflow integration, not at technology evaluation" (Augment Code)
- Overly rigid spec formats that don't accommodate domain-specific needs
- Spec files that are hard to review because they're too lengthy
- For small changes, the spec-first overhead is "like using a sledgehammer to crack a nut" (Martin Fowler's analysis of spec-kit/Kiro)

**Relevance:** HIGH. The spec-anchored pattern is the exact model to adopt. The failure modes are warnings for implementation.

### Confidence: HIGH
These methodologies are well-documented with decades of practice (BDD, ATDD, DbC) or intensive 2025-2026 exploration (SDD).

---

## 5. CASES.md Format Fitness as Spec

### Current Format Analysis

The current CASES.md format per operation:

```markdown
## Operation: SignupBegin

### Rules
- R1: description (Decision reference)

### Side Effects
- DB: what changes
- Redis: what changes

### Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |

### Open Questions
| ID | Question | Impact | Default Recommendation |
```

With Phase Rules (PR) and System-Wide Rules (SR) at document level.

### Comparison with Established Formats

| Property | CASES.md | Gherkin/BDD | OpenAPI | ADR | DbC |
|----------|----------|-------------|---------|-----|-----|
| **Behavioral cases** | Yes (S/F/E tables) | Yes (scenarios) | No (structure only) | No (decisions only) | Partial (pre/post) |
| **Rules/invariants** | Yes (R, PR, SR) | Implicit in scenarios | No | No | Yes (invariants) |
| **Side effects** | Yes (explicit section) | No (hidden in Then) | No | No | Yes (postconditions) |
| **Interface definition** | Partial (in briefing, not in CASES.md) | No | Yes (primary purpose) | No | No |
| **Machine-parseable** | Semi (markdown tables) | Yes (Gherkin grammar) | Yes (YAML/JSON) | No | Yes (language-level) |
| **Priority/importance** | Yes (must/should/could) | No (all equal) | No | No | No |
| **Open questions** | Yes (explicit tracking) | No | No | Yes (consequences) | No |
| **Decision traceability** | Yes (D-XX references) | No | No | Yes (primary purpose) | No |
| **Cross-operation rules** | Yes (PR, SR) | No | No | No | Yes (class invariants) |

### Assessment

CASES.md is **already well-suited as a behavioral specification format**. It captures information that no single established format covers:
- Behavioral cases WITH priority (Gherkin lacks priority)
- Side effects as first-class concept (Gherkin hides them, OpenAPI ignores them)
- Cross-cutting rules at multiple levels (SR, PR, R) -- analogous to DbC invariants
- Open questions with resolution tracking (no equivalent in other formats)
- Decision traceability (D-XX references link to design rationale)

### Modifications Needed for Spec-Anchored Use

| Modification | Why | Effort |
|-------------|-----|--------|
| **Add spec state marker** | Track whether spec is DRAFT/VALIDATED/AUTHORITATIVE/SUPERSEDED | Trivial (metadata header) |
| **Add "Last Verified" date** | Know when spec was last confirmed against implementation | Trivial (metadata header) |
| **Resolve Open Questions before AUTHORITATIVE** | A spec with unresolved OQs cannot be authoritative | Process rule (already natural) |
| **Add version/changelog section** | Track how spec evolved over time (currently only CASE-SCRATCH Changes Log) | Low (append-only section) |
| **Service-level organization** (from memory: `project_spec_driven_workflow.md`) | Specs organized by service rather than phase for cross-phase access | Medium (directory restructure) |

### What NOT to Change

- **Do not adopt Gherkin syntax.** The tabular format is denser and more readable for the AI-developer workflow. Gherkin adds ceremony without benefit.
- **Do not merge CASES.md with OpenAPI.** CASES.md is behavioral; OpenAPI is structural. They are complementary, not competing.
- **Do not add machine-parseable grammar.** The semi-structured markdown tables are sufficient for AI agents to parse. A formal grammar adds tooling overhead without proportional benefit.

### Confidence: HIGH
Direct comparison with the actual CASES.md content from CASE-SCRATCH.md. The format analysis is concrete, not theoretical.

---

## 6. Downstream Invalidation

### The Cascade Problem

When a spec changes, which downstream artifacts need updating?

```
CASES.md (spec)
    |
    +-- PLAN.md (<behavior> items reference case IDs)
    |       |
    |       +-- Tests (implement behavior items)
    |       |       |
    |       |       +-- Implementation (passes tests)
    |       |
    |       +-- Task descriptions (reference operations)
    |
    +-- CONTEXT.md (decisions that informed the spec)
    |
    +-- Cross-phase references (other phases' specs may reference this operation)
```

### Invalidation Rules

| Spec Change Type | What Invalidates | Action Required |
|-----------------|------------------|-----------------|
| **New case added** (e.g., new failure case F11) | Nothing invalidated | New test + implementation needed. May need new PLAN.md task. |
| **Case Expected Outcome changed** | Test for that case | Update test assertion. May need implementation change. |
| **Case removed** (deprioritized) | Test for that case | Remove or skip test. Implementation may simplify. |
| **Rule changed** (e.g., R5: timeout 5s -> 10s) | All tests referencing that rule | Update tests + implementation. Likely small changes. |
| **Operation added** | PLAN.md (needs new tasks) | New plan tasks, tests, implementation. Full /case -> plan -> execute. |
| **Operation removed** | PLAN.md tasks, tests, implementation | Remove code. Flag in validate. |
| **Side effect changed** | Integration tests | Update test expectations. May need implementation change. |
| **Phase Rule changed** (PR) | All operations inheriting that rule | Cascade update to all affected operations' tests. |
| **System-Wide Rule changed** (SR) | All operations in all phases | PROJECT.md update + cascade to all affected specs. |

### Interaction with Iterative Return Paths

The existing workflow already handles most of this through iterative return paths:

| From | To | Already Handles |
|------|----|----------------|
| execute -> plan | Plan flaws discovered during implementation | Yes |
| verify -> execute | UAT failures | Yes |
| plan -> case | Missing behavioral cases | Yes |
| case -> discuss | Gray areas in cases | Yes |

**What spec-anchored adds:** A return path from ANY post-ship change back to the spec:

```
Post-ship change needed
    -> Update CASES.md (spec evolves)
    -> Update PLAN.md if needed (or create fix plan)
    -> Update tests + implementation
    -> Verify
```

This is just the existing return path with one additional step at the front: spec update.

### Cross-Phase Invalidation

The harder problem is when Phase B's implementation changes behavior defined in Phase A's spec.

**Current state:** Phase A's CASES.md is frozen. Phase B creates its own CASES.md. If Phase B changes Phase A behavior (e.g., adds a new claim to JWT -- affects all operations in Phase A that reference PR1), Phase A's spec is stale.

**Spec-anchored approach options:**

1. **Update original spec in place.** Phase B's /case session updates Phase A's CASES.md directly. Risk: merge conflicts, phase ownership ambiguity.

2. **Supersede with new version.** Phase B's CASES.md explicitly states "this supersedes Phase A's [Operation] spec." Original marked SUPERSEDED. Risk: fragmented specs, hard to find current truth.

3. **Service-level spec consolidation.** Specs organized by service (`specs/auth/`, `specs/user/`), not by phase. Each service has one authoritative spec file. Phases update the service-level spec. Risk: larger scope, needs careful organization.

**Recommendation:** Option 3 is the most sustainable for long-lived projects, but Option 2 is lower-risk to start with. Begin with Option 2 (supersession) during the first spec-anchored phases, then evaluate consolidation to service-level specs after completing Milestone 1.

### Confidence: MEDIUM-HIGH
The invalidation rules are derived from the actual workflow and artifact structure. Cross-phase invalidation is the least well-defined area.

---

## Key Risks and Failure Modes

### Risk 1: Spec Rot (HIGH probability without enforcement)

**What:** Developers start changing code without updating specs. Within 2-3 phases, specs are unreliable. Within a milestone, they're abandoned.

**Why it happens:** Under time pressure, the "update spec first" step feels like overhead. The code and tests already capture the behavior. Why maintain a redundant document?

**Mitigation:**
- gsd:validate must cross-reference CASES.md against implementation (already partially does)
- Keep the spec update lightweight -- a single line change in a table, not a multi-paragraph rewrite
- The /case format is already lightweight; this helps

**Historical evidence:** BDD teams that abandon Cucumber cite stale feature files as the #1 reason. Teams that write 20+ scenarios per feature collapse under maintenance burden. The sweet spot is 3-7 scenarios per operation, which CASES.md already targets.

### Risk 2: Overhead for Small Changes (MEDIUM probability)

**What:** For a one-line bug fix, requiring a spec update feels disproportionate. The "sledgehammer to crack a nut" problem.

**Why it happens:** Not all changes are behavioral. Config changes, performance fixes, refactoring that preserves behavior -- none of these need spec updates.

**Mitigation:**
- Define clearly what triggers a spec update: **only changes to observable behavior** (inputs, outputs, side effects, error responses). Internal refactoring does NOT trigger spec update.
- The trigger question: "Does this change any row in a CASES.md table?" If no, no spec update needed.

### Risk 3: Cross-Phase Spec Ownership (MEDIUM probability)

**What:** When Phase B changes behavior from Phase A, who owns Phase A's spec? Can Phase B modify it? What if Phase C also needs to modify it?

**Why it happens:** Phase-based organization creates temporal ownership. Once a phase ships, there's no natural owner for its artifacts.

**Mitigation:**
- Service-level spec organization solves this (see Section 6, Option 3)
- In the interim, the rule is: the LATEST phase that modifies an operation owns that operation's spec

### Risk 4: Spec Scope Creep (LOW probability but high impact)

**What:** The spec grows to cover implementation details, internal data structures, database schemas -- things that are NOT behavioral contracts but internal concerns.

**Why it happens:** Completeness instinct. If the spec is the source of truth, shouldn't it cover everything?

**Mitigation:**
- Strict scope rule: CASES.md covers **observable behavior only** (what a caller sees). Internal implementation details live in code/comments.
- The existing CASES.md format naturally enforces this -- the table columns (Preconditions, Action, Expected Outcome) describe external behavior.

### Risk 5: AI Agent Cognitive Load (LOW probability)

**What:** Requiring AI agents to read, understand, and update specs before every code change adds to context window usage and processing time.

**Why it happens:** Living specs are another artifact to maintain in the AI's working context.

**Mitigation:**
- CASES.md is already in the agent's required reading for execute phase. The additional load is UPDATING, not reading.
- Service-level INDEX.md (from the spec folder idea) reduces search cost.

---

## Open Questions That Emerged

### Q1: Service-Level vs Phase-Level Spec Organization

The memory file `project_spec_driven_workflow.md` proposes reorganizing completed phase specs by service in `.planning/specs/`. This is a structural question that affects:
- How the briefer discovers existing behavioral contracts
- How cross-phase spec updates work
- How developers navigate to find "current truth" for an operation

**Recommendation:** Research this as a separate question. The spec-first methodology decision is independent of the organizational structure. You can be spec-anchored with either phase-level or service-level organization.

### Q2: Granularity of Spec Changes

When a spec changes, what constitutes a "change" that triggers downstream updates?
- Adding a new case? (Yes -- new test needed)
- Changing priority from "could" to "must"? (Maybe -- promotes to acceptance criterion)
- Rewording a rule for clarity without changing meaning? (No -- editorial only)
- Adding a decision reference (D-XX) to existing rule? (No -- traceability, not behavior)

**Recommendation:** Define a lightweight change classification (behavioral change vs editorial change) and only require downstream updates for behavioral changes.

### Q3: Retroactive Spec Creation for Phases 1-2

Phases 1 and 2 completed without CASES.md. If spec-anchored is adopted, should specs be retroactively created?

**Recommendation:** Not immediately. Create specs for Phase 1-2 operations only when a later phase needs to reference or modify their behavior. This is demand-driven, not completeness-driven.

### Q4: Spec vs Test as Source of Truth During Execution

During gsd:execute, the TDD protocol derives tests from PLAN.md `<behavior>` items, which derive from CASES.md. If a test reveals a spec error (e.g., "the spec says 400 but it should be 409"), what's the correction flow?

**Recommendation:** The correction flow should be: pause execution -> update CASES.md -> update PLAN.md behavior item -> continue TDD. This maintains spec-first ordering even during implementation. The alternative (fix the test, update spec later) creates drift.

### Q5: Spec Versioning Strategy

How to version specs? Options:
- Git history (spec changes are commits, use git blame)
- Inline changelog (append-only section in CASES.md)
- Separate version file per spec

**Recommendation:** Git history + lightweight inline changelog (already present as "Changes Log" in CASE-SCRATCH.md). No separate version files.

---

## Summary of Recommendations

| Decision | Recommendation | Confidence |
|----------|---------------|------------|
| Adopt spec-first? | Yes, at spec-anchored level | HIGH |
| Change CASES.md format? | Minimal: add state marker, last-verified date, version log | HIGH |
| Adopt Gherkin/BDD tooling? | No -- CASES.md tabular format is superior for this workflow | HIGH |
| Drift detection mechanism? | Start with process discipline + enhanced gsd:validate; defer CI automation | HIGH |
| Cross-phase spec management? | Start with supersession (Option 2), evaluate service-level consolidation later | MEDIUM |
| Retroactive specs for Phase 1-2? | Demand-driven only, not upfront | HIGH |
| Spec change trigger? | Only observable behavior changes require spec update | HIGH |
| Small change overhead? | Define behavioral vs editorial change classification | MEDIUM |

---

## Sources

### Primary (HIGH confidence)
- Martin Fowler's team analysis of SDD patterns: [martinfowler.com/articles/exploring-gen-ai/sdd-3-tools.html](https://martinfowler.com/articles/exploring-gen-ai/sdd-3-tools.html) - spec-first / spec-anchored / spec-as-source taxonomy
- Thoughtworks Technology Radar: [thoughtworks.com/.../spec-driven-development](https://www.thoughtworks.com/en-us/insights/blog/agile-engineering-practices/spec-driven-development-unpacking-2025-new-engineering-practices) - SDD as emerging 2025 practice, failure modes
- Cucumber BDD documentation: [cucumber.io/docs/bdd/](https://cucumber.io/docs/bdd/) - living documentation principles, scenario best practices
- Agile Alliance ATDD glossary: [agilealliance.org/glossary/atdd/](https://agilealliance.org/glossary/atdd/) - acceptance test driven development principles

### Secondary (MEDIUM confidence)
- InfoQ SDD analysis: [infoq.com/articles/spec-driven-development/](https://www.infoq.com/articles/spec-driven-development/) - enforcement layers, drift detection patterns
- Augment Code SDD guide: [augmentcode.com/guides/what-is-spec-driven-development](https://www.augmentcode.com/guides/what-is-spec-driven-development) - workflow integration as primary failure point
- arXiv paper 2602.00180: [arxiv.org/abs/2602.00180](https://arxiv.org/abs/2602.00180) - academic analysis of SDD, case study results (75% cycle time reduction for API changes)
- GitHub spec-kit: [github.com/github/spec-kit](https://github.com/github/spec-kit) - practical SDD tooling patterns, validation approach

### Tertiary (LOW confidence)
- Various Medium articles on BDD failure modes and team adoption experiences
- Web search results on DbC modern usage (primarily Wikipedia and Eiffel documentation)

### Project-Internal Sources
- `.planning/WORKFLOW.md` - current pipeline structure, iterative return paths
- `.planning/phases/03a-authentication-core/CASE-SCRATCH.md` - actual CASES.md format and content
- `.planning/phases/03a-authentication-core/CASE-BRIEFING.md` - briefer output format
- `.planning/research/BDD-CASE-DISCOVERY.md` - prior BDD research for /case skill
- `.planning/research/CASE-SKILL-SYNTHESIS.md` - unified /case framework
- `project_spec_driven_workflow.md` (memory) - original idea context
- `project_unified_spec_idea.md` (memory) - consolidated spec idea
