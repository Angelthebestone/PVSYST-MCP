//! Protocol-level tests driven through the crate's public API: JSON-RPC
//! framing, handshake and version negotiation, notification semantics, and the
//! shape of `tools/list`, `resources/list` and `resources/read`.
//!
//! In-process counterpart of `mcp_smoke.rs`: it pins the same wire contract but
//! needs no subprocess, so it runs in milliseconds.

// Test code: failing loudly on unexpected state is the point.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::too_many_lines,
    unreachable_pub
)]

mod common;

use common::{protocol_call as call, protocol_server as server_with};
use serde_json::Value;

/// Latest protocol revision the server speaks, as documented in the README.
const LATEST_PROTOCOL: &str = "2025-06-18";

#[derive(Clone, Copy)]
enum Superficie {
    Normal,
    Experta,
}

fn tools_list_len(superficie: Superficie) -> usize {
    let out = server_with(matches!(superficie, Superficie::Experta))
        .handle_line(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#)
        .expect("response");
    serde_json::from_str::<Value>(&out).expect("valid json")["result"]["tools"]
        .as_array()
        .expect("tools array")
        .len()
}

#[test]
fn initialize_echoes_client_protocol_version() {
    let v = call(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"commandcode","version":"0.1.0"}}}"#,
    );
    assert_eq!(v["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(v["result"]["serverInfo"]["name"], "pvsyst-mcp");
}

#[test]
fn initialize_negotiates_once_and_keeps_the_first_version() {
    let server = server_with(false);
    let first = server
        .handle_line(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26"}}"#,
        )
        .expect("response");
    let second = server
        .handle_line(
            r#"{"jsonrpc":"2.0","id":2,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}"#,
        )
        .expect("response");
    assert!(first.contains(r#""protocolVersion":"2025-03-26""#));
    assert!(second.contains(r#""protocolVersion":"2025-03-26""#));
}

#[test]
fn initialize_echoes_older_supported_protocol_version() {
    let v = call(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}"#,
    );
    assert_eq!(v["result"]["protocolVersion"], "2024-11-05");
}

#[test]
fn initialize_defaults_when_client_sends_nothing() {
    let v = call(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#);
    assert_eq!(v["result"]["protocolVersion"], LATEST_PROTOCOL);
}

#[test]
fn initialize_falls_back_on_unknown_protocol_version() {
    let v = call(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"9999-99-99"}}"#,
    );
    assert_eq!(v["result"]["protocolVersion"], LATEST_PROTOCOL);
}

#[test]
fn handle_line_rejects_non_2_0_jsonrpc() {
    let v = call(r#"{"jsonrpc":"1.0","id":1,"method":"ping"}"#);
    assert_eq!(v["error"]["code"], -32600);
}

#[test]
fn handle_line_rejects_missing_jsonrpc() {
    let v = call(r#"{"id":1,"method":"ping"}"#);
    assert_eq!(v["error"]["code"], -32600);
}

#[test]
fn handle_line_rejects_missing_or_non_string_method() {
    let v1 = call(r#"{"jsonrpc":"2.0","id":1}"#);
    assert_eq!(v1["error"]["code"], -32600);
    let v2 = call(r#"{"jsonrpc":"2.0","id":1,"method":123}"#);
    assert_eq!(v2["error"]["code"], -32600);
}

#[test]
fn handle_line_rejects_non_object_params() {
    let v1 = call(r#"{"jsonrpc":"2.0","id":1,"method":"ping","params":"bad"}"#);
    assert_eq!(v1["error"]["code"], -32602);
    let v2 = call(r#"{"jsonrpc":"2.0","id":1,"method":"ping","params":[1,2]}"#);
    assert_eq!(v2["error"]["code"], -32602);
}

#[test]
fn tools_list_as_notification_yields_no_response() {
    assert!(server_with(false)
        .handle_line(r#"{"jsonrpc":"2.0","method":"tools/list"}"#)
        .is_none());
}

#[test]
fn tools_list_with_id_yields_response() {
    let resp = server_with(false)
        .handle_line(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#)
        .expect("requests with id get a response");
    assert!(resp.contains(r#""id":1"#));
    assert!(resp.contains(r#""tools""#));
}

#[test]
fn notification_calls_never_generate_responses() {
    let server = server_with(false);
    for line in [
        r#"{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2025-03-26"}}"#,
        r#"{"jsonrpc":"2.0","method":"resources/read","params":{"uri":"pvsyst://recetas"}}"#,
        r#"{"jsonrpc":"2.0","method":"tools/call","params":{"name":"entorno","arguments":{}}}"#,
        r#"{"jsonrpc":"2.0","method":"ping"}"#,
        r#"{"jsonrpc":"2.0","method":"does/not/exist"}"#,
    ] {
        assert!(
            server.handle_line(line).is_none(),
            "notification should get no response: {line}"
        );
    }
}

#[test]
fn tools_list_exposes_12_tools_without_expert_flag() {
    assert_eq!(tools_list_len(Superficie::Normal), 12);
}

#[test]
fn tools_list_adds_the_expert_tools_with_flag() {
    assert_eq!(tools_list_len(Superficie::Experta), 15);
}

#[test]
fn notifications_get_no_response() {
    assert!(server_with(false)
        .handle_line(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#)
        .is_none());
}

#[test]
fn unknown_method_errors() {
    let v = call(r#"{"jsonrpc":"2.0","id":9,"method":"does/not/exist"}"#);
    assert_eq!(v["error"]["code"], -32601);
}

#[test]
fn tools_list_exposes_annotations_and_bilingual_descriptions() {
    let out = server_with(true)
        .handle_line(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#)
        .expect("response");
    let v: Value = serde_json::from_str(&out).expect("valid json");
    let tools = v["result"]["tools"].as_array().expect("tools array");

    let tool = |name: &str| {
        tools
            .iter()
            .find(|t| t["name"] == name)
            .unwrap_or_else(|| panic!("missing tool {name}"))
    };
    assert_eq!(tool("leer")["annotations"]["readOnlyHint"], true);
    assert_eq!(tool("proyecto")["annotations"]["destructiveHint"], true);
    for t in tools {
        assert!(t["title"].is_string(), "missing title in {t}");
        let description = t["description"].as_str().expect("description");
        assert!(
            description.contains(" / EN: "),
            "description is not bilingual: {description}"
        );
    }
}

#[test]
fn resource_list_has_four_entries() {
    let v = call(r#"{"jsonrpc":"2.0","id":3,"method":"resources/list"}"#);
    assert_eq!(v["result"]["resources"].as_array().unwrap().len(), 4);
}

#[test]
fn resource_read_unknown_uri_is_invalid_params() {
    let v = call(
        r#"{"jsonrpc":"2.0","id":2,"method":"resources/read","params":{"uri":"pvsyst://nope"}}"#,
    );
    assert_eq!(v["error"]["code"], -32602);
}

#[test]
fn structured_content_depends_on_the_negotiated_protocol_version() {
    let server = server_with(false);
    server.handle_line(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}"#,
    );
    let out = server
        .handle_line(
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"listar","arguments":{}}}"#,
        )
        .expect("response");
    let v: Value = serde_json::from_str(&out).expect("valid json");
    assert!(v["result"]["structuredContent"].is_object(), "v={v}");

    let old = server_with(false);
    old.handle_line(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26"}}"#,
    );
    let out = old
        .handle_line(
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"listar","arguments":{}}}"#,
        )
        .expect("response");
    let v: Value = serde_json::from_str(&out).expect("valid json");
    assert!(v["result"].get("structuredContent").is_none(), "v={v}");
}

#[test]
fn serve_reads_any_reader_and_writes_any_writer() {
    let server = server_with(false);
    let input = std::io::Cursor::new(
        br#"{"jsonrpc":"2.0","id":1,"method":"ping"}
"#
        .to_vec(),
    );
    let mut output = Vec::new();
    pvsyst_mcp::serve(&server, input, &mut output).expect("serve");
    let text = String::from_utf8(output).expect("utf8");
    assert!(text.contains(r#""id":1"#), "text={text}");
}
