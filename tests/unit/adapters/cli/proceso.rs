use super::super::salida_json::output_json;
use super::*;

#[test]
fn should_report_timeout_as_failure_in_output_json() {
    let out = CliOutput {
        exit_code: None,
        stdout: String::new(),
        stderr: String::new(),
        command_line: String::new(),
        timed_out: true,
        timeout_s: 1,
        stdout_truncado: false,
        stderr_truncado: false,
    };
    let v = output_json(&out, "run-simulation", &json!({}));
    assert_eq!(v["success"], json!(false));
    assert_eq!(v["clasificacion"], json!("timeout"));
    assert_eq!(v["detected_error"], json!("timeout tras 1 s"));
}

#[test]
fn should_include_truncation_flags_in_output_json() {
    let out = CliOutput {
        exit_code: Some(0),
        stdout: "Simulation done in 1s".to_string(),
        stderr: String::new(),
        command_line: "cmd".to_string(),
        timed_out: false,
        timeout_s: 10,
        stdout_truncado: true,
        stderr_truncado: false,
    };
    let v = output_json(&out, "run-simulation", &json!({}));
    assert_eq!(v["stdout_truncado"], json!(true));
    assert_eq!(v["stderr_truncado"], json!(false));
    assert_eq!(v["success"], json!(true));
}

#[test]
fn should_refuse_cli_options_that_are_not_texts() {
    for opciones in [json!(["-w:C:\\ws", 3]), json!("-w:C:\\ws"), json!(7)] {
        let res = opciones_de(&json!({ "opciones": opciones }));
        assert!(matches!(res, Err(Error::Invalid(_))), "{opciones}");
    }
}

#[test]
fn should_build_cli_options_from_list_or_object() {
    assert_eq!(
        opciones_de(&json!({ "opciones": ["-a", "-b:1"] })).expect("lista"),
        ["-a", "-b:1"]
    );
    assert_eq!(
        opciones_de(&json!({ "opciones": { "-x": 2 } })).expect("objeto"),
        ["--x:2"]
    );
    assert!(opciones_de(&json!({})).expect("vacío").is_empty());
}
