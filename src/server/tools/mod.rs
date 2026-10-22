//! Tool registry: unified single-source-of-truth declarations, schemas,
//! boundary validation and dispatch for MCP tools.

use crate::config::Paths;
use crate::error::{Error, Result};

use crate::progreso::SeguimientoTarea;
use declaracion::Ejecutor;
use serde_json::Value;
use std::time::Instant;

mod aproximadas;
pub(crate) mod componentes;
pub(crate) mod declaracion;
pub(crate) mod entorno;
pub(crate) mod nativos;
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

pub(crate) use declaracion::Herramienta;

/// Whether `tool`/`accion` is an MCP approximation (gated by `PVSYST_MCP_APROXIMACIONES`).
pub(crate) fn es_aproximacion(tool: &str, accion: &str) -> bool {
    aproximadas::de(tool, accion).is_some()
}

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
    nativos::TOOL_PVSYST_CLI,
    nativos::TOOL_GUI_OBSERVAR,
    nativos::TOOL_GUI_ACTUAR,
];

/// Returns the schema catalogue for `tools/list`; without approximations, the
/// in-house actions and the `via: "aproximacion"` route are hidden.
pub(crate) fn list(paths: &Paths) -> Vec<Value> {
    REGISTRO
        .iter()
        .map(|h| {
            let mut esquema = h.esquema();
            if !paths.aproximaciones {
                if let Some(disc) = &h.discriminador {
                    aproximadas::ocultar(h.nombre, disc.campo, &mut esquema);
                }
                aproximadas::ocultar_via(&mut esquema);
            }
            esquema
        })
        .collect()
}

/// One tool invocation: registry lookup, validated arguments and progress channel.
pub(crate) struct Solicitud<'a> {
    pub(crate) paths: &'a Paths,
    pub(crate) name: &'a str,
    pub(crate) args: &'a Value,
    pub(crate) seguimiento: &'a SeguimientoTarea<'a>,
}

/// Executes one tool after validating discriminator, required presence, and property types.
pub(crate) fn call_seguido(solicitud: &Solicitud<'_>) -> Result<Value> {
    let paths = solicitud.paths;
    let name = solicitud.name;
    let args = solicitud.args;
    let seguimiento = solicitud.seguimiento;
    let Some(h) = REGISTRO.iter().find(|h| h.nombre == name) else {
        return Err(Error::NotFound(format!("Unknown tool '{name}'")));
    };

    if !args.is_object() && !args.is_null() {
        return Err(Error::Invalid("Arguments must be a JSON object".into()));
    }

    let t0 = Instant::now();
    let accion = h.validar(args)?;
    aproximadas::exigir(paths, name, &accion)?;
    let args_efectivos = h.preparar_args(args, &accion);
    let mut v = match h.ejecutar {
        Ejecutor::Simple(ejecutar) => ejecutar(paths, &args_efectivos)?,
        Ejecutor::Seguido(ejecutar) => ejecutar(paths, &args_efectivos, seguimiento)?,
    };
    aproximadas::marcar(name, &accion, &mut v);

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
