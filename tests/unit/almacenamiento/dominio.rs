use super::*;

fn parametros(valor: &Value) -> Result<ParametrosAlmacenamiento> {
    ParametrosAlmacenamiento::nuevo(EntradaAlmacenamiento::de_args(valor)?)
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
fn should_configure_peak_shaving_when_the_strategy_is_peak_cut() {
    let p = parametros(&json!({"estrategia": "recorte_picos"})).expect("parámetros");
    let ediciones = p.ediciones();
    let almacen = cambios(&ediciones, "pvGridStorage");

    assert_eq!(valor(almacen, "Strategy").as_deref(), Some("PeakShaving"));
    assert_eq!(
        valor(almacen, "ChargingStrategy").as_deref(),
        Some("PVAboveThresh")
    );
    assert_eq!(
        valor(almacen, "DischStrategy").as_deref(),
        Some("Immediate")
    );
}

#[test]
fn should_reject_batteries_when_they_lack_the_btr_extension_or_carry_path_separators() {
    for bateria in [
        "Bateria.PAN",
        "sub/Bateria.BTR",
        r"sub\Bateria.BTR",
        "C:Bateria.BTR",
    ] {
        parametros(&json!({"estrategia": "autoconsumo", "bateria": bateria}))
            .expect_err("batería inválida");
    }
}
