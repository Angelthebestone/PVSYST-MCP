//! Project creation, deletion, and renaming.

use crate::args::{flag, req_str, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::edit;
use crate::projects::lifecycle::variantes::{
    entero_opcional, ruta_variante, stem_de, variant_files, variante_no_encontrada,
};
use crate::workspace::{mover_a_papelera, validar_nombre, Alcance};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

use super::transacciones::{escribir_par, renombrar_todo};

/// Variant bytes with the optional module / inverter / sizing overrides applied.
fn aplicar_sizing(bytes: &[u8], args: &Value) -> Result<(Vec<u8>, Vec<Value>)> {
    let mut changes: Vec<(String, Option<String>)> = Vec::new();
    if let Some(m) = args.get("modulo").and_then(Value::as_str) {
        changes.push(("PVModule".into(), Some(m.to_string())));
    }
    if let Some(i) = args.get("inversor").and_then(Value::as_str) {
        changes.push(("GInverter".into(), Some(i.to_string())));
    }
    if let Some(n) = entero_opcional(args, "n_mod_serie")? {
        changes.push(("NModSerie".into(), Some(n.to_string())));
    }
    if let Some(n) = entero_opcional(args, "n_strings")? {
        changes.push(("NStringCh".into(), Some(n.to_string())));
        changes.push(("NStrOrient1".into(), Some(n.to_string())));
    }
    if changes.is_empty() {
        return Ok((bytes.to_vec(), Vec::new()));
    }
    let (nb, outcomes) = edit::edit_fields(bytes, "pvSubArray", &changes)?;
    Ok((
        nb,
        outcomes.iter().map(edit::ChangeOutcome::to_json).collect(),
    ))
}

/// Project file bytes with the optional `comentario` applied to `pvProject`.
fn prj_con_comentario(bytes: &[u8], args: &Value) -> Result<Vec<u8>> {
    match args.get("comentario").and_then(Value::as_str) {
        Some(c) => {
            Ok(edit::edit_fields(bytes, "pvProject", &[("Comment".into(), Some(c.into()))])?.0)
        }
        None => Ok(bytes.to_vec()),
    }
}

pub(crate) fn crear_proyecto(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let plantilla = req_str(
        args,
        &["plantilla", "template"],
        "existing project to base on",
    )?;
    let nombre = req_str(args, &["nombre_nuevo", "new_name"], "")?;
    validar_nombre(nombre)?;
    let variante = args
        .get("variante")
        .and_then(|v| v.as_str())
        .unwrap_or("VC0")
        .to_uppercase();

    let alcance = Alcance::de_args(args);
    let src_prj =
        crate::projects::resolve_project_file_con_alcance(&workspace, plantilla, alcance)?;
    let src_stem = stem_de(&src_prj)?;
    let parent = src_prj
        .parent()
        .ok_or_else(|| Error::Invalid("No parent directory".into()))?
        .to_path_buf();
    let src_var = ruta_variante(&parent, &src_stem, &variante)?;
    if !src_var.exists() {
        return Err(variante_no_encontrada(&parent, &src_stem, &variante));
    }
    let new_prj = parent.join(format!("{nombre}.PRJ"));
    let new_var = ruta_variante(&parent, nombre, &variante)?;
    if (new_prj.exists() || new_var.exists()) && !flag(args, &["sobrescribir"]) {
        return Err(Error::Invalid(format!(
            "Project exists: {}",
            new_prj.display()
        )));
    }

    let prj_src = fs::read(&src_prj).map_err(|e| Error::io("Read PRJ", e))?;
    let prj_bytes = prj_con_comentario(&prj_src, args)?;
    let var_src = fs::read(&src_var).map_err(|e| Error::io("Read variant", e))?;
    let (var_bytes, applied) = aplicar_sizing(&var_src, args)?;

    let historial_id = escribir_par(
        &workspace,
        &[(new_prj.clone(), prj_bytes), (new_var.clone(), var_bytes)],
    )?;

    Ok(json!({
        "project": nombre,
        "project_file": new_prj.to_string_lossy(),
        "variant_file": new_var.to_string_lossy(),
        "based_on": src_stem,
        "applied": applied,
        "historial_id": historial_id,
    }))
}

/// Move each path into the recycle bin; returns (moved paths, failure messages).
fn mover_todos(workspace: &Path, origenes: &[PathBuf]) -> (Vec<String>, Vec<String>) {
    let mut movidos = Vec::new();
    let mut fallidos = Vec::new();
    for origen in origenes {
        match mover_a_papelera(workspace, origen) {
            Ok(destino) => movidos.push(destino.to_string_lossy().to_string()),
            Err(e) => fallidos.push(format!("{}: {e}", origen.display())),
        }
    }
    (movidos, fallidos)
}

pub(crate) fn eliminar_proyecto(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let proyecto = req_str(args, &["proyecto"], "")?;
    let alcance = Alcance::de_args(args);
    let prj = crate::projects::resolve_project_file_con_alcance(&workspace, proyecto, alcance)?;
    let stem = stem_de(&prj)?;
    let parent = prj
        .parent()
        .ok_or_else(|| Error::Invalid("No parent directory".into()))?
        .to_path_buf();

    let mut targets = vec![prj];
    targets.extend(variant_files(&parent, &stem));
    let (movidos, fallidos) = mover_todos(&workspace, &targets);
    if !fallidos.is_empty() {
        return Err(Error::Invalid(format!(
            "Papelera incompleta. Movidos: [{}]. Fallidos: [{}]",
            movidos.join(", "),
            fallidos.join("; ")
        )));
    }
    Ok(json!({ "recycled": movidos, "count": movidos.len() }))
}

pub(crate) fn renombrar_proyecto(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let proyecto = req_str(args, &["proyecto"], "")?;
    let nuevo = req_str(args, &["nombre_nuevo"], "")?;
    validar_nombre(nuevo)?;
    let alcance = Alcance::de_args(args);
    let prj = crate::projects::resolve_project_file_con_alcance(&workspace, proyecto, alcance)?;
    let stem = stem_de(&prj)?;
    let parent = prj
        .parent()
        .ok_or_else(|| Error::Invalid("No parent directory".into()))?
        .to_path_buf();

    let mut pares: Vec<(PathBuf, PathBuf)> = vec![(prj, parent.join(format!("{nuevo}.PRJ")))];
    for v in variant_files(&parent, &stem) {
        if let Some(ext) = v.extension().and_then(|s| s.to_str()) {
            let destino = parent.join(format!("{nuevo}.{ext}"));
            pares.push((v.clone(), destino));
        }
    }
    if let Some((_, existente)) = pares.iter().find(|(_, d)| d.exists()) {
        return Err(Error::Invalid(format!(
            "No se puede renombrar: ya existe {}",
            existente.display()
        )));
    }

    let renombrados = renombrar_todo(&pares)?;
    Ok(json!({ "renamed": renombrados }))
}
