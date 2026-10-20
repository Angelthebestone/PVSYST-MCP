use super::*;

fn variante(system_type: &str) -> Vec<u8> {
    format!(
        "PVObject_=pvVCalcul\n  PVObject_System=pvSystem\n    SystemType={system_type}\n  End of PVObject pvSystem\nEnd of PVObject pvVCalcul\n"
    )
    .into_bytes()
}

#[test]
fn should_map_every_known_system_type() {
    assert_eq!(
        TipoSistema::de_variante(&variante("Grid")).expect("grid"),
        TipoSistema::Red
    );
    assert_eq!(
        TipoSistema::de_variante(&variante("Battery")).expect("battery"),
        TipoSistema::Aislado
    );
    assert_eq!(
        TipoSistema::de_variante(&variante("Pumping")).expect("pumping"),
        TipoSistema::Bombeo
    );
    assert_eq!(
        TipoSistema::de_variante(&variante("DCGrid")).expect("dc grid"),
        TipoSistema::RedDc
    );
}

#[test]
fn should_reject_unknown_system_type_as_format_error() {
    let error = TipoSistema::de_variante(&variante("SolarThing")).expect_err("desconocido");
    assert!(matches!(error, Error::Format(_)));
    assert!(error.to_string().contains("SolarThing"), "got: {error}");
}

#[test]
fn should_reject_variant_without_system_type() {
    let error = TipoSistema::de_variante(b"PVObject_=pvVCalcul\nEnd of PVObject pvVCalcul\n")
        .expect_err("sin SystemType");
    assert!(matches!(error, Error::Format(_)));
    assert!(error.to_string().contains("SystemType"));
}

#[test]
fn should_accept_matching_system_type() {
    assert!(TipoSistema::exigir(&variante("Grid"), TipoSistema::Red).is_ok());
}

#[test]
fn should_reject_mismatched_system_type_with_both_names() {
    let error = TipoSistema::exigir(&variante("Battery"), TipoSistema::Red).expect_err("aislado");
    assert!(matches!(error, Error::Invalid(_)));
    let mensaje = error.to_string();
    assert!(mensaje.contains("Aislado"));
    assert!(mensaje.contains("Red"));
}
