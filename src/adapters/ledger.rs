//! Trial-budget ledger. Records the licence mode and the remaining executions
//! that PVsystCLI prints, persists them in `UserData/mcp_cache/ledger.json`, and
//! asks for confirmation before a run that would spend several trial executions.

use crate::args::{flag, CONFIRMAR};
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Estimated trial executions at or above which a trial run needs confirmation.
const UMBRAL_CONFIRMACION: u32 = 2;

/// Licence state persisted in `ledger.json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Estado {
    /// `trial` while PVsystCLI prints the TRIAL banner, `licencia` otherwise.
    pub modo: String,
    pub dias: Option<u64>,
    pub restantes: Option<u64>,
}

/// Cache folder under the workspace; holds the ledger and the simulation cache.
pub(crate) fn carpeta_cache(workspace: &Path) -> PathBuf {
    workspace.join("UserData").join("mcp_cache")
}

/// Seconds since the Unix epoch (0 if the clock is before it).
pub(crate) fn ahora() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn ruta(workspace: &Path) -> PathBuf {
    carpeta_cache(workspace).join("ledger.json")
}

/// Number that follows `etiqueta` in `linea`, e.g. `remaining days: 58`.
fn numero_tras(linea: &str, etiqueta: &str) -> Option<u64> {
    let resto = linea.split_once(etiqueta)?.1.trim_start();
    let digitos: String = resto.chars().take_while(char::is_ascii_digit).collect();
    digitos.parse().ok()
}

/// TRIAL banner counters as `(remaining days, remaining executions)`.
pub(crate) fn parsear_banner(stdout: &str) -> Option<(Option<u64>, u64)> {
    if !stdout.contains("TRIAL mode") {
        return None;
    }
    let linea = stdout
        .lines()
        .find(|l| l.contains("remaining executions:"))?;
    let restantes = numero_tras(linea, "remaining executions:")?;
    Some((numero_tras(linea, "remaining days:"), restantes))
}

/// Record the counters printed by one CLI run and return the `costo` object for the response.
/// The banner is printed before the run is charged, so the remaining count is `printed - used`.
pub(crate) fn registrar(workspace: &Path, stdout: &str, ejecuciones_usadas: u32) -> Value {
    let estado = match parsear_banner(stdout) {
        Some((dias, impresas)) => Estado {
            modo: "trial".to_owned(),
            dias,
            restantes: Some(impresas.saturating_sub(u64::from(ejecuciones_usadas))),
        },
        None => Estado {
            modo: "licencia".to_owned(),
            dias: None,
            restantes: None,
        },
    };
    let mut costo = json!({"ejecuciones": ejecuciones_usadas, "restantes": estado.restantes});
    if let Err(e) = guardar(workspace, &estado) {
        costo["error_ledger"] = json!(e);
    }
    costo
}

/// Last recorded licence state, or `None` before the first registration.
pub(crate) fn estado(workspace: &Path) -> Option<Estado> {
    let texto = fs::read_to_string(ruta(workspace)).ok()?;
    let v: Value = serde_json::from_str(&texto).ok()?;
    Some(Estado {
        modo: v["modo"].as_str()?.to_owned(),
        dias: v["dias"].as_u64(),
        restantes: v["restantes"].as_u64(),
    })
}

fn guardar(workspace: &Path, estado: &Estado) -> Result<()> {
    fs::create_dir_all(carpeta_cache(workspace))
        .map_err(|e| Error::io("No se puede crear la carpeta del ledger", e))?;
    let v = json!({
        "modo": estado.modo,
        "dias": estado.dias,
        "restantes": estado.restantes,
        "actualizado": ahora(),
    });
    fs::write(ruta(workspace), v.to_string())
        .map_err(|e| Error::io("No se puede escribir el ledger", e))
}

/// Confirmation gate for runs that would spend several trial executions.
/// Returns the response to send instead of running, or `None` when the run may proceed.
pub(crate) fn requiere_confirmacion(
    workspace: &Path,
    args: &Value,
    ejecuciones_estimadas: u32,
    plan: &[String],
) -> Option<Value> {
    let estado = estado(workspace)?;
    if estado.modo != "trial"
        || ejecuciones_estimadas < UMBRAL_CONFIRMACION
        || flag(args, &[CONFIRMAR])
    {
        return None;
    }
    Some(json!({
        "requiere_confirmacion": true,
        "ejecuciones_estimadas": ejecuciones_estimadas,
        "restantes": estado.restantes,
        "plan": plan,
        "mensaje": format!(
            "Esta acción consume {ejecuciones_estimadas} ejecuciones de prueba; repítela con confirmar:true"
        ),
    }))
}

#[cfg(test)]
#[path = "../../tests/unit/adapters/ledger.rs"]
mod tests;
