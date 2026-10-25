//! Tool for simulation output and engineering analysis: KPIs, CSV, losses, economics.

use super::declaracion::{Anotaciones, Discriminador, Ejecutor, Herramienta, Sinonimos, TipoDato};
use crate::args::workspace_of;
use crate::config::Paths;
use crate::easy_mode;
use crate::error::{Error, Result};
use crate::projects;
use crate::simulation::{engineering, results};
use serde_json::{json, Value};

pub(crate) const TOOL_RESULTADOS: Herramienta = Herramienta {
    nombre: "resultados",
    descripcion: "Resultados: KPIs, resumen, CSV, perdidas, mensual, comparacion, medido_vs_simulado, P50/P90, carbono, economia y entregable. / EN: Results: KPIs, summary, CSV, losses, monthly, comparison, measured vs simulated, P50/P90, carbon, economics and deliverable.",
    anotaciones: Anotaciones {
        titulo: "Resultados / Results",
        es_solo_lectura: false,
        es_destructiva: false,
        es_idempotente: true,
        usa_red: true,
    },
    propiedades: &[
        ("proyecto", TipoDato::String),
        ("variante", TipoDato::String),
        ("archivo", TipoDato::String),
        ("kwp", TipoDato::Number),
        ("dir", TipoDato::String),
        ("count", TipoDato::Integer),
        ("include_text", TipoDato::Boolean),
        ("variantes", TipoDato::Array),
        ("sigma_meteo_pct", TipoDato::Number),
        ("sigma_modelo_pct", TipoDato::Number),
        ("internet", TipoDato::Boolean),
        ("medido_kwh", TipoDato::Array),
        ("medido_csv", TipoDato::String),
        ("umbral_pct", TipoDato::Number),
        ("workspace", TipoDato::String),
        ("permitir_fuera_del_workspace", TipoDato::Boolean),
        ("via", TipoDato::String),
    ],
    requeridos: &[],
    discriminador: Some(Discriminador {
        campo: "que",
        defecto: Some("kpis"),
        valores: &[
            "kpis",
            "csv",
            "perdidas",
            "mensual",
            "comparar",
            "medido_vs_simulado",
            "p50_p90",
            "carbono",
            "economia",
            "entregable",
            "resumen",
        ],
    }),
    requeridos_por_accion: &[
        ("medido_vs_simulado", &["proyecto"]),
    ],
    sinonimos: SINONIMOS_RESULTADOS,
    ejecutar: Ejecutor::Simple(ejecutar_resultados),
};

const SINONIMOS_RESULTADOS: Sinonimos = Sinonimos {
    argumentos: &[
        ("what", "que"),
        ("project", "proyecto"),
        ("variant", "variante"),
        ("file", "archivo"),
        ("variants", "variantes"),
    ],
    valores: &[
        ("losses", "perdidas"),
        ("monthly", "mensual"),
        ("compare", "comparar"),
        ("measured_vs_simulated", "medido_vs_simulado"),
        ("carbon", "carbono"),
        ("economics", "economia"),
        ("economy", "economia"),
        ("deliverable", "entregable"),
    ],
};

fn ejecutar_resultados(paths: &Paths, args: &Value) -> Result<Value> {
    let valor = args.get("que").and_then(Value::as_str).unwrap_or("kpis");
    exigir_include_text_solo_csv(valor, args)?;
    if matches!(valor, "kpis" | "resumen") {
        exigir_proyecto_de_kpis(paths, args)?;
    }
    match valor {
        "kpis" => easy_mode::leer_kpis(paths, args),
        "csv" => results::leer_resultados(paths, args),
        "perdidas" => engineering::perdidas(paths, args),
        "mensual" => engineering::mensual(paths, args),
        "comparar" => engineering::comparar(paths, args),
        "medido_vs_simulado" => crate::medicion::comparar_medido(paths, args),
        "p50_p90" => engineering::p50_p90(paths, args),
        "carbono" => engineering::carbono(paths, args),
        "economia" => engineering::economia(paths, args),
        "entregable" => engineering::entregable(paths, args),
        "resumen" => resumen(paths, args),
        _ => Err(Error::Invalid(format!(
            "Valor '{valor}' no soportado para resultados"
        ))),
    }
}

/// `include_text` only shapes the CSV listing; elsewhere it would be silently ignored.
fn exigir_include_text_solo_csv(valor: &str, args: &Value) -> Result<()> {
    if valor != "csv" && args.get("include_text").is_some() {
        return Err(Error::Invalid(format!(
            "'include_text' solo aplica a que:csv (recibido que:{valor})"
        )));
    }
    Ok(())
}

/// `kpis` reads one PDF: name the project or the PDF instead of picking one arbitrarily.
fn exigir_proyecto_de_kpis(paths: &Paths, args: &Value) -> Result<()> {
    if let Some(proyecto) = args.get("proyecto").and_then(Value::as_str) {
        return projects::resolve_project_file(&workspace_of(paths, args), proyecto).map(|_| ());
    }
    if ["archivo", "report_pdf"]
        .iter()
        .any(|clave| args.get(clave).is_some())
    {
        return Ok(());
    }
    Err(Error::Invalid(
        "'proyecto' es obligatorio para que:kpis (o 'archivo'/'report_pdf' con el PDF)".into(),
    ))
}

const PERDIDAS_PRINCIPALES: usize = 3;

/// `resumen`: KPIs, monthly energy and the largest losses. A PDF without a loss diagram
/// leaves `perdidas_principales` empty and says why in `avisos`.
fn resumen(paths: &Paths, args: &Value) -> Result<Value> {
    let kpis = easy_mode::leer_kpis(paths, args)?;
    let mensual = engineering::mensual(paths, args)?;
    let (perdidas, avisos) = match engineering::perdidas(paths, args) {
        Ok(p) => (principales_perdidas(&p), Vec::new()),
        Err(Error::Format(motivo)) => (json!([]), vec![motivo]),
        Err(otro) => return Err(otro),
    };
    Ok(json!({
        "kpis": kpis,
        "mensual": mensual,
        "perdidas_principales": perdidas,
        "avisos": avisos,
    }))
}

/// The `PERDIDAS_PRINCIPALES` most negative loss nodes, largest loss first.
fn principales_perdidas(perdidas: &Value) -> Value {
    let mut negativas: Vec<Value> = perdidas["nodos"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|nodo| pct_de(nodo) < 0.0)
        .cloned()
        .collect();
    negativas.sort_by(|a, b| pct_de(a).total_cmp(&pct_de(b)));
    negativas.truncate(PERDIDAS_PRINCIPALES);
    Value::Array(negativas)
}

fn pct_de(nodo: &Value) -> f64 {
    nodo["pct"].as_f64().unwrap_or(0.0)
}
