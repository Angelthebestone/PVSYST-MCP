use super::*;
use crate::progreso::SeguimientoTarea;
use crate::server::tools::{call_seguido, Solicitud};
use serde_json::json;

fn call(name: &str, args: &Value) -> crate::error::Result<Value> {
    let paths = Paths::para_test(std::env::temp_dir());
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
fn should_refuse_a_dry_run_that_would_write_when_the_action_has_no_rehearsal() {
    let args = json!({
        "accion": "ejecutar", "proyecto": "Casa",
        "filas": [{ "tilt": 20 }], "dry_run": true,
    });
    assert!(rechazo("simular", &args).contains("no admite 'filas'"));
}
