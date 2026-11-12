//! `gui_observar que:inventario`: every control of a form, hidden ones and tab
//! pages that are not shown included, as the machine view of the GUI. It lists
//! what exists; `que:formulario` is the human view (labels read by OCR).
//!
//! Win32 gives each window; UI Automation adds the tab order, the patterns each
//! control supports and the elements a control draws itself. Both land in one
//! list, each row saying which API saw it (`fuente`).

pub(crate) mod dominio;
mod etiquetas;
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
use tipos::{Inventario, LecturaUia};

/// Most controls one answer carries; `total` and `truncado` say what was left out.
pub(crate) const TOPE_CONTROLES: usize = 500;

/// Win32 rows first, then the elements only UI Automation sees.
fn filas(inventario: &Inventario) -> Vec<Value> {
    let win32 = inventario.controles.iter().map(|c| json!(c));
    let solo_uia = inventario.solo_uia.iter().map(|c| json!(c));
    win32.chain(solo_uia).collect()
}

fn respuesta(h: ffi::Hwnd, inventario: &Inventario, uia_error: Option<&str>) -> Value {
    let controles = filas(inventario);
    let total = controles.len();
    json!({
        "ventana": format!("0x{h:X}"),
        "titulo": text_of(h),
        "clase": class_of(h),
        "dpi": ffi::dpi_de(h),
        "pestanas": inventario.pestanas,
        "controles": controles.into_iter().take(TOPE_CONTROLES).collect::<Vec<_>>(),
        "total": total,
        "truncado": total > TOPE_CONTROLES,
        "uia_error": uia_error,
    })
}

/// The UI Automation reading, or nothing and why: `incluir_uia:false` skips it
/// and a failure leaves the Win32 inventory, which stands on its own.
fn lectura_uia(paths: &Paths, h: ffi::Hwnd, args: &Value) -> (LecturaUia, Option<String>) {
    if args.get("incluir_uia").and_then(Value::as_bool) == Some(false) {
        return (LecturaUia::default(), None);
    }
    match uia::leer(&paths.estado, h, args) {
        Ok(lectura) => (lectura, None),
        Err(e) => (LecturaUia::default(), Some(e.to_string())),
    }
}

/// The inventory of the target form. Hidden controls are listed unless
/// `todos:false`: the point of an inventory is what exists, not what shows.
/// `incluir_uia:false` skips UI Automation; `profundidad` and `max_nodos`
/// bound its tree.
pub(crate) fn inventario(paths: &Paths, args: &Value) -> Result<Value> {
    ffi::declarar_dpi_por_monitor();
    let h = ventana_objetivo(args)?;
    let todos = args.get("todos").and_then(Value::as_bool).unwrap_or(true);
    let (lectura, uia_error) = lectura_uia(paths, h, args);
    let instantanea = win32::instantanea_de(h, todos, lectura);
    Ok(respuesta(
        h,
        &inventariar(&instantanea),
        uia_error.as_deref(),
    ))
}
