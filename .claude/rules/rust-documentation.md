---
paths: ["*.rs"]
---

# Documentation Conventions

## Code Comments

### When to Comment

Comment only when the code alone does not explain **why**. Do not comment what the code does — rename or restructure to make it obvious.

```
Is the code self-explanatory?
  YES → No comment
  NO  → Does it explain WHAT?
          YES → Improve naming instead, no comment
          NO  → Does it explain WHY (non-obvious business logic)?
                  YES → Keep. Plain English, no external references.
                  NO  → Is it a WORKAROUND or GOTCHA?
                          YES → Keep. e.g. "// Workaround: sea-orm wraps..."
                          NO  → No comment
```

### Forbidden Patterns

- **External numbering references**: `// D-12: ...`, `// REQ-03: ...` — design decisions live in `.planning/`, not in code
- **Narrating the code**: `// Check if user is active` above `if !user.is_active`
- **Restating the type system**: `// Returns a User` above `-> Result<User, UserError>`
- **Duplicating `#[error("...")]`**: error variant messages are already documentation
- **Section separators**: `// --- Role conversions ---` — use modules instead
- **Commented-out code** — use version control

### Good Comment Examples

```rust
// Admins can see inactive users for reactivation, but regular users
// should not know whether a handle belongs to a deactivated account.
if !user.is_active && !is_admin_or_owner {
    return Err(UserError::UserNotFound);
}

// Caller must outrank both the target's current role AND the new role
if !caller_role.can_manage(&target.role) {
    return Err(UserError::InsufficientRole);
}

// Workaround: sea-orm wraps sqlx errors without a stable unique-violation variant
if err_msg.contains("duplicate key") || err_msg.contains("23505") { ... }
```

## Rustdoc (`///`)

### Per-Layer Rules

| Layer | `///` rustdoc | Guideline |
|-------|--------------|-----------|
| **domain ports** (traits) | YES | Contract boundary. Include `# Errors` section |
| **domain validation** (`HandleInput`) | YES | State validation rules (constraints are business logic) |
| **domain types** (`User`, `UserRole`) | Selective | Only non-obvious semantics (e.g. `can_manage` hierarchy rule) |
| **domain errors** | NO | `#[error("...")]` is sufficient |
| **usecase functions** | Selective | Only when behavior is surprising or combines non-obvious rules |
| **usecase payloads** | NO | Field names are self-documenting |
| **gRPC handlers** | NO | Delegates to usecase; trait docs cover the contract |
| **Gateway route handlers** | YES | `///` becomes OpenAPI summary/description via utoipa |
| **Gateway middleware** | YES | Behavior (permissive vs mandatory) affects all downstream |
| **adapters** | NO | Trait docs cover the contract; add inline `//` only for complex queries |

### Summary Line Style

First line = rustdoc summary. Third person singular present indicative:

```rust
/// Validates a user handle: 4-15 alphanumeric/underscore chars, not reserved.
pub struct HandleInput { ... }

/// Persists a new user. Returns the saved user with server-generated fields.
///
/// # Errors
///
/// Returns [`RepositoryError::UniqueViolation`] if the handle already exists.
async fn save(&self, user: &User) -> Result<User, RepositoryError>;
```

### `# Sections` in Doc Comments

| Section | When |
|---------|------|
| `# Errors` | Functions returning `Result` — list error conditions |
| `# Panics` | Functions that can panic (prefer `Result` instead) |
| `# Examples` | Shared crate public APIs only (`madome-core`, `madome-common`) |

### `//!` Module-Level Docs

Use for **what this module contains** + **how it fits in the architecture** (one-liner each). Do NOT repeat information from CLAUDE.md or PROJECT.md.

```rust
//! User profile business logic.
//!
//! Each function receives a payload and operates through [`crate::domain::ports::UserPorts`].
```

Skip `//!` for modules that only re-export sub-modules.

## SSOT (Single Source of Truth)

1. **Document at the definition, link elsewhere** — use intra-doc links: [`HandleInput`], [`UserError::SelfModification`]
2. **Never duplicate error documentation** — `#[error("...")]` is the single source
3. **Never duplicate architecture docs in code** — the explanation lives in `.planning/PROJECT.md`
4. **Cross-crate links work within workspace** — [`madome_core::error::AppError`] resolves with `cargo doc --workspace`

### `#![warn(missing_docs)]`

| Crate type | Apply? | Why |
|------------|--------|-----|
| Shared (`madome-core`, `madome-common`, `madome-proto`) | YES | Consumed by multiple services |
| Service binaries (`auth`, `catalog`, `user`, `gateway`) | NO | Binaries — document selectively |

## API Documentation (OpenAPI)

### Approach

- **utoipa v5** with `#[utoipa::path]` + `#[derive(ToSchema)]` for compile-time annotations
- **Feature-gated** (`openapi` feature) — all utoipa attributes use `#[cfg_attr(feature = "openapi", ...)]` so the runtime binary has zero OpenAPI code
- **Static generation** via a separate `gen-openapi` binary (`--features openapi`), not runtime serving
- **Deployed to GitHub Pages** via CI — spec is not committed to the repo
- Do NOT use `utoipa-axum` (`OpenApiRouter`) — it is for runtime serving

### Annotation Pattern

```rust
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UserJson { ... }

#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/v1/users/@me",
    tag = "users",
    responses((status = 200, body = UserJson)),
))]
pub async fn get_me(...) -> Result<impl IntoResponse, AppError> { ... }
```

### Rules

- `path` must be the **full absolute path** (e.g. `/v1/users/{id}`, not `/{id}`)
- `///` doc comments on Gateway handlers become OpenAPI summary (first line) and description (rest)
- Always specify `body = TypeName` in `responses(...)` — `impl IntoResponse` is opaque to utoipa
- When adding a handler with `#[utoipa::path]`, add it to `paths(...)` in `#[derive(OpenApi)]`

## `cargo doc` Integration

```bash
# CI (master push) — validates docs compile and intra-doc links resolve
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

# Local — includes private items
cargo doc --workspace --no-deps --document-private-items --open
```
