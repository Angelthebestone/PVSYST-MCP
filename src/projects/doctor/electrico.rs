//! Cold and hot string voltage checks and inverter DC/AC ratio verification.

use crate::components;
use crate::components::pan::leer_modulo;
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::object;
use crate::projects::doctor::{leer_bytes, texto, Informe};
use serde_json::{json, Value};
use std::path::Path;

/// Fallback temperature coefficient of Voc (V/K) when the module has none.
const MU_VOC_DEFECTO: f64 = -0.0029;

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
        modulo: &Value,
    ) -> Option<Self> {
        Some(Self {
            ns: num(var, "NModSerie")?,
            tmin: num(prj, "DesignTempMin")?,
            tmax: num(prj, "TempOperMax")?,
            voc: modulo["voc_v"].as_f64()?,
            vmpp: modulo["vmpp_v"].as_f64()?,
            pnom: modulo["pnom_w"].as_f64()?,
            vabs: num(inv, "VAbsMax")?,
            vmin: num(inv, "VMppMin")?,
            vmax: num(inv, "VMPPMax")?,
            pconv: num(inv, "PNomConv")?,
        })
    }
}

pub(crate) fn comprobar_tensiones(d: &DatosElectricos, mu_voc: f64, inf: &mut Informe) {
    let voc_frio = d.ns * d.voc * (1.0 + mu_voc * (d.tmin - 25.0));
    if voc_frio > d.vabs {
        inf.errores.push(format!(
            "Voc en frío {voc_frio:.0} V supera VAbsMax {:.0} V",
            d.vabs
        ));
    }
    let vmpp_calor = d.ns * d.vmpp * (1.0 + mu_voc * (d.tmax - 25.0));
    if vmpp_calor < d.vmin {
        inf.errores.push(format!(
            "Vmpp en calor {vmpp_calor:.0} V por debajo de VMppMin {:.0} V",
            d.vmin
        ));
    }
    let vmpp_frio = d.ns * d.vmpp * (1.0 + mu_voc * (d.tmin - 25.0));
    if vmpp_frio > d.vmax {
        inf.errores.push(format!(
            "Vmpp en frío {vmpp_frio:.0} V supera VMPPMax {:.0} V",
            d.vmax
        ));
    }
}

/// Cold/hot string voltage and DC/AC checks against the module and inverter data.
pub(crate) fn revisar_electrico(ctx: &ContextoElectrico<'_>, inf: &mut Informe) {
    let (Some(prj), Some(var)) = (
        object::parse_first(ctx.prj_txt),
        object::parse_first(ctx.var_txt),
    ) else {
        inf.avisos
            .push("Proyecto o variante no legible: revisión eléctrica omitida".into());
        return;
    };
    let (Some(pan), Some(ond)) = (var.find_deep("PVModule"), var.find_deep("GInverter")) else {
        inf.avisos
            .push("Sin PVModule/GInverter: revisión eléctrica omitida".into());
        return;
    };
    let modulo = match leer_modulo(ctx.paths, &json!({"modulo": pan})) {
        Ok(m) => m,
        Err(e) => return inf.avisos.push(omitido("PAN", pan, &e.to_string())),
    };
    let inv = match cargar_inversor(ctx.ws, ond) {
        Ok(inv) => inv,
        Err(motivo) => return inf.avisos.push(omitido("OND", ond, &motivo.to_string())),
    };
    let mu = modulo["mu_voc_mv_c"].as_f64().map_or_else(
        || {
            inf.avisos.push(format!(
                "μVoc de '{pan}' ausente: usando {MU_VOC_DEFECTO} V/K"
            ));
            MU_VOC_DEFECTO
        },
        |v| v / 1000.0,
    );
    let Some(d) = DatosElectricos::reunir(&prj, &var, &inv, &modulo) else {
        inf.avisos
            .push("Datos eléctricos incompletos: revisión omitida".into());
        return;
    };
    comprobar_tensiones(&d, mu, inf);
    let ninv = num(&var, "NInverter").unwrap_or(1.0);
    let nstr = num(&var, "NStringCh").unwrap_or(1.0);
    let dc_ac = d.pnom * d.ns * nstr * ninv / 1000.0 / (d.pconv * ninv);
    if !(0.8..=1.5).contains(&dc_ac) {
        inf.avisos
            .push(format!("Relación DC/AC {dc_ac:.2} fuera de 0.8–1.5"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_report_error_when_string_voltage_exceeds_limits() {
        let mut inf = Informe::default();
        let d = DatosElectricos {
            ns: 30.0,
            tmin: -10.0,
            tmax: 60.0,
            voc: 50.0,
            vmpp: 41.0,
            pnom: 300.0,
            vabs: 1000.0,
            vmin: 500.0,
            vmax: 900.0,
            pconv: 10000.0,
        };
        comprobar_tensiones(&d, -0.0029, &mut inf);
        assert!(inf.errores.iter().any(|e| e.contains("Voc en frío")));
    }

    #[test]
    fn should_pass_when_string_voltage_is_within_limits() {
        let mut inf = Informe::default();
        let d = DatosElectricos {
            ns: 10.0,
            tmin: -10.0,
            tmax: 60.0,
            voc: 40.0,
            vmpp: 33.0,
            pnom: 300.0,
            vabs: 1000.0,
            vmin: 250.0,
            vmax: 900.0,
            pconv: 10000.0,
        };
        comprobar_tensiones(&d, -0.0029, &mut inf);
        assert!(inf.errores.is_empty(), "{:?}", inf.errores);
    }

    #[test]
    fn should_warn_when_electrical_component_cannot_be_read() {
        let msg = omitido("OND", "Generic_7.5kW.OND", "archivo ilegible");
        assert!(msg.starts_with("Revisión eléctrica omitida: OND 'Generic_7.5kW.OND'"));
    }
}
