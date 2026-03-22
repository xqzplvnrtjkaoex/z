# Phase 2: User Profile - Research

**Researched:** 2026-03-22
**Domain:** gRPC CRUD service with PostgreSQL, 4-layer architecture, role hierarchy enforcement
**Confidence:** HIGH

## Summary

Phase 2 introduces the first service with a real database. The User service manages user records (create, get, list, update, deactivate, activate, role change) via gRPC, following the 4-layer architecture pattern (domain/usecase/app/adapter). This phase establishes patterns that all subsequent services (auth, catalog) will follow: sea-orm entities and migrations, testcontainers integration testing, mockall-based unit testing, and the Ports/Config trait separation pattern.

The primary technical challenge is correctly implementing the `UserPorts` trait with RPITIT accessors while maintaining mockall compatibility for unit tests. Mockall cannot directly mock traits with RPITIT methods (`fn user_repo(&self) -> &impl UserRepository`), so the testing strategy must use a concrete `TestContext` struct that holds `MockUserRepository` and implements `UserPorts` manually. This is a known limitation and the recommended pattern.

The secondary challenge is establishing the PostgreSQL infrastructure: docker-compose for dev environment, sea-orm entity definitions with PostgreSQL enum types, migration files, and testcontainers for integration tests.

**Primary recommendation:** Implement the 4-layer architecture with `trait_variant` for async port traits (e.g., `UserRepository`), use `automock` on individual port traits (not on `UserPorts`), build a manual `TestContext` for unit tests, and use `testcontainers` + `testcontainers-modules` for integration tests with real PostgreSQL.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01~D-05:** Users table schema: `id` (UUIDv4 PK), `handle` (unique, URL-safe), `name` (display name, non-unique), `role` (owner/admin/user enum), `is_active` (boolean), `created_at`, `updated_at`
- **D-06~D-09:** Handle validation: 4-15 chars, alphanumeric + underscore, case-insensitive uniqueness, reserved handles list
- **D-10~D-12:** Name validation: 1-20 chars by `chars().count()`, Unicode allowed, `validator` crate
- **D-13~D-15:** Self-service mutations: PATCH /v1/users/@me, protected tier, admin cannot change others' names/handles
- **D-16~D-23:** Role hierarchy enforcement in User service usecase layer, not Gateway. Caller context via gRPC metadata
- **D-24~D-27:** gRPC RPCs: CreateUser, GetUser, GetUserByHandle, ListUsers, UpdateUser, DeactivateUser, ActivateUser, ChangeRole
- **D-28~D-33:** 4-layer architecture, UserPorts + UserConfig traits, UserRepository single trait, usecase functions, UserError + RepositoryError enums
- **D-34~D-37:** PostgreSQL via docker-compose, sea-orm with auto-migration, testcontainers, per-service database
- **D-38~D-41:** Unit tests with mockall, integration tests with testcontainers, service tests with tonic in-process channel, BDD naming
- **D-42~D-47:** Gateway REST endpoints: /v1/users with @me and :id route groups, nest-based tier separation
- **D-48~D-50:** Handle/name change: immediate release, no cooldown, no history
- **D-51~D-58:** Deactivated user visibility: ListUsers defaults active-only, GetUser always returns, GetUserByHandle admin-gated for inactive, structured tracing for audit
- **D-59~D-62:** Owner seed deferred to Phase 3, CreateUser rejects role=owner, default role is user
- **D-63~D-66:** Single UserResponse shape, admin-only /v1/users/:id, gRPC-to-REST pure translation, single proto message type

### Claude's Discretion
- Proto message structures (request/response types for each RPC)
- Exact sea-orm entity definitions and migration file structure
- docker-compose configuration details
- UserConfig trait contents (what config does user service need?)
- Exact index types based on query patterns
- Pagination cursor implementation for ListUsers
- UserRepository trait method signatures
- Error code string constants
- Test parallelism strategy

### Deferred Ideas (OUT OF SCOPE)
- Profile picture upload -- future phase (requires file service)
- User bio / about me -- future phase
- Collection sharing via handle URL -- future phase
- User deletion (permanent) -- v2
- User search/discovery -- future phase
- Activity timestamps (last_login, last_active) -- future phase
- Public profile endpoint `/v1/users/@handle` -- future phase
- Audit query API via Loki HTTP API wrapping -- future phase
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| USER-PROFILE-01 | User CRUD via gRPC (create, get, get by handle, list, update) | sea-orm entity/migration for users table, gRPC proto RPCs (D-24), 4-layer architecture pattern, UserRepository trait, validator crate for handle/name validation |
| USER-PROFILE-02 | User lifecycle management (deactivate, activate, role change) with role hierarchy enforcement | Usecase-layer role hierarchy checks (D-16~D-22), caller context via gRPC metadata (D-23), UserError enum for business errors (D-32), structured tracing for audit events (D-58) |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| sea-orm | 1.1.19 | ORM for PostgreSQL | Async-first Rust ORM, mature ecosystem, DeriveEntityModel macros |
| sea-orm-migration | 1.1.19 | Database migrations | Paired with sea-orm, programmatic migrations in Rust |
| tonic | 0.14 | gRPC server implementation | Already in workspace, standard Rust gRPC framework |
| prost | 0.14 | Protobuf serialization | Already in workspace, paired with tonic |
| validator | 0.20.0 | Struct validation | Declarative derive-based validation per D-12 |
| trait-variant | 0.1.2 | Async trait Send variant | RPITIT-compatible async trait generation per D-28 |
| chrono | 0.4.44 | DateTime types | Timestamp handling for created_at/updated_at |
| uuid | 1.x | UUIDv4 generation | Already in workspace, user ID generation |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| mockall | 0.14.0 | Mock generation | Unit tests: mock UserRepository trait |
| testcontainers | 0.27.1 | Docker container management | Integration tests: real PostgreSQL |
| testcontainers-modules | 0.15.0 | Pre-built container images | PostgreSQL module for testcontainers |
| async-trait | 0.1.x | Async trait fallback | Only if trait_variant insufficient for specific pattern |
| tokio | 1.50 | Async runtime | Already in workspace |
| tracing | 0.1 | Structured logging | Already in workspace, audit event recording |
| serde | 1.x | Serialization | Already in workspace |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| sea-orm 1.1.19 | sea-orm 2.0.0-rc.37 | 2.0 is release candidate, not stable; 1.1.19 is proven and matches project skills |
| validator | garde | garde is newer with better error types, but validator is the locked decision (D-12) |
| trait-variant | async-trait | async-trait adds runtime overhead (Box<dyn Future>); trait-variant is zero-cost |
| chrono | time | chrono has broader sea-orm integration via `with-chrono` feature |

**Installation (workspace Cargo.toml additions):**
```toml
# ORM
sea-orm = { version = "1.1", features = ["sqlx-postgres", "runtime-tokio-rustls", "macros", "with-chrono", "with-uuid"] }
sea-orm-migration = { version = "1.1", features = ["sqlx-postgres", "runtime-tokio-rustls"] }

# Validation
validator = { version = "0.20", features = ["derive"] }

# DateTime
chrono = { version = "0.4", features = ["serde"] }

# Async traits
trait-variant = "0.1"

# Testing
mockall = "0.14"
testcontainers = "0.27"
testcontainers-modules = { version = "0.15", features = ["postgres"] }
```

**Version verification:** Versions confirmed against crates.io registry as of 2026-03-22. sea-orm 1.1.19 is the latest 1.x stable. sea-orm 2.0 is in RC status and should not be used for production.

## Architecture Patterns

### Recommended Project Structure (User Service)
```
services/user/
  schema/
    mod.rs
    users.rs              # sea-orm entity: Model, ActiveModel, Column, Relation
  migration/
    mod.rs                # Migrator struct, lists all migrations
    m20260322_000001_create_users_table.rs  # Initial migration
  src/
    main.rs               # Composition root: DB connect, migrate, assemble context, start server
    lib.rs                # Re-exports for integration test access
    domain/
      mod.rs
      types/
        mod.rs
        user.rs           # User domain type (separate from sea-orm Model)
        role.rs           # UserRole enum (domain + sea-orm ActiveEnum + proto mapping)
      ports/
        mod.rs            # UserPorts trait (RPITIT accessors)
        user_repository.rs # trait UserRepository (async methods)
      error/
        mod.rs
        user_error.rs     # UserError enum (business errors)
        repository_error.rs # RepositoryError enum (data-access errors)
    usecase/
      mod.rs
      create_user.rs
      get_user.rs
      get_user_by_handle.rs
      list_users.rs
      update_user.rs
      deactivate_user.rs
      activate_user.rs
      change_role.rs
    app/
      mod.rs
      handler/
        mod.rs            # UserHandler<C: UserPorts> impl UserService
        create.rs
        get.rs
        list.rs
        update.rs
        lifecycle.rs      # deactivate, activate, change_role RPCs
    adapter/
      mod.rs
      context.rs          # UserContext: impl UserPorts + UserConfig
      postgres/
        mod.rs
        user_repository.rs # impl UserRepository for PostgresUserRepository
```

### Pattern 1: Ports/Config Trait Separation with RPITIT
**What:** The UserPorts trait provides accessor methods returning `&impl Trait` references (RPITIT). UserConfig provides configuration accessors. UserContext implements both.
**When to use:** All services following the 4-layer architecture.
**Example:**
```rust
// domain/ports/mod.rs
use super::super::domain::ports::user_repository::UserRepository;

pub trait UserPorts {
    fn user_repo(&self) -> &impl UserRepository;
}

pub trait UserConfig {
    // UserConfig is minimal in Phase 2; service needs DB connection only
    // which is encapsulated in the adapter, not exposed as config
}

// Compound type alias for usecase function bounds
pub type Context = impl UserPorts + ?Sized;
```

```rust
// usecase/create_user.rs
use crate::domain::ports::Context;
use crate::domain::error::user_error::UserError;

pub async fn create_user(
    ctx: &Context,
    payload: CreateUserPayload,
) -> Result<User, UserError> {
    // Validation, business logic, then:
    ctx.user_repo().save(/* ... */).await
        .map_err(|e| /* map RepositoryError to UserError */)
}
```

### Pattern 2: Port Traits with trait_variant for Send
**What:** Individual port traits (e.g., UserRepository) use `trait_variant::make` to generate a Send-bound variant, enabling use in multithreaded tokio runtime while keeping the trait mockable.
**When to use:** All async port traits that need mockall compatibility.
**Example:**
```rust
// domain/ports/user_repository.rs
use crate::domain::error::repository_error::RepositoryError;
use crate::domain::types::user::User;

#[cfg_attr(test, mockall::automock)]
#[trait_variant::make(UserRepository: Send)]
pub trait LocalUserRepository {
    async fn save(&self, user: &User) -> Result<User, RepositoryError>;
    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<User>, RepositoryError>;
    async fn find_by_handle(&self, handle: &str) -> Result<Option<User>, RepositoryError>;
    // ... other methods
}
```

**Critical note on attribute ordering:** `#[cfg_attr(test, mockall::automock)]` MUST appear before `#[trait_variant::make]`. Mockall mocks the `LocalUserRepository` trait (not the Send variant). Use `#[automock(target = UserRepository)]` if you need the mock to target the Send variant instead.

### Pattern 3: Manual TestContext for Unit Tests
**What:** Since mockall cannot mock the `UserPorts` trait (it uses RPITIT), build a concrete `TestContext` struct in test modules that holds mock objects and implements `UserPorts` manually.
**When to use:** Unit testing usecase functions that take `&(impl UserPorts + ?Sized)`.
**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ports::user_repository::MockLocalUserRepository;

    struct TestContext {
        user_repo: MockLocalUserRepository,
    }

    impl UserPorts for TestContext {
        fn user_repo(&self) -> &impl UserRepository {
            &self.user_repo
        }
    }

    #[tokio::test]
    async fn should_reject_self_role_change() {
        let mut ctx = TestContext {
            user_repo: MockLocalUserRepository::new(),
        };
        ctx.user_repo.expect_find_by_id()
            .returning(|_| Ok(Some(/* test user */)));

        let result = change_role(&ctx, /* payload with caller_id == target_id */).await;
        assert!(matches!(result, Err(UserError::SelfModification)));
    }
}
```

### Pattern 4: PostgreSQL Enum with DeriveActiveEnum
**What:** The `role` column uses a PostgreSQL native enum type, mapped via sea-orm's `DeriveActiveEnum`.
**When to use:** Any enum column stored in PostgreSQL.
**Example:**
```rust
// This enum lives in domain/types/role.rs for the domain representation
// AND in schema/users.rs for the sea-orm mapping
// Use a shared definition or From conversion

// schema-level (sea-orm entity):
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "user_role")]
pub enum UserRole {
    #[sea_orm(string_value = "owner")]
    Owner,
    #[sea_orm(string_value = "admin")]
    Admin,
    #[sea_orm(string_value = "user")]
    User,
}
```

Migration must create the enum type before the table:
```rust
// In migration up():
manager.create_type(
    extension::postgres::Type::create()
        .as_enum(UserRoleEnum)
        .values([UserRoleEnum::Owner, UserRoleEnum::Admin, UserRoleEnum::User])
        .to_owned(),
).await?;

manager.create_table(
    Table::create()
        .table(Users::Table)
        .if_not_exists()
        .col(pk_uuid(Users::Id))
        .col(string(Users::Handle))
        .col(string(Users::Name))
        .col(ColumnDef::new(Users::Role).custom(UserRoleEnum::Table).not_null())
        .col(boolean(Users::IsActive))
        .col(timestamp_with_time_zone(Users::CreatedAt))
        .col(timestamp_with_time_zone(Users::UpdatedAt))
        .to_owned(),
).await?;
```

### Pattern 5: Cursor-Based Pagination for ListUsers
**What:** ListUsers uses cursor-based pagination with the cursor being a base64-encoded `created_at` timestamp (or `id` for tie-breaking).
**When to use:** All list endpoints per PROJECT.md conventions.
**Example:**
```rust
// Proto message:
// message ListUsersRequest {
//   int32 limit = 1;
//   optional string cursor = 2;
//   bool include_inactive = 3;
// }

// Repository implementation:
pub async fn list_users(
    &self,
    limit: u64,
    cursor: Option<(DateTime<Utc>, Uuid)>,
    include_inactive: bool,
) -> Result<Vec<User>, RepositoryError> {
    let mut query = users::Entity::find()
        .order_by_desc(users::Column::CreatedAt)
        .order_by_desc(users::Column::Id);

    if !include_inactive {
        query = query.filter(users::Column::IsActive.eq(true));
    }

    if let Some((cursor_time, cursor_id)) = cursor {
        query = query.filter(
            Condition::any()
                .add(users::Column::CreatedAt.lt(cursor_time))
                .add(
                    Condition::all()
                        .add(users::Column::CreatedAt.eq(cursor_time))
                        .add(users::Column::Id.lt(cursor_id))
                )
        );
    }

    query.limit(limit + 1) // Fetch one extra to detect "has more"
        .all(&self.db)
        .await
        .map_err(RepositoryError::from)
}
```

### Pattern 6: gRPC Metadata for Caller Context
**What:** Gateway passes `caller_id` and `caller_role` via gRPC metadata headers to User service.
**When to use:** Any RPC that requires authorization context.
**Example:**
```rust
// Gateway side (REST handler):
let mut request = tonic::Request::new(change_role_request);
request.metadata_mut().insert("x-caller-id", caller_id.parse().unwrap());
request.metadata_mut().insert("x-caller-role", caller_role.parse().unwrap());

// User service app/handler side:
fn extract_caller_context(request: &tonic::Request<impl std::any::Any>) -> Result<CallerContext, Status> {
    let caller_id = request.metadata()
        .get("x-caller-id")
        .ok_or_else(|| Status::unauthenticated("missing caller context"))?
        .to_str()
        .map_err(|_| Status::invalid_argument("invalid caller id"))?
        .parse::<Uuid>()
        .map_err(|_| Status::invalid_argument("invalid caller id format"))?;

    let caller_role = request.metadata()
        .get("x-caller-role")
        .ok_or_else(|| Status::unauthenticated("missing caller role"))?
        .to_str()
        .map_err(|_| Status::invalid_argument("invalid caller role"))?
        .parse::<UserRole>()
        .map_err(|_| Status::invalid_argument("invalid caller role value"))?;

    Ok(CallerContext { caller_id, caller_role })
}
```

### Anti-Patterns to Avoid
- **Mocking UserPorts directly with automock:** Mockall cannot handle RPITIT. Use a manual TestContext struct instead.
- **Putting role hierarchy logic in Gateway:** Gateway is a pure translator. Business rules belong in User service usecase layer.
- **Using `String::len()` for name validation:** Must use `chars().count()` for Unicode character counting (D-10).
- **Storing handle with `@` prefix in DB:** Handle is stored without `@` prefix. The `@` is a display convention (D-02).
- **Creating owner via API:** CreateUser RPC must reject `role=owner` (D-60). Owner is DB-seed only.
- **Using `DeriveIden` with `Table` variant for enum type name:** The `Table` variant in DeriveIden generates the identifier matching the enum's name. Use a separate DeriveIden enum for the PostgreSQL enum type to avoid name collision with the table's DeriveIden.
- **Forgetting `auto_increment = false` on UUID PK:** sea-orm's `#[sea_orm(primary_key)]` defaults to auto-increment. UUID PKs must specify `#[sea_orm(primary_key, auto_increment = false)]`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Struct validation | Custom validation functions | `validator` crate with `#[derive(Validate)]` | Declarative, composable, error aggregation built-in |
| PostgreSQL enum mapping | Manual SQL type casting | `DeriveActiveEnum` with `db_type = "Enum"` | Type-safe, handles serialization/deserialization |
| DB migrations | Raw SQL files | `sea-orm-migration` with `MigrationTrait` | Programmatic, type-safe, reversible |
| UUID generation | Manual random bytes | `uuid::Uuid::new_v4()` | Cryptographically secure, standard format |
| Cursor encoding | Custom encoding | Base64 of `created_at,id` tuple | Simple, URL-safe, opaque to client |
| Docker test containers | Manual Docker commands | `testcontainers` + `testcontainers-modules` | Automatic lifecycle, port mapping, cleanup |
| Mock generation | Manual mock structs | `mockall::automock` on port traits | Automatic, type-safe, expectation verification |

**Key insight:** The 4-layer architecture with trait-based ports creates a natural seam for mocking. Mock the individual port traits (`UserRepository`), not the aggregate `UserPorts` trait. The aggregate is too complex for automatic mocking due to RPITIT.

## Common Pitfalls

### Pitfall 1: Mockall + RPITIT Incompatibility
**What goes wrong:** Applying `#[automock]` to a trait with methods returning `&impl Trait` (RPITIT) fails to compile. Mockall internally transforms the return to `Box<dyn Trait>` but cannot handle reference-returning RPITIT.
**Why it happens:** Mockall's code generation cannot handle the opaque return type of RPITIT methods.
**How to avoid:** Mock individual port traits (e.g., `UserRepository`) with `#[automock]`. Build a manual `TestContext` struct that holds mocks and implements `UserPorts` directly. The `UserPorts` trait itself is never mocked.
**Warning signs:** Compile errors mentioning "impl Trait" or "opaque type" in mock-generated code.

### Pitfall 2: PostgreSQL Enum Migration Ordering
**What goes wrong:** Migration creates the users table with an enum column before creating the PostgreSQL enum type, resulting in "type does not exist" error.
**Why it happens:** PostgreSQL requires `CREATE TYPE` before the table references it.
**How to avoid:** In the migration `up()`, always `create_type` first, then `create_table`. In `down()`, drop table first, then drop type.
**Warning signs:** "type user_role does not exist" error during migration.

### Pitfall 3: Case-Insensitive Handle Uniqueness
**What goes wrong:** Two users can register with handles "Alice" and "alice" because the unique index is case-sensitive by default.
**Why it happens:** PostgreSQL default unique indexes are case-sensitive.
**How to avoid:** Create a unique index on `LOWER(handle)` using a functional index in the migration. Alternatively, store handles always lowercased and display original case separately (simpler approach: store as-is, unique index on `LOWER(handle)`).
**Warning signs:** Duplicate handles differing only in case.

### Pitfall 4: sea-orm UUID PK Auto-Increment Default
**What goes wrong:** sea-orm entity with `#[sea_orm(primary_key)]` on a UUID field generates SQL expecting auto-increment, causing insert failures.
**Why it happens:** `DeriveEntityModel` defaults `primary_key` to auto-increment.
**How to avoid:** Always use `#[sea_orm(primary_key, auto_increment = false)]` for UUID primary keys.
**Warning signs:** "null value in column id violates not-null constraint" or unexpected serial column in generated DDL.

### Pitfall 5: trait_variant Attribute Ordering with mockall
**What goes wrong:** `#[automock]` placed after `#[trait_variant::make]` generates mock for the wrong trait or fails to compile.
**Why it happens:** Proc macro execution order is top-to-bottom. Mockall must see the original trait before trait_variant transforms it.
**How to avoid:** Always place `#[cfg_attr(test, mockall::automock)]` before `#[trait_variant::make(UserRepository: Send)]`.
**Warning signs:** Mock struct not found or incorrect method signatures.

### Pitfall 6: Testcontainers OnceLock Container Lifecycle
**What goes wrong:** Container is dropped at end of test, but `OnceLock` keeps reference, preventing cleanup. Or worse: container leaked, port conflicts on next run.
**Why it happens:** `OnceLock<ContainerAsync<Postgres>>` keeps container alive for test suite duration but `tokio::test` uses its own runtime.
**How to avoid:** Use `tokio::sync::OnceCell` (not `std::sync::OnceLock`) for async container initialization. Or use a shared async runtime setup. The container should outlive all tests in the module.
**Warning signs:** "container already stopped" errors, port binding failures, Docker container accumulation.

### Pitfall 7: Timestamp Precision Mismatch
**What goes wrong:** `chrono::DateTime<Utc>` has nanosecond precision but PostgreSQL `TIMESTAMPTZ` has microsecond precision. Round-trip through DB truncates nanoseconds, causing equality assertions to fail.
**Why it happens:** PostgreSQL stores timestamps with microsecond precision.
**How to avoid:** Use `chrono::DateTime<Utc>` throughout but be aware of precision loss. In tests, compare with microsecond tolerance or truncate before insertion.
**Warning signs:** Test assertions failing on timestamp equality after DB round-trip.

## Code Examples

Verified patterns from official sources and project skills:

### Sea-ORM Entity Definition (Users Table)
```rust
// schema/users.rs
// Source: sea-orm 1.1.19 entity-structure skill + DeriveActiveEnum docs
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "user_role")]
pub enum UserRole {
    #[sea_orm(string_value = "owner")]
    Owner,
    #[sea_orm(string_value = "admin")]
    Admin,
    #[sea_orm(string_value = "user")]
    User,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub handle: String,
    pub name: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

### Migration: Create Users Table with Enum
```rust
// migration/m20260322_000001_create_users_table.rs
// Source: sea-orm-migration 1.1.19 migrations skill
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Create enum type first
        manager.create_type(
            extension::postgres::Type::create()
                .as_enum(UserRoleEnum)
                .values([UserRoleEnum::Owner, UserRoleEnum::Admin, UserRoleEnum::User])
                .to_owned(),
        ).await?;

        // 2. Create table referencing enum
        manager.create_table(
            Table::create()
                .table(Users::Table)
                .if_not_exists()
                .col(pk_uuid(Users::Id))
                .col(string_len(Users::Handle, 15))
                .col(string_len(Users::Name, 80))  // 20 chars * ~4 bytes max UTF-8
                .col(ColumnDef::new(Users::Role)
                    .custom(UserRoleEnum::Table)
                    .not_null()
                    .default("user"))
                .col(boolean(Users::IsActive).default(true))
                .col(timestamp_with_time_zone(Users::CreatedAt))
                .col(timestamp_with_time_zone(Users::UpdatedAt))
                .to_owned(),
        ).await?;

        // 3. Case-insensitive unique index on handle
        manager.create_index(
            Index::create()
                .if_not_exists()
                .name("idx-users-handle-lower")
                .table(Users::Table)
                .col(Users::Handle)
                .unique()
                .to_owned(),
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Users::Table).to_owned()).await?;
        manager.drop_type(
            extension::postgres::Type::drop()
                .name(UserRoleEnum::Table)
                .to_owned(),
        ).await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Handle,
    Name,
    Role,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum UserRoleEnum {
    Table,  // Generates "user_role_enum" -- rename to match enum_name
    Owner,
    Admin,
    User,
}
```

**Note on case-insensitive unique index:** The above index is a standard unique index on the `handle` column. For true case-insensitive uniqueness with original-case storage, use a functional index via raw SQL in the migration:
```rust
manager.get_connection().execute_unprepared(
    "CREATE UNIQUE INDEX \"idx-users-handle-lower\" ON \"users\" (LOWER(\"handle\"))"
).await?;
```

### Tonic Handler with Generic Context
```rust
// app/handler/mod.rs
// Source: tonic 0.14 skill + PROJECT.md architecture
use crate::domain::ports::UserPorts;
use madome_proto::user::user_service_server::UserService;

pub struct UserHandler<C: UserPorts> {
    ctx: C,
}

impl<C: UserPorts> UserHandler<C> {
    pub fn new(ctx: C) -> Self {
        Self { ctx }
    }
}

#[tonic::async_trait]
impl<C: UserPorts + Send + Sync + 'static> UserService for UserHandler<C> {
    async fn create_user(
        &self,
        request: tonic::Request<CreateUserRequest>,
    ) -> Result<tonic::Response<UserResponse>, tonic::Status> {
        let req = request.into_inner();
        let payload = CreateUserPayload::try_from(req)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;

        let user = usecase::create_user::create_user(&self.ctx, payload)
            .await
            .map_err(|e| e.into_status())?;

        Ok(tonic::Response::new(UserResponse::from(user)))
    }
    // ... other RPCs
}
```

### Testcontainers Integration Test Setup
```rust
// tests/integration/user_repository.rs
// Source: testcontainers 0.27.1 + testcontainers-modules 0.15.0 docs
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use sea_orm::{Database, DatabaseConnection};
use tokio::sync::OnceCell;

static DB: OnceCell<DatabaseConnection> = OnceCell::const_new();

async fn get_db() -> &'static DatabaseConnection {
    DB.get_or_init(|| async {
        let container = Postgres::default()
            .start()
            .await
            .expect("failed to start postgres");

        let host = container.get_host().await.expect("host");
        let port = container.get_host_port_ipv4(5432).await.expect("port");

        let url = format!(
            "postgres://postgres:postgres@{}:{}/postgres",
            host, port
        );

        let db = Database::connect(&url).await.expect("db connect");

        // Run migrations
        user::migration::Migrator::up(&db, None)
            .await
            .expect("migration");

        db
    }).await
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `async_trait` for async traits | Native async fn in traits (Rust 1.75+) + `trait_variant` | Dec 2023 | Zero-cost async traits, no Box<dyn Future> overhead |
| `testcontainers::clients::Cli` | `testcontainers::runners::AsyncRunner` | testcontainers 0.15+ | Async-native container management |
| `sea_query::Iden` manual impl | `DeriveIden` derive macro | sea-orm-migration 1.0+ | Less boilerplate in migrations |
| Manual enum SQL | `DeriveActiveEnum` + migration `create_type` | sea-orm 0.12+ | Type-safe enum column mapping |

**Deprecated/outdated:**
- `testcontainers::clients::Cli`: Replaced by `runners::AsyncRunner` / `runners::SyncRunner` in 0.15+
- `#[async_trait]` on port traits: Use `trait_variant::make` for Send-bound async traits instead. `async_trait` only as fallback.
- Manual `Iden` implementation for migration identifiers: Use `#[derive(DeriveIden)]` macro.

## Open Questions

1. **trait_variant + mockall: target attribute behavior**
   - What we know: mockall 0.14 docs show `#[automock(target = Foo)]` with `#[trait_variant::make(Foo: Send)]` pattern. This generates `MockFoo` targeting the Send variant.
   - What's unclear: Whether `#[automock]` (without target) on a trait with `#[trait_variant::make]` correctly generates a mock for the original (non-Send) trait, and whether that mock satisfies the Send-variant trait bound too.
   - Recommendation: Start with `#[cfg_attr(test, mockall::automock)]` without `target` on port traits. If it fails, switch to `target = UserRepository`. If both fail, fall back to `async_trait` for port traits. Validate in Wave 0 task.

2. **Case-insensitive unique index implementation**
   - What we know: PostgreSQL supports functional indexes (`CREATE UNIQUE INDEX ... ON users (LOWER(handle))`). sea-orm-migration's `Index::create()` builder does not directly support expression-based indexes.
   - What's unclear: Whether sea-orm-migration has added expression index support in 1.1.x.
   - Recommendation: Use raw SQL via `manager.get_connection().execute_unprepared()` for the functional index. This is a common pattern for PostgreSQL-specific features.

3. **Testcontainers container lifetime with OnceLock vs OnceCell**
   - What we know: `std::sync::OnceLock` works for sync contexts. `tokio::sync::OnceCell` is async-compatible. testcontainers 0.27 `ContainerAsync` requires async start.
   - What's unclear: Whether the container stays alive for the full test suite when held in `OnceCell`. The GitHub issue #528 mentions cleanup issues.
   - Recommendation: Use `tokio::sync::OnceCell` for the `DatabaseConnection` (not the container). Store both container and connection in a struct held by `OnceCell`. The container's `Drop` handles cleanup when the process exits.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (built-in) + mockall 0.14 + testcontainers 0.27 |
| Config file | None -- Rust uses `Cargo.toml` `[dev-dependencies]` and `#[cfg(test)]` |
| Quick run command | `cargo test -p user` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| USER-PROFILE-01 | Create user via gRPC | unit | `cargo test -p user -- create_user -x` | Wave 0 |
| USER-PROFILE-01 | Get user by ID | unit | `cargo test -p user -- get_user -x` | Wave 0 |
| USER-PROFILE-01 | Get user by handle | unit | `cargo test -p user -- get_user_by_handle -x` | Wave 0 |
| USER-PROFILE-01 | List users with pagination | unit | `cargo test -p user -- list_users -x` | Wave 0 |
| USER-PROFILE-01 | Update user name/handle | unit | `cargo test -p user -- update_user -x` | Wave 0 |
| USER-PROFILE-01 | Handle validation (length, chars, reserved) | unit | `cargo test -p user -- handle_validation -x` | Wave 0 |
| USER-PROFILE-01 | Name validation (length, Unicode) | unit | `cargo test -p user -- name_validation -x` | Wave 0 |
| USER-PROFILE-01 | Repository CRUD operations | integration | `cargo test -p user --test user_repository -x` | Wave 0 |
| USER-PROFILE-02 | Deactivate user with role check | unit | `cargo test -p user -- deactivate_user -x` | Wave 0 |
| USER-PROFILE-02 | Activate user | unit | `cargo test -p user -- activate_user -x` | Wave 0 |
| USER-PROFILE-02 | Change role with hierarchy enforcement | unit | `cargo test -p user -- change_role -x` | Wave 0 |
| USER-PROFILE-02 | Self-modification blocked | unit | `cargo test -p user -- self_modification -x` | Wave 0 |
| USER-PROFILE-02 | Owner role rejection in CreateUser | unit | `cargo test -p user -- reject_owner -x` | Wave 0 |
| USER-PROFILE-02 | Deactivated user visibility rules | unit | `cargo test -p user -- deactivated_visibility -x` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p user`
- **Per wave merge:** `cargo test --workspace && cargo clippy --workspace`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `services/user/tests/` directory -- integration test infrastructure
- [ ] `services/user/src/domain/` -- domain types and port traits needed before any test
- [ ] Workspace `[dev-dependencies]` -- mockall, testcontainers, testcontainers-modules
- [ ] Workspace `[dependencies]` -- sea-orm, sea-orm-migration, validator, trait-variant, chrono
- [ ] `docker-compose.yml` -- PostgreSQL for dev environment
- [ ] Validate `trait_variant` + `mockall` compatibility in a minimal test

## Sources

### Primary (HIGH confidence)
- [sea-orm 1.1.19 crate skills] - entity structure, CRUD operations, migrations, advanced features (local `.claude/skills/`)
- [tonic 0.14 crate skill] - gRPC server/client patterns (local `.claude/skills/`)
- [crates.io registry] - Version verification for sea-orm 1.1.19, mockall 0.14.0, testcontainers 0.27.1, trait-variant 0.1.2, validator 0.20.0
- [docs.rs/testcontainers/0.27.1] - ContainerAsync API, AsyncRunner trait
- [docs.rs/validator/0.20.0] - Validate derive macro, built-in validators
- [sea-ql.org/SeaORM/docs/generate-entity/enumeration] - DeriveActiveEnum for PostgreSQL enums

### Secondary (MEDIUM confidence)
- [docs.rs/mockall/0.14.0/mockall/attr.automock.html] - automock target attribute for trait_variant integration
- [docs.rs/testcontainers-modules/0.15.0] - PostgreSQL module API
- [sea-ql.org/SeaORM/docs/schema-statement/create-enum] - Migration enum creation

### Tertiary (LOW confidence)
- [trait_variant + mockall RPITIT compatibility] - Based on mockall docs showing `target` attribute; real-world validation needed in Wave 0

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All library versions verified against crates.io, project skills confirm API patterns
- Architecture: HIGH - 4-layer pattern is a locked decision (D-28), patterns derived from PROJECT.md and established in Phase 1
- Pitfalls: HIGH - mockall/RPITIT limitation confirmed via official docs; sea-orm UUID PK and enum migration patterns verified
- Testing: MEDIUM - testcontainers 0.27 async API confirmed; OnceLock/OnceCell container lifecycle needs Wave 0 validation
- trait_variant + mockall: LOW - Pattern documented in mockall docs but not battle-tested in this project; validate early

**Research date:** 2026-03-22
**Valid until:** 2026-04-22 (stable libraries, unlikely to change significantly)
