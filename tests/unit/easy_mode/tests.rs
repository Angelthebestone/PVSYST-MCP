use super::dimensionado::{ajustar_serie, dimensionar_serie, ejecutar_paso_dimensionado};
use super::parametros::{DimensionadoParametros, DisenoParametros};
use super::simulacion::meteo_de_metadatos;
use super::*;
use serde_json::json;

#[test]
fn dimensionar_serie_finds_exact_layout_inside_window() {
    // Ns 5..=17 keeps Vmpp >= 200 V and Voc*1.15*Ns <= 1000 V; 5x5x400 W = 10 kW.
    assert_eq!(
        dimensionar_serie(10.0, 400.0, 49.0, 41.0, 200.0, 1000.0),
        Some((5, 5))
    );
}

#[test]
fn dimensionar_serie_rejects_empty_window() {
    // Ns >= 22 is needed for Vmpp >= 900 V but Ns <= 14 caps Voc: no layout.
    assert_eq!(
        dimensionar_serie(10.0, 400.0, 49.0, 41.0, 900.0, 800.0),
        None
    );
}

#[test]
fn errores_de_collects_failed_steps_and_kpi_error() {
    let pasos = vec![
        json!({ "paso": "sitio", "ok": true }),
        json!({ "paso": "simulacion", "ok": false }),
    ];
    let errores = errores_de(&pasos, Some("sin CSV"));
    assert_eq!(
        errores,
        vec![
            "paso 'simulacion' fallido".to_string(),
            "leer_kpis: sin CSV".to_string()
        ]
    );
}

#[test]
fn avisos_flatten_step_warnings_and_flag_big_deviation() {
    let pasos = vec![
        json!({ "paso": "sitio", "ok": true }),
        json!({ "paso": "dimensionado", "ok": true, "avisos": ["a", "b"] }),
    ];
    assert_eq!(avisos_de(&pasos), vec!["a".to_string(), "b".to_string()]);
    assert!(aviso_desviacion(10.0, &json!(10.2)).is_none());
    let a = aviso_desviacion(10.0, &json!(9.0)).expect("-10 % is flagged");
    assert!(a.contains("-10.0 %"));
    assert!(aviso_desviacion(10.0, &Value::Null).is_none());
}

#[test]
fn errores_include_the_step_detail_when_it_is_text() {
    let pasos = vec![json!({ "paso": "sitio_proyecto", "ok": false, "detail": "bloque inválido" })];
    assert_eq!(
        errores_de(&pasos, None),
        vec!["paso 'sitio_proyecto' fallido: bloque inválido".to_string()]
    );
}

#[test]
fn meteo_de_metadatos_reads_weather_banner() {
    let csv = "Project;MCP.PRJ;\nWeather data;Bogota.MET;\nhora;GlobHor\n";
    assert_eq!(meteo_de_metadatos(csv), Some("Bogota.MET".to_string()));
    assert_eq!(meteo_de_metadatos("Project;MCP.PRJ;\n"), None);
}

#[test]
fn ajustar_serie_picks_layout_closest_to_target() {
    // 10 kWp of 300 W around Ns 15: 17 x 2 = 10.2 kWp is the closest.
    let (ns, np, real) = ajustar_serie(10.0, 300.0, 15).expect("layout");
    assert_eq!((ns, np), (17, 2));
    assert!((real - 10.2).abs() < 1e-9);
}

#[test]
fn ajustar_serie_needs_a_module_power() {
    assert_eq!(ajustar_serie(10.0, 0.0, 15), None);
}

#[test]
fn should_reject_invalid_tilt_type_at_input_parse() {
    let err_str = DisenoParametros::parse(&json!({
        "latitud": 10.0,
        "longitud": -70.0,
        "kwp": 5.0,
        "tilt": "diez_grados"
    }))
    .unwrap_err();
    assert!(
        err_str.to_string().contains("tilt"),
        "expected error to mention tilt, got: {err_str}"
    );

    let err_bool = DisenoParametros::parse(&json!({
        "latitud": 10.0,
        "longitud": -70.0,
        "kwp": 5.0,
        "tilt": true
    }))
    .unwrap_err();
    assert!(
        err_bool.to_string().contains("tilt"),
        "expected error to mention tilt, got: {err_bool}"
    );
}

#[test]
fn should_reject_invalid_azimut_type_at_input_parse() {
    let err_str = DisenoParametros::parse(&json!({
        "latitud": 10.0,
        "longitud": -70.0,
        "kwp": 5.0,
        "azimut": "sur"
    }))
    .unwrap_err();
    assert!(
        err_str.to_string().contains("azimut"),
        "expected error to mention azimut, got: {err_str}"
    );

    let err_obj = DisenoParametros::parse(&json!({
        "latitud": 10.0,
        "longitud": -70.0,
        "kwp": 5.0,
        "azimut": { "grados": 0 }
    }))
    .unwrap_err();
    assert!(
        err_obj.to_string().contains("azimut"),
        "expected error to mention azimut, got: {err_obj}"
    );
}

#[test]
fn should_reject_invalid_kwp_at_input_parse() {
    let err_neg = DisenoParametros::parse(&json!({
        "latitud": 10.0,
        "longitud": -70.0,
        "kwp": -5.0
    }))
    .unwrap_err();
    assert!(
        err_neg.to_string().contains("kwp"),
        "expected error to mention kwp, got: {err_neg}"
    );

    let err_type = DisenoParametros::parse(&json!({
        "latitud": 10.0,
        "longitud": -70.0,
        "kwp": "alta_potencia"
    }))
    .unwrap_err();
    assert!(
        err_type.to_string().contains("kwp"),
        "expected error to mention kwp, got: {err_type}"
    );
}

#[test]
fn should_reject_unreadable_or_nonexistent_module_in_sizing() {
    let paths = Paths::resolve();
    let params = DimensionadoParametros {
        kwp: 10.0,
        modulo: Some("ModuloInexistente_12345.PAN".to_string()),
        inversor: None,
        pnom_modulo: None,
        n_mod_serie: None,
        n_strings: None,
    };
    let err = ejecutar_paso_dimensionado(&paths, "TestPrj", &params).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("ModuloInexistente_12345.PAN"),
        "expected error to explicitly mention the missing module name, got: {msg}"
    );
}
