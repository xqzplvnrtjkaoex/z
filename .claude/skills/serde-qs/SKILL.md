---
name: serde-qs
description: |
  CRITICAL: Use for query string serialization/deserialization. Triggers on:
  serde_qs, query string parsing, nested query params, bracket notation,
  URL query deserialization, query string format
---

# serde_qs Skill

> **Version:** serde_qs 1.0 | **Last Updated:** 2026-03-01
>
> Check for updates: https://crates.io/crates/serde_qs

You are an expert at the Rust `serde_qs` crate. Help users by:
- **Writing code**: Generate query string parsing and serialization code
- **Answering questions**: Explain bracket notation, nested structs, configuration

## Documentation

- `./references/api.md` — Deserialization, serialization, configuration, nested types

## Key Patterns

### Basic Deserialization

```rust
#[derive(Debug, Deserialize)]
struct Filters {
    page: Option<u32>,
    per_page: Option<u32>,
    sort: Option<String>,
}

// Parses: "page=1&per_page=25&sort=name"
let filters: Filters = serde_qs::from_str(query_string)?;
```

### Nested Structs (Bracket Notation)

```rust
#[derive(Debug, Deserialize)]
struct Query {
    filter: Filter,
}

#[derive(Debug, Deserialize)]
struct Filter {
    name: String,
    age: u32,
}

// Parses: "filter[name]=alice&filter[age]=30"
let q: Query = serde_qs::from_str("filter[name]=alice&filter[age]=30")?;
```

### With axum (Manual Parsing)

```rust
use axum::extract::RawQuery;

async fn handler(RawQuery(query): RawQuery) -> Result<Json<Data>, StatusCode> {
    let params: MyParams = query
        .as_deref()
        .map(serde_qs::from_str)
        .transpose()
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .unwrap_or_default();
    // ...
}
```

## API Reference Table

| Function | Description |
|----------|-------------|
| `serde_qs::from_str(s)` | Deserialize from query string |
| `serde_qs::from_bytes(b)` | Deserialize from byte slice |
| `serde_qs::to_string(v)` | Serialize to query string |
| `Config::new(depth, strict)` | Custom config with max nesting depth |
| `config.deserialize_str(s)` | Deserialize with custom config |

## When Writing Code

1. Use `serde_qs::from_str` for bracket-notation queries (`filter[name]=x`)
2. Standard `serde::Deserialize` works — no special derives needed
3. For flat queries without nesting, axum's built-in `Query<T>` is simpler
4. `Option<T>` fields handle missing parameters gracefully
5. `#[serde(default)]` provides defaults for missing fields

## When Answering Questions

1. `serde_qs` vs axum `Query`: serde_qs handles nested bracket notation; axum's Query is flat only
2. Format: `key[subkey]=value` for nested, `key[0]=a&key[1]=b` for arrays
3. Max nesting depth defaults to 5 — use `Config` to change
4. Enum variants use the variant name as the value string
