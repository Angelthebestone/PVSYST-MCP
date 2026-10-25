//! Solar geometry, irradiance modeling, and climate validation.
//! Nothing here runs PVsyst, so calculations cost no trial executions.

pub(crate) mod posicion;
pub(crate) mod produccion;
#[cfg(test)]
#[path = "../../../tests/unit/sites/solar/tests.rs"]
mod tests;

pub(crate) use posicion::geometria_solar;
pub(crate) use produccion::produccion;

use crate::args::req_numero;
use crate::error::{Error, Result};
use crate::sites::nasa::climatology;
use serde_json::Value;

pub(crate) use crate::format::redondear;

/// Klein's representative day of each month (day of year), used for declination and time equation.
pub(crate) const DIA_KLEIN: [u32; 12] = [17, 47, 75, 105, 135, 162, 198, 228, 258, 289, 319, 347];
/// Temperature coefficient of power (per kelvin).
pub(crate) const GAMMA_POTENCIA: f64 = -0.004;
/// Nominal operating cell temperature (°C) at 800 W/m² and 20 °C ambient.
pub(crate) const NOCT: f64 = 45.0;
/// Overall system losses applied to the modelled AC energy (fraction).
pub(crate) const PERDIDAS_SISTEMA: f64 = 0.14;
/// Ground albedo used for the reflected component.
pub(crate) const ALBEDO: f64 = 0.2;

/// Physical limits for daily global/diffuse horizontal irradiance (kWh/m²/day).
pub(crate) const MIN_IRRADIANCIA_DIARIA_KWH: f64 = 0.0;
pub(crate) const MAX_IRRADIANCIA_DIARIA_KWH: f64 = 15.0;

/// Physical limits for ambient temperature (°C).
pub(crate) const TEMP_MINIMA_C: f64 = -100.0;
pub(crate) const TEMP_MAXIMA_C: f64 = 70.0;

/// Geographic and timezone limits.
pub(crate) const LATITUD_MINIMA: f64 = -90.0;
pub(crate) const LATITUD_MAXIMA: f64 = 90.0;
pub(crate) const LONGITUD_MINIMA: f64 = -180.0;
pub(crate) const LONGITUD_MAXIMA: f64 = 180.0;
pub(crate) const ZONA_HORARIA_MINIMA: f64 = -12.0;
pub(crate) const ZONA_HORARIA_MAXIMA: f64 = 14.0;

/// Orientation limits (degrees).
pub(crate) const INCLINACION_MINIMA: f64 = 0.0;
pub(crate) const INCLINACION_MAXIMA: f64 = 90.0;
pub(crate) const AZIMUT_MINIMO: f64 = -180.0;
pub(crate) const AZIMUT_MAXIMO: f64 = 180.0;

/// Site position and the clock offset from UTC used for solar time.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Sitio {
    pub latitud: f64,
    pub longitud: f64,
    pub zona_horaria: f64,
}

/// Plane-of-array orientation in PVsyst convention: tilt from horizontal (degrees),
/// azimuth where 0 = south and east is negative (degrees).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Orientacion {
    pub inclinacion: f64,
    pub azimut: f64,
}

/// Monthly climatology: daily global and diffuse horizontal (kWh/m²/day) and ambient temperature (°C).
#[derive(Debug, Clone)]
pub(crate) struct Clima {
    pub ghi: Vec<f64>,
    pub dif: Vec<f64>,
    pub temp: Vec<f64>,
}

/// Monthly energy (kWh) and specific yield (kWh/kWp per year) for one orientation.
#[derive(Debug, Clone)]
pub(crate) struct Produccion {
    pub mensual: [f64; 12],
    pub anual: f64,
    pub especifica: f64,
}

impl Clima {
    /// Validates month count (12), finiteness and physical bounds for irradiance and temperature.
    pub(crate) fn nuevo(ghi: Vec<f64>, dif: Vec<f64>, temp: Vec<f64>) -> Result<Self> {
        const MESES: usize = 12;
        if ghi.len() != MESES || dif.len() != MESES || temp.len() != MESES {
            return Err(Error::Invalid(
                "La climatología debe tener 12 valores mensuales por serie".into(),
            ));
        }
        for m in 0..MESES {
            let g = ghi[m];
            let d = dif[m];
            let t = temp[m];
            let mes_humano = m + 1;
            if !g.is_finite() || !d.is_finite() || !t.is_finite() {
                return Err(Error::Invalid(format!(
                    "Mes {mes_humano}: los valores climáticos deben ser números finitos (ghi={g}, dif={d}, temp={t})"
                )));
            }
            if !(MIN_IRRADIANCIA_DIARIA_KWH..=MAX_IRRADIANCIA_DIARIA_KWH).contains(&g) {
                return Err(Error::Invalid(format!(
                    "Mes {mes_humano}: GHI {g} fuera de rango [{MIN_IRRADIANCIA_DIARIA_KWH}..={MAX_IRRADIANCIA_DIARIA_KWH}] kWh/m²/día"
                )));
            }
            if !(MIN_IRRADIANCIA_DIARIA_KWH..=MAX_IRRADIANCIA_DIARIA_KWH).contains(&d) {
                return Err(Error::Invalid(format!(
                    "Mes {mes_humano}: DIF {d} fuera de rango [{MIN_IRRADIANCIA_DIARIA_KWH}..={MAX_IRRADIANCIA_DIARIA_KWH}] kWh/m²/día"
                )));
            }
            if d > g {
                return Err(Error::Invalid(format!(
                    "Mes {mes_humano}: radiación difusa ({d}) supera la global ({g})"
                )));
            }
            if !(TEMP_MINIMA_C..=TEMP_MAXIMA_C).contains(&t) {
                return Err(Error::Invalid(format!(
                    "Mes {mes_humano}: temperatura {t} °C fuera de rango [{TEMP_MINIMA_C}..={TEMP_MAXIMA_C}]"
                )));
            }
        }
        Ok(Self { ghi, dif, temp })
    }

    /// Monthly climatology of the NASA SSE grid cell containing the coordinates.
    pub(crate) fn de_nasa(latitud: f64, longitud: f64) -> Result<Self> {
        let c = climatology(latitud, longitud)?;
        Self::nuevo(c.ghi_daily, c.diff_daily, c.temp)
    }
}

/// Numeric argument given as a JSON number or a numeric string; `None` when absent or not numeric.
pub(crate) fn numero_argumento(args: &Value, clave: &str) -> Option<f64> {
    let valor = args.get(clave)?;
    valor
        .as_f64()
        .or_else(|| valor.as_str()?.trim().parse().ok())
}

pub(crate) const PISTA_NUMERO_SOLAR: &str = "número para el cálculo solar";

/// Site from `latitud`, `longitud` and optional `timezone` (default: round(longitud/15)).
pub(crate) fn sitio_de(args: &Value) -> Result<Sitio> {
    let latitud = req_numero(args, "latitud", PISTA_NUMERO_SOLAR)?;
    let longitud = req_numero(args, "longitud", PISTA_NUMERO_SOLAR)?;
    if !(LATITUD_MINIMA..=LATITUD_MAXIMA).contains(&latitud)
        || !(LONGITUD_MINIMA..=LONGITUD_MAXIMA).contains(&longitud)
    {
        return Err(Error::Invalid(format!(
            "Coordenadas fuera de rango: latitud {LATITUD_MINIMA}..={LATITUD_MAXIMA}, longitud {LONGITUD_MINIMA}..={LONGITUD_MAXIMA}"
        )));
    }
    let zona_horaria = match args.get("timezone") {
        Some(v) => {
            let n = v
                .as_f64()
                .or_else(|| v.as_str().and_then(|s| s.trim().parse().ok()))
                .ok_or_else(|| {
                    Error::Invalid("Zona horaria inválida: debe ser un número finito".into())
                })?;
            if !n.is_finite() || !(ZONA_HORARIA_MINIMA..=ZONA_HORARIA_MAXIMA).contains(&n) {
                return Err(Error::Invalid(format!(
                    "Zona horaria '{n}' fuera de rango: debe ser finita y estar entre {ZONA_HORARIA_MINIMA} y {ZONA_HORARIA_MAXIMA}"
                )));
            }
            n
        }
        None => (longitud / 15.0).round(),
    };
    Ok(Sitio {
        latitud,
        longitud,
        zona_horaria,
    })
}

/// Orientation from `tilt` and `azimut` when at least one is given. A missing value
/// defaults to tilt = |latitud| and azimut = south (north in the southern hemisphere).
pub(crate) fn orientacion_de(args: &Value, latitud: f64) -> Result<Option<Orientacion>> {
    let tilt = numero_argumento(args, "tilt");
    let azimut = numero_argumento(args, "azimut");
    if tilt.is_none() && azimut.is_none() {
        return Ok(None);
    }
    let inclinacion = tilt.unwrap_or_else(|| latitud.abs());
    let azimut = azimut.unwrap_or(if latitud >= 0.0 { 0.0 } else { 180.0 });
    if !(INCLINACION_MINIMA..=INCLINACION_MAXIMA).contains(&inclinacion) {
        return Err(Error::Invalid(format!(
            "'tilt' debe estar entre {INCLINACION_MINIMA} y {INCLINACION_MAXIMA} grados"
        )));
    }
    if !(AZIMUT_MINIMO..=AZIMUT_MAXIMO).contains(&azimut) {
        return Err(Error::Invalid(format!(
            "'azimut' debe estar entre {AZIMUT_MINIMO} y {AZIMUT_MAXIMO} grados (0 = sur, este negativo)"
        )));
    }
    Ok(Some(Orientacion {
        inclinacion,
        azimut,
    }))
}
