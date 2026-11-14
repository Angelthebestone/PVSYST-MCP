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

mod cliente_mcp;

use cliente_mcp::{protocol_call as call, protocol_server as server_with};
use serde_json::Value;

#[test]
fn should_keep_the_first_version_when_initialize_is_sent_twice() {
    let server = server_with();
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
fn should_reject_a_request_when_jsonrpc_is_not_2_0() {
    let v = call(r#"{"jsonrpc":"1.0","id":1,"method":"ping"}"#);
    assert_eq!(v["error"]["code"], -32600);
}

#[test]
fn should_return_no_response_when_a_request_is_a_notification() {
    let server = server_with();
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
fn should_return_method_not_found_when_the_method_is_unknown() {
    let v = call(r#"{"jsonrpc":"2.0","id":9,"method":"does/not/exist"}"#);
    assert_eq!(v["error"]["code"], -32601);
}

#[test]
fn should_return_invalid_params_when_the_resource_uri_is_unknown() {
    let v = call(
        r#"{"jsonrpc":"2.0","id":2,"method":"resources/read","params":{"uri":"pvsyst://nope"}}"#,
    );
    assert_eq!(v["error"]["code"], -32602);
}

#[test]
fn should_gate_structured_content_on_the_negotiated_protocol_version() {
    let server = server_with();
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

    let old = server_with();
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

fn call_tool(server: &pvsyst_mcp::Server, name: &str, arguments: &str) -> Value {
    let line = format!(
        r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"{name}","arguments":{arguments}}}}}"#
    );
    let out = server.handle_line(&line).expect("response");
    serde_json::from_str(&out).expect("valid json")
}

fn server_estructurado() -> pvsyst_mcp::Server {
    let server = server_with();
    server.handle_line(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}"#,
    );
    server
}

#[test]
fn should_attach_a_vista_with_state_summary_and_next_steps_when_a_tool_succeeds() {
    let v = call_tool(&server_estructurado(), "entorno", "{}");
    let vista = &v["result"]["structuredContent"]["vista"];
    assert_eq!(vista["estado"], "ok", "v={v}");
    assert!(vista["resumen"].as_str().is_some_and(|r| !r.is_empty()));
    assert!(vista["siguiente"][0]["tool"].is_string());
    assert!(vista["siguiente"][0]["argumentos"].is_object());
}

#[test]
fn should_attach_a_vista_with_code_and_valid_keys_when_a_tool_fails() {
    let v = call_tool(&server_estructurado(), "proyecto", r#"{"accion":"doctor"}"#);
    assert_eq!(v["result"]["isError"], true, "v={v}");
    let vista = &v["result"]["structuredContent"]["vista"];
    assert_eq!(vista["estado"], "error");
    assert_eq!(vista["error"]["codigo"], "missing_arg");
    assert_eq!(vista["error"]["claves_validas"][0], "proyecto");
    assert_eq!(vista["siguiente"][0]["tool"], "entorno");
}
