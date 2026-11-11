//! What one control of an OCR-inspected form reports: buttons with their
//! caption, fields with their label and value.

use super::dominio::{asociar_etiqueta, normalizar_etiqueta, LineaOcr, ResultadoEtiqueta};
use crate::adapters::gui::ffi::{self, Hwnd, Rect};
use crate::adapters::gui::inventario::dominio::{Control, FuenteEtiqueta};
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

/// Where the control sits: its visibility, parent and tab page.
fn ubicar(obj: &mut Value, c: &Control) {
    if let Some(map) = obj.as_object_mut() {
        map.insert("visible".into(), json!(c.visible));
        map.insert("parent".into(), json!(c.parent));
        map.insert("pestana".into(), json!(c.pestana));
        map.insert("indice_pestana".into(), json!(c.indice_pestana));
    }
}

/// `lineas` are the OCR lines of the form: pass none for a control that is not
/// shown, since the text drawn where it sits belongs to another page.
pub(super) fn procesar_boton(c: &Control, lineas: &[LineaOcr]) -> Value {
    let texto = if c.texto.trim().is_empty() {
        texto_dibujado(&c.rect, lineas)
    } else {
        normalizar_etiqueta(&c.texto)
    };
    let mut obj = json!({
        "hwnd": c.hwnd,
        "clase": c.clase,
        "texto": texto,
        "habilitado": c.enabled,
        "rect": rect_json(&c.rect),
    });
    ubicar(&mut obj, c);
    obj
}

/// The label of a field and where it was read: the caption of an option, else
/// the OCR text beside it, else the text of a window beside it. A tie between
/// two OCR candidates is reported as such, not settled by the fallback.
fn etiqueta_de(
    c: &Control,
    asociacion: &ResultadoEtiqueta,
) -> (Option<String>, Option<FuenteEtiqueta>) {
    let es_opcion = CLASES_OPCION.contains(&c.clase.as_str());
    if es_opcion && !c.texto.trim().is_empty() {
        return (
            Some(normalizar_etiqueta(&c.texto)),
            Some(FuenteEtiqueta::TextoDeVentana),
        );
    }
    if asociacion.etiqueta.is_some() {
        return (asociacion.etiqueta.clone(), Some(FuenteEtiqueta::Ocr));
    }
    if asociacion.ambiguo {
        return (None, None);
    }
    (c.etiqueta.clone(), c.etiqueta_fuente)
}

/// `lineas` as in [`procesar_boton`].
pub(super) fn procesar_campo(c: &Control, lineas: &[LineaOcr]) -> Value {
    let asociacion = asociar_etiqueta(&c.rect, lineas);
    let (etiqueta, fuente) = etiqueta_de(c, &asociacion);
    let mut obj = json!({
        "hwnd": c.hwnd,
        "clase": c.clase,
        "etiqueta": etiqueta,
        "etiqueta_fuente": fuente,
        "ambiguo": asociacion.ambiguo,
        "candidatos": asociacion.candidatos,
        "habilitado": c.enabled,
        "rect": rect_json(&c.rect),
    });
    ubicar(&mut obj, c);
    if let Some(map) = obj.as_object_mut() {
        if CLASES_TEXTO.contains(&c.clase.as_str()) {
            map.insert("valor".into(), json!(c.texto));
        }
        if CLASES_OPCION.contains(&c.clase.as_str()) {
            map.insert("marcado".into(), json!(c.marcado));
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
