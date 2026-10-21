//! Step 5: Simulation execution, result collection and KPI extraction.

use super::parametros::DisenoParametros;
use super::pasos::{EstadoSimulacion, KpisSalida, PasoSimulacionSalida};
use super::proyecto::sustituir_sitio;
use crate::args::{opt_str, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::redondear;
use crate::simulation::{self, results};
use serde_json::{json, Map, Value};
use std::path::Path;

const SIMULATION_LOG_LEVEL: u32 = 3;
const WH_POR_KWH: f64 = 1000.0;
const DECIMALES_ENERGIA: i32 = 1;
const DECIMALES_PR: i32 = 3;

/// Column sum of the first `totals` entry whose name contains `needle` (case-insensitive).
pub(crate) fn total_de(totals: &Value, needle: &str) -> Option<f64> {
    let needle = needle.to_lowercase();
    totals
        .as_object()?
        .iter()
        .find(|(k, _)| k.to_lowercase().contains(&needle))
        .and_then(|(_, v)| v.get("sum").and_then(Value::as_f64))
}

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

/// Read headline KPIs from a result CSV.
fn calcular_metricas_kpi(
    out: &mut Map<String, Value>,
    e_grid_wh: Option<f64>,
    poa: Option<f64>,
    kwp: Option<f64>,
) {
    if let Some(e) = e_grid_wh {
        out.insert(
            "e_grid_kwh".into(),
            json!(redondear(e / WH_POR_KWH, DECIMALES_ENERGIA)),
        );
    }
    if let Some(p) = poa {
        out.insert(
            "poa_kwh_m2".into(),
            json!(redondear(p / WH_POR_KWH, DECIMALES_ENERGIA)),
        );
    }
    if let (Some(e), Some(k)) = (e_grid_wh, kwp) {
        let ekwh = e / WH_POR_KWH;
        out.insert(
            "specific_yield_kwh_per_kwp".into(),
            json!(redondear(ekwh / k, DECIMALES_ENERGIA)),
        );
        if let Some(p) = poa.filter(|p| *p > 0.0) {
            let poa_kwh = p / WH_POR_KWH;
            out.insert(
                "pr".into(),
                json!(redondear(ekwh / (poa_kwh * k), DECIMALES_PR)),
            );
        }
    }
    if let Some(k) = kwp {
        out.insert("kwp".into(), json!(k));
    }
}

/// Read headline KPIs from a result CSV.
pub(crate) fn leer_kpis(paths: &Paths, args: &Value) -> Result<Value> {
    let res = results::leer_resultados(paths, args)?;
    let entry = res
        .get("results")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first());
    let summary = entry.and_then(|e| e.get("summary"));
    let totals = summary
        .and_then(|s| s.get("totals"))
        .or_else(|| res.get("totals"))
        .cloned()
        .unwrap_or(json!({}));
    let metadata = summary.and_then(|s| s.get("metadata"));
    let kwp = opt_str(args, &["kwp", "potencia_kwp"])
        .and_then(|s| s.trim().parse::<f64>().ok())
        .or_else(|| args.get("kwp").and_then(Value::as_f64))
        .or_else(|| metadata.and_then(|m| results::variant_kwp(&workspace_of(paths, args), m)));

    let mut out = Map::new();
    let csv_path = entry
        .and_then(|e| e.get("path"))
        .or_else(|| res.get("path"))
        .cloned()
        .unwrap_or(json!(null));
    out.insert("csv".into(), csv_path);
    calcular_metricas_kpi(
        &mut out,
        total_de(&totals, "E_Grid"),
        total_de(&totals, "GlobInc"),
        kwp,
    );
    Ok(Value::Object(out))
}

type SitioKpisResultado = (
    Option<bool>,
    Option<Value>,
    Option<KpisSalida>,
    Option<String>,
);

fn actualizar_sitio_y_kpis(
    paths: &Paths,
    nombre_proyecto: &str,
    sit_file: &str,
    csv_out: &str,
    estado: EstadoSimulacion,
) -> SitioKpisResultado {
    let mut ok_sitio = None;
    let mut detalle_sitio = None;
    if matches!(estado, EstadoSimulacion::Correcta) {
        let sitio = sustituir_sitio(paths, nombre_proyecto, sit_file, Path::new(csv_out));
        let (s_ok, det) = match sitio {
            Ok(v) => (true, v),
            Err(e) => (false, json!(e)),
        };
        ok_sitio = Some(s_ok);
        detalle_sitio = Some(det);
    }
    let (kpis, kpis_error) = match extraer_kpis(paths, csv_out) {
        Ok(k) => (Some(k), None),
        Err(e) => (None, Some(e.to_string())),
    };
    (ok_sitio, detalle_sitio, kpis, kpis_error)
}

/// Helper that converts `leer_kpis` JSON result into `KpisSalida`.
pub(crate) fn extraer_kpis(paths: &Paths, csv_path: &str) -> Result<KpisSalida> {
    let k_val = leer_kpis(paths, &json!({ "archivo": csv_path }))?;
    Ok(KpisSalida {
        csv: k_val.get("csv").cloned().unwrap_or(json!(csv_path)),
        e_grid_kwh: k_val.get("e_grid_kwh").and_then(Value::as_f64),
        poa_kwh_m2: k_val.get("poa_kwh_m2").and_then(Value::as_f64),
        specific_yield_kwh_per_kwp: k_val
            .get("specific_yield_kwh_per_kwp")
            .and_then(Value::as_f64),
        pr: k_val.get("pr").and_then(Value::as_f64),
        kwp: k_val.get("kwp").and_then(Value::as_f64),
    })
}

/// Step 5: simulate, write the site into the project and read the KPIs.
pub(crate) fn ejecutar_paso_simulacion(
    paths: &Paths,
    params: &DisenoParametros,
    sit_file: &str,
) -> Result<PasoSimulacionSalida> {
    let nombre_proyecto = &params.proyecto.nombre;
    let results_dir = paths.workspace.join("Results");
    std::fs::create_dir_all(&results_dir)
        .map_err(|e| Error::io(format!("crear directorio '{}'", results_dir.display()), e))?;

    let csv_out = params.simulacion.output_csv.clone().unwrap_or_else(|| {
        results_dir
            .join(format!("{nombre_proyecto}.csv"))
            .to_string_lossy()
            .to_string()
    });
    let pdf_out = params.simulacion.report_pdf.clone().unwrap_or_else(|| {
        results_dir
            .join(format!("{nombre_proyecto}.pdf"))
            .to_string_lossy()
            .to_string()
    });

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
    let (ok_sitio, detalle_sitio, kpis, kpis_error) =
        actualizar_sitio_y_kpis(paths, nombre_proyecto, sit_file, &csv_out, estado);
    Ok(PasoSimulacionSalida {
        sim_info: sim.get("stdout_tail").cloned().unwrap_or(Value::Null),
        kpis,
        kpis_error,
        csv_path: csv_out,
        pdf_path: pdf_out,
        sim_ok: ok,
        sitio_proyecto_ok: ok_sitio,
        sitio_proyecto_detalle: detalle_sitio,
    })
}
