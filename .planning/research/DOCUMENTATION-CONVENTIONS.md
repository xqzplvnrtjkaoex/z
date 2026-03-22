# Rust Code Documentation Conventions - Research

**Researched:** 2026-03-23
**Domain:** Rust documentation (rustdoc, code comments, SSOT strategy)
**Confidence:** HIGH

## Summary

This research establishes documentation conventions for the madome project -- a Rust workspace with multiple services following a 4-layer architecture (domain/usecase/app/adapter). The project currently has 16 D-n prefix comments (e.g., `// D-12: Validate name via NameInput struct`) that reference design decisions from CONTEXT.md files, and 41 rustdoc (`///`) comments across 8 files with low adoption.

The primary goal is to replace the D-n pattern with self-explanatory code and targeted comments, adopt a per-layer documentation strategy, and enforce SSOT (Single Source of Truth) by using references instead of duplication.

**Primary recommendation:** Remove all D-n prefix comments. Most are redundant with self-documenting code (well-named error variants, clear method names). For the few that encode non-obvious "why" rationale, replace with plain `// Why:` comments that explain the reason inline without referencing external numbering systems.

## Rustdoc Strategy

### What Gets `///` Doc Comments

Based on the Rust API Guidelines (C-CRATE-DOC, C-EXAMPLE, C-LINK) and RFC 1574, the guiding principle is: **document what is not obvious from the type signature**.

| Item Type | Document? | Rationale |
|-----------|-----------|-----------|
| Public trait definitions | YES | Contract that implementors must understand |
| Public trait methods | YES (brief) | Parameters and return behavior not obvious from signature |
| Public struct fields (domain types) | ONLY if non-obvious | `User.handle` vs `User.id` -- handle's semantics need context |
| Public enum variants | ONLY if non-obvious | `UserError::SelfModification` is self-explanatory; skip |
| Public free functions | YES for complex ones | Simple CRUD helpers are obvious; business rule functions need docs |
| Type aliases | YES | The "why" of the alias matters more than the "what" |
| Constants | YES (brief) | What the constant represents, not what the value is |
| Private/`pub(crate)` items | Selective | Only when the "what" or "why" is not obvious from context |

### What Does NOT Get `///` Doc Comments

- Trivial getters/setters where the name says everything
- `From` trait implementations (the types already document the conversion)
- `Display`/`FromStr` implementations
- Test helper functions (`make_user`, `TestContext`)
- Module re-exports (`pub mod X;`) unless the module needs introduction
- Items where the `#[error("...")]` or type signature already explains behavior

### Summary Line Convention (RFC 1574)

The first line of any doc comment is used by rustdoc as the summary in module indices. Follow these rules:

- Write in **third person singular present indicative**: "Returns", "Validates", "Extracts" (not "Return", "Validate")
- Keep to a **single line** before the first blank line
- Focus on **what**, not **how**: `/// Extracts caller identity from gRPC metadata.` (not `/// Uses metadata().get() to parse headers...`)

### `//!` Module-Level Docs

Use `//!` at the top of `mod.rs` or `lib.rs` files to describe:

- **What this module/crate contains** (one-liner)
- **How it fits in the architecture** (one sentence)
- **Key types or functions** the reader should look at first

Do NOT use `//!` for:
- Repeating information available in CLAUDE.md or PROJECT.md
- Lengthy architecture explanations (link to planning docs instead)
- Module files that only re-export sub-modules (`pub mod X;` -- the child modules document themselves)

### When to Use `# Sections` in Doc Comments

Per RFC 1574 and Rust API Guidelines:

| Section | When to Include |
|---------|-----------------|
| `# Errors` | Functions returning `Result` -- list error conditions |
| `# Panics` | Functions that can panic (rare in this project -- prefer `Result`) |
| `# Examples` | Library crate public APIs; skip for internal service code |
| `# Safety` | `unsafe` blocks only (not applicable currently) |

**For this project:** `# Errors` on usecase functions and port trait methods. `# Examples` on shared crate (`madome-common`, `madome-core`) public APIs only. Internal service code does not need `# Examples` sections -- tests serve as living documentation.

## Per-Layer Guidelines

### domain/ Layer

| Item | Document? | Why |
|------|-----------|-----|
| Domain types (`User`, `UserRole`) | NO (usually) | Field names and types are self-documenting. Document only non-obvious semantics |
| `UserRole::can_manage()` | YES | The "strictly greater" rule is a business invariant worth stating |
| `UserRole::level()` | YES (brief) | Hierarchy mapping is an arbitrary design choice |
| Error enums (`UserError`) | NO | `#[error("...")]` messages already document each variant |
| Port traits (`UserRepository`) | YES | Contract boundary -- document expected behavior, not implementation |
| Port trait methods | YES (brief) | `# Errors` section listing which `RepositoryError` variants are possible |
| Validation types (`HandleInput`) | YES | Validation rules are business constraints worth documenting inline |

**Example -- good domain doc:**
```rust
pub trait UserRepository {
    /// Persists a new user. Returns the saved user with server-generated fields.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::UniqueViolation`] if the handle already exists.
    async fn save(&self, user: &User) -> Result<User, RepositoryError>;
}
```

### usecase/ Layer

| Item | Document? | Why |
|------|-----------|-----|
| Payload structs | NO | Fields are self-documenting (named clearly) |
| Usecase functions | Selective | Only when business rules are non-obvious from the code flow |
| Internal comments | YES (selectively) | For non-obvious "why" logic, not for "what" |

Most usecase functions in this project are short and self-explanatory. The error variant names (`SelfModification`, `OwnerRoleRejected`, `InsufficientRole`) and method names (`can_manage`) already communicate the business rules. A `///` doc on `create_user` adding "Creates a user" is noise.

**Document when:** The function enforces a non-obvious invariant, combines multiple rules in a specific order that matters, or has surprising behavior.

**Example -- when to skip:**
```rust
// The function name, error variants, and method names tell the full story.
// No doc comment needed.
pub async fn deactivate_user(
    ctx: &(impl UserPorts + ?Sized),
    payload: DeactivateUserPayload,
) -> Result<User, UserError> { ... }
```

**Example -- when to document:**
```rust
/// Looks up a user by handle, hiding inactive users from non-admin callers.
///
/// Admin and owner callers see all users. Regular users and anonymous
/// callers receive [`UserError::UserNotFound`] for inactive users
/// (indistinguishable from a non-existent handle).
pub async fn get_user_by_handle(...) -> Result<User, UserError> { ... }
```

### app/handler/ Layer

| Item | Document? | Why |
|------|-----------|-----|
| `UserHandler` struct | NO | Generic handler pattern is documented in PROJECT.md |
| `UserService` trait impl | NO | Generated by tonic; methods delegate to handlers |
| Helper functions (`extract_caller_context`, `user_to_response`) | YES (brief) | Utility functions used across multiple handlers |
| Conversion functions | NO | `domain_role_to_proto` -- the types say it all |

### adapter/ Layer

| Item | Document? | Why |
|------|-----------|-----|
| Concrete types (`PostgresUserRepository`) | NO | It implements the trait; the trait docs cover the contract |
| `From` implementations | NO | Source and target types document the conversion |
| Complex query logic | YES (inline `//`) | Only when the SQL/ORM pattern is non-obvious |

### gateway/ Layer (routes + middleware)

| Item | Document? | Why |
|------|-----------|-----|
| Middleware functions | YES | Behavior (permissive vs. mandatory) affects all downstream handlers |
| Route handler functions | NO | Routes are obvious from the function name + HTTP method |
| `AppState` | NO | Field types document what clients are available |

## SSOT with Intra-doc Links

### How Intra-doc Links Work

Intra-doc links let you reference Rust items by path, avoiding duplicated descriptions:

```rust
/// Validates the handle field. See [`HandleInput`] for validation rules.
pub fn validate_handle(&self) -> Result<(), String> { ... }
```

Syntax:
- `[`TypeName`]` -- links to a type in scope
- `[`module::TypeName`]` -- links to a type in another module
- `[`TypeName::method`]` -- links to a method
- `[`crate::domain::types::role::UserRole`]` -- absolute path from crate root

### Disambiguation Prefixes

When names collide (rare in this project), use prefixes:
- `[struct@Foo]`, `[enum@Bar]`, `[trait@Baz]`, `[fn@func]`
- Function: `[func()]` (append parens)
- Macro: `[mac!]` (append bang)

### SSOT Rules for This Project

1. **Document the definition, reference it elsewhere.** Define validation rules on `HandleInput`; reference with `[`HandleInput`]` in `create_user` if needed.
2. **Do not duplicate error documentation.** `UserError` variants have `#[error("...")]` -- handlers should link to `[`UserError`]`, not restate error messages.
3. **Do not duplicate architecture docs.** Module-level `//!` can state "Follows 4-layer architecture" but MUST NOT re-explain the pattern. The explanation lives in `.planning/PROJECT.md`.
4. **Cross-crate links work within the workspace.** `[`madome_core::error::AppError`]` resolves when running `cargo doc --workspace`. Use this for shared crate references.

### Practical Application

```rust
//! User profile business logic.
//!
//! Each module contains a single usecase function that receives a payload
//! and operates through [`crate::domain::ports::UserPorts`].

pub mod create_user;
pub mod deactivate_user;
// ...
```

### Limitation: Cross-workspace Links

Intra-doc links work within the Cargo workspace (`cargo doc --workspace`). They do NOT link to external crates by path -- for those, use URL links or let rustdoc's auto-linking handle standard library types.

## Replacing D-n Comments

### Current State Analysis

The project has 16 D-n comments across 8 usecase files. Categorizing them:

**Category 1: Redundant with self-documenting code (REMOVE, no replacement)**

These comments restate what the code already says through clear variable names, method names, and error variants:

| Comment | Why Redundant |
|---------|---------------|
| `// D-20: Self-modification blocked` | Code: `caller_id == target_id` + `Err(UserError::SelfModification)` |
| `// D-21: Owner role cannot be assigned via API` | Code: `new_role == UserRole::Owner` + `Err(UserError::OwnerRoleRejected)` |
| `// D-60: Reject owner role assignment via API` | Same as D-21 |
| `// D-12: Validate handle via HandleInput struct` | Code: `HandleInput::new(&payload.handle).validate_handle()` |
| `// D-12: Validate name via NameInput struct` | Code: `NameInput::new(&payload.name).validate_name()` |
| `// D-12: Validate handle` | Same as above |
| `// D-12: Validate name` | Same as above |
| `// D-58: Structured tracing for audit` | Code: `tracing::info!(event = "user.deactivated", ...)` |
| `// D-53: Always return user regardless of is_active` | Code: function returns `Ok(user)` without checking `is_active` |

**Category 2: Encode "why" logic that is partially non-obvious (REPLACE with plain comment)**

These have business logic rationale that is worth preserving but should be stated in plain English:

| D-n Comment | Replacement |
|-------------|-------------|
| `// D-19: Role hierarchy check` | No comment needed -- `can_manage()` already encodes this. If kept: `// Caller must outrank target to manage them` |
| `// D-18: Caller must have strictly greater role than target's current role` | Already a good comment; just drop the `D-18:` prefix |
| `// D-18: Caller must also have strictly greater role than the new role` | Same: drop prefix, the text is useful |
| `// D-55: Hide inactive users from non-admin callers` | Already a good comment; just drop the `D-55:` prefix |
| `// D-19: Role hierarchy check (same rule as deactivate)` | `// Same hierarchy check as deactivate_user` or remove entirely |

**Category 3: Rustdoc comments with D-n references (REWRITE)**

| Current | Replacement |
|---------|-------------|
| `/// Input struct for handle validation per D-06~D-09.` | `/// Validates a user handle: 4-15 alphanumeric/underscore chars, not reserved.` |
| `/// Uses #[derive(Validate)] for declarative struct-level validation per D-12.` | Remove (implementation detail visible in the derive attribute) |
| `/// Input struct for name validation per D-10~D-11.` | `/// Validates a display name: 1-20 Unicode characters.` |
| `/// Caller context extracted from gRPC metadata (D-23).` | `/// Caller context extracted from gRPC metadata.` |

### Replacement Strategy

**Rule: No external numbering systems in code comments.**

Design decisions live in `.planning/phases/*/CONTEXT.md`. Code should be understandable without reading planning docs. If a comment is only comprehensible with external context, either:

1. Make the code self-explanatory (rename, extract method, use descriptive error variants)
2. Write the rationale inline in plain English

**When design rationale matters in code**, use this pattern:

```rust
// Why: Admins can see inactive users to manage reactivation, but regular
// users should not know whether a handle belongs to a deactivated account.
if !user.is_active && !is_admin_or_owner {
    return Err(UserError::UserNotFound);
}
```

The `// Why:` prefix is reserved for non-obvious business rationale. It is never used for "what" comments.

### How Large Rust Projects Handle Design Rationale

From studying tokio, axum, and serde source code patterns:

1. **tokio:** Uses inline comments for "why" explanations (`// We need to do X because Y`). No external reference system. Complex state machine logic gets paragraph-length comments explaining invariants.

2. **axum:** Minimal inline comments. Relies on clear type names, trait names, and doc comments on public APIs. Implementation is expected to be readable from the code itself.

3. **serde:** Heavy doc comments on public API. Internal code uses `// Note:` and `// SAFETY:` prefixes for critical invariants. No external design doc references.

**Common pattern:** Large projects keep rationale comments close to the code, written in plain English, with no external numbering. Architecture decisions live in RFCs, ADRs, or design docs that are referenced in commit messages, not in line-by-line code comments.

## Comment Decision Framework

Use this decision tree for every comment:

```
Is the code self-explanatory without this comment?
  YES --> Delete the comment
  NO  --> Does the comment explain WHAT the code does?
            YES --> Rename variables/functions/types to make it obvious, then delete
            NO  --> Does the comment explain WHY (non-obvious business logic)?
                      YES --> Keep it. Use plain English, no external references.
                      NO  --> Does the comment explain a WORKAROUND or GOTCHA?
                                YES --> Keep it. Prefix with "// Note:" or "// Workaround:"
                                NO  --> Delete the comment
```

### Comments That Add Value

- **Business invariant rationale:** Why a particular check exists when it is not obvious from the error variant name
- **Non-obvious ordering:** When the order of operations matters and reordering would break correctness
- **Workarounds:** When the code does something unusual because of a library limitation or upstream bug
- **Performance notes:** When a non-obvious implementation choice was made for performance reasons
- **Security notes:** When a check exists for security reasons that are not obvious from the domain

### Comments That Are Noise

- Restating the code in English: `// Check if user is active` above `if !user.is_active`
- Restating the type system: `// Returns a User` above `-> Result<User, UserError>`
- Section separators that module structure handles: `// --- Role conversions ---`
- Referencing external numbering: `// D-12: ...`
- "TODO" without a tracking issue or deadline
- Commented-out code (use version control)

## Anti-patterns

### 1. External Reference Comments (D-n Pattern)

**Why bad:** Comments become opaque without the external document. The D-n numbers are meaningless to someone reading the code without access to the CONTEXT.md. As decisions evolve, the D-n references go stale (a decision might be revised in a later phase, but the old D-n comment remains).

**Replace with:** Self-documenting code or plain English "why" comments.

### 2. Narrating the Code

```rust
// BAD:
// D-20: Self-modification blocked
if payload.caller_id == payload.target_id {
    return Err(UserError::SelfModification);
}

// GOOD: (no comment needed -- code is self-explanatory)
if payload.caller_id == payload.target_id {
    return Err(UserError::SelfModification);
}
```

### 3. Documenting Implementation in Trait Docs

```rust
// BAD:
/// Uses #[derive(Validate)] for declarative struct-level validation per D-12.

// GOOD:
/// Validates a user handle: 4-15 alphanumeric/underscore chars, not reserved.
```

The implementation strategy (`#[derive(Validate)]`) is visible in the code. The doc should describe the *contract*, not the *mechanism*.

### 4. Duplicating `#[error("...")]` in Doc Comments

```rust
// BAD:
/// Error when user is not found.
#[error("user not found")]
UserNotFound,

// GOOD: (no doc comment needed -- #[error] serves as documentation)
#[error("user not found")]
UserNotFound,
```

### 5. Section Separator Comments

```rust
// BAD:
// --- Role conversions ---

// GOOD: (let module structure handle organization; or use no separator)
```

If you need separators, it suggests the module is doing too much. Extract into sub-modules.

### 6. Doc Comments on Private Items That Mirror Public Docs

Do not repeat documentation on `pub(crate)` items that simply delegate to documented public items. A handler that calls a documented usecase function does not need its own copy of the business rules.

## `cargo doc` Integration

### Workspace Documentation

```bash
# Generate docs for all workspace crates
cargo doc --workspace --no-deps

# Include private items (useful for internal services)
cargo doc --workspace --no-deps --document-private-items

# Open in browser
cargo doc --workspace --no-deps --open
```

For this project, `--document-private-items` is useful since the service binaries have `pub(crate)` items that are part of the internal API. The CI runs `cargo doc --no-deps` (without `--document-private-items`) on master push -- this validates that public API docs compile and link correctly.

### Useful Attributes

| Attribute | Use Case |
|-----------|----------|
| `#[doc(hidden)]` | Hide re-exports or implementation details from generated docs |
| `#![doc = include_str!("../README.md")]` | Use crate README as lib.rs top-level doc (for library crates) |
| `#[doc(alias = "...")]` | Add search aliases for items |
| `#[warn(missing_docs)]` | Enforce docs on public items (for library crates) |

### Recommendations for This Project

- **Shared crates** (`madome-core`, `madome-common`, `madome-proto`): Apply `#![warn(missing_docs)]` since they are consumed by multiple services.
- **Service crates** (`auth`, `catalog`, `user`, `gateway`): Do NOT apply `#![warn(missing_docs)]` -- these are binaries, not libraries. Document selectively per the layer guidelines above.
- **`#[doc(hidden)]`**: Use on SeaORM schema types if they leak into public API through re-exports.
- **`include_str!`**: Not needed for this project -- service crates do not need README-as-docs.

### Broken Link Detection

Add to CI or use locally:

```bash
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

This fails the build on broken intra-doc links, ensuring SSOT references stay valid.

## References

### Primary (HIGH confidence)
- [Rust API Guidelines - Documentation](https://rust-lang.github.io/api-guidelines/documentation.html) -- C-CRATE-DOC, C-EXAMPLE, C-QUESTION-MARK, C-FAILURE, C-LINK checklist items
- [Rust API Guidelines - Checklist](https://rust-lang.github.io/api-guidelines/checklist.html) -- Full checklist of documentation guidelines
- [RFC 1574 - More API Documentation Conventions](https://rust-lang.github.io/rfcs/1574-more-api-documentation-conventions.html) -- Summary line style, `# Panics`/`# Errors`/`# Examples`/`# Safety` sections
- [RFC 505 - API Comment Conventions](https://rust-lang.github.io/rfcs/0505-api-comment-conventions.html) -- Original comment conventions RFC
- [The rustdoc book - How to Write Documentation](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html) -- `///` vs `//!`, markdown, code examples
- [The rustdoc book - Linking to Items by Name](https://doc.rust-lang.org/rustdoc/write-documentation/linking-to-items-by-name.html) -- Intra-doc link syntax and resolution rules

### Secondary (MEDIUM confidence)
- [Effective Rust - Item 27: Document Public Interfaces](https://effective-rust.com/documentation.html) -- When and how to document
- [Code Comment Anti-Patterns](https://bytedev.medium.com/code-comment-anti-patterns-and-why-the-comment-you-just-wrote-is-probably-not-needed-919a92cf6758) -- Rotting comments, mumbling comments, noise comments

### Project-Specific (HIGH confidence)
- Codebase analysis: 16 D-n comments, 41 rustdoc comments, current patterns examined across all source files
- `.planning/phases/02-user-profile/02-CONTEXT.md` -- Source of D-n decision numbering system (D-01 through D-66)

---
*Researched: 2026-03-23*
*Valid until: indefinite (conventions are stable)*
