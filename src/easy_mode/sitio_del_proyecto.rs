//! Linking the site a design created into its project: the `SitePrj` of the PRJ and
//! the site of the variant's `pvMeteo`, which is the one a simulation reads.

use super::simulacion::meteo_de_metadatos;
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::edit;
use crate::history;
use crate::projects::lifecycle;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// The site a design links into its project: the `.SIT` and the variant it belongs to.
pub(crate) struct SitioDelDiseno<'a> {
    pub(crate) nombre: &'a str,
    pub(crate) variante: &'a str,
    pub(crate) sit_file: &'a str,
}

/// A project file and the bytes that replace it.
struct ArchivoNuevo {
    ruta: PathBuf,
    bytes: Vec<u8>,
}

/// Reads the `.SIT` and returns the project and variant with their site objects
/// replaced by that site: `SitePrj` in the PRJ, the site of `pvMeteo` in the variant.
fn archivos_con_sitio(
    paths: &Paths,
    sitio: &SitioDelDiseno<'_>,
) -> Result<(ArchivoNuevo, ArchivoNuevo)> {
    let carpeta = paths.workspace.join("Projects");
    let leer = |ext: &str| -> Result<(PathBuf, Vec<u8>)> {
        let ruta = carpeta.join(format!("{}.{ext}", sitio.nombre));
        let bytes =
            std::fs::read(&ruta).map_err(|e| Error::io(format!("leer {}", ruta.display()), e))?;
        Ok((ruta, bytes))
    };
    let (prj_path, prj) = leer("PRJ")?;
    let (var_path, var) = leer(sitio.variante)?;
    let sit = Path::new(sitio.sit_file);
    Ok((
        ArchivoNuevo {
            ruta: prj_path,
            bytes: lifecycle::sitio::insertar_sitio(&prj, sit)?,
        },
        ArchivoNuevo {
            ruta: var_path,
            bytes: lifecycle::sitio::insertar_sitio_en_variante(&var, sit)?,
        },
    ))
}

/// Links the created site to the project and its variant right after the project step,
/// so neither keeps the template's site.
pub(crate) fn vincular_sitio(paths: &Paths, sitio: &SitioDelDiseno<'_>) -> Result<()> {
    let (prj, var) = archivos_con_sitio(paths, sitio)?;
    escribir_sitio(paths, &prj, &var)
}

fn escribir_sitio(paths: &Paths, prj: &ArchivoNuevo, var: &ArchivoNuevo) -> Result<()> {
    history::snapshot(
        &paths.workspace,
        &[prj.ruta.clone(), var.ruta.clone()],
        "disenar_sistema: sitio en proyecto",
    )?;
    history::escribir_atomico(&prj.ruta, &prj.bytes)?;
    history::escribir_atomico(&var.ruta, &var.bytes)
}

/// Write the created site into the project and its variant, and point `MeteoFileName`
/// at the weather file the simulation used.
pub(crate) fn sustituir_sitio(
    paths: &Paths,
    sitio: &SitioDelDiseno<'_>,
    csv: &Path,
) -> Result<Value> {
    let (mut prj, var) = archivos_con_sitio(paths, sitio)?;
    let meteo = std::fs::read(csv)
        .ok()
        .and_then(|b| meteo_de_metadatos(&String::from_utf8_lossy(&b)));
    if let Some(m) = &meteo {
        prj.bytes = edit::edit_fields(
            &prj.bytes,
            "pvProject",
            &[("MeteoFileName".to_string(), Some(m.clone()))],
        )?
        .0;
    }
    escribir_sitio(paths, &prj, &var)?;
    Ok(json!({ "meteo_file": meteo }))
}
