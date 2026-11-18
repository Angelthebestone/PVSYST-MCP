//! Step 5: Simulation execution, result collection and KPI extraction.

use super::kpis::leer_kpis;
use super::parametros::DisenoParametros;
use super::pasos::{EstadoSimulacion, KpisSalida, PasoSimulacionSalida};
use super::sitio_del_proyecto::{sustituir_sitio, SitioDelDiseno};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::simulation;
use serde_json::{json, Value};
use std::path::Path;

const SIMULATION_LOG_LEVEL: u32 = 3;

type SitioKpisResultado = (
    Option<bool>,
    Option<Value>,
    Option<KpisSalida>,
    Option<String>,
);

struct ContextoActualizacionSitio<'a> {
    nombre_proyecto: &'a str,
    variante: &'a str,
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
            &SitioDelDiseno {
                nombre: ctx.nombre_proyecto,
                variante: ctx.variante,
                sit_file: ctx.sit_file,
            },
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
    // `leer_kpis` reads the PDF only: the CSV is the one given, and the kWp is the
    // design's `kwp_realizado`, not a KPI of the report.
    Ok(KpisSalida {
        csv: json!(csv_path),
        energia_producida_kwh: k_val.get("energia_anual_kwh").and_then(Value::as_f64),
        produccion_especifica_kwh_kwp: k_val
            .get("produccion_especifica_kwh_kwp")
            .and_then(Value::as_f64),
        pr_pct: k_val.get("pr_pct").and_then(Value::as_f64),
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
        variante: &params.variante,
        sit_file,
        csv_out: &csv_out,
        pdf_out: &pdf_out,
        estado,
    };
    let (ok_sitio, detalle_sitio, kpis, kpis_error) = actualizar_sitio_y_kpis(paths, &ctx_act);
    let avisos_pvsyst = avisos_de_simulacion(&sim);
    let detalle_error = (!ok)
        .then(|| super::fallo_simulacion::detalle(&sim, paths.idioma))
        .flatten();
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
        detalle_error,
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
