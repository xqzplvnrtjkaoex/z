# Architecture Research

> **Terminology note:** This document uses "work" throughout. The project canonical term is **"book"**. See PROJECT.md for current architecture decisions -- some recommendations here (e.g., madome-entity/madome-migration centralized crates, cross-service DB sync queue) have been superseded by per-service schema/migration folders, canonical_id denormalization, and a 3-crate split (proto, core, common).

**Domain:** Rust microservice content mirroring platform (manga aggregation)
**Researched:** 2026-03-21
**Confidence:** HIGH (well-established patterns for Rust workspace + tonic + axum + sea-orm)

## System Overview

```
                          External Clients
                               |
                          [ nginx :443 ]
                         /       |       \
                        /        |        \
               REST API      Images     Scraper REST
                  |            |            |
            +-----------+      |      +-----------+
            |  Gateway  |      |      |  Gateway  |  (API Key auth)
            |  (axum)   |      |      |  (axum)   |
            +-----+-----+      |      +-----+-----+
                  |             |            |
          gRPC (tonic)    auth_request   gRPC (tonic)
          /    |    \         |              |
    +------+ +-------+ +------+        +-------+
    | Auth | | Catal.| | User |        | Catal.|
    +------+ +-------+ +------+        +-------+
                  |
            publish verify
                  |
            +------+
            | File |  <-- nginx serves images directly
            +------+       after auth_request passes
                |
          Local Filesystem

    Separate Server:
    +---------+
    | Scraper | --> REST --> nginx --> Gateway --> Catalog
    +---------+ --> REST --> nginx --> File Service
```

### Component Responsibilities

| Component | Responsibility | Protocol | Database |
|-----------|----------------|----------|----------|
| **nginx** | TLS termination, reverse proxy, auth_request subrequests, static image serving | HTTPS | None |
| **Gateway** | REST API entry point, JWT verification/refresh, route to internal gRPC services | REST in, gRPC out | None (stateless) |
| **Auth** | Passkey registration/authentication, session CRUD, JWT issuance | gRPC | Own PostgreSQL schema |
| **Catalog** | Work CRUD, tag management, publish workflow, work renewal/graph relations | gRPC | Own PostgreSQL schema |
| **User** | Taste preferences (like/dislike), reading histories | gRPC | Own PostgreSQL schema |
| **File** | Image upload endpoint, filesystem management, page count validation | REST (behind nginx) | Minimal metadata in own schema |
| **Scraper** | External source polling, new work detection, decreasing-frequency update checks | REST client | Own state tracking schema |

## Recommended Project Structure

```
madome/
├── Cargo.toml                    # Virtual workspace root
├── proto/                        # All .proto definitions (single source of truth)
│   ├── auth.proto
│   ├── catalog.proto
│   ├── user.proto
│   └── common.proto              # Shared message types (pagination, errors, timestamps)
├── crates/
│   ├── madome-proto/             # Proto compilation crate (build.rs + include_proto!)
│   │   ├── Cargo.toml
│   │   ├── build.rs
│   │   └── src/
│   │       └── lib.rs
│   ├── madome-core/              # Shared domain types, error types, config
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config.rs         # Shared config loading (envconfig/figment pattern)
│   │       ├── error.rs          # Common error types + gRPC Status mapping
│   │       └── auth.rs           # JWT claims struct, token validation logic
│   ├── madome-entity/            # sea-orm entity definitions (shared across DB-using services)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── work.rs           # Catalog entities
│   │       ├── tag.rs
│   │       ├── user.rs           # User entities
│   │       ├── session.rs        # Auth entities
│   │       └── ...
│   └── madome-migration/         # sea-orm migrations (all schemas)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           └── m20260101_000001_initial.rs
├── services/
│   ├── gateway/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── routes/           # axum route handlers
│   │       │   ├── mod.rs
│   │       │   ├── auth.rs
│   │       │   ├── catalog.rs
│   │       │   └── user.rs
│   │       ├── middleware/        # JWT extraction, auth, logging
│   │       │   ├── mod.rs
│   │       │   └── auth.rs
│   │       ├── grpc_clients.rs   # Pooled gRPC client connections
│   │       └── state.rs          # AppState (gRPC clients, JWT config)
│   ├── auth/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── service.rs        # gRPC service implementation
│   │       ├── passkey.rs        # webauthn-rs integration
│   │       ├── session.rs        # Session management
│   │       └── jwt.rs            # Token issuance + caching
│   ├── catalog/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── service.rs        # gRPC service implementation
│   │       ├── repository.rs     # sea-orm queries
│   │       ├── publish.rs        # Publish workflow (verify page counts)
│   │       └── renewal.rs        # Work ID renewal, graph relations
│   ├── user/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── service.rs
│   │       └── repository.rs
│   ├── file/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── routes.rs         # axum REST endpoints (upload)
│   │       ├── storage.rs        # Filesystem operations
│   │       └── state.rs
│   └── scraper/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── scheduler.rs      # Decreasing-frequency poll logic
│           ├── client.rs         # REST client to Gateway + File service
│           ├── source/
│           │   ├── mod.rs
│           │   └── hitomi.rs     # hitomi_la crate integration
│           └── pipeline.rs       # Detect -> Upload metadata -> Upload images -> Publish
├── config/                       # Deployment configs
│   ├── nginx.conf
│   └── docker-compose.yml
└── deploy/                       # Dockerfiles per service
    ├── Dockerfile.gateway
    ├── Dockerfile.auth
    └── ...
```

### Structure Rationale

- **`proto/` at workspace root:** Single source of truth for all service contracts. All services depend on `madome-proto` crate, which compiles these via `tonic-build` in its `build.rs`. Changes to proto files trigger recompilation of only the proto crate and its dependents.

- **`crates/` for shared libraries:** Separates shared code from service binaries. Four shared crates cover the major cross-cutting concerns:
  - `madome-proto`: Generated gRPC code (server traits + client stubs)
  - `madome-core`: Domain types, config, error handling -- things every service needs
  - `madome-entity`: sea-orm entity definitions shared by any service that touches the database
  - `madome-migration`: Single migration runner for the entire database (important -- see Database section)

- **`services/` for binaries:** Each service is its own binary crate. Clean separation. Each has a focused `main.rs` that initializes its dependencies and starts listening.

- **Separation of `routes/` and `service.rs`:** Gateway uses `routes/` (axum REST handlers). Backend services use `service.rs` (tonic gRPC trait implementations). This makes the protocol boundary explicit.

## Architectural Patterns

### Pattern 1: Shared Proto Crate

**What:** A single crate (`madome-proto`) compiles all `.proto` files and re-exports generated types. All services depend on this one crate rather than each compiling protos independently.

**When to use:** Always in a Rust workspace with multiple gRPC services.

**Trade-offs:** Slightly longer compile when any proto changes (all generated code recompiles), but prevents version drift between services and eliminates duplicate build.rs logic.

**Example:**

```rust
// crates/madome-proto/build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
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
pub mod common {
    tonic::include_proto!("madome.common");
}
```

### Pattern 2: Gateway as REST-to-gRPC Translator

**What:** The Gateway holds pooled gRPC client connections in its AppState and translates incoming REST requests into gRPC calls. It performs JWT verification as middleware, then forwards the authenticated user identity as gRPC metadata.

**When to use:** When you want a single external API surface but internal type-safe gRPC communication.

**Trade-offs:** Gateway becomes a bottleneck/SPOF (mitigate with horizontal scaling). Adds one network hop. But centralizes auth logic, API versioning, and rate limiting.

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
    pub jwt_config: JwtConfig,
}

// services/gateway/src/routes/catalog.rs
async fn get_work(
    State(state): State<AppState>,
    claims: JwtClaims,  // extracted by middleware
    Path(work_id): Path<i64>,
) -> Result<Json<WorkResponse>, AppError> {
    let mut client = state.catalog_client.clone();
    let request = tonic::Request::new(GetWorkRequest { id: work_id });
    // Inject authenticated user as gRPC metadata
    request.metadata_mut().insert("x-user-id", claims.sub.parse()?);
    let response = client.get_work(request).await?;
    Ok(Json(response.into_inner().into()))
}
```

### Pattern 3: Service-Repository Separation

**What:** Each backend service (auth, catalog, user) separates its gRPC trait implementation (`service.rs`) from its database queries (`repository.rs`). The service layer handles business logic and validation; the repository handles sea-orm queries.

**When to use:** Always. Even for simple CRUD, this separation makes testing possible (mock the repository) and keeps sea-orm details out of gRPC handler code.

**Trade-offs:** More files, but each file stays focused and testable.

**Example:**

```rust
// services/catalog/src/repository.rs
pub struct CatalogRepository {
    db: DatabaseConnection,
}

impl CatalogRepository {
    pub async fn find_work_by_id(&self, id: i64) -> Result<Option<work::Model>, DbErr> {
        Work::find_by_id(id).one(&self.db).await
    }

    pub async fn find_works_by_tags(&self, tags: &[String], page: u64) -> Result<Vec<work::Model>, DbErr> {
        // sea-orm query with joins on work_tags
    }
}

// services/catalog/src/service.rs
#[tonic::async_trait]
impl CatalogService for CatalogServiceImpl {
    async fn get_work(&self, request: Request<GetWorkRequest>) -> Result<Response<WorkResponse>, Status> {
        let work_id = request.into_inner().id;
        let work = self.repo.find_work_by_id(work_id).await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("work not found"))?;
        Ok(Response::new(work.into()))
    }
}
```

### Pattern 4: JWT Verification in Gateway Middleware (axum extractor)

**What:** A custom axum extractor handles the full JWT lifecycle described in PROJECT.md: valid JWT passes through, expired-within-grace gets a new cookie set, expired-past-grace triggers session verification via Auth service gRPC call.

**When to use:** For the Gateway's auth middleware, and for nginx auth_request endpoint.

**Trade-offs:** Complex extractor logic, but keeps auth concerns out of route handlers entirely. Handlers just receive a `JwtClaims` if auth succeeds.

**Example:**

```rust
// services/gateway/src/middleware/auth.rs
#[async_trait]
impl<S> FromRequestParts<S> for JwtClaims
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let cookie = extract_jwt_cookie(parts)?;

        match validate_jwt(&cookie, &app_state.jwt_config) {
            Ok(claims) => Ok(claims),                    // Valid: stateless pass-through
            Err(JwtError::ExpiredInGrace(claims)) => {
                // Expired within grace: issue new JWT, set cookie header
                let new_token = issue_jwt(&claims, &app_state.jwt_config);
                set_response_cookie(parts, &new_token);
                Ok(claims)
            }
            Err(JwtError::ExpiredPastGrace(claims)) => {
                // Past grace: verify session via Auth service
                let mut auth = app_state.auth_client.clone();
                let session = auth.verify_session(/* ... */).await?;
                let new_token = issue_jwt(&session.claims(), &app_state.jwt_config);
                set_response_cookie(parts, &new_token);
                Ok(session.claims())
            }
            Err(_) => Err(AppError::Unauthorized),
        }
    }
}
```

## Data Flow

### Request Flow: Client Browsing Works

```
Client (browser)
    |  GET /api/works?tag=artist:xxx
    v
nginx (:443)
    |  proxy_pass to gateway
    v
Gateway (axum :8080)
    |  1. JWT extractor middleware: validate/refresh token
    |  2. Parse query params
    |  3. gRPC call to Catalog
    v
Catalog (tonic :50052)
    |  Query sea-orm: works JOIN work_tags WHERE tag IN (...)
    |  Return Vec<WorkResponse>
    v
Gateway
    |  Convert gRPC response to JSON
    |  Set refreshed JWT cookie if needed
    v
Client receives JSON + possibly refreshed cookie
```

### Request Flow: Image Serving

```
Client (browser)
    |  GET /images/{work_id}/{page}.webp
    v
nginx (:443)
    |  auth_request /internal/auth-check
    |     |
    |     v
    |  Gateway (axum :8080) /internal/auth-check
    |     |  JWT validation only (no gRPC call if token valid)
    |     |  Returns 200 + optional Set-Cookie header
    |     v
    |  auth_request_set $auth_cookie $upstream_http_set_cookie
    |
    |  If 200: serve file from local filesystem
    |  add_header Set-Cookie $auth_cookie (forwards refresh cookie)
    v
Client receives image + possibly refreshed cookie
```

### Request Flow: Scraper Upload Pipeline

```
Scraper (separate server)
    |  1. Poll external source (hitomi_la crate)
    |  2. Detect new work
    |
    |  POST /api/works  (API Key in header)
    v
nginx --> Gateway --> Catalog (gRPC)
    |  Create unpublished work with metadata + hash
    |  Return work_id
    |
    |  For each page image:
    |  POST /upload/{work_id}/{page_num}
    v
nginx --> File Service (axum :8090)
    |  Save image to filesystem at /{work_id}/{page_num}.webp
    |
    |  POST /api/works/{work_id}/publish  (API Key)
    v
nginx --> Gateway --> Catalog (gRPC)
    |  Verify: expected page_count == actual file count
    |  If match: set published = true
    |  If mismatch: return error (scraper retries missing pages)
```

### Request Flow: Work Renewal

```
Scraper detects work ID changed (old_id -> new_id)
    |
    |  POST /api/works/{old_id}/renew  { new_id, new_metadata }
    v
Gateway --> Catalog (gRPC)
    |  1. Create new work entry with new_id (if not exists)
    |  2. Create renewal relation edge: old_id -> new_id
    |  3. Set new_id as canonical, old_id as redirect
    |  4. Enqueue cross-service ID update jobs (DB queue)
    |
    |  Background worker in Catalog processes queue:
    |     - Notify User service: update taste/history references
    |     - Image files stay in place (old_id directory remains valid)
    v
User service processes ID update asynchronously
```

### Key Data Flows

1. **Authentication flow:** Client --> nginx --> Gateway (JWT check) --> Auth (gRPC, only if session verification needed) --> Gateway (issue new JWT cookie) --> Client
2. **Read flow:** Client --> nginx --> Gateway (auth + translate) --> Catalog/User (gRPC query) --> Gateway (JSON response) --> Client
3. **Image flow:** Client --> nginx (auth_request to Gateway, then serve file directly) --> Client
4. **Write flow (scraper):** Scraper --> nginx --> Gateway (API key auth) --> Catalog (metadata) + File (images) --> Catalog (publish)
5. **Cross-service sync:** Catalog (DB queue) --> Catalog worker polls --> User service (gRPC call to update references)

## Database Architecture

### Decision: Shared PostgreSQL Instance, Separate Schemas

Use a **single PostgreSQL instance** with **separate schemas per service**. This is the right choice for a small-community project because:

- **Why not database-per-service:** Separate PostgreSQL instances mean separate connection pools, separate backup strategies, separate monitoring, and massively increased operational overhead. For a small team, this is unjustifiable.
- **Why not single shared schema:** Services would be coupled at the database level. Schema changes in one service could break another. Migrations become dangerous.
- **Why separate schemas:** Each service owns its schema (e.g., `auth.*`, `catalog.*`, `user.*`). Services cannot query each other's tables directly -- they must go through gRPC. This enforces service boundaries while keeping ops simple.

```sql
-- PostgreSQL setup
CREATE SCHEMA auth;
CREATE SCHEMA catalog;
CREATE SCHEMA user_service;   -- "user" is reserved in PG
CREATE SCHEMA file;
CREATE SCHEMA scraper;

-- Each service's connection uses search_path to its schema
-- e.g., auth service: search_path = 'auth,public'
```

**Migration strategy:** A single `madome-migration` crate runs all migrations across all schemas. This is pragmatic -- you can split later if needed, but for now one migration runner with schema-prefixed migrations keeps things simple.

```
migrations/
  m20260101_000001_create_auth_schema.rs     # CREATE SCHEMA auth;
  m20260101_000002_create_auth_sessions.rs   # CREATE TABLE auth.sessions (...)
  m20260101_000003_create_catalog_schema.rs
  m20260101_000004_create_catalog_works.rs
  ...
```

### Schema Ownership

| Schema | Owner Service | Key Tables |
|--------|---------------|------------|
| `auth` | Auth | `sessions`, `credentials` (passkey), `jwt_cache` |
| `catalog` | Catalog | `works`, `tags`, `work_tags`, `work_relations` (renewal graph), `publish_queue` |
| `user_service` | User | `tastes` (like/dislike per work), `histories` (reading history) |
| `file` | File | `uploads` (tracking uploaded pages per work -- for publish verification) |
| `scraper` | Scraper | `tracked_works` (work_id, last_checked, check_interval), `scrape_state` |

### Cross-Service Data Consistency

Since services cannot join across schemas via SQL, cross-service references use a **DB-based queue pattern**:

```
catalog.sync_queue
  id SERIAL PRIMARY KEY,
  event_type TEXT,           -- 'work_renewed', 'work_deleted', etc.
  payload JSONB,             -- { old_id: 123, new_id: 456 }
  target_service TEXT,       -- 'user'
  status TEXT DEFAULT 'pending',
  created_at TIMESTAMPTZ,
  processed_at TIMESTAMPTZ
```

Catalog writes events. A background worker in Catalog (or a dedicated worker binary) polls this queue, makes gRPC calls to affected services, and marks events as processed. This avoids introducing a message broker while maintaining eventual consistency.

## Proto File Organization

### Recommended Proto Structure

```protobuf
// proto/common.proto
syntax = "proto3";
package madome.common;

message Pagination {
  uint32 page = 1;
  uint32 per_page = 2;
}

message PaginatedResponse {
  uint32 total = 1;
  uint32 page = 2;
  uint32 per_page = 3;
}

message Empty {}

// proto/auth.proto
syntax = "proto3";
package madome.auth;
import "common.proto";

service AuthService {
  rpc VerifySession(VerifySessionRequest) returns (VerifySessionResponse);
  rpc CreateSession(CreateSessionRequest) returns (CreateSessionResponse);
  rpc InvalidateSession(InvalidateSessionRequest) returns (madome.common.Empty);
  rpc GetPasskeyChallenge(GetPasskeyChallengeRequest) returns (PasskeyChallenge);
  rpc VerifyPasskeyResponse(VerifyPasskeyRequest) returns (AuthResult);
  rpc RegisterPasskey(RegisterPasskeyRequest) returns (RegisterPasskeyResponse);
}

// proto/catalog.proto
syntax = "proto3";
package madome.catalog;
import "common.proto";

service CatalogService {
  rpc GetWork(GetWorkRequest) returns (WorkResponse);
  rpc ListWorks(ListWorksRequest) returns (ListWorksResponse);
  rpc QueryByTag(QueryByTagRequest) returns (ListWorksResponse);
  rpc QueryByTags(QueryByTagsRequest) returns (ListWorksResponse);
  rpc QueryByIds(QueryByIdsRequest) returns (ListWorksResponse);
  rpc CreateWork(CreateWorkRequest) returns (WorkResponse);
  rpc UpdateWork(UpdateWorkRequest) returns (WorkResponse);
  rpc PublishWork(PublishWorkRequest) returns (PublishWorkResponse);
  rpc RenewWork(RenewWorkRequest) returns (RenewWorkResponse);
  rpc GetRenewalHistory(GetRenewalHistoryRequest) returns (RenewalHistoryResponse);
}

// proto/user.proto
syntax = "proto3";
package madome.user;
import "common.proto";

service UserService {
  rpc GetTastes(GetTastesRequest) returns (TastesResponse);
  rpc SetTaste(SetTasteRequest) returns (TasteResponse);
  rpc GetHistory(GetHistoryRequest) returns (HistoryResponse);
  rpc RecordHistory(RecordHistoryRequest) returns (madome.common.Empty);
  rpc UpdateWorkReferences(UpdateWorkReferencesRequest) returns (madome.common.Empty);
}
```

### Proto Design Principles

- **One proto file per service.** Keep it simple. No need for per-RPC files.
- **Common types in `common.proto`.** Pagination, empty responses, timestamp wrappers.
- **Service names match crate names.** `CatalogService` in `catalog.proto` maps to `services/catalog/`.
- **Include cross-service RPCs explicitly.** `UserService.UpdateWorkReferences` is called by Catalog's background worker during renewals.
- **Use wrapper messages, not bare primitives.** `GetWorkRequest { int64 id = 1; }` not `int64` directly. This allows adding fields later without breaking the API.

## Shared Crate Patterns

### What Goes Where

| Crate | Contains | Depends On |
|-------|----------|------------|
| `madome-proto` | Generated gRPC code (server + client stubs) | `tonic`, `prost` |
| `madome-core` | Config loading, error types, JWT claims struct, common middleware types | `serde`, `jsonwebtoken`, `figment` or `config` |
| `madome-entity` | sea-orm entity structs for all schemas | `sea-orm` |
| `madome-migration` | All database migrations | `sea-orm-migration`, `madome-entity` |

### Dependency Graph

```
services/gateway  --> madome-proto (client stubs)
                  --> madome-core  (config, JWT, errors)

services/auth     --> madome-proto (server traits)
                  --> madome-core  (config, JWT, errors)
                  --> madome-entity (auth entities)

services/catalog  --> madome-proto (server traits + user client stub)
                  --> madome-core  (config, errors)
                  --> madome-entity (catalog entities)

services/user     --> madome-proto (server traits)
                  --> madome-core  (config, errors)
                  --> madome-entity (user entities)

services/file     --> madome-core  (config, errors)
                  (no proto dependency -- File uses REST, not gRPC)

services/scraper  --> madome-core  (config, errors)
                  --> hitomi_la    (external crate)
                  (no proto dependency -- Scraper calls REST, not gRPC)
```

### Workspace Cargo.toml

```toml
[workspace]
resolver = "3"
members = [
    "crates/madome-proto",
    "crates/madome-core",
    "crates/madome-entity",
    "crates/madome-migration",
    "services/gateway",
    "services/auth",
    "services/catalog",
    "services/user",
    "services/file",
    "services/scraper",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "UNLICENSED"

[workspace.dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }

# Web framework
axum = "0.8"
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace"] }

# gRPC
tonic = "0.13"
tonic-build = "0.13"
prost = "0.13"

# Database
sea-orm = { version = "1", features = ["sqlx-postgres", "runtime-tokio-rustls", "macros"] }
sea-orm-migration = { version = "1" }

# Auth
jsonwebtoken = { version = "9", features = ["aws_lc"] }
webauthn-rs = "0.5"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Error handling
thiserror = "2"
anyhow = "1"

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Config
figment = { version = "0.10", features = ["env", "toml"] }

# Shared
madome-proto = { path = "crates/madome-proto" }
madome-core = { path = "crates/madome-core" }
madome-entity = { path = "crates/madome-entity" }
madome-migration = { path = "crates/madome-migration" }
```

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| Small community (< 100 users) | Single server, all services as processes, single PostgreSQL. This is the target. Nginx + 6 binaries + 1 PG instance. |
| Growing (100-1K users) | Horizontal scale Gateway (stateless). Add connection pooling (PgBouncer). Consider CDN for images. Still single PG instance. |
| Open service (1K+ users) | Split PostgreSQL per service. Add Redis for JWT cache and rate limiting. Image CDN mandatory. Consider replacing DB queue with NATS/RabbitMQ. |

### Scaling Priorities

1. **First bottleneck: Image serving.** Images are large and numerous. nginx is efficient at static file serving, but disk I/O and bandwidth will be the first constraint. **Mitigation:** nginx sendfile is already efficient. Add response caching headers. For growth, put a CDN in front.
2. **Second bottleneck: Catalog queries.** Tag-based queries with joins will get slow as work count grows. **Mitigation:** Proper indexes on `work_tags`. Consider materialized views or denormalized tag arrays for hot queries.
3. **Third bottleneck: Gateway.** All traffic funnels through it. **Mitigation:** Gateway is stateless -- just run more instances behind nginx upstream round-robin.

## Anti-Patterns

### Anti-Pattern 1: Direct Cross-Schema Queries

**What people do:** Service A queries Service B's database tables directly because "it's faster than gRPC."
**Why it's wrong:** Destroys service boundaries. Schema changes in B break A. You lose the ability to split services later. The whole point of microservices is wasted.
**Do this instead:** Always go through gRPC. If latency is a problem, cache the result. The DB queue pattern handles async cross-service data needs.

### Anti-Pattern 2: Proto Compilation in Every Service

**What people do:** Each service has its own `build.rs` that compiles the proto files it needs.
**Why it's wrong:** Duplicated build logic. Risk of services compiling different versions of the same proto. Slower builds (proto compilation happens N times instead of once). Type mismatches between services if protos drift.
**Do this instead:** Single `madome-proto` crate compiles all protos once. All services depend on it.

### Anti-Pattern 3: Fat Gateway

**What people do:** Put business logic in the Gateway. "It's just a small validation" or "let me aggregate from two services here."
**Why it's wrong:** Gateway becomes a monolith wearing a microservice costume. It gets harder to test, harder to change, and becomes the coupling point for all business logic changes.
**Do this instead:** Gateway should ONLY do: JWT auth, request translation (REST to gRPC), response translation (gRPC to JSON), and routing. If you need to aggregate from two services, either have one service call the other, or create a dedicated aggregation endpoint in the appropriate service.

### Anti-Pattern 4: Shared Mutable Entity Crate Without Schema Boundaries

**What people do:** Put all sea-orm entities in one shared crate and let any service use any entity.
**Why it's wrong:** Any service can accidentally query any table. The entity crate becomes a coupling point.
**Do this instead:** Use the shared entity crate for type definitions (so types are consistent), but enforce at the application level that each service only uses its own schema's entities. Feature flags on the entity crate can help: `madome-entity = { workspace = true, features = ["catalog"] }` only exposes catalog entities to the catalog service.

### Anti-Pattern 5: Synchronous Cross-Service Calls in Write Paths

**What people do:** When publishing a work, Catalog calls User service synchronously to pre-populate something, or calls File service to verify.
**Why it's wrong:** Cascading failures. If User service is down, publishing fails even though User has nothing to do with publishing.
**Do this instead:** Keep write paths within a single service where possible. Use the DB queue for eventual consistency. The publish workflow correctly only needs Catalog (metadata) + File (page count verification via filesystem or File service's own records).

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| hitomi.la | `hitomi_la` crate in Scraper | External source. Rate limit scraping. Handle site structure changes gracefully. |
| PostgreSQL | sea-orm + sqlx driver | Single instance, schema-per-service. Connection pool per service binary. |
| nginx | Reverse proxy + auth_request | Critical path for image serving. Config must be version-controlled. |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| Gateway <-> Auth | gRPC (session verification) | Only called when JWT expired past grace period. Hot path should be stateless JWT validation. |
| Gateway <-> Catalog | gRPC (work queries, mutations) | Most common internal call. Catalog is the busiest backend service. |
| Gateway <-> User | gRPC (tastes, histories) | Read-heavy. Consider caching popular queries in Gateway. |
| Catalog -> User | gRPC (via DB queue worker) | Async. For work ID renewal propagation only. |
| Scraper -> Gateway | REST (API Key auth) | Scraper is a client, not a gRPC peer. Uses same REST API as browser clients, just with API key instead of JWT. |
| Scraper -> File | REST (direct upload) | Bypasses Gateway to avoid routing large image payloads through it. Still goes through nginx. |
| nginx -> Gateway | HTTP (auth_request) | Subrequest for image serving auth. Must be fast -- return 200/401 quickly. |

## Build Order Implications

Services have dependency ordering that should inform the roadmap phase structure:

```
Phase 1: Foundation
  madome-core (config, errors, types)
  madome-proto (proto compilation)
  madome-entity (entity definitions)
  madome-migration (schema setup)

Phase 2: Auth (everything else depends on auth being available)
  Auth service (sessions, passkey, JWT issuance)
  Gateway skeleton (JWT middleware, routing to Auth)

Phase 3: Core Data
  Catalog service (work CRUD, tag queries)
  Gateway routes for Catalog

Phase 4: Content Pipeline
  File service (image upload/storage)
  nginx config (reverse proxy, auth_request, image serving)
  Scraper (detect + upload + publish pipeline)

Phase 5: User Features
  User service (tastes, histories)
  Gateway routes for User

Phase 6: Cross-Service Integration
  Work renewal (graph relations, canonical IDs)
  DB queue + worker (cross-service ID sync)
  Decreasing-frequency update checks
```

**Rationale for this ordering:**
- **Auth first:** Gateway needs JWT verification before any other route works. Every other service implicitly depends on auth.
- **Catalog before File/Scraper:** Scraper needs Catalog API to upload metadata and trigger publishing. File needs work IDs from Catalog.
- **User last among services:** User features (tastes, histories) are additive. The system works without them (browse-only mode).
- **Cross-service integration last:** Renewal and queue sync are the most complex features and depend on all services being operational.

## Sources

- Cargo workspace documentation: https://doc.rust-lang.org/cargo/reference/workspaces.html (HIGH confidence)
- tonic/tonic-build documentation: https://docs.rs/tonic/latest/tonic/ and https://docs.rs/tonic-build/latest/tonic_build/ (HIGH confidence)
- axum documentation: https://docs.rs/axum/latest/axum/ (HIGH confidence)
- sea-orm documentation: https://docs.rs/sea-orm/latest/sea_orm/ (HIGH confidence)
- Project specification: .planning/PROJECT.md (HIGH confidence -- authoritative project definition)
- Rust microservice workspace patterns based on established community conventions (MEDIUM confidence -- based on training data, not verified against 2026 sources)
- nginx auth_request pattern based on established nginx documentation (HIGH confidence -- stable, well-documented feature)

---
*Architecture research for: Madome - Rust microservice content mirroring platform*
*Researched: 2026-03-21*
