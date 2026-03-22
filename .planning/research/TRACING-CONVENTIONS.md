# Tracing Conventions Research

**Researched:** 2026-03-22
**Domain:** Rust structured tracing / observability conventions for microservice architecture
**Confidence:** HIGH

## Summary

This research investigates established conventions for structured tracing in Rust microservice architectures, specifically for a project using `tracing` 0.1 + `tracing-subscriber` 0.3 + `tower-http` 0.6 + `tonic` 0.14. The project has a gateway (axum REST) routing to internal gRPC services (auth, catalog, user), each following a 4-layer architecture (domain/usecase/app/adapter).

The core finding is that the Rust tracing ecosystem has clear, well-documented patterns for `#[instrument]`, span naming, request ID propagation, and TraceLayer customization. The main conventions are: use `#[instrument]` sparingly at boundaries (usecase and app layers, not domain or adapter), use `skip_all` with explicit `fields()` to control what gets recorded, and use tower-http's built-in `SetRequestId`/`PropagateRequestId` middleware rather than hand-rolling request ID propagation.

**Primary recommendation:** Codify conventions as prescriptive rules: instrument usecase functions and gRPC handlers with `#[instrument(skip_all, fields(...))]`, use `event = "entity.action"` naming in usecase events, use tower-http request ID middleware at the gateway, and propagate `x-request-id` via gRPC metadata to internal services.

## `#[instrument]` Convention

### Which Layers to Instrument

Based on the 4-layer architecture (domain/usecase/app/adapter), instrument at the **boundaries where work is dispatched**, not at every function:

| Layer | Instrument? | Rationale |
|-------|-------------|-----------|
| **usecase/** | YES | Business logic entry points. Each usecase function is a unit of work worth tracking. |
| **app/handler/** | YES | gRPC handler entry point. Creates the request-scoped span that wraps usecase execution. |
| **adapter/** | NO (default) | Repository calls are observable through the parent usecase span. Adding `#[instrument]` here creates noisy nested spans for every DB query. Exception: instrument only if the adapter does complex multi-step work (e.g., multi-query transactions). |
| **domain/** | NEVER | Pure types and trait definitions. No async work, no I/O. Nothing to trace. |
| **gateway routes/** | NO | The `TraceLayer` already creates an HTTP request span. Adding `#[instrument]` on route handlers would create a redundant child span. |
| **gateway middleware/** | NO | Middleware runs inside the TraceLayer span. Use `tracing::debug!()` for specific events if needed. |

**Confidence:** HIGH -- This aligns with the tracing crate documentation's guidance that `#[instrument]` creates spans for "units of work," and the Rust Compiler Development Guide's convention of instrumenting at function boundaries, not every helper.

### `skip_all` + Explicit Fields Pattern

Use `skip_all` by default, then re-add specific fields. This prevents accidentally logging large structs, sensitive data, or the `ctx` ports parameter.

```rust
// Usecase function
#[tracing::instrument(
    skip_all,
    fields(user_id = %payload.target_id)
)]
pub async fn deactivate_user(
    ctx: &(impl UserPorts + ?Sized),
    payload: DeactivateUserPayload,
) -> Result<User, UserError> { ... }

// gRPC handler
#[tracing::instrument(
    skip_all,
    fields(rpc = "CreateUser")
)]
pub async fn handle<C: UserPorts>(
    ctx: &C,
    request: Request<CreateUserRequest>,
) -> Result<Response<UserResponse>, Status> { ... }
```

**Why `skip_all` over selective `skip`:**
- `ctx` (ports object) has no useful Debug output
- Payload structs may contain PII or grow over time
- Explicit `fields()` documents exactly what enters the span
- Matches the pattern recommended by the official `#[instrument]` docs: "skip_all, add specific"

**Confidence:** HIGH -- Directly from `tracing::instrument` documentation: "skip_all...prevents any function arguments from being logged." Combined with `fields()` for explicit inclusion.

### Naming Convention

Use the default function name as the span name (do not set `name = "..."`) unless the function name is ambiguous (e.g., `handle` in handler modules, which should use `name = "create_user"` or the `fields(rpc = "CreateUser")` pattern to disambiguate).

For gRPC handlers that are all named `handle` (project convention: one handler per file), use `fields(rpc = "RpcMethodName")` to distinguish them in traces rather than renaming the span.

**Confidence:** HIGH -- Default span naming from function names is the standard convention per tracing docs.

### `err` Attribute for Error Tracing

Use `#[instrument(err)]` on usecase functions to automatically log errors at ERROR level when they return `Err`. This eliminates the need for manual `tracing::error!()` on each error path.

```rust
#[tracing::instrument(skip_all, fields(user_id = %payload.target_id), err)]
pub async fn change_role(
    ctx: &(impl UserPorts + ?Sized),
    payload: ChangeRolePayload,
) -> Result<User, UserError> { ... }
```

Do NOT use `err` on gRPC handlers -- the handler maps domain errors to `tonic::Status`, which is not an "error" in the tracing sense (4xx responses are expected behavior). The usecase is the right place to log domain errors.

**Confidence:** HIGH -- Official `#[instrument]` documentation explicitly covers `err` and `err(level = ...)`.

## Event Naming Convention

### Pattern: `entity.past_tense_verb`

The project already uses `event = "user.created"` and `event = "user.role_changed"` in usecase functions. This dot-notation pattern is well-established:

```
entity.action_completed
```

| Example | Use When |
|---------|----------|
| `event = "user.created"` | Entity created successfully |
| `event = "user.deactivated"` | State change completed |
| `event = "user.role_changed"` | Mutation with audit significance |
| `event = "session.invalidated"` | Auth lifecycle event |
| `event = "book.published"` | Catalog state transition |

Rules:
- **Past tense** for completed actions (`created`, not `create`)
- **Dot separator** between entity and action (`user.created`, not `user_created`)
- **Only in usecase layer** -- events represent completed business operations
- **Only for state changes** -- read operations (get, list) do not emit events

### Events vs Spans

Events (`tracing::info!(event = "...")`) record **point-in-time facts**.
Spans (`#[instrument]`) track **duration of work**.

Do not use events to log "entering function" or "exiting function" -- that is what spans do.

**Confidence:** HIGH -- The project already follows this pattern consistently. The tracing docs explicitly distinguish events (point-in-time) from spans (duration).

## Log Level Guidelines

| Level | Use For | Examples |
|-------|---------|---------|
| **ERROR** | Failures requiring attention. Server-side bugs, unrecoverable states, infrastructure failures. | DB connection lost, migration failure, panic recovery |
| **WARN** | Unexpected but handled conditions. Not bugs, but worth monitoring. | Approaching rate limit, deprecated API usage, retry succeeded after failure |
| **INFO** | Normal operational events. State changes, service lifecycle, completed business operations. | Service started, user created, request completed, gRPC connection established |
| **DEBUG** | Diagnostic detail for development. Request/response content, intermediate computation steps. | Parsed request payload, SQL query generated, cache hit/miss |
| **TRACE** | Very verbose. Span enter/exit, individual field values, iteration steps. | Not used in production. Only for local debugging. |

### HTTP/gRPC Status Code Classification

| Status | Level | Rationale |
|--------|-------|-----------|
| 2xx / OK | INFO (via TraceLayer) | Successful operation |
| 4xx / InvalidArgument, NotFound, PermissionDenied, Unauthenticated | WARN or DEBUG | Client error -- expected behavior, not a bug. Use WARN for auth failures (potential attack), DEBUG for validation errors. |
| 5xx / Internal, Unavailable, Unknown | ERROR | Server error -- unexpected, needs investigation |

The key principle: **4xx is the client's problem, 5xx is our problem**. Do not use ERROR for 4xx responses.

For `#[instrument(err)]` on usecases: domain errors that map to 4xx (UserNotFound, InvalidHandle, etc.) will be logged at ERROR by the `err` attribute. This is acceptable because at the usecase boundary we do not yet know the HTTP status -- the handler maps them. If the noise is excessive, use `err(level = Level::WARN)` on usecases that frequently return expected errors (like validation).

**Confidence:** HIGH -- tower-http TraceLayer docs explicitly separate HTTP and gRPC classifiers. The 4xx=WARN/5xx=ERROR convention is standard across web service frameworks.

## Structured Field Standards

### Standard Field Names

| Field | Format | Use In | Example |
|-------|--------|--------|---------|
| `request_id` | UUIDv7 string | Gateway TraceLayer span | `request_id = %id` |
| `user_id` | UUID string | Usecase events | `user_id = %saved.id` |
| `event` | `entity.action` | Usecase events | `event = "user.created"` |
| `actor_id` | UUID string | Audit events (who performed action) | `actor_id = %caller_id` |
| `target_id` | UUID string | Audit events (who was affected) | `target_id = %target_id` |
| `rpc` | PascalCase RPC name | gRPC handler spans | `rpc = "CreateUser"` |
| `method` | HTTP method | Gateway span (auto) | `method = "GET"` |
| `uri` | Request URI | Gateway span (auto) | `uri = "/v1/users"` |
| `status` | HTTP status code | Gateway span (auto) | `status = 200` |
| `latency_ms` | Milliseconds | Gateway on_response (auto) | `latency_ms = 42` |
| `service` | Service name | Init tracing | Set via `RUST_LOG` target |

### Field Formatting Rules

- Use `%` (Display) for IDs, strings, and human-readable values: `user_id = %id`
- Use `?` (Debug) only for complex types during development: `request = ?req`
- Never use Debug format in INFO+ level events in production code
- Structured fields over string interpolation: `tracing::info!(user_id = %id, "created")` not `tracing::info!("created user {id}")`

**Confidence:** HIGH -- Directly from tracing crate documentation on field formatting syntax.

## TraceLayer Configuration

### Gateway (axum HTTP)

Replace the bare `TraceLayer::new_for_http()` with a customized version that includes request ID:

```rust
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tower_http::request_id::{
    MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer,
};
use http::Request;
use tracing::Level;

// Custom MakeRequestId using UUIDv7 (time-sortable, per PROJECT.md)
#[derive(Clone)]
struct MakeRequestUuidV7;

impl MakeRequestId for MakeRequestUuidV7 {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let id = uuid::Uuid::now_v7().to_string().parse().unwrap();
        Some(RequestId::new(id))
    }
}

// In main.rs or router setup:
use madome_common::headers;

let x_request_id = http::HeaderName::from_static(headers::X_REQUEST_ID);

let app = routes::create_router(app_state)
    .layer(PropagateRequestIdLayer::new(x_request_id.clone()))
    .layer(
        TraceLayer::new_for_http()
            .make_span_with(|request: &Request<_>| {
                let request_id = request
                    .headers()
                    .get("x-request-id")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("unknown");

                tracing::info_span!(
                    "http_request",
                    method = %request.method(),
                    uri = %request.uri(),
                    request_id = %request_id,
                )
            })
            .on_response(
                DefaultOnResponse::new()
                    .level(Level::INFO)
                    .include_headers(false),
            ),
    )
    .layer(SetRequestIdLayer::new(
        x_request_id,
        MakeRequestUuidV7,
    ));
```

**Layer ordering matters:** `SetRequestId` must run first (bottom of stack), then `TraceLayer` reads it, then `PropagateRequestId` copies it to responses. In axum, layers are applied bottom-up, so the order in code is: `PropagateRequestId`, `TraceLayer`, `SetRequestId`.

**Confidence:** HIGH -- tower-http docs explicitly state: "make sure to set request ids before the request reaches TraceLayer."

### Internal Services (tonic gRPC)

Use `TraceLayer::new_for_grpc()` on tonic servers:

```rust
use tower_http::trace::TraceLayer;

Server::builder()
    .layer(
        TraceLayer::new_for_grpc()
            .make_span_with(|request: &http::Request<_>| {
                let request_id = request
                    .headers()
                    .get("x-request-id")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("none");

                tracing::info_span!(
                    "grpc_request",
                    request_id = %request_id,
                    service = "user",
                )
            }),
    )
    .add_service(UserServiceServer::new(handler))
    .serve(addr)
    .await?;
```

Note: `TraceLayer::new_for_grpc()` uses `GrpcErrorsAsFailures` classifier which reads the `grpc-status` header, unlike `new_for_http()` which reads HTTP status codes. Use the correct variant for each service type.

**Confidence:** HIGH -- tower-http docs explicitly distinguish `new_for_http()` and `new_for_grpc()`. The tower-rs/tower-http#366 issue confirms the closure signature pattern works.

## Cross-Service Correlation

### Request ID Propagation: Gateway to gRPC to Service

The `x-request-id` header set by the gateway's `SetRequestIdLayer` needs to reach internal gRPC services. Since the gateway already has `CallerContext.inject_into()` for caller headers, extend this pattern:

**Step 1: Gateway generates request ID** (via `SetRequestIdLayer` + `MakeRequestUuidV7`)

**Step 2: Gateway propagates to gRPC calls.** The `CallerContext` middleware already extracts headers. Add `x-request-id` extraction and injection:

```rust
// In CallerContext or a separate RequestContext:
impl CallerContext {
    pub fn inject_into<T>(&self, request: &mut tonic::Request<T>) {
        let metadata = request.metadata_mut();
        // existing caller_id and caller_role injection...

        // Add request_id propagation
        if let Some(ref request_id) = self.request_id {
            if let Ok(val) = request_id.parse() {
                metadata.insert(headers::X_REQUEST_ID, val);
            }
        }
    }
}
```

**Step 3: Internal service reads request ID from gRPC metadata** (via `TraceLayer::new_for_grpc().make_span_with(...)` as shown above).

This approach does NOT require OpenTelemetry. The request ID is a simple string header propagated through existing infrastructure. When OpenTelemetry is added later, the `x-request-id` pattern can coexist with W3C `traceparent` headers.

**Confidence:** HIGH -- The project already propagates `x-caller-id` and `x-caller-role` via the exact same mechanism. Adding `x-request-id` is identical.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Request ID generation | Custom middleware with `Uuid::now_v7()` in handler | `tower_http::request_id::SetRequestIdLayer` + custom `MakeRequestId` impl | Handles edge cases: won't override existing IDs, proper layer ordering, header management |
| Request ID response propagation | Manual header copy in each handler | `tower_http::request_id::PropagateRequestIdLayer` | Automatic, no per-handler code |
| HTTP request/response logging | Manual `tracing::info!()` in each handler | `tower_http::trace::TraceLayer::new_for_http()` | Handles timing, status codes, span lifecycle |
| gRPC request logging | Manual logging in each handler | `tower_http::trace::TraceLayer::new_for_grpc()` | Understands gRPC status codes, streaming |
| Span creation for handlers | Manual `tracing::info_span!()` + guard | `#[instrument]` attribute | Handles async correctly, automatic span exit |
| Log subscriber init | Custom subscriber pipeline | `tracing_subscriber::fmt()` with `EnvFilter` | Production-tested defaults, env-based control |

**Key insight:** tower-http provides purpose-built middleware for HTTP/gRPC observability. Custom logging code in handlers is a code smell -- it means the middleware layer is missing or misconfigured.

## Common Pitfalls

### Pitfall 1: Instrumenting Too Many Layers

**What goes wrong:** Adding `#[instrument]` to domain types, adapter repo methods, and handlers creates deeply nested span trees that are expensive to process and hard to read.
**Why it happens:** Developers add tracing "just in case" without considering the span hierarchy.
**How to avoid:** Instrument only usecase functions and gRPC handlers. Let the TraceLayer handle HTTP/gRPC boundaries. Adapter operations are visible through timing within the parent span.
**Warning signs:** Log output shows 4+ levels of span nesting for a simple CRUD operation.

### Pitfall 2: Logging Sensitive Data

**What goes wrong:** Passwords, tokens, session IDs, or PII appear in logs because `#[instrument]` records all function arguments by default.
**Why it happens:** Default `#[instrument]` behavior without `skip_all`.
**How to avoid:** Always use `skip_all` + explicit `fields()`. Never record: passwords, tokens, session data, full request bodies, email addresses.
**Warning signs:** Debug-format structs in INFO-level spans. Arguments with names like `password`, `token`, `secret` appearing in trace output.

### Pitfall 3: Double Error Logging

**What goes wrong:** An error is logged by the usecase (`tracing::error!()` or `#[instrument(err)]`), then again by the handler, then again by the TraceLayer's `on_failure`.
**Why it happens:** Each layer independently decides to log errors without knowing the others already did.
**How to avoid:** Log errors at exactly ONE layer -- the usecase (via `#[instrument(err)]`). The handler maps errors to Status/Response codes. The TraceLayer records the status code, not the error message. Error is logged once (usecase), classified once (TraceLayer).
**Warning signs:** The same error message appears 2-3 times in log output at different span depths.

### Pitfall 4: Wrong TraceLayer Variant

**What goes wrong:** Using `TraceLayer::new_for_http()` on a tonic gRPC server results in all responses being classified as "success" because HTTP status is 200 even when gRPC status indicates failure (gRPC uses the `grpc-status` header/trailer).
**Why it happens:** Copy-paste from HTTP examples.
**How to avoid:** Use `new_for_http()` for axum, `new_for_grpc()` for tonic. They use different classifiers.
**Warning signs:** gRPC error responses show as "200 OK" in traces.

### Pitfall 5: TraceLayer Order vs SetRequestIdLayer

**What goes wrong:** Request ID is not included in traces because TraceLayer runs before SetRequestIdLayer.
**Why it happens:** Layer ordering in axum is bottom-up (last `.layer()` call runs first).
**How to avoid:** In code order: `PropagateRequestIdLayer` (top), `TraceLayer` (middle), `SetRequestIdLayer` (bottom). The bottom runs first.
**Warning signs:** `request_id = "unknown"` in all traces.

### Pitfall 6: `#[instrument]` on `handle` Functions Without Disambiguation

**What goes wrong:** All gRPC handler spans show as `handle` in traces, making it impossible to distinguish which RPC was called.
**Why it happens:** The project convention is one `pub async fn handle(...)` per handler file (e.g., `create_user::handle`, `change_role::handle`). The module path is not included in the span name by default.
**How to avoid:** Use `fields(rpc = "CreateUser")` or `name = "create_user"` to disambiguate.
**Warning signs:** Multiple spans all named `handle` in a trace tree.

## Code Examples

### Usecase Function (with `#[instrument]` + event)

```rust
// Source: Project convention based on tracing docs
use tracing::instrument;

#[instrument(skip_all, fields(user_id = %payload.target_id), err)]
pub async fn deactivate_user(
    ctx: &(impl UserPorts + ?Sized),
    payload: DeactivateUserPayload,
) -> Result<User, UserError> {
    if payload.caller_id == payload.target_id {
        return Err(UserError::SelfModification);
    }

    let mut target = ctx
        .user_repo()
        .find_by_id(payload.target_id)
        .await?
        .ok_or(UserError::UserNotFound)?;

    if !payload.caller_role.can_manage(target.role) {
        return Err(UserError::InsufficientRole { /* ... */ });
    }

    target.is_active = false;
    target.updated_at = Utc::now();
    let updated = ctx.user_repo().update(&target).await?;

    tracing::info!(
        event = "user.deactivated",
        actor_id = %payload.caller_id,
        target_id = %payload.target_id,
    );

    Ok(updated)
}
```

### gRPC Handler (with `#[instrument]`)

```rust
// Source: Project convention based on tracing docs
use tracing::instrument;

#[instrument(skip_all, fields(rpc = "ChangeRole"))]
pub async fn handle<C: UserPorts>(
    ctx: &C,
    request: Request<ChangeRoleRequest>,
) -> Result<Response<UserResponse>, Status> {
    let caller_ctx = extract_caller_context(&request)?;
    let req = request.into_inner();
    // ... delegate to usecase ...
}
```

### Gateway TraceLayer Setup

```rust
// Source: tower-http docs + project convention
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tower_http::request_id::{
    MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer,
};
use tracing::Level;

#[derive(Clone)]
struct MakeRequestUuidV7;

impl MakeRequestId for MakeRequestUuidV7 {
    fn make_request_id<B>(&mut self, _request: &http::Request<B>) -> Option<RequestId> {
        let id = uuid::Uuid::now_v7().to_string().parse().unwrap();
        Some(RequestId::new(id))
    }
}

// Layer order in code (bottom-up execution):
let x_request_id = http::HeaderName::from_static(headers::X_REQUEST_ID);

let app = routes::create_router(app_state)
    .layer(PropagateRequestIdLayer::new(x_request_id.clone()))
    .layer(
        TraceLayer::new_for_http()
            .make_span_with(|request: &http::Request<_>| {
                let request_id = request
                    .headers()
                    .get("x-request-id")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("unknown");

                tracing::info_span!(
                    "http_request",
                    method = %request.method(),
                    uri = %request.uri(),
                    request_id = %request_id,
                )
            })
            .on_response(
                DefaultOnResponse::new()
                    .level(Level::INFO)
                    .include_headers(false),
            ),
    )
    .layer(SetRequestIdLayer::new(x_request_id, MakeRequestUuidV7));
```

### tonic Server with TraceLayer

```rust
// Source: tower-http docs (new_for_grpc)
Server::builder()
    .layer(
        TraceLayer::new_for_grpc()
            .make_span_with(|request: &http::Request<_>| {
                let request_id = request
                    .headers()
                    .get("x-request-id")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("none");

                tracing::info_span!(
                    "grpc_request",
                    request_id = %request_id,
                    service = "user",
                )
            }),
    )
    .add_service(UserServiceServer::new(handler))
    .serve(addr)
    .await?;
```

### Startup/Shutdown Events

```rust
// In main.rs -- use info level for lifecycle events
tracing::info!(%addr, "gateway starting");
tracing::info!("connecting to database");
tracing::info!("running migrations");
tracing::info!(%addr, "user service starting");
// These are already correct in the codebase.
```

## Testing with Tracing

### Unit Tests

Tracing events in unit tests are safe to ignore -- they go to the global subscriber (or nowhere if none is set). The `#[instrument]` attribute and `tracing::info!()` calls do not affect test behavior.

If asserting on trace output is needed (rare), use `tracing-test` crate:

```rust
// Only add if specific tracing behavior needs verification
#[cfg(test)]
use tracing_test::traced_test;

#[traced_test]
#[tokio::test]
async fn should_emit_user_created_event() {
    // ... test logic ...
    assert!(logs_contain("user.created"));
}
```

For most tests, tracing assertions are unnecessary -- test the business logic return values instead.

### Integration/Service Tests

Use `test-log` crate or manual `tracing_subscriber::fmt().with_test_writer().try_init()` to see trace output during test runs without polluting parallel test output:

```rust
// In test setup (once per test binary)
let _ = tracing_subscriber::fmt()
    .with_env_filter("debug")
    .with_test_writer()
    .try_init();
```

`try_init()` is important because multiple tests may attempt initialization; it silently succeeds on the first call and is a no-op on subsequent calls.

**Confidence:** HIGH -- `try_init()` behavior is documented in tracing-subscriber. `tracing-test` is the standard crate for trace assertions.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `log` crate | `tracing` crate | 2019+ | Structured spans + events, async-aware |
| Manual request ID in handlers | `tower_http::request_id` middleware | tower-http 0.3+ | Automatic, per-request, layer-based |
| `TraceLayer::new_for_http()` for gRPC | `TraceLayer::new_for_grpc()` | tower-http 0.2+ | Correct gRPC status classification |
| `#[instrument]` on everything | `#[instrument(skip_all, fields(...))]` | Community convention | Prevents PII leaks, reduces noise |

**Not yet needed (defer):**
- `tracing-opentelemetry` -- For when OpenTelemetry backend is set up. Current structured logging to stdout is sufficient.
- `tracing-subscriber` JSON output -- For when log aggregation is set up. Human-readable format is better for development.
- `tracing-error` / `SpanTrace` -- For enhanced error context. Add when error diagnosis becomes a pain point.

## Required Dependency Changes

To implement these conventions, the following workspace dependency changes are needed:

```toml
# In workspace Cargo.toml [workspace.dependencies]:
tower-http = { version = "0.6", features = ["trace", "request-id", "util"] }

# "request-id" enables SetRequestIdLayer, PropagateRequestIdLayer, MakeRequestId
# "util" enables ServiceBuilderExt convenience methods
# "trace" is already enabled
```

The `uuid` crate already has `v7` feature enabled in the workspace.

## Open Questions

1. **`#[instrument(err)]` noise level for validation errors**
   - What we know: `err` logs at ERROR level by default. Validation failures (InvalidHandle, HandleTaken) are expected and frequent.
   - What's unclear: Whether the volume of ERROR-level validation logs will be annoying in practice.
   - Recommendation: Start with `err` on all usecases. If validation errors dominate logs, switch to `err(level = Level::WARN)` on validation-heavy usecases (create_user, update_user). Monitor and adjust.

2. **gRPC method name in TraceLayer span**
   - What we know: The `make_span_with` closure receives the HTTP request. gRPC method is encoded in the URI path (e.g., `/madome.user.UserService/CreateUser`).
   - What's unclear: Whether parsing the URI path to extract the method name is worth the complexity in the TraceLayer.
   - Recommendation: Do not parse -- the handler's `#[instrument(fields(rpc = "CreateUser"))]` already provides this. The TraceLayer span just needs `request_id` and `service`.

3. **CallerContext request_id extraction**
   - What we know: The gateway's `CallerContext` middleware extracts `x-caller-id` and `x-caller-role`. Adding `x-request-id` extraction is straightforward.
   - What's unclear: Whether `request_id` belongs in `CallerContext` (caller identity) or a separate middleware.
   - Recommendation: Add `request_id: Option<String>` to `CallerContext` for simplicity. It is request-scoped context, same as caller identity. If it grows further, extract a separate `RequestContext`.

## Sources

### Primary (HIGH confidence)
- [tracing docs](https://docs.rs/tracing/latest/tracing/) - `#[instrument]` attribute, event macros, span API
- [tracing::instrument docs](https://docs.rs/tracing/latest/tracing/attr.instrument.html) - skip_all, fields, err, ret, name, level
- [tower-http trace module](https://docs.rs/tower-http/latest/tower_http/trace/index.html) - TraceLayer, DefaultMakeSpan, new_for_http vs new_for_grpc
- [tower-http request_id module](https://docs.rs/tower-http/latest/tower_http/request_id/index.html) - SetRequestIdLayer, PropagateRequestIdLayer, MakeRequestId
- [tower-http TraceLayer](https://docs.rs/tower-http/latest/tower_http/trace/struct.TraceLayer.html) - Builder methods, classifier types
- [tracing-subscriber docs](https://docs.rs/tracing-subscriber) - EnvFilter, fmt subscriber

### Secondary (MEDIUM confidence)
- [axum discussion #2273](https://github.com/tokio-rs/axum/discussions/2273) - Request ID in TraceLayer span, confirmed by maintainer
- [tower-http issue #366](https://github.com/tower-rs/tower-http/issues/366) - gRPC TraceLayer custom methods, closure type fix
- [Luca Palmieri: Zero to Production](https://www.lpalmieri.com/posts/2020-09-27-zero-to-production-4-are-we-observable-yet/) - Span structure, request ID, log level guidance
- [Heiko Seeberger: Distributed Tracing Episode 3](https://heikoseeberger.de/2023-08-28-dist-tracing-3/) - gRPC trace context propagation, MetadataInjector
- [Rust Compiler Dev Guide: Tracing](https://rustc-dev-guide.rust-lang.org/tracing.html) - `#[instrument(level = "debug")]` convention, field formatting
- [tower-http source: request_id.rs](https://github.com/tower-rs/tower-http/blob/main/tower-http/src/request_id.rs) - MakeRequestUuid uses v4, confirmed need for custom v7 impl

### Tertiary (LOW confidence)
- None -- all findings verified with at least two sources.

## Metadata

**Confidence breakdown:**
- `#[instrument]` convention: HIGH -- Official tracing docs + Rust compiler convention + community consensus
- Event naming: HIGH -- Already established in project, consistent with structured logging best practices
- Log levels: HIGH -- Standard across web frameworks, tower-http docs
- TraceLayer configuration: HIGH -- Official tower-http docs with working code examples
- Cross-service correlation: HIGH -- Reuses existing CallerContext pattern, tower-http request_id module
- Error tracing: HIGH -- `#[instrument(err)]` well-documented, double-logging avoidance is established pattern
- Testing: HIGH -- tracing-test and try_init() are documented, community-standard

**Research date:** 2026-03-22
**Valid until:** 2026-06-22 (stable ecosystem -- tracing 0.1 and tower-http 0.6 are mature)
