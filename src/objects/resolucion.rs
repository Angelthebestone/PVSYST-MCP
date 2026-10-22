//! Resolution of PVsyst target files by extension, component type, or workspace path.

use crate::args::req_str;
use crate::error::Result;
use crate::workspace::{resolver_ruta, Alcance};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// Folder inside the workspace that holds files with this extension.
fn carpeta_de_extension(ext: &str) -> Option<&'static str> {
    let mayus = ext.to_ascii_uppercase();
    match mayus.as_str() {
        "PRJ" => Some("Projects"),
        "SIT" => Some("Sites"),
        "MET" => Some("Meteo"),
        v if v.len() == 3 && (v.starts_with("VC") || v.starts_with("CM")) => Some("Projects"),
        _ => None,
    }
}

/// File names in `dir` that belong to the same PVsyst folder as `carpeta`.
fn archivos_en(dir: &Path, carpeta: &str) -> Vec<String> {
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .and_then(carpeta_de_extension)
                == Some(carpeta)
        })
        .filter_map(|p| p.file_name().and_then(|s| s.to_str()).map(str::to_string))
        .collect()
}

struct PeticionArchivo<'a> {
    ext: &'a str,
    name: &'a str,
    nombre: &'a std::ffi::OsStr,
    alcance: Alcance,
}

/// Resolve a file by its extension: project, site, weather or component.
/// Missing files in a PVsyst folder yield a "not found" error with candidates.
fn resolver_por_extension(workspace: &Path, req: &PeticionArchivo) -> Result<PathBuf> {
    if let Some(carpeta) = carpeta_de_extension(req.ext) {
        let dir = workspace.join(carpeta);
        let candidato = dir.join(req.nombre);
        if candidato.exists() {
            return Ok(candidato);
        }
        let tipo = match carpeta {
            "Projects" => "Proyecto",
            "Sites" => "Sitio",
            _ => "Meteo",
        };
        let opciones = archivos_en(&dir, carpeta);
        return Err(crate::args::no_encontrado(
            tipo,
            req.name,
            &opciones,
            &dir.to_string_lossy(),
        ));
    }
    let kind = req.ext.to_uppercase();
    if crate::components::folder_for(&kind).is_some() {
        let consulta = crate::components::ConsultaComponente {
            kind: &kind,
            name: req.name,
            alcance: req.alcance,
        };
        return crate::components::resolve_component(workspace, &consulta);
    }
    resolver_ruta(workspace, req.name, req.alcance)
}

/// Resolve a general PVsyst file: a component (with `tipo`), or a path relative
/// to the workspace, or an absolute path (checked against `Alcance`).
pub(crate) fn resolve_file(workspace: &Path, args: &Value) -> Result<PathBuf> {
    let name = req_str(args, &["archivo"], "")?;
    let alcance = Alcance::de_args(args);
    if let Some(kind) = args.get("tipo").and_then(|v| v.as_str()) {
        if !kind.is_empty() {
            let consulta = crate::components::ConsultaComponente {
                kind,
                name,
                alcance,
            };
            return crate::components::resolve_component(workspace, &consulta);
        }
    }
    let p = Path::new(name);
    if p.is_absolute() {
        return resolver_ruta(workspace, name, alcance);
    }
    let ws_path = resolver_ruta(workspace, name, alcance)?;
    if ws_path.exists() {
        return Ok(ws_path);
    }
    let Some(nombre) = p.file_name() else {
        return Ok(ws_path);
    };
    match p.extension().and_then(|e| e.to_str()) {
        Some(ext) => {
            let req = PeticionArchivo {
                ext,
                name,
                nombre,
                alcance,
            };
            resolver_por_extension(workspace, &req)
        }
        None => {
            // A bare project name ("MCP_E2E") means Projects\MCP_E2E.PRJ.
            let prj = workspace
                .join("Projects")
                .join(nombre)
                .with_extension("PRJ");
            Ok(if prj.exists() { prj } else { ws_path })
        }
    }
}
