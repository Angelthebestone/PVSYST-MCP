//! What UI Automation adds to the Win32 inventory. Win32 knows each page of a
//! page control by its caption but not where the page sits in the tab strip;
//! UI Automation lists the tab items in strip order.

use crate::adapters::gui::ffi::Hwnd;
use crate::adapters::gui::uia_vista::aplanar;
use crate::adapters::uia;
use crate::error::Result;
use crate::state::Estado;
use serde_json::Value;

const PROFUNDIDAD: u32 = 4;
const MAX_NODOS: u32 = 100;
const ROL_PESTANA: &str = "tab item";

/// Tab captions of the form `h` in the order of the strip.
pub(super) fn nombres_de_pestanas(estado: &Estado, h: Hwnd) -> Result<Vec<String>> {
    let arbol = uia::arbol(estado, h, PROFUNDIDAD, MAX_NODOS)?;
    let mut plano = Vec::new();
    aplanar(&arbol, &mut plano);
    Ok(plano
        .into_iter()
        .filter(|n| n["rol"].as_str() == Some(ROL_PESTANA))
        .map(|n| {
            n.get("nombre")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        })
        .collect())
}
