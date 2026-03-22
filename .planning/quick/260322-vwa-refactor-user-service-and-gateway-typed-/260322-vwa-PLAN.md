---
phase: quick
plan: 260322-vwa
type: execute
wave: 1
depends_on: []
files_modified:
  # Shared constants
  - crates/madome-common/src/lib.rs
  - crates/madome-common/src/headers.rs
  # User service — From impl + handler renames + typed headers
  - services/user/src/adapter/postgres/user_repository.rs
  - services/user/src/app/handler/mod.rs
  - services/user/src/app/handler/create_user.rs
  - services/user/src/app/handler/get_user.rs
  - services/user/src/app/handler/list_users.rs
  - services/user/src/app/handler/update_user.rs
  - services/user/src/app/handler/deactivate_user.rs
  - services/user/src/app/handler/activate_user.rs
  - services/user/src/app/handler/change_role.rs
  # Gateway — middleware + typed headers
  - services/gateway/src/lib.rs
  - services/gateway/src/middleware/mod.rs
  - services/gateway/src/middleware/caller_context.rs
  - services/gateway/src/routes/users.rs
  - services/gateway/src/routes/health.rs
autonomous: true
requirements: []

must_haves:
  truths:
    - "All hardcoded header strings replaced with typed constants from madome-common"
    - "classify_db_err() replaced with From<DbErr> for RepositoryError, call sites use ? operator"
    - "inject_caller_context() replaced with axum middleware that extracts CallerContext into request extensions"
    - "Handler files named by full domain action (create_user.rs not create.rs), lifecycle.rs split into 3 files"
  artifacts:
    - path: "crates/madome-common/src/headers.rs"
      provides: "Shared header name constants"
      contains: "X_CALLER_ID"
    - path: "services/gateway/src/middleware/caller_context.rs"
      provides: "Axum middleware for CallerContext extraction"
      contains: "CallerContext"
    - path: "services/user/src/app/handler/create_user.rs"
      provides: "Renamed handler file"
    - path: "services/user/src/app/handler/deactivate_user.rs"
      provides: "Split from lifecycle.rs"
  key_links:
    - from: "services/gateway/src/routes/users.rs"
      to: "crates/madome-common/src/headers.rs"
      via: "use madome_common::headers"
      pattern: "madome_common::headers"
    - from: "services/user/src/app/handler/mod.rs"
      to: "crates/madome-common/src/headers.rs"
      via: "use madome_common::headers"
      pattern: "madome_common::headers"
---

<objective>
Refactor user service and gateway to align with newly established coding conventions:
1. Replace hardcoded header string literals with typed constants (rust-conventions.md)
2. Convert classify_db_err() to From<DbErr> for RepositoryError (rust-conventions.md)
3. Convert inject_caller_context() to axum middleware (rust-request-handling.md)
4. Rename handler files from create.rs to create_user.rs pattern (rust-request-handling.md)

Purpose: Enforce consistent coding patterns before Phase 03 (auth) adds more services and handlers.
Output: Cleaner codebase with convention-compliant patterns.
</objective>

<execution_context>
@~/.claude/get-shit-done/workflows/execute-plan.md
@~/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@CLAUDE.md
@.claude/rules/rust-conventions.md
@.claude/rules/rust-request-handling.md
@services/user/src/adapter/postgres/user_repository.rs
@services/user/src/app/handler/mod.rs
@services/user/src/app/handler/create.rs
@services/user/src/app/handler/get.rs
@services/user/src/app/handler/list.rs
@services/user/src/app/handler/update.rs
@services/user/src/app/handler/lifecycle.rs
@services/gateway/src/routes/users.rs
@services/gateway/src/routes/health.rs
@services/gateway/src/routes/mod.rs
@services/gateway/src/lib.rs
@crates/madome-common/src/lib.rs
</context>

<tasks>

<task type="auto">
  <name>Task 1: Shared constants, From impl, and handler file renames in user service</name>
  <files>
    crates/madome-common/src/headers.rs
    crates/madome-common/src/lib.rs
    crates/madome-common/Cargo.toml
    services/user/src/adapter/postgres/user_repository.rs
    services/user/src/app/handler/mod.rs
    services/user/src/app/handler/create_user.rs (renamed from create.rs)
    services/user/src/app/handler/get_user.rs (renamed from get.rs)
    services/user/src/app/handler/list_users.rs (renamed from list.rs)
    services/user/src/app/handler/update_user.rs (renamed from update.rs)
    services/user/src/app/handler/deactivate_user.rs (split from lifecycle.rs)
    services/user/src/app/handler/activate_user.rs (split from lifecycle.rs)
    services/user/src/app/handler/change_role.rs (split from lifecycle.rs)
  </files>
  <action>
**A) Create shared header constants module in madome-common.**

Create `crates/madome-common/src/headers.rs` with `&'static str` constants:
```rust
/// Custom header/metadata key: caller's user ID (UUID).
pub const X_CALLER_ID: &str = "x-caller-id";
/// Custom header/metadata key: caller's role (user/admin/owner).
pub const X_CALLER_ROLE: &str = "x-caller-role";
/// Custom header/metadata key: request trace ID (UUIDv7).
pub const X_REQUEST_ID: &str = "x-request-id";
/// Custom header/metadata key: opaque pagination cursor.
pub const X_NEXT_CURSOR: &str = "x-next-cursor";
```

Add `pub mod headers;` to `crates/madome-common/src/lib.rs`.

madome-common has no http/tonic deps so use plain `&'static str`. Both `axum::http::HeaderName::from_static()` and `tonic::metadata::MetadataMap::insert()` accept `&str`, so this works universally.

**B) Convert classify_db_err() to From<DbErr> for RepositoryError.**

In `services/user/src/adapter/postgres/user_repository.rs`:
- Remove the standalone `fn classify_db_err(err: DbErr) -> RepositoryError` function.
- Add `impl From<sea_orm::DbErr> for RepositoryError` with the same logic (check for "duplicate key"/"unique constraint"/"23505" -> UniqueViolation, else Database).
- Place the `From` impl in `services/user/src/domain/error/repository_error.rs` (where RepositoryError is defined) since that's the idiomatic location for error conversions. Add `use sea_orm::DbErr;` there.
- Update all call sites in `user_repository.rs`: replace `.map_err(classify_db_err)?` with just `?`, and replace `.map_err(|e| RepositoryError::Database(e.to_string()))?` with just `?` since the From impl handles all DbErr cases.

**C) Rename handler files per convention and split lifecycle.rs.**

Use `git mv` for each rename to preserve history:
- `git mv services/user/src/app/handler/create.rs services/user/src/app/handler/create_user.rs`
- `git mv services/user/src/app/handler/get.rs services/user/src/app/handler/get_user.rs`
- `git mv services/user/src/app/handler/list.rs services/user/src/app/handler/list_users.rs`
- `git mv services/user/src/app/handler/update.rs services/user/src/app/handler/update_user.rs`
- Delete `services/user/src/app/handler/lifecycle.rs` and create three new files:
  - `deactivate_user.rs` — contains `handle_deactivate` function (renamed to `handle`)
  - `activate_user.rs` — contains `handle_activate` function (renamed to `handle`)
  - `change_role.rs` — contains `handle_change_role` function (renamed to `handle`)

Each split file gets only its own imports (not all of lifecycle.rs's imports).

**D) Update handler/mod.rs.**

Change module declarations:
```rust
pub mod activate_user;
pub mod change_role;
pub mod create_user;
pub mod deactivate_user;
pub mod get_user;
pub mod list_users;
pub mod update_user;
```

Update UserService impl dispatch calls:
- `create::handle` -> `create_user::handle`
- `get::handle_get` -> `get_user::handle_get`
- `get::handle_get_by_handle` -> `get_user::handle_get_by_handle`
- `list::handle` -> `list_users::handle`
- `update::handle` -> `update_user::handle`
- `lifecycle::handle_deactivate` -> `deactivate_user::handle`
- `lifecycle::handle_activate` -> `activate_user::handle`
- `lifecycle::handle_change_role` -> `change_role::handle`

**E) Replace hardcoded header strings in user service handler/mod.rs.**

In `extract_caller_context` and `try_extract_caller_role`, replace:
- `"x-caller-id"` -> `madome_common::headers::X_CALLER_ID`
- `"x-caller-role"` -> `madome_common::headers::X_CALLER_ROLE`

Add `use madome_common::headers;` at the top.
  </action>
  <verify>
    <automated>cd /Users/syr/Developments/madome && cargo build -p user && cargo test -p user && cargo clippy -p user -- -D warnings</automated>
  </verify>
  <done>
    - classify_db_err function removed, replaced by From impl in repository_error.rs
    - All handler files renamed to entity-qualified names
    - lifecycle.rs split into deactivate_user.rs, activate_user.rs, change_role.rs (one handler per file)
    - No hardcoded "x-caller-id"/"x-caller-role" strings remain in user service (all use madome_common::headers constants)
    - All existing tests pass unchanged
  </done>
</task>

<task type="auto">
  <name>Task 2: Gateway caller context middleware and typed header constants</name>
  <files>
    services/gateway/src/lib.rs
    services/gateway/src/middleware/mod.rs
    services/gateway/src/middleware/caller_context.rs
    services/gateway/src/routes/users.rs
    services/gateway/src/routes/health.rs
  </files>
  <action>
**A) Create axum middleware for caller context extraction.**

Create `services/gateway/src/middleware/` directory with `mod.rs` and `caller_context.rs`.

In `caller_context.rs`, define:

```rust
/// Validated caller identity extracted from HTTP request headers.
#[derive(Clone, Debug)]
pub struct CallerContext {
    pub caller_id: String,
    pub caller_role: String,
}

impl CallerContext {
    /// Inject caller context into tonic gRPC request metadata.
    pub fn inject_into<T>(&self, request: &mut tonic::Request<T>) {
        use madome_common::headers;
        let metadata = request.metadata_mut();
        if let Ok(val) = self.caller_id.parse() {
            metadata.insert(headers::X_CALLER_ID, val);
        }
        if let Ok(val) = self.caller_role.parse() {
            metadata.insert(headers::X_CALLER_ROLE, val);
        }
    }
}
```

Create an axum middleware function using `axum::middleware::from_fn`:

```rust
pub async fn extract_caller_context(
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, AppError> {
    let headers = request.headers();

    // Extract x-caller-id if present (validated as parseable string)
    let caller_id = headers
        .get(madome_common::headers::X_CALLER_ID)
        .map(|v| v.to_str()
            .map_err(|_| AppError::BadRequest("invalid x-caller-id header".to_string())))
        .transpose()?
        .map(|s| s.to_string());

    let caller_role = headers
        .get(madome_common::headers::X_CALLER_ROLE)
        .map(|v| v.to_str()
            .map_err(|_| AppError::BadRequest("invalid x-caller-role header".to_string())))
        .transpose()?
        .map(|s| s.to_string());

    if let (Some(id), Some(role)) = (caller_id, caller_role) {
        request.extensions_mut().insert(CallerContext {
            caller_id: id,
            caller_role: role,
        });
    }

    Ok(next.run(request).await)
}
```

The middleware is permissive: it inserts CallerContext into extensions only when both headers are present. Handlers that require it use `Extension<CallerContext>` (returns 500 if missing, which is correct for pre-auth-middleware routes). Handlers that don't need it ignore extensions.

In `middleware/mod.rs`:
```rust
pub mod caller_context;
pub use caller_context::CallerContext;
```

Update `services/gateway/src/lib.rs`:
```rust
pub mod middleware;
pub mod routes;
pub mod state;
```

**B) Apply middleware to user routes in gateway router.**

In `services/gateway/src/routes/mod.rs`, apply the middleware to user routes:

```rust
use axum::middleware as axum_mw;
use crate::middleware::caller_context;

fn user_routes() -> Router<AppState> {
    let me_routes = Router::new()
        .route("/@me", get(users::get_me).patch(users::update_me));

    let admin_routes = Router::new()
        .route("/", get(users::list_users))
        .route("/{id}", get(users::get_user))
        .route("/{id}/role", patch(users::change_role))
        .route("/{id}/deactivate", post(users::deactivate_user))
        .route("/{id}/activate", post(users::activate_user));

    Router::new()
        .merge(me_routes)
        .merge(admin_routes)
        .layer(axum_mw::from_fn(caller_context::extract_caller_context))
}
```

**C) Refactor gateway user route handlers to use CallerContext from extensions.**

In `services/gateway/src/routes/users.rs`:

1. Remove the `inject_caller_context` helper function entirely.

2. For handlers that need caller_id (get_me, update_me): change signature to accept `Extension(caller_ctx): Extension<CallerContext>` (from axum). Use `caller_ctx.caller_id` instead of parsing from raw headers. Use `caller_ctx.inject_into(&mut request)` to forward to gRPC.

3. For handlers that just forward context (list_users, get_user, change_role, deactivate_user, activate_user): accept `caller_ctx: Option<Extension<CallerContext>>` and call `caller_ctx.inject_into(&mut request)` if present.

4. Specifically for `get_me` and `update_me`: remove the raw header parsing for `x-caller-id`. Instead, get the caller_id from `CallerContext`. If no CallerContext in extensions, return `AppError::Unauthorized("missing caller context")`.

5. For `list_users` response: replace `"content-type"` with `axum::http::header::CONTENT_TYPE` (framework-provided typed constant), and replace `"x-next-cursor"` with `madome_common::headers::X_NEXT_CURSOR`.

6. Remove `headers: HeaderMap` parameter from handler signatures that no longer need raw header access.

**D) Replace hardcoded header strings in gateway health routes.**

In `services/gateway/src/routes/health.rs`, replace:
- `"x-request-id"` -> `madome_common::headers::X_REQUEST_ID`

Add `use madome_common::headers;` at the top.
  </action>
  <verify>
    <automated>cd /Users/syr/Developments/madome && cargo build -p gateway && cargo test -p gateway && cargo clippy -p gateway -- -D warnings</automated>
  </verify>
  <done>
    - inject_caller_context() function removed from routes/users.rs
    - CallerContext middleware extracts and validates caller headers, stores in request extensions
    - All user route handlers consume CallerContext from extensions, not raw HeaderMap
    - No hardcoded header strings remain in gateway (all use madome_common::headers or axum::http::header constants)
    - Gateway health routes use typed X_REQUEST_ID constant
    - All existing tests pass unchanged
  </done>
</task>

</tasks>

<verification>
```bash
# Full workspace build and test
cd /Users/syr/Developments/madome && cargo build --workspace && cargo test --workspace && cargo clippy --workspace -- -D warnings

# Verify no hardcoded header strings remain
grep -rn '"x-caller-id"\|"x-caller-role"\|"x-request-id"\|"x-next-cursor"' services/gateway/src/ services/user/src/
# Expected: no matches (only test files may retain string literals for metadata insertion)

# Verify old files are gone
ls services/user/src/app/handler/create.rs services/user/src/app/handler/get.rs services/user/src/app/handler/list.rs services/user/src/app/handler/update.rs services/user/src/app/handler/lifecycle.rs 2>&1
# Expected: all "No such file or directory"

# Verify classify_db_err function is gone
grep -rn 'fn classify_db_err' services/user/src/
# Expected: no matches

# Verify inject_caller_context function is gone
grep -rn 'fn inject_caller_context' services/gateway/src/
# Expected: no matches
```
</verification>

<success_criteria>
- Zero hardcoded header string literals in production code (gateway + user service src/)
- classify_db_err replaced with From<DbErr> for RepositoryError; call sites use ? operator
- inject_caller_context replaced with axum middleware + CallerContext in extensions
- Handler files follow entity-qualified naming (create_user.rs, not create.rs)
- lifecycle.rs split into one-handler-per-file (deactivate_user.rs, activate_user.rs, change_role.rs)
- Full workspace builds, all tests pass, clippy clean
</success_criteria>

<output>
After completion, create `.planning/quick/260322-vwa-refactor-user-service-and-gateway-typed-/260322-vwa-SUMMARY.md`
</output>
