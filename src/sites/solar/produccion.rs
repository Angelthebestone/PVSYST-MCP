//! Irradiance transposition and energy production model.
//! Hourly global irradiance is split from NASA monthly means with the
//! Collares-Pereira–Rabl correlation and transposed to POA with an isotropic sky.

use crate::sites::nasa::DAYS;
use std::f64::consts::PI;

use super::posicion::{declinacion, hora_solar, posicion};
use super::{
    Clima, Orientacion, Produccion, Sitio, ALBEDO, DIA_KLEIN, GAMMA_POTENCIA, NOCT,
    PERDIDAS_SISTEMA,
};

/// Geometry of one hour of a representative day.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Hora {
    pub(crate) cz: f64,
    pub(crate) ci: f64,
    pub(crate) rt: f64,
}

/// Hourly geometry (cosines of zenith and incidence, Collares-Pereira–Rabl weight) of day `n`.
pub(crate) fn geometria_dia(sitio: &Sitio, n: f64, o: Orientacion) -> [Hora; 24] {
    let lat = sitio.latitud.to_radians();
    let dec = declinacion(n);
    let ws = (-lat.tan() * dec.tan()).clamp(-1.0, 1.0).acos();
    let beta = o.inclinacion.to_radians();
    let gamma = o.azimut.to_radians();
    let mut horas = [Hora::default(); 24];
    for (h, celda) in horas.iter_mut().enumerate() {
        let w = (15.0 * (hora_solar(sitio, n, h as f64 + 0.5) - 12.0)).to_radians();
        let (cz, gs) = posicion(lat, dec, w);
        if cz <= 0.0 {
            continue;
        }
        let sz = (1.0 - cz * cz).max(0.0).sqrt();
        let ci = cz * beta.cos() + sz * beta.sin() * (gs - gamma).cos();
        let rt = if w.abs() < ws {
            PI / 24.0 * (w.cos() - ws.cos()) / (ws.sin() - ws * ws.cos())
        } else {
            0.0
        };
        *celda = Hora {
            cz,
            ci,
            rt: rt.max(0.0),
        };
    }
    horas
}

/// Plane-of-array irradiance (W/m²) for each hour of day `n`, from the monthly daily
/// global and diffuse horizontal means (kWh/m²/day).
/// Parameters for computing plane-of-array hourly irradiance.
pub(crate) struct ParametrosPoa<'a> {
    pub(crate) sitio: &'a Sitio,
    pub(crate) dia_anio: f64,
    pub(crate) ghi_dia: f64,
    pub(crate) dif_dia: f64,
    pub(crate) orientacion: Orientacion,
}

/// Plane-of-array irradiance (W/m²) for each hour of day `n`, from the monthly daily
/// global and diffuse horizontal means (kWh/m²/day).
pub(crate) fn poa_horas(p: &ParametrosPoa<'_>) -> [f64; 24] {
    let geo = geometria_dia(p.sitio, p.dia_anio, p.orientacion);
    let suma: f64 = geo.iter().map(|g| g.rt).sum();
    let mut poa = [0.0; 24];
    if suma <= 0.0 || p.ghi_dia <= 0.0 {
        return poa;
    }
    let difusa = (p.dif_dia / p.ghi_dia).clamp(0.0, 1.0);
    let beta = p.orientacion.inclinacion.to_radians();
    let vista_cielo = (1.0 + beta.cos()) / 2.0;
    let suelo = ALBEDO * (1.0 - beta.cos()) / 2.0;
    for (val, g) in poa.iter_mut().zip(geo.iter()) {
        if g.rt <= 0.0 || g.cz <= 0.0 {
            continue;
        }
        let ih = p.ghi_dia * g.rt / suma;
        let rb = if g.cz > 0.05 {
            g.ci.max(0.0) / g.cz
        } else {
            0.0
        };
        let ib = ih * (1.0 - difusa);
        let id = ih * difusa;
        *val = 1000.0 * (ib * rb + id * vista_cielo + ih * suelo);
    }
    poa
}

/// AC energy (kWh) of one hour at plane irradiance `g` (W/m²) and ambient `t_amb` (°C).
pub(crate) fn energia_hora(g: f64, t_amb: f64, kwp: f64) -> f64 {
    let tc = t_amb + g * (NOCT - 20.0) / 800.0;
    g / 1000.0 * kwp * (1.0 + GAMMA_POTENCIA * (tc - 25.0)) * (1.0 - PERDIDAS_SISTEMA)
}

/// Monthly energy and specific yield of a `kwp` plant in orientation `o`.
pub(crate) fn produccion(sitio: &Sitio, clima: &Clima, o: Orientacion, kwp: f64) -> Produccion {
    let mut mensual = [0.0; 12];
    for (m, energia) in mensual.iter_mut().enumerate() {
        let poa = poa_horas(&ParametrosPoa {
            sitio,
            dia_anio: f64::from(DIA_KLEIN[m]),
            ghi_dia: clima.ghi[m],
            dif_dia: clima.dif[m],
            orientacion: o,
        });
        let por_hora: f64 = poa
            .iter()
            .map(|g| energia_hora(*g, clima.temp[m], kwp))
            .sum();
        *energia = por_hora * f64::from(DAYS[m]);
    }
    let anual: f64 = mensual.iter().sum();
    Produccion {
        mensual,
        anual,
        especifica: anual / kwp,
    }
}

/// Plane-of-array to horizontal global ratio for each month.
pub(crate) fn factor_transposicion(sitio: &Sitio, clima: &Clima, o: Orientacion) -> [f64; 12] {
    let mut factores = [0.0; 12];
    for (m, celda) in factores.iter_mut().enumerate() {
        let poa = poa_horas(&ParametrosPoa {
            sitio,
            dia_anio: f64::from(DIA_KLEIN[m]),
            ghi_dia: clima.ghi[m],
            dif_dia: clima.dif[m],
            orientacion: o,
        });
        let total_kwh: f64 = poa.iter().sum::<f64>() / 1000.0;
        *celda = if clima.ghi[m] > 0.0 {
            total_kwh / clima.ghi[m]
        } else {
            0.0
        };
    }
    factores
}

/// Annual plane-of-array to horizontal ratio, weighting months by their horizontal energy.
pub(crate) fn factor_anual(clima: &Clima, mensual: &[f64; 12]) -> f64 {
    let horizontal: f64 = (0..12).map(|m| clima.ghi[m] * f64::from(DAYS[m])).sum();
    let inclinado: f64 = (0..12)
        .map(|m| clima.ghi[m] * f64::from(DAYS[m]) * mensual[m])
        .sum();
    if horizontal > 0.0 {
        inclinado / horizontal
    } else {
        0.0
    }
}
