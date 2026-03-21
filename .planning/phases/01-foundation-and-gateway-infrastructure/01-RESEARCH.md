# Phase 1: Foundation and Gateway Infrastructure - Research

**Researched:** 2026-03-21
**Domain:** Rust Cargo workspace, gRPC (tonic), REST (axum), proto compilation, service scaffolding
**Confidence:** HIGH

## Summary

Phase 1 establishes the Cargo workspace with 7 crates (gateway + 3 backend services + 3 shared crates), defines proto contracts with Health RPC stubs for all services, and builds a gateway that translates REST requests into gRPC calls. No database, no business logic, no authentication -- pure infrastructure scaffolding and routing verification.

The key technical insight is that tonic 0.14 has restructured its ecosystem: `tonic-prost` replaces direct `prost` dependency for codec, and `tonic-prost-build` replaces `tonic-build` + `prost-build` for proto compilation. The STACK.md recommendations for build dependencies are outdated on this point. Additionally, `tonic-health` (0.14.5) provides a standard gRPC health checking service, but the CONTEXT.md specifies custom Health RPCs per service proto -- so we define our own Health RPC in each `.proto` file using `google.protobuf.Empty`, which provides more control and matches the routing verification goal.

The workspace uses a virtual manifest with `resolver = "3"` (required for edition 2024 in virtual workspaces), `crates/` for shared libraries, and `services/` for binaries. `protoc` must be installed as a prerequisite (`brew install protobuf` on macOS).

**Primary recommendation:** Use `tonic-prost-build` (not `tonic-build`) for proto compilation in the `madome-proto` crate's `build.rs`. Define a Health RPC in each service proto file using `google.protobuf.Empty` from well-known types via `prost-types`. Keep the gateway thin: REST route -> gRPC client call -> JSON response.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- 3 shared crates as defined in PROJECT.md: `madome-proto`, `madome-core`, `madome-common`
- No centralized entity/migration crate -- per-service `schema/` and `migration/` folders instead
- Research doc's 4-crate recommendation (entity + migration) explicitly rejected
- Routing verification only -- each service gets a Health RPC, no business RPCs in Phase 1
- `google.protobuf.Empty` used instead of custom Empty message
- 3 proto files: `auth.proto`, `catalog.proto`, `user.proto` (no `file.proto` -- v2 scope)
- Proto files located at workspace root `proto/` directory (single source of truth)
- Package naming: `madome.auth`, `madome.catalog`, `madome.user`
- All 3 backend services (auth, catalog, user) implemented as Health RPC stubs
- DB-free: pure in-memory stubs, no PostgreSQL connection in Phase 1
- Gateway exposes its own `/health` endpoint
- Gateway routes health checks to each backend service via gRPC
- Success responses: flat JSON (data at top level, no envelope)
- Error responses: `{ "error": "not_found", "message": "Book not found" }`
- Environment variables only (no figment, no config files)
- Static service discovery: Gateway reads gRPC addresses from env vars (AUTH_GRPC_ADDR, CATALOG_GRPC_ADDR, USER_GRPC_ADDR)
- tracing + tracing-subscriber for console logging
- UUIDv7 request_id generated at gateway, propagated via gRPC metadata
- No OpenTelemetry SDK in Phase 1
- `madome-core`: Error types only (AppError with gRPC Status <-> HTTP mapping)
- `madome-common`: tracing initialization + env var parsing utilities
- `crates/` + `services/` directory layout
- All v1 services as workspace members: gateway + auth + catalog + user + 3 shared crates (7 total)
- `schema/` + `migration/` directories created now (empty, structure scaffolding)
- `[workspace.dependencies]` for centralized version management
- Integration tests: Gateway + stub services running together, verified via HTTP requests
- `cargo run` only -- no Docker, no docker-compose
- URL convention: /v1/ prefix, no /api prefix

### Claude's Discretion
- Stub service execution approach (individual binaries vs all-in-one dev binary)
- gRPC Status -> HTTP status code mapping specifics
- common.proto content
- Exact tracing-subscriber configuration
- Port number conventions

### Deferred Ideas (OUT OF SCOPE)
- `file.proto` definition -- v2 scope (FILE-01/02/03)
- CI/CD pipeline (GitHub Actions) -- after real features are implemented
- Docker/docker-compose -- when DB connectivity is needed
- OpenTelemetry SDK integration -- after basic tracing is proven
- Domain types (BookId, UserId newtypes) in madome-core -- when first service needs them
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| GATE-01 | Gateway exposes REST API with route registration for all services | axum 0.8 Router with nested route groups per service; `/v1/` prefix; gateway `/health` endpoint + per-service health routes |
| GATE-02 | Gateway routes REST requests to internal services via gRPC | tonic gRPC client in AppState; `tonic::transport::Channel` for connections; REST handler calls gRPC client and translates response to JSON |
</phase_requirements>

## Standard Stack

### Core (Phase 1 Only)

| Library | Version | Purpose | Why Standard | Confidence |
|---------|---------|---------|--------------|------------|
| **tokio** | 1.50.0 | Async runtime | Required by axum, tonic, every async crate | HIGH |
| **axum** | 0.8.8 | REST framework (gateway) | Tower-native, shared middleware with tonic | HIGH |
| **tonic** | 0.14.5 | gRPC framework | Standard Rust gRPC, server + client | HIGH |
| **tonic-prost** | 0.14.5 | Prost codec for tonic | Bridges tonic and prost for message encoding | HIGH |
| **tonic-prost-build** | 0.14.5 | Proto compilation (build dep) | Replaces tonic-build for protobuf workflows in 0.14 | HIGH |
| **prost** | 0.14.3 | Protobuf types (generated code) | Required by tonic-generated code | HIGH |
| **prost-types** | 0.14.3 | Well-known protobuf types | Provides `google.protobuf.Empty` without compiling well-known types | HIGH |
| **serde** | 1.0.228 | Serialization framework | Required for JSON request/response bodies | HIGH |
| **serde_json** | 1.0.149 | JSON serialization | REST API request/response bodies | HIGH |
| **thiserror** | 2.0.18 | Error type derives | For AppError in madome-core | HIGH |
| **tracing** | 0.1.44 | Structured logging | Ecosystem standard instrumentation | HIGH |
| **tracing-subscriber** | 0.3.23 | Log output formatting | Console logging with env-filter | HIGH |
| **uuid** | 1.22.0 | UUID generation | UUIDv7 for request_id | HIGH |
| **tower** | 0.5.3 | Middleware framework | Shared between axum and tonic | HIGH |
| **tower-http** | 0.6.8 | HTTP middleware | Trace layer for request logging | HIGH |

### CRITICAL: tonic 0.14 Ecosystem Change

The STACK.md recommends `tonic-build` + `prost-build` as build dependencies. **This is outdated for tonic 0.14.** The ecosystem has been restructured:

| Old (pre-0.14) | New (0.14+) | Purpose |
|----------------|-------------|---------|
| `prost` (direct dep) | `tonic-prost` | Codec for encoding/decoding protobuf messages |
| `tonic-build` + `prost-build` (build deps) | `tonic-prost-build` | Proto compilation in build.rs |

`tonic-build` 0.14 explicitly says: "For protobuf compilation via prost, use the `tonic-prost-build` crate instead." The `tonic::include_proto!` macro is still available in tonic itself.

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Custom Health RPC per proto | `tonic-health` (0.14.5) | tonic-health uses the standard `grpc.health.v1.Health` service. Custom Health RPCs give more control and match the "one proto per service" approach. Since Phase 1 is about verifying routing, custom RPCs better demonstrate the full REST->gRPC->Response chain. |
| `prost-types` for Empty | `compile_well_known_types(true)` | Compiling well-known types has historically caused issues (empty google.protobuf.rs files, namespace conflicts). Using `prost-types` is simpler and avoids these pitfalls. |
| Individual service binaries | All-in-one dev binary | Individual binaries match production topology and make port configuration explicit. All-in-one reduces `cargo run` commands but adds complexity. **Recommendation: individual binaries** -- simpler to reason about, matches production, use a shell script or `cargo-make` for convenience. |

### Prerequisites

```bash
# protoc is REQUIRED for tonic-prost-build
brew install protobuf

# Verify
protoc --version  # should be libprotoc 3.x or higher
```

### Installation (workspace Cargo.toml)

```toml
[workspace]
resolver = "3"
members = [
    "crates/madome-proto",
    "crates/madome-core",
    "crates/madome-common",
    "services/gateway",
    "services/auth",
    "services/catalog",
    "services/user",
]

[workspace.package]
version = "0.1.0"
edition = "2024"

[workspace.dependencies]
# Async runtime
tokio = { version = "1.50", features = ["rt-multi-thread", "macros", "net", "signal"] }

# REST framework
axum = "0.8"
tower = "0.5"
tower-http = { version = "0.6", features = ["trace"] }

# gRPC
tonic = "0.14"
tonic-prost = "0.14"
prost = "0.14"
prost-types = "0.14"

# gRPC build
tonic-prost-build = "0.14"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Error handling
thiserror = "2"

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }

# Utilities
uuid = { version = "1", features = ["v7", "serde"] }

# Internal crates
madome-proto = { path = "crates/madome-proto" }
madome-core = { path = "crates/madome-core" }
madome-common = { path = "crates/madome-common" }
```

## Architecture Patterns

### Recommended Project Structure

```
madome/
  Cargo.toml                    # Virtual workspace manifest (resolver = "3")
  proto/                        # Shared .proto definitions (workspace root)
    auth.proto                  # madome.auth package -- Health RPC
    catalog.proto               # madome.catalog package -- Health RPC
    user.proto                  # madome.user package -- Health RPC
  crates/
    madome-proto/               # Proto compilation crate
      Cargo.toml
      build.rs                  # tonic_prost_build::configure()...
      src/lib.rs                # tonic::include_proto!() re-exports
    madome-core/                # Error types, gRPC<->HTTP mapping
      Cargo.toml
      src/
        lib.rs
        error.rs                # AppError + IntoResponse + From<tonic::Status>
    madome-common/              # Tracing init, env parsing
      Cargo.toml
      src/
        lib.rs
        tracing.rs              # init_tracing() function
        env.rs                  # required_env(), optional_env()
  services/
    gateway/
      Cargo.toml
      src/
        main.rs                 # tokio::main, starts axum server
        state.rs                # AppState with gRPC clients
        routes/
          mod.rs                # Router assembly with /v1/ prefix
          health.rs             # GET /health (gateway own) + service health routes
    auth/
      Cargo.toml
      schema/                   # Empty -- structure scaffolding
      migration/                # Empty -- structure scaffolding
      src/
        main.rs                 # tokio::main, starts tonic server
        service.rs              # Health RPC implementation
    catalog/
      Cargo.toml
      schema/                   # Empty -- structure scaffolding
      migration/                # Empty -- structure scaffolding
      src/
        main.rs
        service.rs
    user/
      Cargo.toml
      schema/                   # Empty -- structure scaffolding
      migration/                # Empty -- structure scaffolding
      src/
        main.rs
        service.rs
```

### Pattern 1: Shared Proto Crate with tonic-prost-build

**What:** Single `madome-proto` crate compiles all `.proto` files and re-exports generated modules. All services depend on this one crate.

**When to use:** Always in a Cargo workspace with multiple gRPC services.

**Example:**

```rust
// crates/madome-proto/build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &[
                "../../proto/auth.proto",
                "../../proto/catalog.proto",
                "../../proto/user.proto",
            ],
            &["../../proto"],
        )?;
    Ok(())
}
```

```rust
// crates/madome-proto/src/lib.rs
pub mod auth {
    tonic::include_proto!("madome.auth");
}
pub mod catalog {
    tonic::include_proto!("madome.catalog");
}
pub mod user {
    tonic::include_proto!("madome.user");
}
```

**Note on proto paths:** The `build.rs` in `crates/madome-proto/` references proto files relative to its own directory, so `../../proto/` reaches the workspace root. The include paths (`&["../../proto"]`) tell protoc where to find imports (e.g., `google/protobuf/empty.proto` is resolved by protoc from its own include path automatically when using well-known types).

### Pattern 2: Proto Definition with google.protobuf.Empty

**What:** Each service proto defines a Health RPC using `google.protobuf.Empty` from protobuf well-known types.

**Example:**

```protobuf
// proto/auth.proto
syntax = "proto3";
package madome.auth;

import "google/protobuf/empty.proto";

service AuthService {
  rpc Health(google.protobuf.Empty) returns (google.protobuf.Empty);
}
```

```protobuf
// proto/catalog.proto
syntax = "proto3";
package madome.catalog;

import "google/protobuf/empty.proto";

service CatalogService {
  rpc Health(google.protobuf.Empty) returns (google.protobuf.Empty);
}
```

```protobuf
// proto/user.proto
syntax = "proto3";
package madome.user;

import "google/protobuf/empty.proto";

service UserService {
  rpc Health(google.protobuf.Empty) returns (google.protobuf.Empty);
}
```

On the Rust side, `google.protobuf.Empty` maps to `prost_types::Empty` (from the `prost-types` crate). No need to compile well-known types yourself.

**Note on common.proto:** Since Phase 1 only has Health RPCs using `google.protobuf.Empty`, there is no need for a `common.proto` file yet. It should be introduced when business RPCs need shared types (pagination, etc.) in later phases.

### Pattern 3: Gateway REST-to-gRPC Translation

**What:** Gateway holds gRPC client connections in AppState and translates REST requests into gRPC calls.

**Example:**

```rust
// services/gateway/src/state.rs
use madome_proto::auth::auth_service_client::AuthServiceClient;
use madome_proto::catalog::catalog_service_client::CatalogServiceClient;
use madome_proto::user::user_service_client::UserServiceClient;
use tonic::transport::Channel;

#[derive(Clone)]
pub struct AppState {
    pub auth_client: AuthServiceClient<Channel>,
    pub catalog_client: CatalogServiceClient<Channel>,
    pub user_client: UserServiceClient<Channel>,
}
```

```rust
// services/gateway/src/routes/health.rs
use axum::{extract::State, Json};
use serde::Serialize;

use crate::state::AppState;
use madome_core::error::AppError;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub services: ServiceHealth,
}

#[derive(Serialize)]
pub struct ServiceHealth {
    pub auth: &'static str,
    pub catalog: &'static str,
    pub user: &'static str,
}

pub async fn gateway_health() -> Json<HealthResponse> {
    // Gateway's own health -- always OK if the process is running
    Json(HealthResponse {
        status: "ok",
        services: ServiceHealth {
            auth: "unknown",
            catalog: "unknown",
            user: "unknown",
        },
    })
}

pub async fn service_health(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, AppError> {
    let empty = prost_types::Empty {};

    let mut auth = state.auth_client.clone();
    let mut catalog = state.catalog_client.clone();
    let mut user = state.user_client.clone();

    // Call Health RPC on each service
    let auth_status = auth.health(empty.clone()).await;
    let catalog_status = catalog.health(empty.clone()).await;
    let user_status = user.health(empty).await;

    Ok(Json(HealthResponse {
        status: "ok",
        services: ServiceHealth {
            auth: if auth_status.is_ok() { "ok" } else { "unavailable" },
            catalog: if catalog_status.is_ok() { "ok" } else { "unavailable" },
            user: if user_status.is_ok() { "ok" } else { "unavailable" },
        },
    }))
}
```

### Pattern 4: AppError with gRPC-to-HTTP Mapping

**What:** Central error type in `madome-core` that converts between `tonic::Status` and HTTP responses.

**gRPC Status to HTTP Status mapping (Claude's discretion area):**

| gRPC Code | HTTP Status | Use Case |
|-----------|-------------|----------|
| OK | 200 | Success |
| INVALID_ARGUMENT | 400 | Bad request data |
| UNAUTHENTICATED | 401 | Missing/invalid credentials |
| PERMISSION_DENIED | 403 | Insufficient permissions |
| NOT_FOUND | 404 | Resource not found |
| ALREADY_EXISTS | 409 | Conflict / duplicate |
| FAILED_PRECONDITION | 412 | Precondition failed |
| RESOURCE_EXHAUSTED | 429 | Rate limited |
| INTERNAL | 500 | Server error |
| UNAVAILABLE | 503 | Service unavailable |
| UNIMPLEMENTED | 501 | Not implemented |
| DEADLINE_EXCEEDED | 504 | Timeout |

This mapping follows the standard Google API design guide.

**Example:**

```rust
// crates/madome-core/src/error.rs
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: String,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("unavailable: {0}")]
    Unavailable(String),

    #[error("internal: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code) = match &self {
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            AppError::Unavailable(_) => (StatusCode::SERVICE_UNAVAILABLE, "unavailable"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal"),
        };

        let body = ErrorBody {
            error: error_code.to_string(),
            message: self.to_string(),
        };

        (status, axum::Json(body)).into_response()
    }
}

impl From<tonic::Status> for AppError {
    fn from(status: tonic::Status) -> Self {
        match status.code() {
            tonic::Code::NotFound => AppError::NotFound(status.message().to_string()),
            tonic::Code::InvalidArgument => AppError::BadRequest(status.message().to_string()),
            tonic::Code::Unavailable => AppError::Unavailable(status.message().to_string()),
            _ => AppError::Internal(status.message().to_string()),
        }
    }
}
```

### Pattern 5: Service Stub with Health RPC

**What:** Each backend service runs a tonic gRPC server that implements just the Health RPC.

**Example:**

```rust
// services/auth/src/service.rs
use madome_proto::auth::auth_service_server::AuthService;
use tonic::{Request, Response, Status};

pub struct AuthServiceImpl;

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
    async fn health(
        &self,
        _request: Request<prost_types::Empty>,
    ) -> Result<Response<prost_types::Empty>, Status> {
        Ok(Response::new(prost_types::Empty {}))
    }
}
```

```rust
// services/auth/src/main.rs
use tonic::transport::Server;
use madome_proto::auth::auth_service_server::AuthServiceServer;

mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    madome_common::tracing::init_tracing("auth");

    let addr = madome_common::env::required_env("AUTH_LISTEN_ADDR")
        .parse()
        .expect("invalid AUTH_LISTEN_ADDR");

    tracing::info!(%addr, "auth service starting");

    Server::builder()
        .add_service(AuthServiceServer::new(service::AuthServiceImpl))
        .serve(addr)
        .await?;

    Ok(())
}
```

### Pattern 6: Request ID Propagation via gRPC Metadata

**What:** Gateway generates a UUIDv7 request_id per incoming request and propagates it to gRPC services via metadata.

**Example:**

```rust
// In gateway, when making a gRPC call:
let request_id = uuid::Uuid::now_v7().to_string();
let mut request = tonic::Request::new(prost_types::Empty {});
request.metadata_mut().insert(
    "x-request-id",
    request_id.parse().expect("valid metadata value"),
);
```

```rust
// In service, reading the request_id from metadata:
fn extract_request_id(request: &tonic::Request<impl std::any::Any>) -> Option<String> {
    request
        .metadata()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}
```

### Pattern 7: Tracing Initialization (madome-common)

**What:** Shared tracing setup function used by all services.

**Example:**

```rust
// crates/madome-common/src/tracing.rs
use tracing_subscriber::{fmt, EnvFilter};

pub fn init_tracing(service_name: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("{service_name}=debug,madome=debug,info")));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .init();
}
```

### Pattern 8: Environment Variable Parsing (madome-common)

**What:** Simple utility functions for reading required/optional env vars with clear error messages.

**Example:**

```rust
// crates/madome-common/src/env.rs
use std::env;

pub fn required_env(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("{key} environment variable is required"))
}

pub fn optional_env(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}
```

### Anti-Patterns to Avoid

- **Fat Gateway:** Never put business logic in the gateway. It should ONLY do: route registration, request_id generation, REST-to-gRPC translation, and response serialization. Phase 1 scope makes this easy to enforce.
- **Proto compilation in every service crate:** Always compile protos once in `madome-proto`. Services depend on the compiled crate, not raw proto files.
- **Hardcoded service addresses:** Use env vars (AUTH_GRPC_ADDR, etc.) even in Phase 1 stubs. This pattern must be established from the start.
- **compile_well_known_types(true):** Use `prost-types` crate instead. Compiling well-known types has historically caused namespace issues.
- **Using `tonic-build` directly:** It no longer provides `compile_protos()` for protobuf. Use `tonic-prost-build`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Proto compilation | Custom protoc invocation or code generation | `tonic-prost-build` in build.rs | Handles protoc invocation, code generation, server/client trait generation |
| gRPC health checking | Custom health check protocol | Health RPC in proto (or `tonic-health` for standard) | Standard pattern; proto-defined for full routing verification |
| HTTP error responses | Manual status code + body construction | `IntoResponse` impl on AppError | axum's trait system handles the conversion cleanly |
| Structured logging | println! or log crate | `tracing` + `tracing-subscriber` | Ecosystem standard, async-aware, span-based, used internally by axum and tonic |
| UUID generation | Custom timestamp-based IDs | `uuid` crate with v7 feature | UUIDv7 is time-sortable, cryptographically generated, standard |

## Common Pitfalls

### Pitfall 1: Proto Path Confusion in Workspace

**What goes wrong:** `tonic-prost-build` in `crates/madome-proto/build.rs` cannot find proto files at the workspace root because paths are relative to the crate directory, not the workspace root.
**Why it happens:** Cargo runs `build.rs` with the working directory set to the crate root (`crates/madome-proto/`), not the workspace root.
**How to avoid:** Use `../../proto/` as the proto file path in `build.rs`. Verify with `CARGO_MANIFEST_DIR` environment variable if needed. Add `cargo:rerun-if-changed=../../proto/` to rebuild on proto changes.
**Warning signs:** Build errors about missing proto files; proto changes not triggering recompilation.

### Pitfall 2: protoc Not Installed

**What goes wrong:** Build fails with "Could not find `protoc` installation" or similar error.
**Why it happens:** `prost-build` (used by `tonic-prost-build`) requires the `protoc` compiler to be installed on the system. It no longer bundles or compiles protoc.
**How to avoid:** Document `brew install protobuf` as a prerequisite. Consider adding a check in a workspace-level script.
**Warning signs:** First-time `cargo build` fails immediately in the madome-proto crate.

### Pitfall 3: tonic/prost Version Mismatch

**What goes wrong:** Compilation errors about incompatible types between generated code and tonic runtime.
**Why it happens:** tonic 0.14.x requires prost 0.14.x. Mixing major/minor versions causes type incompatibility in generated server/client traits.
**How to avoid:** Pin all tonic-related crates to the same minor version family via `[workspace.dependencies]`. Current family: `0.14.x` for tonic, tonic-prost, tonic-prost-build, prost, prost-types.
**Warning signs:** Type errors mentioning `prost::Message` or `tonic::codec::ProstCodec` in generated code.

### Pitfall 4: Cargo Feature Unification in Workspace

**What goes wrong:** One service's feature flags affect other services' compilation. Binary sizes unexpectedly large.
**Why it happens:** Cargo workspaces unify features across the dependency graph. If gateway uses `tokio/full` but auth only needs `tokio/rt`, both get `tokio/full`.
**How to avoid:** Use `resolver = "3"` (edition 2024 default for virtual manifests, but must be explicit). Be specific about features in each crate's own Cargo.toml rather than enabling everything in workspace.dependencies.
**Warning signs:** Unexpectedly large binaries; compile-time features you did not request showing up in `cargo tree`.

### Pitfall 5: gRPC Client Channel Not Reused

**What goes wrong:** Gateway creates a new gRPC channel per request instead of reusing connections, causing connection storms.
**Why it happens:** Misunderstanding tonic's `Channel` type -- it is cheap to clone (internally reference-counted) and manages connection pooling.
**How to avoid:** Create `Channel` once during startup, wrap in client structs (`AuthServiceClient<Channel>`), store in AppState, and clone the client per request. The clone shares the underlying channel.
**Warning signs:** Many TCP connections from gateway to service processes; high connection setup latency.

### Pitfall 6: Missing `cargo:rerun-if-changed` in build.rs

**What goes wrong:** Proto file changes do not trigger recompilation of the `madome-proto` crate.
**Why it happens:** By default, Cargo does not watch files outside the crate directory for changes.
**How to avoid:** Add `println!("cargo:rerun-if-changed=../../proto/");` in `build.rs` for each proto file or directory.
**Warning signs:** Editing a `.proto` file and `cargo build` reporting nothing to compile.

## Code Examples

### Complete Gateway Router Assembly

```rust
// services/gateway/src/routes/mod.rs
use axum::{routing::get, Router};
use crate::state::AppState;

mod health;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::gateway_health))
        .nest("/v1", v1_routes())
        .with_state(state)
}

fn v1_routes() -> Router<AppState> {
    Router::new()
        .route("/health/services", get(health::service_health))
}
```

### Complete Gateway Main

```rust
// services/gateway/src/main.rs
use std::net::SocketAddr;
use tonic::transport::Channel;
use madome_proto::auth::auth_service_client::AuthServiceClient;
use madome_proto::catalog::catalog_service_client::CatalogServiceClient;
use madome_proto::user::user_service_client::UserServiceClient;

mod routes;
mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    madome_common::tracing::init_tracing("gateway");

    let auth_addr = madome_common::env::required_env("AUTH_GRPC_ADDR");
    let catalog_addr = madome_common::env::required_env("CATALOG_GRPC_ADDR");
    let user_addr = madome_common::env::required_env("USER_GRPC_ADDR");

    let auth_client = AuthServiceClient::connect(auth_addr).await?;
    let catalog_client = CatalogServiceClient::connect(catalog_addr).await?;
    let user_client = UserServiceClient::connect(user_addr).await?;

    let state = state::AppState {
        auth_client,
        catalog_client,
        user_client,
    };

    let app = routes::create_router(state);

    let addr: SocketAddr = madome_common::env::optional_env("GATEWAY_ADDR", "0.0.0.0:3000")
        .parse()
        .expect("invalid GATEWAY_ADDR");

    tracing::info!(%addr, "gateway starting");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

### Port Number Convention (Claude's Discretion)

| Service | Default Port | Env Var |
|---------|-------------|---------|
| Gateway (REST) | 3000 | GATEWAY_ADDR |
| Auth (gRPC) | 50051 | AUTH_LISTEN_ADDR / AUTH_GRPC_ADDR |
| Catalog (gRPC) | 50052 | CATALOG_LISTEN_ADDR / CATALOG_GRPC_ADDR |
| User (gRPC) | 50053 | USER_LISTEN_ADDR / USER_GRPC_ADDR |

The gRPC ports follow the convention of starting at 50051 (the gRPC default). Each service has a `*_LISTEN_ADDR` (used by the service to bind) and a `*_GRPC_ADDR` (used by the gateway to connect), which typically have the same value in local development (e.g., `http://127.0.0.1:50051`).

### Stub Execution Approach (Claude's Discretion)

**Recommendation: Individual binaries.** Each service is its own binary crate in the workspace. This matches production topology, makes port assignment explicit, and simplifies debugging. Use a shell script for convenience:

```bash
#!/bin/bash
# scripts/dev.sh -- Start all services for local development

export AUTH_LISTEN_ADDR="0.0.0.0:50051"
export AUTH_GRPC_ADDR="http://127.0.0.1:50051"
export CATALOG_LISTEN_ADDR="0.0.0.0:50052"
export CATALOG_GRPC_ADDR="http://127.0.0.1:50052"
export USER_LISTEN_ADDR="0.0.0.0:50053"
export USER_GRPC_ADDR="http://127.0.0.1:50053"
export GATEWAY_ADDR="0.0.0.0:3000"

cargo run --bin auth &
cargo run --bin catalog &
cargo run --bin user &
sleep 2  # Wait for gRPC services to start
cargo run --bin gateway &

wait
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `tonic-build` + `prost-build` for proto compilation | `tonic-prost-build` (single crate) | tonic 0.14 | Must use `tonic_prost_build::configure()` instead of `tonic_build::configure()` |
| Direct `prost` dependency for message types | `tonic-prost` as codec bridge | tonic 0.14 | Add `tonic-prost` to dependencies; `prost` is still a transitive dependency but `tonic-prost` provides the codec |
| Cargo workspace `resolver = "2"` | `resolver = "3"` for edition 2024 | Rust 1.84 / Edition 2024 | Must be explicit in virtual workspace manifests |
| `axum::Server::bind().serve()` | `axum::serve(TcpListener, Router)` | axum 0.7+ | Simpler API; `hyper::Server` is no longer used directly |

**Deprecated/outdated:**
- `tonic_build::compile_protos()` -- Use `tonic_prost_build::compile_protos()` instead
- `hyper::Server` for axum -- Use `axum::serve()` with `TcpListener`
- `resolver = "2"` in workspace -- Use `resolver = "3"` for edition 2024

## Open Questions

1. **tonic-prost vs prost direct dependency**
   - What we know: `tonic-prost` provides `ProstCodec`, `ProstEncoder`, `ProstDecoder`. Generated code from `tonic-prost-build` references these types.
   - What's unclear: Whether services need `prost` and `prost-types` as direct dependencies or if they are re-exported through `tonic-prost`. The generated code uses `prost::Message` trait which requires `prost` to be in scope.
   - Recommendation: Add both `tonic-prost` AND `prost` + `prost-types` to workspace dependencies. `prost` is needed for `Message` trait; `prost-types` for `Empty`.

2. **build.rs proto include path for google well-known types**
   - What we know: `protoc` has a built-in include path for well-known types (`google/protobuf/empty.proto`). `tonic-prost-build` passes include paths to protoc.
   - What's unclear: Whether the `../../proto` include path is sufficient or if we need to add protoc's well-known types include path explicitly.
   - Recommendation: Start with just `&["../../proto"]` as the include path. protoc resolves well-known type imports from its own install path automatically. If it fails, add protoc's include directory (typically `/opt/homebrew/include` on macOS with Homebrew).

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test framework + tokio test runtime |
| Config file | None -- Cargo handles test discovery |
| Quick run command | `cargo test -p gateway` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements to Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| GATE-01 | Gateway exposes REST API with routes for all services | integration | `cargo test -p gateway --test health_integration` | No -- Wave 0 |
| GATE-02 | Gateway routes REST request to internal gRPC service and returns response | integration | `cargo test -p gateway --test health_integration` | No -- Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo build --workspace` (compilation check) + `cargo test --workspace` (if tests exist)
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full workspace compilation + integration tests green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `services/gateway/tests/health_integration.rs` -- integration test: start stub services + gateway, verify HTTP health endpoints return expected JSON
- [ ] Test harness utilities: function to spawn tonic server on random port, function to start axum server on random port, port discovery for test isolation
- [ ] `protoc` installation verification -- add note in test setup or CI prerequisite

**Integration test pattern:**

```rust
// services/gateway/tests/health_integration.rs
// 1. Start auth/catalog/user stub services on random ports
// 2. Start gateway pointing to those ports
// 3. Send HTTP GET /health to gateway
// 4. Verify 200 response with expected JSON structure
// 5. Send HTTP GET /v1/health/services to gateway
// 6. Verify all services report "ok"
```

The integration test spawns real tonic servers and a real axum server in the same test process using `tokio::spawn`, uses random port assignment (bind to `:0` and extract assigned port) for test isolation, and verifies the full REST-to-gRPC chain.

## Sources

### Primary (HIGH confidence)
- [tonic-build docs](https://docs.rs/tonic-build/latest/tonic_build/) -- confirmed tonic-build no longer provides compile_protos for protobuf
- [tonic-prost-build docs](https://docs.rs/tonic-prost-build/latest/tonic_prost_build/) -- current API for proto compilation
- [tonic-prost docs](https://docs.rs/tonic-prost/latest/tonic_prost/) -- codec bridge between tonic and prost
- [tonic-health docs](https://docs.rs/tonic-health/latest/tonic_health/) -- standard gRPC health checking (not used, but evaluated)
- [tonic official helloworld tutorial](https://github.com/hyperium/tonic/blob/master/examples/helloworld-tutorial.md) -- confirms tonic-prost-build usage pattern
- [tonic health server example](https://github.com/hyperium/tonic/blob/master/examples/src/health/server.rs) -- HealthReporter API
- [axum error handling example](https://github.com/tokio-rs/axum/blob/main/examples/error-handling/src/main.rs) -- IntoResponse pattern
- [Cargo workspace docs](https://doc.rust-lang.org/cargo/reference/workspaces.html) -- resolver = "3" requirement for virtual workspaces
- [Rust Edition 2024 resolver](https://doc.rust-lang.org/edition-guide/rust-2024/cargo-resolver.html) -- resolver version 3 is edition 2024 default
- [gRPC status code mapping](https://github.com/grpc/grpc/blob/master/doc/http-grpc-status-mapping.md) -- official gRPC-to-HTTP mapping
- crates.io version verification (2026-03-21): tonic 0.14.5, axum 0.8.8, prost 0.14.3, tokio 1.50.0, all verified via `cargo search`

### Secondary (MEDIUM confidence)
- [gRPC code to HTTP status mapping gist](https://gist.github.com/hamakn/708b9802ca845eb59f3975dbb3ae2a01) -- community reference for status code mapping
- [tonic workspace proto issue #484](https://github.com/hyperium/tonic/issues/484) -- workspace proto path pitfalls
- [well-known types issue #757](https://github.com/hyperium/tonic/issues/757) -- prost-types vs compile_well_known_types guidance
- [axum-tonic multiplexing discussion](https://github.com/tokio-rs/axum/discussions/1840) -- REST + gRPC on same port (not needed for Phase 1 but useful context)

### Tertiary (LOW confidence)
- None -- all findings verified against primary or secondary sources.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all crate versions verified via `cargo search` on 2026-03-21; tonic ecosystem restructuring confirmed via official docs
- Architecture: HIGH -- standard patterns from tonic/axum documentation and examples
- Pitfalls: HIGH -- proto path issues, protoc requirement, and version alignment are well-documented in tonic GitHub issues

**Research date:** 2026-03-21
**Valid until:** 2026-04-21 (30 days -- stable ecosystem, no breaking changes expected)

---
*Phase: 01-foundation-and-gateway-infrastructure*
*Research completed: 2026-03-21*
