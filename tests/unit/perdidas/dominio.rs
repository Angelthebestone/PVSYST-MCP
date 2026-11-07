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

/// Two subarrays, both with PVsyst's "mismatch is predefined" bit set.
fn variante_con_dos_subarrays() -> Vec<u8> {
    let subarray = |flags: &str| {
        format!(
            "    PVObject_=pvSubArray\n      Flags={flags}\n      PMismMPP=2.00\n      LIDLoss=0.00\n    End of PVObject pvSubArray\n"
        )
    };
    format!(
        "PVObject_=pvVCalcul\n  PVObject_SystemCircuit=pvCircuit\n{}{}  End of PVObject pvCircuit\nEnd of PVObject pvVCalcul\n",
        subarray("$019E0002"),
        subarray("$019E0002")
    )
    .into_bytes()
}

#[test]
fn should_write_lid_and_mismatch_on_every_subarray_when_the_variant_has_two() {
    let p = parametros(&json!({"lid_pct": 1.5, "mismatch_pct": 2.25})).expect("parámetros");
    let subarrays = subarrays_de(&variante_con_dos_subarrays());
    let ediciones = p.ediciones(&subarrays);

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
fn should_clear_the_mismatch_default_bit_when_mismatch_pct_is_set() {
    let p = parametros(&json!({"mismatch_pct": 4.0})).expect("parámetros");
    let subarrays = subarrays_de(&variante_con_dos_subarrays());
    let ediciones = p.ediciones(&subarrays);

    assert_eq!(
        valor(cambios(&ediciones, "pvSubArray[0]"), "Flags").as_deref(),
        Some("$019A0002")
    );
    assert_eq!(
        valor(cambios(&ediciones, "pvSubArray[1]"), "Flags").as_deref(),
        Some("$019A0002")
    );
    assert_eq!(
        valor(cambios(&ediciones, "pvSubArray[0]"), "PMismMPP").as_deref(),
        Some("4.00")
    );
    assert!(p.avisos(&subarrays).is_empty());
}
