//! PV module files (.PAN): list available modules and parse characteristics.
//!
//! Two on-disk layouts are supported:
//! * PVsyst >= 6.40 text — a `PVObject_` document with Key=Value fields.
//! * Legacy (< 6.40) hybrid — ASCII header followed by Delphi Real48 binary block.

mod datos_modulo;
pub(crate) mod legacy;
#[cfg(test)]
#[path = "../../../tests/unit/components/pan/tests.rs"]
mod tests;

pub(crate) use datos_modulo::DatosModulo;
pub(crate) use legacy::{es_pan_legacy, parchear_campos_legacy, parse_legacy};

use crate::args::req_str;
use crate::args::workspace_of;
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::object;
use crate::workspace::{walk_ext, Alcance};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn num(v: Option<&str>) -> Value {
    match v {
        Some(s) => {
            let t = s.trim();
            match t.replace(',', ".").parse::<f64>() {
                Ok(f) => json!(f),
                Err(_) => json!(t),
            }
        }
        None => Value::Null,
    }
}

/// First of `names` present in the tree (field aliases of the "Basic data" sheet).
fn alias<'a>(root: &'a object::PvNode, names: &[&str]) -> Option<&'a str> {
    names.iter().find_map(|n| root.find_deep(n))
}

/// Parse a PVsyst >= 6.40 text module file.
fn parse_text_module(text: &str) -> Result<Value> {
    let root = object::parse_first(text)
        .ok_or_else(|| Error::Format("No PVObject found in .PAN".into()))?;

    let (pnom, voc, isc) = (
        num(alias(&root, &["PNom", "Pnom", "NominalPower"])),
        num(root.find_deep("Voc")),
        num(root.find_deep("Isc")),
    );
    Ok(json!({
        "vendor": root.find_deep("Manufacturer").unwrap_or(""),
        "model": root.find_deep("Model").or_else(|| root.field("Model")).unwrap_or(""),
        "data_source": root.find_deep("DataSource").unwrap_or(""),
        "technology": root.find_deep("Technology").unwrap_or(""),
        "version": root.field("Version").unwrap_or(""),
        "pnom": pnom,
        "voc": voc,
        "isc": isc,
        "parameters": {
            "pnom": pnom,
            "voc": voc,
            "isc": isc,
            "impp": num(alias(&root, &["Impp", "Imp", "IMPP"])),
            "vmpp": num(alias(&root, &["Vmpp", "Vmp", "VMPP"])),
            "mu_isc": num(alias(&root, &["muIsc", "MuISC"])),
            "mu_voc": num(alias(&root, &["muVocSpec", "muVoc"])),
            "mu_pnom": num(alias(&root, &["muPnom", "muPmpp", "muPmpReq", "Gamma", "gamma"])),
            "gref": num(root.find_deep("Gref")),
            "tref": num(root.find_deep("Tref")),
        },
        "tree": root.to_json(),
    }))
}

/// Parse the bytes of a .PAN file in either supported layout.
pub(crate) fn parse_pan(bytes: &[u8]) -> Result<Value> {
    let text = String::from_utf8_lossy(bytes);
    if object::is_pvobject(&text) {
        let mut v = parse_text_module(&text)?;
        v["format"] = json!("pvsyst8-text");
        Ok(v)
    } else if text.lines().next().unwrap_or("").starts_with("pvModule;") {
        let mut v = parse_legacy(bytes, &text)?;
        v["format"] = json!("legacy-real48");
        Ok(v)
    } else {
        Err(Error::Invalid("Unrecognized .PAN layout".into()))
    }
}

/// Resolve a module reference (bare name or path) to an existing .PAN file.
///
/// `ws` is the caller's workspace and is searched first; the default workspace
/// database and the bundled read-only one stay as fallbacks. If an absolute path
/// is provided, it is checked against `compos_ro` (PVsyst bundled read-only
/// database) or confined through `resolver_ruta` using the caller's `Alcance`.
pub(crate) fn resolve_module_file(
    ws: &Path,
    paths: &Paths,
    name: &str,
    alcance: Alcance,
) -> Result<PathBuf> {
    let direct = Path::new(name);
    if direct.is_absolute() {
        if direct.starts_with(crate::config::compos_ro()) && direct.exists() {
            return Ok(direct.to_path_buf());
        }
        let resolved = crate::workspace::resolver_ruta(ws, name, alcance)?;
        if resolved.exists() {
            return Ok(resolved);
        }
        return Err(Error::NotFound(format!(
            "Módulo no encontrado: {}",
            resolved.display()
        )));
    }

    let bare = direct.file_name().and_then(|s| s.to_str()).unwrap_or(name);
    let candidates = [
        ws.join("ComposPV").join("PVModules").join(bare),
        ws.join("ComposPV").join("PVmodules").join(bare),
        paths.modules_dir().join(bare),
        crate::config::compos_ro().join("PVModules").join(bare),
    ];
    for c in candidates {
        if c.exists() {
            return Ok(c);
        }
    }

    // Fall back to resolving relative path confined to workspace.
    let resolved = crate::workspace::resolver_ruta(ws, name, alcance)?;
    if resolved.exists() {
        return Ok(resolved);
    }
    Err(Error::NotFound(format!("Módulo '{name}' no encontrado")))
}

pub(crate) fn leer_modulo(paths: &Paths, args: &Value) -> Result<Value> {
    let file = req_str(
        args,
        &["modulo", "module", "archivo", "file"],
        "a .PAN file name or path",
    )?;

    let alcance = Alcance::de_args(args);
    let ws = crate::args::workspace_of(paths, args);
    let path = resolve_module_file(&ws, paths, file, alcance)?;
    let bytes = crate::components::leer_bytes_componente(&path)?;
    let mut result =
        parse_pan(&bytes).map_err(|e| Error::Invalid(format!("{e} in {}", path.display())))?;
    result["file"] = json!(path.to_string_lossy());
    Ok(result)
}

/// List *.PAN files available to the workspace (ComposPV/PVmodules) plus, as a
/// fallback, the read-only DataRO database shipped with PVsyst. Optional
/// `pnom_min`, `pnom_max` and `texto` filters apply to each module's metadata.
pub(crate) fn listar_modulos(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);

    let mut pans: Vec<PathBuf> = Vec::new();
    let mut folder = workspace.join("ComposPV");
    let mut recorrido = walk_ext(&folder, ".pan", &mut pans);
    if pans.is_empty() {
        folder = crate::config::compos_ro();
        recorrido = walk_ext(&folder, ".pan", &mut pans);
    }
    pans.sort();
    let criterios = crate::components::buscar::Criterios::desde(args);
    let modules: Vec<Value> = pans
        .iter()
        .filter_map(|p| {
            let rel = p
                .strip_prefix(&folder)
                .unwrap_or(p)
                .to_string_lossy()
                .into_owned();
            let parsed = fs::read(p)
                .map_err(|e| Error::io("lectura del .PAN", e))
                .and_then(|bytes| parse_pan(&bytes));
            criterios.admite(&rel, &parsed)
        })
        .collect();
    Ok(json!({
        "folder": folder.to_string_lossy(),
        "count": modules.len(),
        "modules": modules,
        "advertencias": recorrido.advertencias(),
    }))
}
