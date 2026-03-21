---
name: serde-derive
description: |
  CRITICAL: Use for serde derive attributes and patterns. Triggers on:
  serde, Serialize, Deserialize, serde attribute, rename_all,
  serde(skip), serde(default), serde(untagged), serde(flatten),
  serde(tag), serde_json, from_str, to_string, Value,
  custom deserializer, serde(with), serde(rename)
---

# Serde Derive Skill

> **Version:** serde 1.0 + serde_json 1.0 | **Last Updated:** 2026-03-01
>
> Check for updates: https://crates.io/crates/serde

You are an expert at the Rust `serde` and `serde_json` crates. Help users by:
- **Writing code**: Generate derive attributes, custom serialization, JSON handling
- **Answering questions**: Explain attribute behavior, enum representations, error handling

## Documentation

- `./references/attributes.md` — All serde derive attributes with examples
- `./references/json.md` — serde_json API, Value, raw JSON, streaming

## Key Patterns

### Container Attributes

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]     // field names → camelCase
struct User {
    user_name: String,                  // serializes as "userName"
    email_address: String,              // serializes as "emailAddress"
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]    // field names → kebab-case
struct BookKind { ... }
```

### Enum Representations

```rust
// Externally tagged (default): {"variant": data}
#[derive(Serialize, Deserialize)]
enum Animal { Cat(String), Dog { name: String } }

// Internally tagged: {"type": "Cat", ...}
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Event { Click { x: i32 }, Scroll { delta: i32 } }

// Adjacently tagged: {"t": "Cat", "c": data}
#[derive(Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum Message { Text(String), Image { url: String } }

// Untagged: tries each variant in order
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum StringOrInt { Str(String), Int(i64) }
```

### Common Field Attributes

```rust
#[derive(Serialize, Deserialize)]
struct Config {
    #[serde(default)]                    // use Default if missing
    retries: u32,

    #[serde(skip)]                       // skip both ser/de
    internal: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]  // omit if None
    nickname: Option<String>,

    #[serde(rename = "type")]            // rename single field
    kind: String,

    #[serde(flatten)]                    // inline nested struct
    metadata: Metadata,

    #[serde(with = "serde_str")]         // custom ser/de module
    timestamp: DateTime<Utc>,

    #[serde(deserialize_with = "de_fn")] // custom deserialize only
    special: MyType,
}
```

## Attribute Reference Table

| Attribute | Level | Description |
|-----------|-------|-------------|
| `rename_all = "..."` | Container | Rename all fields/variants |
| `rename = "..."` | Field/Variant | Rename single field |
| `tag = "..."` | Container (enum) | Internal tag field name |
| `content = "..."` | Container (enum) | Adjacent tag content field |
| `untagged` | Container (enum) | No tag, try variants in order |
| `default` | Container/Field | Use Default::default() if missing |
| `skip` | Field | Skip serialization and deserialization |
| `skip_serializing` | Field | Skip during serialization only |
| `skip_deserializing` | Field | Skip during deserialization only |
| `skip_serializing_if` | Field | Conditionally skip serialization |
| `flatten` | Field | Inline nested struct fields |
| `with = "mod"` | Field | Custom ser/de module |
| `serialize_with` | Field | Custom serialize function |
| `deserialize_with` | Field | Custom deserialize function |
| `deny_unknown_fields` | Container | Error on unexpected fields |
| `transparent` | Container | Delegate to inner type |
| `from = "Type"` | Container | Deserialize via `From<Type>` |
| `into = "Type"` | Container | Serialize via `Into<Type>` |

### rename_all Values

| Value | Example |
|-------|---------|
| `"camelCase"` | `fieldName` |
| `"snake_case"` | `field_name` |
| `"PascalCase"` | `FieldName` |
| `"SCREAMING_SNAKE_CASE"` | `FIELD_NAME` |
| `"kebab-case"` | `field-name` |
| `"lowercase"` | `fieldname` |
| `"UPPERCASE"` | `FIELDNAME` |

## When Writing Code

1. `#[serde(rename_all)]` on container, `#[serde(rename)]` on individual fields
2. `Option<T>` fields are `None` by default when missing — no `#[serde(default)]` needed
3. `#[serde(skip_serializing_if = "Option::is_none")]` to omit null fields from JSON
4. `#[serde(untagged)]` tries variants top-to-bottom — put most specific first
5. `#[serde(flatten)]` on a `HashMap<String, Value>` catches all unknown fields
6. `#[serde(deny_unknown_fields)]` rejects unexpected JSON keys

## When Answering Questions

1. Default enum representation is externally tagged: `{"Variant": data}`
2. `#[serde(tag = "type")]` only works for enums with struct/unit variants (not tuple)
3. `#[serde(default)]` on container = all fields get `Default::default()` if missing
4. `#[serde(default)]` on field = only that field gets `Default::default()`
5. `#[serde(transparent)]` makes a newtype serialize as its inner type
