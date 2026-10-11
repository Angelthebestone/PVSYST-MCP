//! History restoration and manifest validation.

use crate::error::{Error, Result};
use crate::workspace::Alcance;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

struct EntradaRestauracion {
    original: PathBuf,
    copia: PathBuf,
    existia: bool,
}

fn validar_id(id: &str) -> Result<()> {
    if id.is_empty() || id.contains(['/', '\\']) || id.contains("..") {
        return Err(Error::Invalid(format!("Id de historial inválido: '{id}'")));
    }
    Ok(())
}

struct ValidadorManifiesto<'a> {
    ws: &'a Path,
    id: &'a str,
    alcance: Alcance,
}

impl<'a> ValidadorManifiesto<'a> {
    /// Nearest existing ancestor of `joined`, canonicalised; errors when none exists.
    fn ancestro_canonico(&self, joined: &Path, orig_str: &str) -> Result<PathBuf> {
        let mut ancestro = joined;
        while !ancestro.exists() {
            ancestro = ancestro.parent().ok_or_else(|| {
                Error::Invalid(format!(
                    "Manifiesto '{}': ruta original sin ancestro existente: '{orig_str}'",
                    self.id
                ))
            })?;
        }
        ancestro
            .canonicalize()
            .map_err(|e| Error::Invalid(format!("No se pudo resolver {}: {e}", ancestro.display())))
    }

    fn validar_original(&self, obj: &serde_json::Map<String, Value>) -> Result<PathBuf> {
        let val = obj.get("original").ok_or_else(|| {
            Error::Format(format!(
                "Manifiesto '{}': entrada sin campo 'original'",
                self.id
            ))
        })?;
        let orig_str = val.as_str().ok_or_else(|| {
            Error::Format(format!(
                "Manifiesto '{}': campo 'original' no es texto: {val}",
                self.id
            ))
        })?;
        if orig_str.is_empty() {
            return Err(Error::Invalid(format!(
                "Manifiesto '{}': 'original' está vacío",
                self.id
            )));
        }

        let cand = Path::new(orig_str);
        if cand
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(Error::Invalid(format!(
                "Manifiesto '{}': ruta original contiene '..': '{orig_str}'",
                self.id
            )));
        }

        let joined = if cand.is_absolute() {
            cand.to_path_buf()
        } else {
            self.ws.join(cand)
        };

        let ws_canon = self.ws.canonicalize().map_err(|e| {
            Error::Invalid(format!("Workspace no accesible {}: {e}", self.ws.display()))
        })?;

        let real = self.ancestro_canonico(&joined, orig_str)?;

        let marcado_fuera = obj
            .get("fuera_del_workspace")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let esta_fuera = !real.starts_with(&ws_canon) || marcado_fuera;

        if esta_fuera && self.alcance != Alcance::Libre {
            return Err(Error::Invalid(format!(
                "Manifiesto '{}': ruta original fuera del workspace: '{orig_str}'. Pase permitir_fuera_del_workspace: true para restaurarla.",
                self.id
            )));
        }

        Ok(joined)
    }

    fn validar_copia(&self, dir: &Path, obj: &serde_json::Map<String, Value>) -> Result<PathBuf> {
        let val = obj.get("copia").ok_or_else(|| {
            Error::Format(format!(
                "Manifiesto '{}': entrada sin campo 'copia'",
                self.id
            ))
        })?;
        let copia_str = val.as_str().ok_or_else(|| {
            Error::Format(format!(
                "Manifiesto '{}': campo 'copia' no es texto: {val}",
                self.id
            ))
        })?;
        if copia_str.is_empty() {
            return Err(Error::Invalid(format!(
                "Manifiesto '{}': 'copia' está vacío",
                self.id
            )));
        }

        let cand = Path::new(copia_str);
        for comp in cand.components() {
            match comp {
                std::path::Component::Normal(_) => {}
                std::path::Component::Prefix(_)
                | std::path::Component::RootDir
                | std::path::Component::ParentDir
                | std::path::Component::CurDir => {
                    return Err(Error::Invalid(format!(
                        "Manifiesto '{}': 'copia' contiene componente inválido: '{copia_str}'",
                        self.id
                    )));
                }
            }
        }

        Ok(dir.join(cand))
    }

    fn validar_existia(&self, obj: &serde_json::Map<String, Value>) -> Result<bool> {
        let val = obj.get("existia").ok_or_else(|| {
            Error::Format(format!(
                "Manifiesto '{}': entrada sin campo 'existia'",
                self.id
            ))
        })?;
        val.as_bool().ok_or_else(|| {
            Error::Format(format!(
                "Manifiesto '{}': 'existia' debe ser booleano: {val}",
                self.id
            ))
        })
    }

    fn validar_entradas(&self, dir: &Path, entradas: &[Value]) -> Result<Vec<EntradaRestauracion>> {
        let mut validadas = Vec::with_capacity(entradas.len());
        for e in entradas {
            let obj = e.as_object().ok_or_else(|| {
                Error::Format(format!(
                    "Manifiesto '{}': entrada no es objeto: {e}",
                    self.id
                ))
            })?;

            let original = self.validar_original(obj)?;
            let copia = self.validar_copia(dir, obj)?;
            let existia = self.validar_existia(obj)?;

            if existia && !copia.is_file() {
                return Err(Error::Invalid(format!(
                    "Manifiesto '{}': archivo de copia no existe: '{}'",
                    self.id,
                    copia.display()
                )));
            }

            validadas.push(EntradaRestauracion {
                original,
                copia,
                existia,
            });
        }
        Ok(validadas)
    }
}

/// Write the snapshot copy of one entry back to its original path.
fn restaurar_archivo(e: &EntradaRestauracion) -> Result<()> {
    if let Some(parent) = e.original.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| Error::io(format!("No se pudo crear {}", parent.display()), err))?;
    }
    let bytes = fs::read(&e.copia)
        .map_err(|err| Error::Invalid(format!("No se pudo leer {}: {err}", e.copia.display())))?;
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

/// Restore every file recorded in snapshot `id`.
///
/// The entire manifest is validated BEFORE any side effect or pre-restore snapshot:
/// - Every `original` must resolve inside the workspace, unless `alcance == Alcance::Libre`.
/// - Every `copia` must be a relative path strictly inside the snapshot directory.
/// - Any missing or invalid fields abort immediately with zero changes to disk.
pub(crate) fn restaurar(ws: &Path, id: &str, alcance: Alcance) -> Result<Value> {
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

    let originales: Vec<PathBuf> = validadas.iter().map(|e| e.original.clone()).collect();
    let previo = super::snapshot::snapshot(ws, &originales, &format!("antes de restaurar {id}"))?;

    let mut restaurados = Vec::with_capacity(validadas.len());
    for (i, e) in validadas.iter().enumerate() {
        let resultado = if e.existia {
            restaurar_archivo(e)
        } else if e.original.is_file() {
            fs::remove_file(&e.original).map_err(|err| {
                Error::Invalid(format!("No se pudo borrar {}: {err}", e.original.display()))
            })
        } else {
            Ok(())
        };
        if let Err(err) = resultado {
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
