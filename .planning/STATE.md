---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: unknown
stopped_at: PROJECT.md and ROADMAP.md updated with project-wide decisions and phase restructuring
last_updated: "2026-03-22T08:38:00.000Z"
progress:
  total_phases: 6
  completed_phases: 1
  total_plans: 2
  completed_plans: 2
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-22)

**Core value:** Reliably mirror books from external sources and allow authenticated users to browse them.
**Current focus:** Phase 02 — user-profile (pre-planning)

## Current Position

Phase: 02 (user-profile) — NOT STARTED
Plan: 0 of ?

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

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Roadmap]: v1 scope covers gateway, auth, catalog, and user services only. File service, scraper, and renewal are v2.
- [Roadmap]: Phase 4 (Catalog Core) can start in parallel with Phase 3 (Auth) since both only depend on Phase 1.
- [Architecture]: URL convention: /v1/ prefix, no /api prefix. Path-based versioning.
- [Architecture]: 3 shared crates: madome-proto, madome-core (domain types), madome-common (infra).
- [Architecture]: DB schema/migration per service folder (schema/, src/, migration/ co-located).
- [Architecture]: Internal PK (UUID) + external_id column separation. UUIDv7 for predictable, UUIDv4 for security-sensitive.
- [Architecture]: Renewal via canonical_id denormalization (Union-Find pattern). No cross-service sync queue needed.
- [Architecture]: Catalog requests File service for image count during publish verification.
- [Architecture]: Scraper has two isolated tasks: Update Checker (metadata) and New Book Discovery (uploads).
- [Architecture]: OpenTelemetry + tracing. request_id as UUIDv7.
- [01-01]: Stub service Cargo.toml files created for workspace loading; replaced in Plan 02.
- [01-01]: protoc installed via homebrew (system dependency for tonic-prost-build).
- [01-01]: DeadlineExceeded maps to Unavailable with "timeout: " prefix in AppError gRPC mapping.
- [Phase 01-02]: Health RPC uses unit type () not prost_types::Empty — tonic-prost 0.14 maps google.protobuf.Empty to Rust () type
- [Phase 01-02]: Gateway split into lib.rs + main.rs to enable integration test imports from tests/ directory
- [Phase 01-02]: Lazy gRPC client connections (connect_lazy) used in integration tests to avoid startup ordering requirements

### Pending Todos

- Create User Profile phase directory and run discuss-phase for new Phase 2
- Run plan-phase for User Profile (new Phase 2), then plan-phase for Authentication (new Phase 3)

### Blockers/Concerns

- Research flags Phase 4 (Scraper) for deeper research (hitomi_la crate coverage), but that is v2 scope.
- nginx auth_request cookie limitation must be addressed when FILE-01/FILE-02 enter scope (v2).
- Renewal design significantly simplified from original research: canonical_id denormalization replaces cross-service sync queue.
- ARCHITECTURE-PATTERNS.md was written under incorrect "never mock DB" assumption — CONTEXT.md decisions take precedence. Research will re-run during plan-phase.
- mockall + trait_variant compatibility unverified — must validate in Plan 01 (fallback: async_trait)
- Phase restructuring: ROADMAP.md updated. Phase directories need renaming (02-authentication -> 03-authentication) and new 02-user-profile directory creation
- users table moved to User service — auth service uses gRPC client for user data. Cross-service dependency during registration and login flows

## Session Continuity

Last session: 2026-03-22
Stopped at: PROJECT.md and ROADMAP.md updated with project-wide decisions and phase restructuring
Resume file: .planning/ROADMAP.md
Next action: Create User Profile phase directory (02-user-profile), then discuss-phase for Phase 2 User Profile
