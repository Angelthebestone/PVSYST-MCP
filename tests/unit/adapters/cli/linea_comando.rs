use super::*;
use serde_json::json;

#[test]
fn should_quote_arguments_containing_spaces() {
    let args = vec!["param1".to_string(), "arg con espacios".to_string()];
    let cmd = display_command_line("pvsyst.exe", &args);
    assert_eq!(cmd, "pvsyst.exe param1 \"arg con espacios\"");
}

/// The executable lives under `C:\Program Files`: unquoted, the line cannot be copied.
#[test]
fn should_quote_an_executable_path_with_spaces() {
    let cmd = display_command_line(r"C:\Program Files\PVsyst8.1.8\PVsystCLI.exe", &[]);
    assert_eq!(cmd, r#""C:\Program Files\PVsyst8.1.8\PVsystCLI.exe""#);
}

/// A quote, a tab and a trailing backslash each break the line when left as they are.
#[test]
fn should_escape_quotes_tabs_and_trailing_backslashes() {
    let args = vec![
        r#"-p:a"b"#.to_string(),
        "-rpf:C:/a\tb.pdf".to_string(),
        r"-w:C:/my ws\".to_string(),
    ];
    let cmd = display_command_line("p.exe", &args);
    assert_eq!(cmd, r#"p.exe "-p:a\"b" "-rpf:C:/a	b.pdf" "-w:C:/my ws\\""#);
}

/// A real simulation can take longer than two minutes: its default is the
/// simulation one, and the others keep the short default.
#[test]
fn should_use_the_simulation_default_for_run_simulation() {
    let sin_timeout = json!({});
    assert_eq!(
        timeout_de_comando(&sin_timeout, "run-simulation").expect("defecto"),
        Duration::from_secs(TIMEOUT_SIMULACION_S)
    );
    assert_eq!(
        timeout_de_comando(&sin_timeout, "help").expect("defecto"),
        Duration::from_secs(TIMEOUT_DEFECTO_S)
    );
}

/// A value the CLI cannot honour is an error, not a silent clamp.
#[test]
fn should_refuse_a_timeout_outside_the_declared_range() {
    for valor in [json!(0), json!(-1), json!(3601)] {
        let res = timeout_de_comando(&json!({ "timeout_s": valor }), "help");
        assert!(matches!(res, Err(Error::Invalid(_))), "{valor}");
    }
    assert_eq!(
        timeout_de_comando(&json!({ "timeout_s": 42 }), "help").expect("válido"),
        Duration::from_secs(42)
    );
}

/// A folder is not an executable: accepting one turned the spawn failure into an
/// `io` error that did not name `PVSYST_CLI`.
#[test]
fn should_not_accept_a_directory_as_the_cli() {
    let dir = tempfile::tempdir().expect("carpeta temporal");
    let mut paths = Paths::para_test(dir.path().to_path_buf());
    paths.cli_exe = dir.path().to_path_buf();
    assert!(!cli_available(&paths));
}

/// The licence key and the customer id never come back in the answer.
#[test]
fn should_redact_the_licence_key_and_the_customer_id() {
    let args = vec![
        "lic-activate".to_string(),
        "-k:AAAA-BBBB-CCCC".to_string(),
        "--key:DDDD".to_string(),
        "-c:1234".to_string(),
        "-w:C:/ws".to_string(),
    ];
    let cmd = display_command_line("pvsystcli.exe", &args);
    assert!(!cmd.contains("AAAA"), "{cmd}");
    assert!(!cmd.contains("DDDD"), "{cmd}");
    assert!(!cmd.contains("1234"), "{cmd}");
    assert!(cmd.contains("-k:<REDACTADO>"), "{cmd}");
    assert!(cmd.contains("-w:C:/ws"), "{cmd}");
}
