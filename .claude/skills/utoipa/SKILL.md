---
name: utoipa
description: "CRITICAL: Use for utoipa OpenAPI documentation generation in Rust. Triggers on: utoipa, OpenAPI, swagger, #[utoipa::path], ToSchema, ToResponse, OpenApi derive, API docs, Scalar, swagger-ui, utoipa-axum, openapi, api-docs, API documentation"
version: "5"
---

# utoipa Crate Skill (v5)

## Quick Reference

```rust
use utoipa::{OpenApi, ToSchema, ToResponse};

#[derive(OpenApi)]
#[openapi(
    paths(create_user, get_user),
    components(schemas(CreateUserRequest, UserResponse)),
)]
struct ApiDoc;

// Serve: ApiDoc::openapi() returns the OpenAPI JSON
```

## Cargo.toml Setup

```toml
[dependencies]
utoipa = { version = "5", features = ["axum_extras"] }
utoipa-axum = "0.2"          # axum router integration
utoipa-scalar = "0.2"        # Scalar UI (recommended)
# OR
utoipa-swagger-ui = { version = "9", features = ["axum"] }
```

## #[derive(OpenApi)]

```rust
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::create_user,
        handlers::get_user,
        handlers::list_users,
    ),
    components(
        schemas(CreateUserRequest, UserResponse, ErrorResponse),
        responses(UserResponse, ErrorResponse),
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "users", description = "User management"),
        (name = "auth", description = "Authentication"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
struct ApiDoc;
```

## #[utoipa::path] — Documenting Handlers

```rust
/// Create a new user
#[utoipa::path(
    post,
    path = "/api/users",
    tag = "users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created", body = UserResponse),
        (status = 409, description = "Email already exists", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn create_user(Json(req): Json<CreateUserRequest>) -> impl IntoResponse {
    // ...
}
```

### Path parameters
```rust
#[utoipa::path(
    get,
    path = "/api/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID"),
    ),
    responses(
        (status = 200, body = UserResponse),
        (status = 404, body = ErrorResponse),
    )
)]
async fn get_user(Path(id): Path<Uuid>) -> impl IntoResponse { ... }
```

### Query parameters
```rust
#[utoipa::path(
    get,
    path = "/api/users",
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("per_page" = Option<u32>, Query, description = "Items per page"),
    ),
    responses((status = 200, body = Vec<UserResponse>))
)]
async fn list_users(Query(params): Query<ListParams>) -> impl IntoResponse { ... }
```

## ToSchema — Request/Response Types

```rust
#[derive(Serialize, Deserialize, ToSchema)]
struct CreateUserRequest {
    /// User's email address
    #[schema(example = "user@example.com", format = "email")]
    email: String,

    /// Display name (3-50 characters)
    #[schema(min_length = 3, max_length = 50)]
    name: String,

    /// User role
    role: UserRole,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
enum UserRole {
    Admin,
    Member,
    Guest,
}
```

### Schema attributes
```rust
#[schema(example = "value")]       // example value
#[schema(default = "default_fn")]  // default value
#[schema(format = "email")]        // format hint
#[schema(min_length = 1)]          // string constraints
#[schema(minimum = 0)]             // numeric constraints
#[schema(value_type = String)]     // override inferred type
#[schema(nullable)]                // mark as nullable
#[schema(read_only)]               // read-only field
#[schema(write_only)]              // write-only field
```

## ToResponse — Reusable Responses

```rust
#[derive(Serialize, ToSchema, ToResponse)]
#[response(description = "User details")]
struct UserResponse {
    id: Uuid,
    email: String,
    name: String,
}

#[derive(Serialize, ToSchema, ToResponse)]
#[response(description = "Error response")]
struct ErrorResponse {
    code: String,
    message: String,
}
```

## utoipa-axum Integration (OpenApiRouter)

```rust
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
    .routes(routes!(create_user, list_users))
    .routes(routes!(get_user))
    .split_for_parts();

// router: axum Router
// api: OpenApi spec (updated with routes)
```

## Serving API Docs

### Scalar (recommended)
```rust
use utoipa_scalar::{Scalar, Servable};

let app = Router::new()
    .merge(Scalar::with_url("/scalar", ApiDoc::openapi()));
```

### Swagger UI
```rust
use utoipa_swagger_ui::SwaggerUi;

let app = Router::new()
    .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));
```

## Security Schemes

```rust
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}
```

## Common Mistakes

1. **Forgetting to register schemas in `components(schemas(...))`** — types used in request_body/responses must be listed
2. **Path mismatch** — `#[utoipa::path(path = "...")]` must match the actual axum route
3. **Missing `ToSchema` derive** — all types in request/response bodies need `ToSchema`
4. **Enum variant naming** — `#[serde(rename_all)]` is respected by ToSchema, keep them in sync
5. **Nested types** — if `UserResponse` contains `Address`, `Address` also needs `ToSchema` and must be registered in components
6. **`axum_extras` feature** — required for axum extractor support (Path, Query, Json recognition)
