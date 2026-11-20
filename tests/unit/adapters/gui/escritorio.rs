use super::{linea_de_comando, modo_de};
use std::path::Path;

#[test]
fn should_reject_unknown_values_when_a_typo_would_drive_the_users_desktop() {
    let error = modo_de(Some("aislada")).unwrap_err().to_string();
    assert!(error.contains("aislada"), "{error}");
}

#[test]
fn should_quote_every_part_when_building_the_command_line() {
    let linea = linea_de_comando(
        Path::new(r"C:\Program Files\PVsyst8.1.7\PVsyst.exe"),
        &[Path::new(
            r"C:\Users\Usuaria Ejemplo\Proyectos Año\Casa.PRJ",
        )],
    );
    assert_eq!(
        linea,
        r#""C:\Program Files\PVsyst8.1.7\PVsyst.exe" "C:\Users\Usuaria Ejemplo\Proyectos Año\Casa.PRJ""#
    );
}
