use super::comparacion::excluida;
use super::incertidumbre::{coef_variacion, incertidumbre};

#[test]
fn coef_variacion_of_two_values() {
    assert_eq!(coef_variacion(&[100.0, 100.0]), Some(0.0));
    let cv = coef_variacion(&[90.0, 110.0]).unwrap();
    assert!((cv - 0.1414).abs() < 1e-3);
    assert_eq!(coef_variacion(&[100.0]), None);
}

#[test]
fn p90_and_p99_use_combined_sigma() {
    let v = incertidumbre(1000.0, 5.0, 3.0, "defecto");
    assert!((v["p90_kwh"].as_f64().unwrap() - 925.3).abs() < 0.5);
    assert!((v["p99_kwh"].as_f64().unwrap() - 864.4).abs() < 0.5);
}

#[test]
fn excluded_parameter_prefixes_are_skipped() {
    assert!(excluida("VarSimul_TempLss"));
    assert!(!excluida("FieldTilt"));
}

#[test]
fn should_reject_project_or_variant_that_is_not_a_plain_name_when_building_case() {
    let ws = tempfile::tempdir().expect("ws");
    let paths = crate::config::Paths::para_test(ws.path());
    for args in [
        serde_json::json!({ "proyecto": "..\\..\\x" }),
        serde_json::json!({ "proyecto": "Planta", "variante": "../VC0" }),
    ] {
        let res = super::caso(&paths, &args);
        assert!(
            matches!(res, Err(crate::error::Error::Invalid(_))),
            "{args}"
        );
    }
}

#[test]
fn should_reject_compared_variant_that_is_not_text_or_identifier() {
    let ws = tempfile::tempdir().expect("ws");
    let paths = crate::config::Paths::para_test(ws.path());
    for variantes in [
        serde_json::json!(["VC0", 1]),
        serde_json::json!(["VC0", "../VC1"]),
    ] {
        let args = serde_json::json!({ "proyecto": "Planta", "variantes": variantes });
        let res = super::comparar(&paths, &args);
        assert!(
            matches!(res, Err(crate::error::Error::Invalid(_))),
            "{args}"
        );
    }
}

#[test]
fn should_list_report_sections_left_out_with_their_reason() {
    let falta = crate::error::Error::NotFound("sin página de balance de carbono".into());

    let omitidas =
        super::entregable::secciones_omitidas(&[("carbono", Some(&falta)), ("economia", None)]);

    assert_eq!(omitidas.len(), 1);
    assert_eq!(omitidas[0]["seccion"], "carbono");
    assert!(omitidas[0]["motivo"]
        .as_str()
        .is_some_and(|m| m.contains("carbono")));
}
