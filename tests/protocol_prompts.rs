//! MCP prompts: catalogue, template filling and the recipes resource.

mod common;

use common::protocol_call as call;
use serde_json::Value;

#[test]
fn prompts_and_templates_lists_are_not_errors() {
    for m in ["prompts/list", "resources/templates/list"] {
        let v = call(&format!(r#"{{"jsonrpc":"2.0","id":4,"method":"{m}"}}"#));
        assert!(v.get("error").is_none(), "{m} errored");
    }
}

#[test]
fn prompts_list_exposes_the_seven_recipes() {
    let v = call(r#"{"jsonrpc":"2.0","id":4,"method":"prompts/list"}"#);
    assert_eq!(v["result"]["prompts"].as_array().unwrap().len(), 7);
}

#[test]
fn prompts_get_fills_the_template_from_the_arguments() {
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
fn prompts_get_without_a_required_argument_is_invalid_params() {
    let v = call(
        r#"{"jsonrpc":"2.0","id":6,"method":"prompts/get","params":{"name":"disenar_desde_coordenadas","arguments":{"latitud":"7.07","longitud":"-73.84"}}}"#,
    );
    assert_eq!(v["error"]["code"], -32602);
}

#[test]
fn recipes_resource_still_lists_the_seven_recipes() {
    let v = call(
        r#"{"jsonrpc":"2.0","id":7,"method":"resources/read","params":{"uri":"pvsyst://recetas"}}"#,
    );
    let text = v["result"]["contents"][0]["text"].as_str().expect("text");
    let resource: Value = serde_json::from_str(text).expect("valid json");
    assert_eq!(resource["recetas"].as_array().unwrap().len(), 7);
}
