# Roadmap: Madome

## Overview

Madome delivers a Rust microservice backend that mirrors manga metadata from external sources and lets authenticated users browse, query, and interact with the catalog. The build order follows hard dependencies across 6 phases: gateway infrastructure first (everything routes through it), then user profile (provides the users table that authentication depends on), then authentication (gates all access), then catalog write operations (books must exist), then catalog read operations (users discover books), and finally user preference features (require both auth and content). Each phase delivers an independently verifiable capability through the gateway API.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Foundation and Gateway Infrastructure** - Cargo workspace, shared crates, proto definitions, gateway REST-to-gRPC routing (completed 2026-03-21)
- [ ] **Phase 2: User Profile** - User service with profiles, CRUD operations via gRPC
- [ ] **Phase 3: Authentication** - Passkey registration/login, JWT lifecycle, session management, API key auth
- [ ] **Phase 4: Catalog Core** - Book CRUD operations and publish workflow through gateway
- [ ] **Phase 5: Catalog Queries** - Tag-based filtering, ID list lookup, paginated browse listing
- [ ] **Phase 6: User Preferences** - Taste (like/dislike) and reading history tracking per user

## Phase Details

### Phase 1: Foundation and Gateway Infrastructure
**Goal**: A running gateway that accepts REST requests and routes them to internal gRPC services
**Depends on**: Nothing (first phase)
**Requirements**: GATE-01, GATE-02
**Success Criteria** (what must be TRUE):
  1. Cargo workspace compiles with gateway binary, proto crate, and shared library crates
  2. Gateway starts and exposes REST endpoints that accept HTTP requests
  3. Gateway translates a REST request into a gRPC call to an internal service and returns the response
  4. Proto definitions exist for all services (auth, catalog, user) even if service implementations are stubs
**Plans**: 2 plans

Plans:
- [x] 01-01-PLAN.md -- Workspace manifest, proto definitions, shared crates (madome-proto, madome-core, madome-common)
- [ ] 01-02-PLAN.md -- Service stubs, gateway REST-to-gRPC routing, integration tests

### Phase 2: User Profile
**Goal**: User service manages user records with CRUD operations accessible via gRPC, with gateway REST endpoints for self-service and admin operations
**Depends on**: Phase 1
**Requirements**: USER-PROFILE-01, USER-PROFILE-02
**Success Criteria** (what must be TRUE):
  1. User service connects to its own PostgreSQL database with a users table
  2. User can be created via gRPC RPC with name, role, and active status
  3. User can be retrieved, listed, updated, deactivated, and activated via gRPC RPCs
  4. User service follows the 4-layer architecture pattern (domain/usecase/app/adapter)
**Plans**: 4 plans

Plans:
- [x] 02-01-PLAN.md -- Proto definitions, workspace deps, docker-compose, domain layer, sea-orm entity/migration
- [x] 02-02-PLAN.md -- PostgreSQL adapter layer (PostgresUserRepository, UserContext)
- [x] 02-03-PLAN.md -- Usecase functions with unit tests, gRPC handler, main.rs composition root
- [ ] 02-04-PLAN.md -- Gateway REST routes for user operations, integration tests, service tests

### Phase 3: Authentication
**Goal**: Users can register a passkey, authenticate, and access protected endpoints through JWT-verified gateway
**Depends on**: Phase 1, Phase 2
**Requirements**: AUTH-01, AUTH-02, AUTH-03, AUTH-04, AUTH-05, GATE-03, GATE-04, GATE-05
**Success Criteria** (what must be TRUE):
  1. User can register a new passkey credential via the gateway API
  2. User can authenticate with a registered passkey and receive a JWT access token as an HttpOnly cookie
  3. Authenticated requests pass through the gateway without contacting the auth service (stateless JWT verification)
  4. Expired JWT is automatically refreshed (grace period pass-through or session-based reissue) without user action
  5. Scraper can authenticate to the gateway using an API key
**Plans**: 4 plans

Plans:
- [ ] 03-01-PLAN.md -- Proto definitions, workspace dependencies, infrastructure (Docker-compose), DB schema and migrations
- [ ] 03-02-PLAN.md -- Auth service domain modules (WebAuthn, session, JWT, invite, recovery, API key, repositories) and gRPC service
- [ ] 03-03-PLAN.md -- Gateway JWT middleware, 4-tier route guards, API key auth, auth REST route handlers
- [ ] 03-04-PLAN.md -- Auth contract tests and gateway E2E integration tests

### Phase 4: Catalog Core
**Goal**: Books can be created, updated, deleted, and published through the gateway API
**Depends on**: Phase 1
**Requirements**: CATL-01, CATL-02, CATL-03, CATL-04
**Success Criteria** (what must be TRUE):
  1. A book can be created with metadata in unpublished state (including info+tags hash)
  2. A book's metadata can be updated after creation
  3. A book can be deleted
  4. A book can be published only when page count verification succeeds
  5. An unpublished book is not visible in query results; a published book is visible
**Plans**: TBD

Plans:
- [ ] 04-01: TBD
- [ ] 04-02: TBD

### Phase 5: Catalog Queries
**Goal**: Users can discover and browse books through tag filters, ID lookups, and paginated listings
**Depends on**: Phase 4
**Requirements**: CATL-05, CATL-06, CATL-07, CATL-08
**Success Criteria** (what must be TRUE):
  1. User can query published books by a single tag and get matching results
  2. User can query published books by multiple tags (AND logic) and get only books matching all tags
  3. User can fetch a specific set of books by providing a list of IDs
  4. User can browse a paginated list of books sorted by recency (newest first)
**Plans**: TBD

Plans:
- [ ] 05-01: TBD
- [ ] 05-02: TBD

### Phase 6: User Preferences
**Goal**: Authenticated users can express tastes on books and track their reading progress
**Depends on**: Phase 3, Phase 4
**Requirements**: USER-01, USER-02, USER-03, USER-04, USER-05, USER-06
**Success Criteria** (what must be TRUE):
  1. Authenticated user can set a like or dislike on a book
  2. Authenticated user can remove a previously set taste from a book
  3. Authenticated user can retrieve their full list of liked or disliked books
  4. Authenticated user's reading progress (last-read page) is recorded when they read a book
  5. Authenticated user can view their reading history across all books
**Plans**: TBD

Plans:
- [ ] 06-01: TBD
- [ ] 06-02: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6

Note: Phase 4 (Catalog Core) can start in parallel with Phase 3 (Authentication) since both depend on Phase 1 (not Phase 2).

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation and Gateway Infrastructure | 2/2 | Complete   | 2026-03-21 |
| 2. User Profile | 3/4 | In Progress|  |
| 3. Authentication | 0/4 | Not started | - |
| 4. Catalog Core | 0/? | Not started | - |
| 5. Catalog Queries | 0/? | Not started | - |
| 6. User Preferences | 0/? | Not started | - |
