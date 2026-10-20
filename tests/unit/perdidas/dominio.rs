use super::*;

fn parametros(valor: &Value) -> Result<ParametrosPerdidas> {
    ParametrosPerdidas::nuevo(EntradaPerdidas::de_args(valor)?)
}

fn cambios<'a>(ediciones: &'a [EdicionObjeto], selector: &str) -> &'a [(String, Option<String>)] {
    ediciones
        .iter()
        .find(|e| e.selector == selector)
        .map(|e| e.cambios.as_slice())
        .expect("edición del selector")
}

fn valor(cambios: &[(String, Option<String>)], clave: &str) -> Option<String> {
    cambios
        .iter()
        .find(|(k, _)| k == clave)
        .and_then(|(_, v)| v.clone())
}

#[test]
fn should_write_soiling_as_a_fraction_and_thermal_with_one_decimal() {
    let p = parametros(&json!({"suciedad_pct": 25, "uc_termico": 29.04})).expect("parámetros");
    let ediciones = p.ediciones(2);
    let main = cambios(&ediciones, "pvMainArray");

    assert_eq!(valor(main, "SoilingLoss").as_deref(), Some("0.250"));
    assert_eq!(valor(main, "UFactor").as_deref(), Some("29.0"));
}

#[test]
fn should_write_lid_and_mismatch_on_every_subarray() {
    let p = parametros(&json!({"lid_pct": 1.5, "mismatch_pct": 2.25})).expect("parámetros");
    let ediciones = p.ediciones(2);

    assert_eq!(ediciones.len(), 2);
    assert_eq!(
        valor(cambios(&ediciones, "pvSubArray[0]"), "LIDLoss").as_deref(),
        Some("1.50")
    );
    assert_eq!(
        valor(cambios(&ediciones, "pvSubArray[1]"), "LIDLoss").as_deref(),
        Some("1.50")
    );
    assert_eq!(
        valor(cambios(&ediciones, "pvSubArray[1]"), "PMismMPP").as_deref(),
        Some("2.25")
    );
}

#[test]
fn should_reject_a_request_without_any_loss() {
    let error = parametros(&json!({})).expect_err("sin pérdidas");
    assert!(matches!(error, Error::MissingArg { .. }));
    assert!(error.to_string().contains("suciedad_pct"));
}

#[test]
fn should_reject_values_above_their_maximum() {
    for entrada in [
        json!({"suciedad_pct": 51.0}),
        json!({"uc_termico": 101.0}),
        json!({"lid_pct": 21.0}),
        json!({"mismatch_pct": 21.0}),
    ] {
        let error = parametros(&entrada).expect_err("fuera de rango");
        assert!(matches!(error, Error::Invalid(_)), "{entrada}");
    }
}

#[test]
fn should_accept_the_upper_bounds_and_zero() {
    parametros(&json!({
        "suciedad_pct": 50,
        "uc_termico": 100,
        "lid_pct": 20,
        "mismatch_pct": 20
    }))
    .expect("límites");
    parametros(&json!({"suciedad_pct": 0})).expect("cero");
}

#[test]
fn should_detect_pvsyst_new_soiling_model() {
    assert!(usa_formato_nuevo_suciedad(
        "PVObject_=pvMainArray\n  Soiling=Soiling\nEnd of PVObject pvMainArray\n"
    ));
    assert!(!usa_formato_nuevo_suciedad(
        "PVObject_=pvMainArray\n  SoilingLoss=0.000\nEnd of PVObject pvMainArray\n"
    ));
}

#[test]
fn should_emit_no_subarray_editions_when_the_variant_has_none() {
    let p = parametros(&json!({"suciedad_pct": 10})).expect("parámetros");
    assert_eq!(p.ediciones(0).len(), 1);
}
