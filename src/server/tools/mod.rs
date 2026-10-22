//! Tool registry: unified single-source-of-truth declarations, schemas,
//! boundary validation and dispatch for MCP tools.

use crate::config::Paths;
use crate::error::{Error, Result};

use crate::progreso::SeguimientoTarea;
use declaracion::Ejecutor;
use serde_json::{json, Value};
use std::time::Instant;

mod aproximadas;
pub(crate) mod componentes;
pub(crate) mod declaracion;
pub(crate) mod entorno;
pub(crate) mod nativos;
pub(crate) mod proyectos;
pub(crate) mod recorte;
mod restricciones_esquema;
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
                filtrar_aproximaciones(h, &mut esquema);
            }
            esquema
        })
        .collect()
}

fn filtrar_aproximaciones(h: &Herramienta, esquema: &mut Value) {
    if let Some(disc) = &h.discriminador {
        aproximadas::ocultar(h.nombre, disc.campo, esquema);
    }
    aproximadas::ocultar_via(esquema);
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
    let args_efectivos = h.preparar_args(args)?;
    let accion_candidata = h
        .discriminador
        .as_ref()
        .and_then(|disc| {
            args_efectivos
                .get(disc.campo)
                .and_then(Value::as_str)
                .or(disc.defecto)
        })
        .unwrap_or_default();
    aproximadas::exigir(paths, name, accion_candidata)?;
    let accion = h.validar(&args_efectivos)?;
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

/// Returns the declarative map of synonyms (argument and value aliases) for every tool.
pub(crate) fn tabla_alias() -> Value {
    let mut mapa = serde_json::Map::new();
    for h in REGISTRO {
        if h.sinonimos.argumentos.is_empty() && h.sinonimos.valores.is_empty() {
            continue;
        }
        let mut tool_obj = serde_json::Map::new();
        if !h.sinonimos.argumentos.is_empty() {
            tool_obj.insert(
                "argumentos".to_string(),
                pares_a_objeto(h.sinonimos.argumentos),
            );
        }
        if !h.sinonimos.valores.is_empty() {
            tool_obj.insert("valores".to_string(), pares_a_objeto(h.sinonimos.valores));
        }
        mapa.insert(h.nombre.to_string(), Value::Object(tool_obj));
    }
    Value::Object(mapa)
}

fn pares_a_objeto(pares: &[(&str, &str)]) -> Value {
    let mut obj = serde_json::Map::new();
    for &(alias, canonico) in pares {
        obj.insert(alias.to_string(), json!(canonico));
    }
    Value::Object(obj)
}

#[cfg(test)]
pub(crate) fn generar_bloque_alias_markdown() -> String {
    let mut out = String::from("<!-- alias:inicio -->\n| Herramienta / Tool | Argumentos (alias → canónico) | Valores (alias → canónico) |\n|---|---|---|\n");
    for h in REGISTRO {
        if h.sinonimos.argumentos.is_empty() && h.sinonimos.valores.is_empty() {
            continue;
        }
        let args_str = if h.sinonimos.argumentos.is_empty() {
            "—".to_string()
        } else {
            let mut items: Vec<String> = h
                .sinonimos
                .argumentos
                .iter()
                .map(|(a, c)| format!("`{a}` → `{c}`"))
                .collect();
            items.sort();
            items.join(", ")
        };

        let vals_str = if h.sinonimos.valores.is_empty() {
            "—".to_string()
        } else {
            let mut items: Vec<String> = h
                .sinonimos
                .valores
                .iter()
                .map(|(a, c)| format!("`{a}` → `{c}`"))
                .collect();
            items.sort();
            items.join(", ")
        };

        out.push_str(&format!(
            "| `{}` | {} | {} |\n",
            h.nombre, args_str, vals_str
        ));
    }
    out.push_str("<!-- alias:fin -->");
    out
}
