# Requirements: Madome

**Defined:** 2026-03-21
**Core Value:** Reliably mirror books from external sources and allow authenticated users to browse them.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Gateway

- [ ] **GATE-01**: Gateway exposes REST API with route registration for all services
- [ ] **GATE-02**: Gateway routes REST requests to internal services via gRPC
- [ ] **GATE-03**: Gateway verifies JWT access token (stateless pass-through when valid)
- [ ] **GATE-04**: Gateway refreshes JWT on expiry (grace period + session fallback)
- [ ] **GATE-05**: Gateway authenticates Scraper via API Key

### Auth

- [ ] **AUTH-01**: User can register a Passkey credential
- [ ] **AUTH-02**: User can authenticate via Passkey
- [ ] **AUTH-03**: Auth service issues JWT access token (15 min TTL, HttpOnly Cookie)
- [ ] **AUTH-04**: Auth service manages sessions (create, validate, invalidate)
- [ ] **AUTH-05**: Auth service caches recently issued JWT per session (duplicate prevention)

### Catalog

- [ ] **CATL-01**: Create book with metadata (unpublished state, info+tags hash)
- [ ] **CATL-02**: Update book metadata
- [ ] **CATL-03**: Delete book
- [ ] **CATL-04**: Publish book with page count verification
- [ ] **CATL-05**: Query books by single tag
- [ ] **CATL-06**: Query books by multiple tags (AND)
- [ ] **CATL-07**: Query books by ID list
- [ ] **CATL-08**: Paginated book listing (sorted by recency)

### User

- [ ] **USER-01**: Set taste for a book (like/dislike)
- [ ] **USER-02**: Remove taste for a book
- [ ] **USER-03**: List user's liked/disliked books
- [ ] **USER-04**: Record reading history with last-read page
- [ ] **USER-05**: Get reading history for a book
- [ ] **USER-06**: List user's reading history

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Content Pipeline

- **FILE-01**: Image upload via File service (Scraper -> File service)
- **FILE-02**: nginx auth_request-based authenticated image serving
- **FILE-03**: nginx auth_request_set cookie refresh forwarding

### Scraper

- **SCRP-01**: Detect new books on hitomi.la and mirror metadata + images
- **SCRP-02**: Book info update checking with decreasing frequency
- **SCRP-03**: Scraper health monitoring and failure alerting

### Renewal

- **RENW-01**: Book ID renewal handling (graph relation, canonical ID)
- **RENW-02**: DB-based queue for cross-service reference updates
- **RENW-03**: Renewal history display via API

### Search

- **SRCH-01**: Full-text search over book titles and tag values (PostgreSQL)

### Personalization

- **PRSL-01**: Taste-based negative filtering (exclude disliked books/tags from browse)

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Frontend UI | API-first architecture; frontend is a separate milestone |
| Automated image censorship | ML infrastructure overhead; only relevant if service goes public |
| Reporting/moderation | Small trusted community does not need formal moderation initially |
| Message broker (RabbitMQ, etc.) | DB-based queue sufficient for v1 scale; migration path available |
| OAuth/social login | Passkey-only provides stronger security; small community acceptance |
| Mobile app | Web API first, mobile later |
| User-uploaded content | Moderation nightmare; contradicts reliable mirror value proposition |
| Comments/forums/social | Community uses external platform (Discord); avoid moderation burden |
| Real-time notifications | Overkill for periodic content updates; polling-based indicator sufficient |
| Multi-source aggregation | Each source has different schema/ID; premature generalization kills velocity |
| ML recommendations | Cold-start problem; insufficient data initially |
| Batch download/export | Bandwidth spike risk; enables redistribution |
| Reader mode customization | Frontend/client concern; API exposes page dimensions |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| GATE-01 | Phase 1 | Pending |
| GATE-02 | Phase 1 | Pending |
| GATE-03 | Phase 2 | Pending |
| GATE-04 | Phase 2 | Pending |
| GATE-05 | Phase 2 | Pending |
| AUTH-01 | Phase 2 | Pending |
| AUTH-02 | Phase 2 | Pending |
| AUTH-03 | Phase 2 | Pending |
| AUTH-04 | Phase 2 | Pending |
| AUTH-05 | Phase 2 | Pending |
| CATL-01 | Phase 3 | Pending |
| CATL-02 | Phase 3 | Pending |
| CATL-03 | Phase 3 | Pending |
| CATL-04 | Phase 3 | Pending |
| CATL-05 | Phase 4 | Pending |
| CATL-06 | Phase 4 | Pending |
| CATL-07 | Phase 4 | Pending |
| CATL-08 | Phase 4 | Pending |
| USER-01 | Phase 5 | Pending |
| USER-02 | Phase 5 | Pending |
| USER-03 | Phase 5 | Pending |
| USER-04 | Phase 5 | Pending |
| USER-05 | Phase 5 | Pending |
| USER-06 | Phase 5 | Pending |

**Coverage:**
- v1 requirements: 24 total
- Mapped to phases: 24
- Unmapped: 0

---
*Requirements defined: 2026-03-21*
*Last updated: 2026-03-21 after roadmap creation*
