# Tracing Conventions

## `#[instrument]` Rules

### Where to Apply

| Layer | Instrument? | Why |
|-------|-------------|-----|
| **usecase/** | YES | Business logic entry point — unit of work worth tracking |
| **app/handler/** | YES | gRPC handler entry — request-scoped span wrapping usecase |
| **adapter/** | NO (default) | Observable through parent span. Exception: complex multi-step transactions |
| **domain/** | NEVER | Pure types, no I/O |
| **gateway routes/** | NO | `TraceLayer` already creates HTTP span |
| **gateway middleware/** | NO | Runs inside TraceLayer span |

### Pattern: `skip_all` + Explicit Fields

Always use `skip_all` and re-add specific fields. Prevents PII leaks and large struct logging.

```rust
// Usecase function
#[tracing::instrument(skip_all, fields(user_id = %payload.target_id), err)]
pub async fn deactivate_user(
    ctx: &(impl UserPorts + ?Sized),
    payload: DeactivateUserPayload,
) -> Result<User, UserError> { ... }

// gRPC handler (all named `handle` — disambiguate with `rpc` field)
#[tracing::instrument(skip_all, fields(otel.kind = "server", rpc = "DeactivateUser"))]
pub async fn handle<C: UserPorts>(
    ctx: &C,
    request: Request<DeactivateUserRequest>,
) -> Result<Response<UserResponse>, Status> { ... }
```

### Error Tracing

- Use `err` on **usecase functions** — automatically logs errors at ERROR level on `Err` return.
- Do NOT use `err` on **gRPC handlers** — they map domain errors to `tonic::Status` (not a system error).
- This prevents double-logging: error logged once (usecase), classified once (TraceLayer).

## Event Naming

Pattern: `event = "entity.past_tense_verb"`

```rust
tracing::info!(event = "user.created", user_id = %saved.id, handle = %saved.handle);
tracing::info!(event = "user.role_changed", actor_id = %caller_id, target_id = %target_id);
tracing::info!(event = "book.published", book_id = %id);
```

Rules:
- Past tense for completed actions (`created`, not `create`)
- Dot separator (`user.created`, not `user_created`)
- Only in usecase layer — events represent completed business operations
- Only for state changes — reads (get, list) do not emit events

## Log Levels

| Level | Use For |
|-------|---------|
| ERROR | Server-side failures, unrecoverable states, infrastructure failures (5xx) |
| WARN | Unexpected but handled conditions (approaching rate limit, deprecated usage) |
| INFO | Normal operations, state changes, service lifecycle, completed business events |
| DEBUG | Diagnostic detail (parsed payloads, SQL queries, cache hit/miss) |
| TRACE | Very verbose, local debugging only |

**4xx is the client's problem, 5xx is our problem.** Never use ERROR for 4xx responses.

## Structured Fields

- Use `%` (Display) for IDs and strings: `user_id = %id`
- Use `?` (Debug) only for development diagnostics, never in INFO+ production events
- Structured fields over string interpolation: `tracing::info!(user_id = %id, "created")` not `tracing::info!("created user {id}")`

### Standard Field Names

| Field | Format | Layer |
|-------|--------|-------|
| `event` | `"entity.action"` | usecase events |
| `user_id` | UUID Display | usecase |
| `actor_id` | UUID Display | audit events |
| `target_id` | UUID Display | audit events |
| `rpc` | PascalCase RPC name | gRPC handler span |
| `otel.kind` | `"server"` | gRPC handler span |
| `request_id` | UUIDv7 Display | gateway TraceLayer span |

## OTel Readiness

- `init_tracing()` MUST use `registry().with(filter).with(fmt_layer).init()` — NOT `fmt().init()`. This enables adding an OTel layer as a one-line change.
- `otel.kind = "server"` in gRPC handler spans — ignored by fmt layer, used by OTel layer for SpanKind.
- Do NOT generate `traceparent` headers without OTel SDK. Use `x-request-id` for application correlation.
- Do NOT rename fields to OTel semantic convention names (`http.request.method`). OTel middleware adds these separately.

## Subscriber Setup

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
