# Case Scratch: Phase 3A - Authentication Core

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
| S1 | Owner creates invite | Authenticated as owner | POST /v1/auth/invites (empty body) | 200; { token, expires_at }; invite record in DB | must |
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

## Operation: RegisterBegin

### Rules
- R1: Invite token is single-use, 30-minute expiry (D-03)
- R2: Invalid token returns generic invite_invalid — token existence not disclosed (D-52)
- R3: Handle is unique, URL-safe, case-insensitive (D-05, D-57)
- R4: Name is non-unique display name, emoji/special chars allowed (D-05)
- R5: Ceremony state stored in Redis with 5-minute TTL (D-97)
- R6: Ceremony ID delivered via HttpOnly cookie (D-100)
- R7: Discoverable Credential required (D-13)
- R8: User Verification required (D-12)
- R9: Processing order — input validation → token verify+consume → ceremony start. Validation failure does NOT consume token
- R10: Token consumed at begin step. If finish fails, new invite needed

### Side Effects
- DB: invite token marked as consumed
- Redis: ceremony state stored (ceremony:reg:{random_id}, 5min TTL)
- Cookie: ceremony_id HttpOnly cookie set

### Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | Valid registration begin | Valid invite token + valid name + valid handle | POST /v1/auth/register/begin | Challenge options + ceremony cookie; token consumed; ceremony state in Redis | must |
| S2 | Owner first registration via seed invite | Seed invite token exists | POST /v1/auth/register/begin with seed token | Same flow as S1; seed token consumed | must |
| F1 | Token missing | No token in request body | POST /v1/auth/register/begin | invite_invalid; token not consumed | must |
| F2 | Token does not exist | Non-existent token string | POST /v1/auth/register/begin | invite_invalid | must |
| F3 | Token expired | Token older than 30 minutes | POST /v1/auth/register/begin | invite_invalid | must |
| F4 | Token already used | Previously consumed token | POST /v1/auth/register/begin | invite_invalid | must |
| F5 | Handle empty | Valid token; empty handle | POST /v1/auth/register/begin | Validation error; token NOT consumed | must |
| F6 | Handle URL-unsafe chars | Valid token; handle with special chars | POST /v1/auth/register/begin | Validation error; token NOT consumed | must |
| F7 | Handle already taken | Valid token; handle exists (case-insensitive) | POST /v1/auth/register/begin | 409 conflict; token NOT consumed | must |
| F8 | Name empty | Valid token; empty name | POST /v1/auth/register/begin | Validation error; token NOT consumed | must |
| F9 | Database unavailable | Valid inputs; DB down | POST /v1/auth/register/begin | Internal error; token not consumed (can't write) | should |
| F10 | Redis unavailable | Valid inputs; Redis down | POST /v1/auth/register/begin | Internal error; token may be consumed but ceremony not stored | should |
| E1 | Handle case-insensitive collision | Existing handle 'foo'; request with 'Foo' | POST /v1/auth/register/begin | 409 conflict; treated as same handle | should |
| E2 | Handle with unicode | Valid token; handle with unicode chars | POST /v1/auth/register/begin | Validation error (URL-safe only) | should |
| E3 | Name with emoji/special chars | Valid token; name with emoji | POST /v1/auth/register/begin | Success; emoji allowed in display name | should |

### Open Questions
| ID | Question | Impact | Default Recommendation |
|----|----------|--------|------------------------|
| Q1 | Specific max length limits for handle and name | Boundary value tests, DB column sizing | Planner discretion (e.g., handle: 32, name: 64) |
| Q2 | Design Register ceremony logic to be reusable for 3B passkey addition | Architecture, code reuse across phases | Planner separates ceremony logic from registration orchestration internally |

---

## Operation: RegisterFinish

### Rules
- R1: Flow ordering — User.CreateUser → credential save → session create → JWT issue (D-134)
- R2: If auth DB/Redis fails after CreateUser succeeds, attempt compensating User.DeleteUser (D-139)
- R3: Double failure (compensating delete also fails) produces structured orphan log (D-139)
- R4: AAGUID extracted from attestation data at registration time (D-67)
- R5: 10 recovery codes generated, 8-char alphanumeric, hashed before DB storage (D-21, D-103)
- R6: JWT: ES256, 15min TTL, HttpOnly cookie (D-40, D-43, D-44)
- R7: Role inherited from invite record (always user in 3A) (D-02)
- R8: User service call timeout: 5 seconds (D-135)
- R9: Ceremony state carries name/handle from Begin; Finish retrieves them for user creation

### Side Effects

On success:
- User created via User.CreateUser (cross-service)
- Credential stored with AAGUID
- Recovery codes hashed and stored (10 codes)
- Session created in Redis (7d sliding / 30d absolute TTL)
- JWT cookie set (HttpOnly, Secure, SameSite=Strict)
- Ceremony state deleted from Redis

On failure after CreateUser:
- Compensating User.DeleteUser attempted
- On double failure: structured orphan log (user_id + handle + request_id)

### Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | Successful registration | Valid ceremony state + valid credential | POST /v1/auth/register/finish | 201; { user_id, handle, name, role, recovery_codes }; JWT cookie; all side effects | must |
| F1 | Ceremony cookie missing | No ceremony_id cookie | POST /v1/auth/register/finish | Error; no user created | must |
| F2 | Ceremony state expired | ceremony_id refers to expired Redis key (5min TTL) | POST /v1/auth/register/finish | Error; no user created | must |
| F3 | Ceremony state not found | ceremony_id refers to non-existent Redis key | POST /v1/auth/register/finish | Error; no user created | must |
| F4 | Invalid WebAuthn credential | Credential verification fails | POST /v1/auth/register/finish | Error; no user created | must |
| F5 | User service unavailable | Valid ceremony; user service down | POST /v1/auth/register/finish | Error; no credential/session/JWT created | must |
| F6 | User service timeout | Valid ceremony; user service >5s | POST /v1/auth/register/finish | Error; same as F5 | should |
| F7 | Credential save fails after CreateUser | User created but auth DB write fails | POST /v1/auth/register/finish | Error; compensating DeleteUser attempted; no session/JWT | must |
| F8 | Session/Redis fails after CreateUser+credential | User+credential created but Redis fails | POST /v1/auth/register/finish | Error; compensating DeleteUser attempted | must |
| F9 | Double failure (compensation fails) | CreateUser succeeded; auth fails; DeleteUser also fails | POST /v1/auth/register/finish | Error; orphan log emitted (user_id + handle + request_id) | must |
| E1 | Finish called twice with same ceremony_id | First call succeeds | POST /v1/auth/register/finish (2nd call) | Error; ceremony state already consumed | should |

### Open Questions
(none)

---

## Operation: LoginBegin

### Rules
- R1: Username-less / Discoverable Credential — no user identifier in request (D-17)
- R2: Ceremony state stored in Redis with 5-minute TTL (D-97)
- R3: No concurrent ceremony limits — TTL natural expiry only (D-102)
- R4: Ceremony ID delivered via HttpOnly cookie (D-100)

### Side Effects
- Redis: ceremony state stored (ceremony:auth:{random_id}, 5min TTL)
- Cookie: ceremony_id HttpOnly cookie set

### Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | Valid login begin | (none required) | POST /v1/auth/login/begin (empty body) | Challenge options + ceremony cookie; ceremony state in Redis | must |
| F1 | Redis unavailable | Redis down | POST /v1/auth/login/begin | Internal error; ceremony state can't be stored | should |
| E1 | Already authenticated (JWT present) | Valid JWT cookie exists | POST /v1/auth/login/begin | Still succeeds; public route ignores JWT | could |
| E2 | Rapid multiple calls | N requests in quick succession | POST /v1/auth/login/begin x N | All succeed; each gets unique ceremony (D-102) | should |

### Open Questions
(none)

---

## Operation: LoginFinish

### Rules
- R1: Flow — lookup credential by credential_id from assertion → User.GetUser → session create → JWT issue (D-134)
- R2: Deactivated user returns generic `unauthorized` — deactivation not disclosed (D-53)
- R3: JWT claims: sub, role, sid, handle, name, iss, aud, iat, exp (D-39)
- R4: JWT caching: in-memory ~10sec — concurrent requests within window get same JWT (D-37)
- R5: User service timeout: 5 seconds (D-135)
- R6: All auth failures return generic `unauthorized` (D-50)
- R7: Credential last_used_at updated on successful login (D-101)

### Side Effects

On success:
- Session created in Redis (7d sliding / 30d absolute TTL)
- JWT cookie set (HttpOnly, Secure, SameSite=Strict)
- Credential last_used_at updated in DB
- Ceremony state deleted from Redis

### Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | Successful login | Valid ceremony state + valid credential | POST /v1/auth/login/finish | 204 No Content; JWT cookie set; session created; last_used_at updated; ceremony deleted | must |
| F1 | Ceremony cookie missing | No ceremony_id cookie | POST /v1/auth/login/finish | unauthorized | must |
| F2 | Ceremony state expired | ceremony_id refers to expired Redis key (5min TTL) | POST /v1/auth/login/finish | unauthorized | must |
| F3 | Ceremony state not found | ceremony_id refers to non-existent Redis key | POST /v1/auth/login/finish | unauthorized | must |
| F4 | Invalid WebAuthn credential | Credential verification fails | POST /v1/auth/login/finish | unauthorized | must |
| F5 | No matching credential in DB | credential_id from assertion not found | POST /v1/auth/login/finish | unauthorized | must |
| F6 | User service unavailable | Valid ceremony; user service down | POST /v1/auth/login/finish | Internal error | must |
| F7 | User service timeout | Valid ceremony; user service >5s | POST /v1/auth/login/finish | Internal error | should |
| F8 | User deactivated | Valid credential but user is_active=false | POST /v1/auth/login/finish | unauthorized (deactivation hidden, D-53) | must |
| E1 | Finish called twice with same ceremony_id | First call succeeds | POST /v1/auth/login/finish (2nd call) | Error; ceremony state already consumed | should |
| E2 | Concurrent logins same user | Same user from different devices | POST /v1/auth/login/finish x 2 | Both succeed; unlimited concurrent sessions (D-35) | should |

### Open Questions
(none)

---

## Operation: VerifyJwt

### Rules
- R1: Applied to all protected routes via route_layer — prevents 404-to-401 bleed (D-127, D-129)
- R2: Invalid JWT signature (forged/corrupted) → immediate 401, no session fallback (D-48)
- R3: Only expired JWT enters grace/session flow (D-48, D-34)
- R4: Grace period: 1 minute — within 1 min of JWT expiry, Gateway issues new JWT without session verification (D-34)
- R5: Expired JWT past grace → ValidateSession gRPC → RefreshToken gRPC (D-34, PROJECT.md)
- R6: CallerIdentity fields: caller_id (Uuid), caller_role (CallerRole), handle (String), session_id (Uuid) (D-128)
- R7: Public routes exempt: /v1/auth/register/*, /v1/auth/login/*, /health, /v1/health/* (D-47)
- R8: JWT validation: ES256, iss="madome-auth", aud="madome-gateway" (D-40, D-42)
- R9: All JWT errors return unified `unauthorized` to client (D-51)
- R10: Replaces existing extract_caller_identity middleware entirely (D-127)

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
| F4 | Expired JWT, refresh fails | JWT expired; session valid but RefreshToken fails | Request to protected route | 401 unauthorized | should |
| F5 | Wrong iss/aud claims | JWT signed correctly but iss or aud mismatch | Request to protected route | 401 unauthorized (D-42) | must |
| F6 | Auth service unavailable during refresh | JWT expired; auth service down | Request to protected route | 401 unauthorized | should |
| E1 | Valid JWT on non-existent route | Valid JWT; route does not exist | Request to /v1/nonexistent | 404 not found (not 401); route_layer prevents bleed (D-127) | must |
| E2 | Concurrent requests with expiring JWT | Multiple requests near JWT expiry | Parallel requests | Same cached JWT reissued within ~10sec window (D-37) | should |

### Open Questions
(none)
