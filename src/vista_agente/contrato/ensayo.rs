//! Which actions really have a dry run. The switch is declared per tool but only
//! some actions read it: on the others it would be ignored and the action would
//! run for real, so the request is refused before anything happens.

use super::pendiente::clave_de_ensayo;
use crate::args::flag;
use crate::error::{Error, Result};
use serde_json::Value;

/// Whether `accion` of `tool` reads its dry-run switch.
fn implementa_ensayo(tool: &str, accion: &str) -> bool {
    matches!(
        (tool, accion),
        ("pvsyst_cli", _)
            | ("entorno", "licencia" | "licencia_sync" | "exportar_logs")
            | ("simular", "ejecutar")
            | ("sitio", "convertir_meteo" | "crear")
            | ("gui_actuar", "escena" | "escena_importar" | "rellenar")
    )
}

/// Fails when the request asks for a dry run of an action that has none.
pub(crate) fn exigir_ensayo_soportado(tool: &str, accion: &str, args: &Value) -> Result<()> {
    let Some(clave) = clave_de_ensayo(tool) else {
        return Ok(());
    };
    if !flag(args, &[clave]) || implementa_ensayo(tool, accion) {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "'{clave}' no existe en {tool} {accion}: esa acción no tiene ensayo y se ejecutaría de verdad. Quita '{clave}'"
    )))
}
