//! Project discovery, resolution, and reading (`Projects/` folder, `.PRJ` / `.VCi` files).

pub(crate) mod anadir;
pub(crate) mod doctor;
pub(crate) mod lifecycle;
pub(crate) mod temporales;

use crate::args::no_encontrado;
use crate::args::workspace_of;
use crate::args::{flag, req_str};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::object;
use crate::workspace::{resolver_ruta, walk_ext, Alcance};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

fn mtime(path: &Path) -> Option<u64> {
    fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
}

/// Is `path` a variant/comparison file for project `stem`?
fn variant_kind(path: &Path, stem: &str) -> Option<String> {
    let file_stem = path.file_stem()?.to_str()?;
    let ext = path.extension()?.to_str()?.to_uppercase();
    if file_stem != stem {
        return None;
    }
    let bytes = ext.as_bytes();
    if bytes.len() == 3
        && (ext.starts_with("VC") || ext.starts_with("CM"))
        && bytes[2].is_ascii_alphanumeric()
    {
        Some(ext)
    } else {
        None
    }
}

/// Variant extensions (`VC0`, `CM1`, ...) found next to `prj`, sorted.
fn variantes_de(prj: &Path, stem: &str) -> Vec<String> {
    let Some(parent) = prj.parent() else {
        return Vec::new();
    };
    let mut variants: Vec<String> = fs::read_dir(parent)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| variant_kind(&e.path(), stem))
        .collect();
    variants.sort();
    variants
}

/// Header metadata from the .PRJ: comment, version, meteo file, and parse error warning if any.
fn cabecera_prj(prj: &Path) -> (Value, Value, Value, Option<String>) {
    let (node, aviso) = match fs::read_to_string(prj) {
        Ok(text) => match object::parse_first(&text) {
            Some(n) => (Some(n), None),
            None => (
                None,
                Some("Archivo .PRJ no contiene objetos PVObject válidos".to_string()),
            ),
        },
        Err(e) => (None, Some(format!("No se pudo leer .PRJ: {e}"))),
    };
    let campo = |name: &str| {
        node.as_ref()
            .and_then(|n| n.field(name))
            .map_or(Value::Null, |v| json!(v))
    };
    (
        campo("Comment"),
        campo("Version"),
        campo("MeteoFileName"),
        aviso,
    )
}

/// Whether a project summary includes its variants.
#[derive(Clone, Copy, PartialEq, Eq)]
enum DetalleProyecto {
    ConVariantes,
    SinVariantes,
}

fn resumen_proyecto(workspace: &Path, prj: &Path, detalle: DetalleProyecto) -> Value {
    let stem = prj
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    let rel = prj
        .strip_prefix(workspace)
        .unwrap_or(prj)
        .to_string_lossy()
        .to_string();

    let variants = match detalle {
        DetalleProyecto::ConVariantes => variantes_de(prj, &stem),
        DetalleProyecto::SinVariantes => Vec::new(),
    };
    let (comment, version, meteo, aviso) = cabecera_prj(prj);

    let mut item = json!({
        "name": stem,
        "prj": rel,
        "modified_unix": mtime(prj),
        "comment": comment,
        "version": version,
        "meteo_file": meteo,
        "variants": variants,
    });
    if let Some(w) = aviso {
        item["aviso"] = json!(w);
    }
    item
}

pub(crate) fn listar_proyectos(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let include_variants = args
        .get("include_variants")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);

    let projects_dir = workspace.join("Projects");
    if !projects_dir.exists() {
        return Err(Error::NotFound(format!(
            "Projects folder not found: {}",
            projects_dir.display()
        )));
    }

    let mut prj_files: Vec<PathBuf> = Vec::new();
    let recorrido = walk_ext(&projects_dir, ".prj", &mut prj_files);
    prj_files.sort();

    let detalle = if include_variants {
        DetalleProyecto::ConVariantes
    } else {
        DetalleProyecto::SinVariantes
    };
    let projects: Vec<Value> = prj_files
        .iter()
        .map(|prj| resumen_proyecto(&workspace, prj, detalle))
        .collect();

    Ok(json!({
        "workspace": workspace.to_string_lossy(),
        "projects_dir": projects_dir.to_string_lossy(),
        "count": projects.len(),
        "projects": projects,
        "advertencias": recorrido.advertencias(),
    }))
}

/// Read a .PRJ or .VCi file and return its parsed object tree plus key fields.
pub(crate) fn leer_proyecto(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let name = req_str(args, &["proyecto", "project"], "")?;
    let alcance = Alcance::de_args(args);
    let path = resolve_project_file_con_alcance(&workspace, name, alcance)?;
    let text = fs::read_to_string(&path)
        .map_err(|e| Error::io(format!("Cannot read {}", path.display()), e))?;

    let roots = object::parse(&text);

    let mut summary = json!({});
    if let Some(root) = roots.first() {
        summary = json!({
            "type": root.obj_type,
            "comment": root.field("Comment"),
            "version": root.field("Version"),
            "meteo_file": root.field("MeteoFileName"),
            "site": root.find_deep("Site"),
            "latitude": root.find_deep("Latitude"),
            "longitude": root.find_deep("Longitude"),
            "pv_module": root.find_deep("PVModule"),
            "inverter": root.find_deep("GInverter"),
        });
    }

    let mut out = json!({
        "file": path.to_string_lossy(),
        "size_bytes": text.len(),
        "summary": summary,
    });
    // The full tree of a variant is ~15k tokens: opt-in only.
    if flag(args, &["completo"]) {
        out["tree"] = Value::Array(roots.iter().map(object::PvNode::to_json).collect());
    } else {
        out["hint"] = json!(
            "Pass completo:true for the full tree, or leer_objeto with 'objeto' for one object."
        );
    }
    Ok(out)
}

/// Resolve a project name (with or without extension) to an existing file within the workspace.
pub(crate) fn resolve_project_file(workspace: &Path, name: &str) -> Result<PathBuf> {
    resolve_project_file_con_alcance(workspace, name, Alcance::Workspace)
}

/// Resolve project file with confinement according to the given scope.
pub(crate) fn resolve_project_file_con_alcance(
    workspace: &Path,
    name: &str,
    alcance: Alcance,
) -> Result<PathBuf> {
    let direct = Path::new(name);
    if direct.is_absolute() || name.contains("..") {
        let safe = resolver_ruta(workspace, name, alcance)?;
        if safe.exists() {
            return Ok(safe);
        }
        let projects_dir = workspace.join("Projects");
        return Err(no_encontrado(
            "Proyecto",
            name,
            &[],
            &projects_dir.to_string_lossy(),
        ));
    }
    let projects_dir = workspace.join("Projects");
    let candidates = [
        name.to_string(),
        format!("{name}.PRJ"),
        format!("{name}.prj"),
    ];
    for c in &candidates {
        let p = projects_dir.join(c);
        if p.exists() {
            return Ok(p);
        }
    }
    // Recursive search by stem.
    let mut prjs = Vec::new();
    let avisos = walk_ext(&projects_dir, ".prj", &mut prjs).advertencias();
    for p in &prjs {
        if p.file_stem().and_then(|s| s.to_str()) == Some(name) {
            return Ok(p.clone());
        }
    }
    if !avisos.is_empty() {
        return Err(Error::NotFound(format!(
            "Proyecto '{name}' no encontrado y la búsqueda quedó incompleta: {}",
            avisos.join("; ")
        )));
    }
    let opciones: Vec<String> = prjs
        .iter()
        .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(str::to_string))
        .collect();
    Err(no_encontrado(
        "Proyecto",
        name,
        &opciones,
        &projects_dir.to_string_lossy(),
    ))
}

#[cfg(test)]
#[path = "../../tests/unit/projects/mod.rs"]
mod tests;
