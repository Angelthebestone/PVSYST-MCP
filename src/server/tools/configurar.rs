//! `proyecto accion:configurar`: several variant changes in one call, validated
//! as a whole before the first one runs.

use super::proyectos::{ejecutar_proyecto, TOOL_PROYECTO};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use serde_json::Value;

/// Actions `configurar` may chain: they only modify an existing variant.
const PASO_OBJETO: Texto = Texto {
    es: "Cada elemento de 'pasos' debe ser un objeto {accion:…}",
    en: "Each element of 'pasos' must be an object {accion:…}",
};
const PASO_ACCION: Texto = Texto {
    es: "Acción '{accion}' no admitida en 'pasos'; válidas: {validas}",
    en: "Action '{accion}' not supported in 'pasos'; valid ones: {validas}",
};
const PASOS_LISTA: Texto = Texto {
    es: "'pasos' debe ser una lista no vacía de {accion:…}",
    en: "'pasos' must be a non-empty list of {accion:…}",
};
const PASO_INVALIDO: Texto = Texto {
    es: "Paso {n} inválido; no se aplicó ninguno",
    en: "Step {n} invalid; none were applied",
};
const PASO_FALLIDO: Texto = Texto {
    es: "Paso {n} fallido tras aplicar {aplicados} paso(s)",
    en: "Step {n} failed after {aplicados} step(s) applied",
};

const ACCIONES_CONFIGURABLES: &[&str] = &[
    "dimensionar",
    "anadir",
    "seguidor",
    "limite_inyeccion",
    "almacenamiento",
    "perdidas",
    "bombeo",
    "economia",
];

/// Step `paso` as a complete `proyecto` call: the parent's `proyecto`, `variante` and
/// `workspace` fill whatever the step leaves out.
fn paso_completo(padre: &Value, paso: &Value, idioma: Idioma) -> Result<Value> {
    let mut completo = paso
        .as_object()
        .cloned()
        .ok_or_else(|| Error::Invalid(PASO_OBJETO.texto_con(idioma, &[])))?;
    let accion = completo.get("accion").and_then(Value::as_str).unwrap_or("");
    if !ACCIONES_CONFIGURABLES.contains(&accion) {
        return Err(Error::Invalid(PASO_ACCION.texto_con(
            idioma,
            &[
                ("accion", accion),
                ("validas", &ACCIONES_CONFIGURABLES.join(", ")),
            ],
        )));
    }
    for clave in ["proyecto", "variante", "workspace"] {
        if let (false, Some(valor)) = (completo.contains_key(clave), padre.get(clave)) {
            completo.insert(clave.into(), valor.clone());
        }
    }
    Ok(Value::Object(completo))
}

/// Validates one step's values without touching the variant.
///
/// Each action owns its range checks; running them all up front makes
/// `configurar` all-or-nothing for bad values, not just for bad structure.
fn validar_valores_de_paso(accion: &str, args: &Value) -> Result<()> {
    match accion {
        "dimensionar" => crate::projects::lifecycle::validar_dimensionado(args),
        "anadir" => crate::projects::anadir::catalogo::validar_elemento(args),
        "seguidor" => crate::seguidores::validar_parametros(args),
        "limite_inyeccion" => crate::inyeccion::validar_parametros(args),
        "almacenamiento" => crate::almacenamiento::validar_parametros(args),
        "perdidas" => crate::perdidas::validar_parametros(args),
        "bombeo" => crate::bombeo::validar_parametros(args),
        "economia" => crate::evaluacion_economica::validar_parametros(args),
        _ => Ok(()),
    }
}

/// Applies several variant changes in one call, in order. All steps are validated before
/// the first one runs; if one fails the earlier ones stay applied (each is in the history).
pub(super) fn variante(paths: &Paths, args: &Value) -> Result<Value> {
    let pasos = args
        .get("pasos")
        .and_then(Value::as_array)
        .filter(|p| !p.is_empty())
        .ok_or_else(|| Error::Invalid(PASOS_LISTA.texto_con(paths.idioma, &[])))?;
    // Every step goes through the same aliases and schema checks as a direct call.
    let llamadas = pasos
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let llamada = TOOL_PROYECTO
                .preparar_args(&paso_completo(args, p, paths.idioma)?, paths.idioma)?;
            TOOL_PROYECTO.validar(&llamada, paths).map_err(|e| {
                con_prefijo(
                    e,
                    &PASO_INVALIDO.texto_con(paths.idioma, &[("n", &(i + 1).to_string())]),
                )
            })?;
            validar_valores_de_paso(TOOL_PROYECTO.accion(&llamada), &llamada).map_err(|e| {
                con_prefijo(
                    e,
                    &PASO_INVALIDO.texto_con(paths.idioma, &[("n", &(i + 1).to_string())]),
                )
            })?;
            Ok(llamada)
        })
        .collect::<Result<Vec<_>>>()?;
    let mut resultados = Vec::with_capacity(llamadas.len());
    for (indice, llamada) in llamadas.iter().enumerate() {
        let resultado = ejecutar_proyecto(paths, llamada).map_err(|e| {
            con_prefijo(
                e,
                &PASO_FALLIDO.texto_con(
                    paths.idioma,
                    &[
                        ("n", &(indice + 1).to_string()),
                        ("aplicados", &indice.to_string()),
                    ],
                ),
            )
        })?;
        resultados.push(resultado);
    }
    Ok(serde_json::json!({ "pasos_aplicados": resultados.len(), "resultados": resultados }))
}

/// `e` with `prefijo` before its message, keeping its kind so the answer's error code
/// still says what went wrong (not every failure is an invalid argument).
fn con_prefijo(e: Error, prefijo: &str) -> Error {
    let con = |m: String| format!("{prefijo}: {m}");
    match e {
        Error::Invalid(m) => Error::Invalid(con(m)),
        Error::NotFound(m) => Error::NotFound(con(m)),
        Error::Format(m) => Error::Format(con(m)),
        Error::Cli(m) => Error::Cli(con(m)),
        Error::Gui(m) => Error::Gui(con(m)),
        Error::Network(m) => Error::Network(con(m)),
        Error::Io { context, source } => Error::Io {
            context: con(context),
            source,
        },
        otro @ (Error::MissingArg { .. } | Error::MissingArgs { .. }) => otro,
    }
}
