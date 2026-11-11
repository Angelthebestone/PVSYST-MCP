//! `gui_observar que:inventario`: every control of a form, hidden ones and tab
//! pages that are not shown included, as the machine view of the GUI. It lists
//! what exists; `que:formulario` is the human view (labels read by OCR).

pub(crate) mod dominio;
pub(crate) mod tipos;
mod uia;
pub(crate) mod win32;

use crate::adapters::gui::ffi;
use crate::adapters::gui::ocr::formulario::ventana_objetivo;
use crate::adapters::gui::ventanas::{class_of, text_of};
use crate::config::Paths;
use crate::error::Result;
use dominio::inventariar;
use serde_json::{json, Value};
use tipos::Inventario;

/// Most controls one answer carries; `total` and `truncado` say what was left out.
pub(crate) const TOPE_CONTROLES: usize = 500;

fn respuesta(h: ffi::Hwnd, inventario: Inventario, uia_error: Option<&str>) -> Value {
    let total = inventario.controles.len();
    let controles: Vec<_> = inventario
        .controles
        .into_iter()
        .take(TOPE_CONTROLES)
        .collect();
    json!({
        "ventana": format!("0x{h:X}"),
        "titulo": text_of(h),
        "clase": class_of(h),
        "dpi": ffi::dpi_de(h),
        "pestanas": inventario.pestanas,
        "controles": controles,
        "total": total,
        "truncado": total > TOPE_CONTROLES,
        "uia_error": uia_error,
    })
}

/// The inventory of the target form. Hidden controls are listed unless
/// `todos:false`: the point of an inventory is what exists, not what shows.
/// `incluir_uia:false` skips the UI Automation read of the tab strip.
pub(crate) fn inventario(paths: &Paths, args: &Value) -> Result<Value> {
    ffi::declarar_dpi_por_monitor();
    let h = ventana_objetivo(args)?;
    let todos = args.get("todos").and_then(Value::as_bool).unwrap_or(true);
    let usa_uia = args.get("incluir_uia").and_then(Value::as_bool) != Some(false);
    let (pestanas_uia, uia_error) = if usa_uia {
        match uia::nombres_de_pestanas(&paths.estado, h) {
            Ok(nombres) => (nombres, None),
            Err(e) => (Vec::new(), Some(e.to_string())),
        }
    } else {
        (Vec::new(), None)
    };
    let instantanea = win32::instantanea_de(h, todos, pestanas_uia);
    Ok(respuesta(
        h,
        inventariar(&instantanea),
        uia_error.as_deref(),
    ))
}
