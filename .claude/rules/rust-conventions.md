---
paths: ["*.rs"]
---

# Rust Coding Conventions

## Typed Constants Over String Literals

Prefer typed constants over raw string literals for any protocol-level or well-known identifier (HTTP headers, cookie names, MIME types, gRPC metadata keys, etc.).

- MUST: When the framework or library already provides a typed constant (e.g., `header::CONTENT_TYPE`, `header::SET_COOKIE`), always use it.
- SHOULD: When no typed constant exists, define your own (`const` or `HeaderName::from_static(...)`) rather than repeating string literals across call sites. Exceptions are acceptable for one-off or context-local usage.

## Import Style

Do not alias modules when the original name is clear. Merge into grouped `use` statements.

```rust
// Good
use axum::{Router, middleware, routing::get};
middleware::from_fn(...)

// Bad — unnecessary alias
use axum::middleware as axum_mw;
axum_mw::from_fn(...)
```

## Attribute Style

When a field or item needs multiple rules from the same attribute macro, combine them into a single attribute. Do not repeat the attribute on separate lines.

```rust
// Good
#[validate(
    length(min = 4, max = 15, message = "handle must be 4-15 characters"),
    custom(function = "check_handle_chars", message = "must be alphanumeric/underscore")
)]
pub handle: String,

// Bad — same attribute repeated
#[validate(length(min = 4, max = 15, message = "handle must be 4-15 characters"))]
#[validate(custom(function = "check_handle_chars", message = "must be alphanumeric/underscore"))]
pub handle: String,
```

Applies to all derive macro attributes: `#[validate]`, `#[serde]`, `#[sea_orm]`, etc.

## Type Conversion via Standard Traits

Prefer `From`/`TryFrom`/`FromStr` implementations over custom conversion methods or standalone functions. This applies to all type conversions, not just errors — the `?` operator and `.into()` handle propagation idiomatically.

When a `From` impl exists, use `?` directly instead of `.map_err(Foo::from)?`:

```rust
// Good — ? invokes From<tonic::Status> for AppError automatically
let response = client.get_user(request).await?;

// Bad — redundant, From impl already exists
let response = client.get_user(request).await.map_err(AppError::from)?;
```

Prefer `From<T>` (by value) over `From<&T>` when the source is not used after conversion. This enables concise `.into()` calls and avoids unnecessary clones:

```rust
// Good — owned value, concise call site
impl From<User> for UserResponse { ... }
let resp = user.into();

// Avoid — forces verbose call site or awkward (&user).into()
impl From<&User> for UserResponse { ... }
let resp = UserResponse::from(&user);
```

Exception: when the source and target types are the same but the transformation is value-level (e.g., `to_snake_case()`, `to_kebab_case()`), or when the method name must explicitly convey the specific operation, use a named method instead.

When iterating and converting via `From`, prefer `itertools::Itertools::map_into()` over `.map(T::from)`:

```rust
// Good
let users: Vec<User> = inner.users.into_iter().map_into().collect();

// Bad — verbose, From impl is enough
let users: Vec<User> = inner.users.into_iter().map(User::from).collect();
```

## Input Validation

Structs that accept user-controlled values use `#[derive(validator::Validate)]`. Validation (reject invalid) and normalization (default/clamp) are separate concerns:

- **Validation**: `#[validate(...)]` attributes + `validate()?` call — rejects invalid values
- **Normalization**: Getter methods — applies defaults or transforms (e.g., 0 → 25)

```rust
#[derive(Validate)]
pub struct ListUsersPayload {
    #[validate(range(max = 100, message = "limit must be at most 100"))]
    pub limit: u64,
}

impl ListUsersPayload {
    pub fn limit(&self) -> u64 {
        if self.limit == 0 { 25 } else { self.limit }
    }
}
```

All payload fields are accessed through getter methods — even trivial passthrough fields like `include_inactive()`. This ensures callers never read raw fields directly, keeping a consistent access pattern and allowing normalization logic to be added later without changing call sites.

`From<ValidationErrors>` converts to the service's error type (`InvalidInput` variant).

## From Impl Placement Per Layer

`From` impls follow the dependency direction — placed in the layer that knows both types:

| Conversion | Placed in | Example |
|-----------|-----------|---------|
| Proto request → Usecase payload | `app/rpc/` | `From<ListUsersRequest> for ListUsersPayload` |
| sea-orm Model → Domain type | `adapter/` | `From<user::Model> for User` |
| Domain type → Proto response | `app/rpc/` | `From<User> for UserResponse` |

Prefer `From` over `new()` constructors for cross-layer conversions — avoids positional argument lists that grow with field count.

## Proto UUID Conversion

Proto `bytes` UUID fields map to `Vec<u8>` via prost.

- **Proto → Uuid**: `Uuid::from_slice(&req.id)`
- **Uuid → Proto**: `id.as_bytes().to_vec()`
- **Gateway path params**: `Path<Uuid>` — axum deserializes directly, no manual parsing

## Cursor Encoding

Use `base64::engine::general_purpose::URL_SAFE_NO_PAD` for cursor/pagination tokens. Standard base64 contains `+`, `/`, `=` which require URL-encoding in query strings.

## DateTime Serialization

REST API timestamps use RFC 3339 with millisecond precision: `2026-03-23T12:00:00.000Z`. Use a custom serde serializer (`to_rfc3339_ms`) rather than relying on default `DateTime<Utc>` serialization.
