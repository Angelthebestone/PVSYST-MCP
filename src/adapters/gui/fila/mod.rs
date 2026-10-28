//! `gui_actuar accion:fila`: selects row `indice` of a list or tree whose rows
//! have no window of their own. It only selects: it never presses Enter or OK.
//!
//! The click is a physical one on the declared geometry, and the result is
//! checked without OCR: the highlighted band of a capture must be the requested
//! row, and the click must not have opened or closed anything. A click that
//! PVsyst took as "open this element" is reported, never hidden.

mod banda;
mod error;
mod geometria;
mod registro;

use super::acciones::{clic_fisico_en_control, exigir_escritorio};
use super::captura::rgb_de_control;
use super::conduccion::formularios;
use super::escritorio;
use super::ffi::{self, Hwnd, GA_ROOT};
use super::modal;
use super::ventanas::{class_of, hwnd_de, pid_of, visible};
use crate::args::{missing, opt_u32};
use crate::error::{Error, Result};
use error::ErrorFila;
use geometria::Geometria;
use serde_json::{json, Value};
use std::time::Duration;

/// Time PVsyst needs to repaint the selection after the click.
const REPINTADO: Duration = Duration::from_millis(250);

fn indice_de(args: &Value) -> Result<usize> {
    let indice = opt_u32(args, "indice")?.ok_or_else(|| {
        missing(
            "indice",
            "fila a seleccionar, desde 0, dentro de las filas visibles",
        )
    })?;
    Ok(usize::try_from(indice).unwrap_or(usize::MAX))
}

/// Windows of the process of `h`: what a click must not add.
fn ventanas_del_proceso(h: Hwnd) -> Vec<Hwnd> {
    formularios(pid_of(h))
}

fn clases_nuevas(antes: &[Hwnd], h: Hwnd) -> Vec<String> {
    ventanas_del_proceso(h)
        .into_iter()
        .filter(|v| !antes.contains(v))
        .map(class_of)
        .collect()
}

/// The click must not have closed the control or opened a window: that is PVsyst
/// taking the click as "open this element".
fn exigir_sin_activacion(h: Hwnd, pedida: usize, antes: &[Hwnd]) -> Result<()> {
    let clase = class_of(h);
    if !ffi::es_ventana(h) || !visible(h) {
        return Err(ErrorFila::ControlCerrado {
            indice: pedida,
            clase,
        }
        .a_error());
    }
    let nuevas = clases_nuevas(antes, h);
    if !nuevas.is_empty() {
        let clases = nuevas.join(", ");
        return Err(ErrorFila::Activada {
            indice: pedida,
            clases,
        }
        .a_error());
    }
    Ok(())
}

/// The highlighted band must be exactly the requested row.
fn exigir_banda_en(h: Hwnd, geom: &Geometria, pedida: usize) -> Result<()> {
    let filas = banda::filas_resaltadas(&rgb_de_control(h)?, geom);
    match filas.as_slice() {
        [leida] if *leida == pedida => Ok(()),
        [leida] => Err(ErrorFila::FilaDistinta {
            pedida,
            leida: *leida,
        }
        .a_error()),
        [] => Err(ErrorFila::NingunaResaltada { indice: pedida }.a_error()),
        _ => Err(ErrorFila::Ambigua { filas }.a_error()),
    }
}

/// With the focus on another control (a combo) the first click only moves the
/// focus and the row stays as it was. A second click would be a double click,
/// which opens the element, so the focus goes first. Best effort: the capture
/// check that follows tells whether the click did select.
fn prepararse_para_el_clic(h: Hwnd) {
    ffi::traer_al_frente(ffi::get_ancestor(h, GA_ROOT));
    ffi::dar_foco(h);
}

fn exigir_escritorio_medido() -> Result<()> {
    exigir_escritorio()?;
    if escritorio::es_aislado() {
        return Err(Error::Gui(
            "fila usa un clic físico, que no está medido en el escritorio aislado: usa accion:clic_texto o PVSYST_MCP_ESCRITORIO sin aislar".into(),
        ));
    }
    Ok(())
}

pub(crate) fn fila(_paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    exigir_escritorio_medido()?;
    let h = hwnd_de(args, "ventana")?;
    let indice = indice_de(args)?;
    modal::exigir_habilitada(h, modal::PARA_ACTUAR)?;
    let geom = registro::geometria_de(&class_of(h)).map_err(ErrorFila::a_error)?;
    let dy = geom
        .dy_de_fila(indice, rgb_de_control(h)?.alto_px())
        .map_err(ErrorFila::a_error)?;
    let antes = ventanas_del_proceso(h);
    prepararse_para_el_clic(h);
    clic_fisico_en_control(h, Some((geom.x_px, dy)))?;
    std::thread::sleep(REPINTADO);
    exigir_sin_activacion(h, indice, &antes)?;
    exigir_banda_en(h, geom, indice)?;
    Ok(json!({
        "fila": indice,
        "leida": indice,
        "hwnd": format!("0x{h:X}"),
        "clase": geom.clase,
        "dy_px96": dy,
        "x_px96": geom.x_px,
        "dpi": ffi::dpi_de(h),
        "verificado": "banda_resaltada",
    }))
}
