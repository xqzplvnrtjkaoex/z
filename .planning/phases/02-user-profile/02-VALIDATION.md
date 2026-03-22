---
phase: 2
slug: user-profile
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-22
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test + mockall 0.14 + testcontainers 0.27 |
| **Config file** | Cargo.toml `[dev-dependencies]` and `#[cfg(test)]` |
| **Quick run command** | `cargo test -p user` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p user`
- **After every plan wave:** Run `cargo test --workspace && cargo clippy --workspace`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | USER-PROFILE-01 | unit | `cargo test -p user -- create_user` | ❌ W0 | ⬜ pending |
| 02-01-02 | 01 | 1 | USER-PROFILE-01 | unit | `cargo test -p user -- get_user` | ❌ W0 | ⬜ pending |
| 02-01-03 | 01 | 1 | USER-PROFILE-01 | unit | `cargo test -p user -- get_user_by_handle` | ❌ W0 | ⬜ pending |
| 02-01-04 | 01 | 1 | USER-PROFILE-01 | unit | `cargo test -p user -- list_users` | ❌ W0 | ⬜ pending |
| 02-01-05 | 01 | 1 | USER-PROFILE-01 | unit | `cargo test -p user -- update_user` | ❌ W0 | ⬜ pending |
| 02-01-06 | 01 | 1 | USER-PROFILE-01 | unit | `cargo test -p user -- handle_validation` | ❌ W0 | ⬜ pending |
| 02-01-07 | 01 | 1 | USER-PROFILE-01 | unit | `cargo test -p user -- name_validation` | ❌ W0 | ⬜ pending |
| 02-01-08 | 01 | 1 | USER-PROFILE-01 | integration | `cargo test -p user --test user_repository` | ❌ W0 | ⬜ pending |
| 02-02-01 | 02 | 2 | USER-PROFILE-02 | unit | `cargo test -p user -- deactivate_user` | ❌ W0 | ⬜ pending |
| 02-02-02 | 02 | 2 | USER-PROFILE-02 | unit | `cargo test -p user -- activate_user` | ❌ W0 | ⬜ pending |
| 02-02-03 | 02 | 2 | USER-PROFILE-02 | unit | `cargo test -p user -- change_role` | ❌ W0 | ⬜ pending |
| 02-02-04 | 02 | 2 | USER-PROFILE-02 | unit | `cargo test -p user -- self_modification` | ❌ W0 | ⬜ pending |
| 02-02-05 | 02 | 2 | USER-PROFILE-02 | unit | `cargo test -p user -- reject_owner` | ❌ W0 | ⬜ pending |
| 02-02-06 | 02 | 2 | USER-PROFILE-02 | unit | `cargo test -p user -- deactivated_visibility` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `services/user/src/domain/` — domain types and port traits needed before any test
- [ ] Workspace `[dev-dependencies]` — mockall, testcontainers, testcontainers-modules
- [ ] Workspace `[dependencies]` — sea-orm, sea-orm-migration, validator, trait-variant, chrono
- [ ] `services/user/tests/` directory — integration test infrastructure
- [ ] `docker-compose.yml` — PostgreSQL for dev environment
- [ ] Validate `trait_variant` + `mockall` compatibility in a minimal test

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| docker-compose up starts PostgreSQL | USER-PROFILE-01 | Infrastructure | Run `docker-compose up -d` and verify PostgreSQL accepts connections |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
