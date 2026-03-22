# Pitfalls Research

> **Terminology note:** This document uses "work" throughout. The project canonical term is **"book"**. Pitfalls 3 (cross-service orphaned references) and 6 (DB queue bottleneck) are no longer applicable -- the canonical_id denormalization approach eliminates the cross-service sync queue entirely. See PROJECT.md for current design.

**Domain:** Rust microservice content mirroring/aggregation (manga)
**Researched:** 2026-03-21
**Confidence:** MEDIUM-HIGH (nginx auth_request: HIGH via official docs; tonic: HIGH via GitHub issues; sea-orm migrations, scraper reliability, DB queue patterns: MEDIUM via training data + docs.rs; Cargo workspace: MEDIUM via training data)

## Critical Pitfalls

### Pitfall 1: nginx auth_request_set Cannot Forward Multiple Set-Cookie Headers

**What goes wrong:**
The `auth_request_set` directive captures subrequest response headers into nginx variables using `$upstream_http_*`. However, nginx variables are single-valued strings. When the auth subrequest returns multiple `Set-Cookie` headers (e.g., refreshed JWT cookie + a session cookie), `$upstream_http_set_cookie` only captures the **last** one. The other cookies silently vanish. This is especially dangerous because JWT refresh during image requests (the `auth_request` flow described in PROJECT.md) may need to set a new cookie while the client sees no cookie at all or only a partial one.

**Why it happens:**
The nginx `$upstream_http_*` variable mechanism collapses multi-valued headers into a single value (comma-joined for some headers, last-wins for `Set-Cookie`). `Set-Cookie` headers are not comma-combinable per HTTP spec (RFC 6265), so this is a fundamental mismatch. Developers test with a single cookie, it works, then add a second cookie months later and the first silently stops being forwarded.

**How to avoid:**
- Design the auth response to return **exactly one** `Set-Cookie` header. Since Madome uses only a single JWT cookie (HttpOnly), this is achievable if the gateway auth endpoint is disciplined.
- Never add a second cookie to the auth subrequest response without rethinking the forwarding mechanism.
- If multiple cookies become necessary, encode all token data into a single cookie, or switch to a Lua-based approach (`openresty` / `ngx_http_lua_module`) that can iterate response headers.
- Alternative: use `proxy_pass` with `proxy_hide_header` and `add_header` manipulation instead of `auth_request` for the image serving path.

**Warning signs:**
- Intermittent "logged out" states for users browsing images but not API endpoints
- Cookie-related bugs that only appear on image requests, not on gateway API requests
- Auth subrequest returning 2xx but client not receiving the expected Set-Cookie

**Phase to address:**
Phase 1 (Gateway + Auth infrastructure). Lock down the single-cookie contract in the auth subrequest handler before any nginx configuration.

**Confidence:** HIGH -- verified against official nginx documentation for auth_request module.

---

### Pitfall 2: JWT Grace Period Refresh Race Condition

**What goes wrong:**
The PROJECT.md describes a JWT flow where expired JWTs within a "grace period" trigger a new JWT without hitting the auth service, and a per-session JWT cache prevents duplicate generation. The race condition: two concurrent requests arrive with the same expired-but-within-grace JWT. Both threads check the cache, find no cached replacement, both generate new JWTs with slightly different `iat`/`exp` claims, and both set different cookies. The client ends up with whichever cookie arrived last, and subsequent requests may use a JWT that the cache does not recognize, causing unnecessary session lookups.

**Why it happens:**
Per-session JWT caching requires atomic check-and-set operations. A naive implementation (check cache -> miss -> generate -> store) has a TOCTOU window. This is exacerbated under the image-heavy browsing pattern where a single page load triggers dozens of concurrent image requests through `auth_request`, all carrying the same expired JWT.

**How to avoid:**
- Use a lock-per-session-id (not a global lock) when generating refresh JWTs. The first request acquires the lock and generates; concurrent requests for the same session wait and reuse.
- In Rust with tonic/axum: a `DashMap<SessionId, tokio::sync::Mutex<Option<CachedJwt>>>` provides per-session serialization without global contention.
- Set the JWT cache TTL to at least match the grace period duration so a refreshed JWT is always found by concurrent requests.
- Accept that occasional duplicate generation is not catastrophic -- both JWTs are valid -- but design the cache to converge quickly (first-write-wins).

**Warning signs:**
- Multiple `Set-Cookie` headers in rapid succession in browser dev tools during image-heavy page loads
- Auth service receiving session verification calls despite the JWT being within grace period
- Log entries showing JWT generation for the same session within milliseconds of each other

**Phase to address:**
Phase 1 (Auth service). This must be correct from the start because every subsequent feature depends on auth working reliably.

**Confidence:** MEDIUM -- based on standard concurrent systems analysis applied to the specific architecture described.

---

### Pitfall 3: Work ID Renewal Creates Orphaned References Across Services

**What goes wrong:**
When an external source reassigns a work to a new ID (the "renewal" scenario), the old work ID must be updated across catalog, user tastes, user histories, and potentially the scraper's own tracking state. With no foreign keys across service databases (PROJECT.md explicitly notes "no direct FK between separate service DBs"), a DB-based queue propagates these updates. If the queue processing fails partway -- e.g., catalog updated but user service update fails -- the system enters an inconsistent state where some services reference the old ID and others reference the new one. Users see broken likes, missing history entries, or duplicate works.

**Why it happens:**
Distributed transactions across separate databases are fundamentally hard. DB-based queues provide at-least-once delivery but not atomic cross-service consistency. Developers often implement the "happy path" queue processing without retry logic, dead-letter handling, or idempotency, treating the queue as a fire-and-forget mechanism.

**How to avoid:**
- Make all queue consumers **idempotent**: processing the same renewal event twice must produce the same result. Use the `(old_id, new_id)` pair as a natural idempotency key.
- Implement a **status tracker** per renewal event: record which services have been updated. On retry, skip already-completed services.
- Design the canonical ID redirect at the **gateway** level so that even if downstream services still have old IDs, API consumers see consistent behavior. The redirect acts as a consistency safety net.
- Queue entries must have: event_id, old_work_id, new_work_id, status per service (catalog: done, user: pending, etc.), retry_count, last_error, created_at.
- Set a maximum retry count and alert on exhaustion rather than silently dropping.

**Warning signs:**
- Users report works in their history/likes that "don't exist" or lead to 404
- Same work appearing twice in search results with different IDs
- Queue table growing with stuck "pending" entries
- Discrepancy between canonical ID in catalog vs. IDs stored in user service

**Phase to address:**
Phase 3 (Scraper + Renewal logic). Must be designed before implementing the renewal flow, not bolted on after.

**Confidence:** MEDIUM-HIGH -- standard distributed systems concern directly applicable to the described architecture.

---

### Pitfall 4: Scraper Breaks Silently When External Source Changes HTML/API Structure

**What goes wrong:**
The scraper mirrors from hitomi.la. External sources change their DOM structure, API endpoints, anti-scraping measures, or data formats without notice. If the scraper does not detect structural changes, it either: (a) silently produces corrupted/incomplete data (wrong tags, missing images, garbled titles), or (b) crashes and stops mirroring entirely with no alert. Scenario (a) is worse because corrupted data enters the catalog and must be cleaned up.

**Why it happens:**
Scrapers are written against a point-in-time snapshot of the external source's structure. Developers test against current pages, everything works, and then months later the source changes a CSS class name or restructures JSON. The `hitomi_la` crate abstracts some of this but is itself subject to the same breakage if the upstream site changes.

**How to avoid:**
- **Validation at ingestion**: every scraped work must pass a schema validation before being submitted to the catalog. Required fields (title, at least one tag, page count > 0, all image URLs resolving) must be verified.
- **Canary checks**: before a full scraping run, fetch a known work (one that has been previously verified) and compare against stored data. If the canary fails, halt the scraping run and alert.
- **Structured error budgets**: track the ratio of failed-to-successful scrapes over a rolling window. If failure rate exceeds 10%, halt and alert.
- **Pin the `hitomi_la` crate version** and review upstream changes before upgrading. Treat it as an external dependency that can break your data pipeline.
- **Separate "scrape" from "ingest"**: scrape into a staging area, validate, then submit to catalog. Never scrape directly into production data.

**Warning signs:**
- Sudden spike in scraper errors
- New works being created with empty or nonsensical tags
- Page count mismatch between metadata and actual images (the publish check in PROJECT.md catches this, which is good)
- The `hitomi_la` crate releasing a new version (check why -- likely a breaking upstream change)

**Phase to address:**
Phase 3 (Scraper implementation). Build validation and canary infrastructure before scaling up scraping volume.

**Confidence:** HIGH -- this is a universal scraper reliability concern with well-established mitigation patterns.

---

### Pitfall 5: gRPC Service Definitions Leak Internal Domain Boundaries

**What goes wrong:**
With 6 services communicating over gRPC, there is strong temptation to define proto messages that mirror internal database schemas. This couples the proto contract to database structure. When one service changes its schema, the proto must change, which forces rebuilds and redeployments of all consumers. Over time, proto files become a tangled web of cross-service type dependencies where modifying a message field requires coordinating releases across multiple binaries.

**Why it happens:**
Code generation from proto files feels efficient -- define once, use everywhere. Developers reuse the same `Work` message type in catalog, user, and gateway protos instead of defining service-specific views. The Cargo workspace monorepo makes this even easier (just add a dependency on the shared proto crate), masking the coupling until it becomes painful.

**How to avoid:**
- **Each service owns its proto definitions.** The catalog service defines what a `Work` looks like in its API. The user service defines its own `WorkReference` that contains only what it needs (likely just the work ID).
- **Never share entity models across proto boundaries.** If two services need a `Work`, they each define their own message with only the fields they expose/consume.
- **Gateway translates between proto formats.** The gateway maps between the external REST API shape and each service's gRPC shape. This is its job.
- **Use a shared `common.proto` only for truly universal types**: pagination, timestamps, error codes. Not domain entities.
- Organize protos as: `proto/catalog/v1/catalog.proto`, `proto/auth/v1/auth.proto`, etc. Each service compiles only its own protos.

**Warning signs:**
- A proto change in one service requiring proto regeneration in unrelated services
- A "common" or "shared" proto file that grows beyond 50 lines
- Gateway service importing every other service's internal message types
- Circular proto dependencies

**Phase to address:**
Phase 1 (proto definition and Cargo workspace setup). Getting proto boundaries right at the start avoids expensive refactoring later.

**Confidence:** MEDIUM-HIGH -- established microservice boundary pattern applied to gRPC + Cargo workspace context.

---

### Pitfall 6: DB-Based Queue Becomes a Performance Bottleneck with Polling

**What goes wrong:**
Using PostgreSQL as a message queue (for work ID renewal propagation and cross-service sync) introduces polling overhead. Each consumer polls the queue table on an interval. With multiple consumers and no proper indexing or row-locking strategy, this leads to: (a) excessive database load from frequent `SELECT ... WHERE status = 'pending'` queries, (b) duplicate processing when two consumers grab the same row, (c) table bloat from completed entries that are never cleaned up, (d) long-running transactions holding locks during processing.

**Why it happens:**
DB-based queues seem simple -- just another table. But databases are not optimized for the queue access pattern (write-once, read-once, delete). Without `SKIP LOCKED` (available in PostgreSQL 9.5+), row-level contention is handled with either advisory locks (complex) or optimistic locking (retry storms). Developers also forget that the queue table needs regular `VACUUM` since every row is eventually deleted.

**How to avoid:**
- Use `SELECT ... FOR UPDATE SKIP LOCKED LIMIT N` for dequeuing. This is the critical PostgreSQL feature that makes DB queues viable. It prevents duplicate processing without blocking.
- Add a composite index on `(status, created_at)` for efficient polling queries.
- Implement a cleanup job: move completed entries to an archive table or delete entries older than N days. The queue table should stay small.
- Process queue entries in a **separate transaction** from the dequeue: dequeue (set status = 'processing'), commit, do work, then update status to 'completed' or 'failed'. Never hold a transaction open during the actual processing.
- Keep queue entry payloads small (just IDs and event type). Fetch full data from the source service when processing.
- For Madome's scale (small community), polling every 5-10 seconds is fine. Do not over-optimize.

**Warning signs:**
- Queue table growing to thousands of rows
- Same queue entry being processed multiple times (check via logging)
- Database CPU spikes correlating with queue poll intervals
- "idle in transaction" connections from queue consumers

**Phase to address:**
Phase 3 (Renewal/queue infrastructure). Design the queue schema with `SKIP LOCKED` from day one.

**Confidence:** MEDIUM -- standard PostgreSQL queue patterns, well-documented in PostgreSQL community.

---

### Pitfall 7: sea-orm Migration Drift Between Entity Models and Database Schema

**What goes wrong:**
sea-orm has two sources of truth: the migration files (which define the actual database schema) and the entity/model files (which sea-orm uses at runtime for query generation). These can drift apart. A developer adds a column in a migration but forgets to update the entity model (or vice versa). The compiler does not catch this -- it compiles fine. At runtime, queries fail with cryptic "column not found" or "type mismatch" database errors. In a microservice setup with separate databases per service, this drift can happen independently in each service.

**Why it happens:**
sea-orm's code-generation (`sea-orm-cli generate entity`) can regenerate entity files from the database, but this requires a running database with the latest migrations applied. In a Cargo workspace with multiple services each having their own migrations and entities, developers often manually edit entity files and forget to create the corresponding migration, or create the migration and forget to regenerate entities.

**How to avoid:**
- **Establish a strict workflow**: (1) write migration, (2) apply migration to dev database, (3) regenerate entities with `sea-orm-cli generate entity`, (4) review generated diff, (5) commit both migration and entities together.
- Add a CI check that regenerates entities from a freshly-migrated database and diffs against committed entities. Any difference is a CI failure.
- Each service should have its own `migration` crate within the Cargo workspace: `crates/catalog-migration/`, `crates/auth-migration/`, etc. Never share migration crates across services.
- Use `sea-orm-cli` with `--with-serde both` and `--date-time-crate time` (or chrono) flags consistently. Inconsistent generation flags between runs cause spurious diffs.

**Warning signs:**
- Runtime database errors on queries that compiled successfully
- Entity files with commented-out or manually added fields
- Migration crate and entity crate versions out of sync
- Developers saying "just regenerate the entities" as a fix for bugs

**Phase to address:**
Phase 1 (Database schema setup). Establish the migration workflow before any schema exists.

**Confidence:** MEDIUM -- based on sea-orm documentation and common ORM migration patterns.

---

### Pitfall 8: Cargo Workspace Feature Unification Causes Unexpected Dependency Bloat

**What goes wrong:**
Cargo workspaces unify features across the dependency graph. If the `gateway` binary enables `tokio/full` and the `scraper` binary only needs `tokio/rt`, the workspace resolver may unify these so that all crates compile with `tokio/full`. More dangerously, if one service enables `sea-orm/with-json` and another does not, the unified feature set means all services get JSON support for sea-orm, increasing compile times and binary sizes. Worse: a feature flag in one service can enable code paths or dependencies (like TLS backends) in another service that conflict or cause compilation errors.

**Why it happens:**
Cargo's dependency resolution with workspaces (resolver "2" helps but does not eliminate the issue for non-dev dependencies) shares the dependency graph. Developers add features to one binary's `Cargo.toml` without considering the workspace-wide impact. In a 6-binary workspace like Madome, this compounds quickly.

**How to avoid:**
- Set `resolver = "2"` in the workspace `Cargo.toml` (required for edition 2021+ anyway, but verify it is explicit).
- Create a shared `common` crate for truly shared code, but keep its feature set minimal. Services depend on `common` and enable only the features they need.
- Do not use `default-features = true` for major dependencies. Explicitly list required features per crate.
- Periodically audit binary sizes: `cargo bloat --release --bin gateway` can reveal unexpected dependencies.
- Keep service-specific dependencies in the service crate, not in shared crates. If only `auth` needs `webauthn-rs`, it should be a dependency of `crates/auth/` only.

**Warning signs:**
- Compile time increasing disproportionately to code changes
- Binary sizes much larger than expected (> 20-30 MB for a service)
- Compilation errors in one service caused by dependency changes in another
- `cargo tree` showing unexpected features enabled for a crate

**Phase to address:**
Phase 1 (Workspace setup). Define the workspace structure, resolver, and dependency policy before adding the first service binary.

**Confidence:** MEDIUM -- well-known Cargo workspace behavior, documented in Cargo reference.

---

### Pitfall 9: Image Storage Path Design Prevents Future Migration

**What goes wrong:**
Image files are stored on the local filesystem (PROJECT.md: file service serves from local filesystem, nginx serves directly after auth). If the path structure tightly couples to the external source's work ID (e.g., `/images/hitomi/12345/001.jpg`), then work ID renewals require either: (a) moving/copying files (expensive with thousands of images), or (b) maintaining a path-mapping layer. PROJECT.md already says "image files are not moved" during renewal, but the path design must support this. A bad path design forces one of these costly options.

**Why it happens:**
The simplest path design mirrors the source structure. It works until the first renewal, when the old ID directory still contains the images but the canonical ID has changed. Developers then add symlinks, redirect rules, or path rewriting -- all of which are fragile and accumulate over time.

**How to avoid:**
- **Use a content-addressable or internal-ID path scheme**: `/images/{internal_work_id}/{page_number}.{ext}` where `internal_work_id` is Madome's own immutable identifier, not the external source's ID.
- Alternatively, if using external IDs: use the **original** external ID (the one assigned at first import) as the directory name and never change it, even after renewal. The canonical ID mapping is handled at the API/database layer.
- Store the path-to-work mapping in the catalog database. The file service resolves work IDs to filesystem paths via catalog lookup (or a local cache of that mapping).
- PROJECT.md's "image files are not moved" decision is correct. Encode this invariant into the path design so it is structurally impossible to need a move.

**Warning signs:**
- Code that constructs file paths by concatenating external work IDs
- Symlinks appearing in the image storage directory
- nginx rewrite rules that map work IDs to different filesystem paths
- Discussion about "migrating" image directories

**Phase to address:**
Phase 2 (File service + image storage). Must be decided before the first image is stored.

**Confidence:** HIGH -- direct analysis of the renewal scenario described in PROJECT.md.

---

### Pitfall 10: Scraper Rate Limiting and Deduplication Failures

**What goes wrong:**
The scraper checks for new works and updates on a decreasing frequency schedule. Two failure modes: (1) **Rate limiting / IP blocking**: the external source detects automated access and blocks the scraper's IP, causing a complete mirroring halt. (2) **Deduplication failure**: the scraper re-scrapes and re-uploads a work that already exists in the catalog because it fails to check for duplicates (e.g., the work hash check fails or is not implemented correctly). This wastes bandwidth, storage, and creates duplicate entries.

**Why it happens:**
Rate limiting: developers test scrapers with gentle timing, but the decreasing-frequency schedule means recent works get checked every 5 minutes. If the scraper is checking hundreds of recent works, the aggregate request rate can be high. IP blocking can happen suddenly with no warning. Deduplication: the PROJECT.md mentions a "hash of work info + tags" for detecting changes, but hash collisions, changed metadata on unchanged works, or race conditions between the check and upload steps can cause false negatives.

**How to avoid:**
- **Rate limiting**: implement a global request rate limiter in the scraper (e.g., max 1 request per 2 seconds to the external source). Use exponential backoff on HTTP 429 or connection refused. Add jitter to polling intervals.
- **Respect the source**: check for and honor `robots.txt`, `Retry-After` headers, and rate limit headers. Even if the source does not explicitly provide these, be conservative.
- **Deduplication**: use the external work ID as the primary dedup key (not just the hash). Before uploading, query the catalog: "does work with external ID X exist?" If yes, compare hashes to decide if an update is needed.
- **Idempotent uploads**: the publish workflow (upload metadata -> upload images -> publish) should be resumable. If the scraper crashes after uploading metadata but before uploading images, it should detect the unpublished work on restart and resume rather than creating a second entry.
- Store the scraper's "last checked" watermark persistently. If the scraper restarts, it should resume from where it left off, not re-scan everything.

**Warning signs:**
- Sudden increase in HTTP errors from the external source
- Duplicate works in the catalog with identical external IDs
- Scraper logs showing successful uploads for works that already exist
- Large gaps in mirrored work IDs (indicating missed works during an outage)

**Phase to address:**
Phase 3 (Scraper implementation). Rate limiting infrastructure must be in place before any automated scraping begins.

**Confidence:** HIGH -- universal scraper reliability concerns.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Hardcoding external source URLs in scraper | Fast implementation | Cannot add new sources or adapt to URL changes without code changes | MVP only, extract to config before release |
| Single shared proto crate for all services | Less boilerplate | Tight coupling, rebuild-all-on-any-change | Never -- set up per-service protos from start |
| Skipping the "unpublished" state for works | Faster ingestion pipeline | No atomic publish; partial works visible to users | Never -- the publish workflow is a correctness requirement |
| Polling queue with `SELECT * WHERE status='pending'` (no `SKIP LOCKED`) | Simpler SQL | Duplicate processing, lock contention under load | Acceptable only if single consumer is guaranteed |
| Storing JWT secret in environment variable | Easy deployment | Secret rotation requires redeployment of all services | MVP only, move to a secrets manager or config reload mechanism |
| Manual entity model updates instead of `sea-orm-cli generate` | Faster iteration | Schema drift, runtime errors | Never -- always regenerate and review |
| Using `String` for all IDs in proto messages | Avoids type conversion | No type safety, easy to pass wrong ID type | Never -- use typed wrappers or distinct message types |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| nginx auth_request + File service | Returning response body from auth subrequest (nginx ignores it, but it wastes bandwidth) | Auth endpoint returns only status code + headers, with `proxy_pass_request_body off` and empty `Content-Length` |
| tonic gRPC between services | Using `tonic::transport::Channel` without connection timeout/keep-alive | Configure `Channel::builder(uri).connect_timeout(Duration::from_secs(5)).keep_alive_timeout(Duration::from_secs(20))` |
| tonic gRPC streaming | Not handling `RST_STREAM` / stream drops (tonic may not surface these as errors) | Treat stream termination as an expected event; implement reconnect logic for long-lived streams |
| sea-orm with PostgreSQL | Using `sea_orm::Database::connect()` without configuring connection pool size | Use `ConnectOptions` with `max_connections`, `min_connections`, `connect_timeout`, `idle_timeout` explicitly set |
| webauthn-rs Passkey | Storing the challenge in a global cache without session binding | Challenge must be bound to the specific session/user initiating registration/authentication; use a session-keyed cache |
| `hitomi_la` crate | Assuming the crate handles all error cases and rate limiting | The crate likely provides parsing only; wrap every call with timeout, retry, and error handling at the scraper level |
| Scraper -> Gateway API Key auth | Treating the API key like a user JWT (routing through the full auth flow) | API key auth should be a separate, simpler code path in the gateway -- skip JWT/session entirely for service-to-service calls |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| N+1 queries in catalog tag lookups | Slow tag-based work queries, high DB query count | Use sea-orm's `find_with_related()` or explicit JOINs for tag queries; never loop-query | > 100 works with multiple tags |
| Unbounded image page responses | Memory spikes when a work has 200+ pages and all URLs are returned in one response | Paginate image lists or return only metadata (let client request images individually via nginx) | Works with > 100 pages |
| Gateway as gRPC bottleneck | Gateway CPU pegged, services idle | Keep gateway logic minimal (JWT check + forward). Do not deserialize/reserialize gRPC payloads in the gateway | > 50 concurrent users |
| Full table scan on queue polling | Queue processing latency increases with table size | Index on `(status, created_at)`, regular cleanup of completed entries | Queue table > 10K rows |
| Synchronous image validation in publish | Publish endpoint times out for works with many pages | Validate image count asynchronously or via background check before publish is called | Works with > 50 pages |
| Large gRPC message sizes | Memory allocation spikes, potential OOM | Set `max_decoding_message_size` and `max_encoding_message_size` on tonic clients/servers; default decode limit is 4MB which may be too small for bulk work listings but too large for most requests | Bulk listing > 100 works |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| JWT secret shared between gateway and auth as a static string | Secret compromise exposes all tokens; no rotation path | Auth service is the sole JWT issuer; gateway verifies with the public key (use asymmetric RS256/ES256, not symmetric HS256) |
| API key for scraper stored in scraper binary or config without rotation | Compromised scraper gives full catalog write access | Implement API key rotation; store key hash in auth DB; scraper reads key from environment; gateway validates against hash |
| nginx serving image files without path traversal protection | Attacker crafts a request like `/images/../../etc/passwd` | Configure nginx `root` (not `alias`) correctly; validate that resolved path is within the image directory; use `internal` directive on image location blocks |
| auth_request endpoint returning 200 for unauthenticated OPTIONS requests | CORS preflight bypasses auth, potentially exposing image URLs | Handle OPTIONS separately in nginx (before auth_request) or ensure auth endpoint correctly processes OPTIONS |
| Passkey registration without verifying attestation | Allows registration of fake/emulated authenticators | Use `webauthn-rs` attestation verification; for small community this is less critical but still implement `None` attestation at minimum |
| No request rate limiting on gateway REST endpoints | DoS via rapid API calls; scraper endpoint abuse | Implement rate limiting per IP at nginx level (`limit_req_zone`) and per API key in the gateway |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Work disappears during renewal (old ID 404, new ID not yet published) | User sees "not found" for a work they were just browsing | Gateway returns 301/302 redirect from old canonical ID to new canonical ID immediately when renewal is detected, even before full processing completes |
| Search returns duplicates during renewal processing | Confusing duplicate results | Catalog marks old work as "redirected" (exclude from search) atomically with creating the renewal link |
| Image loading fails silently when auth cookie is expired and refresh fails | Broken images with no explanation | File service / nginx returns a meaningful error (401) that the frontend can handle; consider a fallback "login required" placeholder image |
| Tag queries return no results for valid tags due to case sensitivity | User searches "Artist:name" vs "artist:name" and gets different results | Normalize tags to lowercase at ingestion time; search with case-insensitive comparison |

## "Looks Done But Isn't" Checklist

- [ ] **JWT refresh via auth_request:** Test with concurrent image requests (10+ parallel) on an expired JWT -- verify all images load and exactly one new cookie is set, not zero or multiple
- [ ] **Work publish flow:** Test with mismatched page count (metadata says 20, only 15 images uploaded) -- verify publish is rejected, not silently accepted
- [ ] **Work ID renewal:** Test the full chain: old work has user likes/history, renewal occurs, verify old ID redirects, likes/history are preserved under new canonical ID
- [ ] **Scraper resume:** Kill the scraper mid-upload (after metadata, before images) -- verify it resumes correctly on restart without creating duplicate entries
- [ ] **DB queue processing:** Test queue consumer crash mid-processing -- verify the entry is retried, not stuck in "processing" forever (implement a stale-processing-timeout)
- [ ] **gRPC error propagation:** Verify that a sea-orm database error in the catalog service propagates as a meaningful gRPC status code to the gateway, and then as a meaningful HTTP status code to the client -- not a generic 500
- [ ] **nginx path traversal:** Test with URL-encoded path traversal attempts (`%2e%2e%2f`) on the image serving endpoint
- [ ] **Passkey registration + authentication:** Test with multiple authenticators per user, cross-device scenarios, and authenticator deletion
- [ ] **Decreasing check frequency:** Verify the scheduler correctly transitions works through frequency tiers and does not "forget" works (e.g., after a scraper restart, the schedule should be reconstructed from the database, not from in-memory state)

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Schema drift (entities vs migrations) | LOW | Regenerate entities from migrated DB, diff, commit. No data loss. |
| Duplicate works from scraper dedup failure | MEDIUM | Write a one-off dedup script: find works with same external ID, merge user references to the canonical one, delete duplicates, add missing dedup check |
| Orphaned references from failed renewal queue | MEDIUM | Query each service for references to old work IDs that should have been updated; replay failed queue entries; add monitoring to prevent recurrence |
| nginx auth_request cookie not forwarding | LOW | Fix nginx config, redeploy. No data corruption. Users just need to re-login. |
| External source blocks scraper IP | MEDIUM-HIGH | Rotate IP (if possible), reduce request rate, add delays. May require significant scraper architecture changes if persistent. Consider proxy rotation. |
| Proto coupling forces rebuild-all | HIGH | Refactor proto definitions into per-service packages. Requires coordinated change across all services. Best prevented, not recovered from. |
| Corrupted data from scraper parsing failure | HIGH | Identify affected works (by import date range), re-scrape from source, update catalog entries. May require manual review if source data has also changed. |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| nginx auth_request_set cookie limitation | Phase 1: Gateway + nginx config | Integration test: auth subrequest returns cookie, client receives it on image request |
| JWT refresh race condition | Phase 1: Auth service | Load test: 50 concurrent requests with expired JWT, verify single refresh |
| Proto boundary coupling | Phase 1: Workspace + proto setup | Review: no service imports another service's proto messages (only its own) |
| Cargo feature unification | Phase 1: Workspace setup | CI check: `cargo tree --edges features` shows no unexpected feature propagation |
| sea-orm migration drift | Phase 1: DB schema setup | CI check: regenerated entities match committed entities |
| Image storage path design | Phase 2: File service | Review: file paths use internal IDs, not external source IDs |
| DB queue without SKIP LOCKED | Phase 3: Queue infrastructure | Code review: dequeue query uses `FOR UPDATE SKIP LOCKED` |
| Work ID renewal orphaned refs | Phase 3: Renewal flow | Integration test: full renewal cycle, verify all services updated |
| Scraper silent failure | Phase 3: Scraper implementation | Monitoring: alert on scrape failure rate > 10% |
| Scraper rate limiting / IP block | Phase 3: Scraper implementation | Config review: rate limiter present, backoff implemented |

## Sources

- nginx auth_request module official documentation: https://nginx.org/en/docs/http/ngx_http_auth_request_module.html (HIGH confidence)
- tonic GitHub issues (recurring themes around stream lifecycle, error visibility, configuration): https://github.com/hyperium/tonic/issues (HIGH confidence)
- tonic documentation on message size limits and compression: https://docs.rs/tonic/latest/tonic/ (HIGH confidence)
- sea-orm migration crate documentation: https://docs.rs/sea-orm-migration/latest/sea_orm_migration/ (MEDIUM confidence -- documentation coverage is 21%)
- PostgreSQL `SKIP LOCKED` for queue patterns: standard PostgreSQL 9.5+ feature (MEDIUM confidence -- from training data)
- Cargo workspace resolver behavior: Cargo reference documentation (MEDIUM confidence -- from training data)
- JWT concurrent refresh patterns: standard distributed systems analysis (MEDIUM confidence -- applied reasoning)

---
*Pitfalls research for: Rust microservice content mirroring (Madome)*
*Researched: 2026-03-21*
