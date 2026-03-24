# Phase: Authentication Operations - Context

**Gathered:** 2026-03-25 (split from Phase 3 Authentication)
**Status:** Blocked (depends on Phase 3A)

> **Phase split:** Original Phase 3 (Authentication, ~126 decisions) split into 3A (Core) and 3B (Operations). This phase adds operational features on top of 3A's auth foundation.

<domain>
## Phase Boundary

Recovery code authentication, step-up verification for sensitive actions, API key management for scraper, passkey management (list/rename/delete/add), admin operations (invite management, user deactivation/activation, role changes), extended gateway middleware tiers (verified, admin, scraper, recovery), and logout-all-sessions.

**Dependency:** Requires Phase 3A (auth service 4-layer architecture, gateway JWT middleware, session infrastructure, all DB tables created).

**Shared foundation from 3A:** Auth DB schema (all 5 tables), WebAuthn ceremony infrastructure, JWT issuance/validation, Redis session store, auth service ports/adapters pattern, gateway public+protected tiers.

</domain>

<decisions>
## Implementation Decisions

### Role Hierarchy & Admin Operations
- **D-02:** Invite role assignment. Invite table has `role` column determining the registered user's role. Owner invite: DB seed only. Admin invite: owner can issue. User invite: admin+ can issue
- **D-07:** Role system: owner/admin/user 3-tier hierarchy. Hierarchy enforcement: can only manage roles strictly below own. Same-level interference blocked. Self-modification blocked. Minimum 1 active owner guaranteed
- **D-08:** Admin operations: invite issuance, invite cancellation, user deactivation/reactivation, role change, API key management. User deletion is v2
- **D-09:** Invite history recorded in DB, queryable by admin. Invites can be cancelled (revoked) before use

### Passkey Management
- **D-19:** Passkey management: list (GET, no individual GET), rename (PATCH), delete (DELETE, verified tier)
- **D-20:** Additional passkey registration for logged-in users: POST /v1/auth/passkeys/register/begin + /finish
- **D-15:** Last passkey cannot be deleted (minimum 1 always) -- enforcement in delete handler

### Recovery
- **D-22:** Recovery code regeneration: verified tier (POST /v1/auth/recovery-codes/regenerate). All existing codes invalidated on regeneration
- **D-23:** Recovery flow: single public endpoint `POST /v1/auth/recover` (name + recovery_code in body). No challenge-response -- single step. On success: invalidates all existing sessions, issues recovery JWT (recovery: true claim)
- **D-103:** Recovery codes stored as hashes in DB (same approach as API keys D-26). Hash algorithm: Claude's discretion
- **D-104:** Recovery JWT behavior: `recovery: true` claim in JWT. Gateway restricts access to passkey registration, passkey list, and logout only. All other endpoints blocked
- **D-105:** Recovery passkey registration success -> normal JWT reissued (recovery claim removed). User enters standard authenticated state

### Step-Up Authentication (Verify)
- **D-106:** Sensitive actions require recent passkey re-authentication (step-up auth)
- **D-107:** Verify flow: `POST /v1/auth/verify/begin` + `POST /v1/auth/verify/finish` (WebAuthn ceremony). Success -> JWT reissued with `verified_at` timestamp claim
- **D-108:** Verify window: 5 minutes from verified_at. Gateway checks claim freshness
- **D-109:** Verified-required actions (scope): passkey deletion, recovery code regeneration, logout all sessions. List is extensible for future phases
- **D-110:** Step-up auth custom messaging not supported by WebAuthn spec. Frontend responsibility (deferred)

### API Key Management
- **D-24:** Provisioning: DB storage + admin API (POST /v1/admin/api-keys)
- **D-25:** Key format: `madome_sk_` prefix + cryptographically secure random (CSPRNG)
- **D-26:** Key storage: SHA-256 hash in DB. Raw key shown only once at creation. Last 4 characters stored separately for identification
- **D-27:** Transport: Authorization: Bearer header. Gateway distinguishes JWT vs API key by prefix
- **D-28:** Expiry: optional expiration date settable at creation
- **D-29:** Scope: scraper-dedicated routes only. Gateway enforces route-level access control for API key auth
- **D-30:** Rotation: immediate replacement (new key creation instantly invalidates old key)
- **D-31:** Verification: delegated to Auth service via gRPC (Gateway -> Auth)

### Session Extended
- **D-36:** Logout: current session (POST /v1/auth/logout, protected) and all sessions (POST /v1/auth/logout/all, verified)
- **D-38:** User deactivation: immediately invalidates all sessions. Next JWT expiry triggers Gateway session check -> reject

### Gateway Extended
- **D-46:** *(3B SCOPE)* Extends gateway from 2-tier (3A) to 5-tier: public, protected, verified (JWT + recent verify), admin (JWT + admin+ role), scraper (API key). Recovery JWT is a special case of protected -- Gateway checks recovery claim and restricts to allowed endpoints
- **D-47:** *(3B SCOPE)* Public routes extended: adds /v1/auth/recover to public route list

### Auth REST Endpoints (3B additions)

**Auth endpoints:**

| Tier | Method | Path | Description |
|------|--------|------|-------------|
| public | POST | /v1/auth/recover | Recovery code auth -> recovery JWT in cookie |
| protected | GET | /v1/auth/passkeys | List own passkeys |
| protected | PATCH | /v1/auth/passkeys/:id | Rename passkey |
| protected | POST | /v1/auth/passkeys/register/begin | Additional passkey registration start |
| protected | POST | /v1/auth/passkeys/register/finish | Additional passkey registration complete |
| protected | POST | /v1/auth/verify/begin | Step-up auth start |
| protected | POST | /v1/auth/verify/finish | Step-up auth complete -> JWT reissued with verified_at |
| verified | DELETE | /v1/auth/passkeys/:id | Delete passkey (requires recent verify) |
| verified | POST | /v1/auth/recovery-codes/regenerate | Regenerate recovery codes |
| verified | POST | /v1/auth/logout/all | Logout all sessions |
| recovery | GET | /v1/auth/passkeys | List passkeys (allowed in recovery mode) |
| recovery | POST | /v1/auth/passkeys/register/begin | Register passkey (recovery purpose) |
| recovery | POST | /v1/auth/passkeys/register/finish | Complete passkey registration -> normal JWT |
| recovery | POST | /v1/auth/logout | Logout recovery session |

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

- **D-113:** Admin endpoint routing split: `/v1/admin/users/*` -> User service, `/v1/admin/invites/*` and `/v1/admin/api-keys/*` -> Auth service
- **D-114:** Admin user info response (list/detail): id, handle, name, role, is_active, created_at (basic fields only)

### Cross-Service Orchestration
- **D-119:** Gateway orchestration for cross-service operations: sequential calls with compensating transactions. Level 1 now (compensate failure -> log + error return). Level 2 later (outbox + background worker retry)
- **D-120:** Deactivate orchestration: Gateway -> User (DeactivateUser) -> Auth (InvalidateAllSessions). Auth failure -> compensate by calling User (ActivateUser). Compensate failure -> error log + error response to client

### Proto Definition (3B additions)
- **D-59:** *(3B SCOPE)* Additional RPCs: Recover, VerifyBegin/Finish, VerifyApiKey, InvalidateAllSessions, ListPasskeys, RenamePasskey, DeletePasskey, RegenerateRecoveryCodes, ListInvites, CancelInvite, CreateApiKey, ListApiKeys, RevokeApiKey

### Internal Architecture (3B additions)
- **D-71:** *(3B SCOPE)* Additional usecase flows: recovery/, verify/, api_key/, passkey_mgmt/, invite_mgmt/
- **D-74:** *(3B SCOPE)* Additional domain type: ApiKey in domain/types/
- **D-78:** *(3B SCOPE)* Additional port: ApiKeyRepository

### Claude's Discretion
- Recovery code count/status endpoint design (if needed)
- Retry count for compensating transactions
- API key CSPRNG implementation details
- Ceremony type for verify flow Redis key (`ceremony:verify:{id}`)
- Additional passkey registration flow details (same WebAuthn ceremony, different context)

</decisions>

<canonical_refs>
## Canonical References

### Phase 3A (Foundation)
- `.planning/phases/03a-authentication-core/03a-CONTEXT.md` -- Auth service architecture, JWT config, session lifecycle, gateway middleware base, testing infrastructure, established patterns

### Architecture
- `.planning/PROJECT.md` -- Auth Design, Service Topology, ID Design, Internal Service Architecture

### Requirements
- `.planning/REQUIREMENTS.md` -- AUTH-04, AUTH-05 (recovery, API key auth)
- `.planning/REQUIREMENTS.md` -- GATE-05 (gateway API key verification)

### Research
- `.planning/phases/03a-authentication-core/03a-RESEARCH.md` -- Technology stack (webauthn-rs, jsonwebtoken, sea-orm)

</canonical_refs>

<code_context>
## Existing Code Insights

Phase 3B builds directly on Phase 3A's implementation. The auth service 4-layer architecture, gateway middleware, and all adapters already exist from 3A. 3B extends them with:

### Extension Points (created in 3A)
- Auth service usecase/ directory: add new flow modules (recovery/, verify/, api_key/, passkey_mgmt/, invite_mgmt/)
- Auth service domain/ports/: add ApiKeyRepository trait
- Auth service adapter/postgres/: add ApiKeyRepository impl
- Auth service app/handler/: add RPC handlers for new operations
- Gateway routes/: add admin/, recovery auth route modules
- Gateway middleware: extend from 2-tier to 5-tier
- Proto auth.proto: add new RPCs and message types

### Testing (from 3A)
- madome-test-utils crate already exists with container setup, factory functions, test config
- testcontainers infrastructure (PostgreSQL + Redis) already running
- Service test and E2E test crate (madome-tests) already set up

</code_context>

<specifics>
## Specific Ideas

- Recovery codes follow GitHub/Discord pattern -- user provides name + code, single step (no challenge-response)
- Compensating transactions chosen over eventual consistency -- security correctness over implementation simplicity
- Step-up auth (verify) reuses WebAuthn ceremony infrastructure from registration/login
- API key format follows GitHub/Stripe convention (`madome_sk_` prefix) for easy identification
- Gateway distinguishes auth types by token prefix -- no ambiguity between JWT and API key

</specifics>

<deferred>
## Deferred Ideas

- Rate limiting on auth endpoints -- separate phase or infrastructure concern
- JWT key rotation with kid claim
- User deletion (admin operation) -- v2
- Audit logging (beyond invite history) -- v2+
- Compensating transaction Level 2 (outbox + background worker)
- Recovery code count/status endpoint -- decide in future phase
- Step-up auth UI messaging -- frontend responsibility

</deferred>

---

*Phase: authentication-operations (3B)*
*Split from: Phase 3 Authentication*
*Context gathered: 2026-03-25*
