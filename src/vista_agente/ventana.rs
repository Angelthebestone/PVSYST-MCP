//! Which window an agent-view call is about.

use super::dominio::Entidad;
use crate::adapters::gui::ffi::Hwnd;
use crate::adapters::gui::ventanas::{class_of, hwnd_de, ventanas_de, visible};
use crate::error::{Error, Result};
use serde_json::Value;

/// `ventana` when given, else the visible window of the entity's class.
pub(crate) fn ventana_de(args: &Value, entidad: &Entidad) -> Result<Hwnd> {
    if args.get("ventana").is_some() {
        return hwnd_de(args, "ventana");
    }
    ventanas_de(args)?
        .into_iter()
        .find(|h| visible(*h) && class_of(*h) == entidad.formulario)
        .ok_or_else(|| {
            Error::NotFound(format!(
                "No hay una ventana {} abierta para {}: ábrela con gui_actuar accion:funcion",
                entidad.formulario, entidad.clave
            ))
        })
}
