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
| **Quick run command** | `cargo test -p madome-auth` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p madome-auth`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 0 | AUTH-01 | unit | `cargo test -p madome-auth` | ❌ W0 | ⬜ pending |
| 02-01-02 | 01 | 1 | AUTH-01 | integration | `cargo test -p madome-auth --test passkey_registration` | ❌ W0 | ⬜ pending |
| 02-02-01 | 02 | 1 | AUTH-02 | integration | `cargo test -p madome-auth --test passkey_authentication` | ❌ W0 | ⬜ pending |
| 02-03-01 | 03 | 1 | AUTH-03 | integration | `cargo test -p madome-gateway --test jwt_verification` | ❌ W0 | ⬜ pending |
| 02-04-01 | 04 | 2 | AUTH-04 | integration | `cargo test -p madome-gateway --test jwt_refresh` | ❌ W0 | ⬜ pending |
| 02-05-01 | 05 | 2 | AUTH-05 | integration | `cargo test -p madome-auth --test session_management` | ❌ W0 | ⬜ pending |
| 02-06-01 | 06 | 1 | GATE-03 | integration | `cargo test -p madome-gateway --test route_auth` | ❌ W0 | ⬜ pending |
| 02-07-01 | 07 | 2 | GATE-04 | integration | `cargo test -p madome-gateway --test api_key_auth` | ❌ W0 | ⬜ pending |
| 02-08-01 | 08 | 2 | GATE-05 | contract | `cargo test -p madome-gateway --test error_responses` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/madome-auth/tests/` — test directory structure
- [ ] `crates/madome-auth/tests/common/mod.rs` — shared fixtures (testcontainers PostgreSQL + Redis)
- [ ] `crates/madome-gateway/tests/common/mod.rs` — shared gateway test fixtures
- [ ] `crates/madome-auth/src/schema/` — SeaORM entity definitions
- [ ] `crates/madome-auth/migration/` — SeaORM migration crate
- [ ] `docker-compose.yml` — PostgreSQL + Redis for local dev/test

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| WebAuthn browser flow | AUTH-01 | Requires browser authenticator API | Use webauthn-rs SoftPasskey in tests as substitute |

*SoftPasskey provides automated substitute for browser WebAuthn flow in integration tests.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
