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

pub(super) fn column_total(sum: f64, n: usize, unit: &str) -> Value {
    let round = |v: f64, d: i32| (v * 10f64.powi(d)).round() / 10f64.powi(d);
    if unit.eq_ignore_ascii_case("ratio") {
        #[expect(clippy::cast_precision_loss, reason = "row counts are far below 2^52")]
        let mean = sum / n as f64;
        return json!({ "mean": round(mean, 3) });
    }
    let unit = match unit {
        "W" => "Wh".to_string(),
        "kW" => "kWh".to_string(),
        u => u
            .strip_prefix("W/")
            .map_or_else(|| u.to_string(), |rest| format!("Wh/{rest}")),
    };
    json!({ "sum": round(sum, 1), "unit": unit })
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
        .map(|(idx, name)| {
            let unit = units.get(idx).map_or("", String::as_str);
            (name.clone(), column_total(totals[idx], counts[idx], unit))
        })
        .collect();

    json!({
        "metadata": metadata,
        "header": header,
        "row_count": row_count,
        "totals": Value::Object(totals_json),
    })
}
