//! Cold and hot string voltage checks and inverter DC/AC ratio verification.

use crate::components;
use crate::components::pan::{leer_modulo, DatosModulo};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::object;
use crate::projects::doctor::{texto, Informe};
use crate::workspace::leer_bytes;
use serde_json::json;
use std::path::Path;

/// Fallback relative temperature coefficient of Voc (1/K, i.e. −0.29 %/K)
/// when the module has none.
const MU_VOC_DEFECTO: f64 = -0.0029;
/// mV → V.
const MILIVOLTIOS_POR_VOLTIO: f64 = 1000.0;

pub(crate) struct ContextoElectrico<'a> {
    pub(crate) paths: &'a Paths,
    pub(crate) ws: &'a Path,
    pub(crate) prj_txt: &'a str,
    pub(crate) var_txt: &'a str,
}

pub(crate) fn omitido(tipo: &str, nombre: &str, motivo: &str) -> String {
    format!("Revisión eléctrica omitida: {tipo} '{nombre}' no legible ({motivo})")
}

fn num(nodo: &object::PvNode, clave: &str) -> Option<f64> {
    nodo.find_deep(clave)?.trim().parse().ok()
}

/// The inverter's OND parsed, or why it could not be read.
fn cargar_inversor(ws: &Path, ond: &str) -> Result<object::PvNode> {
    let ruta =
        components::resolve_component(ws, &components::ConsultaComponente::referencia("OND", ond))?;
    object::parse_first(&texto(&leer_bytes(&ruta)?))
        .ok_or_else(|| Error::Format("archivo ilegible".into()))
}

/// Electrical data needed for the string-voltage and DC/AC checks.
pub(crate) struct DatosElectricos {
    pub(crate) ns: f64,
    pub(crate) tmin: f64,
    pub(crate) tmax: f64,
    pub(crate) voc: f64,
    pub(crate) vmpp: f64,
    pub(crate) pnom: f64,
    pub(crate) vabs: f64,
    pub(crate) vmin: f64,
    pub(crate) vmax: f64,
    pub(crate) pconv: f64,
}

impl DatosElectricos {
    fn reunir(
        prj: &object::PvNode,
        var: &object::PvNode,
        inv: &object::PvNode,
        modulo: &DatosModulo,
    ) -> Option<Self> {
        Some(Self {
            ns: num(var, "NModSerie")?,
            tmin: num(prj, "DesignTempMin")?,
            tmax: num(prj, "TempOperMax")?,
            voc: modulo.voc_v(),
            vmpp: modulo.vmpp_v(),
            pnom: modulo.pnom_w(),
            vabs: num(inv, "VAbsMax")?,
            vmin: num(inv, "VMppMin")?,
            vmax: num(inv, "VMPPMax")?,
            pconv: num(inv, "PNomConv")?,
        })
    }
}

/// Module temperature coefficients for string-voltage checks.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CoeficientesTemperatura {
    pub(crate) mu_voc: f64,
    pub(crate) mu_pnom: Option<f64>,
}

pub(crate) fn comprobar_tensiones(
    d: &DatosElectricos,
    coefs: &CoeficientesTemperatura,
    inf: &mut Informe,
) {
    let tag = crate::aproximaciones::ORIGEN_APROXIMACION;
    let voc_frio = d.ns * d.voc * (1.0 + coefs.mu_voc * (d.tmin - 25.0));
    if voc_frio > d.vabs {
        inf.errores.push(format!(
            "[{tag}] Voc en frío {voc_frio:.0} V supera VAbsMax {:.0} V",
            d.vabs
        ));
    }
    let (mu_vmpp, usando_voc) = match coefs.mu_pnom {
        Some(gamma) => (gamma, false),
        None => (coefs.mu_voc, true),
    };
    let vmpp_calor = d.ns * d.vmpp * (1.0 + mu_vmpp * (d.tmax - 25.0));
    if vmpp_calor < d.vmin {
        let detalle = if usando_voc {
            " (aproximación: Vmpp estimado con μVoc por falta de γ en .PAN)"
        } else {
            ""
        };
        inf.errores.push(format!(
            "[{tag}] Vmpp en calor {vmpp_calor:.0} V por debajo de VMppMin {:.0} V{detalle}",
            d.vmin
        ));
    }
    let vmpp_frio = d.ns * d.vmpp * (1.0 + mu_vmpp * (d.tmin - 25.0));
    if vmpp_frio > d.vmax {
        let detalle = if usando_voc {
            " (aproximación: Vmpp estimado con μVoc por falta de γ en .PAN)"
        } else {
            ""
        };
        inf.errores.push(format!(
            "[{tag}] Vmpp en frío {vmpp_frio:.0} V supera VMPPMax {:.0} V{detalle}",
            d.vmax
        ));
    }
}
fn extraer_coeficientes(
    modulo: &DatosModulo,
    pan: &str,
    inf: &mut Informe,
) -> CoeficientesTemperatura {
    let tag = crate::aproximaciones::ORIGEN_APROXIMACION;
    let mu_voc = modulo.mu_voc_mv_c().map_or_else(
        || {
            inf.avisos.push(format!(
                "[{tag}] μVoc de '{pan}' ausente: usando {MU_VOC_DEFECTO} 1/K"
            ));
            MU_VOC_DEFECTO
        },
        |v| mu_voc_relativo(v, modulo.voc_v()),
    );
    let mu_pnom = modulo.mu_pnom_pct_c().map(|pct| pct / 100.0);
    if mu_pnom.is_none() {
        inf.avisos.push(format!(
            "[{tag}] μPnom/gamma de '{pan}' ausente: Vmpp en temperatura estimado con μVoc"
        ));
    }
    CoeficientesTemperatura { mu_voc, mu_pnom }
}

fn comprobar_relacion_dc_ac(d: &DatosElectricos, var: &object::PvNode, inf: &mut Informe) {
    let tag = crate::aproximaciones::ORIGEN_APROXIMACION;
    let ninv = num(var, "NInverter").unwrap_or(1.0);
    let nstr = num(var, "NStringCh").unwrap_or(1.0);
    let dc_ac = d.pnom * d.ns * nstr * ninv / 1000.0 / (d.pconv * ninv);
    if !(0.8..=1.5).contains(&dc_ac) {
        inf.avisos.push(format!(
            "[{tag}] Relación DC/AC {dc_ac:.2} fuera de 0.8–1.5"
        ));
    }
}

/// Cold/hot string voltage and DC/AC checks against the module and inverter data.
///
/// These are MCP approximations of PVsyst's own sizing checks, so they only run
/// with `PVSYST_MCP_APROXIMACIONES`; otherwise PVsyst reports the window when it
/// simulates (`avisos_pvsyst`).
pub(crate) fn revisar_electrico(ctx: &ContextoElectrico<'_>, inf: &mut Informe) {
    if !ctx.paths.aproximaciones {
        inf.avisos.push(
            "La ventana eléctrica es validada por PVsyst durante la simulación (ver avisos_pvsyst en la salida de simular)".into(),
        );
        return;
    }
    if let Err(motivo) = revisar_aproximado(ctx, inf) {
        let tag = crate::aproximaciones::ORIGEN_APROXIMACION;
        inf.avisos.push(format!("[{tag}] {motivo}"));
    }
}

/// The approximated checks; `Err` explains why they could not run.
fn revisar_aproximado(
    ctx: &ContextoElectrico<'_>,
    inf: &mut Informe,
) -> std::result::Result<(), String> {
    let (Some(prj), Some(var)) = (
        object::parse_first(ctx.prj_txt),
        object::parse_first(ctx.var_txt),
    ) else {
        return Err("Proyecto o variante no legible: revisión eléctrica omitida".into());
    };
    let (Some(pan), Some(ond)) = (var.find_deep("PVModule"), var.find_deep("GInverter")) else {
        return Err("Sin PVModule/GInverter: revisión eléctrica omitida".into());
    };
    let lectura = leer_modulo(ctx.paths, &json!({"modulo": pan}))
        .map_err(|e| omitido("PAN", pan, &e.to_string()))?;
    let modulo = DatosModulo::de_lectura(&lectura)
        .ok_or_else(|| omitido("PAN", pan, "faltan Pnom, Voc o Vmpp"))?;
    let inv = cargar_inversor(ctx.ws, ond).map_err(|m| omitido("OND", ond, &m.to_string()))?;
    let coefs = extraer_coeficientes(&modulo, pan, inf);
    let d = DatosElectricos::reunir(&prj, &var, &inv, &modulo)
        .ok_or("Datos eléctricos incompletos: revisión omitida")?;
    comprobar_tensiones(&d, &coefs, inf);
    comprobar_relacion_dc_ac(&d, &var, inf);
    Ok(())
}

/// μVoc as the datasheet gives it (mV/°C) relative to Voc (1/K), the form
/// [`comprobar_tensiones`] multiplies by `(T - 25)`.
pub(crate) fn mu_voc_relativo(mu_mv_c: f64, voc_v: f64) -> f64 {
    mu_mv_c / MILIVOLTIOS_POR_VOLTIO / voc_v
}

#[cfg(test)]
#[path = "../../../tests/unit/projects/doctor/electrico.rs"]
mod tests;
