# Madome

## What This Is

A manga metadata and image mirroring service from external sources (initially hitomi.la). Built with a Rust-based microservice architecture, targeting a small community. Has potential to expand into an open service with automated image censorship in the future.

## Core Value

Reliably mirror works from external sources and allow authenticated users to browse them.

## Architecture

### Service Topology

Cargo workspace monorepo with 6 binaries:

| Binary | Role | Communication |
|--------|------|---------------|
| **gateway** | REST entry point, JWT verification, gRPC routing | REST (external) → gRPC (internal) |
| **auth** | JWT issuance, session management, Passkey | gRPC |
| **catalog** | Work CRUD, tag queries, publishing | gRPC |
| **user** | tastes (like/dislike), histories | gRPC |
| **file** | Dedicated image upload/storage | REST (behind nginx reverse proxy) |
| **scraper** | External source mirroring (separate server) | REST → Gateway (API Key) |

### Communication Flow

```
Client → REST → nginx → Gateway → gRPC → { auth, catalog, user }
Scraper → REST (API Key) → nginx → Gateway → gRPC → catalog
Scraper → REST → nginx → File Service → Local Filesystem
Image Read: Client → nginx auth_request → Gateway (auth check) → nginx serves file
```

- **Inter-service**: gRPC (tonic)
- **External API**: REST (axum, JSON)
- **Scraper → Gateway**: REST + API Key authentication
- **Image serving**: nginx `auth_request` + `auth_request_set` for cookie refresh forwarding
- All services run behind nginx reverse proxy

### Auth Design

**Passkey + JWT + Session hybrid:**

1. Authenticate via Passkey (webauthn-rs) → create session + issue JWT
2. JWT access token (15 min TTL), managed via HttpOnly Cookie
3. Client does not manage token refresh (server-side automatic management)

**Gateway JWT processing flow:**

1. JWT valid → stateless pass-through (no auth service call)
2. JWT expired + within Grace Period → pass-through + set new JWT cookie
3. JWT expired + past Grace Period → verify session, issue new JWT
4. Session invalid → reject request

**Duplicate JWT prevention:** Per-session JWT caching (reuse same JWT within N seconds)

**Image request token refresh:** nginx `auth_request_set $auth_cookie $upstream_http_set_cookie` + `add_header Set-Cookie $auth_cookie` forwards Set-Cookie from auth subrequest to client

### Upload Scenario

1. Scraper periodically checks external source for new works
2. New work found → upload metadata to Catalog (unpublished, includes hash of work info + tags)
3. Upload images to File service
4. Scraper requests Catalog to publish the work
5. Catalog verifies page count matches actual image count → publish

### Update/Renewal Scenario

**Update check frequency (decreasing intervals):**
- Within 6 hours: every 5 minutes
- 6h~24h: every 30 minutes
- 1d~7d: every 2 hours
- 7d+: every 12 hours

**Same work ID:** Scraper requests info update to Catalog. Done.

**Work ID changed (renewal):**
- Preserve original work data (no overwriting)
- Link duplicate works via graph/relation
- Canonical ID: new ID becomes canonical, old ID redirects
- Work info page shows renewal history
- DB-based queue updates work ID references across services (user, etc.)
- Image files are not moved
- Future migration to message broker possible

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Gateway REST API + gRPC routing
- [ ] Passkey authentication (webauthn-rs)
- [ ] JWT + Session hybrid authentication
- [ ] Catalog work CRUD (including publish workflow)
- [ ] Query works by single tag
- [ ] Query works by multiple tags
- [ ] Query works by ID list
- [ ] Image upload (File service)
- [ ] nginx auth_request-based image serving
- [ ] Work info update checking (decreasing frequency)
- [ ] Work ID renewal handling (graph relation, canonical ID, queue)
- [ ] User tastes (like/dislike)
- [ ] User histories
- [ ] Scraper: hitomi.la new work detection and mirroring

### Out of Scope

- Frontend UI — API-first, frontend later
- Automated image censorship — for future open service consideration
- Reporting/moderation — v2+
- Message broker (RabbitMQ, etc.) — DB-based queue sufficient, revisit later
- OAuth/social login — Passkey only
- Mobile app — web API first

## Context

- Higher work numbers on hitomi.la indicate more recent works
- Work renewal is primarily caused by deduplication on the external source
- Existing image/data already on local filesystem, migration planned just before release
- No direct FK between separate service DBs → treat as indirect FK (queue-based sync)
- `hitomi_la` crate available for use

## Constraints

- **Tech Stack**: Rust (Cargo workspace monorepo)
- **Database**: PostgreSQL (sea-orm)
- **Internal Comm**: gRPC (tonic + prost)
- **External API**: REST (axum)
- **Image Storage**: Local filesystem
- **Reverse Proxy**: nginx (auth_request, reverse proxy)
- **Scraper Deployment**: Runs on separate server from API
- **Auth**: Passkey only (webauthn-rs, minicbor for AAGUID)
- **Token**: JWT (jsonwebtoken, aws_lc backend)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Microservice (6 binaries) | Independent deploy/scaling per service, separation of concerns | — Pending |
| gRPC internal communication | Type safety, performance, code generation | — Pending |
| API Gateway pattern | Centralized JWT verification, prevent direct service exposure | — Pending |
| nginx auth_request + auth_request_set | Image serving auth + cookie refresh forwarding | — Pending |
| DB-based queue (not message broker) | Minimize infra complexity, migration path available | — Pending |
| Canonical ID + graph relation | Preserve historical data, track renewal history | — Pending |
| Per-session JWT caching | Prevent unnecessary duplicate token generation on concurrent requests | — Pending |
| Separate File service | Avoid Gateway load from image traffic | — Pending |
| Decreasing update check frequency | More recent works have higher change probability | — Pending |

---
*Last updated: 2026-03-21 after initialization*
