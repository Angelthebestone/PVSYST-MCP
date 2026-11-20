//! The button of a form whose own caption is the text a caller asked for.
//!
//! A caption on a button is the place a person means, even when a combo box or
//! an edit shows the same word, so this search runs before the drawn text. Two
//! enabled buttons with the same caption are refused, never guessed: the error
//! names each one by class, hwnd and position. A caption only disabled buttons
//! carry is refused as well, because a disabled control swallows the click; when
//! an enabled button shares the caption with disabled ones, the enabled button is
//! the one pressed and the disabled ones are reported as discarded.

use super::ffi::{self, Hwnd};
use super::inventario::tipos::Rol;
use super::ocr::ubicacion::plegar;
use super::ventanas::{class_of, rect_of, visible};
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use serde_json::{json, Value};

/// Buttons the error names before cutting the list.
const BOTONES_EN_ERROR: usize = 8;

/// A visible button of the form whose own caption is the requested text.
struct Coincidencia {
    hwnd: Hwnd,
    clase: String,
    punto: (i32, i32),
}

/// The button a click presses: its centre and the disabled buttons that carry
/// the same caption, which the click skipped and the answer names.
#[derive(Debug)]
pub(super) struct BotonElegido {
    pub(super) punto: (i32, i32),
    pub(super) descartados: Vec<Value>,
}

/// The only enabled button of the form whose own caption is `texto`; `None` when
/// no button carries it, so the caller reads the drawn text instead. A caption
/// only disabled buttons carry is refused: a disabled control swallows the click
/// without a trace (GUI-07) and would answer a dead `verificado:false`, hiding
/// the caption the caller asked about.
pub(super) fn boton_con_caption(
    h: Hwnd,
    texto: &str,
    idioma: Idioma,
) -> Result<Option<BotonElegido>> {
    let (habilitados, deshabilitados): (Vec<_>, Vec<_>) = botones_con_texto(h, texto)
        .into_iter()
        .partition(|c| ffi::estado_ventana(c.hwnd).habilitada);
    match habilitados.as_slice() {
        [] if deshabilitados.is_empty() => Ok(None),
        [] => Err(botones_deshabilitados(texto, &deshabilitados, idioma)),
        [unico] => Ok(Some(BotonElegido {
            punto: unico.punto,
            descartados: descartados_json(&deshabilitados),
        })),
        varios => Err(botones_ambiguos(texto, varios, idioma)),
    }
}

/// The buttons the click skips, each with its class, hwnd and centre, so the
/// answer says which control was left alone; empty when every button of that
/// caption was enabled.
fn descartados_json(botones: &[Coincidencia]) -> Vec<Value> {
    botones
        .iter()
        .map(|c| {
            json!({
                "clase": c.clase,
                "hwnd": format!("0x{:X}", c.hwnd),
                "centro": { "x": c.punto.0, "y": c.punto.1 },
            })
        })
        .collect()
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

const CAPTION_DESHABILITADO: Texto = Texto {
    es: "El rótulo '{texto}' aparece solo en botones deshabilitados ({n} en total, {lista}): el clic no haría nada. Habilita lo que los condiciona (gui_observar que:vista) y repite",
    en: "The caption '{texto}' appears only on disabled buttons ({n} in total, {lista}): the click would do nothing. Enable what conditions them (gui_observar que:vista) and retry",
};

/// Each button with its class, hwnd and position, so the error names it and the
/// caller can address it.
fn lista_de_botones(botones: &[Coincidencia]) -> String {
    let mut lista = botones
        .iter()
        .take(BOTONES_EN_ERROR)
        .map(|c| format!("0x{:X} {} ({}, {})", c.hwnd, c.clase, c.punto.0, c.punto.1))
        .collect::<Vec<_>>()
        .join("; ");
    if botones.len() > BOTONES_EN_ERROR {
        lista.push_str("; …");
    }
    lista
}

/// The refusal for a caption that names several buttons: each with its class and
/// position, so the caller picks one instead of the server pressing a wrong one.
fn botones_ambiguos(texto: &str, botones: &[Coincidencia], idioma: Idioma) -> Error {
    let total = botones.len().to_string();
    Error::Gui(CAPTION_COMPARTIDO.texto_con(
        idioma,
        &[
            ("texto", texto),
            ("n", &total),
            ("lista", &lista_de_botones(botones)),
        ],
    ))
}

/// The refusal for a caption no enabled button carries: it names what to enable
/// instead of pressing a button that ignores the click.
fn botones_deshabilitados(texto: &str, botones: &[Coincidencia], idioma: Idioma) -> Error {
    let total = botones.len().to_string();
    Error::Gui(CAPTION_DESHABILITADO.texto_con(
        idioma,
        &[
            ("texto", texto),
            ("n", &total),
            ("lista", &lista_de_botones(botones)),
        ],
    ))
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/boton_por_caption.rs"]
mod tests;
