# Phase: Authentication Core - Context

**Gathered:** 2026-03-21
**Updated:** 2026-03-26 (signup/register separation D-146, ceremony cleanup D-147, route table + D-47/D-59/D-134 updated)
**Status:** Ready for planning

> **Phase split:** Original Phase 3 (Authentication, ~126 decisions) split into 3A (Core) and 3B (Operations). 3A establishes the authentication foundation; 3B builds operational features on top.

<domain>
## Phase Boundary

Users can register a passkey via invite token, authenticate via username-less passkey login, and access protected endpoints through JWT-verified gateway. Includes session management with Redis, JWT lifecycle, and owner bootstrap via seed invite. Recovery codes are generated at registration (stored hashed) but the recovery login flow is deferred to Phase 3B.

**Dependency:** Requires Phase 1 (gateway infrastructure) and Phase 2 (user profile -- users table, CRUD RPCs).

**Downstream:** Phase 3B (Authentication Operations) builds on this phase's auth service, gateway middleware, and session infrastructure.

</domain>

<decisions>
## Implementation Decisions

### User Onboarding
- **D-01:** Invite-only registration. Users register a passkey using a valid invite token
- **D-03:** Invite token policy: single-use, 30-minute expiry. Token string returned via API, shared manually
- **D-04:** Registration URL pattern: `/register?token=xxx`
- **D-05:** User sets `name` (display name, non-unique) and `handle` (unique identifier, URL-safe) during passkey registration
- **D-06:** Profile info: name (display name, non-unique) and handle (unique identifier). No email

### Invite Creation (3A Scope)
- **D-02:** *(3A SCOPE)* Invite table has `role` column determining the registered user's role. In 3A, invite role is always `user` (hardcoded) -- owner creates invites via a protected endpoint (handler-level role check), but cannot specify role. Role-selectable invite creation and admin invite management deferred to 3B
- **D-09:** *(3A SCOPE)* Invite history recorded in DB. Admin query/cancel endpoints deferred to 3B

### Passkey Configuration
- **D-10:** Attestation: None (no hardware attestation required)
- **D-11:** Authenticator Attachment: no restriction (platform + roaming both allowed)
- **D-12:** User Verification: Required (always biometric/PIN)
- **D-13:** Discoverable Credential (Resident Key): Required (username-less login)
- **D-14:** Multiple passkeys per user allowed (multi-device support)
- **D-15:** Last passkey cannot be deleted (minimum 1 always)
- **D-67:** AAGUID stored at registration time. Extract from raw attestationObject CBOR (authData[37..53]) using minicbor, store as UUID column in credentials table. webauthn-rs does not expose AAGUID for attestation=none, so direct CBOR parsing required

### Passkey Flows
- **D-16:** *(UPDATED)* Passkey ceremony: 2-step (begin + finish). Pure WebAuthn credential registration/verification -- separated from signup orchestration (see D-146)
- **D-146:** Signup/Register operation separation. **SignupBegin/SignupFinish** (`/v1/auth/signup/*`): full signup flow -- invite validation, User.CreateUser, passkey ceremony, session/JWT/recovery codes. User created in SignupBegin; if SignupFinish never called, orphan user cleanup needed. **RegisterBegin/RegisterFinish** (`/v1/auth/register/*`): pure passkey ceremony -- WebAuthn challenge/credential only, JWT auth required (authenticated users adding passkeys). Internal WebAuthn ceremony logic shared between both flows. Both endpoint pairs implemented in 3A
- **D-147:** Ceremony state deletion is best-effort. Failure logged as warning but does not fail the operation. Redis TTL (5min) is the failsafe cleanup. Applies to: SignupFinish, RegisterFinish, LoginFinish
- **D-17:** Authentication: username-less (Discoverable Credential based). No identifier in login/begin request body
- **D-18:** RP configuration: RP_ID, RP_ORIGIN environment variables

### Recovery Code Generation
- **D-21:** Recovery codes issued at registration: 10 codes, 8-character alphanumeric, each single-use
- **D-103:** Recovery codes stored as hashes in DB (SHA-256 or similar). Raw codes shown only once at registration

> Recovery login flow (D-23), recovery JWT behavior (D-104/105), and regeneration (D-22) deferred to Phase 3B.

### Session Lifecycle
- **D-32:** Session storage: Redis (TTL auto-expiry + fast lookup)
- **D-33:** Session TTL: sliding 7 days + absolute 30 days. JWT refresh extends sliding window; session created_at + 30 days is hard limit
- **D-34:** JWT grace period: 1 minute (within 1 min of JWT expiry, Gateway issues new JWT without session verification)
- **D-35:** Concurrent sessions: unlimited per user
- **D-36:** *(3A SCOPE)* Logout: current session only (POST /v1/auth/logout). Logout all sessions deferred to 3B (verified tier)
- **D-37:** JWT caching: in-memory in Auth service with short TTL (~10 sec). Concurrent requests within window receive same JWT

### JWT Configuration
- **D-39:** *(3A SCOPE)* Claims: sub (user_id), role (owner/admin/user), sid (session_id), handle, name, iss ("madome-auth"), aud ("madome-gateway"), iat, exp. Conditional claims (`recovery`, `verified_at`) not issued in 3A -- added in 3B
- **D-40:** Algorithm: ES256 (ECDSA P-256, aws_lc backend)
- **D-41:** Key management: JWT_PRIVATE_KEY, JWT_PUBLIC_KEY environment variables (PEM format)
- **D-42:** Validation: iss + aud claim verification enabled
- **D-43:** TTL: 15 minutes
- **D-44:** Cookie: HttpOnly=true, Secure=true, SameSite=Strict, Path=/. Dev override: COOKIE_SECURE=false env var
- **D-45:** Key rotation: out of scope. Single key pair, no kid claim

### Gateway Middleware
- **D-46:** *(3A SCOPE)* 2-tier route authentication: public (no auth) and protected (JWT required). Additional tiers (verified, admin, scraper, recovery) added in Phase 3B
- **D-47:** *(3A SCOPE, UPDATED)* Public routes: /v1/auth/signup/\*, /v1/auth/login/\*, /health, /v1/health/\*. Protected routes: /v1/auth/register/\* (JWT required, authenticated users adding passkeys)
- **D-48:** JWT signature invalid (forged/corrupted): immediate 401 (no session fallback). Only expired JWT enters grace/session flow
- **D-49:** User context propagation: user_id, role, handle, session_id extracted from JWT, passed as gRPC metadata (same pattern as request_id)
- **D-127:** `verify_jwt` middleware replaces `extract_caller_identity` entirely. JWT is the sole identity source. Uses `route_layer` + `from_fn_with_state` for AppState access (JWT public key). `route_layer` prevents 404-to-401 bleed (middleware only runs on matched routes)
- **D-128:** `CallerIdentity` extended with `handle: String` and `session_id: Uuid`. Single `Extension<CallerIdentity>` covers all caller context. Fields: `caller_id` (Uuid), `caller_role` (CallerRole), `handle` (String), `session_id` (Uuid)
- **D-129:** Public vs protected routes structurally split at Router level. Protected routes get `verify_jwt` via `route_layer`. Grace period handled via response mutation (Set-Cookie header after `next.run()`)

### Auth Error Policy
- **D-50:** Client-facing errors: generic `unauthorized` for all auth failures. No specific failure reason exposed
- **D-51:** JWT errors: unified `unauthorized` (Gateway handles internally)
- **D-52:** Invite token errors: unified `invite_invalid` (token existence not disclosed)
- **D-53:** Deactivated user login: generic `unauthorized` (deactivation not disclosed)
- **D-54:** Server logs: detailed error info with request_id + user_id + specific failure reason
- **D-55:** Rate limiting: deferred (out of scope)

### Auth DB Schema
- **D-56:** *(3A NOTE)* 5 tables in auth DB: credentials, sessions, invitations, api_keys, recovery_codes. All tables created in 3A migrations for schema completeness. api_keys table populated in 3B. Users table is in User service DB. Auth tables reference user_id without FK (cross-service boundary)
- **D-57:** users table (User service DB): id (UUIDv4 PK), handle (unique, case-insensitive), name (display name, non-unique), role (owner/admin/user enum), is_active (boolean), created_at, updated_at. Owner role value included in enum
- **D-58:** Indexes: matching query patterns (credentials.user_id, sessions.user_id, api_keys.key_hash UNIQUE, invitations.token_hash UNIQUE, recovery_codes.user_id)

### Proto Definition
- **D-59:** *(3A SCOPE, UPDATED)* Auth gRPC RPCs: SignupBegin/Finish, RegisterBegin/Finish, LoginBegin/Finish, ValidateSession, RefreshToken, InvalidateSession (logout), CreateInvite. User gRPC: CreateUser, GetUser, DeleteUser (compensation only, D-140). Additional RPCs added in 3B
- **D-60:** Gateway is REST-to-gRPC translator for auth operations

### Infrastructure
- **D-61:** docker-compose for PostgreSQL + Redis (latest stable versions). Services run via cargo run (Phase 1 pattern preserved)
- **D-62:** *(3A NOTE)* DB initialization: auto-migration on service startup (sea-orm). Owner seed: migration creates owner-role invite token. Owner registers through standard flow
- **D-63:** justfile: recipes for `docker-compose up -d`, service startup, and common dev commands. No shell scripts in scripts/
- **D-130:** JWT key provisioning: `just dev-keys` recipe generates ES256 PEM key pair via openssl, writes to `.env` (gitignored). Zero committed secrets
- **D-131:** Docker-compose additions: auth-db (postgres:18-alpine, port 5434) + redis (redis:8-alpine, port 6379). Extends existing user-db pattern from Phase 2
- **D-132:** Seed invite: fixed well-known dev token (SHA-256 hash stored in migration). Deterministic for dev/test automation. **Prod seed MUST use a separate, non-committed mechanism**
- **D-143:** Redis crate: `redis` crate with `ConnectionManager` (auto-reconnect, cheaply cloneable). No connection pool needed for low-concurrency session/ceremony KV
- **D-144:** Auth service startup: fail-fast (consistent with user/catalog services). Docker-compose `depends_on` + `healthcheck` handles service ordering. No application-level retry
- **D-145:** Graceful shutdown: tonic `serve_with_shutdown` + `tokio::signal` (ctrl_c + SIGTERM). No new deps. Upgrade to `CancellationToken` when background tasks are added

### REST API Endpoints

**Auth endpoints (3A scope):**

| Tier | Method | Path | Description |
|------|--------|------|-------------|
| public | POST | /v1/auth/signup/begin | Signup start (invite + user creation + passkey ceremony) |
| public | POST | /v1/auth/signup/finish | Signup complete -> 201, recovery codes in body |
| protected | POST | /v1/auth/register/begin | Add passkey start (pure WebAuthn ceremony, JWT auth) |
| protected | POST | /v1/auth/register/finish | Add passkey complete -> 201, credential saved |
| public | POST | /v1/auth/login/begin | Login start (username-less) |
| public | POST | /v1/auth/login/finish | Login complete -> 204 No Content, JWT in cookie |
| protected | POST | /v1/auth/logout | Logout current session |
| protected | POST | /v1/auth/invites | Create invite (owner-only, handler-level role check) |

**User endpoints (3A scope):**

| Tier | Method | Path | Description |
|------|--------|------|-------------|
| protected | GET | /v1/users/@me | Current user profile (DB query, latest info) |

- **D-111:** Login success response: 204 No Content (empty body). JWT delivered via HttpOnly cookie only
- **D-112:** Registration success response: 201 Created with `{ "user_id", "handle", "name", "role", "recovery_codes" }`. Recovery codes are raw strings shown this once only
- **D-141:** Invite creation request body: empty (no fields). Role is always `user` in 3A (see D-02). Role-selectable requests deferred to 3B
- **D-142:** Invite creation response body: `{ "token": "...", "expires_at": "..." }`. Token is the one-time secret for sharing; expires_at confirms the 30-minute window

### Owner Bootstrapping
- **D-115:** Owner is DB seed only. Exactly 1 owner. Cannot create or change owner role via API
- **D-116:** Owner bootstrap: seed migration creates a single owner-role invite token. Owner uses standard registration flow to set name + handle + passkey. Token hash stored in invitations table like any other invite

### Cross-Service Communication
- **D-117:** Service-to-service direct gRPC calls allowed (project-wide architecture decision)
- **D-118:** Auth -> User service calls: CreateUser (registration finish, passes name + handle + role from invite), GetUser (by ID, login finish, for JWT claims including handle), DeleteUser (registration failure compensation only, D-139). Auth service requires `USER_GRPC_ADDR` environment variable

### Cross-Service Error Handling
- **D-133:** Fail-fast strategy for Auth -> User gRPC calls. User service call executes first; auth DB writes only after User service succeeds. No compensating transactions, no saga
- **D-134:** *(UPDATED)* Signup flow ordering: SignupBegin (invite validate -> User.CreateUser -> ceremony start) -> SignupFinish (credential verify -> credential save -> recovery codes -> session create -> JWT issue). Login flow: User.GetUser -> session create -> JWT issue. User service failure at any point = entire auth operation fails cleanly
- **D-135:** Per-endpoint gRPC timeout: 5 seconds via `tonic::Request::set_timeout`. Applies to all Auth -> User calls
- **D-136:** User service unavailable = auth operation failure. Acceptable for this service scale. No partial recovery or retry logic
- **D-139:** Registration failure recovery: if auth DB/Redis fails after User.CreateUser succeeds, attempt compensating `User.DeleteUser` call. If compensating delete also fails (double failure), log structured orphan event (`user_id + handle + request_id`) for manual cleanup
- **D-140:** Internal `DeleteUser` RPC added to user.proto in 3A. Auth service compensation only -- not exposed via gateway REST endpoint. Admin user deletion deferred to 3B or later

### Pagination (Project-Wide)
- **D-121:** All list endpoints use cursor-based pagination. Opaque URL-safe cursor value
- **D-122:** Cursor transport: server -> client via `X-Next-Cursor` response header (omitted when no more pages). Client -> server via `?cursor=` query parameter. Response body stays flat array
- **D-123:** Page size: `?limit=N` query parameter. Default 25, max 100
- **D-124:** Sort parameter: `?sort=field-order` kebab-case format (e.g., `?sort=created-at-desc`)

### API Conventions (Project-Wide)
- **D-125:** Query string parameters: kebab-case (e.g., `?sort=created-at-desc`, `?limit=25`)
- **D-126:** Request/response body fields: snake_case (e.g., `{ "user_id": "...", "recovery_codes": [...] }`)

### Internal Architecture
- **D-68:** Auth service follows 4-layer architecture: domain/ (types, ports, errors), usecase/ (business logic), app/ (tonic handler), adapter/ (concrete implementations)
- **D-69:** `AuthPorts` and `AuthConfig` are **separate traits**. `AuthPorts` (domain/ports/) provides `&impl Trait` accessors (RPITIT) for data ports. `AuthConfig` (domain/ports/) provides config accessors via `fn config(&self) -> &impl ConfigAccessor` with getter methods. `AuthContext` struct (adapter/context.rs) implements both. Compound bound reused via type alias: `type Context = impl AuthPorts + AuthConfig + ?Sized;`
- **D-70:** `AuthHandler<C>` generic handler in app/handler/ -- directly implements tonic generated service trait. Depends on domain only, not adapter. Concrete type resolved in main.rs
- **D-71:** *(3A SCOPE)* Use case functions in usecase/ organized per-flow: register/, login/, session/, invite/. Each as directory with begin.rs, finish.rs etc. Additional flows (recovery/, api_key/, passkey_mgmt/, verify/) added in 3B
- **D-72:** Repository trait returns `RepositoryError` (data-access facts: NotFound, UniqueViolation, Database, Cache). Cache variant covers Redis errors. UseCase interprets into `AuthError`
- **D-73:** `AuthError` and `RepositoryError` defined per-service in domain/error/. No dependency on madome-core for error types. Handler converts `AuthError -> tonic::Status`
- **D-74:** *(3A SCOPE)* Separate domain types in domain/types/ (Credential, Session, Invite, RecoveryCode). No User domain type in auth service. `From<sea_orm::Model> for DomainType` in adapter/. `From<DomainType> for ProtoResponse` in app/. Additional types (ApiKey etc.) added in 3B
- **D-75:** All modules use `mod.rs` convention (not named file convention) for consistency in deep nesting
- **D-76:** *(3A SCOPE)* adapter/ directory structure: `adapter/postgres/`, `adapter/redis/`, `adapter/webauthn/`, `adapter/jwt/`, `adapter/user_client/` + `adapter/context.rs`. All created in 3A even if some adapters are minimal initially
- **D-77:** domain/types/, domain/ports/, domain/error/, usecase/\*, app/handler/ use directory-based organization (one concept per file)
- **D-78:** *(3A SCOPE)* Port granularity: CredentialRepository, SessionStore, InviteRepository, RecoveryCodeRepository + PasskeyProvider + JwtIssuer + UserServicePort (gRPC client to user service). ApiKeyRepository added in 3B
- **D-79:** WebAuthn ceremony state (PasskeyRegistration/PasskeyAuthentication between begin/finish) stored through SessionStore port abstraction. Concrete adapter: Redis with TTL
- **D-80:** Usecase composite return: usecase-specific result structs per flow (e.g., `RegisterResult { user_id, name, role, recovery_codes }`)
- **D-81:** Usecase data parameter naming: "payload" (e.g., `RegisterFinishPayload`)
- **D-82:** Prefer `trait_variant` over `async_trait` for async trait definitions

### WebAuthn Challenge State
- **D-97:** Challenge state TTL: 5 minutes
- **D-98:** Challenge state serialization: JSON (serde_json) via webauthn-rs `danger-allow-state-serialisation` feature. **Security constraint:** serialized state MUST remain Redis-internal only, NEVER exposed to clients
- **D-99:** Challenge state Redis key: `ceremony:{type}:{random_id}` (e.g., `ceremony:reg:abc123`, `ceremony:auth:xyz789`)
- **D-100:** Ceremony ID transport: HttpOnly cookie. begin response sets ceremony_id cookie, finish request includes it automatically. Cookie security follows JWT cookie pattern (D-44)
- **D-101:** Credential storage: JSONB column (`passkey_data`) for full webauthn-rs `Passkey` blob. Identification columns: credential_id (BYTEA, UNIQUE), name (VARCHAR), aaguid (UUID), created_at, last_used_at
- **D-102:** Concurrent ceremony policy: TTL natural expiry only (5 min). No limits

### Testing
- **D-64:** Integration tests use testcontainers for PostgreSQL + Redis (`[dev-dependencies]`). Single container per entire test suite via OnceLock. Container lifecycle managed in code (not docker-compose). `just test` = `cargo test`
- **D-64b:** Unit tests mock ports via mockall crate. Verify mockall + trait_variant compatibility in Plan 01; fallback to async_trait if incompatible
- **D-65:** Test layers: unit tests (mockall mock ports, internal `#[cfg(test)]`) + integration tests (testcontainers, `services/*/tests/`) + service tests (tonic in-process channel, `tests/service/`) + E2E (Gateway flow, `tests/e2e/`)
- **D-66:** WebAuthn testing: unit tests mock PasskeyProvider via mockall; integration and service tests use webauthn-rs SoftPasskey for real ceremony simulation
- **D-83:** Test isolation: PostgreSQL -- transaction rollback per test. Redis -- adapter key_prefix with UUID per test
- **D-84:** Test DB: per-service databases in shared PostgreSQL container (auth_test, user_test, etc.)
- **D-85:** DB migration: once per container start (not per test)
- **D-86:** Test data setup: factory functions + builder pattern
- **D-87:** Test naming convention: `should_*` BDD style
- **D-88:** Test utilities: `crates/madome-test-utils/` crate for shared utilities. Service-specific test utils in `src/test_utils.rs` (`#[cfg(test)]`)
- **D-89:** Service + E2E test crate: `tests/` directory as separate workspace member (`madome-tests`)
- **D-90:** Test config: fixed constants in madome-test-utils crate (test JWT key pair, RP_ID, RP_ORIGIN, etc.)
- **D-91:** Container images: postgres:18-alpine, redis:8-alpine (verify availability; fall back to latest stable)
- **D-92:** From impl tests: trivial field-mapping excluded. Only test From impls with transformation logic
- **D-93:** Infra failure testing: YES -- test both success and failure scenarios
- **D-94:** Service tests use real adapters (testcontainers), not mocks
- **D-95:** Test assertions: std `assert_eq!` only
- **D-96:** Test parallelism: Claude's discretion. Transaction rollback enables parallel DB tests
- **D-137:** Service tests (tests/service/auth.rs) mock UserServicePort -- test auth gRPC API contract in isolation. Keeps auth service test independent of user service code
- **D-138:** E2E tests (tests/e2e/) use real User service (both services in-process via tonic channel) for cross-service integration verification. This is the only layer where Auth + User interact via real gRPC

### Claude's Discretion
- Proto message structures (request/response types for each RPC)
- Exact Redis key structure and TTL values for JWT cache
- Recovery code generation and hashing algorithm specifics
- Migration file organization and ordering
- Exact index types (btree vs hash) based on query patterns
- Error code string constants
- Ceremony cookie name (e.g., `ceremony_id` or similar)
- Test parallelism strategy
- Transaction rollback alternatives for edge cases
- UserServicePort gRPC error to AuthError mapping (specific variant mapping)
- Opaque cursor encoding format

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Architecture
- `.planning/PROJECT.md` -- Auth Design section (Passkey + JWT + Session hybrid, Gateway JWT processing flow, duplicate JWT prevention, image request token refresh)
- `.planning/PROJECT.md` -- Service Topology (auth service role, communication flow, service-to-service gRPC)
- `.planning/PROJECT.md` -- ID Design (UUIDv4 for User, Session, API Key)
- `.planning/PROJECT.md` -- Internal Service Architecture (4-layer pattern, ports/config separation)

### Requirements
- `.planning/REQUIREMENTS.md` -- AUTH-01 through AUTH-03 (passkey registration, JWT issuance, session management)
- `.planning/REQUIREMENTS.md` -- GATE-03, GATE-04 (gateway JWT verification, refresh)

### Prior Phases
- `.planning/phases/01-foundation-and-gateway-infrastructure/01-CONTEXT.md` -- REST API format, configuration pattern, observability, workspace structure
- `.planning/phases/02-user-profile/02-CONTEXT.md` -- User service architecture, users table schema, user CRUD RPCs

### Research
- `03a-RESEARCH.md` (co-located) -- Technology stack research (webauthn-rs, jsonwebtoken, sea-orm)
- `.planning/research/ARCHITECTURE.md` -- System-level architectural patterns
- `.planning/research/ARCHITECTURE-PATTERNS.md` -- Internal service architecture patterns. **CAVEAT:** Written under incorrect "never mock DB" assumption. CONTEXT.md decisions take precedence
- `.planning/research/STACK.md` -- Technology choices

### Conventions
- `.claude/rules/rust-tracing.md` -- Tracing conventions (#[instrument] rules, event naming, log levels)
- `.claude/rules/rust-conventions.md` -- Typed constants, import style, type conversion, proto UUID handling
- `.claude/rules/rust-documentation.md` -- Per-layer rustdoc rules, comment guidelines

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `services/gateway/src/error.rs`: `AppError` enum with `From<tonic::Status>` impl. Maps gRPC status codes to HTTP status codes (NotFound, BadRequest, Unauthorized, Forbidden, Conflict, Unavailable, Internal)
- `crates/madome-common/src/caller.rs`: `CallerIdentity` struct (caller_id: Uuid, caller_role: CallerRole) + `CallerRole` enum (User, Admin, Owner) with FromStr impl
- `crates/madome-common/src/headers.rs`: Typed header constants (X_CALLER_ID, X_CALLER_ROLE, X_REQUEST_ID)
- `crates/madome-common/src/env.rs`: Environment variable parsing utilities
- `crates/madome-common/src/tracing.rs`: `init_tracing()` with registry + EnvFilter + fmt_layer pattern
- `crates/madome-proto/build.rs`: Proto compilation pipeline (tonic + prost)
- `services/gateway/src/state.rs`: `AppState` with gRPC client connections (AuthServiceClient, CatalogServiceClient, UserServiceClient)
- `services/gateway/src/routes/`: Route module pattern (one handler per file, `execute` function pattern)

### Established Patterns (from Phase 2)
- **4-layer architecture:** domain/ (types/, ports/, error/) -> usecase/ -> app/rpc/ -> adapter/ + top-level payload/ module (user service as reference implementation)
- **Ports/Config separation:** `UserPorts` trait with RPITIT (`fn user_repo(&self) -> &impl UserRepository`), separate `UserConfig` trait, `UserContext` struct implements both
- **Error chain:** `RepositoryError` (NotFound, UniqueViolation, Database) with `From<DbErr>` -> `UserError` with `From<RepositoryError>` -> `tonic::Status` with `From<UserError>`
- **Payload validation:** `#[derive(Validate)]` structs in `payload/` module with getter methods for normalization. `From<ValidationErrors>` converts to `InvalidInput` error variant. See `.claude/rules/rust-conventions.md` Input Validation section.
- **Role hierarchy:** `UserRole` enum with `level()` and `can_manage()` methods
- **REST-to-gRPC routing** in Gateway with `From<tonic::Status> for AppError`
- **Caller identity middleware:** `extract_caller_identity` in gateway/middleware/ (to be replaced by `verify_jwt` in 3A)
- **Gateway modules:** `payload/` (request DTOs), `model/` (response DTOs), `util/` (serde helpers like `to_rfc3339_ms`)
- **trait_variant + mockall:** `#[cfg_attr(test, mockall::automock)]` before `#[trait_variant::make]`
- **Per-test fresh DatabaseConnection** against shared testcontainers container via OnceLock
- **`mod.rs` convention** for all directory-based modules
- **Proto UUID as bytes:** `Uuid::from_slice(&req.id)` / `id.as_bytes().to_vec()`

### Integration Points
- `proto/auth.proto`: Currently Health RPC only -- needs full auth RPC definitions
- `services/auth/src/`: Currently minimal stub with health RPC -- needs full 4-layer architecture
- `services/gateway/src/middleware/caller_identity.rs`: Current caller identity extraction -- replaced by verify_jwt middleware
- `services/gateway/src/lib.rs`: Needs JWT verification middleware layer + auth route registration
- `services/gateway/src/state.rs`: Needs JWT public key field in AppState
- `services/gateway/src/routes/mod.rs`: Needs auth route module registration
- `justfile`: Needs docker-compose recipes + `dev-keys` recipe
- `docker-compose.yml`: Needs auth-db + redis services alongside existing user-db
- Root `Cargo.toml`: Needs workspace dependencies (webauthn-rs, jsonwebtoken, sea-orm, redis, mockall, testcontainers, trait-variant, minicbor)
- New: `crates/madome-test-utils/` -- shared test utilities crate (container setup, factory functions, test config)
- New: `tests/` -- service + E2E test crate (madome-tests workspace member)

</code_context>

<specifics>
## Specific Ideas

- Generic error responses for all auth failures -- security over debuggability at the client level
- Server logs capture detailed auth failure info with request_id correlation
- Sliding session with absolute maximum -- balance between UX convenience and session hijack protection
- Recovery codes follow GitHub/Discord pattern (8-char alphanumeric x10), hashed in DB
- Redis chosen for session storage -- TTL auto-expiry and fast lookup justify the infrastructure addition
- Owner bootstrap via invite token -- owner uses same registration flow as everyone else
- Ceremony state in Redis with HttpOnly cookie transport -- stateless auth service between begin/finish
- Fail-fast on cross-service calls -- complexity not justified at this scale; User service down = auth down
- Compensating delete on registration failure -- accept orphan risk only on double failure (auth + user service both down)
- Redis via `redis` crate with `ConnectionManager` -- minimal dep, auto-reconnect, sufficient for low-concurrency KV

</specifics>

<deferred>
## Deferred Ideas

- Rate limiting on auth endpoints -- separate phase or infrastructure concern
- JWT key rotation with kid claim
- OAuth/social login -- explicitly out of scope
- CORS configuration -- relevant when frontend is built
- Audit logging -- v2+
- API documentation (OpenAPI/Swagger) -- future phase
- All Phase 3B features (recovery flow, step-up auth, API keys, admin operations, passkey management)

</deferred>

---

*Phase: authentication-core (3A)*
*Split from: Phase 3 Authentication*
*Context gathered: 2026-03-21*
*Updated: 2026-03-25 -- context update session (middleware, dev env, error handling, testing)*
