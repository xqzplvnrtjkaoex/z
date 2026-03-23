---
type: quick
autonomous: true
files_modified:
  - services/user/Cargo.toml
  - services/user/src/app/rpc/mod.rs
  - services/user/src/app/rpc/get_user.rs
  - services/user/src/app/rpc/create_user.rs
  - services/user/src/app/rpc/update_user.rs
  - services/user/src/app/rpc/deactivate_user.rs
  - services/user/src/app/rpc/activate_user.rs
  - services/user/src/app/rpc/change_role.rs
  - services/user/src/app/rpc/get_user_by_handle.rs
  - services/user/src/app/rpc/list_users.rs
---

<objective>
Refactor user service conversion functions to use standard `From`/`TryFrom` trait implementations instead of standalone functions, per the project's "Type Conversion via Standard Traits" rule in `.claude/rules/rust-conventions.md`.

Purpose: Align with project conventions — idiomatic Rust uses `From`/`TryFrom` for type conversions, enabling `.into()`, `?` operator, and `map_into()` patterns.
Output: All conversion functions replaced with trait impls, all callers updated.
</objective>

<context>
@.claude/rules/rust-conventions.md
@services/user/src/app/rpc/mod.rs
@services/user/src/domain/types/user.rs
@services/user/src/domain/types/role.rs
</context>

<tasks>

<task type="auto">
  <name>Task 1: Replace standalone conversion functions with From/TryFrom impls and update all callers</name>
  <files>
    services/user/Cargo.toml,
    services/user/src/app/rpc/mod.rs,
    services/user/src/app/rpc/get_user.rs,
    services/user/src/app/rpc/create_user.rs,
    services/user/src/app/rpc/update_user.rs,
    services/user/src/app/rpc/deactivate_user.rs,
    services/user/src/app/rpc/activate_user.rs,
    services/user/src/app/rpc/change_role.rs,
    services/user/src/app/rpc/get_user_by_handle.rs,
    services/user/src/app/rpc/list_users.rs
  </files>
  <action>
    **In `services/user/Cargo.toml`:**
    - Add `itertools = { workspace = true }` to `[dependencies]` (already in workspace Cargo.toml as `itertools = "0.14"`, not yet in user service).

    **In `services/user/src/app/rpc/mod.rs`:**

    1. Replace `fn domain_role_to_proto(role: UserRole) -> Role` with:
       ```rust
       impl From<UserRole> for Role {
           fn from(role: UserRole) -> Self {
               match role {
                   UserRole::User => Role::User,
                   UserRole::Admin => Role::Admin,
                   UserRole::Owner => Role::Owner,
               }
           }
       }
       ```

    2. Replace `fn user_to_response(user: &User) -> UserResponse` with:
       ```rust
       impl From<&User> for UserResponse {
           fn from(user: &User) -> Self {
               UserResponse {
                   id: user.id.to_string(),
                   handle: user.handle.clone(),
                   name: user.name.clone(),
                   role: Role::from(user.role) as i32,
                   is_active: user.is_active,
                   created_at: Some(Timestamp {
                       seconds: user.created_at.timestamp(),
                       nanos: user.created_at.timestamp_subsec_nanos() as i32,
                   }),
                   updated_at: Some(Timestamp {
                       seconds: user.updated_at.timestamp(),
                       nanos: user.updated_at.timestamp_subsec_nanos() as i32,
                   }),
               }
           }
       }
       ```
       Inside the `From<&User>` body, use `Role::from(user.role)` for the role conversion (the `From<UserRole> for Role` impl defined just above).

    3. Replace `fn proto_role_to_domain(role: i32) -> Result<UserRole, Status>` with `TryFrom`:
       ```rust
       /// Newtype for proto role `i32` values to enable `TryFrom` conversion to domain `UserRole`.
       pub(crate) struct ProtoRole(pub i32);

       impl TryFrom<ProtoRole> for UserRole {
           type Error = Status;

           fn try_from(value: ProtoRole) -> Result<Self, Self::Error> {
               match Role::try_from(value.0) {
                   Ok(Role::User) => Ok(UserRole::User),
                   Ok(Role::Admin) => Ok(UserRole::Admin),
                   Ok(Role::Owner) => Ok(UserRole::Owner),
                   Ok(Role::Unspecified) | Err(_) => {
                       Err(Status::invalid_argument("invalid role value"))
                   }
                }
           }
       }
       ```
       A newtype `ProtoRole(i32)` is needed because orphan rules prevent `impl TryFrom<i32> for UserRole` (neither type is local). The newtype is zero-cost and makes the conversion explicit at call sites.

    4. Remove the three standalone functions (`user_to_response`, `domain_role_to_proto`, `proto_role_to_domain`). Keep `CallerContext`, `extract_caller_context`, and `try_extract_caller_role` unchanged.

    5. Remove any doc comments on the standalone functions. The From/TryFrom impls are self-documenting per convention (no rustdoc on adapter-layer impls per rust-documentation.md rules). The `ProtoRole` newtype doc comment is acceptable since it explains WHY the newtype exists.

    **In caller files (7 files using `user_to_response`):**

    Replace `user_to_response(&user)` with `UserResponse::from(&user)` in:
    - `get_user.rs:21` — `Ok(Response::new(UserResponse::from(&user)))`
    - `create_user.rs:27` — `Ok(Response::new(UserResponse::from(&user)))`
    - `update_user.rs:31` — `Ok(Response::new(UserResponse::from(&user)))`
    - `deactivate_user.rs:32` — `Ok(Response::new(UserResponse::from(&user)))`
    - `activate_user.rs:32` — `Ok(Response::new(UserResponse::from(&user)))`
    - `change_role.rs:35` — `Ok(Response::new(UserResponse::from(&user)))`
    - `get_user_by_handle.rs:25` — `Ok(Response::new(UserResponse::from(&user)))`

    Remove `user_to_response` from each file's `use crate::app::rpc::{...}` import. The `From` trait is in the prelude so no new import needed, but the impl must be in scope (it is, since these files are submodules of `rpc`).

    **In `create_user.rs` and `change_role.rs` (callers of `proto_role_to_domain`):**

    - `create_user.rs:17` — Replace `proto_role_to_domain(req.role)?` with `UserRole::try_from(ProtoRole(req.role))?`
    - `change_role.rs:24` — Replace `proto_role_to_domain(req.new_role)?` with `UserRole::try_from(ProtoRole(req.new_role))?`
    - Update imports: remove `proto_role_to_domain`, add `ProtoRole` from `crate::app::rpc::ProtoRole`. Also add `use crate::domain::types::role::UserRole;` if not already imported.

    **In `list_users.rs` (uses `map` with function reference):**

    Replace:
    ```rust
    let user_responses: Vec<UserResponse> = users.iter().map(user_to_response).collect();
    ```
    With (per convention, using `map_into()`):
    ```rust
    let user_responses: Vec<UserResponse> = users.iter().map_into().collect();
    ```
    Add `use itertools::Itertools;` import. Remove `user_to_response` from `use crate::app::rpc::{...}` import.
  </action>
  <verify>
    <automated>cd /Users/syr/Developments/madome && cargo build -p user 2>&1 && cargo test -p user 2>&1 && cargo clippy -p user 2>&1</automated>
  </verify>
  <done>
    All three standalone conversion functions removed from mod.rs. `From<UserRole> for Role`, `From<&User> for UserResponse`, and `TryFrom<ProtoRole> for UserRole` trait impls in their place. All 10 caller sites updated. `cargo build`, `cargo test`, and `cargo clippy` pass for the user crate.
  </done>
</task>

</tasks>

<verification>
- `cargo build -p user` compiles without errors
- `cargo test -p user` — all existing tests pass (no behavior change, pure refactor)
- `cargo clippy -p user` — no warnings
- `grep -r "user_to_response\|domain_role_to_proto\|proto_role_to_domain" services/user/src/` returns no matches (standalone functions fully removed)
</verification>

<success_criteria>
- Zero standalone conversion functions remain in `services/user/src/app/rpc/mod.rs`
- `From<&User> for UserResponse`, `From<UserRole> for Role`, and `TryFrom<ProtoRole> for UserRole` trait impls exist
- All callers use `.into()`, `UserResponse::from(&user)`, `UserRole::try_from(ProtoRole(...))`, or `map_into()` as appropriate
- All tests pass, no clippy warnings
</success_criteria>
