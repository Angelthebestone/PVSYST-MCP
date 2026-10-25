//! Module (`modulo`) and inverter (`inversor`) that a sizing request writes into a variant.
//! Both are checked against the component library before any byte is written.

use crate::components::{pan, resolve_component, ConsultaComponente};
use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::path::Path;

/// Component references a sizing request asks to write into `pvSubArray`.
#[derive(Default)]
pub(crate) struct Componentes {
    /// Bare `.PAN` file name written as `PVModule`.
    pub(crate) modulo: Option<String>,
    /// Bare `.OND` file name written as `GInverter`.
    pub(crate) inversor: Option<String>,
}

impl Componentes {
    /// Reads `modulo` and `inversor`; each must exist in the component library.
    pub(crate) fn leer(paths: &Paths, args: &Value) -> Result<Self> {
        let modulo = args
            .get("modulo")
            .and_then(Value::as_str)
            .map(|m| validar_modulo(paths, args, m))
            .transpose()?;
        let inversor = args
            .get("inversor")
            .and_then(Value::as_str)
            .map(|i| validar_inversor(paths, i))
            .transpose()?;
        Ok(Self { modulo, inversor })
    }

    /// `pvSubArray` changes for the components given.
    pub(crate) fn cambios(&self) -> Vec<(String, Option<String>)> {
        self.modulo
            .iter()
            .map(|m| ("PVModule".to_string(), Some(m.clone())))
            .chain(
                self.inversor
                    .iter()
                    .map(|i| ("GInverter".to_string(), Some(i.clone()))),
            )
            .collect()
    }
}

/// Rejects `modulo` / `inversor` on the PVsyst pre-sizing route, which picks its own components.
pub(crate) fn exigir_sin_pre_dimensionado(args: &Value) -> Result<()> {
    match ["modulo", "inversor"]
        .into_iter()
        .find(|k| args.get(*k).is_some())
    {
        Some(k) => Err(Error::Invalid(format!(
            "'{k}' no admite el pre-dimensionado de PVsyst: pase n_mod_serie y n_strings, o via 'aproximacion'"
        ))),
        None => Ok(()),
    }
}

/// Bare file name of a component reference; the error names the parameter.
fn nombre_archivo(parametro: &str, valor: &str) -> Result<String> {
    Path::new(valor)
        .file_name()
        .and_then(|f| f.to_str())
        .map(str::to_string)
        .ok_or_else(|| {
            Error::Invalid(format!(
                "'{parametro}' debe ser el nombre de un componente: '{valor}'"
            ))
        })
}

fn validar_modulo(paths: &Paths, args: &Value, modulo: &str) -> Result<String> {
    let nombre = nombre_archivo("modulo", modulo)?;
    let mut pedido = json!({ "modulo": modulo });
    if let Some(ws) = args.get("workspace") {
        pedido["workspace"] = ws.clone();
    }
    pan::leer_modulo(paths, &pedido)?;
    Ok(nombre)
}

fn validar_inversor(paths: &Paths, inversor: &str) -> Result<String> {
    let nombre = nombre_archivo("inversor", inversor)?;
    resolve_component(
        &paths.workspace,
        &ConsultaComponente::referencia("OND", inversor),
    )?;
    Ok(nombre)
}
