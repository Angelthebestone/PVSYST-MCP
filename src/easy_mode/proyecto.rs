//! Steps 2 & 3: Project lifecycle from template and fixed-plane orientation.

use super::parametros::DisenoParametros;
use super::pasos::{PasoOrientacionSalida, PasoProyectoSalida};
use super::simulacion::meteo_de_metadatos;
use crate::config::{self, Paths};
use crate::error::{Error, Result};
use crate::format::{edit, insert};
use crate::history;
use crate::projects::lifecycle;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

/// Ensure a project template exists in the workspace, copying the shipped
/// PVsyst demo from `DataRO` if necessary.
pub(crate) fn ensure_template(paths: &Paths, name: &str) -> Result<()> {
    let projects = paths.workspace.join("Projects");
    if projects.join(format!("{name}.PRJ")).exists() {
        return Ok(());
    }
    let src = config::proyectos_demo_ro();
    let prj = src.join(format!("{name}.PRJ"));
    if !prj.exists() {
        return Err(Error::NotFound(format!(
            "Template '{name}' not found in the workspace nor in {}",
            src.display()
        )));
    }
    std::fs::create_dir_all(&projects)
        .map_err(|e| Error::io(format!("No se pudo crear {}", projects.display()), e))?;
    for entry in std::fs::read_dir(&src)
        .map_err(|e| Error::io(format!("No se pudo leer {}", src.display()), e))?
    {
        let p: PathBuf = entry
            .map_err(|e| {
                Error::io(
                    format!("No se pudo leer una entrada de {}", src.display()),
                    e,
                )
            })?
            .path();
        if let (Some(stem), Some(file_name)) = (p.file_stem(), p.file_name()) {
            if stem != name {
                continue;
            }
            let dst = projects.join(file_name);
            std::fs::copy(&p, &dst)
                .map_err(|e| Error::io(format!("copy {} -> {}", p.display(), dst.display()), e))?;
        }
    }
    Ok(())
}

/// Executes the project creation step from the given template.
pub(crate) fn ejecutar_paso_proyecto(
    paths: &Paths,
    params: &DisenoParametros,
) -> Result<PasoProyectoSalida> {
    let mut pa = Map::new();
    pa.insert("plantilla".into(), json!(params.proyecto.plantilla));
    pa.insert("nombre_nuevo".into(), json!(params.proyecto.nombre));
    pa.insert("variante".into(), json!(params.variante));
    pa.insert("sobrescribir".into(), json!(params.proyecto.sobrescribir));
    if let Some(m) = &params.dimensionado.modulo {
        pa.insert("modulo".into(), json!(m));
    }
    if let Some(inv) = &params.dimensionado.inversor {
        pa.insert("inversor".into(), json!(inv));
    }
    if let Some(ns) = params.dimensionado.n_mod_serie {
        pa.insert("n_mod_serie".into(), json!(ns));
    }
    if let Some(np) = params.dimensionado.n_strings {
        pa.insert("n_strings".into(), json!(np));
    }

    let project = lifecycle::crear_proyecto(paths, &Value::Object(pa))?;
    Ok(PasoProyectoSalida {
        nombre: params.proyecto.nombre.clone(),
        ok: true,
        raw_response: project,
    })
}

/// Executes the fixed-plane orientation step.
pub(crate) fn ejecutar_paso_orientacion(
    paths: &Paths,
    params: &DisenoParametros,
) -> Result<PasoOrientacionSalida> {
    let ev = json!({
        "proyecto": params.proyecto.nombre,
        "variante": params.variante,
        "ediciones": [{ "objeto": "pvOrient", "campos": {
            "FieldType": "FixedPlane",
            "FieldTilt": params.orientacion.tilt,
            "FieldAzim": params.orientacion.azimut
        }}]
    });
    let detalle = lifecycle::editar_variante(paths, &ev)?;
    Ok(PasoOrientacionSalida {
        tilt: params.orientacion.tilt,
        azimut: params.orientacion.azimut,
        ok: true,
        detalle,
    })
}

/// Reads the `.SIT` and returns the project path with its bytes, `SitePrj` replaced by that site.
fn prj_con_sitio(paths: &Paths, nombre: &str, sit_file: &str) -> Result<(PathBuf, Vec<u8>)> {
    let sit = std::fs::read(sit_file).map_err(|e| Error::io(format!("leer {sit_file}"), e))?;
    let bloque = insert::extraer_objeto(&sit, "pvSite")?;
    let prj_path = paths
        .workspace
        .join("Projects")
        .join(format!("{nombre}.PRJ"));
    let prj = std::fs::read(&prj_path)
        .map_err(|e| Error::io(format!("leer {}", prj_path.display()), e))?;
    let nuevo = insert::insertar_objeto(
        &prj,
        "pvProject",
        &bloque,
        Some("SitePrj"),
        insert::SiExiste::Reemplazar,
    )?;
    Ok((prj_path, nuevo))
}

/// Links the created site to the project (its `SitePrj` object) right after the project step,
/// so the project never keeps the template's site.
pub(crate) fn vincular_sitio(paths: &Paths, nombre: &str, sit_file: &str) -> Result<()> {
    let (prj_path, nuevo) = prj_con_sitio(paths, nombre, sit_file)?;
    history::snapshot(
        &paths.workspace,
        std::slice::from_ref(&prj_path),
        "disenar_sistema: sitio en proyecto",
    )?;
    history::escribir_atomico(&prj_path, &nuevo)?;
    Ok(())
}

/// Write the created site into the project (replacing its `SitePrj` object)
/// and point `MeteoFileName` at the weather file the simulation used.
pub(crate) fn sustituir_sitio(
    paths: &Paths,
    nombre: &str,
    sit_file: &str,
    csv: &Path,
) -> Result<Value> {
    let (prj_path, mut nuevo) = prj_con_sitio(paths, nombre, sit_file)?;
    let meteo = std::fs::read(csv)
        .ok()
        .and_then(|b| meteo_de_metadatos(&String::from_utf8_lossy(&b)));
    if let Some(m) = &meteo {
        nuevo = edit::edit_fields(
            &nuevo,
            "pvProject",
            &[("MeteoFileName".to_string(), Some(m.clone()))],
        )?
        .0;
    }
    history::snapshot(
        &paths.workspace,
        std::slice::from_ref(&prj_path),
        "disenar_sistema: sitio en proyecto",
    )?;
    history::escribir_atomico(&prj_path, &nuevo)?;
    Ok(json!({ "meteo_file": meteo }))
}

/// Files currently in the workspace `Projects` folder (empty when it does not exist).
pub(crate) fn archivos_de_proyectos(paths: &Paths) -> Vec<PathBuf> {
    std::fs::read_dir(paths.workspace.join("Projects"))
        .map(|entradas| entradas.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default()
}

/// Bytes of the files of project `nombre` present in `antes`, to restore them if a design
/// run that overwrites the project fails halfway.
pub(crate) fn originales_del_proyecto(antes: &[PathBuf], nombre: &str) -> Vec<(PathBuf, Vec<u8>)> {
    antes
        .iter()
        .filter(|p| {
            p.file_stem()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.eq_ignore_ascii_case(nombre))
        })
        .filter_map(|p| Some((p.clone(), std::fs::read(p).ok()?)))
        .collect()
}

/// Removes the project files created since `antes` and puts back the `originales`, so a
/// failed design leaves neither partial nor half-overwritten files. Best effort: the
/// step error is what the caller reports.
pub(crate) fn deshacer_nuevos(paths: &Paths, antes: &[PathBuf], originales: &[(PathBuf, Vec<u8>)]) {
    for nuevo in archivos_de_proyectos(paths)
        .into_iter()
        .filter(|p| !antes.contains(p))
    {
        if let Err(error) = std::fs::remove_file(&nuevo) {
            tracing::warn!(archivo = %nuevo.display(), %error, "el deshacer no pudo borrar un archivo nuevo");
        }
    }
    for (ruta, bytes) in originales {
        if let Err(error) = history::escribir_atomico(ruta, bytes) {
            tracing::warn!(archivo = %ruta.display(), %error, "el deshacer no pudo restaurar un original");
        }
    }
}
