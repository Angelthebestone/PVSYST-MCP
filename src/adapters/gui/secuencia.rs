//! `gui_actuar accion:secuencia`: runs several actions in order and stops at
//! the first failure, so a later step never runs on a form an earlier one
//! could not reach.

use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::{json, Value};

/// The steps of a sequence. Every step is checked before any runs: a list
/// that is empty, nests `secuencia` or lacks `accion` is rejected whole.
pub(crate) fn pasos_de(args: &Value) -> Result<&[Value]> {
    let pasos = args.get("pasos").and_then(Value::as_array).ok_or_else(|| {
        Error::Invalid("'pasos' debe ser una lista de acciones [{accion:…}, …]".into())
    })?;
    if pasos.is_empty() {
        return Err(Error::Invalid("'pasos' no puede estar vacío".into()));
    }
    pasos
        .iter()
        .enumerate()
        .try_for_each(|(i, paso)| paso_valido(i, paso))?;
    Ok(pasos)
}

fn paso_valido(i: usize, paso: &Value) -> Result<()> {
    match paso.get("accion").and_then(Value::as_str) {
        Some("secuencia") => Err(Error::Invalid(format!(
            "'pasos'[{i}]: las secuencias no se anidan"
        ))),
        Some(_) => Ok(()),
        None => Err(Error::Invalid(format!("'pasos'[{i}] necesita 'accion'"))),
    }
}

/// Runs the steps in order; the first failure ends the sequence with an error
/// naming the step, and nothing after it runs.
pub(crate) fn secuencia(paths: &Paths, args: &Value) -> Result<Value> {
    let pasos = pasos_de(args)?;
    let mut resultados = Vec::with_capacity(pasos.len());
    for (i, paso) in pasos.iter().enumerate() {
        let accion = paso
            .get("accion")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let res = super::actuar(paths, paso).map_err(|e| {
            Error::Gui(format!("secuencia detenida en el paso {i} ({accion}): {e}"))
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
