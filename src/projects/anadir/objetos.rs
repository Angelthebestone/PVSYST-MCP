//! Generic PVObject insertion, deletion, and structural validation.

use crate::args::{self, flag, opt_str, req_str};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::edit::{self, ObjSpan};
use crate::format::{insert, object};
use crate::objects;
use crate::projects::anadir::{escribir_con_historial, leer_bytes};
use serde_json::{json, Value};

/// Fields listed per inserted object in the response.
const CAMPOS_MAX: usize = 12;

/// Hourly profiles, table values and continuation keys, left out of the response.
fn es_ruido(clave: &str) -> bool {
    ["Day_", "Values", "Accum", "ValMonth"]
        .iter()
        .any(|p| clave.starts_with(p))
        || clave.ends_with("_contd")
}

pub(crate) fn campos_de(bloque: &[u8]) -> Value {
    let Some(nodo) = object::parse_first(&String::from_utf8_lossy(bloque)) else {
        return Value::Null;
    };
    let total = nodo.fields.len();
    let mut json = nodo.to_json();
    if let Some(mapa) = json.as_object_mut() {
        mapa.retain(|k, _| k.starts_with('_') || !es_ruido(k));
        let claves: Vec<String> = mapa
            .keys()
            .filter(|k| !k.starts_with('_'))
            .cloned()
            .collect();
        for clave in claves.iter().skip(CAMPOS_MAX) {
            mapa.remove(clave);
        }
        let listados = claves.len().min(CAMPOS_MAX);
        mapa.insert("campos_omitidos".into(), json!(total - listados));
    }
    json
}

pub(crate) fn padre_de(spans: &[ObjSpan], si: usize) -> Option<String> {
    let s = &spans[si];
    spans
        .iter()
        .enumerate()
        .filter(|(j, t)| *j != si && t.open < s.open && t.close >= s.close)
        .max_by_key(|(_, t)| t.open)
        .map(|(_, t)| t.ty.clone())
}

pub(crate) fn verificar_tipo(bytes: &[u8], tipo: &str) -> Result<()> {
    if edit::spans_of(bytes).iter().any(|s| s.ty == tipo) {
        Ok(())
    } else {
        Err(Error::Invalid(format!(
            "La operación no dejó el objeto {tipo} en el archivo"
        )))
    }
}

pub(crate) fn tipo_del_bloque(bloque: &[u8]) -> Result<String> {
    edit::spans_of(bloque)
        .into_iter()
        .next()
        .map(|s| s.ty)
        .ok_or_else(|| Error::Format("El bloque no contiene ningún PVObject".into()))
}

/// `editar accion:insertar_objeto {archivo, padre, desde:{archivo, objeto}, rol?, reemplazar?}`.
pub(crate) fn insertar_objeto_archivo(paths: &Paths, args: &Value) -> Result<Value> {
    let ws = args::workspace_of(paths, args);
    let destino = objects::resolve_file(&ws, args)?;
    let padre = req_str(args, &["padre"], "padre")?;
    let desde = args
        .get("desde")
        .ok_or_else(|| args::missing("desde", "desde:{archivo, objeto}"))?;
    let origen = objects::resolve_file(&ws, desde)?;
    let objeto = req_str(desde, &["objeto"], "desde.objeto")?;
    let bloque = insert::extraer_objeto(&leer_bytes(&origen)?, objeto)?;
    let tipo = tipo_del_bloque(&bloque)?;
    let nuevo = insert::insertar_objeto(
        &leer_bytes(&destino)?,
        padre,
        &bloque,
        opt_str(args, &["rol"]),
        if flag(args, &["reemplazar"]) {
            insert::SiExiste::Reemplazar
        } else {
            insert::SiExiste::Rechazar
        },
    )?;
    verificar_tipo(&nuevo, &tipo)?;
    let id = escribir_con_historial(
        &ws,
        &destino,
        &nuevo,
        &format!("insertar {tipo} en {padre}"),
    )?;
    Ok(json!({
        "ok": true,
        "archivo": destino.display().to_string(),
        "insertado": tipo,
        "padre": padre,
        "historial_id": id,
        "campos": campos_de(&bloque)
    }))
}

/// `editar accion:eliminar_objeto {archivo, objeto}`. The root object cannot be removed.
pub(crate) fn eliminar_objeto_archivo(paths: &Paths, args: &Value) -> Result<Value> {
    let ws = args::workspace_of(paths, args);
    let destino = objects::resolve_file(&ws, args)?;
    let objeto = req_str(args, &["objeto"], "objeto")?;
    let antes = leer_bytes(&destino)?;
    let spans = edit::spans_of(&antes);
    if edit::find_span(&spans, objeto) == Some(0) {
        return Err(Error::Format(
            "No se puede eliminar el objeto raíz del archivo".into(),
        ));
    }
    let nuevo = insert::eliminar_objeto(&antes, objeto)?;
    if edit::spans_of(&nuevo).len() >= spans.len() {
        return Err(Error::Invalid(format!(
            "No se eliminó '{objeto}' del archivo"
        )));
    }
    let id = escribir_con_historial(&ws, &destino, &nuevo, &format!("eliminar {objeto}"))?;
    Ok(json!({
        "ok": true,
        "archivo": destino.display().to_string(),
        "eliminado": objeto,
        "historial_id": id
    }))
}

#[cfg(test)]
#[path = "../../../tests/unit/projects/anadir/objetos.rs"]
mod tests;
