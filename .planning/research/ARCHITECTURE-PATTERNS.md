# Internal Service Architecture Patterns - Research

**Researched:** 2026-03-22
**Domain:** Internal layering patterns for Rust microservices (Clean/Hexagonal/Onion/Pragmatic)
**Confidence:** HIGH (well-established patterns with strong Rust community consensus)

## Summary

This research investigates how to structure the **internal layers** of each Rust gRPC/REST microservice in the madome project. Three classical patterns were evaluated -- Clean Architecture, Hexagonal Architecture (Ports & Adapters), and Onion Architecture -- alongside what the Rust community actually does in practice.

The key finding is that the Rust community has a strong and consistent opinion: **strict Clean/Hexagonal/Onion architecture is over-engineering for most Rust projects, especially small-team microservices**. The community overwhelmingly favors a pragmatic approach that borrows the *principles* (dependency direction, separation of concerns) without the full ceremony (multiple trait layers, separate crates per layer, DTOs between layers). Experienced Rust developers on both Hacker News and the Rust Users Forum explicitly warn against "over-abstracted, verbose, unmaintainable C++/Java written as Rust."

The project's specific constraint -- **testcontainers with real databases, never mock the DB** -- further reduces the motivation for trait-based repository abstractions, since the primary driver for repository traits in Clean Architecture is mock-based unit testing.

**Primary recommendation:** Adopt a **Pragmatic Layered Architecture** -- a simplified 3-layer pattern (handler/service/repository) using concrete types for database access, trait boundaries only where genuinely useful (e.g., the tonic-generated service trait is already a natural port), and module-level visibility to enforce dependency direction. This aligns with what successful Rust gRPC projects do in practice, matches the project's existing code patterns, and avoids premature abstraction for a 1-2 developer team.

## Architecture Patterns Investigated

### Pattern 1: Clean Architecture (Uncle Bob)

**What:** 4 concentric layers -- Entities (innermost), Use Cases, Interface Adapters, Frameworks & Drivers (outermost). The Dependency Rule requires source code dependencies point inward only. Inner layers define trait boundaries; outer layers implement them.

**Rust mapping:**
- **Entities layer** -> Domain structs (pure Rust types, no framework dependencies)
- **Use Cases layer** -> Service structs with business logic, depend on repository traits
- **Interface Adapters layer** -> gRPC handler (tonic trait impl), REST handler (axum), repository implementations
- **Frameworks layer** -> tonic/axum/sea-orm themselves

**Strengths:**
- Clear dependency direction enforced by traits
- Business logic testable in isolation (mock repositories)
- Framework-agnostic core

**Weaknesses for this project:**
- **Mocking is explicitly banned** -- the primary benefit (mock DB for unit tests) contradicts CLAUDE.md's "never mock the database" rule
- **4 layers is excessive** for services with 5-10 RPCs each
- **Trait proliferation** -- every repository operation needs a trait method, creating 2x code for a single implementation
- **Cognitive overhead** for 1-2 developers maintaining 4 services
- **False future-proofing** -- switching from sea-orm to diesel or from PostgreSQL to MySQL is extremely unlikely, and the semantic differences would dwarf any abstraction benefit

**Confidence:** HIGH -- well-documented pattern, clearly understood tradeoffs

### Pattern 2: Hexagonal Architecture (Ports & Adapters)

**What:** Application core (domain + use cases) surrounded by ports (trait interfaces) and adapters (implementations). Ports are divided into "driving" (inbound -- things that call the app) and "driven" (outbound -- things the app calls). Adapters implement ports.

**Rust mapping:**
- **Domain** -> Pure domain types and business logic functions
- **Driving ports** -> The tonic-generated `AuthService` trait IS already a driving port
- **Driven ports** -> `trait UserRepository`, `trait SessionStore`, `trait JwtIssuer`
- **Driving adapters** -> The gRPC service impl struct (implements the tonic trait)
- **Driven adapters** -> `PostgresUserRepository`, `RedisSessionStore`, `Es256JwtIssuer`

**Key insight:** tonic's generated service trait is a natural "driving port." The gRPC service impl struct is already an adapter. Half the pattern is baked into tonic's code generation.

**Strengths:**
- Maps naturally to tonic's trait-based service pattern
- Clean separation of domain from infrastructure
- Well-suited for services with multiple infrastructure dependencies (auth has DB + Redis + WebAuthn + JWT)

**Weaknesses for this project:**
- **Driven port traits for DB** add abstraction without benefit when not mocking
- **Module structure overhead** -- inbound/outbound/domain directories add navigation friction for small services
- **Rust community skepticism** -- HN thread on "Master Hexagonal Architecture in Rust" received significant pushback: "Attempting these design patterns is a common part of getting over OOP when new to Rust"

**Confidence:** HIGH -- well-understood pattern, clear community feedback

### Pattern 3: Onion Architecture

**What:** Similar to Clean Architecture but explicitly organized as concentric rings: Domain Model (center), Domain Services, Application Services, Infrastructure (outermost). The key distinction from Clean Architecture is that domain services (business rules operating on domain models) are a separate ring from application services (orchestration, transactions, use case coordination).

**Rust mapping:** Nearly identical to Clean Architecture in practice. The distinction between "domain service" (e.g., password hashing rules) and "application service" (e.g., registration ceremony orchestration) maps to separate modules within the service, but rarely justifies separate crates or trait layers.

**Assessment:** For this project's scale, Onion Architecture adds no meaningful benefit over Clean Architecture. The domain service vs application service distinction is useful as a mental model but doesn't need architectural enforcement. **Not recommended as a distinct choice.**

**Confidence:** HIGH -- trivially differentiated from Clean Architecture for this scale

### Pattern 4: Pragmatic Layered Architecture (Recommended)

**What:** A simplified 3-layer pattern common in production Rust services. Borrows the *dependency direction principle* from Clean/Hexagonal without the trait ceremony. Uses Rust's module system for encapsulation instead of trait-based boundaries.

**Layers:**
1. **Handler layer** (tonic service trait impl / axum route handlers) -- thin translation layer
2. **Service layer** (business logic, orchestration) -- the core
3. **Repository layer** (sea-orm queries, Redis operations) -- data access

**Key principles:**
- Handler -> Service -> Repository (one-way dependency)
- Repository uses **concrete types** (no trait abstraction), because:
  - DB is never mocked (testcontainers policy)
  - Single implementation per repository
  - sea-orm's `DatabaseConnection` is already an abstraction over different backends
- Service layer contains all business logic -- handler is a thin adapter
- Module visibility (`pub(crate)`, private modules) prevents backward dependencies
- Trait boundaries only where there are genuinely multiple implementations or genuinely separable concerns

**Where traits DO make sense:**
- The tonic-generated service trait (already exists, no choice)
- JWT operations (where the auth service issues and the gateway verifies -- different code paths, same interface)
- Session storage abstraction IF Redis could plausibly be replaced (debatable -- probably not worth it)

**Strengths:**
- Minimal code overhead, maximum clarity
- Matches existing project patterns (service.rs already exists)
- Natural for Rust module system
- Easy to test with testcontainers
- Easy to understand for 1-2 developers
- Can evolve toward hexagonal IF needed (adding traits later is cheap)

**Weaknesses:**
- Less explicit architectural boundaries than trait-based patterns
- Service layer could grow fat without discipline
- No compile-time enforcement that repository doesn't call handler

**Confidence:** HIGH -- matches community consensus, real project patterns, and project constraints

## Recommendation: Pragmatic Layered Architecture

### Why This Pattern

| Factor | Clean/Hexagonal | Pragmatic Layered | Winner |
|--------|----------------|-------------------|--------|
| Team size (1-2 devs) | High cognitive load | Low cognitive load | Pragmatic |
| No DB mocking | Trait overhead without benefit | Direct concrete types | Pragmatic |
| 4 small services | 4x trait proliferation | Simple modules | Pragmatic |
| testcontainers testing | Tests ignore traits anyway | Direct DB testing | Pragmatic |
| Future extensibility | Pre-built for change | Evolve as needed (YAGNI) | Pragmatic |
| tonic integration | Fits well (generated traits) | Uses the same generated traits | Tie |
| Auth service complexity | Good for multi-infra | Good enough with modules | Close, slight Hexagonal edge |

### Per-Service Assessment

**Gateway (axum REST -> gRPC):**
- Simplest structure. Handler layer only (axum routes) + middleware. No service/repository layers needed.
- It is a pure translator. Business logic belongs in backend services.
- Structure: `routes/`, `middleware/`, `state.rs`

**Auth service (most complex):**
- Has multiple infrastructure dependencies: PostgreSQL, Redis, WebAuthn, JWT
- Benefits from separating concerns into modules: `passkey.rs`, `session.rs`, `jwt.rs`, `repository.rs`
- Does NOT need trait-based abstraction -- module boundaries provide sufficient separation
- The tonic service trait impl delegates to specialized modules

**Catalog service:**
- Straightforward CRUD + tag queries + publish workflow
- Service layer handles publish validation logic
- Repository layer handles sea-orm queries
- Simple 3-layer pattern works perfectly

**User service:**
- Simplest backend service. Tastes + reading history.
- May not even need a separate service layer initially -- handler can call repository directly
- Can add service layer when business rules emerge

### Recommended Module Structure

**Auth service (most complex example):**
```
services/auth/
  schema/                    # sea-orm entities (auth schema)
    mod.rs
    users.rs
    credentials.rs
    sessions.rs
    invitations.rs
    api_keys.rs
    recovery_codes.rs
  migration/                 # sea-orm migrations
    mod.rs
    m20260101_000001_create_auth_tables.rs
  src/
    main.rs                  # Bootstrap: DB connection, Redis, build tonic server
    service.rs               # tonic AuthService trait impl (thin handler layer)
    domain.rs                # Domain types (UserRole enum, session TTL constants, etc.)
    passkey.rs               # WebAuthn ceremony logic (register/authenticate)
    session.rs               # Session create/validate/invalidate + Redis operations
    jwt.rs                   # JWT issuance, validation, in-memory cache
    repository.rs            # PostgreSQL queries via sea-orm (users, credentials, invitations, api_keys, recovery_codes)
    recovery.rs              # Recovery code generation + verification
    api_key.rs               # API key generation, hashing, verification
```

**Catalog service:**
```
services/catalog/
  schema/
    mod.rs
    books.rs
    tags.rs
    book_tags.rs
    book_relations.rs
  migration/
    mod.rs
    ...
  src/
    main.rs
    service.rs               # tonic CatalogService trait impl
    repository.rs             # sea-orm queries
    publish.rs                # Publish workflow validation
```

**User service:**
```
services/user/
  schema/
    mod.rs
    tastes.rs
    histories.rs
  migration/
    mod.rs
    ...
  src/
    main.rs
    service.rs                # tonic UserService trait impl
    repository.rs             # sea-orm queries
```

**Gateway:**
```
services/gateway/
  src/
    main.rs
    lib.rs
    state.rs                  # AppState (gRPC clients, JWT public key)
    routes/
      mod.rs
      health.rs
      auth.rs                 # REST endpoints for auth operations
      catalog.rs              # REST endpoints for catalog operations
      user.rs                 # REST endpoints for user operations
    middleware/
      mod.rs
      auth.rs                 # JWT verification/refresh extractor
```

### Dependency Flow Enforcement

Rust's module system naturally enforces dependency direction without traits:

```
service.rs (handler)
    |
    v  (calls)
passkey.rs, session.rs, jwt.rs, repository.rs (service/infra modules)
    |
    v  (uses)
domain.rs (pure types, no dependencies)
```

**Enforcement mechanisms:**
1. **Module visibility:** `repository.rs` only exposes `pub(crate)` functions. It cannot import from `service.rs` because `service.rs` calls it, not the other way around.
2. **Compilation dependency:** `domain.rs` has no imports from other internal modules. If someone adds one, code review catches it.
3. **Separate crate (nuclear option):** If a module grows beyond manageable size, extract to a crate. This gives compile-time enforcement. Not needed initially.

### The tonic Service Trait as Natural Handler

The tonic-generated service trait already acts as the "port" in hexagonal terminology:

```rust
// Generated by tonic -- this IS our handler interface
#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
    async fn register_begin(
        &self,
        request: Request<RegisterBeginRequest>,
    ) -> Result<Response<RegisterBeginResponse>, Status> {
        // Thin handler: extract, delegate to service/domain modules, map errors
        let req = request.into_inner();
        let invite = self.repository.verify_invite(&req.token).await
            .map_err(|e| Status::from(e))?;
        let (challenge, reg_state) = self.passkey.begin_registration(&req.name).await
            .map_err(|e| Status::from(e))?;
        // Store reg_state in short-lived cache...
        Ok(Response::new(RegisterBeginResponse { /* ... */ }))
    }
}
```

The `AuthServiceImpl` struct holds its dependencies as fields:

```rust
pub struct AuthServiceImpl {
    pub(crate) db: DatabaseConnection,           // sea-orm
    pub(crate) redis: redis::Client,             // session storage
    pub(crate) webauthn: Webauthn,               // webauthn-rs instance
    pub(crate) jwt_config: JwtConfig,            // key pair + settings
}
```

No trait abstraction needed for these fields -- they are concrete types injected through the constructor in `main.rs`.

### When to Add Trait Boundaries Later

The pragmatic approach does NOT mean "never use traits." It means "add traits when evidence demands it." Triggers:

| Trigger | Action |
|---------|--------|
| Need to test business logic without DB (rare given testcontainers) | Extract trait for the specific repository method |
| Genuinely multiple implementations (e.g., different session backends) | Add trait for the differing component |
| Module grows beyond ~500 lines | Split into sub-modules, consider trait boundaries |
| New team member confused by dependency direction | Add trait to make it explicit |
| Extracting shared logic to a crate | Crate boundary forces trait definitions |

### Testing Strategy Without Mocks

The project mandates testcontainers with real PostgreSQL and Redis. This changes the testing architecture significantly:

**Unit tests** (`#[cfg(test)]` in module files):
- Test pure domain logic (validation, computation, type conversions)
- No DB needed. These are functions in `domain.rs`, `passkey.rs` (ceremony logic), `jwt.rs` (claim construction)
- Example: "JWT with expired timestamp should fail validation"

**Integration tests** (testcontainers):
- Test repository functions against real PostgreSQL
- Test session operations against real Redis
- Each test gets a clean database (migration runs in setup)
- Example: "create_user then find_by_name returns same user"

**Service tests** (tonic in-process channel + testcontainers):
- Construct `AuthServiceImpl` with real DB + Redis
- Call gRPC methods through tonic's in-process channel
- Example: "register_begin with valid invite returns challenge"

**E2E contract tests** (through Gateway REST API):
- Full flow: register -> login -> access protected endpoint
- All services running, real DBs
- Example: "complete auth flow returns JWT cookie"

```rust
// Integration test example -- no mocks
#[tokio::test]
async fn test_create_and_find_user() {
    let db = setup_test_db().await; // testcontainers + migration
    let repo = UserRepository::new(db.clone());

    let user = repo.create_user("alice", UserRole::User).await.unwrap();
    let found = repo.find_by_name("alice").await.unwrap();

    assert_eq!(found.unwrap().id, user.id);
}
```

## Comparing Approaches: Concrete Example

Consider implementing "register a new passkey" (AUTH-01). Here is how the code differs:

### Full Hexagonal Approach (NOT recommended)

```rust
// domain/ports.rs - Trait for every dependency
trait InviteRepository: Send + Sync {
    async fn verify_and_consume(&self, token_hash: &[u8]) -> Result<Invite, DomainError>;
}
trait UserRepository: Send + Sync {
    async fn create(&self, name: &str, role: UserRole) -> Result<User, DomainError>;
}
trait CredentialRepository: Send + Sync {
    async fn store(&self, user_id: Uuid, cred: StoredCredential) -> Result<(), DomainError>;
}
trait RecoveryCodeRepository: Send + Sync {
    async fn store_batch(&self, user_id: Uuid, codes: Vec<HashedCode>) -> Result<(), DomainError>;
}
trait PasskeyProvider: Send + Sync {
    async fn begin_registration(&self, name: &str) -> Result<(Challenge, RegState), DomainError>;
    async fn finish_registration(&self, state: RegState, response: &[u8]) -> Result<StoredCredential, DomainError>;
}

// domain/use_cases/register.rs - Use case depends on traits
struct RegisterUseCase<IR, UR, CR, RR, PP> {
    invite_repo: IR,
    user_repo: UR,
    cred_repo: CR,
    recovery_repo: RR,
    passkey: PP,
}

// adapters/postgres_invite_repo.rs - Adapter implements trait
struct PostgresInviteRepository { db: DatabaseConnection }
impl InviteRepository for PostgresInviteRepository { /* ... */ }

// 5 traits, 5 adapter files, 1 use case struct with 5 generic parameters
// Total: ~15 files for one feature
```

### Pragmatic Approach (Recommended)

```rust
// repository.rs - Concrete type with all DB operations
pub(crate) struct Repository {
    db: DatabaseConnection,
}

impl Repository {
    pub(crate) async fn verify_and_consume_invite(&self, token_hash: &[u8]) -> Result<Invite, AppError> { /* ... */ }
    pub(crate) async fn create_user(&self, name: &str, role: UserRole) -> Result<User, AppError> { /* ... */ }
    pub(crate) async fn store_credential(&self, user_id: Uuid, cred: &StoredCredential) -> Result<(), AppError> { /* ... */ }
    pub(crate) async fn store_recovery_codes(&self, user_id: Uuid, codes: &[HashedCode]) -> Result<(), AppError> { /* ... */ }
}

// service.rs - tonic trait impl delegates to concrete modules
impl AuthService for AuthServiceImpl {
    async fn register_finish(&self, request: Request<RegisterFinishRequest>) -> Result<Response<RegisterFinishResponse>, Status> {
        let req = request.into_inner();
        let cred = self.passkey.finish_registration(/* ... */).await?;
        let user = self.repository.create_user(&req.name, UserRole::User).await?;
        self.repository.store_credential(user.id, &cred).await?;
        let codes = recovery::generate_codes();
        self.repository.store_recovery_codes(user.id, &codes).await?;
        // ...
    }
}

// Total: 3 files (service.rs, repository.rs, recovery.rs) for the same feature
```

The pragmatic approach produces ~5x less boilerplate for equivalent functionality, with no loss of testability (testcontainers tests work identically).

## Common Pitfalls

### Pitfall 1: Trait-Based Repository When Not Mocking
**What goes wrong:** Team creates `trait UserRepository` with 15 methods, implements it once for PostgreSQL, never creates a second implementation. All tests use testcontainers anyway.
**Why it happens:** Habit from Java/C# or theoretical Clean Architecture adherence.
**How to avoid:** Ask "will there ever be a second implementation?" If no, use concrete type. If maybe, add the trait when the second implementation appears.
**Warning signs:** Every repository method has a 1:1 trait method. Zero mock implementations in tests.

### Pitfall 2: Fat Service.rs
**What goes wrong:** All business logic ends up in `service.rs` (the tonic trait impl), making it 1000+ lines.
**Why it happens:** Without explicit layers, the handler absorbs business logic.
**How to avoid:** `service.rs` should be a thin handler -- extract business logic into domain-specific modules (passkey.rs, session.rs, jwt.rs). The tonic trait impl should read like a recipe: validate, delegate, map, respond.
**Warning signs:** `service.rs` growing beyond ~300 lines. Complex match/if logic in handler methods.

### Pitfall 3: Repository Returning sea-orm Models to Handler
**What goes wrong:** `repository.rs` returns `entity::users::Model` directly to `service.rs`, coupling the handler to sea-orm's generated types.
**Why it happens:** Convenience -- sea-orm models are already there.
**How to avoid:** Repository can return sea-orm models to the service layer (this is fine for internal use). The service layer converts to domain types or proto response types before returning to the handler. Alternatively, define lightweight domain structs in `domain.rs` and convert in repository.
**Warning signs:** `service.rs` importing `sea_orm::*`. Proto response construction mixing with DB query results.

### Pitfall 4: Gateway Acquiring Business Logic
**What goes wrong:** JWT validation logic grows complex, session refresh logic is added, error mapping becomes sophisticated -- gateway becomes a mini-service.
**Why it happens:** JWT and session management straddle the gateway/auth boundary.
**How to avoid:** Gateway middleware should be as thin as possible. JWT cryptographic validation is OK in gateway (stateless). Anything requiring DB/Redis lookup goes through Auth service gRPC. This is already correctly designed in the project (D-60: Gateway is pure REST-to-gRPC translator).
**Warning signs:** Gateway importing sea-orm or Redis crates. Gateway having its own database connection.

### Pitfall 5: Over-Modularizing Small Services
**What goes wrong:** User service (tastes + histories) gets split into 8 files with domain/, application/, infrastructure/ directories.
**Why it happens:** Applying the same structure to all services regardless of complexity.
**How to avoid:** Scale structure to service complexity. User service can start with just `service.rs` + `repository.rs`. Auth service needs more modules because it has more concerns. Let complexity drive structure, not templates.
**Warning signs:** Files with < 50 lines of actual logic. Module directories with 1-2 files.

## Code Examples

### Example 1: Auth Service Bootstrap (main.rs)

```rust
// services/auth/src/main.rs
use sea_orm::Database;
use tonic::transport::Server;

mod api_key;
mod domain;
mod jwt;
mod passkey;
mod recovery;
mod repository;
mod service;
mod session;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    madome_common::tracing::init_tracing("auth");

    let addr = madome_common::env::required_env("AUTH_LISTEN_ADDR").parse()?;
    let db_url = madome_common::env::required_env("AUTH_DATABASE_URL");
    let redis_url = madome_common::env::required_env("AUTH_REDIS_URL");

    let db = Database::connect(&db_url).await?;
    let redis = redis::Client::open(redis_url)?;
    let webauthn = passkey::build_webauthn()?; // reads RP_ID, RP_ORIGIN

    let jwt_config = jwt::JwtConfig::from_env()?;

    let auth_service = service::AuthServiceImpl {
        repository: repository::Repository::new(db),
        session: session::SessionManager::new(redis),
        webauthn,
        jwt_config,
    };

    tracing::info!(%addr, "auth service starting");

    Server::builder()
        .add_service(madome_proto::auth::auth_service_server::AuthServiceServer::new(auth_service))
        .serve(addr)
        .await?;

    Ok(())
}
```

### Example 2: Repository with Concrete Types

```rust
// services/auth/src/repository.rs
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set};
use uuid::Uuid;

pub(crate) struct Repository {
    db: DatabaseConnection,
}

impl Repository {
    pub(crate) fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub(crate) async fn find_user_by_id(&self, id: Uuid) -> Result<Option<users::Model>, AppError> {
        users::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))
    }

    pub(crate) async fn create_user(&self, name: &str, role: UserRole) -> Result<users::Model, AppError> {
        let user = users::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(name.to_string()),
            role: Set(role.to_string()),
            is_active: Set(true),
            ..Default::default()
        };
        user.insert(&self.db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))
    }
    // ... more methods
}
```

### Example 3: Service Layer (Thin Handler)

```rust
// services/auth/src/service.rs
use crate::{jwt, passkey, recovery, repository::Repository, session::SessionManager};
use madome_proto::auth::auth_service_server::AuthService;
use tonic::{Request, Response, Status};

pub struct AuthServiceImpl {
    pub(crate) repository: Repository,
    pub(crate) session: SessionManager,
    pub(crate) webauthn: webauthn_rs::prelude::Webauthn,
    pub(crate) jwt_config: jwt::JwtConfig,
}

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
    async fn register_begin(
        &self,
        request: Request<RegisterBeginRequest>,
    ) -> Result<Response<RegisterBeginResponse>, Status> {
        let req = request.into_inner();

        // 1. Verify invite token
        let _invite = self.repository
            .verify_and_consume_invite(&req.token)
            .await
            .map_err(|_| Status::permission_denied("invite_invalid"))?;

        // 2. Begin passkey registration ceremony
        let (challenge, reg_state) = passkey::begin_registration(
            &self.webauthn,
            &req.name,
        ).await.map_err(|e| Status::internal(e.to_string()))?;

        // 3. Store registration state (short-lived)
        // ... cache reg_state keyed by challenge

        Ok(Response::new(RegisterBeginResponse {
            challenge: serde_json::to_string(&challenge)?,
        }))
    }

    async fn health(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }
}
```

### Example 4: Domain Module (Pure Types)

```rust
// services/auth/src/domain.rs
use std::time::Duration;

/// User roles in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRole {
    Admin,
    User,
}

/// Session TTL configuration
pub const SESSION_SLIDING_TTL: Duration = Duration::from_secs(7 * 24 * 3600); // 7 days
pub const SESSION_ABSOLUTE_TTL: Duration = Duration::from_secs(30 * 24 * 3600); // 30 days
pub const JWT_TTL: Duration = Duration::from_secs(15 * 60); // 15 minutes
pub const JWT_GRACE_PERIOD: Duration = Duration::from_secs(60); // 1 minute
pub const JWT_CACHE_TTL: Duration = Duration::from_secs(10); // 10 seconds

/// Recovery code configuration
pub const RECOVERY_CODE_COUNT: usize = 10;
pub const RECOVERY_CODE_LENGTH: usize = 8;

/// API key prefix for identification
pub const API_KEY_PREFIX: &str = "madome_sk_";
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `dyn Trait` for DI | Generics or concrete types | Rust community evolution ~2022+ | Avoids boxing overhead, keeps compile-time guarantees |
| Separate crate per layer | Module-per-concern in single crate | Community practice ~2023+ | Reduces workspace complexity for small services |
| Mock everything for unit tests | testcontainers for integration tests | testcontainers-rs maturity ~2024 | Tests real behavior, not mocked behavior |
| `#[tonic::async_trait]` macro | Native async traits (Rust 1.75+) | Dec 2023 | Eventually removes macro requirement (tonic still uses macro as of 0.13) |
| OOP-style DI frameworks | Constructor injection in `main.rs` | Rust convention | No DI container needed; Rust's type system is sufficient |

## Open Questions

1. **Should `repository.rs` return sea-orm `Model` types or domain types?**
   - What we know: Returning `Model` directly couples service layer to sea-orm. Returning domain types requires mapping code.
   - What's unclear: For a 1-2 dev team, is the mapping overhead worth the decoupling?
   - Recommendation: Start with returning `Model` types. If proto response construction becomes messy, introduce domain types. The mapping can be added incrementally. For Phase 2 specifically, `users::Model` fields are close enough to domain needs.

2. **Should the auth service split `repository.rs` into multiple files?**
   - What we know: Auth has 6 tables (users, credentials, sessions, invitations, api_keys, recovery_codes). A single repository file could grow large.
   - What's unclear: Whether the cognitive benefit of splitting outweighs the navigation overhead.
   - Recommendation: Start with single `repository.rs`. Split when it exceeds ~400 lines (e.g., into `repository/users.rs`, `repository/credentials.rs`). Use a `repository/mod.rs` to re-export a unified `Repository` struct.

3. **Does the gateway need any service-layer logic?**
   - What we know: D-60 says gateway is a pure REST-to-gRPC translator. JWT validation is middleware. Session refresh involves an Auth gRPC call.
   - What's unclear: Is JWT cookie construction + gRPC metadata injection "business logic"?
   - Recommendation: No. JWT cookie parsing and gRPC metadata injection are middleware/translation concerns, not business logic. Gateway stays as routes/ + middleware/ + state.rs.

## Sources

### Primary (HIGH confidence)
- [Master Hexagonal Architecture in Rust](https://www.howtocodeit.com/guides/master-hexagonal-architecture-in-rust) - Comprehensive guide with Rust code examples for ports, adapters, domain types
- [Hacker News discussion on Hexagonal Architecture in Rust](https://news.ycombinator.com/item?id=41518698) - Community pushback from experienced Rust developers
- [Rust Users Forum: Of Architecture, Traits and Unit Testing](https://users.rust-lang.org/t/of-architecture-traits-and-unit-testing/37287) - Generics vs dyn Trait for DI
- [Rust Users Forum: Hexagonal Architecture Code Review](https://users.rust-lang.org/t/rust-hexagonal-architecture/98672) - Practical critique of hexagonal in Rust
- [SeaORM Tonic Example](https://github.com/SeaQL/sea-orm/tree/master/examples/tonic_example) - Official sea-orm + tonic integration pattern
- [Archetype: Rust Service Tonic Workspace](https://github.com/archetect/archetype-rust-service-tonic-workspace) - Three-tier Server/Core/Persistence pattern

### Secondary (MEDIUM confidence)
- [An Opinionated Clean Architecture in Rust](https://dev.to/guuri11/an-opinionated-clean-architecture-in-rust-4jn8) - Three-layer model with Rust crate separation
- [Practical Clean Architecture in Typescript, Rust & Python](https://dev.to/msc29/practical-clean-architecture-in-typescript-rust-python-3a6d) - Cross-language comparison
- [Make Your Rust Code Unit Testable with Dependency Inversion](https://worldwithouteng.com/articles/make-your-rust-code-unit-testable-with-dependency-inversion/) - Trait-based DI patterns
- [When (and When Not) to Use Hexagonal Architecture](https://medium.com/fastned/when-and-when-not-to-use-hexagonal-architecture-c0850d643b3b) - Decision criteria for adoption
- [Hexagonal Architecture Common Pitfalls](https://medium.com/@allousas/hexagonal-architecture-common-pitfalls-f155e12388a3) - What goes wrong in practice

### Tertiary (LOW confidence)
- [Building Microservices with Rust Tonic gRPC: Production Best Practices](https://markaicode.com/building-microservices-with-rust-tonic-grpc-best-practices/) - General best practices, not project-specific

## Metadata

**Confidence breakdown:**
- Architecture pattern comparison: HIGH - well-documented patterns with clear community consensus
- Recommended pattern for this project: HIGH - matches constraints (no mocking, small team, testcontainers)
- Module structure: MEDIUM - specific file layout is opinion-based, depends on implementation details
- Code examples: MEDIUM - illustrative, not yet validated against actual project dependencies
- When-to-evolve triggers: MEDIUM - based on community experience, not project-specific data

**Research date:** 2026-03-22
**Valid until:** 2026-06-22 (patterns are stable; Rust edition/tonic version changes could affect specifics)
