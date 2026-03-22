---
phase: 02-user-profile
plan: 03
subsystem: api
tags: [rust, tonic, grpc, mockall, trait-variant, base64, usecase, business-logic]

# Dependency graph
requires:
  - phase: 02-user-profile plan 01
    provides: Domain types, error enums, UserRepository port, validation types
  - phase: 02-user-profile plan 02
    provides: PostgresUserRepository, UserContext, adapter layer

provides:
  - 8 usecase functions with full business rule enforcement
  - UserHandler<C> implementing all 9 UserService gRPC RPCs
  - Composition root (main.rs) with DB connect, migrate, assemble, serve
  - Cursor-based pagination for ListUsers with base64 encoding
  - CallerContext extraction from gRPC metadata

affects:
  - 03-authentication (Auth calls CreateUser and GetUser RPCs)
  - gateway (routes will delegate to these RPCs)

# Tech tracking
tech-stack:
  added:
    - base64 = "0.22" (workspace dep, for cursor encoding in list_users)
  patterns:
    - "usecase free async functions with &(impl UserPorts + ?Sized) receiver"
    - "#[automock(target = UserRepository)] for Send-variant mock generation"
    - "TestContext pattern: concrete struct implementing UserPorts with MockUserRepository"
    - "CallerContext extracted from x-caller-id / x-caller-role gRPC metadata headers"
    - "App handler sub-modules: create, get, list, update, lifecycle"
    - "Composition root: main.rs assembles all layers, only place that knows concrete types"

key-files:
  created:
    - services/user/src/usecase/create_user.rs
    - services/user/src/usecase/get_user.rs
    - services/user/src/usecase/get_user_by_handle.rs
    - services/user/src/usecase/list_users.rs
    - services/user/src/usecase/update_user.rs
    - services/user/src/usecase/deactivate_user.rs
    - services/user/src/usecase/activate_user.rs
    - services/user/src/usecase/change_role.rs
    - services/user/src/usecase/mod.rs
    - services/user/src/app/handler/mod.rs
    - services/user/src/app/handler/create.rs
    - services/user/src/app/handler/get.rs
    - services/user/src/app/handler/list.rs
    - services/user/src/app/handler/update.rs
    - services/user/src/app/handler/lifecycle.rs
    - services/user/src/app/mod.rs
  modified:
    - services/user/src/lib.rs (added pub mod usecase, pub mod app)
    - services/user/src/main.rs (full composition root)
    - services/user/src/domain/ports/user_repository.rs (automock target fix)
    - services/user/src/domain/ports/mod.rs (updated import)
    - services/user/src/adapter/context.rs (updated import)
    - CLAUDE.md (added USER_DATABASE_URL env var)
    - Cargo.toml (added base64 workspace dep)
    - services/user/Cargo.toml (added base64 dep)

key-decisions:
  - "Use #[automock(target = UserRepository)] not #[automock] to generate mock implementing Send variant (UserRepository), not local variant (LocalUserRepository)"
  - "base64 crate added to workspace deps for opaque cursor encoding in list_users"
  - "CallerContext extracted from gRPC metadata headers (x-caller-id, x-caller-role) per D-23"
  - "ListUsersRequest.limit is i32 in proto, cast to u64 in usecase payload"

patterns-established:
  - "Pattern: usecase free async fn with &(impl UserPorts + ?Sized) receiver"
  - "Pattern: TestContext struct with MockUserRepository implementing UserPorts for unit tests"
  - "Pattern: CallerContext extracted in app/handler before delegating to usecase"
  - "Pattern: handler sub-modules per RPC group (create, get, list, update, lifecycle)"

requirements-completed:
  - USER-PROFILE-01
  - USER-PROFILE-02

# Metrics
duration: 13min
completed: 2026-03-22
---

# Phase 02 Plan 03: Usecase Layer, gRPC Handler, and Composition Root Summary

**8 usecase functions with full role-hierarchy enforcement + UserHandler<C> gRPC impl + main.rs composition root making User service a fully functional gRPC server**

## Performance

- **Duration:** 13 min
- **Started:** 2026-03-22T12:25:19Z
- **Completed:** 2026-03-22T12:38:06Z
- **Tasks:** 2
- **Files modified:** 24

## Accomplishments

- All 8 usecase functions implemented with complete business rule enforcement (D-16 through D-22, D-53, D-55, D-58, D-60)
- 59 unit tests passing: 37 new usecase tests + 22 existing domain tests
- UserHandler<C: UserPorts> implementing all 9 UserService gRPC RPCs with proper caller context extraction
- Composition root in main.rs: Database::connect, Migrator::up, assemble context, serve gRPC
- Old service.rs stub deleted and replaced by 4-layer architecture

## Task Commits

Each task was committed atomically:

1. **Task 1: 8 usecase functions with TDD** - `a46188a` + `2540e23` (feat + fix)
2. **Task 2: gRPC handler, main.rs, and remove stub** - `ccfa2fe` (feat)

## Files Created/Modified

- `services/user/src/usecase/create_user.rs` - CreateUser with owner rejection, handle/name validation, UUIDv4 generation
- `services/user/src/usecase/get_user.rs` - GetUser always returns regardless of is_active (D-53)
- `services/user/src/usecase/get_user_by_handle.rs` - Admin-gated inactive user visibility (D-55)
- `services/user/src/usecase/list_users.rs` - Base64 cursor pagination, limit cap at 100
- `services/user/src/usecase/update_user.rs` - Handle uniqueness check (excluding self), name validation
- `services/user/src/usecase/deactivate_user.rs` - Role hierarchy, self-deactivation block, structured tracing
- `services/user/src/usecase/activate_user.rs` - Same rules as deactivate, structured tracing
- `services/user/src/usecase/change_role.rs` - D-18: caller_role > target_current AND caller_role > new_role
- `services/user/src/app/handler/mod.rs` - UserHandler<C>, CallerContext extraction helpers, role converters
- `services/user/src/app/handler/lifecycle.rs` - deactivate, activate, change_role RPCs
- `services/user/src/main.rs` - Full composition root with DB + migration + gRPC serve
- `services/user/src/domain/ports/user_repository.rs` - automock(target=UserRepository) for Send mock
- `CLAUDE.md` - Added USER_DATABASE_URL env var

## Decisions Made

- Used `#[automock(target = UserRepository)]` instead of bare `#[automock]`: with `trait_variant::make(UserRepository: Send)`, the macro generates `UserRepository` as a separate trait with Send futures. The bare automock generates `MockLocalUserRepository` implementing `LocalUserRepository` (non-Send). Setting `target = UserRepository` generates `MockUserRepository` implementing the Send variant, which satisfies the `UserPorts::user_repo() -> &impl UserRepository` return bound and works in tonic's async context.
- `base64` crate added to workspace for opaque cursor encoding. Cursor format: `{rfc3339_timestamp},{uuid}` base64-encoded.
- `ListUsersRequest.limit` is `i32` in proto; cast to `u64` with 0-default-to-25 and 100-cap in usecase.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed mockall automock target for trait_variant Send variant**
- **Found during:** Task 1 (usecase unit tests)
- **Issue:** `MockLocalUserRepository` (bare `#[automock]`) implements `LocalUserRepository` (non-Send futures), not `UserRepository` (Send futures). Using it as `&impl UserRepository` in `TestContext::user_repo()` caused trait bound errors. The tonic handler also requires Send futures.
- **Fix:** Changed `#[automock]` to `#[automock(target = UserRepository)]` in `user_repository.rs`, and updated all test `TestContext` impls and test imports to use `MockUserRepository` and `UserRepository`.
- **Files modified:** `services/user/src/domain/ports/user_repository.rs`, all 8 usecase files
- **Verification:** `cargo test -p user` passes all 59 tests including domain and usecase layers
- **Committed in:** a46188a (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - Bug)
**Impact on plan:** Auto-fix necessary for correct mock compatibility with trait_variant + tonic. No scope creep.

## Issues Encountered

- `#[automock]` + `#[trait_variant::make(UserRepository: Send)]` interaction: The `automock` macro targets the immediately annotated trait (`LocalUserRepository`), not the generated `UserRepository`. This means `MockLocalUserRepository` only implements the non-Send variant. Solution: use `#[automock(target = UserRepository)]` per mockall documentation.

## Known Stubs

None - all RPCs are fully implemented with real business logic.

## User Setup Required

None - no external service configuration required beyond what Plan 02-02 established (USER_DATABASE_URL, USER_LISTEN_ADDR).

## Next Phase Readiness

- User service is a complete, compilable gRPC server
- All RPCs: CreateUser, GetUser, GetUserByHandle, ListUsers, UpdateUser, DeactivateUser, ActivateUser, ChangeRole, Health
- Ready for Phase 3 (Authentication): Auth service can call CreateUser (registration) and GetUser (login/JWT claims)
- Gateway routes for user management can be added at any point

---
*Phase: 02-user-profile*
*Completed: 2026-03-22*

## Self-Check: PASSED

- FOUND: services/user/src/usecase/create_user.rs
- FOUND: services/user/src/usecase/change_role.rs
- FOUND: services/user/src/app/handler/mod.rs
- FOUND: services/user/src/main.rs
- FOUND: .planning/phases/02-user-profile/02-03-SUMMARY.md
- FOUND commit: a46188a (Task 1: usecase functions)
- FOUND commit: ccfa2fe (Task 2: gRPC handler + main.rs)
- FOUND commit: 2540e23 (clippy fix)
