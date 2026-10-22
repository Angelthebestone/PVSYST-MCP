use super::{linea_de_comando, modo_de, Modo};
use std::path::Path;

#[test]
fn should_share_the_desktop_when_variable_is_absent_or_empty() {
    assert_eq!(modo_de(None).unwrap(), Modo::Compartido);
    assert_eq!(modo_de(Some("  ")).unwrap(), Modo::Compartido);
    assert_eq!(modo_de(Some("compartido")).unwrap(), Modo::Compartido);
}

#[test]
fn should_isolate_when_variable_says_aislado_in_any_case() {
    assert_eq!(modo_de(Some("Aislado")).unwrap(), Modo::Aislado);
    assert_eq!(modo_de(Some(" AISLADO ")).unwrap(), Modo::Aislado);
}

#[test]
fn should_reject_unknown_values_when_a_typo_would_drive_the_users_desktop() {
    let error = modo_de(Some("aislada")).unwrap_err().to_string();
    assert!(error.contains("aislada"), "{error}");
}

#[test]
fn should_quote_every_part_when_building_the_command_line() {
    let linea = linea_de_comando(
        Path::new(r"C:\Program Files\PVsyst8.1.7\PVsyst.exe"),
        &[Path::new(r"C:\Users\Ana María\Projects\Casa.PRJ")],
    );
    assert_eq!(
        linea,
        r#""C:\Program Files\PVsyst8.1.7\PVsyst.exe" "C:\Users\Ana María\Projects\Casa.PRJ""#
    );
}
