//! "P50 - P90 evaluation" page of a PVsyst report: PVsyst's weather
//! variability, its simulation uncertainties and the resulting P50, P90 and
//! other Pxx annual productions.

use super::parse::parse_numero;
use crate::error::{Error, Result};
use serde_json::{json, Map, Value};

const PAGINA: &str = "P50 - P90 evaluation";
const SIN_PAGINA: &str = "El informe PDF no contiene la página 'P50 - P90 evaluation'. \
    Añade la evaluación a la variante con 'proyecto accion:p50_p90' y vuelve a simular \
    con paginas_reporte que incluya 'p50p90'.";

fn posicion(t: &[String], etiqueta: &str) -> Option<usize> {
    t.iter().position(|x| x == etiqueta)
}

fn es_numero(t: &str) -> bool {
    parse_numero(t).is_some()
}

/// Labels from `inicio` up to the first number, then as many numbers.
fn filas(t: &[String], inicio: usize) -> Vec<(String, f64)> {
    let etiquetas: Vec<&String> = t[inicio..].iter().take_while(|x| !es_numero(x)).collect();
    let valores: Vec<f64> = t[inicio + etiquetas.len()..]
        .iter()
        .filter_map(|x| parse_numero(x))
        .take(etiquetas.len())
        .collect();
    etiquetas.into_iter().cloned().zip(valores).collect()
}

fn primer_numero(t: &[String], desde: usize) -> Option<f64> {
    t.get(desde..)?.iter().find_map(|x| parse_numero(x))
}

/// Reads the P50-P90 page from the report tokens.
pub(crate) fn p50(tokens: &[String]) -> Result<Value> {
    let inicio = tokens
        .iter()
        .rposition(|x| x == PAGINA)
        .ok_or_else(|| Error::NotFound(SIN_PAGINA.into()))?;
    let t = &tokens[inicio..];
    let falta = || Error::Format("Página P50 - P90 incompleta en el informe".into());
    let tipo = posicion(t, "Kind").ok_or_else(falta)?;
    let incertidumbres =
        posicion(t, "Simulation and parameters uncertainties").ok_or_else(falta)?;
    let probabilidad = posicion(t, "Annual production probability").ok_or_else(falta)?;
    let global = t[..probabilidad].iter().rev().find_map(|x| parse_numero(x));
    let resultado = filas(t, probabilidad + 1);
    let pxx: Map<String, Value> = resultado
        .iter()
        .filter(|(k, _)| k.starts_with('P'))
        .map(|(k, v)| (k.clone(), json!(v)))
        .collect();
    if pxx.is_empty() {
        return Err(falta());
    }
    Ok(json!({
        "fuente_meteo": t.get(tipo + 1),
        "tipo_datos_meteo": t.get(tipo + 2),
        "variabilidad_meteo_pct": primer_numero(t, tipo + 1),
        "desviacion_media_pct": posicion(t, "Mean energy shift").and_then(|p| primer_numero(t, p)),
        "incertidumbres_pct": filas(t, incertidumbres + 1)
            .into_iter()
            .map(|(k, v)| (k, json!(v)))
            .collect::<Map<String, Value>>(),
        "variabilidad_global_pct": global,
        "variabilidad_kwh": resultado.iter().find(|(k, _)| k == "Variability").map(|(_, v)| v),
        "produccion_kwh": pxx,
    }))
}
