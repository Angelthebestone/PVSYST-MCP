//! Trial-budget ledger. Records the licence mode and the remaining executions
//! that PVsystCLI prints, persists them in `UserData/mcp_cache/ledger.json`, and
//! asks for confirmation before a run that would spend several trial executions.

use crate::args::{flag, CONFIRMAR};
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};

const CONFIRMACION_TRIAL: Texto = Texto {
    es: "Esta acción consume {n} ejecuciones de prueba; repítela con confirmar:true",
    en: "This action spends {n} trial runs; repeat it with confirmar:true",
};
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

/// Remaining executions that can be trusted: the counter PVsystCLI prints was
/// measured jumping between two states (133/134 and 249/250) without a licence
/// change, so a count above the stored one is only accepted when the licence
/// renewed (more days). Otherwise the stored, lower count stands.
fn restantes_creibles(previo: Option<&Estado>, nuevo: u64, dias: Option<u64>) -> u64 {
    let Some(previo) = previo.filter(|p| p.modo == "trial") else {
        return nuevo;
    };
    let Some(anteriores) = previo.restantes else {
        return nuevo;
    };
    let renovada = matches!((dias, previo.dias), (Some(hoy), Some(antes)) if hoy > antes);
    if renovada || nuevo <= anteriores {
        nuevo
    } else {
        anteriores
    }
}

/// Licence state as `lic-info` prints it (`Status: TRIAL`, `Remaining days: N`,
/// `Remaining executions: N`). A licensed install prints `Status:` without the
/// trial counters. Measured on 8.1.8 (rev. 49527).
///
/// It is a separate shape from the run banner: feeding `lic-info` to
/// [`registrar`] would find no banner and store `modo:"licencia"`, wiping a good
/// ledger.
pub(crate) fn parsear_lic_info(stdout: &str) -> Option<Estado> {
    let estado = stdout
        .lines()
        .find_map(|l| l.trim().strip_prefix("Status:"))?
        .trim();
    let modo = if estado.eq_ignore_ascii_case("TRIAL") {
        "trial"
    } else {
        "licencia"
    };
    Some(Estado {
        modo: modo.to_owned(),
        dias: stdout
            .lines()
            .find_map(|l| numero_tras(l, "Remaining days:")),
        restantes: stdout
            .lines()
            .find_map(|l| numero_tras(l, "Remaining executions:")),
    })
}

/// Stores the licence state a `lic-info` run printed, so the budget follows every
/// licence read and not only the runs that spend it.
pub(crate) fn registrar_lic_info(workspace: &Path, stdout: &str) -> Option<Estado> {
    let estado = parsear_lic_info(stdout)?;
    if let Err(e) = guardar(workspace, &estado) {
        tracing::warn!(error = %e, "no se pudo guardar el ledger leído de lic-info");
    }
    Some(estado)
}

/// Reads the licence state with `lic-info`, which only inspects the licence (no
/// trial execution). Nothing is stored: the callers are dry runs, which write no file.
fn leer_lic_info(paths: &crate::config::Paths) -> Option<Estado> {
    let out = super::cli::run(
        paths,
        &["lic-info".to_string()],
        super::cli::timeout_de(&json!({}), super::cli::TIMEOUT_DEFECTO_S),
    )
    .ok()?;
    parsear_lic_info(&out.stdout)
}

/// Record the counters printed by one CLI run and return the `costo` object for the response.
/// The banner is printed before the run is charged, so the remaining count is `printed - used`.
pub(crate) fn registrar(workspace: &Path, stdout: &str, ejecuciones_usadas: u32) -> Value {
    let previo = estado(workspace);
    let estado = match parsear_banner(stdout) {
        Some((dias, impresas)) => Estado {
            modo: "trial".to_owned(),
            dias,
            restantes: Some(restantes_creibles(
                previo.as_ref(),
                impresas.saturating_sub(u64::from(ejecuciones_usadas)),
                dias,
            )),
        },
        // A run that prints no TRIAL banner says nothing about the licence: only
        // `lic-info` can. Keep what the ledger knows instead of wiping it.
        None => previo.unwrap_or(Estado {
            modo: "licencia".to_owned(),
            dias: None,
            restantes: None,
        }),
    };
    let mut costo = json!({"ejecuciones": ejecuciones_usadas, "restantes": estado.restantes});
    if let Err(e) = guardar(workspace, &estado) {
        costo["error_ledger"] = json!(e);
    }
    costo
}

/// Trial cost a real run would charge: estimated executions and what the licence
/// has left. Every dry run reports it so the caller decides without a rehearsal.
/// `paths.workspace` is the workspace of the call; `paths.cli_exe` answers `lic-info`.
pub(crate) fn costo_estimado(paths: &crate::config::Paths, ejecuciones: u32) -> Value {
    // A cold ledger knows nothing: `lic-info` is a read, so it answers the budget
    // before the first run instead of `restantes:null`.
    let restantes = match estado(&paths.workspace) {
        Some(e) => e.restantes,
        None => leer_lic_info(paths).and_then(|e| e.restantes),
    };
    json!({ "ejecuciones": ejecuciones, "restantes": restantes })
}

/// Last recorded licence state, or `None` before the first registration or when
/// the ledger cannot be read (logged; see [`estado_leido`]).
pub(crate) fn estado(workspace: &Path) -> Option<Estado> {
    estado_leido(workspace).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "ledger ilegible");
        None
    })
}

/// The ledger as stored: `Ok(None)` when it does not exist yet, an error when it
/// exists but is unreadable or corrupt, so the caller does not mistake it for "no data".
fn estado_leido(workspace: &Path) -> Result<Option<Estado>> {
    let ruta = ruta(workspace);
    let texto = match fs::read_to_string(&ruta) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(Error::io(format!("No se pudo leer {}", ruta.display()), e)),
    };
    let v: Value = serde_json::from_str(&texto)
        .map_err(|e| Error::Format(format!("Ledger corrupto {}: {e}", ruta.display())))?;
    let modo = v["modo"]
        .as_str()
        .ok_or_else(|| Error::Format(format!("Ledger sin 'modo': {}", ruta.display())))?;
    Ok(Some(Estado {
        modo: modo.to_owned(),
        dias: v["dias"].as_u64(),
        restantes: v["restantes"].as_u64(),
    }))
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
    // Atomic: a write cut half way must not leave a corrupt ledger behind.
    crate::history::escribir_atomico(&ruta(workspace), v.to_string().as_bytes())
}

/// Confirmation gate for runs that would spend several trial executions.
/// Returns the response to send instead of running, or `None` when the run may proceed.
pub(crate) fn requiere_confirmacion(
    workspace: &Path,
    args: &Value,
    ejecuciones_estimadas: u32,
    plan: &[String],
) -> Option<Value> {
    if ejecuciones_estimadas < UMBRAL_CONFIRMACION || flag(args, &[CONFIRMAR]) {
        return None;
    }
    // An unreadable ledger may hide a trial licence: ask rather than spend blindly.
    let estado = match estado_leido(workspace) {
        Ok(Some(e)) if e.modo == "trial" => e,
        Ok(_) => return None,
        Err(e) => Estado {
            modo: format!("desconocido ({e})"),
            dias: None,
            restantes: None,
        },
    };
    Some(json!({
        "requiere_confirmacion": true,
        "ejecuciones_estimadas": ejecuciones_estimadas,
        "restantes": estado.restantes,
        "plan": plan,
        "mensaje": CONFIRMACION_TRIAL.texto_con(
            Idioma::de_entorno(),
            &[("n", &ejecuciones_estimadas.to_string())],
        ),
    }))
}

#[cfg(test)]
#[path = "../../tests/unit/adapters/ledger.rs"]
mod tests;
