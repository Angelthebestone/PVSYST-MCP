use serde_json::{json, Value};

pub(super) fn parse_number(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    t.replace(',', ".").parse::<f64>().ok()
}

pub(super) fn accumulate_columns(rows: &[&str], ncols: usize) -> (Vec<f64>, Vec<usize>, usize) {
    let mut totals = vec![0.0; ncols];
    let mut counts = vec![0usize; ncols];
    let mut row_count = 0usize;
    for l in rows.iter().filter(|l| !l.trim().is_empty()) {
        row_count += 1;
        for (idx, field) in l
            .split(';')
            .enumerate()
            .skip(1)
            .take(ncols.saturating_sub(1))
        {
            if let Some(v) = parse_number(field) {
                totals[idx] += v;
                counts[idx] += 1;
            }
        }
    }
    (totals, counts, row_count)
}

/// Energy integral of an hourly power column, labelled with its energy unit.
///
/// Only power columns are integrated: averaging ratios (`PR`) or summing
/// angles would invent figures PVsyst never reports; its PR is in the PDF
/// (`resultados que:kpis`).
pub(super) fn column_total(sum: f64, unit: &str) -> Option<Value> {
    let round = |v: f64, d: i32| (v * 10f64.powi(d)).round() / 10f64.powi(d);
    let energia = match unit {
        "W" | "kW" | "MW" => format!("{unit}h"),
        u => {
            let (potencia, resto) = u.split_once('/')?;
            if !matches!(potencia, "W" | "kW" | "MW") {
                return None;
            }
            format!("{potencia}h/{resto}")
        }
    };
    Some(json!({ "sum": round(sum, 1), "unit": energia }))
}

pub(super) fn summarize_csv(text: &str) -> Value {
    let lines: Vec<&str> = text.trim_start_matches('\u{feff}').lines().collect();
    let header_idx = lines.iter().position(|l| {
        l.split(';')
            .next()
            .unwrap_or("")
            .trim()
            .eq_ignore_ascii_case("date")
    });

    let cells = |l: &str| -> Vec<String> { l.split(';').map(|s| s.trim().to_string()).collect() };
    let (header, units, data_start) = match header_idx {
        Some(i) => (
            cells(lines[i]),
            lines.get(i + 1).map(|l| cells(l)).unwrap_or_default(),
            i + 2,
        ),
        None => (Vec::new(), Vec::new(), 0),
    };

    let metadata: Vec<String> = lines
        .iter()
        .take(header_idx.unwrap_or(lines.len().min(20)))
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string())
        .collect();

    let (totals, counts, row_count) =
        accumulate_columns(&lines[data_start.min(lines.len())..], header.len());

    let totals_json: serde_json::Map<String, Value> = header
        .iter()
        .enumerate()
        .skip(1)
        .filter(|(idx, _)| counts[*idx] > 0)
        .filter_map(|(idx, name)| {
            let unit = units.get(idx).map_or("", String::as_str);
            column_total(totals[idx], unit).map(|t| (name.clone(), t))
        })
        .collect();

    json!({
        "metadata": metadata,
        "header": header,
        "row_count": row_count,
        "totals": Value::Object(totals_json),
    })
}
