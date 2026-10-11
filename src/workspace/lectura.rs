//! Chunked and paged file reading with UTF-8 boundary snapping.

use crate::args::{flag, req_str, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::workspace::base64::base64;
use crate::workspace::rutas::{resolver_ruta, Alcance};
use serde_json::{json, Value};
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Default maximum bytes to return per read call (~10k tokens).
pub(crate) const MAX_BYTES_DEFECTO: usize = 32_768;

/// Hard ceiling for requested bytes to prevent OOM.
pub(crate) const MAX_BYTES_MAXIMO: usize = 10 * 1024 * 1024;

/// Byte sample size inspected to classify file content as text vs binary.
const MUESTRA_TEXTO_BYTES: usize = 1024;

/// Inspect the initial bytes of a file to check if it contains valid UTF-8 text.
fn detectar_si_es_texto(file: &mut File, tamano: u64) -> Result<bool> {
    if tamano == 0 {
        return Ok(true);
    }
    let a_leer =
        usize::try_from(tamano.min(MUESTRA_TEXTO_BYTES as u64)).unwrap_or(MUESTRA_TEXTO_BYTES);
    let mut buf = vec![0u8; a_leer];
    file.seek(SeekFrom::Start(0))
        .map_err(|e| Error::io("Seek al inicio para detección de formato", e))?;
    file.read_exact(&mut buf)
        .map_err(|e| Error::io("Lectura de muestra de texto", e))?;
    if buf.contains(&0) {
        return Ok(false);
    }
    match std::str::from_utf8(&buf) {
        Ok(_) => Ok(true),
        Err(e) => Ok(e.error_len().is_none()),
    }
}

/// Valid UTF-8 without NUL bytes, i.e. safe to return as text without loss.
fn es_texto_limpio(bytes: &[u8]) -> bool {
    !bytes.contains(&0) && std::str::from_utf8(bytes).is_ok()
}

/// Read a binary window using seek and exact bounded read.
fn leer_ventana_binaria(
    file: &mut File,
    tamano: usize,
    offset: usize,
    cap: usize,
) -> Result<(usize, usize, Vec<u8>)> {
    let start = offset.min(tamano);
    let end = start.saturating_add(cap).min(tamano);
    let cant = end - start;
    let mut chunk = vec![0u8; cant];
    if cant > 0 {
        file.seek(SeekFrom::Start(start as u64))
            .map_err(|e| Error::io("Seek para lectura binaria", e))?;
        file.read_exact(&mut chunk)
            .map_err(|e| Error::io("Lectura de datos binarios", e))?;
    }
    Ok((start, end, chunk))
}

/// Read a text window with UTF-8 character boundary snapping at both edges.
fn leer_ventana_texto(
    file: &mut File,
    tamano: usize,
    offset: usize,
    cap: usize,
) -> Result<(usize, usize, Vec<u8>)> {
    if tamano == 0 || offset >= tamano {
        let pos = offset.min(tamano);
        return Ok((pos, pos, Vec::new()));
    }
    let pos_inicio = offset.saturating_sub(3);
    let a_leer = (offset - pos_inicio)
        .saturating_add(cap)
        .saturating_add(1)
        .min(tamano.saturating_sub(pos_inicio));
    let mut buf = vec![0u8; a_leer];
    file.seek(SeekFrom::Start(pos_inicio as u64))
        .map_err(|e| Error::io("Seek para lectura de texto", e))?;
    file.read_exact(&mut buf)
        .map_err(|e| Error::io("Lectura de buffer de texto", e))?;

    let snap_archivo = |pos: usize| -> usize {
        let mut p = pos.min(tamano);
        while p > 0 && p < tamano {
            let idx = p.saturating_sub(pos_inicio);
            if idx < buf.len() && (buf[idx] & 0xC0) == 0x80 {
                p -= 1;
            } else {
                break;
            }
        }
        p
    };

    let start = snap_archivo(offset);
    let nominal_end = start.saturating_add(cap).min(tamano);
    let end = snap_archivo(nominal_end).max(start);
    let chunk = buf[start - pos_inicio..end - pos_inicio].to_vec();
    Ok((start, end, chunk))
}

/// Read a file within a bounded memory window without loading the whole file into RAM.
pub(crate) fn leer_archivo(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let p = req_str(args, &["archivo", "file"], "")?;
    let path = resolver_ruta(&workspace, p, Alcance::de_args(args))?;
    let meta = fs::metadata(&path)
        .map_err(|e| Error::io(format!("Cannot read metadata {}", path.display()), e))?;
    if meta.is_dir() {
        return Err(Error::Invalid(format!(
            "'{}' es un directorio, no un archivo",
            path.display()
        )));
    }
    let total_size = usize::try_from(meta.len()).unwrap_or(usize::MAX);
    let mut file =
        File::open(&path).map_err(|e| Error::io(format!("Cannot read {}", path.display()), e))?;

    let as_base64 = flag(args, &["base64"]);
    let parece_texto = detectar_si_es_texto(&mut file, meta.len())?;

    let num = |k: &str| {
        args.get(k)
            .and_then(Value::as_u64)
            .and_then(|n| usize::try_from(n).ok())
    };
    let cap = num("max_bytes")
        .unwrap_or(MAX_BYTES_DEFECTO)
        .min(MAX_BYTES_MAXIMO);
    let offset = num("offset").unwrap_or(0);

    // The leading sample can miss binary content further in: the window actually
    // returned must itself be clean UTF-8, else it is served as base64.
    let texto = if parece_texto && !as_base64 {
        Some(leer_ventana_texto(&mut file, total_size, offset, cap)?)
            .filter(|v| es_texto_limpio(&v.2))
    } else {
        None
    };
    let is_text = parece_texto && (as_base64 || texto.is_some());
    let (start, end, chunk) = match texto {
        Some(v) => v,
        None => leer_ventana_binaria(&mut file, total_size, offset, cap)?,
    };

    let ventana = VentanaLectura {
        start,
        end,
        total: total_size,
        is_text,
        as_base64,
    };
    Ok(formatear_respuesta_lectura(&path, &ventana, &chunk))
}

struct VentanaLectura {
    start: usize,
    end: usize,
    total: usize,
    is_text: bool,
    as_base64: bool,
}

fn formatear_respuesta_lectura(path: &Path, v: &VentanaLectura, chunk: &[u8]) -> Value {
    let mut out = json!({
        "path": path.to_string_lossy(),
        "size_bytes": v.total,
        "is_text": v.is_text,
    });
    if v.is_text && !v.as_base64 {
        out["text"] = json!(String::from_utf8_lossy(chunk));
    } else {
        out["base64"] = json!(base64(chunk));
    }
    if v.start > 0 || v.end < v.total {
        out["range"] = json!([v.start, v.end]);
        if v.end < v.total {
            out["truncated"] = json!(true);
            out["next_offset"] = json!(v.end);
        }
    }
    out
}
