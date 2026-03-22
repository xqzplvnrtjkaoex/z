# Phase 2: User Profile - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

<domain>
## Phase Boundary

User service manages user records with CRUD operations accessible via gRPC. Includes user creation (called by auth service during registration), profile retrieval, listing, self-service profile updates (name/handle change), deactivation/activation, and role changes. User service enforces role hierarchy business rules in its usecase layer. This is the first service with a real database (PostgreSQL).

**Dependency:** Phase 1 (gateway infrastructure, proto compilation, shared crates).

</domain>

<decisions>
## Implementation Decisions

### User Profile Fields
- **D-01:** Users table schema: `id` (UUIDv4 PK), `handle` (unique, URL-safe identifier), `name` (display name, non-unique), `role` (owner/admin/user enum), `is_active` (boolean), `created_at`, `updated_at`
- **D-02:** Handle: unique identifier for URL paths and user lookup. Format: `@handle` for display, stored without `@` prefix in DB. Designed for future routes like `/v1/users/@handle/collections`
- **D-03:** Name: display name, non-unique, Unicode allowed. For presentation purposes only
- **D-04:** Handle and name are set during registration (Phase 3), both changeable after registration
- **D-05:** Future profile fields (profile picture, bio) deferred to later phases

### Handle Validation
- **D-06:** Handle length: 4~15 characters
- **D-07:** Handle characters: alphanumeric (a-z, 0-9) + underscore. URL-safe. Numbers can start handle
- **D-08:** Handle uniqueness: case-insensitive matching. DB stores original case, uniqueness checked on lowercase
- **D-09:** Reserved handles: `me`, `admin`, `system`, `support`, `help`, `deleted`, `unknown`, `anonymous`, `madome`

### Name Validation
- **D-10:** Name length: 1~20 characters (measured by `chars().count()`, not byte length)
- **D-11:** Name characters: Unicode allowed (Korean, Japanese, etc.)
- **D-12:** Validation: `validator` crate for declarative struct-level validation

### Self-Service Mutations
- **D-13:** Users can change their own name and handle via `PATCH /v1/users/@me` (Gateway route -> User service UpdateUser RPC)
- **D-14:** Self-service mutations require protected tier (standard JWT). Step-up auth not required
- **D-15:** Admin cannot change other users' names or handles

### Role Hierarchy Enforcement
- **D-16:** Role hierarchy (owner > admin > user) enforced in User service usecase layer, not Gateway
- **D-17:** Gateway responsibility: route-level tier only (e.g., admin+ for `/v1/users/:id` routes, authenticated for `/v1/users/@me` routes). Business rule enforcement is User service domain
- **D-18:** Role change validation: usecase checks `caller_role > target_current_role AND caller_role > new_role`
- **D-19:** Deactivation validation: usecase checks `caller_role > target_role`
- **D-20:** Self-modification blocked: usecase rejects when `caller_id == target_id` (no self role change, no self deactivation)
- **D-21:** Owner protection: owner role is not available as a target in role change API. Owner cannot be deactivated (no API path exists). Owner manipulation is DB-only (seed migration)
- **D-22:** Minimum owner guarantee: no application-level check needed. API has no path to reduce owner count
- **D-23:** Caller context: Gateway passes `caller_id` + `caller_role` via gRPC metadata to User service

### gRPC RPCs (user.proto)
- **D-24:** User service RPCs: CreateUser, GetUser, GetUserByHandle, ListUsers, UpdateUser, DeactivateUser, ActivateUser, ChangeRole
- **D-25:** Auth service calls: CreateUser (registration, passes name + handle + role), GetUser (by ID, for JWT claims after login)
- **D-26:** Gateway calls: GetUser (detail), GetUserByHandle (future), ListUsers (paginated), UpdateUser (self-service), DeactivateUser, ActivateUser, ChangeRole
- **D-27:** Self-service call: Gateway routes `PATCH /v1/users/@me` -> UpdateUser RPC with caller's own user_id

### Internal Architecture
- **D-28:** User service follows 4-layer architecture (domain/usecase/app/adapter) per PROJECT.md
- **D-29:** `UserPorts` and `UserConfig` separate traits. `UserContext` struct implements both
- **D-30:** Port: `UserRepository` trait for all user data access (single repository, single table)
- **D-31:** Usecase functions: create_user, get_user, get_user_by_handle, list_users, update_user, deactivate_user, activate_user, change_role
- **D-32:** `UserError` enum for business errors: HandleTaken, HandleReserved, UserNotFound, InsufficientRole, SelfModification, InvalidHandle, InvalidName, UserInactive, etc.
- **D-33:** `RepositoryError` for data-access facts (NotFound, UniqueViolation, Database)

### Database Infrastructure
- **D-34:** First service with PostgreSQL. docker-compose introduced in this phase for dev environment
- **D-35:** sea-orm for ORM, auto-migration on service startup
- **D-36:** Integration tests use testcontainers (PostgreSQL). Single container per test suite via `OnceLock`
- **D-37:** Per-service database: `user` database in PostgreSQL

### Testing
- **D-38:** Unit tests: mockall for UserRepository port. Test usecase logic (role hierarchy, self-modification, validation)
- **D-39:** Integration tests: testcontainers with real PostgreSQL. Test adapter (repository) correctness
- **D-40:** Service tests: tonic in-process channel with real adapters + testcontainers
- **D-41:** Test naming: `should_*` BDD style (per project convention)

### Gateway REST Endpoints
- **D-42:** All user endpoints under `/v1/users` (plural only, no singular `/v1/user`)
- **D-43:** Self-service routes: `GET /v1/users/@me` (get own profile), `PATCH /v1/users/@me` (update own name/handle). Requires authenticated tier
- **D-44:** Admin routes: `GET /v1/users` (list), `GET /v1/users/:id` (detail), `PATCH /v1/users/:id/role` (change role), `POST /v1/users/:id/deactivate`, `POST /v1/users/:id/activate`. Requires admin+ tier
- **D-45:** Deactivate/activate use action path style (`POST .../deactivate`) not field patch, because they carry separate business logic (role hierarchy checks)
- **D-46:** Gateway tier separation via axum `nest`: `@me` routes and `:id` routes as separate router groups with different middleware
- **D-47:** Future escalation: if routing conditions become complex (multi-factor: role + ownership + path), replace nest-based separation with a custom routing handler that inspects `RequestParts` directly

### Handle/Name Change Policy
- **D-48:** Handle change releases old handle immediately -- other users can claim it right away. No hold period, no `handle_history` table
- **D-49:** No cooldown on handle or name changes -- users can change freely. No `handle_changed_at` column needed
- **D-50:** Name change follows same policy as handle: immediate, unlimited, no history tracking

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

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Architecture
- `.planning/PROJECT.md` -- Internal Service Architecture (4-layer pattern, ports/config separation, adapter directory structure, testing infrastructure)
- `.planning/PROJECT.md` -- Service Topology (user service role, communication flow)
- `.planning/PROJECT.md` -- ID Design (UUIDv4 for User)
- `.planning/PROJECT.md` -- Role Hierarchy (owner > admin > user, enforcement rules)
- `.planning/PROJECT.md` -- Pagination (cursor-based, X-Next-Cursor header)
- `.planning/PROJECT.md` -- API Conventions (kebab-case query, snake_case body)

### Prior Phases
- `.planning/phases/01-foundation-and-gateway-infrastructure/01-CONTEXT.md` -- REST API format, configuration pattern (env vars), observability (tracing + UUIDv7 request_id), workspace structure

### Downstream Consumers
- `.planning/phases/03-authentication/03-CONTEXT.md` -- Auth service depends on User service RPCs (D-118: CreateUser, GetUser). Auth CONTEXT decisions D-05, D-06, D-57, D-112, D-114, D-118 updated to reflect handle introduction

### Research
- `.planning/research/ARCHITECTURE.md` -- System-level patterns
- `.planning/research/ARCHITECTURE-PATTERNS.md` -- Internal service architecture patterns. **CAVEAT:** Written under incorrect "never mock DB" assumption. CONTEXT.md decisions take precedence

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/madome-core/src/error.rs`: AppError with all HTTP status variants + gRPC Status mapping
- `crates/madome-common/src/env.rs`: Environment variable parsing utilities
- `crates/madome-common/src/tracing.rs`: Tracing initialization
- `crates/madome-proto/build.rs`: Proto compilation pipeline (tonic + prost)
- `services/gateway/src/state.rs`: AppState already has `UserServiceClient<Channel>`
- `services/gateway/src/routes/mod.rs`: Route registration pattern (nest + with_state)

### Established Patterns
- REST-to-gRPC routing in Gateway (Phase 1)
- gRPC Status to HTTP status mapping in AppError
- Environment variable configuration (no config files)
- UUIDv7 request_id via gRPC metadata propagation
- Service binary: main.rs + service.rs (lib pattern for testability)
- Health RPC pattern (already in user service stub)

### Integration Points
- `proto/user.proto`: Currently Health RPC only -- needs full CRUD RPCs
- `services/user/src/service.rs`: Health RPC stub -- replaced by 4-layer architecture
- `services/user/src/main.rs`: Needs PostgreSQL connection, migration, context assembly
- `services/user/schema/`: Empty (.gitkeep) -- needs sea-orm entity definitions
- `services/user/migration/`: Empty (.gitkeep) -- needs migration files
- `services/gateway/src/routes/mod.rs`: Needs user route registration (admin + self-service)
- Root `Cargo.toml`: Needs new workspace.dependencies (sea-orm, sea-orm-migration, validator, mockall, testcontainers, trait-variant)
- New: `crates/madome-test-utils/` -- shared test utilities crate
- New: `docker-compose.yml` -- PostgreSQL for dev environment
- `justfile`: Needs docker-compose and service recipes

</code_context>

<specifics>
## Specific Ideas

- Handle follows GitHub username model -- unique, URL-safe, used in paths. Name follows Discord display name model -- non-unique, for presentation
- Handle is designed for future collection sharing feature: `/v1/users/@handle/collections` (not in scope for this phase)
- Role hierarchy enforcement in User service domain ensures business rules are co-located with user data, not scattered across Gateway
- Owner is effectively immutable at API level -- all owner manipulation is DB-seed only
- String length validation uses `chars().count()` (Unicode scalar values), not `String::len()` (byte count)
- `/v1/users` plural-only convention applies to all future resource endpoints (`/v1/books`, `/v1/auth`, etc.)
- Handle/name change simplicity (no cooldown, immediate release) follows YAGNI for small community -- `handle_history` table can be added later if needed

</specifics>

<deferred>
## Deferred Ideas

- Profile picture upload -- future phase (requires file service)
- User bio / about me -- future phase
- Collection sharing via handle URL -- future phase
- User deletion (permanent) -- v2
- User search/discovery -- future phase
- Activity timestamps (last_login, last_active) -- future phase

</deferred>

---

*Phase: 02-user-profile*
*Context gathered: 2026-03-22 (updated: 2026-03-22)*
