---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: active
stopped_at: madome-core removed, AppError moved to gateway, nightly rustfmt configured
last_updated: "2026-03-24T00:00:00.000Z"
last_activity: 2026-03-24
progress:
  total_phases: 6
  completed_phases: 2
  total_plans: 10
  completed_plans: 6
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-23)

**Core value:** Reliably mirror books from external sources and allow authenticated users to browse them.
**Current focus:** Phase 02 complete — next: Phase 03 authentication

## Current Position

Phase: 03
Plan: Not started

## Performance Metrics

**Velocity:**

- Total plans completed: 1
- Average duration: 3 min
- Total execution time: 0.05 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-foundation-and-gateway-infrastructure | 1/2 | 3 min | 3 min |

**Recent Trend:**

- Last 5 plans: 3 min
- Trend: -

*Updated after each plan completion*
| Phase 01-foundation-and-gateway-infrastructure P02 | 7 | 3 tasks | 28 files |
| Phase 02-user-profile P01 | 8 | 2 tasks | 22 files |
| Phase 02-user-profile P02 | 2 | 2 tasks | 5 files |
| Phase 02-user-profile P03 | 13 | 2 tasks | 24 files |
| Phase 02-user-profile P04 | 16 | 2 tasks | 6 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Roadmap]: v1 scope covers gateway, auth, catalog, and user services only. File service, scraper, and renewal are v2.
- [Roadmap]: Phase 4 (Catalog Core) can start in parallel with Phase 3 (Auth) since both only depend on Phase 1.
- [Architecture]: URL convention: /v1/ prefix, no /api prefix. Path-based versioning.
- [Architecture]: 2 shared crates: madome-proto (cross-service contract), madome-common (infra). Domain types live in each service.
- [Architecture]: DB schema/migration per service folder (schema/, src/, migration/ co-located).
- [Architecture]: Internal PK (UUID) + external_id column separation. UUIDv7 for predictable, UUIDv4 for security-sensitive.
- [Architecture]: Renewal via canonical_id denormalization (Union-Find pattern). No cross-service sync queue needed.
- [Architecture]: Catalog requests File service for image count during publish verification.
- [Architecture]: Scraper has two isolated tasks: Update Checker (metadata) and New Book Discovery (uploads).
- [Architecture]: OpenTelemetry + tracing. request_id as UUIDv7.
- [01-01]: Stub service Cargo.toml files created for workspace loading; replaced in Plan 02.
- [01-01]: protoc installed via homebrew (system dependency for tonic-prost-build).
- [01-01]: DeadlineExceeded maps to Unavailable with "timeout: " prefix in gateway's gRPC-to-REST error mapping.
- [Phase 01-02]: Health RPC uses unit type () not prost_types::Empty — tonic-prost 0.14 maps google.protobuf.Empty to Rust () type
- [Phase 01-02]: Gateway split into lib.rs + main.rs to enable integration test imports from tests/ directory
- [Phase 01-02]: Lazy gRPC client connections (connect_lazy) used in integration tests to avoid startup ordering requirements
- [Phase 02-user-profile]: DeriveIden Table variant used as .as_enum(UserRoleEnum::Table) - sea-query API requires variant value not bare enum type
- [Phase 02-user-profile]: trait_variant + mockall: #[cfg_attr(test, mockall::automock)] must appear BEFORE #[trait_variant::make] - validated working in Plan 01
- [Phase 02-user-profile]: Handle validation uses custom check_handle_chars function (not #[validate(regex)] attribute) for clean LazyLock<Regex> integration
- [Phase 02-user-profile]: Func::lower() from sea_query used for case-insensitive handle filter (type-safe vs Expr::cust raw SQL)
- [Phase 02-user-profile]: classify_db_err inspects error message for 'duplicate key'/'23505' to map to RepositoryError::UniqueViolation since sea-orm wraps sqlx errors without stable unique-violation enum variant (now replaced by From<DbErr> impl per quick task 260322-vwa)
- [Phase 02-user-profile]: Use #[automock(target = UserRepository)] with trait_variant to generate Send-compatible mock for unit tests
- [Phase 02-user-profile]: base64 crate added to workspace for opaque cursor encoding in ListUsers pagination
- [Phase 02-user-profile]: CallerContext extracted from gRPC metadata headers x-caller-id and x-caller-role
- [Phase 02-user-profile]: Per-test fresh DatabaseConnection (not shared pool) against shared testcontainers container URL: prevents pool exhaustion across independent tokio runtimes in integration tests
- [Phase 02-user-profile]: Migration user_role enum via raw SQL execute_unprepared: DeriveIden generates 'user_role_enum' from UserRoleEnum, sea-orm entity expects 'user_role'; raw SQL ensures correct name

### Pending Todos

- Plan and execute Authentication (Phase 3)
- Plan and execute Catalog Core (Phase 4) — can parallelize with Phase 3
- ~~Research and codify tracing/OpenTelemetry conventions as `.claude/rules/rust-tracing.md`~~ DONE (2026-03-23)
- ~~Research and codify documentation conventions as `.claude/rules/rust-documentation.md`~~ DONE (2026-03-23)
- Apply tracing conventions to remaining services (gateway, auth, catalog) as they are built
- Implement OpenAPI documentation: utoipa feature gate, gen-openapi binary, GitHub Pages deployment
- Apply `#![warn(missing_docs)]` to shared crates (madome-proto, madome-common) and add missing rustdoc

### Blockers/Concerns

- Research flags Phase 4 (Scraper) for deeper research (hitomi_la crate coverage), but that is v2 scope.
- nginx auth_request cookie limitation must be addressed when FILE-01/FILE-02 enter scope (v2).
- Renewal design significantly simplified from original research: canonical_id denormalization replaces cross-service sync queue.
- ARCHITECTURE-PATTERNS.md was written under incorrect "never mock DB" assumption — CONTEXT.md decisions take precedence. Research will re-run during plan-phase.
- mockall + trait_variant compatibility verified: use #[automock(target = SendTrait)] pattern
- users table moved to User service — auth service uses gRPC client for user data. Cross-service dependency during registration and login flows

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 260322-of1 | Update PROJECT.md and ROADMAP.md with pending decisions and phase restructuring | 2026-03-22 | 9e691ba | [260322-of1-update-project-md-and-roadmap-md-with-pe](./quick/260322-of1-update-project-md-and-roadmap-md-with-pe/) |
| 260322-vwa | Refactor user service and gateway: typed header constants, From<DbErr>, CallerContext middleware, handler file renames | 2026-03-22 | 17e1012 | [260322-vwa-refactor-user-service-and-gateway-typed-](./quick/260322-vwa-refactor-user-service-and-gateway-typed-/) |

## Session Continuity

Last session: 2026-03-24
Last activity: Removed madome-core crate (YAGNI — AppError was gateway-only, domain types stay per-service). Moved AppError to gateway/src/error.rs. Simplified .map_err(AppError::from)? to ?. Added rustfmt.toml with nightly import grouping (group_imports, imports_granularity). Updated conventions, PROJECT.md, CLAUDE.md.
Stopped at: 2 commits on dev (refactor + style). Clean working tree.
Resume file: .planning/STATE.md
Next action: /gsd:plan-phase 3 (authentication)
