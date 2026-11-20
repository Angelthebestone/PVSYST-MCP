use super::tests::con_vista;
use serde_json::json;

#[test]
fn should_ask_for_confirmation_when_the_trial_run_needs_it() {
    let args = json!({ "latitud": 40.4, "longitud": -3.7, "kwp": 8 });
    let salida = json!({
        "requiere_confirmacion": true,
        "mensaje": "Esta acción consume 2 ejecuciones de prueba; repítela con confirmar:true",
    });
    let vista = con_vista(("disenar_sistema", ""), &args, salida);
    assert_eq!(vista["estado"], "parcial");
    assert_eq!(vista["error"]["codigo"], "requiere_confirmacion");
    assert!(vista["resumen"]
        .as_str()
        .is_some_and(|r| r.starts_with("No se ejecutó nada")));
    assert_eq!(vista["siguiente"].as_array().map(Vec::len), Some(1));
    assert_eq!(vista["siguiente"][0]["tool"], "disenar_sistema");
    assert_eq!(
        vista["siguiente"][0]["argumentos"],
        json!({ "latitud": 40.4, "longitud": -3.7, "kwp": 8, "confirmar": true })
    );
}
