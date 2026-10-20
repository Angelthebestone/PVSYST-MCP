//! Pumping parameters and the `pvSystem` / `WaterUse` changes they require.

use crate::args::missing;
use crate::error::{Error, Result};
use crate::format::edit::EdicionObjeto;
use serde_json::{json, Value};

const MAX_CAUDAL_M3_DIA: f64 = 10_000.0;
const MAX_PROFUNDIDAD_M: f64 = 1_000.0;
const DECIMALES_CAUDAL: usize = 3;
const DECIMALES_PROFUNDIDAD: usize = 1;

/// Raw pumping arguments.
#[derive(Debug, Clone)]
pub(crate) struct EntradaBombeo {
    pub(crate) caudal_m3_dia: Option<f64>,
    pub(crate) profundidad_estatica_m: Option<f64>,
    pub(crate) bomba: Option<String>,
}

impl EntradaBombeo {
    pub(crate) fn de_args(args: &Value) -> Result<Self> {
        Ok(Self {
            caudal_m3_dia: numero(args, "caudal_m3_dia")?,
            profundidad_estatica_m: numero(args, "profundidad_estatica_m")?,
            bomba: args
                .get("bomba")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string),
        })
    }
}

fn numero(args: &Value, clave: &str) -> Result<Option<f64>> {
    match args.get(clave) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => v
            .as_f64()
            .filter(|n| n.is_finite())
            .map(Some)
            .ok_or_else(|| Error::Invalid(format!("'{clave}' debe ser un número finito"))),
    }
}

fn rango(valor: Option<f64>, max: f64, clave: &str) -> Result<()> {
    let Some(valor) = valor else {
        return Ok(());
    };
    if valor > 0.0 && valor <= max {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "'{clave}' debe estar en (0, {max}]: {valor}"
    )))
}

fn validar_bomba(bomba: &str) -> Result<()> {
    if bomba.contains(['/', '\\', ':']) || !bomba.to_ascii_uppercase().ends_with(".PMP") {
        return Err(Error::Invalid(format!(
            "Nombre de bomba inválido '{bomba}': debe terminar en .PMP y no contener separadores de ruta"
        )));
    }
    Ok(())
}

/// Validated pumping configuration.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ParametrosBombeo {
    caudal_m3_dia: Option<f64>,
    profundidad_estatica_m: Option<f64>,
    bomba: Option<String>,
}

impl ParametrosBombeo {
    pub(crate) fn nuevo(entrada: EntradaBombeo) -> Result<Self> {
        if entrada.caudal_m3_dia.is_none()
            && entrada.profundidad_estatica_m.is_none()
            && entrada.bomba.is_none()
        {
            return Err(missing("caudal_m3_dia", "o profundidad_estatica_m, bomba"));
        }
        rango(entrada.caudal_m3_dia, MAX_CAUDAL_M3_DIA, "caudal_m3_dia")?;
        rango(
            entrada.profundidad_estatica_m,
            MAX_PROFUNDIDAD_M,
            "profundidad_estatica_m",
        )?;
        if let Some(bomba) = &entrada.bomba {
            validar_bomba(bomba)?;
        }
        Ok(Self {
            caudal_m3_dia: entrada.caudal_m3_dia,
            profundidad_estatica_m: entrada.profundidad_estatica_m,
            bomba: entrada.bomba,
        })
    }

    pub(crate) fn como_json(&self) -> Value {
        json!({
            "caudal_m3_dia": self.caudal_m3_dia,
            "profundidad_estatica_m": self.profundidad_estatica_m,
            "bomba": self.bomba,
        })
    }

    fn profundidad_texto(&self) -> Option<String> {
        self.profundidad_estatica_m
            .map(|valor| format!("-{:.*}", DECIMALES_PROFUNDIDAD, valor))
    }

    /// `pvSystem` edition, or `None` when only the section changes.
    pub(crate) fn edicion_sistema(&self) -> Option<EdicionObjeto> {
        let bomba = self
            .bomba
            .as_ref()
            .map(|nombre| ("PumpFile".to_string(), Some(nombre.clone())));
        let profundidad = self
            .profundidad_texto()
            .map(|valor| ("StaticDepth".to_string(), Some(valor)));
        let cambios: Vec<(String, Option<String>)> = bomba.into_iter().chain(profundidad).collect();
        (!cambios.is_empty()).then(|| EdicionObjeto {
            selector: "pvSystem".into(),
            cambios,
        })
    }

    /// `WaterUse` section changes, or empty when only the pump changes.
    pub(crate) fn cambios_seccion(&self) -> Vec<(String, Option<String>)> {
        if self.caudal_m3_dia.is_none() && self.profundidad_estatica_m.is_none() {
            return Vec::new();
        }
        let caudal = self.caudal_m3_dia.map(|valor| {
            (
                "FlowYear".to_string(),
                Some(format!("{:.*}", DECIMALES_CAUDAL, valor)),
            )
        });
        let profundidad = self
            .profundidad_texto()
            .map(|valor| ("StaticDepth".to_string(), Some(valor)));
        std::iter::once(("FlowModul".to_string(), Some("Yearly".to_string())))
            .chain(caudal)
            .chain(profundidad)
            .collect()
    }
}

#[cfg(test)]
#[path = "../../tests/unit/bombeo/dominio.rs"]
mod tests;
