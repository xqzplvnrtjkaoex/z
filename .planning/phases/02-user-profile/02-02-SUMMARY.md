---
phase: 02-user-profile
plan: "02"
subsystem: database
tags: [sea-orm, postgres, user-repository, adapter, context]

# Dependency graph
requires:
  - phase: 02-user-profile/02-01
    provides: Domain types (User, UserRole), port traits (UserRepository, UserPorts, UserConfig), RepositoryError, and sea-orm schema entity (users.rs)

provides:
  - PostgresUserRepository implementing all 5 UserRepository port methods using sea-orm
  - Domain-to-entity From conversions (UserRole bidirectional, Model->User)
  - UserContext implementing UserPorts + UserConfig, wiring PostgresUserRepository
  - Adapter module hierarchy (adapter/postgres/, adapter/context.rs)

affects:
  - 02-user-profile/02-03 (usecases reference UserContext and UserPorts)
  - 02-user-profile/02-04 (gRPC handler assembles UserContext in main.rs)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "PostgresUserRepository: concrete sea-orm adapter implementing domain port trait directly (impl UserRepository for PostgresUserRepository) — no async_trait needed with trait_variant"
    - "Domain-entity From conversions in adapter layer: infra knows domain, domain stays pure"
    - "UserContext composition root: holds concrete adapters, exposes &impl Trait via UserPorts RPITIT accessors"
    - "classify_db_err helper: error string inspection to map unique constraint DB errors to domain RepositoryError::UniqueViolation"
    - "Case-insensitive handle lookup via sea_query Func::lower() — Func::lower(Expr::col(...)).eq(handle.to_lowercase())"
    - "Composite cursor pagination: (created_at < t) OR (created_at = t AND id < id) using sea_orm::Condition::any()"

key-files:
  created:
    - services/user/src/adapter/postgres/user_repository.rs
    - services/user/src/adapter/postgres/mod.rs
    - services/user/src/adapter/mod.rs
    - services/user/src/adapter/context.rs
  modified:
    - services/user/src/lib.rs

key-decisions:
  - "Func::lower() from sea_query used for case-insensitive handle filter (Expr::cust approach from plan replaced — Func::lower is type-safe)"
  - "classify_db_err inspects error message string for 'duplicate key'/'unique constraint'/'23505' to detect unique violations since sea-orm wraps sqlx errors"
  - "chrono::DateTime<chrono::FixedOffset> conversion used for cursor comparison with DateTimeWithTimeZone sea-orm columns"

patterns-established:
  - "Adapter module hierarchy: adapter/mod.rs -> adapter/postgres/mod.rs -> adapter/postgres/user_repository.rs"
  - "UserContext in adapter/context.rs: holds concrete adapters, implements UserPorts returning &impl Trait accessors"
  - "From<schema::Model> for domain::Type in adapter layer (infra knows domain, not the other way)"

requirements-completed: [USER-PROFILE-01]

# Metrics
duration: 2min
completed: 2026-03-22
---

# Phase 2 Plan 02: User Service Adapter Layer Summary

**PostgresUserRepository with sea-orm and UserContext wiring, providing all 5 CRUD repository methods with domain-entity conversions and case-insensitive handle lookup.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-22T12:25:01Z
- **Completed:** 2026-03-22T12:27:35Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- PostgresUserRepository implements all 5 UserRepository methods (save, find_by_id, find_by_handle, list, update) using sea-orm against the users entity
- Bidirectional From conversions between schema::users::UserRole and domain::types::role::UserRole, plus From<Model> for User
- UserContext struct in adapter/context.rs implements UserPorts + UserConfig, wiring PostgresUserRepository as the concrete repository
- Case-insensitive handle lookup via Func::lower() in sea-query
- Composite keyset cursor pagination in list() using (created_at, id) ordering

## Task Commits

Each task was committed atomically:

1. **Task 1: PostgresUserRepository with domain-entity conversions** - `7a5c7c9` (feat)
2. **Task 2: UserContext wiring** - `14bfb04` (feat)

**Plan metadata:** committed alongside SUMMARY.md (docs)

## Files Created/Modified

- `services/user/src/adapter/postgres/user_repository.rs` - PostgresUserRepository: all 5 repository methods + domain-entity From conversions
- `services/user/src/adapter/postgres/mod.rs` - Module re-export for postgres adapter
- `services/user/src/adapter/mod.rs` - Module re-export for adapter layer (postgres + context)
- `services/user/src/adapter/context.rs` - UserContext implementing UserPorts + UserConfig
- `services/user/src/lib.rs` - Added `pub mod adapter;` declaration

## Decisions Made

- **Func::lower() for case-insensitive handle filter:** The plan suggested `Expr::cust("LOWER(handle) = LOWER($1)")` but `Func::lower(Expr::col(...)).eq(handle.to_lowercase())` is type-safe sea-query API and was preferred. Validated in sea-query 0.32.7.
- **Error classification via string inspection:** `classify_db_err` checks error message for "duplicate key", "unique constraint", "23505" to map to `RepositoryError::UniqueViolation("handle")`. This is the pragmatic approach since sea-orm wraps sqlx errors without a stable enum variant for unique violations.
- **chrono FixedOffset for cursor time:** sea-orm `DateTimeWithTimeZone` maps to `chrono::DateTime<FixedOffset>`, so cursor `DateTime<Utc>` must be converted with `.into()` before using in sea-orm filter conditions.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Replaced Expr::cust with Func::lower for handle lookup**
- **Found during:** Task 1 (PostgresUserRepository)
- **Issue:** Plan suggested `Expr::cust("LOWER(handle) = LOWER($1)").bind(handle)` but `to_lowercase()` is not a method on `sea_orm::sea_query::Expr`. Build error: `no method named 'to_lowercase' found`.
- **Fix:** Used `Func::lower(Expr::col(users::Column::Handle)).eq(handle.to_lowercase())` from sea_query — type-safe and idiomatic.
- **Files modified:** services/user/src/adapter/postgres/user_repository.rs
- **Verification:** `cargo build -p user` passes.
- **Committed in:** `7a5c7c9` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 — bug in plan's suggested API call)
**Impact on plan:** Functionally equivalent fix. Func::lower() is actually preferred over raw SQL string. No scope creep.

## Issues Encountered

None beyond the Func::lower deviation documented above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Adapter layer complete: PostgresUserRepository + UserContext ready for use in usecases (Plan 03)
- UserContext can be assembled in main.rs by providing a DatabaseConnection
- All 28 existing domain unit tests still pass after adapter layer addition

---
*Phase: 02-user-profile*
*Completed: 2026-03-22*
