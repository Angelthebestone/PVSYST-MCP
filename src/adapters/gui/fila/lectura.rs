//! The text of a row. The highlighted band proves that a row is selected, not
//! which element it holds: row 0 of the project tree is highlighted whether it
//! is the project the caller wanted or another one. Reading the row with OCR and
//! comparing it with the text the caller expects closes that gap.

use super::error::ErrorFila;
use super::geometria::Geometria;
use crate::adapters::gui::ffi::{self, Hwnd};
use crate::adapters::gui::ocr::captura_ocr::lineas_vivas;
use crate::adapters::gui::ocr::dominio::LineaOcr;
use crate::adapters::gui::ocr::ubicacion::plegar;
use crate::adapters::gui::ventanas::rect_of;

/// Vertical extent of row `indice` on screen: `(top, bottom)` in physical px.
pub(super) fn banda_en_pantalla(
    geom: &Geometria,
    indice: usize,
    (origen_y, dpi): (i32, u32),
) -> (i32, i32) {
    let arriba = geom.centro_de_fila(indice) - geom.alto_fila_px / 2;
    (
        origen_y + ffi::escalar_a_dpi(arriba, dpi),
        origen_y + ffi::escalar_a_dpi(arriba + geom.alto_fila_px, dpi),
    )
}

/// The text of the lines whose centre lies in `banda`, from left to right.
pub(super) fn texto_en_banda(lineas: &[LineaOcr], (arriba, abajo): (i32, i32)) -> String {
    let mut en_fila: Vec<&LineaOcr> = lineas
        .iter()
        .filter(|l| (arriba..abajo).contains(&l.rect.centro_y()))
        .collect();
    en_fila.sort_by_key(|l| l.rect.left);
    en_fila
        .iter()
        .map(|l| l.texto.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Reads row `indice` of control `h` by OCR. Fails when OCR cannot read the
/// control or finds no text in the row: a row that cannot be read cannot be verified.
pub(super) fn leer(h: Hwnd, geom: &Geometria, indice: usize) -> Result<String, ErrorFila> {
    let lineas = lineas_vivas(h).map_err(|e| ErrorFila::SinLectura {
        indice,
        motivo: e.to_string(),
    })?;
    let banda = banda_en_pantalla(geom, indice, (rect_of(h).top, ffi::dpi_de(h)));
    let leido = texto_en_banda(&lineas, banda);
    if plegar(&leido).is_empty() {
        return Err(ErrorFila::SinLectura {
            indice,
            motivo: "OCR found no text in the row".into(),
        });
    }
    Ok(leido)
}

/// The row must hold the text the caller expects: case, accents, spaces and
/// punctuation are ignored, the way OCR reads them.
pub(super) fn exigir_texto(leido: &str, pedido: &str, indice: usize) -> Result<(), ErrorFila> {
    if plegar(leido).contains(&plegar(pedido)) {
        return Ok(());
    }
    Err(ErrorFila::TextoDistinto {
        indice,
        pedido: pedido.to_string(),
        leido: leido.to_string(),
    })
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/fila/lectura.rs"]
mod tests;
