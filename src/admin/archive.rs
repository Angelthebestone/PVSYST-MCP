//! Zip and archive execution helpers using the OS' built-in `tar.exe` (bsdtar).

use crate::error::{Error, Result};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

/// The system bsdtar (`%SystemRoot%\System32\tar.exe`).
pub(crate) fn tar_exe() -> PathBuf {
    std::env::var_os("SystemRoot")
        .map_or_else(|| PathBuf::from(r"C:\Windows"), PathBuf::from)
        .join(r"System32\tar.exe")
}

/// Most entries a workspace archive may hold.
const MAX_ENTRADAS: usize = 100_000;
/// Most bytes a workspace archive may expand to (guards against zip bombs).
const MAX_BYTES_EXTRAIDOS: u64 = 16 * 1024 * 1024 * 1024;

/// Validates a bsdtar listing: `nombres` from `tar -tf`, `detalle` from
/// `tar -tvf` (same order, `ls -l` style). Rejects paths escaping the
/// destination, anything but regular files and directories (links could
/// point outside it), and archives above the entry or size budget.
pub(crate) fn validar_listado_zip(nombres: &str, detalle: &str) -> Result<()> {
    let nombres: Vec<&str> = nombres
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let detalle: Vec<&str> = detalle.lines().filter(|l| !l.trim().is_empty()).collect();
    if nombres.len() != detalle.len() {
        return Err(Error::Invalid("Listado del zip inconsistente".into()));
    }
    if nombres.len() > MAX_ENTRADAS {
        return Err(Error::Invalid(format!(
            "El zip tiene {} entradas; máximo {MAX_ENTRADAS}",
            nombres.len()
        )));
    }
    let mut total: u64 = 0;
    for (nombre, linea) in nombres.iter().zip(&detalle) {
        validar_entrada(nombre, linea)?;
        let tamano = linea
            .split_whitespace()
            .nth(4)
            .and_then(|s| s.parse::<u64>().ok());
        total = tamano
            .and_then(|t| total.checked_add(t))
            .ok_or_else(|| Error::Invalid(format!("Tamaño ilegible en el zip: {linea}")))?;
    }
    if total > MAX_BYTES_EXTRAIDOS {
        return Err(Error::Invalid(format!(
            "El zip se expande a {total} bytes; máximo {MAX_BYTES_EXTRAIDOS}"
        )));
    }
    Ok(())
}

/// One entry: a regular file or directory with a path confined to the destination.
fn validar_entrada(nombre: &str, linea: &str) -> Result<()> {
    let inseguro = Path::new(nombre).components().any(|c| {
        matches!(
            c,
            Component::ParentDir | Component::Prefix(_) | Component::RootDir
        )
    });
    if inseguro {
        return Err(Error::Invalid(format!(
            "Entrada insegura en el zip: {nombre}"
        )));
    }
    if !matches!(linea.chars().next(), Some('-' | 'd')) {
        return Err(Error::Invalid(format!(
            "Entrada no admitida en el zip (enlace u objeto especial): {nombre}"
        )));
    }
    Ok(())
}

/// `tar.exe` listing of `zip` (`-tf`, or `-tvf` with `detallado`).
fn listar(zip: &Path, opcion: &str) -> Result<String> {
    let out = Command::new(tar_exe())
        .arg(opcion)
        .arg("-f")
        .arg(zip)
        .output()
        .map_err(|e| Error::Invalid(format!("Failed to run tar.exe: {e}")))?;
    if !out.status.success() {
        return Err(Error::Invalid(format!(
            "No se pudo leer el zip: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Lists the zip entries and rejects unsafe paths, links and oversized archives.
pub(crate) fn validar_entradas_zip(zip: &Path) -> Result<()> {
    validar_listado_zip(&listar(zip, "-t")?, &listar(zip, "-tv")?)
}

/// Run the system bsdtar (not a GNU tar from PATH, which cannot write zips).
pub(crate) fn run_tar(args: &[&std::ffi::OsStr]) -> Result<()> {
    let out = Command::new(tar_exe())
        .args(args)
        .output()
        .map_err(|e| Error::Invalid(format!("Failed to run tar.exe: {e}")))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(Error::Invalid(format!(
            "tar failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )))
    }
}

/// Zip `names` (relative to `dir`) into `dest`. The archive is built next to
/// `dest` and only then moved over it, so a failed export keeps the old file.
pub(crate) fn zip_into(dest: &Path, dir: &Path, names: &[std::ffi::OsString]) -> Result<()> {
    let mut temporal = dest.as_os_str().to_owned();
    temporal.push(format!(".mcp_tmp_{}.zip", std::process::id()));
    let temporal = PathBuf::from(temporal);
    let mut args: Vec<&std::ffi::OsStr> = vec![
        "-a".as_ref(),
        "-c".as_ref(),
        "-f".as_ref(),
        temporal.as_os_str(),
        "-C".as_ref(),
        dir.as_os_str(),
    ];
    args.extend(names.iter().map(std::ffi::OsString::as_os_str));
    let resultado = run_tar(&args).and_then(|()| {
        fs::rename(&temporal, dest)
            .map_err(|e| Error::io(format!("Cannot replace {}", dest.display()), e))
    });
    if resultado.is_err() {
        // Best effort: the temporary may not exist if tar failed early.
        let _ = fs::remove_file(&temporal);
    }
    resultado
}
