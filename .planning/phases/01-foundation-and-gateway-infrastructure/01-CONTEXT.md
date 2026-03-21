# Phase 1: Foundation and Gateway Infrastructure - Context

**Gathered:** 2026-03-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Set up Cargo workspace with shared crates, proto definitions, and a gateway that accepts REST requests and routes them to internal gRPC services. All three backend services (auth, catalog, user) exist as Health RPC stubs. No database, no business logic, no authentication -- pure infrastructure scaffolding and routing verification.

</domain>

<decisions>
## Implementation Decisions

### Crate Structure
- 3 shared crates as defined in PROJECT.md: `madome-proto`, `madome-core`, `madome-common`
- No centralized entity/migration crate -- per-service `schema/` and `migration/` folders instead
- Research doc's 4-crate recommendation (entity + migration) explicitly rejected

### Proto Contract
- Routing verification only -- each service gets a Health RPC, no business RPCs in Phase 1
- `google.protobuf.Empty` used instead of custom Empty message
- 3 proto files: `auth.proto`, `catalog.proto`, `user.proto` (no `file.proto` -- v2 scope)
- Proto files located at workspace root `proto/` directory (single source of truth)
- Package naming: `madome.auth`, `madome.catalog`, `madome.user`
- `common.proto` content: Claude's discretion based on what Health RPC needs

### Service Stubs
- All 3 backend services (auth, catalog, user) implemented as Health RPC stubs
- DB-free: pure in-memory stubs, no PostgreSQL connection in Phase 1
- Stub execution approach: Claude's discretion (individual binaries vs dev binary)

### Gateway Routes
- Gateway exposes its own `/health` endpoint
- Gateway routes health checks to each backend service via gRPC
- gRPC Status to HTTP status code mapping: Claude's discretion

### REST API Format
- Success responses: flat JSON (data at top level, no envelope)
  - Single: `{ "id": "...", "title": "..." }`
  - List: `[{ ... }, { ... }]`
- Error responses: `{ "error": "not_found", "message": "Book not found" }` (machine-readable code + human-readable message)

### Configuration
- Environment variables only (no figment, no config files)
- Static service discovery: Gateway reads gRPC addresses from env vars (AUTH_GRPC_ADDR, CATALOG_GRPC_ADDR, USER_GRPC_ADDR)

### Observability
- tracing + tracing-subscriber for console logging
- UUIDv7 request_id generated at gateway, propagated via gRPC metadata
- No OpenTelemetry SDK in Phase 1 -- added later

### Shared Crate Content (Phase 1)
- `madome-core`: Error types only (AppError with gRPC Status <-> HTTP mapping). Domain types (BookId, UserId) added in later phases
- `madome-common`: tracing initialization + env var parsing utilities

### Workspace Structure
- `crates/` + `services/` directory layout
- All v1 services as workspace members: gateway + auth + catalog + user + 3 shared crates (7 total)
- `schema/` + `migration/` directories created now (empty, structure scaffolding)
- `[workspace.dependencies]` for centralized version management

### Testing
- Integration tests: Gateway + stub services running together, verified via HTTP requests
- No CI pipeline in Phase 1

### Deployment
- `cargo run` only -- no Docker, no docker-compose
- Docker setup added when DB is needed (Phase 2+)

### Claude's Discretion
- Stub service execution approach (individual binaries vs all-in-one dev binary)
- gRPC Status -> HTTP status code mapping specifics
- common.proto content
- Exact tracing-subscriber configuration
- Port number conventions

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Architecture
- `.planning/PROJECT.md` -- Authoritative architecture decisions, service topology, communication flow, ID design, auth design
- `.planning/research/ARCHITECTURE.md` -- Detailed patterns (REST-to-gRPC translator, shared proto crate, service-repository separation). NOTE: some recommendations superseded by PROJECT.md (entity/migration crate structure)

### Requirements
- `.planning/REQUIREMENTS.md` -- GATE-01 (REST API + route registration), GATE-02 (REST-to-gRPC routing) are Phase 1 scope

### Stack
- `.planning/research/STACK.md` -- Technology choices and version recommendations

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None -- project is greenfield (only hello world main.rs exists)

### Established Patterns
- None -- Phase 1 establishes the foundational patterns

### Integration Points
- Workspace root `Cargo.toml` must be converted from single package to virtual workspace
- Existing `src/main.rs` will be removed (replaced by service-specific binaries)

</code_context>

<specifics>
## Specific Ideas

- User explicitly called out using `google.protobuf.Empty` instead of defining a custom Empty message -- avoid unnecessary proto duplication
- Flat JSON responses chosen for simplicity over envelope pattern -- pagination metadata will be handled differently when needed (Phase 4)

</specifics>

<deferred>
## Deferred Ideas

- `file.proto` definition -- v2 scope (FILE-01/02/03)
- CI/CD pipeline (GitHub Actions) -- after real features are implemented
- Docker/docker-compose -- when DB connectivity is needed
- OpenTelemetry SDK integration -- after basic tracing is proven
- Domain types (BookId, UserId newtypes) in madome-core -- when first service needs them

</deferred>

---

*Phase: 01-foundation-and-gateway-infrastructure*
*Context gathered: 2026-03-21*
