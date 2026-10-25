//! Sub-array DC sizing and nominal power deduction.
use super::componentes_variante::{exigir_sin_pre_dimensionado, Componentes};
use super::potencia_modulo::{pnom_del_modulo, ConsultaModulo};
use crate::aproximaciones::{self, Aproximacion};
use crate::args::missing;
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::{edit, object};
use crate::history;
use crate::projects::lifecycle::variantes::{abrir_variante, entero_opcional, VarianteAbierta};
use serde_json::{json, Value};

pub(crate) const APROXIMACION_DIMENSIONAR_KWP: Aproximacion = Aproximacion {
    funcion: "proyecto dimensionar por kwp via aproximacion",
    alternativa:
        "proyecto dimensionar con kwp sin 'via': lo dimensiona el pre-dimensionado de PVsyst (ventana Sistema)",
};

/// Largest DC target accepted (kWp): above any real single PVsyst variant.
const MAX_KWP: f64 = 5_000_000.0;
/// Largest string length accepted (modules in series).
const MAX_MODULOS_SERIE: u64 = 200;

/// Sizing calculation outputs to be written into `pvSubArray`.
struct ResultadoDimensionado {
    n_series: u64,
    n_strings: u64,
    realised_kwp: f64,
    n_inverter: Option<u64>,
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
    validar_n_serie(n)?;
    Ok(n)
}

fn validar_n_serie(n: u64) -> Result<()> {
    if n == 0 || n > MAX_MODULOS_SERIE {
        return Err(Error::Invalid(format!(
            "NModSerie={n} inválido: debe estar entre 1 y {MAX_MODULOS_SERIE} (pase 'n_mod_serie')"
        )));
    }
    Ok(())
}

fn validar_n_strings(n: u64) -> Result<()> {
    if n == 0 {
        return Err(Error::Invalid("n_strings debe ser mayor que cero".into()));
    }
    Ok(())
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
fn leer_kwp_opcional(args: &Value) -> Result<Option<f64>> {
    let Some(v) = args.get("kwp").or_else(|| args.get("kwp_objetivo")) else {
        return Ok(None);
    };
    let Some(kwp) = v.as_f64() else {
        return Ok(None);
    };
    if !kwp.is_finite() || kwp <= 0.0 || kwp > MAX_KWP {
        return Err(Error::Invalid(format!(
            "kwp={kwp} inválido: debe estar en (0, {MAX_KWP}]"
        )));
    }
    Ok(Some(kwp))
}

fn leer_kwp(args: &Value) -> Result<f64> {
    leer_kwp_opcional(args)?.ok_or_else(|| missing("kwp", ""))
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

struct DisposicionCalculada {
    n_series: u64,
    n_strings: u64,
    realised_kwp: f64,
    target_kwp: f64,
    es_nativo: bool,
}

fn deducir_disposicion(c: &ConsultaModulo<'_>, pnom: f64) -> Result<DisposicionCalculada> {
    if let (Some(ns), Some(nc)) = (
        entero_opcional(c.args, "n_mod_serie")?,
        entero_opcional(c.args, "n_strings")?,
    ) {
        validar_n_serie(ns)?;
        validar_n_strings(nc)?;
        let modulos = ns
            .checked_mul(nc)
            .ok_or_else(|| Error::Invalid(format!("{ns} × {nc} módulos desborda")))?;
        let real = modulos as f64 * pnom / 1000.0;
        let kwp_obj = leer_kwp_opcional(c.args)?.unwrap_or(real);
        return Ok(DisposicionCalculada {
            n_series: ns,
            n_strings: nc,
            realised_kwp: real,
            target_kwp: kwp_obj,
            es_nativo: true,
        });
    }
    aproximaciones::exigir(c.paths, &APROXIMACION_DIMENSIONAR_KWP)?;
    let kwp = leer_kwp(c.args)?;
    let ns = n_serie_de(c.args, c.root)?;
    let (nc, real) = calcular_strings(kwp, pnom, ns)?;
    Ok(DisposicionCalculada {
        n_series: ns,
        n_strings: nc,
        realised_kwp: real,
        target_kwp: kwp,
        es_nativo: false,
    })
}

/// Size a variant's sub-array: as given (`n_mod_serie` + `n_strings`), by
/// PVsyst's pre-sizing for a target `kwp`, or with the MCP's own rule when
/// `via: "aproximacion"` is asked for.
pub(crate) fn dimensionar_variante(paths: &Paths, args: &Value) -> Result<Value> {
    let manual = entero_opcional(args, "n_mod_serie")?.is_some()
        && entero_opcional(args, "n_strings")?.is_some();
    if manual || aproximaciones::pide_aproximacion(args) {
        return dimensionar_en_archivo(paths, args);
    }
    let kwp = leer_kwp(args)?;
    exigir_sin_pre_dimensionado(args)?;
    aproximaciones::exigir_ruta_pvsyst_sin_workspace(paths, args, "El pre-dimensionado")?;
    let v = abrir_variante(paths, args)?;
    crate::projects::gui::dimensionado::dimensionar(paths, &v.stem, &v.variante, kwp)
}

/// Writes the sizing into the variant file.
fn dimensionar_en_archivo(paths: &Paths, args: &Value) -> Result<Value> {
    let VarianteAbierta {
        workspace,
        stem,
        variante,
        vpath,
        bytes,
    } = abrir_variante(paths, args)?;
    let componentes = Componentes::leer(paths, args)?;
    let text = String::from_utf8_lossy(&bytes);

    let root = object::parse_first(&text)
        .ok_or_else(|| Error::Format("Variant is not a PVObject file".into()))?;
    let pan_name = match &componentes.modulo {
        Some(m) => m.clone(),
        None => modulo_de_variante(&root)?,
    };
    let consulta = ConsultaModulo {
        paths,
        args,
        root: &root,
    };
    let pnom = pnom_del_modulo(&consulta, &pan_name)?;
    let disp = deducir_disposicion(&consulta, pnom)?;

    let res = ResultadoDimensionado {
        n_series: disp.n_series,
        n_strings: disp.n_strings,
        realised_kwp: disp.realised_kwp,
        n_inverter: leer_inversores(args, &root),
    };
    let changes: Vec<_> = cambios_de_dimensionado(&res)
        .into_iter()
        .chain(componentes.cambios())
        .collect();
    let (new_bytes, outcomes) = edit::edit_fields(&bytes, "pvSubArray", &changes)?;
    let historial_id = history::reemplazar(&workspace, &vpath, &new_bytes, "dimensionar_variante")?;

    let ctx = ContextoRespuestaSizing {
        stem: &stem,
        variante: &variante,
        module: &pan_name,
        pnom,
        kwp: disp.target_kwp,
        historial_id: historial_id.as_deref(),
    };
    let mut resp = respuesta_dimensionado(&ctx, &res, &outcomes);
    if !disp.es_nativo {
        aproximaciones::marcar(&mut resp, &APROXIMACION_DIMENSIONAR_KWP);
    }
    Ok(resp)
}
