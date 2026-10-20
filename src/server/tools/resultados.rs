//! Tool for simulation output and engineering analysis: KPIs, CSV, losses, economics.

use super::declaracion::{Acceso, Anotaciones, Discriminador, Ejecutor, Herramienta, TipoDato};
use crate::config::Paths;
use crate::easy_mode;
use crate::error::{Error, Result};
use crate::simulation::{engineering, results};
use serde_json::Value;

pub(crate) const TOOL_RESULTADOS: Herramienta = Herramienta {
    nombre: "resultados",
    descripcion: "Resultados: KPIs, CSV, perdidas, mensual, comparacion, medido_vs_simulado, P50/P90, carbono, economia y entregable. / EN: Results: KPIs, CSV, losses, monthly, comparison, measured vs simulated, P50/P90, carbon, economics and deliverable.",
    acceso: Acceso::Normal,
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
        ("vida_anios", TipoDato::Number),
        ("degradacion", TipoDato::Number),
        ("lce_kgco2_por_kwp", TipoDato::Number),
        ("capex_por_kwp", TipoDato::Number),
        ("tarifa_por_kwh", TipoDato::Number),
        ("opex_anual_por_kwp", TipoDato::Number),
        ("tasa_descuento", TipoDato::Number),
        ("anios", TipoDato::Integer),
        ("moneda", TipoDato::String),
        ("medido_kwh", TipoDato::Array),
        ("medido_csv", TipoDato::String),
        ("umbral_pct", TipoDato::Number),
        ("workspace", TipoDato::String),
        ("permitir_fuera_del_workspace", TipoDato::Boolean),
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
        ],
    }),
    requeridos_por_accion: &[
        ("economia", &["capex_por_kwp", "tarifa_por_kwh"]),
        ("medido_vs_simulado", &["proyecto"]),
    ],
    ejecutar: Ejecutor::Simple(ejecutar_resultados),
};

fn ejecutar_resultados(paths: &Paths, args: &Value) -> Result<Value> {
    let valor = args.get("que").and_then(Value::as_str).unwrap_or("kpis");
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
        _ => Err(Error::Invalid(format!(
            "Valor '{valor}' no soportado para resultados"
        ))),
    }
}
