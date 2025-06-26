# jsot

*A tiny helper for shipping generic data in a short string.*

---

## What is this?

The crate lets you **encode** any `serde_json::Value` into a compact text payload and **decode** it back.

The string is prefixed with a Transport ID to allow for future compression alternatives.

---

## Quick start

Add to `Cargo.toml`:

```toml
[dependencies]
jsot = "0.1"
serde_json = "1"
```

### Encoding

```rust
use jsot::encode;
use serde_json::json;

let value = json!({ "hello": "world" });
let blob = encode(value)?;
assert_eq!("0KLUv/QBoiQAAeyJoZWxsbyI6IndvcmxkIn0=", blob);
```

### Decoding

```rust
use jsot::decode;

let blob = "0KLUv/QBoiQAAeyJoZWxsbyI6IndvcmxkIn0=";
let value = decode(blob.as_bytes())?;
assert_eq!(json!({ "hello": "world" }), value);
```

---

## Format specification

The data format consists of a transport ID byte, followed by Base64 data.

| Transport ID | Compression | Wrapper |
|--------------|-------------|---------|
| `'0'`        | **Zstd**    | Base64  |

For now the only transport ID is `'0'`. All other IDs are reserved for future
considerations.

JSON data is encoded as `JSON -> Transport (e.g. Zstd) -> Base64`.

---

## License

Dual-licensed under **MIT OR Apache-2.0**.

---

## Contributing

* **Heads-up:** this repository is published **for reference only**.  
* We are **not accepting pull requests, issues, or feature requests** at this time.

## Why is the code public?

* To document the wire / file format in a runnable, audited form.
* To make it easy for other teams to port the encoder/decoder to their own language or stack.
