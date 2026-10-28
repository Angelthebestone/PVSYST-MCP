use super::*;
use crate::progreso::SeguimientoTarea;
use crate::server::tools::{call_seguido, Solicitud};
use serde_json::json;

fn call(name: &str, args: &Value) -> crate::error::Result<Value> {
    let paths = Paths::resolve();
    let seguimiento = SeguimientoTarea::inerte();
    call_seguido(&Solicitud {
        paths: &paths,
        name,
        args,
        seguimiento: &seguimiento,
    })
}

fn rechazo(tool: &str, args: &Value) -> String {
    match call(tool, args) {
        Err(Error::Invalid(texto)) => texto,
        otro => panic!("se esperaba un rechazo por falta de ensayo: {otro:?}"),
    }
}

#[test]
fn should_refuse_a_dry_run_of_the_preferences_edit_because_it_would_write() {
    let args = json!({
        "accion": "preferencias_editar", "seccion": "Preferences",
        "clave": "X", "valor": "Y", "dry_run": true,
    });
    assert!(rechazo("entorno", &args).contains("no tiene ensayo"));
}

#[test]
fn should_refuse_a_dry_run_of_the_series_write() {
    let args = json!({ "accion": "escribir_serie", "dry_run": true });
    assert!(rechazo("sitio", &args).contains("no tiene ensayo"));
}

#[test]
fn should_refuse_a_dry_run_of_preparing_a_batch() {
    let args = json!({ "accion": "preparar", "filas": [], "dry_run": true });
    assert!(rechazo("simular", &args).contains("no tiene ensayo"));
}

#[test]
fn should_refuse_simular_on_the_close_all_action_because_it_would_close_windows() {
    let args = json!({ "accion": "cerrar_todo", "simular": true });
    assert!(rechazo("gui_actuar", &args).contains("no tiene ensayo"));
}

#[test]
fn should_not_read_a_request_dry_run_as_one_when_the_answer_does_not_say_so() {
    let args = json!({ "accion": "preferencias_editar", "dry_run": true });
    let mut salida = json!({ "ok": true });
    let paths = Paths::resolve();
    let contexto = Contexto {
        tool: "entorno",
        accion: "preferencias_editar",
        args: &args,
    };
    anadir_vista(&paths, &contexto, &mut salida);
    let resumen = salida["vista"]["resumen"].as_str().unwrap_or_default();
    assert!(!resumen.starts_with("Ensayo"), "{resumen}");
}

#[test]
fn should_refuse_a_dry_run_of_a_run_that_carries_rows_because_preparing_them_writes_files() {
    let args = json!({
        "accion": "ejecutar", "proyecto": "Casa",
        "filas": [{ "tilt": 20 }], "dry_run": true,
    });
    assert!(rechazo("simular", &args).contains("no admite 'filas'"));
}
