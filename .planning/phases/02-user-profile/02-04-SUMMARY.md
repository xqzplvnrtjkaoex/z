---
phase: 02-user-profile
plan: "04"
subsystem: api
tags: [axum, grpc, tonic, testcontainers, sea-orm, gateway, user-routes, integration-tests]

# Dependency graph
requires:
  - phase: 02-user-profile/02-01
    provides: proto definitions (UserService RPCs, Role enum), domain types, UserRepository port trait
  - phase: 02-user-profile/02-02
    provides: PostgresUserRepository, UserContext, adapter layer
  - phase: 02-user-profile/02-03
    provides: UserHandler, all 8 gRPC RPCs implemented (create, get, get-by-handle, list, update, deactivate, activate, change-role)

provides:
  - "Gateway REST routes for user operations: GET/PATCH /v1/users/@me (self-service) and GET/POST/PATCH /v1/users (admin management)"
  - "UserJson response type with RFC3339 timestamps, role string conversion"
  - "Integration tests for PostgresUserRepository: 8 tests proving correctness against real PostgreSQL via testcontainers"
  - "Service tests for UserHandler via tonic in-process gRPC: 6 tests proving full request/response cycle"
  - "Migration bug fix: user_role PostgreSQL enum created with correct name"

affects:
  - 03-authentication (gateway user routes will be protected by JWT middleware in Phase 3)
  - E2E tests (user REST endpoints now accessible via gateway for scenario-based tests)

# Tech tracking
tech-stack:
  added:
    - chrono in gateway dependencies (timestamp formatting for UserJson)
  patterns:
    - "Gateway REST-to-gRPC translation: extract from HTTP headers/path/body, build tonic::Request, inject caller context via metadata, map gRPC response to JSON"
    - "Per-test fresh DatabaseConnection against shared testcontainers container: avoids pool exhaustion across independent tokio runtimes"
    - "inject_caller_context helper: copies X-Caller-Id/X-Caller-Role HTTP headers into tonic gRPC metadata"
    - "X-Next-Cursor response header for cursor-based pagination in list_users handler"
    - "user_routes() fn with nested Router: me_routes (self-service) and admin_routes merged into /v1/users"

key-files:
  created:
    - services/gateway/src/routes/users.rs
    - services/user/tests/user_repository_integration.rs
    - services/user/tests/user_service_test.rs
  modified:
    - services/gateway/src/routes/mod.rs
    - services/gateway/Cargo.toml
    - services/user/migration/m20260322_000001_create_users_table.rs

key-decisions:
  - "Per-test fresh DatabaseConnection vs shared pool: create new DatabaseConnection per test against shared container URL to prevent pool exhaustion across independent tokio runtimes"
  - "Migration user_role enum via raw SQL: DeriveIden generates 'user_role_enum' from UserRoleEnum::Table but sea-orm entity expects 'user_role'; use execute_unprepared raw SQL for correctness"
  - "Alias::new('user_role') for column type in create_table: sea-query Alias implements IntoIden providing correct type reference after enum creation fix"

patterns-established:
  - "Gateway user_routes(): separate me_routes (self-service) and admin_routes merged via Router::merge() — placeholder for Phase 3 middleware tiers"
  - "TestContainer struct: holds ContainerAsync (to keep alive) + url String; each test opens fresh DatabaseConnection via Database::connect(&tc.url)"

requirements-completed: [USER-PROFILE-01, USER-PROFILE-02]

# Metrics
duration: 15min
completed: 2026-03-22
---

# Phase 2 Plan 04: Gateway Routes and Tests Summary

**Gateway REST user endpoints (7 routes), 8 PostgreSQL integration tests, and 6 gRPC in-process service tests proving the full user service stack end-to-end.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-03-22T12:42:14Z
- **Completed:** 2026-03-22T12:57:14Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Gateway exposes all 7 user REST endpoints: GET/PATCH `/v1/users/@me` (self-service) and GET `/v1/users`, GET `/v1/users/:id`, PATCH `/v1/users/:id/role`, POST `/:id/deactivate`, POST `/:id/activate` (admin)
- Integration tests prove `PostgresUserRepository` correctness against real PostgreSQL: case-insensitive handle uniqueness, pagination, active-only filtering, full CRUD
- Service tests prove the full gRPC handler chain works: create/get user, owner-role rejection, role change with caller context, lifecycle (deactivate/activate), pagination, not-found
- Fixed pre-existing migration bug: PostgreSQL enum type `user_role` was being created as `user_role_enum` due to DeriveIden naming, causing all INSERT operations to fail

## Task Commits

1. **Task 1: Gateway REST routes for user operations** - `42e6137` (feat)
2. **Task 2: Integration tests, service tests, migration fix** - `8df1aa9` (test)

## Files Created/Modified

- `services/gateway/src/routes/users.rs` - All 7 user REST route handlers with REST-to-gRPC translation; UserJson response type; inject_caller_context helper
- `services/gateway/src/routes/mod.rs` - Updated to register user routes under /v1/users with me_routes and admin_routes
- `services/gateway/Cargo.toml` - Added chrono dependency for timestamp formatting
- `services/user/tests/user_repository_integration.rs` - 8 integration tests for PostgresUserRepository against real PostgreSQL via testcontainers
- `services/user/tests/user_service_test.rs` - 6 service tests for UserHandler via tonic in-process gRPC channel
- `services/user/migration/m20260322_000001_create_users_table.rs` - Fixed enum type creation to use raw SQL for correct "user_role" name; Alias::new for column type reference

## Decisions Made

- **Per-test fresh DatabaseConnection:** Multiple `#[tokio::test]` tests each run in their own tokio runtime. Sharing a single `DatabaseConnection` across runtimes via `OnceCell` causes connection pool exhaustion when the first runtime completes. Fix: store only the URL in the static, open a fresh connection per test via `Database::connect(&tc.url)`.
- **Migration enum type via raw SQL:** `DeriveIden` on `UserRoleEnum` generates identifier `"user_role_enum"` from the enum name, not `"user_role"`. The sea-orm entity has `#[sea_orm(enum_name = "user_role")]` expecting the type to be named `"user_role"`. Using `execute_unprepared("CREATE TYPE \"user_role\" AS ENUM ...")` ensures the correct name.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed migration: user_role enum type name mismatch**
- **Found during:** Task 2 (integration tests)
- **Issue:** `DeriveIden` on `enum UserRoleEnum` generates identifier `"user_role_enum"` for the `Table` variant (sea-query uses the enum name). Sea-orm entity has `#[sea_orm(enum_name = "user_role")]` expecting type `"user_role"`. All INSERT operations failed with `type "user_role" does not exist`.
- **Fix:** Replaced `manager.create_type(...)` with `execute_unprepared("CREATE TYPE \"user_role\" AS ENUM ('owner', 'admin', 'user')")`. Replaced `UserRoleEnum::Table` in `ColumnDef::custom()` with `Alias::new("user_role")`. Removed unused `UserRoleEnum` definition.
- **Files modified:** services/user/migration/m20260322_000001_create_users_table.rs
- **Verification:** All 8 integration tests pass with real PostgreSQL.
- **Committed in:** `8df1aa9` (Task 2 commit)

**2. [Rule 1 - Bug] Fixed test infrastructure: per-test DB connection to prevent pool exhaustion**
- **Found during:** Task 2 (integration tests, second attempt after migration fix)
- **Issue:** Tests sharing a single `static OnceCell<TestDb { db: DatabaseConnection }>` across independent `#[tokio::test]` runtimes caused pool timeouts. When the first runtime completed, the pool associated with that runtime began being dropped, and subsequent tests on different runtimes failed with "Failed to acquire connection from pool: Connection pool timed out".
- **Fix:** Changed from storing `DatabaseConnection` in static to storing the connection URL string. Each test creates its own fresh connection via `Database::connect(&tc.url)`. Container stays alive via `_container` field.
- **Files modified:** services/user/tests/user_repository_integration.rs, services/user/tests/user_service_test.rs
- **Verification:** All 8 integration tests and 6 service tests pass with parallel test execution.
- **Committed in:** `8df1aa9` (Task 2 commit)

**3. [Rule 1 - Bug] Fixed clippy: redundant Ok() wrapping in list_users handler**
- **Found during:** Task 1 verification (cargo clippy)
- **Issue:** `Ok(resp.body(...).map_err(...)?)` is redundant (needless_question_mark warning).
- **Fix:** Removed the `Ok()` wrapper and `?` operator; returned the `Result<Response, AppError>` directly.
- **Files modified:** services/gateway/src/routes/users.rs
- **Verification:** `cargo clippy --workspace` passes clean.
- **Committed in:** `8df1aa9` (Task 2 commit)

---

**Total deviations:** 3 auto-fixed (all Rule 1 — bugs)
**Impact on plan:** All fixes were necessary for correctness. The migration bug would have blocked all integration tests. The connection pool fix was needed for stable test runs. The clippy fix is mandatory for CI. No scope creep.

## Issues Encountered

- OrbStack (local Docker daemon) was not running at test execution time. Required starting OrbStack to enable testcontainers. Remote Docker via SSH is not supported by testcontainers library.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 2 (User Profile) is fully complete: domain, usecases, adapter, gRPC handler, gateway REST routes, and tests all delivered
- Phase 3 (Authentication) can begin: it will add JWT middleware to protect the user routes added here
- Phase 4 (Catalog Core) can begin in parallel with Phase 3 (both depend only on Phase 1)
- The migration bug fix ensures testcontainers-based tests will work correctly in Phase 3 and Phase 4

## Self-Check: PASSED

- services/gateway/src/routes/users.rs: FOUND
- services/gateway/src/routes/mod.rs: FOUND (updated)
- services/user/tests/user_repository_integration.rs: FOUND
- services/user/tests/user_service_test.rs: FOUND
- Commit 42e6137 (Task 1): FOUND
- Commit 8df1aa9 (Task 2): FOUND

---
*Phase: 02-user-profile*
*Completed: 2026-03-22*
