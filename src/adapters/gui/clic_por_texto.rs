//! `gui_actuar accion:clic_texto`: clicks the text a person reads on screen.
//! VCL speed buttons, grid rows and tree nodes have no window handle, so they
//! cannot be addressed any other way; the click is posted as mouse messages to
//! the window under the text, which works on a private desktop too.
//!
//! A button whose own caption is that text wins over the drawn text: a caption
//! on a button is the place a person means, even when a combo box or an edit
//! shows the same word. Two buttons with the same caption are refused, never
//! guessed.

use super::efecto::{Efecto, Instantanea};
use super::ffi::{self, FalloClic, Hwnd};
use super::inventario::tipos::Rol;
use super::modal;
use super::ocr::captura_ocr::lineas_vivas;
use super::ocr::segmentacion::segmentar_lineas;
use super::ocr::ubicacion::{elegir_caja, plegar, ubicar_texto};
use super::primer_plano;
use super::ventanas::{class_of, hwnd_de, rect_of, visible};
use crate::args::{opt_u32, req_str};
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use serde_json::{json, Value};
use std::time::Duration;

/// Lets the form process a posted click before the tab is read back.
const ASENTAMIENTO_CLIC: Duration = Duration::from_millis(400);

/// Buttons the error names before cutting the list.
const BOTONES_EN_ERROR: usize = 8;

/// Clicks the on-screen text `texto` inside window `ventana`; `indice` picks
/// among several matches.
pub(crate) fn clic_texto(args: &Value, idioma: Idioma) -> Result<Value> {
    let texto = req_str(args, &["texto"], "texto visible sobre el que pulsar")?;
    let indice = opt_u32(args, "indice")?.map(|i| i as usize);
    let h = hwnd_de(args, "ventana")?;
    modal::exigir_habilitada(h, modal::PARA_ACTUAR)?;
    // OCR and the click read screen coordinates: a minimised window has none.
    primer_plano::restaurar_al_frente(h, idioma)?;
    let elegido = destino_de_clic(h, Pedido { texto, indice }, idioma)?;
    let punto = elegido.punto;
    let objetivo = ffi::control_en_punto(h, punto);
    let pestana_antes = pestana_seleccionada(objetivo);
    let antes = Instantanea::de(h);
    enviar_clic(objetivo, punto)?;
    std::thread::sleep(ASENTAMIENTO_CLIC);
    let efecto = Efecto::entre(&antes, &antes.de_nuevo());
    let mut resultado = json!({
        "accion": "clic_texto",
        "texto": texto,
        "origen": elegido.origen.nombre(),
        "coincidencias": elegido.coincidencias,
        "punto": { "x": punto.0, "y": punto.1 },
        "control": { "hwnd": format!("0x{objetivo:X}"), "clase": class_of(objetivo) },
        "verificado": efecto.verificado(),
        "efecto": efecto.a_json(),
    });
    let avisos: Vec<String> = [
        pestana_sin_cambio(objetivo, pestana_antes),
        sin_efecto(&efecto, idioma),
    ]
    .into_iter()
    .flatten()
    .collect();
    if let (false, Some(obj)) = (avisos.is_empty(), resultado.as_object_mut()) {
        obj.insert("advertencias".into(), json!(avisos));
    }
    marcar_sin_ocr(&mut resultado, texto, elegido.origen, idioma);
    Ok(resultado)
}

fn enviar_clic(objetivo: Hwnd, punto: (i32, i32)) -> Result<()> {
    ffi::clic_en_pantalla(objetivo, punto).map_err(|fallo| match fallo {
        FalloClic::SinConversion => Error::Gui(format!(
            "Windows no pudo convertir el punto de pantalla al control 0x{objetivo:X}"
        )),
        FalloClic::SinEnvio => {
            Error::Gui(format!("PostMessage falló para el control 0x{objetivo:X}"))
        }
    })
}

/// A click that opened, closed and repainted nothing may have hit a dead spot:
/// the call succeeded, the effect did not happen (`verificado:false`).
fn sin_efecto(efecto: &Efecto, idioma: Idioma) -> Option<String> {
    if efecto.verificado() != Some(false) {
        return None;
    }
    let nada = Texto {
        es: "El clic no abrió ni cerró ninguna ventana ni repintó la ventana (verificado:false): puede haber caído en un punto muerto o el efecto es lento; comprueba con gui_observar que:ventanas o que:vista",
        en: "The click opened and closed no window and did not repaint the window (verificado:false): it may have hit a dead spot or its effect is slow; check with gui_observar que:ventanas or que:vista",
    };
    Some(nada.en_idioma(idioma).to_string())
}

/// A click without an OCR match went to the UIA element of that name: it
/// happened, but nothing confirms it hit the text the caller read, so the answer
/// carries `error` and `vista` reports it as partial instead of "ok" (BUG-03).
fn marcar_sin_ocr(resultado: &mut Value, texto: &str, origen: Origen, idioma: Idioma) {
    if origen != Origen::Uia {
        return;
    }
    if let Some(obj) = resultado.as_object_mut() {
        let sin_ocr = Texto {
            es: "El OCR no encontró '{texto}' (coincidencias:0): se pulsó el elemento UIA de ese nombre; comprueba el efecto con gui_observar que:ventanas",
            en: "OCR did not find '{texto}' (matches:0): the UIA element with that name was clicked; check the effect with gui_observar que:ventanas",
        };
        obj.insert(
            "error".into(),
            json!(sin_ocr.texto_con(idioma, &[("texto", texto)])),
        );
    }
}

/// Where a click on `texto` lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Origen {
    /// A visible button of the form whose own caption is the text.
    Control,
    /// The text drawn on screen: a speed button, a grid cell, a tree node.
    Ocr,
    /// The UI Automation element of that name, when OCR drew nothing.
    Uia,
}

impl Origen {
    /// The value the answer carries in `origen`.
    fn nombre(self) -> &'static str {
        match self {
            Self::Control => "control",
            Self::Ocr => "ocr",
            Self::Uia => "uia",
        }
    }
}

/// What `clic_texto` asks for: the caption and, when given, which drawn match.
#[derive(Clone, Copy)]
struct Pedido<'a> {
    texto: &'a str,
    indice: Option<usize>,
}

/// What a click on `texto` presses and where that target came from.
#[derive(Debug, Clone, Copy)]
struct Destino {
    punto: (i32, i32),
    origen: Origen,
    /// Places where OCR drew the text (`coincidencias` in the answer), 0 when a
    /// button carried the caption and OCR never ran.
    coincidencias: usize,
}

/// A visible button of the form whose own caption is the requested text.
struct Coincidencia {
    hwnd: Hwnd,
    clase: String,
    punto: (i32, i32),
}

/// Screen point to click for `texto`. The caption a control carries wins: a
/// person asking for «Abrir» means the button, not the combo box that shows the
/// same word. `indice` counts the drawn matches, so it skips that step; without
/// a control named that, the OCR box it picks, or the UIA element of that name
/// when OCR does not draw the text (VCL buttons sometimes lack pixels).
fn destino_de_clic(h: Hwnd, pedido: Pedido<'_>, idioma: Idioma) -> Result<Destino> {
    let boton = match pedido.indice {
        None => boton_con_caption(h, pedido.texto, idioma)?,
        Some(_) => None,
    };
    if let Some(punto) = boton {
        return Ok(Destino {
            punto,
            origen: Origen::Control,
            coincidencias: 0,
        });
    }
    let lineas = lineas_vivas(h)?;
    let cajas = ubicar_texto(&segmentar_lineas(&lineas, &[]), pedido.texto);
    match elegir_caja(&cajas, pedido.indice, pedido.texto) {
        Ok(caja) => Ok(Destino {
            punto: ((caja.left + caja.right) / 2, (caja.top + caja.bottom) / 2),
            origen: Origen::Ocr,
            coincidencias: cajas.len(),
        }),
        Err(ocr) if recurre_a_uia(&ocr) => {
            crate::adapters::uia::centro(h, Some(pedido.texto), None)
                .map(|punto| Destino {
                    punto,
                    origen: Origen::Uia,
                    coincidencias: cajas.len(),
                })
                .map_err(|_| ocr)
        }
        Err(ocr) => Err(ocr),
    }
}

/// Centre of the only button of the form whose own caption is `texto`; `None`
/// when no button carries it, so the caller reads the drawn text instead.
fn boton_con_caption(h: Hwnd, texto: &str, idioma: Idioma) -> Result<Option<(i32, i32)>> {
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

/// UIA stands in only when OCR drew no match at all. A match refused for its
/// `indice` (out of range) or for being ambiguous is the caller's mistake: a UIA
/// click there would press something the caller did not choose (GUI-09).
fn recurre_a_uia(ocr: &Error) -> bool {
    matches!(ocr, Error::NotFound(_))
}

/// Tab selected in `control` when it is a page control, which a click may switch.
fn pestana_seleccionada(control: Hwnd) -> Option<usize> {
    if class_of(control) != ffi::CLASE_PESTANAS {
        return None;
    }
    ffi::pagina_seleccionada(control)
}

/// Warns when a click on a page control left the selected tab where it was:
/// PVsyst vetoes the change while the current page has invalid fields.
fn pestana_sin_cambio(control: Hwnd, antes: Option<usize>) -> Option<String> {
    let antes = antes?;
    (ffi::pagina_seleccionada(control)? == antes).then(|| {
        format!(
            "La pestaña seleccionada no cambió (sigue en la {antes}): PVsyst la vetó por validación del formulario o ya estaba seleccionada; el motivo está en gui_observar que:vista ocr:true (textos)"
        )
    })
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/clic_por_texto.rs"]
mod tests;
