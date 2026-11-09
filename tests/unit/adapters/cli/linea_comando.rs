use super::*;

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
