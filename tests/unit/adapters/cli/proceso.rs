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
