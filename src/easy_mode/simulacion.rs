//! Step 5: Simulation execution, result collection and KPI extraction.

use super::parametros::DisenoParametros;
use super::pasos::{EstadoSimulacion, KpisSalida, PasoSimulacionSalida};
use super::proyecto::sustituir_sitio;
use crate::args::{opt_str, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::simulation::{self, results};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

const SIMULATION_LOG_LEVEL: u32 = 3;
/// Weather file named in a result CSV banner (`Weather data;<file>;...`).
pub(crate) fn meteo_de_metadatos(texto: &str) -> Option<String> {
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

/// Read headline KPIs from the simulation PDF report.
pub(crate) fn leer_kpis(paths: &Paths, args: &Value) -> Result<Value> {
    let pdf_path = resolver_ruta_pdf(paths, args)?;
    let bytes = crate::workspace::leer_bytes(&pdf_path)?;
    let tokens = crate::simulation::report_pdf::tokens(&bytes);
    let resumen = crate::simulation::report_pdf::resumen(&tokens)?;

    let mut out = Map::new();
    out.insert("pdf".into(), json!(pdf_path.to_string_lossy()));

    if let Some(e) = resumen.get("energia_anual_kwh") {
        out.insert("energia_anual_kwh".into(), e.clone());
    }
    if let Some(s) = resumen.get("produccion_especifica_kwh_kwp") {
        out.insert("specific_yield_kwh_per_kwp".into(), s.clone());
        out.insert("produccion_especifica_kwh_kwp".into(), s.clone());
    }
    if let Some(pr) = resumen.get("pr_pct") {
        out.insert("pr".into(), pr.clone());
        out.insert("pr_pct".into(), pr.clone());
    }

    let ws = workspace_of(paths, args);
    let meteo = csv_de_resultado(&ws, args)
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

    Ok(Value::Object(out))
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

type SitioKpisResultado = (
    Option<bool>,
    Option<Value>,
    Option<KpisSalida>,
    Option<String>,
);

struct ContextoActualizacionSitio<'a> {
    nombre_proyecto: &'a str,
    sit_file: &'a str,
    csv_out: &'a str,
    pdf_out: &'a str,
    estado: EstadoSimulacion,
}

fn actualizar_sitio_y_kpis(
    paths: &Paths,
    ctx: &ContextoActualizacionSitio<'_>,
) -> SitioKpisResultado {
    let mut ok_sitio = None;
    let mut detalle_sitio = None;
    if matches!(ctx.estado, EstadoSimulacion::Correcta) {
        let sitio = sustituir_sitio(
            paths,
            ctx.nombre_proyecto,
            ctx.sit_file,
            Path::new(ctx.csv_out),
        );
        let (s_ok, det) = match sitio {
            Ok(v) => (true, v),
            Err(e) => (false, json!(e)),
        };
        ok_sitio = Some(s_ok);
        detalle_sitio = Some(det);
    }
    let (kpis, kpis_error) = match extraer_kpis(paths, ctx.csv_out, ctx.pdf_out) {
        Ok(k) => (Some(k), None),
        Err(e) => (None, Some(e.to_string())),
    };
    (ok_sitio, detalle_sitio, kpis, kpis_error)
}

/// Helper that converts `leer_kpis` JSON result into `KpisSalida`.
pub(crate) fn extraer_kpis(paths: &Paths, csv_path: &str, pdf_path: &str) -> Result<KpisSalida> {
    let k_val = leer_kpis(
        paths,
        &json!({ "archivo": csv_path, "report_pdf": pdf_path }),
    )?;
    Ok(KpisSalida {
        csv: k_val.get("csv").cloned().unwrap_or(json!(csv_path)),
        energia_producida_kwh: k_val.get("energia_anual_kwh").and_then(Value::as_f64),
        specific_yield_kwh_per_kwp: k_val
            .get("specific_yield_kwh_per_kwp")
            .and_then(Value::as_f64),
        pr: k_val.get("pr").and_then(Value::as_f64),
        kwp: k_val.get("kwp").and_then(Value::as_f64),
    })
}

fn rutas_salida_simulacion(
    workspace: &Path,
    params: &DisenoParametros,
) -> Result<(String, String)> {
    let results_dir = workspace.join("Results");
    std::fs::create_dir_all(&results_dir)
        .map_err(|e| Error::io(format!("crear directorio '{}'", results_dir.display()), e))?;
    let nombre = &params.proyecto.nombre;
    let csv = params.simulacion.output_csv.clone().unwrap_or_else(|| {
        results_dir
            .join(format!("{nombre}.csv"))
            .to_string_lossy()
            .into_owned()
    });
    let pdf = params.simulacion.report_pdf.clone().unwrap_or_else(|| {
        results_dir
            .join(format!("{nombre}.pdf"))
            .to_string_lossy()
            .into_owned()
    });
    Ok((csv, pdf))
}

/// Step 5: simulate, write the site into the project and read the KPIs.
pub(crate) fn ejecutar_paso_simulacion(
    paths: &Paths,
    params: &DisenoParametros,
    sit_file: &str,
) -> Result<PasoSimulacionSalida> {
    let nombre_proyecto = &params.proyecto.nombre;
    let (csv_out, pdf_out) = rutas_salida_simulacion(&paths.workspace, params)?;
    let sv = json!({
        "proyecto": nombre_proyecto,
        "variante": params.variante,
        "site": sit_file,
        "synthetic_weather": params.simulacion.synthetic_weather,
        "output_csv": csv_out,
        "report_pdf": pdf_out,
        "log_level": SIMULATION_LOG_LEVEL,
    });
    let sim = simulation::lanzar_simulacion(paths, &sv)?;
    let ok = sim.get("success").and_then(Value::as_bool).unwrap_or(false);

    let estado = if ok {
        EstadoSimulacion::Correcta
    } else {
        EstadoSimulacion::Fallida
    };
    let ctx_act = ContextoActualizacionSitio {
        nombre_proyecto,
        sit_file,
        csv_out: &csv_out,
        pdf_out: &pdf_out,
        estado,
    };
    let (ok_sitio, detalle_sitio, kpis, kpis_error) = actualizar_sitio_y_kpis(paths, &ctx_act);
    let avisos_pvsyst = avisos_de_simulacion(&sim);
    Ok(PasoSimulacionSalida {
        sim_info: sim.get("stdout_tail").cloned().unwrap_or(Value::Null),
        avisos_pvsyst,
        kpis,
        kpis_error,
        csv_path: csv_out,
        pdf_path: pdf_out,
        sim_ok: ok,
        sitio_proyecto_ok: ok_sitio,
        sitio_proyecto_detalle: detalle_sitio,
    })
}

/// `avisos_pvsyst` of a `simular` result, prefixed so they read as PVsyst's.
fn avisos_de_simulacion(sim: &Value) -> Vec<String> {
    sim.get("avisos_pvsyst")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|a| format!("PVsyst: {a}"))
        .collect()
}
