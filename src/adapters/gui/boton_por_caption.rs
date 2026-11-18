//! The button of a form whose own caption is the text a caller asked for.
//!
//! A caption on a button is the place a person means, even when a combo box or
//! an edit shows the same word, so this search runs before the drawn text. Two
//! buttons with the same caption are refused, never guessed: the error names each
//! one by class, hwnd and position.

use super::ffi::{self, Hwnd};
use super::inventario::tipos::Rol;
use super::ocr::ubicacion::plegar;
use super::ventanas::{class_of, rect_of, visible};
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};

/// Buttons the error names before cutting the list.
const BOTONES_EN_ERROR: usize = 8;

/// A visible button of the form whose own caption is the requested text.
struct Coincidencia {
    hwnd: Hwnd,
    clase: String,
    punto: (i32, i32),
}

/// Centre of the only button of the form whose own caption is `texto`; `None`
/// when no button carries it, so the caller reads the drawn text instead.
pub(super) fn boton_con_caption(
    h: Hwnd,
    texto: &str,
    idioma: Idioma,
) -> Result<Option<(i32, i32)>> {
    let botones = botones_con_texto(h, texto);
    match botones.as_slice() {
        [] => Ok(None),
        [unico] => Ok(Some(unico.punto)),
        varios => Err(botones_ambiguos(texto, varios, idioma)),
    }
}

/// Every visible button of `h` whose own caption is `texto`, folded as OCR folds
/// it: `&Abrir`, `Abrir` and `abrir` are the same caption. Top to bottom, then
/// left to right; `EnumChildWindows` lists every descendant. The class is read
/// before the caption, so only the buttons of the form cost a message.
fn botones_con_texto(h: Hwnd, texto: &str) -> Vec<Coincidencia> {
    let aguja = plegar(texto);
    if aguja.is_empty() {
        return Vec::new();
    }
    let mut encontrados: Vec<Coincidencia> = ffi::enum_children(h)
        .into_iter()
        .filter(|c| visible(*c) && Rol::de_clase(&class_of(*c)) == Rol::Boton)
        .filter(|c| plegar(&ffi::texto_control(*c)) == aguja)
        .map(|c| {
            let r = rect_of(c);
            Coincidencia {
                hwnd: c,
                clase: class_of(c),
                punto: ((r.left + r.right) / 2, (r.top + r.bottom) / 2),
            }
        })
        .collect();
    encontrados.sort_by_key(|c| (c.punto.1, c.punto.0));
    encontrados
}

const CAPTION_COMPARTIDO: Texto = Texto {
    es: "'{texto}' es el rótulo de {n} botones ({lista}): elige uno por su hwnd con gui_actuar accion:click ventana:<hwnd>, o pasa 'indice' para pulsar el texto dibujado",
    en: "'{texto}' is the caption of {n} buttons ({lista}): choose one by its hwnd with gui_actuar accion:click ventana:<hwnd>, or pass 'indice' to click the drawn text",
};

/// The refusal for a caption that names several buttons: each with its class and
/// position, so the caller picks one instead of the server pressing a wrong one.
fn botones_ambiguos(texto: &str, botones: &[Coincidencia], idioma: Idioma) -> Error {
    let mut lista = botones
        .iter()
        .take(BOTONES_EN_ERROR)
        .map(|c| format!("0x{:X} {} ({}, {})", c.hwnd, c.clase, c.punto.0, c.punto.1))
        .collect::<Vec<_>>()
        .join("; ");
    if botones.len() > BOTONES_EN_ERROR {
        lista.push_str("; …");
    }
    let total = botones.len().to_string();
    Error::Gui(CAPTION_COMPARTIDO.texto_con(
        idioma,
        &[("texto", texto), ("n", &total), ("lista", &lista)],
    ))
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/boton_por_caption.rs"]
mod tests;
