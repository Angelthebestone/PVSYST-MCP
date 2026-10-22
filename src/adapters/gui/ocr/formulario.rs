//! Form inspection orchestrator: pairs Win32 child controls with OCR labels
//! and UIA tab items into a single structured view of an interactive form.

use super::captura_ocr::{capturar_lineas_pantalla, ParametrosCaptura};
use super::dominio::{asociar_etiqueta, normalizar_etiqueta, LineaOcr, RectOcr};
use super::segmentacion::segmentar_lineas;
use crate::adapters::gui::ffi::{self, Hwnd, Rect};
use crate::adapters::gui::uia_vista::{aplanar, raiz_por_defecto};
use crate::adapters::gui::ventanas::{class_of, hwnd_de, rect_of, text_of, ventanas_de, visible};
use crate::adapters::uia;
use crate::error::{Error, Result};
use serde_json::{json, Value};

const BM_GETCHECK: u32 = 0x00F0;
const BST_CHECKED: isize = 1;
const UIA_PROFUNDIDAD_PESTANAS: u32 = 4;
const UIA_MAX_NODOS_PESTANAS: u32 = 100;

const CLASES_BOTON: &[&str] = &[
    "TButton",
    "TBitBtn",
    "TPngBitBtn",
    "TSpeedButton",
    "TToolButton",
    "Button",
];

/// Fields whose text is their value.
const CLASES_TEXTO: &[&str] = &["TEdit", "TMemo", "TLabeledEdit", "TMaskEdit", "TComboBox"];
/// Check boxes and radio buttons: their caption is their label.
const CLASES_OPCION: &[&str] = &["TRadioButton", "TCheckBox"];
/// Other controls a form is filled in with.
const CLASES_OTRO_CAMPO: &[&str] = &["TListBox", "TTrackBar", "TUpDown", "TDateTimePicker"];
/// How far outside a button its OCR text may still lie and count as its caption.
const MARGEN_BOTON_PX: i32 = 4;

pub(crate) fn esta_marcado(h: Hwnd) -> bool {
    ffi::send_message_res(h, BM_GETCHECK, 0, 0) == BST_CHECKED
}

fn rect_json(r: &Rect) -> Value {
    json!({
        "left": r.left,
        "top": r.top,
        "right": r.right,
        "bottom": r.bottom,
        "width": r.right - r.left,
        "height": r.bottom - r.top,
    })
}

/// Caption of a button that has none as window text: the OCR words drawn inside it.
fn texto_dibujado(r: &Rect, lineas: &[LineaOcr]) -> String {
    let caja = RectOcr::new(
        r.left - MARGEN_BOTON_PX,
        r.top - MARGEN_BOTON_PX,
        r.right + MARGEN_BOTON_PX,
        r.bottom + MARGEN_BOTON_PX,
    );
    let palabras: Vec<&str> = lineas
        .iter()
        .flat_map(|l| &l.palabras)
        .filter(|p| centro_dentro(&p.rect, &caja))
        .map(|p| p.texto.as_str())
        .collect();
    normalizar_etiqueta(&palabras.join(" "))
}

fn centro_dentro(palabra: &RectOcr, caja: &RectOcr) -> bool {
    let (x, y) = ((palabra.left + palabra.right) / 2, palabra.centro_y());
    x >= caja.left && x <= caja.right && y >= caja.top && y <= caja.bottom
}

fn procesar_boton(c: Hwnd, clase: &str, lineas: &[LineaOcr]) -> Value {
    let r = rect_of(c);
    let estado = ffi::estado_ventana(c);
    let raw_texto = ffi::texto_control(c);
    let texto = if raw_texto.trim().is_empty() {
        texto_dibujado(&r, lineas)
    } else {
        normalizar_etiqueta(&raw_texto)
    };
    json!({
        "hwnd": format!("0x{:X}", c),
        "clase": clase,
        "texto": texto,
        "habilitado": estado.habilitada,
        "rect": rect_json(&r),
    })
}

fn procesar_campo(c: Hwnd, clase: &str, lineas: &[LineaOcr]) -> Value {
    let r = rect_of(c);
    let estado = ffi::estado_ventana(c);
    let raw_texto = ffi::texto_control(c);
    let r_ocr = RectOcr::new(r.left, r.top, r.right, r.bottom);
    let asociacion = asociar_etiqueta(&r_ocr, lineas);
    let es_opcion = CLASES_OPCION.contains(&clase);
    let etiqueta = if es_opcion && !raw_texto.trim().is_empty() {
        Some(normalizar_etiqueta(&raw_texto))
    } else {
        asociacion.etiqueta
    };
    let mut obj = json!({
        "hwnd": format!("0x{:X}", c),
        "clase": clase,
        "etiqueta": etiqueta,
        "ambiguo": asociacion.ambiguo,
        "candidatos": asociacion.candidatos,
        "habilitado": estado.habilitada,
        "rect": rect_json(&r),
    });
    if CLASES_TEXTO.contains(&clase) {
        obj["valor"] = json!(raw_texto);
    }
    if es_opcion {
        obj["marcado"] = json!(esta_marcado(c));
    }
    obj
}

fn es_boton(clase: &str) -> bool {
    CLASES_BOTON.contains(&clase)
}

fn es_campo(clase: &str) -> bool {
    [CLASES_TEXTO, CLASES_OPCION, CLASES_OTRO_CAMPO]
        .iter()
        .any(|clases| clases.contains(&clase))
}

fn rect_ocr_de(c: Hwnd) -> RectOcr {
    let r = rect_of(c);
    RectOcr::new(r.left, r.top, r.right, r.bottom)
}

fn extraer_pestanas(paths: &crate::config::Paths, h: Hwnd) -> Result<Vec<Value>> {
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
        .map(|n| {
            let seleccionada = n["patrones"]
                .as_array()
                .is_some_and(|p| p.iter().any(|v| v.as_str() == Some("SelectionItem")));
            json!({
                "nombre": n["nombre"],
                "rect": n["rect"],
                "seleccionada": seleccionada,
            })
        })
        .collect())
}

/// Inspects an active GUI form: recognizes all labels via OCR, pairs them with
/// their interactive controls, and extracts buttons and tab pages. `textos`
/// lists every caption drawn outside a control (a message box's question).
pub(crate) fn formulario(paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    ffi::declarar_dpi_por_monitor();
    let h = match args.get("ventana").and_then(Value::as_str) {
        Some(_) => hwnd_de(args, "ventana")?,
        None => raiz_por_defecto(&ventanas_de(args)?)
            .ok_or_else(|| Error::Gui("No hay una ventana visible para inspeccionar".into()))?,
    };
    let rect_win = rect_of(h);
    let params = ParametrosCaptura {
        paths,
        args,
        rect_win: &rect_win,
    };
    let lineas = capturar_lineas_pantalla(&params, h)?;
    let hijos: Vec<(Hwnd, String)> = ffi::enum_children(h)
        .into_iter()
        .filter(|c| visible(*c))
        .map(|c| (c, class_of(c)))
        .collect();
    let rects: Vec<RectOcr> = hijos
        .iter()
        .filter(|(_, clase)| es_boton(clase) || es_campo(clase))
        .map(|(c, _)| rect_ocr_de(*c))
        .collect();
    let etiquetas = segmentar_lineas(&lineas, &rects);
    let mut campos = Vec::new();
    let mut botones = Vec::new();
    for (c, clase) in &hijos {
        if es_boton(clase) {
            botones.push(procesar_boton(*c, clase, &lineas));
        } else if es_campo(clase) {
            campos.push(procesar_campo(*c, clase, &etiquetas));
        }
    }
    let (pestanas, pestanas_error) = match extraer_pestanas(paths, h) {
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
