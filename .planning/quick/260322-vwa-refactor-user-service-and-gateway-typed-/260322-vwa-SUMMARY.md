---
phase: quick
plan: 260322-vwa
subsystem: api
tags: [axum, tonic, sea-orm, middleware, refactor]

requires: []
provides:
  - "madome-common::headers module with X_CALLER_ID/X_CALLER_ROLE/X_REQUEST_ID/X_NEXT_CURSOR constants"
  - "From<DbErr> for RepositoryError replacing classify_db_err() helper"
  - "CallerContext axum middleware in gateway replacing inject_caller_context() helper"
  - "Entity-qualified handler file names in user service"
  - "One-handler-per-file split of lifecycle.rs"
affects: [phase-03-auth, any-new-service, any-new-gateway-route]

tech-stack:
  added: []
  patterns:
    - "Typed header constants in madome-common::headers, used by both HTTP (axum) and gRPC (tonic) layers"
    - "From<SourceError> for TargetError pattern for clean ? propagation; no manual .map_err() conversion functions"
    - "Cross-cutting HTTP concern (caller identity extraction) as axum middleware, stored in request extensions"
    - "One handler per file, named by full domain action including entity (create_user.rs not create.rs)"

key-files:
  created:
    - crates/madome-common/src/headers.rs
    - services/gateway/src/middleware/caller_context.rs
    - services/gateway/src/middleware/mod.rs
    - services/user/src/app/handler/create_user.rs
    - services/user/src/app/handler/get_user.rs
    - services/user/src/app/handler/list_users.rs
    - services/user/src/app/handler/update_user.rs
    - services/user/src/app/handler/deactivate_user.rs
    - services/user/src/app/handler/activate_user.rs
    - services/user/src/app/handler/change_role.rs
  modified:
    - crates/madome-common/src/lib.rs
    - services/user/src/domain/error/repository_error.rs
    - services/user/src/adapter/postgres/user_repository.rs
    - services/user/src/app/handler/mod.rs
    - services/gateway/src/lib.rs
    - services/gateway/src/routes/mod.rs
    - services/gateway/src/routes/users.rs
    - services/gateway/src/routes/health.rs

key-decisions:
  - "Header constants defined as &'static str in madome-common (no http/tonic deps) — compatible with both axum::http::HeaderName::from_static() and tonic::metadata::MetadataMap::insert()"
  - "CallerContext middleware is permissive: inserts into extensions only when both headers present; handlers requiring auth use Extension<CallerContext> which returns 500 if missing"
  - "From<DbErr> impl placed in domain/error/repository_error.rs (idiomatic: conversions live with target type)"

patterns-established:
  - "Shared protocol constants: define once in madome-common::headers, use everywhere"
  - "Error conversions: From<SourceError> for TargetError in target type's module"
  - "Middleware pattern: extract cross-cutting data into request extensions, handlers pull via Extension<T>"
  - "File naming: handler files named by full domain action with entity (verb_entity.rs)"

requirements-completed: []

duration: 15min
completed: 2026-03-22
---

# Quick Task 260322-vwa: Refactor user service and gateway — typed headers, From impl, handler renames

**Convention-compliant user service and gateway: madome-common header constants replacing all hardcoded strings, From<DbErr> for clean ? propagation, CallerContext axum middleware, and entity-qualified one-handler-per-file structure**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-03-22T14:04:00Z
- **Completed:** 2026-03-22T14:19:00Z
- **Tasks:** 2
- **Files modified:** 18 (10 created, 8 modified, 5 deleted/renamed)

## Accomplishments

- Created `madome-common::headers` module with four typed constants (`X_CALLER_ID`, `X_CALLER_ROLE`, `X_REQUEST_ID`, `X_NEXT_CURSOR`) usable by both axum and tonic layers
- Replaced `classify_db_err()` standalone function with `From<sea_orm::DbErr> for RepositoryError`; all 4 call sites in `user_repository.rs` now use `?` operator directly
- Replaced `inject_caller_context()` helper in gateway with a proper axum middleware (`extract_caller_context`); handlers receive `CallerContext` via `Extension<CallerContext>` from extensions
- Renamed 4 handler files (create/get/list/update → create_user/get_user/list_users/update_user) using `git mv` to preserve history
- Split `lifecycle.rs` (3 handlers) into `deactivate_user.rs`, `activate_user.rs`, `change_role.rs` (one handler each)
- All 76 tests pass (59 user unit + 8 user integration + 6 user service + 3 gateway integration); workspace clippy clean

## Task Commits

1. **Task 1: Shared constants, From impl, handler file renames (user service)** - `4a4cd68` (refactor)
2. **Task 2: Gateway CallerContext middleware and typed header constants** - `17e1012` (refactor)

## Files Created/Modified

- `crates/madome-common/src/headers.rs` — new module with 4 typed header name constants
- `crates/madome-common/src/lib.rs` — added `pub mod headers;`
- `services/user/src/domain/error/repository_error.rs` — added `From<sea_orm::DbErr> for RepositoryError`
- `services/user/src/adapter/postgres/user_repository.rs` — removed `classify_db_err()`, updated 4 call sites to use `?`
- `services/user/src/app/handler/mod.rs` — updated module declarations and dispatch calls, added typed header usage
- `services/user/src/app/handler/create_user.rs` — renamed from create.rs (git mv)
- `services/user/src/app/handler/get_user.rs` — renamed from get.rs (git mv)
- `services/user/src/app/handler/list_users.rs` — renamed from list.rs (git mv)
- `services/user/src/app/handler/update_user.rs` — renamed from update.rs (git mv)
- `services/user/src/app/handler/deactivate_user.rs` — split from lifecycle.rs
- `services/user/src/app/handler/activate_user.rs` — split from lifecycle.rs
- `services/user/src/app/handler/change_role.rs` — split from lifecycle.rs
- `services/gateway/src/lib.rs` — added `pub mod middleware;`
- `services/gateway/src/middleware/mod.rs` — new module re-exporting CallerContext
- `services/gateway/src/middleware/caller_context.rs` — CallerContext struct + axum middleware function
- `services/gateway/src/routes/mod.rs` — applied `caller_context::extract_caller_context` layer to user_routes()
- `services/gateway/src/routes/users.rs` — removed inject_caller_context(), handlers use Extension<CallerContext>, typed header constants for content-type and x-next-cursor
- `services/gateway/src/routes/health.rs` — replaced 3 "x-request-id" string literals with `headers::X_REQUEST_ID`

## Decisions Made

- Header constants defined as `&'static str` (not `axum::http::HeaderName`) in `madome-common` to avoid adding http/tonic dependencies to the shared crate. Both `axum::http::HeaderName::from_static()` and `tonic::metadata::MetadataMap::insert()` accept `&str`, making this universally compatible.
- `From<DbErr>` impl placed in `domain/error/repository_error.rs` (where `RepositoryError` is defined) rather than `user_repository.rs`, per Rust convention that conversions live with the target type.
- CallerContext middleware is permissive (only inserts when both headers present) rather than enforcing — authentication enforcement belongs in Phase 3 auth middleware tier.

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## Known Stubs

None.

## Next Phase Readiness

- `madome-common::headers` constants are ready for Phase 3 (auth service) to use for gRPC metadata key consistency
- CallerContext middleware scaffold is in place; Phase 3 will add authentication enforcement layer on top
- All handler files follow entity-qualified naming convention ready for auth service handler files

---
*Quick task: 260322-vwa*
*Completed: 2026-03-22*
