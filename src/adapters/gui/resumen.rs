//! Compact summary of a form after `funcion`: its visible buttons, tab count
//! and field captions. Pure over the `window_brief` JSON so it is testable
//! without a window.

use super::ffi::{self, Hwnd};
use super::ventanas::window_brief;
use serde_json::{json, Value};

const CLASES_PESTANAS: &[&str] = &["TPageControl", "TabControl"];
const CLASES_CAMPO: &[&str] = &["Edit", "ComboBox", "CheckBox", "Memo"];

fn clase(c: &Value) -> &str {
    c.get("class").and_then(Value::as_str).unwrap_or("")
}

fn titulo(c: &Value) -> &str {
    c.get("title").and_then(Value::as_str).unwrap_or("")
}

fn visible(c: &Value) -> bool {
    c.get("visible").and_then(Value::as_bool) == Some(true)
}

/// Captions of the visible controls whose class matches `clases`, in order,
/// skipping blank captions.
fn rotulos(visibles: &[&Value], clases: &[&str]) -> Vec<String> {
    visibles
        .iter()
        .filter(|c| clases.iter().any(|k| clase(c).contains(k)))
        .map(|c| titulo(c))
        .filter(|t| !t.is_empty())
        .map(String::from)
        .collect()
}

/// Summary of a form from its child controls (`window_brief` JSON).
pub(crate) fn resumen_formulario(controles: &[Value]) -> Value {
    let visibles: Vec<&Value> = controles.iter().filter(|c| visible(c)).collect();
    let pestanas = visibles
        .iter()
        .filter(|c| CLASES_PESTANAS.iter().any(|k| clase(c).contains(k)))
        .count();
    json!({
        "botones": rotulos(&visibles, &["Button"]),
        "pestanas": pestanas,
        "campos": rotulos(&visibles, CLASES_CAMPO),
    })
}

/// Summary of the form `h`, read from its direct children.
pub(crate) fn resumen_de_ventana(h: Hwnd) -> Value {
    let controles: Vec<Value> = ffi::enum_children(h)
        .into_iter()
        .map(window_brief)
        .collect();
    resumen_formulario(&controles)
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/resumen.rs"]
mod tests;
