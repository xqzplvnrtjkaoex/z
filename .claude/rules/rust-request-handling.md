---
paths: ["**/handler/**/*.rs", "**/routes/**/*.rs"]
---

# Request Handling Conventions

## One Handler Per File

Each handler lives in its own file. Name files by the full domain action including the entity:
`create_user.rs`, `get_user.rs`, `activate_user.rs` — not `create.rs`, `get.rs`, `lifecycle.rs`.
Never group multiple handlers into a single file.

## Cross-Cutting Concerns as Middleware

Logic that repeats identically across multiple handlers (authentication context injection, request ID propagation, logging decoration, etc.) belongs in a middleware / interceptor / layer — not in a helper function called manually per handler.
