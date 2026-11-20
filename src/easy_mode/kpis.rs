//! Result KPIs and reports: locating the result pair and reading its figures.

use crate::args::{opt_str, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::simulation::results;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

/// Weather file named in a result CSV banner (`Weather data;<file>;...`).
pub(super) fn meteo_de_metadatos(texto: &str) -> Option<String> {
    texto.lines().find_map(|linea| {
        let mut campos = linea.split(';');
        if campos.next()?.trim() != "Weather data" {
            return None;
        }
        campos
            .next()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    })
}

fn buscar_resultado_csv(ws: &Path, args: &Value) -> Option<PathBuf> {
    let prj = args.get("proyecto").and_then(Value::as_str);
    let var = args.get("variante").and_then(Value::as_str);
    let arch = args.get("archivo").and_then(Value::as_str);
    results::resultado_de(ws, prj, var, arch)
        .ok()
        .map(|(p, _)| p)
}

fn resolver_ruta_pdf(paths: &Paths, args: &Value) -> Result<PathBuf> {
    let ws = workspace_of(paths, args);
    if let Some(p) = opt_str(args, &["report_pdf"]) {
        let ruta = PathBuf::from(p);
        let abs = if ruta.is_absolute() {
            ruta
        } else {
            ws.join(ruta)
        };
        if abs.exists() {
            return Ok(abs);
        }
        return Err(Error::NotFound(format!(
            "Informe PDF no encontrado en '{}'",
            abs.display()
        )));
    }
    if let Some(archivo) = opt_str(args, &["archivo"]) {
        let p = PathBuf::from(archivo);
        let abs = if p.is_absolute() { p } else { ws.join(p) };
        if abs
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("pdf"))
            && abs.exists()
        {
            return Ok(abs);
        }
        let con_pdf = abs.with_extension("pdf");
        if con_pdf.exists() {
            return Ok(con_pdf);
        }
    }
    if let Some(csv_path) = buscar_resultado_csv(&ws, args) {
        let con_pdf = csv_path.with_extension("pdf");
        if con_pdf.exists() {
            return Ok(con_pdf);
        }
    }
    Err(Error::NotFound(
        "Informe PDF de simulación no encontrado: asegúrate de simular con report_pdf (-rpf)"
            .into(),
    ))
}

/// The result report already read: where it is and the tokens extracted from it.
#[derive(Clone, Copy)]
pub(crate) struct InformeLeido<'a> {
    pub(crate) workspace: &'a Path,
    pub(crate) pdf: &'a Path,
    pub(crate) tokens: &'a [String],
}

/// Read headline KPIs from the simulation PDF report.
pub(crate) fn leer_kpis(paths: &Paths, args: &Value) -> Result<Value> {
    let pdf = resolver_ruta_pdf(paths, args)?;
    let bytes = crate::workspace::leer_bytes(&pdf)?;
    let tokens = crate::simulation::report_pdf::tokens(&bytes);
    let workspace = workspace_of(paths, args);
    kpis_de(
        InformeLeido {
            workspace: &workspace,
            pdf: &pdf,
            tokens: &tokens,
        },
        args,
    )
}

/// Headline KPIs from a report whose tokens are already extracted.
pub(crate) fn kpis_de(informe: InformeLeido<'_>, args: &Value) -> Result<Value> {
    let resumen = crate::simulation::report_pdf::resumen(informe.tokens)?;

    let mut out = Map::new();
    out.insert("pdf".into(), json!(informe.pdf.to_string_lossy()));

    if let Some(e) = resumen.get("energia_anual_kwh") {
        out.insert("energia_anual_kwh".into(), e.clone());
    }
    if let Some(s) = resumen.get("produccion_especifica_kwh_kwp") {
        out.insert("produccion_especifica_kwh_kwp".into(), s.clone());
    }
    if let Some(pr) = resumen.get("pr_pct") {
        out.insert("pr_pct".into(), pr.clone());
    }

    let meteo = csv_de_resultado(informe.workspace, args)
        .and_then(|csv| std::fs::read_to_string(csv).ok())
        .and_then(|texto| meteo_de_metadatos(&texto));
    if let Some(meteo) = meteo {
        out.insert("weather_file".into(), json!(meteo));
    }

    let kwp = opt_str(args, &["kwp"])
        .and_then(|s| s.trim().parse::<f64>().ok())
        .or_else(|| args.get("kwp").and_then(Value::as_f64));
    if let Some(k) = kwp {
        out.insert("kwp".into(), json!(k));
    }

    let mut leidos = Value::Object(out);
    // A saved result can be older than the configuration it is read with: its own PR
    // is the only figure that says whether it can still be used.
    let pr = leidos.get("pr_pct").and_then(Value::as_f64);
    crate::simulation::coherencia::anotar(&mut leidos, pr);
    Ok(leidos)
}

/// The result CSV named by `archivo`/`output_csv`/`csv`, else the latest one
/// matching the project and variant.
fn csv_de_resultado(ws: &Path, args: &Value) -> Option<PathBuf> {
    let explicito = opt_str(args, &["archivo"]).map(|archivo| {
        let p = PathBuf::from(archivo);
        if p.is_absolute() {
            p
        } else {
            ws.join(p)
        }
    });
    explicito
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("csv")) && p.exists())
        .or_else(|| buscar_resultado_csv(ws, args).filter(|p| p.exists()))
}
