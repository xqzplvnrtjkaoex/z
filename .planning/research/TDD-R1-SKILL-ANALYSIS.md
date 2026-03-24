# TDD Skill Analysis for Executor Injection

**Analyzed:** 2026-03-24
**Domain:** TDD skill rules, anti-patterns, and integration gaps for GSD executor injection
**Source:** `.planning/research/_tdd-skill-source/SKILL.md` and `testing-anti-patterns.md` (from superpowers plugin)
**Confidence:** HIGH (direct source analysis, no external lookups needed)

## Summary

The existing TDD skill from superpowers is a well-structured, opinionated document that prescribes strict Red-Green-Refactor discipline with enforcement through rationalizations tables and red flags. However, it is designed as a **standalone developer mindset document**, not as an executable protocol for an AI agent operating within a task loop. It assumes a human developer making continuous judgment calls, uses TypeScript/npm examples exclusively, and has no awareness of GSD artifacts (CASES.md, PLAN.md), test layers, or per-task cycling. The anti-patterns document is higher-quality for our purposes -- its "gate functions" (decision trees before taking action) are directly translatable to agent decision points.

**Primary finding:** The skill's *principles* are fully reusable. Its *execution protocol* needs significant rework. The rules tell an agent WHAT to believe about TDD but not HOW to execute TDD within a structured task loop. The gap is the operational protocol: when to enter/exit the TDD cycle, which test layer to target, how to handle Rust-specific compilation failures, and how to map CASES.md behaviors to test functions.

---

## 1. TDD Skill Rules Inventory

### Red-Green-Refactor Rules Defined

The skill defines a strict 5-step cycle (not the typical 3):

| Step | Rule | Enforcement |
|------|------|-------------|
| **RED** | Write ONE minimal test showing what should happen | "One behavior, clear name, real code (no mocks unless unavoidable)" |
| **Verify RED** | Run test, confirm it FAILS (not errors) | "MANDATORY. Never skip." Explicit check: fails because feature missing, not typos |
| **GREEN** | Write simplest code to pass the test | "Don't add features, refactor other code, or 'improve' beyond the test" |
| **Verify GREEN** | Run test, confirm it PASSES, confirm other tests still pass | "MANDATORY." Explicit check: output pristine (no errors, warnings) |
| **REFACTOR** | Remove duplication, improve names, extract helpers | "Keep tests green. Don't add behavior." |

The 5-step cycle is important -- it separates "write" from "verify" in both RED and GREEN phases, making the verification steps first-class.

### "Write Test First" Enforcement

The skill uses an absolute enforcement model:

1. **The Iron Law:** `NO PRODUCTION CODE WITHOUT A FAILING TEST FIRST`
2. **Deletion rule:** "Write code before the test? Delete it. Start over." -- No exceptions: don't keep as reference, don't adapt it, don't look at it.
3. **Red Flags list** (12 items) that all trigger "Delete code. Start over with TDD."
4. **Common Rationalizations table** (11 excuses with rebuttals)

### Cycle Structure

The skill prescribes **per-behavior** cycling, not per-function or per-feature:

- "Write one minimal test showing what should happen" (singular)
- "One behavior" in the quality table
- "and" in test name means split it
- The example flow (bug fix) shows one test -> one implementation -> one refactor

This is the correct granularity for agent injection. Each test-implement-verify cycle covers exactly one behavior from CASES.md or the plan's `<behavior>` list.

### Handling Non-Existent Types/Functions

The skill does NOT explicitly address this. It assumes the test can call code that doesn't exist yet (TypeScript is lenient here). The closest guidance:

- RED phase: "Write wished-for API" (from the "When Stuck" table)
- Verify RED: "Test fails (not errors)" -- but in Rust, a test calling a non-existent function is a **compilation error**, not a test failure

**This is a critical gap for Rust.** In TypeScript, you can write `const result = await retryOperation(operation)` and the test will fail at runtime. In Rust, `use crate::usecase::create_user` will fail at compile time if `create_user` doesn't exist. The skill has no protocol for distinguishing "expected compilation failure" (the function doesn't exist yet, which is correct in RED phase) from "unexpected compilation failure" (typo, wrong import, bad type signature).

---

## 2. Testing Anti-Patterns

### Anti-Patterns Catalogued

| # | Anti-Pattern | Gate Function Provided? |
|---|-------------|------------------------|
| 1 | Testing mock behavior instead of real behavior | Yes |
| 2 | Test-only methods in production classes | Yes |
| 3 | Mocking without understanding dependencies | Yes |
| 4 | Incomplete mocks (partial data structures) | Yes |
| 5 | Integration tests as afterthought | No (just "use TDD") |

### AI Agent-Specific Relevance

**Highly relevant to AI agents:**

- **Anti-pattern 1 (testing mocks):** AI agents are particularly prone to this. When generating tests, an agent might mock a dependency and then assert on the mock's return value rather than the SUT's behavior. The gate function ("Am I testing real component behavior or just mock existence?") is directly embeddable as an agent checkpoint.

- **Anti-pattern 4 (incomplete mocks):** AI agents tend to create minimal mock return values with only the fields they "know about" from the current context. The gate function ("What fields does the real API response contain?") is critical -- an agent should check the actual type definition, not guess.

- **Anti-pattern 5 (tests as afterthought):** This is the ENTIRE reason for TDD injection. Without enforcement, the executor will naturally write implementation first and tests second (or not at all). The skill's Iron Law directly addresses this.

**Less relevant to AI agents:**

- **Anti-pattern 2 (test-only methods):** AI agents rarely add methods to production classes for test convenience; they're more likely to over-mock. Still worth keeping as a rule.

- **Anti-pattern 3 (mocking without understanding):** AI agents may actually be BETTER at this than humans because they can read the entire dependency chain in one pass. However, the "mock just to be safe" pattern is still a risk.

### Rust-Specific Relevance

The anti-patterns document is language-agnostic but TypeScript-centric in examples. For Rust:

- **Testing behavior vs implementation:** The project already follows this well. Unit tests mock ports (trait implementations), not internal functions. The `MockUserRepository` pattern tests usecase behavior, not repository implementation.

- **Mock complexity:** The mockall + trait_variant pattern in this project is a good fit. The `TestContext` struct wrapping `MockUserRepository` is clean. The anti-pattern of "mock setup longer than test logic" should trigger consideration of integration tests (testcontainers).

- **Missing Rust-specific anti-pattern:** The documents don't address the Rust-specific issue of testing `From`/`Into` implementations, error mapping, and type conversion chains -- which this project does extensively (every `UserError` variant has a `tonic::Status` mapping test).

---

## 3. Integration Points

### GSD Artifact Awareness

**None.** The skill has zero references to:

- CASES.md (behavioral specifications)
- PLAN.md (task decomposition with `<behavior>` sections)
- CONTEXT.md (design decisions)
- RESEARCH.md
- Any GSD workflow artifact

It is a standalone skill designed to be activated by a developer reading it, not consumed by an orchestration system.

### Executor Workflow Interaction

The skill is designed for **manual invocation** -- a developer reads it, internalizes the rules, and applies them. It is NOT:

- A machine-parseable protocol
- Integrated with any task loop
- Aware of task boundaries (when to start/stop cycling)
- Aware of multi-task dependencies (test from task 2 depends on code from task 1)

### Composition Design

The skill references `testing-anti-patterns.md` via `@testing-anti-patterns.md`, suggesting it was designed for Claude's `@` file reference system. The anti-patterns document is meant to be loaded alongside the main skill when writing tests.

The skill is **not composable with other workflow stages**. It doesn't define entry/exit conditions, doesn't produce artifacts, and doesn't have machine-readable status indicators.

---

## 4. Gaps for Executor Injection

### Critical Gaps

**Gap 1: No per-task TDD protocol.**
The skill defines per-behavior cycling but has no concept of "task" as a unit. In GSD execution, a task may contain 3-10 behaviors (see the `02-01-PLAN.md` Task 2 which lists 13 behaviors). The skill doesn't address: Do you write all 13 tests first? One at a time? Grouped by concern?

**Recommendation:** Per-behavior cycling within a task, but with grouping intelligence. For Rust, behaviors that share a module/type should be grouped so the compilation scaffolding (stubs) is created once.

**Gap 2: Rust compilation barrier in RED phase.**
In Rust, "write a failing test" often means "write a test that doesn't compile." The skill's Verify RED step says "Test fails (not errors)" -- but in Rust, a compilation error IS the expected "failure" when the function under test doesn't exist yet.

**Recommendation:** Define a two-stage RED for Rust:
1. RED-COMPILE: Write test. Expected outcome: compilation fails because SUT doesn't exist. Create minimal stub (function signature + `todo!()` or `unimplemented!()`).
2. RED-FAIL: Test compiles but fails. This is the real RED. The stub returns wrong value or panics.

**Gap 3: No test layer awareness.**
The skill treats all tests as the same. This project has 4 layers:

| Layer | Location | Speed | When |
|-------|----------|-------|------|
| Unit | `#[cfg(test)] mod tests` inline | <1s | Always (every behavior) |
| Integration | `tests/*.rs` | 5-30s (testcontainers) | Adapter implementations |
| Service | `tests/*_service_test.rs` | 10-60s | After service wiring |
| E2E | `tests/*_e2e.rs` | 30s+ | After gateway routes |

The skill has no guidance on which layer to use for which behavior. The plan's `<behavior>` list doesn't specify layers either (though the plan's `tdd="true"` attribute exists).

**Recommendation:** The executor needs a layer selection rule:
- Domain/usecase behaviors -> unit tests (inline `mod tests`)
- Adapter behaviors -> integration tests (`tests/` directory)
- Service wiring behaviors -> service tests
- Cross-service behaviors -> E2E tests

**Gap 4: No `#[cfg(test)]` and file placement guidance.**
Rust tests can live:
- Inline in `#[cfg(test)] mod tests` (same file as SUT)
- In `tests/` directory (integration tests, separate compilation)
- In `tests/` as `#[test]` functions (no `cfg(test)` needed)

The skill doesn't address where to put tests. For this project, the convention is well-established (unit tests inline, integration/service tests in `tests/`), but the executor needs explicit instructions.

**Gap 5: No CASES.md -> test mapping.**
When CASES.md exists, each case ID (e.g., `CreateUser.S1`, `CreateUser.F2`) should map to a test function. The skill has no awareness of this mapping. The plan's `<behavior>` section partially bridges this gap (it lists behaviors derived from cases), but the executor needs to know: "For behavior X from CASES.md, create test `should_...` in module Y."

**Gap 6: No handling of task dependencies.**
Task 2 may depend on types created in Task 1. The TDD cycle for Task 2 can't start until Task 1's artifacts exist. The skill has no concept of cross-task dependencies affecting test writability.

**Gap 7: No refactor-across-tasks guidance.**
The skill's REFACTOR step is scoped to the current behavior. In practice, after completing a task (multiple behaviors), there's a natural refactoring opportunity across the task's behaviors. The skill doesn't distinguish "refactor within one cycle" from "refactor at task boundary."

### Minor Gaps

- **No `cargo test` command guidance** (uses `npm test` exclusively)
- **No test naming convention** for Rust (`should_...` pattern used in this project)
- **No mockall-specific patterns** (how to set up `MockUserRepository`, `expect_*` calls)
- **No async test guidance** (`#[tokio::test]`)
- **No test helper/fixture patterns** for Rust (the `TestContext` pattern)

---

## 5. Reusable Components

### Directly Reusable (embed as-is)

| Component | Location in Skill | Use in Executor |
|-----------|------------------|-----------------|
| The Iron Law | Lines 33-36 | Agent pre-check before writing any implementation code |
| RED quality requirements | Lines 109-111 | Test quality gate: one behavior, clear name, real code |
| GREEN minimality rule | Lines 130-166 | Implementation scope gate: simplest code to pass |
| Verification Checklist | Lines 329-341 | Task completion gate (adapted for per-task scope) |
| Common Rationalizations | Lines 258-271 | Agent self-check (the agent WILL rationalize skipping TDD) |
| Red Flags list | Lines 273-287 | Agent self-monitoring for TDD violation |
| Anti-pattern gate functions | All 4 gates | Agent decision points before mocking, before adding methods |

### Needs Adaptation

| Component | Adaptation Needed |
|-----------|-------------------|
| Verify RED ("fails, not errors") | Add Rust compilation stage: compile error -> create stub -> then verify fails |
| `npm test` commands | Replace with `cargo test -p {crate} -- {test_name}` |
| "Delete and start over" | Too extreme for agent context -- agent should revert to last green state, not delete everything |
| TypeScript examples | Replace with Rust equivalents from the project's existing test patterns |
| "Ask your human partner" | Replace with "flag for human review" in PLAN.md |
| Mock guidance ("no mocks unless unavoidable") | Adapt to "mock ports for unit tests, real implementations for integration tests" |

### New Rules Needed (not in skill)

| Rule | Purpose |
|------|---------|
| **Test layer selection** | Given a behavior, determine unit/integration/service/E2E |
| **Rust RED-COMPILE protocol** | How to handle the compilation barrier before getting to RED-FAIL |
| **Stub creation rules** | What stubs are acceptable (function signature + `todo!()`, empty trait impl) |
| **CASES.md -> test mapping** | How to derive test function name and location from a case ID |
| **Per-task entry/exit** | When to start TDD cycling for a task, when to stop, what to verify at task boundary |
| **Cross-task dependency handling** | How task N's tests can reference task N-1's artifacts |
| **Test file placement** | When to use inline `mod tests` vs `tests/` directory |
| **mockall setup patterns** | Standard `TestContext` + `MockUserRepository` setup for this project |
| **Async test patterns** | `#[tokio::test]` usage, when `#[test]` vs `#[tokio::test]` |
| **Error mapping tests** | Pattern for testing `UserError -> tonic::Status` and `From` impls |
| **Compilation verification** | `cargo build -p {crate}` as intermediate check (not just `cargo test`) |

---

## 6. Structural Observations

### Skill's Pedagogical vs Operational Nature

The skill is pedagogical: it teaches principles and builds conviction. Approximately 60% of its content is motivational/persuasive (the "Why Order Matters" section, rationalizations table, red flags). This content is valuable for preventing an AI agent from rationalizing TDD skips, but it's not operational.

For executor injection, we need the opposite ratio: 80% operational protocol ("do this, then this, check this") and 20% principle reinforcement ("remember: no production code without a failing test").

### The `<behavior>` Section in Existing Plans

The project's PLAN.md already includes a `<behavior>` section listing expected tests (see `02-01-PLAN.md` Task 2 with 13 behaviors). This is a natural mapping point. The executor already knows WHAT to test -- it needs HOW to test it in TDD order.

The `tdd="true"` attribute on `<task>` elements is also already present, suggesting the plan format was designed with TDD injection in mind but the executor doesn't act on it yet.

### Anti-Patterns Gate Functions as Agent Checkpoints

The anti-patterns document's gate functions are the most operationally useful part of both documents. They follow a decision-tree format that translates directly to agent logic:

```
BEFORE [action]:
  Ask: "[question]"
  IF [condition]:
    STOP - [corrective action]
  [proceed]
```

This pattern should be the model for all new TDD rules in the executor.

---

## 7. Summary: Reuse Strategy

```
                    Existing TDD Skill
                    ==================

  Directly Reusable          Needs Adaptation          New (doesn't exist)
  =================          ================          ===================
  - Iron Law                 - Verify RED step         - Test layer selection
  - RED quality rules        - npm -> cargo            - Rust RED-COMPILE protocol
  - GREEN minimality         - Delete -> revert        - Stub creation rules
  - Verification checklist   - TS -> Rust examples     - CASES.md -> test mapping
  - Rationalizations table   - Mock guidance           - Per-task entry/exit
  - Red flags list           - "Ask human" -> flag     - Cross-task dependencies
  - Gate functions (all 4)                             - Test file placement
                                                       - mockall patterns
                                                       - Async test patterns
                                                       - Error mapping tests
                                                       - Compilation verification
```

**Ratio:** ~35% directly reusable, ~25% needs adaptation, ~40% must be created new.

The skill provides a strong philosophical foundation and useful enforcement mechanisms. What it lacks is the entire operational layer for Rust-specific, task-scoped, artifact-aware TDD execution.

---

## Sources

### Primary (HIGH confidence)
- `.planning/research/_tdd-skill-source/SKILL.md` -- full TDD skill definition, 371 lines
- `.planning/research/_tdd-skill-source/testing-anti-patterns.md` -- anti-patterns reference, 300 lines

### Secondary (HIGH confidence)
- `CLAUDE.md` -- project test conventions, test layers, build commands
- `services/user/src/usecase/create_user.rs` -- existing unit test patterns (mockall, TestContext)
- `services/user/tests/user_service_test.rs` -- existing service test patterns (testcontainers, tonic)
- `services/user/src/domain/ports/user_repository.rs` -- mockall + trait_variant pattern
- `services/user/src/domain/input/handle_input.rs` -- domain validation test patterns
- `.planning/phases/02-user-profile/02-01-PLAN.md` -- existing PLAN.md format with `tdd="true"` and `<behavior>` sections
- `.planning/WORKFLOW.md` -- pipeline definition showing /test-gen and executor positions

### Analysis Methodology
Direct source reading and comparison against project codebase. No web searches needed -- this is a closed-system analysis of existing artifacts.
