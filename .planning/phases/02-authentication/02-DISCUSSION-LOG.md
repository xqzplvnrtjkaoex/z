# Phase 2: Authentication - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md -- this log preserves the alternatives considered.

**Date:** 2026-03-21
**Phase:** 02-authentication
**Areas discussed:** User onboarding, API Key management, Session lifecycle, Auth error detail, DB schema, Gateway middleware, Docker/infra, Testing, JWT details, API endpoints, Proto definitions, Cookie settings, JWT key rotation

---

## User Onboarding

| Option | Description | Selected |
|--------|-------------|----------|
| Invite-only | Admin issues invite token, token-based registration | Y |
| Open registration | Anyone can register directly | |
| Admin pre-creates | Admin creates account, user adds passkey | |

**User's choice:** Invite-only
**Notes:** Small community, prevent spam/abuse

### Inviter

| Option | Description | Selected |
|--------|-------------|----------|
| Admin only | Only admin role can issue invites, initial admin via DB seed | Y |
| All users | Any logged-in user can invite | |

### Token Policy

| Option | Description | Selected |
|--------|-------------|----------|
| Single-use, 24h expiry | | |
| Single-use, 30min expiry | | Y |
| Single-use, no expiry | | |

**User's choice:** Single-use, 30-minute expiry (custom input)

### Profile Info

| Option | Description | Selected |
|--------|-------------|----------|
| Name set at registration | User sets name during passkey registration | Y |
| Name set at invitation | Inviter pre-sets the name | |
| No name | UUID only | |

**Notes:** Changed from "nickname" to "username" to "name" during discussion

### Role System

| Option | Description | Selected |
|--------|-------------|----------|
| admin/user two-tier | Two roles with different permissions | Y |
| Single role + flag | Boolean is_admin flag | |

### Passkey Count

| Option | Description | Selected |
|--------|-------------|----------|
| Multiple allowed | Multi-device support, WebAuthn standard | Y |
| Single only | One passkey per user | |

### Name Constraints

| Option | Description | Selected |
|--------|-------------|----------|
| Basic + unique | Length limit + uniqueness required | Y |
| Basic only | Length limit, no uniqueness | |

### Recovery

| Option | Description | Selected |
|--------|-------------|----------|
| Recovery codes | 10 codes, 8-char alphanumeric, each single-use | Y |
| Admin reset | Admin issues new invite token | |
| Out of scope | Defer to future phase | |

### Recovery Code Reissue

| Option | Description | Selected |
|--------|-------------|----------|
| Reissue while logged in | New codes generated, old codes invalidated | Y |
| No reissue | Only initial codes | |

### Invite History

| Option | Description | Selected |
|--------|-------------|----------|
| Recorded in DB | Who invited whom, admin queryable | Y |
| No record | Token deleted on use | |

### Initial Admin

| Option | Description | Selected |
|--------|-------------|----------|
| DB seed script | sea-orm migration creates initial admin | Y |
| Env var bootstrap | Auto-create from env vars on first run | |

### Admin Operations

| Option | Description | Selected |
|--------|-------------|----------|
| Invite + deactivate | Token issuance + user deactivation. Delete in v2 | Y |
| Invite only | Only token issuance | |

### Token Delivery

| Option | Description | Selected |
|--------|-------------|----------|
| Token string direct | API returns token string, admin shares via messenger | Y |
| Registration link | API generates full URL | |

### Profile Requirements

| Option | Description | Selected |
|--------|-------------|----------|
| Name only | API-only service, no email needed | Y |
| Name + email | Email for recovery/notifications | |

---

## Passkey Configuration

### Attestation

| Option | Description | Selected |
|--------|-------------|----------|
| None | No hardware attestation required | Y |
| Direct | Require and verify attestation | |

### Authenticator Attachment

| Option | Description | Selected |
|--------|-------------|----------|
| No restriction | Platform + roaming both allowed | Y |
| Platform only | Touch ID, Windows Hello only | |
| Cross-platform only | YubiKey only | |

### User Verification

| Option | Description | Selected |
|--------|-------------|----------|
| Required | Always biometric/PIN | Y |
| Preferred | If available | |
| Discouraged | Not required | |

### Discoverable Credential

| Option | Description | Selected |
|--------|-------------|----------|
| Required | Username-less login, all modern authenticators support | Y |
| Preferred | Fallback for legacy authenticators | |

### Registration Flow

| Option | Description | Selected |
|--------|-------------|----------|
| 2-step (begin + finish) | Standard WebAuthn, token verified in begin | Y |
| 3-step (verify + begin + finish) | Separate token verification step | |

### Authentication Flow

| Option | Description | Selected |
|--------|-------------|----------|
| Username-less | Discoverable credential, no identifier input | Y |
| Username required | Identifier input before ceremony | |

### RP Configuration

| Option | Description | Selected |
|--------|-------------|----------|
| Environment variables | RP_ID, RP_ORIGIN env vars | Y |
| Config file | Settings file | |

### Passkey Management

| Option | Description | Selected |
|--------|-------------|----------|
| List + rename + delete | Full CRUD minus create (separate flow) | Y |
| List + delete only | No rename | |

---

## API Key Management

### Provisioning

| Option | Description | Selected |
|--------|-------------|----------|
| DB + admin API | Full CRUD via admin endpoints | Y |
| Environment variable | Fixed key in env var | |
| DB seed only | No management API | |

### Format

| Option | Description | Selected |
|--------|-------------|----------|
| Prefix + CSPRNG | madome_sk_ prefix + cryptographically secure random | Y |
| UUIDv4 | Plain UUID | |

**Notes:** User emphasized cryptographically secure, not simple random

### Transport

| Option | Description | Selected |
|--------|-------------|----------|
| Authorization: Bearer | Standard header, prefix distinguishes from JWT | Y |
| X-API-Key header | Custom header | |

### Expiry

| Option | Description | Selected |
|--------|-------------|----------|
| Optional expiration date | Settable at creation | Y |
| No expiry | Valid until manual revocation | |

### Storage

| Option | Description | Selected |
|--------|-------------|----------|
| SHA-256 hash | Raw shown once, hash stored, last 4 chars for identification | Y |
| Encrypted | AES-256 reversible encryption | |
| Plaintext | Direct storage | |

### Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Scraper routes only | Gateway enforces route-level access control | Y |
| Full access | Same as JWT scope | |

### Rotation

| Option | Description | Selected |
|--------|-------------|----------|
| Immediate replacement | New key invalidates old key instantly | Y |
| Grace period | Old key valid for configurable period | |

---

## Session Lifecycle

### Max Lifetime

| Option | Description | Selected |
|--------|-------------|----------|
| Sliding 7d + absolute 30d | JWT refresh extends sliding; 30d hard limit from creation | Y |
| Sliding 7d only | No absolute limit | |
| Fixed 7d | No extension | |

**Notes:** User asked about UX impact of absolute expiry. Discussed sliding + absolute balance.

### Grace Period

| Option | Description | Selected |
|--------|-------------|----------|
| 1 minute | Short grace, more frequent auth service calls | Y |
| 5 minutes | Moderate grace | |
| 15 minutes | Long grace, effectively doubles JWT TTL | |

### Concurrent Sessions

| Option | Description | Selected |
|--------|-------------|----------|
| Unlimited | Multiple devices, no restriction | Y |
| Max 5 | Auto-evict oldest | |

### Logout

| Option | Description | Selected |
|--------|-------------|----------|
| Both current and all | /logout + /logout/all | Y |
| Current only | Single session logout | |
| All only | Bulk logout | |
| Not needed | Rely on session expiry | |

### JWT Caching

| Option | Description | Selected |
|--------|-------------|----------|
| In-memory + TTL | Auth service memory, ~10s TTL, same JWT for concurrent requests | Y |
| DB record | Session table column | |

### Session Storage

| Option | Description | Selected |
|--------|-------------|----------|
| Redis | TTL auto-expiry, fast lookup | Y |
| PostgreSQL | Same stack, sea-orm managed | |

### Deactivation

| Option | Description | Selected |
|--------|-------------|----------|
| Immediate full invalidation | All sessions deleted, next JWT expiry -> reject | Y |
| Session kept + JWT blacklist | Immediate block but complex | |

---

## Auth Error Policy

### Error Detail Level

| Option | Description | Selected |
|--------|-------------|----------|
| Generic only | Single 'unauthorized' for all auth failures | Y |
| Stage-specific | Different codes per failure type | |
| Fully detailed | Specific failure reasons exposed | |

### JWT Errors

| Option | Description | Selected |
|--------|-------------|----------|
| Unified unauthorized | All JWT issues = unauthorized | Y |
| Distinct codes | token_expired vs token_invalid vs token_missing | |

### Invite Errors

| Option | Description | Selected |
|--------|-------------|----------|
| Unified invite_invalid | All token issues = invite_invalid | Y |
| Distinct codes | invite_expired vs invite_used vs invite_invalid | |

### Rate Limiting

| Option | Description | Selected |
|--------|-------------|----------|
| Out of Phase 2 scope | Deferred to later phase | Y |
| IP-based rate limit | Per-IP request limiting on auth endpoints | |

### Log vs Response

| Option | Description | Selected |
|--------|-------------|----------|
| Detailed logs + generic responses | Server logs: specific; client: generic | Y |
| Both generic | Minimal logging | |

### Deactivated User Error

| Option | Description | Selected |
|--------|-------------|----------|
| Generic unauthorized | Deactivation not disclosed | Y |
| account_deactivated | Explicit deactivation message | |

### WebAuthn Error

| Option | Description | Selected |
|--------|-------------|----------|
| Generic unauthorized | No ceremony stage disclosure | Y |
| Stage-specific | challenge_timeout, verification_failed, etc. | |

---

## DB Schema

### Table Structure

| Option | Description | Selected |
|--------|-------------|----------|
| 6 tables separated | users, credentials, sessions, invitations, api_keys, recovery_codes | Y |
| 4 tables consolidated | Invitations and recovery embedded | |

### Users Columns

**User's choice:** id (UUIDv4) + name (unique) + role (admin/user) + is_active + timestamps

**Notes:** Changed from "nickname" -> "username" -> "name" across discussion

### Indexes

| Option | Description | Selected |
|--------|-------------|----------|
| Matching indexes | Per query pattern: unique, FK, hash lookups | Y |

---

## Gateway Middleware

### Route Tiers

| Option | Description | Selected |
|--------|-------------|----------|
| 4-tier | public, protected, admin, scraper | Y |
| 2-tier + override | public/protected with per-route override | |

### Public Routes

| Option | Description | Selected |
|--------|-------------|----------|
| Register/login/recover/health only | Minimal public surface | Y |

### JWT Invalid Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Immediate 401 | Forged/corrupted JWT -> 401, no fallback. Only expired enters flow | Y |

### User Context Propagation

| Option | Description | Selected |
|--------|-------------|----------|
| gRPC metadata | user_id + role in metadata, same as request_id pattern | Y |
| JWT forwarding | Forward raw JWT, backends parse | |

---

## Docker/Infrastructure

### Docker Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| docker-compose (DB + Redis only) | Services run locally via cargo run | Y |
| Full containerization | All services + infra in docker-compose | |

### DB Init

| Option | Description | Selected |
|--------|-------------|----------|
| Auto migration on startup | sea-orm migration + admin seed at service start | Y |
| Manual migration command | Separate migration step | |

### Dev Script

| Option | Description | Selected |
|--------|-------------|----------|
| Extend existing | Add docker-compose up -d to dev-start.sh | Y |
| Separate script | Infra-only script | |

---

## Testing

### Testcontainers Scope

| Option | Description | Selected |
|--------|-------------|----------|
| All tests (PostgreSQL + Redis) | No mock DB anywhere. Container reuse for speed | Y |
| Integration=mock, E2E=testcontainers | Split approach | |

### Test Layers

| Option | Description | Selected |
|--------|-------------|----------|
| Contract + Integration | Per-service gRPC contract + Gateway E2E flow | Y |
| Integration only | Gateway E2E only | |

### WebAuthn Testing

| Option | Description | Selected |
|--------|-------------|----------|
| webauthn-rs SoftPasskey | Server-side ceremony simulation, no browser needed | Y |
| API level only | JSON payload validation only | |

---

## JWT Details

### Claims

| Option | Description | Selected |
|--------|-------------|----------|
| Extended claims | sub, role, sid, name, iss, aud, iat, exp | Y |
| Core claims only | sub, role, sid, iat, exp | |

### Algorithm

| Option | Description | Selected |
|--------|-------------|----------|
| ES256 | ECDSA P-256, aws_lc backend | Y |
| HS256 | HMAC-SHA256 symmetric | |
| RS256 | RSA-SHA256 | |

### Key Management

| Option | Description | Selected |
|--------|-------------|----------|
| Environment variables | JWT_PRIVATE_KEY, JWT_PUBLIC_KEY (PEM) | Y |
| File path | Env var points to key file | |

### iss/aud Validation

| Option | Description | Selected |
|--------|-------------|----------|
| Enabled | iss: madome-auth, aud: madome-gateway | Y |
| Disabled | No iss/aud claims | |

### Cookie Settings

| Option | Description | Selected |
|--------|-------------|----------|
| Secure defaults | HttpOnly, Secure, SameSite=Strict, Path=/ | Y |
| SameSite=Lax | Allow cross-site navigation | |

### Key Rotation

| Option | Description | Selected |
|--------|-------------|----------|
| Out of Phase 2 scope | Single key pair, no kid claim | Y |
| kid claim included | Future-proofing for rotation | |

---

## API Endpoints

Confirmed complete list of 23 endpoints across 4 tiers (public, protected, admin, recovery). See CONTEXT.md for full list.

---

## Proto Definition

### API Key Verification Location

| Option | Description | Selected |
|--------|-------------|----------|
| Auth service (gRPC) | Gateway delegates to Auth via gRPC | Y |
| Gateway direct | Gateway queries DB/Redis directly | |

### RPC Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Full auth operations | All auth operations as gRPC RPCs | Y |
| Core only | ValidateSession + RefreshToken + VerifyApiKey only | |

### Proto Messages

| Option | Description | Selected |
|--------|-------------|----------|
| Claude's discretion | Design based on API endpoints and requirements | Y |
| User-defined | Manually specify each message structure | |

---

## Claude's Discretion

- Proto message structures (request/response types for each RPC)
- Exact Redis key structure and TTL values
- Credential storage format details
- Recovery code generation algorithm
- Migration file organization
- Index type selection (btree vs hash)
- Error code string constants

## Deferred Ideas

- Rate limiting on auth endpoints
- JWT key rotation with kid claim
- User deletion (admin operation) -- v2
- CORS configuration -- when frontend is built
- Audit logging beyond invite history -- v2+
