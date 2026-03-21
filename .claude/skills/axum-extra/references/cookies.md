# axum-extra Cookie Reference

> axum-extra 0.12 (feature: `cookie`) | Source: https://docs.rs/axum-extra/0.12

## CookieJar

`CookieJar` is both an **extractor** (reads `Cookie` header) and a **response**
(sets `Set-Cookie` headers). Return it in a tuple to set cookies.

```rust
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};

// Read + write
async fn handler(jar: CookieJar) -> (CookieJar, &'static str) {
    // Read
    if let Some(cookie) = jar.get("session") {
        println!("session = {}", cookie.value());
    }

    // Write (add)
    let jar = jar.add(
        Cookie::build(("session", "value123"))
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Lax)
            .max_age(time::Duration::days(7))
    );

    (jar, "ok")
}
```

## Cookie Builder

```rust
use axum_extra::extract::cookie::{Cookie, SameSite};

// Full builder
let cookie = Cookie::build(("name", "value"))
    .path("/")
    .domain("example.com")
    .http_only(true)
    .secure(true)
    .same_site(SameSite::Lax)
    .max_age(time::Duration::seconds(604800));

// Shorthand (name + value only)
let cookie = Cookie::new("name", "value");
```

### Cookie Builder Methods

| Method | Type | Description |
|--------|------|-------------|
| `.path(p)` | `&str` | URL path scope |
| `.domain(d)` | `&str` | Domain scope |
| `.http_only(b)` | `bool` | Prevent JavaScript access |
| `.secure(b)` | `bool` | HTTPS only |
| `.same_site(ss)` | `SameSite` | Cross-site policy |
| `.max_age(d)` | `time::Duration` | Lifetime (0 = delete) |
| `.expires(t)` | `OffsetDateTime` | Absolute expiry |

### SameSite Values

| Value | Behavior |
|-------|----------|
| `SameSite::Strict` | Never sent cross-site |
| `SameSite::Lax` | Sent on top-level navigations |
| `SameSite::None` | Sent always (requires `secure(true)`) |

## Cookie Removal

To remove a cookie, set it with `max_age(Duration::ZERO)`.
The `path` and `domain` must match the original cookie.

```rust
async fn logout(jar: CookieJar) -> (CookieJar, &'static str) {
    let removal = Cookie::build(("session", ""))
        .path("/")
        .max_age(time::Duration::ZERO);

    (jar.remove(removal), "logged out")
}
```

Alternative: use `jar.remove("cookie_name")` for simple cases (path = /).

## Multiple Cookies

```rust
async fn handler(jar: CookieJar) -> (CookieJar, &'static str) {
    let jar = jar
        .add(Cookie::build(("access_token", token1)).path("/"))
        .add(Cookie::build(("refresh_token", token2)).path("/auth/token"));

    (jar, "ok")
}
```

## Iterating Cookies

```rust
async fn handler(jar: CookieJar) {
    for cookie in jar.iter() {
        println!("{} = {}", cookie.name(), cookie.value());
    }
}
```

## SignedCookieJar (feature: `cookie-signed`)

Adds HMAC signature — detects tampering but value is still readable.

```rust
use axum_extra::extract::cookie::{SignedCookieJar, Key};

// Key must be in axum State
let key = Key::generate(); // or Key::from(master_key_bytes)

async fn handler(jar: SignedCookieJar) -> (SignedCookieJar, &'static str) {
    let jar = jar.add(Cookie::new("user_id", "42"));
    (jar, "ok")
}
```

## PrivateCookieJar (feature: `cookie-private`)

Encrypts cookie value — not readable by client.

```rust
use axum_extra::extract::cookie::{PrivateCookieJar, Key};

async fn handler(jar: PrivateCookieJar) -> (PrivateCookieJar, &'static str) {
    let jar = jar.add(Cookie::new("secret", "classified"));
    (jar, "ok")
}
```

## Important Notes

- `CookieJar` does NOT persist across requests — it reads from the request `Cookie` header
  and writes to response `Set-Cookie` headers
- `time::Duration` is from the `time` crate, NOT `std::time::Duration`
- Removing a cookie that was set with a specific `path` requires matching that `path`
- `CookieJar` implements `IntoResponse` and `IntoResponseParts`
