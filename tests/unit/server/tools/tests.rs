use super::*;
use crate::config::Paths;
use crate::progreso::SeguimientoTarea;
use serde_json::{json, Value};

fn call(paths: &Paths, name: &str, args: &Value) -> Result<Value> {
    call_seguido(&Solicitud {
        paths,
        name,
        args,
        seguimiento: &SeguimientoTarea::inerte(),
    })
}

mod aproximaciones_tests;
mod esquema_tests;
mod validacion_tests;
