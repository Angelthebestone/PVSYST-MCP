//! Project discovery, resolution, and reading (`Projects/` folder, `.PRJ` / `.VCi` files).

pub(crate) mod anadir;
pub(crate) mod comentario;
pub(crate) mod doctor;
pub(crate) mod gui;
pub(crate) mod lifecycle;
mod resolucion;
pub(crate) mod temporales;
pub(crate) mod tipo_sistema;

use crate::args::workspace_of;
use crate::args::{flag, req_str};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::object;
use crate::workspace::{walk_ext, Alcance};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) use resolucion::{resolve_project_file, resolve_project_file_con_alcance};

/// Variant extensions (`VC0`, `CM1`, ...) found next to `prj`, sorted; an unreadable
/// folder lists none (logged): this only feeds listings and hints.
pub(crate) fn variantes_de(prj: &Path, stem: &str) -> Vec<String> {
    let Some(parent) = prj.parent() else {
        return Vec::new();
    };
    let archivos = lifecycle::variantes::variant_files(parent, stem).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "no se pudieron listar las variantes");
        Vec::new()
    });
    let mut variantes: Vec<String> = archivos
        .iter()
        .filter_map(|p| p.extension()?.to_str().map(str::to_uppercase))
        .collect();
    variantes.sort();
    variantes
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
        "modified_unix": crate::workspace::mtime_unix(prj),
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

/// Whether a project's file stem contains `texto` (case-insensitive); no filter keeps every project.
fn coincide_texto(prj: &Path, texto: Option<&str>) -> bool {
    let Some(t) = texto else {
        return true;
    };
    prj.file_stem()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.to_lowercase().contains(t))
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
    let texto = args
        .get("texto")
        .and_then(serde_json::Value::as_str)
        .map(str::to_lowercase);
    prj_files.retain(|prj| coincide_texto(prj, texto.as_deref()));

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
        "templates_dir": crate::config::proyectos_demo_ro().to_string_lossy(),
        "templates": plantillas_ro(texto.as_deref()),
        "advertencias": recorrido.advertencias(),
    }))
}

/// The read-only demo projects PVsyst ships: `proyecto crear` takes them as templates,
/// so a listing that left them out hid half of what can be created from.
fn plantillas_ro(texto: Option<&str>) -> Vec<Value> {
    let base = crate::config::proyectos_demo_ro();
    let mut prjs: Vec<PathBuf> = Vec::new();
    // Read-only folder: an unreadable one leaves the list empty, it is not an error.
    let _ = walk_ext(&base, ".prj", &mut prjs);
    prjs.sort();
    prjs.retain(|prj| coincide_texto(prj, texto));
    prjs.iter()
        .map(|prj| {
            json!({
                "name": prj.file_stem().and_then(|s| s.to_str()).unwrap_or(""),
                "prj": prj.to_string_lossy(),
                "solo_lectura": true,
            })
        })
        .collect()
}

/// Read a .PRJ or .VCi file and return its parsed object tree plus key fields.
pub(crate) fn leer_proyecto(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let name = req_str(args, &["proyecto"], "")?;
    let alcance = Alcance::de_args(args);
    let path = resolve_project_file_con_alcance(&workspace, name, alcance)?;
    let text = fs::read_to_string(&path)
        .map_err(|e| Error::io(format!("Cannot read {}", path.display()), e))?;

    let roots = object::parse_completo(&text, &path)?;

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
            "Pass detalle:\"completo\" for the full tree, or leer with 'objeto' for one object."
        );
    }
    Ok(out)
}

#[cfg(test)]
#[path = "../../tests/unit/projects/mod.rs"]
mod tests;
