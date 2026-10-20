//! Tool registry: unified single-source-of-truth declarations, schemas,
//! boundary validation and dispatch for MCP tools.

use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::Value;
use std::time::Instant;

pub(crate) mod componentes;
pub(crate) mod declaracion;
pub(crate) mod entorno;
pub(crate) mod experto;
pub(crate) mod proyectos;
pub(crate) mod recorte;
pub(crate) mod resultados;
pub(crate) mod simulacion;
pub(crate) mod sitios;
pub(crate) mod validacion;
pub(crate) mod workspace;
mod workspace_ejecucion;

#[cfg(test)]
#[path = "../../../tests/unit/server/tools/tests.rs"]
mod tests;

pub(crate) use declaracion::{Acceso, Herramienta};

/// Consolidated registry of every tool exposed by the MCP server.
static REGISTRO: &[Herramienta] = &[
    entorno::TOOL_ENTORNO,
    entorno::TOOL_DISENAR_SISTEMA,
    workspace::TOOL_LISTAR,
    workspace::TOOL_LEER,
    workspace::TOOL_EDITAR,
    workspace::TOOL_ESCRIBIR_ARCHIVO,
    proyectos::TOOL_PROYECTO,
    componentes::TOOL_COMPONENTE,
    sitios::TOOL_SITIO,
    simulacion::TOOL_SIMULAR,
    resultados::TOOL_RESULTADOS,
    workspace::TOOL_WORKSPACE,
    experto::TOOL_PVSYST_CLI,
    experto::TOOL_GUI_OBSERVAR,
    experto::TOOL_GUI_ACTUAR,
];

/// Returns the schema catalogue for `tools/list` according to access permissions.
pub(crate) fn list(paths: &Paths) -> Vec<Value> {
    REGISTRO
        .iter()
        .filter(|h| h.acceso == Acceso::Normal || paths.experto)
        .map(|h| h.esquema())
        .collect()
}

/// Executes one tool after validating discriminator, required presence, and property types.
pub(crate) fn call(paths: &Paths, name: &str, args: &Value) -> Result<Value> {
    let Some(h) = REGISTRO.iter().find(|h| h.nombre == name) else {
        return Err(Error::Invalid(format!("Unknown tool '{name}'")));
    };

    if h.acceso == Acceso::Experto && !paths.experto {
        return Err(Error::Invalid(
            "Herramienta experta: define PVSYST_MCP_EXPERTO=1 en el servidor MCP".into(),
        ));
    }

    if !args.is_object() && !args.is_null() {
        return Err(Error::Invalid("Arguments must be a JSON object".into()));
    }

    let t0 = Instant::now();
    let accion = h.validar(args)?;
    let args_efectivos = h.preparar_args(args, &accion);
    let mut v = (h.ejecutar)(paths, &args_efectivos)?;

    if let Some(lim) = recorte::limites(args) {
        recorte::recortar(&mut v, lim);
    }

    let ms = u64::try_from(t0.elapsed().as_millis()).unwrap_or(u64::MAX);
    if let Some(obj) = v.as_object_mut() {
        obj.insert("elapsed_ms".to_string(), Value::from(ms));
        if let Some(sig) = recorte::siguiente(name, &accion) {
            obj.insert("siguiente".to_string(), Value::from(sig));
        }
    }

    Ok(v)
}
