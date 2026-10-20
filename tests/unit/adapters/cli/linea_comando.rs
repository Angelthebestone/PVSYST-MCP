use super::*;
use serde_json::json;

#[test]
fn should_quote_arguments_containing_spaces() {
    let args = vec!["param1".to_string(), "arg con espacios".to_string()];
    let cmd = display_command_line("pvsyst.exe", &args);
    assert_eq!(cmd, "pvsyst.exe param1 \"arg con espacios\"");
}

#[test]
fn should_clamp_timeout_within_valid_range() {
    assert_eq!(
        timeout_de(&json!({"timeout_s": 0}), 120),
        Duration::from_secs(TIMEOUT_MINIMO_S)
    );
    assert_eq!(
        timeout_de(&json!({"timeout_s": 99999}), 120),
        Duration::from_secs(TIMEOUT_MAXIMO_S)
    );
}

#[test]
fn should_use_default_when_timeout_omitted() {
    assert_eq!(
        timeout_de(&json!({}), TIMEOUT_DEFECTO_S),
        Duration::from_secs(TIMEOUT_DEFECTO_S)
    );
}
