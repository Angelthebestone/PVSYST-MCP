use super::*;

fn parametros(valor: &Value) -> Result<ParametrosBombeo> {
    ParametrosBombeo::nuevo(EntradaBombeo::de_args(valor)?)
}

fn valor(cambios: &[(String, Option<String>)], clave: &str) -> Option<String> {
    cambios
        .iter()
        .find(|(k, _)| k == clave)
        .and_then(|(_, v)| v.clone())
}

#[test]
fn should_edit_only_water_use_when_only_flow_is_given() {
    let p = parametros(&json!({"caudal_m3_dia": 25})).expect("parámetros");
    assert!(p.edicion_sistema().is_none());
    let seccion = p.cambios_seccion();
    assert_eq!(valor(&seccion, "FlowModul").as_deref(), Some("Yearly"));
    assert_eq!(valor(&seccion, "FlowYear").as_deref(), Some("25.000"));
    assert_eq!(valor(&seccion, "StaticDepth"), None);
}

#[test]
fn should_repeat_static_depth_in_system_and_water_use() {
    let p = parametros(&json!({"profundidad_estatica_m": 80.06})).expect("parámetros");
    let sistema = p.edicion_sistema().expect("edición de pvSystem");
    assert_eq!(sistema.selector, "pvSystem");
    assert_eq!(
        valor(&sistema.cambios, "StaticDepth").as_deref(),
        Some("-80.1")
    );

    let seccion = p.cambios_seccion();
    assert_eq!(valor(&seccion, "StaticDepth").as_deref(), Some("-80.1"));
    assert_eq!(valor(&seccion, "FlowYear"), None);
}

#[test]
fn should_edit_only_the_pump_when_nothing_else_is_given() {
    let p = parametros(&json!({"bomba": "Generic_6kW_H60_160_FR10.PMP"})).expect("parámetros");
    let sistema = p.edicion_sistema().expect("edición de pvSystem");
    assert_eq!(
        valor(&sistema.cambios, "PumpFile").as_deref(),
        Some("Generic_6kW_H60_160_FR10.PMP")
    );
    assert!(p.cambios_seccion().is_empty());
}

#[test]
fn should_round_flow_to_three_decimals() {
    let p = parametros(&json!({"caudal_m3_dia": 25.1234})).expect("parámetros");
    assert_eq!(
        valor(&p.cambios_seccion(), "FlowYear").as_deref(),
        Some("25.123")
    );
}

#[test]
fn should_reject_a_request_without_any_pumping_parameter() {
    let error = parametros(&json!({})).expect_err("sin parámetros");
    assert!(matches!(error, Error::MissingArg { .. }));
    assert!(error.to_string().contains("caudal_m3_dia"));
}

#[test]
fn should_reject_flow_and_depth_outside_their_ranges() {
    for entrada in [
        json!({"caudal_m3_dia": 0}),
        json!({"caudal_m3_dia": 10_001}),
        json!({"profundidad_estatica_m": 0}),
        json!({"profundidad_estatica_m": 1_001}),
    ] {
        let error = parametros(&entrada).expect_err("fuera de rango");
        assert!(matches!(error, Error::Invalid(_)), "{entrada}");
    }
}

#[test]
fn should_reject_pumps_without_pmp_extension_or_with_path_separators() {
    for bomba in [
        "Bomba.PAN",
        "sub/Bomba.PMP",
        r"sub\Bomba.PMP",
        "C:Bomba.PMP",
    ] {
        let error = parametros(&json!({"bomba": bomba})).expect_err("bomba inválida");
        assert!(matches!(error, Error::Invalid(_)), "bomba {bomba}");
    }
}
