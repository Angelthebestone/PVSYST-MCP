//! Directory traversal engines: bounded listings and extension walks.
//! Neither follows symlinks, junctions or other reparse points.

use super::listado::{InfoRecorrido, ResumenRecorrido, MAX_ARCHIVOS, MAX_PROFUNDIDAD};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// Traversal recursion mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModoRecorrido {
    Recursivo,
    Plano,
}

pub(crate) struct OpcionesRecorrido {
    pub(crate) modo: ModoRecorrido,
    pub(crate) max_profundidad: usize,
    pub(crate) max_archivos: usize,
}

#[cfg(windows)]
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;

/// True if the directory entry is a symlink or Windows reparse point (junction / mount).
fn es_enlace_o_reparse(entry: &fs::DirEntry) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if let Ok(meta) = fs::symlink_metadata(entry.path()) {
            if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                return true;
            }
        }
    }
    match entry.file_type() {
        Ok(ft) => ft.is_symlink(),
        Err(_) => true,
    }
}

struct EstadoRecorrido<'a> {
    profundidad: usize,
    opciones: &'a OpcionesRecorrido,
}

fn recorrer_nivel(dir: &Path, estado: &mut EstadoRecorrido, info: &mut InfoRecorrido) {
    if info.archivos.len() >= estado.opciones.max_archivos {
        info.truncado = true;
        return;
    }
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            info.errores.push(format!("{}: {e}", dir.display()));
            return;
        }
    };
    for entry_res in entries {
        if info.archivos.len() >= estado.opciones.max_archivos {
            info.truncado = true;
            break;
        }
        let entry = match entry_res {
            Ok(e) => e,
            Err(e) => {
                info.errores.push(format!("{}: {e}", dir.display()));
                continue;
            }
        };
        if es_enlace_o_reparse(&entry) {
            continue;
        }
        let p = entry.path();
        if p.is_dir() {
            if estado.opciones.modo == ModoRecorrido::Recursivo {
                if estado.profundidad + 1 > estado.opciones.max_profundidad {
                    info.truncado = true;
                } else {
                    estado.profundidad += 1;
                    recorrer_nivel(&p, estado, info);
                    estado.profundidad -= 1;
                }
            }
        } else {
            info.archivos.push(p);
        }
    }
}

pub(crate) fn recorrer(dir: &Path, opciones: &OpcionesRecorrido, info: &mut InfoRecorrido) {
    let mut estado = EstadoRecorrido {
        profundidad: 0,
        opciones,
    };
    recorrer_nivel(dir, &mut estado, info);
}

pub(crate) fn extraer_opciones_listado(args: &Value) -> OpcionesRecorrido {
    let recursivo = if args
        .get("recursivo")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true)
    {
        ModoRecorrido::Recursivo
    } else {
        ModoRecorrido::Plano
    };
    let max_profundidad = args
        .get("max_profundidad")
        .and_then(Value::as_u64)
        .and_then(|n| usize::try_from(n).ok())
        .unwrap_or(MAX_PROFUNDIDAD)
        .min(MAX_PROFUNDIDAD);
    let max_archivos = args
        .get("max_archivos")
        .and_then(Value::as_u64)
        .and_then(|n| usize::try_from(n).ok())
        .unwrap_or(MAX_ARCHIVOS)
        .min(MAX_ARCHIVOS);
    OpcionesRecorrido {
        modo: recursivo,
        max_profundidad,
        max_archivos,
    }
}

struct EstadoWalk<'a> {
    profundidad: usize,
    max_profundidad: usize,
    max_archivos: usize,
    out: &'a mut Vec<PathBuf>,
    resumen: ResumenRecorrido,
}

fn walk_ext_paso(dir: &Path, ext: &str, estado: &mut EstadoWalk) {
    if estado.profundidad >= estado.max_profundidad || estado.out.len() >= estado.max_archivos {
        estado.resumen.truncado = true;
        return;
    }
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            estado
                .resumen
                .errores
                .push(format!("{}: {e}", dir.display()));
            return;
        }
    };
    for entry_res in entries {
        if estado.out.len() >= estado.max_archivos {
            estado.resumen.truncado = true;
            break;
        }
        let entry = match entry_res {
            Ok(e) => e,
            Err(e) => {
                estado
                    .resumen
                    .errores
                    .push(format!("{}: {e}", dir.display()));
                continue;
            }
        };
        if es_enlace_o_reparse(&entry) {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            estado.profundidad += 1;
            walk_ext_paso(&path, ext, estado);
            estado.profundidad -= 1;
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| format!(".{}", e.to_lowercase()) == ext)
        {
            estado.out.push(path);
        }
    }
}

/// Recursively collect files whose extension (lower-cased) matches `ext`
/// (including the dot, e.g. ".prj"). Does not follow symlinks or junctions.
pub(crate) fn walk_ext(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) -> ResumenRecorrido {
    // A missing root is an empty inventory, not an unreadable one.
    if !dir.is_dir() {
        return ResumenRecorrido::default();
    }
    let mut estado = EstadoWalk {
        profundidad: 0,
        max_profundidad: MAX_PROFUNDIDAD,
        max_archivos: MAX_ARCHIVOS,
        out,
        resumen: ResumenRecorrido::default(),
    };
    walk_ext_paso(dir, ext, &mut estado);
    estado.resumen
}
