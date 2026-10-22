//! Calculations the MCP can make on its own that PVsyst also makes.
//!
//! By default the server only returns what PVsyst computes (through PVsystCLI,
//! the variant it simulates or its report). The MCP's own approximations stay
//! available behind `PVSYST_MCP_APROXIMACIONES=1`, and every result they
//! produce is labelled so it is never mistaken for a PVsyst figure.

use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::{json, Value};

/// Environment variable that enables the MCP's own approximations.
pub(crate) const VARIABLE_ENTORNO: &str = "PVSYST_MCP_APROXIMACIONES";

/// Value of the `origen` field on every approximated result.
pub(crate) const ORIGEN_APROXIMACION: &str = "aproximacion_mcp";

/// Argument that picks the MCP's approximation instead of PVsyst's own window.
pub(crate) const ARGUMENTO_VIA: &str = "via";
/// Value of [`ARGUMENTO_VIA`] that asks for the approximation.
pub(crate) const VIA_APROXIMACION: &str = "aproximacion";

/// Whether the caller asked for the MCP approximation (`via: "aproximacion"`)
/// rather than the PVsyst route, which is the default.
pub(crate) fn pide_aproximacion(args: &Value) -> bool {
    args.get(ARGUMENTO_VIA).and_then(Value::as_str) == Some(VIA_APROXIMACION)
}

/// Runs `aproximar` only when it was asked for and approximations are enabled,
/// labelling its result; otherwise runs `pvsyst`.
pub(crate) fn segun_via(
    paths: &Paths,
    args: &Value,
    aproximacion: &Aproximacion,
    rutas: (
        impl FnOnce() -> Result<Value>,
        impl FnOnce() -> Result<Value>,
    ),
) -> Result<Value> {
    let (pvsyst, aproximar) = rutas;
    if !pide_aproximacion(args) {
        return pvsyst();
    }
    exigir(paths, aproximacion)?;
    let mut v = aproximar()?;
    marcar(&mut v, aproximacion);
    Ok(v)
}

/// One MCP calculation that duplicates a PVsyst function, and the way to get
/// the same answer from PVsyst itself.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Aproximacion {
    /// What the MCP would compute, e.g. `"simular preestimar"`.
    pub(crate) funcion: &'static str,
    /// How to obtain it from PVsyst instead.
    pub(crate) alternativa: &'static str,
}

/// Fails unless approximations are enabled, naming the PVsyst route instead.
///
/// ```ignore
/// exigir(&paths, &Aproximacion { funcion: "simular preestimar",
///     alternativa: "simular ejecutar" })?;
/// ```
pub(crate) fn exigir(paths: &Paths, aproximacion: &Aproximacion) -> Result<()> {
    if paths.aproximaciones {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "'{}' es un cálculo propio del MCP, no de PVsyst. Para obtenerlo de PVsyst: {}. \
         Si aceptas la aproximación, define {VARIABLE_ENTORNO}=1 en el servidor MCP.",
        aproximacion.funcion, aproximacion.alternativa
    )))
}

/// Labels an approximated result so it is not read as a PVsyst figure.
pub(crate) fn marcar(resultado: &mut Value, aproximacion: &Aproximacion) {
    let Some(obj) = resultado.as_object_mut() else {
        return;
    };
    obj.insert("origen".into(), json!(ORIGEN_APROXIMACION));
    obj.insert(
        "aviso_origen".into(),
        json!(format!(
            "Calculado por el MCP, no por PVsyst; puede diferir del simulador. Equivalente en PVsyst: {}",
            aproximacion.alternativa
        )),
    );
}

#[cfg(test)]
#[path = "../../tests/unit/aproximaciones/mod.rs"]
mod tests;
