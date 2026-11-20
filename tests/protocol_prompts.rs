//! MCP prompts: template filling and the error contract of `prompts/get`.

// Test code: failing loudly on unexpected state is the point.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod cliente_mcp;

use cliente_mcp::protocol_call as call;

#[test]
fn should_fill_the_template_when_prompts_get_receives_the_arguments() {
    let v = call(
        r#"{"jsonrpc":"2.0","id":5,"method":"prompts/get","params":{"name":"disenar_desde_coordenadas","arguments":{"latitud":"7.07","longitud":"-73.84","kwp":"10"}}}"#,
    );
    let text = v["result"]["messages"][0]["content"]["text"]
        .as_str()
        .expect("prompt text");
    assert!(text.contains("10 kWp"), "text={text}");
    assert!(text.contains("-73.84"), "text={text}");
}

#[test]
fn should_return_invalid_params_when_a_required_prompt_argument_is_missing() {
    let v = call(
        r#"{"jsonrpc":"2.0","id":6,"method":"prompts/get","params":{"name":"disenar_desde_coordenadas","arguments":{"latitud":"7.07","longitud":"-73.84"}}}"#,
    );
    assert_eq!(v["error"]["code"], -32602);
}
