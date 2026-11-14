//! Project creation, deletion, and renaming.

use crate::args::{flag, req_str, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::edit;
use crate::idioma::Texto;
use crate::projects::comentario::Comentario;
use crate::projects::lifecycle::variantes::{
    entero_opcional, ruta_variante, stem_de, variant_files, variante_no_encontrada,
};
use crate::workspace::{mover_a_papelera, validar_nombre, Alcance};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

use super::plantilla::{carpeta_de_salida, resolver_plantilla, sit_pedido, variante_pedida};
use super::transacciones::{escribir_par, renombrar_todo};

/// A project whose files already exist and was not asked to overwrite them.
const PROYECTO_EXISTENTE: Texto = Texto {
    es: "'{archivo}' ya existe: pasa 'sobrescribir': true para reemplazarlo",
    en: "'{archivo}' already exists: pass 'sobrescribir': true to replace it",
};

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

/// Name of the new project: `nombre_nuevo`, or its alias `proyecto`.
fn nombre_del_nuevo_proyecto(args: &Value) -> Result<&str> {
    let nombre = req_str(
        args,
        &["nombre_nuevo", "proyecto"],
        "name of the new project",
    )?;
    validar_nombre(nombre)?;
    Ok(nombre)
}

/// Project and variant bytes of a new project, with `comentario` and sizing applied.
struct Personalizado {
    prj: Vec<u8>,
    var: Vec<u8>,
    applied: Vec<Value>,
}

/// Applies `comentario` (or the project name over a DEMO comment) and the sizing overrides.
fn personalizar(args: &Value, prj_src: &[u8], var_src: &[u8]) -> Result<Personalizado> {
    let comentario = Comentario {
        explicito: args.get("comentario").and_then(Value::as_str),
        nombre: nombre_del_nuevo_proyecto(args)?,
    };
    let (prj, mut applied) = comentario.aplicar(prj_src, "pvProject")?;
    let (sizing, sizing_applied) = aplicar_sizing(var_src, args)?;
    let (var, var_applied) = comentario.aplicar(&sizing, "pvVCalcul")?;
    applied.extend(sizing_applied);
    applied.extend(var_applied);
    Ok(Personalizado { prj, var, applied })
}

/// The site changed but the project still reads the template's weather file: the
/// simulation would run the new location with the old weather (E116's symptom).
fn aviso_meteo_de_plantilla(prj: &[u8]) -> Option<String> {
    let meteo = super::sitio::meteo_del_proyecto(prj)?;
    Some(format!(
        "the project still simulates with the template's weather file '{meteo}': give the site its own .MET with convertir_meteo or simular"
    ))
}

/// Links the requested site into the project and its variant, before either is written.
fn vincular_sitio_pedido(nuevo: &mut Personalizado, sit: Option<&Path>) -> Result<Option<String>> {
    let Some(sit) = sit else {
        return Ok(None);
    };
    nuevo.prj = super::sitio::insertar_sitio(&nuevo.prj, sit)?;
    nuevo.var = super::sitio::insertar_sitio_en_variante(&nuevo.var, sit)?;
    Ok(aviso_meteo_de_plantilla(&nuevo.prj))
}

pub(crate) fn crear_proyecto(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let plantilla = req_str(args, &["plantilla"], "existing project to base on")?;
    let nombre = nombre_del_nuevo_proyecto(args)?;
    let variante = variante_pedida(args);

    let alcance = Alcance::de_args(args);
    let src_prj = resolver_plantilla(&workspace, plantilla, alcance)?;
    let src_stem = stem_de(&src_prj)?;
    let src_parent = src_prj
        .parent()
        .ok_or_else(|| Error::Invalid("No parent directory".into()))?
        .to_path_buf();
    let parent = carpeta_de_salida(&workspace, &src_prj, &src_parent);
    let src_var = ruta_variante(&src_parent, &src_stem, &variante)?;
    if !src_var.exists() {
        return Err(variante_no_encontrada(&parent, &src_stem, &variante));
    }
    let new_prj = parent.join(format!("{nombre}.PRJ"));
    let new_var = ruta_variante(&parent, nombre, &variante)?;
    if (new_prj.exists() || new_var.exists()) && !flag(args, &["sobrescribir"]) {
        return Err(Error::Invalid(PROYECTO_EXISTENTE.texto_con(
            paths.idioma,
            &[("archivo", &new_prj.to_string_lossy())],
        )));
    }

    let prj_src = fs::read(&src_prj).map_err(|e| Error::io("Read PRJ", e))?;
    let var_src = fs::read(&src_var).map_err(|e| Error::io("Read variant", e))?;
    let mut nuevo = personalizar(args, &prj_src, &var_src)?;
    let sit = sit_pedido(&workspace, args)?;
    // The site goes into the project before the pair is written: a site that cannot be
    // linked leaves nothing behind, and it lands in this call's workspace and folder.
    let aviso_meteo = vincular_sitio_pedido(&mut nuevo, sit.as_deref())?;
    let historial_id = escribir_par(
        &workspace,
        &[(new_prj.clone(), nuevo.prj), (new_var.clone(), nuevo.var)],
    )?;

    let mut respuesta = json!({
        "project": nombre,
        "project_file": new_prj.to_string_lossy(),
        "variant_file": new_var.to_string_lossy(),
        "based_on": src_stem,
        "sitio": sit.as_ref().map(|p| p.to_string_lossy().to_string()),
        "variant_site_updated": sit.is_some(),
        "applied": nuevo.applied,
        "historial_id": historial_id,
    });
    if let Some(aviso) = aviso_meteo {
        respuesta["warning"] = json!(aviso);
    }
    Ok(respuesta)
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

/// Files of one variant and its results (`Results/<stem>_<VARIANTE>.csv|pdf`).
/// A variant that does not exist is an error, never a silent project delete.
fn archivos_de_variante(workspace: &Path, prj: &Path, variante: &str) -> Result<Vec<PathBuf>> {
    let stem = stem_de(prj)?;
    let parent = prj
        .parent()
        .ok_or_else(|| Error::Invalid("No parent directory".into()))?;
    let variante = variante.to_uppercase();
    let vpath = ruta_variante(parent, &stem, &variante)?;
    if !vpath.exists() {
        return Err(variante_no_encontrada(parent, &stem, &variante));
    }
    let base = format!("{stem}_{variante}");
    let resultados = ["csv", "pdf"]
        .iter()
        .map(|ext| workspace.join("Results").join(format!("{base}.{ext}")))
        .filter(|p| p.exists());
    Ok(std::iter::once(vpath).chain(resultados).collect())
}

/// `proyecto accion:eliminar`: without `variante` the whole project (and every
/// variant) goes to the recycle bin; with `variante` only that variant and its results.
pub(crate) fn eliminar_proyecto(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let proyecto = req_str(args, &["proyecto"], "")?;
    let alcance = Alcance::de_args(args);
    let prj = crate::projects::resolve_project_file_con_alcance(&workspace, proyecto, alcance)?;
    let targets = match crate::args::opt_str(args, &["variante"]) {
        Some(variante) => archivos_de_variante(&workspace, &prj, variante)?,
        None => {
            let stem = stem_de(&prj)?;
            let parent = prj
                .parent()
                .ok_or_else(|| Error::Invalid("No parent directory".into()))?;
            std::iter::once(prj.clone())
                .chain(variant_files(parent, &stem)?)
                .collect()
        }
    };
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
    for v in variant_files(&parent, &stem)? {
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
    let mut respuesta = json!({ "renamed": renombrados });
    // The rename already happened: a comment that cannot be updated is reported, not fatal.
    if let Err(e) = actualizar_comentario(&pares[0].1, &stem, nuevo) {
        respuesta["aviso"] = json!(format!(
            "The project was renamed but its Comment still says '{stem}': {e}"
        ));
    }
    Ok(respuesta)
}

/// The PRJ's `Comment` follows the new name when it was the old one: PVsyst lists the
/// project by it, so a renamed project kept announcing its old name.
fn actualizar_comentario(prj: &Path, anterior: &str, nuevo: &str) -> Result<()> {
    let bytes =
        fs::read(prj).map_err(|e| Error::io(format!("Cannot read {}", prj.display()), e))?;
    let texto = String::from_utf8_lossy(&bytes);
    let raiz = crate::format::object::parse_first(&texto)
        .ok_or_else(|| Error::Format("El PRJ no contiene un PVObject".into()))?;
    if raiz.field("Comment") != Some(anterior) {
        return Ok(());
    }
    let cambios = [("Comment".to_string(), Some(nuevo.to_string()))];
    let (nuevos, _) = edit::edit_fields(&bytes, &raiz.obj_type, &cambios)?;
    crate::history::escribir_atomico(prj, &nuevos)
}

#[cfg(test)]
#[path = "../../../tests/unit/projects/lifecycle/creacion.rs"]
mod tests;
