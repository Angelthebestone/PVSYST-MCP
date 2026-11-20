use super::*;

fn variante(system_type: &str) -> Vec<u8> {
    format!(
        "PVObject_=pvVCalcul\n  PVObject_System=pvSystem\n    SystemType={system_type}\n  End of PVObject pvSystem\nEnd of PVObject pvVCalcul\n"
    )
    .into_bytes()
}

#[test]
fn should_reject_unknown_system_type_as_format_error() {
    let error = TipoSistema::de_variante(&variante("SolarThing")).expect_err("desconocido");
    assert!(matches!(error, Error::Format(_)));
    assert!(error.to_string().contains("SolarThing"), "got: {error}");
}
