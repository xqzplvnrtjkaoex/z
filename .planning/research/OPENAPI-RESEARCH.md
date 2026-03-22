# Static OpenAPI Documentation Generation - Research

**Researched:** 2026-03-23
**Domain:** Rust / axum / OpenAPI specification generation
**Confidence:** HIGH

## Summary

The Rust ecosystem has a clear winner for code-first OpenAPI documentation: **utoipa**. With 22M+ downloads, active maintenance (v5.4.0, June 2025), and first-class axum support, it is the standard choice. The alternatives (aide, paperclip, oasgen) are either less mature, less downloaded, or lack the static generation ergonomics needed here.

The critical constraint -- no OpenAPI code at runtime -- is fully achievable with utoipa. The `#[utoipa::path]` and `#[derive(ToSchema)]` macros are purely compile-time annotations that generate no runtime code beyond trait implementations. The OpenAPI spec is materialized only when `OpenApi::openapi()` is called, which we isolate in a dedicated binary (`gen-openapi`) that never ships with the Gateway.

**Primary recommendation:** Use `utoipa` v5 with `utoipa-gen` for compile-time annotations, and a separate `[[bin]]` target (`gen-openapi`) in the gateway crate to write the spec to `docs/openapi.json`. Add a `cargo test` assertion that the committed file matches the generated output.

## Recommended Tool

**utoipa v5.4.0** with the following ecosystem crates:

| Crate | Version | Purpose |
|-------|---------|---------|
| `utoipa` | 5.4.0 | Core: `#[derive(OpenApi)]`, `#[derive(ToSchema)]`, `#[derive(IntoParams)]` |
| `utoipa-gen` | 5.3.1 | Proc macros (auto-pulled by `utoipa` with `macros` feature) |
| `utoipa-axum` | 0.2.0 | **NOT recommended** -- see rationale below |

### Why NOT utoipa-axum (OpenApiRouter)

`utoipa-axum` provides `OpenApiRouter` which combines route registration with spec generation. This is designed for runtime serving of the spec alongside the API. For this project:

1. It would interleave OpenAPI concerns into the Gateway's router construction, violating the "no OpenAPI at runtime" constraint.
2. It requires restructuring `create_router()` to use `OpenApiRouter` instead of `axum::Router`.
3. The standard `#[derive(OpenApi)]` + `#[utoipa::path]` approach achieves the same result without touching the router.

Instead, use the **manual registration pattern**: annotate handlers with `#[utoipa::path]`, list them in `#[derive(OpenApi)]`'s `paths(...)`, and generate the spec in a separate binary.

### Cargo.toml Configuration

```toml
# In services/gateway/Cargo.toml

[dependencies]
utoipa = { version = "5", features = ["macros", "uuid", "chrono", "axum_extras"] }

[[bin]]
name = "gen-openapi"
path = "src/gen_openapi.rs"
```

**Feature rationale:**
- `macros` (default): Enables `#[derive(OpenApi)]`, `#[utoipa::path]`, `#[derive(ToSchema)]`, `#[derive(IntoParams)]`
- `uuid`: Maps `uuid::Uuid` to OpenAPI `string` with `format: uuid` automatically
- `chrono`: Maps `chrono::DateTime`, `NaiveDateTime`, etc. to `string` with `format: date-time`
- `axum_extras`: Auto-resolves `parameter_in` from axum's `Path<>` and `Query<>` extractors (no manual `in = Path` needed in `#[utoipa::path]` params)

**Optional features to consider:**
- `yaml`: Enables `to_yaml()` via `serde_norway` (a serde_yaml alternative). Only needed if YAML output is desired alongside JSON.
- `preserve_order`: Preserves field insertion order in schemas (uses `IndexMap`). Recommended for deterministic output in CI diff checks.

## Static Generation Pattern

### Architecture

```
services/gateway/
  src/
    main.rs              # Gateway binary -- NO utoipa runtime code
    lib.rs               # Shared: routes, state, middleware
    routes/
      mod.rs             # create_router() -- unchanged
      users.rs           # handlers + #[utoipa::path] annotations
      health.rs          # handlers + #[utoipa::path] annotations
    gen_openapi.rs       # Separate binary: generates openapi.json
docs/
  openapi.json           # Committed, CI-validated
```

### The gen-openapi Binary

```rust
// services/gateway/src/gen_openapi.rs

use gateway::openapi::ApiDoc;  // Re-export from lib.rs
use utoipa::OpenApi;

fn main() {
    let spec = ApiDoc::openapi()
        .to_pretty_json()
        .expect("Failed to serialize OpenAPI spec");

    let out_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "docs/openapi.json".to_string());

    std::fs::write(&out_path, &spec)
        .unwrap_or_else(|e| panic!("Failed to write {out_path}: {e}"));

    eprintln!("OpenAPI spec written to {out_path}");
}
```

### The OpenApi Definition (in lib.rs or a dedicated module)

```rust
// services/gateway/src/openapi.rs

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Madome Gateway API",
        description = "REST API for the Madome platform",
        version = "0.1.0"
    ),
    paths(
        crate::routes::health::gateway_health,
        crate::routes::health::service_health,
        crate::routes::users::get_me,
        crate::routes::users::update_me,
        crate::routes::users::list_users,
        crate::routes::users::get_user,
        crate::routes::users::change_role,
        crate::routes::users::deactivate_user,
        crate::routes::users::activate_user,
    ),
    components(schemas(
        crate::routes::health::GatewayHealthResponse,
        crate::routes::health::HealthResponse,
        crate::routes::health::ServiceStatuses,
        crate::routes::users::UserJson,
        crate::routes::users::UpdateMeBody,
        crate::routes::users::ListUsersQuery,
        crate::routes::users::ChangeRoleBody,
        madome_core::error::ErrorBody,
    )),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "users", description = "User management endpoints")
    ),
    servers(
        (url = "http://localhost:3000", description = "Local development")
    )
)]
pub struct ApiDoc;
```

### Running

```bash
# Generate the spec
cargo run --bin gen-openapi

# Or with custom output path
cargo run --bin gen-openapi -- docs/openapi.yaml

# Add to justfile
just gen-openapi
```

### Key Insight: No Runtime Overhead

The `#[utoipa::path]` and `#[derive(ToSchema)]` macros implement traits (`__path_handler_name`, `ToSchema`) at compile time. These trait implementations exist in the binary, but they are **dead code** in the Gateway binary since nothing calls them. The Rust compiler/linker will strip them in release builds. The `gen-openapi` binary is the only thing that calls `ApiDoc::openapi()`.

For extra safety, the `openapi.rs` module with the `ApiDoc` struct can be gated behind a feature flag:

```toml
[features]
openapi = ["utoipa"]  # Only included when generating docs

[dependencies]
utoipa = { version = "5", features = ["macros", "uuid", "chrono", "axum_extras"], optional = true }
```

However, this adds complexity (conditional compilation on every `#[utoipa::path]` annotation). The simpler approach -- always including utoipa but only calling `openapi()` in the gen binary -- is standard practice and adds negligible binary size.

## Derive Macro Usage

### Annotating Response Types with `#[derive(ToSchema)]`

```rust
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct UserJson {
    /// Unique user identifier (UUIDv7)
    #[schema(example = "01912345-6789-7abc-def0-123456789abc")]
    pub id: String,
    /// User's unique handle
    #[schema(example = "alice")]
    pub handle: String,
    /// User's display name
    #[schema(example = "Alice")]
    pub name: String,
    /// User's role: "user", "admin", or "owner"
    #[schema(example = "user")]
    pub role: String,
    /// Whether the user account is active
    pub is_active: bool,
    /// Account creation timestamp (RFC 3339)
    #[schema(example = "2025-01-15T10:30:00Z")]
    pub created_at: String,
    /// Last update timestamp (RFC 3339)
    #[schema(example = "2025-06-01T14:22:00Z")]
    pub updated_at: String,
}
```

### Annotating Request Types

```rust
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, ToSchema)]
pub struct UpdateMeBody {
    /// New handle (must be unique, 3-30 chars, lowercase alphanumeric + hyphens)
    pub handle: Option<String>,
    /// New display name
    pub name: Option<String>,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListUsersQuery {
    /// Maximum number of users to return
    #[param(minimum = 1, maximum = 100, example = 25)]
    pub limit: Option<i32>,
    /// Cursor for pagination (from x-next-cursor header)
    pub cursor: Option<String>,
    /// Include deactivated users in results
    #[serde(rename = "include-inactive")]
    #[param(rename = "include-inactive")]
    pub include_inactive: Option<bool>,
}
```

**Note on serde compatibility:** utoipa has partial serde attribute support. `#[serde(rename = "...")]` works on fields, and `#[serde(rename_all = "...")]` works on containers. However, the `IntoParams` derive needs a matching `#[param(rename = "...")]` because `IntoParams` processes serde rename attributes for serialization but may need the param-level rename for the OpenAPI parameter name.

### Annotating Error Types

```rust
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    /// Error code (e.g., "not_found", "bad_request")
    #[schema(example = "not_found")]
    pub error: String,
    /// Human-readable error message
    #[schema(example = "not found: user with id 01912345 does not exist")]
    pub message: String,
}
```

### Annotating Handlers with `#[utoipa::path]`

```rust
/// Get the authenticated user's profile
#[utoipa::path(
    get,
    path = "/v1/users/@me",
    tag = "users",
    responses(
        (status = 200, description = "User profile", body = UserJson),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
    security(("bearer" = []))
)]
pub async fn get_me(
    State(state): State<AppState>,
    Extension(caller_ctx): Extension<CallerContext>,
) -> Result<impl IntoResponse, AppError> {
    // ... handler body unchanged ...
}

/// List users (admin only)
///
/// Returns a paginated list of users. Use the `x-next-cursor` response header
/// for subsequent page requests.
#[utoipa::path(
    get,
    path = "/v1/users",
    tag = "users",
    params(ListUsersQuery),
    responses(
        (status = 200, description = "User list", body = Vec<UserJson>,
         headers(("x-next-cursor" = Option<String>, description = "Cursor for next page"))),
        (status = 401, description = "Not authenticated", body = ErrorBody),
        (status = 403, description = "Forbidden (admin required)", body = ErrorBody),
    ),
    security(("bearer" = []))
)]
pub async fn list_users(
    State(state): State<AppState>,
    Extension(caller_ctx): Extension<CallerContext>,
    Query(query): Query<ListUsersQuery>,
) -> Result<impl IntoResponse, AppError> {
    // ... handler body unchanged ...
}

/// Deactivate a user account (admin only)
#[utoipa::path(
    post,
    path = "/v1/users/{id}/deactivate",
    tag = "users",
    params(("id" = String, Path, description = "User ID (UUIDv7)")),
    responses(
        (status = 200, description = "User deactivated", body = UserJson),
        (status = 401, description = "Not authenticated", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "User not found", body = ErrorBody),
    ),
    security(("bearer" = []))
)]
pub async fn deactivate_user(
    State(state): State<AppState>,
    Extension(caller_ctx): Extension<CallerContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // ... handler body unchanged ...
}
```

### Key Path Annotation Rules

1. **`path` must be the full path** as it appears in the OpenAPI spec, including any prefix from `nest()`. For nested routes under `/v1`, the path is `/v1/users/{id}`, not just `/{id}`.
2. **Path parameters use `{name}` syntax** (OpenAPI style), not `/:name` (axum style).
3. **`tag` groups related endpoints** in the generated spec. Use consistent tag names.
4. **Doc comments become `summary` (first line) and `description` (rest)**. No need to duplicate in the macro.
5. **`security(("bearer" = []))` references a security scheme** that must be defined in the `#[derive(OpenApi)]` modifiers or security attributes.

## CI Integration

### Pattern: Test-Based Spec Validation

Add a test in the gateway crate that generates the spec and compares it to the committed file:

```rust
// services/gateway/tests/openapi_sync.rs

#[test]
fn openapi_spec_is_up_to_date() {
    use gateway::openapi::ApiDoc;
    use utoipa::OpenApi;

    let generated = ApiDoc::openapi()
        .to_pretty_json()
        .expect("Failed to generate OpenAPI spec");

    let committed_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/openapi.json");
    let committed = std::fs::read_to_string(committed_path)
        .expect("docs/openapi.json not found -- run `cargo run --bin gen-openapi` first");

    assert_eq!(
        generated.trim(),
        committed.trim(),
        "\n\nOpenAPI spec is out of date!\n\
         Run `cargo run --bin gen-openapi` and commit the result.\n"
    );
}
```

### Why a Test (Not a build.rs)

- **build.rs has limitations:** It runs during compilation, before the full crate is built. Since `#[derive(OpenApi)]` depends on all handler paths being compiled first, you cannot call `ApiDoc::openapi()` from build.rs.
- **A test binary runs after full compilation**, so all derive macros have been expanded and the `ApiDoc::openapi()` method is available.
- **Tests integrate naturally with CI** -- `cargo test` already runs in the pipeline.

### Justfile Recipe

```just
# Generate OpenAPI spec
gen-openapi:
    cargo run --bin gen-openapi
```

### CI Workflow Addition

The existing CI runs `cargo test --workspace` on `dev` push. The `openapi_sync` test will automatically fail if someone adds/changes a handler annotation but forgets to regenerate the spec. No additional CI step needed.

### Workflow

1. Developer adds/modifies `#[utoipa::path]` annotations
2. Developer runs `just gen-openapi` to regenerate `docs/openapi.json`
3. Developer commits both code changes and updated spec
4. CI runs `cargo test` which includes `openapi_sync` test
5. If spec is stale, CI fails with a clear message

## Alternatives Considered

| Crate | Version | Downloads | Last Updated | axum Support | Static Gen | Verdict |
|-------|---------|-----------|--------------|--------------|------------|---------|
| **utoipa** | 5.4.0 | 22M+ | Jun 2025 | First-class (`axum_extras` feature) | Yes (separate binary) | **Recommended** |
| aide | 0.16.0-alpha.3 | 1.5M | Mar 2026 | Yes (`aide::axum`) | Not documented | Still in alpha; uses `schemars` (extra dep); runtime-focused |
| paperclip | 0.9.5 | 787K | Mar 2025 | Partial (actix-web focused) | Possible | Poor axum support; actix-web first |
| oasgen | - | Low | - | Yes | Yes (env var trigger) | Low adoption; less documented |
| openapi-from-source | - | Low | - | Yes (static analysis) | Yes (CLI tool) | Interesting but no type safety; analyzes source text |

### Why Not aide

- **Alpha status:** v0.16.0-alpha.3 signals instability. API may change.
- **schemars dependency:** Requires all types to derive `schemars::JsonSchema` in addition to serde traits. This is a heavier annotation burden than utoipa's `ToSchema`.
- **Runtime-focused:** Documentation and examples center on serving the spec at runtime, not writing to a file.
- **Smaller ecosystem:** 1.5M downloads vs utoipa's 22M+.

### Why Not paperclip

- **actix-web focused:** axum support is secondary and less mature.
- **Different paradigm:** Uses a plugin-based approach that is more invasive.
- **Lower adoption:** 787K downloads.

## Common Pitfalls

### Pitfall 1: Path Mismatch Between Router and Annotations

**What goes wrong:** The `path` in `#[utoipa::path]` does not match the actual route registered in `Router::new().route(...)`. The spec says `/v1/users/{id}` but the router has `/{id}` (because it is nested under `/v1/users`).

**Why it happens:** utoipa annotations are independent of axum's router -- they do not read the nesting. The developer must manually ensure the full path in the annotation matches the actual URL.

**How to avoid:**
- Always use the full, absolute path in `#[utoipa::path(path = "/v1/users/{id}")]`.
- The `openapi_sync` test will catch spec drift, but path mismatches require manual review or E2E testing against the actual routes.
- Consider adding a comment next to each `#[utoipa::path]` noting the route structure.

**Warning signs:** Generated spec paths do not match curl/client requests.

### Pitfall 2: Forgetting to Register Paths in `#[derive(OpenApi)]`

**What goes wrong:** A handler has `#[utoipa::path]` but is not listed in `paths(...)` of `#[derive(OpenApi)]`. The endpoint is missing from the spec.

**Why it happens:** The `paths(...)` list is manually maintained -- it is not auto-discovered.

**How to avoid:** Whenever adding a new handler with `#[utoipa::path]`, immediately add it to the `paths(...)` list. Make this part of the "new endpoint" checklist.

**Warning signs:** `cargo build` succeeds but `gen-openapi` output does not include the new endpoint.

### Pitfall 3: `impl IntoResponse` Opacity

**What goes wrong:** Handlers return `Result<impl IntoResponse, AppError>`. utoipa cannot infer the response type from `impl IntoResponse` -- you must explicitly declare `body = UserJson` in the `responses(...)` attribute.

**Why it happens:** `impl IntoResponse` is an opaque return type. utoipa's proc macros cannot look through it to determine the actual response schema.

**How to avoid:** Always specify `body = TypeName` in every response tuple. This is required for any non-trivial handler.

### Pitfall 4: serde Rename Not Fully Mirrored

**What goes wrong:** A field uses `#[serde(rename = "include-inactive")]` but the OpenAPI spec shows `include_inactive` (the Rust field name).

**Why it happens:** utoipa supports serde rename attributes on `ToSchema` derives but `IntoParams` has partial support. The `#[param(rename = "include-inactive")]` attribute may be needed alongside the serde one.

**How to avoid:** For `IntoParams` structs with serde renames, add matching `#[param(rename = "...")]` attributes and verify in the generated spec.

### Pitfall 5: Stale Spec in CI

**What goes wrong:** Developer adds `#[utoipa::path]` annotations but forgets to run `just gen-openapi` before pushing.

**Why it happens:** The spec file is a generated artifact that must be manually regenerated.

**How to avoid:** The `openapi_sync` test catches this automatically. CI will fail with a clear error message telling the developer to regenerate.

### Pitfall 6: Feature Flag Interactions with Workspace

**What goes wrong:** utoipa's `uuid` or `chrono` features are not enabled, so `Uuid` or `DateTime` fields are not recognized and fail to derive `ToSchema`.

**Why it happens:** utoipa needs feature flags to know about third-party types.

**How to avoid:** Enable `uuid` and `chrono` features explicitly in the utoipa dependency. Since the workspace already uses these types, the features must be active.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| utoipa v4 (`#[api_doc]`) | utoipa v5 (`#[utoipa::path]`) | 2024 | Simplified macro syntax, better axum support |
| Manual OpenAPI YAML | Code-first generation | N/A | Spec guaranteed to match code types |
| Runtime spec serving | Static file generation + CI validation | Emerging pattern | Zero runtime overhead, spec in version control |
| `serde_yaml` | `serde_norway` (in utoipa) | utoipa v5 | `serde_yaml` is unmaintained; utoipa uses `serde_norway` fork for YAML |

## Open Questions

1. **JSON vs YAML output format**
   - What we know: `to_pretty_json()` works out of the box. `to_yaml()` requires the `yaml` feature and pulls in `serde_norway`.
   - What's unclear: User preference for output format.
   - Recommendation: Default to JSON (`docs/openapi.json`). JSON is universally supported by OpenAPI tools, does not need an extra dependency, and produces deterministic output for CI diffing.

2. **Security scheme definition**
   - What we know: Auth is Phase 03 (not yet implemented). The `security(("bearer" = []))` in path annotations references a security scheme that must be defined.
   - What's unclear: Whether to define security schemes now (placeholder) or defer until auth is implemented.
   - Recommendation: Define a placeholder `Bearer` security scheme in the `ApiDoc` struct using a `Modify` trait implementation. This allows documenting auth requirements on endpoints now without the auth implementation existing.

3. **Scope of `utoipa` dependency**
   - What we know: utoipa can be a normal dependency or an optional one behind a feature flag.
   - What's unclear: Whether the minor binary size overhead of always-included utoipa trait impls matters.
   - Recommendation: Start with utoipa as a normal dependency (simpler). If binary size becomes a concern, refactor to an optional feature later. The trait implementations are dead code in the Gateway binary and will be stripped by the linker in release builds.

## Sources

### Primary (HIGH confidence)
- [utoipa docs.rs](https://docs.rs/utoipa/latest/utoipa/) - Core API, derive macros, features
- [utoipa OpenApi derive](https://docs.rs/utoipa/latest/utoipa/derive.OpenApi.html) - Full `#[derive(OpenApi)]` attributes
- [utoipa path attribute](https://docs.rs/utoipa/latest/utoipa/attr.path.html) - Full `#[utoipa::path]` documentation
- [utoipa OpenApi struct](https://docs.rs/utoipa/latest/utoipa/openapi/struct.OpenApi.html) - `to_json()`, `to_pretty_json()`, `to_yaml()`, `merge()`, `nest()` methods
- [utoipa-axum docs.rs](https://docs.rs/utoipa-axum/latest/utoipa_axum/) - OpenApiRouter, routes! macro
- [utoipa GitHub](https://github.com/juhaku/utoipa) - README, examples, ecosystem overview
- [utoipa GitHub issue #214](https://github.com/juhaku/utoipa/issues/214) - Static file generation discussion and recommended pattern

### Secondary (MEDIUM confidence)
- [crates.io utoipa](https://crates.io/crates/utoipa) - Version 5.4.0, 22M+ downloads
- [crates.io utoipa-axum](https://crates.io/crates/utoipa-axum) - Version 0.2.0
- [crates.io aide](https://crates.io/crates/aide) - Version 0.16.0-alpha.3, 1.5M downloads
- [crates.io paperclip](https://crates.io/crates/paperclip) - Version 0.9.5, 787K downloads
- [Identeco blog - Auto-Generating & Validating OpenAPI Docs](https://identeco.de/en/blog/generating_and_validating_openapi_docs_in_rust/) - CI validation pattern with Schemathesis
- [Identeco example repo](https://github.com/Identeco/utoipa-blog-example-code) - gen_api binary + test validation pattern

### Tertiary (LOW confidence)
- [lib.rs utoipa features](https://lib.rs/crates/utoipa/features) - Feature flag listing (could not fully fetch)

## Metadata

**Confidence breakdown:**
- Standard stack (utoipa selection): HIGH - dominant crate, official docs verified, 22M+ downloads
- Architecture (separate binary pattern): HIGH - recommended by utoipa maintainer in issue #214, used in production examples
- Derive macro usage: HIGH - verified against docs.rs official documentation
- CI integration (test-based validation): HIGH - standard pattern, verified in Identeco example
- Pitfalls: MEDIUM - based on documentation analysis and community discussions, not personal production experience

**Research date:** 2026-03-23
**Valid until:** 2026-06-23 (stable ecosystem, utoipa v5 is mature)
