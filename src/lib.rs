//! Data format to storing and loading JSON data.
//!
//! The first byte tells us how the data is encoded.
//! The following bytes are base64, whose content is determined
//! by the first byte.
//!
//! However, the layers are always `base64 -> <something> -> JSON`.

use std::io::Cursor;

use anyhow::bail;
use base64::{Engine, prelude::BASE64_STANDARD};
use serde_json::Value;

pub fn decode(src: &[u8]) -> anyhow::Result<Value> {
    let Some((transport, mut encoded_data)) = src.split_first() else {
        bail!("Data is empty");
    };
    // Just a quick effort of cleaning up whitespace from copy/pasting.
    // This is not meant to perfectly sanitize base64 strings.
    if let Some(end) = encoded_data.iter().position(|v| *v < b'+' || *v > b'z') {
        encoded_data = &encoded_data[..end];
    }
    let compressed_data = BASE64_STANDARD.decode(encoded_data)?;
    let json_string = match *transport {
        b'1' => {
            // 1 is for ZSTD
            zstd::decode_all(Cursor::new(compressed_data))?
        }
        b'2' => {
            // 2 is for Plain
            compressed_data
        }
        _ => {
            bail!("Unsupported format ID: {}", transport);
        }
    };
    Ok(serde_json::from_slice(&json_string)?)
}

pub fn encode(value: Value) -> anyhow::Result<String> {
    let json_string = value.to_string();
    let zstd_data = zstd::encode_all(Cursor::new(json_string.as_bytes()), 19)?;
    let (prefix, compressed_data) =
        pick_shortest([("1", zstd_data.as_slice()), ("2", json_string.as_bytes())]);
    let mut transport = String::with_capacity(1 + compressed_data.len().div_ceil(3) * 4);
    transport += prefix;
    BASE64_STANDARD.encode_string(compressed_data, &mut transport);
    Ok(transport)
}

fn pick_shortest<'a>(
    options: impl IntoIterator<Item = (&'a str, &'a [u8])>,
) -> (&'a str, &'a [u8]) {
    options
        .into_iter()
        .reduce(|a, b| if a.1.len() > b.1.len() { b } else { a })
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn encode_hello_world() {
        let value = json!({ "hello": "world" });
        let blob = encode(value).unwrap();
        assert_eq!("2eyJoZWxsbyI6IndvcmxkIn0=", blob);
    }

    #[test]
    fn decode_hello_world_zstd() {
        let blob = "1KLUv/QBoiQAAeyJoZWxsbyI6IndvcmxkIn0=";
        let value = decode(blob.as_bytes()).unwrap();
        assert_eq!(json!({ "hello": "world" }), value);
    }

    #[test]
    fn decode_hello_world_plain() {
        let blob = "2eyJoZWxsbyI6IndvcmxkIn0=";
        let value = decode(blob.as_bytes()).unwrap();
        assert_eq!(json!({ "hello": "world" }), value);
    }

    #[test]
    fn decode_hello_world_with_garbage() {
        let blob = "1KLUv/QBoiQAAeyJoZWxsbyI6IndvcmxkIn0=&1312";
        let value = decode(blob.as_bytes()).unwrap();
        assert_eq!(json!({ "hello": "world" }), value);
    }
}
