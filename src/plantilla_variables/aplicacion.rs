//! Application orchestration for RVT variable template management.

use super::dominio::{
    anadir_variables, eliminar_variables, parse_rvt, reordenar_variables, serializar_rvt,
    VARIABLES_VALIDAS,
};
use crate::args::{opt_str, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::history;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// Resolves an RVT file path within the workspace.
fn resolver_ruta_rvt(ws: &Path, args: &Value) -> Result<PathBuf> {
    let ruta_rel = opt_str(args, &["batch_rvt_file", "archivo", "ruta"])
        .ok_or_else(|| Error::Invalid("Falta 'batch_rvt_file' o 'archivo'".into()))?;
    let path = PathBuf::from(ruta_rel);
    if path.is_absolute() {
        Ok(path)
    } else if ruta_rel.contains('/') || ruta_rel.contains('\\') {
        Ok(ws.join(path))
    } else {
        let models = ws.join("Models").join(&path);
        if models.exists() {
            Ok(models)
        } else {
            Ok(ws.join(path))
        }
    }
}

/// Reads string items from a JSON array argument.
fn extraer_variables(args: &Value) -> Result<Vec<&str>> {
    let arr = args
        .get("variables")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::Invalid("Se requiere el campo 'variables' como arreglo".into()))?;
    let vars: Vec<&str> = arr.iter().filter_map(Value::as_str).collect();
    if vars.is_empty() {
        return Err(Error::Invalid(
            "El arreglo 'variables' no puede estar vacío".into(),
        ));
    }
    Ok(vars)
}

/// Reads existing variables from a file, returning empty if file does not exist.
fn leer_existentes(ruta: &Path) -> Result<Vec<String>> {
    if !ruta.is_file() {
        return Ok(Vec::new());
    }
    let texto = fs::read_to_string(ruta)
        .map_err(|e| Error::io(format!("No se pudo leer {}", ruta.display()), e))?;
    parse_rvt(&texto)
}

/// Executes an RVT template operation (listar, anadir, eliminar, reordenar, catalogo).
pub(crate) fn gestionar_rvt(paths: &Paths, args: &Value) -> Result<Value> {
    let ws = workspace_of(paths, args);
    let op = opt_str(args, &["operacion", "accion_rvt"]).unwrap_or("listar");

    if op == "catalogo" || op == "validas" {
        return Ok(json!({
            "ok": true,
            "operacion": "catalogo",
            "variables_validas": VARIABLES_VALIDAS,
            "total": VARIABLES_VALIDAS.len()
        }));
    }

    let ruta = resolver_ruta_rvt(&ws, args)?;
    let existentes = leer_existentes(&ruta)?;

    match op {
        "listar" => Ok(json!({
            "ok": true,
            "archivo": ruta.display().to_string(),
            "operacion": "listar",
            "variables": existentes,
            "total": existentes.len()
        })),
        "anadir" => {
            let nuevas = extraer_variables(args)?;
            let resultado = anadir_variables(&existentes, &nuevas)?;
            guardar_y_responder(&ws, &ruta, &resultado, "anadir variables rvt")
        }
        "eliminar" => {
            let a_quitar = extraer_variables(args)?;
            let resultado = eliminar_variables(&existentes, &a_quitar)?;
            guardar_y_responder(&ws, &ruta, &resultado, "eliminar variables rvt")
        }
        "reordenar" => {
            let orden = extraer_variables(args)?;
            let resultado = reordenar_variables(&existentes, &orden)?;
            guardar_y_responder(&ws, &ruta, &resultado, "reordenar variables rvt")
        }
        "crear" | "reemplazar" => {
            let nuevas = extraer_variables(args)?;
            let resultado = anadir_variables(&[], &nuevas)?;
            guardar_y_responder(&ws, &ruta, &resultado, "crear/reemplazar plantilla rvt")
        }
        otro => Err(Error::Invalid(format!(
            "Operación RVT '{otro}' desconocida; válidas: listar, anadir, eliminar, reordenar, crear, catalogo"
        ))),
    }
}

/// Persists variables via history snapshot and builds response JSON.
fn guardar_y_responder(
    ws: &Path,
    ruta: &Path,
    variables: &[String],
    motivo: &str,
) -> Result<Value> {
    if let Some(parent) = ruta.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| Error::io(format!("No se pudo crear {}", parent.display()), e))?;
        }
    }
    let contenido = serializar_rvt(variables);
    let id = history::reemplazar(ws, ruta, contenido.as_bytes(), motivo)?;
    Ok(json!({
        "ok": true,
        "archivo": ruta.display().to_string(),
        "variables": variables,
        "total": variables.len(),
        "historial_id": id
    }))
}

#[cfg(test)]
#[path = "../../tests/unit/plantilla_variables/aplicacion.rs"]
mod tests;
