//! Component-database operations (ComposPV/): list, read, create, recycle.
//!
//! Covers every component kind PVsyst uses, keyed by extension.

pub(crate) mod buscar;
mod cambios;
pub(crate) mod ficha;
pub(crate) mod pan;
mod resolucion;
pub(crate) mod sobrescritura;
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
use crate::format::object;
use crate::history;
use crate::idioma::{Idioma, Texto};
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

/// Spanish/English names accepted as `tipo` in addition to the extension codes.
const ALIAS_TIPO: &[(&str, &str)] = &[
    ("MODULO", "PAN"),
    ("MODULOS", "PAN"),
    ("MODULES", "PAN"),
    ("INVERSOR", "OND"),
    ("INVERSORES", "OND"),
    ("INVERTERS", "OND"),
    ("BATERIA", "BTR"),
    ("BATERIAS", "BTR"),
    ("BATTERIES", "BTR"),
    ("BOMBA", "PMP"),
    ("BOMBAS", "PMP"),
    ("PUMPS", "PMP"),
];

/// Canonical `(extension, ComposPV folder)` for a `tipo` given as extension, PVsyst folder name or alias.
pub(crate) fn tipo_canonico(tipo: &str) -> Option<(&'static str, &'static str)> {
    let up = tipo.trim_start_matches('.').to_uppercase();
    let directo = KINDS
        .iter()
        .copied()
        .find(|(e, f)| *e == up || f.to_uppercase() == up);
    directo.or_else(|| {
        let (_, ext) = ALIAS_TIPO.iter().copied().find(|(a, _)| *a == up)?;
        KINDS.iter().copied().find(|(e, _)| *e == ext)
    })
}

pub(crate) fn folder_for(kind: &str) -> Option<&'static str> {
    tipo_canonico(kind).map(|(_, f)| f)
}

/// Component file name with the kind's extension when the caller left it out.
///
/// `listar componentes` filters by extension, so a component without one would
/// be invisible to every later call.
pub(crate) fn nombre_con_extension(nombre: &str, kind: &str) -> String {
    if Path::new(nombre).extension().is_some() {
        return nombre.to_string();
    }
    format!("{nombre}.{}", kind.to_uppercase())
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

const TIPO_DESCONOCIDO: Texto = Texto {
    es: "Tipo de componente desconocido '{tipo}'. Conocidos: {conocidos}",
    en: "Unknown component kind '{tipo}'. Known: {conocidos}",
};

/// The rejection of a component kind that is not one of [`KINDS`].
pub(crate) fn tipo_desconocido(tipo: &str, idioma: Idioma) -> Error {
    let conocidos = KINDS.iter().map(|(e, _)| *e).collect::<Vec<_>>().join(", ");
    Error::Invalid(TIPO_DESCONOCIDO.texto_con(idioma, &[("tipo", tipo), ("conocidos", &conocidos)]))
}

pub(crate) fn listar_componentes(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let kind = args
        .get("tipo")
        .or_else(|| args.get("kind"))
        .and_then(|v| v.as_str())
        .unwrap_or("PAN");
    let (canon, folder) =
        tipo_canonico(kind).ok_or_else(|| tipo_desconocido(kind, paths.idioma))?;

    let mut dirs = vec![compos_dir(&workspace, folder)];
    // Fallback to the bundled read-only database.
    dirs.push(crate::config::compos_ro().join(folder));

    let (items, source) = componentes_de(&dirs, &canon.to_lowercase());
    let mut out = json!({
        "kind": kind.to_uppercase(),
        "folder": source,
        "count": items.len(),
        "components": items,
    });
    // PVsyst ships only some component kinds: an absent folder is not "none found".
    if !dirs.iter().any(|d| d.exists()) {
        out["aviso"] = json!(SIN_CARPETA.texto_con(
            Idioma::de_entorno(),
            &[("carpeta", folder), ("kind", canon)],
        ));
    }
    Ok(out)
}

/// Files of `ext` in the first folder that has any, and the folder to report: the one
/// that provided them, or the one PVsyst would use when none exists.
fn componentes_de(dirs: &[PathBuf], ext: &str) -> (Vec<Value>, String) {
    let existentes: Vec<&PathBuf> = dirs.iter().filter(|d| d.exists()).collect();
    for dir in &existentes {
        let found = files_with_ext(dir, ext);
        if found.is_empty() {
            continue;
        }
        let items = found
            .iter()
            .map(|p| {
                json!({
                    "file": p.file_name().and_then(|s| s.to_str()),
                    "size_bytes": fs::metadata(p).map_or(0, |m| m.len()),
                })
            })
            .collect();
        return (items, dir.to_string_lossy().to_string());
    }
    let source = existentes
        .first()
        .map(|d| d.to_string_lossy().to_string())
        .or_else(|| dirs.first().map(|d| d.to_string_lossy().to_string()))
        .unwrap_or_default();
    (Vec::new(), source)
}

/// No `ComposPV` folder for this kind, in the workspace or in DataRO.
const SIN_CARPETA: Texto = Texto {
    es: "No existe la carpeta ComposPV\\{carpeta} (ni en el workspace ni en DataRO): no hay {kind} que listar. PVsyst no incluye todas las clases de componente; copia el archivo en esa carpeta",
    en: "There is no ComposPV\\{carpeta} folder (neither in the workspace nor in DataRO): there are no {kind} to list. PVsyst does not ship every component kind; copy the file into that folder",
};

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
        // A truncated file would nest its objects wrongly: refuse it instead of listing them.
        let roots = object::parse_completo(&text, &path)?;
        out["objects"] = Value::Array(roots.iter().map(object::PvNode::to_json).collect());
    } else {
        out["note"] = json!(
            "Binary or legacy component; use leer on the .PAN module, or editar for text files."
        );
    }
    Ok(out)
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
    let applied = cambios::aplicar_cambios(&mut bytes, args)?;
    if applied.is_empty() {
        return Err(Error::Invalid(
            "editar necesita 'cambios' (o 'campos') con al menos un campo: no se escribió nada"
                .into(),
        ));
    }
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
    let folder = folder_for(&kind).ok_or_else(|| tipo_desconocido(&kind, paths.idioma))?;

    let src = resolve_component(&workspace, &consulta(&kind, plantilla, args))?;
    let mut bytes = leer_bytes_componente(&src)?;

    let applied = cambios::aplicar_cambios(&mut bytes, args)?;

    let out_dir = compos_dir(&workspace, folder);
    fs::create_dir_all(&out_dir)
        .map_err(|e| Error::io(format!("Cannot create {}", out_dir.display()), e))?;
    let name = nombre_con_extension(nuevo, &kind);
    let target = out_dir.join(&name);
    sobrescritura::exigir_sobrescribir(&target, args)?;
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
