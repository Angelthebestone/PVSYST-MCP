//! Form inspection orchestrator: pairs Win32 child controls with OCR labels
//! and UIA tab items into a single structured view of an interactive form.

use super::captura_ocr::{capturar_lineas_pantalla, ParametrosCaptura};
use super::controles::{es_boton, es_campo, procesar_boton, procesar_campo, rect_json};
use super::dominio::LineaOcr;
use super::segmentacion::segmentar_lineas;
use crate::adapters::gui::ffi::{self, Hwnd, Rect};
use crate::adapters::gui::uia_paso::raiz_por_defecto;
use crate::adapters::gui::uia_vista::aplanar;
use crate::adapters::gui::ventanas::{
    class_of, hijos_visibles_con_clase, hwnd_de, rect_of, text_of, ventanas_de,
};
use crate::adapters::uia;
use crate::error::{Error, Result};
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

/// Selected tab of the form's (first) page control.
fn pestana_activa(hijos: &[(Hwnd, String)]) -> Option<usize> {
    let (pestanas, _) = hijos
        .iter()
        .find(|(_, clase)| clase == ffi::CLASE_PESTANAS)?;
    ffi::pagina_seleccionada(*pestanas)
}

/// OCR lines of the form; none with `ocr:false`, which skips the capture so
/// values and buttons come from the controls alone.
fn lineas_o_nada(params: &ParametrosCaptura, h: Hwnd) -> Result<Vec<LineaOcr>> {
    if params.args.get("ocr").and_then(Value::as_bool) == Some(false) {
        return Ok(Vec::new());
    }
    capturar_lineas_pantalla(params, h)
}

/// The window named by `ventana`, or the front-most one of the process.
pub(crate) fn ventana_objetivo(args: &Value) -> Result<Hwnd> {
    match args.get("ventana").and_then(Value::as_str) {
        Some(_) => hwnd_de(args, "ventana"),
        None => raiz_por_defecto(&ventanas_de(args)?)
            .ok_or_else(|| Error::Gui("No hay una ventana visible para inspeccionar".into())),
    }
}

fn clasificar_controles(
    hijos: &[(Hwnd, String)],
    lineas: &[LineaOcr],
    etiquetas: &[LineaOcr],
) -> (Vec<Value>, Vec<Value>) {
    let mut campos = Vec::new();
    let mut botones = Vec::new();
    for (c, clase) in hijos {
        if es_boton(clase) {
            botones.push(procesar_boton(*c, clase, lineas));
        } else if es_campo(clase) {
            campos.push(procesar_campo(*c, clase, etiquetas));
        }
    }
    (campos, botones)
}

/// Inspects an active GUI form: recognizes all labels via OCR, pairs them with
/// their interactive controls, and extracts buttons and tab pages. `textos`
/// lists every caption drawn outside a control (a message box's question).
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
    let hijos: Vec<(Hwnd, String)> = hijos_visibles_con_clase(h).collect();
    let rects: Vec<Rect> = hijos
        .iter()
        .filter(|(_, clase)| es_boton(clase) || es_campo(clase))
        .map(|(c, _)| rect_of(*c))
        .collect();
    let etiquetas = segmentar_lineas(&lineas, &rects);
    let (campos, botones) = clasificar_controles(&hijos, &lineas, &etiquetas);
    let (pestanas, pestanas_error) = match extraer_pestanas(paths, h, pestana_activa(&hijos)) {
        Ok(pestanas) => (pestanas, None),
        Err(e) => (Vec::new(), Some(e.to_string())),
    };
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
    }))
}
