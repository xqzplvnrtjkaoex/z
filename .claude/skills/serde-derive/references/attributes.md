# Serde Derive Attributes Reference

> serde 1.0 | Source: https://serde.rs/attributes.html

## Container Attributes

Applied to structs or enums with `#[serde(...)]`.

### rename_all

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct User {
    first_name: String,  // → "firstName"
    last_name: String,   // → "lastName"
}
```

Values: `"lowercase"`, `"UPPERCASE"`, `"PascalCase"`, `"camelCase"`,
`"snake_case"`, `"SCREAMING_SNAKE_CASE"`, `"kebab-case"`,
`"SCREAMING-KEBAB-CASE"`.

Can also apply separately: `#[serde(rename_all(serialize = "camelCase", deserialize = "PascalCase"))]`

### deny_unknown_fields

```rust
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Strict {
    name: String,
}
// Error if JSON has extra fields
```

### default

```rust
#[derive(Deserialize)]
#[serde(default)]
struct Config {
    retries: u32,     // uses Config::default() for all missing fields
    timeout: u64,
}
```

### transparent

```rust
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
struct Wrapper(String);
// Serializes as plain String, not {"0": "..."}
```

### from / into / try_from

```rust
#[derive(Deserialize)]
#[serde(from = "RawData")]
struct Processed { /* ... */ }

impl From<RawData> for Processed { /* ... */ }
// Deserializes as RawData first, then converts
```

### bound

Override trait bounds for generic types:

```rust
#[derive(Serialize)]
#[serde(bound(serialize = "T: Serialize + Clone"))]
struct Wrapper<T> { inner: T }
```

## Enum Representations

### Externally Tagged (default)

```rust
#[derive(Serialize, Deserialize)]
enum E { A(u32), B { x: i32 } }
// {"A": 42} or {"B": {"x": 1}}
```

### Internally Tagged

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum E {
    A { value: u32 },
    B { x: i32 },
}
// {"type": "A", "value": 42}
```

Only works with struct variants and unit variants. Not with tuple variants or newtype variants containing non-struct types.

### Adjacently Tagged

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum E {
    A(u32),
    B { x: i32 },
}
// {"t": "A", "c": 42}
```

### Untagged

```rust
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum E {
    Int(i64),
    Float(f64),
    Str(String),
}
// Tries each variant in order; 42 → Int, 3.14 → Float, "hi" → Str
```

Put most specific variants first — matching stops at first success.

## Field Attributes

### skip

```rust
#[derive(Serialize, Deserialize)]
struct S {
    #[serde(skip)]                        // skip both
    internal: u32,

    #[serde(skip_serializing)]            // skip ser only
    write_only: String,

    #[serde(skip_deserializing)]          // skip de only
    read_only: String,
}
```

### skip_serializing_if

```rust
#[serde(skip_serializing_if = "Option::is_none")]
nickname: Option<String>,

#[serde(skip_serializing_if = "Vec::is_empty")]
tags: Vec<String>,

#[serde(skip_serializing_if = "is_default")]
count: u32,

// Helper function
fn is_default<T: Default + PartialEq>(v: &T) -> bool {
    *v == T::default()
}
```

### default

```rust
#[serde(default)]                  // use Default::default()
retries: u32,

#[serde(default = "default_port")] // use custom function
port: u16,

fn default_port() -> u16 { 8080 }
```

### rename

```rust
#[serde(rename = "type")]          // rename to reserved keyword
kind: String,

#[serde(rename(serialize = "output", deserialize = "input"))]
data: String,
```

### flatten

```rust
#[derive(Serialize, Deserialize)]
struct Request {
    id: u32,
    #[serde(flatten)]
    metadata: Metadata,  // fields inlined into Request
}

// Also works with HashMap to capture unknown fields
#[serde(flatten)]
extra: HashMap<String, serde_json::Value>,
```

### with / serialize_with / deserialize_with

```rust
// Use a module with serialize/deserialize functions
#[serde(with = "chrono::serde::ts_seconds")]
timestamp: DateTime<Utc>,

// Custom serialize function only
#[serde(serialize_with = "serialize_uppercase")]
name: String,

fn serialize_uppercase<S>(val: &str, s: S) -> Result<S::Ok, S::Error>
where S: Serializer {
    s.serialize_str(&val.to_uppercase())
}

// Custom deserialize function only
#[serde(deserialize_with = "deserialize_from_str")]
port: u16,

fn deserialize_from_str<'de, D>(d: D) -> Result<u16, D::Error>
where D: Deserializer<'de> {
    let s = String::deserialize(d)?;
    s.parse().map_err(serde::de::Error::custom)
}
```

### alias

```rust
#[serde(alias = "user_name")]  // accept "user_name" OR "username"
username: String,
```

## Variant Attributes

```rust
#[derive(Serialize, Deserialize)]
enum Status {
    #[serde(rename = "ok")]
    Success,

    #[serde(skip)]
    Internal,

    #[serde(alias = "err")]
    Error(String),

    #[serde(other)]       // catch-all for unknown variants (unit variant only)
    Unknown,
}
```

## Common Patterns

### Optional fields with default

```rust
#[derive(Deserialize)]
struct Query {
    #[serde(default = "default_page")]
    page: u32,
    #[serde(default = "default_per_page")]
    per_page: u32,
}

fn default_page() -> u32 { 1 }
fn default_per_page() -> u32 { 25 }
```

### String ↔ enum (Display + FromStr)

```rust
use serde_with::{SerializeDisplay, DeserializeFromStr};

#[derive(SerializeDisplay, DeserializeFromStr)]
enum Color { Red, Blue, Green }

impl Display for Color { /* ... */ }
impl FromStr for Color { /* ... */ }
// Serializes as "Red", "Blue", "Green"
```

### Deserialize from multiple formats

```rust
#[derive(Deserialize)]
#[serde(untagged)]
enum FlexibleId {
    Num(u64),
    Str(String),
}
```
