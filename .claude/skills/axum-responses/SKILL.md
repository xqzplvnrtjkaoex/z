---
name: axum-responses
description: |
  CRITICAL: Use for axum responses, error handling, IntoResponse. Triggers on:
  IntoResponse, axum response, StatusCode, axum error handling,
  axum middleware, from_fn, axum::middleware, response headers,
  Json response, axum rejection, AppError
---

# Axum Responses Skill

> **Version:** axum 0.8 | **Last Updated:** 2026-03-01
>
> Check for updates: https://crates.io/crates/axum

You are an expert at the Rust `axum` crate. Help users by:
- **Writing code**: Generate response types, error handlers, middleware
- **Answering questions**: Explain IntoResponse, error patterns, middleware ordering

## Documentation

- `./references/responses.md` — IntoResponse, status codes, headers, Json, custom responses
- `./references/middleware.md` — from_fn middleware, Tower layers, ordering

## Key Patterns

### Tuple Response

```rust
use axum::http::StatusCode;

async fn handler() -> (StatusCode, String) {
    (StatusCode::CREATED, "created".to_string())
}
```

### Json Response

```rust
use axum::Json;

async fn handler() -> Json<User> {
    Json(User { id: 1, name: "Alice".into() })
}
```

### Error Handling

```rust
use axum::response::{IntoResponse, Response};

enum AppError {
    NotFound,
    Internal(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "not found"),
            Self::Internal(e) => {
                tracing::error!(error = %e, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error")
            }
        };
        (status, body).into_response()
    }
}
```

### Middleware (from_fn)

```rust
use axum::middleware::{self, Next};
use axum::extract::Request;

async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth = request.headers().get("authorization")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(next.run(request).await)
}

let app = Router::new()
    .route("/", get(handler))
    .route_layer(middleware::from_fn(auth_middleware));
```

## API Reference Table

| Type | Description |
|------|-------------|
| `impl IntoResponse` | Any type implementing IntoResponse |
| `(StatusCode, T)` | Response with status + body |
| `(StatusCode, HeaderMap, T)` | Status + headers + body |
| `Json<T>` | JSON response (Content-Type: application/json) |
| `Response` | Full http::Response |
| `StatusCode` | Status code only (empty body) |
| `String` / `&str` | Plain text response |
| `Redirect` | HTTP redirect |
| `Html<T>` | HTML response |

## When Writing Code

1. Implement `IntoResponse` on your error type for ergonomic `Result<T, E>` returns
2. Use `(StatusCode, Json(body))` for JSON error responses
3. `from_fn` middleware takes `(Request, Next)` and returns `Result<Response, E>`
4. `.layer()` is global, `.route_layer()` is matched-routes-only

## When Answering Questions

1. `IntoResponse` is implemented for tuples: `(StatusCode, impl IntoResponse)`
2. Headers can be added via `(StatusCode, [(header, value)], body)` tuples
3. `Json(value)` automatically sets `Content-Type: application/json`
4. Multiple `impl IntoResponse` types can't be returned directly; use `Response`
