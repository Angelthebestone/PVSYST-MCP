//! History restoration: validate a manifest, snapshot, write files, revert on failure.

use super::validador::{validar_id, EntradaRestauracion, ValidadorManifiesto};
use crate::config::Alcance;
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// Restore one entry: rewrite the file, or delete it when it did not exist yet.
fn aplicar_entrada(e: &EntradaRestauracion) -> Result<()> {
    if e.existia {
        return restaurar_archivo(e);
    }
    if !e.original.is_file() {
        return Ok(());
    }
    fs::remove_file(&e.original)
        .map_err(|err| Error::io(format!("No se pudo borrar {}", e.original.display()), err))
}

/// Write the snapshot copy of one entry back to its original path.
fn restaurar_archivo(e: &EntradaRestauracion) -> Result<()> {
    if let Some(parent) = e.original.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| Error::io(format!("No se pudo crear {}", parent.display()), err))?;
    }
    let bytes = fs::read(&e.copia)
        .map_err(|err| Error::io(format!("No se pudo leer {}", e.copia.display()), err))?;
    super::escritura_atomica::escribir_atomico(&e.original, &bytes)
}

/// Undo the first `aplicadas` restore entries using recovery snapshot `previo`,
/// whose copies are recorded in the same order as those entries. Returns a
/// description of the reversal and, when it was partial, the failed reversions.
fn revertir_aplicadas(
    ws: &Path,
    previo: &str,
    aplicadas: &[EntradaRestauracion],
) -> (String, Vec<String>) {
    let dir = super::snapshot::history_root(ws).join(previo);
    let mut fallos = Vec::new();
    for (i, e) in aplicadas.iter().enumerate().rev() {
        let copia = dir.join(format!("{i:03}.bin"));
        let resultado = if copia.is_file() {
            fs::read(&copia)
                .map_err(|err| Error::io(format!("No se pudo leer {}", copia.display()), err))
                .and_then(|bytes| super::escritura_atomica::escribir_atomico(&e.original, &bytes))
        } else if e.original.is_file() {
            fs::remove_file(&e.original).map_err(|err| {
                Error::io(format!("No se pudo borrar {}", e.original.display()), err)
            })
        } else {
            Ok(())
        };
        if let Err(err) = resultado {
            fallos.push(format!("{}: {err}", e.original.display()));
        }
    }
    let reversion = if fallos.is_empty() {
        format!("las entradas aplicadas se revirtieron (snapshot previo: {previo})")
    } else {
        format!("la reversión quedó incompleta; snapshot de recuperación: {previo}")
    };
    (reversion, fallos)
}

/// One restore request: which snapshot, under which path scope, and whether to rehearse it.
pub(crate) struct PeticionRestauracion<'a> {
    pub(crate) ws: &'a Path,
    pub(crate) id: &'a str,
    pub(crate) alcance: Alcance,
    pub(crate) dry_run: bool,
}

/// What restoring one entry would do to its file.
fn accion_prevista(e: &EntradaRestauracion) -> &'static str {
    if !e.existia {
        return if e.original.is_file() {
            "borrar"
        } else {
            "sin_cambio"
        };
    }
    let iguales = fs::read(&e.original)
        .is_ok_and(|actual| fs::read(&e.copia).is_ok_and(|copia| actual == copia));
    if iguales {
        "sin_cambio"
    } else {
        "restaurar"
    }
}

/// Rehearsal of [`restaurar`]: validates everything and reports what would change.
fn ensayar(id: &str, validadas: &[EntradaRestauracion]) -> Value {
    let acciones: Vec<Value> = validadas
        .iter()
        .map(|e| {
            json!({
                "archivo": e.original.to_string_lossy(),
                "accion": accion_prevista(e),
            })
        })
        .collect();
    let cambiarian: Vec<&Value> = acciones
        .iter()
        .filter(|a| a["accion"] != "sin_cambio")
        .collect();
    json!({
        "simulado": true,
        "restauraria": id,
        "cambiarian": cambiarian.len(),
        "acciones": acciones,
    })
}

/// Restore every file recorded in snapshot `id`, or report what a restore would do.
///
/// The entire manifest is validated BEFORE any side effect or pre-restore snapshot:
/// - Every `original` must resolve inside the workspace, unless `alcance == Alcance::Libre`.
/// - Every `copia` must be a relative path strictly inside the snapshot directory.
/// - Any missing or invalid fields abort immediately with zero changes to disk.
pub(crate) fn restaurar(peticion: &PeticionRestauracion<'_>) -> Result<Value> {
    let (ws, id, alcance) = (peticion.ws, peticion.id, peticion.alcance);
    validar_id(id)?;

    let dir = super::snapshot::history_root(ws).join(id);
    if !dir.is_dir() {
        return Err(Error::NotFound(format!("Snapshot '{id}' no encontrado")));
    }

    let manifest_path = dir.join("manifest.json");
    let texto = fs::read_to_string(&manifest_path)
        .map_err(|_| Error::NotFound(format!("Snapshot '{id}' no contiene manifest.json")))?;
    let manifest: Value = serde_json::from_str(&texto)
        .map_err(|e| Error::Format(format!("Manifiesto corrupto en '{id}': {e}")))?;

    let entradas = manifest
        .get("archivos")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::Format(format!("Manifiesto '{id}' sin lista de archivos")))?;

    // Crucial: Complete manifest validation occurs BEFORE any side effect.
    let validador = ValidadorManifiesto { ws, id, alcance };
    let validadas = validador.validar_entradas(&dir, entradas)?;

    if peticion.dry_run {
        return Ok(ensayar(id, &validadas));
    }
    aplicar(ws, id, &validadas)
}

/// Apply a validated manifest, reverting what was already written when an entry fails.
fn aplicar(ws: &Path, id: &str, validadas: &[EntradaRestauracion]) -> Result<Value> {
    let originales: Vec<PathBuf> = validadas.iter().map(|e| e.original.clone()).collect();
    let previo = super::snapshot::snapshot(ws, &originales, &format!("antes de restaurar {id}"))?;

    let mut restaurados = Vec::with_capacity(validadas.len());
    for (i, e) in validadas.iter().enumerate() {
        if let Err(err) = aplicar_entrada(e) {
            let (reversion, fallos) = revertir_aplicadas(ws, &previo, &validadas[..i]);
            let detalle = if fallos.is_empty() {
                reversion
            } else {
                format!("{reversion}; reversiones fallidas: [{}]", fallos.join("; "))
            };
            return Err(Error::Invalid(format!(
                "Restauración de '{id}' cancelada en {}: {err}; {detalle}",
                e.original.display()
            )));
        }
        restaurados.push(e.original.to_string_lossy().to_string());
    }

    Ok(json!({
        "restaurado": id,
        "historial_id_previo": previo,
        "archivos": restaurados,
    }))
}
