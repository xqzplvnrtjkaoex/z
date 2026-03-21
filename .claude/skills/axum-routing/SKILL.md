---
name: axum-routing
description: |
  CRITICAL: Use for axum routing and extractors. Triggers on:
  axum router, get post put delete, Path Query Json State,
  handler, extractor, Router::new, method_router, nest,
  FromRequestParts, FromRequest, axum middleware
---

# Axum Routing Skill

> **Version:** axum 0.8 | **Last Updated:** 2026-03-01
>
> Check for updates: https://crates.io/crates/axum

You are an expert at the Rust `axum` crate. Help users by:
- **Writing code**: Generate routers, handlers, extractors following axum 0.8 patterns
- **Answering questions**: Explain routing, state management, extractor ordering

## Documentation

- `./references/routing.md` — Router, method routing, nesting, merging, fallback
- `./references/extractors.md` — All extractors: Path, Query, Json, State, Form, etc.

## Key Patterns

### Basic Router

```rust
use axum::{Router, routing::get};

let app = Router::new()
    .route("/", get(root_handler))
    .route("/users/{id}", get(get_user).post(create_user));
```

### Handler with Extractors

```rust
use axum::extract::{Path, Query, State, Json};

async fn get_user(
    State(db): State<DatabaseConnection>,
    Path(id): Path<u32>,
) -> Result<Json<User>, AppError> {
    let user = find_user(&db, id).await?;
    Ok(Json(user))
}
```

### Shared State

```rust
use std::sync::Arc;

let shared_state = Arc::new(AppState { db, config });
let app = Router::new()
    .route("/", get(handler))
    .with_state(shared_state);

// In handler:
async fn handler(State(state): State<Arc<AppState>>) -> impl IntoResponse { ... }
```

### Nesting

```rust
let api = Router::new()
    .route("/users", get(list_users))
    .route("/users/{id}", get(get_user));

let app = Router::new()
    .nest("/api/v1", api);
// Matches: /api/v1/users, /api/v1/users/{id}
```

## API Reference Table

| Function | Description |
|----------|-------------|
| `Router::new()` | Create empty router |
| `.route(path, method_router)` | Add route |
| `.nest(prefix, router)` | Mount sub-router |
| `.merge(router)` | Combine routers |
| `.with_state(state)` | Attach shared state |
| `.layer(layer)` | Add middleware (global) |
| `.route_layer(layer)` | Add middleware (matched routes only) |
| `.fallback(handler)` | 404 handler |
| `get(handler)` | GET method router |
| `post(handler)` | POST method router |
| `put(handler)` | PUT method router |
| `delete(handler)` | DELETE method router |

## Extractor Order Rule

Extractors run left-to-right. The **last** extractor can consume the request body.
Only one body-consuming extractor per handler.

```rust
// OK: Path (no body) then Json (body)
async fn handler(Path(id): Path<u32>, Json(body): Json<CreateUser>) -> ...

// ERROR: Two body extractors
async fn handler(Json(a): Json<A>, Json(b): Json<B>) -> ... // compile error
```

## When Writing Code

1. State must be `Clone + Send + Sync + 'static` — wrap in `Arc` for complex types
2. Path params use `{name}` syntax (not `:name`)
3. Only one body-consuming extractor per handler (last position)
4. Use `Router::new().with_state(state)` not `.layer(Extension(state))`
5. Return `impl IntoResponse` or a concrete response type

## When Answering Questions

1. axum 0.8 uses `{param}` path syntax (changed from 0.7's `:param`)
2. `State` is preferred over `Extension` for shared state (type-safe at compile time)
3. Handlers are async fns that take extractors and return `impl IntoResponse`
4. `Router` is generic over state type `S` — use `.with_state()` to finalize
