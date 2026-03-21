---
name: tracing-obs
description: |
  CRITICAL: Use for tracing and observability. Triggers on:
  tracing, tracing-subscriber, spans, events, instrument,
  tracing::info, tracing::error, tracing::warn, tracing::debug,
  EnvFilter, subscriber, structured logging, log levels
---

# Tracing Observability Skill

> **Version:** tracing 0.1 + tracing-subscriber 0.3 | **Last Updated:** 2026-03-01
>
> Check for updates: https://crates.io/crates/tracing

You are an expert at the Rust `tracing` and `tracing-subscriber` crates. Help users by:
- **Writing code**: Generate tracing setup, structured logging, instrumentation
- **Answering questions**: Explain spans, events, subscribers, filtering

## Documentation

- `./references/api.md` — Subscriber setup, macros, spans, EnvFilter, JSON output

## Key Patterns

### Subscriber Setup

```rust
use tracing_subscriber::{fmt, EnvFilter};

tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env()) // RUST_LOG
    .with_target(true)
    .init();
```

### Events (Log Messages)

```rust
tracing::info!("server started on port {}", port);
tracing::error!(error = %err, "failed to connect");
tracing::warn!(user_id = %id, "rate limit exceeded");
tracing::debug!(?request, "incoming request");
```

### Instrument Async Functions

```rust
use tracing::instrument;

#[instrument(skip(db), fields(user_id = %user_id))]
async fn get_user(db: &Database, user_id: Uuid) -> Result<User, Error> {
    // automatically creates a span with function name
    tracing::info!("fetching user");
    db.find(user_id).await
}
```

### Span Creation

```rust
let span = tracing::info_span!("process_request", request_id = %id);
let _guard = span.enter();
// everything in this scope is inside the span
```

## API Reference Table

| Macro | Level | Use for |
|-------|-------|---------|
| `tracing::error!` | ERROR | Failures requiring attention |
| `tracing::warn!` | WARN | Unusual but expected conditions |
| `tracing::info!` | INFO | Normal operational events |
| `tracing::debug!` | DEBUG | Detailed diagnostic info |
| `tracing::trace!` | TRACE | Very verbose debugging |

### Field Formatting

| Syntax | Format | Example |
|--------|--------|---------|
| `field = value` | Display | `port = 3000` |
| `field = %value` | Display | `error = %err` |
| `field = ?value` | Debug | `request = ?req` |
| `field` | Display (variable name = field name) | `port` |

## When Writing Code

1. Use `#[instrument]` on async functions for automatic span creation
2. Use `skip(field)` in `#[instrument]` to avoid logging sensitive/large fields
3. Use structured fields (`key = value`) not string interpolation
4. Only `tracing::error!` for 5xx/Internal errors; 4xx are expected
5. Set up with `EnvFilter` for runtime log level control via `RUST_LOG`

## When Answering Questions

1. `RUST_LOG=info,my_crate=debug` sets default to info, my_crate to debug
2. `#[instrument]` creates a span, not an event; events go inside spans
3. `%` = Display format, `?` = Debug format in field values
4. `tracing` is the facade crate; `tracing-subscriber` is the implementation
5. For JSON output: `.json()` on the subscriber builder
