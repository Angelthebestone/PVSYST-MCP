//! Locates and loads the CSV/PDF pair for one simulation case.

use crate::args::{opt_str, req_str, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::object::parse_first;
use crate::projects::resolve_project_file;
use crate::simulation::report_pdf;
use crate::simulation::results::resultado_de;
use crate::workspace::leer_bytes;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// One simulation result: CSV, PDF report and the figures read from both.
pub(crate) struct Caso {
    pub(crate) workspace: PathBuf,
    pub(crate) proyecto: String,
    pub(crate) variante: String,
    pub(crate) pdf: PathBuf,
    pub(crate) tokens: Vec<String>,
    pub(crate) energia_kwh: f64,
    pub(crate) kwp: Option<f64>,
    /// Site read from the project; the failure is kept as text so the case stays `Clone`-free.
    pub(crate) prj: Result<Sitio, String>,
}

impl Caso {
    pub(crate) fn sitio(&self) -> Result<&Sitio> {
        self.prj.as_ref().map_err(|m| Error::Format(m.clone()))
    }
}

/// Site data read from the project's `SitePrj` (searched through the whole tree).
pub(crate) struct Sitio {
    pub(crate) lat: Option<f64>,
    pub(crate) lon: Option<f64>,
}

pub(crate) fn leer_sitio(workspace: &Path, proyecto: &str) -> Result<Sitio> {
    let prj = resolve_project_file(workspace, proyecto)?;
    let texto = String::from_utf8_lossy(&leer_bytes(&prj)?).into_owned();
    let raiz = parse_first(&texto)
        .ok_or_else(|| Error::Format(format!("PRJ sin objeto raíz: {}", prj.display())))?;
    Ok(Sitio {
        lat: raiz
            .find_deep("Latitude")
            .and_then(|v| v.trim().parse().ok()),
        lon: raiz
            .find_deep("Longitude")
            .and_then(|v| v.trim().parse().ok()),
    })
}

/// Country PVsyst writes in the `Geographical Site;<file>;<date>;<name>;<country>;…` line.
#[allow(dead_code)]
pub(crate) fn pais_simulado(texto: &str) -> Option<String> {
    texto
        .lines()
        .map(|l| l.trim_start_matches('\u{feff}'))
        .find(|l| l.starts_with("Geographical Site;"))?
        .split(';')
        .nth(4)
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
}

pub(crate) fn round1(x: f64) -> f64 {
    crate::format::redondear(x, 1)
}

pub(crate) fn especifica(c: &Caso) -> Option<f64> {
    c.kwp.map(|k| round1(c.energia_kwh / k))
}

/// Add the common identification and energy fields to a result object.
pub(crate) fn con_cabecera(c: &Caso, mut v: Value) -> Value {
    if let Some(o) = v.as_object_mut() {
        o.insert("proyecto".into(), json!(c.proyecto));
        o.insert("variante".into(), json!(c.variante));
        o.insert("pdf".into(), json!(c.pdf.display().to_string()));
        o.insert("energia_kwh".into(), json!(c.energia_kwh));
        o.insert("kwp".into(), json!(c.kwp));
        o.insert("produccion_especifica_kwh_kwp".into(), json!(especifica(c)));
    }
    v
}

/// Resolve the result files and read the PDF figures for one project/variant.
pub(crate) fn caso(paths: &Paths, args: &Value) -> Result<Caso> {
    let workspace = workspace_of(paths, args);
    let proyecto = req_str(args, &["proyecto", "project"], "proyecto")?.to_string();
    let variante_pedida = opt_str(args, &["variante", "variant"]);
    let archivo = opt_str(args, &["archivo", "file"]);
    let (csv, pdf) = resultado_de(&workspace, Some(&proyecto), variante_pedida, archivo)?;
    let pdf = pdf.ok_or_else(|| {
        Error::NotFound(format!(
            "No hay informe PDF para {} (ejecute la simulación con -rpf)",
            csv.display()
        ))
    })?;
    let tokens = report_pdf::tokens(&leer_bytes(&pdf)?);
    let resumen = report_pdf::resumen(&tokens)?;
    let energia_kwh = resumen["energia_anual_kwh"]
        .as_f64()
        .ok_or_else(|| Error::Format("El informe PDF no incluye la energía anual".into()))?;
    let csv_texto = String::from_utf8_lossy(&leer_bytes(&csv)?).into_owned();
    let prj = leer_sitio(&workspace, &proyecto).map_err(|e| e.to_string());
    let variante = variante_pedida.map_or_else(
        || {
            csv.file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| s.rsplit_once('_'))
                .map_or_else(|| "VC0".to_string(), |(_, v)| v.to_string())
        },
        str::to_string,
    );
    let kwp = args
        .get("kwp")
        .and_then(|v| v.as_f64().or_else(|| v.as_str()?.trim().parse().ok()))
        .or_else(|| {
            let metadata = json!(csv_texto.lines().take(40).collect::<Vec<_>>());
            crate::simulation::results::variant_kwp(&workspace, &metadata)
        });
    Ok(Caso {
        workspace,
        proyecto,
        variante,
        pdf,
        tokens,
        energia_kwh,
        kwp,
        prj,
    })
}
