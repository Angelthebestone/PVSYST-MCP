//! `gui_actuar accion:campo`: sets a form field by the label a person reads
//! next to it. The label comes from `gui_observar que:formulario` (OCR), so
//! no coordinates or per-version field tables are needed.

use super::acciones::clic_por_mensajes;
use super::conduccion::Formulario;
use super::ffi::{self, Hwnd};
use super::ocr::seleccion::{campo_por_etiqueta, valor_logico, CampoEtiquetado, Desambiguador};
use super::ocr::{self, esta_marcado};
use super::sondeo::sondear_hasta;
use crate::args::{opt_u32, req_str};
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::time::Duration;

pub(crate) const CLASES_TEXTO: &[&str] =
    &["TEdit", "TMemo", "TLabeledEdit", "TMaskEdit", "TSpinEdit"];
pub(crate) const CLASES_LISTA: &[&str] = &["TComboBox", "TComboBoxEx", "TListBox"];
const CLASES_OPCION: &[&str] = &["TCheckBox", "TRadioButton"];
/// How long a check box gets to show the state the click asked for: the click
/// is posted, so a read right after it can still return the old state.
const ESPERA_CASILLA: Duration = Duration::from_millis(1_000);
const SONDEO_CASILLA: Duration = Duration::from_millis(50);

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

/// The error of an option whose state PVsyst did not report: it names the
/// label, the class and the control, and says how to read it again.
fn opcion_ilegible(c: &CampoEtiquetado) -> Error {
    let etiqueta = c.etiqueta.as_deref().unwrap_or(c.clase.as_str());
    Error::Gui(format!(
        "Could not read the state of the option '{etiqueta}' ({}, 0x{:X}): PVsyst did not answer in {} ms. gui_observar que:formulario (or que:controles) shows the control, and gui_actuar accion:campo retries once the form answers",
        c.clase,
        c.hwnd,
        ffi::TIMEOUT_MENSAJE_MS
    ))
}

/// Current value of a field as text, for the before/after report. An option
/// whose state cannot be read is an error, not a value the control never held.
fn leer_valor(c: &CampoEtiquetado) -> Result<String> {
    if CLASES_OPCION.contains(&c.clase.as_str()) {
        return esta_marcado(c.hwnd)
            .map(|m| m.to_string())
            .ok_or_else(|| opcion_ilegible(c));
    }
    Ok(ffi::texto_control(c.hwnd))
}

/// The click was posted but the control did not show the requested state:
/// answering success would send the old state back as the new one.
fn error_marca(c: &CampoEtiquetado, pedido: bool, leido: Option<bool>) -> Error {
    let etiqueta = c.etiqueta.as_deref().unwrap_or(c.clase.as_str());
    let leido = leido.map_or_else(
        || "no answer from PVsyst".to_string(),
        |marcado| marcado.to_string(),
    );
    Error::Gui(format!(
        "The option '{etiqueta}' ({}) asked for '{pedido}' still reads '{leido}' when its {} ms deadline ran out. gui_observar que:formulario (or que:controles) shows the current state; gui_actuar accion:campo retries once the form answers",
        c.clase,
        ESPERA_CASILLA.as_millis()
    ))
}

fn esperar_marca(c: &CampoEtiquetado, pedido: bool) -> Result<()> {
    let mut leido = None;
    let alcanzado = sondear_hasta(ESPERA_CASILLA, SONDEO_CASILLA, || {
        leido = esta_marcado(c.hwnd);
        leido.filter(|marcado| *marcado == pedido)
    })
    .is_some();
    if alcanzado {
        return Ok(());
    }
    Err(error_marca(c, pedido, leido))
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
        let pedido = valor_logico(valor)?;
        let marcado = esta_marcado(c.hwnd).ok_or_else(|| opcion_ilegible(c))?;
        if marcado != pedido {
            clic_por_mensajes(c.hwnd)?;
            esperar_marca(c, pedido)?;
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

/// Whether the form answer reports the control enabled. A control the answer
/// does not describe is not written either: nothing proves it takes the value.
fn habilitado_en(ficha: &Value, hwnd: Hwnd) -> bool {
    let descrito =
        |c: &&Value| c["hwnd"].as_str().and_then(|t| hwnd_de_texto(t).ok()) == Some(hwnd);
    ficha["campos"]
        .as_array()
        .into_iter()
        .flatten()
        .find(descrito)
        .is_some_and(|c| c["habilitado"].as_bool() == Some(true))
}

/// A disabled control swallows the text and the click without a trace, so the
/// answer would report a value PVsyst never took.
fn deshabilitado(c: &CampoEtiquetado) -> Error {
    let etiqueta = c.etiqueta.as_deref().unwrap_or(c.clase.as_str());
    Error::Gui(format!(
        "The field '{etiqueta}' ({}) is disabled: PVsyst ignores the text and the click and keeps its own value. Another control of the form enables it: gui_observar que:formulario (or que:controles with todos:true) shows which one, and gui_actuar accion:campo on it retries",
        c.clase
    ))
}

/// The field `etiqueta` names in `ficha`, if PVsyst reports it enabled.
fn elegir_campo(ficha: &Value, etiqueta: &str, como: Desambiguador<'_>) -> Result<CampoEtiquetado> {
    let campos = campos_de(ficha)?;
    let elegido = campo_por_etiqueta(&campos, etiqueta, como)?.clone();
    if habilitado_en(ficha, elegido.hwnd) {
        return Ok(elegido);
    }
    Err(deshabilitado(&elegido))
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
    let como = Desambiguador {
        clase: args.get("clase").and_then(Value::as_str),
        indice: opt_u32(args, "indice")?.map(|i| i as usize),
    };
    let elegido = match elegir_campo(&ficha, etiqueta, como) {
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
    let antes = leer_valor(&elegido)?;
    poner_valor(&formulario, &elegido, valor)?;
    let despues = leer_valor(&elegido)?;
    Ok(json!({
        "accion": "campo",
        "etiqueta": elegido.etiqueta,
        "clase": elegido.clase,
        "antes": antes,
        "despues": despues,
    }))
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/campo_por_etiqueta.rs"]
mod tests;
