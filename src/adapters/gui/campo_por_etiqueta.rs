//! `gui_actuar accion:campo`: sets a form field by the label a person reads
//! next to it. The label comes from `gui_observar que:formulario` (OCR), so
//! no coordinates or per-version field tables are needed.

use super::acciones::clic_por_mensajes;
use super::conduccion::Formulario;
use super::ffi::{self, Hwnd};
use super::ocr::seleccion::{campo_por_etiqueta, valor_logico, CampoEtiquetado};
use super::ocr::{self, esta_marcado};
use crate::args::req_str;
use crate::error::{Error, Result};
use serde_json::{json, Value};

const CLASES_TEXTO: &[&str] = &["TEdit", "TMemo", "TLabeledEdit", "TMaskEdit", "TSpinEdit"];
const CLASES_LISTA: &[&str] = &["TComboBox", "TComboBoxEx", "TListBox"];
const CLASES_OPCION: &[&str] = &["TCheckBox", "TRadioButton"];

fn hwnd_de_texto(texto: &str) -> Result<Hwnd> {
    texto
        .strip_prefix("0x")
        .and_then(|h| Hwnd::from_str_radix(h, 16).ok())
        .ok_or_else(|| Error::Gui(format!("Identificador de ventana no válido: '{texto}'")))
}

fn campos_de(ficha: &Value) -> Result<Vec<CampoEtiquetado>> {
    ficha["campos"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|c| {
            Ok(CampoEtiquetado {
                hwnd: hwnd_de_texto(c["hwnd"].as_str().unwrap_or_default())?,
                clase: c["clase"].as_str().unwrap_or_default().to_string(),
                etiqueta: c["etiqueta"].as_str().map(str::to_string),
            })
        })
        .collect()
}

/// Current value of a field as text, for the before/after report.
fn leer_valor(c: &CampoEtiquetado) -> String {
    if CLASES_OPCION.contains(&c.clase.as_str()) {
        return esta_marcado(c.hwnd).to_string();
    }
    ffi::texto_control(c.hwnd)
}

fn poner_valor(f: &Formulario, c: &CampoEtiquetado, valor: &str) -> Result<()> {
    let clase = c.clase.as_str();
    if CLASES_TEXTO.contains(&clase) {
        f.escribir_en(c.hwnd, valor);
        return Ok(());
    }
    if CLASES_LISTA.contains(&clase) {
        if ffi::elegir_item_por_clase(c.hwnd, &c.clase, valor) {
            return Ok(());
        }
        return Err(Error::Invalid(format!("La lista no ofrece '{valor}'")));
    }
    if CLASES_OPCION.contains(&clase) {
        if esta_marcado(c.hwnd) != valor_logico(valor)? {
            clic_por_mensajes(c.hwnd)?;
        }
        return Ok(());
    }
    Err(Error::Gui(format!("No sé poner valores en un {clase}")))
}

/// Sets the field labelled `etiqueta` of the target form to `valor`.
pub(crate) fn campo(paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    let etiqueta = req_str(args, &["etiqueta"], "rótulo del campo")?;
    let valor = req_str(args, &["valor"], "valor a poner")?;
    let ficha = ocr::formulario(paths, args)?;
    let campos = campos_de(&ficha)?;
    let elegido = campo_por_etiqueta(&campos, etiqueta)?;
    let formulario = Formulario::de(hwnd_de_texto(
        ficha["ventana"].as_str().unwrap_or_default(),
    )?);
    let antes = leer_valor(elegido);
    poner_valor(&formulario, elegido, valor)?;
    Ok(json!({
        "accion": "campo",
        "etiqueta": elegido.etiqueta,
        "clase": elegido.clase,
        "antes": antes,
        "despues": leer_valor(elegido),
    }))
}
