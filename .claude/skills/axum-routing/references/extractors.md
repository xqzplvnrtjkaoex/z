# Axum Extractors Reference

> axum 0.8 | Source: https://docs.rs/axum/0.8

## Extractor Ordering Rule

Extractors are applied left-to-right in the handler signature. The **last** extractor
may consume the request body. Only one body-consuming extractor is allowed.

```rust
// OK: non-body extractors first, body extractor last
async fn handler(
    State(state): State<AppState>,  // no body
    Path(id): Path<u32>,            // no body
    Json(body): Json<CreateUser>,   // body (last)
) -> impl IntoResponse { ... }
```

## Path

```rust
use axum::extract::Path;

// Single
async fn handler(Path(id): Path<u32>) -> ...

// Multiple (tuple)
async fn handler(Path((a, b)): Path<(String, u32)>) -> ...

// Struct (requires Deserialize)
#[derive(Deserialize)]
struct PathParams { id: u32, slug: String }
async fn handler(Path(params): Path<PathParams>) -> ...
```

## Query

```rust
use axum::extract::Query;

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    per_page: Option<u32>,
}

// GET /users?page=1&per_page=10
async fn handler(Query(params): Query<Pagination>) -> ...
```

## Json (body)

```rust
use axum::Json;

#[derive(Deserialize)]
struct CreateUser { name: String, email: String }

async fn handler(Json(payload): Json<CreateUser>) -> ...
```

## State

```rust
use axum::extract::State;
use std::sync::Arc;

async fn handler(State(state): State<Arc<AppState>>) -> ...
```

State type must match what was passed to `.with_state()`.

## Extension

```rust
use axum::Extension;

// Add via layer
let app = Router::new().layer(Extension(shared_data));

// Extract
async fn handler(Extension(data): Extension<SharedData>) -> ...
```

Prefer `State` over `Extension` when possible (compile-time type safety).

## Headers

```rust
use axum::http::HeaderMap;

async fn handler(headers: HeaderMap) -> ... {
    let auth = headers.get("authorization");
}
```

## TypedHeader (axum-extra)

```rust
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;

async fn handler(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> ... {
    let token = auth.token();
}
```

## Cookie (axum-extra)

```rust
use axum_extra::extract::CookieJar;

async fn handler(jar: CookieJar) -> ... {
    let value = jar.get("session_id").map(|c| c.value());
}

// Set cookies
async fn login(jar: CookieJar) -> (CookieJar, impl IntoResponse) {
    let jar = jar.add(Cookie::new("session_id", "abc123"));
    (jar, "logged in")
}
```

## Form (body)

```rust
use axum::Form;

#[derive(Deserialize)]
struct LoginForm { username: String, password: String }

async fn handler(Form(form): Form<LoginForm>) -> ...
```

## Request / Body

```rust
use axum::body::Bytes;
use axum::http::Request;

// Raw bytes
async fn handler(body: Bytes) -> ...

// Full request
async fn handler(req: Request) -> ...

// String body
async fn handler(body: String) -> ...
```

## Optional Extractors

```rust
// Option<T> — returns None if extraction fails (no error)
async fn handler(query: Option<Query<Params>>) -> ...

// Result<T, E> — returns Err if extraction fails
async fn handler(result: Result<Json<Body>, JsonRejection>) -> ...
```

## Custom Extractor

```rust
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

struct UserId(Uuid);

#[async_trait]
impl<S> FromRequestParts<S> for UserId
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let header = parts.headers
            .get("x-user-id")
            .ok_or((StatusCode::UNAUTHORIZED, "missing user id".into()))?;
        let id = header.to_str()
            .map_err(|_| (StatusCode::BAD_REQUEST, "invalid header".into()))?
            .parse::<Uuid>()
            .map_err(|_| (StatusCode::BAD_REQUEST, "invalid uuid".into()))?;
        Ok(UserId(id))
    }
}

// Use like any other extractor
async fn handler(UserId(id): UserId) -> ...
```

## Extractors Summary

| Extractor | Source | Consumes Body |
|-----------|--------|---------------|
| `Path<T>` | URL path params | No |
| `Query<T>` | Query string | No |
| `State<T>` | Router state | No |
| `Extension<T>` | Request extensions | No |
| `HeaderMap` | All headers | No |
| `TypedHeader<T>` | Typed header | No |
| `CookieJar` | Cookies | No |
| `Json<T>` | JSON body | Yes |
| `Form<T>` | Form body | Yes |
| `Bytes` | Raw bytes | Yes |
| `String` | UTF-8 body | Yes |
| `Request` | Full request | Yes |
