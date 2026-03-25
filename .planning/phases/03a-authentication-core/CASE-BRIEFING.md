# Case Briefing: Phase 3A - Authentication Core

**Generated:** 2026-03-25
**Operations found:** 10
**Categories:** 5

---

## Passkey Registration

### RegisterBegin

- **Interface:** `POST /v1/auth/register/begin` (public, no auth required)
- **Auth:** None (public route, D-47)
- **Inputs:**
  - `token`: `String` -- invite token string provided to the user (EXPLICIT, D-16, D-03)
  - `name`: `String` -- display name to set on the new user account (EXPLICIT, D-05)
  - `handle`: `String` -- unique URL-safe identifier to set on the new user account (EXPLICIT, D-05)
- **Outputs:**
  - `publicKey`: `JSON` -- WebAuthn `PublicKeyCredentialCreationOptions` challenge options for the client (EXPLICIT, D-16)
  - `Set-Cookie: ceremony_id` header -- HttpOnly ceremony state cookie identifying the pending registration ceremony in Redis (EXPLICIT, D-100, D-99)
- **Decided constraints:**
  - Invite token is single-use and expires after 30 minutes (D-03)
  - Invite token validity verified in this begin step; invalid token returns generic `invite_invalid` error (D-16, D-52)
  - User Verification: Required (biometric/PIN enforced by RP configuration) (D-12)
  - Authenticator Attachment: no restriction -- platform and roaming authenticators both allowed (D-11)
  - Discoverable Credential (Resident Key): Required (D-13)
  - Attestation: None (no hardware attestation) (D-10)
  - Ceremony state stored in Redis with 5-minute TTL (D-97)
  - Challenge state key pattern: `ceremony:reg:{random_id}` (D-99)
  - Ceremony state serialized as JSON via `danger-allow-state-serialisation` feature -- MUST remain Redis-internal only, never exposed to client (D-98)
  - RP configuration sourced from `RP_ID` and `RP_ORIGIN` environment variables (D-18)
- **Open decisions:**
  - Ceremony cookie name (e.g., `ceremony_id` or similar)
  - Proto message structures for RegisterBeginRequest/RegisterBeginResponse
- **Requirements:** AUTH-01

---

### RegisterFinish

- **Interface:** `POST /v1/auth/register/finish` (public, no auth required)
- **Auth:** None (public route, D-47)
- **Inputs:**
  - `credential`: `JSON` -- WebAuthn `PublicKeyCredential` attestation response from the authenticator (EXPLICIT, D-16)
  - `Cookie: ceremony_id` -- HttpOnly cookie carrying ceremony state key; provided automatically by browser from RegisterBegin response (EXPLICIT, D-100)
- **Outputs:**
  - HTTP 201 Created
  - `user_id`: `String (UUID)` -- newly created user's UUID (EXPLICIT, D-112)
  - `handle`: `String` -- confirmed unique handle (EXPLICIT, D-112)
  - `name`: `String` -- confirmed display name (EXPLICIT, D-112)
  - `role`: `String` -- assigned role from invite record (always `"user"` in 3A) (EXPLICIT, D-112, D-02)
  - `recovery_codes`: `Array<String>` -- 10 raw 8-character alphanumeric codes, shown exactly once (EXPLICIT, D-21, D-112, D-103)
  - `Set-Cookie: <jwt_cookie>` -- HttpOnly JWT access token cookie (EXPLICIT, D-44)
  - [Inferred: session is created and session_id is embedded in the issued JWT]
- **Decided constraints:**
  - Registration flow ordering: `User.CreateUser` -> credential save -> session create -> JWT issue (D-134)
  - If auth DB or Redis fails after `User.CreateUser` succeeds, attempt compensating `User.DeleteUser`; double failure produces structured orphan log (D-139)
  - Credential stored with JSONB `passkey_data` blob + separate columns: `credential_id` (BYTEA, UNIQUE), `name`, `aaguid` (UUID), `created_at`, `last_used_at` (D-101)
  - AAGUID extracted from raw attestation CBOR (authData[37..53]) using minicbor (D-67)
  - Recovery codes hashed (SHA-256 or similar) before DB storage; raw codes returned in this response only (D-103, D-21)
  - JWT cookie: HttpOnly=true, Secure=true, SameSite=Strict, Path=/. Dev override via `COOKIE_SECURE=false` (D-44)
  - JWT claims issued: sub (user_id), role, sid (session_id), handle, name, iss ("madome-auth"), aud ("madome-gateway"), iat, exp (D-39)
  - JWT TTL: 15 minutes (D-43), algorithm ES256 (D-40)
  - User service call timeout: 5 seconds (D-135)
  - Invite token is consumed (single-use) on successful registration (D-03)
  - Role always `user` in 3A (D-02)
- **Open decisions:**
  - Recovery code generation algorithm specifics (character set details, entropy source)
  - Recovery code hashing algorithm specifics (SHA-256 vs bcrypt vs argon2)
  - Proto message structures for RegisterFinishRequest/RegisterFinishResponse
- **Requirements:** AUTH-01, AUTH-03

---

## Passkey Authentication

### LoginBegin

- **Interface:** `POST /v1/auth/login/begin` (public, no auth required)
- **Auth:** None (public route, D-47)
- **Inputs:**
  - (empty body -- username-less flow) (EXPLICIT, D-17)
- **Outputs:**
  - `publicKey`: `JSON` -- WebAuthn `PublicKeyCredentialRequestOptions` challenge options for the client (EXPLICIT, D-17)
  - `Set-Cookie: ceremony_id` header -- HttpOnly ceremony state cookie identifying the pending authentication ceremony in Redis (EXPLICIT, D-100, D-99)
- **Decided constraints:**
  - Username-less (Discoverable Credential based) -- no user identifier in request (D-17)
  - Ceremony state stored in Redis with 5-minute TTL (D-97)
  - Challenge state key pattern: `ceremony:auth:{random_id}` (D-99)
  - Ceremony state serialized as JSON -- MUST remain Redis-internal only (D-98)
  - No concurrent ceremony limits -- natural TTL expiry only (D-102)
- **Open decisions:**
  - Ceremony cookie name
  - Proto message structures for LoginBeginRequest/LoginBeginResponse
- **Requirements:** AUTH-02

---

### LoginFinish

- **Interface:** `POST /v1/auth/login/finish` (public, no auth required)
- **Auth:** None (public route, D-47)
- **Inputs:**
  - `credential`: `JSON` -- WebAuthn `PublicKeyCredential` assertion response from the authenticator (EXPLICIT, D-17)
  - `Cookie: ceremony_id` -- HttpOnly cookie carrying ceremony state key (EXPLICIT, D-100)
- **Outputs:**
  - HTTP 204 No Content (empty body) (EXPLICIT, D-111)
  - `Set-Cookie: <jwt_cookie>` -- HttpOnly JWT access token cookie (EXPLICIT, D-44, D-111)
- **Decided constraints:**
  - Login flow ordering: `User.GetUser` (by user_id from credential) -> session create -> JWT issue (D-134)
  - Deactivated user returns generic `unauthorized` (deactivation not disclosed to client) (D-53)
  - JWT claims issued: sub, role, sid, handle, name, iss, aud, iat, exp (D-39)
  - JWT TTL: 15 minutes (D-43)
  - Per-session JWT caching: in-memory with short TTL (~10 sec) -- concurrent requests within window receive same JWT (D-37)
  - User service call timeout: 5 seconds (D-135)
  - All auth failures return generic `unauthorized` error (D-50)
  - `last_used_at` updated on credential (D-101)
- **Open decisions:**
  - Proto message structures for LoginFinishRequest/LoginFinishResponse
  - UserServicePort gRPC error to AuthError mapping (specific variant mapping)
- **Requirements:** AUTH-02, AUTH-03

---

## Session & JWT

### Logout

- **Interface:** `POST /v1/auth/logout` (protected, JWT required)
- **Auth:** JWT required; any authenticated role (user/admin/owner) (D-46, D-36)
- **Inputs:**
  - `Cookie: <jwt_cookie>` -- HttpOnly JWT cookie (implicit, provided by browser) (EXPLICIT, D-44)
  - [Inferred: session_id extracted from JWT claims by gateway middleware, propagated via gRPC metadata]
- **Outputs:**
  - [Not specified -- response status code not stated; likely 204 No Content]
- **Decided constraints:**
  - Invalidates current session only (D-36); logout-all deferred to Phase 3B
  - Gateway extracts session_id from JWT `sid` claim and propagates via gRPC metadata (D-49, D-128)
  - Calls `InvalidateSession` gRPC RPC on auth service (D-59)
- **Open decisions:**
  - Response body and status code for successful logout
  - Proto message structures for InvalidateSessionRequest/InvalidateSessionResponse
- **Requirements:** AUTH-04 [Note: AUTH-04 is listed as Phase 3B scope in REQUIREMENTS.md but Logout (current session) is explicitly 3A scope per D-36]

---

### ValidateSession (internal gRPC)

- **Interface:** `ValidateSession` gRPC RPC on auth service (D-59)
- **Auth:** Internal service-to-service only (Gateway calls auth service)
- **Inputs:**
  - `session_id`: `Uuid` -- session identifier from expired JWT `sid` claim (EXPLICIT, D-59, D-128)
  - `user_id`: `Uuid` -- user identifier from expired JWT `sub` claim [Inferred: needed to look up session]
- **Outputs:**
  - Session validity result: valid/invalid boolean [Inferred: needed for refresh decision]
  - [Not specified -- whether user claims are returned for JWT reissue or just validity]
- **Decided constraints:**
  - Called by Gateway only when JWT is expired AND past grace period (D-34, PROJECT.md Auth Design)
  - Session TTL: sliding 7 days + absolute 30 days hard limit (D-33)
  - Session invalid -> Gateway rejects request (D-48, PROJECT.md Auth Design step 4)
  - User service unavailable = auth operation failure (D-136)
- **Open decisions:**
  - Proto message structures for ValidateSessionRequest/ValidateSessionResponse
- **Requirements:** AUTH-04, GATE-04

---

### RefreshToken (internal gRPC)

- **Interface:** `RefreshToken` gRPC RPC on auth service (D-59)
- **Auth:** Internal service-to-service only (Gateway calls auth service)
- **Inputs:**
  - `session_id`: `Uuid` -- session to extend sliding window for (EXPLICIT, D-33, D-59)
  - `user_id`: `Uuid` -- user to issue JWT for [Inferred: needed to populate JWT claims]
- **Outputs:**
  - New JWT string [Inferred: returned to Gateway for Set-Cookie injection]
  - [Not specified -- whether session metadata (new expiry) is also returned]
- **Decided constraints:**
  - Extends sliding session window on each successful JWT refresh (D-33)
  - Absolute 30-day hard limit enforced regardless of refresh activity (D-33)
  - Per-session JWT caching applies here too: same JWT reused within ~10 sec window (D-37)
  - Gateway calls `User.GetUser` [Inferred] or auth uses cached claims to populate new JWT
- **Open decisions:**
  - Proto message structures for RefreshTokenRequest/RefreshTokenResponse
  - Exact Redis key structure and TTL values for JWT cache
- **Requirements:** AUTH-03, AUTH-05, GATE-04

---

## Invite Management

### CreateInvite

- **Interface:** `POST /v1/auth/invites` (protected, owner-only)
- **Auth:** JWT required; owner role only -- enforced at handler level (D-02, D-119 [handler-level role check])
- **Inputs:**
  - (empty body) (EXPLICIT, D-141)
  - [Inferred: caller identity (role) extracted from JWT by gateway middleware and propagated via gRPC metadata]
- **Outputs:**
  - `token`: `String` -- one-time secret invite token for sharing (EXPLICIT, D-142)
  - `expires_at`: `String (ISO 8601)` -- confirms the 30-minute expiry window (EXPLICIT, D-142)
- **Decided constraints:**
  - Request body is empty; role is always `user` for invites created in 3A (hardcoded) (D-02, D-141)
  - Token is single-use and expires in 30 minutes (D-03)
  - Invite history recorded in DB (D-09)
  - Token hash stored in invitations table (D-58, D-56)
  - Only owner role may call this endpoint; non-owner authenticated requests rejected at handler level (D-02)
- **Open decisions:**
  - Proto message structures for CreateInviteRequest/CreateInviteResponse
- **Requirements:** [Inferred: supports AUTH-01 prerequisite -- no explicit REQ-ID covers invite creation itself in REQUIREMENTS.md]

---

## Gateway JWT Middleware

### VerifyJwt (gateway middleware)

- **Interface:** Axum `route_layer` middleware applied to all protected routes via `from_fn_with_state` (D-127, D-129)
- **Auth:** N/A -- this middleware IS the auth mechanism
- **Inputs:**
  - `Cookie: <jwt_cookie>` -- HttpOnly JWT cookie extracted from incoming HTTP request (EXPLICIT, D-44)
  - JWT public key -- loaded from `JWT_PUBLIC_KEY` env var, stored in `AppState` (D-41, D-129)
- **Outputs:**
  - On valid JWT: `Extension<CallerIdentity>` injected for downstream handlers (EXPLICIT, D-128, D-49)
  - On expired JWT within grace period: request passes through + `Set-Cookie` header with new JWT appended to response (EXPLICIT, D-34, D-129)
  - On expired JWT past grace period: calls `ValidateSession` then `RefreshToken` gRPC RPCs; on success injects identity + sets new JWT cookie (EXPLICIT, D-34, PROJECT.md Auth Design)
  - On invalid JWT (forged/corrupted): immediate 401 `unauthorized` -- no session fallback (EXPLICIT, D-48)
  - On invalid/expired session: 401 `unauthorized` (EXPLICIT, D-48, PROJECT.md Auth Design step 4)
- **Decided constraints:**
  - `CallerIdentity` fields: `caller_id` (Uuid), `caller_role` (CallerRole), `handle` (String), `session_id` (Uuid) (D-128)
  - Public routes (no middleware): `/v1/auth/register/*`, `/v1/auth/login/*`, `/health`, `/v1/health/*` (D-47)
  - JWT signature invalid -> immediate 401, no fallback (D-48)
  - Only expired JWT enters grace/session flow (D-48, D-34)
  - Grace period: 1 minute before JWT expiry (D-34)
  - `route_layer` used (not `layer`) to prevent 404-to-401 bleed (D-127)
  - User context fields propagated to gRPC services as metadata: user_id, role, handle, session_id (D-49)
  - Algorithm: ES256, iss="madome-auth", aud="madome-gateway" validation enabled (D-40, D-42)
  - All JWT errors return unified `unauthorized` to client (D-51)
  - `verify_jwt` replaces the existing `extract_caller_identity` middleware entirely (D-127)
- **Open decisions:**
  - [None -- middleware behavior fully specified in decisions]
- **Requirements:** GATE-03, GATE-04

---

## Current User

### GetCurrentUser

- **Interface:** `GET /v1/users/@me` (protected, JWT required)
- **Auth:** JWT required; any authenticated role (EXPLICIT, D-46, CONTEXT.md REST API table)
- **Inputs:**
  - [Inferred: caller identity (user_id) from `Extension<CallerIdentity>` injected by VerifyJwt middleware]
- **Outputs:**
  - User profile fields: [Not specified -- exact response shape not stated in CONTEXT.md]
  - [Inferred: name, handle, role, user_id, created_at from users table; consistent with D-57 users table schema]
- **Decided constraints:**
  - Response reflects current DB state, not JWT claims (described as "DB query, latest info" in REST API table)
  - Gateway calls `User.GetUser` gRPC RPC (EXPLICIT, D-118 pattern; D-59 gRPC RPCs list)
  - Returns current user's profile; no other user's data accessible via this endpoint
- **Open decisions:**
  - Exact response body field set and shape for GET /v1/users/@me
- **Requirements:** [Inferred: AUTH-01 validation path; no direct REQ-ID maps to this endpoint]

---

## Extraction Confidence

| Operation | Confidence | Notes |
|-----------|------------|-------|
| RegisterBegin | EXPLICIT | Interface, invite verification, ceremony flow all specified (D-16, D-97-D-100) |
| RegisterFinish | EXPLICIT | Response shape (D-112), flow ordering (D-134), compensation (D-139), recovery codes (D-21, D-103) all specified |
| LoginBegin | EXPLICIT | Username-less flow, ceremony pattern fully specified (D-17, D-97-D-100) |
| LoginFinish | EXPLICIT | Response (D-111), flow ordering (D-134), JWT caching (D-37) specified; response body confirmed empty |
| Logout | EXPLICIT | Endpoint and scope (D-36) explicit; response status code not specified |
| ValidateSession | INFERRED | RPC named in D-59; inputs/outputs derived from gateway JWT flow (PROJECT.md) and D-33/D-34 |
| RefreshToken | INFERRED | RPC named in D-59; inputs/outputs derived from session/JWT lifecycle decisions (D-33, D-37) |
| CreateInvite | EXPLICIT | Request body (D-141) and response body (D-142) fully specified |
| VerifyJwt | EXPLICIT | Middleware behavior, CallerIdentity shape (D-127-D-129), all four outcome paths specified |
| GetCurrentUser | PARTIAL | Endpoint listed in REST API table; response shape not specified in any decision |

---

## Observations

**Requirement mapping gaps:**

- AUTH-04 (session management: create, validate, invalidate) spans Logout, ValidateSession, and RefreshToken. The REQUIREMENTS.md traceability table lists AUTH-04 under "Phase 3: Authentication" without 3A/3B distinction. Per D-36, Logout (current session) is explicitly 3A scope. ValidateSession and RefreshToken are also 3A scope as they underpin GATE-04. AUTH-04 should be considered partially satisfied by 3A.
- AUTH-05 (JWT caching / duplicate prevention) is satisfied by D-37 (in-memory per-session cache with ~10 sec TTL). This is an internal behavior of RefreshToken and LoginFinish -- no dedicated operation; maps to those two.
- No explicit REQ-ID covers invite creation (CreateInvite) or current user retrieval (GetCurrentUser) in REQUIREMENTS.md. Both are 3A success criteria in ROADMAP.md (criteria 2 and implicitly criterion 5).

**Cross-cutting patterns:**

- All mutation operations on protected routes receive `CallerIdentity` via `Extension<CallerIdentity>` -- this is the sole identity source post-middleware (D-127, D-128).
- All auth failures (invalid token, deactivated user, bad JWT, expired session) return generic error strings with no specific failure reason disclosed to clients (D-50 through D-53). Detailed failure info goes to server logs with request_id correlation (D-54).
- Error response format is unified: `unauthorized` for JWT/session failures, `invite_invalid` for token failures. Error code string constants are open decisions.
- All inter-service gRPC calls (Auth -> User service, Gateway -> Auth service) use a 5-second timeout (D-135).
- User service unavailability propagates as auth operation failure -- no retry, no partial recovery (D-136).
- The seed invite (D-132) uses a fixed well-known dev token hash in migration. This is the only way owner registration can occur (D-115, D-116). Production seeding requires a separate non-committed mechanism.
- `CallerIdentity` in `madome-common` will be extended with `handle: String` and `session_id: Uuid` fields in this phase (D-128). This is a breaking change to the shared struct -- all existing consumers must be updated.
- Recovery codes are deferred at the flow level (login-with-recovery is 3B) but generated and stored in 3A (D-21, D-103). The `recovery_codes` table is created in 3A migrations even though recovery login is not implemented (D-56).
- The `api_keys` table is also created in 3A migrations for schema completeness but is not populated until 3B (D-56).
- JWT key pair provisioned via `just dev-keys` recipe (openssl, writes to `.env`) -- zero committed secrets (D-130).
