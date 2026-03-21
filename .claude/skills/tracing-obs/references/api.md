# Tracing API Reference

> tracing 0.1 + tracing-subscriber 0.3

## Subscriber Setup

### Basic (stdout, human-readable)

```rust
tracing_subscriber::fmt::init();
```

### With EnvFilter

```rust
use tracing_subscriber::{fmt, EnvFilter};

tracing_subscriber::fmt()
    .with_env_filter(
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"))
    )
    .init();
```

### JSON Output

```rust
tracing_subscriber::fmt()
    .json()
    .with_env_filter(EnvFilter::from_default_env())
    .init();
```

### With Target and Thread Info

```rust
tracing_subscriber::fmt()
    .with_target(true)
    .with_thread_ids(true)
    .with_file(true)
    .with_line_number(true)
    .init();
```

### Layered Subscriber

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

tracing_subscriber::registry()
    .with(fmt::layer().json())
    .with(EnvFilter::from_default_env())
    .init();
```

## RUST_LOG Syntax

```bash
# Global level
RUST_LOG=info

# Per-crate
RUST_LOG=warn,my_crate=debug

# Per-module
RUST_LOG=my_crate::db=trace

# With span filtering
RUST_LOG=my_crate[span_name]=debug

# Multiple targets
RUST_LOG=info,tower_http=debug,sea_orm=warn
```

## Events (Log Messages)

```rust
// Simple message
tracing::info!("server started");

// With structured fields
tracing::info!(port = 3000, host = "0.0.0.0", "server started");

// Display format
tracing::error!(error = %err, "request failed");

// Debug format
tracing::debug!(request = ?req, "incoming request");

// With target (overrides module path)
tracing::info!(target: "audit", user_id = %id, "user logged in");
```

## Spans

```rust
// Create and enter span
let span = tracing::info_span!("process", request_id = %id);
let _guard = span.enter();

// Async span (use .instrument())
use tracing::Instrument;
async_fn().instrument(tracing::info_span!("my_task")).await;

// Span levels
tracing::error_span!("name");
tracing::warn_span!("name");
tracing::info_span!("name");
tracing::debug_span!("name");
tracing::trace_span!("name");
```

## #[instrument] Attribute

```rust
use tracing::instrument;

// Basic
#[instrument]
async fn handler(id: u32) -> Result<(), Error> { ... }
// Creates span: handler{id=42}

// Skip fields
#[instrument(skip(db, password))]
async fn login(db: &Database, email: &str, password: &str) { ... }

// Custom name
#[instrument(name = "user_lookup")]
async fn get_user(id: Uuid) { ... }

// Custom fields
#[instrument(fields(user_id = %id, service = "auth"))]
async fn authenticate(id: Uuid) { ... }

// Skip all, add specific
#[instrument(skip_all, fields(user_id = %user_id))]
async fn process(db: &Database, user_id: Uuid, data: LargeStruct) { ... }

// Set level
#[instrument(level = "debug")]
async fn internal_fn() { ... }

// With error handling
#[instrument(err)]
async fn fallible() -> Result<(), MyError> { ... }
// Automatically logs error on Err return
```

## Recording Dynamic Values in Spans

```rust
let span = tracing::info_span!("process", result = tracing::field::Empty);
let _guard = span.enter();

// Later, record the value
span.record("result", &"success");
```

## Conditional Logging

```rust
if tracing::enabled!(tracing::Level::DEBUG) {
    let expensive = compute_debug_info();
    tracing::debug!(info = ?expensive, "debug details");
}
```
