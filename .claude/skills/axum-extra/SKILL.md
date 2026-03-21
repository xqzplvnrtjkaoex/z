---
name: axum-extra
description: |
  CRITICAL: Use for axum-extra extractors and utilities. Triggers on:
  axum-extra, CookieJar, Cookie, SameSite, TypedHeader,
  cookie handling, set cookie, remove cookie, cookie attributes,
  Form, Query extraction, cached header
---

# axum-extra Skill

> **Version:** axum-extra 0.12 | **Last Updated:** 2026-03-01
>
> Check for updates: https://crates.io/crates/axum-extra

You are an expert at the Rust `axum-extra` crate. Help users by:
- **Writing code**: Generate cookie handling, typed header extraction, form parsing
- **Answering questions**: Explain cookie attributes, extractor ordering, header types

## Documentation

- `./references/cookies.md` — CookieJar, Cookie builder, SameSite, removal, signed/private jars
- `./references/extractors.md` — TypedHeader, Form, Query, cached extractors

## Key Patterns

### Cookie Extraction and Setting

```rust
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};

async fn handler(jar: CookieJar) -> (CookieJar, &'static str) {
    // Read cookie
    let token = jar.get("session").map(|c| c.value().to_owned());

    // Set cookie
    let cookie = Cookie::build(("session", "abc123"))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::days(7))
        .secure(true);

    (jar.add(cookie), "ok")
}
```

### Cookie Removal

```rust
async fn logout(jar: CookieJar) -> (CookieJar, &'static str) {
    // Must match path/domain of the original cookie
    let removal = Cookie::build(("session", ""))
        .path("/")
        .max_age(time::Duration::ZERO);

    (jar.remove(removal), "logged out")
}
```

### TypedHeader Extraction

```rust
use axum_extra::TypedHeader;
use axum_extra::headers::{Authorization, authorization::Bearer};

async fn handler(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> String {
    format!("token: {}", auth.token())
}
```

## API Reference Table

| Type | Feature | Description |
|------|---------|-------------|
| `CookieJar` | `cookie` | Read/write cookies (unsigned) |
| `SignedCookieJar` | `cookie-signed` | Tamper-proof cookies (HMAC) |
| `PrivateCookieJar` | `cookie-private` | Encrypted cookies |
| `Cookie` | `cookie` | Cookie builder (name, value, attributes) |
| `TypedHeader<T>` | `typed-header` | Strongly-typed header extraction |
| `Form<T>` | — | URL-encoded form body |
| `Query<T>` | — | Query string extraction |
| `Cached<T>` | — | Cache extractor result for reuse |

## When Writing Code

1. `CookieJar` is an extractor AND a response — return it in a tuple to set cookies
2. Cookie removal requires matching `path` and `domain` of the original cookie
3. Use `Cookie::build(("name", "value"))` — the tuple form, not separate args
4. `max_age` uses `time::Duration`, not `std::time::Duration`
5. For auth cookies: always set `http_only(true)`, `secure(true)`, `same_site(SameSite::Lax)`
6. `CookieJar` does not persist — it reads from request headers and writes to response headers

## When Answering Questions

1. `cookie` feature enables `CookieJar`; `typed-header` enables `TypedHeader`
2. `CookieJar` uses the `cookie` crate internally — `Cookie` is re-exported from there
3. Removing a cookie = setting it with `max_age(Duration::ZERO)` (or negative)
4. `SignedCookieJar` needs a `Key` in state; `PrivateCookieJar` also needs a `Key`
5. `TypedHeader` rejects the request with 400 if the header is missing or malformed
