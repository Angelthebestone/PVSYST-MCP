//! Component creation from datasheet values (`componente crear_desde_ficha`).
//!
//! * **OND** inverters are cloned from the generic DataRO template, the
//!   datasheet values are applied and the DC efficiency profile is rescaled.
//! * **PAN** modules clone an existing text `.PAN` template in the workspace or DataRO.

pub(crate) mod inversor;
pub(crate) mod modulo;
#[cfg(test)]
#[path = "../../../tests/unit/components/ficha/tests.rs"]
mod tests;

use crate::args::{missing, req_str, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::edit;
use crate::workspace::validar_nombre;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) const BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BomPresencia {
    ConBom,
    SinBom,
}

pub(crate) fn sin_bom(bytes: &[u8]) -> (BomPresencia, &[u8]) {
    match bytes.strip_prefix(BOM) {
        Some(resto) => (BomPresencia::ConBom, resto),
        None => (BomPresencia::SinBom, bytes),
    }
}

pub(crate) fn con_bom(presencia: BomPresencia, cuerpo: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(cuerpo.len() + BOM.len());
    if presencia == BomPresencia::ConBom {
        out.extend_from_slice(BOM);
    }
    out.extend_from_slice(cuerpo);
    out
}

pub(crate) fn aplicar(
    bytes: &mut Vec<u8>,
    selector: &str,
    cambios: &[(String, Option<String>)],
) -> Result<()> {
    if cambios.is_empty() {
        return Ok(());
    }
    let (salida, _) = edit::edit_fields(bytes, selector, cambios)?;
    *bytes = salida;
    Ok(())
}

pub(crate) fn valor_texto(clave: &str, valor: &Value) -> Result<String> {
    match valor {
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        _ => Err(Error::Invalid(format!(
            "Valor no válido para '{clave}': usa texto o número"
        ))),
    }
}

/// Target path inside `ComposPV\<carpeta>`; refuses to overwrite.
pub(crate) fn destino(ws: &Path, carpeta: &str, archivo: &str) -> Result<PathBuf> {
    let dir = super::compos_dir(ws, carpeta);
    fs::create_dir_all(&dir)
        .map_err(|e| Error::io(format!("No se pudo crear {}", dir.display()), e))?;
    let dest = dir.join(archivo);
    if dest.exists() {
        return Err(Error::Invalid(format!(
            "Ya existe {}: elige otro nombre_nuevo",
            dest.display()
        )));
    }
    Ok(dest)
}

/// Create a component of kind `tipo` (`OND` or `PAN`) named `nombre_nuevo`
/// from the values in `datos`.
pub(crate) fn crear_desde_ficha(paths: &Paths, args: &Value) -> Result<Value> {
    let tipo = req_str(args, &["tipo"], "tipo")?.to_uppercase();
    let nombre = req_str(args, &["nombre_nuevo", "nombre"], "nombre_nuevo")?;
    validar_nombre(nombre)?;
    let datos = args
        .get("datos")
        .and_then(Value::as_object)
        .ok_or_else(|| missing("datos", "objeto con los valores de la ficha"))?;
    let ws = workspace_of(paths, args);
    let nombre = sin_extension(&tipo, nombre);
    match tipo.as_str() {
        "OND" => inversor::ond(&ws, &nombre, datos),
        "PAN" => modulo::pan(&ws, &nombre, datos),
        otro => Err(Error::Invalid(format!(
            "Tipo '{otro}' no soportado: usa OND o PAN"
        ))),
    }
}

/// `nombre` without a trailing extension equal to `tipo`: each creator appends
/// its own suffix, so `X.OND` must not become `X.OND.OND`.
fn sin_extension(tipo: &str, nombre: &str) -> String {
    match nombre.rsplit_once('.') {
        Some((raiz, ext)) if ext.eq_ignore_ascii_case(tipo) => raiz.to_string(),
        _ => nombre.to_string(),
    }
}
