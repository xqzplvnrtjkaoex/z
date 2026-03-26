# Madome

## What This Is

A manga metadata and image mirroring service from external sources (initially hitomi.la). Built with a Rust-based microservice architecture, targeting a small community. Has potential to expand into an open service with automated image censorship in the future.

## Core Value

Reliably mirror books from external sources and allow authenticated users to browse them.

## Architecture

### URL Convention

- Path-based versioning: `/v1/books/123`
- No `/api` prefix (API-only service, no ambiguity)

### Service Topology

Cargo workspace monorepo with 6 binaries and 2 shared crates:

| Binary | Role | Communication |
|--------|------|---------------|
| **gateway** | REST entry point, JWT verification, gRPC routing, multi-service orchestration | REST (external) -> gRPC (internal) |
| **auth** | JWT issuance, session management, Passkey | gRPC |
| **catalog** | Book CRUD, tag queries, publishing, renewal | gRPC |
| **user** | user profiles, tastes (like/dislike), histories | gRPC |
| **file** | Dedicated image upload/storage | REST (behind nginx reverse proxy) |
| **scraper** | External source mirroring (separate server) | REST -> Gateway (API Key) |

Each service with a database has the following folder structure:

```
service-name/
  schema/      -- sea-orm entity definitions
  src/         -- service implementation (layered architecture below)
  migration/   -- sea-orm migration files
```

### Internal Service Architecture

All services (except gateway) follow a consistent 4-layer architecture with trait-based dependency inversion. Uses `mod.rs` convention for modules.

**Layers:**

| Layer | Folder | Depends On | Responsibility |
|-------|--------|-----------|----------------|
| **domain/** | types/, ports/, error/ | Nothing external | Pure types, trait port definitions, domain errors |
| **usecase/** | per-use-case modules | domain/ only | Business logic orchestration |
| **app/** | handler/ | domain/ only | tonic gRPC trait impl (thin: validate, delegate, map) |
| **adapter/** | subdirectories (postgres/, redis/, etc.) + context.rs | domain/ + external crates | Concrete implementations of domain traits |
| **main.rs** | — | All layers | Composition root: assembles adapters, injects into handler |

**Dependency direction (enforced via module visibility + generics):**

```
main.rs (assembles all layers — sole place that knows concrete types)
   |
app/rpc/     ---> domain/ (ports, types, error)
   |                ^
usecase/*    ------/
   |                ^
adapter/*   ------/
```

**Key patterns:**

- **Ports/Config separation:** Each service defines **two separate traits** in `domain/ports/`:
  - `{Service}Ports` — provides `&impl Trait` accessors (RPITIT) for data ports (repositories, external clients)
  - `{Service}Config` — provides config accessors via `fn config(&self) -> &impl ConfigAccessor` with getter methods (e.g., `ctx.config().jwt_ttl()`)
  - `{Service}Context` struct in `adapter/context.rs` implements both traits
  - Compound bound reused via type alias: `type Context = impl {Service}Ports + {Service}Config + ?Sized;`
- **Generic handler:** `{Service}Handler<C: {Service}Ports>` in `app/rpc/` — generic over ports, concrete type resolved only in `main.rs`. Each RPC function is named `execute` in its own file.
- **Use case functions:** Free async functions in `usecase/`, receive `&(impl Ports + ?Sized)` as first argument.
- **Error separation:** `RepositoryError` (data-access facts) in `domain/error/`, `{Service}Error` (business meaning) in `domain/error/`. Usecase maps repository errors to domain errors. Handler maps domain errors to `tonic::Status`.
- **Domain types + From:** Separate domain types in `domain/types/`. `From<sea_orm::Model> for DomainType` in adapter/ (infra knows domain). `From<DomainType> for ProtoResponse` in app/ (handler knows domain + proto).
- **Async traits:** Prefer `trait_variant` over `async_trait` for async trait definitions. Use `async_trait` only when `trait_variant` is insufficient.
- **Usecase parameters:** Data parameters use "payload" suffix (e.g., `RegisterFinishPayload`), not "input".

**Example structure (auth service):**

```
services/auth/src/
  domain/
    mod.rs
    types/
      mod.rs
      credential.rs       # Credential, StoredCredential
      session.rs          # Session
      invite.rs           # Invite
      api_key.rs          # ApiKey
      recovery_code.rs    # RecoveryCode
    ports/
      mod.rs              # AuthPorts trait
      credential_repo.rs  # trait CredentialRepository
      session_store.rs    # trait SessionStore
      passkey_provider.rs # trait PasskeyProvider
      jwt_issuer.rs       # trait JwtIssuer
      user_service_port.rs # trait UserServicePort (gRPC client to user service)
    error/
      mod.rs
      auth_error.rs       # AuthError enum (business errors)
      repository_error.rs # RepositoryError enum (data-access errors)
  usecase/
    mod.rs
    register/             # begin.rs, finish.rs
    login/                # begin.rs, finish.rs
    session/              # create.rs, validate.rs, invalidate.rs
    recovery/             # begin.rs, finish.rs, regenerate.rs
    api_key/              # create.rs, verify.rs, revoke.rs
    passkey_mgmt/         # list.rs, rename.rs, delete.rs
  app/
    mod.rs
    handler/
      mod.rs              # AuthHandler<C: AuthPorts>
      register.rs         # register RPC impls
      login.rs            # login RPC impls
      ...
  adapter/
    mod.rs
    context.rs            # AuthContext: impl AuthPorts + AuthConfig
    postgres/
      mod.rs
      credential_repo.rs  # impl CredentialRepository
      invite_repo.rs      # impl InviteRepository
      api_key_repo.rs     # impl ApiKeyRepository
      recovery_code_repo.rs # impl RecoveryCodeRepository
    redis/
      mod.rs
      session_store.rs    # impl SessionStore
    webauthn/
      mod.rs              # impl PasskeyProvider
    jwt/
      mod.rs              # impl JwtIssuer (ES256)
    user_client/
      mod.rs              # impl UserServicePort (gRPC client)
  main.rs
```

**Gateway exception:** Gateway is a pure REST-to-gRPC translator and does NOT follow the 4-layer pattern. It uses `routes/`, `middleware/`, `state.rs` (established in Phase 1).

**Framework usage:** Only gateway and file service use axum. Auth, catalog, and user services use tonic only.

**Testing:**

- **Unit tests:** Mock ports via `mockall` crate for isolated business logic testing. Verify usecase logic without real infrastructure.
- **Integration tests:** testcontainers with real PostgreSQL/Redis. Single container per suite via `OnceLock`. Container lifecycle managed in code (not docker-compose). Test through actual adapters.
- **Service tests:** tonic in-process channel with real adapters + testcontainers.
- **E2E tests:** Through Gateway REST API, scenario-based.
- **Shared utilities:** `crates/madome-test-utils/` crate provides shared test infrastructure (container setup, factory functions, test config) across services.
- **Test crate:** `tests/` directory as separate workspace member (`madome-tests`) for service-level and E2E tests. Structure: `tests/service/auth.rs`, `tests/e2e/auth_flow.rs`.

### Shared Crates

| Crate | Role | Contents |
|-------|------|----------|
| **madome-proto** | Protobuf definitions | `.proto` files, tonic generated code |
| **madome-common** | Shared infrastructure | Tracing init, env helpers, header constants, `CallerIdentity` + `CallerRole` |

Domain types live in each service's `domain/` layer, not in a shared crate. Proto definitions serve as the cross-service contract.

**CallerIdentity:** Shared struct in `madome-common` representing validated caller identity (who is making this request). Used by both Gateway (extracted from HTTP headers) and gRPC services (extracted from gRPC metadata, injected by Gateway).

| Field | Type | Source |
|-------|------|--------|
| `caller_id` | `Uuid` | `x-caller-id` header/metadata |
| `caller_role` | `CallerRole` | `x-caller-role` header/metadata |

`CallerRole` is a standalone enum (`User`, `Admin`, `Owner`) in `madome-common` — not proto `Role` (which includes `Unspecified`). Each service converts `CallerRole` to its domain role type via `From` impl. Adding a variant to `CallerRole` produces compile errors at all `match` sites, enforcing exhaustive handling.

### Communication Flow

```
Client -> REST -> nginx -> Gateway -> gRPC -> { auth, catalog, user }
Scraper -> REST (API Key) -> nginx -> Gateway -> gRPC -> catalog
Scraper -> REST -> nginx -> File Service -> Local Filesystem
Image Read: Client -> nginx auth_request -> Gateway (auth check) -> nginx serves file
Catalog -> File Service (gRPC): image count verification for publish workflow
```

- **Inter-service**: gRPC (tonic)
- **External API**: REST (axum, JSON)
- **Scraper -> Gateway**: REST + API Key authentication
- **Image serving**: nginx `auth_request` + `auth_request_set` for cookie refresh forwarding
- All services run behind nginx reverse proxy

### Authentication Policy

Private service — all endpoints require authentication. There is no anonymous access. Gateway middleware extracts `CallerIdentity` from every request; handlers reject requests missing identity via `Extension<CallerIdentity>`. gRPC services use `CallerIdentity::from_metadata` (mandatory extraction, returns `Unauthenticated` on missing headers).

### Role Hierarchy

3-tier role system: **owner > admin > user**

| Rule | Description |
|------|-------------|
| Hierarchy enforcement | Can only manage roles strictly below own level |
| Same-level blocked | Admin cannot manage other admins; only owner can |
| Self-modification blocked | Cannot change own role or deactivate self |
| Minimum owner | At least 1 active owner must exist at all times |

### Service-to-Service Communication

Direct gRPC calls between internal services are allowed. Communication is not restricted to Gateway-to-service only.

- **Auth -> User service:** Auth calls User service for user creation (registration) and user lookup (login, JWT claims)
- **Future services:** Any service may call another directly via gRPC when the use case requires it

Each calling service requires the target service's `{SERVICE}_GRPC_ADDR` environment variable.

### Cross-Service Operations

Gateway orchestrates sequential calls to multiple services with compensating transactions for operations that span service boundaries.

**Level 1 (current):** Compensate-on-failure with error log + error response to client.
**Level 2 (future):** Outbox + background worker retry for stronger consistency guarantees.

**Example: User Deactivation**
```
Gateway -> User(DeactivateUser) -> Auth(InvalidateAllSessions)
  Auth failure -> compensate via User(ActivateUser)
  Compensate failure -> error log + error response to client
```

### Pagination

All list endpoints use cursor-based pagination.

| Aspect | Convention |
|--------|-----------|
| Cursor format | Opaque URL-safe value |
| Server -> client | `X-Next-Cursor` response header (omitted when no more pages) |
| Client -> server | `?cursor=` query parameter |
| Response body | Flat array (no wrapper object) |
| Page size | `?limit=N` (default 25, max 100) |
| Sort | `?sort=field-order` kebab-case (e.g., `?sort=created-at-desc`) |

### API Conventions

| Context | Convention | Example |
|---------|-----------|---------|
| Query string parameters | kebab-case | `?sort=created-at-desc`, `?limit=25` |
| Request/response body fields | snake_case | `{ "user_id": "...", "recovery_codes": [...] }` |

### ID Design

**Internal PK + External ID separation:**

| Column | Type | Purpose |
|--------|------|---------|
| `id` (PK) | UUID | Internal identifier, all services reference this |
| `external_id` | i64 | Source platform ID (hitomi.la number), can change on renewal |

**UUID Policy:**

| Entity | UUID Version | Reason |
|--------|-------------|--------|
| Book, Taste, History record | UUIDv7 | Time-sortable, predictability acceptable |
| User, Session, API Key | UUIDv4 | Must be unpredictable |
| request_id | UUIDv7 | Time-sortable for log correlation |

### User Identity

Users table lives in the User service DB:

| Column | Type | Notes |
|--------|------|-------|
| `id` (PK) | UUIDv4 | Must be unpredictable |
| `handle` | VARCHAR, UNIQUE | Case-insensitive, URL-safe identifier (e.g., `syr`) |
| `name` | VARCHAR | Display name, non-unique |
| `role` | ENUM | owner / admin / user |
| `is_active` | BOOLEAN | Deactivation flag |
| `created_at` | TIMESTAMPTZ | |
| `updated_at` | TIMESTAMPTZ | |

- `handle` is the user-facing unique identifier (appears in JWT claims, URLs, admin views)
- `name` is a display-only field with no uniqueness constraint
- Auth service references `user_id` without FK (cross-service boundary); calls User service via gRPC for user creation and lookup

### Auth Design

**Passkey + JWT + Session hybrid:**

1. Authenticate via Passkey (webauthn-rs) -> create session + issue JWT
2. JWT access token (15 min TTL), managed via HttpOnly Cookie
3. Client does not manage token refresh (server-side automatic management)

**Gateway JWT processing flow:**

1. JWT valid -> stateless pass-through (no auth service call)
2. JWT expired + within Grace Period -> pass-through + set new JWT cookie
3. JWT expired + past Grace Period -> verify session, issue new JWT
4. Session invalid -> reject request

**Registration flow (Signup vs Register):**

- **SignupBegin/SignupFinish** (`/v1/auth/signup/*`): Full signup flow — invite validation, user creation (via User service), passkey ceremony, session/JWT issuance, recovery code generation
- **RegisterBegin/RegisterFinish** (`/v1/auth/register/*`): Pure passkey ceremony — WebAuthn challenge/credential only, for authenticated users adding passkeys
- Internal WebAuthn ceremony logic is shared between both flows; Signup wraps it with signup-specific orchestration

**Duplicate JWT prevention:** Per-session JWT caching (reuse same JWT within N seconds)

**Image request token refresh:** nginx `auth_request_set $auth_cookie $upstream_http_set_cookie` + `add_header Set-Cookie $auth_cookie` forwards Set-Cookie from auth subrequest to client

### Upload Scenario

1. Scraper periodically checks external source for new books
2. New book found -> upload metadata to Catalog (unpublished, includes hash of book info + tags)
3. Upload images to File service
4. Scraper requests Catalog to publish the book
5. Catalog requests File service for actual image count -> verifies against metadata page count -> publish

### Renewal Design

**Problem:** On hitomi.la, books are frequently renewed (deleted and re-uploaded under a new external ID). This can happen multiple times for the same book. The renewed book (new external_id) may already exist in the catalog.

**DB Structure:**
- `books.canonical_id` column: denormalized Union-Find root pointer
- `book_relations` table: preserves renewal history graph
- Current version: rows where `id = canonical_id`

**Canonical ID Update (single SQL, handles chain merging):**
```sql
UPDATE books SET canonical_id = :new_canonical
WHERE canonical_id = :old_canonical;
```

**Cross-service impact:** None. User service likes/history stay linked to original PK. Canonical resolution happens at query-time in catalog. No cross-service sync queue needed.

**Renew API:**
- `POST /v1/books/renew { old_external_id, new_external_id }`
- Both books must exist in catalog (published)
- Creates relation record + updates canonical_id chain
- Upload flow and renewal are completely separate concerns

### Scraper Architecture

Two isolated tasks running on separate threads:

**Task A: Update Checker**
Monitors catalog books for metadata changes and renewals (decreasing frequency).

```
For each book in catalog:
  Request metadata from hitomi.la using known external_id

  Response id matches -> Update Process (update metadata in catalog)
  Response id differs (ext:111 -> ext:222) ->
    ext:222 in catalog:
      -> Update Process for ext:222 (update metadata)
      -> Renewal Process (call renew API)
    ext:222 not in catalog:
      -> Delegate ext:222 upload to New Book Discovery
      -> After upload completes -> Renewal Process
```

**Task B: New Book Discovery**
Discovers and uploads new books.

```
- Detects new books on hitomi.la not in catalog -> upload flow
- Receives upload requests from Update Checker -> upload flow
- Upload flow: create metadata -> upload images -> publish
```

**Principle:** Update Checker handles metadata checking/updating only. All uploads go through New Book Discovery.

**Update check frequency (decreasing intervals):**
- Within 6 hours: every 5 minutes
- 6h~24h: every 30 minutes
- 1d~7d: every 2 hours
- 7d+: every 12 hours

### Observability

- **Stack:** OpenTelemetry + tracing crate
- **request_id:** UUIDv7, generated at gateway, propagated via gRPC metadata
- **Span hierarchy:** request -> service -> db_query
- **Backend:** TBD (decided with infrastructure)

## Requirements

### Validated

- [x] Gateway REST API + gRPC routing — Validated in Phase 01: Foundation and Gateway Infrastructure

### Active

- [ ] Gateway REST API + gRPC routing (extended: middleware, auth integration)
- [ ] Passkey authentication (webauthn-rs)
- [ ] JWT + Session hybrid authentication
- [ ] Catalog book CRUD (including publish workflow)
- [ ] Query books by single tag
- [ ] Query books by multiple tags
- [ ] Query books by ID list
- [ ] Image upload (File service)
- [ ] nginx auth_request-based image serving
- [ ] Book metadata update checking (decreasing frequency)
- [ ] Book renewal handling (canonical_id, relation table, renew API)
- [ ] User tastes (like/dislike)
- [ ] User histories
- [ ] Scraper: hitomi.la new book detection and mirroring

### Out of Scope

- Frontend UI -- API-first, frontend later
- Automated image censorship -- for future open service consideration
- Reporting/moderation -- v2+
- Message broker (RabbitMQ, etc.) -- not needed (no cross-service sync queue)
- OAuth/social login -- Passkey only
- Mobile app -- web API first

## Context

- Higher book numbers on hitomi.la indicate more recent books
- Book renewal is frequent on hitomi.la; a book can be renewed multiple times
- The renewed book (new external_id) may already exist in catalog
- Existing image/data already on local filesystem, migration planned just before release
- No direct FK between separate service DBs -> user data stays linked to original PK, canonical resolution at query-time
- `hitomi_la` crate available for use

## Constraints

- **Tech Stack**: Rust (Cargo workspace monorepo)
- **Database**: PostgreSQL (sea-orm), schema/migration per service
- **Internal Comm**: gRPC (tonic + prost)
- **External API**: REST (axum), path-based versioning (/v1/...)
- **Image Storage**: Local filesystem
- **Reverse Proxy**: nginx (auth_request, reverse proxy)
- **Scraper Deployment**: Runs on separate server from API
- **Auth**: Passkey only (webauthn-rs, minicbor for AAGUID)
- **Token**: JWT (jsonwebtoken, aws_lc backend)
- **Observability**: OpenTelemetry + tracing
- **ID Strategy**: UUIDv7 (predictable OK) / UUIDv4 (must be unpredictable)

## System-Wide Rules

Rules that apply to ALL services and ALL phases. Phase Rules (PR) and
operation Rules (R) may reference these. Override requires explicit
justification in the phase's CASES.md Phase Rules section.

| ID | Rule | Scope | Source |
|----|------|-------|--------|
| SR-01 | All inter-service gRPC calls use 5-second timeout via `tonic::Request::set_timeout` | All gRPC callers | D-135 (3A) |
| SR-02 | Service unavailability in synchronous call chain = operation failure; no retry, no partial recovery | All cross-service calls | D-136 (3A) |
| SR-03 | Detailed error info logged server-side with request_id correlation; generic errors to clients | All services | D-54 (3A) |
| SR-04 | UUIDv4 for security-sensitive entities; UUIDv7 for time-sortable entities | All services | ID Design |

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Microservice (6 binaries) | Independent deploy/scaling per service, separation of concerns | -- Pending |
| gRPC internal communication | Type safety, performance, code generation | Validated (Phase 01) |
| API Gateway pattern | Centralized JWT verification, prevent direct service exposure | Validated (Phase 01) |
| nginx auth_request + auth_request_set | Image serving auth + cookie refresh forwarding | -- Pending |
| Path-based API versioning (/v1/) | Simple, explicit, no /api prefix needed for API-only service | -- Pending |
| 2 shared crates (proto, common) | Proto for cross-service contract, common for infra utilities. Domain types stay in each service | Validated (Phase 02) |
| DB schema/migration per service | Each service owns its data; independent migration | -- Pending |
| Internal PK + external_id separation | Decouple from source platform; stable references across renewal | -- Pending |
| canonical_id denormalization | O(1) read for renewal chain resolution; Union-Find pattern | -- Pending |
| No cross-service sync queue | Renewal resolved in catalog via canonical_id; user data stays on original PK | -- Pending |
| Catalog -> File service for image count | Catalog doesn't access filesystem directly; separation of concerns | -- Pending |
| Scraper: two isolated tasks | Update checking and new book discovery have different concerns and schedules | -- Pending |
| UUIDv7 for predictable entities | Time-sortable PKs for books, records; UUIDv4 for security-sensitive entities | -- Pending |
| UUIDv7 for request_id | Time-sortable for log correlation | -- Pending |
| OpenTelemetry | Standard observability; backend TBD | -- Pending |
| Per-session JWT caching | Prevent unnecessary duplicate token generation on concurrent requests | -- Pending |
| Separate File service | Avoid Gateway load from image traffic | -- Pending |
| Decreasing update check frequency | More recent books have higher change probability | -- Pending |
| 4-layer service architecture (domain/usecase/app/adapter) | Compile-time dependency direction, trait-based ports, testable business logic | -- Pending |
| Ports trait with RPITIT (`&impl Trait`) | Zero-cost DI without generic parameter explosion | -- Pending |
| Per-service domain errors + RepositoryError | Business meaning separation; repo reports facts, usecase interprets | -- Pending |
| `mod.rs` module convention | Consistent for deep nesting (3-4 levels) | -- Pending |
| 3-tier role hierarchy (owner/admin/user) | Hierarchy-based access control; manage only lower roles | -- Pending |
| Service-to-service direct gRPC | Services call each other directly, not restricted to Gateway-only routing | -- Pending |
| Compensating transactions (Level 1) | Sequential cross-service calls with compensate-on-failure; correctness over complexity | -- Pending |
| Cursor-based pagination | Opaque cursor via X-Next-Cursor header; scalable for large datasets | -- Pending |
| API conventions (kebab-case query, snake_case body) | Consistent casing rules across all endpoints | -- Pending |
| Ports/Config trait separation | Separate data access ports from configuration; both on {Service}Context | -- Pending |
| Adapter directory structure | Subdirectories from start (postgres/, redis/, etc.); scales without refactoring | -- Pending |
| trait_variant over async_trait | Zero-cost async traits where possible; async_trait as fallback | -- Pending |
| Payload naming convention | Usecase data params use "payload" suffix for clarity | -- Pending |
| madome-test-utils shared crate | Shared test utilities (containers, factories, config) across services | -- Pending |
| Handle as unique user identifier | URL-safe, case-insensitive handle separate from display name; name is non-unique | -- Pending |
| Users table in User service | Auth service references user_id without FK; calls User service via gRPC | -- Pending |
| CallerIdentity in madome-common | Shared caller identity type with own CallerRole enum (not proto Role); avoids invalid Unspecified state; exhaustive match on variant additions | Validated (Phase 02) |
| Proto `bytes` for UUID fields | 16 bytes vs 36; no format ambiguity; Gateway uses `Path<Uuid>` for direct deserialization | Validated (Phase 02) |
| Gateway multi-service orchestration | Not restricted to proxy/routing; compose multi-service flows when trade-offs favor it | -- Pending |
| Signup/Register operation separation | Signup = full flow (invite+user+passkey+session); Register = pure passkey ceremony (reusable for adding passkeys) | -- Pending |

---
*Last updated: 2026-03-22 — Phase 2 complete: User service fully operational with 8 gRPC RPCs, domain layer, PostgreSQL adapter, gateway REST routes, 73 tests (unit + integration + service)*
