//! Variant identification, file resolution, opening and creation.

use crate::args::{flag, no_encontrado, req_str, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::history;
use crate::workspace::{validar_nombre, Alcance};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// Is `nombre` a variant identifier: `VCn` or `CMn` (case-insensitive), with
/// `n` one ASCII alphanumeric character?
pub(crate) fn es_variante_valida(nombre: &str) -> bool {
    let b = nombre.as_bytes();
    b.len() == 3
        && (b[..2].eq_ignore_ascii_case(b"VC") || b[..2].eq_ignore_ascii_case(b"CM"))
        && b[2].is_ascii_alphanumeric()
}

/// Variant file `<parent>/<stem>.<variante>`; rejects anything that is not a
/// variant identifier so user input cannot escape the project folder.
pub(crate) fn ruta_variante(parent: &Path, stem: &str, variante: &str) -> Result<PathBuf> {
    if !es_variante_valida(variante) {
        return Err(Error::Invalid(format!(
            "Variante inválida '{variante}': debe ser VCn o CMn (n alfanumérico)"
        )));
    }
    Ok(parent.join(format!("{stem}.{variante}")))
}

/// File stem of a project path as an owned string.
pub(crate) fn stem_de(path: &Path) -> Result<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(str::to_string)
        .ok_or_else(|| Error::Invalid("Nombre de proyecto inválido".into()))
}

/// Parse a positive integer that may arrive as a JSON number or digit-only string.
pub(crate) fn parse_entero(v: &Value) -> Option<u64> {
    let n = match v {
        Value::Number(n) => n.as_u64(),
        Value::String(s) if !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) => {
            s.parse().ok()
        }
        _ => None,
    };
    n.filter(|n| *n > 0)
}

/// Optional positive integer argument `clave` (`None` when absent or null).
pub(crate) fn entero_opcional(args: &Value, clave: &str) -> Result<Option<u64>> {
    let Some(v) = args.get(clave).filter(|v| !v.is_null()) else {
        return Ok(None);
    };
    parse_entero(v)
        .map(Some)
        .ok_or_else(|| Error::Invalid(format!("'{clave}' debe ser un entero positivo")))
}

/// Variant files `<stem>.VCn` / `<stem>.CMn` in `parent`, sorted.
pub(crate) fn variant_files(parent: &Path, stem: &str) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = fs::read_dir(parent)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.file_stem().and_then(|s| s.to_str()) == Some(stem))
        .filter(|p| {
            p.extension()
                .and_then(|s| s.to_str())
                .is_some_and(es_variante_valida)
        })
        .collect();
    v.sort();
    v
}

/// "Variant not found" error listing the variants that do exist beside the project.
pub(crate) fn variante_no_encontrada(parent: &Path, stem: &str, pedido: &str) -> Error {
    let opciones: Vec<String> = variant_files(parent, stem)
        .iter()
        .filter_map(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(str::to_uppercase)
        })
        .collect();
    no_encontrado("Variante", pedido, &opciones, &parent.to_string_lossy())
}

/// A variant file located from `proyecto` / `variante` arguments and read into memory.
pub(crate) struct VarianteAbierta {
    pub(crate) workspace: PathBuf,
    pub(crate) stem: String,
    pub(crate) variante: String,
    pub(crate) vpath: PathBuf,
    pub(crate) bytes: Vec<u8>,
}

pub(crate) fn abrir_variante(paths: &Paths, args: &Value) -> Result<VarianteAbierta> {
    let workspace = workspace_of(paths, args);
    let proyecto = req_str(args, &["proyecto"], "")?;
    let variante = args
        .get("variante")
        .and_then(|v| v.as_str())
        .unwrap_or("VC0")
        .to_uppercase();
    let alcance = Alcance::de_args(args);
    let prj = crate::projects::resolve_project_file_con_alcance(&workspace, proyecto, alcance)?;
    let stem = stem_de(&prj)?;
    let parent = prj
        .parent()
        .ok_or_else(|| Error::Invalid("No parent directory".into()))?;
    let vpath = ruta_variante(parent, &stem, &variante)?;
    if !vpath.exists() {
        return Err(variante_no_encontrada(parent, &stem, &variante));
    }
    let bytes = fs::read(&vpath)
        .map_err(|e| Error::io(format!("Cannot read variant {}", vpath.display()), e))?;
    Ok(VarianteAbierta {
        workspace,
        stem,
        variante,
        vpath,
        bytes,
    })
}

pub(crate) fn crear_variante(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let proyecto = req_str(args, &["proyecto"], "")?;
    let origen = args
        .get("variante_origen")
        .and_then(|v| v.as_str())
        .unwrap_or("VC0")
        .to_uppercase();
    let nueva = req_str(args, &["variante_nueva"], "e.g. \"VC1\"")?.to_uppercase();
    validar_nombre(&nueva)?;
    if !es_variante_valida(&nueva) {
        return Err(Error::Invalid(format!(
            "Nombre de variante inválido '{nueva}': debe ser VCn o CMn (n alfanumérico)"
        )));
    }

    let alcance = Alcance::de_args(args);
    let prj = crate::projects::resolve_project_file_con_alcance(&workspace, proyecto, alcance)?;
    let stem = stem_de(&prj)?;
    let parent = prj
        .parent()
        .ok_or_else(|| Error::Invalid("No parent dir".into()))?
        .to_path_buf();

    let src = ruta_variante(&parent, &stem, &origen)?;
    if !src.exists() {
        return Err(variante_no_encontrada(&parent, &stem, &origen));
    }
    let dst = ruta_variante(&parent, &stem, &nueva)?;
    if dst.exists() && !flag(args, &["sobrescribir"]) {
        return Err(Error::Invalid(format!(
            "Target variant exists: {}",
            dst.display()
        )));
    }
    let bytes = fs::read(&src)
        .map_err(|e| Error::io(format!("Cannot read variant {}", src.display()), e))?;
    let historial_id = history::reemplazar(&workspace, &dst, &bytes, "crear_variante")?;
    let todas: Vec<String> = variant_files(&parent, &stem)
        .iter()
        .filter_map(|p| p.file_name().and_then(|s| s.to_str()).map(str::to_string))
        .collect();
    Ok(json!({
        "project": stem,
        "created": dst.to_string_lossy(),
        "from": src.to_string_lossy(),
        "historial_id": historial_id,
        "all_variants": todas,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_validate_variant_identifier_format() {
        assert!(es_variante_valida("VC0"));
        assert!(es_variante_valida("cm9"));
        assert!(es_variante_valida("VCA"));
        assert!(!es_variante_valida("VC"));
        assert!(!es_variante_valida("VC10"));
        assert!(!es_variante_valida("XX1"));
        assert!(!es_variante_valida("VC.."));
        assert!(!es_variante_valida("V/1"));
    }

    #[test]
    fn should_reject_path_escape_in_variant_route() {
        let parent = Path::new("proj");
        assert!(ruta_variante(parent, "Planta", "..\\x").is_err());
        assert_eq!(
            ruta_variante(parent, "Planta", "VC1").expect("valid"),
            parent.join("Planta.VC1")
        );
    }

    #[test]
    fn should_parse_only_positive_integers() {
        assert_eq!(parse_entero(&json!(15)), Some(15));
        assert_eq!(parse_entero(&json!("15")), Some(15));
        assert_eq!(parse_entero(&json!("1a")), None);
        assert_eq!(parse_entero(&json!("")), None);
        assert_eq!(parse_entero(&json!(0)), None);
        assert_eq!(parse_entero(&json!(2.5)), None);
    }
}
