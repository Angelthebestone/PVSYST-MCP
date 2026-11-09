//! `gui_actuar accion:lanzar proyecto:…`. PVsyst 8.1.8 ignores a project given on its
//! command line (measured with `/` and `\` separators: it shows the home screen),
//! so it is started without one and the project is loaded through PVsyst's own
//! «Cargar proyecto» window, the same path the project steps use.

use super::{abrir_sesion, ProyectoEnPvsyst};
use crate::adapters::gui::{self, proceso};
use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::{json, Value};

/// Variant opened with the project when the request names none.
const VARIANTE_POR_DEFECTO: &str = "VC0";

/// Whether the request is a launch that wants a project open.
pub(crate) fn pide_proyecto(args: &Value) -> bool {
    args.get("accion").and_then(Value::as_str) == Some("lanzar") && args.get("proyecto").is_some()
}

/// The launch request without the project, which PVsyst would ignore.
fn sin_proyecto(args: &Value) -> Value {
    let mut sin = args.clone();
    if let Some(mapa) = sin.as_object_mut() {
        mapa.remove("proyecto");
    }
    sin
}

/// Starts PVsyst and opens the project in it. A PVsyst that is already running
/// is answered by the plain launch, which refuses a project with its reason.
pub(crate) fn lanzar_con_proyecto(paths: &Paths, args: &Value) -> Result<Value> {
    if proceso::gui_running() {
        return gui::actuar(paths, args);
    }
    let pedido = args
        .get("proyecto")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let ruta = proceso::resolver_proyecto(paths, args, pedido)?;
    let nombre = ruta
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .ok_or_else(|| Error::Invalid(format!("'{pedido}' no es el nombre de un proyecto")))?;
    let variante = args
        .get("variante")
        .and_then(Value::as_str)
        .unwrap_or(VARIANTE_POR_DEFECTO);
    let mut respuesta = gui::actuar(paths, &sin_proyecto(args))?;
    // The window stays open on purpose: no `Sesion::ejecutar`, which returns to the home screen.
    let sesion = abrir_sesion(paths)?;
    let abierto = ProyectoEnPvsyst::abrir(&sesion, paths, &nombre, variante)?;
    if let Some(mapa) = respuesta.as_object_mut() {
        mapa.insert(
            "proyecto".into(),
            json!({
                "abierto": abierto.ventana.titulo(),
                "variante": variante.to_ascii_uppercase(),
                "ventana": format!("0x{:X}", abierto.ventana.h),
            }),
        );
        mapa.insert(
            "aviso".into(),
            json!("PVsyst 8.1.8 ignora el proyecto de su línea de comandos: se lanzó sin él y se cargó con su ventana «Cargar proyecto»."),
        );
    }
    Ok(respuesta)
}

#[cfg(test)]
#[path = "../../../tests/unit/projects/gui/lanzar.rs"]
mod tests;
