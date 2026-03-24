# TDD Executor Injection: Prescriptive Design Document

**Project:** madome
**Domain:** Replacing /test-gen with TDD behavior injected into GSD executor
**Synthesized:** 2026-03-24
**Sources:** R1 (Skill Analysis), R2 (Execute Workflow), R3 (Cases Mapping), R4 (AI TDD Patterns)
**Confidence:** HIGH (grounded in codebase analysis, established patterns, and multi-source AI TDD research)

---

## Executive Summary

We are replacing `/test-gen` (a standalone command that generates all test skeletons upfront before execution) with TDD discipline injected directly into the GSD executor's per-task workflow. The motivation is that batch skeleton generation front-loads decisions that are better made during implementation: the executor has richer context about types, stubs, and dependencies when it is actively working a task than /test-gen has when it runs before any code exists. Tight red-green-refactor feedback loops within each task produce higher-quality tests than pre-generated skeletons that may not compile or may test the wrong abstraction level.

The injection mechanism is a TDD protocol section added to CLAUDE.md (primary) with structured `<action>` steps in PLAN.md tasks (secondary). CLAUDE.md is read by every GSD executor before task execution and is the single most reliable, maintenance-free injection point [R2]. The protocol follows a **Stub-then-Red-then-Green** cycle operating at case-group granularity (all S/F/E cases for one operation), which balances TDD discipline against Rust compilation costs [R4]. CASES.md remains as the behavioral specification -- its case IDs flow through PLAN.md `<behavior>` items to guide which tests the executor writes, but the executor decides test implementation details at execution time rather than /test-gen deciding them beforehand.

The key risks are: (1) the executor ignoring TDD ordering because its built-in methodology may not enforce it beyond following `<action>` steps -- mitigated by dual injection through CLAUDE.md rules and explicit RED/GREEN action steps in PLAN.md [R2]; (2) AI agents naturally writing tests-after-code or writing trivially passing tests -- mitigated by explicit guardrails and prohibited behaviors in the protocol [R4]; and (3) Rust's two-stage Red phase (compile-error then assertion-failure) confusing the executor -- mitigated by a mandatory Stub phase that eliminates compilation barriers before test writing begins [R1, R4].

---

## Injection Strategy

### WHERE: CLAUDE.md (Primary) + PLAN.md Action Field (Secondary)

**CLAUDE.md** is the primary injection point. Every GSD executor reads `./CLAUDE.md` before executing any task. It is project-scoped, version-controlled, survives GSD plugin updates, and requires zero modification to GSD internals [R2 Section 4.2].

**PLAN.md `<action>` field** is the secondary injection point. For tasks with `tdd="true"`, the planner structures action steps as explicit RED/GREEN/REFACTOR phases with specific `cargo test` commands. The executor follows action steps precisely -- even an executor unaware of TDD will follow the right order because the action prescribes it [R2 Section 4.3].

### WHY These Injection Points

R2 analyzed six potential injection points and eliminated three (D, E, F) because they require modifying GSD plugin files, affect all projects, and break on GSD updates. Of the remaining three project-scoped options:

| Point | Mechanism | Chosen? | Rationale |
|-------|-----------|---------|-----------|
| A: CLAUDE.md | Instructional rules | **Primary** | Zero risk, zero maintenance, immediate, read by all agents |
| B: PLAN.md actions | Structural step ordering | **Secondary** | Ensures TDD ordering even if executor ignores CLAUDE.md rules |
| C: Local executor override | Full agent replacement | **Fallback only** | High maintenance burden, breaks on GSD updates, requires GSD source |

Point C (`.claude/agents/gsd-executor.md` override) is held in reserve. If CLAUDE.md + PLAN.md action structuring prove insufficient after testing, a project-local executor override can be created. Do not implement it preemptively [R2 Section 6.5].

### Changes to CLAUDE.md

Add a new `## TDD Protocol` section to CLAUDE.md (details in the "TDD Protocol for Executor" section below). This section:

1. Defines the Stub-Red-Green-Refactor cycle for `tdd="true"` tasks
2. Lists prohibited behaviors (writing implementation before tests, skipping Red verification)
3. Specifies Rust-specific patterns (two-stage Red, stub creation rules, test placement)
4. References CASES.md case IDs for coverage verification

Estimated addition: 40-60 lines of concise, operational rules.

### Changes to WORKFLOW.md

Remove `/test-gen` from the pipeline diagram and step table. Replace with a note that TDD is enforced during `gsd:execute` via CLAUDE.md protocol. CASES.md remains in the pipeline as the behavioral specification consumed by the planner.

Updated pipeline (relevant section):

```
  (gsd:assumptions)
         |
    gsd:execute ──(stuck)──> gsd:debug    [TDD enforced via CLAUDE.md protocol]
         |
```

CASES.md continues to serve its dual role as functional spec and behavioral spec [WORKFLOW.md]. The only change is that test skeletons are created by the executor during task execution, not by a separate /test-gen step beforehand.

---

## TDD Protocol for Executor

This is the step-by-step protocol an executor follows for each task with `tdd="true"`. It is the core deliverable of this document.

### Pre-Task: Identify Test Requirements

```
1. Read the task's <behavior> section. Each item is one test to write.

2. If behavior items have case ID annotations (e.g., [CreateUser.F1]),
   note them for post-task coverage verification.

3. Determine test layer from task context:
   - Files in domain/ or usecase/  -> unit tests (#[cfg(test)] mod tests, inline)
   - Files in adapter/             -> integration tests (tests/ directory)
   - Files in app/rpc/             -> covered by service tests (separate task)
   - Files in tests/               -> service or E2E tests (tests/ directory)

4. Check if required domain types and port traits already exist:
   - If YES: proceed to Red phase.
   - If NO: proceed to Stub phase.
```

### Stub Phase: Create Minimal Type/Trait Stubs

This phase exists because Rust tests cannot reference types that do not exist yet. In dynamic languages, you skip this. In Rust, it is mandatory when the domain layer is being created in the same task as the tests [R1, R4].

```
1. Create struct definitions with ONLY the fields that test setup
   and assertions will reference. Use todo!() for method bodies.

2. Create trait definitions with method signatures only.
   mockall + trait_variant will generate MockXxx automatically.

3. Create error enum variants that tests will assert against.
   Only variants referenced in <behavior> items.

4. Verify stubs compile: cargo check -p {crate}

5. Do NOT implement any business logic in stubs.
   Struct methods: todo!() or unimplemented!()
   Usecase functions: todo!() or unimplemented!()
```

**When to skip Stub phase:** When a prior task in the same plan (or a prior wave) already created the domain types and ports. The executor checks whether `use crate::domain::...` resolves before deciding [R4 Section 3.1].

### Red Phase: Write Failing Tests

```
1. Write ALL test functions for the current task's <behavior> items.
   One test function per behavior item. Order: ZOMBIES
   (simplest success -> degenerate failures -> richer cases -> boundaries).

2. Test naming: should_{behavior}_when_{condition} or
   should_{behavior}_with_{input_description}
   Use the project's established should_* convention.
   Do NOT include case IDs in test names. [R3 Section 2]

3. Test assertions: assert on observable outputs.
   - Return values: assert_eq!, assert!(matches!(...))
   - Error variants: assert!(matches!(result, Err(XxxError::Variant)))
   - Do NOT use assert!(result.is_ok()) alone -- unwrap and verify fields.
   - Use returning() for mock behavior, not times() for call counting
     (unless call-count is the behavior under test). [R4 Section 2.1]

4. Run tests to verify Red state:
   cargo test -p {crate} -- {test_name_prefix}

5. Verify output: ALL tests must FAIL with assertion failures (not
   compile errors). If compile errors occur, return to Stub phase.
   If any test passes unexpectedly, the test is not testing behavior --
   rewrite it with a meaningful assertion.

6. Do NOT write any implementation code during this phase.
```

### Green Phase: Implement to Pass Tests

```
1. Implement the minimum code required to make ALL failing tests pass.
   - ONLY logic paths exercised by tests.
   - Do NOT add error handling for untested error cases.
   - Do NOT add match arms for untested variants.
   - Do NOT add struct fields not referenced by tests.

2. Run tests to verify Green state:
   cargo test -p {crate} -- {test_name_prefix}

3. If any test fails:
   - Read the failure message.
   - Fix the IMPLEMENTATION (not the test), unless the test has a
     genuine bug (wrong expected value, wrong setup).
   - Re-run until all pass.

4. Do NOT proceed to the next case group or task until Green.
```

### Refactor Phase: Clean Up

```
1. Run cargo clippy -p {crate} and fix all warnings.

2. Check for:
   - Duplicated test setup -> extract into helper function
   - Duplicated implementation logic -> extract into function
   - Naming that drifted during implementation -> rename
   - Unnecessary complexity from Green phase -> simplify

3. Verify refactoring preserved correctness:
   cargo test -p {crate}  (full crate, not just current tests)

4. Refactoring scope: ONLY code paths with test coverage.
   Do NOT refactor untested code during this phase.
```

### Verification: Post-Task Checks

```
1. cargo test -p {crate}           -- full crate green
2. cargo clippy -p {crate}         -- zero warnings
3. cargo +nightly fmt --all        -- consistent formatting

4. If CASES.md case IDs are annotated in <behavior> items:
   verify every must-priority case has a corresponding test function.
   List any gaps as blocking issues.

5. Verify no dead code: every code path in the implementation should
   be exercised by at least one test. Unused match arms, unreachable
   branches, and unused functions indicate over-implementation.
```

### Commit: Atomic Test + Implementation

```
Commit the test AND implementation together as one atomic commit.
Do NOT commit failing tests separately (that creates a commit where
cargo test fails). [R2 Section 6.1]

Conventional commit message:
  feat({scope}): implement {operation} with TDD
  - {N} tests covering S/F/E cases
```

---

## CASES.md to Test Mapping Rules

### How Case IDs Map to Test Functions

The linkage chain is [R3 Section 2]:

```
CASES.md case ID  -->  PLAN.md <behavior> item  -->  test function name
   CreateUser.F1       "Test: ...Owner... [CreateUser.F1]"    should_reject_owner_role_...
```

The `<behavior>` item is the bridge. It carries both the human-readable description and the case ID annotation. The executor reads behavior items to know what tests to write.

**Rule:** 1 case = 1 test function (with exceptions for parameterized groups where multiple inputs produce the same behavioral outcome) [R3 Section 2].

### Test Naming Conventions

- **Use:** `should_{behavior}_when_{condition}` or `should_{behavior}_with_{input_description}`
- **Do NOT** include case IDs in test names. Case IDs are planning artifacts; test names describe behavior. Traceability lives in PLAN.md, not in code [R3 Section 2].
- **Do NOT** use `test_` prefix (Rust convention -- `#[test]` already marks it as a test).

### Test Layer Selection

The executor infers the correct test layer from the task's `<files>` list and context [R3 Section 4]:

| Task Targets | Test Layer | Location | Mock Strategy |
|-------------|-----------|----------|---------------|
| `domain/types/`, `domain/input/` | Unit | `#[cfg(test)] mod tests` (same file) | None needed (pure logic) |
| `usecase/` | Unit | `#[cfg(test)] mod tests` (same file) | Mock ports via `MockXxxRepository` |
| `adapter/postgres/`, `adapter/redis/` | Integration | `tests/` directory | testcontainers (real DB) |
| `app/rpc/` | Service | `tests/` directory | testcontainers + in-process gRPC |
| Gateway routes | E2E | `tests/` directory | testcontainers + HTTP client |

**CASES.md does NOT specify the test layer.** CASES.md is technology-neutral. The layer decision belongs to PLAN.md and the executor [R3 Section 4].

### Handling Tasks Without Case References

| Task Type | Has Cases? | Executor Action |
|-----------|-----------|-----------------|
| Proto definition, workspace setup | No | Verify with `cargo build`. No tests needed. |
| Domain types with logic | Yes | Unit tests for methods with behavior (validation, `can_manage()`, etc.) |
| Usecase functions | Yes | Full TDD cycle. This is the primary TDD target. |
| Adapter implementations | Yes | Integration tests with testcontainers. |
| gRPC handlers (app/ layer) | No | Thin translation layer. Covered by service tests. |
| Service wiring | Yes | Service-level tests with in-process gRPC. |
| Infrastructure (docker, justfile) | No | Manual or script verification. |

**The executor's rule:** If the task has `<behavior>` items, write tests. If it has `tdd="true"`, write tests FIRST. If it has neither, verify with `cargo build`/`cargo check` [R3 Section 3].

### ZOMBIES Ordering Within a Task

When a task has multiple behavior items, write tests in ZOMBIES order, not case-ID numerical order [R3 Section 6]:

1. **Z**ero/simplest success (S1 typically)
2. **O**ne meaningful success case
3. Degenerate failures (F cases with simplest preconditions)
4. Richer failures (F cases requiring more setup)
5. **B**oundary and edge cases (E cases)
6. **M**any/complex scenarios (if applicable)

ZOMBIES is a writing-order heuristic, not a coverage checklist. If CASES.md was thorough, the behavior items already cover all dimensions. The executor should trust the case list [R3 Section 6].

---

## Guardrails and Prohibited Behaviors

### Prohibited Behaviors

These are concrete things the executor MUST NOT do. Each is derived from documented AI agent failure modes [R4 Section 1.2] and adapted for this project's Rust context.

| # | Prohibited Behavior | Why | Enforcement |
|---|---------------------|-----|-------------|
| G1 | Writing implementation before tests (for `tdd="true"` tasks) | Defeats TDD -- tests become after-the-fact confirmation of whatever was written [R4] | Red phase verification: tests must fail before Green phase begins |
| G2 | Writing test and implementation in the same edit | Skips Red verification entirely [R4] | Explicit phase separation in protocol: write tests, run them, see failure, THEN implement |
| G3 | Tests that assert only `is_ok()` or `is_err()` without inspecting the value | Weak behavioral guarantee -- passes with any Ok/Err regardless of content [R4 Section 7, Pitfall 5] | After `is_ok()`, unwrap and verify key fields. After `is_err()`, verify specific error variant. |
| G4 | Testing mock configuration instead of behavior | Tests verify call patterns rather than outputs [R1, R4 Section 2.1] | Use `returning()` for setup, `assert!()` for verification. Reserve `times()`/`withf()` for when interaction is the specified behavior. |
| G5 | Over-implementation beyond test scope | Adding error branches, optional features, utility functions not exercised by tests [R4 Section 5] | Post-task dead code check: every code path must be exercised by at least one test |
| G6 | Skipping Refactor phase | Code quality degrades; duplication accumulates [R1, R4] | Explicit `cargo clippy` step after Green. Refactor checklist in protocol. |
| G7 | Batch-generating all tests across multiple operations at once | Loses iterative design benefit; scope explosion [R4] | Case-group granularity: write tests for one operation, implement, then next operation |
| G8 | Adding case IDs to test function names | Couples test names to planning artifact numbering [R3] | Test names describe behavior: `should_*` convention only |
| G9 | Refactoring untested code paths | Introduces bugs in code without test coverage [R4 Section 7, Pitfall 4] | Refactoring scope limited to code with tests. Add tests first if refactoring touches untested code. |

### Recovery Procedures

| Violation | Recovery |
|-----------|----------|
| Implementation written before tests | Revert the implementation (do not keep as reference). Write tests from scratch. Then re-implement. [R1 Section 1 -- adapted from "Delete and start over"] |
| Test passes unexpectedly in Red phase | The test is not testing behavior. Rewrite the assertion to be meaningful. Do NOT proceed to Green with a vacuously passing test. |
| Compile errors during Red phase | Return to Stub phase. Create/fix the missing type/trait. Verify stubs compile. Then return to Red. |
| Green phase fails after 3 attempts | Apply executor deviation rules (document as Deferred Issue, continue to next task) [R2 Section 2.3]. This is not a TDD failure -- it is a task-level issue. |

---

## Rust-Specific Patterns

### Two-Stage Red Phase

In Rust, "write a failing test" often means "write a test that doesn't compile" because the function under test does not exist yet. The existing TDD skill says "Test fails (not errors)" but in Rust a compilation error IS the expected state when the SUT does not exist [R1 Section 1, Gap 2].

**Resolution: Mandatory Stub phase before Red phase.**

The Stub-then-Red-then-Green cycle eliminates the two-stage ambiguity. After stubs exist, tests compile. Red means assertion failure, not compile error. This matches the project's 4-layer architecture where domain types/ports are defined before usecases [R4 Section 3.1].

| Stage | State | Agent Action |
|-------|-------|-------------|
| Pre-Stub | Test references non-existent types | Create minimal stubs (struct + fields, trait + signatures, error + variants) |
| Post-Stub | Test compiles, fails on assertion | This is the real Red. Proceed to Green. |
| Green | Test passes | Implementation complete for this case group |

### Stub Creation Rules

```
Acceptable stubs:
  - Struct: pub struct Foo { pub field: Type } (only fields tests reference)
  - Trait method: async fn save(&self, entity: &Foo) -> Result<(), Error>;
    (signature only -- mockall generates the mock)
  - Error enum: pub enum FooError { VariantA, VariantB }
    (only variants tests assert against)
  - Function body: todo!() or unimplemented!()

Unacceptable stubs:
  - Full method implementations "to save time"
  - Speculative fields/variants not referenced by any test
  - Default trait implementations (defeats the purpose of testing)
```

### Test Placement Conventions

Follows the established project patterns [R3 Section 5]:

| Layer | Location | Access Level |
|-------|----------|-------------|
| Unit (domain, usecase) | `#[cfg(test)] mod tests` in same file | Private items accessible |
| Integration (adapter) | `tests/{entity}_repository_integration.rs` | Public API only |
| Service (gRPC) | `tests/{entity}_service_test.rs` | gRPC client interface |
| E2E (Gateway) | `tests/{feature}_integration.rs` | HTTP client interface |

### Test Infrastructure Patterns

**Unit tests (mock pattern):**
```rust
struct TestContext {
    xxx_repo: MockXxxRepository,
}
impl XxxPorts for TestContext {
    fn xxx_repo(&self) -> &impl XxxRepository { &self.xxx_repo }
}
```

**Integration tests (testcontainers pattern):**
```rust
static TEST_CONTAINER: OnceCell<TestContainer> = OnceCell::const_new();
async fn make_db() -> DatabaseConnection { ... }
```

**Service tests (in-process gRPC):**
```rust
async fn start_service() -> XxxServiceClient<tonic::transport::Channel> { ... }
```

### Compilation Cost Considerations

Per-test TDD cycling is prohibitively expensive in Rust. Each `cargo test` cycle takes 3-8 seconds (warm cache). A usecase with 6 cases means 18-48 seconds just in compilation per full round-trip [R4 Section 4].

**Chosen granularity: case group (one operation from CASES.md).**

Write ALL tests for one operation, verify Red, implement, verify Green, refactor. Then move to the next operation. This provides meaningful scope boundaries without excessive compilation cycles [R4 Section 4.3].

| Trigger | Command | Purpose |
|---------|---------|---------|
| After stubs | `cargo check -p {crate}` | Fast compilation check (no test execution) |
| After writing tests (Red) | `cargo test -p {crate} -- {prefix}` | Verify tests fail for expected reason |
| After implementation (Green) | `cargo test -p {crate} -- {prefix}` | Verify tests pass |
| After refactoring | `cargo test -p {crate}` | Full crate regression check |
| After task completion | `cargo test -p {crate}` + `cargo clippy -p {crate}` | Final verification |

---

## What Changes in the Workflow

### Current Pipeline (with /test-gen)

```
gsd:discuss -> /case -> (gsd:research) -> gsd:plan -> (gsd:review) -> (gsd:assumptions)
  -> /test-gen -> gsd:execute -> (gsd:validate) -> gsd:verify -> gsd:ship
```

/test-gen sits between plan and execute. It reads CASES.md and PLAN.md, generates test skeleton files with `todo!()` bodies, and the executor then fills in the implementations.

### Proposed Pipeline (without /test-gen)

```
gsd:discuss -> /case -> (gsd:research) -> gsd:plan -> (gsd:review) -> (gsd:assumptions)
  -> gsd:execute -> (gsd:validate) -> gsd:verify -> gsd:ship
```

/test-gen is removed. The executor writes tests as part of each task's TDD cycle, guided by CLAUDE.md protocol and PLAN.md `<behavior>` items.

### What Happens to Each Artifact

| Artifact | Change |
|----------|--------|
| **CASES.md** | **No change.** Remains as behavioral specification. Case IDs flow through PLAN.md `<behavior>` items. |
| **/case skill** | **No change.** Still produces CASES.md with S/F/E cases per operation. |
| **PLAN.md** | **Minor change.** `<behavior>` items already exist. Planner should annotate them with case IDs when CASES.md exists. `<action>` steps for `tdd="true"` tasks should use RED/GREEN/REFACTOR structure. |
| **CLAUDE.md** | **Addition.** New `## TDD Protocol` section with the executor injection rules. |
| **WORKFLOW.md** | **Update.** Remove /test-gen from pipeline diagram and step table. Add note that TDD is executor-level. |
| **Test skeletons** | **Eliminated as a separate artifact.** Tests are created during execution, not before. |
| **gsd:execute** | **Enhanced.** Executor follows TDD protocol from CLAUDE.md for `tdd="true"` tasks. |

### Impact on the Full Flow

**discuss -> case:** Unchanged. /case still discovers operations and S/F/E cases.

**case -> plan:** Unchanged. Planner reads CASES.md, creates `<behavior>` items with case ID annotations, sets `tdd="true"` on appropriate tasks.

**plan -> execute:** Simplified. No /test-gen step between them. The executor reads `<behavior>` items and creates tests during the task, not before.

**execute (internal):** For each `tdd="true"` task, the executor follows Stub-Red-Green-Refactor per case group. For non-TDD tasks, no change.

---

## Conflict Resolution

### R1 vs R4: "Delete and start over" when implementation written before tests

R1 (from the superpowers TDD skill) prescribes: "Write code before the test? Delete it. Start over. Don't keep as reference, don't adapt it, don't look at it."

R4 notes this is too extreme for an AI agent context: "Agent should revert to last green state, not delete everything."

**Resolution: Revert the implementation, do not keep as reference.** The spirit of the rule (prevent the implementation from biasing test design) is correct and important. But "delete everything including context" is impractical for an agent operating within a task loop. The agent should `git checkout` the implementation files back to the pre-implementation state, then write tests without referencing what it previously wrote. This preserves TDD integrity without destroying task-level progress. [R1 adapted per R4]

### R1 vs R4: Per-behavior vs case-group granularity

R1 recommends per-behavior cycling (one test, one implementation, one refactor per behavior item -- matching the superpowers skill).

R4 recommends case-group granularity (all tests for one operation, then implementation) based on Rust compilation costs and the TDAD paper finding that per-test TDD costs 4x more than batch approaches.

**Resolution: Case-group granularity.** The compilation cost argument is decisive for Rust. Writing 6 test stubs for `create_user` and running them once (3-5 seconds) is far more efficient than 6 separate compile-test-implement cycles (18-30+ seconds). The TDAD research confirms that granular TDD instructions without proper context actually increase regressions [R4 Section 4.3]. Case-group provides meaningful scope boundaries (one operation = one TDD cycle) that are natural for both the codebase and the executor.

### R3 vs R4: Where CASES.md case IDs appear

R3 notes that CLAUDE.md says case IDs should go in `acceptance_criteria`, but recommends `<behavior>` items instead.

R4's proposed rules reference case IDs in task acceptance criteria for coverage verification.

**Resolution: Case IDs in `<behavior>` items, NOT in `<acceptance_criteria>`.** R3's argument is more grounded in how the artifacts actually work. `<acceptance_criteria>` contains grep-verifiable conditions ("file X contains Y"). Case IDs are traceability markers, not verifiable conditions. The `<behavior>` item is the natural bridge between case IDs and test functions. Update CLAUDE.md's "CASES.md Integration with Plan-Phase" section to reflect this. [R3 Section 2]

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Injection mechanism (CLAUDE.md) | **HIGH** | Confirmed by GSD-AGENT-PATTERNS.md, WORKFLOW-RESEARCH.md, CLAUDE-AGENT-MECHANISMS.md. Every executor reads CLAUDE.md. [R2] |
| TDD protocol (Stub-Red-Green-Refactor) | **HIGH** | Grounded in existing codebase patterns (13 test files analyzed), multi-source AI TDD research, and Rust compilation cost measurements. [R1, R3, R4] |
| CASES.md -> test mapping | **HIGH** | Based on established `<behavior>` pattern in existing PLAN.md files and 1:1 mapping observed in implemented test functions. [R3] |
| AI agent failure modes | **HIGH** | Documented independently by 8+ sources (TDAD paper, Simon Willison, Addy Osmani, Jason Gorman, Builder.io, etc.). [R4] |
| Executor TDD enforcement | **MEDIUM** | The exact content of `gsd-executor.md` was not directly readable. Whether the executor has built-in TDD methodology or simply follows `<action>` steps is inferred, not confirmed. [R2 Section 8] |
| Case-group granularity | **MEDIUM-HIGH** | Supported by TDAD research and compilation cost analysis, but not yet validated in practice with this project's executor. [R4 Section 4.3] |

**Overall confidence: HIGH.** The injection mechanism is well-understood, the protocol is grounded in concrete patterns, and the risks have clear mitigations.

### Gaps to Address

1. **Executor's built-in TDD behavior is unknown.** The GSD executor agent definition (`gsd-executor.md`) could not be directly read [R2 Section 8]. If the executor already has TDD enforcement that conflicts with CLAUDE.md rules, there could be tension. **Mitigation:** Test with a single `tdd="true"` task before full adoption. If the executor ignores CLAUDE.md TDD rules, escalate to Point C (project-local executor override).

2. **CASES.md has never been exercised with the planner.** No phase has used /case yet. The `<behavior>` item + case ID annotation format is specified but untested [R3 Section 1]. **Mitigation:** The first phase using /case will validate the mapping. If the planner does not produce case ID annotations in `<behavior>` items, add explicit planner guidance to CLAUDE.md.

3. **Refactoring quality is hard to enforce.** R4 identifies refactoring as the least-developed area in AI TDD tooling. `cargo clippy` catches mechanical issues but not design improvements. **Mitigation:** Start with clippy + explicit checklist. Evaluate after a few phases whether a more structured refactoring protocol is needed [R4 Q2].

4. **Integration test TDD cadence is untested.** The Stub-Red-Green-Refactor cycle is designed for unit tests (usecase layer). Integration tests with testcontainers have 5-10 second startup overhead that may warrant a lighter cycle [R4 Q3]. **Mitigation:** For integration test tasks, use a lighter protocol: write tests first (Red), implement adapter (Green), skip per-case-group granularity. Test all adapter operations in one cycle.

---

## Open Questions

### Q1: Should CLAUDE.md's Existing "CASES.md Integration" Text Be Updated?

**Status:** Needs user confirmation.

CLAUDE.md currently says: "Reference case IDs as `OperationName.S1` in task `acceptance_criteria`." R3 recommends moving case IDs to `<behavior>` items instead. This is a documentation change to CLAUDE.md that should be made alongside the TDD protocol addition.

**Recommendation:** Update `acceptance_criteria` reference to `<behavior>` items during implementation. Both are CLAUDE.md changes and can be committed together.

### Q2: Scope of CLAUDE.md TDD Protocol -- All Tasks or Only tdd="true" Tasks?

**Status:** Needs user input.

R2 suggests two levels: strict TDD for `tdd="true"` tasks, and "prefer test-first" for all behavioral tasks. R4's guardrails are written for `tdd="true"` tasks specifically.

**Recommendation:** Strict protocol for `tdd="true"` tasks only. Other tasks follow the existing CLAUDE.md testing guidance ("TDD strongly preferred"). Forcing full Stub-Red-Green-Refactor on infrastructure or proto tasks is wasteful.

### Q3: Atomic Commits -- Test+Impl Together or Separate?

**Status:** Resolved in favor of together, but flagging for awareness.

R2 explicitly recommends committing test and implementation together: "Do NOT commit failing tests separately (that creates a commit where cargo test fails)." This is the correct approach for GSD where each task produces one atomic commit. The Red phase is a transient state within task execution, not a committed state.

---

## Sources

### Primary (HIGH confidence -- direct project artifacts and analysis)
- [R1] `.planning/research/TDD-R1-SKILL-ANALYSIS.md` -- Superpowers TDD skill gap analysis
- [R2] `.planning/research/TDD-R2-EXECUTE-WORKFLOW.md` -- GSD execute workflow injection points
- [R3] `.planning/research/TDD-R3-CASES-MAPPING.md` -- CASES.md to test mapping strategy
- [R4] `.planning/research/TDD-R4-AI-TDD-PATTERNS.md` -- AI agent TDD behavioral patterns

### Secondary (HIGH confidence -- project codebase)
- `services/user/src/usecase/create_user.rs` -- Established unit test patterns
- `services/user/tests/user_service_test.rs` -- Established service test patterns
- `services/user/tests/user_repository_integration.rs` -- Established integration test patterns
- `.planning/phases/02-user-profile/02-01-PLAN.md` -- Existing PLAN.md format with `tdd="true"` and `<behavior>` sections
- `.planning/WORKFLOW.md` -- Current pipeline with /test-gen
- `CLAUDE.md` -- Project test conventions

### Tertiary (MEDIUM confidence -- external research)
- TDAD Paper (arXiv:2603.17973) -- Test-Driven Agentic Development research [cited in R4]
- Simon Willison -- Red/Green TDD as highest-leverage AI agent pattern [cited in R4]
- GSD-AGENT-PATTERNS.md, CLAUDE-AGENT-MECHANISMS.md -- GSD internals [cited in R2]

---

*Research synthesized: 2026-03-24*
*Ready for implementation: yes -- pending user confirmation on Q1 and Q2*
