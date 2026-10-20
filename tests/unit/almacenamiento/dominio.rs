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
fn should_configure_self_consumption_with_the_commercial_demo_defaults() {
    let p = parametros(&json!({"estrategia": "autoconsumo"})).expect("parámetros");
    let ediciones = p.ediciones();
    let sistema = cambios(&ediciones, "pvSystem");
    let almacen = cambios(&ediciones, "pvGridStorage");

    assert_eq!(
        valor(sistema, "BatteryFile").as_deref(),
        Some(BATERIA_DEFECTO)
    );
    assert_eq!(valor(sistema, "NBattSerie").as_deref(), Some("10"));
    assert_eq!(valor(sistema, "NBattParall").as_deref(), Some("20"));
    assert_eq!(
        valor(almacen, "Strategy").as_deref(),
        Some("SelfConsumption")
    );
    assert_eq!(
        valor(almacen, "ChargingStrategy").as_deref(),
        Some("PVAvailable")
    );
    assert_eq!(
        valor(almacen, "DischStrategy").as_deref(),
        Some("Immediate")
    );
    assert_eq!(valor(almacen, "BattSoCInit").as_deref(), Some("0.600"));
    assert_eq!(valor(almacen, "PMaxCharge").as_deref(), Some("380.000"));
    assert_eq!(valor(almacen, "PMaxDisch").as_deref(), Some("170.000"));
}

#[test]
fn should_configure_peak_shaving_for_the_peak_cut_strategy() {
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
fn should_accept_custom_battery_and_power_values() {
    let p = parametros(&json!({
        "estrategia": "AUTOCONSUMO",
        "bateria": "Otro.BTR",
        "n_serie": 2,
        "n_paralelo": 3,
        "p_max_carga_kw": 12.5,
        "p_max_descarga_kw": 7.25,
        "soc_inicial": 0.25
    }))
    .expect("parámetros");
    let ediciones = p.ediciones();

    assert_eq!(
        valor(cambios(&ediciones, "pvSystem"), "BatteryFile").as_deref(),
        Some("Otro.BTR")
    );
    assert_eq!(
        valor(cambios(&ediciones, "pvGridStorage"), "PMaxDisch").as_deref(),
        Some("7.250")
    );
    assert_eq!(
        valor(cambios(&ediciones, "pvGridStorage"), "BattSoCInit").as_deref(),
        Some("0.250")
    );
}

#[test]
fn should_reject_a_missing_or_unknown_strategy() {
    let falta = parametros(&json!({})).expect_err("sin estrategia");
    assert!(matches!(falta, Error::MissingArg { .. }));

    let desconocida =
        parametros(&json!({"estrategia": "arbitraria"})).expect_err("estrategia inválida");
    assert!(matches!(desconocida, Error::Invalid(_)));
}

#[test]
fn should_reject_soc_outside_the_open_unit_interval() {
    for soc in [0.0, 1.0, -0.1, 1.1] {
        let error = parametros(&json!({"estrategia": "autoconsumo", "soc_inicial": soc}))
            .expect_err("soc inválido");
        assert!(matches!(error, Error::Invalid(_)), "soc {soc}");
    }
}

#[test]
fn should_reject_batteries_without_btr_extension_or_with_path_separators() {
    for bateria in [
        "Bateria.PAN",
        "sub/Bateria.BTR",
        r"sub\Bateria.BTR",
        "C:Bateria.BTR",
    ] {
        let error = parametros(&json!({"estrategia": "autoconsumo", "bateria": bateria}))
            .expect_err("batería inválida");
        assert!(matches!(error, Error::Invalid(_)), "batería {bateria}");
    }
}

#[test]
fn should_reject_zero_sizes_and_non_positive_powers() {
    let serie =
        parametros(&json!({"estrategia": "autoconsumo", "n_serie": 0})).expect_err("serie cero");
    assert!(matches!(serie, Error::Invalid(_)));

    let potencia = parametros(&json!({
        "estrategia": "autoconsumo",
        "p_max_carga_kw": 0.0
    }))
    .expect_err("potencia cero");
    assert!(matches!(potencia, Error::Invalid(_)));
}
