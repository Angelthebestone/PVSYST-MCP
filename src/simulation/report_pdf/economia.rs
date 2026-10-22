//! Economic evaluation reader from PVsyst PDF report summary page.

use super::parse::parse_numero;
use crate::error::{Error, Result};
use serde_json::{json, Map, Value};

const SIN_ECONOMIA: &str = "El informe PDF no contiene el bloque de evaluación económica ('Economic evaluation'). \
    Configure la evaluación económica con 'proyecto accion:economia' en una variante con EconomicEval \
    y vuelva a simular generando el informe PDF (-rpf).";

fn parse_bloque(tokens: &[String], encabezado: &str) -> Option<Vec<(String, f64, String)>> {
    let p = tokens.iter().position(|t| t == encabezado)?;
    let mut cols = Vec::new();
    let mut k = p + 1;
    while k < tokens.len() && parse_numero(&tokens[k]).is_none() {
        cols.push(tokens[k].clone());
        k += 1;
    }
    if cols.is_empty() || k + 2 * cols.len() > tokens.len() {
        return None;
    }
    let mut filas = Vec::with_capacity(cols.len());
    for (i, col) in cols.iter().enumerate() {
        let val = parse_numero(&tokens[k + i])?;
        let unidad = tokens[k + cols.len() + i].clone();
        filas.push((col.clone(), val, unidad));
    }
    Some(filas)
}

fn asignar_inversion(out: &mut Map<String, Value>, inv: Vec<(String, f64, String)>) {
    for (col, val, unidad) in inv {
        if col == "Global" {
            out.insert("inversion_global".into(), json!(val));
            out.insert("moneda".into(), json!(unidad));
        } else if col == "Specific" {
            out.insert("inversion_especifica".into(), json!(val));
            out.insert("unidad_especifica".into(), json!(unidad));
        }
    }
}

fn asignar_costes(out: &mut Map<String, Value>, cost: Vec<(String, f64, String)>) {
    for (col, val, unidad) in cost {
        if col == "Annuities" {
            out.insert("anualidades".into(), json!(val));
            out.insert("anualidades_unidad".into(), json!(unidad));
        } else if col == "Run. costs" {
            out.insert("costes_operacion".into(), json!(val));
            out.insert("costes_operacion_unidad".into(), json!(unidad));
        } else if col == "Payback period" {
            out.insert("payback_anios".into(), json!(val));
        }
    }
}

fn asignar_lcoe(out: &mut Map<String, Value>, lcoe: Vec<(String, f64, String)>) {
    for (_col, val, unidad) in lcoe {
        out.insert("lcoe".into(), json!(val));
        out.insert("lcoe_unidad".into(), json!(unidad));
    }
}

/// Reads the 'Economic evaluation' block from the PDF report summary tokens.
pub(crate) fn economia(tokens: &[String]) -> Result<Value> {
    let inicio = tokens
        .iter()
        .position(|t| t == "Economic evaluation")
        .ok_or_else(|| Error::NotFound(SIN_ECONOMIA.into()))?;
    let sub = &tokens[inicio..];
    let mut out = Map::new();
    if let Some(inv) = parse_bloque(sub, "Investment") {
        asignar_inversion(&mut out, inv);
    }
    if let Some(cost) = parse_bloque(sub, "Yearly cost") {
        asignar_costes(&mut out, cost);
    }
    if let Some(lcoe) = parse_bloque(sub, "LCOE") {
        asignar_lcoe(&mut out, lcoe);
    }
    if out.is_empty() {
        return Err(Error::NotFound(SIN_ECONOMIA.into()));
    }
    Ok(Value::Object(out))
}
