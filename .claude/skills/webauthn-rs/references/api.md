# webauthn-rs API Reference

> webauthn-rs 0.5 | Source: https://docs.rs/webauthn-rs/0.5

## WebauthnBuilder

```rust
use webauthn_rs::prelude::*;
use url::Url;

let rp_origin = Url::parse("https://example.com").unwrap();
let webauthn = WebauthnBuilder::new("example.com", &rp_origin)?
    .rp_name("My Application")
    .allow_subdomains(true)        // allow credentials from subdomains
    .append_allowed_origin(&Url::parse("https://sub.example.com")?)
    .build()?;
```

| Method | Description |
|--------|-------------|
| `new(rp_id, rp_origin)` | Create builder with RP ID (domain) and origin |
| `.rp_name(name)` | Human-readable RP name |
| `.allow_subdomains(bool)` | Allow credentials from subdomains |
| `.append_allowed_origin(url)` | Add additional allowed origins |
| `.build()` | Consume builder, return `Webauthn` |

## Registration Ceremony

### Step 1: Start

```rust
let exclude_credentials: Option<Vec<Credential>> = Some(
    existing_passkeys.iter().map(|pk| pk.cred_id().clone()).collect()
);

let (ccr, reg_state) = webauthn.start_passkey_registration(
    user_id,                // Uuid — unique, stable user identifier
    &user_name,             // &str — username / email
    &display_name,          // &str — human-readable name
    exclude_credentials.as_deref(),
)?;

// ccr: CreationChallengeResponse — JSON-serialize and send to client
// reg_state: PasskeyRegistration — store server-side (Redis/session)
```

### Step 2: Finish

```rust
// Client sends RegisterPublicKeyCredential as JSON
let reg: RegisterPublicKeyCredential = serde_json::from_str(&body)?;

let passkey: Passkey = webauthn.finish_passkey_registration(
    &reg,
    &reg_state,  // retrieved from Redis/session
)?;

// Store `passkey` in database (JSON-serializable)
```

## Authentication Ceremony

### Step 1: Start

```rust
// Load user's stored passkeys from DB
let passkeys: Vec<Passkey> = db.get_passkeys(user_id).await?;

let (rcr, auth_state) = webauthn.start_passkey_authentication(&passkeys)?;

// rcr: RequestChallengeResponse — JSON-serialize and send to client
// auth_state: PasskeyAuthentication — store server-side (Redis/session)
```

### Step 2: Finish

```rust
// Client sends PublicKeyCredential as JSON
let cred: PublicKeyCredential = serde_json::from_str(&body)?;

let auth_result = webauthn.finish_passkey_authentication(
    &cred,
    &auth_state,  // retrieved from Redis/session
)?;

// Update credential counter for cloning detection
if let Some(passkey) = passkeys.iter_mut().find(|pk| {
    pk.cred_id() == auth_result.cred_id()
}) {
    passkey.update_credential(&auth_result);
    db.update_passkey(passkey).await?;
}
```

## Passkey Type

```rust
// Passkey is Serialize + Deserialize (always, not behind feature flag)
// Store as JSON in database

// Key methods:
passkey.cred_id()  // &CredentialID — unique credential identifier

// AuthenticationResult methods:
auth_result.cred_id()            // which credential was used
auth_result.needs_update()       // whether counter changed
```

## State Serialization

The `danger-allow-state-serialisation` feature enables `Serialize`/`Deserialize`
on transient state types:

- `PasskeyRegistration`
- `PasskeyAuthentication`

```rust
// Store in Redis during ceremony
let state_json = serde_json::to_vec(&reg_state)?;
conn.set_ex::<_, _, ()>(&cache_key, state_json.as_slice(), 300).await?;

// Retrieve
let bytes: Vec<u8> = conn.get(&cache_key).await?;
let reg_state: PasskeyRegistration = serde_json::from_slice(&bytes)?;
```

Without this feature, state types must be held in memory (not suitable for
multi-instance deployments).

## Error Handling

```rust
use webauthn_rs::prelude::WebauthnError;

match webauthn.finish_passkey_registration(&reg, &state) {
    Ok(passkey) => { /* store passkey */ }
    Err(WebauthnError::CredentialAlreadyExists) => { /* duplicate */ }
    Err(e) => { /* other error */ }
}
```

## Prelude Exports

```rust
use webauthn_rs::prelude::*;
// Exports: Webauthn, WebauthnBuilder, Passkey, Url,
//   CreationChallengeResponse, RequestChallengeResponse,
//   RegisterPublicKeyCredential, PublicKeyCredential,
//   PasskeyRegistration, PasskeyAuthentication,
//   AuthenticationResult, WebauthnError, Uuid, CredentialID
```
