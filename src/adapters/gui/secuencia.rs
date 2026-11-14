//! `gui_actuar accion:secuencia`: runs several actions in order and stops at
//! the first failure, so a later step never runs on a form an earlier one
//! could not reach.

use crate::config::Paths;
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use serde_json::{json, Value};

/// The steps of a sequence. Every step is checked before any runs: a list
/// that is empty, nests `secuencia` or lacks `accion` is rejected whole.
pub(crate) fn pasos_de(args: &Value, idioma: Idioma) -> Result<&[Value]> {
    let no_es_lista = Texto {
        es: "'pasos' debe ser una lista de acciones [{accion:…}, …]",
        en: "'pasos' must be a list of actions [{accion:…}, …]",
    };
    let pasos = args
        .get("pasos")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::Invalid(no_es_lista.en_idioma(idioma).into()))?;
    if pasos.is_empty() {
        let vacia = Texto {
            es: "'pasos' no puede estar vacío",
            en: "'pasos' cannot be empty",
        };
        return Err(Error::Invalid(vacia.en_idioma(idioma).into()));
    }
    pasos
        .iter()
        .enumerate()
        .try_for_each(|(i, paso)| paso_valido(i, paso, idioma))?;
    Ok(pasos)
}

fn paso_valido(i: usize, paso: &Value, idioma: Idioma) -> Result<()> {
    match paso.get("accion").and_then(Value::as_str) {
        Some("secuencia") => {
            let anidada = Texto {
                es: "'pasos'[{i}]: las secuencias no se anidan",
                en: "'pasos'[{i}]: sequences are not nested",
            };
            Err(Error::Invalid(
                anidada.texto_con(idioma, &[("i", &i.to_string())]),
            ))
        }
        Some(_) => Ok(()),
        None => {
            let sin_accion = Texto {
                es: "'pasos'[{i}] necesita 'accion'",
                en: "'pasos'[{i}] needs 'accion'",
            };
            Err(Error::Invalid(
                sin_accion.texto_con(idioma, &[("i", &i.to_string())]),
            ))
        }
    }
}

/// Runs the steps in order through `actuar`, the same router a single
/// `gui_actuar` call goes through; the first failure ends the sequence with an
/// error naming the step, and nothing after it runs.
pub(crate) fn secuencia(
    paths: &Paths,
    args: &Value,
    actuar: fn(&Paths, &Value) -> Result<Value>,
) -> Result<Value> {
    let pasos = pasos_de(args, paths.idioma)?;
    let mut resultados = Vec::with_capacity(pasos.len());
    for (i, paso) in pasos.iter().enumerate() {
        let accion = paso
            .get("accion")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let res = actuar(paths, paso).map_err(|e| {
            let detenida = Texto {
                es: "secuencia detenida en el paso {i} ({accion}): {error}",
                en: "sequence stopped at step {i} ({accion}): {error}",
            };
            Error::Gui(detenida.texto_con(
                paths.idioma,
                &[
                    ("i", &i.to_string()),
                    ("accion", accion),
                    ("error", &e.to_string()),
                ],
            ))
        })?;
        resultados.push(res);
    }
    Ok(json!({
        "accion": "secuencia",
        "completados": resultados.len(),
        "resultados": resultados,
    }))
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/secuencia.rs"]
mod tests;
