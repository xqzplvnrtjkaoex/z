---
paths: ["**/rpc/**/*.rs", "**/routes/**/*.rs", "**/model/**/*.rs", "**/payload/**/*.rs"]
---

# Request Handling Conventions

## One Handler Per File

Each handler lives in its own file. Name files by the full domain action including the entity:
`create_user.rs`, `get_user.rs`, `activate_user.rs` — not `create.rs`, `get.rs`, `lifecycle.rs`.
Never group multiple handlers into a single file.

## Handler Function Naming

Names match the proto RPC action. Don't force CRUD verbs where the domain action has a specific name:
- `change_role` (matches `ChangeRole` RPC) — not `update_role`
- `activate_user` (matches `ActivateUser` RPC) — not `update_user_status`

Related types (payload structs, etc.) must use the same verb: `ChangeRoleBody`, not `UpdateUserRoleBody`.

| Layer | Function name | Example |
|-------|---------------|---------|
| **gRPC `app/rpc/`** | `execute` (module name provides context) | `create_user::execute(...)` |
| **Gateway `routes/`** | Same as file name (axum needs a named function) | `get_me::get_me(...)` |

## gRPC Service Structure

`{Service}Handler` lives in `app/mod.rs` — it implements the tonic service trait and delegates each RPC to `app/rpc/{action}::execute(...)`. gRPC-specific helpers (metadata extraction, proto conversion) live in `app/rpc/mod.rs`.

```
app/
  mod.rs          ← UserHandler + UserService impl
  rpc/
    mod.rs        ← gRPC helpers (CallerContext, proto conversion)
    create_user.rs
    get_user.rs
    ...
```

## Cross-Cutting Concerns as Middleware

Logic that repeats identically across multiple handlers (authentication context injection, request ID propagation, logging decoration, etc.) belongs in a middleware / interceptor / layer — not in a helper function called manually per handler.

## Gateway Module Structure

Gateway has its own type system for the REST-gRPC translation layer. gRPC services do NOT need this — proto defines their types.

| Module | Purpose | Example |
|--------|---------|---------|
| `model/` | Response/view types with proper Rust types | `model::User`, `model::UserRole` |
| `payload/` | Request/input types (deserialized from client) | `ListUsersQuery`, `ChangeRoleBody` |
| `util/` | Shared serialization helpers | `to_rfc3339_ms` custom serializer |
| `routes/{entity}/` | One handler per file | `get_me.rs`, `change_role.rs` |

### Model Types

Use proper Rust types, not String wrappers:
- `Uuid` not `String` for IDs
- `DateTime<Utc>` not `String` for timestamps
- Domain enums (`UserRole`) not `String` for enumerated values

### Serialization Conventions

| Target | `rename_all` | Example |
|--------|-------------|---------|
| Struct field names (query) | `kebab-case` | `?include-inactive=true` |
| Struct field names (JSON body) | `snake_case` | `{ "is_active": true }` |
| Enum variants (all contexts) | `kebab-case` | `"image-set"`, `?kind=image-set` |

Enum variants use `kebab-case` universally so `model/` enums can be reused in both body and query payloads without serialization conflicts.

### Type Sharing Between `model/` and `payload/`

`payload/` types may reference `model/` enums directly (e.g., `model::UserRole` in `ChangeRoleBody`). This works because enum variant serialization is self-contained — determined by the enum's own `rename_all`, not the parent struct's.

Do NOT share structs across contexts with different `rename_all` strategies (e.g., `#[serde(flatten)]` a `snake_case` struct into a `kebab-case` query type) — serde has no override mechanism.

### Gateway Response Building

Use axum's tuple response `(AppendHeaders, Json)` instead of manual `Response::builder()` + `serde_json::to_string`. Manual building is verbose and loses axum's built-in Content-Type handling.

```rust
// Good
Ok((AppendHeaders([(X_NEXT_CURSOR, next_cursor)]), Json(users)))

// Bad
let mut resp = Response::builder().header(CONTENT_TYPE, "application/json");
let body = serde_json::to_string(&users)?;
resp.body(Body::from(body))
```

### Query String Parsing

Use `serde_qs::axum::QsQuery` instead of axum's `Query` extractor. `serde_qs` supports nested/complex query strings that axum's built-in extractor cannot handle.
