# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Language

- All git-tracked content (code, comments, docs, planning files, commit messages) MUST be in English.

## Domain Terminology

- Use "book" for the primary domain entity (manga/comic/gallery), not "work".
- Applies to: code identifiers, API endpoints, database entities, proto definitions, documentation.

## Build & Test Commands

```bash
cargo build                              # Build all workspace members
cargo build -p gateway                   # Build a single crate
cargo test                               # Run all tests
cargo test -p gateway                    # Run tests for a single crate
cargo test -p gateway -- gateway_own     # Run a single test by name substring
cargo test -p gateway --test health_integration  # Run a specific integration test file
cargo clippy --workspace                 # Lint all crates
```

## Environment Variables

Services discover each other via environment variables (no config files):

| Variable | Default | Used By |
|----------|---------|---------|
| `AUTH_LISTEN_ADDR` | (required) | auth |
| `CATALOG_LISTEN_ADDR` | (required) | catalog |
| `USER_LISTEN_ADDR` | (required) | user |
| `AUTH_GRPC_ADDR` | (required) | gateway |
| `CATALOG_GRPC_ADDR` | (required) | gateway |
| `USER_GRPC_ADDR` | (required) | gateway |
| `GATEWAY_ADDR` | `0.0.0.0:3000` | gateway |
| `USER_DATABASE_URL` | (required) | user |
| `RUST_LOG` | `{service}=debug,madome=debug,info` | all |

Convention: gRPC services listen on 50051+ (auth=50051, catalog=50052, user=50053).

`DOCKER_HOST` is set in `.env` (for testcontainers remote Docker). Image builds use local Docker (unset `DOCKER_HOST` in justfile recipe).

## Git Workflow

### Branches
- `master` — latest stable. No direct push. All changes via PR.
- `dev` — development branch. Direct push allowed.
- Phase branches branch from `dev`, merge back to `dev` with `--no-ff`.
- `dev` → `master` merge at milestone completion via PR with `--no-ff`.
- Delete phase branch after merge. Tag if needed for reference.
- Hotfix: branch from `master` → PR to `master` → merge back to `dev`.

### Commit Messages
[Conventional Commits](https://www.conventionalcommits.org/) with a **descriptive scope**:

```
feat(auth): implement passkey registration ceremony
fix(gateway): correct JWT expiry check off-by-one
docs(planning): capture phase context for authentication
refactor(catalog): extract tag query builder
deps: bump sea-orm to 1.2
deps(auth): add webauthn-rs dependency
```

**Types:** `feat`, `fix`, `docs`, `refactor`, `test`, `perf`, `ci`, `build`, `style`, `chore`, `deps`

- Scope should describe the area of change (service, crate, topic, etc.).
- Use `deps` as a type (not `chore(deps)`) for dependency changes.
- Breaking changes: append `!` after scope (e.g., `feat(auth)!: remove password login`) or add `BREAKING CHANGE:` footer.
- Never use phase numbers as scope (e.g., `docs(02)` is wrong).

### CI (GitHub Actions)

Workflows authored in TypeScript via [gaji](https://github.com/dodok8/gaji) (`npx gaji build`). Do not install gaji globally or as a project dependency.

| Trigger | Checks |
|---------|--------|
| `dev` push | `cargo fmt --check`, `cargo clippy --workspace`, unit + integration tests |
| `master` push | Above + service tests + E2E contract tests + `cargo audit` + `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` |
| Weekly cron | `cargo audit` (dependency vulnerability scan) |

All merge styles are `--no-ff` (preserve full commit history).

## Development Tooling

- Use `justfile` for dev commands. Do NOT create shell scripts in `scripts/`.
- Use latest stable versions for infrastructure dependencies (PostgreSQL, Redis, etc.).
- Test-only dependencies (testcontainers, etc.) go in `[dev-dependencies]`, not `[dependencies]`.

## Testing

- TDD strongly preferred: write tests before implementation.
- Verification must be automated. Write test code (unit/integration/service/E2E) that an agent can execute via `cargo test`. If automation is genuinely infeasible, write structured test scenario scripts that an agent can follow step-by-step (e.g., curl commands with expected responses). Never rely on human-only manual testing.
- Unit tests MAY mock ports (trait implementations) for isolated business logic testing.
- Integration tests use testcontainers with real PostgreSQL/Redis.
- Test layers:
  1. Unit tests — mock ports, test usecase/domain logic (`#[cfg(test)]`)
  2. Integration tests — testcontainers with real DB, test adapters
  3. Service tests — individual gRPC service with tonic in-process Channel
  4. E2E contract tests — through Gateway REST API, scenario-based

## Crate Skills

Store generated crate skills in `.claude/skills/` (project-local), not `~/.claude/skills/` (global).

### Creation

- When: after plan-phase completes, read PLAN.md to identify new crates that lack
  bundled or local skills. Dispatch a background subagent to run
  `sync-crate-skills {crate1} {crate2} ...` with explicit crate names.
- Skip: crates with existing bundled or local skills, well-known simple crates
  (uuid, serde, rand, etc.).
- Only generate for crates with complex APIs or uncommon usage patterns.

### Update

- Patch version bump (1.0.1 → 1.0.2): no update needed.
- Minor version bump (0.7 → 0.8): update recommended.
- Major version bump (1.x → 2.x): update required.
- API mismatch discovered (build error, deprecated warning): update immediately.
- New milestone start: review all local skills for staleness.
- Use `update-crate-skill {crate}` for individual updates, dispatch via background subagent.

## Workspace Structure

Rust 2024 edition, resolver 3. Cargo workspace with shared crates + service binaries:

- `proto/` — `.proto` source files (single source of truth for all gRPC contracts)
- `crates/madome-proto` — compiles protos via `tonic-prost-build`, re-exports as `madome_proto::{auth,catalog,user}`
- `crates/madome-core` — domain error types (`AppError`) with axum `IntoResponse` + tonic `Status` conversion
- `crates/madome-common` — tracing init, env helpers
- `services/gateway` — axum REST entry point, holds gRPC clients in `AppState`, translates REST→gRPC. Uses routes/ + middleware/ + state.rs (no 4-layer pattern)
- `services/{auth,catalog,user}` — tonic gRPC services, each follows 4-layer architecture: `domain/` (types, ports, errors) → `usecase/` (business logic) → `app/` (tonic handler) → `adapter/` (concrete implementations). See `.planning/PROJECT.md` Internal Service Architecture for details

### Proto Workflow

Proto files live in `proto/` and are compiled by `crates/madome-proto/build.rs`. When modifying protos:
1. Edit `proto/*.proto`
2. If adding a new `.proto` file, add it to `build.rs` compile list and create a module in `src/lib.rs`
3. `cargo build -p madome-proto` triggers recompilation; downstream crates pick up changes automatically

## Architecture Reference

Detailed architecture, data flows, and design decisions are in `.planning/`:
- `.planning/PROJECT.md` — domain decisions, service topology, internal service architecture, auth/renewal design
- `.planning/research/ARCHITECTURE.md` — system overview, data flows, DB schema design
- `.planning/research/ARCHITECTURE-PATTERNS.md` — internal service architecture pattern research (Clean/Hexagonal/Pragmatic comparison)
- `.planning/ROADMAP.md` — phased build plan with dependencies

## Keeping CLAUDE.md in Sync

When implementation changes affect information in this file (e.g., new crates, renamed env vars, changed ports, added proto files), update CLAUDE.md as part of the same change. Do not leave stale references.

## Planning

- Discuss changes collaboratively before modifying planning docs. Do not immediately edit.
- Present options, share opinion, iterate until confirmation.
