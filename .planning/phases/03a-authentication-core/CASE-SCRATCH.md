# Case Scratch: Phase 3A - Authentication Core

## Phase Rules (Pending)

- PR1: JWT claim set contract: {sub=user_id, role, sid=session_id, handle, name, iss="madome-auth", aud="madome-gateway", exp=iat+900} (D-39, D-40, D-43). All JWT-issuing operations reference this rule.
- PR2: Ceremony state deletion is best-effort; failure logged as warning, does not fail the operation. Redis TTL (5min) is the failsafe cleanup. Applies to: SignupFinish, RegisterFinish, LoginFinish.
- PR3: COOKIE_SECURE=false dev override disables Secure flag on JWT cookie (D-44). Needs unit test coverage at implementation. Applies to: SignupFinish, LoginFinish, RefreshToken (all JWT cookie setters).

---

## Operation: CreateInvite

### Rules
- R1: Only owner role can create invites — handler-level check (D-02)
- R2: Token is single-use, 30-minute expiry (D-03)
- R3: Invite history recorded in DB (D-09)
- R4: Token hash stored in invitations table (D-58)
- R5: Invite role is always `user` in 3A (D-02)
- R6: Insufficient role returns 403 Forbidden — consistent across all role-restricted endpoints

### Side Effects
- DB: invite record created (token_hash, role, expires_at, created_by)

### Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | Owner creates invite | Authenticated as owner | POST /v1/auth/invites (empty body) | 200; { token, expires_at }; invite record in DB with created_by=owner user_id, role=user | must |
| S2 | Owner creates multiple invites | Authenticated as owner | POST /v1/auth/invites x N | Each returns unique token; N invite records in DB | should |
| F1 | Unauthenticated request | No JWT cookie | POST /v1/auth/invites | 401 unauthorized | must |
| F2 | User role requests | Authenticated as user | POST /v1/auth/invites | 403 forbidden | must |
| F3 | Admin role requests | Authenticated as admin | POST /v1/auth/invites | 403 forbidden | must |
| F4 | Database unavailable | Authenticated as owner; DB down | POST /v1/auth/invites | 500 internal error; no invite created | should |
| E1 | Many invites created | Owner; no rate limit (D-55 deferred) | POST /v1/auth/invites many times | All succeed; no upper limit enforced | could |
| E2 | Rapid duplicate requests | Owner; 2 requests in quick succession | POST /v1/auth/invites x 2 | Both succeed with distinct tokens (not idempotent) | should |
| E3 | Many expired/used invites exist | Owner; large invite history | POST /v1/auth/invites | Succeeds normally; old records do not affect creation | could |

### Open Questions
(none)

---

## Operation: SignupBegin

> Replaces old RegisterBegin. User.CreateUser moved here from old RegisterFinish.
> Handle reservation removed (User.CreateUser unique constraint replaces it).
> Q1 resolved: handle 4-15, name 1-20. Q2 resolved: ceremony logic separated.

### Rules
- R1: Anyone with a valid invite token can sign up (public route)
- R2: Token is single-use, 30-minute expiry (D-03)
- R3: Invalid token returns generic invite_invalid — token existence not disclosed (D-52)
- R4: Handle: 4-15 chars, URL-safe (alphanumeric + hyphen + underscore), case-insensitive unique (D-05, D-57)
- R5: Name: 1-20 chars, non-unique display name, emoji/special chars allowed (D-05)
- R6: Processing order: input validation → token verify+consume → User.CreateUser → ceremony start
- R7: Validation failure does NOT consume token
- R8: Token consume-first is fail-safe ordering (D-52): losing a token (recoverable — owner creates new invite) is safer than allowing token reuse (security violation)
- R9: Token consumed at begin step. If User.CreateUser or ceremony start fails, new invite needed
- R10: User.CreateUser timeout: 5 seconds (D-135)
- R11: Discoverable Credential required (D-13), User Verification required (D-12)
- R12: Ceremony state stored in Redis with 5-minute TTL (D-97), key: ceremony:reg:{random_id}
- R13: Ceremony ID delivered via HttpOnly cookie (D-100)
- R14: Ceremony state content MUST NOT appear in any response body or error detail (D-98)
- R15: If ceremony start fails after User.CreateUser → compensating User.DeleteUser attempted
- R16: If SignupFinish never called within ceremony TTL → orphan user needs cleanup
- Inherits: PR1, PR2

### Side Effects
- DB: invite token marked as consumed
- Cross-service: user created via User.CreateUser (handle, name, role=user)
- Redis: ceremony state stored (ceremony:reg:{random_id}, 5min TTL)
- Cookie: ceremony_id HttpOnly cookie set

### Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | Valid signup begin | Valid invite token + valid name + valid handle | POST /v1/auth/signup/begin | Challenge options + ceremony cookie; token consumed; user created; ceremony state in Redis | must |
| S2 | Owner first registration via seed invite | Seed invite token exists | POST /v1/auth/signup/begin with seed token | Same flow as S1; seed token consumed | must |
| F1 | Token missing | No token in request body | POST /v1/auth/signup/begin | 400 invite_invalid; token not consumed | must |
| F2 | Handle empty | Valid token; empty handle | POST /v1/auth/signup/begin | 400 validation_error; token NOT consumed | must |
| F3 | Handle URL-unsafe chars | Valid token; handle with special chars | POST /v1/auth/signup/begin | 400 validation_error; token NOT consumed | must |
| F4 | Handle out of range (4-15) | Valid token; handle too short or too long | POST /v1/auth/signup/begin | 400 validation_error; token NOT consumed | must |
| F5 | Name empty | Valid token; empty name | POST /v1/auth/signup/begin | 400 validation_error; token NOT consumed | must |
| F6 | Name too long (>20) | Valid token; name exceeds max | POST /v1/auth/signup/begin | 400 validation_error; token NOT consumed | must |
| F7 | Token does not exist | Non-existent token string | POST /v1/auth/signup/begin | 400 invite_invalid | must |
| F8 | Token expired | Token older than 30 minutes | POST /v1/auth/signup/begin | 400 invite_invalid | must |
| F9 | Token already used | Previously consumed token | POST /v1/auth/signup/begin | 400 invite_invalid | must |
| F10 | Handle already taken | Valid token; handle exists (case-insensitive) | POST /v1/auth/signup/begin | 409 conflict; token consumed (fail-safe) | must |
| F11 | User service unavailable | Valid inputs; user service down | POST /v1/auth/signup/begin | 500 internal error; token consumed (fail-safe) | should |
| F12 | User service timeout | Valid inputs; user service >5s | POST /v1/auth/signup/begin | 500 internal error; token consumed (fail-safe) | should |
| F13 | Redis unavailable (ceremony) | Valid inputs; user created; Redis down | POST /v1/auth/signup/begin | 500 internal error; compensating User.DeleteUser attempted | should |
| E1 | Handle case-insensitive collision | Existing handle 'foo'; request with 'Foo' | POST /v1/auth/signup/begin | 409 conflict; treated as same handle | should |
| E2 | Handle with unicode | Valid token; handle with unicode chars | POST /v1/auth/signup/begin | 400 validation_error (URL-safe only) | should |
| E3 | Name with emoji/special chars | Valid token; name with emoji | POST /v1/auth/signup/begin | Success; emoji allowed in display name | should |

### Open Questions
(none)

---

## Operation: SignupFinish

> Replaces old RegisterFinish. User.CreateUser removed (moved to SignupBegin).
> Compensation target: user created in SignupBegin, delete on failure.
> E1 fixed: 400 ceremony_invalid (was "400 bad request").

### Rules
- R1: Flow ordering — credential verify → save credential → recovery codes → session create → JWT issue
- R2: If auth DB/Redis fails after user exists (created in SignupBegin), attempt compensating User.DeleteUser (D-139)
- R3: Double failure (compensating delete also fails) produces structured orphan log (D-139)
- R4: AAGUID extracted from attestation data at registration time (D-67)
- R5: 10 recovery codes generated, 8-char alphanumeric, hashed before DB storage (D-21, D-103)
- R6: Role inherited from invite record (always user in 3A) (D-02)
- R7: User service call timeout: 5 seconds (D-135)
- R8: Ceremony state carries user_id, name, handle from SignupBegin
- R9: handle and name in response match values from SignupBegin
- R10: Ceremony state content MUST NOT appear in any response body or error detail (D-98)
- R11: WebAuthn credential verification failure → 400 ceremony_invalid (registration is not authentication; 401 is semantically incorrect)
- Inherits: PR1 (JWT claims), PR2 (ceremony cleanup), PR3 (cookie secure)

### Side Effects

On success:
- Credential stored with AAGUID
- Recovery codes hashed and stored (10 codes)
- Session created in Redis (7d sliding / 30d absolute TTL)
- JWT cookie set (HttpOnly, Secure, SameSite=Strict) per PR1
- Ceremony state deleted from Redis (best-effort per PR2)

On failure after user exists:
- Compensating User.DeleteUser attempted
- On double failure: structured orphan log (user_id + handle + request_id)

### Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | Successful signup completion | Valid ceremony state + valid credential | POST /v1/auth/signup/finish | 201; { user_id, handle, name, role, recovery_codes }; handle and name match values from SignupBegin; JWT cookie (per PR1); all side effects | must |
| F1 | Ceremony cookie missing | No ceremony_id cookie | POST /v1/auth/signup/finish | 400 ceremony_invalid; no credential/session created | must |
| F2 | Ceremony state expired | ceremony_id refers to expired Redis key (5min TTL) | POST /v1/auth/signup/finish | 400 ceremony_invalid; orphan user may exist (cleanup per R16 of SignupBegin) | must |
| F3 | Ceremony state not found | ceremony_id refers to non-existent Redis key | POST /v1/auth/signup/finish | 400 ceremony_invalid | must |
| F4 | Invalid WebAuthn credential | Credential verification fails | POST /v1/auth/signup/finish | 400 ceremony_invalid; no credential/session created | must |
| F5 | Credential save fails | User exists but auth DB write fails | POST /v1/auth/signup/finish | 500 internal error; compensating User.DeleteUser attempted; no session/JWT | must |
| F6 | Recovery code save fails | Credential saved but recovery code write fails | POST /v1/auth/signup/finish | 500 internal error; compensating User.DeleteUser attempted | must |
| F7 | Session/Redis fails | Credential+recovery saved but Redis session fails | POST /v1/auth/signup/finish | 500 internal error; compensating User.DeleteUser attempted | must |
| F8 | Double failure (compensation fails) | User exists; auth fails; DeleteUser also fails | POST /v1/auth/signup/finish | 500 internal error; orphan log emitted (user_id + handle + request_id) | must |
| E1 | Finish called twice with same ceremony_id | First call succeeds | POST /v1/auth/signup/finish (2nd call) | 400 ceremony_invalid; ceremony state already consumed by first call | should |

### Open Questions
(none)

---

## Operation: RegisterBegin

> New operation: pure passkey ceremony start for authenticated users.
> Reuses same WebAuthn ceremony logic as SignupBegin internally.
> REST endpoint exposed in 3A for 3B "add passkey" feature readiness.

### Rules
- R1: User identity from CallerIdentity (JWT claims via VerifyJwt middleware)
- R2: Discoverable Credential required (D-13), User Verification required (D-12)
- R3: Ceremony state stored in Redis with 5-minute TTL (D-97), key: ceremony:reg:{random_id}
- R4: Ceremony state content MUST NOT appear in any response body or error detail (D-98)
- R5: No limit on passkeys per user (D-14)
- R6: No concurrent ceremony limits — TTL natural expiry only (D-102)
- R7: Ceremony ID delivered via HttpOnly cookie (D-100)

### Side Effects
- Redis: ceremony state stored (ceremony:reg:{random_id}, 5min TTL)
- Cookie: ceremony_id HttpOnly cookie set

### Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | Authenticated user starts passkey registration | Valid JWT | POST /v1/auth/register/begin (empty body) | Challenge options + ceremony cookie; ceremony state in Redis with user_id from CallerIdentity | must |
| F1 | Redis unavailable | JWT valid; Redis down | POST /v1/auth/register/begin | 500 internal error; ceremony state can't be stored | should |
| E1 | User already has passkeys | User has 1+ existing passkeys | POST /v1/auth/register/begin | Still succeeds; additive (D-14) | should |
| E2 | Multiple concurrent ceremonies | N requests in quick succession | POST /v1/auth/register/begin x N | All succeed; each gets unique ceremony (D-102) | should |

### Open Questions
(none)

---

## Operation: RegisterFinish

> New operation: pure passkey credential verify + save for authenticated users.
> No session/JWT creation (user already authenticated). No recovery codes.
> REST endpoint exposed in 3A for 3B "add passkey" feature readiness.

### Rules
- R1: WebAuthn credential verification failure → 400 ceremony_invalid (not 401; registration is not authentication)
- R2: AAGUID extracted from attestation CBOR (D-67)
- R3: Ceremony state carries user_id from RegisterBegin; must match CallerIdentity
- R4: Ceremony state content MUST NOT appear in any response body or error detail (D-98)
- R5: No session/JWT created (user already authenticated via JWT)
- R6: No recovery codes generated (add-passkey flow, not signup)
- Inherits: PR2 (ceremony cleanup)

### Side Effects
- DB: credential record created (credential_id, passkey_data, aaguid, name, created_at)
- Redis: ceremony state deleted (best-effort per PR2)

### Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | Successful passkey registration | Valid JWT + valid ceremony state + valid credential | POST /v1/auth/register/finish | 201; { credential_id }; credential stored with AAGUID; ceremony deleted | must |
| F1 | Ceremony cookie missing | No ceremony_id cookie | POST /v1/auth/register/finish | 400 ceremony_invalid; no credential saved | must |
| F2 | Ceremony state expired | ceremony_id refers to expired Redis key (5min TTL) | POST /v1/auth/register/finish | 400 ceremony_invalid | must |
| F3 | Ceremony state not found | ceremony_id refers to non-existent Redis key | POST /v1/auth/register/finish | 400 ceremony_invalid | must |
| F4 | Ceremony user_id mismatch | ceremony state user_id ≠ CallerIdentity user_id | POST /v1/auth/register/finish | 400 ceremony_invalid | must |
| F5 | Invalid WebAuthn credential | Credential verification fails | POST /v1/auth/register/finish | 400 ceremony_invalid | must |
| F6 | DB write fails | Credential verified but DB unavailable | POST /v1/auth/register/finish | 500 internal error; no credential saved | should |
| E1 | Finish called twice with same ceremony_id | First call succeeds | POST /v1/auth/register/finish (2nd call) | 400 ceremony_invalid; ceremony state already consumed | should |

### Open Questions
(none)

---

## Operation: LoginBegin

### Rules
- R1: Username-less / Discoverable Credential — no user identifier in request (D-17)
- R2: Ceremony state stored in Redis with 5-minute TTL (D-97)
- R3: No concurrent ceremony limits — TTL natural expiry only (D-102)
- R4: Ceremony ID delivered via HttpOnly cookie (D-100)
- R5: Ceremony state content MUST NOT appear in any response body or error detail (D-98)

### Side Effects
- Redis: ceremony state stored (ceremony:auth:{random_id}, 5min TTL)
- Cookie: ceremony_id HttpOnly cookie set

### Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | Valid login begin | (none required) | POST /v1/auth/login/begin (empty body) | Challenge options + ceremony cookie; ceremony state in Redis | must |
| F1 | Redis unavailable | Redis down | POST /v1/auth/login/begin | 500 internal error; ceremony state can't be stored | should |
| E1 | Already authenticated (JWT present) | Valid JWT cookie exists | POST /v1/auth/login/begin | Still succeeds; public route ignores JWT | could |
| E2 | Rapid multiple calls | N requests in quick succession | POST /v1/auth/login/begin x N | All succeed; each gets unique ceremony (D-102) | should |

### Open Questions
(none)

---

## Operation: LoginFinish

> E1 fixed: "400 bad request" → "400 ceremony_invalid" (same Redis miss as F3)

### Rules
- R1: Flow — lookup credential by credential_id from assertion → User.GetUser → session create → JWT issue (D-134)
- R2: Deactivated user returns generic `unauthorized` — deactivation not disclosed (D-53)
- R3: JWT claims per PR1
- R4: JWT caching: in-memory ~10sec — concurrent requests within window get same JWT (D-37)
- R5: User service timeout: 5 seconds (D-135)
- R6: All auth failures return generic `unauthorized` (D-50)
- R7: Credential last_used_at updated on successful login (D-101)
- R8: Ceremony state content MUST NOT appear in any response body or error detail (D-98)
- R9: WebAuthn credential verification failure → 401 unauthorized (D-50: authentication attempt, all auth failures generic)
- Inherits: PR1 (JWT claims), PR2 (ceremony cleanup), PR3 (cookie secure)

### Side Effects

On success:
- Session created in Redis (7d sliding / 30d absolute TTL)
- JWT cookie set (HttpOnly, Secure, SameSite=Strict) per PR1
- Credential last_used_at updated in DB
- Ceremony state deleted from Redis (best-effort per PR2)

### Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | Successful login | Valid ceremony state + valid credential | POST /v1/auth/login/finish | 204 No Content; JWT cookie (per PR1); session created; last_used_at updated; ceremony deleted | must |
| F1 | Ceremony cookie missing | No ceremony_id cookie | POST /v1/auth/login/finish | 400 ceremony_invalid | must |
| F2 | Ceremony state expired | ceremony_id refers to expired Redis key (5min TTL) | POST /v1/auth/login/finish | 400 ceremony_invalid | must |
| F3 | Ceremony state not found | ceremony_id refers to non-existent Redis key | POST /v1/auth/login/finish | 400 ceremony_invalid | must |
| F4 | Invalid WebAuthn credential | Credential verification fails | POST /v1/auth/login/finish | 401 unauthorized | must |
| F5 | No matching credential in DB | credential_id from assertion not found | POST /v1/auth/login/finish | 401 unauthorized | must |
| F6 | User service unavailable | Valid ceremony; user service down | POST /v1/auth/login/finish | 500 internal error | must |
| F7 | User service timeout | Valid ceremony; user service >5s | POST /v1/auth/login/finish | 500 internal error | should |
| F8 | User deactivated | Valid credential but user is_active=false | POST /v1/auth/login/finish | 401 unauthorized (deactivation hidden, D-53) | must |
| F9 | Redis fails during session creation | Credential verified; User.GetUser succeeded; Redis down | POST /v1/auth/login/finish | 500 internal error; no session/JWT created | should |
| E1 | Finish called twice with same ceremony_id | First call succeeds | POST /v1/auth/login/finish (2nd call) | 400 ceremony_invalid; ceremony state already consumed | should |
| E2 | Concurrent logins same user | Same user from different devices | POST /v1/auth/login/finish x 2 | Both succeed; unlimited concurrent sessions (D-35) | should |

### Open Questions
(none)

---

## Operation: VerifyJwt

> E-new added: broken JWT on public route → still succeeds (route_layer not applied)

### Rules
- R1: Applied to all protected routes via route_layer — prevents 404-to-401 bleed (D-127, D-129)
- R2: Invalid JWT signature (forged/corrupted) → immediate 401, no session fallback (D-48)
- R3: Only expired JWT enters grace/session flow (D-48, D-34)
- R4: Grace period: 1 minute — within 1 min of JWT expiry, Gateway issues new JWT without session verification (D-34)
- R5: Expired JWT past grace → ValidateSession gRPC → RefreshToken gRPC (D-34, PROJECT.md)
- R6: CallerIdentity fields: caller_id (Uuid), caller_role (CallerRole), handle (String), session_id (Uuid) (D-128)
- R7: Public routes exempt: /v1/auth/signup/*, /v1/auth/register/*, /v1/auth/login/*, /health, /v1/health/* (D-47)
- R8: JWT validation: ES256, iss="madome-auth", aud="madome-gateway" (D-40, D-42)
- R9: All JWT errors return unified `unauthorized` to client (D-51)
- R10: Replaces existing extract_caller_identity middleware entirely (D-127)
- Inherits: PR1 (JWT claims)

### Side Effects
- On grace period or refresh: Set-Cookie header with new JWT appended to response
- CallerIdentity injected as Extension for downstream handlers

### Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | Valid non-expired JWT | Valid JWT cookie present | Request to protected route | Extract CallerIdentity; request proceeds | must |
| S2 | Expired JWT within grace period | JWT expired <1min ago | Request to protected route | Pass through + new JWT cookie on response | must |
| S3 | Expired JWT with valid session | JWT expired >1min; session valid | Request to protected route | ValidateSession + RefreshToken; pass through + new JWT cookie | must |
| F1 | No JWT cookie | No cookie present | Request to protected route | 401 unauthorized | must |
| F2 | Invalid JWT signature | Forged or corrupted JWT | Request to protected route | 401 unauthorized; no session fallback (D-48) | must |
| F3 | Expired JWT with expired session | JWT expired; session past 30d absolute or 7d sliding | Request to protected route | 401 unauthorized | must |
| F4 | Expired JWT, refresh fails (infra) | JWT expired; session valid but RefreshToken fails due to infra | Request to protected route | 500 internal error | should |
| F5 | Wrong iss/aud claims | JWT signed correctly but iss or aud mismatch | Request to protected route | 401 unauthorized (D-42) | must |
| F6 | Auth service unavailable during refresh | JWT expired; auth service down | Request to protected route | 500 internal error | should |
| F7 | Grace-period JWT reissue fails | JWT expired <1min; signing error | Request to protected route | 500 internal error | could |
| E1 | Valid JWT on non-existent route | Valid JWT; route does not exist | Request to /v1/nonexistent | 404 not found (not 401); route_layer prevents bleed (D-127) | must |
| E2 | Concurrent requests with expiring JWT | Multiple requests near JWT expiry | Parallel requests | Same cached JWT reissued within ~10sec window (D-37) | should |
| E3 | Broken JWT on public route | Malformed/expired JWT cookie present | Request to public route (e.g., POST /v1/auth/login/begin) | Succeeds normally; public routes exempt from VerifyJwt (D-47, route_layer) | should |

### Open Questions
(none)

---

## Operation: ValidateSession

### Rules
- R1: Called only when JWT expired + grace period(1min) passed (D-34)
- R2: Session TTL — sliding 7d + absolute 30d hard limit (D-33)
- R3: gRPC timeout 5 seconds (D-135)
- R4: Pure validation — no sliding window extension (RefreshToken's responsibility)
- R5: Returns session data alongside validity to avoid redundant lookups

### Side Effects
None (read-only)

### Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | Valid session within both TTLs | Session exists; within 7d sliding and 30d absolute | ValidateSession(session_id, user_id) | Valid + session data (user_id, role, handle, name) | must |
| F1 | Session not found in Redis | Session deleted, evicted, or never existed | ValidateSession(session_id, user_id) | Invalid | must |
| F2 | Session past 30-day absolute limit | Session exists but created >30d ago | ValidateSession(session_id, user_id) | Invalid | must |
| F3 | Session past 7-day sliding window | Session exists but last activity >7d ago | ValidateSession(session_id, user_id) | Invalid | must |
| F4 | Session user_id mismatch | Session exists; stored user_id ≠ request user_id | ValidateSession(session_id, user_id) | Invalid | must |
| F5 | gRPC timeout | Auth service responds >5s | ValidateSession(session_id, user_id) | Internal error (Gateway returns 500 internal error) | should |
| E1 | Redis unavailable during validation | Redis down | ValidateSession(session_id, user_id) | Internal error (Gateway returns 500 internal error) | should |

### Open Questions
(none)

---

## Operation: RefreshToken

### Rules
- R1: Extends sliding window on success (D-33)
- R2: Per-session lock: one refresh at a time per session; concurrent requests wait for lock, then receive cached JWT. Design intent: lock ensures exactly one RefreshToken execution per session at a time; waiters get the cached result after lock release.
- R3: Cache/lock must work across multiple instances. Design intent: multi-instance safety is a project design philosophy. Either use Redis directly, or start with in-memory behind an abstraction trait so it's easily swappable to Redis when scaling out.
- R4: JWT claims from session data — freshness maintained by update operations, not RefreshToken. Design intent: when user info changes, the update operation writes to session data in Redis; RefreshToken trusts session data as-is.
- R5: gRPC timeout 5 seconds (D-135)
- Inherits: PR1 (JWT claims)

### Side Effects
- Redis: session sliding TTL reset to 7 days from now
- In-memory (or Redis): JWT cache entry written with ~10 sec TTL

### Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | Session data available | ValidateSession confirmed validity | RefreshToken(session_id, user_id, session_data) | New JWT (per PR1) + sliding window extended | must |
| F1 | Redis unavailable | Redis down for lock/extend/cache | RefreshToken(session_id, user_id, session_data) | Internal error (Gateway returns 500 internal error) | should |
| F2 | gRPC timeout | Auth service responds >5s | RefreshToken(session_id, user_id, session_data) | Internal error (Gateway returns 500 internal error) | should |
| E1 | Concurrent refresh same session | Multiple requests trigger refresh simultaneously | RefreshToken for same session_id | One executes; others wait for lock, receive cached JWT | must |

### Open Questions
| ID | Question | Impact | Default Recommendation |
|----|----------|--------|------------------------|
| Q1 | Cache/lock storage: Redis (multi-instance ready) vs in-memory (with swappable abstraction) | Deployment topology, concurrency correctness across instances | Start with in-memory behind an abstraction trait, swap to Redis when multi-instance is needed |

---

## Operation: Logout

### Rules
- R1: Invalidates current session only — logout-all is 3B (D-36)
- R2: Uses verify-only middleware (no refresh on expired JWT). Design intent: parameterized verify function — one creates full middleware (verify+refresh), another creates verify-only middleware. Shared signature verification, divergent expiry handling.
- R3: Expired but signature-valid JWT accepted — session_id extracted for invalidation
- R4: Gateway clears JWT cookie via Set-Cookie on response
- R5: Calls InvalidateSession gRPC RPC on auth service (D-59)

### Side Effects
- Redis: session deleted
- Cookie: JWT cookie cleared (Gateway Set-Cookie with expiry)

### Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | Authenticated user logs out | Valid or expired (signature-valid) JWT | POST /v1/auth/logout | 204 No Content; session deleted; JWT cookie cleared | must |
| F1 | Invalid/forged JWT | No JWT or invalid signature | POST /v1/auth/logout | 401 unauthorized | must |
| F2 | Redis unavailable | JWT valid; Redis down | POST /v1/auth/logout | 500 internal error | should |
| E1 | Already-logged-out session | Session already deleted | POST /v1/auth/logout | Success (idempotent) | should |
| E2 | Expired JWT (signature valid) | JWT expired but not forged | POST /v1/auth/logout | Accepted without refresh; session invalidated | must |

### Open Questions
(none)

---

## Operation: GetCurrentUser

> F2 expanded: "user service unavailable or timeout" (was "unavailable" only)

### Rules
- R1: Response reflects current DB state, not JWT claims
- R2: Response fields: user_id, handle, name, role, created_at, updated_at
- R3: Gateway calls User.GetUser gRPC (D-59, D-118)
- R4: gRPC timeout 5 seconds (D-135)

### Side Effects
None (read-only)

### Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | Authenticated user | Valid JWT | GET /v1/users/@me | { user_id, handle, name, role, created_at, updated_at } from DB | must |
| F1 | Unauthenticated | No JWT or invalid | GET /v1/users/@me | 401 unauthorized | must |
| F2 | User service unavailable or timeout | JWT valid; user service down or >5s | GET /v1/users/@me | 500 internal error | must |
| F3 | User not found (data inconsistency) | JWT valid but user_id not in user service | GET /v1/users/@me | 500 internal error | should |
| E1 | JWT claims stale | name/role changed since login | GET /v1/users/@me | DB has latest values, not JWT claims | should |

### Open Questions
(none)

---

## Changes Log (for next re-validate session)

### Validation findings applied:
1. **[1] JWT claims → PR1**: Phase Rule added, S1 cases reference PR1
2. **[2] E1 ceremony_invalid**: RegisterFinish E1, LoginFinish E1 → 400 ceremony_invalid
3. **[3] CreateInvite role=user**: S1 Expected Outcome now includes role=user
4. **[4] COOKIE_SECURE → PR3**: Phase Rule for dev override test coverage
5. **[5] Operation restructure**: RegisterBegin/Finish → SignupBegin/Finish + new pure RegisterBegin/Finish
6. **[6] GetCurrentUser timeout**: F2 expanded to "unavailable or timeout"
7. **[7] RefreshToken cache**: Skipped (E1 covers)
8. **[8] VerifyJwt public route**: E3 added (broken JWT on public route)
9. **PR2 ceremony cleanup**: Best-effort deletion, TTL failsafe

### Structural changes:
- RegisterBegin renamed → SignupBegin (+ User.CreateUser added from old RegisterFinish)
- RegisterFinish renamed → SignupFinish (- User.CreateUser removed)
- New RegisterBegin: pure passkey ceremony (JWT auth)
- New RegisterFinish: pure passkey credential save (JWT auth)
- Handle reservation removed from SignupBegin (User.CreateUser unique constraint replaces it)
- Q1 resolved: handle 4-15, name 1-20 (from existing user service implementation)
- Q2 resolved: ceremony logic separated into reusable operations
- VerifyJwt R7: public routes updated (/v1/auth/signup/*, /v1/auth/register/*)

### Pending for next session:
- Full re-validate of restructured operations
- Cross-operation forward concerns finalization (3A → 3B)
- Write final 03a-CASES.md
