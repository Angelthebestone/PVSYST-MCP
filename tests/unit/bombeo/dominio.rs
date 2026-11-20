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
fn should_repeat_static_depth_when_it_is_given_in_system_and_water_use() {
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
    assert_eq!(valor(&seccion, "FlowModul"), None);
}

#[test]
fn should_reject_pumps_when_they_lack_the_pmp_extension_or_carry_path_separators() {
    for bomba in [
        "Bomba.PAN",
        "sub/Bomba.PMP",
        r"sub\Bomba.PMP",
        "C:Bomba.PMP",
    ] {
        parametros(&json!({"bomba": bomba})).expect_err("bomba inválida");
    }
}
