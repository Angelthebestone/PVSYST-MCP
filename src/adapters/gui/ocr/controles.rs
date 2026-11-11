//! What one control of an OCR-inspected form reports: buttons with their
//! caption, fields with their label and value.

use super::dominio::{asociar_etiqueta, normalizar_etiqueta, LineaOcr};
use crate::adapters::gui::ffi::{self, Hwnd, Rect};
use crate::adapters::gui::ventanas::rect_of;
use serde_json::{json, Value};

const BM_GETCHECK: u32 = 0x00F0;
const BST_CHECKED: isize = 1;

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

pub(crate) fn esta_marcado(h: Hwnd) -> Option<bool> {
    ffi::send_message_timeout(
        h,
        ffi::Mensaje::simple(BM_GETCHECK),
        ffi::TIMEOUT_MENSAJE_MS,
    )
    .map(|res| res == BST_CHECKED)
}

pub(super) fn rect_json(r: &Rect) -> Value {
    json!({
        "left": r.left,
        "top": r.top,
        "right": r.right,
        "bottom": r.bottom,
        "width": r.right - r.left,
        "height": r.bottom - r.top,
    })
}

struct DatosControl {
    rect: Rect,
    estado: ffi::EstadoVentana,
    texto_raw: String,
}

fn leer_control(c: Hwnd) -> DatosControl {
    DatosControl {
        rect: rect_of(c),
        estado: ffi::estado_ventana(c),
        texto_raw: ffi::texto_control(c),
    }
}

/// Caption of a button that has none as window text: the OCR words drawn inside it.
fn texto_dibujado(r: &Rect, lineas: &[LineaOcr]) -> String {
    let caja = Rect {
        left: r.left - MARGEN_BOTON_PX,
        top: r.top - MARGEN_BOTON_PX,
        right: r.right + MARGEN_BOTON_PX,
        bottom: r.bottom + MARGEN_BOTON_PX,
    };
    let palabras: Vec<&str> = lineas
        .iter()
        .flat_map(|l| &l.palabras)
        .filter(|p| centro_dentro(&p.rect, &caja))
        .map(|p| p.texto.as_str())
        .collect();
    normalizar_etiqueta(&palabras.join(" "))
}

fn centro_dentro(palabra: &Rect, caja: &Rect) -> bool {
    let (x, y) = ((palabra.left + palabra.right) / 2, palabra.centro_y());
    x >= caja.left && x <= caja.right && y >= caja.top && y <= caja.bottom
}

pub(super) fn procesar_boton(c: Hwnd, clase: &str, lineas: &[LineaOcr]) -> Value {
    let ctrl = leer_control(c);
    let texto = if ctrl.texto_raw.trim().is_empty() {
        texto_dibujado(&ctrl.rect, lineas)
    } else {
        normalizar_etiqueta(&ctrl.texto_raw)
    };
    json!({
        "hwnd": format!("0x{:X}", c),
        "clase": clase,
        "texto": texto,
        "habilitado": ctrl.estado.habilitada,
        "rect": rect_json(&ctrl.rect),
    })
}

pub(super) fn procesar_campo(c: Hwnd, clase: &str, lineas: &[LineaOcr]) -> Value {
    let ctrl = leer_control(c);
    let asociacion = asociar_etiqueta(&ctrl.rect, lineas);
    let es_opcion = CLASES_OPCION.contains(&clase);
    let etiqueta = if es_opcion && !ctrl.texto_raw.trim().is_empty() {
        Some(normalizar_etiqueta(&ctrl.texto_raw))
    } else {
        asociacion.etiqueta
    };
    let mut obj = json!({
        "hwnd": format!("0x{:X}", c),
        "clase": clase,
        "etiqueta": etiqueta,
        "ambiguo": asociacion.ambiguo,
        "candidatos": asociacion.candidatos,
        "habilitado": ctrl.estado.habilitada,
        "rect": rect_json(&ctrl.rect),
    });
    if let Some(map) = obj.as_object_mut() {
        if CLASES_TEXTO.contains(&clase) {
            map.insert("valor".into(), json!(ctrl.texto_raw));
        }
        if es_opcion {
            map.insert("marcado".into(), json!(esta_marcado(c)));
        }
    }
    obj
}

pub(super) fn es_boton(clase: &str) -> bool {
    CLASES_BOTON.contains(&clase)
}

pub(super) fn es_campo(clase: &str) -> bool {
    [CLASES_TEXTO, CLASES_OPCION, CLASES_OTRO_CAMPO]
        .iter()
        .any(|clases| clases.contains(&clase))
}
