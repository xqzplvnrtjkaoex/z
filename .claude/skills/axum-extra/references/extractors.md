# axum-extra Extractors Reference

> axum-extra 0.12 | Source: https://docs.rs/axum-extra/0.12

## TypedHeader (feature: `typed-header`)

Strongly-typed header extraction using the `headers` crate.

```rust
use axum_extra::TypedHeader;
use axum_extra::headers::{Authorization, authorization::Bearer};
use axum_extra::headers::UserAgent;

// Bearer token
async fn auth(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> String {
    auth.token().to_string()
}

// User-Agent
async fn ua(
    TypedHeader(ua): TypedHeader<UserAgent>,
) -> String {
    ua.to_string()
}

// Optional header
async fn maybe_auth(
    auth: Option<TypedHeader<Authorization<Bearer>>>,
) -> &'static str {
    match auth {
        Some(TypedHeader(a)) => "authenticated",
        None => "anonymous",
    }
}
```

### Common Header Types

| Type | Header | Access |
|------|--------|--------|
| `Authorization<Bearer>` | `Authorization: Bearer xxx` | `.token()` |
| `Authorization<Basic>` | `Authorization: Basic xxx` | `.username()`, `.password()` |
| `ContentType` | `Content-Type` | `.to_string()` |
| `UserAgent` | `User-Agent` | `.to_string()` |
| `Host` | `Host` | `.hostname()`, `.port()` |
| `Origin` | `Origin` | `.hostname()`, `.port()` |
| `Referer` | `Referer` | `.to_string()` |
| `Cookie` | `Cookie` | `.get("name")` |

## Form

URL-encoded form body extraction (re-exported from axum, enhanced in axum-extra).

```rust
use axum::extract::Form;

#[derive(Deserialize)]
struct Login {
    email: String,
    password: String,
}

async fn login(Form(input): Form<Login>) -> String {
    format!("email: {}", input.email)
}
```

## Query

Query string extraction.

```rust
use axum::extract::Query;

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    per_page: Option<u32>,
}

async fn list(Query(p): Query<Pagination>) -> String {
    format!("page {}", p.page.unwrap_or(1))
}
```

## Cached

Cache an extractor result so it can be used multiple times without re-parsing.

```rust
use axum_extra::extract::Cached;

async fn handler(
    Cached(TypedHeader(auth)): Cached<TypedHeader<Authorization<Bearer>>>,
) {
    // auth is cached — subsequent uses of Cached<TypedHeader<...>>
    // in the same request return the cached value
}
```

## WithRejection

Customize rejection (error) responses for extractors.

```rust
use axum_extra::extract::WithRejection;
use axum::extract::rejection::JsonRejection;

async fn handler(
    WithRejection(Json(body), _): WithRejection<Json<MyStruct>, MyError>,
) -> Result<(), MyError> {
    Ok(())
}

// MyError must implement From<JsonRejection> + IntoResponse
```
