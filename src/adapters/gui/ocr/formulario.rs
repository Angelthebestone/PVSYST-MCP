//! Form inspection orchestrator: pairs Win32 child controls with OCR labels
//! and UIA tab items into a single structured view of an interactive form.

use super::captura_ocr::{capturar_lineas_pantalla, ParametrosCaptura};
use super::controles::{es_boton, es_campo, procesar_boton, procesar_campo, rect_json};
use super::dominio::LineaOcr;
use super::segmentacion::segmentar_lineas;
use crate::adapters::gui::ffi::{self, Hwnd, Rect};
use crate::adapters::gui::inventario::dominio::inventariar;
use crate::adapters::gui::inventario::tipos::{Control, LecturaUia};
use crate::adapters::gui::inventario::win32;
use crate::adapters::gui::uia_paso::raiz_por_defecto;
use crate::adapters::gui::uia_vista::aplanar;
use crate::adapters::gui::ventanas::{class_of, hwnd_de, rect_of, text_of, ventanas_de};
use crate::adapters::uia;
use crate::args::flag;
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use serde_json::{json, Value};

const UIA_PROFUNDIDAD_PESTANAS: u32 = 4;
const UIA_MAX_NODOS_PESTANAS: u32 = 100;

/// Tab items of the form's page control; `seleccionada` is the page the control
/// reports as current (`TCM_GETCURSEL`), not a UI Automation capability flag.
fn extraer_pestanas(
    paths: &crate::config::Paths,
    h: Hwnd,
    seleccionada: Option<usize>,
) -> Result<Vec<Value>> {
    let arbol = uia::arbol(
        &paths.estado,
        h,
        UIA_PROFUNDIDAD_PESTANAS,
        UIA_MAX_NODOS_PESTANAS,
    )?;
    let mut plano = Vec::new();
    aplanar(&arbol, &mut plano);
    Ok(plano
        .into_iter()
        .filter(|n| n["rol"].as_str() == Some("tab item"))
        .enumerate()
        .map(|(indice, n)| {
            json!({
                "nombre": n["nombre"],
                "rect": n["rect"],
                "seleccionada": Some(indice) == seleccionada,
            })
        })
        .collect())
}

/// OCR lines of the form; none with `ocr:false`, which skips the capture so
/// values and buttons come from the controls alone.
fn lineas_o_nada(params: &ParametrosCaptura, h: Hwnd) -> Result<Vec<LineaOcr>> {
    if params.args.get("ocr").and_then(Value::as_bool) == Some(false) {
        return Ok(Vec::new());
    }
    capturar_lineas_pantalla(params, h)
}

const SIN_VENTANA_VISIBLE: Texto = Texto {
    es: "No hay una ventana visible para inspeccionar",
    en: "There is no visible window to inspect",
};

/// The window named by `ventana`, or the front-most one of the process.
pub(crate) fn ventana_objetivo(args: &Value) -> Result<Hwnd> {
    match args.get("ventana").and_then(Value::as_str) {
        Some(_) => hwnd_de(args, "ventana"),
        None => raiz_por_defecto(&ventanas_de(args)?)
            .ok_or_else(|| Error::Gui(SIN_VENTANA_VISIBLE.en_idioma(Idioma::de_entorno()).into())),
    }
}

/// Buttons, fields and everything else, one list each. A control that is not
/// shown (`todos:true`) is read without OCR: the text drawn where it sits
/// belongs to the page that is.
fn clasificar_controles(
    controles: &[Control],
    lineas: &[LineaOcr],
    etiquetas: &[LineaOcr],
) -> (Vec<Value>, Vec<Value>, Vec<Value>) {
    let (mut campos, mut botones, mut otros) = (Vec::new(), Vec::new(), Vec::new());
    for c in controles {
        let (propias, rotulos) = if c.visible {
            (lineas, etiquetas)
        } else {
            (&[][..], &[][..])
        };
        if es_boton(&c.clase) {
            botones.push(procesar_boton(c, propias));
        } else if es_campo(&c.clase) {
            campos.push(procesar_campo(c, rotulos));
        } else {
            otros.push(json!(c));
        }
    }
    (campos, botones, otros)
}

/// Captions of the tab items UI Automation lists, in strip order.
fn nombres_de(pestanas: &[Value]) -> Vec<String> {
    pestanas
        .iter()
        .filter_map(|p| p["nombre"].as_str().map(str::to_string))
        .collect()
}

/// Inspects an active GUI form: recognizes all labels via OCR, pairs them with
/// their interactive controls, and extracts buttons and tab pages. `textos`
/// lists every caption drawn outside a control (a message box's question).
/// With `todos:true` the controls that are not shown (other tab pages, hidden
/// panels) are listed too; `otros` holds every control that is neither a
/// button nor a field.
pub(crate) fn formulario(paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    ffi::declarar_dpi_por_monitor();
    let h = ventana_objetivo(args)?;
    let rect_win = rect_of(h);
    let params = ParametrosCaptura {
        paths,
        args,
        rect_win: &rect_win,
    };
    let lineas = lineas_o_nada(&params, h)?;
    let nodos = win32::nodos_de(h, flag(args, &["todos"]));
    let (pestanas, pestanas_error) = match extraer_pestanas(paths, h, win32::pagina_actual(&nodos))
    {
        Ok(pestanas) => (pestanas, None),
        Err(e) => (Vec::new(), Some(e.to_string())),
    };
    let uia = LecturaUia {
        pestanas: nombres_de(&pestanas),
        ..LecturaUia::default()
    };
    let inventario = inventariar(&win32::instantanea_con(h, nodos, uia));
    let rects: Vec<Rect> = inventario
        .controles
        .iter()
        .filter(|c| c.visible && (es_boton(&c.clase) || es_campo(&c.clase)))
        .map(|c| c.rect)
        .collect();
    let etiquetas = segmentar_lineas(&lineas, &rects);
    let (campos, botones, otros) = clasificar_controles(&inventario.controles, &lineas, &etiquetas);
    Ok(json!({
        "ventana": format!("0x{:X}", h),
        "titulo": text_of(h),
        "clase": class_of(h),
        "dpi": ffi::dpi_de(h),
        "rect": rect_json(&rect_win),
        "campos": campos,
        "textos": etiquetas.iter().map(|l| l.texto.as_str()).collect::<Vec<_>>(),
        "pestanas": pestanas,
        "pestanas_error": pestanas_error,
        "botones": botones,
        "otros": otros,
    }))
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/ocr/formulario.rs"]
mod tests;
