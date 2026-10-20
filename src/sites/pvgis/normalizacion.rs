//! Data normalization and coordinate transformations for PVGIS data.
//! Handles calendar conversions, timezone shifts, and horizon profile decimation.

use crate::error::{Error, Result};
use crate::format::object;
use serde_json::Value;

/// Maximum horizon points accepted by PVsyst's `TCubicProfile`.
pub(crate) const MAX_PUNTOS: usize = 100;

/// Hours in a TMY year (PVGIS never includes 29 February).
pub(crate) const HORAS_ANIO: usize = 8_760;
/// Non-leap year used for the legal-time shift: a TMY mixes months from
/// different years, and shifting inside a leap February would invent 29 Feb.
const ANIO_REFERENCIA: i64 = 2019;
/// Physical bounds of the hourly fields.
const MAX_IRRADIANCIA_W_M2: f64 = 1_500.0;
const TEMP_MIN_C: f64 = -90.0;
const TEMP_MAX_C: f64 = 70.0;
const MAX_VIENTO_M_S: f64 = 100.0;
/// PVGIS emits small negative wind speeds from rounding: accept and clamp them.
const TOLERANCIA_VIENTO_NEGATIVO_M_S: f64 = -1.0;

/// One hourly TMY record already shifted to legal time.
#[derive(Debug, Clone)]
pub(crate) struct Hora {
    pub(crate) mes: i64,
    pub(crate) dia: i64,
    pub(crate) hh: i64,
    pub(crate) mm: i64,
    pub(crate) g: f64,
    pub(crate) gd: f64,
    pub(crate) t: f64,
    pub(crate) v: f64,
}

/// Civil date-time of a PVGIS `YYYYMMDD:HHMM` stamp, `None` unless every
/// separator, digit and calendar range is exact.
pub(crate) fn fecha_utc(t: &str) -> Option<(i64, i64, i64, i64, i64)> {
    let b = t.as_bytes();
    let forma = b.len() == 13
        && b.iter().enumerate().all(|(i, c)| {
            if i == 8 {
                *c == b':'
            } else {
                c.is_ascii_digit()
            }
        });
    if !forma {
        return None;
    }
    let n = |a: usize, z: usize| t.get(a..z)?.parse::<i64>().ok();
    let (y, mo, d, hh, mi) = (n(0, 4)?, n(4, 6)?, n(6, 8)?, n(9, 11)?, n(11, 13)?);
    let dias_mes =
        dias_desde_civil(y + i64::from(mo == 12), mo % 12 + 1, 1) - dias_desde_civil(y, mo, 1);
    let valida = (1..=12).contains(&mo) && (1..=dias_mes).contains(&d) && hh < 24 && mi < 60;
    valida.then_some((y, mo, d, hh, mi))
}

/// Days since 1970-01-01 of a proleptic Gregorian date (Howard Hinnant's algorithm).
pub(crate) fn dias_desde_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Inverse of `dias_desde_civil`: `(year, month, day)`.
pub(crate) fn civil_desde_dias(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}

/// Wind speed, clamping PVGIS' small negative rounding artifacts to zero.
fn velocidad_viento(f: &Value, t: &str) -> Result<f64> {
    let x = f["WS10m"]
        .as_f64()
        .ok_or_else(|| Error::Format("PVGIS: falta WS10m".into()))?;
    if !x.is_finite() || x < TOLERANCIA_VIENTO_NEGATIVO_M_S || x > MAX_VIENTO_M_S {
        return Err(Error::Format(format!(
            "PVGIS: WS10m={x} fuera de [{TOLERANCIA_VIENTO_NEGATIVO_M_S}, {MAX_VIENTO_M_S}] en {t}"
        )));
    }
    Ok(x.max(0.0))
}

/// Parse one TMY hourly JSON record and shift to legal time.
fn parsear_registro_hora(f: &Value, tz: i64) -> Result<Hora> {
    let t = f["time(UTC)"]
        .as_str()
        .ok_or_else(|| Error::Format("PVGIS: falta time(UTC)".into()))?;
    let (_, mo, d, hh, mi) =
        fecha_utc(t).ok_or_else(|| Error::Format(format!("PVGIS: time(UTC) inválido '{t}'")))?;
    let campo = |k: &str, min: f64, max: f64| {
        let x = f[k]
            .as_f64()
            .ok_or_else(|| Error::Format(format!("PVGIS: falta {k}")))?;
        if !x.is_finite() || x < min || x > max {
            return Err(Error::Format(format!(
                "PVGIS: {k}={x} fuera de [{min}, {max}] en {t}"
            )));
        }
        Ok(x)
    };
    let minutos = dias_desde_civil(ANIO_REFERENCIA, mo, d) * 1440 + hh * 60 + mi + tz * 60;
    let (_, mes, dia) = civil_desde_dias(minutos.div_euclid(1440));
    let resto = minutos.rem_euclid(1440);
    Ok(Hora {
        mes,
        dia,
        hh: resto / 60,
        mm: resto % 60,
        g: campo("G(h)", 0.0, MAX_IRRADIANCIA_W_M2)?,
        gd: campo("Gd(h)", 0.0, MAX_IRRADIANCIA_W_M2)?,
        t: campo("T2m", TEMP_MIN_C, TEMP_MAX_C)?,
        v: velocidad_viento(f, t)?,
    })
}

/// A yearly MET needs every hour of a non-leap year exactly once.
fn validar_anio_completo(horas: &[Hora]) -> Result<()> {
    if horas.len() != HORAS_ANIO {
        return Err(Error::Format(format!(
            "PVGIS: el TMY tiene {} registros; se esperaban {HORAS_ANIO}",
            horas.len()
        )));
    }
    let clave = |h: &Hora| (h.mes, h.dia, h.hh);
    if let Some(par) = horas.windows(2).find(|p| clave(&p[0]) == clave(&p[1])) {
        let (m, d, hh) = clave(&par[0]);
        return Err(Error::Format(format!(
            "PVGIS: hora repetida {d:02}/{m:02} {hh:02}h en el TMY"
        )));
    }
    Ok(())
}

/// Parse the PVGIS `tmy` JSON into legal-time records (`tz` hours from UTC).
pub(crate) fn parsear_tmy(texto: &str, tz: i64) -> Result<Vec<Hora>> {
    let v: Value = serde_json::from_str(texto)
        .map_err(|e| Error::Network(format!("Respuesta PVGIS no JSON: {e}")))?;
    let filas = v["outputs"]["tmy_hourly"]
        .as_array()
        .ok_or_else(|| Error::Format("PVGIS sin outputs.tmy_hourly".into()))?;
    let mut horas = filas
        .iter()
        .map(|f| parsear_registro_hora(f, tz))
        .collect::<Result<Vec<Hora>>>()?;
    // The shift moves the first rows into December: a yearly MET must run
    // from 1 January 00:00 to 31 December 23:00.
    horas.sort_by_key(|h| (h.mes, h.dia, h.hh, h.mm));
    validar_anio_completo(&horas)?;
    Ok(horas)
}

/// `;MM;DD;hh;mm;G;Gd;T;WS` per line, matching the field numbers of `PVsyst_ANE-NEDO.MEF`.
pub(crate) fn formatear_csv(horas: &[Hora]) -> String {
    let lineas: Vec<String> = horas
        .iter()
        .map(|h| {
            format!(
                ";{};{};{};{};{};{};{};{}",
                h.mes, h.dia, h.hh, h.mm, h.g, h.gd, h.t, h.v
            )
        })
        .collect();
    lineas.join("\r\n") + "\r\n"
}

/// Keep at most `MAX_PUNTOS` points, evenly spaced over the original profile.
pub(crate) fn diezmar(p: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    let n = p.len();
    if n <= MAX_PUNTOS {
        return p;
    }
    (0..MAX_PUNTOS)
        .map(|i| p[i * (n - 1) / (MAX_PUNTOS - 1)])
        .collect()
}

/// Compute FHHDiffus factor from horizon points.
pub(crate) fn calcular_fhh(puntos: &[(f64, f64)]) -> f64 {
    if puntos.is_empty() {
        return 1.0;
    }
    puntos
        .iter()
        .map(|(_, h)| h.to_radians().cos().powi(2))
        .sum::<f64>()
        / puntos.len() as f64
}

/// Recursive search for `pvHorizon` node.
fn encontrar_horizonte(nodos: &[object::PvNode]) -> Option<&object::PvNode> {
    nodos.iter().find_map(|n| {
        if n.obj_type == "pvHorizon" {
            Some(n)
        } else {
            encontrar_horizonte(&n.children)
        }
    })
}

/// Collect indices from `Point_<k>` fields.
fn indices_puntos(n: &object::PvNode, out: &mut Vec<usize>) {
    out.extend(n.fields.iter().filter_map(|(k, _)| {
        k.to_ascii_lowercase()
            .strip_prefix("point_")?
            .parse::<usize>()
            .ok()
    }));
    for c in &n.children {
        indices_puntos(c, out);
    }
}

/// Indices `k` of the `Point_<k>` fields already present in the variant's `pvHorizon`.
pub(crate) fn puntos_existentes(bytes: &[u8]) -> Vec<usize> {
    let nodos = object::parse(&String::from_utf8_lossy(bytes));
    let mut out = Vec::new();
    if let Some(h) = encontrar_horizonte(&nodos) {
        indices_puntos(h, &mut out);
    }
    out
}

/// Set `NPtsEff` and the `Point_i=az,elev` lines in place; surplus old points are zeroed.
pub(crate) fn cambios_perfil(
    puntos: &[(f64, f64)],
    existentes: &[usize],
) -> Vec<(String, Option<String>)> {
    let n = puntos.len();
    let mut c = vec![("NPtsEff".to_string(), Some(n.to_string()))];
    for (i, (az, el)) in puntos.iter().enumerate() {
        c.push((format!("Point_{}", i + 1), Some(format!("{az:.1},{el:.1}"))));
    }
    for k in existentes.iter().copied().filter(|k| *k > n) {
        c.push((format!("Point_{k}"), Some("0.0,0.0".to_string())));
    }
    c
}
