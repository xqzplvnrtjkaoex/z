# Axum Middleware Reference

> axum 0.8 | Source: https://docs.rs/axum/0.8

## from_fn Middleware

The simplest way to write custom middleware in axum.

```rust
use axum::{
    extract::Request,
    middleware::{self, Next},
    response::Response,
};

async fn my_middleware(
    request: Request,
    next: Next,
) -> Response {
    // Do something before the handler
    let response = next.run(request).await;
    // Do something after the handler
    response
}

let app = Router::new()
    .route("/", get(handler))
    .layer(middleware::from_fn(my_middleware));
```

## from_fn with State

```rust
async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = request.headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate token using state
    validate_token(&state, token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(next.run(request).await)
}

let app = Router::new()
    .route("/protected", get(handler))
    .route_layer(middleware::from_fn_with_state(
        shared_state.clone(),
        auth_middleware,
    ))
    .with_state(shared_state);
```

## Middleware with Request Extensions

```rust
use axum::Extension;

async fn inject_user(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user_id = extract_user_from_headers(&request)?;
    request.extensions_mut().insert(UserId(user_id));
    Ok(next.run(request).await)
}

// In handler
async fn handler(Extension(user_id): Extension<UserId>) -> impl IntoResponse {
    // use user_id
}
```

## Layer vs Route Layer

```rust
let app = Router::new()
    .route("/public", get(public_handler))
    .route("/private", get(private_handler))
    // .layer() applies to ALL routes (including fallback)
    .layer(TraceLayer::new_for_http())
    // .route_layer() applies only to matched routes
    .route_layer(middleware::from_fn(auth_middleware));

// Result:
// GET /public   -> auth_middleware -> public_handler
// GET /private  -> auth_middleware -> private_handler
// GET /unknown  -> TraceLayer only (no auth_middleware)
```

## Layer Ordering

Layers are applied in reverse declaration order. The last `.layer()` call
is the outermost (first to process requests).

```rust
let app = Router::new()
    .route("/", get(handler))
    .layer(layer_a)  // inner (closer to handler)
    .layer(layer_b); // outer (first to see request)

// Request flow: layer_b -> layer_a -> handler
// Response flow: handler -> layer_a -> layer_b
```

## Tower ServiceBuilder with axum

```rust
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

let app = Router::new()
    .route("/", get(handler))
    .layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(TimeoutLayer::new(Duration::from_secs(30)))
    );
```

## Scoped Middleware per Router

```rust
let admin_routes = Router::new()
    .route("/dashboard", get(dashboard))
    .route_layer(middleware::from_fn(admin_only));

let user_routes = Router::new()
    .route("/profile", get(profile))
    .route_layer(middleware::from_fn(auth_required));

let app = Router::new()
    .nest("/admin", admin_routes)
    .nest("/user", user_routes)
    .route("/public", get(public));
// /admin/* requires admin auth
// /user/* requires user auth
// /public has no auth
```
