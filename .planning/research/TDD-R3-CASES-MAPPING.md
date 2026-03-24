# TDD Research 3: CASES.md to Test Mapping Strategy

**Researched:** 2026-03-24
**Domain:** CASES.md -> PLAN.md -> executor test derivation pipeline
**Confidence:** HIGH (based on existing codebase patterns and concrete artifacts)

---

## Summary

This research analyzes how CASES.md behavioral cases (S/F/E) should map to per-task tests during TDD execution in this project's Rust workspace. The analysis is grounded in three concrete data sources: (1) the existing user service test code (the only service with complete tests), (2) the PLAN.md task format already in use across phases 01-03, and (3) the TDD-CASE-DISCOVERY.md research on ZOMBIES/TPP ordering heuristics.

**Key finding:** The current codebase already demonstrates a mature test mapping pattern -- but it evolved without CASES.md. PLAN.md tasks use a `<behavior>` section that lists test cases as prose sentences (e.g., "Test: create_user with valid handle/name/role=User succeeds"). These behavior items map 1:1 to test functions in the implementation. The missing piece is connecting CASES.md case IDs to these behavior items, so the executor can trace requirements through to green tests.

**Primary recommendation:** CASES.md case IDs should appear in PLAN.md `<behavior>` items as trailing annotations (e.g., `- Test: create_user with role=Owner returns Err(OwnerRoleRejected) [CreateUser.F1]`). The executor writes one test function per behavior item, following the codebase's established `should_*` naming convention. ZOMBIES ordering (Z->O->M) should guide the order of test writing within a task, not the numerical order of case IDs.

---

## 1. Current CASES.md <-> PLAN.md Linkage

### Status: No Linkage Exists Yet

**Finding (HIGH confidence):** No CASES.md files exist in any phase directory. Phases 01, 02, and 03 were all planned without the /case skill.

```
.planning/phases/01-foundation-and-gateway-infrastructure/ -- no CASES.md
.planning/phases/02-user-profile/                          -- no CASES.md
.planning/phases/03-authentication/                        -- no CASES.md
```

A grep for case ID patterns (e.g., `.S1`, `.F1`, `.E1`) across all PLAN.md files returned zero matches. The acceptance_criteria sections contain file-content assertions ("services/user/src/domain/types/role.rs contains `pub enum UserRole`") rather than case references.

### Current Behavior Item Pattern

PLAN.md tasks with `tdd="true"` use a `<behavior>` section that lists test cases as prose:

```xml
<behavior>
  - Test: create_user with valid handle/name/role=User succeeds and returns User with UUIDv4 id
  - Test: create_user with role=Owner returns Err(UserError::OwnerRoleRejected) per D-60
  - Test: create_user with invalid handle returns Err(UserError::InvalidHandle)
  ...
</behavior>
```

Each behavior item maps to exactly one `#[tokio::test]` or `#[test]` function in the implementation. The mapping is 1:1 and consistent across all observed tasks.

### How CLAUDE.md Specifies the Linkage (Aspirational)

CLAUDE.md section "CASES.md Integration with Plan-Phase" specifies:

> Map must-priority cases to required test tasks in PLAN.md. Reference case IDs as `OperationName.S1` in task `acceptance_criteria` (IDs restart per operation).

This is the intended design. It has not been exercised yet because no phase has used /case.

### Gap Analysis

| What Exists | What Is Specified | What Is Missing |
|-------------|-------------------|-----------------|
| `<behavior>` items in PLAN.md | Case IDs in `acceptance_criteria` | The actual linkage format and semantics |
| Test functions named `should_*` | Case ID traceability | Mapping from case ID to test function name |
| D-number references (e.g., "per D-60") | `OperationName.S1` references | Clear rules for which PLAN.md field carries case IDs |

---

## 2. Per-Task Test Derivation Strategy

### Recommended Mapping: 1 Case = 1 Test Function (with exceptions)

Based on the existing codebase pattern, each CASES.md case should produce exactly one test function in the implementation. This is already the natural grain observed in `create_user.rs` tests:

| Case Description (hypothetical) | Test Function |
|--------------------------------|---------------|
| CreateUser.S1: valid input succeeds | `should_create_user_with_valid_handle_name_and_role_user` |
| CreateUser.F1: owner role rejected | `should_reject_owner_role_with_owner_role_rejected_error` |
| CreateUser.F2: invalid handle | `should_reject_invalid_handle_with_invalid_handle_error` |
| CreateUser.F3: reserved handle | `should_reject_reserved_handle_with_handle_reserved_error` |
| CreateUser.F4: invalid name | `should_reject_invalid_name_with_invalid_name_error` |
| CreateUser.F5: handle already taken | `should_return_handle_taken_when_handle_already_exists` |

**Exceptions -- parameterized groups:** When CASES.md identifies a parameterized group (e.g., "handle validation: too short, too long, invalid chars, reserved -- all INVALID_ARGUMENT"), the group MAY map to a single test function with multiple assertions (as seen in `handle_input.rs::should_reject_handle_with_non_alphanumeric_or_underscore_characters` which tests three inputs). But each distinct behavioral outcome should still be a separate test.

### Test Naming Convention

The codebase uses `should_*` BDD-style naming. Test names should NOT include case IDs.

**Correct:**
```rust
#[tokio::test]
async fn should_reject_owner_role_with_owner_role_rejected_error() { ... }
```

**Incorrect:**
```rust
#[tokio::test]
async fn test_create_user_f1_owner_role_rejected() { ... }  // case ID in test name
```

**Rationale:** Case IDs are planning artifacts. Test names describe behavior. If case IDs appear in test names, renumbering cases during /case iteration forces test renames. The traceability lives in PLAN.md's `<behavior>` items, not in code. This also aligns with the project's rust-documentation.md rule: "External numbering references: `// D-12: ...`, `// REQ-03: ...` -- design decisions live in `.planning/`, not in code."

### Traceability Through PLAN.md

The linkage chain should be:

```
CASES.md case ID  -->  PLAN.md <behavior> item  -->  test function name
   CreateUser.F1       "Test: ...Owner... [CreateUser.F1]"    should_reject_owner_role_...
```

The `<behavior>` item is the bridge. It carries both the human-readable description and the case ID annotation. The executor reads the behavior item to know what test to write.

### Handling Tasks with 5+ Cases

When a task references many cases (e.g., `create_user` usecase with 6 behavior items), the executor should:

1. **Write all tests first (red phase).** All behavior items for the task become test function stubs that compile but fail. This is the TDD approach -- see all the failing tests, then make them pass one at a time.

2. **Implement in ZOMBIES order (not case-ID order).** The simplest success case first (S1), then simple failures (F1, F2...), then edge cases (E1, E2...). Within each category, follow Zero->One->Many progression.

3. **Each test should be independently runnable.** No test should depend on another test's state. This is already the pattern in the codebase (each test creates its own `TestContext` with a fresh mock).

**Practical consideration for the executor:** Writing 6+ stubs at once is only viable if the type signatures are known. For the first task in a new service (where types don't exist yet), the executor writes one test, implements enough types to make it compile (but fail), then adds the next test. The stubs-first approach works best for usecase tasks where the domain types already exist from a prior task.

---

## 3. Tasks Without Case References

### Taxonomy of Case-Less Tasks

Based on analysis of all PLAN.md files across phases 01-03, tasks fall into these categories:

| Task Type | Example | Has Cases? | Test Type |
|-----------|---------|------------|-----------|
| Proto definition | "Define user.proto with 8 RPCs" | No | Compilation check only |
| Workspace/dependency setup | "Add sea-orm to Cargo.toml" | No | Compilation check only |
| Domain types/ports | "Create UserRole, User, UserRepository trait" | Yes (unit) | `#[cfg(test)]` in same file |
| Usecase logic | "Implement create_user" | Yes (unit) | `#[cfg(test)]` in same file |
| Adapter (repository) | "Implement PostgresUserRepository" | Yes (integration) | `tests/` directory |
| gRPC handler | "Implement UserHandler" | No (thin layer) | Covered by service tests |
| Service wiring | "Wire handler with context" | Yes (service) | `tests/` directory |
| E2E/contract | "Gateway integration tests" | Yes (e2e) | `tests/` directory |
| Infrastructure | "docker-compose, justfile" | No | Manual verification |
| Migration | "Create users table migration" | No | Tested via integration tests |

### Recommendation for Case-Less Tasks

**Proto/setup/infrastructure tasks:** No tests. Verification is `cargo build` or `cargo check`. These tasks have `<verify><automated>cargo build -p foo</automated></verify>` blocks. CASES.md should not attempt to cover them -- they are structural, not behavioral.

**gRPC handler tasks (app/ layer):** Currently no dedicated tests. The handler is a thin translation layer (extract proto fields -> call usecase -> map result to proto response). It is tested indirectly by service-level tests (`user_service_test.rs`). This is consistent with the project's 4-layer architecture where the app layer "delegates to usecase; trait docs cover the contract."

**Migration tasks:** Tested indirectly when integration tests run migrations on testcontainers. No separate migration tests needed.

**The executor's rule:** If a PLAN.md task has a `<behavior>` section, write tests. If it has `tdd="true"`, write tests FIRST. If it has neither, the task is structural -- verify with `cargo build`/`cargo check` and move on.

---

## 4. Test Layer Selection

### Layer Selection Matrix

| Task Layer | Test Layer | Test Location | Mock Strategy | Example |
|------------|-----------|---------------|---------------|---------|
| `domain/types/` | Unit | `#[cfg(test)] mod tests` in same file | No mocks needed (pure logic) | `role.rs::tests`, `handle_input.rs::tests` |
| `domain/input/` | Unit | `#[cfg(test)] mod tests` in same file | No mocks needed (validation) | `handle_input.rs::tests` |
| `usecase/` | Unit | `#[cfg(test)] mod tests` in same file | Mock ports via `MockUserRepository` | `create_user.rs::tests` |
| `adapter/postgres/` | Integration | `tests/` directory | testcontainers PostgreSQL | `user_repository_integration.rs` |
| `adapter/redis/` | Integration | `tests/` directory | testcontainers Redis | (not yet implemented) |
| `app/rpc/` | Service | `tests/` directory | testcontainers + in-process gRPC | `user_service_test.rs` |
| Gateway routes | E2E | `tests/` directory | testcontainers + HTTP client | `health_integration.rs` |

### Who Specifies the Layer?

**PLAN.md should specify the test layer implicitly through task structure.** The executor infers the correct layer from:

1. **The `<files>` list:** If files are in `usecase/`, write unit tests with mocks. If files are in `adapter/`, write integration tests with testcontainers.

2. **The `<behavior>` items:** If behavior items mention "returns Err(UserError::...)", that is a unit test. If they mention "via gRPC", that is a service test.

3. **The `tdd="true"` flag:** Currently only applied to domain, usecase, and adapter tasks. Infrastructure tasks never have this flag.

**CASES.md should NOT specify the test layer.** CASES.md is technology-neutral (per the project's feedback: "/case skill must be technology-neutral, no REST/gRPC/proto assumptions"). The case `CreateUser.F1: owner role rejected` does not know whether it will be tested at the unit, service, or E2E level. That decision belongs to PLAN.md.

**However, PLAN.md should be explicit about which test file to create.** The existing plans already do this well -- the `<files>` section lists test files when they are expected outputs:

```xml
<files>
  services/gateway/tests/auth_integration.rs,
  services/gateway/Cargo.toml
</files>
```

### Layer Stacking: Same Case, Multiple Layers

A single CASES.md case (e.g., `CreateUser.S1: valid input succeeds`) can be tested at multiple layers:

- **Unit (usecase):** `create_user.rs::should_create_user_with_valid_handle_name_and_role_user` -- mocked repo, verifies business logic
- **Service:** `user_service_test.rs::should_create_and_get_user_via_grpc` -- real DB, verifies gRPC contract
- **E2E:** (future) -- real DB + gateway, verifies REST contract

The planner should decide which layers test which cases. A reasonable default:

| Case Priority | Layers |
|--------------|--------|
| must-priority S/F cases | Unit + at least one higher layer |
| should-priority S/F cases | Unit only |
| E (edge) cases | Unit only (unless concurrency-related) |

---

## 5. Rust Test Placement

### Established Patterns in This Codebase

**Unit tests: `#[cfg(test)] mod tests` in the same file.**

This is the pattern used consistently for domain types and usecase functions:

```
services/user/src/domain/types/role.rs         -- has #[cfg(test)] mod tests
services/user/src/domain/input/handle_input.rs  -- has #[cfg(test)] mod tests
services/user/src/domain/input/name_input.rs    -- has #[cfg(test)] mod tests
services/user/src/domain/error/user_error.rs    -- has #[cfg(test)] mod tests
services/user/src/domain/ports/user_repository.rs -- has #[cfg(test)] mod tests
services/user/src/usecase/create_user.rs        -- has #[cfg(test)] mod tests
services/user/src/usecase/get_user.rs           -- has #[cfg(test)] mod tests
... (all 8 usecase files)
```

**Integration and service tests: `tests/` directory at crate root.**

```
services/user/tests/user_repository_integration.rs  -- adapter tests with testcontainers
services/user/tests/user_service_test.rs             -- service tests with in-process gRPC
services/gateway/tests/health_integration.rs         -- E2E tests
```

### Test File Naming Conventions

| Layer | Location | Naming Pattern |
|-------|----------|----------------|
| Unit (domain/usecase) | Same file, `mod tests` | N/A (inline) |
| Integration (adapter) | `tests/{entity}_repository_integration.rs` | Entity + "repository_integration" |
| Service (gRPC) | `tests/{entity}_service_test.rs` | Entity + "service_test" |
| E2E (Gateway) | `tests/{feature}_integration.rs` | Feature + "integration" |

### Idiomatic Rust Approach

The project follows standard Rust conventions:

1. **Unit tests colocated with code** (`#[cfg(test)]`): Tests have access to private items. This is the Rust-recommended approach and is used for all domain and usecase tests.

2. **Integration tests in `tests/` directory**: Each file is compiled as a separate crate, so it can only access the crate's public API. This naturally enforces integration-level testing (no access to internals).

3. **Shared test helpers**: The existing tests use `OnceCell<TestContainer>` for container sharing within a test file, and helper functions like `make_db()` and `make_test_user()`. There is no shared test utility crate yet -- each test file is self-contained.

### Test Infrastructure Patterns

**Mock pattern (unit tests):**
```rust
struct TestContext {
    user_repo: MockUserRepository,
}

impl UserPorts for TestContext {
    fn user_repo(&self) -> &impl UserRepository {
        &self.user_repo
    }
}
```

**Testcontainers pattern (integration/service tests):**
```rust
static TEST_CONTAINER: OnceCell<TestContainer> = OnceCell::const_new();

async fn make_db() -> DatabaseConnection {
    let tc = TEST_CONTAINER.get_or_init(|| async { ... }).await;
    Database::connect(&tc.url).await.expect("...")
}
```

**In-process gRPC pattern (service tests):**
```rust
async fn start_service() -> UserServiceClient<tonic::transport::Channel> {
    let db = make_db().await;
    // ... setup handler and server on random port
    UserServiceClient::new(channel)
}
```

---

## 6. ZOMBIES Heuristic Integration

### ZOMBIES vs CASES.md Ordering

CASES.md organizes cases by category: S (Success), F (Failure), E (Edge). Within each category, cases are numbered sequentially (S1, S2, F1, F2, ...).

ZOMBIES organizes by complexity progression: Zero (simplest) -> One (first meaningful) -> Many (generalization), with BIE (Boundary, Interface, Exception) as cross-cutting dimensions.

These are **different axes** and they do NOT conflict:

| CASES.md Category | Typical ZOMBIES Mapping |
|-------------------|------------------------|
| S1 (simplest success) | Z/O -- Zero or One level success |
| S2, S3 (richer success) | M -- Many level, or additional One variations |
| F1, F2 (simple failures) | E at Z/O level -- exceptions for degenerate/simple cases |
| F3, F4 (complex failures) | E at M level -- exceptions for multi-item/state-dependent cases |
| E1, E2 (edge cases) | B at Z/O/M levels -- boundary behaviors |

### Recommendation: Use ZOMBIES Ordering for Test Writing, CASES.md Ordering for Traceability

Within a single PLAN.md task, the executor should:

1. **List tests in CASES.md order** in the `<behavior>` section (for traceability -- planner maps cases to behavior items in the same order they appear in CASES.md).

2. **Write tests in ZOMBIES order** during execution (for TDD effectiveness -- simplest first, building complexity).

The reordering is natural and small. For `create_user`:

| CASES.md Order | ZOMBIES Order (write this order) |
|---------------|--------------------------------|
| S1: valid input | **1st** -- Zero/One success (the simplest thing that works) |
| S2: all fields | 3rd -- One with richer input |
| S3: minimal fields | 2nd -- One with boundary input (minimum valid) |
| F1: owner role rejected | **4th** -- first conditional failure |
| F2: invalid handle | 5th -- validation failure |
| F3: reserved handle | 6th -- validation failure (different partition) |
| F4: invalid name | 7th -- validation failure (different field) |
| F5: handle taken | **8th** -- infrastructure failure (requires mock setup) |

### When to Deviate from ZOMBIES

ZOMBIES assumes you are building from scratch. But in this codebase, many tasks operate on existing infrastructure:

- **Adapter tasks** (PostgresUserRepository): Types and ports already exist from a prior task. Start with the simplest DB operation (save + find_by_id), not with zero-state tests. ZOMBIES O->M applies to the operation set, not the system state.

- **Service tasks** (gRPC handler tests): The service already has a running stack from setup. Start with the simplest RPC call. ZOMBIES applies to the breadth of RPC coverage.

- **E2E tasks** (Gateway integration): Everything is wired. Write tests for the main flows first, then error cases. ZOMBIES applies to scenario complexity.

### ZOMBIES as Quality Check, Not Rigid Process

The primary value of ZOMBIES for the executor is as a **completeness check**, not a strict ordering rule:

After writing all tests for a task's behavior items, ask:
- **Z:** Did I test the empty/zero state? (empty request, empty system)
- **O:** Did I test the simplest success case?
- **M:** Did I test with multiple items? (pagination, lists)
- **B:** Did I test at the boundaries? (max length, limits)
- **I:** Did I verify the contract shape? (response fields, error types)
- **E:** Did I test all failure modes? (validation, auth, not-found, conflict)

If CASES.md was thorough (which is /case's job), the behavior items already cover all ZOMBIES dimensions. The executor should trust the case list and not add extra tests unless a gap is obvious.

---

## 7. /test-gen Integration Points

### How /test-gen Should Consume CASES.md

/test-gen sits between plan-phase and execute-phase. It reads CASES.md and PLAN.md to produce test skeleton files. Based on this research, the recommended design:

1. **Input:** PLAN.md `<behavior>` items with case ID annotations.

2. **Output per task type:**

   | Task Layer | /test-gen Output |
   |-----------|-----------------|
   | domain types | `#[cfg(test)] mod tests { }` block in the target file (stub) |
   | usecase | `#[cfg(test)] mod tests { }` block with test function stubs |
   | adapter | New file in `tests/` with testcontainer setup and test stubs |
   | service | New file in `tests/` with in-process gRPC setup and test stubs |
   | E2E | New file in `tests/` with HTTP client setup and test stubs |

3. **Skeleton format:** Each test function should:
   - Have the correct `should_*` name derived from the behavior item
   - Include the case ID as a code comment (acceptable per project rules since it is a planning reference inside `#[cfg(test)]` code, not production code)
   - Contain a `todo!()` or `panic!("not yet implemented")` body
   - Include the expected assertion pattern as a comment

4. **Example skeleton:**
   ```rust
   #[tokio::test]
   async fn should_reject_owner_role_with_owner_role_rejected_error() {
       // [CreateUser.F1] Create with role=Owner -> OwnerRoleRejected
       // assert!(matches!(result, Err(UserError::OwnerRoleRejected)));
       todo!("implement after CreateUserPayload and UserError exist")
   }
   ```

### Compile-But-Fail vs Compile-Error

The WORKFLOW.md says /test-gen should produce skeletons that "compile but fail." In Rust, this is nuanced:

- **`todo!()`**: Compiles, panics at runtime. Good for tests where the type signatures are known.
- **Type-not-found errors**: If the test references `UserError` but the domain types don't exist yet, the test won't compile. This is fine if the test file is in `tests/` (Cargo won't try to compile it until `cargo test` is run for that crate, and even then, it will produce a clear error).

**Recommendation:** Use `todo!()` for test bodies. For imports, use `// use crate::domain::...;` (commented-out) when the types don't exist yet, with a note that the executor should uncomment when implementing.

---

## 8. Concrete Example: Full Pipeline for a Hypothetical CASES.md

Assume phase 03-authentication had used /case and produced:

```markdown
## RegisterPasskey

### Success Cases
- S1: First-time registration with valid invite token succeeds, returns passkey credential
- S2: Registration stores credential in database, verifiable via ListPasskeys

### Failure Cases
- F1: Registration with expired invite token returns INVALID_ARGUMENT
- F2: Registration with already-used invite token returns INVALID_ARGUMENT
- F3: Registration without invite token returns UNAUTHENTICATED

### Edge Cases
- E1: Two concurrent registrations with same invite token -- exactly one succeeds
```

The planner would produce a PLAN.md task:

```xml
<task type="auto" tdd="true">
  <name>Task: Register passkey usecase</name>
  <behavior>
    - Test: register_passkey with valid invite and valid ceremony succeeds [RegisterPasskey.S1]
    - Test: register_passkey stores credential retrievable by list_passkeys [RegisterPasskey.S2]
    - Test: register_passkey with expired invite returns Err(InviteExpired) [RegisterPasskey.F1]
    - Test: register_passkey with used invite returns Err(InviteAlreadyUsed) [RegisterPasskey.F2]
    - Test: register_passkey without invite returns Err(InviteRequired) [RegisterPasskey.F3]
  </behavior>
  ...
</task>
```

The executor writes tests in ZOMBIES order:
1. S1 (Zero/One: simplest success -- the first passkey registration)
2. F3 (Zero-level exception: no invite at all -- degenerate case)
3. F1 (One-level exception: expired invite)
4. F2 (One-level exception: used invite)
5. S2 (Many: verify stored credential -- requires a second operation)

E1 (concurrency edge case) would be in a separate integration or service test task, not in the usecase unit test task.

---

## 9. Recommendations Summary

### For the Planner (PLAN.md generation)

1. **Annotate `<behavior>` items with case IDs.** Format: `- Test: description [OperationName.S1]`. This is the single point of traceability.

2. **Do NOT put case IDs in `<acceptance_criteria>`.** Acceptance criteria should remain file-content assertions and compilation checks. Case traceability lives in `<behavior>`.

3. **Group cases by test layer in separate tasks.** A usecase task gets unit-level cases. A service test task gets service-level cases. Do not mix layers in one task.

4. **Include test file path in `<files>` for non-inline tests.** If the task should produce `tests/auth_service_test.rs`, list it explicitly.

### For the Executor (test writing during TDD)

1. **One test function per behavior item.** Match the `should_*` naming convention. Do not include case IDs in test names.

2. **Write tests in ZOMBIES order** (simplest success -> degenerate failures -> richer cases -> boundaries -> complex failures). This naturally diverges from the numerical case order.

3. **For tasks with 3+ behavior items:** Write all test stubs first (`todo!()`), then implement one at a time. For tasks where types don't exist yet, write and implement incrementally.

4. **Infer test layer from task context:** Files in `usecase/` -> unit tests with mocks. Files in `adapter/` -> integration tests with testcontainers. Files in `tests/` -> service or E2E tests.

### For /test-gen (skeleton generation)

1. **Read PLAN.md `<behavior>` items** as the primary input. Each item becomes one test function skeleton.

2. **Use `todo!()` for test bodies.** Add the case ID and expected assertion as comments.

3. **Generate the correct infrastructure** per test layer: mock `TestContext` for unit tests, `OnceCell<TestContainer>` for integration tests, `start_service()` for service tests.

4. **Skeletons should compile when types exist.** If the task depends on types from a prior task in the same wave, the skeleton can reference those types. If types don't exist yet, comment out the imports.

### For /case (CASES.md generation)

1. **Case IDs should be stable within a phase.** Once assigned, avoid renumbering. Add new cases at the end (S4, F7, E3).

2. **Annotate parameterized groups.** When multiple cases follow the same pattern (e.g., handle validation: too short, too long, invalid chars), note "parameterized group" so the planner can optionally collapse them into fewer behavior items.

3. **Mark cases with recommended test layer.** While CASES.md is technology-neutral, it can note "this case requires real database" or "this case involves concurrency" which helps the planner assign cases to the right task type.

---

## Sources

### Primary (HIGH confidence)
- Existing codebase: `services/user/src/usecase/create_user.rs` -- complete unit test example with mocks
- Existing codebase: `services/user/tests/user_service_test.rs` -- complete service test example
- Existing codebase: `services/user/tests/user_repository_integration.rs` -- complete integration test example
- Existing codebase: `services/user/src/domain/input/handle_input.rs` -- domain unit test example
- Existing codebase: `services/user/src/domain/types/role.rs` -- pure logic unit test example
- `.planning/phases/02-user-profile/02-03-PLAN.md` -- TDD task with 25 behavior items
- `.planning/phases/03-authentication/03-04-PLAN.md` -- TDD task for service and E2E tests
- `.planning/WORKFLOW.md` -- CASES.md integration specification
- `.planning/research/TDD-CASE-DISCOVERY.md` -- ZOMBIES and TPP research
- `.planning/research/WORKFLOW-RESEARCH.md` -- /test-gen design considerations
- `CLAUDE.md` -- test layer definitions, TDD preference, naming conventions

### Secondary (MEDIUM confidence)
- `.planning/research/TDD-CASE-DISCOVERY.md` sections on triangulation and parameterized groups -- derived from external TDD literature, applied to this project's context

---

## Metadata

**Confidence breakdown:**
- Current linkage analysis: HIGH -- based on exhaustive search of existing artifacts
- Test derivation strategy: HIGH -- based on 13 existing test files with consistent patterns
- Layer selection: HIGH -- based on 4 distinct test layers already implemented in codebase
- ZOMBIES integration: MEDIUM -- theoretical recommendation not yet validated in practice with CASES.md
- /test-gen design: MEDIUM -- /test-gen is not yet built, recommendations are speculative

**Research date:** 2026-03-24
**Valid until:** Until /case and /test-gen are built and exercised on a real phase
