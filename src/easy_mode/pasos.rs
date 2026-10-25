//! Step output data structures and run summary for the easy-mode pipeline.

use serde::Serialize;
use serde_json::{json, Value};

/// Whether the site step reused an existing site or created a new one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OrigenSitioDiseno {
    Reutilizado,
    Nuevo,
}

/// Outcome of the simulation step, as seen by the post-processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EstadoSimulacion {
    Correcta,
    Fallida,
}

/// Output of step 1: site creation or reuse.
#[derive(Debug)]
pub(crate) struct PasoSitioSalida {
    pub(crate) sit_file: String,
    pub(crate) site_name: Option<String>,
    pub(crate) weather_source: Option<String>,
    pub(crate) annual_ghi_kwh_m2: Option<f64>,
    pub(crate) reutilizado: bool,
    pub(crate) ok: bool,
}

/// Output of step 2: project creation from template.
#[derive(Debug)]
pub(crate) struct PasoProyectoSalida {
    pub(crate) nombre: String,
    pub(crate) ok: bool,
    pub(crate) raw_response: Value,
}

/// Output of step 3: fixed-plane orientation update.
#[derive(Debug)]
pub(crate) struct PasoOrientacionSalida {
    pub(crate) tilt: f64,
    pub(crate) azimut: f64,
    pub(crate) ok: bool,
    pub(crate) detalle: Value,
}

/// Output of step 4: array electrical sizing.
#[derive(Debug)]
pub(crate) struct PasoDimensionadoSalida {
    pub(crate) kwp_realizado: Option<f64>,
    pub(crate) criterio: &'static str,
    pub(crate) avisos: Vec<String>,
    pub(crate) ok: bool,
    pub(crate) detalle: Value,
}

/// Output of step 5: simulation execution and result collection.
#[derive(Debug)]
pub(crate) struct PasoSimulacionSalida {
    pub(crate) sim_info: Value,
    pub(crate) kpis: Option<KpisSalida>,
    pub(crate) kpis_error: Option<String>,
    pub(crate) csv_path: String,
    pub(crate) pdf_path: String,
    pub(crate) sim_ok: bool,
    pub(crate) sitio_proyecto_ok: Option<bool>,
    pub(crate) sitio_proyecto_detalle: Option<Value>,
    /// Warnings PVsyst logged during the run (`avisos_pvsyst` of `simular`).
    pub(crate) avisos_pvsyst: Vec<String>,
}

/// Key performance indicators extracted from simulation outputs.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct KpisSalida {
    pub(crate) csv: Value,
    /// PVsyst's "Produced Energy" (kWh/year) from the report summary.
    pub(crate) energia_producida_kwh: Option<f64>,
    pub(crate) produccion_especifica_kwh_kwp: Option<f64>,
    pub(crate) pr_pct: Option<f64>,
    pub(crate) kwp: Option<f64>,
}

/// Complete typed state of an end-to-end execution run.
#[derive(Debug)]
pub(crate) struct Corrida {
    pub(crate) sitio: PasoSitioSalida,
    pub(crate) proyecto: PasoProyectoSalida,
    pub(crate) orientacion: PasoOrientacionSalida,
    pub(crate) dimensionado: PasoDimensionadoSalida,
    pub(crate) simulacion: Option<PasoSimulacionSalida>,
}

impl Corrida {
    /// Formats step summaries as JSON objects for tool output.
    pub(crate) fn a_pasos_json(&self) -> Vec<Value> {
        let mut pasos = Vec::with_capacity(6);
        pasos.push(json!({
            "paso": "sitio",
            "ok": self.sitio.ok,
            "sit_file": self.sitio.sit_file,
        }));
        pasos.push(json!({
            "paso": "proyecto",
            "ok": self.proyecto.ok,
            "proyecto": self.proyecto.nombre,
        }));
        pasos.push(json!({
            "paso": "orientacion",
            "ok": self.orientacion.ok,
            "tilt": self.orientacion.tilt,
            "azimut": self.orientacion.azimut,
            "detail": self.orientacion.detalle,
        }));
        pasos.push(json!({
            "paso": "dimensionado",
            "ok": self.dimensionado.ok,
            "criterio": self.dimensionado.criterio,
            "avisos": self.dimensionado.avisos,
            "detail": self.dimensionado.detalle,
        }));
        if let Some(sim) = &self.simulacion {
            pasos.push(json!({
                "paso": "simulacion",
                "ok": sim.sim_ok,
                "csv": sim.csv_path,
                "pdf": sim.pdf_path,
            }));
            if sim.sim_ok {
                pasos.push(json!({
                    "paso": "sitio_proyecto",
                    "ok": sim.sitio_proyecto_ok.unwrap_or(false),
                    "detail": sim.sitio_proyecto_detalle,
                }));
            }
        }
        pasos
    }
}

/// Which CLI-backed phases the end-to-end run will execute.
pub(crate) struct Fases {
    pub(crate) crear_sitio: bool,
    pub(crate) simular: bool,
}
