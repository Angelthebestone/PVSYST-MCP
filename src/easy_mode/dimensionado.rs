//! Step 4: Array sizing and inverter electrical window calculations.

use super::baterias::aplicar_baterias_al_resultado;
use super::parametros::{DisenoParametros, TipoDiseno};
use super::pasos::PasoDimensionadoSalida;
use super::serie_plantilla::{
    referencias_de_variante, resolver_serie_plantilla, Referencias, ResolucionSerie,
};
use crate::aproximaciones::{self, Aproximacion};
use crate::components::pan::{self, DatosModulo};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::projects::lifecycle;
use serde_json::{json, Map, Value};

pub(crate) const APROXIMACION_DISENO_AUTOMATICO: Aproximacion = Aproximacion {
    funcion: "disenar_sistema dimensionado automatico via aproximacion",
    alternativa:
        "disenar_sistema sin 'via' exige pasar n_mod_serie y n_strings; el pre-dimensionado de PVsyst (ventana Sistema) solo lo usa 'proyecto dimensionar'",
};

const FACTOR_SEGURIDAD_VOC: f64 = 1.15;
pub(crate) const WATTS_POR_KILOWATT: f64 = 1000.0;
const RANGO_SERIE_OFFSET: u32 = 2;
/// Most modules in one string the search tries: a bad voltage in a database file (a tiny `Voc`, a huge
/// `VAbsMax`) must not turn the search into billions of candidates. Same ceiling as the schema's `n_modulos_serie`.
const MAX_MODULOS_EN_SERIE: u32 = 1000;

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
    let finitos = [
        v.kwp,
        v.pmod_w,
        v.voc_v,
        v.vmpp_v,
        v.vmpp_min_v,
        v.vabs_max_v,
    ];
    if finitos.iter().any(|x| !x.is_finite())
        || [v.pmod_w, v.voc_v, v.vmpp_v].iter().any(|x| *x <= 0.0)
    {
        return None;
    }
    let ns_min = (v.vmpp_min_v / v.vmpp_v).ceil().max(1.0) as u32;
    let ns_max = ((v.vabs_max_v / (FACTOR_SEGURIDAD_VOC * v.voc_v)).floor() as u32)
        .min(MAX_MODULOS_EN_SERIE);
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

/// Module data read for sizing plus the substitute module name when the template's own
/// module is not in any database.
struct ModuloResuelto {
    datos: Option<DatosModulo>,
    sustituto: Option<String>,
}

fn leer_modulo(
    paths: &Paths,
    params: &DisenoParametros,
    refs: &Referencias,
) -> Result<ModuloResuelto> {
    let explicito = params.dimensionado.modulo.is_some();
    let sin_modulo = ModuloResuelto {
        datos: None,
        sustituto: None,
    };
    let Some(m) = params
        .dimensionado
        .modulo
        .as_deref()
        .or(refs.modulo.as_deref())
    else {
        return Ok(sin_modulo);
    };
    match pan::leer_modulo(paths, &json!({ "modulo": m })) {
        Ok(val) => Ok(ModuloResuelto {
            datos: DatosModulo::de_lectura(&val),
            sustituto: None,
        }),
        Err(Error::NotFound(_)) if !explicito => sustituir_modulo(paths, params, m),
        Err(e) => Err(Error::Invalid(format!(
            "Error al leer el módulo '{m}': {e}"
        ))),
    }
}

/// The template's module is missing: size with the first module available in the
/// workspace or DataRO (stand-alone designs keep their own sizing path).
fn sustituir_modulo(
    paths: &Paths,
    params: &DisenoParametros,
    faltante: &str,
) -> Result<ModuloResuelto> {
    if params.tipo_sistema == TipoDiseno::Aislado {
        return Ok(ModuloResuelto {
            datos: None,
            sustituto: None,
        });
    }
    let ws = crate::args::workspace_of(paths, &Value::Null);
    let nombre = pan::primer_modulo_disponible(&ws).ok_or_else(|| {
        Error::Invalid(format!(
            "El módulo '{faltante}' de la plantilla no existe y no hay ningún .PAN en el workspace ni en DataRO: pasa 'modulo'"
        ))
    })?;
    let val = pan::leer_modulo(paths, &json!({ "modulo": nombre }))?;
    Ok(ModuloResuelto {
        datos: DatosModulo::de_lectura(&val),
        sustituto: Some(nombre),
    })
}

struct ResolucionDiseno {
    ns_opt: Option<u32>,
    nc_opt: Option<u32>,
    criterio: &'static str,
    avisos: Vec<String>,
}

fn determinar_disposicion(paths: &Paths, res: &ResolucionSerie<'_>) -> Result<ResolucionDiseno> {
    if res.params.n_mod_serie.is_some() && res.params.n_strings.is_some() {
        return Ok(ResolucionDiseno {
            ns_opt: res.params.n_mod_serie,
            nc_opt: res.params.n_strings,
            criterio: "manual",
            avisos: Vec::new(),
        });
    }
    if !res.params.via_aproximacion {
        return Ok(ResolucionDiseno {
            ns_opt: None,
            nc_opt: None,
            criterio: "pvsyst",
            avisos: Vec::new(),
        });
    }
    aproximaciones::exigir(paths, &APROXIMACION_DISENO_AUTOMATICO)?;
    let (ns, crit) = resolver_serie_plantilla(paths, res);
    let avisos = if crit == "ajuste_local" {
        vec![
            "Sin ventana eléctrica del inversor: Ns ajustado localmente alrededor de la plantilla"
                .to_string(),
        ]
    } else {
        Vec::new()
    };
    Ok(ResolucionDiseno {
        ns_opt: ns,
        nc_opt: None,
        criterio: crit,
        avisos,
    })
}

/// Step 4: size the array to the target kWp; returns the realised kWp.
/// If module reading fails, the error is propagated with context.
pub(crate) fn ejecutar_paso_dimensionado(
    paths: &Paths,
    params: &DisenoParametros,
) -> Result<PasoDimensionadoSalida> {
    let refs = referencias_de_variante(paths, &params.proyecto.nombre, &params.variante);
    let modulo = leer_modulo(paths, params, &refs)?;
    let resolucion = ResolucionSerie {
        params: &params.dimensionado,
        refs: &refs,
        modulo_info: modulo.datos.as_ref(),
    };
    let mut disp = determinar_disposicion(paths, &resolucion)?;
    if let Some(nombre) = &modulo.sustituto {
        disp.avisos.push(format!(
            "El módulo de la plantilla no existe: se usa '{nombre}' (primer .PAN disponible en el workspace o DataRO)"
        ));
    }
    let da = argumentos_dimensionado(params, &modulo, &disp)?;

    let sizing = lifecycle::dimensionar_variante(paths, &Value::Object(da))?;
    let realised_kwp = sizing.get("realised_kwp").and_then(Value::as_f64);
    let detalle = aplicar_baterias_al_resultado(paths, params, sizing)?;

    Ok(PasoDimensionadoSalida {
        kwp_realizado: realised_kwp,
        criterio: disp.criterio,
        avisos: disp.avisos,
        ok: true,
        detalle,
    })
}

/// Arguments of the `proyecto dimensionar` call that applies the chosen layout.
fn argumentos_dimensionado(
    params: &DisenoParametros,
    modulo: &ModuloResuelto,
    disp: &ResolucionDiseno,
) -> Result<Map<String, Value>> {
    let pnom = params
        .dimensionado
        .pnom_modulo
        .or_else(|| modulo.datos.as_ref().map(DatosModulo::pnom_w));
    let mut da = Map::new();
    da.insert("proyecto".into(), json!(params.proyecto.nombre));
    da.insert("variante".into(), json!(params.variante));
    da.insert("kwp".into(), json!(params.dimensionado.kwp));
    if let Some(nombre) = &modulo.sustituto {
        da.insert("modulo".into(), json!(nombre));
    }
    if let Some(p) = pnom {
        da.insert("pnom_modulo".into(), json!(p));
    }
    if let Some(ns) = disp.ns_opt {
        da.insert("n_mod_serie".into(), json!(ns));
    }
    if let Some(nc) = disp.nc_opt {
        da.insert("n_strings".into(), json!(nc));
    }
    if params.dimensionado.via_aproximacion {
        da.insert(
            aproximaciones::ARGUMENTO_VIA.into(),
            json!(aproximaciones::VIA_APROXIMACION),
        );
        return Ok(da);
    }
    if disp.nc_opt.is_none() && params.tipo_sistema == TipoDiseno::Aislado {
        return Err(Error::Invalid(
            "En sistemas aislados pasa n_mod_serie y n_strings: el pre-dimensionado de PVsyst automatizado es el de la ventana de sistemas conectados a red".into(),
        ));
    }
    Ok(da)
}
