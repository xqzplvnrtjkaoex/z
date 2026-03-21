---
phase: 01
slug: foundation-and-gateway-infrastructure
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-21
---

# Phase 01 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml workspace |
| **Quick run command** | `cargo test --workspace` |
| **Full suite command** | `cargo test --workspace --all-features` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --workspace`
- **After every plan wave:** Run `cargo test --workspace --all-features`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | GATE-01 | build | `cargo build --workspace` | N/A | pending |
| 01-01-02 | 01 | 1 | GATE-01 | unit | `cargo test -p madome-proto` | pending W0 | pending |
| 01-02-01 | 02 | 1 | GATE-02 | integration | `cargo test -p madome-gateway` | pending W0 | pending |
| 01-02-02 | 02 | 1 | GATE-02 | integration | `cargo test -p madome-gateway --test health` | pending W0 | pending |

*Status: pending / green / red / flaky*

---

## Wave 0 Requirements

- [ ] Test infrastructure compiles with `cargo test --workspace`
- [ ] Proto compilation succeeds in build.rs
- [ ] Gateway binary starts without panic

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Gateway accepts HTTP request and returns gRPC response | GATE-02 | E2E flow requires running services | Start gateway + stub service, curl endpoint, verify response |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
