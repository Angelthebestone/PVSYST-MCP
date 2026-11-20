//! What the click must prove: it did not open or close a window and the
//! highlighted band is the requested row. The band says a row is selected, not
//! which element it holds, so with `texto` the row is also read by OCR. The
//! focus is checked before the click: on a control that does not have it, the
//! first click only moves the focus and a second one would open the element.

use super::super::captura::{rgb_de_control, ImagenRgb};
use super::super::conduccion::formularios;
use super::super::ventanas::{class_of, pid_of, visible};
use super::banda::{self, Capturas};
use super::error::ErrorFila;
use super::ffi::{self, Hwnd, GA_ROOT};
use super::lectura;
use super::Pedido;
use crate::error::Result;
use serde_json::Value;
use std::time::Duration;

/// Time Windows needs to finish bringing a window to the front.
const ESPERA_PRIMER_PLANO: Duration = Duration::from_millis(80);
/// Hint windows come and go with the mouse: they are not what the click did.
const CLASE_AVISO: &str = "THintWindow";

pub(super) type VentanaConClase = (Hwnd, String);

/// Windows of the process of `h`, with their class: what a click must not change.
pub(super) fn ventanas_del_proceso(h: Hwnd) -> Vec<VentanaConClase> {
    formularios(pid_of(h))
        .into_iter()
        .map(|v| (v, class_of(v)))
        .filter(|(_, clase)| clase != CLASE_AVISO)
        .collect()
}

/// Classes of the windows of `de` that are not in `contra`.
fn clases_de_las_que_faltan(de: &[VentanaConClase], contra: &[VentanaConClase]) -> String {
    de.iter()
        .filter(|(v, _)| !contra.iter().any(|(c, _)| c == v))
        .map(|(_, clase)| clase.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// The click must not have closed the control, opened a window or closed one:
/// that is PVsyst taking the click as an action on the element.
pub(super) fn exigir_sin_activacion(p: &Pedido<'_>, antes: &[VentanaConClase]) -> Result<()> {
    if !ffi::es_ventana(p.h) || !visible(p.h) {
        let clase = class_of(p.h);
        return Err(ErrorFila::ControlCerrado {
            indice: p.indice,
            clase,
        }
        .a_error());
    }
    let despues = ventanas_del_proceso(p.h);
    let abiertas = clases_de_las_que_faltan(&despues, antes);
    if !abiertas.is_empty() {
        let (indice, clases) = (p.indice, abiertas);
        return Err(ErrorFila::Activada { indice, clases }.a_error());
    }
    let cerradas = clases_de_las_que_faltan(antes, &despues);
    if !cerradas.is_empty() {
        let (indice, clases) = (p.indice, cerradas);
        return Err(ErrorFila::VentanaCerrada { indice, clases }.a_error());
    }
    Ok(())
}

/// The band that appeared (or stayed) must be exactly the requested row.
pub(super) fn exigir_banda_en(p: &Pedido<'_>, antes: &ImagenRgb) -> Result<()> {
    let despues = rgb_de_control(p.h)?;
    let capturas = Capturas {
        antes,
        despues: &despues,
    };
    let filas = banda::filas_seleccionadas(&capturas, p.geom);
    match filas.as_slice() {
        [leida] if *leida == p.indice => Ok(()),
        [leida] => Err(ErrorFila::FilaDistinta {
            pedida: p.indice,
            leida: *leida,
        }
        .a_error()),
        [] => Err(ErrorFila::NingunaResaltada { indice: p.indice }.a_error()),
        _ => Err(ErrorFila::Ambigua { filas }.a_error()),
    }
}

/// With the focus on another control (a combo) the first click only moves the
/// focus and the row stays as it was, and a second click would be a double click
/// that opens the element: so the click is only sent once the window is in front
/// and the control has the focus.
pub(super) fn exigir_foco(h: Hwnd) -> Result<()> {
    if ffi::traer_y_focar(h, ESPERA_PRIMER_PLANO) {
        return Ok(());
    }
    let clase = class_of(ffi::get_ancestor(h, GA_ROOT));
    Err(ErrorFila::SinFoco { clase }.a_error())
}

/// Row text the caller expects, read by OCR and compared. `None` when none was asked.
pub(super) fn exigir_texto_pedido(args: &Value, p: &Pedido<'_>) -> Result<Option<String>> {
    let Some(pedido) = args.get("texto").and_then(Value::as_str) else {
        return Ok(None);
    };
    let leido = lectura::leer(p.h, p.geom, p.indice).map_err(ErrorFila::a_error)?;
    lectura::exigir_texto(&leido, pedido, p.indice).map_err(ErrorFila::a_error)?;
    Ok(Some(leido))
}
