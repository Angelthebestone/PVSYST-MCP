//! Inverter-window and template-based series resolution for easy-mode sizing.

use super::dimensionado::{ajustar_serie, dimensionar_serie, WATTS_POR_KILOWATT};
use super::parametros::DimensionadoParametros;
use crate::components::pan::DatosModulo;
use crate::components::resolve_component;
use crate::config::Paths;
use crate::format::object;
use crate::projects::lifecycle;

/// Electrical data used to place a module inside the inverter window.
struct ContextoVentana<'a> {
    inversor: Option<&'a str>,
    kwp: f64,
    pmod_w: f64,
    modulo: &'a DatosModulo,
}

/// Inputs for the template-based series search.
struct ContextoSerie<'a> {
    kwp: f64,
    pmod_w: f64,
    inversor: Option<&'a str>,
    modulo: Option<&'a DatosModulo>,
    refs: &'a Referencias,
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

/// Inputs for resolving the series count of the created variant.
pub(crate) struct ResolucionSerie<'a> {
    pub(crate) params: &'a DimensionadoParametros,
    pub(crate) refs: &'a Referencias,
    pub(crate) modulo_info: Option<&'a DatosModulo>,
}

pub(crate) fn referencias_de_variante(paths: &Paths, nombre: &str, variante: &str) -> Referencias {
    let ruta = paths
        .workspace
        .join("Projects")
        .join(format!("{nombre}.{variante}"));
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

/// Layout from the inverter's DC window (`VMppMin`, `VAbsMax` of the OND file)
/// and the module's STC values. `None` when any datum is missing.
fn disposicion_electrica(paths: &Paths, ctx: &ContextoVentana<'_>) -> Option<(u32, u32)> {
    let inversor = ctx.inversor?;
    let (voc, vmpp) = (ctx.modulo.voc_v(), ctx.modulo.vmpp_v());
    let consulta = crate::components::ConsultaComponente::referencia("OND", inversor);
    let ond = resolve_component(&paths.workspace, &consulta).ok()?;
    let bytes = crate::components::leer_bytes_componente(&ond).ok()?;
    let texto = String::from_utf8_lossy(&bytes).into_owned();
    let root = object::parse_first(&texto)?;
    let vmin = root.find_deep("VMppMin")?.trim().parse::<f64>().ok()?;
    let vabs = root.find_deep("VAbsMax")?.trim().parse::<f64>().ok()?;
    dimensionar_serie(ctx.kwp, ctx.pmod_w, voc, vmpp, vmin, vabs)
}

/// Series count and the criterion that chose it: the inverter's DC window when
/// the datasheets allow it, else a local search around the template's series.
fn serie_de_plantilla(paths: &Paths, ctx: &ContextoSerie<'_>) -> Option<(u32, &'static str)> {
    let ventana = ctx.modulo.and_then(|modulo| {
        disposicion_electrica(
            paths,
            &ContextoVentana {
                inversor: ctx.inversor,
                kwp: ctx.kwp,
                pmod_w: ctx.pmod_w,
                modulo,
            },
        )
    });
    if let Some((ns, _)) = ventana {
        return Some((ns, "ventana_electrica"));
    }
    ctx.refs
        .ns_plantilla
        .and_then(|ns0| ajustar_serie(ctx.kwp, ctx.pmod_w, ns0))
        .map(|(ns, _, _)| (ns, "ajuste_local"))
}

/// Series count of the created variant and the criterion that chose it.
pub(crate) fn resolver_serie_plantilla(
    paths: &Paths,
    resolucion: &ResolucionSerie<'_>,
) -> (Option<u32>, &'static str) {
    let params = resolucion.params;
    if let Some(ns) = params.n_mod_serie {
        return (Some(ns), "argumento");
    }
    let inversor = params
        .inversor
        .as_deref()
        .or(resolucion.refs.inversor.as_deref());
    let pnom = params
        .pnom_modulo
        .or_else(|| resolucion.modulo_info.map(DatosModulo::pnom_w));
    let pnom_nombre = resolucion
        .refs
        .modulo
        .as_deref()
        .and_then(lifecycle::pnom_from_name);
    let datos = pnom
        .or(resolucion.refs.pnom_derivado)
        .or(pnom_nombre)
        .and_then(|pmod_w| {
            serie_de_plantilla(
                paths,
                &ContextoSerie {
                    kwp: params.kwp,
                    pmod_w,
                    inversor,
                    modulo: resolucion.modulo_info,
                    refs: resolucion.refs,
                },
            )
        });
    match datos {
        Some((ns, por)) => (Some(ns), por),
        None => (None, "plantilla"),
    }
}
