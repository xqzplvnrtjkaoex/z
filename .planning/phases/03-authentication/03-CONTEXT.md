# Phase: Authentication - Context

**Gathered:** 2026-03-21
**Updated:** 2026-03-22 (handle introduction)
**Status:** Ready for planning

> **Phase restructured:** Originally Phase 2, renumbered to Phase 3 after User Profile was inserted as Phase 2. Users table belongs in User service; Authentication depends on User Profile phase.

<domain>
## Phase Boundary

Users can register a passkey via invite token, authenticate, and access protected endpoints through JWT-verified gateway. Admin can manage invitations and API keys. Scraper authenticates via API key. Includes session management with Redis, JWT lifecycle, recovery code system, and step-up authentication for sensitive operations.

**Dependency:** Requires User Profile phase complete (users table, basic CRUD RPCs in user service). Auth service calls user service via gRPC for user creation and profile lookup.

</domain>

<decisions>
## Implementation Decisions

### User Onboarding
- **D-01:** Invite-only registration. Users with admin+ role issue invite tokens; new users register Passkey using a valid token
- **D-02:** *(MODIFIED)* Invite role assignment. Invite table has `role` column determining the registered user's role. Owner invite: DB seed only. Admin invite: owner can issue. User invite: admin+ can issue
- **D-03:** Invite token policy: single-use, 30-minute expiry. Token string returned via API, admin shares manually (e.g., Discord)
- **D-04:** Registration URL pattern: `/register?token=xxx`
- **D-05:** *(MODIFIED)* User sets `name` (display name, non-unique) and `handle` (unique identifier, URL-safe) during Passkey registration
- **D-06:** *(MODIFIED)* Profile info: name (display name, non-unique) and handle (unique identifier). No email. Future fields (profile picture, bio) deferred
- **D-07:** *(MODIFIED)* Role system: owner/admin/user 3-tier hierarchy (owner > admin > user). Hierarchy enforcement: can only manage roles strictly below own. Same-level interference blocked. Self-modification blocked. Minimum 1 active owner guaranteed
- **D-08:** *(MODIFIED)* Admin operations: invite issuance, invite cancellation, user deactivation/reactivation, role change, API key management. User deletion is v2
- **D-09:** Invite history recorded in DB, queryable by admin. Invites can be cancelled (revoked) before use

### Passkey Configuration
- **D-10:** Attestation: None (no hardware attestation required)
- **D-11:** Authenticator Attachment: no restriction (platform + roaming both allowed)
- **D-12:** User Verification: Required (always biometric/PIN)
- **D-13:** Discoverable Credential (Resident Key): Required (username-less login)
- **D-14:** Multiple passkeys per user allowed (multi-device support)
- **D-15:** Last passkey cannot be deleted (minimum 1 always)
- **D-67:** AAGUID stored at registration time. Extract from raw attestationObject CBOR (authData[37..53]) using minicbor, store as UUID column in credentials table. webauthn-rs does not expose AAGUID for attestation=none, so direct CBOR parsing required

### Passkey Flows
- **D-16:** Registration: 2-step (begin + finish). Invite token verified in begin step
- **D-17:** Authentication: username-less (Discoverable Credential based). No identifier in login/begin request body
- **D-18:** RP configuration: RP_ID, RP_ORIGIN environment variables
- **D-19:** *(MODIFIED)* Passkey management: list (GET, no individual GET), rename (PATCH), delete (DELETE, verified tier)
- **D-20:** Additional passkey registration for logged-in users: POST /v1/auth/passkeys/register/begin + /finish

### Recovery
- **D-21:** Recovery codes issued at registration: 10 codes, 8-character alphanumeric, each single-use
- **D-22:** *(MODIFIED)* Recovery code regeneration: verified tier (POST /v1/auth/recovery-codes/regenerate). All existing codes invalidated on regeneration
- **D-23:** *(MODIFIED)* Recovery flow: single public endpoint `POST /v1/auth/recover` (name + recovery_code in body). No challenge-response needed — single step, not begin/finish. On success: invalidates all existing sessions, issues recovery JWT (recovery: true claim)
- **D-103:** Recovery codes stored as hashes in DB (same approach as API keys D-26). Raw codes shown only once at registration. Hash algorithm: Claude's discretion (SHA-256 or similar)
- **D-104:** Recovery JWT behavior: `recovery: true` claim in JWT. Gateway restricts access to passkey registration, passkey list, and logout only. All other endpoints blocked
- **D-105:** Recovery passkey registration success → normal JWT reissued (recovery claim removed). User enters standard authenticated state

### Step-Up Authentication (Verify)
- **D-106:** Sensitive actions require recent passkey re-authentication (step-up auth)
- **D-107:** Verify flow: `POST /v1/auth/verify/begin` + `POST /v1/auth/verify/finish` (WebAuthn ceremony). Success → JWT reissued with `verified_at` timestamp claim
- **D-108:** Verify window: 5 minutes from verified_at. Gateway checks claim freshness
- **D-109:** Verified-required actions (Phase 3 scope): passkey deletion, recovery code regeneration, logout all sessions. List is extensible for future phases
- **D-110:** Step-up auth custom messaging in WebAuthn browser UI is not supported by spec. Frontend responsibility to show explanation before triggering verify flow (deferred to frontend)

### API Key Management
- **D-24:** Provisioning: DB storage + admin API (POST /v1/admin/api-keys)
- **D-25:** Key format: `madome_sk_` prefix + cryptographically secure random (CSPRNG)
- **D-26:** Key storage: SHA-256 hash in DB. Raw key shown only once at creation. Last 4 characters stored separately for identification
- **D-27:** Transport: Authorization: Bearer header. Gateway distinguishes JWT vs API key by prefix
- **D-28:** Expiry: optional expiration date settable at creation
- **D-29:** Scope: scraper-dedicated routes only. Gateway enforces route-level access control for API key auth
- **D-30:** Rotation: immediate replacement (new key creation instantly invalidates old key)
- **D-31:** Verification: delegated to Auth service via gRPC (Gateway -> Auth)

### Session Lifecycle
- **D-32:** Session storage: Redis (TTL auto-expiry + fast lookup)
- **D-33:** Session TTL: sliding 7 days + absolute 30 days. JWT refresh extends sliding window; session created_at + 30 days is hard limit
- **D-34:** JWT grace period: 1 minute (within 1 min of JWT expiry, Gateway issues new JWT without session verification)
- **D-35:** Concurrent sessions: unlimited per user
- **D-36:** Logout: current session (POST /v1/auth/logout, protected) and all sessions (POST /v1/auth/logout/all, verified)
- **D-37:** JWT caching: in-memory in Auth service with short TTL (~10 sec). Concurrent requests within window receive same JWT (duplicate prevention per PROJECT.md)
- **D-38:** User deactivation: immediately invalidates all sessions. Next JWT expiry triggers Gateway session check -> reject

### JWT Configuration
- **D-39:** *(MODIFIED)* Claims: sub (user_id), role (owner/admin/user), sid (session_id), handle, name, iss ("madome-auth"), aud ("madome-gateway"), iat, exp. Conditional claims: `recovery` (boolean, present only in recovery mode), `verified_at` (timestamp, present after step-up auth)
- **D-40:** Algorithm: ES256 (ECDSA P-256, aws_lc backend)
- **D-41:** Key management: JWT_PRIVATE_KEY, JWT_PUBLIC_KEY environment variables (PEM format)
- **D-42:** Validation: iss + aud claim verification enabled
- **D-43:** TTL: 15 minutes (from PROJECT.md)
- **D-44:** Cookie: HttpOnly=true, Secure=true, SameSite=Strict, Path=/. Dev override: COOKIE_SECURE=false env var
- **D-45:** Key rotation: out of scope. Single key pair, no kid claim

### Gateway Middleware
- **D-46:** *(MODIFIED)* 5-tier route authentication: public (no auth), protected (JWT required), verified (JWT + recent verify), admin (JWT + admin+ role), scraper (API key). Recovery JWT is a special case of protected — Gateway checks recovery claim and restricts to allowed endpoints
- **D-47:** *(MODIFIED)* Public routes: /v1/auth/register/*, /v1/auth/login/*, /v1/auth/recover, /health, /v1/health/*
- **D-48:** JWT signature invalid (forged/corrupted): immediate 401 (no session fallback). Only expired JWT enters grace/session flow
- **D-49:** User context propagation: user_id and role extracted from JWT, passed as gRPC metadata (same pattern as request_id)

### Auth Error Policy
- **D-50:** Client-facing errors: generic `unauthorized` for all auth failures. No specific failure reason exposed
- **D-51:** JWT errors: unified `unauthorized` (Gateway handles internally)
- **D-52:** Invite token errors: unified `invite_invalid` (token existence not disclosed)
- **D-53:** Deactivated user login: generic `unauthorized` (deactivation not disclosed)
- **D-54:** Server logs: detailed error info with request_id + user_id + specific failure reason. Logs enable debugging without client exposure
- **D-55:** Rate limiting: deferred (out of scope)

### Auth DB Schema
- **D-56:** *(MODIFIED)* 5 tables in auth DB: credentials, sessions, invitations, api_keys, recovery_codes. users table is in User service DB. Auth tables reference user_id without FK (cross-service boundary)
- **D-57:** *(MODIFIED)* users table (User service DB): id (UUIDv4 PK), handle (unique, case-insensitive), name (display name, non-unique), role (owner/admin/user enum), is_active (boolean), created_at, updated_at. Owner role value included in enum
- **D-58:** Indexes: matching query patterns (credentials.user_id, sessions.user_id, api_keys.key_hash UNIQUE, invitations.token_hash UNIQUE, recovery_codes.user_id)

### Proto Definition
- **D-59:** *(MODIFIED)* Auth gRPC RPCs: RegisterBegin/Finish, LoginBegin/Finish, Recover (single), VerifyBegin/Finish, ValidateSession, RefreshToken, VerifyApiKey, InvalidateAllSessions, and credential management RPCs (ListPasskeys, RenamePasskey, DeletePasskey, RegenerateRecoveryCodes)
- **D-60:** *(MODIFIED)* Gateway is REST-to-gRPC translator for auth operations. For cross-service operations (deactivate), Gateway orchestrates sequential calls to multiple services

### Infrastructure
- **D-61:** docker-compose for PostgreSQL + Redis (latest stable versions). Services run via cargo run (Phase 1 pattern preserved)
- **D-62:** *(MODIFIED)* DB initialization: auto-migration on service startup (sea-orm). Owner seed: migration creates owner-role invite token. Owner registers through standard flow
- **D-63:** justfile: recipes for `docker-compose up -d`, service startup, and common dev commands. No shell scripts in scripts/

### REST API Endpoints

**Auth endpoints (`/v1/auth/`):**

| Tier | Method | Path | Description |
|------|--------|------|-------------|
| public | POST | /v1/auth/register/begin | Passkey registration start (invite token verified) |
| public | POST | /v1/auth/register/finish | Passkey registration complete → 201, recovery codes in body |
| public | POST | /v1/auth/login/begin | Login start (username-less) |
| public | POST | /v1/auth/login/finish | Login complete → 204 No Content, JWT in cookie |
| public | POST | /v1/auth/recover | Recovery code auth → recovery JWT in cookie |
| protected | POST | /v1/auth/logout | Logout current session |
| protected | GET | /v1/auth/passkeys | List own passkeys |
| protected | PATCH | /v1/auth/passkeys/:id | Rename passkey |
| protected | POST | /v1/auth/passkeys/register/begin | Additional passkey registration start |
| protected | POST | /v1/auth/passkeys/register/finish | Additional passkey registration complete |
| protected | POST | /v1/auth/verify/begin | Step-up auth start |
| protected | POST | /v1/auth/verify/finish | Step-up auth complete → JWT reissued with verified_at |
| verified | DELETE | /v1/auth/passkeys/:id | Delete passkey (requires recent verify) |
| verified | POST | /v1/auth/recovery-codes/regenerate | Regenerate recovery codes (requires recent verify) |
| verified | POST | /v1/auth/logout/all | Logout all sessions (requires recent verify) |
| recovery | GET | /v1/auth/passkeys | List passkeys (allowed in recovery mode) |
| recovery | POST | /v1/auth/passkeys/register/begin | Register passkey (recovery purpose) |
| recovery | POST | /v1/auth/passkeys/register/finish | Complete passkey registration → normal JWT |
| recovery | POST | /v1/auth/logout | Logout recovery session |

- **D-111:** Login success response: 204 No Content (empty body). JWT delivered via HttpOnly cookie only
- **D-112:** *(MODIFIED)* Registration success response: 201 Created with `{ "user_id", "handle", "name", "role", "recovery_codes" }`. Recovery codes are raw strings shown this once only

**Admin endpoints (`/v1/admin/`):**

| Tier | Method | Path | Routes To | Description |
|------|--------|------|-----------|-------------|
| admin | GET | /v1/admin/users | User svc | User list (paginated) |
| admin | GET | /v1/admin/users/:id | User svc | User detail |
| admin | POST | /v1/admin/users/:id/deactivate | User svc + Auth svc | Deactivate (compensating tx) |
| admin | POST | /v1/admin/users/:id/activate | User svc | Reactivate |
| admin | PATCH | /v1/admin/users/:id/role | User svc | Change role `{ "role": "admin" }` |
| admin | POST | /v1/admin/invites | Auth svc | Issue invite (with role) |
| admin | GET | /v1/admin/invites | Auth svc | Invite history (paginated) |
| admin | DELETE | /v1/admin/invites/:id | Auth svc | Cancel invite |
| admin | POST | /v1/admin/api-keys | Auth svc | Create API key |
| admin | GET | /v1/admin/api-keys | Auth svc | API key list (paginated) |
| admin | DELETE | /v1/admin/api-keys/:id | Auth svc | Revoke API key |

- **D-113:** Admin endpoint routing split: `/v1/admin/users/*` → User service, `/v1/admin/invites/*` and `/v1/admin/api-keys/*` → Auth service
- **D-114:** *(MODIFIED)* Admin user info response (list/detail): id, handle, name, role, is_active, created_at (basic fields only)

**User endpoints (`/v1/user/`):**

| Tier | Method | Path | Routes To | Description |
|------|--------|------|-----------|-------------|
| protected | GET | /v1/user/@me | User svc | Current user profile (DB query, latest info) |

### Owner Bootstrapping
- **D-115:** Owner is DB seed only. Exactly 1 owner. Cannot create or change owner role via API. Owner role not available in invite creation API
- **D-116:** Owner bootstrap: seed migration creates a single owner-role invite token. Owner uses standard registration flow (POST /v1/auth/register/*) to set name + passkey. Token hash stored in invitations table like any other invite

### Cross-Service Communication
- **D-117:** Service-to-service direct gRPC calls allowed (project-wide architecture decision). Not restricted to Gateway-to-service only
- **D-118:** *(MODIFIED)* Auth → User service calls: CreateUser (registration finish, passes name + handle + role from invite), GetUser (by ID, login finish, for JWT claims including handle). Auth service requires `USER_GRPC_ADDR` environment variable
- **D-119:** Gateway orchestration for cross-service operations: sequential calls with compensating transactions. Level 1 now (compensate failure → log + error return). Level 2 later (outbox + background worker retry)
- **D-120:** Deactivate orchestration: Gateway → User (DeactivateUser) → Auth (InvalidateAllSessions). Auth failure → compensate by calling User (ActivateUser). Compensate failure → error log + error response to client

### Pagination (Project-Wide)
- **D-121:** All list endpoints use cursor-based pagination. Opaque URL-safe cursor value
- **D-122:** Cursor transport: server → client via `X-Next-Cursor` response header (omitted when no more pages). Client → server via `?cursor=` query parameter. Response body stays flat array (Phase 1 convention preserved)
- **D-123:** Page size: `?limit=N` query parameter. Default 25, max 100. Exceptions documented per-endpoint where applicable
- **D-124:** Sort parameter: `?sort=field-order` kebab-case format (e.g., `?sort=created-at-desc`). Client-specified sort criteria

### API Conventions (Project-Wide)
- **D-125:** Query string parameters: kebab-case (e.g., `?sort=created-at-desc`, `?limit=25`)
- **D-126:** Request/response body fields: snake_case (e.g., `{ "user_id": "...", "recovery_codes": [...] }`)

### Internal Architecture
- **D-68:** Auth service follows 4-layer architecture: domain/ (types, ports, errors), usecase/ (business logic), app/ (tonic handler), adapter/ (concrete implementations). See PROJECT.md Internal Service Architecture
- **D-69:** `AuthPorts` and `AuthConfig` are **separate traits**. `AuthPorts` (domain/ports/) provides `&impl Trait` accessors (RPITIT) for data ports. `AuthConfig` (domain/ports/) provides config accessors via `fn config(&self) -> &impl ConfigAccessor` with getter methods (e.g., `ctx.config().jwt_ttl()`). `AuthContext` struct (adapter/context.rs) implements both AuthPorts and AuthConfig. Compound bound reused via type alias: `type Context = impl AuthPorts + AuthConfig + ?Sized;`
- **D-70:** `AuthHandler<C>` generic handler in app/handler/ — directly implements tonic generated service trait (`impl AuthService for AuthHandler<C>`). Depends on domain only, not adapter. Concrete type resolved in main.rs
- **D-71:** Use case functions in usecase/ organized per-flow: register/, login/, session/, recovery/, api_key/, passkey_mgmt/, verify/. Each as directory with begin.rs, finish.rs etc. Shared logic between flows handled by delegation to the owning usecase (e.g., register/finish delegates session creation to session/create)
- **D-72:** Repository trait returns `RepositoryError` (data-access facts: NotFound, UniqueViolation, Database, Cache). Cache variant covers Redis errors — DB and Redis are both "data access" ports returning the same error type. UseCase interprets into `AuthError` (business meaning: InviteInvalid, UserNameTaken, etc.)
- **D-73:** `AuthError` and `RepositoryError` defined per-service in domain/error/. No dependency on madome-core for error types. Handler converts `AuthError -> tonic::Status`
- **D-74:** *(MODIFIED)* Separate domain types in domain/types/ (Credential, Session, Invite, ApiKey, RecoveryCode, etc.). No User domain type in auth service — user data comes from User service via gRPC. `From<sea_orm::Model> for DomainType` in adapter/ (dependency-safe). `From<DomainType> for ProtoResponse` in app/
- **D-75:** All modules use `mod.rs` convention (not named file convention) for consistency in deep nesting
- **D-76:** adapter/ uses directory structure from the start: `adapter/postgres/`, `adapter/redis/`, `adapter/webauthn/`, `adapter/jwt/`, `adapter/user_client/` + `adapter/context.rs`
- **D-77:** domain/types/, domain/ports/, domain/error/, usecase/*, app/handler/ use directory-based organization (one concept per file)
- **D-78:** *(MODIFIED)* Port granularity: 5 repository ports (CredentialRepository, SessionStore, InviteRepository, ApiKeyRepository, RecoveryCodeRepository) + PasskeyProvider + JwtIssuer + UserServicePort (gRPC client to user service). UserRepository moved to user service
- **D-79:** WebAuthn ceremony state (PasskeyRegistration/PasskeyAuthentication between begin/finish) stored through SessionStore port abstraction. Concrete adapter: Redis with TTL. See D-97 through D-102 for details
- **D-80:** Usecase composite return: usecase-specific result structs per flow (e.g., `RegisterResult { user_id, name, role, recovery_codes }` in usecase/register/). Not domain types — workflow results don't belong in domain/types/
- **D-81:** Usecase data parameter naming: "payload" (e.g., `RegisterFinishPayload`), not "input"
- **D-82:** Prefer `trait_variant` over `async_trait` for async trait definitions. Use `async_trait` only when `trait_variant` is insufficient

### WebAuthn Challenge State
- **D-97:** Challenge state TTL: 5 minutes. Covers slow authenticators (Bluetooth roaming keys)
- **D-98:** Challenge state serialization: JSON (serde_json) via webauthn-rs `danger-allow-state-serialisation` feature. **Security constraint:** serialized state contains challenge nonce and internal verification data — MUST remain Redis-internal only, NEVER exposed to clients via API responses
- **D-99:** Challenge state Redis key: `ceremony:{type}:{random_id}` (e.g., `ceremony:reg:abc123`, `ceremony:auth:xyz789`, `ceremony:verify:def456`). random_id generated at begin step
- **D-100:** Ceremony ID transport: HttpOnly cookie. begin response sets ceremony_id cookie, finish request includes it automatically. Cookie security settings follow JWT cookie pattern (D-44: Secure, SameSite=Strict)
- **D-101:** Credential storage: JSONB column (`passkey_data`) for full webauthn-rs `Passkey` blob. Identification/management columns extracted separately: credential_id (BYTEA, UNIQUE), name (VARCHAR), aaguid (UUID, D-67), created_at, last_used_at
- **D-102:** Concurrent ceremony policy: TTL natural expiry only (5 min). No per-user or per-device limits. Abuse prevention deferred to Rate Limiting (D-55)

### Testing
- **D-64:** Integration tests use testcontainers for PostgreSQL + Redis (`[dev-dependencies]`). Single container per entire test suite via OnceLock — all tests share one PostgreSQL and one Redis container. Container lifecycle managed in code (not docker-compose). `just test` = `cargo test`
- **D-64b:** Unit tests mock ports via mockall crate. Verify mockall + trait_variant compatibility in Plan 01; fallback to async_trait if incompatible
- **D-65:** Test layers: unit tests (mockall mock ports, test usecase logic, internal `#[cfg(test)]`) + integration tests (testcontainers, test adapters, `services/*/tests/`) + service tests (tonic in-process channel, real adapters + testcontainers, `tests/service/`) + E2E (Gateway flow, `tests/e2e/`)
- **D-66:** WebAuthn testing: unit tests mock PasskeyProvider via mockall; integration and service tests use webauthn-rs SoftPasskey for real ceremony simulation
- **D-83:** Test isolation: PostgreSQL — transaction rollback per test (Claude's discretion for cases where rollback is impractical). Redis — adapter key_prefix with UUID per test (programmatic isolation, no human-assigned prefixes)
- **D-84:** Test DB: per-service databases in shared PostgreSQL container (auth_test, user_test, catalog_test, etc.). Matches production isolation model
- **D-85:** DB migration: once per container start (not per test). All tests share the migrated schema
- **D-86:** Test data setup: factory functions for frequently used entities + builder pattern for one-off customizations. Mix as appropriate
- **D-87:** Test naming convention: `should_*` BDD style (e.g., `should_return_challenge_when_valid_invite`, `should_reject_expired_invite`)
- **D-88:** Test utilities: `crates/madome-test-utils/` crate for shared utilities (container setup, factory functions, test config). Service-specific test utils in `src/test_utils.rs` (`#[cfg(test)]`)
- **D-89:** Service + E2E test crate: `tests/` directory as separate workspace member (`madome-tests`). Structure: `tests/service/auth.rs`, `tests/e2e/auth_flow.rs`
- **D-90:** Test config: fixed constants in madome-test-utils crate (test JWT key pair, RP_ID, RP_ORIGIN, etc.)
- **D-91:** Container images: postgres:18-alpine, redis:8-alpine (verify availability at implementation time; fall back to latest stable if unavailable)
- **D-92:** From impl tests: trivial field-mapping From impls excluded from testing. Only test From impls with transformation logic
- **D-93:** Infra failure testing: YES — test both success and failure scenarios (connection failures, timeouts, etc.)
- **D-94:** Service tests use real adapters (testcontainers), not mock adapters
- **D-95:** Test assertions: std `assert_eq!` only (no pretty_assertions or external assertion libraries)
- **D-96:** Test parallelism: Claude's discretion. Transaction rollback enables parallel DB tests; serialize only when necessary

### Claude's Discretion
- Proto message structures (request/response types for each RPC)
- Exact Redis key structure and TTL values for JWT cache
- Recovery code generation and hashing algorithm specifics
- Migration file organization and ordering
- Exact index types (btree vs hash) based on query patterns
- Error code string constants
- Ceremony cookie name (e.g., `ceremony_id` or similar)
- Test parallelism strategy (serialize specific tests as needed)
- Transaction rollback alternatives for edge cases where rollback is impractical
- UserServicePort error handling strategy (gRPC errors from user service)
- Opaque cursor encoding format (base64, etc.)
- Retry count for compensating transactions

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
- `.planning/REQUIREMENTS.md` -- AUTH-01 through AUTH-05 (passkey, JWT, session management)
- `.planning/REQUIREMENTS.md` -- GATE-03 through GATE-05 (gateway JWT verification, refresh, API key)

### Prior Phases
- `.planning/phases/01-foundation-and-gateway-infrastructure/01-CONTEXT.md` -- REST API format (flat JSON, error format), configuration pattern (env vars only), observability (tracing + UUIDv7 request_id), workspace structure
- `.planning/phases/02-user-profile/02-CONTEXT.md` -- User service architecture, users table schema, user CRUD RPCs

### Research
- `.planning/research/ARCHITECTURE.md` -- Detailed architectural patterns (system-level)
- `.planning/research/ARCHITECTURE-PATTERNS.md` -- Internal service architecture patterns (Clean/Hexagonal/Pragmatic comparison). **CAVEAT:** Written under incorrect "never mock DB" assumption. CONTEXT.md decisions take precedence
- `.planning/research/STACK.md` -- Technology choices (webauthn-rs, jsonwebtoken, sea-orm versions)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/madome-core/src/error.rs`: AppError with Unauthorized/Forbidden variants already defined. gRPC Status mapping established
- `crates/madome-common/src/env.rs`: Environment variable parsing utilities
- `crates/madome-common/src/tracing.rs`: Tracing initialization
- `crates/madome-proto/build.rs`: Proto compilation pipeline (tonic + prost)
- `services/gateway/src/state.rs`: Gateway state management pattern (gRPC client connections)
- `services/gateway/src/routes/`: Route module pattern established in Phase 1

### Established Patterns
- REST-to-gRPC routing: Gateway receives REST, calls gRPC backend, maps response
- gRPC Status to HTTP status mapping in AppError
- Environment variable based configuration (no config files)
- UUIDv7 request_id via gRPC metadata propagation
- Integration testing via HTTP requests to running Gateway
- Service binary structure: main.rs + service.rs (lib pattern for testability)

### Integration Points
- `proto/auth.proto`: Currently Health RPC only -- needs full auth RPC definitions
- `proto/user.proto`: Currently Health RPC only -- needs user CRUD RPCs (User Profile phase)
- `services/auth/src/service.rs`: Health RPC stub -- needs full 4-layer architecture replacement
- `services/gateway/src/lib.rs`: Needs JWT verification middleware layer + compensating transaction orchestration
- `services/gateway/src/routes/mod.rs`: Needs auth route registration + admin route registration
- `services/gateway/src/state.rs`: Needs JWT public key + user service client
- `justfile`: Needs docker-compose recipes and service startup commands
- Root `Cargo.toml`: Needs new dependencies (webauthn-rs, jsonwebtoken, sea-orm, redis, mockall, testcontainers, trait-variant)
- New: `crates/madome-test-utils/` -- shared test utilities crate (workspace member)
- New: `tests/` -- service + E2E test crate (madome-tests, workspace member)

</code_context>

<specifics>
## Specific Ideas

- Generic error responses for all auth failures -- security over debuggability at the client level
- Server logs capture detailed auth failure info with request_id correlation for debugging
- Sliding session with absolute maximum -- balance between UX convenience and session hijack protection
- Recovery codes follow GitHub/Discord pattern (8-char alphanumeric x10), hashed in DB like API keys
- API key format follows GitHub/Stripe convention (`madome_sk_` prefix)
- Redis chosen for session storage despite PostgreSQL-only initial constraint -- TTL auto-expiry and fast lookup justify the infrastructure addition
- Owner bootstrap via invite token -- owner uses same registration flow as everyone else, no special endpoints
- Compensating transactions chosen over eventual consistency for cross-service operations -- security correctness over implementation simplicity

</specifics>

<deferred>
## Deferred Ideas

- Rate limiting on auth endpoints (brute force prevention) -- separate phase or infrastructure concern
- JWT key rotation with kid claim -- add when key management becomes a concern
- User deletion (admin operation) -- v2
- OAuth/social login -- explicitly out of scope (Passkey only per PROJECT.md)
- CORS configuration -- relevant when frontend is built
- Audit logging (beyond invite history) -- v2+
- API documentation (OpenAPI/Swagger) -- future phase
- Step-up auth UI messaging -- frontend responsibility, not backend
- Recovery code count/status endpoint -- decide in future phase
- Compensating transaction Level 2 (outbox + background worker) -- when operational complexity justifies it

</deferred>

---

*Phase: authentication*
*Context gathered: 2026-03-21*
*Updated: 2026-03-22 -- Area A (endpoint map, pagination, API conventions), Area B (3-tier roles, owner bootstrap, admin management, invite role, compensating tx), Area C (response formats, recovery redesign, step-up auth), Area D (user service separation, cross-service communication, phase restructuring), Handle introduction (D-05, D-06, D-39, D-57, D-112, D-114, D-118 modified)*
