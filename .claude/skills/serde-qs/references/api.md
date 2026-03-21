# serde_qs API Reference

> serde_qs 1.0 | Source: https://docs.rs/serde_qs/1.0

## Basic Usage

### Deserialization

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Params {
    page: Option<u32>,
    per_page: Option<u32>,
    sort: Option<String>,
}

// Simple flat query
let params: Params = serde_qs::from_str("page=1&per_page=25&sort=name")?;
```

### Serialization

```rust
use serde::Serialize;

#[derive(Serialize)]
struct Params {
    page: u32,
    sort: String,
}

let qs = serde_qs::to_string(&Params { page: 1, sort: "name".into() })?;
// "page=1&sort=name"
```

## Nested Structs (Bracket Notation)

The main advantage of `serde_qs` over standard query parsers.

```rust
#[derive(Debug, Deserialize)]
struct Query {
    filter: Filter,
    sort: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Filter {
    name: String,
    min_age: Option<u32>,
}

// Parses: "filter[name]=alice&filter[min_age]=18&sort=desc"
let q: Query = serde_qs::from_str(
    "filter[name]=alice&filter[min_age]=18&sort=desc"
)?;
assert_eq!(q.filter.name, "alice");
assert_eq!(q.filter.min_age, Some(18));
```

## Arrays / Vectors

```rust
#[derive(Debug, Deserialize)]
struct Query {
    ids: Vec<u32>,
}

// Indexed: "ids[0]=1&ids[1]=2&ids[2]=3"
let q: Query = serde_qs::from_str("ids[0]=1&ids[1]=2&ids[2]=3")?;

// Also works without indices: "ids[]=1&ids[]=2&ids[]=3"
```

## Enums

```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Deserialize)]
struct Query {
    order: SortOrder,
}

// Parses: "order=asc"
let q: Query = serde_qs::from_str("order=asc")?;
```

## Config

Custom configuration for depth limits and strict mode.

```rust
use serde_qs::Config;

let config = Config::new(10, false);
//                       ^    ^
//                       |    strict mode (reject unknown fields)
//                       max nesting depth (default: 5)

let params: MyStruct = config.deserialize_str("nested[deep][key]=value")?;
```

### Config Methods

| Method | Description |
|--------|-------------|
| `Config::new(depth, strict)` | Create config with max depth and strict mode |
| `config.deserialize_str(s)` | Deserialize with this config |
| `config.deserialize_bytes(b)` | Deserialize from bytes with this config |

## From Bytes

```rust
let bytes = b"page=1&sort=name";
let params: Params = serde_qs::from_bytes(bytes)?;
```

## Error Handling

```rust
use serde_qs::Error;

match serde_qs::from_str::<Params>(query) {
    Ok(params) => { /* use params */ }
    Err(e) => {
        // e implements Display with descriptive message
        eprintln!("failed to parse query: {e}");
    }
}
```

## Integration with axum

`serde_qs` does not provide its own axum extractor. Use `RawQuery` and parse manually:

```rust
use axum::extract::RawQuery;

async fn handler(RawQuery(query): RawQuery) -> Result<Json<Data>, StatusCode> {
    let params: MyParams = query
        .as_deref()
        .map(serde_qs::from_str)
        .transpose()
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .unwrap_or_default();

    Ok(Json(process(params)))
}
```

Or create a custom extractor:

```rust
use axum::extract::FromRequestParts;
use http::request::Parts;

struct Qs<T>(pub T);

impl<S, T> FromRequestParts<S> for Qs<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let query = parts.uri.query().unwrap_or_default();
        serde_qs::from_str(query)
            .map(Qs)
            .map_err(|_| StatusCode::BAD_REQUEST)
    }
}
```
