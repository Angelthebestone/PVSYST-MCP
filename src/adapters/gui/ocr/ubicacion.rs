//! Finds where a piece of text is drawn, so a click can land on a control that
//! has no window handle (a speed button, a grid row, a tree node). Pure.

use super::dominio::{LineaOcr, PalabraOcr};
use crate::adapters::gui::ffi::Rect;
use crate::error::{Error, Result};

/// Candidates listed in the error when the text is drawn more than once.
const CANDIDATAS_EN_ERROR: usize = 8;

/// Lower-case letters and digits only, with Spanish diacritics folded: OCR
/// drops accents and punctuation unpredictably.
pub(crate) fn plegar(texto: &str) -> String {
    texto
        .chars()
        .flat_map(char::to_lowercase)
        .map(sin_acento)
        .filter(|c| c.is_alphanumeric())
        .collect()
}

fn sin_acento(c: char) -> char {
    match c {
        'á' | 'à' | 'ä' => 'a',
        'é' | 'è' | 'ë' => 'e',
        'í' | 'ì' | 'ï' => 'i',
        'ó' | 'ò' | 'ö' => 'o',
        'ú' | 'ù' | 'ü' => 'u',
        'ñ' => 'n',
        otra => otra,
    }
}

/// Boxes of the places where `aguja` is drawn: for every line, the union of the
/// words that spell it. Matching ignores case, accents, spaces and punctuation
/// and may start or end inside a word. Reading order (top to bottom, then left
/// to right). O(letters + matches·log words) per line.
pub(crate) fn ubicar_texto(lineas: &[LineaOcr], aguja: &str) -> Vec<Rect> {
    let aguja = plegar(aguja);
    if aguja.is_empty() {
        return Vec::new();
    }
    let mut cajas: Vec<Rect> = lineas
        .iter()
        .flat_map(|l| coincidencias(&l.palabras, &aguja))
        .collect();
    cajas.sort_by_key(|r| (r.top, r.left));
    cajas
}

/// The folded words of a line glued together, and where each one starts.
fn texto_plegado(palabras: &[PalabraOcr]) -> (String, Vec<usize>) {
    let plegadas: Vec<String> = palabras.iter().map(|p| plegar(&p.texto)).collect();
    let inicios = plegadas
        .iter()
        .scan(0, |acumulado, palabra| {
            let inicio = *acumulado;
            *acumulado += palabra.len();
            Some(inicio)
        })
        .collect();
    (plegadas.concat(), inicios)
}

fn coincidencias(palabras: &[PalabraOcr], aguja: &str) -> Vec<Rect> {
    let (texto, inicios) = texto_plegado(palabras);
    texto
        .match_indices(aguja)
        .filter_map(|(desde, _)| {
            let hasta = desde + aguja.len();
            let primera = inicios.partition_point(|i| *i <= desde).checked_sub(1)?;
            let ultima = inicios.partition_point(|i| *i < hasta).checked_sub(1)?;
            palabras
                .get(primera..=ultima)?
                .iter()
                .map(|p| p.rect)
                .reduce(|a, b| a.union(&b))
        })
        .collect()
}

fn centro(r: &Rect) -> (i32, i32) {
    ((r.left + r.right) / 2, (r.top + r.bottom) / 2)
}

fn descripcion(cajas: &[Rect]) -> String {
    cajas
        .iter()
        .take(CANDIDATAS_EN_ERROR)
        .enumerate()
        .map(|(i, r)| format!("{i}: ({}, {})", centro(r).0, centro(r).1))
        .collect::<Vec<_>>()
        .join("; ")
}

/// The box to click: `indice` when given, the only one otherwise. Several
/// matches without `indice` are refused instead of guessed.
pub(crate) fn elegir_caja<'a>(
    cajas: &'a [Rect],
    indice: Option<usize>,
    texto: &str,
) -> Result<&'a Rect> {
    match (indice, cajas) {
        (_, []) => Err(Error::NotFound(format!(
            "'{texto}' no está dibujado en la ventana (el OCR puede haberlo leído mal: gui_observar que:capturar)"
        ))),
        (Some(i), _) => cajas.get(i).ok_or_else(|| {
            Error::Invalid(format!("indice fuera de rango (0..{})", cajas.len()))
        }),
        (None, [unica]) => Ok(unica),
        (None, varias) => Err(Error::Invalid(format!(
            "'{texto}' aparece {} veces; elige con 'indice': {}",
            varias.len(),
            descripcion(varias)
        ))),
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/ocr/ubicacion.rs"]
mod tests;
