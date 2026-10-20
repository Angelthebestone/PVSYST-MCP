//! Manifest validation for history restoration.

use crate::config::Alcance;
use crate::error::{Error, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};

pub(super) struct EntradaRestauracion {
    pub(super) original: PathBuf,
    pub(super) copia: PathBuf,
    pub(super) existia: bool,
}

pub(super) fn validar_id(id: &str) -> Result<()> {
    if id.is_empty() || id.contains(['/', '\\']) || id.contains("..") {
        return Err(Error::Invalid(format!("Id de historial inválido: '{id}'")));
    }
    Ok(())
}

pub(super) struct ValidadorManifiesto<'a> {
    pub(super) ws: &'a Path,
    pub(super) id: &'a str,
    pub(super) alcance: Alcance,
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
            .map_err(|e| Error::io(format!("No se pudo resolver {}", ancestro.display()), e))
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

        let ws_canon = self
            .ws
            .canonicalize()
            .map_err(|e| Error::io(format!("Workspace no accesible {}", self.ws.display()), e))?;

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

    pub(super) fn validar_entradas(
        &self,
        dir: &Path,
        entradas: &[Value],
    ) -> Result<Vec<EntradaRestauracion>> {
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
                return Err(Error::NotFound(format!(
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
