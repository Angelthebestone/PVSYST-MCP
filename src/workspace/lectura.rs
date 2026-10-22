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

/// Reads a whole file, keeping the path in the I/O error context.
pub(crate) fn leer_bytes(ruta: &Path) -> Result<Vec<u8>> {
    fs::read(ruta).map_err(|e| Error::io(format!("No se pudo leer {}", ruta.display()), e))
}
/// Reads a file ensuring its size on disk does not exceed `max_bytes`.
pub(crate) fn leer_bytes_acotados(ruta: &Path, max_bytes: usize) -> Result<Vec<u8>> {
    let mut file = File::open(ruta)
        .map_err(|e| Error::io(format!("No se pudo abrir {}", ruta.display()), e))?;
    let meta = file.metadata().map_err(|e| {
        Error::io(
            format!("No se pudo leer metadatos de {}", ruta.display()),
            e,
        )
    })?;
    if meta.len() > max_bytes as u64 {
        return Err(Error::Invalid(format!(
            "Archivo {} excede el límite máximo de tamaño ({} bytes, máx {})",
            ruta.display(),
            meta.len(),
            max_bytes
        )));
    }
    let mut buf = Vec::with_capacity(usize::try_from(meta.len()).unwrap_or(0).min(max_bytes));
    file.by_ref()
        .take(max_bytes as u64 + 1)
        .read_to_end(&mut buf)
        .map_err(|e| Error::io(format!("No se pudo leer {}", ruta.display()), e))?;
    if buf.len() > max_bytes {
        return Err(Error::Invalid(format!(
            "Archivo {} excede el límite máximo de tamaño tras lectura (máx {})",
            ruta.display(),
            max_bytes
        )));
    }
    Ok(buf)
}

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

struct RangoVentana {
    tamano: usize,
    offset: usize,
    cap: usize,
}

/// Read a binary window using seek and exact bounded read.
fn leer_ventana_binaria(file: &mut File, r: &RangoVentana) -> Result<(usize, usize, Vec<u8>)> {
    let start = r.offset.min(r.tamano);
    let end = start.saturating_add(r.cap).min(r.tamano);
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
fn leer_ventana_texto(file: &mut File, r: &RangoVentana) -> Result<(usize, usize, Vec<u8>)> {
    if r.tamano == 0 || r.offset >= r.tamano {
        let pos = r.offset.min(r.tamano);
        return Ok((pos, pos, Vec::new()));
    }
    let pos_inicio = r.offset.saturating_sub(3);
    let a_leer = (r.offset - pos_inicio)
        .saturating_add(r.cap)
        .saturating_add(1)
        .min(r.tamano.saturating_sub(pos_inicio));
    let mut buf = vec![0u8; a_leer];
    file.seek(SeekFrom::Start(pos_inicio as u64))
        .map_err(|e| Error::io("Seek para lectura de texto", e))?;
    file.read_exact(&mut buf)
        .map_err(|e| Error::io("Lectura de buffer de texto", e))?;

    let snap_archivo = |pos: usize| -> usize {
        let mut p = pos.min(r.tamano);
        while p > pos_inicio && p <= r.tamano {
            let idx = p - pos_inicio;
            if idx < buf.len() && (buf[idx] & 0xC0) == 0x80 {
                p -= 1;
            } else {
                break;
            }
        }
        p.max(pos_inicio)
    };

    let start = snap_archivo(r.offset);
    let nominal_end = start.saturating_add(r.cap).min(r.tamano);
    let end = snap_archivo(nominal_end).max(start);
    let start_idx = start.saturating_sub(pos_inicio);
    let end_idx = end.saturating_sub(pos_inicio).max(start_idx).min(buf.len());
    let chunk = buf
        .get(start_idx..end_idx)
        .map_or_else(Vec::new, <[u8]>::to_vec);
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

    let (offset, cap) = determinar_offset_y_cap(args);

    // The leading sample can miss binary content further in: the window actually
    // returned must itself be clean UTF-8, else it is served as base64.
    let rango = RangoVentana {
        tamano: total_size,
        offset,
        cap,
    };
    let texto = if parece_texto && !as_base64 {
        Some(leer_ventana_texto(&mut file, &rango)?).filter(|v| es_texto_limpio(&v.2))
    } else {
        None
    };
    let is_text = parece_texto && (as_base64 || texto.is_some());
    let (start, end, chunk) = match texto {
        Some(v) => v,
        None => leer_ventana_binaria(&mut file, &rango)?,
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
fn determinar_offset_y_cap(args: &Value) -> (usize, usize) {
    let num = |k: &str| {
        args.get(k)
            .and_then(Value::as_u64)
            .and_then(|n| usize::try_from(n).ok())
    };
    let cap = num("max_bytes")
        .unwrap_or(MAX_BYTES_DEFECTO)
        .min(MAX_BYTES_MAXIMO);
    let offset = num("offset").unwrap_or(0);
    (offset, cap)
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
