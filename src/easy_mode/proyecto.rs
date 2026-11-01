//! Steps 2 & 3: Project lifecycle from template and fixed-plane orientation.

use super::parametros::DisenoParametros;
use super::pasos::{PasoOrientacionSalida, PasoProyectoSalida};
use super::simulacion::meteo_de_metadatos;
use crate::config::{self, Paths};
use crate::error::{Error, Result};
use crate::format::edit;
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
    let prj_path = paths
        .workspace
        .join("Projects")
        .join(format!("{nombre}.PRJ"));
    let prj = std::fs::read(&prj_path)
        .map_err(|e| Error::io(format!("leer {}", prj_path.display()), e))?;
    let nuevo = lifecycle::sitio::insertar_sitio(&prj, Path::new(sit_file))?;
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

/// Files of project `nombre` in the workspace `Projects` folder: empty when the folder
/// does not exist, an error when it cannot be read (a rollback must not guess).
fn archivos_del_proyecto(paths: &Paths, nombre: &str) -> Result<Vec<PathBuf>> {
    let dir = paths.workspace.join("Projects");
    let entradas = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(Error::io(format!("No se pudo leer {}", dir.display()), e)),
    };
    let mut archivos = Vec::new();
    for entrada in entradas {
        let ruta = entrada
            .map_err(|e| Error::io(format!("No se pudo leer {}", dir.display()), e))?
            .path();
        if es_del_proyecto(&ruta, nombre) {
            archivos.push(ruta);
        }
    }
    Ok(archivos)
}

fn es_del_proyecto(ruta: &Path, nombre: &str) -> bool {
    ruta.file_stem()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case(nombre))
}

/// The files of one project before a design run, to undo the run if it fails. Only
/// that project's files are touched: other files PVsyst or the user add meanwhile stay.
pub(crate) struct FotoProyecto {
    nombre: String,
    originales: Vec<(PathBuf, Vec<u8>)>,
}

impl FotoProyecto {
    /// Reads the project's current files; fails if the folder or a file cannot be read.
    pub(crate) fn tomar(paths: &Paths, nombre: &str) -> Result<Self> {
        let originales = archivos_del_proyecto(paths, nombre)?
            .into_iter()
            .map(|p| {
                let bytes = crate::workspace::leer_bytes(&p)?;
                Ok((p, bytes))
            })
            .collect::<Result<_>>()?;
        Ok(Self {
            nombre: nombre.to_string(),
            originales,
        })
    }

    /// Removes the project's files created since the photo and puts the originals back.
    /// Best effort: the step error is what the caller reports.
    pub(crate) fn deshacer(&self, paths: &Paths) {
        let actuales = match archivos_del_proyecto(paths, &self.nombre) {
            Ok(a) => a,
            Err(error) => {
                tracing::warn!(%error, "el deshacer no pudo listar el proyecto");
                Vec::new()
            }
        };
        let nuevos = actuales
            .iter()
            .filter(|p| !self.originales.iter().any(|(o, _)| o == *p));
        for nuevo in nuevos {
            if let Err(error) = std::fs::remove_file(nuevo) {
                tracing::warn!(archivo = %nuevo.display(), %error, "el deshacer no pudo borrar un archivo nuevo");
            }
        }
        for (ruta, bytes) in &self.originales {
            if let Err(error) = history::escribir_atomico(ruta, bytes) {
                tracing::warn!(archivo = %ruta.display(), %error, "el deshacer no pudo restaurar un original");
            }
        }
    }
}
