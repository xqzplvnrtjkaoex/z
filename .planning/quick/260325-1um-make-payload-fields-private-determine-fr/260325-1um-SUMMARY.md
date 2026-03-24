---
phase: quick
plan: 260325-1um
subsystem: user
tags: [refactor, encapsulation, payload]
dependency_graph:
  requires: []
  provides: [private payload fields, new() constructors, getter methods]
  affects: [services/user/src/payload/, services/user/src/app/rpc/, services/user/src/usecase/]
tech_stack:
  added: []
  patterns: [constructor pattern, ownership-transferring take_ methods, into_parts() decomposition]
key_files:
  created: []
  modified:
    - services/user/src/payload/user.rs
    - services/user/src/app/rpc/create_user.rs
    - services/user/src/app/rpc/update_user.rs
    - services/user/src/app/rpc/deactivate_user.rs
    - services/user/src/app/rpc/activate_user.rs
    - services/user/src/app/rpc/change_role.rs
    - services/user/src/app/rpc/get_user_by_handle.rs
    - services/user/src/app/rpc/list_users.rs
    - services/user/src/usecase/create_user.rs
    - services/user/src/usecase/update_user.rs
    - services/user/src/usecase/deactivate_user.rs
    - services/user/src/usecase/activate_user.rs
    - services/user/src/usecase/change_role.rs
    - services/user/src/usecase/get_user_by_handle.rs
    - services/user/src/usecase/list_users.rs
decisions:
  - "UpdateUserPayload uses take_handle/take_name (Option::take) to transfer ownership without cloning, enabling mutable consumption pattern in usecase"
  - "CreateUserPayload uses into_parts() to destructure into owned (handle, name, role) tuple, avoiding clone when building User struct"
metrics:
  duration: "~10 min"
  completed: "2026-03-25"
  tasks: 2
  files_modified: 15
---

# Quick Task 260325-1um: Make Payload Fields Private, Add Constructors and Getters

**One-liner:** Enforced encapsulation on all 7 user payload structs via private fields, `new()` constructors, getter/take methods, and `into_parts()` decomposition.

## What Was Done

All 7 payload structs in `services/user/src/payload/user.rs` had their fields made private. The refactor added:

- `pub fn new(...)` constructors on every struct
- Getter methods on structs that lacked them (`CreateUserPayload`, `UpdateUserPayload`)
- `CreateUserPayload::into_parts(self) -> (String, String, UserRole)` for zero-clone destructuring in the create_user usecase
- `UpdateUserPayload::take_handle(&mut self) -> Option<String>` and `take_name(&mut self) -> Option<String>` for ownership-transferring field access

All 15 call-site files (7 rpc handlers + 7 usecase files + payload module tests) were updated to use constructors and getter methods exclusively. No struct literal construction exists outside of the payload module tests (which also use `new()`).

## Tasks

| # | Name | Status | Commit |
|---|------|--------|--------|
| 1 | Make payload fields private, add new() constructors and missing getters | Done | 191a1ea |
| 2 | Update all rpc handlers and usecase call sites | Done | 191a1ea |

Both tasks committed atomically in one commit since Task 1 leaves call sites broken until Task 2 completes — they form a single coherent change.

## Decisions Made

1. **`into_parts()` over cloning in create_user**: The usecase was doing `handle: payload.handle` (move). With private fields, `into_parts(self)` preserves zero-allocation semantics by destructuring the payload entirely. Alternative (`handle()` returning `&str` + `.to_string()`) would add a clone.

2. **`take_handle`/`take_name` over `Option<&str>` getters in update_user**: The usecase assigns `user.handle = handle` (owned String required). `take_*` methods use `Option::take` which zeros the field in-place and returns the `Option<String>` without cloning. Requires `mut payload` at the usecase callsite.

3. **`tracing::instrument` field syntax**: Changed `%payload.field` to `%payload.field()` in all usecase `#[instrument]` attributes. The `%` Display format works identically with method calls as with field access.

## Deviations from Plan

None — plan executed exactly as written. The two tasks were committed together (single commit) because Task 1 alone produces compile errors in call sites — this is an implementation detail, not a plan deviation. The commit covers all 15 files as planned.

## Verification

```
cargo test -p user  — 37 tests pass (8 integration, 6 service, 23 unit)
cargo clippy -p user — no warnings
```

Zero `pub` keywords on payload struct fields. All call sites use `Payload::new(...)` or getter/take methods.

## Known Stubs

None.

## Self-Check: PASSED

- All 15 modified files verified present and committed at 191a1ea
- `cargo test -p user` output confirmed: 37 tests pass, 0 failed
- `cargo clippy -p user` output confirmed: no warnings
