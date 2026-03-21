---
name: minicbor
description: "CRITICAL: Use for CBOR binary serialization and deserialization in Rust. Triggers on:\nminicbor, CBOR, RFC 8949, attestation object, AAGUID, Encoder, Decoder, minicbor::decode,\nminicbor::encode, Encode Decode CborLen derive, no_std serialization, binary encoding,\nCBOR map array tag, cbor attribute, #[n(index)], #[b(index)]"
---

> **Version:** minicbor 2.2 | **Last Updated:** 2026-03-03

# minicbor — Quick Reference

Source: https://docs.rs/minicbor

minicbor is a compact, `no_std`-compatible CBOR (RFC 8949) codec. It does not abstract over
the encoder/decoder (unlike serde), giving you direct control over the wire format.

## Key Patterns

### Top-level functions (simplest API)

```rust
// Encode to a Vec<u8> (requires feature = "std" or "alloc")
let bytes: Vec<u8> = minicbor::to_vec(&value)?;

// Encode into a fixed-size buffer
let mut buf = [0u8; 256];
minicbor::encode(&value, buf.as_mut())?;

// Decode from bytes
let value: MyType = minicbor::decode(&bytes)?;

// Human-readable CBOR diagnostic notation (for debugging)
println!("{}", minicbor::display(&bytes));

// Calculate encoded byte length without encoding
let len = minicbor::len(&value);
```

### Derive macros (recommended for structs and enums)

Every field and variant must have a numeric index with `#[n(idx)]` (copy) or `#[b(idx)]` (borrow).

```toml
minicbor = { version = "2.2", features = ["derive"] }
```

```rust
use minicbor::{Encode, Decode, CborLen};

// Array encoding (default): fields at positional indices, gaps filled with null
#[derive(Encode, Decode, CborLen)]
#[cbor(array)]
struct Point {
    #[n(0)] pub x: f64,
    #[n(1)] pub y: f64,
}

// Map encoding: fields keyed by index, None fields omitted
#[derive(Encode, Decode, CborLen)]
#[cbor(map)]
struct Config {
    #[n(0)] pub host: String,
    #[n(1)] pub port: u16,
    #[n(2)] pub tls: Option<bool>,   // omitted when None in map mode
}

// Fieldless enum: encode only the variant index
#[derive(Encode, Decode, CborLen)]
#[cbor(index_only)]
enum Status {
    #[n(0)] Active,
    #[n(1)] Inactive,
    #[n(2)] Banned,
}

// Enum with data: use #[n] on variants, #[n] on fields
#[derive(Encode, Decode, CborLen)]
enum Message {
    #[n(0)] Ping,
    #[n(1)] Text(#[n(0)] String),
    #[n(2)] Point { #[n(0)] x: f64, #[n(1)] y: f64 },
}
```

### Manual encoding (ad-hoc / partial CBOR)

```rust
use minicbor::Encoder;

let mut buf = Vec::new();
let mut enc = Encoder::new(&mut buf);

// Build a CBOR map manually
enc.map(2)?
    .str("name")?.str("Alice")?
    .str("age")?.u32(30)?;

// Indefinite-length array
enc.begin_array()?
    .u32(1)?.u32(2)?.u32(3)?
    .end()?;

// CBOR tag (e.g., tag 1 = Unix timestamp)
enc.tag(minicbor::data::Tag::new(1))?.u64(1_700_000_000)?;

// Encode a value implementing Encode
enc.encode(&my_value)?;
```

### Manual decoding (ad-hoc / partial CBOR)

```rust
use minicbor::Decoder;

let mut dec = Decoder::new(&bytes);

// Read a definite-length map
let map_len = dec.map()?; // Option<u64> — None = indefinite

// Read fields
let key = dec.str()?;    // borrows from input (&str)
let val = dec.u32()?;

// Read bytes — borrows from input slice (zero-copy)
let raw: &[u8] = dec.bytes()?;

// Skip a value (any type) without decoding it
dec.skip()?;

// Peek at the next data type without consuming
let ty = dec.datatype()?; // minicbor::data::Type enum

// Check ahead without advancing position
let probe = dec.probe();
// ... inspect probe ...

// Current position in input
let pos = dec.position();

// Decode a value implementing Decode
let value: MyType = dec.decode()?;
```

### Parsing CBOR without knowing the schema (tokenizer)

```rust
use minicbor::decode::Tokenizer;

let tokens: Vec<_> = Decoder::new(&bytes).tokens().collect::<Result<_, _>>()?;
for token in &tokens {
    println!("{:?}", token);
}
```

### Borrowing from input (zero-copy)

Use `#[b(idx)]` instead of `#[n(idx)]` to borrow `&str` or `&[u8]` from the decoder's
input slice, avoiding allocation:

```rust
#[derive(Encode, Decode)]
#[cbor(map)]
struct Frame<'a> {
    #[n(0)] pub id: u32,
    #[b(1)] pub payload: &'a [u8],   // borrows from decoder input
    #[b(2)] pub label: &'a str,      // borrows from decoder input
}
```

### Custom encode/decode with attributes

```rust
mod my_codec {
    use minicbor::{Encoder, Decoder, encode::Write, decode::Error};
    use std::time::Duration;

    pub fn encode<W: Write>(d: &Duration, e: &mut Encoder<W>, _ctx: &mut ()) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.u64(d.as_secs())?;
        Ok(())
    }

    pub fn decode(d: &mut Decoder<'_>, _ctx: &mut ()) -> Result<Duration, Error> {
        Ok(Duration::from_secs(d.u64()?))
    }
}

#[derive(Encode, Decode)]
#[cbor(map)]
struct Event {
    #[n(0)] pub name: String,
    #[n(1)]
    #[cbor(with = "my_codec")]
    pub duration: std::time::Duration,
}
```

### Real-world: parse AAGUID from WebAuthn attestation object

```rust
fn parse_aaguid(attestation_object: &[u8]) -> Option<[u8; 16]> {
    let mut dec = minicbor::Decoder::new(attestation_object);
    dec.map().ok()?;                // open the top-level map

    // Skip fmt key + value
    dec.skip().ok()?;
    dec.skip().ok()?;

    // Skip attStmt key + value
    dec.skip().ok()?;
    dec.skip().ok()?;

    let key = dec.str().ok()?;
    if key != "authData" {
        return None;
    }
    let auth_data = dec.bytes().ok()?;  // zero-copy borrow
    if auth_data.len() < 53 {
        return None;
    }

    let mut aaguid = [0u8; 16];
    aaguid.copy_from_slice(&auth_data[37..53]);
    Some(aaguid)
}
```

## API Reference

### Top-level functions

| Function | Signature |
|----------|-----------|
| `encode` | `fn encode<T: Encode<()>>(t: &T, w: impl Write) -> Result<(), Error>` |
| `decode` | `fn decode<'b, T: Decode<'b, ()>>(b: &'b [u8]) -> Result<T, Error>` |
| `to_vec` | `fn to_vec<T: Encode<()>>(t: &T) -> Result<Vec<u8>, Error>` |
| `len` | `fn len<T: CborLen<()>>(t: &T) -> usize` |
| `display` | `fn display(b: &[u8]) -> impl fmt::Display` |

### `Encoder<W>` key methods

| Method | Description |
|--------|-------------|
| `new(writer: W)` | Create encoder |
| `u8/u16/u32/u64(v)` | Encode unsigned integer |
| `i8/i16/i32/i64(v)` | Encode signed integer |
| `f32/f64(v)` | Encode float |
| `bool(v)` | Encode boolean |
| `null()` | Encode null |
| `str(s: &str)` | Encode text string |
| `bytes(b: &[u8])` | Encode byte string |
| `array(len: u64)` | Definite-length array header |
| `begin_array()` | Indefinite-length array |
| `map(len: u64)` | Definite-length map header |
| `begin_map()` | Indefinite-length map |
| `end()` | Close indefinite collection |
| `tag(t: Tag)` | Encode CBOR tag |
| `encode(v)` | Encode any `T: Encode<()>` |
| `encode_with(v, ctx)` | Encode with context |
| `into_writer()` | Consume encoder, return writer |

All methods return `Result<&mut Self, Error>` enabling method chaining.

### `Decoder<'b>` key methods

| Method | Returns |
|--------|---------|
| `new(bytes: &'b [u8])` | Constructor |
| `u8/u16/u32/u64()` | `Result<uN, Error>` |
| `i8/i16/i32/i64()` | `Result<iN, Error>` |
| `f32/f64()` | `Result<fN, Error>` |
| `bool()` | `Result<bool, Error>` |
| `null()` | `Result<(), Error>` |
| `str()` | `Result<&'b str, Error>` — borrows from input |
| `bytes()` | `Result<&'b [u8], Error>` — borrows from input |
| `array()` | `Result<Option<u64>, Error>` — None = indefinite |
| `array_iter::<T>()` | `Result<ArrayIter<T>, Error>` |
| `map()` | `Result<Option<u64>, Error>` — None = indefinite |
| `map_iter::<K, V>()` | `Result<MapIter<K, V>, Error>` |
| `tag()` | `Result<Tag, Error>` |
| `skip()` | `Result<(), Error>` — skip any value |
| `datatype()` | `Result<Type, Error>` — peek type |
| `probe()` | `Probe<'_, 'b>` — non-consuming lookahead |
| `tokens()` | `Tokenizer<'_, 'b>` |
| `position()` | `usize` |
| `set_position(pos)` | `()` |
| `decode::<T>()` | `Result<T, Error>` |

### Derive macro attributes

| Attribute | Level | Meaning |
|-----------|-------|---------|
| `#[cbor(array)]` | struct/enum | Encode as CBOR array (default); gaps = null |
| `#[cbor(map)]` | struct/enum | Encode as CBOR map; `None` fields omitted |
| `#[cbor(index_only)]` | fieldless enum | Encode variant as plain integer, no wrapper |
| `#[cbor(flat)]` | enum | Array with variant index as first element |
| `#[cbor(transparent)]` | single-field struct | Forward encode/decode to inner field |
| `#[n(idx)]` | field/variant | Numeric index; value is copied/owned |
| `#[b(idx)]` | field/variant | Numeric index; value borrows from input |
| `#[cbor(skip)]` | field | Skip encoding; type must impl `Default` for decode |
| `#[cbor(skip_if = "fn")]` | field | Skip encoding when predicate returns true |
| `#[cbor(default)]` | field | Use `Default` when field is absent in decode |
| `#[cbor(encode_with = "path")]` | field | Custom encode function |
| `#[cbor(decode_with = "path")]` | field | Custom decode function |
| `#[cbor(with = "module")]` | field | Module with both `encode` and `decode` fns |
| `#[cbor(tag(N))]` | field | Wrap field in CBOR tag N |
| `#[cbor(context_bound = "T: Trait")]` | struct/enum | Add bound to context generic `C` |

### Trait signatures

```rust
// Encode with a context (use () for no context)
pub trait Encode<C> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C)
        -> Result<(), encode::Error<W::Error>>;
}

// Decode with a context; 'b lifetime borrows from input bytes
pub trait Decode<'b, C>: Sized {
    fn decode(d: &mut Decoder<'b>, ctx: &mut C) -> Result<Self, decode::Error>;
}

// Calculate encoded length
pub trait CborLen<C> {
    fn cbor_len(&self, ctx: &mut C) -> usize;
}
```

### Feature flags

```toml
minicbor = { version = "2.2", features = ["std"] }    # Vec, HashMap, String (implies alloc)
minicbor = { version = "2.2", features = ["alloc"] }  # collections without std
minicbor = { version = "2.2", features = ["derive"] } # proc macros for Encode/Decode/CborLen
```

### CBOR data types (`minicbor::data::Type`)

`U8`, `U16`, `U32`, `U64`, `I8`, `I16`, `I32`, `I64`, `Int`,
`F16`, `F32`, `F64`, `Bytes`, `BytesIndef`, `String`, `StringIndef`,
`Array`, `ArrayIndef`, `Map`, `MapIndef`, `Tag`, `Simple`, `Bool`, `Null`, `Undefined`, `Break`.

## Gotchas

**Every field needs `#[n(idx)]` or `#[b(idx)]`.** There is no automatic numbering. Forgetting
the attribute is a compile error.

**Index gaps in array mode create null entries.** If you use indices 0 and 2 but skip 1, the
encoded array is `[val0, null, val2]`. This can surprise decoders that do not expect nulls.
Use map mode to avoid this.

**`#[b(idx)]` requires a lifetime on the containing struct.** Borrowing from the input (`&str`,
`&[u8]`) means the struct must carry the decoder's input lifetime.

**`decode()` on `str` and `bytes` borrows from the input slice.** The returned `&str` / `&[u8]`
has lifetime `'b` tied to the input buffer. You cannot return it from a function that owns the
buffer without cloning.

**`array()` / `map()` return `Option<u64>`, not the count directly.** `None` means
indefinite-length; you must loop until you see a `Break`. Check `dec.datatype()` for
`Type::Break` or use `array_iter` / `map_iter`.

**`map` mode vs `array` mode affects wire format.** A struct encoded in `map` mode and decoded
in `array` mode (or vice versa) will fail or produce garbage. Both sides must use the same mode.

**`skip()` handles nested structures.** Calling `skip()` once skips the entire next value,
including nested maps, arrays, and tags. Safe to use when iterating over unknown keys.

**No schema evolution in `array` mode.** Adding a field at a new index that breaks existing
readers is safe in `map` mode (unknown keys are ignored by default), but in `array` mode the
length changes which can break fixed-size assumptions.

**Context type `C` defaults to `()`.** When using `Encode<()>` / `Decode<'b, ()>` you pass `&mut ()`.
Top-level functions (`to_vec`, `decode`) use `()` context automatically. Custom contexts are
useful for passing configuration or shared state to codecs.

## Tips

**Use `map` mode for extensible formats.** Map mode omits `None` fields and is robust to adding
new optional fields. Array mode is more compact but fragile for schema evolution.

**Use `#[cbor(transparent)]` for newtypes.** A single-field wrapper struct can forward
encoding/decoding directly to the inner type without manual `impl` blocks.

**`decoder.probe()`** lets you inspect the next CBOR type without consuming it. Useful for
discriminating between map and array formats in a unified decoder.

**`minicbor::display(&bytes)`** prints CBOR in diagnostic notation (RFC 8949 §8). Use it in
tests or debugging to visually inspect encoded data.

**`CborLen` enables pre-allocation.** If you need to allocate a buffer before encoding, derive
`CborLen` and call `minicbor::len(&value)` to get the exact required size.

**For WebAuthn attestation parsing**, use the `Decoder` directly (no derive needed). The
`authData` field is a raw byte string at a known position in the map; use `skip()` to bypass
fields you do not need, then `bytes()` for the authData byte string (zero-copy borrow).
