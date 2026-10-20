//! Atomic file writes, serialized concurrency per destination, and temporary cleanup.

use crate::error::{Error, Result};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Maximum age for temporary sibling files before they are considered abandoned and deleted.
pub(super) const UMBRAL_EXPIRACION_TEMPORAL: Duration = Duration::from_secs(300);

/// Number of stripes for synchronizing concurrent writes to the same destination.
const NUMERO_BLOQUEOS: usize = 64;

// WHY a static striped lock array: it serializes writers to the same destination
// within this process without lock files (on Windows, sibling lock files hit
// ERROR_ACCESS_DENIED under create/delete churn because NTFS deletes lazily) and
// without an unbounded, never-pruned `Mutex<HashMap<PathBuf, _>>`. Two paths
// sharing a stripe only serialize more than strictly needed, which is harmless.
static BLOQUEOS_DESTINO: [Mutex<()>; NUMERO_BLOQUEOS] = [const { Mutex::new(()) }; NUMERO_BLOQUEOS];

static CONTADOR_ESCRITURA: AtomicU64 = AtomicU64::new(0);

fn bloqueo_para_ruta(path: &Path) -> &Mutex<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent_canon = parent
        .canonicalize()
        .unwrap_or_else(|_| parent.to_path_buf());
    let file_name = path.file_name().unwrap_or_default();

    let mut hasher = DefaultHasher::new();
    parent_canon.hash(&mut hasher);
    file_name.hash(&mut hasher);
    let idx = (hasher.finish() as usize) % NUMERO_BLOQUEOS;
    &BLOQUEOS_DESTINO[idx]
}

fn ruta_temporal_unica(path: &Path) -> PathBuf {
    let pid = std::process::id();
    let seq = CONTADOR_ESCRITURA.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let stem = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("archivo");
    let nombre_tmp = format!("{stem}.mcp_tmp_{pid}_{seq}_{nanos}");
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    parent.join(nombre_tmp)
}

struct GuardiaTemporal<'a> {
    ruta: &'a Path,
    activa: bool,
}

impl<'a> GuardiaTemporal<'a> {
    fn nueva(ruta: &'a Path) -> Self {
        Self { ruta, activa: true }
    }

    fn desactivar(&mut self) {
        self.activa = false;
    }
}

impl<'a> Drop for GuardiaTemporal<'a> {
    fn drop(&mut self) {
        if self.activa {
            let _ = fs::remove_file(self.ruta);
        }
    }
}

pub(super) fn limpiar_temporales_abandonados(
    directorio: &Path,
    ahora: SystemTime,
    umbral: Duration,
) {
    let Ok(entradas) = fs::read_dir(directorio) else {
        return;
    };
    for entrada in entradas.flatten() {
        let nombre = entrada.file_name();
        let texto = nombre.to_string_lossy();
        if !texto.contains(".mcp_tmp") {
            continue;
        }
        let Ok(meta) = entrada.metadata() else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        let Ok(modificado) = meta.modified() else {
            continue;
        };
        if let Ok(edad) = ahora.duration_since(modificado) {
            if edad >= umbral {
                let _ = fs::remove_file(entrada.path());
            }
        }
    }
}

/// Write `bytes` to `path` through a temporary sibling, then rename over it.
/// Concurrent writes to the same destination path are serialized.
pub(crate) fn escribir_atomico(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    // Serialize concurrent writes to the same destination.
    let _guardia_bloqueo = bloqueo_para_ruta(path)
        .lock()
        .unwrap_or_else(|p| p.into_inner());

    limpiar_temporales_abandonados(parent, SystemTime::now(), UMBRAL_EXPIRACION_TEMPORAL);

    let tmp = ruta_temporal_unica(path);
    let mut guardia_tmp = GuardiaTemporal::nueva(&tmp);

    fs::write(&tmp, bytes)
        .map_err(|e| Error::io(format!("No se pudo escribir {}", tmp.display()), e))?;

    renombrar_con_reintento(&tmp, path)
        .map_err(|e| Error::io(format!("No se pudo reemplazar {}", path.display()), e))?;

    guardia_tmp.desactivar();
    Ok(())
}

const MAX_REINTENTOS_RENOMBRADO: usize = 30;
const PAUSA_REINTENTO: Duration = Duration::from_millis(5);

fn renombrar_con_reintento(tmp: &Path, destino: &Path) -> std::io::Result<()> {
    let mut intentos = 0;
    loop {
        match fs::rename(tmp, destino) {
            Ok(()) => return Ok(()),
            Err(e) => {
                let reintentable = cfg!(windows) && matches!(e.raw_os_error(), Some(5) | Some(32));
                if reintentable && intentos < MAX_REINTENTOS_RENOMBRADO {
                    intentos += 1;
                    std::thread::sleep(PAUSA_REINTENTO);
                    continue;
                }
                return Err(e);
            }
        }
    }
}

/// Replace `path` with `bytes`: snapshot the previous version when it exists,
/// then write atomically. Returns the history id, or `None` for a new file.
pub(crate) fn reemplazar(
    ws: &Path,
    path: &Path,
    bytes: &[u8],
    motivo: &str,
) -> Result<Option<String>> {
    let id = if path.exists() {
        Some(super::snapshot::snapshot(
            ws,
            &[path.to_path_buf()],
            motivo,
        )?)
    } else {
        None
    };
    escribir_atomico(path, bytes)?;
    Ok(id)
}
