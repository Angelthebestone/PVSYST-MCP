//! `resultados que:medido_vs_simulado`: compares PVsyst's monthly `E_Grid`
//! with a measured series supplied as an array or a CSV file.

use crate::args::{missing, opt_str, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::medicion::dominio::{comparar, serie_de_csv, SerieMensual, UMBRAL_DEFECTO_PCT};
use crate::simulation::{engineering, report_pdf};
use crate::workspace::{leer_bytes, resolver_ruta, Alcance};
use serde_json::{json, Value};
use std::path::PathBuf;

const COLUMNA: &str = "E_Grid";

pub(crate) fn comparar_medido(paths: &Paths, args: &Value) -> Result<Value> {
    let (medida, fuente) = serie_medida(paths, args)?;
    let umbral_pct = umbral(args)?;
    let caso = engineering::caso(paths, args)?;
    let mensual = report_pdf::mensual(&caso.tokens)?;
    let simulada = simulada_de(&mensual)?;
    let comparacion = comparar(&simulada, &medida, umbral_pct);
    Ok(engineering::con_cabecera(
        &caso,
        json!({
            "fuente": fuente,
            "umbral_pct": umbral_pct,
            "meses": comparacion.meses,
            "anual": comparacion.anual,
            "fuera_de_umbral": comparacion.fuera_de_umbral,
        }),
    ))
}

fn umbral(args: &Value) -> Result<f64> {
    match args.get("umbral_pct") {
        None | Some(Value::Null) => Ok(UMBRAL_DEFECTO_PCT),
        Some(valor) => valor
            .as_f64()
            .filter(|umbral| umbral.is_finite() && *umbral >= 0.0)
            .ok_or_else(|| Error::Invalid("'umbral_pct' debe ser un número no negativo".into())),
    }
}

fn serie_medida(paths: &Paths, args: &Value) -> Result<(SerieMensual, &'static str)> {
    match args.get("medido_kwh") {
        Some(Value::Array(valores)) => {
            let numeros: Result<Vec<f64>> = valores
                .iter()
                .map(|valor| {
                    valor.as_f64().ok_or_else(|| {
                        Error::Invalid("'medido_kwh' debe contener solo números".into())
                    })
                })
                .collect();
            Ok((SerieMensual::nuevo(&numeros?)?, "medido_kwh"))
        }
        Some(_) => Err(Error::Invalid(
            "'medido_kwh' debe ser un array de 12 números".into(),
        )),
        None => {
            let ruta = opt_str(args, &["medido_csv"])
                .ok_or_else(|| missing("medido_kwh", "o medido_csv"))?;
            let serie = serie_de_archivo(paths, args, ruta)?;
            Ok((serie, "medido_csv"))
        }
    }
}

fn serie_de_archivo(paths: &Paths, args: &Value, ruta: &str) -> Result<SerieMensual> {
    let workspace = workspace_of(paths, args);
    let ruta: PathBuf = resolver_ruta(&workspace, ruta, Alcance::de_args(args))?;
    let texto = String::from_utf8_lossy(&leer_bytes(&ruta)?).into_owned();
    serie_de_csv(&texto)
}

fn simulada_de(mensual: &Value) -> Result<SerieMensual> {
    let columnas = mensual["columnas"]
        .as_array()
        .ok_or_else(|| Error::Format("La tabla mensual no declara columnas".into()))?;
    let indice = columnas
        .iter()
        .position(|columna| {
            columna
                .as_str()
                .is_some_and(|nombre| nombre.eq_ignore_ascii_case(COLUMNA))
        })
        .ok_or_else(|| Error::Format("El informe no tiene la columna E_Grid mensual".into()))?;
    let clave = columnas[indice]
        .as_str()
        .ok_or_else(|| Error::Format("Columna E_Grid sin nombre".into()))?;
    let unidad = mensual["unidades"]
        .as_array()
        .and_then(|unidades| unidades.get(indice))
        .and_then(Value::as_str)
        .ok_or_else(|| Error::Format("La tabla mensual no declara unidades".into()))?;
    let factor = factor_de_unidad(unidad)?;
    let valores: Result<Vec<f64>> = mensual["meses"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|mes| {
            mes["valores"]
                .get(clave)
                .and_then(Value::as_f64)
                .map(|valor| valor * factor)
                .ok_or_else(|| Error::Format(format!("Fila mensual sin E_Grid: {mes}")))
        })
        .collect();
    SerieMensual::nuevo(&valores?)
}

fn factor_de_unidad(unidad: &str) -> Result<f64> {
    match unidad.trim().to_ascii_lowercase().as_str() {
        "mwh" => Ok(1_000.0),
        "kwh" => Ok(1.0),
        otra => Err(Error::Format(format!(
            "Unidad no soportada para E_Grid: '{otra}'"
        ))),
    }
}
