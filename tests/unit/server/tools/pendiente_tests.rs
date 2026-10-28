use super::tests::{con_vista, vista_del_error};
use super::*;
use serde_json::json;

#[test]
fn should_ask_to_repeat_the_call_with_confirmation_when_trial_mode_needs_it() {
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

#[test]
fn should_offer_the_real_run_after_a_dry_run() {
    let args = json!({ "accion": "ejecutar", "proyecto": "Casa", "dry_run": true });
    let vista = con_vista(("simular", "ejecutar"), &args, json!({ "dry_run": true }));
    assert_eq!(vista["estado"], "ok");
    assert!(vista["resumen"]
        .as_str()
        .is_some_and(|r| r.starts_with("Ensayo")));
    assert_eq!(
        vista["siguiente"][0]["argumentos"],
        json!({ "accion": "ejecutar", "proyecto": "Casa" })
    );
}

#[test]
fn should_not_suggest_results_of_a_project_the_answer_never_names() {
    let vista = con_vista(("disenar_sistema", ""), &json!({}), json!({ "ok": true }));
    let herramientas: Vec<_> = vista["siguiente"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|p| p["tool"].clone())
        .collect();
    assert_eq!(herramientas, [json!("entorno")]);
}

#[test]
fn should_not_offer_the_same_scene_call_without_simular_because_it_has_no_real_run() {
    let args = json!({ "accion": "escena", "escena": "Casa_A", "simular": true });
    let vista = con_vista(("gui_actuar", "escena"), &args, json!({ "creado": false }));
    assert!(vista["resumen"]
        .as_str()
        .is_some_and(|r| !r.starts_with("Ensayo")));
    let repite = vista["siguiente"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|p| p["argumentos"]["accion"] == "escena");
    assert!(!repite, "{vista}");
}

#[test]
fn should_not_read_simular_as_a_dry_run_when_the_tool_uses_it_to_run_the_simulation() {
    let args = json!({ "latitud": 40.4, "longitud": -3.7, "kwp": 8, "simular": true });
    let vista = con_vista(
        ("disenar_sistema", ""),
        &args,
        json!({ "proyecto": "S", "ok": true }),
    );
    assert!(
        vista["resumen"]
            .as_str()
            .is_some_and(|r| !r.starts_with("Ensayo")),
        "{vista}"
    );
}

#[test]
fn should_not_offer_to_apply_a_dry_run_that_failed() {
    let paths = Paths::resolve();
    let args = json!({ "accion": "ejecutar", "proyecto": "No_existe", "dry_run": true });
    let error = Error::NotFound("Proyecto no encontrado".into());
    let vista = vista_del_error(&paths, ("simular", &args), &error).expect("known tool");
    let repite = vista["siguiente"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|p| p["tool"] == "simular");
    assert!(!repite, "{vista}");
}

#[test]
fn should_keep_the_failed_dry_run_as_partial_when_the_answer_lists_errors() {
    let args = json!({ "accion": "ejecutar", "proyecto": "Casa", "dry_run": true });
    let salida = json!({ "dry_run": true, "errores": ["falta el módulo"] });
    let vista = con_vista(("simular", "ejecutar"), &args, salida);
    assert_eq!(vista["estado"], "parcial");
    assert!(
        vista["resumen"]
            .as_str()
            .is_some_and(|r| !r.starts_with("Ensayo")),
        "{vista}"
    );
}
