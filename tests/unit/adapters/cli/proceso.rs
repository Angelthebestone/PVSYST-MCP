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
}

#[test]
fn should_refuse_cli_options_that_are_not_texts() {
    for opciones in [json!(["-w:C:\\ws", 3]), json!("-w:C:\\ws"), json!(7)] {
        let res = opciones_de(&json!({ "opciones": opciones }));
        assert!(matches!(res, Err(Error::Invalid(_))), "{opciones}");
    }
}
