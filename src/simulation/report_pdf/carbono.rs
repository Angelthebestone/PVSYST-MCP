//! Carbon balance reader from PVsyst PDF report page.

use super::parse::parse_numero;
use crate::error::{Error, Result};
use serde_json::{json, Map, Value};

const SIN_CARBONO: &str =
    "El informe PDF no contiene la página de balance de carbono ('CO2 Emission Balance'). \
    Active el balance de carbono en la variante (PrintInReport=True) y vuelva a simular \
    generando el informe PDF (-rpf).";

fn valor_total_tras(sub: &[String], encabezado: &str, max_dist: usize) -> Option<f64> {
    let p = sub.iter().position(|t| t == encabezado)?;
    sub.get(p..p + max_dist)?
        .iter()
        .skip(1)
        .find_map(|t| parse_numero(t))
}

fn valor_balance_neto(sub: &[String]) -> Option<f64> {
    let p_gen = sub.iter().position(|t| t == "Generated emissions")?;
    let p_tot = sub[..p_gen].iter().position(|t| t.starts_with("Total:"))?;
    parse_numero(&sub[p_tot + 1])
}

fn parse_campo_parametro(out: &mut Map<String, Value>, label: &str, val: &str) {
    let l = label.to_lowercase();
    if l.contains("system production") {
        if let Some(n) = parse_numero(val) {
            out.insert("produccion_sistema".into(), json!(n));
            out.insert("produccion_sistema_unidad".into(), json!("MWh/yr"));
        }
    } else if l.contains("grid lifecycle") {
        if let Some(n) = parse_numero(val) {
            out.insert("factor_lce_red".into(), json!(n));
            out.insert("factor_lce_red_unidad".into(), json!("gCO2/kWh"));
        }
    } else if l.contains("source") {
        out.insert("fuente".into(), json!(val.trim()));
    } else if l.contains("country") {
        out.insert("pais".into(), json!(val.trim()));
    } else if l.contains("lifetime") {
        if let Some(n) = parse_numero(val) {
            out.insert("vida_util_anios".into(), json!(n));
        }
    } else if l.contains("degradation") {
        if let Some(n) = parse_numero(val) {
            out.insert("degradacion_anual_pct".into(), json!(n));
        }
    }
}

fn parse_parametros(tokens: &[String], inicio: usize) -> Option<Map<String, Value>> {
    let rel = tokens[inicio..]
        .iter()
        .position(|t| t.starts_with("System production"))?;
    let base = inicio + rel;
    let n_labels = tokens[base..]
        .iter()
        .take_while(|t| t.trim().ends_with(':'))
        .count();
    if n_labels == 0 || base + 2 * n_labels > tokens.len() {
        return None;
    }
    let labels = &tokens[base..base + n_labels];
    let values = &tokens[base + n_labels..base + 2 * n_labels];
    let mut out = Map::new();
    for (label, val) in labels.iter().zip(values) {
        parse_campo_parametro(&mut out, label, val);
    }
    Some(out)
}

fn parse_detalles_lce(tokens: &[String], inicio: usize) -> Vec<Value> {
    let Some(rel_p) = tokens[inicio..]
        .iter()
        .position(|t| t == "System Lifecycle Emissions Details")
    else {
        return Vec::new();
    };
    let base = inicio + rel_p;
    let Some(end_header) = tokens[base..].iter().position(|t| t == "]") else {
        return Vec::new();
    };
    let mut start = base + end_header + 1;
    let mut filas = Vec::new();
    while start + 4 <= tokens.len() {
        let concepto = &tokens[start];
        if concepto.starts_with("Page ") || concepto.is_empty() {
            break;
        }
        let lce = &tokens[start + 1];
        let cantidad = &tokens[start + 2];
        let subtotal_raw = &tokens[start + 3];
        let Some(subtotal) = parse_numero(subtotal_raw) else {
            break;
        };
        filas.push(json!({
            "concepto": concepto,
            "lce": lce,
            "cantidad": cantidad,
            "subtotal_kgco2": subtotal,
        }));
        start += 4;
    }
    filas
}

/// Reads the Carbon Balance page from the PDF report tokens.
pub(crate) fn carbono(tokens: &[String]) -> Result<Value> {
    let inicio = tokens
        .iter()
        .position(|t| t.contains("Emission Balance"))
        .ok_or_else(|| Error::NotFound(SIN_CARBONO.into()))?;
    let sub = &tokens[inicio..];
    let emisiones_evitadas = valor_total_tras(sub, "Replaced Emissions", 6).ok_or_else(|| {
        Error::Format("No se encontró el total de emisiones evitadas (Replaced Emissions)".into())
    })?;
    let emisiones_ciclo_vida =
        valor_total_tras(sub, "Generated emissions", 8).ok_or_else(|| {
            Error::Format(
                "No se encontró el total de emisiones generadas (Generated emissions)".into(),
            )
        })?;
    let balance_neto = valor_balance_neto(sub);

    let mut out = Map::new();
    out.insert("emisiones_evitadas_total".into(), json!(emisiones_evitadas));
    out.insert("emisiones_evitadas_unidad".into(), json!("tCO2"));
    out.insert(
        "emisiones_ciclo_vida_total".into(),
        json!(emisiones_ciclo_vida),
    );
    out.insert("emisiones_ciclo_vida_unidad".into(), json!("tCO2"));
    if let Some(neto) = balance_neto {
        out.insert("balance_neto_total".into(), json!(neto));
        out.insert("balance_neto_unidad".into(), json!("tCO2"));
    }
    if let Some(params) = parse_parametros(tokens, inicio) {
        for (k, v) in params {
            out.insert(k, v);
        }
    }
    let detalles = parse_detalles_lce(tokens, inicio);
    out.insert("detalles_lce".into(), json!(detalles));

    Ok(Value::Object(out))
}
