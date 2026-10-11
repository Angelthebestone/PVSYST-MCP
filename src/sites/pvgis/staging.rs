//! Per-run staging of the PVGIS weather artifacts (CSV, MEF, MET): a failed
//! or interrupted run leaves nothing in `Meteo`, and the published MET/CSV
//! pair is all-or-nothing.

use crate::config::data_ro;
use crate::error::{Error, Result};
use crate::history;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Sequence counter for generating process-unique run identifiers.
static PVGIS_EJECUCION_ID: AtomicU64 = AtomicU64::new(1);

/// Unique run id: pid, nanoseconds and a process-wide counter.
fn generar_id_ejecucion() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let seq = PVGIS_EJECUCION_ID.fetch_add(1, Ordering::Relaxed);
    format!("{}_{now}_{seq}", std::process::id())
}

/// Paths configuration for transactional staging of PVGIS meteo files.
pub(crate) struct ConfiguracionStaging<'a> {
    pub(crate) ws: &'a Path,
    pub(crate) nombre: &'a str,
    pub(crate) final_met: PathBuf,
    pub(crate) final_csv: PathBuf,
}

/// Transactional staging environment isolating CSV, MEF, and MET generation.
pub(crate) struct StagingMeteo {
    pub(crate) staging_dir: PathBuf,
    pub(crate) csv_path: PathBuf,
    pub(crate) mef_path: PathBuf,
    pub(crate) staging_met_path: PathBuf,
    pub(crate) final_csv_path: PathBuf,
    pub(crate) final_met_path: PathBuf,
}

/// Bytes at `path`, or `None` when it does not exist.
fn leer_si_existe(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(b) => Ok(Some(b)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::io(format!("No se pudo leer {}", path.display()), e)),
    }
}

/// Put back what each published destination held before (newest first);
/// returns a description of any rollback failure, empty when all succeeded.
fn revertir(publicados: &[(&PathBuf, Option<Vec<u8>>)]) -> String {
    let fallos: Vec<String> = publicados
        .iter()
        .rev()
        .filter_map(|(destino, previo)| {
            let r = match previo {
                Some(b) => history::escribir_atomico(destino, b),
                None => fs::remove_file(destino)
                    .map_err(|e| Error::io(format!("No se pudo borrar {}", destino.display()), e)),
            };
            r.err().map(|e| e.to_string())
        })
        .collect();
    if fallos.is_empty() {
        String::new()
    } else {
        format!("; la reversión falló: {}", fallos.join("; "))
    }
}

impl StagingMeteo {
    /// Prepares staging directory and isolated paths under the workspace Meteo directory.
    pub(crate) fn preparar(cfg: &ConfiguracionStaging<'_>) -> Result<Self> {
        let run_id = generar_id_ejecucion();
        let staging_dir = cfg
            .ws
            .join("Meteo")
            .join(".staging")
            .join(format!("pvgis_{run_id}"));
        fs::create_dir_all(&staging_dir).map_err(|e| {
            Error::io(
                format!("No se pudo crear staging {}", staging_dir.display()),
                e,
            )
        })?;
        Ok(Self {
            csv_path: staging_dir.join(format!("{}_PVGIS.csv", cfg.nombre)),
            mef_path: staging_dir.join(format!("MCP_PVGIS_{run_id}.MEF")),
            staging_met_path: staging_dir.join(format!("{}_PVGIS.MET", cfg.nombre)),
            staging_dir,
            final_csv_path: cfg.final_csv.clone(),
            final_met_path: cfg.final_met.clone(),
        })
    }

    /// Writes staging CSV and copies the template MEF with a per-run isolated filename.
    pub(crate) fn inicializar_archivos(&self, csv_data: &str) -> Result<()> {
        fs::write(&self.csv_path, csv_data).map_err(|e| {
            Error::io(
                format!("No se pudo escribir CSV {}", self.csv_path.display()),
                e,
            )
        })?;
        let plantilla = data_ro()
            .join("PVsyst8.1_Data")
            .join("Meteo")
            .join("PVsyst_ANE-NEDO.MEF");
        if !plantilla.exists() {
            return Err(Error::NotFound(format!(
                "Plantilla MEF no encontrada en {}",
                plantilla.display()
            )));
        }
        fs::copy(&plantilla, &self.mef_path).map_err(|e| {
            Error::io(
                format!("No se pudo copiar MEF a {}", self.mef_path.display()),
                e,
            )
        })?;
        Ok(())
    }

    /// Publishes MET and CSV as a pair: if one publication fails, the ones
    /// already made are reverted, so `Meteo` never holds half a result.
    pub(crate) fn commit(&self) -> Result<()> {
        if let Some(p) = self.final_met_path.parent() {
            fs::create_dir_all(p)
                .map_err(|e| Error::io(format!("No se pudo crear {}", p.display()), e))?;
        }
        let pares = [
            (&self.staging_met_path, &self.final_met_path),
            (&self.csv_path, &self.final_csv_path),
        ];
        // Read everything first: a read failure must abort before any write.
        // Neither artifact may be missing: publishing only one leaves half a result.
        let mut plan = Vec::with_capacity(pares.len());
        for (origen, destino) in pares {
            let nuevo = leer_si_existe(origen)?.ok_or_else(|| {
                Error::NotFound(format!(
                    "Publicación cancelada: falta el artefacto esperado {}",
                    origen.display()
                ))
            })?;
            plan.push((destino, nuevo, leer_si_existe(destino)?));
        }
        let mut publicados: Vec<(&PathBuf, Option<Vec<u8>>)> = Vec::with_capacity(plan.len());
        for (destino, nuevo, previo) in plan {
            if let Err(e) = history::escribir_atomico(destino, &nuevo) {
                let reversion = revertir(&publicados);
                return Err(Error::Invalid(format!(
                    "Publicación cancelada en {}: {e}{reversion}",
                    destino.display()
                )));
            }
            publicados.push((destino, previo));
        }
        self.limpiar();
        Ok(())
    }

    /// On failure or cancellation: deletes all staging files and the staging directory.
    pub(crate) fn rollback(&self) {
        self.limpiar();
    }

    fn limpiar(&self) {
        if let Err(e) = fs::remove_dir_all(&self.staging_dir) {
            tracing::warn!(dir = %self.staging_dir.display(), error = %e, "staging PVGIS no eliminado");
        }
        // Only succeeds when no other run is staging; a busy `.staging` stays.
        if let Some(parent) = self.staging_dir.parent() {
            let _ = fs::remove_dir(parent);
        }
    }
}
