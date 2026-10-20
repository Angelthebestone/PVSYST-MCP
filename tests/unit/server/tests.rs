//! Unit test for the internal tool-result encoder: a base64 PNG payload becomes
//! an `image` content item, and the internal marker key never leaks into the
//! text.
//!
//! Protocol-level coverage (handshake, JSON-RPC framing, version negotiation,
//! `tools/list`, `resources/*`) lives in `tests/protocol.rs`, which drives the
//! same server through the public API.

use super::*;
use serde_json::json;

#[test]
fn image_payload_becomes_an_image_content_item() {
    let r = resultado_de_herramienta(json!({ "ok": true, "_imagen_png_base64": "AAA" }));
    assert_eq!(r["content"][1]["type"], "image");
    assert_eq!(r["content"][1]["data"], "AAA");
    assert!(!r["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("_imagen"));
}
