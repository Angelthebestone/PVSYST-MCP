//! Step 4: Array sizing and inverter electrical window calculations.

use super::parametros::DimensionadoParametros;
use super::pasos::PasoDimensionadoSalida;
use crate::components::{pan, resolve_component};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::object;
use crate::projects::lifecycle;
use serde_json::{json, Map, Value};

const FACTOR_SEGURIDAD_VOC: f64 = 1.15;
const WATTS_POR_KILOWATT: f64 = 1000.0;
const RANGO_SERIE_OFFSET: u32 = 2;

/// Electrical parameters for series string sizing.
pub(crate) struct VentanaElectrica {
    pub(crate) kwp: f64,
    pub(crate) pmod_w: f64,
    pub(crate) voc_v: f64,
    pub(crate) vmpp_v: f64,
    pub(crate) vmpp_min_v: f64,
    pub(crate) vabs_max_v: f64,
}

/// Series/parallel layout that gets closest to `kwp` within the inverter's DC
/// window: Ns keeps the MPP voltage above `vmpp_min_v` and the cold
/// open-circuit voltage (+15 % margin) below `vabs_max_v`. Ties go to the
/// larger string count. `None` when no Ns fits the window.
pub(crate) fn dimensionar_serie(
    kwp: f64,
    pmod_w: f64,
    voc_v: f64,
    vmpp_v: f64,
    vmpp_min_v: f64,
    vabs_max_v: f64,
) -> Option<(u32, u32)> {
    dimensionar_serie_ventana(&VentanaElectrica {
        kwp,
        pmod_w,
        voc_v,
        vmpp_v,
        vmpp_min_v,
        vabs_max_v,
    })
}

/// Sizing calculation over a typed electrical window DTO.
pub(crate) fn dimensionar_serie_ventana(v: &VentanaElectrica) -> Option<(u32, u32)> {
    if [v.pmod_w, v.voc_v, v.vmpp_v].iter().any(|x| *x <= 0.0) {
        return None;
    }
    let ns_min = (v.vmpp_min_v / v.vmpp_v).ceil().max(1.0) as u32;
    let ns_max = (v.vabs_max_v / (FACTOR_SEGURIDAD_VOC * v.voc_v)).floor() as u32;
    mejor_disposicion(ns_min..=ns_max, v.kwp * WATTS_POR_KILOWATT, v.pmod_w)
}

/// Best `(Ns, Np)` over the candidate series counts: `Np` rounds to the target power
/// and the layout with the smallest power error wins; ties go to the larger Ns.
fn mejor_disposicion(
    series: std::ops::RangeInclusive<u32>,
    objetivo_w: f64,
    pmod_w: f64,
) -> Option<(u32, u32)> {
    series
        .map(|ns| {
            let np = (objetivo_w / (f64::from(ns) * pmod_w)).round().max(1.0) as u32;
            let error = (f64::from(ns) * f64::from(np) * pmod_w - objetivo_w).abs();
            (ns, np, error)
        })
        .reduce(|mejor, c| if c.2 <= mejor.2 { c } else { mejor })
        .map(|(ns, np, _)| (ns, np))
}

/// Layout from the inverter's DC window (`VMppMin`, `VAbsMax` of the OND file)
/// and the module's STC values. `None` when any electrical datum is missing.
fn disposicion_electrica(
    paths: &Paths,
    inversor: Option<&str>,
    kwp: f64,
    pmod: f64,
    modulo: &Value,
) -> Option<(u32, u32)> {
    let inversor = inversor?;
    let voc = modulo.get("voc_v").and_then(Value::as_f64)?;
    let vmpp = modulo.get("vmpp_v").and_then(Value::as_f64)?;
    let consulta = crate::components::ConsultaComponente::referencia("OND", inversor);
    let ond = resolve_component(&paths.workspace, &consulta).ok()?;
    let bytes = crate::components::leer_bytes_componente(&ond).ok()?;
    let texto = String::from_utf8_lossy(&bytes).into_owned();
    let root = object::parse_first(&texto)?;
    let vmin = root.find_deep("VMppMin")?.trim().parse::<f64>().ok()?;
    let vabs = root.find_deep("VAbsMax")?.trim().parse::<f64>().ok()?;
    dimensionar_serie(kwp, pmod, voc, vmpp, vmin, vabs)
}

/// Series count within ±2 of the template's `ns0` and the parallel count that
/// gets closest to `kwp` for a module of `pmod_w`. Used when the inverter window
/// is unknown. Returns (Ns, Np, realised kWp); ties go to the larger Ns.
pub(crate) fn ajustar_serie(kwp: f64, pmod_w: f64, ns0: u32) -> Option<(u32, u32, f64)> {
    if kwp <= 0.0 || pmod_w <= 0.0 || ns0 == 0 {
        return None;
    }
    let min_s = ns0.saturating_sub(RANGO_SERIE_OFFSET).max(1);
    let max_s = ns0 + RANGO_SERIE_OFFSET;
    let series = min_s..=max_s;
    mejor_disposicion(series, kwp * WATTS_POR_KILOWATT, pmod_w).map(|(ns, np)| {
        (
            ns,
            np,
            f64::from(ns) * f64::from(np) * pmod_w / WATTS_POR_KILOWATT,
        )
    })
}

/// Series count and the criterion that chose it: the inverter's DC window when the
/// datasheets allow it, else a local search around the template's series count.
fn serie_de_plantilla(
    paths: &Paths,
    kwp: f64,
    pmod_w: f64,
    (inversor, modulo): (Option<&str>, Option<&Value>),
    refs: &Referencias,
) -> Option<(u32, &'static str)> {
    let ventana = modulo.and_then(|m| disposicion_electrica(paths, inversor, kwp, pmod_w, m));
    if let Some((ns, _)) = ventana {
        return Some((ns, "ventana_electrica"));
    }
    refs.ns_plantilla
        .and_then(|ns0| ajustar_serie(kwp, pmod_w, ns0))
        .map(|(ns, _, _)| (ns, "ajuste_local"))
}

/// References and sizing hints of the variant created for `nombre`. Each field
/// is `None` when the variant or the value is missing.
#[derive(Default)]
pub(crate) struct Referencias {
    pub(crate) modulo: Option<String>,
    pub(crate) inversor: Option<String>,
    pub(crate) ns_plantilla: Option<u32>,
    pub(crate) pnom_derivado: Option<f64>,
}

pub(crate) fn referencias_de_variante(paths: &Paths, nombre: &str) -> Referencias {
    let ruta = paths
        .workspace
        .join("Projects")
        .join(format!("{nombre}.VC0"));
    let Ok(bytes) = std::fs::read(ruta) else {
        return Referencias::default();
    };
    let texto = String::from_utf8_lossy(&bytes);
    let Some(root) = object::parse_first(&texto) else {
        return Referencias::default();
    };
    let numero = |clave: &str| {
        root.find_deep(clave)
            .and_then(|v| v.trim().parse::<f64>().ok())
    };
    let ns = numero("NModSerie");
    let pnom_derivado = match (numero("PNomSizing"), ns, numero("NStringCh")) {
        (Some(kwp), Some(serie), Some(cadenas)) if serie > 0.0 && cadenas > 0.0 => {
            Some(kwp * WATTS_POR_KILOWATT / (serie * cadenas))
        }
        _ => None,
    };
    Referencias {
        modulo: root.find_deep("PVModule").map(str::to_string),
        inversor: root.find_deep("GInverter").map(str::to_string),
        ns_plantilla: ns.filter(|n| *n >= 1.0).map(|n| n as u32),
        pnom_derivado,
    }
}

/// Step 4: size the array to the target kWp; returns the realised kWp.
/// If module reading fails, the error is propagated with context.
fn resolver_serie_plantilla(
    paths: &Paths,
    params: &DimensionadoParametros,
    refs: &Referencias,
    modulo_info: Option<&Value>,
) -> (Option<u32>, &'static str) {
    if let Some(ns) = params.n_mod_serie {
        return (Some(ns), "argumento");
    }
    let inversor = params.inversor.as_deref().or(refs.inversor.as_deref());
    let pnom = params
        .pnom_modulo
        .or_else(|| modulo_info.and_then(|v| v.get("pnom_w")?.as_f64()));
    let pnom_nombre = refs.modulo.as_deref().and_then(lifecycle::pnom_from_name);
    let datos = pnom
        .or(refs.pnom_derivado)
        .or(pnom_nombre)
        .and_then(|p| serie_de_plantilla(paths, params.kwp, p, (inversor, modulo_info), refs));
    match datos {
        Some((ns, por)) => (Some(ns), por),
        None => (None, "plantilla"),
    }
}

/// Step 4: size the array to the target kWp; returns the realised kWp.
/// If module reading fails, the error is propagated with context.
pub(crate) fn ejecutar_paso_dimensionado(
    paths: &Paths,
    nombre_proyecto: &str,
    params: &DimensionadoParametros,
) -> Result<PasoDimensionadoSalida> {
    let refs = referencias_de_variante(paths, nombre_proyecto);
    let modulo_ref = params.modulo.as_deref().or(refs.modulo.as_deref());
    let modulo_info = match modulo_ref {
        Some(m) => {
            let val = pan::leer_modulo(paths, &json!({ "modulo": m }))
                .map_err(|e| Error::Invalid(format!("Error al leer el módulo '{m}': {e}")))?;
            Some(val)
        }
        None => None,
    };

    let (ns_opt, criterio) = resolver_serie_plantilla(paths, params, &refs, modulo_info.as_ref());
    let pnom = params
        .pnom_modulo
        .or_else(|| modulo_info.as_ref().and_then(|v| v.get("pnom_w")?.as_f64()));

    let mut da = Map::new();
    da.insert("proyecto".into(), json!(nombre_proyecto));
    da.insert("variante".into(), json!("VC0"));
    da.insert("kwp".into(), json!(params.kwp));
    if let Some(p) = pnom {
        da.insert("pnom_modulo".into(), json!(p));
    }
    if let Some(ns) = ns_opt {
        da.insert("n_mod_serie".into(), json!(ns));
    }

    let sizing = lifecycle::dimensionar_variante(paths, &Value::Object(da))?;
    let realised_kwp = sizing.get("realised_kwp").and_then(Value::as_f64);
    let avisos = if criterio == "ajuste_local" {
        vec![
            "Sin ventana eléctrica del inversor: Ns ajustado localmente alrededor de la plantilla"
                .to_string(),
        ]
    } else {
        Vec::new()
    };

    Ok(PasoDimensionadoSalida {
        kwp_realizado: realised_kwp,
        criterio,
        avisos,
        ok: true,
        detalle: sizing,
    })
}
