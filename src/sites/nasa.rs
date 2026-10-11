//! Build a monthly climatology from the NASA SSE 1°×1° grids that ship with
//! PVsyst (`DataRO\Global_World.txt`, `Diffuse_World.TXT`, `Tamb10m_World.txt`).
//!
//! This removes the biggest friction for `create-site`: the caller only needs
//! coordinates — no weather file. The grids are regional averages per 1° cell
//! (lower-left corner = Lat/Lon value).

use crate::config::data_ro;
use crate::error::{Error, Result};
use std::fmt::Write as _;
use std::path::Path;

pub(crate) const DAYS: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

#[derive(Debug, Clone)]
pub(crate) struct Climatology {
    pub cell_lat: i64,
    pub cell_lon: i64,
    /// kWh/m²/day per month
    pub ghi_daily: Vec<f64>,
    /// kWh/m²/day per month
    pub diff_daily: Vec<f64>,
    /// °C per month
    pub temp: Vec<f64>,
}

/// Column index offset for monthly data: Lat=col 1, Lon=col 2, Jan=col 3.
const COLUMNA_PRIMER_MES: usize = 3;
/// Minimum columns required per row in NASA grid files (Lat, Lon, 12 months).
const COLUMNAS_MINIMAS_GRID: usize = 14;

/// Parse one grid file and return the 12 monthly values for the cell.
/// Corrupted or non-numeric monthly values in a matching cell row are rejected with contextual error.
pub(crate) fn parse_grid_at(text: &str, lat: i64, lon: i64) -> Result<Option<Vec<f64>>> {
    for line in text.lines() {
        let t = line.trim();
        let first = match t.chars().next() {
            Some(c) => c,
            None => continue,
        };
        if !(first.is_ascii_digit() || first == '-') {
            continue;
        }
        let parts: Vec<&str> = t.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let la: i64 = parts[0].parse().unwrap_or(i64::MIN);
        let lo: i64 = parts[1].parse().unwrap_or(i64::MIN);
        if la == lat && lo == lon {
            if parts.len() < COLUMNAS_MINIMAS_GRID {
                return Err(Error::Format(format!(
                    "Fila incompleta para celda ({lat}, {lon}): esperaba {COLUMNAS_MINIMAS_GRID} columnas, se encontraron {}",
                    parts.len()
                )));
            }
            let mut vals = Vec::with_capacity(12);
            for (idx, token) in parts[2..14].iter().enumerate() {
                let columna = idx + COLUMNA_PRIMER_MES;
                let val: f64 = token.parse().map_err(|_| {
                    Error::Format(format!(
                        "Valor mensual no numérico '{token}' en celda ({lat}, {lon}), columna {columna} (mes {})",
                        idx + 1
                    ))
                })?;
                if !val.is_finite() {
                    return Err(Error::Format(format!(
                        "Valor mensual no finito '{token}' en celda ({lat}, {lon}), columna {columna} (mes {})",
                        idx + 1
                    )));
                }
                vals.push(val);
            }
            return Ok(Some(vals));
        }
    }
    Ok(None)
}

fn read_grid(path: &Path, lat: i64, lon: i64) -> Result<Vec<f64>> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| Error::io(format!("Cannot read {}", path.display()), e))?;
    parse_grid_at(&text, lat, lon)?.ok_or_else(|| {
        Error::NotFound(format!(
            "Cell {}/{} not found in {}",
            lat,
            lon,
            path.display()
        ))
    })
}

pub(crate) fn climatology(lat: f64, lon: f64) -> Result<Climatology> {
    let la = lat.floor() as i64;
    let lo = lon.floor() as i64;
    let base = data_ro();
    let ghi = read_grid(&base.join("Global_World.txt"), la, lo)?;
    let dif = read_grid(&base.join("Diffuse_World.TXT"), la, lo)?;
    let tmp = read_grid(&base.join("Tamb10m_World.txt"), la, lo)?;
    Ok(Climatology {
        cell_lat: la,
        cell_lon: lo,
        ghi_daily: ghi,
        diff_daily: dif,
        temp: tmp,
    })
}

/// Convert a daily-average series into monthly totals (kWh/m²/month).
pub(crate) fn monthly_totals(daily: &[f64]) -> Vec<f64> {
    daily
        .iter()
        .enumerate()
        .map(|(i, v)| v * DAYS.get(i).copied().unwrap_or(30) as f64)
        .collect()
}

impl Climatology {
    /// `create-site` monthly-weather CSV (semicolon separated, 2 header rows +
    /// 12 months). Columns: GlobH, DiffH, Temp, WindVel.
    pub(crate) fn to_create_site_csv(&self) -> String {
        let ghi = monthly_totals(&self.ghi_daily);
        let dif = monthly_totals(&self.diff_daily);
        let mut s = String::from("GlobH;DiffH;Temp;WindVel\nkWh/m2;kWh/m2;degC;m/s\n");
        // Writing to a String is infallible.
        for ((g, d), t) in ghi.iter().zip(&dif).zip(&self.temp) {
            let _ = writeln!(
                s,
                "{:.1};{:.1};{:.1};2.0",
                round1(*g),
                round1(*d),
                round1(*t)
            );
        }
        s
    }

    pub(crate) fn annual_ghi(&self) -> f64 {
        monthly_totals(&self.ghi_daily).iter().sum()
    }
}

fn round1(x: f64) -> f64 {
    crate::format::redondear(x, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "Lat Lon Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec Ann\n-90 -180 9.0 8.0 7.0 6.0 5.0 4.0 3.0 2.0 1.0 2.0 3.0 4.0 4.5\n7 -74 5.34 5.34 5.28 4.96 5.01 5.16 5.57 5.55 5.27 4.80 4.72 4.86 5.16\n";

    #[test]
    fn should_find_cell_and_return_twelve_values() {
        let v = parse_grid_at(SAMPLE, 7, -74)
            .expect("lectura exitosa")
            .expect("celda presente");
        assert_eq!(v.len(), 12);
        assert!((v[0] - 5.34).abs() < 1e-9);
        assert!((v[11] - 4.86).abs() < 1e-9);
    }

    #[test]
    fn should_return_none_when_cell_is_missing() {
        assert!(parse_grid_at(SAMPLE, 10, 10).expect("lectura").is_none());
    }

    #[test]
    fn should_reject_row_with_unparseable_monthly_value() {
        let sample_con_abc = "Lat Lon Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec Ann\n7 -74 5.34 abc 5.28 4.96 5.01 5.16 5.57 5.55 5.27 4.80 4.72 4.86 5.16\n";
        let res = parse_grid_at(sample_con_abc, 7, -74);
        assert!(res.is_err(), "debe fallar con valor no numérico");
        let err = res.unwrap_err().to_string();
        assert!(err.contains("abc"), "debe reportar 'abc': {err}");
        assert!(
            err.contains("7") && err.contains("-74"),
            "debe reportar celda 7/-74"
        );
    }

    #[test]
    fn should_reject_row_with_nan_monthly_value() {
        let sample_con_nan = "Lat Lon Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec Ann\n7 -74 5.34 NaN 5.28 4.96 5.01 5.16 5.57 5.55 5.27 4.80 4.72 4.86 5.16\n";
        let res = parse_grid_at(sample_con_nan, 7, -74);
        assert!(res.is_err(), "debe fallar con NaN");
    }

    #[test]
    fn should_calculate_monthly_totals_using_days_per_month() {
        let daily = vec![1.0; 12];
        let m = monthly_totals(&daily);
        assert_eq!(m[0], 31.0);
        assert_eq!(m[1], 28.0);
        assert_eq!(m[11], 31.0);
    }
}
