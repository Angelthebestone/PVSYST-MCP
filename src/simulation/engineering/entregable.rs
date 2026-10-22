//! Builds the downloadable deliverable from one simulation case.

use super::caso::{caso, con_cabecera, Caso};
use super::incertidumbre::{incertidumbre, sigma_meteo, sigma_modelo};
use crate::aproximaciones::{marcar, Aproximacion};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::simulation::report_pdf;
use serde_json::{json, Value};
use std::fs;

const AP_P50: Aproximacion = Aproximacion {
    funcion: "resultados p50_p90",
    alternativa:
        "configurar la evaluación P50-P90 en la variante y simular con la página de informe p50p90",
};

pub(super) fn tabla_mensual(mens: &Value) -> Vec<String> {
    let cols: Vec<&str> = mens["columnas"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let mut lineas = vec![
        format!("| Mes | {} |", cols.join(" | ")),
        format!("| --- | {} |", vec!["---"; cols.len()].join(" | ")),
    ];
    for m in mens["meses"].as_array().into_iter().flatten() {
        let mes = m["mes"].as_str().unwrap_or("");
        let fila: Vec<String> = cols.iter().map(|c| m["valores"][c].to_string()).collect();
        lineas.push(format!("| {mes} | {} |", fila.join(" | ")));
    }
    lineas
}

pub(super) fn seccion_resultados(c: &Caso, resumen: &Value) -> Vec<String> {
    vec![
        format!("# Informe de simulación: {} ({})", c.proyecto, c.variante),
        String::new(),
        "## Resumen".to_string(),
        format!(
            "- Energía producida anual: {} kWh",
            resumen["energia_anual_kwh"]
        ),
        format!(
            "- Producción específica: {} kWh/kWp",
            resumen["produccion_especifica_kwh_kwp"]
        ),
        format!("- Performance Ratio: {} %", resumen["pr_pct"]),
    ]
}

fn seccion_carbono(carb: &Value) -> Vec<String> {
    let mut l = vec![String::new(), "## Balance de carbono".to_string()];
    let unit_ev = carb["emisiones_evitadas_unidad"].as_str().unwrap_or("tCO2");
    let unit_cv = carb["emisiones_ciclo_vida_unidad"]
        .as_str()
        .unwrap_or("tCO2");
    let neto = carb
        .get("balance_neto_total")
        .map_or("-".to_string(), |v| v.to_string());
    let unit_neto = carb["balance_neto_unidad"].as_str().unwrap_or("tCO2");
    l.push(format!(
        "- Emisiones evitadas: {} {unit_ev}; ciclo de vida (LCE): {} {unit_cv}; balance neto: {neto} {unit_neto}",
        carb["emisiones_evitadas_total"], carb["emisiones_ciclo_vida_total"]
    ));
    if carb.get("factor_lce_red").is_some() {
        let unit_f = carb["factor_lce_red_unidad"].as_str().unwrap_or("gCO2/kWh");
        let fuente = carb["fuente"].as_str().unwrap_or("");
        let pais = carb["pais"].as_str().unwrap_or("");
        l.push(format!(
            "- Factor de red: {} {unit_f} ({fuente}, {pais}); vida útil: {} años; degradación: {} %",
            carb["factor_lce_red"], carb["vida_util_anios"], carb["degradacion_anual_pct"]
        ));
    }
    l
}

fn seccion_economia(econ: &Value) -> Vec<String> {
    let mut l = vec![String::new(), "## Evaluación económica".to_string()];
    let moneda = econ["moneda"].as_str().unwrap_or("");
    let unit_esp = econ["unidad_especifica"].as_str().unwrap_or("");
    l.push(format!(
        "- Inversión global: {} {moneda} (específica: {} {unit_esp})",
        econ["inversion_global"], econ["inversion_especifica"]
    ));
    let u_an = econ["anualidades_unidad"].as_str().unwrap_or("");
    let u_op = econ["costes_operacion_unidad"].as_str().unwrap_or("");
    l.push(format!(
        "- Costes anuales: anualidades {} {u_an}, operación {} {u_op}",
        econ["anualidades"], econ["costes_operacion"]
    ));
    if let Some(pb) = econ.get("payback_anios").filter(|v| !v.is_null()) {
        l.push(format!("- Período de retorno: {pb} años"));
    }
    let u_lcoe = econ["lcoe_unidad"].as_str().unwrap_or("");
    l.push(format!("- LCOE: {} {u_lcoe}", econ["lcoe"]));
    l
}

fn seccion_p50(p50: &Value) -> Vec<String> {
    vec![
        String::new(),
        "## P50 / P90".to_string(),
        "> *Nota: Esta sección es una aproximación del MCP (no proviene de PVsyst).*".to_string(),
        format!(
            "- P50: {} kWh; P90: {} kWh; P99: {} kWh (σ total {} %)",
            p50["p50_kwh"], p50["p90_kwh"], p50["p99_kwh"], p50["sigma_total_pct"]
        ),
    ]
}

pub(super) fn markdown(
    c: &Caso,
    perd: &Value,
    mens: &Value,
    resumen: &Value,
    carbono: Option<&Value>,
    econ: Option<&Value>,
    p50: Option<&Value>,
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
    if let Some(carb) = carbono {
        l.extend(seccion_carbono(carb));
    }
    if let Some(e) = econ {
        l.extend(seccion_economia(e));
    }
    if let Some(p) = p50 {
        l.extend(seccion_p50(p));
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

fn calcular_p50(paths: &Paths, c: &Caso, args: &Value) -> Result<(Option<Value>, Option<String>)> {
    if !paths.aproximaciones {
        return Ok((None, None));
    }
    let (sm, fuente, aviso) = sigma_meteo(c, args)?;
    let mut p50 = incertidumbre(c.energia_kwh, sm, sigma_modelo(args)?, &fuente);
    marcar(&mut p50, &AP_P50);
    Ok((Some(p50), aviso))
}

pub(crate) fn entregable(paths: &Paths, args: &Value) -> Result<Value> {
    let c = caso(paths, args)?;
    let perd = report_pdf::perdidas(&c.tokens)?;
    let mens = report_pdf::mensual(&c.tokens)?;
    let resumen = report_pdf::resumen(&c.tokens)?;
    let carbono = report_pdf::carbono(&c.tokens).ok();
    let economia = report_pdf::economia(&c.tokens).ok();
    let (p50, aviso) = calcular_p50(paths, &c, args)?;
    let texto = markdown(
        &c,
        &perd,
        &mens,
        &resumen,
        carbono.as_ref(),
        economia.as_ref(),
        p50.as_ref(),
    );
    let dir = c.workspace.join("Results");
    fs::create_dir_all(&dir)
        .map_err(|e| Error::io(format!("No se pudo crear {}", dir.display()), e))?;
    let destino = dir.join(format!("{}_{}_entregable.md", c.proyecto, c.variante));
    let historial_id = crate::history::reemplazar(
        &c.workspace,
        &destino,
        texto.as_bytes(),
        "resultados entregable",
    )?;
    let mut payload = json!({
        "archivo": destino.display().to_string(),
        "historial_id": historial_id,
    });
    if let Some(p) = p50 {
        payload["p50_p90"] = p;
    }
    if let Some(a) = aviso {
        payload["aviso"] = json!(a);
    }
    Ok(con_cabecera(&c, payload))
}
