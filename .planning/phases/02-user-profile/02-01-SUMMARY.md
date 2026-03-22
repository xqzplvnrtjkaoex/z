---
phase: 02-user-profile
plan: 01
subsystem: database
tags: [sea-orm, grpc, tonic, postgresql, validator, trait-variant, mockall, chrono, uuid]

# Dependency graph
requires:
  - phase: 01-foundation-and-gateway-infrastructure
    provides: "proto compilation pipeline, madome-proto crate, workspace structure, shared crates"

provides:
  - "user.proto with all 8 RPCs (CreateUser, GetUser, GetUserByHandle, ListUsers, UpdateUser, DeactivateUser, ActivateUser, ChangeRole)"
  - "UserRole domain enum with hierarchy ordering and can_manage logic"
  - "User domain struct with chrono DateTime fields"
  - "HandleInput/NameInput with #[derive(Validate)] declarative validation (D-12)"
  - "LocalUserRepository async trait with trait_variant + mockall::automock compatibility"
  - "UserPorts and UserConfig traits with RPITIT accessor pattern"
  - "UserError business error enum with tonic::Status mapping"
  - "RepositoryError data-access error enum"
  - "sea-orm entity (schema/users.rs) with DeriveActiveEnum for user_role PostgreSQL enum"
  - "Migration creating user_role enum type, users table, and case-insensitive LOWER(handle) unique index"
  - "docker-compose.yml with postgres:17 on port 5433"
  - "justfile with db-up/down/reset and test/lint/build recipes"

affects:
  - 02-02 (usecase layer builds on domain types, ports, and errors)
  - 02-03 (adapter layer implements UserRepository port, uses schema entity and migration)
  - 02-04 (gateway routes call user service gRPC RPCs defined here)
  - 03-authentication (auth service calls CreateUser and GetUser RPCs via gRPC client)

# Tech tracking
tech-stack:
  added:
    - sea-orm 1.1 (ORM with PostgreSQL, with-chrono, with-uuid features)
    - sea-orm-migration 1.1 (programmatic migrations)
    - validator 0.20 (derive-based struct validation)
    - trait-variant 0.1 (Send-bound async trait generation)
    - async-trait 0.1 (required for sea-orm-migration MigrationTrait)
    - chrono 0.4 (DateTime types with serde)
    - regex 1 (handle character pattern validation)
    - mockall 0.14 (mock generation for unit tests)
    - testcontainers 0.27 (Docker container lifecycle for tests)
    - testcontainers-modules 0.15 (PostgreSQL module)
    - uuid v4 feature added (was v7-only before)
  patterns:
    - "4-layer domain foundation: domain/types, domain/ports, domain/error"
    - "UserPorts RPITIT accessor pattern (fn user_repo(&self) -> &impl UserRepository)"
    - "trait_variant::make with #[cfg_attr(test, mockall::automock)] ordering (automock BEFORE trait_variant)"
    - "Manual TestContext pattern for unit testing usecase functions with RPITIT ports"
    - "DeriveActiveEnum for PostgreSQL enum types in sea-orm entities"
    - "Migration: create_type before create_table; raw SQL for expression indexes"
    - "validator #[derive(Validate)] with custom functions for complex rules (reserved handles, Unicode char count)"

key-files:
  created:
    - proto/user.proto
    - services/user/src/lib.rs
    - services/user/src/domain/mod.rs
    - services/user/src/domain/types/role.rs
    - services/user/src/domain/types/user.rs
    - services/user/src/domain/types/validation.rs
    - services/user/src/domain/types/mod.rs
    - services/user/src/domain/ports/user_repository.rs
    - services/user/src/domain/ports/mod.rs
    - services/user/src/domain/error/user_error.rs
    - services/user/src/domain/error/repository_error.rs
    - services/user/src/domain/error/mod.rs
    - services/user/schema/users.rs
    - services/user/schema/mod.rs
    - services/user/migration/m20260322_000001_create_users_table.rs
    - services/user/migration/mod.rs
    - docker-compose.yml
    - justfile
  modified:
    - proto/user.proto (replaced Health-only stub with full 8-RPC definition)
    - Cargo.toml (added sea-orm, sea-orm-migration, validator, chrono, trait-variant, async-trait, mockall, testcontainers, regex deps; added v4 to uuid features)
    - services/user/Cargo.toml (added [lib] section, all new deps, dev-dependencies)
    - services/user/src/service.rs (added stub impls for all 8 new RPCs to keep compilation)

key-decisions:
  - "DeriveIden Table variant used as .as_enum(UserRoleEnum::Table) not bare enum - sea-query API requires variant not enum"
  - "regex crate added to workspace for handle character validation (LazyLock<Regex>)"
  - "service.rs kept with stub impls to avoid breaking main.rs; will be replaced in Plan 03 with 4-layer handler"
  - "check_handle_chars as custom validator (not #[validate(regex)]) for clean LazyLock integration"

patterns-established:
  - "Domain error separation: UserError (business meaning) vs RepositoryError (data-access facts)"
  - "UserRole.can_manage uses strict greater-than (>) for hierarchy enforcement"
  - "Handle validation: length by validator attribute, chars by custom function, reserved by custom function"
  - "Name validation: Unicode char count via chars().count() in custom validator (not byte length)"
  - "MockLocalUserRepository (not MockUserRepository) - mockall targets LocalUserRepository before trait_variant transform"

requirements-completed:
  - USER-PROFILE-01
  - USER-PROFILE-02

# Metrics
duration: 6min
completed: 2026-03-22
---

# Phase 02 Plan 01: User Service Foundation Summary

**gRPC user.proto with 8 RPCs, domain layer with trait_variant+mockall async ports, sea-orm entity and migration for users table with PostgreSQL enum, and validator-based handle/name validation**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-22T12:12:51Z
- **Completed:** 2026-03-22T12:18:57Z
- **Tasks:** 2
- **Files modified:** 22 (4 modified, 18 created)

## Accomplishments

- Full user.proto with all 8 RPCs (CreateUser, GetUser, GetUserByHandle, ListUsers, UpdateUser, DeactivateUser, ActivateUser, ChangeRole) compiling via madome-proto
- Domain layer with UserRole hierarchy (can_manage), User struct, HandleInput/NameInput validation, UserPorts RPITIT trait, UserRepository async trait with trait_variant+mockall, UserError and RepositoryError enums
- sea-orm entity for users table with DeriveActiveEnum for PostgreSQL user_role enum type; migration with create_type-before-create_table ordering and LOWER(handle) functional unique index
- 28 unit tests passing: role ordering/hierarchy, validation (handle length/chars/reserved/Unicode names), error-to-status mapping, mockall mock compilation and usage

## Task Commits

1. **Task 1: Proto definitions, workspace dependencies, and dev infrastructure** - `46425d4` (feat)
2. **Task 2: Domain layer, sea-orm entity, and migration** - `abd6f2d` (feat)

## Files Created/Modified

- `proto/user.proto` - Full gRPC service definition with 8 RPCs, Role enum, and all message types
- `Cargo.toml` - Added sea-orm, validator, trait-variant, mockall, testcontainers, regex, chrono deps
- `services/user/Cargo.toml` - Added [lib] section, all new service dependencies
- `services/user/src/lib.rs` - Crate root re-exporting domain, schema, migration modules
- `services/user/src/domain/types/role.rs` - UserRole enum with hierarchy ordering and can_manage
- `services/user/src/domain/types/user.rs` - User domain struct
- `services/user/src/domain/types/validation.rs` - HandleInput/NameInput with #[derive(Validate)]
- `services/user/src/domain/ports/user_repository.rs` - LocalUserRepository with trait_variant + mockall
- `services/user/src/domain/ports/mod.rs` - UserPorts and UserConfig traits
- `services/user/src/domain/error/user_error.rs` - UserError with tonic::Status mapping
- `services/user/src/domain/error/repository_error.rs` - RepositoryError enum
- `services/user/schema/users.rs` - sea-orm entity with DeriveActiveEnum
- `services/user/migration/m20260322_000001_create_users_table.rs` - Initial migration
- `docker-compose.yml` - PostgreSQL dev environment (postgres:17, port 5433)
- `justfile` - Dev recipes (db-up, db-down, db-reset, test, lint, fmt-check, build)

## Decisions Made

- `DeriveIden` enum `Table` variant used as `.as_enum(UserRoleEnum::Table)` - sea-query API requires a variant value, not the enum type itself
- Added `regex` crate to workspace deps for `LazyLock<Regex>` in handle character validation
- Kept `service.rs` with stub impls for the 8 new RPCs to preserve binary compilation; this file will be replaced in Plan 02/03 with the 4-layer handler
- Used custom validator function `check_handle_chars` instead of `#[validate(regex)]` attribute for clean `LazyLock` integration

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated service.rs to implement all 8 new proto RPCs**
- **Found during:** Task 2 (domain layer compilation)
- **Issue:** Proto expansion added 8 new RPCs; service.rs only implemented `health`, causing compile error "not all trait items implemented"
- **Fix:** Added stub implementations returning `Err(Status::unimplemented("not implemented"))` for all 8 new RPCs
- **Files modified:** `services/user/src/service.rs`
- **Verification:** `cargo build -p user` passes
- **Committed in:** `abd6f2d` (Task 2 commit)

**2. [Rule 2 - Missing Critical] Added regex dependency for handle validation**
- **Found during:** Task 2 (validation implementation)
- **Issue:** `check_handle_chars` validator uses `regex::Regex` for alphanumeric+underscore pattern; crate not in workspace deps
- **Fix:** Added `regex = "1"` to workspace deps and `regex = { workspace = true }` to user Cargo.toml
- **Files modified:** `Cargo.toml`, `services/user/Cargo.toml`
- **Verification:** Compiles and regex validation tests pass
- **Committed in:** `abd6f2d` (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 bug, 1 missing critical dependency)
**Impact on plan:** Both fixes necessary for correctness. No scope creep.

## Issues Encountered

- sea-orm-migration `as_enum()` API requires an enum variant (`UserRoleEnum::Table`), not the bare enum type - research example had a syntax error. Fixed immediately during first build.

## Known Stubs

- `services/user/src/service.rs` - All 8 RPCs return `Status::unimplemented`. This is intentional: the real handler will be built in Plan 02 (usecase layer) and Plan 03 (adapter + app layers). The service.rs file is a placeholder to keep the binary compilable.

## Next Phase Readiness

- Domain contracts fully defined: types, ports, errors ready for usecase layer (Plan 02)
- sea-orm entity and migration ready for adapter implementation (Plan 03)
- Proto compiled and available via madome-proto crate for gateway integration (Plan 04)
- docker-compose.yml ready for development and integration test PostgreSQL

---
*Phase: 02-user-profile*
*Completed: 2026-03-22*
