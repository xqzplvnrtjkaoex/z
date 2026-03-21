# Tower-HTTP Reference

> tower-http 0.6 | Source: https://docs.rs/tower-http/0.6

## Cargo Features

```toml
tower-http = { version = "0.6", features = [
    "trace", "request-id", "cors", "compression-gzip",
    "sensitive-headers", "timeout", "set-header", "util",
] }
```

## TraceLayer

```rust
use tower_http::trace::TraceLayer;

// Minimal
let app = Router::new().layer(TraceLayer::new_for_http());

// For gRPC
let app = Router::new().layer(TraceLayer::new_for_grpc());

// Custom classifier (4xx/5xx as failures)
use tower_http::classify::StatusInRangeAsFailures;
let layer = TraceLayer::new(
    StatusInRangeAsFailures::new(400..=599).into_make_classifier()
);

// Custom spans
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse};
use tracing::Level;
let layer = TraceLayer::new_for_http()
    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
    .on_response(DefaultOnResponse::new().level(Level::INFO));
```

## Request ID

```rust
use tower_http::request_id::{
    MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer,
};

// Standard x-request-id
let set = SetRequestIdLayer::x_request_id(MakeRequestUuid);
let propagate = PropagateRequestIdLayer::x_request_id();

// In axum (propagate before set — outermost layer declared last)
let app = Router::new()
    .layer(propagate)
    .layer(set)
    .layer(TraceLayer::new_for_http());
```

## CorsLayer

```rust
use tower_http::cors::{CorsLayer, Any};

// Development only
let cors = CorsLayer::permissive();

// Production
let cors = CorsLayer::new()
    .allow_origin("https://app.example.com".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
    .allow_credentials(true)
    .max_age(Duration::from_secs(3600));
```

## Compression

```rust
use tower_http::compression::CompressionLayer;
use tower_http::decompression::DecompressionLayer;

let app = Router::new()
    .layer(CompressionLayer::new())    // compress responses
    .layer(DecompressionLayer::new()); // decompress request bodies
```

## Timeout

```rust
use tower_http::timeout::{TimeoutLayer, RequestBodyTimeoutLayer};

let app = Router::new()
    .layer(TimeoutLayer::new(Duration::from_secs(30)))
    .layer(RequestBodyTimeoutLayer::new(Duration::from_secs(10)));
```

## Sensitive Headers

```rust
use tower_http::sensitive_headers::{
    SetSensitiveRequestHeadersLayer, SetSensitiveResponseHeadersLayer,
};
use std::sync::Arc;

let app = Router::new()
    .layer(SetSensitiveRequestHeadersLayer::new(Arc::new([
        header::AUTHORIZATION, header::COOKIE,
    ])))
    .layer(SetSensitiveResponseHeadersLayer::new(Arc::new([
        header::SET_COOKIE,
    ])));
```

## Set Header

```rust
use tower_http::set_header::SetResponseHeaderLayer;

let layer = SetResponseHeaderLayer::overriding(
    header::SERVER,
    HeaderValue::from_static("my-server/1.0"),
);
```

## Typical Production Stack

```rust
let app = Router::new()
    // ... routes ...
    .layer(
        ServiceBuilder::new()
            .layer(SetSensitiveRequestHeadersLayer::new(
                Arc::new([header::COOKIE, header::AUTHORIZATION])
            ))
            .layer(TraceLayer::new_for_http())
            .layer(PropagateRequestIdLayer::x_request_id())
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
            .layer(CompressionLayer::new())
            .layer(TimeoutLayer::new(Duration::from_secs(30)))
            .layer(cors_layer)
    );
```
