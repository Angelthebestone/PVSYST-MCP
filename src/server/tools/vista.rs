//! Adds the `vista` block to every tool answer, success or failure. Next steps
//! are kept only when the target tool accepts them exactly as written.

use super::recorte::SIN_RECORTE;
use super::{aproximadas, Herramienta, Solicitud, REGISTRO};
use crate::config::Paths;
use crate::error::Error;
use crate::vista_agente::contrato::{construir, desde_error, Contexto, Vista};
use serde_json::Value;

fn herramienta(nombre: &str) -> Option<&'static Herramienta> {
    REGISTRO.iter().find(|h| h.nombre == nombre)
}

/// Whether the tool named by a step accepts its arguments as they are.
fn acepta_paso(paths: &Paths, tool: &str, argumentos: &Value) -> bool {
    let Some(h) = herramienta(tool) else {
        return false;
    };
    h.preparar_args(argumentos)
        .and_then(|efectivos| {
            aproximadas::exigir(paths, tool, h.accion(&efectivos))?;
            h.validar(&efectivos, paths.aproximaciones)
        })
        .is_ok()
}

fn solo_ejecutables(paths: &Paths, vista: &mut Vista) {
    vista.siguiente.retain_mut(|p| {
        let Some(h) = herramienta(&p.tool) else {
            return false;
        };
        if let Some(argumentos) = p.argumentos.as_object_mut() {
            argumentos.retain(|clave, _| h.declara(clave));
        }
        acepta_paso(paths, &p.tool, &p.argumentos)
    });
}

/// Adds `vista` to an answer that arrived. Answers that are not objects are left as they are.
pub(super) fn anadir_vista(paths: &Paths, contexto: &Contexto<'_>, respuesta: &mut Value) {
    if !respuesta.is_object() {
        return;
    }
    let mut vista = construir(contexto, respuesta);
    solo_ejecutables(paths, &mut vista);
    if let Err(motivo) = vista.validar(SIN_RECORTE) {
        tracing::warn!(tool = contexto.tool, %motivo, "vista incoherente");
    }
    if let (Some(obj), Ok(valor)) = (respuesta.as_object_mut(), serde_json::to_value(&vista)) {
        obj.insert("vista".to_string(), valor);
    }
}

/// Keys the caller can use to fix an input error: what the action requires,
/// or every argument the tool declares.
fn claves_validas(h: &Herramienta, accion: &str, error: &Error) -> Vec<String> {
    if error.kind() == "missing_arg" {
        return h
            .requeridos_de(accion)
            .into_iter()
            .map(str::to_string)
            .collect();
    }
    h.propiedades
        .iter()
        .map(|(k, _)| (*k).to_string())
        .collect()
}

/// The `vista` block of a failed call, or `None` when the tool is unknown.
pub(crate) fn vista_de_error(solicitud: &Solicitud<'_>, error: &Error) -> Option<Value> {
    let h = herramienta(solicitud.name)?;
    let efectivos = h
        .preparar_args(solicitud.args)
        .unwrap_or_else(|_| solicitud.args.clone());
    let accion = h.accion(&efectivos);
    let contexto = Contexto {
        tool: solicitud.name,
        accion,
        args: &efectivos,
    };
    let mut vista = desde_error(&contexto, error, claves_validas(h, accion, error));
    solo_ejecutables(solicitud.paths, &mut vista);
    serde_json::to_value(&vista).ok()
}

#[cfg(test)]
#[path = "../../../tests/unit/server/tools/vista_tests.rs"]
mod tests;
