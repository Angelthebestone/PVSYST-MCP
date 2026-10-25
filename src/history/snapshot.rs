//! History snapshots creation, pruning, and listing.

use crate::calendario::civil_desde_dias;
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Number of snapshot directories retained under `mcp_history`.
const MAX_SNAPSHOTS: usize = 200;

/// Length of the compact UTC timestamp prefix `YYYYMMDD_HHMMSS`.
const LONGITUD_SELLO_COMPACTO: usize = 15;

static CONTADOR_SNAPSHOT: AtomicU64 = AtomicU64::new(0);

pub(super) fn history_root(ws: &Path) -> PathBuf {
    ws.join("UserData").join("mcp_history")
}

/// Civil date and time (UTC) for a count of seconds since the Unix epoch.
pub(super) fn civil(secs: u64) -> (i64, u32, u32, u32, u32, u32) {
    let days = i64::try_from(secs.div_euclid(86_400)).unwrap_or(i64::MAX);
    let rem = secs.rem_euclid(86_400);
    let fecha = civil_desde_dias(days);
    (
        fecha.anio(),
        fecha.mes(),
        fecha.dia(),
        u32::try_from(rem / 3600).unwrap_or(0),
        u32::try_from((rem % 3600) / 60).unwrap_or(0),
        u32::try_from(rem % 60).unwrap_or(0),
    )
}

/// Timestamp `YYYYMMDD_HHMMSS_mmm` (UTC) used to name snapshot directories.
pub(super) fn stamp(now: SystemTime) -> (String, String) {
    let dur = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    let (y, mo, d, h, mi, s) = civil(dur.as_secs());
    let ms = dur.subsec_millis();
    (
        format!("{y:04}{mo:02}{d:02}_{h:02}{mi:02}{s:02}_{ms:03}"),
        format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}.{ms:03}Z"),
    )
}

/// Compact UTC timestamp `YYYYMMDD_HHMMSS` for names that must stay unique.
pub(crate) fn sello_compacto() -> String {
    let (ts, _) = stamp(SystemTime::now());
    ts[..LONGITUD_SELLO_COMPACTO].to_string()
}

fn es_fuera_del_workspace(ws: &Path, original: &Path) -> bool {
    let Ok(ws_canon) = ws.canonicalize() else {
        return true;
    };
    let joined = if original.is_absolute() {
        original.to_path_buf()
    } else {
        ws.join(original)
    };
    let mut ancestro = joined.as_path();
    while !ancestro.exists() {
        match ancestro.parent() {
            Some(p) => ancestro = p,
            None => return true,
        }
    }
    match ancestro.canonicalize() {
        Ok(real) => !real.starts_with(&ws_canon),
        Err(_) => true,
    }
}

/// Where a snapshot is stored: under a workspace, or in the per-user MCP state
/// directory for files outside every workspace (such as `PVsyst.ini`).
enum Ambito<'a> {
    Workspace(&'a Path),
    Usuario,
}

/// Copy `archivos` into a new snapshot directory and return its id.
///
/// Missing files are recorded with `existia:false` so a restore can delete
/// them again. Any copy failure aborts the snapshot with an error, so callers
/// never write over a file that could not be preserved.
pub(crate) fn snapshot(ws: &Path, archivos: &[PathBuf], motivo: &str) -> Result<String> {
    crear_snapshot(&Ambito::Workspace(ws), archivos, motivo)
}

/// Snapshot files outside every workspace, stored under [`raiz_usuario`].
pub(crate) fn snapshot_usuario(archivos: &[PathBuf], motivo: &str) -> Result<String> {
    crear_snapshot(&Ambito::Usuario, archivos, motivo)
}

/// Directory holding snapshots of files outside every workspace.
pub(crate) fn raiz_usuario() -> Result<PathBuf> {
    Ok(crate::config::local_app_data()?
        .join("pvsyst-mcp")
        .join("historial"))
}

fn raiz_de(ambito: &Ambito) -> Result<PathBuf> {
    match ambito {
        Ambito::Workspace(ws) => Ok(history_root(ws)),
        Ambito::Usuario => raiz_usuario(),
    }
}

fn fuera_del_workspace(ambito: &Ambito, original: &Path) -> bool {
    match ambito {
        Ambito::Workspace(ws) => es_fuera_del_workspace(ws, original),
        Ambito::Usuario => true,
    }
}

fn crear_snapshot(ambito: &Ambito, archivos: &[PathBuf], motivo: &str) -> Result<String> {
    tracing::debug!(motivo, files = archivos.len(), "taking history snapshot");
    let (ts, fecha) = stamp(SystemTime::now());
    let n = CONTADOR_SNAPSHOT.fetch_add(1, Ordering::Relaxed);
    let id = format!("{ts}_{n:04}");
    let raiz = raiz_de(ambito)?;
    let dir = raiz.join(&id);
    fs::create_dir_all(&dir)
        .map_err(|e| Error::io(format!("No se pudo crear {}", dir.display()), e))?;

    let mut entradas = Vec::with_capacity(archivos.len());
    for (i, original) in archivos.iter().enumerate() {
        let existia = original.is_file();
        let copia = format!("{i:03}.bin");
        let fuera = fuera_del_workspace(ambito, original);
        if existia {
            fs::copy(original, dir.join(&copia)).map_err(|e| {
                Error::io(
                    format!("No se pudo copiar {} al historial", original.display()),
                    e,
                )
            })?;
        }
        entradas.push(json!({
            "original": original.to_string_lossy(),
            "copia": copia,
            "existia": existia,
            "fuera_del_workspace": fuera,
        }));
    }
    let manifest = json!({
        "id": id,
        "fecha": fecha,
        "motivo": motivo,
        "archivos": entradas,
    });
    let manifest_bytes = manifest.to_string();
    super::escritura_atomica::escribir_atomico(
        &dir.join("manifest.json"),
        manifest_bytes.as_bytes(),
    )?;
    prune(&raiz);
    Ok(id)
}

/// Delete the oldest snapshot directories beyond [`MAX_SNAPSHOTS`].
fn prune(root: &Path) {
    let mut dirs = sorted_dirs(root);
    if dirs.len() <= MAX_SNAPSHOTS {
        return;
    }
    let excess = dirs.len() - MAX_SNAPSHOTS;
    for old in dirs.drain(..excess) {
        // Best effort: a directory that cannot be removed is kept until the next prune.
        let _ = fs::remove_dir_all(old);
    }
}

/// Snapshot directories sorted oldest first (ids sort chronologically).
pub(super) fn sorted_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = fs::read_dir(root)
        .map(|rd| {
            rd.filter_map(std::result::Result::ok)
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect()
        })
        .unwrap_or_default();
    dirs.sort();
    dirs
}

/// List the newest `n` snapshots (newest first) with their manifests' summaries.
///
/// Unreadable or corrupt snapshot manifests are surfaced in the `corruptos`
/// list instead of being silently ignored.
pub(crate) fn listar(ws: &Path, n: usize) -> Value {
    let mut dirs = sorted_dirs(&history_root(ws));
    dirs.reverse();
    let mut items = Vec::new();
    let mut corruptos = Vec::new();

    for d in dirs.iter().take(n) {
        let manifest_path = d.join("manifest.json");
        let dir_nombre = d
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("desconocido");
        match fs::read_to_string(&manifest_path) {
            Ok(texto) => match serde_json::from_str::<Value>(&texto) {
                Ok(m) => {
                    items.push(json!({
                        "id": m["id"],
                        "fecha": m["fecha"],
                        "motivo": m["motivo"],
                        "archivos": m["archivos"].as_array().map_or(0, Vec::len),
                    }));
                }
                Err(err) => {
                    corruptos.push(json!({
                        "dir": dir_nombre,
                        "error": format!("JSON corrupto: {err}"),
                    }));
                }
            },
            Err(err) => {
                corruptos.push(json!({
                    "dir": dir_nombre,
                    "error": format!("No se pudo leer manifest.json: {err}"),
                }));
            }
        }
    }

    json!({
        "total": dirs.len(),
        "snapshots": items,
        "corruptos": corruptos,
    })
}
