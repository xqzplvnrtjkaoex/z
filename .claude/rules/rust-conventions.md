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
