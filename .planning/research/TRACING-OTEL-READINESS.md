# Tracing OpenTelemetry Readiness Research

**Researched:** 2026-03-22
**Domain:** OpenTelemetry readiness for Rust tracing infrastructure
**Confidence:** HIGH
**Companion document:** TRACING-CONVENTIONS.md (this supplements, not replaces)

## Summary

This research investigates how to structure the Madome project's tracing infrastructure so that adding OpenTelemetry requires zero application code changes -- only configuration and dependency additions. The project currently uses `tracing` 0.1 + `tracing-subscriber` 0.3 with a simple `fmt().init()` subscriber pipeline. The key finding is that the path to OTel readiness requires exactly three changes to the current conventions:

1. **Subscriber pipeline refactor:** Replace `fmt().with_env_filter(...).init()` with the layered `registry().with(env_filter).with(fmt_layer).init()` pattern. This is the single most important change -- it makes adding `.with(otel_layer)` a one-line addition later.

2. **gRPC TraceLayer span enrichment:** Add `otel.kind` field to gRPC spans (`otel.kind = "server"` for internal services, implied "internal" for gateway HTTP spans). These fields are ignored by the fmt layer but picked up automatically by the OTel layer when added.

3. **Keep `x-request-id`, do NOT adopt `traceparent` yet.** The W3C `traceparent` header is an OTel concern and should be added only when OTel is enabled. The current `x-request-id` propagation via `CallerContext` works for application-level correlation. When OTel is introduced, it will handle `traceparent` injection/extraction automatically via its own middleware layer -- the two can coexist.

**Primary recommendation:** Refactor `init_tracing()` to use Registry + layer composition. Add `otel.kind` field to gRPC handler spans. Everything else in TRACING-CONVENTIONS.md remains unchanged.

## Changes to Existing TRACING-CONVENTIONS.md

### Change 1: `init_tracing()` Must Use Registry + Layers (CRITICAL)

**Current (TRACING-CONVENTIONS.md):**
```rust
pub fn init_tracing(service_name: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("{service_name}=debug,madome=debug,info")));
    fmt().with_env_filter(filter).with_target(true).with_thread_ids(false).with_file(false).init();
}
```

**OTel-ready (REPLACE WITH):**
```rust
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_tracing(service_name: &str) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("{service_name}=debug,madome=debug,info")));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}
```

**Why this matters:**
- `fmt().init()` creates a monolithic subscriber. You cannot add layers to it later.
- `registry().with(fmt_layer).init()` creates a composable pipeline. Adding OTel is one line:
  ```rust
  tracing_subscriber::registry()
      .with(env_filter)
      .with(fmt_layer)
      .with(otel_layer)  // <-- only addition needed later
      .init();
  ```
- The `fmt::layer()` produces a `Layer` that can be composed with `Registry`. The `fmt()` shorthand produces a standalone `Subscriber`.
- **Behavioral equivalence:** The output format is identical. This is a structural refactor, not a behavior change.

**Required Cargo.toml change:**
```toml
# Current:
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }

# OTel-ready:
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "registry"] }
```

The `registry` feature is NOT in the default feature set. It must be explicitly enabled. It pulls in `sharded-slab` and `thread_local` for per-span storage.

**Confidence:** HIGH -- Verified against `tracing-subscriber` 0.3.23 docs.rs documentation. The `registry()` function requires `registry` + `std` features. The `fmt::layer()` API is documented as the layer-compatible equivalent of `fmt()`.

### Change 2: Add `otel.kind` Field to gRPC Handler Spans

**Current (TRACING-CONVENTIONS.md):**
```rust
#[instrument(skip_all, fields(rpc = "CreateUser"))]
pub async fn handle<C: UserPorts>(...) -> ... { ... }
```

**OTel-ready (ADD `otel.kind`):**
```rust
#[instrument(skip_all, fields(otel.kind = "server", rpc = "CreateUser"))]
pub async fn handle<C: UserPorts>(...) -> ... { ... }
```

**Why:**
- The `otel.kind` field is reserved by `tracing-opentelemetry`. When no OTel layer is present, it is treated as an ordinary field by the fmt layer (harmlessly included in log output).
- When the OTel layer is added later, it automatically interprets `otel.kind = "server"` to set the OpenTelemetry SpanKind, which is required for correct trace visualization in backends like Jaeger/Tempo.
- Without `otel.kind`, all spans default to SpanKind::Internal, which makes the trace tree hard to interpret.

**Where to set `otel.kind`:**

| Location | Value | Rationale |
|----------|-------|-----------|
| gRPC handler `#[instrument]` | `"server"` | Receiving an RPC request |
| Gateway gRPC client calls | `"client"` | Sending an RPC request (set when OTel layer is added, via propagation middleware) |
| Gateway HTTP TraceLayer | Not needed | The HTTP instrumentation layer will set it automatically when OTel is added |
| Usecase functions | Not needed | Internal spans default to SpanKind::Internal, which is correct |

**Confidence:** HIGH -- Verified against `tracing-opentelemetry` 0.32.1 docs.rs documentation. Fields with `otel.` prefix are "reserved for this crate and have specific meaning. They are treated as ordinary fields by other layers."

### Change 3: No Other Field Name Changes Needed

**Question from TRACING-CONVENTIONS.md:** Do the current field names (`method`, `uri`, `request_id`, `status`) need to change to match OTel semantic conventions (`http.request.method`, `url.path`, `http.response.status_code`)?

**Answer: NO.** `tracing-opentelemetry` does NOT automatically rename fields to match OTel semantic conventions. All tracing fields pass through as-is to become OTel span attributes. The mapping is direct: `method = %request.method()` becomes an OTel attribute named `method`.

This means two things:
1. Our current field names (`method`, `uri`, `request_id`) will appear as custom attributes in OTel, not as standard semantic convention attributes.
2. When OTel is actually adopted, the HTTP/gRPC instrumentation middleware (e.g., `axum-tracing-opentelemetry` or a custom tower layer) will add the semantic convention attributes separately. Our custom attributes coexist alongside them.

**Do NOT rename fields now.** The tracing fields serve application-level logging. OTel semantic convention attributes are an OTel concern and will be added by OTel-specific middleware when adopted.

However, if desired, you CAN use dotted field names that match OTel semantic conventions:
```rust
tracing::info_span!(
    "http_request",
    "http.request.method" = %request.method(),
    "url.path" = %request.uri().path(),
    request_id = %request_id,
)
```
These dotted names work in tracing (they become string-keyed attributes). But this is NOT recommended because:
- It makes log output verbose and harder to read
- It couples application logging to OTel conventions prematurely
- The OTel layer can add these attributes via its own middleware

**Confidence:** HIGH -- Verified against `tracing-opentelemetry` docs: "OpenTelemetry defines conventional names for attributes of common operations. These names can be assigned directly as fields... and they will be passed through to your configured OpenTelemetry exporter." This confirms pass-through, not auto-mapping.

### Change 4: Update "Don't Hand-Roll" and "State of the Art" Sections

Add to TRACING-CONVENTIONS.md "Don't Hand-Roll" table:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| OTel trace context propagation | Custom `traceparent` parsing/injection | `opentelemetry::propagation::TraceContextPropagator` + `opentelemetry-http::HeaderInjector/HeaderExtractor` | W3C Trace Context has precise parsing rules, sampling flag semantics, and version negotiation |
| OTel span export | Custom HTTP/gRPC exporter | `opentelemetry-otlp` with tonic transport | OTLP protocol is complex, handles batching/retry/backpressure |
| gRPC trace context in metadata | Manual `traceparent` header handling | Custom `MetadataInjector`/`MetadataExtractor` implementing `Injector`/`Extractor` traits | Standard pattern documented in opentelemetry-rust, handles tonic MetadataMap correctly |

Update "State of the Art" deferred items:

| Item | Status | When to Add |
|------|--------|-------------|
| `tracing-opentelemetry` | Deferred -- code is READY | When OTel backend (Jaeger/Tempo/etc.) is provisioned |
| `tracing-subscriber` JSON output | Deferred | When log aggregation (Loki/etc.) is set up |
| `tracing-error` / SpanTrace | Deferred | When error diagnosis becomes a pain point |

## OpenTelemetry Readiness

### What "OTel-Ready" Means Concretely

When the team decides to add OpenTelemetry, the changes should be:

1. **Add dependencies** (Cargo.toml only):
   ```toml
   opentelemetry = "0.31"
   opentelemetry_sdk = { version = "0.31", features = ["rt-tokio"] }
   opentelemetry-otlp = { version = "0.31", features = ["tonic"] }
   tracing-opentelemetry = "0.32"
   opentelemetry-http = "0.31"
   ```

2. **Add OTel layer to `init_tracing()`** (one function, ~15 lines):
   ```rust
   pub fn init_tracing(service_name: &str) {
       let env_filter = EnvFilter::try_from_default_env()
           .unwrap_or_else(|_| EnvFilter::new(format!("{service_name}=debug,madome=debug,info")));

       let fmt_layer = fmt::layer()
           .with_target(true)
           .with_thread_ids(false)
           .with_file(false);

       // -- NEW: OTel layer (only addition) --
       let provider = opentelemetry_otlp::new_pipeline()
           .tracing()
           .with_exporter(opentelemetry_otlp::new_exporter().tonic())
           .with_trace_config(
               opentelemetry_sdk::trace::Config::default()
                   .with_resource(opentelemetry_sdk::Resource::new(vec![
                       opentelemetry::KeyValue::new("service.name", service_name.to_string()),
                   ])),
           )
           .install_batch(opentelemetry_sdk::runtime::Tokio)
           .expect("failed to init OTel tracer");

       let otel_layer = tracing_opentelemetry::layer()
           .with_tracer(provider);
       // -- END NEW --

       tracing_subscriber::registry()
           .with(env_filter)
           .with(fmt_layer)
           .with(otel_layer)  // <-- the only structural addition
           .init();
   }
   ```

3. **Add trace context propagation middleware** (gateway + internal services, ~50 lines total):
   - Gateway: Extract `traceparent` from incoming HTTP requests, inject into outgoing gRPC calls
   - Internal services: Extract `traceparent` from incoming gRPC metadata

4. **Zero changes to:**
   - `#[instrument]` attributes on any function
   - `tracing::info!()` / `tracing::error!()` event calls
   - TraceLayer configuration
   - Field names or span names
   - Business logic code

### Subscriber Layer Composition

#### Layer Ordering

In `tracing-subscriber`, layers are evaluated **top-down (outermost first)** for filtering. The `EnvFilter` should be outermost so it gates what both `fmt_layer` and `otel_layer` see:

```rust
tracing_subscriber::registry()
    .with(env_filter)     // Outermost: gates everything
    .with(fmt_layer)      // Console output
    .with(otel_layer)     // OTel export (added later)
    .init();
```

This means `EnvFilter` acts as a **global filter** -- spans/events that don't pass the filter are invisible to ALL layers. This is the correct behavior: we don't want to export DEBUG spans to OTel in production.

#### Per-Layer Filtering (Future Option)

If different filtering is needed per layer (e.g., TRACE to console but only INFO+ to OTel), use per-layer filters:

```rust
tracing_subscriber::registry()
    .with(fmt_layer.with_filter(EnvFilter::new("debug")))
    .with(otel_layer.with_filter(EnvFilter::new("info")))
    .init();
```

This is NOT needed now. Start with a single global `EnvFilter`. Per-layer filtering adds complexity and should only be introduced when there is a concrete need.

**Confidence:** HIGH -- Verified against `tracing-subscriber` layer docs: "The filtering methods on a stack of Layers are evaluated in a top-down order, starting with the outermost Layer."

### Trace Context Propagation Strategy

#### Decision: Keep `x-request-id`, Defer `traceparent`

| Header | Purpose | When Used | Protocol |
|--------|---------|-----------|----------|
| `x-request-id` | Application-level request correlation | NOW | Custom (UUIDv7) |
| `traceparent` | Distributed trace context (W3C Trace Context) | When OTel is enabled | W3C standard |

**Rationale:**

1. `traceparent` format is `{version}-{trace_id}-{parent_id}-{trace_flags}` (e.g., `00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01`). This is NOT a UUID -- it is a 32-hex-char trace ID + 16-hex-char span ID. Generating valid `traceparent` values without an OTel SDK requires implementing the W3C spec manually, which violates our "Don't Hand-Roll" principle.

2. `x-request-id` (UUIDv7) serves application-level correlation: grep logs by request ID, correlate gateway logs with internal service logs. This remains useful even with OTel.

3. When OTel is enabled, the OTel propagation middleware will handle `traceparent` injection/extraction automatically. The `x-request-id` and `traceparent` headers coexist -- they serve different purposes.

4. `x-request-id` is already implemented via `CallerContext.inject_into()`. Replacing it with `traceparent` now would require adding OTel dependencies just for header generation -- the opposite of our goal.

**When OTel is added, the propagation flow becomes:**

```
Client -> [traceparent: ...] -> Gateway
  Gateway: extract traceparent -> set as parent span
  Gateway -> [traceparent: ...] + [x-request-id: ...] -> Internal Service
  Internal Service: extract traceparent -> set as parent span
```

Both headers flow through. `traceparent` enables distributed tracing. `x-request-id` enables application log correlation.

#### Future: OTel Trace Context Propagation Implementation

When OTel is adopted, trace context propagation uses these patterns:

**Gateway (receiving HTTP, sending gRPC):**

```rust
// Receiving side: extract traceparent from HTTP headers
fn accept_trace(request: http::Request<Body>) -> http::Request<Body> {
    let parent_context = opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&opentelemetry_http::HeaderExtractor(request.headers()))
    });
    tracing::Span::current().set_parent(parent_context);
    request
}

// Sending side: inject traceparent into gRPC metadata
// Custom MetadataInjector needed because opentelemetry-http only handles http::HeaderMap
struct MetadataInjector<'a>(&'a mut tonic::metadata::MetadataMap);

impl opentelemetry::propagation::Injector for MetadataInjector<'_> {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(key) = tonic::metadata::MetadataKey::from_bytes(key.as_bytes()) {
            if let Ok(val) = tonic::metadata::MetadataValue::try_from(&value) {
                self.0.insert(key, val);
            }
        }
    }
}

fn inject_trace<T>(request: &mut tonic::Request<T>) {
    opentelemetry::global::get_text_map_propagator(|propagator| {
        let cx = tracing::Span::current().context();
        propagator.inject_context(&cx, &mut MetadataInjector(request.metadata_mut()));
    });
}
```

**Internal services (receiving gRPC):**

```rust
// Extract traceparent from gRPC metadata
struct MetadataExtractor<'a>(&'a tonic::metadata::MetadataMap);

impl opentelemetry::propagation::Extractor for MetadataExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }
    fn keys(&self) -> Vec<&str> {
        self.0.keys().filter_map(|k| match k {
            tonic::metadata::KeyRef::Ascii(k) => Some(k.as_str()),
            _ => None,
        }).collect()
    }
}

fn accept_trace(request: http::Request<Body>) -> http::Request<Body> {
    let parent_context = opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&MetadataExtractor(request.headers()))
        // Note: tonic gRPC metadata IS http::HeaderMap under the hood
        // So opentelemetry_http::HeaderExtractor also works here
    });
    tracing::Span::current().set_parent(parent_context);
    request
}
```

**This code lives in `madome-common` when OTel is adopted. No service code changes needed.**

**Confidence:** HIGH -- MetadataInjector/MetadataExtractor pattern verified against Heiko Seeberger's "Distributed Tracing Episode 3" and `opentelemetry-http` source code on GitHub. The `Injector`/`Extractor` trait implementations are minimal and well-documented.

### OTel Semantic Conventions Reference

For reference when OTel is adopted. These are the current stable attribute names (semantic conventions v1.40.0):

#### HTTP Spans (gateway)

| Attribute | Required | Example |
|-----------|----------|---------|
| `http.request.method` | Required | `"GET"` |
| `url.path` | Required (server) | `"/v1/users"` |
| `url.scheme` | Required (server) | `"https"` |
| `http.response.status_code` | Conditionally required | `200` |
| `http.route` | Conditionally required | `"/v1/users/{id}"` |
| `server.address` | Recommended (server) | `"api.madome.app"` |

#### gRPC Spans (internal services)

| Attribute | Required | Example |
|-----------|----------|---------|
| `rpc.system` | Required | `"grpc"` |
| `rpc.method` | Required (client) | `"CreateUser"` |
| `rpc.service` | Recommended | `"madome.user.UserService"` |
| `rpc.response.status_code` | Required | `"OK"` |
| `server.address` | Required (client) | `"localhost"` |
| `server.port` | Conditionally required | `50053` |

**These attributes will be added by OTel-specific middleware when adopted, NOT by manual field additions to `#[instrument]`.** Our current `fields(rpc = "CreateUser")` serves application logging and is separate from OTel semantic attributes.

**Confidence:** HIGH -- Verified against OpenTelemetry semantic conventions documentation at opentelemetry.io (v1.40.0).

### `#[instrument]` Compatibility with OTel

**Question:** Does `#[instrument]` work unchanged when an OTel layer is added?

**Answer: YES, completely unchanged.**

- `#[instrument]` creates tracing spans. The OTel layer observes these spans and converts them to OTel spans.
- Span names from `#[instrument]` become OTel span names (can be overridden with `otel.name` field if needed).
- Fields from `fields(...)` become OTel span attributes.
- `err` attribute works -- the OTel layer also supports `with_error_events_to_status()` to map ERROR-level events to OTel span status.
- No naming conventions break. The function-name-as-span-name convention works fine with OTel.

The only addition for OTel-aware spans is the `otel.kind` field (already addressed in Change 2 above).

**Confidence:** HIGH -- Verified against `tracing-opentelemetry` 0.32.1 docs. The layer implements `Layer<S>` trait methods (`on_new_span`, `on_record`, `on_event`, `on_close`) that translate tracing primitives to OTel primitives transparently.

### Feature Flag Strategy for OTel

**Decision: Do NOT use Cargo feature flags for OTel now.**

Reasoning:
1. Feature flags add compile-time complexity (conditional compilation, feature unions, testing matrix).
2. OTel adoption is an all-or-nothing decision per deployment, not a per-crate toggle.
3. When OTel is adopted, it will be added as regular dependencies to the workspace. The `init_tracing()` function will be modified in one place (`madome-common`).
4. If a feature flag is ever needed (e.g., to keep non-OTel builds lightweight), it can be added at adoption time with a simple `#[cfg(feature = "otel")]` wrapper around the OTel layer in `init_tracing()`.

**Confidence:** MEDIUM -- This is a judgment call. Feature flags are a valid approach but add complexity. The "add it when needed" approach is simpler and aligns with the project's current scale.

## What NOT to Do Now

| Temptation | Why Not | When to Do It |
|------------|---------|---------------|
| Add `opentelemetry` dependency | Adds 5+ crates to the dependency tree for unused functionality | When OTel backend is provisioned |
| Generate `traceparent` headers | Requires OTel SDK for correct generation; hand-rolling violates W3C spec | When OTel layer is added |
| Use OTel semantic convention field names in tracing | Verbose, couples logging to OTel, harder to read in console | When OTel middleware handles it automatically |
| Add `tracing-opentelemetry` dependency | No exporter configured = spans go nowhere | When OTel backend is provisioned |
| Use per-layer filtering | Adds complexity for no current benefit | When OTel and fmt need different filter levels |
| Add `otel.name` to every span | Default function names are sufficient; override only for ambiguous cases | When OTel trace visualization reveals naming issues |
| Build custom OTel middleware for axum/tonic | Community crates exist; premature to build before knowing OTel backend requirements | When OTel is adopted; evaluate `axum-tracing-opentelemetry` vs custom |

## Version Compatibility Matrix

When OTel is adopted, use these compatible versions (verified 2026-03-22):

| Crate | Version | Published | Notes |
|-------|---------|-----------|-------|
| `tracing` | 0.1 | Stable | Already in project |
| `tracing-subscriber` | 0.3.23 | 2026-03-13 | Add `registry` feature |
| `tracing-opentelemetry` | 0.32.1 | 2026-01-12 | Latest; requires `opentelemetry ^0.31` |
| `opentelemetry` | 0.31.0 | 2025-09-25 | Core API |
| `opentelemetry_sdk` | 0.31.0 | 2025-09-25 | Runtime SDK; use `rt-tokio` feature |
| `opentelemetry-otlp` | 0.31.0 | 2025-09-25 | OTLP exporter; use `tonic` feature |
| `opentelemetry-http` | 0.31.0 | 2025-09-25 | HeaderInjector/Extractor |

**Version rule:** `tracing-opentelemetry` is one major version ahead of `opentelemetry`. E.g., `tracing-opentelemetry` 0.32 requires `opentelemetry` 0.31. This is a known versioning convention in the Rust OTel ecosystem.

**Confidence:** HIGH -- All versions verified against crates.io API.

## Updated `init_tracing()` Design

### Current Implementation (services/*/src/main.rs)

```rust
// crates/madome-common/src/tracing.rs
use tracing_subscriber::{EnvFilter, fmt};

pub fn init_tracing(service_name: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("{service_name}=debug,madome=debug,info")));
    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .init();
}
```

### OTel-Ready Implementation (CHANGE TO)

```rust
// crates/madome-common/src/tracing.rs
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_tracing(service_name: &str) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("{service_name}=debug,madome=debug,info")));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}
```

### Key Imports

- `layer::SubscriberExt` -- provides `.with()` method on `Registry`
- `util::SubscriberInitExt` -- provides `.init()` method on the composed subscriber
- `fmt::layer()` -- produces a `Layer` (not a standalone `Subscriber`)

### Behavioral Equivalence

The output format, filtering behavior, and performance are identical. The only difference is internal structure: layers vs monolithic subscriber. Callers (`main.rs` in each service) need zero changes -- they still call `madome_common::tracing::init_tracing("service_name")`.

### Test Writer Compatibility

For integration tests, the pattern also changes to use Registry:

```rust
// Before:
let _ = tracing_subscriber::fmt()
    .with_env_filter("debug")
    .with_test_writer()
    .try_init();

// After:
let _ = tracing_subscriber::registry()
    .with(EnvFilter::new("debug"))
    .with(fmt::layer().with_test_writer())
    .try_init();
```

**Confidence:** HIGH -- Both patterns are documented in `tracing-subscriber` docs. The `fmt::layer()` method accepts the same builder methods as `fmt()`.

## Updated gRPC Handler Span Pattern

### Before (current TRACING-CONVENTIONS.md)

```rust
#[tracing::instrument(skip_all, fields(rpc = "CreateUser"))]
pub async fn handle<C: UserPorts>(
    ctx: &C,
    request: Request<CreateUserRequest>,
) -> Result<Response<UserResponse>, Status> { ... }
```

### After (OTel-ready)

```rust
#[tracing::instrument(skip_all, fields(otel.kind = "server", rpc = "CreateUser"))]
pub async fn handle<C: UserPorts>(
    ctx: &C,
    request: Request<CreateUserRequest>,
) -> Result<Response<UserResponse>, Status> { ... }
```

### Console Output Impact

With fmt layer only (no OTel), the `otel.kind` field appears as a regular field in log output:

```
2026-03-22T10:00:00Z INFO handle{otel.kind="server" rpc="CreateUser"}: user::app::handler ...
```

This is acceptable -- it adds minimal noise and provides useful context even without OTel. When the OTel layer is present, it intercepts the `otel.kind` field and does NOT pass it through as a regular attribute (it sets the SpanKind instead).

## Workspace Dependency Change Summary

**Immediate change (for OTel readiness):**

```toml
# In workspace Cargo.toml [workspace.dependencies]:
# Change:
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
# To:
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "registry"] }

# Also add (already documented in TRACING-CONVENTIONS.md):
tower-http = { version = "0.6", features = ["trace", "request-id", "util"] }
```

**Future change (when OTel is adopted):**

```toml
# Add to workspace Cargo.toml [workspace.dependencies]:
opentelemetry = "0.31"
opentelemetry_sdk = { version = "0.31", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.31", features = ["tonic"] }
tracing-opentelemetry = "0.32"
opentelemetry-http = "0.31"
```

## Common Pitfalls (OTel-Specific)

### Pitfall 1: Using `fmt().init()` Instead of Registry

**What goes wrong:** Cannot add OTel layer later without rewriting `init_tracing()`.
**Why it happens:** `fmt()` is simpler and more commonly shown in tutorials.
**How to avoid:** Always use `registry().with(env_filter).with(fmt_layer).init()`.
**Warning signs:** `init_tracing()` contains `fmt()` followed by `.init()`.

### Pitfall 2: Generating `traceparent` Without OTel SDK

**What goes wrong:** Invalid trace context headers that break OTel propagation.
**Why it happens:** Developers try to be "OTel-ready" by adding `traceparent` headers manually.
**How to avoid:** Use `x-request-id` for application correlation. Let the OTel SDK handle `traceparent` when enabled.
**Warning signs:** Hand-crafted `traceparent` strings with random hex values.

### Pitfall 3: Renaming Fields to Match OTel Semantic Conventions Prematurely

**What goes wrong:** Log output becomes verbose (`http.request.method` instead of `method`), code is coupled to OTel conventions, and the OTel layer still doesn't recognize them as semantic attributes (it just passes them through as custom attributes).
**Why it happens:** Confusion about how `tracing-opentelemetry` maps fields.
**How to avoid:** Keep application-friendly field names. OTel semantic attributes are added by OTel-specific middleware, not by renaming tracing fields.
**Warning signs:** Dotted field names in `#[instrument(fields(...))]`.

### Pitfall 4: Missing `registry` Feature Flag

**What goes wrong:** Compilation error: `registry()` function not found.
**Why it happens:** The `registry` feature is NOT in the default feature set of `tracing-subscriber`.
**How to avoid:** Explicitly add `"registry"` to features list.
**Warning signs:** Build fails after refactoring `init_tracing()` to use `registry()`.

### Pitfall 5: Forgetting OTel Shutdown

**What goes wrong:** Span data is lost because the OTel exporter has unflushed batches at process exit.
**Why it happens:** OTel batch exporter buffers spans and flushes periodically.
**How to avoid:** Call `opentelemetry::global::shutdown_tracer_provider()` before process exit (when OTel is adopted).
**Warning signs:** Missing spans in OTel backend, especially the last batch before shutdown.

## Open Questions

1. **`otel.kind` on fmt output noise**
   - What we know: Adding `otel.kind = "server"` to gRPC handler spans adds 20 characters to each log line.
   - What's unclear: Whether this noise bothers developers in practice.
   - Recommendation: Add it. The information is useful even without OTel (it documents the span's role in the call chain). If noisy, the fmt layer can be configured to exclude specific fields later.

2. **Per-layer filtering need**
   - What we know: Global `EnvFilter` is sufficient for now (same filter for console and future OTel).
   - What's unclear: Whether production will need different verbosity for console vs OTel export.
   - Recommendation: Defer per-layer filtering. Add when the need arises. The Registry architecture supports it without structural changes.

3. **`opentelemetry-http::HeaderExtractor` for tonic gRPC metadata**
   - What we know: tonic's `MetadataMap` is backed by `http::HeaderMap` internally. `HeaderExtractor` from `opentelemetry-http` should work directly on the underlying `HeaderMap`.
   - What's unclear: Whether this is guaranteed by tonic's API contract or an implementation detail.
   - Recommendation: When OTel is adopted, use the custom `MetadataExtractor` pattern (shown above) for safety. It is minimal code (10 lines) and does not depend on tonic internals.

## Sources

### Primary (HIGH confidence)
- [tracing-opentelemetry 0.32.1 docs.rs](https://docs.rs/tracing-opentelemetry/latest/tracing_opentelemetry/) - OpenTelemetryLayer API, otel.* field semantics, layer setup
- [tracing-opentelemetry OpenTelemetrySpanExt](https://docs.rs/tracing-opentelemetry/latest/tracing_opentelemetry/trait.OpenTelemetrySpanExt.html) - set_parent, context(), propagation API
- [tracing-subscriber layer docs](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/layer/index.html) - Layer composition, ordering, per-layer filtering
- [tracing-subscriber registry docs](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fn.registry.html) - Feature requirements, Registry usage
- [OpenTelemetry HTTP semantic conventions](https://opentelemetry.io/docs/specs/semconv/http/http-spans/) - Stable attribute names v1.40.0
- [OpenTelemetry gRPC semantic conventions](https://opentelemetry.io/docs/specs/semconv/rpc/grpc/) - rpc.system, rpc.method, status codes
- [W3C Trace Context spec](https://www.w3.org/TR/trace-context/) - traceparent format (version-trace_id-parent_id-trace_flags)
- [opentelemetry-http source](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-http/src/lib.rs) - HeaderInjector/HeaderExtractor implementations
- crates.io API -- version verification for all crate versions and dependency requirements

### Secondary (MEDIUM confidence)
- [Heiko Seeberger: Distributed Tracing Episode 3](https://heikoseeberger.de/2023-08-28-dist-tracing-3/) - MetadataInjector pattern for tonic, accept_trace/send_trace functions
- [broch.tech: Flexible Tracing with Rust and OpenTelemetry OTLP](https://broch.tech/posts/rust-tracing-opentelemetry/) - Registry + layer composition pattern, OTLP pipeline setup
- [OneUptime: tracing-subscriber with OpenTelemetry Layer](https://oneuptime.com/blog/post/2026-02-06-tracing-subscriber-opentelemetry-layer-rust/view) - Layer composition examples (Feb 2026)
- [OneUptime: OpenTelemetry Tracing in Rust](https://oneuptime.com/blog/post/2026-02-06-opentelemetry-tracing-rust-tracing-crate/view) - Dependency versions (Feb 2026)

### Tertiary (LOW confidence)
- None -- all findings verified with at least two sources.

## Metadata

**Confidence breakdown:**
- Subscriber pipeline refactor: HIGH -- Official tracing-subscriber docs, verified feature flags via crates.io
- OTel field mapping: HIGH -- tracing-opentelemetry docs.rs, otel.* prefix semantics confirmed
- Trace context propagation: HIGH -- W3C spec + opentelemetry-rust source code + community patterns
- x-request-id vs traceparent decision: HIGH -- W3C spec format analysis + OTel propagation architecture
- Version compatibility: HIGH -- All versions verified via crates.io API (2026-03-22)
- Feature flag strategy: MEDIUM -- Judgment call, reasonable but debatable

**Research date:** 2026-03-22
**Valid until:** 2026-06-22 (opentelemetry-rust releases roughly quarterly; check for new major versions before adopting)
