---
type: quick
id: 260323-pwj
title: Replace standalone conversion functions with From/TryFrom impls in user rpc layer
completed: "2026-03-23T09:44:34Z"
duration_min: 5
tasks_completed: 1
files_modified: 11
commit: 4f002ff
tags: [refactor, user, type-conversion, idiomatic-rust]
key_decisions:
  - ProtoRole newtype required to satisfy orphan rules (cannot impl TryFrom<i32> for UserRole since neither is local)
  - itertools map_into() used in list_users over manual .map(|u| UserResponse::from(u))
---

# Quick Task 260323-pwj: Replace standalone conversion functions with From/TryFrom impls

**One-liner:** Replaced three standalone rpc conversion fns (`user_to_response`, `domain_role_to_proto`, `proto_role_to_domain`) with idiomatic `From<&User> for UserResponse`, `From<UserRole> for Role`, and `TryFrom<ProtoRole> for UserRole` trait impls across the user service rpc layer.

## What Changed

### `services/user/src/app/rpc/mod.rs`

Removed three standalone functions, added three trait impls:

- `fn user_to_response(user: &User) -> UserResponse` → `impl From<&User> for UserResponse`
- `fn domain_role_to_proto(role: UserRole) -> Role` → `impl From<UserRole> for Role`
- `fn proto_role_to_domain(role: i32) -> Result<UserRole, Status>` → `pub(crate) struct ProtoRole(pub i32)` + `impl TryFrom<ProtoRole> for UserRole`

The `ProtoRole` newtype is needed because orphan rules prevent `impl TryFrom<i32> for UserRole` (neither `i32` nor `UserRole` is defined in the user crate).

### Caller files (10 files updated)

| File | Change |
|------|--------|
| `get_user.rs` | `user_to_response(&user)` → `UserResponse::from(&user)`; removed import |
| `create_user.rs` | `proto_role_to_domain(req.role)?` → `UserRole::try_from(ProtoRole(req.role))?`; `user_to_response(&user)` → `UserResponse::from(&user)` |
| `update_user.rs` | `user_to_response(&user)` → `UserResponse::from(&user)`; removed import |
| `deactivate_user.rs` | `user_to_response(&user)` → `UserResponse::from(&user)`; removed import |
| `activate_user.rs` | `user_to_response(&user)` → `UserResponse::from(&user)`; removed import |
| `change_role.rs` | `proto_role_to_domain(req.new_role)?` → `UserRole::try_from(ProtoRole(req.new_role))?`; `user_to_response(&user)` → `UserResponse::from(&user)` |
| `get_user_by_handle.rs` | `user_to_response(&user)` → `UserResponse::from(&user)`; removed import |
| `list_users.rs` | `.map(user_to_response)` → `.map_into()` with `use itertools::Itertools;`; removed import |
| `services/user/Cargo.toml` | Added `itertools = { workspace = true }` |
| `Cargo.lock` | Updated to record itertools for user crate |

## Verification

- `cargo build -p user` — passes (Finished dev profile)
- `cargo test -p user` — 59/59 unit tests pass; 8 integration tests skipped due to Docker socket not found (pre-existing environment limitation, not caused by this change)
- `cargo clippy -p user` — no warnings
- `grep -r "user_to_response|domain_role_to_proto|proto_role_to_domain" services/user/src/` — no matches

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Self-Check: PASSED

- Commit `4f002ff` exists in git log
- All 10 planned files modified
- Standalone functions fully removed (grep returns no output)
