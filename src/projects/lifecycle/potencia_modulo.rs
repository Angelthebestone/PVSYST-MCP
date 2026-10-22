//! Module nominal power (Pnom) as the variant and its PAN declare it.

use crate::components::pan;
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::object;
use serde_json::{json, Value};

/// Largest module nominal power accepted (W): covers current utility modules.
const MAX_PNOM_MODULO_W: f64 = 3_000.0;

/// Parameters for module nominal power (Pnom) deduction.
pub(super) struct ConsultaModulo<'a> {
    pub(super) paths: &'a Paths,
    pub(super) args: &'a Value,
    pub(super) root: &'a object::PvNode,
}

/// Module Pnom: explicit `pnom_modulo`, else the PAN file, else the variant's
/// own sizing, else the module name.
pub(super) fn pnom_del_modulo(consulta: &ConsultaModulo<'_>, pan_name: &str) -> Result<f64> {
    let pnom = if let Some(p) = consulta.args.get("pnom_modulo").and_then(Value::as_f64) {
        p
    } else {
        let mut pedido = json!({ "modulo": pan_name });
        if let Some(ws) = consulta.args.get("workspace") {
            pedido["workspace"] = ws.clone();
        }
        pan::leer_modulo(consulta.paths, &pedido)
            .ok()
            .and_then(|m| m.get("pnom").and_then(Value::as_f64))
            .or_else(|| pnom_from_sizing(consulta.root))
            .or_else(|| pnom_from_name(pan_name))
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "Module '{pan_name}' not found and its Pnom cannot be inferred; pass 'pnom_modulo'"
                ))
            })?
    };
    let origen = if consulta.args.get("pnom_modulo").is_some() {
        OrigenPnom::Argumento
    } else {
        OrigenPnom::Inferido
    };
    validar_pnom(pnom, origen)
}

/// Where the module Pnom came from.
#[derive(Clone, Copy, PartialEq, Eq)]
enum OrigenPnom {
    Argumento,
    Inferido,
}

/// Bounds the deduced module power; the `pnom_modulo` hint only makes sense
/// when the value was inferred rather than passed by the caller.
fn validar_pnom(pnom: f64, origen: OrigenPnom) -> Result<f64> {
    if !pnom.is_finite() || pnom <= 0.0 || pnom > MAX_PNOM_MODULO_W {
        let pista = if matches!(origen, OrigenPnom::Argumento) {
            ""
        } else {
            "; pase 'pnom_modulo'"
        };
        return Err(Error::Invalid(format!(
            "Pnom del módulo inválido ({pnom} W): debe estar en (0, {MAX_PNOM_MODULO_W}]{pista}"
        )));
    }
    Ok(pnom)
}

/// Pnom encoded in a module name, e.g. `Generic_Mono_300W.PAN` -> 300.
pub(crate) fn pnom_from_name(name: &str) -> Option<f64> {
    let stem = name.rsplit_once('.').map_or(name, |(s, _)| s);
    stem.split(|c: char| !c.is_ascii_alphanumeric())
        .filter_map(|tok| tok.strip_suffix(['W', 'w']))
        .filter_map(|n| n.parse::<f64>().ok())
        .find(|w| *w > 0.0)
}

/// Module Pnom (W) implied by the variant itself: `PNomSizing` (kWp) over the module count.
fn pnom_from_sizing(root: &object::PvNode) -> Option<f64> {
    let num = |k: &str| root.find_deep(k).and_then(|v| v.trim().parse::<f64>().ok());
    let modules = num("NModSerie")? * num("NStringCh")?;
    let kwp = num("PNomSizing")?;
    (modules > 0.0).then(|| (kwp * 1000.0 / modules).round())
}

#[cfg(test)]
#[path = "../../../tests/unit/projects/lifecycle/potencia_modulo.rs"]
mod tests;
