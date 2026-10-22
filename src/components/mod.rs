//! Component-database operations (ComposPV/): list, read, create, recycle.
//!
//! Covers every component kind PVsyst uses, keyed by extension.

pub(crate) mod buscar;
pub(crate) mod ficha;
pub(crate) mod pan;
mod resolucion;
#[cfg(test)]
#[path = "../../tests/unit/components/tests.rs"]
mod tests;

pub(crate) use resolucion::{
    es_base_solo_lectura, files_with_ext, leer_bytes_componente, resolve_component,
    ConsultaComponente,
};

use crate::args::{missing, req_str, tipo_de, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::edit;
use crate::format::object;
use crate::history;
use crate::workspace::{mover_a_papelera, validar_nombre, Alcance};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// extension (upper) -> folder under ComposPV (as PVsyst names it)
pub(crate) const KINDS: &[(&str, &str)] = &[
    ("PAN", "PVModules"),
    ("OND", "Inverters"),
    ("BTR", "Batteries"),
    ("PMP", "Pumps"),
    ("RLT", "Regulators"),
    ("RLP", "RegulPmp"),
    ("GEN", "Gensets"),
    ("REV", "Sellers"),
    ("CLI", "Clients"),
];

pub(crate) fn folder_for(kind: &str) -> Option<&'static str> {
    let up = kind.trim_start_matches('.').to_uppercase();
    KINDS.iter().find(|(e, _)| *e == up).map(|(_, f)| *f)
}

/// Resolve the workspace ComposPV sub-folder, honouring PVsyst's casing.
pub(crate) fn compos_dir(workspace: &Path, folder: &str) -> PathBuf {
    let base = workspace.join("ComposPV");
    let direct = base.join(folder);
    if direct.exists() {
        return direct;
    }
    // Case-insensitive scan (PVsyst uses e.g. PVmodules vs PVModules).
    if let Ok(entries) = fs::read_dir(&base) {
        for e in entries.flatten() {
            if e.file_name().to_string_lossy().eq_ignore_ascii_case(folder) {
                return e.path();
            }
        }
    }
    direct
}

pub(crate) fn listar_componentes(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let kind = args
        .get("tipo")
        .or_else(|| args.get("kind"))
        .and_then(|v| v.as_str())
        .unwrap_or("PAN");
    let folder = folder_for(kind).ok_or_else(|| {
        Error::Invalid(format!(
            "Unknown component kind '{}'. Known: {}",
            kind,
            KINDS.iter().map(|(e, _)| *e).collect::<Vec<_>>().join(", ")
        ))
    })?;

    let mut dirs = vec![compos_dir(&workspace, folder)];
    // Fallback to the bundled read-only database.
    dirs.push(crate::config::compos_ro().join(folder));

    let ext = kind.trim_start_matches('.').to_lowercase();
    let mut items: Vec<Value> = Vec::new();
    let mut source = String::new();
    for dir in dirs.iter().filter(|d| d.exists()) {
        let found = files_with_ext(dir, &ext);
        if !found.is_empty() {
            source = dir.to_string_lossy().to_string();
            items = found
                .iter()
                .map(|p| {
                    json!({
                        "file": p.file_name().and_then(|s| s.to_str()),
                        "size_bytes": fs::metadata(p).map_or(0, |m| m.len()),
                    })
                })
                .collect();
            break;
        }
    }
    Ok(json!({
        "kind": kind.to_uppercase(),
        "folder": source,
        "count": items.len(),
        "components": items,
    }))
}

pub(crate) fn leer_componente(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let file = req_str(args, &["componente"], "")?;
    let kind = tipo_de(args, file).unwrap_or_else(|| "PAN".to_string());

    let path = resolve_component(&workspace, &consulta(&kind, file, args))?;
    let bytes = leer_bytes_componente(&path)?;
    let text = String::from_utf8_lossy(&bytes);

    let mut out = json!({
        "file": path.to_string_lossy(),
        "kind": kind.to_uppercase(),
        "size_bytes": bytes.len(),
        "is_text": object::is_pvobject(&text),
    });
    if object::is_pvobject(&text) {
        let roots = object::parse(&text);
        out["objects"] = Value::Array(roots.iter().map(object::PvNode::to_json).collect());
    } else {
        out["note"] = json!("Binary or legacy component; use leer_modulo for .PAN, or editar_objeto for text files.");
    }
    Ok(out)
}

/// Apply `args.cambios` to the root object of a component, in place.
fn aplicar_cambios(bytes: &mut Vec<u8>, args: &Value) -> Result<Vec<Value>> {
    let text = String::from_utf8_lossy(bytes);
    if !object::is_pvobject(&text) {
        if pan::es_pan_legacy(bytes) {
            let Some(cambios) = args
                .get("cambios")
                .or_else(|| args.get("campos"))
                .and_then(|v| v.as_object())
            else {
                return Ok(Vec::new());
            };
            let changes = edit::changes_from_json(cambios);
            if changes.is_empty() {
                return Ok(Vec::new());
            }
            let (new_bytes, outcomes) = pan::parchear_campos_legacy(bytes, &changes)?;
            *bytes = new_bytes;
            return Ok(outcomes.iter().map(edit::ChangeOutcome::to_json).collect());
        }
        let cambios_vacios = args
            .get("cambios")
            .and_then(|v| v.as_object())
            .is_none_or(|m| m.is_empty());
        return if cambios_vacios {
            Ok(Vec::new())
        } else {
            Err(Error::Invalid(
                "Target component is binary; field editing is not supported.".into(),
            ))
        };
    }
    let (Some(root), Some(cambios)) = (
        object::parse_first(&text),
        args.get("cambios").and_then(|v| v.as_object()),
    ) else {
        return Ok(Vec::new());
    };
    let changes = edit::changes_from_json(cambios);
    if changes.is_empty() {
        return Ok(Vec::new());
    }
    let (new_bytes, outcomes) = edit::edit_fields(bytes, &root.obj_type, &changes)?;
    *bytes = new_bytes;
    Ok(outcomes.iter().map(edit::ChangeOutcome::to_json).collect())
}

/// Edit an existing component in place with history snapshot.
pub(crate) fn editar_componente(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let file = req_str(args, &["componente"], "")?;
    let kind = tipo_de(args, file).unwrap_or_else(|| "PAN".to_string());
    let path = resolve_component(&workspace, &consulta(&kind, file, args))?;
    if es_base_solo_lectura(&path) {
        return Err(Error::Invalid(format!(
            "{} pertenece a la base de solo lectura de PVsyst y no se puede editar",
            path.display()
        )));
    }
    let mut bytes = leer_bytes_componente(&path)?;
    let applied = aplicar_cambios(&mut bytes, args)?;
    let historial_id =
        history::snapshot(&workspace, std::slice::from_ref(&path), "editar_componente")?;
    history::escribir_atomico(&path, &bytes)?;
    Ok(json!({
        "file": path.to_string_lossy(),
        "kind": kind.to_uppercase(),
        "changes_applied": applied,
        "historial_id": historial_id,
    }))
}

/// Create a new component by copying `plantilla` and applying field changes.
pub(crate) fn crear_componente(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let plantilla = req_str(args, &["plantilla"], "existing component")?;
    let nuevo = req_str(args, &["nombre_nuevo"], "")?;
    validar_nombre(nuevo)?;
    let kind = tipo_de(args, plantilla).ok_or_else(|| missing("tipo", "p.ej. \"PAN\""))?;
    let folder = folder_for(&kind)
        .ok_or_else(|| Error::Invalid(format!("Unknown component kind '{kind}'")))?;

    let src = resolve_component(&workspace, &consulta(&kind, plantilla, args))?;
    let mut bytes = leer_bytes_componente(&src)?;

    let applied = aplicar_cambios(&mut bytes, args)?;

    let out_dir = compos_dir(&workspace, folder);
    fs::create_dir_all(&out_dir)
        .map_err(|e| Error::io(format!("Cannot create {}", out_dir.display()), e))?;
    let mut name = nuevo.to_string();
    if Path::new(&name).extension().is_none() {
        name = format!("{}.{}", name, kind.to_uppercase());
    }
    let target = out_dir.join(&name);
    let historial_id = history::reemplazar(&workspace, &target, &bytes, "crear_componente")?;

    Ok(json!({
        "created": target.to_string_lossy(),
        "from": src.to_string_lossy(),
        "kind": kind.to_uppercase(),
        "changes_applied": applied,
        "historial_id": historial_id,
    }))
}

pub(crate) fn borrar_componente(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let file = req_str(args, &["componente"], "")?;
    let kind = tipo_de(args, file).unwrap_or_else(|| "PAN".to_string());
    let src = resolve_component(&workspace, &consulta(&kind, file, args))?;
    if es_base_solo_lectura(&src) {
        return Err(Error::Invalid(format!(
            "{} pertenece a la base de solo lectura de PVsyst y no se puede borrar",
            src.display()
        )));
    }

    // Non-destructive by default: move to the workspace recycle bin.
    let target = mover_a_papelera(&workspace, &src)?;
    Ok(json!({
        "recycled": src.to_string_lossy(),
        "moved_to": target.to_string_lossy(),
    }))
}

/// A component named by the caller, resolved under the caller's path scope.
fn consulta<'a>(kind: &'a str, name: &'a str, args: &Value) -> ConsultaComponente<'a> {
    ConsultaComponente {
        kind,
        name,
        alcance: Alcance::de_args(args),
    }
}
