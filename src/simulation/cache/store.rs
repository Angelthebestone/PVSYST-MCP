#[cfg(test)]
use super::key::ClaveCache;
use crate::adapters::ledger::{ahora, carpeta_cache};
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

const MAX_ENTRADAS: usize = 50;

fn podar(workspace: &Path) -> Result<()> {
    let cache = carpeta_cache(workspace);
    let Ok(entradas) = fs::read_dir(&cache) else {
        return Ok(());
    };
    let mut dirs: Vec<_> = entradas
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir())
        .collect();
    if dirs.len() <= MAX_ENTRADAS {
        return Ok(());
    }
    dirs.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
    let borrar = dirs.len().saturating_sub(MAX_ENTRADAS);
    for entry in dirs.into_iter().take(borrar) {
        let _ = fs::remove_dir_all(entry.path());
    }
    Ok(())
}

pub(crate) fn guardar(
    workspace: &Path,
    clave: &str,
    ocf: Option<&Path>,
    rpf: Option<&Path>,
    resultado: &Value,
) -> Result<()> {
    podar(workspace)?;
    let dir = carpeta_cache(workspace).join(clave);
    fs::create_dir_all(&dir)
        .map_err(|e| Error::io(format!("No se puede crear '{}'", dir.display()), e))?;
    let mut tiene_csv = false;
    let mut tiene_pdf = false;
    if let Some(src) = ocf {
        if src.is_file() {
            fs::copy(src, dir.join("result.csv")).map_err(|e| {
                Error::io(
                    format!("No se puede copiar CSV de resultados '{}'", src.display()),
                    e,
                )
            })?;
            tiene_csv = true;
        }
    }
    if let Some(src) = rpf {
        if src.is_file() {
            fs::copy(src, dir.join("report.pdf")).map_err(|e| {
                Error::io(
                    format!("No se puede copiar PDF de resultados '{}'", src.display()),
                    e,
                )
            })?;
            tiene_pdf = true;
        }
    }
    let meta = json!({
        "clave": clave,
        "guardado_en": ahora(),
        "tiene_csv": tiene_csv,
        "tiene_pdf": tiene_pdf,
        "resultado": resultado,
    });
    let payload = serde_json::to_vec_pretty(&meta)
        .map_err(|e| Error::Format(format!("No se puede serializar meta.json: {e}")))?;
    fs::write(dir.join("meta.json"), payload).map_err(|e| {
        Error::io(
            format!("No se puede escribir meta.json en '{}'", dir.display()),
            e,
        )
    })?;
    Ok(())
}

pub(crate) fn buscar(
    workspace: &Path,
    clave: &str,
    ocf: Option<&Path>,
    rpf: Option<&Path>,
) -> Result<Option<Value>> {
    let dir = carpeta_cache(workspace).join(clave);
    let Ok(texto) = fs::read_to_string(dir.join("meta.json")) else {
        return Ok(None);
    };
    let meta: Value = serde_json::from_str(&texto)
        .map_err(|e| Error::Format(format!("meta.json de la caché inválido: {e}")))?;
    let tiene_csv = meta["tiene_csv"].as_bool().unwrap_or(false);
    let tiene_pdf = meta["tiene_pdf"].as_bool().unwrap_or(false);

    if ocf.is_some() && !tiene_csv {
        return Ok(None);
    }
    if rpf.is_some() && !tiene_pdf {
        return Ok(None);
    }

    if let Some(dest) = ocf {
        if let Some(padre) = dest.parent() {
            fs::create_dir_all(padre).map_err(|e| {
                Error::io(
                    format!("No se puede crear carpeta para '{}'", dest.display()),
                    e,
                )
            })?;
        }
        fs::copy(dir.join("result.csv"), dest).map_err(|e| {
            Error::io(
                format!("No se puede restaurar CSV en '{}'", dest.display()),
                e,
            )
        })?;
    }
    if let Some(dest) = rpf {
        if let Some(padre) = dest.parent() {
            fs::create_dir_all(padre).map_err(|e| {
                Error::io(
                    format!("No se puede crear carpeta para '{}'", dest.display()),
                    e,
                )
            })?;
        }
        fs::copy(dir.join("report.pdf"), dest).map_err(|e| {
            Error::io(
                format!("No se puede restaurar PDF en '{}'", dest.display()),
                e,
            )
        })?;
    }

    let mut out = meta.get("resultado").cloned().unwrap_or_else(|| json!({}));
    out["cache"] = json!(true);
    out["cache_clave"] = json!(clave);
    Ok(Some(out))
}

#[cfg(test)]
pub(crate) fn buscar_con_clave(
    workspace: &Path,
    clave: &ClaveCache,
    ocf: Option<&Path>,
    rpf: Option<&Path>,
) -> Result<Option<Value>> {
    match clave {
        ClaveCache::Valida(k) => buscar(workspace, k, ocf, rpf),
        ClaveCache::Inutilizable { .. } => Ok(None),
    }
}
