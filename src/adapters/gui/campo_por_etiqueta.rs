//! `gui_actuar accion:campo`: sets a form field by the label a person reads
//! next to it. The label comes from `gui_observar que:formulario` (OCR), so
//! no coordinates or per-version field tables are needed.

use super::acciones::clic_por_mensajes;
use super::conduccion::Formulario;
use super::ffi::{self, Hwnd};
use super::ocr::seleccion::{campo_por_etiqueta, valor_logico, CampoEtiquetado, Desambiguador};
use super::ocr::{self, esta_marcado};
use crate::args::{opt_u32, req_str};
use crate::error::{Error, Result};
use serde_json::{json, Value};

pub(crate) const CLASES_TEXTO: &[&str] =
    &["TEdit", "TMemo", "TLabeledEdit", "TMaskEdit", "TSpinEdit"];
pub(crate) const CLASES_LISTA: &[&str] = &["TComboBox", "TComboBoxEx", "TListBox"];
const CLASES_OPCION: &[&str] = &["TCheckBox", "TRadioButton"];

fn hwnd_de_texto(texto: &str) -> Result<Hwnd> {
    texto
        .strip_prefix("0x")
        .and_then(|h| Hwnd::from_str_radix(h, 16).ok())
        .ok_or_else(|| Error::Gui(format!("Invalid window identifier: '{texto}'")))
}

/// One side of the `rect` a form answer gives a field (0 when it is missing).
fn lado(campo: &Value, lado: &str) -> i32 {
    campo
        .get("rect")
        .and_then(|r| r.get(lado))
        .and_then(Value::as_i64)
        .and_then(|v| i32::try_from(v).ok())
        .unwrap_or(0)
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
                posicion: (lado(c, "top"), lado(c, "left")),
            })
        })
        .collect()
}

/// Current value of a field as text, for the before/after report.
fn leer_valor(c: &CampoEtiquetado) -> String {
    if CLASES_OPCION.contains(&c.clase.as_str()) {
        return esta_marcado(c.hwnd)
            .map(|m| m.to_string())
            .unwrap_or_else(|| "desconocido".to_string());
    }
    ffi::texto_control(c.hwnd)
}

fn poner_valor(f: &Formulario, c: &CampoEtiquetado, valor: &str) -> Result<()> {
    let clase = c.clase.as_str();
    if CLASES_TEXTO.contains(&clase) {
        f.escribir_en(c.hwnd, valor)?;
        return Ok(());
    }
    if CLASES_LISTA.contains(&clase) {
        if ffi::elegir_item_por_clase(c.hwnd, &c.clase, valor)? {
            return Ok(());
        }
        return Err(Error::Invalid(format!("The list offers no '{valor}'")));
    }
    if CLASES_OPCION.contains(&clase) {
        let marcado = esta_marcado(c.hwnd).ok_or_else(|| {
            Error::Gui("PVsyst did not answer in 3 s when reading the check box state".into())
        })?;
        if marcado != valor_logico(valor)? {
            clic_por_mensajes(c.hwnd)?;
        }
        return Ok(());
    }
    Err(Error::Gui(format!("Cannot set values on a {clase}")))
}

/// The keys `rellenar` accepts for the open form, or how to find them. A label
/// that is not text (a bitmap, an icon) leaves `campo` with nothing to match,
/// and `rellenar` works by stable key instead of by OCR.
fn sugerencia_de_relleno(vista: &Value) -> String {
    let claves: Vec<&str> = vista["campos"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|c| c["clave"].as_str())
        .collect();
    let Some(entidad) = vista["entidad"].as_str().filter(|_| !claves.is_empty()) else {
        return "Use gui_actuar accion:rellenar with the keys of gui_observar que:vista (que:esquema lists the entities)".into();
    };
    format!(
        "Use gui_actuar accion:rellenar with entidad '{entidad}' and keys: {}",
        claves.join(", ")
    )
}

/// The not-found error plus the way that works when the label is not text.
fn error_sin_etiqueta(motivo: &str, vista: &Value) -> Error {
    Error::NotFound(format!("{motivo}. {}", sugerencia_de_relleno(vista)))
}

/// Sets the field labelled `etiqueta` of the target form to `valor`. `clase` and
/// `indice` pick one of several fields that share the label. When no label
/// matches, `vista_de` reads the window so the error can suggest `rellenar`.
pub(crate) fn campo(
    paths: &crate::config::Paths,
    args: &Value,
    vista_de: fn(&crate::config::Paths, &Value) -> Result<Value>,
) -> Result<Value> {
    let etiqueta = req_str(args, &["etiqueta"], "label of the field")?;
    let valor = req_str(args, &["valor"], "value to set")?;
    let ficha = ocr::formulario(paths, args)?;
    let campos = campos_de(&ficha)?;
    let como = Desambiguador {
        clase: args.get("clase").and_then(Value::as_str),
        indice: opt_u32(args, "indice")?.map(|i| i as usize),
    };
    let elegido = match campo_por_etiqueta(&campos, etiqueta, como) {
        Ok(elegido) => elegido,
        Err(Error::NotFound(motivo)) => {
            let vista = vista_de(paths, args).unwrap_or(Value::Null);
            return Err(error_sin_etiqueta(&motivo, &vista));
        }
        Err(error) => return Err(error),
    };
    let ventana_str = ficha
        .get("ventana")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let formulario = Formulario::de(hwnd_de_texto(ventana_str)?);
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
