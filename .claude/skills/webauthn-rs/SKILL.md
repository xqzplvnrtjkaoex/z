---
name: webauthn-rs
description: |
  CRITICAL: Use for WebAuthn/passkey implementation. Triggers on:
  webauthn-rs, Webauthn, passkey, FIDO2, PublicKeyCredential,
  RegisterPublicKeyCredential, CreationChallengeResponse,
  RequestChallengeResponse, passkey registration, passkey authentication,
  start_passkey_registration, finish_passkey_registration
---

# webauthn-rs Skill

> **Version:** webauthn-rs 0.5 | **Last Updated:** 2026-03-01
>
> Check for updates: https://crates.io/crates/webauthn-rs

You are an expert at the Rust `webauthn-rs` crate. Help users by:
- **Writing code**: Generate passkey registration/authentication flows
- **Answering questions**: Explain WebAuthn ceremonies, state management, credential storage

## Documentation

- `./references/api.md` — Webauthn setup, registration, authentication, types, state serialization

## Key Patterns

### Webauthn Instance Setup

```rust
use webauthn_rs::prelude::*;

let rp_id = "example.com";
let rp_origin = Url::parse("https://example.com").unwrap();
let webauthn = WebauthnBuilder::new(rp_id, &rp_origin)?
    .rp_name("My Application")
    .build()?;
```

### Passkey Registration (Two-Step)

```rust
// Step 1: Start registration (server → client)
let (ccr, reg_state) = webauthn.start_passkey_registration(
    user_unique_id,   // Uuid
    &user_name,       // &str
    &display_name,    // &str
    None,             // exclude_credentials: Option<Vec<Credential>>
)?;
// Send `ccr` (CreationChallengeResponse) to client as JSON
// Store `reg_state` (PasskeyRegistration) in session/cache

// Step 2: Finish registration (client → server)
let passkey: Passkey = webauthn.finish_passkey_registration(
    &reg_public_key_credential, // RegisterPublicKeyCredential from client
    &reg_state,                 // PasskeyRegistration from step 1
)?;
// Store `passkey` in database
```

### Passkey Authentication (Two-Step)

```rust
// Step 1: Start authentication (server → client)
let (rcr, auth_state) = webauthn.start_passkey_authentication(
    &[passkey],  // &[Passkey] — user's stored credentials
)?;
// Send `rcr` (RequestChallengeResponse) to client as JSON
// Store `auth_state` (PasskeyAuthentication) in session/cache

// Step 2: Finish authentication (client → server)
let auth_result = webauthn.finish_passkey_authentication(
    &public_key_credential,  // PublicKeyCredential from client
    &auth_state,             // PasskeyAuthentication from step 1
)?;
// auth_result.update_credential(&mut passkey) to bump counter
```

## API Reference Table

| Type | Description |
|------|-------------|
| `Webauthn` | Main service object — holds RP config |
| `WebauthnBuilder` | Builder for `Webauthn` (rp_id, rp_origin, rp_name) |
| `Passkey` | Stored credential (serialize to DB) |
| `PasskeyRegistration` | Transient state during registration ceremony |
| `PasskeyAuthentication` | Transient state during authentication ceremony |
| `CreationChallengeResponse` | JSON sent to client for `navigator.credentials.create()` |
| `RequestChallengeResponse` | JSON sent to client for `navigator.credentials.get()` |
| `RegisterPublicKeyCredential` | JSON received from client after `create()` |
| `PublicKeyCredential` | JSON received from client after `get()` |
| `AuthenticationResult` | Result of `finish_passkey_authentication` |

## When Writing Code

1. `Webauthn` is `Clone` + `Send + Sync` — share via axum `State`
2. Registration/authentication states are **transient** — store in Redis/session, not DB
3. `danger-allow-state-serialisation` feature enables Serialize/Deserialize on state types
4. `Passkey` is the **persistent** credential — store in DB (JSON column or structured)
5. All `start_*` methods return `(challenge, state)` — challenge goes to client, state stays server-side
6. All `finish_*` methods consume the state — one-time use per ceremony

## When Answering Questions

1. WebAuthn has two **ceremonies**: registration and authentication, each with start/finish
2. `rp_id` must match the domain (or a registrable suffix) — cannot be changed after credentials are created
3. State types are not `Serialize` by default — need `danger-allow-state-serialisation` feature
4. `AuthenticationResult.update_credential()` updates the signature counter for cloning detection
5. `exclude_credentials` in registration prevents re-registering the same authenticator
