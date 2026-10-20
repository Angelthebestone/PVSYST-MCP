use super::caso::{caso, con_cabecera, leer_bytes, sin_potencia, Caso};
use super::supuestos::{acotado, anios, Rango, DEGRADACION};
use crate::config::{data_ro, Paths};
use crate::error::{Error, Result};
use serde_json::{json, Value};

const LCE: Rango = Rango {
    clave: "lce_kgco2_por_kwp",
    min: 0.0,
    max: 10_000.0,
};

pub(super) fn factor_pais(tabla: &str, pais: Option<&str>) -> Result<(f64, String)> {
    let filas: Vec<(&str, f64)> = tabla
        .lines()
        .filter_map(|l| {
            let mut partes = l.split(';');
            let nombre = partes.next()?.trim();
            let valor = partes.next()?.trim().parse::<f64>().ok()?;
            Some((nombre, valor))
        })
        .collect();
    if let Some(p) = pais {
        if let Some((_, f)) = filas.iter().find(|(n, _)| n.eq_ignore_ascii_case(p.trim())) {
            return Ok((*f, p.trim().to_string()));
        }
    }
    let (_, world) = filas
        .iter()
        .find(|(n, _)| *n == "World")
        .ok_or_else(|| Error::NotFound("Tabla de factores sin la fila World".into()))?;
    let etiqueta = match pais {
        Some(p) => format!("World (no encontrado {})", p.trim()),
        None => "World (sin país en la simulación)".to_string(),
    };
    Ok((*world, etiqueta))
}

/// Avoided CO2 (kg) over `vida` whole years with compound degradation. O(vida).
pub(super) fn ahorro_co2_kg(energia_kwh: f64, deg: f64, vida: u32, factor_g_kwh: f64) -> f64 {
    let energia_total: f64 = (0..vida)
        .scan(energia_kwh, |energia, _| {
            let actual = *energia;
            *energia *= 1.0 - deg;
            Some(actual)
        })
        .sum();
    energia_total * factor_g_kwh / 1000.0
}

pub(crate) fn carbono_con(c: &Caso, args: &Value) -> Result<Value> {
    let kwp = c.kwp.ok_or_else(sin_potencia)?;
    let vida = anios(args, "vida_anios", 30.0)?;
    let deg = acotado(args, &DEGRADACION, Some(0.01))?;
    let lce = acotado(args, &LCE, Some(1700.0))?;
    let tabla = String::from_utf8_lossy(&leer_bytes(
        &data_ro().join("CarbonBalance_Definitions.txt"),
    )?)
    .into_owned();
    let pais = c
        .pais_csv
        .as_deref()
        .or_else(|| c.sitio().ok()?.pais.as_deref());
    let (factor, etiqueta) = factor_pais(&tabla, pais)?;
    let ahorro = ahorro_co2_kg(c.energia_kwh, deg, vida, factor);
    let lce_kg = lce * kwp;
    Ok(json!({
        "ahorro_co2_kg": ahorro.round(),
        "lce_co2_kg": lce_kg.round(),
        "neto_co2_kg": (ahorro - lce_kg).round(),
        "factor_gco2_kwh": factor,
        "pais_factor": etiqueta,
        "supuestos": {
            "vida_anios": vida,
            "degradacion": deg,
            "lce_kgco2_por_kwp": lce,
            "factor_fuente": "CarbonBalance_Definitions.txt",
        },
    }))
}

pub(crate) fn carbono(paths: &Paths, args: &Value) -> Result<Value> {
    let c = caso(paths, args)?;
    Ok(con_cabecera(&c, carbono_con(&c, args)?))
}
