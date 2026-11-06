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
fn should_write_lid_and_mismatch_on_every_subarray_when_the_variant_has_two() {
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
