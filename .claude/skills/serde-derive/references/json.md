# serde_json API Reference

> serde_json 1.0 | Source: https://docs.rs/serde_json/1.0

## Serialize / Deserialize

```rust
use serde_json;

// Struct → JSON string
let json = serde_json::to_string(&my_struct)?;
let json_pretty = serde_json::to_string_pretty(&my_struct)?;

// JSON string → struct
let parsed: MyStruct = serde_json::from_str(&json)?;

// Struct → JSON bytes
let bytes = serde_json::to_vec(&my_struct)?;

// JSON bytes → struct
let parsed: MyStruct = serde_json::from_slice(&bytes)?;

// Struct → writer (file, socket, etc.)
serde_json::to_writer(writer, &my_struct)?;
serde_json::to_writer_pretty(writer, &my_struct)?;

// Reader → struct
let parsed: MyStruct = serde_json::from_reader(reader)?;
```

## Value (Dynamic JSON)

```rust
use serde_json::{Value, json};

// Parse to dynamic Value
let v: Value = serde_json::from_str(r#"{"name":"alice","age":30}"#)?;

// Access fields
let name = v["name"].as_str();       // Option<&str>
let age = v["age"].as_i64();         // Option<i64>
let missing = v["missing"].is_null(); // true (missing fields are Null)

// json! macro for construction
let v = json!({
    "name": "alice",
    "age": 30,
    "tags": ["admin", "user"],
    "active": true,
    "address": null
});
```

### Value Variants

| Variant | Type | Access method |
|---------|------|---------------|
| `Value::Null` | null | `.is_null()` |
| `Value::Bool(b)` | boolean | `.as_bool()` |
| `Value::Number(n)` | number | `.as_i64()`, `.as_u64()`, `.as_f64()` |
| `Value::String(s)` | string | `.as_str()` |
| `Value::Array(v)` | array | `.as_array()` → `Option<&Vec<Value>>` |
| `Value::Object(m)` | object | `.as_object()` → `Option<&Map<String, Value>>` |

### Value ↔ Typed

```rust
// Value → typed struct
let user: User = serde_json::from_value(value)?;

// Typed struct → Value
let value: Value = serde_json::to_value(&user)?;
```

## Map Type

```rust
use serde_json::Map;

let mut map = Map::new();
map.insert("key".to_string(), json!("value"));

// Iterate
for (key, value) in &map {
    println!("{}: {}", key, value);
}
```

## Number Type

```rust
use serde_json::Number;

let n = Number::from(42i64);
let f = Number::from_f64(3.14).unwrap(); // None for NaN/Inf

n.as_i64();  // Some(42)
n.as_f64();  // Some(42.0)
n.is_i64();  // true
```

## Raw Value (Defer Parsing)

```rust
use serde_json::value::RawValue;

#[derive(Deserialize)]
struct Envelope {
    #[serde(rename = "type")]
    kind: String,
    #[serde(borrow)]
    payload: &'de RawValue,  // kept as raw JSON string
}

// Parse envelope without parsing payload
let env: Envelope = serde_json::from_str(json)?;

// Later, parse payload based on type
match env.kind.as_str() {
    "user" => {
        let user: User = serde_json::from_str(env.payload.get())?;
    }
    _ => {}
}
```

## Error Handling

```rust
use serde_json::Error;

match serde_json::from_str::<User>(json) {
    Ok(user) => { /* success */ }
    Err(e) => {
        // e.line() and e.column() for parse position
        // e.classify() → Category (Io, Syntax, Data, Eof)
        eprintln!("Parse error at {}:{}: {}", e.line(), e.column(), e);
    }
}
```

## Streaming (Large Files)

```rust
use serde_json::Deserializer;

// Stream array elements
let reader = std::io::BufReader::new(file);
let stream = Deserializer::from_reader(reader).into_iter::<Value>();

for item in stream {
    let value = item?;
    // process each top-level value
}
```

## Common Functions

| Function | Description |
|----------|-------------|
| `to_string(v)` | Serialize to JSON string |
| `to_string_pretty(v)` | Serialize to pretty JSON |
| `to_vec(v)` | Serialize to bytes |
| `to_writer(w, v)` | Serialize to writer |
| `from_str(s)` | Deserialize from string |
| `from_slice(b)` | Deserialize from bytes |
| `from_reader(r)` | Deserialize from reader |
| `from_value(v)` | Value → typed |
| `to_value(v)` | Typed → Value |
| `json!({...})` | Construct Value literal |
