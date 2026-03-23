---
paths: ["*.rs"]
---

# Rust Coding Conventions

## Typed Constants Over String Literals

Prefer typed constants over raw string literals for any protocol-level or well-known identifier (HTTP headers, cookie names, MIME types, gRPC metadata keys, etc.).

- MUST: When the framework or library already provides a typed constant (e.g., `header::CONTENT_TYPE`, `header::SET_COOKIE`), always use it.
- SHOULD: When no typed constant exists, define your own (`const` or `HeaderName::from_static(...)`) rather than repeating string literals across call sites. Exceptions are acceptable for one-off or context-local usage.

## Error Conversion via `From` Trait

Implement `From<SourceError> for TargetError` instead of writing standalone conversion functions.
This enables the `?` operator for clean propagation — no `.map_err(helper)?` pattern.

## Cursor Encoding

Use `base64::engine::general_purpose::URL_SAFE_NO_PAD` for cursor/pagination tokens. Standard base64 contains `+`, `/`, `=` which require URL-encoding in query strings.

## DateTime Serialization

REST API timestamps use RFC 3339 with millisecond precision: `2026-03-23T12:00:00.000Z`. Use a custom serde serializer (`to_rfc3339_ms`) rather than relying on default `DateTime<Utc>` serialization.
