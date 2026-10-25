use super::*;
use std::path::PathBuf;

const PREESTIMAR: Aproximacion = Aproximacion {
    funcion: "simular preestimar",
    alternativa: "simular ejecutar",
};

fn paths(aproximaciones: bool) -> Paths {
    let mut p = Paths::para_test(PathBuf::new());
    p.aproximaciones = aproximaciones;
    p
}

#[test]
fn should_reject_approximation_with_pvsyst_route_when_flag_is_off() {
    let err = exigir(&paths(false), &PREESTIMAR).unwrap_err().to_string();

    assert!(err.contains("simular ejecutar"), "{err}");
    assert!(err.contains(VARIABLE_ENTORNO), "{err}");
}

#[test]
fn should_allow_approximation_when_flag_is_on() {
    assert!(exigir(&paths(true), &PREESTIMAR).is_ok());
}

#[test]
fn should_label_result_as_mcp_approximation() {
    let mut v = json!({ "anual_kwh": 1.0 });

    marcar(&mut v, &PREESTIMAR);

    assert_eq!(v["origen"], ORIGEN_APROXIMACION);
    assert!(v["aviso_origen"]
        .as_str()
        .unwrap()
        .contains("simular ejecutar"));
    assert_eq!(v["anual_kwh"], 1.0);
}
