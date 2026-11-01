//! The fields a form shows right now, read from the controls themselves: no
//! OCR and no captions, so it works at any DPI and costs milliseconds.

use super::dominio::{Ancla, CampoEsquema, Entidad, TOLERANCIA_ANCLA_PX};
use crate::adapters::gui::ffi::{self, Hwnd};
use crate::adapters::gui::ocr::esta_marcado;
use crate::adapters::gui::ventanas::{hijos_visibles_con_clase, rect_of};
use serde_json::{json, Value};

const CLASES_TEXTO: &[&str] = &[
    "TEdit",
    "TMemo",
    "TLabeledEdit",
    "TMaskEdit",
    "TSpinEdit",
    "TDateTimePicker",
];
/// Controls that offer a list of items to choose from.
pub(crate) const CLASES_LISTA: &[&str] = &["TComboBox", "TComboBoxEx"];
const CLASES_OPCION: &[&str] = &["TCheckBox", "TRadioButton"];

/// A visible field control with its value as text and its 96-DPI position.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CampoVivo {
    pub(crate) hwnd: Hwnd,
    pub(crate) clase: String,
    pub(crate) texto: String,
    pub(crate) habilitado: bool,
    pub(crate) posicion: Ancla,
}

fn es_campo(clase: &str) -> bool {
    [CLASES_TEXTO, CLASES_LISTA, CLASES_OPCION]
        .iter()
        .any(|clases| clases.contains(&clase))
}

/// The value of control `c` as the view shows it: `true`/`false` for options, its text otherwise.
pub(crate) fn valor_de(c: Hwnd, clase: &str) -> String {
    if CLASES_OPCION.contains(&clase) {
        return esta_marcado(c)
            .map(|m| m.to_string())
            .unwrap_or_else(|| "desconocido".to_string());
    }
    ffi::texto_control(c)
}

/// Visible field controls of form `h`, in creation order. A DPI-unaware window
/// reports physical rectangles for a layout drawn at 96 DPI: positions are
/// divided back by the scale. O(controls).
pub(crate) fn campos_vivos(h: Hwnd) -> Vec<CampoVivo> {
    let escala = ffi::escala_de_captura(h);
    let origen = rect_of(h);
    let a_logico = |v: i32| (f64::from(v) / escala).round() as i32;
    hijos_visibles_con_clase(h)
        .filter(|(_, clase)| es_campo(clase))
        .map(|(c, clase)| {
            let r = rect_of(c);
            CampoVivo {
                hwnd: c,
                texto: valor_de(c, &clase),
                habilitado: ffi::estado_ventana(c).habilitada,
                posicion: Ancla {
                    x: a_logico(r.left - origen.left),
                    y: a_logico(r.top - origen.top),
                },
                clase,
            }
        })
        .collect()
}

/// The live control of `campo`: same class, nearest to its anchor and within
/// [`TOLERANCIA_ANCLA_PX`] on both axes. O(controls).
pub(crate) fn localizar<'v>(campo: &CampoEsquema, vivos: &'v [CampoVivo]) -> Option<&'v CampoVivo> {
    vivos
        .iter()
        .filter(|v| v.clase == campo.clase)
        .map(|v| {
            let (dx, dy) = (
                (v.posicion.x - campo.ancla.x).abs(),
                (v.posicion.y - campo.ancla.y).abs(),
            );
            (v, dx.max(dy))
        })
        .filter(|(_, distancia)| *distancia <= TOLERANCIA_ANCLA_PX)
        .min_by_key(|(_, distancia)| *distancia)
        .map(|(v, _)| v)
}

/// Fields PVsyst recomputed on its own: mapped or not, those not asked for
/// whose text changed. O(controls).
pub(super) fn recalculados(entidad: &Entidad, antes: &[CampoVivo], pedidos: &[Hwnd]) -> Vec<Value> {
    con_texto_actual(antes)
        .into_iter()
        .filter(|(a, _)| !pedidos.contains(&a.hwnd))
        .filter(|(a, ahora)| a.texto != *ahora)
        .map(|(a, ahora)| {
            let clave = entidad
                .campos
                .iter()
                .find(|c| localizar(c, std::slice::from_ref(a)).is_some())
                .map(|c| c.clave);
            json!({ "clave": clave, "hwnd": format!("0x{:X}", a.hwnd), "antes": a.texto, "despues": ahora })
        })
        .collect()
}

fn con_texto_actual(antes: &[CampoVivo]) -> Vec<(&CampoVivo, String)> {
    antes
        .iter()
        .map(|a| (a, valor_de(a.hwnd, &a.clase)))
        .collect()
}
