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
use super::captura::{rgb_de_control, ImagenRgb};
use super::conduccion::formularios;
use super::escritorio;
use super::ffi::{self, Hwnd, GA_ROOT};
use super::modal;
use super::ventanas::{class_of, hwnd_de, pid_of, text_of, visible};
use crate::args::{missing, opt_u32};
use crate::error::{Error, Result};
use banda::Capturas;
use error::ErrorFila;
use geometria::Geometria;
use registro::Control;
use serde_json::{json, Value};
use std::time::Duration;

/// Time PVsyst needs to repaint the selection after the click.
const REPINTADO: Duration = Duration::from_millis(250);
/// Time Windows needs to finish bringing a window to the front.
const ESPERA_PRIMER_PLANO: Duration = Duration::from_millis(80);
/// Hint windows come and go with the mouse: they are not what the click did.
const CLASE_AVISO: &str = "THintWindow";

/// What is being selected: the control, its measured geometry and the row.
struct Pedido<'a> {
    h: Hwnd,
    geom: &'a Geometria,
    indice: usize,
}

type VentanaConClase = (Hwnd, String);

fn indice_de(args: &Value) -> Result<usize> {
    let indice = opt_u32(args, "indice")?.ok_or_else(|| {
        missing(
            "indice",
            "fila a seleccionar, desde 0, dentro de las filas visibles",
        )
    })?;
    Ok(usize::try_from(indice).unwrap_or(usize::MAX))
}

/// Windows of the process of `h`, with their class: what a click must not change.
fn ventanas_del_proceso(h: Hwnd) -> Vec<VentanaConClase> {
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
fn exigir_sin_activacion(p: &Pedido<'_>, antes: &[VentanaConClase]) -> Result<()> {
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
fn exigir_banda_en(p: &Pedido<'_>, antes: &ImagenRgb) -> Result<()> {
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
fn exigir_foco(h: Hwnd) -> Result<()> {
    if ffi::traer_y_focar(h, ESPERA_PRIMER_PLANO) {
        return Ok(());
    }
    let clase = class_of(ffi::get_ancestor(h, GA_ROOT));
    Err(ErrorFila::SinFoco { clase }.a_error())
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

/// Geometry declared for this very control: its class and the window hosting it.
fn geometria_del_control(h: Hwnd) -> Result<&'static Geometria> {
    let raiz = ffi::get_ancestor(h, GA_ROOT);
    let (clase, anfitrion, titulo) = (class_of(h), class_of(raiz), text_of(raiz));
    let control = Control {
        clase: &clase,
        anfitrion: &anfitrion,
        titulo: &titulo,
    };
    registro::geometria_de(&control).map_err(ErrorFila::a_error)
}

pub(crate) fn fila(args: &Value) -> Result<Value> {
    exigir_escritorio_medido()?;
    let h = hwnd_de(args, "ventana")?;
    let indice = indice_de(args)?;
    modal::exigir_habilitada(h, modal::PARA_ACTUAR)?;
    let geom = geometria_del_control(h)?;
    let antes = rgb_de_control(h)?;
    let (x, dy) = geom
        .punto_de_fila(indice, (antes.ancho_px(), antes.alto_px()))
        .map_err(ErrorFila::a_error)?;
    let ventanas_antes = ventanas_del_proceso(h);
    exigir_foco(h)?;
    clic_fisico_en_control(h, Some((x, dy)))?;
    std::thread::sleep(REPINTADO);
    let pedido = Pedido { h, geom, indice };
    exigir_sin_activacion(&pedido, &ventanas_antes)?;
    exigir_banda_en(&pedido, &antes)?;
    Ok(json!({
        "fila": indice,
        "leida": indice,
        "hwnd": format!("0x{h:X}"),
        "clase": geom.clase,
        "dy_px96": dy,
        "x_px96": x,
        "dpi": ffi::dpi_de(h),
        "verificado": "banda_resaltada",
    }))
}
