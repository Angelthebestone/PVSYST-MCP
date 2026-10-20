//! Solar geometry and sun position algorithms.
//! Sun position follows Spencer's declination and equation of time
//! (PVsyst azimuth: 0 = south, east negative).

use crate::args::opt_str;
use crate::calendario::{dias_del_mes, dias_desde_civil, FechaCivil};
use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::{json, Value};

use super::produccion::{factor_anual, factor_transposicion};
use super::{orientacion_de, redondear, sitio_de, Clima, Sitio};

/// Date used when `fecha` is not given: the June solstice (only the day of year matters).
const FECHA_DEFECTO: &str = "2021-06-21";

/// Day of the year (1-365 in standard years, 1-366 in leap years) of a `AAAA-MM-DD` date.
/// Follows the Gregorian calendar, properly supporting February 29 and shifting subsequent days.
pub(crate) fn dia_del_anio(fecha: &str) -> Result<u32> {
    let error = || Error::Invalid(format!("Fecha inválida '{fecha}': use AAAA-MM-DD"));
    let mut partes = fecha.split('-');
    let anio: u32 = partes
        .next()
        .and_then(|a| a.parse::<u32>().ok())
        .filter(|a| *a >= 1000)
        .ok_or_else(error)?;
    let mes: usize = partes
        .next()
        .and_then(|m| m.parse().ok())
        .filter(|m| (1..=12).contains(m))
        .ok_or_else(error)?;
    let dia: u32 = partes
        .next()
        .and_then(|d| d.parse().ok())
        .ok_or_else(error)?;
    let anio_civil = i64::from(anio);
    let max_dias = dias_del_mes(anio_civil, mes as u32);
    if partes.next().is_some() || dia == 0 || dia > max_dias {
        return Err(error());
    }
    let inicio = dias_desde_civil(FechaCivil::nuevo(anio_civil, 1, 1));
    let actual = dias_desde_civil(FechaCivil::nuevo(anio_civil, mes as u32, dia));
    u32::try_from(actual - inicio + 1).map_err(|_| error())
}

/// Solar declination (radians) for day of year `n` (Spencer).
pub(crate) fn declinacion(n: f64) -> f64 {
    (23.45 * (360.0 * (284.0 + n) / 365.0).to_radians().sin()).to_radians()
}

/// Equation of time in minutes for day of year `n`.
pub(crate) fn ecuacion_tiempo(n: f64) -> f64 {
    let b = (360.0 * (n - 1.0) / 365.0).to_radians();
    229.18
        * (0.000_075 + 0.001_868 * b.cos()
            - 0.032_077 * b.sin()
            - 0.014_615 * (2.0 * b).cos()
            - 0.040_849 * (2.0 * b).sin())
}

/// Local solar time (hours) at clock time `hora` on day `n`.
pub(crate) fn hora_solar(sitio: &Sitio, n: f64, hora: f64) -> f64 {
    hora + (4.0 * (15.0 * sitio.zona_horaria - sitio.longitud) + ecuacion_tiempo(n)) / 60.0
}

/// Cosine of the zenith and azimuth gamma_s (radians, 0 = south, positive west) of the sun
/// at hour angle `w` (radians).
pub(crate) fn posicion(lat: f64, dec: f64, w: f64) -> (f64, f64) {
    let cz = lat.sin() * dec.sin() + lat.cos() * dec.cos() * w.cos();
    let gs = w.sin().atan2(w.cos() * lat.sin() - dec.tan() * lat.cos());
    (cz, gs)
}

/// Sun elevation and PVsyst azimuth (degrees) at clock time `hora` (hours) on day `n`.
pub(crate) fn sol(sitio: &Sitio, n: f64, hora: f64) -> (f64, f64) {
    let w = (15.0 * (hora_solar(sitio, n, hora) - 12.0)).to_radians();
    let (cz, gs) = posicion(sitio.latitud.to_radians(), declinacion(n), w);
    (cz.clamp(-1.0, 1.0).asin().to_degrees(), gs.to_degrees())
}

/// Sun path of one day (sun-up hours only) and, with a tilt, the monthly transposition factor.
pub(crate) fn geometria_solar(_paths: &Paths, args: &Value) -> Result<Value> {
    let sitio = sitio_de(args)?;
    let fecha = opt_str(args, &["fecha"]).unwrap_or(FECHA_DEFECTO);
    let n = f64::from(dia_del_anio(fecha)?);
    let horas: Vec<Value> = (0..24)
        .filter_map(|h| {
            let hora = f64::from(h) + 0.5;
            let (elevacion, azimut) = sol(&sitio, n, hora);
            (elevacion > 0.0).then(|| {
                json!({
                    "hora_centro": hora,
                    "elevacion_grados": redondear(elevacion, 2),
                    "azimut_grados": redondear(azimut, 2),
                })
            })
        })
        .collect();
    let mut salida = json!({
        "latitud": sitio.latitud,
        "longitud": sitio.longitud,
        "zona_horaria": sitio.zona_horaria,
        "fecha": fecha,
        "horas_sol": horas,
    });
    if let Some(o) = orientacion_de(args, sitio.latitud)? {
        let clima = Clima::de_nasa(sitio.latitud, sitio.longitud)?;
        let mensual = factor_transposicion(&sitio, &clima, o);
        salida["transposicion"] = json!({
            "inclinacion": o.inclinacion,
            "azimut": o.azimut,
            "factor_mensual": mensual.iter().map(|f| redondear(*f, 4)).collect::<Vec<f64>>(),
            "factor_anual": redondear(factor_anual(&clima, &mensual), 4),
        });
    }
    Ok(salida)
}
