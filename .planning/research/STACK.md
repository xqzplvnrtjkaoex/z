# Technology Stack

**Project:** Madome -- Manga Mirroring & Aggregation Service
**Researched:** 2026-03-21
**Overall Confidence:** HIGH (versions verified via crates.io API on research date)

## Version Verification Method

All crate versions verified by querying `crates.io/api/v1/crates/{name}` on 2026-03-21. Rust toolchain verified locally: `rustc 1.94.0`, `cargo 1.94.0`. WebSearch/WebFetch were unavailable, so documentation-level details rely on training data (flagged where relevant).

---

## Recommended Stack

### Runtime & Language

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **Rust** | Edition 2024 | Language | Already chosen per project constraints. Edition 2024 is stable on rustc 1.94. Provides lifetime elision improvements and `gen` blocks. | HIGH |
| **tokio** | 1.50.0 | Async runtime | The only production-grade async runtime for the Rust ecosystem. Every major crate (tonic, axum, reqwest, sea-orm) depends on it. Use `features = ["full"]` during development, narrow to `["rt-multi-thread", "macros", "net", "time", "signal", "sync"]` for production binaries. | HIGH |

### REST API (External-Facing)

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **axum** | 0.8.8 | HTTP framework (Gateway, File service) | The ecosystem standard for Rust HTTP services. Built on tower/hyper by the tokio team. Type-safe extractors, middleware via tower layers, first-class WebSocket support. Axum 0.8 is the current stable line (released Dec 2025). | HIGH |
| **axum-extra** | 0.12.5 | Cookie extraction, typed headers | Provides `CookieJar` extractor needed for JWT-in-cookie pattern and `TypedHeader` for structured header access. | HIGH |
| **tower** | 0.5.3 | Middleware framework | Shared middleware between axum and tonic. Enables writing middleware that applies to both REST and gRPC handlers (rate limiting, logging, auth). | HIGH |
| **tower-http** | 0.6.8 | HTTP-specific middleware | CORS, compression, request tracing, timeout layers. Essential for production REST APIs. Use `features = ["cors", "trace", "timeout", "compression-gzip"]`. | HIGH |

### gRPC (Internal Communication)

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **tonic** | 0.14.5 | gRPC framework | The standard Rust gRPC implementation. Actively maintained (Feb 2026 release). Built on hyper and tower, shares middleware with axum. Supports streaming, interceptors, TLS. | HIGH |
| **tonic-build** | 0.14.5 | Proto code generation (build dep) | Compile-time .proto to Rust code generation. Use in `build.rs` for each service. | HIGH |
| **tonic-reflection** | 0.14.5 | gRPC reflection service | Enables `grpcurl` and other tools to introspect services during development. Only enable in dev/staging builds. | HIGH |
| **prost** | 0.14.3 | Protobuf serialization | Tonic's protobuf backend. Generates Rust structs from .proto files. Version must match tonic's expected prost version (0.14.x for tonic 0.14.x). | HIGH |
| **prost-build** | 0.14.3 | Proto compilation (build dep) | Required by tonic-build. Same version alignment rule. | HIGH |

**Critical note on tonic + prost version alignment:** Tonic 0.14.x depends on prost 0.14.x. Never mix major/minor versions between tonic and prost -- they share generated code types.

### Database & ORM

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **PostgreSQL** | 16+ | Primary database | Required per project constraints. Each microservice gets its own database/schema for isolation. | HIGH |
| **sea-orm** | 1.1.19 | Async ORM | Stable release, actively maintained. Use `features = ["sqlx-postgres", "runtime-tokio-native-tls", "macros", "with-chrono", "with-uuid"]`. Provides derive-based entity definitions, migrations, and query builder. | HIGH |
| **sea-orm-migration** | 1.1.19 | Database migrations | Rust-native migrations. Each service binary has its own migration set. Version must match sea-orm. | HIGH |
| **sea-orm-cli** | 1.1.19 | Entity generation CLI | Generate entity files from existing schema. Development tool only. | HIGH |

**Why sea-orm 1.1.19 and NOT 2.0.0-rc.37:** Sea-orm 2.0 has been in RC since December 2025 with 37 release candidates in 3 months. This is a rapidly iterating pre-release with breaking changes between RCs. For a greenfield project that needs stability during development, use 1.1.19 (stable, November 2025). Migrate to 2.0 after its stable release. The 2.0 line will bring sea-query 1.0 and potential API improvements, but the churn risk is not worth it for a project starting now.

**Alternative considered: sqlx (raw SQL)**
Why not: Sea-orm provides migration tooling, entity derives, and relation definitions that reduce boilerplate significantly for CRUD-heavy services (catalog, user). The type-safety of compile-time checked SQL (sqlx's strength) is replicated by sea-orm's query builder. For a 6-service monorepo, ORM consistency across services outweighs raw SQL flexibility.

### Authentication

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **webauthn-rs** | 0.5.4 | Passkey/WebAuthn server | The only mature Rust WebAuthn implementation. 0.5.4 is the latest stable (Dec 2025). Supports passkey registration, authentication, conditional UI, resident keys. Use `features = ["resident-key-support"]`. | HIGH |
| **webauthn-rs-proto** | 0.5.4 | WebAuthn protocol types | Shared types between webauthn-rs and your API layer. Needed for serializing challenge/response to the gateway REST API. | HIGH |
| **jsonwebtoken** | 10.3.0 | JWT creation & verification | Major version 10 released Sep 2025. Supports ES256, RS256, EdDSA. The project specifies `aws_lc` backend -- use `default-features = false, features = ["aws_lc"]` for FIPS-capable crypto. | HIGH |
| **aws-lc-rs** | 1.16.2 | Cryptography backend | Used by jsonwebtoken when `aws_lc` feature is enabled. AWS-maintained, FIPS-validated. Faster than ring for most operations. | HIGH |
| **minicbor** | 2.2.1 | CBOR encoding (AAGUID parsing) | Lightweight CBOR decoder needed for parsing authenticator attestation data (AAGUID extraction from passkey metadata). | HIGH |

**Why webauthn-rs 0.5.4 and NOT 0.6.0-dev:** The 0.6.0-dev was just published (2026-03-20, one day before research). It is a `-dev` pre-release, not suitable for production. 0.5.4 has full passkey support including resident keys and conditional UI.

**Why jsonwebtoken and NOT jose/josekit:** jsonwebtoken has 10x the download count, simpler API for the JWT-only use case, and explicit aws-lc-rs backend support. josekit is more featureful for JOSE/JWE but overkill when you only need JWS (signed tokens).

### Serialization

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **serde** | 1.0.228 | Serialization framework | Ubiquitous. Required by virtually every crate in the stack. | HIGH |
| **serde_json** | 1.0.149 | JSON serialization | REST API request/response bodies, configuration files. | HIGH |

### HTTP Client (Scraper)

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **reqwest** | 0.13.2 | HTTP client | The standard async HTTP client for Rust. Used by the scraper for fetching pages and downloading images. Supports connection pooling, cookie jars, redirect policies, proxy. Use `features = ["json", "cookies", "gzip", "brotli"]`. | HIGH |
| **hitomi_la** | 0.1.4 | hitomi.la API wrapper | Already identified in project context. Wraps source-specific API calls. Low download count (1,904) suggests it is either niche or author-maintained -- verify it covers needed endpoints before relying on it exclusively. | MEDIUM |

### Scraping & Parsing

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **scraper** | 0.26.0 | HTML parsing + CSS selectors | For parsing hitomi.la pages if the hitomi_la crate does not cover all needs. Built on html5ever (spec-compliant HTML parser). Simpler than maintaining a headless browser dependency. | HIGH |

**Why NOT headless_chrome/chromiumoxide:** Hitomi.la requires JavaScript execution for some content, but the `hitomi_la` crate likely handles this at the API level. If JS execution is needed, prefer chromiumoxide (0.9.1) over headless_chrome -- it uses the Chrome DevTools Protocol directly and has better async support. But start without it and only add if `hitomi_la` + `reqwest` + `scraper` are insufficient.

### Caching (Per-Session JWT Cache)

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **moka** | 0.12.14 | In-memory cache with TTL | For per-session JWT caching in the gateway (duplicate JWT prevention). Lock-free concurrent cache with time-based expiration. 65M downloads, battle-tested. Use `features = ["future"]` for async support. | HIGH |

**Why moka over dashmap + manual TTL:** The project needs TTL-based eviction (JWT cache entries expire). Moka provides this natively. DashMap (6.1.0 stable) is a concurrent HashMap but has no built-in eviction -- you would need to implement TTL cleanup yourself, which is error-prone.

### Scheduling (Scraper Update Checks)

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **tokio-cron-scheduler** | 0.15.1 | Task scheduling | For the scraper's decreasing-interval update check pattern. Supports cron expressions and dynamic job scheduling. However, the decreasing interval pattern (5min -> 30min -> 2h -> 12h) is better modeled as a custom `tokio::time::sleep` loop with computed delays rather than cron jobs. | MEDIUM |

**Recommendation:** For the specific decreasing-interval pattern described in PROJECT.md, use a custom scheduler built on `tokio::time::sleep` + `tokio::select!`. The logic is: compute next check time based on work age, sleep until then, check, repeat. This is ~50 lines of code and avoids a cron dependency. Use tokio-cron-scheduler only if you later need classic cron-style scheduling for other tasks.

### Image Processing

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **image** | 0.25.10 | Image validation/metadata | Validate uploaded images (format, dimensions) before storage. Not needed for serving (nginx handles that). Use only if you need to verify image integrity or extract metadata. | HIGH |

**Note:** The File service stores images on the local filesystem and nginx serves them directly. No image processing library is strictly required unless you need validation, thumbnail generation, or future censorship features.

### Observability

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **tracing** | 0.1.44 | Structured logging/tracing | The ecosystem standard for Rust instrumentation. Used by axum, tonic, tower, and reqwest internally. Structured spans + events. | HIGH |
| **tracing-subscriber** | 0.3.23 | Log output formatting | Configurable output (JSON for production, pretty for development). Use `features = ["env-filter", "json", "fmt"]` for environment-based log level control. | HIGH |

### Configuration

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **config** | 0.15.22 | Hierarchical configuration | Supports TOML files + environment variable overrides. Each service binary loads its own config. Use layered sources: defaults -> config file -> env vars. | MEDIUM |

**Alternative: figment (0.10.19)**
Figment has a nicer API with `Provider` trait composition but is less actively maintained (last update May 2024). config (0.15.22, updated March 2026) is more actively maintained. Use config for this project.

**Alternative: dotenvy (0.15.7)**
Only for `.env` file loading in development. Last updated March 2023. Still works fine but is minimal. Use alongside config, not as a replacement.

### Error Handling

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **thiserror** | 2.0.18 | Typed error definitions | For library/shared code error types. Derive-based, zero-overhead. Version 2.0 (Jan 2026) is the current major. | HIGH |
| **anyhow** | 1.0.102 | Ad-hoc error handling | For application binary code where you want to propagate errors without defining types. Use in service binaries, not in shared libraries. | HIGH |

### Common Utilities

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **uuid** | 1.22.0 | UUID generation | Primary keys, correlation IDs. Use `features = ["v7", "serde"]`. UUIDv7 is time-sortable, preferred over v4 for database primary keys. | HIGH |
| **chrono** | 0.4.44 | Date/time handling | Timestamps, duration calculations (work age for update scheduling). Sea-orm integrates via `with-chrono` feature. | HIGH |
| **bytes** | 1.11.1 | Byte buffer | Shared dependency of hyper/tonic/axum. May need directly for image upload streaming. | HIGH |
| **base64** | 0.22.1 | Base64 encoding | WebAuthn challenge encoding, JWT payload inspection. Use `base64::engine::general_purpose::URL_SAFE_NO_PAD` for WebAuthn. | HIGH |
| **rand** | 0.10.0 | Random number generation | Session IDs, nonces. Version 0.10 (Feb 2026) is current. | HIGH |
| **sha2** | 0.10.8 (stable) | SHA-256 hashing | Work info hash computation (for deduplication detection). Use 0.10.8 stable, NOT 0.11.0-rc.5. | HIGH |
| **tokio-stream** | 0.1.18 | Stream utilities for tokio | Needed for gRPC streaming responses and async iteration patterns. | HIGH |
| **futures-util** | 0.3.32 | Future/stream combinators | `StreamExt`, `FutureExt` for async composition. | HIGH |
| **parking_lot** | 0.12.5 | Faster mutexes | Drop-in replacement for `std::sync::Mutex` with better performance under contention. Useful for shared state in services. | HIGH |

---

## Infrastructure (Non-Rust)

### Reverse Proxy

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **nginx** | 1.26+ (stable) or 1.27+ (mainline) | Reverse proxy, static file serving, auth subrequest | Required per project constraints. Handles: (1) reverse proxy to gateway/file services, (2) `auth_request` module for image auth, (3) `auth_request_set` for cookie refresh forwarding, (4) direct image file serving from filesystem. | HIGH |

**Key nginx modules needed:**
- `ngx_http_auth_request_module` -- for subrequest-based auth on image paths
- `ngx_http_proxy_module` -- for reverse proxying to Rust services
- `ngx_http_grpc_module` -- NOT needed (gRPC is internal only, gateway handles translation)

**nginx image serving pattern:**
```nginx
location /images/ {
    auth_request /auth;
    auth_request_set $auth_cookie $upstream_http_set_cookie;
    add_header Set-Cookie $auth_cookie;
    alias /path/to/image/storage/;
    expires 30d;
    add_header Cache-Control "public, immutable";
}

location = /auth {
    internal;
    proxy_pass http://gateway:3000/auth/verify;
    proxy_pass_request_body off;
    proxy_set_header Content-Length "";
    proxy_set_header X-Original-URI $request_uri;
    proxy_set_header Cookie $http_cookie;
}
```

### Database

| Technology | Version | Purpose | Confidence |
|------------|---------|---------|------------|
| **PostgreSQL** | 16+ | Primary data store for all services | HIGH |

**Database per service pattern:** Each service (auth, catalog, user) gets its own PostgreSQL database (or schema). No cross-service JOINs. The scraper communicates through the gateway API, not directly to databases.

---

## Workspace Structure

```
madome/
  Cargo.toml              # Workspace root
  proto/                   # Shared .proto definitions
    auth.proto
    catalog.proto
    user.proto
  crates/
    madome-core/           # Shared types, error types, config
    madome-proto/          # Generated protobuf code (build.rs)
    madome-gateway/        # REST API + gRPC client
    madome-auth/           # Auth service (gRPC server)
    madome-catalog/        # Catalog service (gRPC server)
    madome-user/           # User service (gRPC server)
    madome-file/           # File service (REST, image upload)
    madome-scraper/        # Scraper binary (runs separately)
```

**Why `crates/` not `services/`:** The `crates/` directory holds both library crates (core, proto) and binary crates (services). This is the conventional Cargo workspace layout.

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| ORM | sea-orm 1.1.19 | diesel 2.x | Diesel is sync-only (needs `spawn_blocking`), heavier macros, less ergonomic for async services. Sea-orm is async-native with tokio. |
| ORM | sea-orm 1.1.19 | sqlx 0.8.x | sqlx is excellent for raw SQL but provides no entity abstraction, relations, or migration tooling comparable to sea-orm. More boilerplate for CRUD services. |
| ORM | sea-orm 1.1.19 (stable) | sea-orm 2.0.0-rc.37 | 37 RCs in 3 months = high churn. Risk of breaking changes during development. Upgrade after 2.0 stable release. |
| HTTP framework | axum 0.8 | actix-web 4.x | Actix-web is slightly faster in benchmarks but uses its own runtime. Axum shares the tokio + tower + hyper ecosystem with tonic, enabling shared middleware. Ecosystem coherence wins. |
| gRPC | tonic 0.14 | grpc-rs (C-based) | grpc-rs wraps the C gRPC library. Tonic is pure Rust, better IDE support, simpler builds, tighter tower integration. |
| JWT | jsonwebtoken 10 | frank_jwt, alcoholic_jwt | jsonwebtoken is the most downloaded JWT crate by far, actively maintained, supports aws-lc-rs backend. Others are less maintained. |
| Cache | moka 0.12 | redis | Redis adds infrastructure complexity. For per-session JWT caching within a single gateway process, in-memory cache is simpler and faster. Add Redis only if gateway scales horizontally (multiple instances). |
| Passkey | webauthn-rs 0.5.4 | (none) | Only mature Rust WebAuthn implementation. No real alternative. |
| Config | config 0.15 | figment 0.10 | Figment has a nicer API but less active maintenance. config is more recently updated. |
| Scheduling | tokio::time (manual) | tokio-cron-scheduler | The decreasing-interval pattern is simpler as a custom sleep loop. Cron library is overhead for this use case. |

---

## What NOT to Use

| Technology | Why Avoid |
|------------|-----------|
| **actix-web** | Different runtime ecosystem. Cannot share tower middleware with tonic. Forces you to maintain two middleware stacks. |
| **diesel** | Sync-only ORM. Requires `spawn_blocking` wrappers in async code. Sea-orm is async-native. |
| **sea-orm 2.0-rc** | 37 release candidates in 3 months. Too unstable for a greenfield project. Upgrade path available later. |
| **webauthn-rs 0.6.0-dev** | Released one day ago as a `-dev` tag. Use stable 0.5.4. |
| **sha2 0.11.0-rc** | Use sha2 0.10.8 (stable). The 0.11 line has been in RC since Feb 2025. |
| **redis (initially)** | Single gateway instance does not need distributed cache. Moka is simpler. Add Redis when horizontal scaling is needed. |
| **message broker (RabbitMQ/NATS)** | Explicitly out of scope per PROJECT.md. DB-based queue is sufficient at current scale. |
| **GraphQL** | REST is specified. GraphQL adds complexity (schema stitching across gRPC services) with no clear benefit for this use case. |
| **MongoDB** | PostgreSQL is specified and appropriate. Manga metadata is relational (tags, works, users, relations). |

---

## Installation

### Workspace Cargo.toml (root)

```toml
[workspace]
resolver = "2"
members = [
    "crates/madome-core",
    "crates/madome-proto",
    "crates/madome-gateway",
    "crates/madome-auth",
    "crates/madome-catalog",
    "crates/madome-user",
    "crates/madome-file",
    "crates/madome-scraper",
]

[workspace.package]
edition = "2024"
rust-version = "1.85"

[workspace.dependencies]
# Async runtime
tokio = { version = "1.50", features = ["rt-multi-thread", "macros", "net", "time", "signal", "sync"] }
tokio-stream = "0.1"
futures-util = "0.3"

# REST framework
axum = "0.8"
axum-extra = { version = "0.12", features = ["cookie"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace", "timeout", "compression-gzip"] }

# gRPC
tonic = "0.14"
tonic-build = "0.14"
tonic-reflection = "0.14"
prost = "0.14"
prost-build = "0.14"

# Database
sea-orm = { version = "1.1", features = ["sqlx-postgres", "runtime-tokio-native-tls", "macros", "with-chrono", "with-uuid"] }
sea-orm-migration = "1.1"

# Auth
webauthn-rs = { version = "0.5", features = ["resident-key-support"] }
webauthn-rs-proto = "0.5"
jsonwebtoken = { version = "10", default-features = false, features = ["aws_lc"] }
minicbor = { version = "2.2", features = ["std"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# HTTP client
reqwest = { version = "0.13", features = ["json", "cookies", "gzip", "brotli"] }

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json", "fmt"] }

# Utilities
uuid = { version = "1", features = ["v7", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
bytes = "1"
base64 = "0.22"
rand = "0.10"
sha2 = "0.10"
parking_lot = "0.12"

# Caching
moka = { version = "0.12", features = ["future"] }

# Error handling
thiserror = "2"
anyhow = "1"

# Configuration
config = "0.15"

# Scraping
scraper = "0.26"
hitomi_la = "0.1"
```

### Build Dependencies (per service with gRPC)

```toml
[build-dependencies]
tonic-build = { workspace = true }
prost-build = { workspace = true }
```

### Dev Dependencies

```toml
[workspace.dependencies]
# Testing
tokio-test = "0.4"
```

---

## Key Configuration Decisions

### jsonwebtoken aws_lc Backend

The project specifies `aws_lc` backend for jsonwebtoken. This means:
- `default-features = false` to disable the default `ring` backend
- `features = ["aws_lc"]` to use aws-lc-rs
- aws-lc-rs requires CMake and a C compiler at build time
- Faster ECDSA operations than ring, FIPS-validated

### sea-orm Runtime Feature

Use `runtime-tokio-native-tls` (not `runtime-tokio-rustls`). The native-tls feature uses the OS TLS stack, which is simpler to configure for local PostgreSQL connections. If deploying to environments where native TLS is unavailable, switch to `runtime-tokio-rustls`.

### UUIDv7 for Primary Keys

UUIDv7 (time-sortable) is preferred over UUIDv4 (random) for PostgreSQL primary keys because:
- B-tree index-friendly (monotonically increasing)
- Embeds timestamp (useful for debugging, no separate created_at needed for ordering)
- No sequential ID guessing (unlike auto-increment)

### Protobuf Shared Crate

The `madome-proto` crate centralizes all generated protobuf code. All service crates depend on it. This ensures type consistency across services and avoids duplicate codegen.

---

## Sources

All versions verified via `https://crates.io/api/v1/crates/{crate_name}` on 2026-03-21:

- tonic 0.14.5 (2026-02-19)
- axum 0.8.8 (2025-12-20)
- sea-orm 1.1.19 stable (2025-11-11), 2.0.0-rc.37 (2026-03-09)
- prost 0.14.3 (2026-01-10)
- jsonwebtoken 10.3.0 (2026-01-27)
- webauthn-rs 0.5.4 stable (2025-12-10), 0.6.0-dev (2026-03-20)
- tokio 1.50.0 (2026-03-03)
- reqwest 0.13.2 (2026-02-06)
- moka 0.12.14 (2026-03-02)
- tracing 0.1.44 (2025-12-18)
- sea-orm-migration 1.1.19 (2025-11-11)
- hitomi_la 0.1.4 (2026-01-20)
- minicbor 2.2.1 (2026-02-24)
- config 0.15.22 (2026-03-17)
- All other versions documented inline

**Confidence notes:**
- Crate versions: HIGH (verified via crates.io API)
- Library feature flags and behavior: MEDIUM (based on training data, not verified against current docs -- WebSearch/WebFetch were unavailable)
- nginx configuration patterns: MEDIUM (standard patterns, but specific directive syntax should be verified against nginx docs)
- Architecture recommendations: HIGH (based on well-established Rust ecosystem patterns)
