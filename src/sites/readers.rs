//! Structured readers for the two "site/weather" file types, so callers get
//! JSON instead of having to walk the raw object tree.

use crate::args::req_str;
use crate::args::workspace_of;
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::object;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// Largest SIT/MET file read whole (a real MET with an hourly year is < 20 MB).
const MAX_BYTES_LECTURA: u64 = 64 * 1024 * 1024;

/// Read `p` whole, refusing files above `limite` bytes before allocating.
fn leer_bytes_acotado(p: &Path, limite: u64) -> Result<Vec<u8>> {
    let error = |e| Error::io(format!("Cannot read {}", p.display()), e);
    let tamano = fs::metadata(p).map_err(error)?.len();
    if tamano > limite {
        return Err(Error::Invalid(format!(
            "{} ocupa {tamano} bytes; el máximo admitido es {limite}",
            p.display()
        )));
    }
    fs::read(p).map_err(error)
}

fn parse_list(s: &str) -> Vec<f64> {
    s.split(',')
        .filter_map(|x| {
            let t = x.trim();
            if t.is_empty() {
                None
            } else {
                t.replace(',', ".").parse::<f64>().ok()
            }
        })
        .collect()
}

fn monthly(node: &object::PvNode) -> Value {
    let mut m = Map::new();
    for key in [
        "GlobalH",
        "DiffuseH",
        "TAmb",
        "WindVel",
        "Linke",
        "RelHum",
        "Extraterr",
        "Albedo",
    ] {
        if let Some(v) = node.find_deep(key) {
            let vals = parse_list(v);
            if !vals.is_empty() {
                m.insert(key.to_string(), json!(vals));
            }
        }
    }
    Value::Object(m)
}

fn site_fields(node: &object::PvNode) -> Map<String, Value> {
    let mut m = Map::new();
    for key in [
        "Site",
        "Country",
        "Region",
        "Source",
        "SourceType",
        "Latitude",
        "Longitude",
        "Altitude",
        "TimeZone",
        "UnitsDisp",
        "Comment",
        "NomF",
    ] {
        if let Some(v) = node.field(key) {
            if key == "Latitude" || key == "Longitude" || key == "Altitude" || key == "TimeZone" {
                if let Ok(n) = v.trim().replace(',', ".").parse::<f64>() {
                    m.insert(key.to_string(), json!(n));
                    continue;
                }
            }
            m.insert(key.to_string(), json!(v));
        }
    }
    m
}

/// Search specification for resolving a weather or site file in the workspace.
struct DestinoLectura<'a> {
    carpeta: &'a str,
    extension: &'a str,
    claves: &'a [&'a str],
}

fn resolve_in(paths: &Paths, args: &Value, dest: &DestinoLectura<'_>) -> Result<PathBuf> {
    let name = req_str(args, dest.claves, "")?;
    let ws = workspace_of(paths, args);
    let alcance = crate::workspace::Alcance::de_args(args);

    if let Ok(p) = crate::workspace::resolver_ruta(&ws, name, alcance) {
        if p.exists() {
            return Ok(p);
        }
    }
    let in_folder = format!("{}/{}", dest.carpeta, name);
    if let Ok(p) = crate::workspace::resolver_ruta(&ws, &in_folder, alcance) {
        if p.exists() {
            return Ok(p);
        }
    }
    let in_folder_ext = format!("{}/{}.{}", dest.carpeta, name, dest.extension);
    if let Ok(p) = crate::workspace::resolver_ruta(&ws, &in_folder_ext, alcance) {
        if p.exists() {
            return Ok(p);
        }
    }
    let direct_ext = format!("{}.{}", name, dest.extension);
    if let Ok(p) = crate::workspace::resolver_ruta(&ws, &direct_ext, alcance) {
        if p.exists() {
            return Ok(p);
        }
    }
    if paths.workspace != ws {
        if let Ok(p) = crate::workspace::resolver_ruta(&paths.workspace, &in_folder, alcance) {
            if p.exists() {
                return Ok(p);
            }
        }
    }
    Err(Error::NotFound(format!(
        "'{name}' not found in {}",
        ws.join(dest.carpeta).display()
    )))
}

/// First root object of a PVObject file; empty or truncated files are errors.
fn leer_raiz(path: &Path, extension: &str) -> Result<object::PvNode> {
    let bytes = leer_bytes_acotado(path, MAX_BYTES_LECTURA)?;
    let text = String::from_utf8_lossy(&bytes);
    object::parse_completo(&text, path)?
        .into_iter()
        .next()
        .ok_or_else(|| Error::Format(format!("Not a PVObject .{extension} file")))
}

pub(crate) fn leer_meteo(paths: &Paths, args: &Value) -> Result<Value> {
    let dest = DestinoLectura {
        carpeta: "Meteo",
        extension: "MET",
        claves: &["meteo"],
    };
    let path = resolve_in(paths, args, &dest)?;
    let root = leer_raiz(&path, "MET")?;

    let mut out = Map::new();
    out.insert("file".into(), json!(path.to_string_lossy()));
    out.insert("type".into(), json!(root.obj_type));
    for key in [
        "Comment",
        "SiteM",
        "SourceM",
        "TypeM",
        "NomFormat",
        "DateFormat",
        "HourShift",
        "TimeShift",
        "PasAccum",
        "UsesPerez",
    ] {
        if let Some(v) = root.field(key) {
            out.insert(key.to_string(), json!(v));
        }
    }
    // The nested pvSite block holds the coordinates.
    for (k, v) in site_fields(&root) {
        out.insert(k, v);
    }
    out.insert("monthly".into(), monthly(&root));
    Ok(Value::Object(out))
}

pub(crate) fn leer_sitio(paths: &Paths, args: &Value) -> Result<Value> {
    let dest = DestinoLectura {
        carpeta: "Sites",
        extension: "SIT",
        claves: &["sitio"],
    };
    let path = resolve_in(paths, args, &dest)?;
    let root = leer_raiz(&path, "SIT")?;

    let mut out = Map::new();
    out.insert("file".into(), json!(path.to_string_lossy()));
    out.insert("type".into(), json!(root.obj_type));
    for (k, v) in site_fields(&root) {
        out.insert(k, v);
    }
    out.insert("monthly".into(), monthly(&root));
    Ok(Value::Object(out))
}
