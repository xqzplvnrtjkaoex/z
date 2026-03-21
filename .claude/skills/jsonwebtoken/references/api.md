# jsonwebtoken API Reference

> jsonwebtoken 10.3 | Source: https://docs.rs/jsonwebtoken/10.3

## Encode

```rust
use jsonwebtoken::{encode, EncodingKey, Header};

// HS256 (default)
let token = encode(
    &Header::default(),
    &claims,
    &EncodingKey::from_secret(secret.as_bytes()),
)?;

// RS256
let token = encode(
    &Header::new(Algorithm::RS256),
    &claims,
    &EncodingKey::from_rsa_pem(private_key_pem)?,
)?;
```

## Decode

```rust
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

let mut validation = Validation::new(Algorithm::HS256);
validation.validate_exp = true;

let token_data = decode::<MyClaims>(
    &token,
    &DecodingKey::from_secret(secret.as_bytes()),
    &validation,
)?;

// token_data.header: Header
// token_data.claims: MyClaims
```

## Validation Options

```rust
let mut validation = Validation::new(Algorithm::HS256);

// Expiry
validation.validate_exp = true;  // default: true

// Required claims
validation.required_spec_claims.clear(); // remove defaults
validation.set_required_spec_claims(&["exp", "sub"]);

// Audience
validation.set_audience(&["my-app"]);

// Issuer
validation.set_issuer(&["my-issuer"]);

// Leeway (clock skew tolerance)
validation.leeway = 60; // seconds

// Disable NBF validation
validation.validate_nbf = false;
```

## Error Handling

```rust
use jsonwebtoken::errors::ErrorKind;

match decode::<Claims>(&token, &key, &validation) {
    Ok(data) => data.claims,
    Err(e) => match e.kind() {
        ErrorKind::ExpiredSignature => { /* token expired */ }
        ErrorKind::InvalidToken => { /* malformed token */ }
        ErrorKind::InvalidSignature => { /* bad signature */ }
        ErrorKind::InvalidAudience => { /* wrong audience */ }
        ErrorKind::InvalidIssuer => { /* wrong issuer */ }
        _ => { /* other error */ }
    }
}
```

## EncodingKey Variants

| Constructor | Key type |
|------------|----------|
| `from_secret(bytes)` | HMAC shared secret |
| `from_rsa_pem(pem)` | RSA private key (PEM) |
| `from_rsa_der(der)` | RSA private key (DER) |
| `from_ec_pem(pem)` | ECDSA private key (PEM) |
| `from_ed_pem(pem)` | EdDSA private key (PEM) |

## DecodingKey Variants

| Constructor | Key type |
|------------|----------|
| `from_secret(bytes)` | HMAC shared secret |
| `from_rsa_pem(pem)` | RSA public key (PEM) |
| `from_rsa_der(der)` | RSA public key (DER) |
| `from_ec_pem(pem)` | ECDSA public key (PEM) |
| `from_ed_pem(pem)` | EdDSA public key (PEM) |
| `from_jwk(jwk)` | JSON Web Key |

## Claims Struct Pattern

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,    // subject (user ID)
    pub exp: u64,       // expiry (UNIX timestamp)
    pub iat: u64,       // issued at
    pub role: u8,       // custom claim
}
```

Standard claims: `exp`, `nbf`, `sub`, `aud`, `iss`, `iat`, `jti`.
All are optional in the struct but can be required via `Validation`.
