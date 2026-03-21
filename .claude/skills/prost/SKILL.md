---
name: prost
description: "CRITICAL: Use for Protocol Buffers serialization and code generation in Rust. Triggers on:\nprost, protobuf, proto3, proto2, Message trait, encode, decode, prost-build, prost-types, build.rs proto, FileDescriptorSet, include_proto, generated types, .proto compilation"
---

> **Version:** prost 0.14 | **Last Updated:** 2026-03-03

# prost — Quick Reference

Source: https://docs.rs/prost

## Key Patterns

### Cargo.toml setup

```toml
[dependencies]
prost = "0.14"
prost-types = "0.14"      # for google.protobuf.* well-known types

[build-dependencies]
prost-build = "0.14"
```

### build.rs — basic compilation

```rust
// build.rs
fn main() -> std::io::Result<()> {
    prost_build::compile_protos(
        &["proto/items.proto", "proto/auth.proto"],
        &["proto/"],          // include paths (for imports)
    )?;
    Ok(())
}
```

### Including generated code

Generated files land in `OUT_DIR`. Include them with a module declaration:

```rust
// src/lib.rs or src/proto.rs
pub mod snazzy {
    pub mod items {
        // File is named after the protobuf package: "snazzy.items" → snazzy.items.rs
        include!(concat!(env!("OUT_DIR"), "/snazzy.items.rs"));
    }
}

// Flat package (no package statement in .proto):
pub mod my_service {
    include!(concat!(env!("OUT_DIR"), "/my_service.rs"));
}
```

### Using generated types

```rust
use prost::Message;
use crate::proto::snazzy::items::Item;

// Encode to bytes
let item = Item { name: "Widget".into(), id: 42, ..Default::default() };
let bytes: Vec<u8> = item.encode_to_vec();

// Decode from bytes
let decoded = Item::decode(bytes.as_ref())?;
assert_eq!(decoded.id, 42);

// Encode/decode with length delimiter (for streaming framing)
let mut buf = Vec::new();
item.encode_length_delimited(&mut buf)?;
let decoded = Item::decode_length_delimited(buf.as_ref())?;

// Merge partial messages
let mut target = Item::default();
target.merge(bytes.as_ref())?;
```

### prost-build Config — advanced options

```rust
// build.rs
fn main() -> std::io::Result<()> {
    let mut config = prost_build::Config::new();

    // Add derive attributes to generated types
    config.type_attribute("snazzy.items.Item", "#[derive(serde::Serialize, serde::Deserialize)]");
    config.type_attribute(".", "#[derive(Hash)]");  // "." applies to all types

    // Add attributes to individual fields
    config.field_attribute("snazzy.items.Item.name", "#[serde(rename = \"item_name\")]");

    // Use BTreeMap instead of HashMap for map fields
    config.btree_map(["."]);

    // Change bytes fields to Bytes instead of Vec<u8>
    config.bytes(["snazzy.items.Item.data"]);

    // Custom service generator (for non-tonic RPC)
    // config.service_generator(Box::new(MyServiceGenerator));

    config.compile_protos(&["proto/items.proto"], &["proto/"])?;
    Ok(())
}
```

### Using with FileDescriptorSet (protox integration)

```rust
// build.rs — when using protox instead of protoc
fn main() -> std::io::Result<()> {
    let file_descriptor_set = protox::compile(["proto/items.proto"], ["proto/"])?;

    prost_build::Config::new()
        .type_attribute(".", "#[derive(serde::Serialize)]")
        .compile_fds(file_descriptor_set)?;

    Ok(())
}
```

### Oneof fields

A `oneof` in proto generates a nested enum:

```proto
message Foo {
  oneof widget {
    int32 quux = 1;
    string bar = 2;
  }
}
```

```rust
// Generated:
pub struct Foo {
    pub widget: Option<foo::Widget>,
}
pub mod foo {
    pub enum Widget {
        Quux(i32),
        Bar(String),
    }
}

// Usage:
match foo.widget {
    Some(foo::Widget::Quux(n)) => { /* ... */ }
    Some(foo::Widget::Bar(s)) => { /* ... */ }
    None => { /* ... */ }
}
```

### Enum fields

Proto enums compile to `i32` on the wire with a companion Rust enum:

```proto
enum Status { UNKNOWN = 0; ACTIVE = 1; INACTIVE = 2; }
message User { Status status = 1; }
```

```rust
// Generated field:
pub struct User {
    pub status: i32,  // raw wire value
}

// Use the generated enum + typed accessor:
use proto::Status;
let status = Status::try_from(user.status).unwrap_or(Status::Unknown);

// Or use the typed setter:
user.set_status(Status::Active);
let typed: Status = user.status(); // returns Status or default
```

### prost-types: well-known types

```rust
use prost_types::{Timestamp, Duration, Any};

// Convert from std::time::SystemTime
let ts = Timestamp::from(std::time::SystemTime::now());

// Convert to/from chrono (requires prost-types feature or manual impl)
let seconds = ts.seconds;
```

## API Reference

### `prost::Message` trait

| Method | Description |
|--------|-------------|
| `encode(&self, buf: &mut impl BufMut) -> Result<(), EncodeError>` | Encode into existing buffer |
| `encode_to_vec(&self) -> Vec<u8>` | Encode into new Vec |
| `encode_length_delimited(&self, buf)` | Encode with varint length prefix |
| `encode_length_delimited_to_vec(&self) -> Vec<u8>` | Encode with length prefix into new Vec |
| `decode(buf: impl Buf) -> Result<Self, DecodeError>` | Decode from buffer (consumes all) |
| `decode_length_delimited(buf)` | Decode with length prefix |
| `merge(&mut self, buf: impl Buf) -> Result<(), DecodeError>` | Merge bytes into self |
| `encoded_len(&self) -> usize` | Byte size of encoded form |
| `clear(&mut self)` | Reset all fields to defaults |

### `prost_build::Config` key methods

| Method | Description |
|--------|-------------|
| `type_attribute(path, attr)` | Add Rust attribute to generated type |
| `field_attribute(path, attr)` | Add Rust attribute to generated field |
| `bytes(paths)` | Use `bytes::Bytes` instead of `Vec<u8>` |
| `btree_map(paths)` | Use `BTreeMap` instead of `HashMap` |
| `out_dir(path)` | Override output directory (default: `OUT_DIR`) |
| `compile_protos(protos, includes)` | Compile .proto files |
| `compile_fds(FileDescriptorSet)` | Compile pre-parsed descriptor |
| `service_generator(Box<dyn ServiceGenerator>)` | Custom RPC code generation |

### Type mapping (proto3 → Rust)

| Protobuf type | Rust type |
|---------------|-----------|
| `double` | `f64` |
| `float` | `f32` |
| `int32` / `sint32` | `i32` |
| `int64` / `sint64` | `i64` |
| `uint32` | `u32` |
| `uint64` | `u64` |
| `bool` | `bool` |
| `string` | `String` |
| `bytes` | `Vec<u8>` (or `Bytes` with config) |
| `optional T` | `Option<T>` |
| `repeated T` | `Vec<T>` |
| `map<K, V>` | `HashMap<K, V>` (or `BTreeMap`) |
| enum | `i32` + companion Rust enum |
| message | Nested Rust struct |

## Gotchas

1. **`protoc` is required** (v0.11+): `prost-build` shells out to `protoc`. It must be on `PATH` or set via `PROTOC` env var. Use `protox` crate to avoid this dependency entirely.

2. **Proto3 scalar defaults**: In proto3, scalar fields missing from the wire are decoded as their zero value (`0`, `""`, `false`). There is no way to distinguish "field absent" from "field set to zero" without using `optional T` → `Option<T>`.

3. **Enum fields are `i32`**: Proto enums compile to raw `i32` on the struct. Use `MyEnum::try_from(value)` or the typed accessor `.my_field()` method. Unknown values round-trip safely — they are preserved.

4. **Module paths must match proto package**: If your `.proto` says `package foo.bar;`, the generated file is `foo.bar.rs` and your `include!` macro path must match. A flat proto (no package) generates `<message_name>.rs`.

5. **`encode()` requires sufficient buffer capacity**: Returns `EncodeError` if the buffer is full. Use `encode_to_vec()` to avoid managing capacity.

6. **Recursive message types**: prost automatically wraps self-referential message fields in `Box<T>` to avoid infinite-size types.

7. **`merge()` vs `decode()`**: `decode()` creates a fresh value. `merge()` overlays bytes onto an existing value — repeated fields are appended, scalar fields are overwritten. Use `clear()` before `merge()` if you want fresh decode semantics.

8. **`prost-types` timestamp precision**: `Timestamp` stores `seconds: i64` and `nanos: i32`. Converting to/from `std::time::SystemTime` via `From`/`TryFrom` is provided, but chrono conversion requires manual impl or a helper crate.

9. **`type_attribute(".", ...)` applies globally**: The path `"."` means "all types in all packages." Use a specific proto path like `"mypackage.MyMessage"` to target a single type.

10. **Build cache**: `prost-build` respects Cargo's rebuild logic via `println!("cargo:rerun-if-changed=proto/")`. Add this in `build.rs` to avoid unnecessary recompilation.

## Tips

- Add `println!("cargo:rerun-if-changed=proto/");` in `build.rs` so Cargo only reruns proto compilation when `.proto` files change.
- Use `protox` instead of `protoc` for a zero-external-dependency build: `protox::compile(...)` returns a `FileDescriptorSet` compatible with `prost_build::Config::compile_fds`.
- `config.bytes(["."])` makes all `bytes` fields use `bytes::Bytes` (zero-copy slicing) instead of `Vec<u8>` — useful when working with tonic which already depends on `bytes`.
- For gRPC, prefer `tonic-build` (which wraps `prost-build`) over raw `prost-build` — it generates both message types and service stubs in one step.
- `#[allow(clippy::all)]` on generated modules suppresses clippy warnings from proto-generated code that you don't control.
