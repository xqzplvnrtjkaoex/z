---
name: jsonwebtoken
description: |
  CRITICAL: Use for JWT token creation and validation. Triggers on:
  jsonwebtoken, JWT, encode, decode, EncodingKey, DecodingKey,
  token claims, token validation, Header, Validation, Algorithm
---

# jsonwebtoken Skill

> **Version:** jsonwebtoken 10.3 | **Last Updated:** 2026-03-01
>
> Check for updates: https://crates.io/crates/jsonwebtoken

You are an expert at the Rust `jsonwebtoken` crate. Help users by:
- **Writing code**: Generate JWT encoding/decoding following best practices
- **Answering questions**: Explain claims, validation, algorithms, key types

## Documentation

Refer to the local files for detailed documentation:
- `./references/api.md` — Full API: encode, decode, Validation, Header, algorithms

## Key Patterns

### Encode (Create Token)

```rust
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: u64,
}

let claims = Claims {
    sub: user_id.to_string(),
    exp: now_secs + 3600,
};

let token = encode(
    &Header::default(), // HS256
    &claims,
    &EncodingKey::from_secret(secret.as_bytes()),
)?;
```

### Decode (Validate Token)

```rust
use jsonwebtoken::{decode, DecodingKey, Validation};

let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
validation.validate_exp = true;
validation.required_spec_claims.clear();
validation.set_required_spec_claims(&["exp", "sub"]);

let data = decode::<Claims>(
    &token,
    &DecodingKey::from_secret(secret.as_bytes()),
    &validation,
)?;

let claims = data.claims;
```

### Custom Header (Algorithm)

```rust
let header = Header::new(jsonwebtoken::Algorithm::RS256);
let token = encode(&header, &claims, &EncodingKey::from_rsa_pem(pem)?)?;
```

## API Reference Table

| Function/Type | Description |
|---------------|-------------|
| `encode(header, claims, key)` | Create JWT string |
| `decode::<T>(token, key, validation)` | Validate and parse JWT |
| `Header::default()` | HS256 header |
| `Header::new(algo)` | Custom algorithm header |
| `EncodingKey::from_secret(bytes)` | HMAC secret key |
| `EncodingKey::from_rsa_pem(pem)` | RSA private key |
| `DecodingKey::from_secret(bytes)` | HMAC secret key |
| `DecodingKey::from_rsa_pem(pem)` | RSA public key |
| `Validation::new(algo)` | Validation config |
| `validation.validate_exp` | Enable/disable exp check |
| `validation.set_required_spec_claims` | Require specific claims |
| `validation.set_audience(&[aud])` | Require audience |
| `validation.set_issuer(&[iss])` | Require issuer |

## Algorithms

| Algorithm | Type | Key |
|-----------|------|-----|
| `HS256` (default) | HMAC | Shared secret |
| `HS384`, `HS512` | HMAC | Shared secret |
| `RS256`, `RS384`, `RS512` | RSA | PEM key pair |
| `ES256`, `ES384` | ECDSA | PEM key pair |
| `EdDSA` | EdDSA | PEM key pair |

## When Writing Code

1. Always set `exp` claim for token expiry
2. Use `Validation::new(Algorithm)` to enforce algorithm
3. Clear default required claims with `.required_spec_claims.clear()` if needed
4. HMAC keys: use `from_secret(secret.as_bytes())`
5. Map decode errors to your domain error type

## When Answering Questions

1. `decode()` validates signature + exp by default
2. `TokenData { header, claims }` is the decode return type
3. `jsonwebtoken::errors::ErrorKind` has `ExpiredSignature`, `InvalidToken`, etc.
4. v10.x uses `aws_lc_rs` feature for crypto backend
