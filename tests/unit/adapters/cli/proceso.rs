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

fn salida(command: &str, stdout: &str) -> serde_json::Value {
    let out = CliOutput {
        exit_code: Some(0),
        stdout: stdout.to_string(),
        stderr: String::new(),
        command_line: String::new(),
        timed_out: false,
        timeout_s: 120,
        stdout_truncado: false,
        stderr_truncado: false,
        crash: None,
    };
    output_json(&out, command, &json!({}))
}

/// `help` lists `lic-activate`; the license-banner filter must not eat that line.
#[test]
fn should_keep_the_license_lines_of_a_help_answer() {
    let stdout = "Available commands:\r\n   lic-activate: Activate PVsystCLI with the given license key\r\n   version: Show PVsystCLI version and check for available updates\r\n";
    let v = salida("help", stdout);
    let tail = v["stdout_tail"].as_str().unwrap_or_default();
    assert!(tail.contains("lic-activate"), "{v}");
    assert!(tail.contains("version"), "{v}");
}

/// The `%` filter exists for the simulation progress; another command may print one.
#[test]
fn should_keep_a_percentage_line_of_a_command_that_is_not_a_simulation() {
    let v = salida("version", "PR: 80 %\r\nPVsystCLI 8.1.8 (rev. 49527)\r\n");
    let tail = v["stdout_tail"].as_str().unwrap_or_default();
    assert!(tail.contains("PR: 80 %"), "{v}");
}

/// A tail cut by line count must say so, or the caller reads half an answer.
#[test]
fn should_flag_a_stdout_tail_cut_by_line_count() {
    let stdout: String = (1..=70).map(|n| format!("linea {n}\r\n")).collect();
    let v = salida("help", &stdout);
    assert_eq!(v["stdout_tail_truncado"], json!(true), "{v}");
    assert!(!v["stdout_truncado"].as_bool().unwrap_or(true), "{v}");
}
