//! Tokenisation and field extraction from PVsyst PDF reports.

use crate::error::{Error, Result};
use serde_json::{json, Map, Value};

/// Parses a PVsyst report number. When a token has more than one `.`,
/// the last `.` is the decimal separator and preceding ones are thousands
/// separators (`23.410.00` -> 23410.0); otherwise normal parse.
pub(crate) fn parse_numero(t: &str) -> Option<f64> {
    let s = t.trim();
    if s.is_empty() {
        return None;
    }
    let dots = s.matches('.').count();
    if dots > 1 {
        let last_dot = s.rfind('.')?;
        let enteros = &s[..last_dot];
        let decimales = &s[last_dot + 1..];
        let normalizado = format!("{}.{}", enteros.replace(['.', ' '], ""), decimales);
        normalizado.parse::<f64>().ok()
    } else {
        s.replace(' ', "").parse::<f64>().ok()
    }
}

/// `+0.9%`, `-10.1%`: a signed percentage token.
pub(super) fn parse_pct(t: &str) -> Option<f64> {
    let body = t.strip_suffix('%')?;
    let digits = body
        .strip_prefix(|c: char| c == '+' || c == '-')
        .unwrap_or(body);
    let valid = digits.starts_with(|c: char| c.is_ascii_digit())
        && !digits.ends_with('.')
        && digits.chars().all(|c| c.is_ascii_digit() || c == '.')
        && digits.matches('.').count() <= 1;
    if valid {
        body.parse::<f64>().ok()
    } else {
        None
    }
}

/// `16693 kWh`, `1854  kWh/m²`: an energy or irradiation token.
pub(super) fn parse_energia(t: &str) -> Option<(f64, &'static str)> {
    let unidad = ["kWh/m²", "MWh", "kWh"]
        .into_iter()
        .find(|u| t.ends_with(u))?;
    let numero = &t[..t.len() - unidad.len()];
    parse_numero(numero).map(|v| (v, unidad))
}

/// A label token: text that is neither a percentage nor an energy value.
fn es_etiqueta(t: &str) -> bool {
    !t.is_empty()
        && parse_pct(t).is_none()
        && parse_energia(t).is_none()
        && t.chars().any(char::is_alphabetic)
        && !t.starts_with("Page ")
}

/// Spanish name for the standard loss-diagram labels; others keep PVsyst's text.
fn etiqueta_es(label: &str) -> String {
    let l = label.to_lowercase();
    let es = if l.contains("irradiance level") {
        "nivel de irradiancia"
    } else if l.contains("due to temperature") {
        "temperatura"
    } else if l.contains("module quality") {
        "calidad de módulo"
    } else if l.contains("mismatch") {
        "mismatch"
    } else if l.contains("ohmic wiring") {
        "cableado óhmico"
    } else if l.contains("iam factor") {
        "IAM"
    } else if l.contains("loss during operation") {
        "eficiencia del inversor"
    } else if l.contains("soiling") {
        "suciedad"
    } else if l.contains("shading") {
        "sombras"
    } else if l.contains("unavailability") {
        "indisponibilidad"
    } else if l.contains("over nominal inv. power") {
        "recorte de potencia"
    } else {
        return label.to_string();
    };
    es.to_string()
}

/// Loss diagram: a signed percentage precedes the label it applies to, and an
/// energy value follows its label. Uses the second `Loss diagram` heading.
pub(crate) fn perdidas(tokens: &[String]) -> Result<Value> {
    const SIN_DIAGRAMA: &str = "El PDF no contiene diagrama de pérdidas (¿se generó con -rpf?)";
    let headings: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter(|(_, t)| t.as_str() == "Loss diagram")
        .map(|(i, _)| i)
        .collect();
    let inicio = *headings
        .get(1)
        .or(headings.first())
        .ok_or_else(|| Error::Format(SIN_DIAGRAMA.into()))?;

    let fin = tokens[inicio + 1..]
        .iter()
        .position(|t| t.starts_with("Page "))
        .map_or(tokens.len(), |p| inicio + 1 + p);
    let mut nodos = Vec::new();
    for i in inicio + 1..fin {
        let t = &tokens[i];
        if let Some(pct) = parse_pct(t) {
            if let Some(label) = tokens.get(i + 1).filter(|s| es_etiqueta(s)) {
                nodos.push(json!({"concepto": label, "pct": pct}));
            }
        } else if let Some((valor, unidad)) = parse_energia(t) {
            if i > inicio + 1 && es_etiqueta(&tokens[i - 1]) {
                nodos.push(json!({"concepto": tokens[i - 1], "valor": valor, "unidad": unidad}));
            }
        }
    }

    let mut negativas: Vec<(String, f64)> = nodos
        .iter()
        .filter_map(|n| Some((n["concepto"].as_str()?.to_string(), n["pct"].as_f64()?)))
        .filter(|(_, pct)| *pct < 0.0)
        .collect();
    negativas.sort_by(|a, b| a.1.total_cmp(&b.1));
    let resumen = negativas
        .iter()
        .take(6)
        .map(|(label, pct)| format!("{pct:.1} % {}", etiqueta_es(label)).replace('-', "−"))
        .collect::<Vec<_>>()
        .join(", ");
    Ok(json!({"nodos": nodos, "resumen": resumen}))
}

fn tramo(tokens: &[String], inicio: usize, n: usize) -> Result<&[String]> {
    tokens
        .get(inicio..inicio + n)
        .ok_or_else(|| Error::Format("Fin inesperado del texto de la tabla mensual".into()))
}

/// Monthly table after `Balances and main results`: 8 column names, 8 units,
/// then 12 monthly rows and the `Year` row.
pub(crate) fn mensual(tokens: &[String]) -> Result<Value> {
    const SIN_TABLA: &str = "El PDF no contiene la tabla mensual (Balances and main results)";
    let p = tokens
        .iter()
        .position(|t| t == "Balances and main results")
        .ok_or_else(|| Error::Format(SIN_TABLA.into()))?;
    let columnas = tramo(tokens, p + 1, 8)?.to_vec();
    let unidades = tramo(tokens, p + 9, 8)?.to_vec();

    let mut k = p + 17;
    let mut meses = Vec::new();
    let mut anual = None;
    for _ in 0..13 {
        let etiqueta = tramo(tokens, k, 1)?[0].clone();
        let numeros = tramo(tokens, k + 1, 8)?;
        let mut valores = Map::new();
        for (col, raw) in columnas.iter().zip(numeros) {
            let n = parse_numero(raw).ok_or_else(|| {
                Error::Invalid(format!("Valor mensual no numérico '{raw}' en {etiqueta}"))
            })?;
            valores.insert(col.clone(), json!(n));
        }
        k += 9;
        if etiqueta == "Year" {
            anual = Some(Value::Object(valores));
        } else {
            meses.push(json!({"mes": etiqueta, "valores": valores}));
        }
    }
    let anual =
        anual.ok_or_else(|| Error::Format("La tabla mensual no contiene la fila Year".into()))?;
    Ok(json!({"columnas": columnas, "unidades": unidades, "meses": meses, "anual": anual}))
}

/// Headline results: produced energy (required), specific production and PR.
pub(crate) fn resumen(tokens: &[String]) -> Result<Value> {
    let energia = valor_de_etiqueta(tokens, "Produced Energy")
        .ok_or_else(|| Error::Format("El informe no contiene Produced Energy".into()))?;
    Ok(json!({
        "energia_anual_kwh": energia,
        "produccion_especifica_kwh_kwp": valor_de_etiqueta(tokens, "Specific production"),
        "pr_pct": valor_de_etiqueta(tokens, "Perf. Ratio PR"),
    }))
}

/// Value printed for `etiqueta` in a PVsyst results box.
///
/// The report lays a box out either as `label value` pairs or as a row of
/// labels followed by a row of values (`Produced Energy`, `Used Energy`,
/// `16539`, `40515`). The labels after `etiqueta` and before the first number
/// tell its column among the values that follow. O(n) in the tokens.
pub(crate) fn valor_de_etiqueta(tokens: &[String], etiqueta: &str) -> Option<f64> {
    let p = tokens.iter().position(|t| t == etiqueta)?;
    let resto = &tokens[p + 1..];
    let etiquetas_detras = resto
        .iter()
        .take_while(|t| parse_numero(t).is_none())
        .count();
    let valores: Vec<f64> = resto[etiquetas_detras..]
        .iter()
        .map_while(|t| parse_numero(t))
        .collect();
    let columna = valores.len().checked_sub(etiquetas_detras + 1)?;
    valores.get(columna).copied()
}
