use super::caso::{caso, con_cabecera, round1, sin_potencia};
use super::supuestos::{acotado, anios, Rango, DEGRADACION};
use crate::args::opt_str;
use crate::config::Paths;
use crate::error::Result;
use serde_json::{json, Value};

const CAPEX: Rango = Rango {
    clave: "capex_por_kwp",
    min: 0.0,
    max: 100_000.0,
};
const TARIFA: Rango = Rango {
    clave: "tarifa_por_kwh",
    min: 0.0,
    max: 100.0,
};
const OPEX: Rango = Rango {
    clave: "opex_anual_por_kwp",
    min: 0.0,
    max: 10_000.0,
};
const TASA: Rango = Rango {
    clave: "tasa_descuento",
    min: 0.0,
    max: 1.0,
};

pub(crate) struct Economia {
    pub(crate) capex_kwp: f64,
    pub(crate) tarifa: f64,
    pub(crate) opex_kwp: f64,
    pub(crate) tasa: f64,
    pub(crate) anios: u32,
    pub(crate) deg: f64,
    pub(crate) moneda: String,
}

pub(crate) fn parse_economia(args: &Value) -> Result<Economia> {
    Ok(Economia {
        capex_kwp: acotado(args, &CAPEX, None)?,
        tarifa: acotado(args, &TARIFA, None)?,
        opex_kwp: acotado(args, &OPEX, Some(0.0))?,
        tasa: acotado(args, &TASA, Some(0.05))?,
        anios: anios(args, "anios", 25.0)?,
        deg: acotado(args, &DEGRADACION, Some(0.005))?,
        moneda: opt_str(args, &["moneda"]).unwrap_or("USD").to_string(),
    })
}

/// Discounted cash flow over `p.anios`, one pass and O(1) memory.
pub(crate) fn economia_de(e0: f64, kwp: f64, p: &Economia) -> Value {
    let capex = p.capex_kwp * kwp;
    let opex = p.opex_kwp * kwp;
    let (mut vp_energia, mut vp_opex, mut van, mut acumulado) = (0.0, 0.0, -capex, 0.0);
    let (mut energia, mut descuento) = (e0, 1.0);
    let mut payback = None;
    for anio in 1..=p.anios {
        descuento *= 1.0 + p.tasa;
        let flujo = energia * p.tarifa - opex;
        vp_energia += energia / descuento;
        vp_opex += opex / descuento;
        van += flujo / descuento;
        acumulado += flujo;
        if payback.is_none() && acumulado >= capex {
            payback = Some(anio);
        }
        energia *= 1.0 - p.deg;
    }
    // No discounted energy (e.g. zero yield) leaves the LCOE undefined, not infinite.
    let lcoe =
        (vp_energia > 0.0).then(|| ((capex + vp_opex) / vp_energia * 10_000.0).round() / 10_000.0);
    json!({
        "lcoe_por_kwh": lcoe,
        "moneda": p.moneda,
        "capex_total": capex,
        "van": round1(van),
        "payback_anios": payback,
        "supuestos": {
            "capex_por_kwp": p.capex_kwp,
            "tarifa_por_kwh": p.tarifa,
            "opex_anual_por_kwp": p.opex_kwp,
            "tasa_descuento": p.tasa,
            "anios": p.anios,
            "degradacion": p.deg,
        },
    })
}

pub(crate) fn economia(paths: &Paths, args: &Value) -> Result<Value> {
    let c = caso(paths, args)?;
    let kwp = c.kwp.ok_or_else(sin_potencia)?;
    let p = parse_economia(args)?;
    Ok(con_cabecera(&c, economia_de(c.energia_kwh, kwp, &p)))
}
