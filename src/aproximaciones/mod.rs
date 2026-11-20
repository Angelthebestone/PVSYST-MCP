//! Calculations the MCP can make on its own that PVsyst also makes.
//!
//! By default the server only returns what PVsyst computes (through PVsystCLI,
//! the variant it simulates or its report). The MCP's own approximations stay
//! available behind `PVSYST_MCP_APROXIMACIONES=1`, and every result they
//! produce is labelled so it is never mistaken for a PVsyst figure.

use crate::config::Paths;
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use serde_json::{json, Value};
use std::path::Path;

pub(crate) use crate::config::VARIABLE_APROXIMACIONES as VARIABLE_ENTORNO;

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

/// Rejects an explicit `workspace` other than the configured one on the PVsyst
/// route, because PVsyst's GUI only works on its own workspace. Approximations
/// read the given workspace, so they are never rejected here.
pub(crate) fn exigir_ruta_pvsyst_sin_workspace(
    paths: &Paths,
    args: &Value,
    quien: Texto,
) -> Result<()> {
    if pide_aproximacion(args) {
        return Ok(());
    }
    match args.get("workspace").and_then(Value::as_str) {
        Some(w) if Path::new(w) != paths.workspace.as_path() => Err(Error::Invalid(
            Texto {
                es: "{quien} lo hace PVsyst en su propio espacio de trabajo: no admite 'workspace'",
                en: "{quien} runs on PVsyst in its own workspace: 'workspace' is not supported",
            }
            .texto_con(paths.idioma, &[("quien", quien.en_idioma(paths.idioma))]),
        )),
        _ => Ok(()),
    }
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
    marcar(&mut v, aproximacion, paths.idioma);
    Ok(v)
}

/// One MCP calculation that duplicates a PVsyst function, and the way to get
/// the same answer from PVsyst itself.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Aproximacion {
    /// What the MCP would compute, e.g. `"simular preestimar"`.
    pub(crate) funcion: &'static str,
    /// How to obtain it from PVsyst instead.
    pub(crate) alternativa: Texto,
}

/// Fails unless approximations are enabled, naming the PVsyst route instead.
///
/// ```ignore
/// exigir(&paths, &Aproximacion { funcion: "simular preestimar",
///     alternativa: Texto { es: "simular ejecutar", en: "simular ejecutar" } })?;
/// ```
pub(crate) fn exigir(paths: &Paths, aproximacion: &Aproximacion) -> Result<()> {
    if paths.aproximaciones {
        return Ok(());
    }
    Err(Error::Invalid(Texto {
        es: "'{funcion}' es un cálculo propio del MCP, no de PVsyst. Para obtenerlo de PVsyst: {alternativa}. Si aceptas la aproximación, define {variable}=1 en el servidor MCP.",
        en: "'{funcion}' is an MCP in-house calculation, not a PVsyst one. To get it from PVsyst: {alternativa}. If you accept the approximation, set {variable}=1 in the MCP server.",
    }
    .texto_con(
        paths.idioma,
        &[
            ("funcion", aproximacion.funcion),
            ("alternativa", aproximacion.alternativa.en_idioma(paths.idioma)),
            ("variable", VARIABLE_ENTORNO),
        ],
    )))
}

/// Labels an approximated result so it is not read as a PVsyst figure.
pub(crate) fn marcar(resultado: &mut Value, aproximacion: &Aproximacion, idioma: Idioma) {
    let Some(obj) = resultado.as_object_mut() else {
        return;
    };
    obj.insert("origen".into(), json!(ORIGEN_APROXIMACION));
    obj.insert(
        "aviso_origen".into(),
        json!(Texto {
            es: "Calculado por el MCP, no por PVsyst; puede diferir del simulador. Equivalente en PVsyst: {alternativa}",
            en: "Computed by the MCP, not by PVsyst; it may differ from the simulator. PVsyst equivalent: {alternativa}",
        }
        .texto_con(idioma, &[("alternativa", aproximacion.alternativa.en_idioma(idioma))])),
    );
}

#[cfg(test)]
#[path = "../../tests/unit/aproximaciones/mod.rs"]
mod tests;
