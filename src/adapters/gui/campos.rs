//! External field actions for `gui_actuar`: type text, choose a combo item,
//! switch tab pages and wait for a form to close. The form geometry and the
//! caption/coordinate vocabulary live in [`super::conduccion`].

use super::conduccion::{esperar_cierre, Campo, Formulario};
use super::ffi::{self, Hwnd};
use super::modal;
use super::ventanas::{class_of, hwnd_de};
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::time::Duration;

/// VCL control classes accepted from the `campo` argument.
fn clase_de_campo(clase: &str) -> Result<&'static str> {
    match clase.to_ascii_lowercase().as_str() {
        "tedit" | "edit" => Ok("TEdit"),
        "tcombobox" | "combo" => Ok("TComboBox"),
        "tradiobutton" | "opcion" => Ok("TRadioButton"),
        "tcheckbox" | "check" => Ok("TCheckBox"),
        _ => Err(Error::Invalid(format!(
            "Clase de campo no soportada '{clase}'; usa TEdit, TComboBox, TRadioButton o TCheckBox"
        ))),
    }
}

/// Field selector from the `campo` object: `{clase, x, y}` at 96 DPI.
pub(crate) fn campo_de(args: &Value) -> Result<Campo> {
    let campo = args
        .get("campo")
        .ok_or_else(|| crate::args::missing("campo", "{clase, x, y}"))?;
    let clase = campo
        .get("clase")
        .and_then(Value::as_str)
        .unwrap_or("TEdit");
    let coordenada = |clave: &str| -> Result<i32> {
        campo
            .get(clave)
            .and_then(Value::as_i64)
            .and_then(|v| i32::try_from(v).ok())
            .ok_or_else(|| {
                crate::args::missing(
                    &format!("campo.{clave}"),
                    "posición en px a 96 DPI dentro del formulario",
                )
            })
    };
    Ok(Campo {
        clase: clase_de_campo(clase)?,
        x: coordenada("x")?,
        y: coordenada("y")?,
    })
}

/// Target window of an input action; refused while a modal dialog disables it.
fn objetivo_habilitado(args: &Value) -> Result<Hwnd> {
    let h = hwnd_de(args, "ventana")?;
    modal::exigir_habilitada(h, modal::PARA_ACTUAR)?;
    Ok(h)
}

/// Rejects a plain-window target: typing or picking into a form window has no
/// effect, so the caller must name the control (`campo`) or pick a control of `clases`.
fn exigir_clase(h: Hwnd, clases: &[&str], accion: &str) -> Result<()> {
    let clase = class_of(h);
    if clases.contains(&clase.as_str()) {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "'{accion}' sobre la clase {clase} no tiene efecto: indique 'campo' (clase, x, y) del control o una ventana de clase {}",
        clases.join("/")
    )))
}

fn formulario(args: &Value) -> Result<Formulario> {
    objetivo_habilitado(args).map(Formulario::de)
}

/// Types `texto` into a form field (`campo`) or directly into the target control.
pub(crate) fn escribir(_paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    let texto = crate::args::req_str(args, &["texto"], "texto a escribir")?;
    if args.get("campo").is_some() {
        let f = formulario(args)?;
        f.escribir(campo_de(args)?, texto)?;
        return Ok(json!({
            "accion": "escribir",
            "hwnd": format!("0x{:X}", f.h),
            "clase": f.clase(),
            "pid": f.pid(),
            "texto": texto,
        }));
    }
    let h = objetivo_habilitado(args)?;
    exigir_clase(h, super::campo_por_etiqueta::CLASES_TEXTO, "escribir")?;
    ffi::teclear_en(h, texto)?;
    Ok(json!({
        "accion": "escribir",
        "hwnd": format!("0x{h:X}"),
        "texto": texto,
    }))
}

/// Picks the first combo item that starts with `texto`.
pub(crate) fn seleccionar(_paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    let texto = crate::args::req_str(args, &["texto"], "prefijo del item de la lista")?;
    let elegido = if args.get("campo").is_some() {
        let f = formulario(args)?;
        f.seleccionar(campo_de(args)?, texto)?;
        true
    } else {
        let h = objetivo_habilitado(args)?;
        exigir_clase(h, super::campo_por_etiqueta::CLASES_LISTA, "seleccionar")?;
        ffi::elegir_item_por_clase(h, &class_of(h), texto)?
    };
    if !elegido {
        return Err(Error::Invalid(format!(
            "PVsyst no ofrece '{texto}' en esa lista"
        )));
    }
    Ok(json!({ "accion": "seleccionar", "texto": texto }))
}

/// Shows tab page `indice` of the target form.
pub(crate) fn pagina(_paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    let indice = match args.get("indice") {
        None => return Err(crate::args::missing("indice", "pestaña a mostrar")),
        Some(v) => v
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .ok_or_else(|| {
                Error::Invalid("'indice' debe ser un entero mayor o igual que 0".into())
            })?,
    };
    let f = formulario(args)?;
    f.pagina(indice)?;
    Ok(json!({ "accion": "pagina", "hwnd": format!("0x{:X}", f.h), "indice": indice }))
}

/// Waits until the target window is gone or hidden.
pub(crate) fn esperar_cierre_accion(_paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    let h = hwnd_de(args, "ventana")?;
    let espera_ms = args
        .get("timeout_ms")
        .and_then(Value::as_u64)
        .unwrap_or(10_000);
    let cerrado = esperar_cierre(h, Duration::from_millis(espera_ms));
    Ok(json!({
        "accion": "esperar_cierre",
        "hwnd": format!("0x{h:X}"),
        "cerrado": cerrado,
        "espera_ms": espera_ms,
    }))
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/campos.rs"]
mod tests;
