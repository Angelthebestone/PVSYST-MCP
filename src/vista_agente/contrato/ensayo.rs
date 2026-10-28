//! Which actions really have a dry run. The switch is declared per tool but only
//! some actions read it: on the others it would be ignored and the action would
//! run for real, so the request is refused before anything happens.

use super::pendiente::clave_de_ensayo;
use crate::args::{flag, opt_str};
use crate::error::{Error, Result};
use serde_json::Value;

/// Whether `accion` of `tool` reads its dry-run switch.
fn implementa_ensayo(tool: &str, accion: &str, args: &Value) -> bool {
    match (tool, accion) {
        ("pvsyst_cli", _) => true,
        ("entorno", "licencia" | "licencia_sync" | "exportar_logs") => true,
        ("simular", "ejecutar") => true,
        ("sitio", "convertir_meteo") => true,
        ("sitio", "crear") => opt_str(args, &["origen"]) == Some("csv"),
        ("gui_actuar", "escena" | "rellenar") => true,
        _ => false,
    }
}

/// Fails when the request asks for a dry run of an action that has none.
pub(crate) fn exigir_ensayo_soportado(tool: &str, accion: &str, args: &Value) -> Result<()> {
    let Some(clave) = clave_de_ensayo(tool) else {
        return Ok(());
    };
    if !flag(args, &[clave]) || implementa_ensayo(tool, accion, args) {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "'{clave}' no existe en {tool} {accion}: esa acción no tiene ensayo y se ejecutaría de verdad. Quita '{clave}'"
    )))
}
