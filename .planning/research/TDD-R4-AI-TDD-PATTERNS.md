# AI Agent TDD Behavioral Patterns - Research

**Researched:** 2026-03-24
**Domain:** AI agent test-driven development, Rust-specific TDD, executor agent behavioral rules
**Confidence:** MEDIUM-HIGH (synthesized from multiple 2025-2026 sources, cross-verified)

## Summary

AI coding agents have a fundamental tension with TDD: they are trained on completed codebases, not on the disciplined process that produced them. Without explicit rules, agents skip the Red phase, write tests that confirm broken behavior, over-implement beyond what tests require, and neglect refactoring. The research community converged in 2025-2026 on a clear consensus: **TDD is the highest-leverage workflow for AI agents**, but only when the agent is given explicit behavioral constraints that prevent its natural shortcuts.

For Rust specifically, the "Red" phase has a unique characteristic: tests often fail to *compile* before they fail to *assert*. This creates a two-stage Red phase (compile-error Red, then assertion-failure Red) that requires different treatment from dynamic languages. The project's 4-layer architecture with trait-based ports provides a natural boundary for TDD: usecase tests mock ports, and the compilation step validates the contract before behavior is tested.

**Primary recommendation:** Adopt a "Stub-then-Red-then-Green" cycle for Rust AI TDD, where the agent creates minimal type/trait stubs to make tests compile, then writes assertion-failing tests, then implements. Enforce at the case-group level (not per-test-function) for cost-effective feedback loops in a compiled language.

## 1. AI Agent TDD vs Human TDD

### 1.1 Current Tool Landscape

**Confidence: HIGH** (based on multiple 2025-2026 sources including official tool docs and user experience reports)

No major AI coding tool has a built-in enforced TDD mode. TDD compliance is achieved through prompting, hooks, or external enforcement:

| Tool | TDD Support | Mechanism |
|------|-------------|-----------|
| Claude Code | Prompt-based ("use red-green TDD") | Follows instructions but drifts without enforcement. Hooks API enables external enforcement (tdd-guard). |
| Cursor | No explicit TDD mode | User must prompt for test-first. Naturally writes implementation first. |
| Aider | No explicit TDD mode | Can be prompted for TDD. No built-in enforcement. |
| Codex (OpenAI) | No explicit TDD mode | Sandbox execution enables test feedback loops. No TDD enforcement. |

**Key insight from Simon Willison:** "Use red-green TDD" is the highest-leverage prompt for a coding agent -- it simultaneously validates correctness, prevents unnecessary code, and builds a regression suite. Five tokens that fundamentally change agent output quality.

### 1.2 Common Failure Modes

**Confidence: HIGH** (documented across multiple independent sources)

| Failure Mode | Description | Frequency | Impact |
|--------------|-------------|-----------|--------|
| **Tests-after-code** | Agent writes implementation first, then tests that confirm existing behavior | Very common (default behavior) | Tests cannot catch bugs because they were written to pass |
| **Trivially passing tests** | Tests that assert truthy values, test setup code, or verify obvious things | Common | False sense of coverage; zero regression protection |
| **Testing implementation details** | Tests depend on internal structure (field names, private methods, call order) rather than behavior | Common | Tests break on refactoring; maintenance burden |
| **Simultaneous test+impl** | Agent writes test and implementation in the same edit | Very common | Defeats the purpose of TDD -- no Red phase verification |
| **Over-implementation** | Agent writes full feature implementation when only one test needs to pass | Very common | Unnecessary code, potential bugs in untested paths |
| **Over-mocking** | Mocking too many dependencies, testing mock configuration rather than business logic | Moderate | Tests pass but integration fails; false confidence |
| **Batch test generation** | Writing all tests at once, then implementing everything | Common | No iterative design benefit; scope explosion |
| **Skipping refactoring** | After Green, moving directly to next test without cleanup | Very common | Code quality degrades; duplication accumulates |

### 1.3 Why TDD Works Better for AI Agents Than for Humans

**Confidence: HIGH** (multiple authoritative sources agree)

From Jason Gorman (Codemanship): TDD works well with AI because it prevents the fundamental failure mode where agents write tests that verify broken behavior. When tests exist before code, the agent cannot "cheat" by writing a test that confirms whatever incorrect implementation it produced.

From Builder.io: Everything that makes TDD tedious for humans (writing boilerplate, repetitive assertion patterns, edge case enumeration) is exactly what AI agents excel at. TDD's biggest human weakness becomes an AI accelerator.

From TDAD paper (arXiv:2603.17973): Evaluated on SWE-bench Verified, test-aware development reduced regressions by 70%. However, **TDD procedural instructions without context about which tests to check actually increased regressions** (6.08% baseline to 9.94% with TDD-only prompting). The agent needs to know *what* to test, not just *how* to test.

**Critical finding:** Procedural TDD instructions consume context tokens that smaller models need for repository understanding. Rules must be concise and targeted, not verbose.

## 2. Guardrails for AI TDD

### 2.1 Test Quality Guardrail

**Confidence: MEDIUM-HIGH**

**Problem:** Agent writes tests that test implementation details instead of behavior.

**Rule:** Tests must assert on observable outputs (return values, side effects via mock expectations, error variants) -- never on internal implementation structure.

**Enforcement mechanisms:**
- Test names must describe behavior, not implementation: `should_reject_owner_role_with_owner_role_rejected_error` (good, matches project pattern) vs `test_create_user_calls_save` (bad -- tests call sequence)
- Mock expectations should use `returning()` for output verification, not excessive `withf()` for input structure verification
- Assert against domain error variants (`UserError::HandleTaken`), not error messages or string contents

**Project-specific pattern (already established):**
```rust
// Good -- tests behavior (observable output)
let result = create_user(&ctx, payload).await;
assert!(matches!(result, Err(UserError::OwnerRoleRejected)));

// Bad -- tests implementation detail (call count, argument structure)
mock.expect_save().times(1).withf(|u| u.role == UserRole::User);
```

### 2.2 Red Phase Guardrail

**Confidence: HIGH**

**Problem:** Agent writes test and implementation simultaneously, skipping verification that the test actually fails.

**Rule:** After writing a test (or group of tests for one case), the agent MUST run the test and verify it produces the expected failure before writing implementation code.

**Rust-specific nuance:** In Rust, "failure" has two stages:
1. **Compile-error failure** -- type/function does not exist yet
2. **Assertion failure** -- code compiles but behavior is wrong

Both are valid "Red" states. The agent must explicitly document which Red state it expects.

**Enforcement:** After writing tests, run `cargo test -p {crate} -- {test_name}` and verify the output contains the expected error (compile error or assertion failure). Log the failure reason before proceeding.

### 2.3 Scope Guardrail

**Confidence: HIGH** (strongest consensus across sources)

**Problem:** Agent writes complete feature implementation when only one test case needs to pass.

**Rule:** Implement only what the current failing test(s) require. Do not add error handling for cases not yet tested. Do not add branches for scenarios not yet covered.

**Practical adaptation for AI agents:** Strict "one test at a time" is prohibitively expensive in compiled languages due to compilation costs. The pragmatic unit is a **case group** -- all S/F/E cases for one operation from CASES.md. This provides meaningful scope boundaries without excessive compilation cycles.

### 2.4 Refactor Guardrail

**Confidence: MEDIUM** (least-developed area in current tooling)

**Problem:** Agent skips refactoring after getting tests to pass, or makes only superficial changes.

**Rule:** After all tests for a case group pass, explicitly check for:
- Duplicated code between test cases (extract helpers/fixtures)
- Duplicated code in implementation (extract functions)
- Naming that no longer reflects behavior
- Unnecessary complexity introduced during Green phase

**Practical trigger for Rust:** Run `cargo clippy -p {crate}` after Green phase. Clippy catches many refactoring opportunities (redundant clones, unnecessary allocations, missing derives) that an agent might miss.

### 2.5 Coverage Guardrail

**Confidence: HIGH**

**Problem:** Agent skips edge cases, testing only the happy path.

**Rule:** When CASES.md exists, every must-priority S/F/E case must have a corresponding test. The agent must verify case coverage before marking a task complete.

**Enforcement:** After implementation, cross-reference test functions against CASES.md case IDs. Any must-priority case without a test is a blocking deficiency.

## 3. Rust-Specific TDD Challenges

### 3.1 The Two-Stage Red Phase

**Confidence: HIGH** (verified against Rust official docs and community practice)

In dynamic languages, the Red phase is simple: write a test, run it, see it fail. In Rust, there are two distinct failure modes:

| Stage | What Happens | Example | Agent Action |
|-------|-------------|---------|--------------|
| **Compile-error Red** | Test references types/functions that don't exist yet | `use crate::domain::types::session::Session;` -- but Session doesn't exist | Create minimal stub (struct with required fields, trait with method signatures) |
| **Assertion-failure Red** | Code compiles but produces wrong output | `assert_eq!(session.is_expired(), true)` -- but `is_expired` returns `false` (stub) | Implement the actual logic |

**Recommendation: Stub-then-Red-then-Green (Option B)**

Create minimal type stubs first, then write tests that compile but fail on assertions, then implement.

**Rationale for Option B over Option A:**

| Factor | Option A (Compile-error first) | Option B (Stub first) |
|--------|-------------------------------|----------------------|
| Compilation cycles | 3+ per test (write test, fail to compile, add stub, compile again, assertion fail, implement) | 2 per group (write stubs + tests, assertion fail, implement) |
| Error signal quality | Compile errors tell you "type missing" (obvious), not "behavior wrong" (useful) | Assertion failures tell you "behavior wrong" (useful signal) |
| Agent context efficiency | Agent must context-switch between "make it compile" and "make it correct" | Agent can focus on correctness after stubs exist |
| Matches project pattern | No -- project stubs already exist from architecture layer ordering | Yes -- domain types/ports are defined before usecases |

**Project-specific alignment:** The 4-layer architecture naturally supports Option B. Domain types and port traits are defined first (they are the stubs). Usecase tests import from `domain/` -- types already exist as compile-target stubs. The "Red" that matters is assertion failure in usecase tests, not "type doesn't exist."

**Practical stub pattern for this project:**

```rust
// Step 1: Define the port trait (stub -- no implementation yet)
#[trait_variant::make(SessionStore: Send)]
pub trait LocalSessionStore {
    async fn save(&self, session: &Session) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Session>, RepositoryError>;
}

// Step 2: Write test against the port (compiles, fails on assertion)
#[tokio::test]
async fn should_create_session_for_authenticated_user() {
    let mut mock_store = MockSessionStore::new();
    mock_store.expect_save().once().returning(|_| Ok(()));
    // ... setup ...
    let result = create_session(&ctx, payload).await;
    assert!(result.is_ok()); // Red: create_session returns Err or is unimplemented
}

// Step 3: Implement create_session to make the test pass (Green)
```

### 3.2 Borrow Checker Interactions in Tests

**Confidence: MEDIUM-HIGH**

Common borrow checker issues in tests:

| Issue | Cause | Solution |
|-------|-------|----------|
| `mock.expect_*().returning()` closure captures | Closure needs to return cloned data | Use `.returning(move \|_\| Ok(value.clone()))` pattern |
| Multiple tests sharing test fixtures | Cannot move value into multiple tests | Use factory functions (`make_test_user()`) -- already established in project |
| Async test lifetimes | References in async blocks outlive scope | Owned types in test fixtures; clone into closures |

**Project-established pattern (from user service tests):**
```rust
// Factory function avoids borrow issues
fn make_test_user(handle: &str, name: &str, role: UserRole) -> User {
    User {
        id: Uuid::new_v4(),
        handle: handle.to_string(),
        name: name.to_string(),
        role,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}
```

### 3.3 Async Test Runners

**Confidence: HIGH** (verified against tokio docs)

| Aspect | Convention | Notes |
|--------|-----------|-------|
| Unit tests | `#[tokio::test]` | Default multi-threaded runtime. Use `#[tokio::test(flavor = "current_thread")]` only when test requires single-threaded execution. |
| Integration tests | `#[tokio::test]` + `OnceCell` for container | Shared container across test file, separate DB connections per test. |
| Service tests | `#[tokio::test]` + tonic in-process channel | Start service on ephemeral port, connect client. |
| Test isolation | Fresh `DatabaseConnection` per test | `OnceCell<TestContainer>` ensures single container startup. |

### 3.4 Trait-Based Testing (Ports Pattern)

**Confidence: HIGH** (established in project, verified in codebase)

The project uses `mockall` with `#[automock(target = UserRepository)]` on `trait_variant`-based traits. This is the correct approach for TDD:

| Test Layer | What to Test | Mock Strategy |
|-----------|-------------|---------------|
| Usecase unit tests | Business logic, validation, error mapping | Mock all ports via `MockXxxRepository`. Use `TestContext` struct implementing `XxxPorts`. |
| Integration tests | Adapter correctness (SQL queries, data mapping) | Real DB (testcontainers). No mocks. Test through port trait interface. |
| Service tests | gRPC request/response mapping, error translation | Real adapters + testcontainers. Test through tonic client. |

**Critical rule for mock setup in TDD:**
- `expect_*().returning()` defines the mock's behavior (what the dependency returns)
- `expect_*().times(N)` verifies the mock was called correctly (interaction verification)
- Prefer `returning()` over `times()` for most tests -- behavior verification is more valuable than interaction verification
- Use `times()` only when the number of calls is part of the behavioral specification (e.g., "save must be called exactly once")

## 4. Test Execution Frequency

### 4.1 Compilation Cost Analysis

**Confidence: HIGH** (measured on actual project)

| Scenario | Command | Time (warm cache) | Time (cold cache) |
|----------|---------|-------------------|-------------------|
| Full workspace build | `cargo test --no-run` | ~8 seconds | ~30-45 seconds |
| Single crate test | `cargo test -p user -- test_name` | ~3-5 seconds | ~15-20 seconds |
| Check only (no codegen) | `cargo check -p user` | ~2-3 seconds | ~10-15 seconds |
| Clippy lint | `cargo clippy -p user` | ~3-5 seconds | ~15-20 seconds |

### 4.2 Recommended Execution Cadence

**Confidence: MEDIUM-HIGH** (synthesized from cost analysis and community practice)

| Trigger | Command | Purpose |
|---------|---------|---------|
| After writing stubs (types/traits) | `cargo check -p {crate}` | Verify stubs compile. Fast feedback, no test execution overhead. |
| After writing test(s) for a case group | `cargo test -p {crate} -- {test_prefix}` | Verify Red state: tests compile and fail for expected reason. |
| After implementing code for a case group | `cargo test -p {crate} -- {test_prefix}` | Verify Green state: tests pass. |
| After refactoring | `cargo test -p {crate}` | Verify refactoring didn't break anything. Run full crate tests, not just the current group. |
| After completing a task (all case groups) | `cargo test -p {crate}` + `cargo clippy -p {crate}` | Full crate verification. Catches cross-test interactions and lint issues. |
| After completing a wave | `cargo test --workspace` | Full workspace regression check. |

### 4.3 Granularity Rationale: Case Group, Not Individual Test

**Confidence: MEDIUM-HIGH**

Strict "one test at a time" TDD with an AI agent in Rust is counterproductive:

- **Cost:** Each `cargo test` cycle takes 3-8 seconds. A usecase with 6 cases means 18-48 seconds just in compilation, plus context window re-reads on each iteration. The TDAD research found incremental TDD cost 4x more than batch approaches.
- **Signal value:** In Rust, if one test for `create_session` compiles and fails correctly, all tests for `create_session` will compile. The marginal information from running one test at a time is low.
- **Natural grouping:** CASES.md already groups cases by operation (e.g., `CreateSession.S1`, `CreateSession.F1`, `CreateSession.E1`). This is the natural TDD unit.

**Recommended granularity:** Write all tests for one operation (one case group from CASES.md), verify Red, implement, verify Green, refactor. Then move to the next operation.

## 5. The "Minimal Implementation" Problem

### 5.1 Why AI Agents Over-Implement

**Confidence: HIGH**

From Addy Osmani's "80% Problem": Given free rein, agents overcomplicate relentlessly, scaffolding 1,000 lines where 100 would suffice and creating elaborate class hierarchies where a function would do. This is because agents are trained on completed codebases, not on the incremental process that built them.

Specific manifestations in Rust:
- Adding error handling branches for error types not yet tested
- Implementing optional features (pagination, sorting) when only basic CRUD is tested
- Creating utility functions that "might be useful later"
- Adding `impl Display`, `impl Debug`, `impl Default` without test-driven need

### 5.2 Strict Minimal vs Task-Scoped Implementation

**Confidence: MEDIUM-HIGH**

| Approach | Description | Pros | Cons |
|----------|-------------|------|------|
| **Strict minimal** (classic TDD) | Implement ONLY what the current failing test requires | Maximum discipline; zero waste | Extremely expensive in compiled languages; artificial refactoring later |
| **Task-scoped** (pragmatic AI TDD) | Implement what the current task's test cases require | Cost-effective; natural scope boundaries | Risk of over-implementation within task |
| **Feature-scoped** (common AI default) | Implement the entire feature | Fast, but high regression risk | No TDD benefit; tests become after-the-fact verification |

**Recommendation:** Task-scoped implementation with explicit constraints.

The agent should implement what the current task requires (all case groups for that task) but MUST NOT implement anything beyond the task boundary. The PLAN.md task definition is the scope boundary, and CASES.md case groups within that task are the iteration units.

**Enforcement rule:** After completing a task, the agent must verify that no code was written that is not exercised by at least one test in the task. Unused code paths are a signal of over-implementation.

### 5.3 The "Just Enough Types" Principle

In the 4-layer architecture, "minimal implementation" has a specific meaning per layer:

| Layer | "Minimal" Means |
|-------|-----------------|
| **domain/types/** | Only fields referenced by tests. No speculative fields. |
| **domain/ports/** | Only trait methods called by tests. No speculative methods. |
| **domain/error/** | Only error variants that tests assert against. No speculative variants. |
| **usecase/** | Only logic paths exercised by tests. No speculative branches. |
| **adapter/** | Not written during TDD unit testing. Written during integration test phase. |
| **app/rpc/** | Not written during TDD unit testing. Written during service test phase. |

## 6. Proposed AI TDD Behavioral Rules

### 6.1 Pre-Implementation Rules

These rules apply before writing any implementation code for a task.

```
RULE P1: Read the task's acceptance criteria and identify all case IDs
         from CASES.md that this task must cover.

RULE P2: Check if required domain types and port traits exist.
         - If they exist: proceed to writing tests.
         - If they don't exist: create minimal stubs (struct with
           required fields, trait with method signatures, error enum
           with required variants). Verify stubs compile with
           `cargo check -p {crate}`.

RULE P3: When creating stubs, implement ONLY what the test signatures
         require. A stub function body should be:
         - For trait methods: just the signature (mockall handles the rest)
         - For concrete functions: `todo!()` or `unimplemented!()`
         - For types: only fields referenced in test setup or assertions

RULE P4: If CASES.md exists for this phase, verify all must-priority
         cases for the current operation have corresponding test
         functions BEFORE writing any implementation.
```

### 6.2 During-Implementation Rules

These rules govern the Red-Green-Refactor cycle.

```
RULE D1 (Write Tests): For the current case group (one operation from
         CASES.md), write ALL test functions (S cases, F cases, E cases).
         Each test function must:
         - Have a descriptive name: should_{behavior}_when_{condition}
           or should_{behavior}_with_{input_description}
         - Assert on observable output (return value, error variant)
         - Use mock expectations for dependency behavior, not verification
         - Be independent (no shared mutable state between tests)

RULE D2 (Verify Red): Run `cargo test -p {crate} -- {operation_prefix}`
         BEFORE writing implementation. Verify that:
         - All tests compile (stubs exist)
         - All tests FAIL (expected assertion failures)
         - Log the failure output to confirm failures are for the
           expected reason, not infrastructure issues
         If tests pass unexpectedly: the test is not testing behavior.
         Rewrite the test with a meaningful assertion.

RULE D3 (Implement Green): Write implementation code to make all
         failing tests pass. Constraints:
         - Implement ONLY logic paths that tests exercise
         - Do NOT add error handling for untested error cases
         - Do NOT add branches for scenarios without tests
         - When a test requires calling a port, implement the usecase
           function using the port trait (not a concrete implementation)

RULE D4 (Verify Green): Run `cargo test -p {crate} -- {operation_prefix}`
         Verify ALL tests pass. If any test fails:
         - Read the failure message carefully
         - Fix the implementation (not the test) unless the test
           has a genuine bug
         - Re-run until green

RULE D5 (Refactor): After Green, run `cargo clippy -p {crate}` and
         address warnings. Then evaluate:
         - Extract shared test setup into helper functions
         - Extract duplicated implementation logic into functions
         - Improve naming if it drifted during implementation
         - Run `cargo test -p {crate}` (full crate, not just current
           tests) to verify refactoring didn't break existing tests

RULE D6 (Next Group): Move to the next case group (next operation)
         and repeat from RULE D1.
```

### 6.3 Post-Implementation Rules

These rules apply after all case groups for a task are complete.

```
RULE E1 (Full Crate Test): Run `cargo test -p {crate}` to verify
         all tests pass, not just the ones written in this task.

RULE E2 (Clippy Clean): Run `cargo clippy -p {crate}` and ensure
         zero warnings. Fix any new warnings introduced.

RULE E3 (Coverage Check): If CASES.md exists, verify every must-priority
         case ID has a corresponding test function. List any gaps.

RULE E4 (No Dead Code): Verify that all code paths in the
         implementation are exercised by at least one test. Look for
         match arms, if-branches, and functions that no test reaches.
         Remove or flag unreachable code.

RULE E5 (Format): Run `cargo +nightly fmt --all` to ensure consistent
         formatting.
```

### 6.4 Per-Task TDD Cycle Definition

One complete TDD cycle for an AI agent working on a task in this project:

```
TASK: "Implement create_session usecase"
Cases from CASES.md: CreateSession.S1, CreateSession.F1, CreateSession.F2, CreateSession.E1

CYCLE:
  1. [STUBS]   Create/verify domain types + port traits compile
                -> cargo check -p auth
  2. [RED]     Write tests for CreateSession.S1, F1, F2, E1
                -> cargo test -p auth -- create_session (expect: all fail)
  3. [GREEN]   Implement create_session() usecase function
                -> cargo test -p auth -- create_session (expect: all pass)
  4. [REFACTOR] Address clippy warnings, extract helpers, clean naming
                -> cargo clippy -p auth
                -> cargo test -p auth (full crate, verify no regressions)
  5. [VERIFY]  Cross-reference test functions against case IDs
                -> All must-priority cases covered? Yes -> DONE
```

### 6.5 Layer-Specific TDD Rules

| Layer | Test Type | TDD Cycle | When in Task |
|-------|-----------|-----------|-------------|
| **domain/types/** | Behavioral tests only if type has logic (e.g., `can_manage()`, validation) | Write test -> implement method -> verify | Early in task, before usecase tests need the method |
| **domain/ports/** | Mock compilation test (verify automock works) | Write trait -> verify `MockXxx` compiles | Part of stub phase |
| **usecase/** | Unit tests with mocked ports | Full Red-Green-Refactor cycle per case group | Core of every task |
| **adapter/** | Integration tests with testcontainers | Write test -> implement adapter -> verify against real DB | Separate task/wave from usecase tests |
| **app/rpc/** | Service tests with tonic in-process | Write test -> implement handler -> verify gRPC mapping | Separate task/wave from usecase tests |

### 6.6 Rules Injection Format

The following is the condensed rule set suitable for injection into an executor agent's system prompt or CLAUDE.md:

```markdown
## TDD Rules for Executor

### Cycle: Stub -> Red -> Green -> Refactor (per case group)

1. **Stub**: If domain types/ports don't exist, create minimal stubs.
   Verify: `cargo check -p {crate}`

2. **Red**: Write ALL tests for one operation's cases (S/F/E from CASES.md).
   Tests must: name behavior (`should_X_when_Y`), assert outputs not internals,
   use factory functions for test data.
   Verify: `cargo test -p {crate} -- {prefix}` -- ALL must FAIL.
   If any passes unexpectedly, the test is not testing behavior.

3. **Green**: Implement ONLY what failing tests require.
   Do NOT add untested error branches, speculative fields, or unused methods.
   Verify: `cargo test -p {crate} -- {prefix}` -- ALL must PASS.

4. **Refactor**: `cargo clippy -p {crate}`, extract duplicates, improve names.
   Verify: `cargo test -p {crate}` (full crate, not just current tests).

### Completion: After all case groups for a task
- `cargo test -p {crate}` (full crate green)
- `cargo clippy -p {crate}` (zero warnings)
- `cargo +nightly fmt --all`
- Cross-reference tests against CASES.md must-priority case IDs

### Prohibited Behaviors
- Writing test and implementation in the same edit
- Skipping Red verification (running tests to see them fail)
- Adding code not exercised by any test
- Using `times()` for interaction verification unless call-count is business logic
- Writing adapter/handler code during usecase unit test tasks
```

## 7. Common Pitfalls

### Pitfall 1: TDD Procedural Instructions Without Context

**What goes wrong:** Adding verbose TDD instructions to the agent prompt without specifying which tests to check or which behavior to test increases regressions rather than reducing them.
**Why it happens:** Verbose instructions consume context tokens, pushing out repository context the agent needs for accurate changes.
**How to avoid:** Keep TDD rules concise. Provide specific case IDs and test names, not generic TDD philosophy.
**Warning signs:** Agent generates tests for behaviors not in the plan. Agent repeats TDD steps mechanically without testing meaningful behavior.

### Pitfall 2: Testing Mock Configuration Instead of Behavior

**What goes wrong:** Tests verify that mocks were called with specific arguments rather than verifying output behavior.
**Why it happens:** Agent treats `expect_*().withf()` as the primary assertion mechanism.
**How to avoid:** Use `returning()` for mock behavior setup. Use `assert!()` / `assert_eq!()` / `assert!(matches!())` for test assertions. Reserve `withf()` for when input validation is the behavior under test.
**Warning signs:** Tests have no assertions outside mock expectations. Test failure messages reference mock panics, not assertion failures.

### Pitfall 3: Cascading Compilation Failures

**What goes wrong:** Agent writes tests referencing types that don't exist, gets a wall of compilation errors, then tries to fix them all at once and introduces more errors.
**Why it happens:** Agent skips the Stub phase and goes directly to writing tests.
**How to avoid:** Always create type/trait stubs BEFORE writing tests. Verify stubs compile with `cargo check`. Only then write tests.
**Warning signs:** Multiple rounds of "fix compile error, introduce new compile error." Agent adds unrelated `use` statements or type modifications to silence errors.

### Pitfall 4: Over-Scoped Refactoring

**What goes wrong:** During the Refactor phase, agent restructures code beyond what tests cover, introducing bugs in untested paths.
**Why it happens:** Agent optimistically refactors "while it's there" without adequate test coverage.
**How to avoid:** Refactoring scope must be limited to code paths with test coverage. If refactoring would touch untested code, add tests first.
**Warning signs:** Refactoring touches files not in the current task scope. New code appears during refactoring that wasn't exercised by existing tests.

### Pitfall 5: The "assert!(result.is_ok())" Trap

**What goes wrong:** Tests assert only that the result is Ok/Err without checking the actual value, providing weak behavioral guarantees.
**Why it happens:** Agent takes the shortest path to a "passing" assertion.
**How to avoid:** After `assert!(result.is_ok())`, always unwrap and verify the returned value's key fields. After `assert!(matches!(result, Err(..)))`, verify the specific error variant.
**Warning signs:** Tests have single-line assertions that don't inspect the return value. Multiple tests with identical assertion patterns that differ only in setup.

## 8. Open Questions

### Q1: Optimal Strictness Level for /test-gen Skill

**What we know:** /test-gen is planned to generate failing test skeletons from CASES.md before execution begins. This is the ideal Red phase -- tests exist before any implementation.
**What's unclear:** Should /test-gen generate tests that compile (requiring stubs) or tests that don't compile (requiring the executor to create stubs first)?
**Recommendation:** /test-gen should generate compilable tests with stubs. This front-loads the compilation validation and lets the executor focus on Green + Refactor. Creating stubs is a mechanical task better handled by /test-gen's analysis of CASES.md interfaces.

### Q2: Refactoring Trigger Automation

**What we know:** Agents skip refactoring without explicit rules. Clippy catches some issues.
**What's unclear:** Can refactoring quality be automated beyond clippy? Should the agent be given a refactoring checklist, or is clippy + test coverage sufficient?
**Recommendation:** Start with clippy + explicit checklist (extract helpers, check naming, remove duplication). Evaluate after a few phases whether additional automation is needed.

### Q3: Integration Test TDD Cadence

**What we know:** Unit tests (usecase layer) follow the case-group TDD cycle well. Integration tests (adapter layer) with testcontainers have a 5-10 second startup overhead.
**What's unclear:** Should integration tests follow the same Red-Green-Refactor cycle, or is a "write adapter, verify against tests" approach more practical?
**Recommendation:** Integration tests should follow a lighter cycle: write tests first (Red), implement adapter (Green), but skip the per-case-group granularity. Test all adapter operations in one cycle because testcontainer startup dominates the time cost.

## Sources

### Primary (HIGH confidence)
- [Agentic Engineering Patterns - Simon Willison](https://simonwillison.net/guides/agentic-engineering-patterns/red-green-tdd/) -- Red/Green TDD as highest-leverage pattern for AI agents
- [TDAD Paper (arXiv:2603.17973)](https://arxiv.org/abs/2603.17973) -- Test-Driven Agentic Development, regression reduction by 70%, TDD-only prompting paradox
- [Agentic Coding Handbook - Tweag](https://tweag.github.io/agentic-coding-handbook/WORKFLOW_TDD/) -- TDD workflow for agentic coding, tests as prompts
- [TDD Guard (GitHub)](https://github.com/nizos/tdd-guard) -- Automated TDD enforcement hooks for Claude Code
- Project codebase: `services/user/src/usecase/create_user.rs`, `services/user/tests/` -- Established test patterns

### Secondary (MEDIUM confidence)
- [The 80% Problem in Agentic Coding - Addy Osmani](https://addyo.substack.com/p/the-80-problem-in-agentic-coding) -- Over-implementation, assumption propagation, comprehension debt
- [The Compiler Is the Harness - Adam Benenson](https://medium.com/@ashbenen/the-compiler-is-the-harness-why-agentic-coding-works-so-well-in-rust-730bca7faf8e) -- Rust compiler as AI agent feedback loop
- [Why TDD Works Well in AI-assisted Programming - Jason Gorman](https://codemanship.wordpress.com/2026/01/09/why-does-test-driven-development-work-so-well-in-ai-assisted-programming/) -- Prevents agents from writing tests that verify broken behavior
- [Agentic TDD - Nizar](https://nizar.se/agentic-tdd/) -- Practical rules: one test at a time, single assertion, fail for correct reason
- [Making AI Coding Agents Follow True TDD - brgr.one](https://www.brgr.one/blog/ai-coding-agents-tdd-enforcement) -- Enforcement mechanisms, cost tradeoffs
- [Test-Driven Development with AI - Builder.io](https://www.builder.io/blog/test-driven-development-ai) -- TDD slog for humans = perfect for AI
- [TDD Best Practices for AI Agents - qaskills.sh](https://qaskills.sh/blog/tdd-ai-agents-best-practices) -- Start with high-value behavior, keep test scopes tight
- [Test-Driven Development Ideal for AI - The Register](https://www.theregister.com/2026/02/20/from_agile_to_ai_anniversary/) -- Agile workshop consensus on TDD + AI

### Tertiary (LOW confidence)
- [Guardrails for Agentic Coding - jvaneyck](https://jvaneyck.wordpress.com/2026/02/22/guardrails-for-agentic-coding-how-to-move-up-the-ladder-without-lowering-your-bar/) -- Hook-based enforcement mechanisms
- [Better AI Driven Development with TDD - Eric Elliott](https://medium.com/effortless-programming/better-ai-driven-development-with-test-driven-development-d4849f67e339) -- RITE Way assertion pattern, 5 essential test questions
- [Stop AI Agents from Writing Spaghetti - yuv.ai](https://yuv.ai/blog/superpowers) -- Superpowers framework for TDD enforcement

## Metadata

**Confidence breakdown:**
- AI agent failure modes: HIGH -- multiple independent sources confirm identical failure modes
- Rust-specific TDD patterns: HIGH -- verified against project codebase and official Rust docs
- Guardrail mechanisms: MEDIUM-HIGH -- emerging consensus but limited Rust-specific tooling
- Execution cadence: MEDIUM-HIGH -- based on project measurements and community practice
- Proposed behavioral rules: MEDIUM -- synthesized from research, not yet validated in production use

**Research date:** 2026-03-24
**Valid until:** 2026-06-24 (3 months -- this is a fast-moving area with new tools appearing monthly)
