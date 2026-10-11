//! Unit tests for MCP server handshake, JSON-RPC framing and protocol negotiation.

use super::*;
use serde_json::json;

fn call(line: &str) -> Value {
    let s = Server::new(Paths::resolve());
    let out = s.handle_line(line).expect("response");
    serde_json::from_str(&out).expect("valid json")
}

fn tools_list_len(experto: bool) -> usize {
    let mut paths = Paths::resolve();
    paths.experto = experto;
    tools::list(&paths).len()
}

#[test]
fn initialize_echoes_client_protocol_version() {
    let v = call(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"commandcode","version":"0.1.0"}}}"#,
    );
    assert_eq!(v["result"]["protocolVersion"], "2025-03-26");
    assert_eq!(v["result"]["serverInfo"]["name"], "pvsyst-mcp");
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
    assert_eq!(v["result"]["protocolVersion"], PROTOCOL_VERSION);
}

#[test]
fn initialize_falls_back_on_unknown_protocol_version() {
    let v = call(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"9999-99-99"}}"#,
    );
    assert_eq!(v["result"]["protocolVersion"], PROTOCOL_VERSION);
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
    let s = Server::new(Paths::resolve());
    assert!(s
        .handle_line(r#"{"jsonrpc":"2.0","method":"tools/list"}"#)
        .is_none());
}

#[test]
fn tools_list_with_id_yields_response() {
    let s = Server::new(Paths::resolve());
    let resp = s
        .handle_line(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#)
        .expect("requests with id get a response");
    assert!(resp.contains(r#""id":1"#));
    assert!(resp.contains(r#""tools""#));
}

#[test]
fn notification_calls_never_generate_responses() {
    let s = Server::new(Paths::resolve());
    for line in [
        r#"{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2025-03-26"}}"#,
        r#"{"jsonrpc":"2.0","method":"resources/read","params":{"uri":"pvsyst://recetas"}}"#,
        r#"{"jsonrpc":"2.0","method":"tools/call","params":{"name":"entorno","arguments":{}}}"#,
        r#"{"jsonrpc":"2.0","method":"ping"}"#,
        r#"{"jsonrpc":"2.0","method":"does/not/exist"}"#,
    ] {
        assert!(
            s.handle_line(line).is_none(),
            "notification should get no response: {line}"
        );
    }
}

#[test]
fn tools_list_exposes_12_tools_without_expert_flag() {
    assert_eq!(tools_list_len(false), 12);
}

#[test]
fn tools_list_adds_the_expert_tools_with_flag() {
    assert_eq!(tools_list_len(true), 15);
}

#[test]
fn notifications_get_no_response() {
    let s = Server::new(Paths::resolve());
    assert!(s
        .handle_line(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#)
        .is_none());
}

#[test]
fn unknown_method_errors() {
    let v = call(r#"{"jsonrpc":"2.0","id":9,"method":"does/not/exist"}"#);
    assert_eq!(v["error"]["code"], -32601);
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
fn image_payload_becomes_an_image_content_item() {
    let r = resultado_de_herramienta(json!({ "ok": true, "_imagen_png_base64": "AAA" }));
    assert_eq!(r["content"][1]["type"], "image");
    assert_eq!(r["content"][1]["data"], "AAA");
    assert!(!r["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("_imagen"));
}

#[test]
fn prompts_and_templates_lists_are_empty_not_errors() {
    for m in ["prompts/list", "resources/templates/list"] {
        let v = call(&format!(r#"{{"jsonrpc":"2.0","id":4,"method":"{m}"}}"#));
        assert!(v.get("error").is_none(), "{m} errored");
    }
}
