---
name: tower-middleware
description: |
  CRITICAL: Use for tower Service/Layer and tower-http middleware. Triggers on:
  tower, Layer, Service, ServiceBuilder, middleware,
  TraceLayer, RequestIdLayer, CorsLayer, tower-http,
  SetRequestHeaderLayer, compression, timeout
---

# Tower Middleware Skill

> **Version:** tower 0.5 + tower-http 0.6 | **Last Updated:** 2026-03-01
>
> Check for updates: https://crates.io/crates/tower

You are an expert at the Rust `tower` and `tower-http` crates. Help users by:
- **Writing code**: Generate middleware, layers, service compositions
- **Answering questions**: Explain Service/Layer concepts, ordering, tower-http layers

## Documentation

- `./references/tower-core.md` — Service trait, Layer trait, ServiceBuilder
- `./references/tower-http.md` — TraceLayer, RequestIdLayer, CorsLayer, etc.

## Key Patterns

### Service Trait (tower 0.5)

```rust
pub trait Service<Request> {
    type Response;
    type Error;
    type Future: Future<Output = Result<Self::Response, Self::Error>>;
    fn call(&self, req: Request) -> Self::Future;
}
```

Note: tower 0.5 removed `poll_ready`. `call` takes `&self`.

### Layer Trait

```rust
pub trait Layer<S> {
    type Service;
    fn layer(&self, inner: S) -> Self::Service;
}
```

### TraceLayer with axum

```rust
use tower_http::trace::TraceLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};

let app = Router::new()
    .route("/", get(handler))
    .layer(PropagateRequestIdLayer::x_request_id())
    .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
    .layer(TraceLayer::new_for_http());
```

### ServiceBuilder

```rust
use tower::ServiceBuilder;

let svc = ServiceBuilder::new()
    .timeout(Duration::from_secs(30))
    .buffer(1024)
    .layer(MyCustomLayer::new())
    .service(inner_service);
```

## API Reference Table

| Layer (tower-http) | Feature | Description |
|--------------------|---------|-------------|
| `TraceLayer` | `trace` | Request/response tracing spans |
| `SetRequestIdLayer` | `request-id` | Add request ID header |
| `PropagateRequestIdLayer` | `request-id` | Echo request ID on response |
| `CorsLayer` | `cors` | CORS headers |
| `CompressionLayer` | `compression-*` | Response compression |
| `TimeoutLayer` | `timeout` | Total request timeout |
| `SetSensitiveRequestHeadersLayer` | `sensitive-headers` | Redact headers in traces |

## When Writing Code

1. Layer ordering matters: last `.layer()` in axum Router is outermost
2. Use `ServiceBuilder` when composing multiple layers
3. Use `.route_layer()` for route-scoped middleware in axum
4. State in custom middleware should be in `Arc` for Clone
5. Mark sensitive headers with `SetSensitiveRequestHeadersLayer`

## When Answering Questions

1. `Layer` is a factory, `Service` is the async function abstraction
2. axum `Router::layer()` applies in reverse declaration order
3. `TraceLayer::new_for_http()` for HTTP, `::new_for_grpc()` for gRPC
4. `CorsLayer::permissive()` is dev-only; production needs explicit origins
5. Both `DatabaseConnection` and `DatabaseTransaction` implement tower's `Service` pattern
