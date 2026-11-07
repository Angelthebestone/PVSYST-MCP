use super::*;

#[test]
fn should_quote_arguments_containing_spaces() {
    let args = vec!["param1".to_string(), "arg con espacios".to_string()];
    let cmd = display_command_line("pvsyst.exe", &args);
    assert_eq!(cmd, "pvsyst.exe param1 \"arg con espacios\"");
}
