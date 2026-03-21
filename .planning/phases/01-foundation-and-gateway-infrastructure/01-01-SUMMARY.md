---
phase: 01-foundation-and-gateway-infrastructure
plan: 01
subsystem: infra
tags: [rust, cargo-workspace, grpc, tonic, protobuf, axum, tracing]

# Dependency graph
requires: []
provides:
  - Virtual Cargo workspace (resolver 3) with 7 members
  - Three proto files defining Health RPCs for auth, catalog, user services
  - madome-proto crate generating gRPC server+client code via tonic-prost-build
  - AppError enum with HTTP/gRPC status mapping and flat JSON error body
  - init_tracing() structured logging with RUST_LOG/EnvFilter support
  - required_env() and optional_env() configuration utilities
affects:
  - 01-02 (gateway, auth, catalog, user services — all depend on these shared crates)
  - all subsequent phases (shared foundation)

# Tech tracking
tech-stack:
  added:
    - tokio 1.50 (async runtime)
    - axum 0.8 (REST framework)
    - tonic 0.14 + tonic-prost 0.14 (gRPC)
    - prost 0.14 + prost-types 0.14 (protobuf)
    - tonic-prost-build 0.14 (proto compilation)
    - thiserror 2 (error handling)
    - tracing 0.1 + tracing-subscriber 0.3 (observability)
    - serde 1 + serde_json 1 (serialization)
    - uuid 1 (UUIDv7 support)
    - protoc (installed via homebrew, required by tonic-prost-build)
  patterns:
    - Virtual workspace manifest with centralized workspace.dependencies
    - Single madome-proto crate compiles all protos once, services depend on compiled crate
    - AppError enum with IntoResponse + From<tonic::Status> for gateway error translation
    - Flat JSON error body: { "error": "machine_code", "message": "human readable" }

key-files:
  created:
    - Cargo.toml (virtual workspace manifest, resolver 3, 7 members)
    - proto/auth.proto (AuthService Health RPC)
    - proto/catalog.proto (CatalogService Health RPC)
    - proto/user.proto (UserService Health RPC)
    - crates/madome-proto/Cargo.toml
    - crates/madome-proto/build.rs (tonic-prost-build compilation)
    - crates/madome-proto/src/lib.rs (include_proto! re-exports)
    - crates/madome-core/Cargo.toml
    - crates/madome-core/src/error.rs (AppError with IntoResponse + From<tonic::Status>)
    - crates/madome-core/src/lib.rs
    - crates/madome-common/Cargo.toml
    - crates/madome-common/src/tracing.rs (init_tracing())
    - crates/madome-common/src/env.rs (required_env, optional_env)
    - crates/madome-common/src/lib.rs
    - services/*/Cargo.toml + src/main.rs (stub placeholders for workspace loading)
  modified:
    - .gitignore (already had /target and .env entries)

key-decisions:
  - "Stub service Cargo.toml files created for workspace loading; services/ stubs not part of plan spec but required for workspace to resolve"
  - "protoc installed via homebrew (Rule 3 - blocking dependency) as tonic-prost-build requires it"
  - "pub mod tracing module name used in madome-common (no collision since tracing crate accessed via tracing_subscriber directly)"
  - "DeadlineExceeded maps to Unavailable with timeout prefix per plan spec"

patterns-established:
  - "Pattern 1: All workspace deps centralized in [workspace.dependencies]; crates use workspace = true"
  - "Pattern 2: Proto compiled once in madome-proto; all services depend on compiled crate, not raw protos"
  - "Pattern 3: AppError as the gateway error translation layer - From<tonic::Status> enables ? operator in handlers"
  - "Pattern 4: Flat JSON error format { error, message } - machine-readable code + human message"

requirements-completed: [GATE-01, GATE-02]

# Metrics
duration: 3min
completed: 2026-03-21
---

# Phase 01 Plan 01: Foundation and Gateway Infrastructure Summary

**Cargo workspace with tonic-prost-build proto compilation, AppError gRPC-to-HTTP mapping, and shared tracing/env utilities across three shared crates (madome-proto, madome-core, madome-common)**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-21T12:48:31Z
- **Completed:** 2026-03-21T12:51:34Z
- **Tasks:** 2
- **Files modified:** 19

## Accomplishments
- Virtual workspace manifest (resolver 3) with all 7 crate members and centralized dependencies
- Three proto files with Health RPCs compile via tonic-prost-build, generating gRPC server+client code for auth, catalog, user
- AppError with IntoResponse and From<tonic::Status> for transparent REST-gRPC error translation at gateway layer
- init_tracing() with RUST_LOG/EnvFilter support and structured logging
- required_env()/optional_env() utilities with clear panic messages

## Task Commits

Each task was committed atomically:

1. **Task 1: Workspace manifest, proto definitions, and madome-proto crate** - `41ef18b` (feat)
2. **Task 2: madome-core error types and madome-common utilities** - `d58c307` (feat)

## Files Created/Modified
- `Cargo.toml` - Virtual workspace manifest with resolver 3, 7 members, workspace.dependencies
- `proto/auth.proto` - AuthService Health RPC using google.protobuf.Empty
- `proto/catalog.proto` - CatalogService Health RPC using google.protobuf.Empty
- `proto/user.proto` - UserService Health RPC using google.protobuf.Empty
- `crates/madome-proto/build.rs` - tonic_prost_build::configure().compile_protos() for all 3 protos
- `crates/madome-proto/src/lib.rs` - tonic::include_proto! re-exports for madome.auth, madome.catalog, madome.user
- `crates/madome-core/src/error.rs` - AppError enum, IntoResponse, From<tonic::Status> with 7-variant gRPC mapping
- `crates/madome-common/src/tracing.rs` - init_tracing() with EnvFilter::try_from_default_env
- `crates/madome-common/src/env.rs` - required_env() and optional_env()
- `services/*/Cargo.toml` + `services/*/src/main.rs` - Stub placeholders required for workspace to resolve

## Decisions Made
- Stub service Cargo.toml + src/main.rs files created for workspace to load; they are placeholder-level (empty fn main()) and will be replaced in Plan 02
- protoc installed via homebrew as it is required by tonic-prost-build at build time
- pub mod tracing module name retained in madome-common (no collision since the tracing crate itself is accessed through tracing_subscriber)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Installed protoc via homebrew**
- **Found during:** Task 1 (building madome-proto)
- **Issue:** tonic-prost-build requires a protoc binary at build time; not present on the system
- **Fix:** Ran `brew install protobuf`, protoc v34.0 installed at /opt/homebrew/bin/protoc
- **Files modified:** None (system dependency)
- **Verification:** `cargo build -p madome-proto` succeeded after installation
- **Committed in:** 41ef18b (Task 1 commit - Cargo.lock updated with resolved deps)

**2. [Rule 3 - Blocking] Created stub service Cargo.toml files for workspace member resolution**
- **Found during:** Task 1 (first `cargo build -p madome-proto` attempt)
- **Issue:** Workspace lists 7 members; Cargo refuses to build if any member Cargo.toml is missing, even when only building a specific package
- **Fix:** Created minimal Cargo.toml + src/main.rs for gateway/auth/catalog/user
- **Files modified:** services/gateway/Cargo.toml, services/auth/Cargo.toml, services/catalog/Cargo.toml, services/user/Cargo.toml (and corresponding src/main.rs stubs)
- **Verification:** `cargo build -p madome-proto` succeeded
- **Committed in:** 41ef18b (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both auto-fixes necessary for compilation to proceed. No scope creep; stub services are intentional workspace placeholders to be replaced in Plan 02.

## Issues Encountered
None beyond the blocking issues documented above, which were resolved automatically.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All three shared crates (madome-proto, madome-core, madome-common) compile independently
- Service stub placeholders are in place for workspace resolution
- Plan 02 can proceed: implement gateway, auth, catalog, and user service binaries using the shared crates as dependencies

---
*Phase: 01-foundation-and-gateway-infrastructure*
*Completed: 2026-03-21*

## Self-Check: PASSED

- All key files confirmed present on disk
- Both task commits (41ef18b, d58c307) verified in git history
- src/main.rs confirmed deleted
- All three shared crates build successfully (verified during execution)
