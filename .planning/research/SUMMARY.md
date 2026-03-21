# Project Research Summary

> **Terminology note:** This document uses "work" throughout. The project canonical term is **"book"**. See PROJECT.md for current architecture decisions -- some research recommendations (e.g., cross-service sync queue, centralized migration crate) have been superseded.

**Project:** Madome — Manga Mirroring and Aggregation Service
**Domain:** Rust microservice content mirroring platform (manga aggregation)
**Researched:** 2026-03-21
**Confidence:** HIGH (stack verified via crates.io; architecture patterns well-established; pitfalls directly derived from project design)

## Executive Summary

Madome is a private, authenticated manga mirroring service that scrapes hitomi.la, stores images locally, and serves them through a microservices API. The recommended approach is a Rust Cargo workspace containing 6 binary crates (gateway, auth, catalog, user, file, scraper) backed by a single PostgreSQL instance with per-service schemas, communicating via gRPC internally and exposing REST externally through an axum gateway. The entire stack — tokio, axum, tonic, sea-orm, webauthn-rs — is mature, version-verified, and forms a coherent Rust async ecosystem without mixing incompatible runtimes. This architectural choice enables shared middleware (tower) between REST and gRPC layers, which is a significant operational advantage.

The product's genuine differentiator in the manga aggregation space is work renewal tracking: when hitomi.la deletes and re-uploads a work under a new ID, Madome preserves the full continuity of user likes, reading history, and identity through a canonical ID graph and async cross-service sync queue. No competing platform (hitomi.la, nhentai, e-hentai, Komga) does this. The MVP requires authentication, content pipeline (scraper + catalog + file service), and basic browsing and search before user preference features add value. Build in that order.

The two highest-severity risks are: (1) the nginx `auth_request_set` directive silently dropping cookie headers in the image serving authentication flow — this must be locked down in Phase 1 with a strict single-cookie contract; and (2) the DB-based cross-service sync queue for work renewal orphaning user references if queue processing fails partway — this requires idempotent consumers and per-service status tracking from day one. All other pitfalls (scraper fragility, JWT race conditions, entity-migration drift, proto boundary coupling) have clear, well-established mitigations that must be established in their respective phases.

## Key Findings

### Recommended Stack

The Rust stack is fully specified with versions verified against crates.io on 2026-03-21. The core runtime is tokio 1.50.0 with axum 0.8.8 for REST and tonic 0.14.5 for gRPC — all using the same tower middleware layer, enabling shared rate limiting, tracing, and auth across both protocols. Database access uses sea-orm 1.1.19 (stable, not the churn-heavy 2.0.0-rc.37 line), and passkey authentication uses webauthn-rs 0.5.4 with jsonwebtoken 10.3.0 and an aws-lc-rs FIPS crypto backend. The workspace resolver must be "2" (edition 2024) to prevent Cargo feature unification from polluting service binaries with unintended dependencies.

Two notable non-obvious stack decisions: (1) use a custom `tokio::time::sleep` loop instead of tokio-cron-scheduler for the decreasing-frequency update check pattern, since the computed-delay model maps more naturally to the algorithm than cron expressions; (2) use moka 0.12 for in-memory JWT caching rather than Redis, since a single gateway process does not need distributed cache — Redis is the correct upgrade path only if the gateway is horizontally scaled.

**Core technologies:**
- **tokio 1.50.0**: async runtime — the only production-grade option; every other crate depends on it
- **axum 0.8.8**: REST framework for gateway and file service — shares tower middleware with tonic
- **tonic 0.14.5 + prost 0.14.3**: gRPC for internal service communication — versions must be aligned
- **sea-orm 1.1.19**: async ORM — stable, not 2.0-rc; provides migration tooling and entity derives
- **webauthn-rs 0.5.4**: passkey authentication — the only mature Rust WebAuthn implementation
- **jsonwebtoken 10.3.0**: JWT with aws-lc-rs backend — use `default-features = false, features = ["aws_lc"]`
- **moka 0.12.14**: per-session JWT cache with TTL — simpler than Redis for single-process caching
- **nginx 1.26+**: reverse proxy, TLS termination, auth_request subrequests, static image serving

### Expected Features

The feature set has a clear dependency hierarchy: auth is the universal prerequisite, the content pipeline (scraper + catalog + file service) is the second critical path, and user features (tastes, history) require both auth and content before they provide any value.

**Must have (table stakes for MVP):**
- Passkey authentication — gate access to the private community; without it the service is an open mirror
- Scraper: hitomi.la new work detection and ingestion — the content pipeline; without it there is no ongoing value
- Catalog CRUD with publish workflow — stores and serves work metadata with page count verification
- Image upload and nginx-served image delivery — the core value proposition
- Catalog browsing (paginated, recency-sorted) — primary interaction loop
- Tag-based filtering with multi-tag compound queries — primary discovery mechanism in this domain
- Basic search (PostgreSQL full-text or trigram, no Elasticsearch needed at this scale)
- Work metadata update checking with decreasing-frequency schedule

**Should have (differentiators, add post-MVP validation):**
- User tastes (binary like/dislike) — enables negative filtering, stronger than favorites-only
- Reading history with per-page progress — no competing platform offers this with a full account system
- Work renewal tracking (canonical ID graph, cross-service sync) — genuinely novel; the strongest differentiator
- Taste-based negative filtering — high value once taste data accumulates
- Scraper failure monitoring and alerting — operational necessity once the content pipeline is live

**Defer to v2+:**
- Additional source scrapers — design the adapter interface now, implement only hitomi.la
- RSS/Atom feeds — low cost, low priority; implement when power users request it
- Automated image censorship — ML infrastructure overhead, only relevant if the service goes public
- Recommendations — requires critical mass of taste data and catalog size

### Architecture Approach

The architecture follows the gateway pattern: nginx handles TLS termination and static image serving with auth_request subrequests; the gateway (axum) translates REST to gRPC and manages the JWT lifecycle; three gRPC backend services (auth, catalog, user) each own their PostgreSQL schema; the file service accepts image uploads via REST and writes to the local filesystem; the scraper runs as a separate process and communicates with the gateway via REST using an API key. A single `madome-proto` crate compiles all .proto files and is shared by all services — this prevents version drift and eliminates duplicate codegen. A single `madome-migration` crate manages all schemas in one runner, which is the pragmatic choice for a small-team deployment even though services are logically isolated. Cross-service data consistency for work renewals uses a DB-based queue with `SELECT ... FOR UPDATE SKIP LOCKED` pattern rather than a message broker (explicitly out of scope per project constraints).

**Major components:**
1. **nginx** — TLS termination, auth_request subrequests for images, static file serving
2. **Gateway (axum)** — JWT verification/refresh, REST-to-gRPC translation, API key auth for scraper
3. **Auth service (tonic)** — Passkey registration/authentication, session CRUD, JWT issuance
4. **Catalog service (tonic)** — Work CRUD, tag queries, publish workflow, renewal graph
5. **User service (tonic)** — Like/dislike tastes, reading histories per user
6. **File service (axum REST)** — Image upload, filesystem management, page count validation
7. **Scraper** — External source polling, decreasing-frequency update checks, upload pipeline
8. **madome-proto / madome-core / madome-entity / madome-migration** — Shared library crates

### Critical Pitfalls

1. **nginx auth_request_set drops multiple Set-Cookie headers** — The `$upstream_http_set_cookie` variable captures only the last cookie. Design the Gateway auth endpoint to return exactly one `Set-Cookie` header. Never add a second cookie to the auth subrequest response without redesigning the forwarding mechanism. Address in Phase 1.

2. **JWT grace-period refresh race condition** — Concurrent image requests with an expired-but-within-grace JWT can all miss the cache and generate separate new JWTs. Use per-session-id locking (a `DashMap<SessionId, tokio::sync::Mutex<Option<CachedJwt>>>`) so concurrent requests for the same session serialize at the first refresh. Address in Phase 1.

3. **Work ID renewal orphans cross-service references** — A partial queue failure leaves catalog updated but user service still referencing old IDs. Every queue consumer must be idempotent, use `(old_id, new_id)` as a natural idempotency key, track per-service completion status, and implement max-retry alerting. Address in Phase 5 (renewal design) with gateway-level canonical ID redirect as a consistency safety net.

4. **Scraper breaks silently when external source structure changes** — The hitomi_la crate is subject to upstream changes. Implement canary checks against a known verified work before each scraping run, schema validation at ingestion, and an error-rate circuit breaker (halt if failure rate exceeds 10%). Address in Phase 4 (scraper implementation).

5. **sea-orm entity-migration drift** — Migration files and entity structs can diverge without compiler errors. Establish a strict workflow: write migration, apply to dev database, regenerate entities with sea-orm-cli, commit both together. Add a CI check that regenerates entities from a fresh migration run and diffs against committed files. Address in Phase 1 (database setup).

## Implications for Roadmap

Based on research, the build order is dictated by hard dependencies: auth is required before any other user-facing feature; catalog is required before the scraper can upload; file service is required before images can be stored; user service is additive; renewal tracking is the last complex integration. The architecture research explicitly derives this 6-phase ordering, which maps cleanly to deliverable milestones.

### Phase 1: Foundation and Authentication

**Rationale:** Auth is the universal prerequisite. The JWT middleware, nginx auth_request single-cookie contract, and proto/workspace setup are foundational decisions that cannot be changed cheaply later. Getting these right first prevents cascading rework across all services.

**Delivers:** Working Cargo workspace with shared crates, all .proto definitions, database schema with all migrations, passkey registration and authentication flow, JWT issuance and verification, nginx auth_request plumbing, gateway skeleton with JWT middleware.

**Addresses:** Passkey authentication (table stakes), image access control prerequisite.

**Avoids:**
- nginx auth_request_set cookie limitation (single-cookie contract enforced from day one)
- JWT grace-period refresh race condition (per-session locking built into auth from the start)
- Proto boundary coupling (workspace and proto crates set up correctly before any services)
- Cargo feature unification (workspace.resolver = "2", minimal shared features)
- sea-orm migration drift (CI check and workflow established before first schema)

**Research flag:** Standard patterns. No additional phase research needed. axum, tonic, webauthn-rs, and nginx auth_request are well-documented.

### Phase 2: Catalog Service and Browse API

**Rationale:** Catalog is the second dependency in the content pipeline. The scraper and file service cannot function without work IDs from catalog. Delivering catalog browsing and tag filtering here also produces the first working user-facing API.

**Delivers:** Book CRUD (create, update, get, list), tag storage and multi-tag compound queries, publish workflow with page count verification, paginated browse endpoint, basic text search via PostgreSQL full-text or trigram, gateway routes for all catalog operations.

**Addresses:** Catalog browsing (table stakes), tag-based filtering (table stakes), basic search (table stakes).

**Avoids:**
- N+1 queries in tag lookups (explicit JOINs with sea-orm, indexes on work_tags from day one)
- Image storage path design (establish internal-ID-based paths before any image is stored)

**Research flag:** Standard patterns. sea-orm tag-join queries and PostgreSQL full-text are well-documented.

### Phase 3: File Service and Image Serving

**Rationale:** File service and nginx image serving are a coupled unit — they must be built and tested together. Image serving is the core value proposition. The publish workflow (metadata + images + publish) completes the catalog integration.

**Delivers:** Image upload endpoint (axum), local filesystem storage with internal-ID-based paths, nginx static image serving with auth_request integration, complete scraper-facing upload pipeline (POST metadata, POST images, POST publish).

**Addresses:** Image upload and serving (table stakes), image quality preservation (no transcoding, serve originals).

**Avoids:**
- Image storage path design coupling to external source IDs (use internal work IDs as directory names)
- nginx path traversal on image location block (use `root` directive correctly with internal paths)

**Research flag:** Standard patterns. nginx static file serving and auth_request are well-documented.

### Phase 4: Scraper

**Rationale:** The scraper is a self-contained binary that depends on a functioning catalog and file service API. Building it last among the core pipeline means the API it targets is stable. The decreasing-frequency update check is implemented as a custom tokio::time::sleep loop, not a cron library.

**Delivers:** hitomi.la new work detection and ingestion, decreasing-frequency update check scheduler, image upload pipeline (detect → upload metadata → upload images → publish), hash-based change detection for metadata updates, canary checks, schema validation at ingestion, error-rate circuit breaker, rate limiting with jitter, API key authentication.

**Addresses:** Scraper ingestion (table stakes), work metadata update checking (table stakes).

**Avoids:**
- Scraper silent failure on external source structure changes (canary + validation + circuit breaker)
- Scraper rate limiting and IP blocking (global request rate limiter, exponential backoff, jitter)
- Scraper deduplication failure (external ID as primary dedup key, idempotent upload pipeline)

**Research flag:** Needs research-phase attention. The hitomi_la crate (0.1.4, 1,904 downloads) is low-traffic and may have gaps. Verify which endpoints it covers before committing to it as the sole hitomi.la interface. If insufficient, fall back to reqwest + scraper crate with CSS selectors.

### Phase 5: User Service and Personalization

**Rationale:** User features (tastes, history) are additive and require both auth and a populated catalog. They cannot be meaningfully developed or tested without real content in the catalog and real users authenticating. Delivering them as a dedicated phase keeps the content pipeline simple during initial validation.

**Delivers:** User taste storage (binary like/dislike per work), reading history with per-page progress tracking, gateway routes for all user operations, taste-based negative filtering integrated into catalog browse queries.

**Addresses:** User tastes (differentiator), reading history (differentiator), taste-based negative filtering (differentiator).

**Avoids:**
- N+1 queries in user history lookups (explicit queries, indexes on user_id + work_id)

**Research flag:** Standard patterns. Simple CRUD service on sea-orm with well-established patterns.

### Phase 6: Work Renewal and Cross-Service Integration

**Rationale:** Work renewal is the most architecturally complex feature and depends on all other services being operational. It is also a v1.x feature triggered by the first observed renewal event on hitomi.la, not a day-one requirement. Deferring it avoids premature complexity in the core pipeline.

**Delivers:** Canonical work ID graph (old_id → new_id relations in catalog), gateway-level 301/302 redirect for old IDs, DB-based sync queue with `SELECT ... FOR UPDATE SKIP LOCKED`, per-service status tracking for idempotent processing, cross-service user reference update via gRPC, scraper renewal detection and reporting, scraper health monitoring and alerting.

**Addresses:** Work renewal tracking (differentiator), scraper failure monitoring (operational necessity).

**Avoids:**
- Work ID renewal orphaning cross-service references (idempotency keys, per-service status, max-retry alerting)
- DB queue performance bottleneck (SKIP LOCKED, composite index on status+created_at, cleanup job)
- Synchronous cross-service calls in write paths (all renewal propagation via async DB queue)

**Research flag:** Needs research-phase attention. The distributed state management for work renewal across separate database schemas with no cross-schema foreign keys is the most novel aspect of this architecture. Review and design the queue schema and per-service status tracking structure carefully before implementation.

### Phase Ordering Rationale

- **Auth first:** JWT middleware is required before any route handler works. The nginx auth_request single-cookie contract must be established before the file service or image serving is built.
- **Catalog second:** Work IDs from catalog are foreign keys for file storage paths and scraper tracking state. The catalog API surface (metadata, tag schema, publish workflow) must be stable before the scraper targets it.
- **File third:** The file service and nginx image serving are tightly coupled and form the delivery side of the content pipeline. The scraper's upload pipeline targets both catalog and file service.
- **Scraper fourth:** The last piece of the content pipeline. By this phase, the full API surface is stable. The scraper is a client, not a server — it does not block any other service.
- **User fifth:** Additive features that depend on real content and authenticated users. No other service blocks on user service functionality.
- **Renewal sixth:** The most complex cross-service integration. Best designed and implemented after all services are independently operational, since it must orchestrate them all.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 4 (Scraper):** The hitomi_la crate has low download counts (1,904) and may not cover all needed endpoints (work listing, image URL construction, JS-obfuscated patterns). Verify crate coverage before implementation sprint. If insufficient, plan to use reqwest + scraper (HTML parser) directly.
- **Phase 6 (Renewal/Cross-Service Integration):** The DB queue design with per-service idempotency tracking is the most novel part of the system. Deeper design work (exact queue schema, saga-like status machine, stale-processing timeout) should be done before the implementation sprint.

Phases with standard patterns (skip additional research):
- **Phase 1 (Foundation/Auth):** axum, tonic, webauthn-rs, and nginx auth_request are thoroughly documented. Patterns are well-established in the Rust ecosystem.
- **Phase 2 (Catalog):** sea-orm CRUD with tag joins and PostgreSQL full-text search are standard patterns with abundant documentation and community examples.
- **Phase 3 (File/Images):** nginx static file serving with auth_request is a stable, well-documented nginx feature. File upload in axum is straightforward.
- **Phase 5 (User Service):** Simple CRUD service with sea-orm — the most straightforward service in the system.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All crate versions verified via crates.io API on 2026-03-21. Feature flags are MEDIUM — based on training data, not live docs. |
| Features | MEDIUM | Based on training data knowledge of nhentai, hitomi.la, e-hentai, Komga, Kavita. Web search unavailable to verify current platform states. Core feature categories are stable and unlikely to have shifted. |
| Architecture | HIGH | Well-established Rust async ecosystem patterns (gateway + gRPC backend services, shared proto crate, schema-per-service). The specific nginx auth_request pattern is verified against official nginx documentation. |
| Pitfalls | MEDIUM-HIGH | nginx auth_request cookie limitation: HIGH (official nginx docs). JWT race condition: MEDIUM (applied concurrent systems analysis). Renewal orphans: MEDIUM-HIGH (standard distributed systems). Scraper fragility: HIGH (universal scraper pattern). sea-orm drift: MEDIUM (docs-based). |

**Overall confidence:** HIGH

### Gaps to Address

- **hitomi_la crate coverage:** The crate (0.1.4, January 2026, 1,904 downloads) may not expose all needed hitomi.la functionality — specifically new-work listing, JS-obfuscated image URL construction, and rate-limit-aware request scheduling. Validate crate API during Phase 4 planning sprint. Budget for direct reqwest + scraper fallback implementation.

- **nginx auth_request_set forwarding with cookie refresh:** The single-cookie constraint is clear, but the exact nginx directive sequence for forwarding the refresh cookie from the auth subrequest response to the image response (including correct ordering of `auth_request_set`, `add_header`, and cache-control headers) should be smoke-tested against a real nginx instance early in Phase 1 to avoid surprises.

- **sea-orm version upgrade path:** sea-orm 1.1.19 is the correct choice now. When sea-orm 2.0 reaches stable release, the migration will involve sea-query 1.0 API changes. Plan for a dedicated upgrade sprint rather than upgrading mid-project.

- **webauthn-rs conditional UI / passkey autofill:** webauthn-rs 0.5.4 supports conditional UI (passkey autofill in login forms via `PublicKeyCredentialRequestOptions` with `mediation: "conditional"`), but the browser-side JavaScript integration for this pattern needs careful setup. Treat this as a known rough edge during Phase 1 implementation.

## Sources

### Primary (HIGH confidence)
- crates.io API (`/api/v1/crates/{name}`) — version verification for all Rust dependencies on 2026-03-21
- nginx auth_request module official documentation (`https://nginx.org/en/docs/http/ngx_http_auth_request_module.html`) — auth_request_set cookie limitation
- tonic GitHub issues (`https://github.com/hyperium/tonic/issues`) — gRPC stream lifecycle, message size limits, error visibility
- PROJECT.md — authoritative project specification, architecture decisions, feature scope

### Secondary (MEDIUM confidence)
- docs.rs documentation for sea-orm, tonic-build, axum, webauthn-rs — feature flags and API patterns
- Domain knowledge of hitomi.la, nhentai, e-hentai/exhentai, Komga, Kavita — feature comparison
- Cargo workspace reference documentation — feature unification behavior, resolver = "2"
- PostgreSQL documentation — SKIP LOCKED pattern for DB queues

### Tertiary (LOW confidence)
- hitomi_la crate coverage — crate has low download counts; actual endpoint coverage unverified beyond training data
- sea-orm CLI generation flags consistency — exact flags for consistent entity generation not verified against current CLI version

---
*Research completed: 2026-03-21*
*Ready for roadmap: yes*
