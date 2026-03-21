# Phase 2: Authentication - Research

**Researched:** 2026-03-22
**Domain:** WebAuthn/Passkey + JWT + Session management + API Key auth in Rust
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**User Onboarding**
- D-01: Invite-only registration. Admin issues invite tokens; new users register Passkey using a valid token
- D-02: Admin-only invitation. Only users with admin role can issue invite tokens. Initial admin created via DB seed migration
- D-03: Invite token policy: single-use, 30-minute expiry. Token string returned via API, admin shares manually
- D-04: Registration URL pattern: `/register?token=xxx`
- D-05: User sets `name` (unique) during Passkey registration
- D-06: Profile info: name only (no email)
- D-07: Role system: admin/user two-tier
- D-08: Admin operations in Phase 2: invite token issuance + user deactivation/reactivation
- D-09: Invite history recorded in DB, queryable by admin

**Passkey Configuration**
- D-10: Attestation: None
- D-11: Authenticator Attachment: no restriction
- D-12: User Verification: Required
- D-13: Discoverable Credential (Resident Key): Required
- D-14: Multiple passkeys per user allowed
- D-15: Last passkey cannot be deleted

**Passkey Flows**
- D-16: Registration: 2-step (begin + finish). Invite token verified in begin step
- D-17: Authentication: username-less (Discoverable Credential based)
- D-18: RP configuration: RP_ID, RP_ORIGIN environment variables
- D-19: Passkey management: list, rename, delete (GET/PATCH/DELETE /v1/auth/passkeys/:id)
- D-20: Additional passkey registration for logged-in users: POST /v1/auth/passkeys/register/begin + /finish

**Recovery**
- D-21: Recovery codes issued at registration: 10 codes, 8-character alphanumeric, each single-use
- D-22: Recovery code regeneration while logged in; all existing codes invalidated
- D-23: Recovery flow: POST /v1/auth/recover/begin + /finish (no auth required)

**API Key Management**
- D-24: Provisioning: DB storage + admin API (POST /v1/admin/api-keys)
- D-25: Key format: `madome_sk_` prefix + CSPRNG
- D-26: Storage: SHA-256 hash in DB. Raw key shown once. Last 4 chars stored separately
- D-27: Transport: Authorization: Bearer header. Gateway distinguishes JWT vs API key by prefix
- D-28: Expiry: optional expiration date at creation
- D-29: Scope: scraper-dedicated routes only
- D-30: Rotation: immediate replacement (new key instantly invalidates old)
- D-31: Verification: delegated to Auth service via gRPC

**Session Lifecycle**
- D-32: Session storage: Redis (TTL auto-expiry + fast lookup)
- D-33: Session TTL: sliding 7 days + absolute 30 days
- D-34: JWT grace period: 1 minute
- D-35: Concurrent sessions: unlimited per user
- D-36: Logout: current session + all sessions
- D-37: JWT caching: in-memory in Auth service with short TTL (~10 sec)
- D-38: User deactivation: immediately invalidates all sessions

**JWT Configuration**
- D-39: Claims: sub, role, sid, name, iss ("madome-auth"), aud ("madome-gateway"), iat, exp
- D-40: Algorithm: ES256 (ECDSA P-256, aws_lc backend)
- D-41: Key management: JWT_PRIVATE_KEY, JWT_PUBLIC_KEY env vars (PEM format)
- D-42: Validation: iss + aud claim verification enabled
- D-43: TTL: 15 minutes
- D-44: Cookie: HttpOnly=true, Secure=true, SameSite=Strict, Path=/. COOKIE_SECURE=false dev override
- D-45: Key rotation: out of scope for Phase 2

**Gateway Middleware**
- D-46: 4-tier route authentication: public, protected, admin, scraper
- D-47: Public routes: /v1/auth/register/*, /v1/auth/login/*, /v1/auth/recover/*, /health, /v1/health/*
- D-48: JWT signature invalid: immediate 401. Only expired JWT enters grace/session flow
- D-49: User context propagation: user_id and role as gRPC metadata

**Auth Error Policy**
- D-50: Client-facing errors: generic `unauthorized` for all auth failures
- D-51: JWT errors: unified `unauthorized`
- D-52: Invite token errors: unified `invite_invalid`
- D-53: Deactivated user login: generic `unauthorized`
- D-54: Server logs: detailed error with request_id + user_id + specific failure reason
- D-55: Rate limiting: deferred

**DB Schema**
- D-56: 6 tables: users, credentials, sessions, invitations, api_keys, recovery_codes
- D-57: users table: id (UUIDv4 PK), name (unique), role, is_active, created_at, updated_at
- D-58: Indexes: users.name UNIQUE, credentials.user_id, sessions.user_id, api_keys.key_hash UNIQUE, invitations.token_hash UNIQUE

**Proto Definition**
- D-59: Full auth operations as gRPC RPCs: RegisterBegin/Finish, LoginBegin/Finish, RecoverBegin/Finish, ValidateSession, RefreshToken, VerifyApiKey, admin management RPCs
- D-60: Gateway is pure REST-to-gRPC translator for all auth operations

**Infrastructure**
- D-61: docker-compose for PostgreSQL + Redis only
- D-62: DB initialization: auto-migration on Auth service startup (sea-orm). Admin seed in migration
- D-63: dev-start.sh: extended with `docker-compose up -d` step

**Testing**
- D-64: All tests use testcontainers for PostgreSQL + Redis (no mock DB)
- D-65: Test layers: contract tests (per-service gRPC contract) + integration tests (Gateway E2E)
- D-66: WebAuthn testing: webauthn-rs SoftPasskey/mock authenticator

### Claude's Discretion

- Proto message structures (request/response types for each RPC)
- Exact Redis key structure and TTL values for JWT cache
- Credential storage format details (webauthn-rs internal types)
- Recovery code generation algorithm specifics
- Migration file organization and ordering
- Exact index types (btree vs hash) based on query patterns
- Error code string constants

### Deferred Ideas (OUT OF SCOPE)

- Rate limiting on auth endpoints
- JWT key rotation with kid claim
- User deletion (admin operation)
- OAuth/social login
- CORS configuration
- Audit logging (beyond invite history)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AUTH-01 | User can register a Passkey credential | webauthn-rs `start_passkey_registration` / `finish_passkey_registration` flow; invite token verification; DB `credentials` + `users` tables |
| AUTH-02 | User can authenticate via Passkey | webauthn-rs `start_passkey_authentication` (username-less discoverable credential) / `finish_passkey_authentication`; session creation |
| AUTH-03 | Auth service issues JWT access token (15 min TTL, HttpOnly Cookie) | jsonwebtoken `encode` with ES256/aws_lc; axum-extra `CookieJar` for HttpOnly cookie; `madome-auth` Redis session creation |
| AUTH-04 | Auth service manages sessions (create, validate, invalidate) | Redis `ConnectionManager` with set_ex/get/del; per-session sliding TTL; absolute expiry enforcement |
| AUTH-05 | Auth service caches recently issued JWT per session (duplicate prevention) | moka `Cache` with `~10s` TTL; `get_with` pattern for atomic get-or-insert |
| GATE-03 | Gateway verifies JWT access token (stateless pass-through when valid) | `FromRequestParts` extractor on gateway; `jsonwebtoken::decode` with aud/iss validation; no Auth service call on valid JWT |
| GATE-04 | Gateway refreshes JWT on expiry (grace period + session fallback) | 1-min grace period check in extractor; gRPC `ValidateSession`/`RefreshToken` for past-grace case; new cookie set in response |
| GATE-05 | Gateway authenticates Scraper via API key | Bearer prefix detection (`madome_sk_`); gRPC `VerifyApiKey` to Auth service; route-level scope enforcement |
</phase_requirements>

---

## Summary

Phase 2 implements a complete authentication system on top of the Phase 1 gateway/service skeleton. The core challenge is a three-party system: the Gateway handles JWT lifecycle statelessly (verifying, grace-period refreshing, or delegating session verification); the Auth service owns Passkey ceremonies, session state, JWT issuance with deduplication, and API key verification; Redis holds session state with sliding/absolute TTL.

The WebAuthn flow (webauthn-rs 0.5.4) uses discoverable credentials (resident keys) for username-less login. The `Webauthn` builder requires `RP_ID` (domain string) and `RP_ORIGIN` (full URL). Both registration and authentication are two-phase request/response ceremonies: `begin` returns a challenge serialized to JSON and a server-side state object (stored in a short-lived Redis key); `finish` receives the authenticator response, verifies it against the stored state, and returns a `Passkey` credential (or `AuthenticationResult`).

JWT handling uses jsonwebtoken 10.3.0 with the `aws_lc` feature (disable default ring backend). ES256 keys load from PEM via `EncodingKey::from_ec_pem` / `DecodingKey::from_ec_pem`. Claims include all fields from D-39. The Gateway's 4-tier route middleware distinguishes public/protected/admin/scraper routes; the core JWT extractor implements `FromRequestParts` and covers the three-case flow (valid / grace / past-grace). API key requests are identified by `madome_sk_` prefix and dispatched to Auth via gRPC for verification.

**Primary recommendation:** Implement Auth service first (proto RPCs → DB schema → webauthn-rs ceremonies → session/JWT logic → Redis caching), then add Gateway middleware layers (JWT extractor → 4-tier route guards → API key path).

---

## Standard Stack

### Core Auth Dependencies (new for Phase 2)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| webauthn-rs | 0.5.4 | WebAuthn/Passkey server ceremonies | Only mature Rust WebAuthn implementation |
| webauthn-rs-proto | 0.5.4 | WebAuthn protocol types for REST API | Required companion to webauthn-rs for serialization |
| jsonwebtoken | 10.3.0 | JWT create/verify with ES256 | Most downloaded Rust JWT crate; explicit aws_lc support |
| redis | 1.0.5 | Session storage (async ConnectionManager) | Official Redis client; built-in reconnection via ConnectionManager |
| sea-orm | 1.1.19 | ORM for PostgreSQL (users, credentials, sessions tables) | Already in STACK.md as standard; stable line |
| sea-orm-migration | 1.1.19 | DB migrations + admin seed | Must match sea-orm version |
| moka | 0.12.14 | In-memory JWT deduplication cache | Lock-free async TTL cache; no Redis hop for 10s window |
| axum-extra | 0.12.5 | CookieJar extractor for HttpOnly cookies | Official axum companion; provides CookieJar in/out pattern |
| sha2 | 0.10.9 | SHA-256 hashing for API key + invite token storage | Stable 0.10.x line (0.11 still pre-release) |
| rand | 0.10.0 | CSPRNG for API keys, recovery codes, invite tokens | Current stable |
| base64 | 0.22.1 | URL-safe base64 for WebAuthn challenge encoding | Already in workspace; URL_SAFE_NO_PAD engine |
| uuid | 1.22.0 | UUIDv4 for User/Session/APIKey IDs | Already in workspace; v4 feature required here |
| chrono | 0.4.44 | Timestamps, TTL calculations | Already in workspace |

### Supporting (test-only)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| testcontainers | 0.27.1 | Container lifecycle management in tests | All Auth service integration tests |
| testcontainers-modules | 0.15.0 | Pre-built Postgres + Redis containers | DB and Redis tests; use `postgres` and `redis` features |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| moka (in-memory) | Redis for JWT cache | Redis adds network hop; in-memory is faster for 10s TTL window. Use Redis only if gateway scales to multiple instances |
| redis::ConnectionManager | deadpool-redis / bb8-redis | ConnectionManager is simpler; no separate pool crate needed for single-instance use |
| sha2 | argon2/bcrypt | API keys are long random strings (no need for password-hashing KDF); SHA-256 is sufficient and fast |

**Installation additions to workspace Cargo.toml:**
```toml
# Auth
webauthn-rs = { version = "0.5", features = ["resident-key-support"] }
webauthn-rs-proto = "0.5"
jsonwebtoken = { version = "10", default-features = false, features = ["aws_lc"] }
redis = { version = "1", features = ["tokio-comp"] }
sha2 = "0.10"
rand = "0.10"
base64 = "0.22"
chrono = { version = "0.4", features = ["serde"] }

# Caching
moka = { version = "0.12", features = ["future"] }

# Cookie handling
axum-extra = { version = "0.12", features = ["cookie"] }

# Database
sea-orm = { version = "1.1", features = ["sqlx-postgres", "runtime-tokio-native-tls", "macros", "with-chrono", "with-uuid"] }
sea-orm-migration = "1.1"

# Testing (dev)
testcontainers = "0.27"
testcontainers-modules = { version = "0.15", features = ["postgres", "redis"] }
```

**Version verification (2026-03-22 via crates.io API):**
- webauthn-rs: 0.5.4 (stable, 2025-12-10) — 0.6.0-dev published 2026-03-20, avoid
- jsonwebtoken: 10.3.0 (2026-01-27)
- redis: 1.0.5 (latest stable)
- moka: 0.12.14 (2026-03-02)
- axum-extra: 0.12.5
- sha2: 0.10.9 (stable; 0.11.x pre-release only)
- sea-orm: 1.1.19 stable (2.0.0-rc.37 still RC)
- testcontainers: 0.27.1
- testcontainers-modules: 0.15.0

---

## Architecture Patterns

### Recommended Auth Service Structure

```
services/auth/
  Cargo.toml
  migration/
    src/
      lib.rs           # Migrator registration
      m20260322_000001_create_users.rs
      m20260322_000002_create_credentials.rs
      m20260322_000003_create_sessions.rs
      m20260322_000004_create_invitations.rs
      m20260322_000005_create_api_keys.rs
      m20260322_000006_create_recovery_codes.rs
      m20260322_000007_seed_admin.rs
  schema/
    src/
      lib.rs
      users.rs          # sea-orm entity
      credentials.rs
      sessions.rs
      invitations.rs
      api_keys.rs
      recovery_codes.rs
  src/
    main.rs             # startup: connect DB, run migrations, start gRPC server
    lib.rs              # re-exports for testability
    service.rs          # gRPC trait implementation (AuthService)
    webauthn.rs         # webauthn-rs builder + begin/finish wrappers
    session.rs          # Redis session CRUD (create, get, invalidate, invalidate_all)
    jwt.rs              # JWT encode/decode + moka cache
    invite.rs           # Invite token generation, validation, history
    api_key.rs          # API key generation, hashing, verification
    recovery.rs         # Recovery code generation, validation, regeneration
    repository/
      mod.rs
      user.rs
      credential.rs
      session.rs        # DB session record (separate from Redis session)
      invitation.rs
      api_key.rs
      recovery_code.rs
```

### Gateway Middleware Structure (additions to Phase 1)

```
services/gateway/src/
  middleware/
    mod.rs
    auth.rs             # JwtClaims extractor (FromRequestParts) + ApiKeyAuth guard
  routes/
    mod.rs              # Updated to add auth routes, route-tier guards
    auth.rs             # Auth endpoints (register/login/logout/passkeys/recover/admin)
    health.rs           # Existing
  state.rs              # Add: jwt_public_key (DecodingKey), redis connection (for future scale)
```

### Pattern 1: webauthn-rs Builder Initialization

**What:** Build a `Webauthn` instance once at startup, store in `Arc<Webauthn>` in AppState.
**When to use:** Auth service startup; also needed in test fixtures.

```rust
// services/auth/src/webauthn.rs
// Source: https://docs.rs/webauthn-rs/0.5.4/webauthn_rs/struct.WebauthnBuilder.html
use url::Url;
use webauthn_rs::prelude::*;

pub fn build_webauthn(rp_id: &str, rp_origin: &str) -> WebauthnResult<Webauthn> {
    let origin = Url::parse(rp_origin)?;
    WebauthnBuilder::new(rp_id, &origin)?
        .rp_name("Madome")
        .build()
}
```

### Pattern 2: Passkey Registration (2-phase)

**What:** Begin stores `PasskeyRegistration` state in Redis (short TTL, e.g. 5 min). Finish reads and deletes state.
**When to use:** POST /v1/auth/register/begin and /finish.

```rust
// services/auth/src/webauthn.rs
// Source: https://docs.rs/webauthn-rs/0.5.4/webauthn_rs/struct.Webauthn.html

// Begin: called with invite token already validated
pub async fn registration_begin(
    webauthn: &Webauthn,
    user_id: Uuid,
    user_name: &str,
    exclude_creds: Option<Vec<CredentialID>>,
) -> WebauthnResult<(CreationChallengeResponse, PasskeyRegistration)> {
    webauthn.start_passkey_registration(
        user_id,
        user_name,
        user_name,       // display_name same as name per D-05/D-06
        exclude_creds,
    )
}

// Finish: state retrieved from Redis
pub async fn registration_finish(
    webauthn: &Webauthn,
    credential: &RegisterPublicKeyCredential,
    state: &PasskeyRegistration,
) -> WebauthnResult<Passkey> {
    webauthn.finish_passkey_registration(credential, state)
}
```

**Challenge state storage in Redis:**
```rust
// Key pattern: "reg_challenge:{session_uuid}" with 5-minute TTL
// Value: serde_json::to_string(&passkey_registration_state)
// After finish_passkey_registration: immediately delete the key
redis_conn.set_ex(
    format!("reg_challenge:{challenge_id}"),
    serde_json::to_string(&state)?,
    300, // 5 min
).await?;
```

### Pattern 3: Username-less Authentication (Discoverable Credential)

**What:** `start_passkey_authentication` takes empty slice for discoverable flow. No username needed in begin request body (D-17).
**When to use:** POST /v1/auth/login/begin — no user identifier in request body.

```rust
// Source: https://docs.rs/webauthn-rs/0.5.4/webauthn_rs/struct.Webauthn.html
// Begin: pass empty slice for username-less discoverable credential flow
let (rcr, auth_state) = webauthn.start_passkey_authentication(&[])?;
// rcr serialized to JSON -> REST response
// auth_state stored in Redis with short TTL (5 min)

// Finish: authenticator response + stored state
let auth_result = webauthn.finish_passkey_authentication(
    &credential,  // PublicKeyCredential from client
    &auth_state,  // retrieved from Redis
)?;
// auth_result.cred_id() identifies which credential was used -> look up user
```

### Pattern 4: JWT Encode/Decode with ES256

**What:** Auth service encodes JWTs; Gateway decodes them. Private key on Auth; public key on Gateway.

```rust
// services/auth/src/jwt.rs
// Source: https://docs.rs/jsonwebtoken/10.3.0/jsonwebtoken/
use jsonwebtoken::{encode, decode, Header, Algorithm, EncodingKey, DecodingKey, Validation};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtClaims {
    pub sub: String,    // user_id (UUIDv4)
    pub role: String,   // "admin" or "user"
    pub sid: String,    // session_id (UUIDv4)
    pub name: String,   // username
    pub iss: String,    // "madome-auth"
    pub aud: Vec<String>, // ["madome-gateway"]
    pub iat: i64,
    pub exp: i64,
}

// Encoding (Auth service only)
let header = Header::new(Algorithm::ES256);
let private_key = EncodingKey::from_ec_pem(pem_bytes)?;
let token = encode(&header, &claims, &private_key)?;

// Decoding (Gateway)
let mut validation = Validation::new(Algorithm::ES256);
validation.set_issuer(&["madome-auth"]);
validation.set_audience(&["madome-gateway"]);
let public_key = DecodingKey::from_ec_pem(pem_bytes)?;
let token_data = decode::<JwtClaims>(&token, &public_key, &validation)?;
```

### Pattern 5: JWT Grace Period Detection

**What:** Distinguish "expired within grace" vs "expired past grace" by inspecting the `exp` claim after decode failure.

```rust
// services/gateway/src/middleware/auth.rs
use jsonwebtoken::{decode, errors::ErrorKind, Validation, DecodingKey};

pub enum JwtVerifyResult {
    Valid(JwtClaims),
    ExpiredInGrace(JwtClaims),   // exp < now but exp > now - 60s
    ExpiredPastGrace,
    Invalid,
}

pub fn verify_jwt(token: &str, key: &DecodingKey) -> JwtVerifyResult {
    // First: try standard decode (exp validation on)
    let mut validation = make_validation();
    match decode::<JwtClaims>(token, key, &validation) {
        Ok(data) => return JwtVerifyResult::Valid(data.claims),
        Err(e) if matches!(e.kind(), ErrorKind::ExpiredSignature) => {}
        Err(_) => return JwtVerifyResult::Invalid,
    }
    // JWT is expired. Decode without exp validation to read claims.
    let mut lax = make_validation();
    lax.validate_exp = false;
    match decode::<JwtClaims>(token, key, &lax) {
        Ok(data) => {
            let now = chrono::Utc::now().timestamp();
            let grace_cutoff = data.claims.exp + 60; // 1-minute grace (D-34)
            if now <= grace_cutoff {
                JwtVerifyResult::ExpiredInGrace(data.claims)
            } else {
                JwtVerifyResult::ExpiredPastGrace
            }
        }
        Err(_) => JwtVerifyResult::Invalid, // signature invalid
    }
}
```

### Pattern 6: Per-Session JWT Cache (Moka)

**What:** `Arc<Cache<String, String>>` in Auth service AppState. Key = session_id; value = JWT string. TTL ~10 seconds.

```rust
// services/auth/src/jwt.rs
// Source: https://docs.rs/moka/0.12.14/moka/future/struct.Cache.html
use moka::future::Cache;
use std::time::Duration;

pub fn build_jwt_cache() -> Cache<String, String> {
    Cache::builder()
        .time_to_live(Duration::from_secs(10))
        .build()
}

// In RefreshToken / LoginFinish RPC handler:
pub async fn get_or_issue_jwt(
    cache: &Cache<String, String>,
    session_id: &str,
    claims: &JwtClaims,
    key: &EncodingKey,
) -> Result<String, JwtError> {
    // get_with is atomic: if key absent, calls closure to generate and insert
    let token = cache.get_with(session_id.to_string(), async {
        encode(&Header::new(Algorithm::ES256), claims, key)
            .expect("JWT encoding should not fail")
    }).await;
    Ok(token)
}
```

### Pattern 7: Redis Session Management

**What:** Sessions stored in Redis with sliding + absolute TTL. Key = `session:{session_id}`.

```rust
// services/auth/src/session.rs
// Source: https://docs.rs/redis/1.0.5/redis/aio/struct.ConnectionManager.html
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: String,
    pub role: String,
    pub name: String,
    pub created_at: i64,         // absolute TTL anchor
    pub last_refreshed_at: i64,
}

const SLIDING_TTL_SECS: u64 = 7 * 24 * 3600;     // 7 days
const ABSOLUTE_TTL_SECS: i64 = 30 * 24 * 3600;   // 30 days

pub async fn create_session(
    redis: &mut ConnectionManager,
    session_id: &str,
    data: &SessionData,
) -> redis::RedisResult<()> {
    let key = format!("session:{session_id}");
    let value = serde_json::to_string(data).unwrap();
    redis.set_ex(key, value, SLIDING_TTL_SECS).await
}

pub async fn validate_and_slide_session(
    redis: &mut ConnectionManager,
    session_id: &str,
) -> redis::RedisResult<Option<SessionData>> {
    let key = format!("session:{session_id}");
    let raw: Option<String> = redis.get(&key).await?;
    if let Some(raw) = raw {
        let mut data: SessionData = serde_json::from_str(&raw).unwrap();
        let now = chrono::Utc::now().timestamp();
        // Enforce absolute 30-day limit
        if now - data.created_at > ABSOLUTE_TTL_SECS {
            redis.del(&key).await?;
            return Ok(None);
        }
        // Slide the TTL
        data.last_refreshed_at = now;
        let updated = serde_json::to_string(&data).unwrap();
        redis.set_ex(&key, updated, SLIDING_TTL_SECS).await?;
        Ok(Some(data))
    } else {
        Ok(None)
    }
}

pub async fn invalidate_session(
    redis: &mut ConnectionManager,
    session_id: &str,
) -> redis::RedisResult<()> {
    redis.del(format!("session:{session_id}")).await
}
```

**Invalidate all sessions for a user:** Requires a secondary index. Use a Redis Set: `user_sessions:{user_id}` containing all active session_ids.

```rust
// On session create: also SADD user_sessions:{user_id} {session_id}
// On logout_all: SMEMBERS user_sessions:{user_id} -> DEL each session:{sid} -> DEL user_sessions:{user_id}
// On deactivation: same pattern as logout_all
```

### Pattern 8: API Key Authentication in Gateway

**What:** Detect `madome_sk_` prefix in `Authorization: Bearer` header; delegate verification to Auth gRPC.

```rust
// services/gateway/src/middleware/auth.rs
pub enum AuthCredential {
    Jwt(String),
    ApiKey(String),
}

pub fn extract_credential(headers: &HeaderMap) -> Option<AuthCredential> {
    let bearer = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())?
        .strip_prefix("Bearer ")?;
    if bearer.starts_with("madome_sk_") {
        Some(AuthCredential::ApiKey(bearer.to_string()))
    } else {
        // Fall through to cookie check
        None
    }
}
```

### Pattern 9: 4-Tier Route Guard in axum

**What:** Different route sets use different extractors/middleware. Implemented via axum layer nesting.

```rust
// services/gateway/src/routes/mod.rs
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::gateway_health))
        .nest("/v1", v1_routes(state.clone()))
        .with_state(state)
}

fn v1_routes(state: AppState) -> Router<AppState> {
    Router::new()
        // Public — no auth extractor applied
        .nest("/auth", public_auth_routes())
        // Protected — JwtClaims extractor on each handler
        .nest("/auth", protected_auth_routes())
        // Admin — requires JwtClaims with role="admin"
        .nest("/admin", admin_routes())
        // Scraper — ApiKeyAuth extractor
        .nest("/scraper", scraper_routes())
        .route("/health/services", get(health::service_health))
}
```

### Pattern 10: HttpOnly Cookie Setting

**What:** Return `CookieJar` from handler with JWT cookie added; axum-extra translates to `Set-Cookie` header.

```rust
// Source: https://docs.rs/axum-extra/0.12.5/axum_extra/extract/cookie/
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};

pub fn set_jwt_cookie(jar: CookieJar, token: &str, secure: bool) -> CookieJar {
    let mut cookie = Cookie::new("token", token.to_string());
    cookie.set_http_only(true);
    cookie.set_secure(secure);  // false in dev via COOKIE_SECURE=false
    cookie.set_same_site(SameSite::Strict);
    cookie.set_path("/");
    jar.add(cookie)
}
```

### Pattern 11: Auto-Migration on Service Startup

**What:** Run `Migrator::up(db, None)` before starting gRPC server. Admin seed is the final migration.

```rust
// services/auth/src/main.rs
// Source: https://www.sea-ql.org/SeaORM/docs/migration/running-migration/
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use crate::migration::Migrator;

let db = Database::connect(&config.database_url).await?;
Migrator::up(&db, None).await?;
// Then start gRPC server
```

### Pattern 12: Recovery Code Generation

```rust
// Claude's discretion: 8-character alphanumeric from [A-Z0-9] (GitHub/Discord pattern per D-21)
use rand::Rng;

pub fn generate_recovery_code() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    (0..8)
        .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
        .collect()
}

pub fn generate_recovery_codes() -> Vec<String> {
    (0..10).map(|_| generate_recovery_code()).collect()
}
```

### Anti-Patterns to Avoid

- **Storing challenge state in client cookie:** WebAuthn challenge state (PasskeyRegistration, PasskeyAuthentication) MUST be stored server-side (Redis). Never send it to the client — this enables replay attacks.
- **Calling Auth service on every valid JWT:** The whole point of stateless JWT is that the Gateway does NOT call Auth for valid tokens. Only expired-past-grace JWTs trigger a session check.
- **Using `rand::thread_rng()` without seeding in tests:** Tests need deterministic or properly seeded RNG. Use `OsRng` in production.
- **Storing raw API keys:** Only the SHA-256 hash goes in the DB (D-26). The raw key is returned once and never stored.
- **Storing raw invite tokens:** Same pattern — hash in DB, raw returned once.
- **Fat gateway pattern:** Auth business logic (credential storage, session management, code generation) belongs in Auth service, not Gateway. Gateway is a translator only (D-60).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| WebAuthn ceremony | Custom CBOR/COSE parsing, challenge/response validation | webauthn-rs 0.5.4 | Cryptographic correctness; attestation handling; credential update after counter increment |
| JWT signing/verification | Custom ECDSA or HMAC token | jsonwebtoken 10.3.0 | Timing-safe comparison; spec-compliant claims; key format handling |
| In-memory TTL cache | DashMap + manual eviction goroutine | moka 0.12 | Lock-free eviction; race conditions in manual TTL management |
| DB migrations | Running raw SQL at startup | sea-orm-migration | Idempotent up/down; status tracking; consistent across services |
| Redis connection management | Manual reconnect logic | redis::ConnectionManager | Automatic reconnection; multiplexed commands |
| Cookie security attributes | Manual `Set-Cookie` header string | axum-extra CookieJar | Encoding, attribute merging, jar-as-response-part contract |

**Key insight:** Auth is a domain where small mistakes (timing attacks, state confusion, parameter substitution) have severe consequences. Every item in this list has been exploited in production systems that hand-rolled it.

---

## Common Pitfalls

### Pitfall 1: WebAuthn RP ID / RP Origin Mismatch

**What goes wrong:** `start_passkey_registration` or `start_passkey_authentication` returns `WebauthnError::InvalidRpOrigin` / credentials bound to different RP cannot authenticate.
**Why it happens:** RP ID must be the effective domain (e.g., `example.com`). RP Origin must be the full scheme+host (e.g., `https://example.com`). In dev, using `localhost` as RP ID requires `http://localhost` as origin (not HTTPS). Mismatches cause ceremony failures.
**How to avoid:** Set RP_ID and RP_ORIGIN as environment variables (D-18). Use `COOKIE_SECURE=false` + `http://localhost` origin for dev. Never hardcode.
**Warning signs:** `WebauthnError` at begin or finish time with origin/id mismatch messages.

### Pitfall 2: Challenge State Not Persisted Between Begin and Finish

**What goes wrong:** `finish_passkey_registration` or `finish_passkey_authentication` fails with challenge mismatch.
**Why it happens:** The `PasskeyRegistration` / `PasskeyAuthentication` state object contains the challenge nonce. If not serialized and stored server-side (e.g., stored in process memory instead of Redis), it is lost on restart or if a different gateway instance handles the finish request.
**How to avoid:** Always serialize challenge state to Redis with a short TTL (5 min). The challenge ID ties begin to finish. Delete the state from Redis on successful finish.
**Warning signs:** Intermittent ceremony failures, especially after service restart.

### Pitfall 3: `start_passkey_authentication` with Non-empty Credentials Breaks Discoverable Flow

**What goes wrong:** Passing stored credentials to `start_passkey_authentication` causes the authenticator to only accept those specific credentials, defeating the username-less flow.
**Why it happens:** The method signature takes `&[Passkey]`. Passing a list hints the authenticator about which credential to use, requiring the user to select a specific device. For discoverable/username-less (D-17), pass `&[]`.
**How to avoid:** Always call `webauthn.start_passkey_authentication(&[])` for the login flow. Credential-specific auth (for additional passkey registration D-20) uses a different approach.
**Warning signs:** Platform authenticator shows specific key choices instead of any passkey on device.

### Pitfall 4: JWT Grace Period Logic Inverted

**What goes wrong:** Every expired JWT is treated as requiring a session check, defeating the grace period purpose; or valid JWTs are treated as in-grace.
**Why it happens:** The grace check compares `exp + grace_seconds >= now`, but `exp` is the expiry timestamp (past), and `now` is current. The inequality direction is easy to invert.
**How to avoid:** Grace period = `now <= exp + 60`. In words: "current time is within 60 seconds after expiry." Use the pattern in Pattern 5 above; write a unit test with boundary values.
**Warning signs:** All auth refreshes go through gRPC (no grace period savings) or all expired tokens bypass session check.

### Pitfall 5: Session Absolute TTL Bypass

**What goes wrong:** A session refreshed continuously never expires beyond 30 days.
**Why it happens:** If the sliding TTL is reset without checking `created_at`, a long-lived active session never hits the absolute limit.
**How to avoid:** On every `validate_and_slide_session` call, check `now - data.created_at > ABSOLUTE_TTL_SECS` before sliding (Pattern 7). If over absolute limit, delete and return None.
**Warning signs:** Sessions that remain valid beyond 30 days.

### Pitfall 6: API Key Prefix Ambiguity

**What goes wrong:** A JWT that begins with `madome_sk_` (extremely unlikely but possible) is misidentified as an API key.
**Why it happens:** Base64url characters include letters and underscores. A JWT header+payload starting with `madome_sk_` is theoretically possible.
**How to avoid:** JWTs have two dots (`.`). Check: `if token.contains('.') { treat as JWT } else if token.starts_with("madome_sk_") { treat as API key }`. JWTs cannot contain underscores in the typical positions that would form the prefix.
**Warning signs:** Auth failures for valid JWTs, 401 errors on protected routes.

### Pitfall 7: sea-orm Runtime Feature Flag

**What goes wrong:** Compilation error or TLS handshake failure when connecting to PostgreSQL.
**Why it happens:** sea-orm has two TLS feature variants: `runtime-tokio-native-tls` (OS TLS) and `runtime-tokio-rustls` (Rust TLS). Using rustls on macOS/Linux for local dev without proper CA configuration fails.
**How to avoid:** Use `runtime-tokio-native-tls` for local dev (simpler, uses OS cert store). Only switch to rustls if deploying to containers without native TLS available.
**Warning signs:** `SslHandshakeFailure` or TLS errors at database connect time.

### Pitfall 8: webauthn-rs Passkey Counter Update

**What goes wrong:** Authenticator counter does not update in DB after successful authentication; `AuthenticationResult` contains `needs_update()` = true.
**Why it happens:** After `finish_passkey_authentication`, the returned `AuthenticationResult` may indicate the stored `Passkey` credential needs its counter updated. If this update is skipped, counter-based clone detection stops working.
**How to avoid:** After each successful `finish_passkey_authentication`, check `auth_result.needs_update()`. If true, call `passkey.update_credential(&auth_result)` and persist the updated `Passkey` to DB.
**Warning signs:** Gradual drift in counter values; potential security log warnings from webauthn-rs.

---

## Code Examples

### webauthn-rs WebauthnBuilder Setup
```rust
// Source: https://docs.rs/webauthn-rs/0.5.4/webauthn_rs/
use webauthn_rs::prelude::*;
use url::Url;

let rp_id = std::env::var("RP_ID").unwrap(); // e.g. "madome.example.com"
let rp_origin = Url::parse(&std::env::var("RP_ORIGIN").unwrap()).unwrap(); // "https://madome.example.com"

let webauthn = WebauthnBuilder::new(&rp_id, &rp_origin)
    .unwrap()
    .rp_name("Madome")
    .build()
    .unwrap();
```

### jsonwebtoken ES256 Key Loading
```rust
// Source: https://docs.rs/jsonwebtoken/10.3.0/jsonwebtoken/
use jsonwebtoken::{EncodingKey, DecodingKey};

// Auth service (has private key)
let private_pem = std::env::var("JWT_PRIVATE_KEY").unwrap();
let encoding_key = EncodingKey::from_ec_pem(private_pem.as_bytes()).unwrap();

// Gateway (has public key only)
let public_pem = std::env::var("JWT_PUBLIC_KEY").unwrap();
let decoding_key = DecodingKey::from_ec_pem(public_pem.as_bytes()).unwrap();
```

### Testcontainers Setup for Auth Integration Tests
```rust
// Source: https://docs.rs/testcontainers-modules/0.15.0/
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::{postgres::Postgres, redis::Redis};

#[tokio::test]
async fn test_passkey_registration_flow() {
    let pg = Postgres::default().start().await.unwrap();
    let redis = Redis::default().start().await.unwrap();

    let db_url = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        pg.get_host_port_ipv4(5432).await.unwrap()
    );
    let redis_url = format!(
        "redis://127.0.0.1:{}/",
        redis.get_host_port_ipv4(6379).await.unwrap()
    );
    // Connect, run migrations, run test scenario
}
```

### moka Cache for JWT Deduplication
```rust
// Source: https://docs.rs/moka/0.12.14/moka/future/
use moka::future::Cache;
use std::time::Duration;

let jwt_cache: Cache<String, String> = Cache::builder()
    .time_to_live(Duration::from_secs(10))
    .build();

// Atomic get-or-create pattern
let token = jwt_cache.get_with(session_id.to_string(), async {
    issue_new_jwt(&claims, &encoding_key)
}).await;
```

### Redis Connection in Auth Service State
```rust
// Source: https://docs.rs/redis/1.0.5/redis/aio/
use redis::{aio::ConnectionManager, Client};

let client = Client::open(config.redis_url.as_str())?;
let redis = ConnectionManager::new(client).await?;
// Store Arc<ConnectionManager> in AppState; clone cheaply per request
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Password + TOTP 2FA | Passkey (discoverable credential) | WebAuthn L2 / FIDO2 2019+ | Single-gesture MFA; phishing-resistant |
| JWT in localStorage | HttpOnly cookie | Security best practice shift ~2020 | Prevents XSS token theft |
| RS256 JWT | ES256 JWT | General recommendation 2022+ | Smaller signatures; comparable security |
| ring backend for JWT | aws_lc backend | aws-lc-rs stable ~2023 | FIPS validation; faster ECDSA on x86_64 |
| Single JWT with long TTL | Short JWT (15m) + session refresh | Standard since ~2021 | Limits stolen token window |
| Username-based passkey | Discoverable credential (username-less) | WebAuthn L3 / passkey spec 2022+ | User just touches authenticator, no username input |

**Deprecated/outdated:**
- webauthn-rs 0.4.x: Replaced by 0.5.x with resident-key-support and updated passkey API
- jsonwebtoken 8.x/9.x: 10.x adds explicit aws_lc backend support
- `rand` 0.8.x: 0.10.0 is current stable (released Feb 2026)
- sea-orm 2.0-rc: Still pre-release (RC37); use 1.1.19 stable

---

## Open Questions

1. **Proto message structure for WebAuthn JSON blobs**
   - What we know: `CreationChallengeResponse` and `RegisterPublicKeyCredential` are JSON (serde-serializable)
   - What's unclear: Should proto messages carry these as `string` (JSON-encoded) or decompose them into fields? Decomposing requires duplicating the WebAuthn spec in proto.
   - Recommendation: Use `string` fields for WebAuthn challenge/response blobs. Gateway serializes to/from JSON; Auth deserializes using webauthn-rs-proto types. Avoids proto schema coupling to WebAuthn spec evolution.

2. **Redis key structure for user_sessions index (logout_all)**
   - What we know: Need to find all sessions by user_id to invalidate them all (D-36, D-38)
   - What's unclear: Redis Set (`user_sessions:{user_id}`) vs Redis SCAN with pattern matching
   - Recommendation (Claude's discretion): Use `SADD user_sessions:{user_id} {session_id}` on create; `SMEMBERS` + `DEL` on logout_all. Set members are the session IDs; expire the set with an absolute TTL matching the absolute session limit (30 days).

3. **SoftPasskey API in webauthn-rs for testing**
   - What we know: D-66 specifies webauthn-rs SoftPasskey/mock authenticator for tests
   - What's unclear: Exact API for SoftPasskey in 0.5.4 — may be in a `testable` or `softpasskey` feature
   - Recommendation: Check `webauthn-rs-proto` and `webauthn-rs` test utilities. If SoftPasskey is unavailable in 0.5.4, test the begin/finish steps independently: verify challenge is issued correctly; test finish with a pre-generated valid credential JSON fixture.

---

## Validation Architecture

> `workflow.nyquist_validation` is `true` in `.planning/config.json` — section included.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) + tokio-test runtime |
| Config file | `Cargo.toml` per-crate `[dev-dependencies]` |
| Quick run command | `cargo test -p auth --lib` |
| Full suite command | `cargo test -p auth && cargo test -p gateway` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AUTH-01 | Passkey registration: begin returns challenge; finish creates credential | Integration (testcontainers) | `cargo test -p auth --test registration_integration` | ❌ Wave 0 |
| AUTH-02 | Passkey authentication: username-less begin/finish returns session+JWT | Integration (testcontainers) | `cargo test -p auth --test authentication_integration` | ❌ Wave 0 |
| AUTH-03 | JWT issued with correct claims (sub/role/sid/iss/aud/exp) | Unit | `cargo test -p auth --lib jwt::tests` | ❌ Wave 0 |
| AUTH-04 | Session create/validate/invalidate in Redis | Integration (testcontainers) | `cargo test -p auth --test session_integration` | ❌ Wave 0 |
| AUTH-05 | Concurrent JWT requests within 10s return same token | Unit (moka cache) | `cargo test -p auth --lib jwt::tests::duplicate_prevention` | ❌ Wave 0 |
| GATE-03 | Valid JWT passes stateless (no Auth gRPC call) | Integration (gateway E2E) | `cargo test -p gateway --test auth_middleware` | ❌ Wave 0 |
| GATE-04 | Expired JWT in grace gets new cookie; past-grace calls ValidateSession | Integration (gateway E2E) | `cargo test -p gateway --test auth_middleware` | ❌ Wave 0 |
| GATE-05 | API key request on scraper route succeeds; JWT on scraper route fails | Integration (gateway E2E) | `cargo test -p gateway --test api_key_auth` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p auth --lib && cargo test -p gateway --lib`
- **Per wave merge:** `cargo test -p auth && cargo test -p gateway`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `services/auth/tests/registration_integration.rs` — covers AUTH-01
- [ ] `services/auth/tests/authentication_integration.rs` — covers AUTH-02
- [ ] `services/auth/src/jwt.rs` (unit test module) — covers AUTH-03, AUTH-05
- [ ] `services/auth/tests/session_integration.rs` — covers AUTH-04
- [ ] `services/gateway/tests/auth_middleware.rs` — covers GATE-03, GATE-04
- [ ] `services/gateway/tests/api_key_auth.rs` — covers GATE-05
- [ ] Auth service `migration/` crate (new sub-crate) — prerequisite for all DB tests
- [ ] Auth service `schema/` crate (new sub-crate) — prerequisite for all entity tests
- [ ] `docker-compose.yml` in project root — prerequisite for dev-start.sh + local testcontainer availability
- [ ] `[dev-dependencies]` additions to `services/auth/Cargo.toml` and `services/gateway/Cargo.toml`

---

## Sources

### Primary (HIGH confidence)

- `https://docs.rs/webauthn-rs/0.5.4/webauthn_rs/` — Webauthn struct methods, builder API, PasskeyRegistration/Authentication types
- `https://docs.rs/jsonwebtoken/10.3.0/jsonwebtoken/` — encode/decode, EncodingKey/DecodingKey from PEM, Validation struct
- `https://docs.rs/moka/0.12.14/moka/future/struct.Cache.html` — Cache builder, get_with atomic pattern
- `https://docs.rs/redis/1.0.5/redis/` — ConnectionManager, AsyncCommands (set_ex, get, del, exists)
- `https://docs.rs/axum-extra/0.12.5/axum_extra/extract/cookie/` — CookieJar extractor, Cookie builder
- `https://www.sea-ql.org/SeaORM/docs/migration/running-migration/` — Migrator::up programmatic startup
- `crates.io API` — All version numbers verified 2026-03-22
- `.planning/phases/02-authentication/02-CONTEXT.md` — All D-XX decisions (authoritative project constraints)
- `.planning/research/STACK.md` — Confirmed stack versions (HIGH, verified 2026-03-21)
- `.planning/PROJECT.md` — Auth design section, JWT flow, duplicate prevention

### Secondary (MEDIUM confidence)

- `https://docs.rs/testcontainers-modules/0.15.0/testcontainers_modules/` — postgres/redis module usage pattern
- WebSearch 2026-03-22: axum FromRequestParts cookie JWT middleware — confirmed pattern against multiple blog posts + official axum docs

### Tertiary (LOW confidence)

- webauthn-rs SoftPasskey test authenticator API — not verified against current 0.5.4 docs; needs investigation in Wave 0

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all versions verified via crates.io API on 2026-03-22
- Architecture: HIGH — all patterns derived from official docs + locked decisions in CONTEXT.md
- Pitfalls: HIGH — items 1-7 from official webauthn spec security requirements and project design; item 8 (counter update) from webauthn-rs docs
- Validation architecture: HIGH — test framework well-established; test file paths are new (Wave 0 gaps confirmed by ls)

**Research date:** 2026-03-22
**Valid until:** 2026-04-22 (stable crates; webauthn-rs 0.5.4 stable, no new stable version expected soon)
