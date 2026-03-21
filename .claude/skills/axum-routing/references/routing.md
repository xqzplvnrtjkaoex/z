# Axum Routing Reference

> axum 0.8 | Source: https://docs.rs/axum/0.8

## Router

```rust
use axum::{Router, routing::{get, post, put, delete}};

let app = Router::new()
    .route("/", get(root))
    .route("/users", get(list_users).post(create_user))
    .route("/users/{id}", get(get_user).put(update_user).delete(delete_user));
```

## Path Parameters

```rust
// Single param
.route("/users/{id}", get(handler))
async fn handler(Path(id): Path<u32>) -> ...

// Multiple params
.route("/users/{user_id}/posts/{post_id}", get(handler))
async fn handler(Path((user_id, post_id)): Path<(u32, u32)>) -> ...

// Named params via struct
#[derive(Deserialize)]
struct Params { user_id: u32, post_id: u32 }
async fn handler(Path(params): Path<Params>) -> ...

// Wildcard (catch-all)
.route("/files/*path", get(handler))
async fn handler(Path(path): Path<String>) -> ...
```

## Nesting

```rust
// Mount sub-router under prefix
let api = Router::new()
    .route("/users", get(list_users));

let app = Router::new()
    .nest("/api/v1", api);
// Matches: /api/v1/users
```

## Merging

```rust
let user_routes = Router::new()
    .route("/users", get(list_users));
let health_routes = Router::new()
    .route("/healthz", get(healthz));

let app = user_routes.merge(health_routes);
```

## State

```rust
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
}

// Option 1: Clone-able state
let app = Router::new()
    .route("/", get(handler))
    .with_state(AppState { db });

// Option 2: Arc-wrapped state
let app = Router::new()
    .route("/", get(handler))
    .with_state(Arc::new(AppState { db }));
```

## Layers (Middleware)

```rust
use tower_http::trace::TraceLayer;

// Global middleware — applies to all routes including fallback
let app = Router::new()
    .route("/", get(handler))
    .layer(TraceLayer::new_for_http());

// Route-scoped middleware — only matched routes
let app = Router::new()
    .route("/", get(handler))
    .route_layer(middleware::from_fn(auth_middleware));
```

## Fallback

```rust
// Custom 404
let app = Router::new()
    .route("/", get(handler))
    .fallback(not_found);

async fn not_found() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "not found")
}
```

## Method Routing

```rust
use axum::routing::{get, post, put, patch, delete, any, MethodRouter};

// Chain methods on same path
.route("/resource", get(list).post(create))
.route("/resource/{id}", get(read).put(update).delete(remove))

// Any method
.route("/webhook", any(webhook_handler))

// Method-specific with different handlers
.route("/items",
    get(list_items)
        .post(create_item)
        .layer(some_layer) // layer on this method router only
)
```

## Serving with Tokio

```rust
let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
axum::serve(listener, app).await?;
```
