---
phase: 01-foundation-and-gateway-infrastructure
plan: 02
subsystem: infra
tags: [rust, axum, tonic, grpc, gateway, rest-to-grpc, integration-tests, tdd]

# Dependency graph
requires:
  - 01-01 (madome-proto gRPC types, madome-core AppError, madome-common env/tracing utilities)
provides:
  - auth binary: gRPC server implementing Health RPC, reads AUTH_LISTEN_ADDR
  - catalog binary: gRPC server implementing Health RPC, reads CATALOG_LISTEN_ADDR
  - user binary: gRPC server implementing Health RPC, reads USER_LISTEN_ADDR
  - gateway binary: Axum REST server proxying to gRPC backends via AppState
  - GET /health: gateway own-health endpoint (no gRPC calls)
  - GET /v1/health/services: aggregated gRPC Health RPC responses from all backends
  - UUIDv7 request_id propagated via x-request-id gRPC metadata per request
  - Integration tests proving full REST-to-gRPC-to-REST chain
  - scripts/dev.sh: starts all four services with correct env vars
affects:
  - Phase 2 (auth service — real business logic replaces Health stub)
  - Phase 3 (catalog service — real business logic replaces Health stub)

# Tech tracking
tech-stack:
  added:
    - reqwest 0.12 (HTTP client for integration tests)
  patterns:
    - Gateway lib.rs pattern: split gateway crate into lib.rs + main.rs so integration tests
      in tests/ can import gateway::state::AppState and gateway::routes::create_router
    - Lazy gRPC connect pattern: tonic Endpoint::connect_lazy() used in tests so gateway
      can start without requiring all backends to be running
    - Partial failure aggregation: individual gRPC failures map to "unavailable" string
      without failing the whole REST response
    - Unit type () for Health RPC: tonic-prost 0.14 maps google.protobuf.Empty to () not
      prost_types::Empty

key-files:
  created:
    - services/auth/Cargo.toml (tonic + madome-proto + madome-common)
    - services/auth/src/main.rs (gRPC server reading AUTH_LISTEN_ADDR)
    - services/auth/src/service.rs (AuthServiceImpl with Health RPC returning ())
    - services/auth/schema/.gitkeep (DB schema scaffolding)
    - services/auth/migration/.gitkeep (DB migration scaffolding)
    - services/catalog/Cargo.toml
    - services/catalog/src/main.rs (CATALOG_LISTEN_ADDR)
    - services/catalog/src/service.rs (CatalogServiceImpl)
    - services/catalog/schema/.gitkeep
    - services/catalog/migration/.gitkeep
    - services/user/Cargo.toml
    - services/user/src/main.rs (USER_LISTEN_ADDR)
    - services/user/src/service.rs (UserServiceImpl)
    - services/user/schema/.gitkeep
    - services/user/migration/.gitkeep
    - services/gateway/src/lib.rs (pub mod routes; pub mod state; for test access)
    - services/gateway/src/state.rs (AppState: Auth/Catalog/UserServiceClient<Channel>)
    - services/gateway/src/routes/mod.rs (create_router, /health and /v1 nesting)
    - services/gateway/src/routes/health.rs (gateway_health, service_health handlers)
    - services/gateway/tests/health_integration.rs (3 integration tests)
    - scripts/dev.sh (start all 4 services with env vars, Ctrl-C cleanup)
  modified:
    - services/gateway/Cargo.toml (full deps + [lib] section + dev-dependencies)
    - services/gateway/src/main.rs (full gateway entry point, imports from gateway lib)
    - services/auth/Cargo.toml (replaced stub with full deps)
    - services/auth/src/main.rs (replaced stub with full implementation)
    - services/catalog/Cargo.toml (replaced stub)
    - services/catalog/src/main.rs (replaced stub)
    - services/user/Cargo.toml (replaced stub)
    - services/user/src/main.rs (replaced stub)

key-decisions:
  - "Health RPC uses unit type () not prost_types::Empty — tonic-prost 0.14 maps google.protobuf.Empty to Rust () type"
  - "Gateway split into lib.rs + main.rs to enable integration test imports from tests/ directory"
  - "Lazy gRPC client connections (connect_lazy) used in integration tests to avoid startup ordering requirements"
  - "Partial backend failure returns 200 with service: unavailable rather than propagating error to caller"

requirements-completed: [GATE-01, GATE-02]

# Metrics
duration: 7min
completed: 2026-03-21
---

# Phase 01 Plan 02: Gateway and Backend Service Binaries Summary

**Four runnable binaries (gateway, auth, catalog, user) with REST-to-gRPC routing proven by integration tests: gateway aggregates Health RPCs from all three backends, propagates UUIDv7 request_id via gRPC metadata, and handles partial backend failures gracefully**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-21T12:55:24Z
- **Completed:** 2026-03-21T13:02:24Z
- **Tasks:** 3
- **Files modified:** 28

## Accomplishments

- Auth, catalog, and user gRPC servers each implement Health RPC and read their listen address from a required env var (AUTH/CATALOG/USER_LISTEN_ADDR)
- Gateway Axum server exposes GET /health (own status) and GET /v1/health/services (aggregated gRPC health check)
- UUIDv7 request_id generated per request and propagated as x-request-id gRPC metadata to all three backends
- Individual backend failures return `"unavailable"` without failing the whole REST response (200 with partial status)
- Three integration tests verify the full REST-to-gRPC-to-REST chain: own health, all services ok, partial failure
- scripts/dev.sh starts all four services with correct environment variables and handles Ctrl-C cleanup

## Task Commits

Each task was committed atomically:

1. **Task 1: Backend service stubs (auth, catalog, user)** - `3673f0b` (feat)
2. **Task 2: Gateway with REST-to-gRPC routing** - `9a444a5` (feat)
3. **Task 3: Integration tests and dev startup script** - `278948a` (feat)

## Files Created/Modified

- `services/auth/Cargo.toml` - tonic, madome-proto, madome-common deps
- `services/auth/src/main.rs` - AUTH_LISTEN_ADDR env var, AuthServiceServer startup
- `services/auth/src/service.rs` - AuthServiceImpl: Health RPC returning ()
- `services/catalog/` - Same pattern with CATALOG_LISTEN_ADDR and CatalogServiceImpl
- `services/user/` - Same pattern with USER_LISTEN_ADDR and UserServiceImpl
- `services/*/schema/.gitkeep` + `services/*/migration/.gitkeep` - DB scaffolding directories
- `services/gateway/Cargo.toml` - [lib] section, full deps, reqwest/serde_json dev-deps
- `services/gateway/src/lib.rs` - Exposes pub mod routes and pub mod state for integration tests
- `services/gateway/src/main.rs` - Imports from gateway lib, connects gRPC clients, TraceLayer
- `services/gateway/src/state.rs` - AppState with three cloneable gRPC clients
- `services/gateway/src/routes/mod.rs` - create_router(), /health and /v1/health/services routes
- `services/gateway/src/routes/health.rs` - gateway_health(), service_health(), call_*_health() helpers
- `services/gateway/tests/health_integration.rs` - 3 tokio integration tests
- `scripts/dev.sh` - Start all 4 services, wait, Ctrl-C trap

## Decisions Made

- Health RPC trait signature uses `Request<()>` / `Response<()>` — tonic-prost 0.14 maps `google.protobuf.Empty` to Rust unit type `()`, not `prost_types::Empty`
- Gateway crate split into lib.rs + main.rs so integration tests in `tests/` can access `gateway::state::AppState` and `gateway::routes::create_router`
- `connect_lazy()` used in integration tests so gateway starts without requiring all backends to be live at connection time
- Individual gRPC call failures return `"unavailable"` string; only the overall response status is affected if all services are down simultaneously (returns 200 regardless in current design per plan spec)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] prost_types::Empty replaced with unit type ()**
- **Found during:** Task 1 verification (cargo build -p auth -p catalog -p user)
- **Issue:** Plan spec showed `Request<prost_types::Empty>` / `Response<prost_types::Empty>` but tonic-prost 0.14 generates Health RPC with `Request<()>` / `Response<()>` — google.protobuf.Empty maps to Rust `()` in this prost version, not `prost_types::Empty`
- **Fix:** Changed all service.rs Health RPC signatures to use `()` and removed unnecessary `prost-types` dependency from service Cargo.toml files
- **Files modified:** services/auth/src/service.rs, services/catalog/src/service.rs, services/user/src/service.rs, and their Cargo.toml files
- **Commit:** 3673f0b

**2. [Rule 3 - Blocking] Added lib.rs to gateway crate for integration test access**
- **Found during:** Task 3 (TDD RED — writing integration tests)
- **Issue:** Integration tests in `services/gateway/tests/` need to import `gateway::state::AppState` and `gateway::routes::create_router`. Rust integration tests in the `tests/` directory can only import from the crate's lib, not from its binary. The gateway had only a binary (`main.rs`) with no library target.
- **Fix:** Added `services/gateway/src/lib.rs` declaring `pub mod routes; pub mod state;` and updated `Cargo.toml` with a `[lib]` section. Updated `main.rs` to import from `gateway::` (the lib) rather than declaring modules directly.
- **Files modified:** services/gateway/Cargo.toml, services/gateway/src/lib.rs (new), services/gateway/src/main.rs
- **Commit:** 278948a

**3. [Rule 3 - Blocking] tokio_stream not available; replaced with alternative test startup pattern**
- **Found during:** Task 3 (TDD RED — first compile attempt)
- **Issue:** Initial test implementation used `tokio_stream::wrappers::TcpListenerStream` for zero-port gRPC server binding, but `tokio_stream` is not a workspace dependency and would require adding a new crate.
- **Fix:** Replaced with `free_port()` helper (std::net::TcpListener::bind("127.0.0.1:0") + port extraction) plus `Server::builder().serve(addr)`. Used `connect_lazy()` (Endpoint API) for gRPC clients so partial failure tests don't require eager connection.
- **Files modified:** services/gateway/tests/health_integration.rs
- **Commit:** 278948a

---

**Total deviations:** 3 auto-fixed (1 bug, 2 blocking)
**Impact:** All fixes necessary for correct operation. No scope creep.

## Issues Encountered

None beyond the blocking/bug issues documented above, all resolved automatically.

## User Setup Required

To run all services locally:
```bash
./scripts/dev.sh
```

Then verify:
```bash
curl http://127.0.0.1:3000/health
curl http://127.0.0.1:3000/v1/health/services
```

## Next Phase Readiness

- All four binaries compile and integration tests pass (`cargo test --workspace`)
- Gateway correctly routes REST to gRPC and aggregates responses
- Auth, catalog, user service stubs ready to accept real business logic in subsequent phases
- Phase 2 (Auth) and Phase 3 (Catalog) can now proceed in parallel (both depend only on Phase 1)

---
*Phase: 01-foundation-and-gateway-infrastructure*
*Completed: 2026-03-21*

## Self-Check: PASSED

- All 14 key files confirmed present on disk
- All three task commits (3673f0b, 9a444a5, 278948a) verified in git history
- cargo build --workspace: Finished dev profile, all 7 crates compiled
- cargo test --workspace: 3 integration tests passed, 0 failed
