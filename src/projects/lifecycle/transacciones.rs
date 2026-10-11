//! Transactional multi-file writes: every destination is snapshotted first and
//! previous bytes are restored if any write or rename fails.

use crate::error::{Error, Result};
use crate::history;
use std::fs;
use std::path::{Path, PathBuf};

/// Write several files as one unit. Existing files are snapshotted first; if a
/// write fails, previous bytes are restored. Rollback errors are preserved in the error.
pub(super) fn escribir_par(
    workspace: &Path,
    archivos: &[(PathBuf, Vec<u8>)],
) -> Result<Option<String>> {
    let existentes: Vec<PathBuf> = archivos
        .iter()
        .filter(|(p, _)| p.exists())
        .map(|(p, _)| p.clone())
        .collect();

    let mut previos: Vec<Option<Vec<u8>>> = Vec::with_capacity(archivos.len());
    for (p, _) in archivos {
        if p.exists() {
            let b = fs::read(p)
                .map_err(|e| Error::io(format!("No se pudo leer previo {}", p.display()), e))?;
            previos.push(Some(b));
        } else {
            previos.push(None);
        }
    }

    let historial_id = if existentes.is_empty() {
        None
    } else {
        Some(history::snapshot(workspace, &existentes, "crear_proyecto")?)
    };

    for (i, (path, bytes)) in archivos.iter().enumerate() {
        if let Err(e) = history::escribir_atomico(path, bytes) {
            let mut fallos_rollback = Vec::new();
            for (j, (done, _)) in archivos.iter().enumerate().take(i) {
                let res = match &previos[j] {
                    Some(old) => history::escribir_atomico(done, old),
                    None => {
                        fs::remove_file(done).map_err(|err| Error::io("revertir creación", err))
                    }
                };
                if let Err(r_err) = res {
                    fallos_rollback.push(format!("{}: {r_err}", done.display()));
                }
            }
            if fallos_rollback.is_empty() {
                return Err(e);
            }
            return Err(Error::io(
                format!(
                    "Fallo escribiendo {} ({e}); rollback fallido en: [{}]",
                    path.display(),
                    fallos_rollback.join("; ")
                ),
                std::io::Error::other(e.to_string()),
            ));
        }
    }
    Ok(historial_id)
}

/// Rename every pair or none: on the first failure, already-moved files are
/// moved back (newest first) and any rollback failure is reported too.
pub(super) fn renombrar_todo(pares: &[(PathBuf, PathBuf)]) -> Result<Vec<String>> {
    for (i, (origen, destino)) in pares.iter().enumerate() {
        let Err(e) = fs::rename(origen, destino) else {
            continue;
        };
        let fallos_reversion: Vec<String> = pares[..i]
            .iter()
            .rev()
            .filter_map(|(o, d)| {
                fs::rename(d, o)
                    .err()
                    .map(|e| format!("{} -> {}: {e}", d.display(), o.display()))
            })
            .collect();
        let reversion = if fallos_reversion.is_empty() {
            "los archivos ya movidos se restauraron".to_string()
        } else {
            format!("la reversión falló: {}", fallos_reversion.join("; "))
        };
        return Err(Error::Invalid(format!(
            "Renombrado cancelado en {} -> {}: {e}; {reversion}",
            origen.display(),
            destino.display()
        )));
    }
    Ok(pares
        .iter()
        .map(|(_, d)| d.to_string_lossy().to_string())
        .collect())
}
