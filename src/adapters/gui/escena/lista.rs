//! Positions of the objects: PVsyst's «Lista y gestión de objetos» table has
//! an editable X, Y, Z and azimuth cell per object. A cell is clicked, its
//! inline editor (`TVTEdit`) typed into and committed with Tab.

use super::pasos::{abrir, escribir, hijo_n, hijos, ventana};
use crate::adapters::gui::conduccion::Formulario;
use crate::adapters::gui::ffi::{self, Hwnd};
use crate::adapters::gui::sondeo::sondear_hasta;
use crate::adapters::gui::ventanas::rect_of;
use crate::error::{Error, Result};
use crate::escena_3d::ObjetoPedido;
use std::time::Duration;

const LISTA: &str = "TShdObjectSelectionForm";
/// Offsets (px at 96 DPI) from the top-left corner of the table, read from captures
/// of PVsyst 8.1.8 (a capture is in 96-DPI px): the first row's centre and the row pitch.
const PRIMERA_FILA_PX: i32 = 45;
const PASO_FILA_PX: i32 = 26;
const COLUMNA_X_PX: i32 = 389;
const COLUMNA_Y_PX: i32 = 444;
const COLUMNA_Z_PX: i32 = 499;
const COLUMNA_AZIMUT_PX: i32 = 554;
/// Rows the table shows without scrolling; further rows are out of reach of a click.
pub(super) const MAX_FILAS_VISIBLES: usize = 20;
const ESPERA_EDITOR: Duration = Duration::from_secs(2);
const SONDEO: Duration = Duration::from_millis(100);

/// Sets the position of the objects, row `primera + i` for `objetos[i]`.
/// Values left at 0 are not typed: a new object already sits at the origin.
pub(super) fn fijar(objetos: &[ObjetoPedido], primera: usize) -> Result<()> {
    if primera + objetos.len() > MAX_FILAS_VISIBLES {
        return Err(Error::Invalid(format!(
            "La lista de objetos muestra {MAX_FILAS_VISIBLES} filas sin desplazarse: pide menos objetos"
        )));
    }
    abrir("escena_3d.herramientas.lista_de_objetos")?;
    let lista = Formulario::de(ventana(LISTA)?);
    let resultado = objetos
        .iter()
        .enumerate()
        .try_for_each(|(i, o)| fijar_objeto(&lista, primera + i, o));
    let cierre = lista.pulsar("Cerrar");
    resultado.and(cierre)
}

fn fijar_objeto(lista: &Formulario, fila: usize, objeto: &ObjetoPedido) -> Result<()> {
    let p = objeto.posicion;
    let celdas = [
        (COLUMNA_X_PX, p.x),
        (COLUMNA_Y_PX, p.y),
        (COLUMNA_Z_PX, p.z),
        (COLUMNA_AZIMUT_PX, p.rotacion_azimut_deg),
    ];
    celdas
        .into_iter()
        .filter(|(_, valor)| *valor != 0.0)
        .try_for_each(|(columna, valor)| celda(lista, (columna, fila), valor))
}

/// Types `valor` in the cell at (`columna` px, `fila`). The editor typed into is the
/// one under the click: the previous cell's editor may still exist for a moment.
fn celda(lista: &Formulario, (columna, fila): (i32, usize), valor: f64) -> Result<()> {
    let fila = i32::try_from(fila).map_err(|_| Error::Invalid("Fila fuera de rango".into()))?;
    let dy = PRIMERA_FILA_PX + fila * PASO_FILA_PX;
    let tabla = hijo_n(lista.h, "TVirtualStringTree", 0)?;
    let (x, y) = punto_fisico(tabla, (columna, dy));
    lista.clic_en_unico("TVirtualStringTree", (columna, dy))?;
    let editor =
        sondear_hasta(ESPERA_EDITOR, SONDEO, || editor_en(lista.h, (x, y))).ok_or_else(|| {
            Error::Gui(format!(
                "PVsyst no abrió el editor de la celda (columna {columna}, fila {fila})"
            ))
        })?;
    escribir(lista.h, editor, &valor.to_string())
}

/// Screen point at (`dx`, `dy`) px at 96 DPI from the corner of `tabla`.
fn punto_fisico(tabla: Hwnd, (dx, dy): (i32, i32)) -> (i32, i32) {
    let (r, dpi) = (rect_of(tabla), ffi::dpi_de(tabla));
    (
        r.left + ffi::escalar_a_dpi(dx, dpi),
        r.top + ffi::escalar_a_dpi(dy, dpi),
    )
}

/// The cell editor (`TVTEdit`) that covers the screen point `(x, y)`.
fn editor_en(lista: Hwnd, (x, y): (i32, i32)) -> Option<Hwnd> {
    hijos(lista, "TVTEdit").into_iter().find(|e| {
        let r = rect_of(*e);
        (r.left..=r.right).contains(&x) && (r.top..=r.bottom).contains(&y)
    })
}
