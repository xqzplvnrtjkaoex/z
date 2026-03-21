---
phase: 2
slug: authentication
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-21
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust native) |
| **Config file** | Cargo.toml workspace members |
| **Quick run command** | `cargo test -p auth` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo check -p auth` (wave 1-2) or `cargo test -p auth` (wave 3)
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirements | Test Type | Automated Command | Status |
|---------|------|------|-------------|-----------|-------------------|--------|
| 02-01-T1 | 01 | 1 | AUTH-01..05, GATE-03..05 | compile | `cargo check -p madome-proto` | ⬜ pending |
| 02-01-T2 | 01 | 1 | (infra) | shell | `docker-compose config --quiet` | ⬜ pending |
| 02-01-T3 | 01 | 1 | AUTH-01..05 | compile | `cargo check -p auth-schema -p auth-migration` | ⬜ pending |
| 02-02-T1 | 02 | 2 | AUTH-01, AUTH-02 | compile | `cargo check -p auth` | ⬜ pending |
| 02-02-T2 | 02 | 2 | AUTH-03..05, GATE-05 | compile | `cargo check -p auth` | ⬜ pending |
| 02-02-T3 | 02 | 2 | AUTH-01..05, GATE-05 | compile | `cargo check -p auth` | ⬜ pending |
| 02-03-T1 | 03 | 2 | GATE-03, GATE-04 | compile | `cargo check -p gateway` | ⬜ pending |
| 02-03-T2 | 03 | 2 | GATE-03..05 | compile | `cargo check -p gateway` | ⬜ pending |
| 02-04-T1 | 04 | 3 | AUTH-01..05 | contract | `cargo test -p auth --test contract_tests -- --list` | ⬜ pending |
| 02-04-T2 | 04 | 3 | GATE-03..05 | integration | `cargo test -p gateway --test auth_integration -- --list` | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `docker-compose.yml` — PostgreSQL + Redis containers (Plan 01, Task 2)
- [ ] `services/auth/schema/` — SeaORM entity definitions (Plan 01, Task 3)
- [ ] `services/auth/migration/` — SeaORM migration crate (Plan 01, Task 3)

*Wave 0 items are covered by Plan 01 (wave 1). No separate Wave 0 plan needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| WebAuthn browser flow | AUTH-01 | Requires browser authenticator API | Use webauthn-rs SoftPasskey in contract tests as substitute |

*SoftPasskey provides automated substitute for browser WebAuthn flow in contract tests (services/auth/tests/contract_tests.rs).*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
