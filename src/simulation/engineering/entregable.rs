use super::carbono::carbono_con;
use super::caso::{caso, con_cabecera, especifica, num, sin_potencia, Caso};
use super::economia::{economia_de, parse_economia};
use super::incertidumbre::{incertidumbre, sigma_meteo, sigma_modelo};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::simulation::report_pdf;
use serde_json::{json, Value};
use std::fs;

pub(super) fn tabla_mensual(mens: &Value) -> Vec<String> {
    let cols: Vec<&str> = mens["columnas"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let mut lineas = vec![
        format!("| mes | {} |", cols.join(" | ")),
        format!("|{}", "---|".repeat(cols.len() + 1)),
    ];
    for m in mens["meses"].as_array().into_iter().flatten() {
        let celdas: Vec<String> = cols.iter().map(|c| m["valores"][*c].to_string()).collect();
        lineas.push(format!(
            "| {} | {} |",
            m["mes"].as_str().unwrap_or(""),
            celdas.join(" | ")
        ));
    }
    lineas
}

pub(super) fn seccion_resultados(c: &Caso, resumen: &Value) -> Vec<String> {
    vec![
        format!("# {} · {}", c.proyecto, c.variante),
        String::new(),
        "## Resultados".to_string(),
        format!("- Energía anual: {} kWh", c.energia_kwh),
        format!(
            "- Potencia: {} kWp",
            c.kwp.map_or("desconocida".to_string(), |k| k.to_string())
        ),
        format!(
            "- Producción específica: {} kWh/kWp",
            especifica(c).map_or("desconocida".to_string(), |e| e.to_string())
        ),
        format!("- Performance Ratio: {} %", resumen["pr_pct"]),
    ]
}

pub(super) fn linea_carbono(carbono: &Result<Value>) -> String {
    match carbono {
        Ok(v) => format!(
            "- Balance neto: {} kgCO2 (ahorro {}, LCE {}); factor {} gCO2/kWh ({})",
            v["neto_co2_kg"],
            v["ahorro_co2_kg"],
            v["lce_co2_kg"],
            v["factor_gco2_kwh"],
            v["pais_factor"].as_str().unwrap_or("")
        ),
        Err(e) => format!("- No disponible: {e}"),
    }
}

pub(super) fn markdown(
    c: &Caso,
    perd: &Value,
    mens: &Value,
    resumen: &Value,
    p50: &Value,
    carbono: &Result<Value>,
    econ: Option<&Value>,
) -> String {
    let mut l = seccion_resultados(c, resumen);
    l.push(String::new());
    l.push("## Pérdidas".to_string());
    l.push(perd["resumen"].as_str().unwrap_or_default().to_string());
    for n in perd["nodos"].as_array().into_iter().flatten() {
        if let Some(p) = n.get("pct") {
            l.push(format!(
                "- {}: {} %",
                n["concepto"].as_str().unwrap_or(""),
                p
            ));
        }
    }
    l.push(String::new());
    l.push("## Tabla mensual".to_string());
    l.extend(tabla_mensual(mens));
    l.push(String::new());
    l.push("## P50 / P90".to_string());
    l.push(format!(
        "- P50: {} kWh; P90: {} kWh; P99: {} kWh (σ total {} %)",
        p50["p50_kwh"], p50["p90_kwh"], p50["p99_kwh"], p50["sigma_total_pct"]
    ));
    l.push(String::new());
    l.push("## Carbono".to_string());
    l.push(linea_carbono(carbono));
    if let Some(e) = econ {
        let moneda = e["moneda"].as_str().unwrap_or("");
        l.push(String::new());
        l.push("## Economía".to_string());
        l.push(format!(
            "- LCOE: {} {moneda}/kWh; payback: {} años; VAN: {} {moneda}",
            e["lcoe_por_kwh"], e["payback_anios"], e["van"]
        ));
    }
    l.push(String::new());
    l.push(format!("PDF: {}", c.pdf.display()));
    l.join("\n") + "\n"
}

pub(crate) fn perdidas(paths: &Paths, args: &Value) -> Result<Value> {
    let c = caso(paths, args)?;
    Ok(con_cabecera(&c, report_pdf::perdidas(&c.tokens)?))
}

pub(crate) fn mensual(paths: &Paths, args: &Value) -> Result<Value> {
    let c = caso(paths, args)?;
    Ok(con_cabecera(&c, report_pdf::mensual(&c.tokens)?))
}

pub(crate) fn entregable(paths: &Paths, args: &Value) -> Result<Value> {
    let c = caso(paths, args)?;
    let perd = report_pdf::perdidas(&c.tokens)?;
    let mens = report_pdf::mensual(&c.tokens)?;
    let resumen = report_pdf::resumen(&c.tokens)?;
    let (sm, fuente, aviso) = sigma_meteo(&c, args)?;
    let p50 = incertidumbre(c.energia_kwh, sm, sigma_modelo(args)?, &fuente);
    let carbono = carbono_con(&c, args);
    let econ = if num(args, "capex_por_kwp").is_some() || num(args, "tarifa_por_kwh").is_some() {
        let p = parse_economia(args)?;
        let kwp = c.kwp.ok_or_else(sin_potencia)?;
        Some(economia_de(c.energia_kwh, kwp, &p))
    } else {
        None
    };
    let texto = markdown(&c, &perd, &mens, &resumen, &p50, &carbono, econ.as_ref());
    let dir = c.workspace.join("Results");
    fs::create_dir_all(&dir)
        .map_err(|e| Error::io(format!("No se pudo crear {}", dir.display()), e))?;
    let destino = dir.join(format!("{}_{}_entregable.md", c.proyecto, c.variante));
    fs::write(&destino, texto)
        .map_err(|e| Error::io(format!("No se pudo escribir {}", destino.display()), e))?;
    let mut v = con_cabecera(
        &c,
        json!({"archivo": destino.display().to_string(), "p50_p90": p50}),
    );
    if let Some(a) = aviso {
        v["aviso"] = json!(a);
    }
    Ok(v)
}
