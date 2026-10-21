use crate::error::{Error, Result};
use crate::workspace::{resolver_ruta, walk_ext, Alcance, ResumenRecorrido};
use serde_json::Value;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ModoCoincidencia {
    NombreOExtension,
    SoloExtension,
}

pub(super) fn mtime_secs(path: &Path) -> u64 {
    fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs())
}

pub(super) fn metadata_value(text: &str, key: &str) -> Option<String> {
    text.lines()
        .take(40)
        .map(|l| l.trim_start_matches('\u{feff}'))
        .find(|l| l.starts_with(key))
        .and_then(|l| l.split(';').nth(1))
        .map(|v| v.trim().to_string())
}

pub(super) fn nombre_coincide(
    valor: Option<String>,
    pedido: Option<&str>,
    modo: ModoCoincidencia,
) -> bool {
    let Some(pedido) = pedido else {
        return true;
    };
    let Some(valor) = valor else {
        return false;
    };
    let path = Path::new(&valor);
    let parte = match modo {
        ModoCoincidencia::SoloExtension => path.extension().and_then(|e| e.to_str()),
        ModoCoincidencia::NombreOExtension => path.file_stem().and_then(|s| s.to_str()),
    };
    parte.is_some_and(|p| p.eq_ignore_ascii_case(pedido)) || valor.eq_ignore_ascii_case(pedido)
}

pub(super) fn metadata_matches(text: &str, proyecto: Option<&str>, variante: Option<&str>) -> bool {
    nombre_coincide(
        metadata_value(text, "Project;"),
        proyecto,
        ModoCoincidencia::NombreOExtension,
    ) && nombre_coincide(
        metadata_value(text, "Simulation variant;"),
        variante,
        ModoCoincidencia::SoloExtension,
    )
}

/// Largest result CSV loaded whole (an hourly year with ~60 columns is ~5 MB).
const MAX_BYTES_CSV: u64 = 64 * 1024 * 1024;
/// Prefix read to match a CSV's metadata, which PVsyst writes in its first lines.
const BYTES_CABECERA: u64 = 16 * 1024;

pub(super) fn leer_texto(path: &Path) -> Result<String> {
    let error = |e| Error::io(format!("No se pudo leer {}", path.display()), e);
    let tamano = fs::metadata(path).map_err(error)?.len();
    if tamano > MAX_BYTES_CSV {
        return Err(Error::Invalid(format!(
            "{} ocupa {tamano} bytes; el máximo admitido es {MAX_BYTES_CSV}",
            path.display()
        )));
    }
    let bytes = fs::read(path).map_err(error)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// Only the metadata header of a result CSV, without loading the data rows.
fn leer_cabecera(path: &Path) -> Result<String> {
    let error = |e| Error::io(format!("No se pudo leer {}", path.display()), e);
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(error)?
        .take(BYTES_CABECERA)
        .read_to_end(&mut bytes)
        .map_err(error)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

pub(crate) fn variant_kwp(workspace: &Path, metadata: &Value) -> Option<f64> {
    let line = metadata
        .as_array()?
        .iter()
        .filter_map(Value::as_str)
        .find(|l| l.starts_with("Simulation variant;"))?;
    let file = line.split(';').nth(1)?.trim();
    let text = std::fs::read_to_string(workspace.join("Projects").join(file)).ok()?;
    let root = crate::format::object::parse_first(&text)?;
    root.find_deep("PNomSizing")?
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|k| *k > 0.0)
}

pub(super) fn csv_explicito(workspace: &Path, f: &str, alcance: Alcance) -> Result<PathBuf> {
    let p = resolver_ruta(workspace, f, alcance)?;
    if p.exists() {
        Ok(p)
    } else {
        Err(Error::NotFound(format!(
            "CSV file not found: {}",
            p.display()
        )))
    }
}

pub(crate) fn resultado_de(
    workspace: &Path,
    proyecto: Option<&str>,
    variante: Option<&str>,
    archivo: Option<&str>,
) -> Result<(PathBuf, Option<PathBuf>)> {
    resultado_de_con_alcance(workspace, proyecto, variante, archivo, Alcance::Workspace)
}

pub(super) fn resultado_de_con_alcance(
    workspace: &Path,
    proyecto: Option<&str>,
    variante: Option<&str>,
    archivo: Option<&str>,
    alcance: Alcance,
) -> Result<(PathBuf, Option<PathBuf>)> {
    let csv = match archivo {
        Some(f) => csv_explicito(workspace, f, alcance)?,
        None => {
            let dir = workspace.join("Results");
            let mut candidatos = Vec::new();
            exigir_inventario_completo(&walk_ext(&dir, ".csv", &mut candidatos))?;
            let mut filtrados = Vec::with_capacity(candidatos.len());
            for p in candidatos {
                let texto = leer_cabecera(&p)?;
                if metadata_matches(&texto, proyecto, variante) {
                    filtrados.push(p);
                }
            }
            filtrados.sort_by_key(|p| std::cmp::Reverse(mtime_secs(p)));
            filtrados.into_iter().next().ok_or_else(|| {
                let proyecto = proyecto.unwrap_or("(sin proyecto)");
                let variante = variante.unwrap_or("(sin variante)");
                Error::NotFound(format!(
                    "No hay resultados CSV para proyecto '{proyecto}' variante '{variante}' en {}",
                    dir.display()
                ))
            })?
        }
    };
    let pdf = csv.with_extension("pdf");
    Ok((csv, pdf.exists().then_some(pdf)))
}

/// "Latest result" is only meaningful over a complete inventory.
fn exigir_inventario_completo(recorrido: &ResumenRecorrido) -> Result<()> {
    let avisos = recorrido.advertencias();
    if avisos.is_empty() {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "Inventario de resultados incompleto: {}",
        avisos.join("; ")
    )))
}

pub(super) fn csvs_a_leer(workspace: &Path, args: &Value, count: usize) -> Result<Vec<PathBuf>> {
    let explicit = args
        .get("archivo")
        .or_else(|| args.get("file"))
        .and_then(|v| v.as_str());

    let alcance = Alcance::de_args(args);
    let mut csv_files: Vec<PathBuf> = Vec::new();
    if let Some(f) = explicit {
        csv_files.push(csv_explicito(workspace, f, alcance)?);
    } else {
        let results_dir = match args.get("dir").and_then(Value::as_str) {
            Some(d) => resolver_ruta(workspace, d, alcance)?,
            None => workspace.join("Results"),
        };
        let search_root = if results_dir.exists() {
            results_dir
        } else {
            workspace.to_path_buf()
        };
        exigir_inventario_completo(&walk_ext(&search_root, ".csv", &mut csv_files))?;
    }
    let proyecto = args.get("proyecto").and_then(Value::as_str);
    let variante = args.get("variante").and_then(Value::as_str);
    if proyecto.is_some() || variante.is_some() {
        let mut filtrados = Vec::with_capacity(csv_files.len());
        for p in csv_files {
            let texto = leer_cabecera(&p)?;
            if metadata_matches(&texto, proyecto, variante) {
                filtrados.push(p);
            }
        }
        csv_files = filtrados;
    }

    csv_files.sort_by_key(|p| std::cmp::Reverse(mtime_secs(p)));
    csv_files.truncate(count);

    if csv_files.is_empty() {
        return Err(Error::NotFound(format!(
            "No CSV result files found under {}",
            workspace.display()
        )));
    }
    Ok(csv_files)
}
