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
cargo +nightly fmt --all                 # Format (nightly for rustfmt.toml unstable options)
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
| `dev` push | `cargo +nightly fmt --all --check`, `cargo clippy --workspace`, unit + integration tests |
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

## TDD Protocol (Executor Injection)

For tasks with `tdd="true"`, the executor follows a Stub-Red-Green-Refactor cycle at **case-group granularity** (all S/F/E cases for one operation per cycle). Tasks without `tdd="true"` follow normal execution — test-first is preferred but the strict protocol below is not enforced.

### Cycle per Task

1. **Pre-task:** Read `<behavior>` items. Each item = one test function. Determine test layer from file context:
   - `domain/`, `usecase/` → unit test (`#[cfg(test)] mod tests`, same file, mock ports)
   - `adapter/` → integration test (`tests/` directory, testcontainers)
   - `app/rpc/` → covered by service tests (separate task)

2. **Stub** (only if domain types/ports don't exist yet): Create minimal stubs — struct fields tests reference, trait method signatures, error variants tests assert against. Bodies: `todo!()`. Verify: `cargo check -p {crate}`.

3. **Red:** Write ALL test functions for the task's `<behavior>` items. Order by ZOMBIES (simplest success → failures → boundaries). Naming: `should_{behavior}_when_{condition}`. Run `cargo test -p {crate} -- {prefix}` — all must **fail with assertion errors** (not compile errors). If compile errors: return to Stub.

4. **Green:** Implement minimum code to pass all tests. Only logic paths exercised by tests. Run `cargo test -p {crate} -- {prefix}` — all must pass.

5. **Refactor:** Run `cargo clippy -p {crate}`, fix warnings. Extract duplicated setup/logic. Only refactor code with test coverage. Verify: `cargo test -p {crate}` (full crate).

6. **Commit:** Test + implementation together as one atomic commit. Do NOT commit failing tests separately.

### Prohibited Behaviors

- Writing implementation before tests for `tdd="true"` tasks (recovery: revert implementation, write tests from scratch)
- Writing test and implementation in the same edit (must see Red before Green)
- Asserting success without inspecting the value — unwrap and verify fields
- Testing mock configuration (`times()`, `withf()`) instead of behavior — use `returning()` for setup, `assert!()` for output
- Over-implementation beyond test scope — every code path must be exercised by at least one test
- Adding case IDs to test function names — test names describe behavior, not planning artifacts

### Reference

See `.planning/research/TDD-EXECUTOR-INJECTION.md` for full research and rationale.

## Crate Skills

- Store project-local crate skills in `.claude/skills/`, not `~/.claude/skills/`.
- Prefer on-demand docs lookup (context7, `rust-skills:docs`) over generating local skills.
- Generate local skills for crates with complex APIs or low popularity that lack bundled skills.

## Workspace Structure

Rust 2024 edition, resolver 3. Cargo workspace with shared crates + service binaries:

- `proto/` — `.proto` source files (single source of truth for all gRPC contracts)
- `crates/madome-proto` — compiles protos via `tonic-prost-build`, re-exports as `madome_proto::{auth,catalog,user}`
- `crates/madome-common` — tracing init, env helpers
- `services/gateway` — axum REST entry point, holds gRPC clients in `AppState`, translates REST→gRPC. Uses routes/ + middleware/ + state.rs (no 4-layer pattern)
- `services/{auth,catalog,user}` — tonic gRPC services, each follows 4-layer architecture: `domain/` (types, ports, errors) → `usecase/` (business logic) → `app/` (tonic handler) → `adapter/` (concrete implementations). See `.planning/PROJECT.md` Internal Service Architecture for details

### Proto Workflow

Proto files live in `proto/` and are compiled by `crates/madome-proto/build.rs`. When modifying protos:
1. Edit `proto/*.proto`
2. If adding a new `.proto` file, add it to `build.rs` compile list and create a module in `src/lib.rs`
3. `cargo build -p madome-proto` triggers recompilation; downstream crates pick up changes automatically

## Extended Workflow

This project extends the GSD workflow with custom skills between standard stages:

### Pipeline Order

```
discuss -> /case -> (ui-phase) -> (research) -> plan -> (review) -> (assumptions)
  -> execute -> (ui-review) -> (validate) -> verify -> (simplify) -> ship
```

- `/case`: behavioral case discovery. Produces `{padded_phase}-CASES.md` with S/F/E case tables.
- `gsd:ui-phase`: frontend only, UI design contract. `gsd:ui-review`: post-implementation visual audit.
- `gsd:research`: standalone pre-plan research for unfamiliar libraries/protocols.
- `gsd:review`: cross-AI peer review of PLAN.md before execution.
- `gsd:assumptions`: surface implicit assumptions in the plan.
- `gsd:validate`: plan-vs-implementation audit (complements verify's UAT).
- `gsd:simplify`: code quality review + cleanup (deduplication, efficiency, reuse). Runs 3 parallel review agents, then fixes issues directly.
- `gsd:debug`: systematic debugging during execute (not a pipeline step, used on demand).
- Steps in `(parentheses)` are optional. The workflow is iterative — any stage can return to an earlier stage and redo forward.

### CASES.md Integration with Plan-Phase

When `{phase_dir}/*-CASES.md` exists, the planner should:
- Read it as additional input alongside CONTEXT.md
- Map must-priority cases to required test tasks in PLAN.md
- Annotate case IDs as `[OperationName.S1]` in task `<behavior>` items (IDs restart per operation)
- Set `tdd="true"` on tasks with behavioral test requirements
- Flag open questions (Q1-QN) as items requiring resolution

When CASES.md does not exist, plan-phase works normally from CONTEXT.md + REQUIREMENTS.md alone.

### CASES.md Integration with Validate-Phase

When `{phase_dir}/*-CASES.md` exists, validate-phase should:
- Cross-reference CASES.md cases against PLAN.md `<behavior>` items
- Flag `must`-priority cases in CASES.md that have no corresponding test as MISSING
- Include unmapped cases in the gap analysis alongside Nyquist coverage checks

### CASES.md Integration with Verify-Work

When `{phase_dir}/*-CASES.md` exists, verify-work should:
- Use `must`-priority success and failure cases as UAT scenario source
- Verify each `must` case's Expected Outcome matches actual behavior
- Report cases as PASS/FAIL alongside standard UAT checks

### Detailed Workflow Reference

See `.planning/WORKFLOW.md` for the complete artifact dependency chain, stage-by-stage breakdown, and iterative return path rules.

## Architecture Reference

Detailed architecture, data flows, and design decisions are in `.planning/`:
- `.planning/PROJECT.md` — domain decisions, service topology, internal service architecture, auth/renewal design
- `.planning/research/ARCHITECTURE.md` — system overview, data flows, DB schema design
- `.planning/research/ARCHITECTURE-PATTERNS.md` — internal service architecture pattern research (Clean/Hexagonal/Pragmatic comparison)
- `.planning/WORKFLOW.md` — extended GSD pipeline with custom skills, session management, milestone lifecycle, idea management
- `.planning/REQUIREMENTS.md` — functional requirements with IDs (REQ-XX), referenced by ROADMAP phases
- `.planning/ROADMAP.md` — phased build plan with dependencies

## Keeping CLAUDE.md in Sync

When implementation changes affect information in this file (e.g., new crates, renamed env vars, changed ports, added proto files), update CLAUDE.md as part of the same change. Do not leave stale references.

## Planning

- Discuss changes collaboratively before modifying planning docs. Do not immediately edit.
- Present options, share opinion, iterate until confirmation.
