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
        crash: None,
    };
    let v = output_json(&out, "run-simulation", &json!({}));
    assert_eq!(v["success"], json!(false));
    assert_eq!(v["clasificacion"], json!("timeout"));
}

#[test]
fn should_report_a_crash_as_error_instead_of_ambiguous() {
    let out = CliOutput {
        exit_code: Some(0),
        stdout: "Checking license...".to_string(),
        stderr: String::new(),
        command_line: String::new(),
        timed_out: false,
        timeout_s: 900,
        stdout_truncado: false,
        stderr_truncado: false,
        crash: Some("Instruction privilégiée".to_string()),
    };
    let v = output_json(&out, "run-simulation", &json!({}));
    assert_eq!(v["success"], json!(false));
    assert_eq!(v["clasificacion"], json!("error"));
    assert_eq!(v["crash"], json!("Instruction privilégiée"));
    assert_eq!(v["detected_error"], json!("crash: Instruction privilégiée"));
}

#[test]
fn should_refuse_cli_options_that_are_not_texts() {
    for opciones in [json!(["-w:C:\\ws", 3]), json!("-w:C:\\ws"), json!(7)] {
        let res = opciones_de(&json!({ "opciones": opciones }));
        assert!(matches!(res, Err(Error::Invalid(_))), "{opciones}");
    }
}
