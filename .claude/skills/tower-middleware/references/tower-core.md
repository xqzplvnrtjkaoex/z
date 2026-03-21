# Tower Core Reference

> tower 0.5 | Source: https://docs.rs/tower/0.5

## Service Trait

```rust
pub trait Service<Request> {
    type Response;
    type Error;
    type Future: Future<Output = Result<Self::Response, Self::Error>>;
    fn call(&self, req: Request) -> Self::Future;
}
```

Tower 0.5 removed `poll_ready`. `call` takes `&self` (shared reference).

## Layer Trait

```rust
pub trait Layer<S> {
    type Service;
    fn layer(&self, inner: S) -> Self::Service;
}
```

## Custom Middleware Pattern

```rust
#[derive(Clone)]
pub struct LoggingMiddleware<S> {
    inner: S,
}

impl<S, B> Service<Request<B>> for LoggingMiddleware<S>
where
    S: Service<Request<B>> + Clone + Send + 'static,
    S::Future: Send,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn call(&self, req: Request<B>) -> Self::Future {
        tracing::info!("{} {}", req.method(), req.uri());
        self.inner.call(req)
    }
}

#[derive(Clone)]
pub struct LoggingLayer;

impl<S> Layer<S> for LoggingLayer {
    type Service = LoggingMiddleware<S>;
    fn layer(&self, inner: S) -> Self::Service {
        LoggingMiddleware { inner }
    }
}
```

## ServiceBuilder

```rust
use tower::ServiceBuilder;

let svc = ServiceBuilder::new()
    .timeout(Duration::from_secs(30))
    .rate_limit(100, Duration::from_secs(1))
    .buffer(1024)
    .service_fn(|req| async move { Ok::<_, BoxError>("ok") });
```

### Built-in Methods

| Method | Feature | Description |
|--------|---------|-------------|
| `.timeout(dur)` | `timeout` | Fail if inner takes too long |
| `.buffer(bound)` | `buffer` | Buffer requests in channel |
| `.rate_limit(num, per)` | `limit` | Token-bucket rate limiting |
| `.concurrency_limit(n)` | `limit` | Max in-flight requests |
| `.retry(policy)` | `retry` | Retry failed requests |
| `.layer(L)` | - | Apply any Layer |
| `.service_fn(f)` | `util` | Wrap async fn |
| `.service(s)` | - | Attach inner service |

## ServiceExt

```rust
use tower::ServiceExt;

// Call once
let response = svc.oneshot(request).await?;

// Map response
let mapped = svc.map_response(|r| transform(r));

// Map error
let mapped = svc.map_err(|e| MyError::from(e));
```

## BoxCloneService

```rust
use tower::util::BoxCloneService;
let boxed: BoxCloneService<Request, Response, BoxError> = svc.boxed_clone();
```

## Cargo Features

```toml
tower = { version = "0.5", features = ["full"] }
# or individually: ["timeout", "buffer", "limit", "retry", "util"]
```
