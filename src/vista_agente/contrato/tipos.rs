//! Types of the `vista` block: the same agent-facing answer for every tool.

use serde::Serialize;
use serde_json::Value;
use std::collections::HashSet;

/// Outcome of one call as an agent should read it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Estado {
    Ok,
    Parcial,
    Error,
}

/// Where a value comes from, so an approximation is never read as a PVsyst figure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum Origen {
    #[serde(rename = "pvsyst")]
    Pvsyst,
    #[serde(rename = "aproximacion_mcp")]
    AproximacionMcp,
    #[serde(rename = "mcp")]
    Mcp,
}

/// One relevant value with a stable key. A missing `unidad` means none; a
/// missing `editable` means read-only.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct Campo {
    pub(crate) clave: String,
    pub(crate) valor: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) unidad: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub(crate) editable: bool,
}

/// A call the agent can make as it is: `{tool, argumentos}`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct PasoSiguiente {
    pub(crate) tool: String,
    pub(crate) argumentos: Value,
    pub(crate) motivo: &'static str,
}

/// A list the detail level cut, and how to ask for the rest.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct Recorte {
    pub(crate) clave: String,
    pub(crate) mostrados: usize,
    pub(crate) total: usize,
    pub(crate) pedir_resto: &'static str,
}

/// Why a call failed and what to do about it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct ErrorVista {
    pub(crate) codigo: String,
    pub(crate) motivo: String,
    pub(crate) causa_probable: &'static str,
    pub(crate) que_hacer: &'static str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) claves_validas: Vec<String>,
}

/// The `vista` block added to every tool answer.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct Vista {
    pub(crate) estado: Estado,
    pub(crate) resumen: String,
    /// Where every value of `campos` comes from: they all belong to the same call.
    pub(crate) origen: Origen,
    pub(crate) campos: Vec<Campo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) avisos: Vec<String>,
    pub(crate) siguiente: Vec<PasoSiguiente>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) recortado: Vec<Recorte>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<ErrorVista>,
}

impl Vista {
    /// Checks the invariants of the contract; `sin_recorte` are the keys that must never be cut.
    pub(crate) fn validar(&self, sin_recorte: &[&str]) -> Result<(), String> {
        let mut vistas = HashSet::new();
        if let Some(repetida) = self.campos.iter().find(|c| !vistas.insert(&c.clave)) {
            return Err(format!("clave repetida en campos: {}", repetida.clave));
        }
        if let Some(r) = self
            .recortado
            .iter()
            .find(|r| sin_recorte.contains(&r.clave.as_str()))
        {
            return Err(format!("se recortó una clave protegida: {}", r.clave));
        }
        if self.resumen.trim().is_empty() {
            return Err("resumen vacío".into());
        }
        if self.estado == Estado::Error && self.error.is_none() {
            return Err("estado error sin bloque error".into());
        }
        match self.siguiente.iter().find(|s| !s.argumentos.is_object()) {
            Some(s) => Err(format!("siguiente de '{}' sin argumentos objeto", s.tool)),
            None => Ok(()),
        }
    }
}
