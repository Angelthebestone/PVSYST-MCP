//! PVsyst economic-evaluation parameters and their section changes.

use crate::args::{missing, opt_f64, opt_u32, rango_opcional};
use crate::error::{Error, Result};
use serde_json::{json, Value};

const MAX_VIDA_ANIOS: u32 = 100;
const MAX_TASA: f64 = 1.0;
const MAX_TARIFA_KWH: f64 = 100.0;
const LONGITUD_CODIGO_MONEDA: usize = 3;
const DECIMALES_TASA: usize = 4;
const DECIMALES_TARIFA: usize = 5;

/// Raw economic arguments.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct EntradaEconomicos<'a> {
    pub(crate) moneda: Option<&'a str>,
    pub(crate) vida_anios: Option<u32>,
    pub(crate) tasa_descuento: Option<f64>,
    pub(crate) inflacion: Option<f64>,
    pub(crate) tarifa_venta_kwh: Option<f64>,
}

impl<'a> EntradaEconomicos<'a> {
    pub(crate) fn de_args(args: &'a Value) -> Result<Self> {
        Ok(Self {
            moneda: args
                .get("moneda")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty()),
            vida_anios: opt_u32(args, "vida_anios")?,
            tasa_descuento: opt_f64(args, "tasa_descuento")?,
            inflacion: opt_f64(args, "inflacion")?,
            tarifa_venta_kwh: opt_f64(args, "tarifa_venta_kwh")?,
        })
    }
}

fn validar_moneda(moneda: &str) -> Result<()> {
    if moneda.len() == LONGITUD_CODIGO_MONEDA && moneda.chars().all(|c| c.is_ascii_uppercase()) {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "'moneda' debe ser un código de 3 letras mayúsculas: '{moneda}'"
    )))
}

/// Validated economic parameters.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ParametrosEconomicos {
    moneda: Option<String>,
    vida_anios: Option<u32>,
    tasa_descuento: Option<f64>,
    inflacion: Option<f64>,
    tarifa_venta_kwh: Option<f64>,
}

impl ParametrosEconomicos {
    pub(crate) fn nuevo(entrada: EntradaEconomicos<'_>) -> Result<Self> {
        if entrada.moneda.is_none()
            && entrada.vida_anios.is_none()
            && entrada.tasa_descuento.is_none()
            && entrada.inflacion.is_none()
            && entrada.tarifa_venta_kwh.is_none()
        {
            return Err(missing(
                "moneda",
                "o vida_anios, tasa_descuento, inflacion, tarifa_venta_kwh",
            ));
        }
        if let Some(moneda) = entrada.moneda {
            validar_moneda(moneda)?;
        }
        if let Some(vida) = entrada.vida_anios {
            if !(1..=MAX_VIDA_ANIOS).contains(&vida) {
                return Err(Error::Invalid(format!(
                    "'vida_anios' debe estar entre 1 y {MAX_VIDA_ANIOS}: {vida}"
                )));
            }
        }
        rango_opcional(entrada.tasa_descuento, MAX_TASA, "tasa_descuento")?;
        rango_opcional(entrada.inflacion, MAX_TASA, "inflacion")?;
        rango_opcional(entrada.tarifa_venta_kwh, MAX_TARIFA_KWH, "tarifa_venta_kwh")?;
        Ok(Self {
            moneda: entrada.moneda.map(str::to_string),
            vida_anios: entrada.vida_anios,
            tasa_descuento: entrada.tasa_descuento,
            inflacion: entrada.inflacion,
            tarifa_venta_kwh: entrada.tarifa_venta_kwh,
        })
    }

    pub(crate) fn tiene_tarifa_venta(&self) -> bool {
        self.tarifa_venta_kwh.is_some()
    }

    pub(crate) fn como_json(&self) -> Value {
        json!({
            "moneda": self.moneda,
            "vida_anios": self.vida_anios,
            "tasa_descuento": self.tasa_descuento,
            "inflacion": self.inflacion,
            "tarifa_venta_kwh": self.tarifa_venta_kwh,
        })
    }

    /// `EconomicEval` section changes.
    pub(crate) fn cambios(&self) -> Vec<(String, Option<String>)> {
        let moneda = self
            .moneda
            .as_ref()
            .map(|valor| ("Currency".to_string(), Some(valor.clone())));
        let vida = self
            .vida_anios
            .map(|valor| ("ProjectLife".to_string(), Some(valor.to_string())));
        let descuento = self.tasa_descuento.map(|valor| {
            (
                "DiscountRate".to_string(),
                Some(format!("{:.*}", DECIMALES_TASA, valor)),
            )
        });
        let inflacion = self.inflacion.map(|valor| {
            (
                "InflationRate".to_string(),
                Some(format!("{:.*}", DECIMALES_TASA, valor)),
            )
        });
        let tarifa = self.tarifa_venta_kwh.map(|valor| {
            (
                "SummerPeakSaleTariff".to_string(),
                Some(format!("{:.*}", DECIMALES_TARIFA, valor)),
            )
        });
        moneda
            .into_iter()
            .chain(vida)
            .chain(descuento)
            .chain(inflacion)
            .chain(tarifa)
            .collect()
    }
}

#[cfg(test)]
#[path = "../../tests/unit/evaluacion_economica/dominio.rs"]
mod tests;
