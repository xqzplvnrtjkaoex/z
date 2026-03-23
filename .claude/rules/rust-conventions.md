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

## Type Conversion via Standard Traits

Prefer `From`/`TryFrom`/`FromStr` implementations over custom conversion methods or standalone functions. This applies to all type conversions, not just errors — the `?` operator and `.into()` handle propagation idiomatically.

When a `From` impl exists, use `?` directly instead of `.map_err(Foo::from)?`:

```rust
// Good — ? invokes From<tonic::Status> for AppError automatically
let response = client.get_user(request).await?;

// Bad — redundant, From impl already exists
let response = client.get_user(request).await.map_err(AppError::from)?;
```

Exception: when the source and target types are the same but the transformation is value-level (e.g., `to_snake_case()`, `to_kebab_case()`), or when the method name must explicitly convey the specific operation, use a named method instead.

## Cursor Encoding

Use `base64::engine::general_purpose::URL_SAFE_NO_PAD` for cursor/pagination tokens. Standard base64 contains `+`, `/`, `=` which require URL-encoding in query strings.

## DateTime Serialization

REST API timestamps use RFC 3339 with millisecond precision: `2026-03-23T12:00:00.000Z`. Use a custom serde serializer (`to_rfc3339_ms`) rather than relying on default `DateTime<Utc>` serialization.
