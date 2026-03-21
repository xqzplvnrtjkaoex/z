# Feature Research

> **Terminology note:** This document uses "work" throughout. The project canonical term is **"book"**. See PROJECT.md for current decisions.

**Domain:** Manga mirroring and content aggregation service
**Researched:** 2026-03-21
**Confidence:** MEDIUM (based on training data knowledge of similar platforms: nhentai, e-hentai/exhentai, hitomi.la, MangaDex, Komga, Kavita; no live web search available to verify current state)

## Feature Landscape

### Table Stakes (Users Expect These)

Features users of manga aggregation/mirroring services assume exist. Missing any of these and the product feels broken or unusable.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Catalog browsing (paginated)** | Core interaction loop. Users must be able to scroll through available works sorted by recency or relevance. | LOW | Standard cursor-based or offset pagination on catalog service. Recency ordering maps naturally to hitomi.la's ID ordering (higher = newer). |
| **Work detail page (metadata view)** | Users expect to see title, artist, group, language, type, tags, page count, upload date before deciding to read. | LOW | Direct mapping from hitomi.la's gallery info structure. Serve via catalog gRPC. |
| **Tag-based filtering/querying** | The primary discovery mechanism for this domain. Users filter by artist, character, series, tag, language, type. | MEDIUM | Multi-dimensional tag system. Hitomi uses typed tags (artist:X, series:Y, etc.). Need compound queries (AND/OR). Already in PROJECT.md requirements. |
| **Image serving (page reader)** | The entire value proposition. Users read manga pages as images. Must be fast, sequential, and reliable. | MEDIUM | nginx auth_request pattern already designed. Need to handle large images efficiently (webp/avif conversion optional but beneficial). Sequential prefetch hints improve UX. |
| **Authentication and access control** | Service is for a small community, not public. Users must log in to access content. | MEDIUM | Passkey-only is already decided. JWT + session hybrid designed. This is table stakes for any gated community. |
| **Search (text-based)** | Users expect to find works by title, artist name, or other text fields. Tag-only browsing is insufficient. | MEDIUM | Full-text search over titles and tag values. PostgreSQL's `tsvector`/`tsquery` or `ILIKE` with trigram indexes sufficient at this scale. No need for Elasticsearch. |
| **Image quality preservation** | Aggregation users are sensitive to quality loss. Lossy re-encoding or heavy compression drives users away. | LOW | Serve original files. Do not transcode unless explicitly requested. Store originals on filesystem as-is from source. |
| **Work page ordering** | Pages must display in correct order. Misordered pages = completely broken reading experience. | LOW | Scraper must capture and store page ordering metadata. Hitomi.la provides ordered file lists. |
| **Scraper reliability (new work ingestion)** | The catalog must stay current. If new works stop appearing, the service has no ongoing value over a static archive. | HIGH | Hitomi.la changes its obfuscation/JS regularly. `hitomi_la` crate helps but external source changes are the biggest operational risk. Needs monitoring/alerting when scraping fails. |
| **Work metadata accuracy** | Tags, titles, artists must match the source. Inaccurate metadata destroys trust and makes search/filtering useless. | LOW | Direct mirroring from source metadata. Hash-based change detection (already in PROJECT.md) catches updates. |

### Differentiators (Competitive Advantage)

Features that set Madome apart from using hitomi.la directly or other aggregators. These are the reasons a small community would prefer this service.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Book renewal tracking (version graph)** | Hitomi.la frequently deletes and re-uploads works under new IDs. Users lose their history/bookmarks. Madome preserves the continuity by linking old and new IDs via a relation graph. No public aggregator does this well. | HIGH | Already designed in PROJECT.md. Canonical ID + graph relation + queue-based cross-service sync. This is genuinely novel for the domain. |
| **User tastes (like/dislike)** | Most aggregators offer favorites only. A binary like/dislike system enables future recommendation and personal filtering (hide disliked works, surface similar to liked). | MEDIUM | User service already scoped. Simple per-user-per-work preference storage. Value multiplies when combined with filtering. |
| **Reading history with progress** | Know which works you have read and where you stopped. Hitomi.la has no account system; other aggregators offer basic bookmarks at best. | MEDIUM | User service already scoped. Store last-read page per work per user. Enables "continue reading" and "unread" filtering. |
| **Reliable availability** | Source sites go down, get blocked by ISPs, or have aggressive rate limiting. A local mirror is always available when the source is not. | LOW | This is inherent to the mirroring architecture. The value is in the operational model, not a feature to build. |
| **Decreasing-frequency update checks** | Smart polling that catches early corrections (common in first hours after upload) while reducing load over time. Better than fixed-interval polling. | MEDIUM | Already designed in PROJECT.md. Elegant approach that balances freshness with efficiency. |
| **Taste-based filtering (negative filtering)** | Filter out works matching disliked tags or explicitly disliked works from browse results. Most aggregators lack negative filtering entirely. | MEDIUM | Requires: user tastes + tag query integration. Query catalog with exclusion set derived from user preferences. Very high value for curating experience. |
| **Duplicate/renewal history display** | Show users the full lifecycle of a work: original upload, renewals, ID changes. Transparency that no other platform offers. | LOW | UI concern (deferred per PROJECT.md), but the API must expose renewal graph data. Catalog service stores relations; API returns them. |
| **API-first design** | Enables third-party clients, browser extensions, Tachiyomi/Mihon extensions, automation scripts. Most aggregators are scrape-only with no official API. | LOW | Already the chosen architecture. REST API is the primary interface. Document it well and it becomes a platform, not just a site. |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but should be deliberately avoided for Madome.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **User-uploaded content** | "Let users contribute works from other sources." | Moderation nightmare. Legal liability. Quality control collapse. Contradicts the "reliable mirror" value proposition. Community splits between curated and user-uploaded. | Keep single-source mirroring. Add more sources by adding scraper modules, not user uploads. |
| **Comments/forums/social features** | "Build community engagement." | Massive moderation burden. Shifts product focus from content delivery to social platform. Small community can use Discord/Matrix instead. | Link to external community platform (Discord invite). |
| **Real-time notifications (WebSocket push)** | "Notify me when new works match my interests." | Complex infrastructure (WebSocket server, connection management, per-user subscription state). Overkill for a small community with periodic content updates. | Polling-based "new since last visit" indicator. RSS/Atom feed for power users (very cheap to implement). |
| **Multi-source aggregation (from day one)** | "Support nhentai, e-hentai, MangaDex simultaneously." | Each source has different metadata schemas, ID systems, tag taxonomies, image hosting patterns. Cross-source deduplication is an unsolved hard problem. Premature generalization kills velocity. | Design scraper interface to be source-agnostic (adapter pattern), but implement only hitomi.la first. Add sources one at a time after v1 is stable. |
| **Automated content recommendations** | "Recommend works based on reading history." | Requires significant ML/statistical infrastructure. Cold-start problem with small community. Taste data is sparse initially. ROI is low until substantial user activity exists. | Simple "more by this artist/series" links (zero ML, just tag matching). Revisit recommendations when taste data reaches critical mass. |
| **Automated image censorship** | Already in PROJECT.md out-of-scope. | ML model integration, false positives, performance overhead, continuous model updates. | Defer entirely to v2+ as noted. |
| **OAuth/social login** | "Let me log in with Google/Discord." | Additional attack surface, dependency on external auth providers, contradicts security-focused Passkey approach. Small community does not need onboarding friction reduction. | Passkey only (already decided). Passkey is more secure and simpler to implement than OAuth flows. |
| **Download/batch export** | "Let me download full galleries as ZIP." | Massive bandwidth spike potential. Enables easy redistribution. Storage/CPU cost for ZIP generation. One user can saturate the server. | Serve images individually. If needed, add rate-limited single-work download later, not batch. |
| **Reading mode customization (webtoon scroll, double page, etc.)** | "Different manga formats need different readers." | Frontend complexity explosion. Madome is API-first with no frontend yet. Reader UX is a client concern. | Expose page dimensions and orientation in API metadata. Let clients implement their own reading modes. |

## Feature Dependencies

```
[Authentication (Passkey + JWT)]
    |
    +--required-by--> [Catalog Browsing]
    |                      |
    |                      +--required-by--> [Tag-based Filtering]
    |                      |                      |
    |                      |                      +--enhances--> [Taste-based Negative Filtering]
    |                      |
    |                      +--required-by--> [Search]
    |
    +--required-by--> [Image Serving]
    |
    +--required-by--> [User Tastes (like/dislike)]
    |                      |
    |                      +--enhances--> [Taste-based Negative Filtering]
    |                      +--enhances--> [Catalog Browsing] (personalization)
    |
    +--required-by--> [Reading History]
                           |
                           +--enhances--> [Catalog Browsing] (unread indicator)

[Scraper (hitomi.la ingestion)]
    |
    +--required-by--> [Catalog CRUD + Publish Workflow]
    |                      |
    |                      +--required-by--> [Work Renewal Tracking]
    |                      |
    |                      +--required-by--> [Tag-based Filtering]
    |                      |
    |                      +--required-by--> [Search]
    |
    +--required-by--> [Image Upload (File Service)]
                           |
                           +--required-by--> [Image Serving]

[Work Renewal Tracking]
    |
    +--requires--> [Catalog CRUD]
    +--requires--> [DB-based Queue (cross-service sync)]
    +--enhances--> [Reading History] (preserve history across renewals)
    +--enhances--> [User Tastes] (preserve tastes across renewals)
```

### Dependency Notes

- **Authentication is the universal prerequisite:** Every user-facing feature requires auth. It must be the first thing built and working.
- **Scraper + Catalog + File Service form the content pipeline:** Without ingested content, nothing else matters. This is the second critical path after auth.
- **User features (tastes, history) require both auth AND content:** They cannot be meaningfully tested without works in the catalog and authenticated users.
- **Work renewal tracking requires a functioning catalog first:** Renewal detection happens during scraper update checks, which depend on existing works in the catalog.
- **Taste-based filtering is an enhancement, not a standalone feature:** It combines user tastes with catalog queries. Both must exist independently first.
- **Search enhances catalog browsing but does not replace it:** Tag-based browsing is the primary path; text search is supplementary.

## MVP Definition

### Launch With (v1)

Minimum viable product -- what is needed for the small community to start using Madome instead of hitomi.la directly.

- [ ] **Passkey authentication** -- Gate access to the community. Without auth, it is an open mirror (unacceptable for a private community).
- [ ] **Scraper: hitomi.la new work detection and ingestion** -- The content pipeline. Without new works flowing in, there is no ongoing value.
- [ ] **Catalog CRUD with publish workflow** -- Store and serve work metadata. The publish verification (page count match) prevents incomplete works from appearing.
- [ ] **Image upload and serving** -- The core value: view manga pages. nginx auth_request pattern for authenticated image access.
- [ ] **Catalog browsing (paginated, sorted by recency)** -- Browse available works. Recency sort maps to hitomi.la ID ordering.
- [ ] **Tag-based filtering (single and multi-tag)** -- Primary discovery mechanism. Users must filter by artist, series, language, tag type.
- [ ] **Basic search (title and tag text matching)** -- Find specific works by name. PostgreSQL full-text or trigram matching.
- [ ] **Work metadata update checking** -- Decreasing-frequency checks catch corrections to recently uploaded works.

### Add After Validation (v1.x)

Features to add once the core content pipeline is stable and the community is actively using the service.

- [ ] **User tastes (like/dislike)** -- Trigger: community members request personal curation. Simple binary preference per work.
- [ ] **Reading history** -- Trigger: users want to track what they have read. Per-user-per-work last-read page.
- [ ] **Book renewal tracking (canonical_id denormalization, book_relations)** -- Trigger: first observed renewal event on hitomi.la. Canonical ID, old-to-new linking, cross-service queue sync.
- [ ] **Taste-based negative filtering** -- Trigger: users have accumulated enough taste data. Exclude disliked works/tags from browse results.
- [ ] **Scraper failure monitoring/alerting** -- Trigger: first silent scraper failure. Health check endpoint, stale-content detection.

### Future Consideration (v2+)

Features to defer until the platform is stable and the community has clear needs.

- [ ] **Additional source scrapers** -- Why defer: each source requires its own adapter, metadata mapping, and deduplication logic. Do not generalize prematurely.
- [ ] **RSS/Atom feeds** -- Why defer: low implementation cost but low priority. Add when power users request it.
- [ ] **Automated image censorship** -- Why defer: ML infrastructure, false positive management, performance impact. Only relevant if the service goes public.
- [ ] **Reporting/moderation tools** -- Why defer: small trusted community does not need formal moderation initially.
- [ ] **Simple recommendations ("more by this artist")** -- Why defer: requires sufficient catalog size and user interaction data. Pure tag-matching, no ML needed.

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Passkey authentication | HIGH | MEDIUM | P1 |
| Scraper (hitomi.la ingestion) | HIGH | HIGH | P1 |
| Catalog CRUD + publish workflow | HIGH | MEDIUM | P1 |
| Image upload + serving | HIGH | MEDIUM | P1 |
| Catalog browsing (paginated) | HIGH | LOW | P1 |
| Tag-based filtering (multi-tag) | HIGH | MEDIUM | P1 |
| Basic search | MEDIUM | LOW | P1 |
| Work metadata update checks | MEDIUM | MEDIUM | P1 |
| User tastes (like/dislike) | MEDIUM | LOW | P2 |
| Reading history | MEDIUM | LOW | P2 |
| Work renewal tracking | HIGH | HIGH | P2 |
| Taste-based negative filtering | MEDIUM | MEDIUM | P2 |
| Scraper health monitoring | MEDIUM | LOW | P2 |
| Additional source scrapers | LOW | HIGH | P3 |
| RSS/Atom feeds | LOW | LOW | P3 |
| Automated image censorship | LOW | HIGH | P3 |
| Simple recommendations | LOW | MEDIUM | P3 |

**Priority key:**
- P1: Must have for launch -- without these the product has no value
- P2: Should have, add once core is stable -- these differentiate Madome
- P3: Nice to have, future consideration -- defer until clear demand

## Competitor Feature Analysis

Analysis based on domain knowledge of major platforms in this space.

| Feature | hitomi.la (source) | nhentai | e-hentai/exhentai | Komga (self-hosted) | Madome (our approach) |
|---------|-------------------|---------|-------------------|--------------------|-----------------------|
| Catalog browsing | Yes, paginated | Yes, paginated | Yes, paginated | Yes, paginated | Yes, paginated + authenticated |
| Tag filtering | Multi-tag, typed tags | Multi-tag | Complex tag system with namespaces | Basic metadata filters | Multi-tag with typed tags (mirror source taxonomy) |
| Text search | Title search | Title + tag search | Advanced search syntax | Full-text over metadata | PostgreSQL full-text over title + tags |
| User accounts | No | No (third-party) | Yes (account system) | Yes (OIDC/basic auth) | Yes (Passkey only, high security) |
| Favorites/tastes | No | Favorites only | Favorites + ratings | None (reading status only) | Binary like/dislike (richer than favorites) |
| Reading history | No | No | No | Yes (per-page progress) | Yes (per-page progress, persisted) |
| Work versioning | None (old ID deleted) | None | None | None | Full renewal graph with canonical IDs (unique) |
| API access | JS-obfuscated, no official API | Unofficial API (fragile) | Official API (authenticated) | Full REST API | Full REST API (first-class, documented) |
| Content updates | Live | Periodic scraping | User-submitted | Local file watch | Decreasing-frequency polling (smart) |
| Image quality | Original | Often re-encoded | Original (with limits) | Original (local files) | Original (no transcoding) |
| Availability | Subject to blocks/downtime | Subject to blocks/downtime | Requires invite for exhentai | Always available (local) | Always available (local mirror) |
| Duplicate handling | Deletes old, new ID | No handling | Manual tagging | N/A | Automatic detection + graph linking (unique) |

**Key competitive insights:**

1. **Work renewal tracking is genuinely novel.** No platform in this space handles the "old ID deleted, new ID created" problem well. Users of hitomi.la lose their mental map of works they have seen. This is Madome's strongest differentiator.

2. **API-first is rare.** Most aggregation sites are scrape-targets, not API providers. A well-documented REST API enables ecosystem growth (browser extensions, Tachiyomi/Mihon integration, CLI tools).

3. **Binary like/dislike beats favorites-only.** Dislikes enable negative filtering, which is extremely valuable in a domain with high volume and varied content types. Favorites-only systems cannot express "never show me this again."

4. **Passkey-only auth is both a strength and a barrier.** Higher security than any competitor, but requires users to have Passkey-capable devices. Acceptable for a small, technical community. Would need reassessment if the service goes public.

5. **Self-hosted mirrors always win on availability.** This is table stakes for the concept itself. The value is inherent in running a mirror, not a feature to build.

## Sources

- Domain knowledge of hitomi.la gallery structure (typed tags, gallery info JSON, image URL patterns, JS-based obfuscation)
- Domain knowledge of nhentai API patterns and metadata schema
- Domain knowledge of e-hentai/exhentai gallery system (categories, namespaced tags, rating system)
- Domain knowledge of Komga and Kavita self-hosted manga server features (REST API, OPDS, metadata management)
- Domain knowledge of Tachiyomi/Mihon extension system (source adapter pattern)
- PROJECT.md architecture decisions and existing feature scope

**Confidence note:** All findings are based on training data through May 2025. Web search was unavailable to verify current platform states. Feature comparisons may not reflect changes after that date. The core feature categories (catalog, tags, auth, images, search) are stable across this domain and unlikely to have changed significantly. LOW confidence specifically on: current state of hitomi.la obfuscation patterns, exact Komga/Kavita feature sets in their latest releases.

---
*Feature research for: Manga mirroring and content aggregation service (Madome)*
*Researched: 2026-03-21*
