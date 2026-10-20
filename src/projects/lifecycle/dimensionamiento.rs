//! Sub-array DC sizing and nominal power deduction.

use crate::args::missing;
use crate::components::pan;
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::{edit, object};
use crate::history;
use crate::projects::lifecycle::variantes::{abrir_variante, entero_opcional, VarianteAbierta};
use serde_json::{json, Value};

/// Largest DC target accepted (kWp): above any real single PVsyst variant.
const MAX_KWP: f64 = 5_000_000.0;
/// Largest module nominal power accepted (W): covers current utility modules.
const MAX_PNOM_MODULO_W: f64 = 3_000.0;
/// Largest string length accepted (modules in series).
const MAX_MODULOS_SERIE: u64 = 200;

/// Parameters for module nominal power (Pnom) deduction.
struct ConsultaModulo<'a> {
    paths: &'a Paths,
    args: &'a Value,
    root: &'a object::PvNode,
}

/// Sizing calculation outputs to be written into `pvSubArray`.
struct ResultadoDimensionado {
    n_series: u64,
    n_strings: u64,
    realised_kwp: f64,
    n_inverter: Option<u64>,
}

/// Module Pnom: explicit `pnom_modulo`, else the PAN file, else the variant's
/// own sizing, else the module name.
fn pnom_del_modulo(consulta: &ConsultaModulo<'_>, pan_name: &str) -> Result<f64> {
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
    validar_pnom(pnom, consulta.args.get("pnom_modulo").is_some())
}

/// Bounds the deduced module power; the `pnom_modulo` hint only makes sense
/// when the value was inferred rather than passed by the caller.
fn validar_pnom(pnom: f64, explicito: bool) -> Result<f64> {
    if !pnom.is_finite() || pnom <= 0.0 || pnom > MAX_PNOM_MODULO_W {
        let pista = if explicito {
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

/// Modules in series: explicit `n_mod_serie`, else the variant's current `NModSerie`.
fn n_serie_de(args: &Value, root: &object::PvNode) -> Result<u64> {
    let n = match entero_opcional(args, "n_mod_serie")? {
        Some(n) => n,
        None => root
            .find_deep("NModSerie")
            .and_then(|s| s.trim().parse::<u64>().ok())
            .ok_or_else(|| missing("n_mod_serie", "el módulo no declara NModSerie"))?,
    };
    if n == 0 || n > MAX_MODULOS_SERIE {
        return Err(Error::Invalid(format!(
            "NModSerie={n} inválido: debe estar entre 1 y {MAX_MODULOS_SERIE} (pase 'n_mod_serie')"
        )));
    }
    Ok(n)
}

fn cambios_de_dimensionado(res: &ResultadoDimensionado) -> Vec<(String, Option<String>)> {
    let mut changes = vec![
        ("NModSerie".into(), Some(res.n_series.to_string())),
        ("NStringCh".into(), Some(res.n_strings.to_string())),
        ("NStrOrient1".into(), Some(res.n_strings.to_string())),
        (
            "PNomSizing".into(),
            Some(format!("{:.1}", res.realised_kwp)),
        ),
    ];
    if let Some(ni) = res.n_inverter {
        changes.push(("NInverter".into(), Some(ni.to_string())));
    }
    changes
}

fn modulo_de_variante(root: &object::PvNode) -> Result<String> {
    root.find_deep("PVModule")
        .map(str::to_string)
        .ok_or_else(|| Error::Format("Variant has no PVModule= reference".into()))
}
fn leer_kwp(args: &Value) -> Result<f64> {
    let kwp = args
        .get("kwp")
        .or_else(|| args.get("kwp_objetivo"))
        .and_then(Value::as_f64)
        .ok_or_else(|| missing("kwp", ""))?;
    if !kwp.is_finite() || kwp <= 0.0 || kwp > MAX_KWP {
        return Err(Error::Invalid(format!(
            "kwp={kwp} inválido: debe estar en (0, {MAX_KWP}]"
        )));
    }
    Ok(kwp)
}

fn leer_inversores(args: &Value, root: &object::PvNode) -> Option<u64> {
    args.get("n_inversores")
        .and_then(Value::as_u64)
        .or_else(|| {
            root.find_deep("NInverter")
                .and_then(|s| s.parse::<u64>().ok())
        })
}

/// Strings needed for `kwp` and the power they realise. Inputs are validated,
/// so the counts are bounded; the module total is still checked for overflow.
fn calcular_strings(kwp: f64, pnom: f64, n_series: u64) -> Result<(u64, f64)> {
    let total = (kwp * 1000.0 / pnom).round().max(1.0);
    let n_strings = (total / n_series as f64).round().max(1.0) as u64;
    let modulos = n_series
        .checked_mul(n_strings)
        .ok_or_else(|| Error::Invalid(format!("{n_series} × {n_strings} módulos desborda")))?;
    Ok((n_strings, modulos as f64 * pnom / 1000.0))
}

struct ContextoRespuestaSizing<'a> {
    stem: &'a str,
    variante: &'a str,
    module: &'a str,
    pnom: f64,
    kwp: f64,
    historial_id: Option<&'a str>,
}

fn respuesta_dimensionado(
    ctx: &ContextoRespuestaSizing<'_>,
    res: &ResultadoDimensionado,
    applied: &[edit::ChangeOutcome],
) -> Value {
    json!({
        "project": ctx.stem,
        "variant": ctx.variante,
        "module": ctx.module,
        "module_pnom_w": ctx.pnom,
        "target_kwp": ctx.kwp,
        "n_mod_serie": res.n_series,
        "n_strings": res.n_strings,
        "total_modules": res.n_series * res.n_strings,
        "realised_kwp": (res.realised_kwp * 1000.0).round() / 1000.0,
        "applied": applied.iter().map(edit::ChangeOutcome::to_json).collect::<Vec<_>>(),
        "historial_id": ctx.historial_id,
    })
}

/// Size a variant's sub-array to a target DC power.
pub(crate) fn dimensionar_variante(paths: &Paths, args: &Value) -> Result<Value> {
    let VarianteAbierta {
        workspace,
        stem,
        variante,
        vpath,
        bytes,
    } = abrir_variante(paths, args)?;
    let text = String::from_utf8_lossy(&bytes);

    let root = object::parse_first(&text)
        .ok_or_else(|| Error::Format("Variant is not a PVObject file".into()))?;
    let pan_name = modulo_de_variante(&root)?;
    let consulta = ConsultaModulo {
        paths,
        args,
        root: &root,
    };
    let pnom = pnom_del_modulo(&consulta, &pan_name)?;

    let kwp = leer_kwp(args)?;
    let n_series = n_serie_de(args, &root)?;
    let (n_strings, realised_kwp) = calcular_strings(kwp, pnom, n_series)?;
    let res = ResultadoDimensionado {
        n_series,
        n_strings,
        realised_kwp,
        n_inverter: leer_inversores(args, &root),
    };
    let changes = cambios_de_dimensionado(&res);
    let (new_bytes, outcomes) = edit::edit_fields(&bytes, "pvSubArray", &changes)?;
    let historial_id = history::reemplazar(&workspace, &vpath, &new_bytes, "dimensionar_variante")?;

    let ctx = ContextoRespuestaSizing {
        stem: &stem,
        variante: &variante,
        module: &pan_name,
        pnom,
        kwp,
        historial_id: historial_id.as_deref(),
    };
    Ok(respuesta_dimensionado(&ctx, &res, &outcomes))
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
#[path = "../../../tests/unit/projects/lifecycle/dimensionamiento.rs"]
mod tests;
