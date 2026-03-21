# Phase 2: Authentication - Context

**Gathered:** 2026-03-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can register a passkey via invite token, authenticate, and access protected endpoints through JWT-verified gateway. Admin can manage invitations, users, and API keys. Scraper authenticates via API key. Includes session management with Redis, JWT lifecycle, and recovery code system.

</domain>

<decisions>
## Implementation Decisions

### User Onboarding
- **D-01:** Invite-only registration. Admin issues invite tokens; new users register Passkey using a valid token
- **D-02:** Admin-only invitation. Only users with admin role can issue invite tokens. Initial admin created via DB seed migration
- **D-03:** Invite token policy: single-use, 30-minute expiry. Token string returned via API, admin shares manually (e.g., Discord)
- **D-04:** Registration URL pattern: `/register?token=xxx`
- **D-05:** User sets `name` (unique) during Passkey registration
- **D-06:** Profile info: name only (no email)
- **D-07:** Role system: admin/user two-tier. Admin has invitation + user management access
- **D-08:** Admin operations in Phase 2: invite token issuance + user deactivation/reactivation. User deletion is v2
- **D-09:** Invite history recorded in DB, queryable by admin

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
- **D-19:** Passkey management: list, rename, delete (GET/PATCH/DELETE /v1/auth/passkeys/:id)
- **D-20:** Additional passkey registration for logged-in users: POST /v1/auth/passkeys/register/begin + /finish

### Recovery
- **D-21:** Recovery codes issued at registration: 10 codes, 8-character alphanumeric, each single-use
- **D-22:** Recovery code regeneration: available while logged in (POST /v1/auth/recovery-codes/regenerate). All existing codes invalidated on regeneration
- **D-23:** Recovery flow: public endpoints POST /v1/auth/recover/begin + /finish (no auth required, recovery code validates identity)

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
- **D-36:** Logout: both current session (POST /v1/auth/logout) and all sessions (POST /v1/auth/logout/all)
- **D-37:** JWT caching: in-memory in Auth service with short TTL (~10 sec). Concurrent requests within window receive same JWT (duplicate prevention per PROJECT.md)
- **D-38:** User deactivation: immediately invalidates all sessions. Next JWT expiry triggers Gateway session check -> reject

### JWT Configuration
- **D-39:** Claims: sub (user_id), role (admin/user), sid (session_id), name, iss ("madome-auth"), aud ("madome-gateway"), iat, exp
- **D-40:** Algorithm: ES256 (ECDSA P-256, aws_lc backend)
- **D-41:** Key management: JWT_PRIVATE_KEY, JWT_PUBLIC_KEY environment variables (PEM format)
- **D-42:** Validation: iss + aud claim verification enabled
- **D-43:** TTL: 15 minutes (from PROJECT.md)
- **D-44:** Cookie: HttpOnly=true, Secure=true, SameSite=Strict, Path=/. Dev override: COOKIE_SECURE=false env var
- **D-45:** Key rotation: out of Phase 2 scope. Single key pair, no kid claim

### Gateway Middleware
- **D-46:** 4-tier route authentication: public (no auth), protected (JWT required), admin (JWT + admin role), scraper (API key)
- **D-47:** Public routes: /v1/auth/register/*, /v1/auth/login/*, /v1/auth/recover/*, /health, /v1/health/*
- **D-48:** JWT signature invalid (forged/corrupted): immediate 401 (no session fallback). Only expired JWT enters grace/session flow
- **D-49:** User context propagation: user_id and role extracted from JWT, passed as gRPC metadata (same pattern as request_id)

### Auth Error Policy
- **D-50:** Client-facing errors: generic `unauthorized` for all auth failures. No specific failure reason exposed
- **D-51:** JWT errors: unified `unauthorized` (Gateway handles internally)
- **D-52:** Invite token errors: unified `invite_invalid` (token existence not disclosed)
- **D-53:** Deactivated user login: generic `unauthorized` (deactivation not disclosed)
- **D-54:** Server logs: detailed error info with request_id + user_id + specific failure reason. Logs enable debugging without client exposure
- **D-55:** Rate limiting: deferred (out of Phase 2 scope)

### DB Schema
- **D-56:** 6 tables: users, credentials, sessions, invitations, api_keys, recovery_codes
- **D-57:** users table: id (UUIDv4 PK), name (unique), role (admin/user enum), is_active (boolean), created_at, updated_at
- **D-58:** Indexes: matching query patterns (users.name UNIQUE, credentials.user_id, sessions.user_id, api_keys.key_hash UNIQUE, invitations.token_hash UNIQUE)

### Proto Definition
- **D-59:** Full auth operations defined as gRPC RPCs: RegisterBegin/Finish, LoginBegin/Finish, RecoverBegin/Finish, ValidateSession, RefreshToken, VerifyApiKey, and admin management RPCs
- **D-60:** Gateway is pure REST-to-gRPC translator for all auth operations

### Infrastructure
- **D-61:** docker-compose for PostgreSQL + Redis only. Services run via cargo run (Phase 1 pattern preserved)
- **D-62:** DB initialization: auto-migration on Auth service startup (sea-orm). Admin seed included in migration
- **D-63:** dev-start.sh: extended with `docker-compose up -d` step before service startup

### Testing
- **D-64:** All tests use testcontainers for PostgreSQL + Redis (no mock DB). Container reuse for speed optimization
- **D-65:** Test layers: contract tests (per-service gRPC contract) + integration tests (Gateway E2E flow: register -> login -> access protected)
- **D-66:** WebAuthn testing: webauthn-rs SoftPasskey/mock authenticator for server-side ceremony simulation

### Claude's Discretion
- Proto message structures (request/response types for each RPC)
- Exact Redis key structure and TTL values for JWT cache
- Credential storage format details (webauthn-rs internal types, except AAGUID which is a locked decision D-67)
- Recovery code generation algorithm specifics
- Migration file organization and ordering
- Exact index types (btree vs hash) based on query patterns
- Error code string constants

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Architecture
- `.planning/PROJECT.md` -- Auth Design section (Passkey + JWT + Session hybrid, Gateway JWT processing flow, duplicate JWT prevention, image request token refresh)
- `.planning/PROJECT.md` -- Service Topology (auth service role, communication flow)
- `.planning/PROJECT.md` -- ID Design (UUIDv4 for User, Session, API Key)

### Requirements
- `.planning/REQUIREMENTS.md` -- AUTH-01 through AUTH-05 (passkey, JWT, session management)
- `.planning/REQUIREMENTS.md` -- GATE-03 through GATE-05 (gateway JWT verification, refresh, API key)

### Prior Phase
- `.planning/phases/01-foundation-and-gateway-infrastructure/01-CONTEXT.md` -- REST API format (flat JSON, error format), configuration pattern (env vars only), observability (tracing + UUIDv7 request_id), workspace structure

### Research
- `.planning/research/ARCHITECTURE.md` -- Detailed architectural patterns
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
- `services/auth/src/service.rs`: Health RPC stub -- needs real auth implementation
- `services/gateway/src/lib.rs`: Needs JWT verification middleware layer
- `services/gateway/src/routes/mod.rs`: Needs auth route registration
- `services/gateway/src/state.rs`: Needs JWT public key and Redis connection
- `scripts/dev-start.sh`: Needs docker-compose integration
- Root `Cargo.toml`: Needs new dependencies (webauthn-rs, jsonwebtoken, sea-orm, redis)

</code_context>

<specifics>
## Specific Ideas

- Generic error responses for all auth failures -- security over debuggability at the client level
- Server logs capture detailed auth failure info with request_id correlation for debugging
- Sliding session with absolute maximum -- balance between UX convenience and session hijack protection
- Recovery codes follow GitHub/Discord pattern (8-char alphanumeric x10)
- API key format follows GitHub/Stripe convention (`madome_sk_` prefix)
- Redis chosen for session storage despite PostgreSQL-only initial constraint -- TTL auto-expiry and fast lookup justify the infrastructure addition

</specifics>

<deferred>
## Deferred Ideas

- Rate limiting on auth endpoints (brute force prevention) -- separate phase or infrastructure concern
- JWT key rotation with kid claim -- add when key management becomes a concern
- User deletion (admin operation) -- v2
- OAuth/social login -- explicitly out of scope (Passkey only per PROJECT.md)
- CORS configuration -- relevant when frontend is built
- Audit logging (beyond invite history) -- v2+

</deferred>

---

*Phase: 02-authentication*
*Context gathered: 2026-03-21*
