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

Cargo workspace monorepo with 6 binaries and 3 shared crates:

| Binary | Role | Communication |
|--------|------|---------------|
| **gateway** | REST entry point, JWT verification, gRPC routing | REST (external) -> gRPC (internal) |
| **auth** | JWT issuance, session management, Passkey | gRPC |
| **catalog** | Book CRUD, tag queries, publishing, renewal | gRPC |
| **user** | tastes (like/dislike), histories | gRPC |
| **file** | Dedicated image upload/storage | REST (behind nginx reverse proxy) |
| **scraper** | External source mirroring (separate server) | REST -> Gateway (API Key) |

Each service with a database has the following folder structure:

```
service-name/
  schema/      -- sea-orm entity definitions
  src/         -- service implementation
  migration/   -- sea-orm migration files
```

### Shared Crates

| Crate | Role | Contents |
|-------|------|----------|
| **madome-proto** | Protobuf definitions | `.proto` files, tonic generated code |
| **madome-core** | Domain types | BookId, UserId, error trait, shared domain primitives |
| **madome-common** | Shared infrastructure | OpenTelemetry setup, config loading, shared middleware |

Principle: prevent god crate growth. If `madome-core` grows beyond domain types, split further.

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

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Microservice (6 binaries) | Independent deploy/scaling per service, separation of concerns | -- Pending |
| gRPC internal communication | Type safety, performance, code generation | Validated (Phase 01) |
| API Gateway pattern | Centralized JWT verification, prevent direct service exposure | Validated (Phase 01) |
| nginx auth_request + auth_request_set | Image serving auth + cookie refresh forwarding | -- Pending |
| Path-based API versioning (/v1/) | Simple, explicit, no /api prefix needed for API-only service | -- Pending |
| 3 shared crates (proto, core, common) | Prevent god crate; split by responsibility | Validated (Phase 01) |
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

---
*Last updated: 2026-03-21 — Phase 01 complete: foundation workspace, gateway REST-to-gRPC routing, shared crates*
