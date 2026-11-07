//! Steps 2 & 3: Project lifecycle from template and fixed-plane orientation.

use super::parametros::DisenoParametros;
use super::pasos::{PasoOrientacionSalida, PasoProyectoSalida};
use super::simulacion::meteo_de_metadatos;
use crate::config::{self, Paths};
use crate::error::{Error, Result};
use crate::format::edit;
use crate::history;
use crate::idioma::Texto;
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

/// Fields of the variant that `proyecto crear` sets from the request, with the
/// value the request asks for. Orientation is not here: step 3 writes it.
fn campos_pedidos(params: &DisenoParametros) -> Vec<(&'static str, String)> {
    let d = &params.dimensionado;
    let mut campos = Vec::with_capacity(4);
    if let Some(m) = &d.modulo {
        campos.push(("PVModule", m.clone()));
    }
    if let Some(i) = &d.inversor {
        campos.push(("GInverter", i.clone()));
    }
    if let Some(n) = d.n_mod_serie {
        campos.push(("NModSerie", n.to_string()));
    }
    if let Some(n) = d.n_strings {
        campos.push(("NStringCh", n.to_string()));
    }
    campos
}

/// The request's values the existing variant does not hold. `None` when the
/// project does not exist yet, so there is nothing to compare.
fn diferencias_con_lo_existente(paths: &Paths, params: &DisenoParametros) -> Option<Vec<String>> {
    let carpeta = paths.workspace.join("Projects");
    let prj = carpeta.join(format!("{}.PRJ", params.proyecto.nombre));
    if !prj.is_file() {
        return None;
    }
    let variante = carpeta.join(format!("{}.{}", params.proyecto.nombre, params.variante));
    let Ok(bytes) = std::fs::read(&variante) else {
        return Some(vec![Texto {
            es: "falta su variante {archivo}",
            en: "its variant {archivo} is missing",
        }
        .texto_con(
            paths.idioma,
            &[("archivo", &variante.display().to_string())],
        )]);
    };
    let Some(raiz) = crate::format::object::parse_first(&String::from_utf8_lossy(&bytes)) else {
        return Some(vec![Texto {
            es: "su variante no es un archivo PVObject",
            en: "its variant is not a PVObject file",
        }
        .en_idioma(paths.idioma)
        .to_string()]);
    };
    Some(
        campos_pedidos(params)
            .into_iter()
            .filter_map(|(campo, pedido)| {
                let actual = raiz.find_deep(campo)?;
                (actual.trim() != pedido.trim()).then(|| {
                    Texto {
                        es: "{campo} vale '{actual}', no '{pedido}'",
                        en: "{campo} is '{actual}', not '{pedido}'",
                    }
                    .texto_con(
                        paths.idioma,
                        &[
                            ("campo", campo),
                            ("actual", actual.trim()),
                            ("pedido", &pedido),
                        ],
                    )
                })
            })
            .collect(),
    )
}

/// A project that already exists with the requested parameters: the run
/// continues from the step that failed instead of demanding a rebuild.
fn reutilizar(paths: &Paths, params: &DisenoParametros) -> PasoProyectoSalida {
    let prj = paths
        .workspace
        .join("Projects")
        .join(format!("{}.PRJ", params.proyecto.nombre));
    PasoProyectoSalida {
        nombre: params.proyecto.nombre.clone(),
        ok: true,
        raw_response: json!({
            "project": params.proyecto.nombre,
            "project_file": prj.to_string_lossy(),
            "reutilizado": true,
        }),
    }
}

/// Executes the project creation step from the given template, or resumes a
/// project an earlier run of the same design left behind.
pub(crate) fn ejecutar_paso_proyecto(
    paths: &Paths,
    params: &DisenoParametros,
) -> Result<PasoProyectoSalida> {
    if !params.proyecto.sobrescribir {
        match diferencias_con_lo_existente(paths, params) {
            None => {}
            Some(diferencias) if diferencias.is_empty() => return Ok(reutilizar(paths, params)),
            Some(diferencias) => {
                return Err(Error::Invalid(
                    Texto {
                        es: "El proyecto '{nombre}' ya existe con otros parámetros ({diferencias}): pasa sobrescribir:true para rehacerlo, u otro 'nombre'",
                        en: "Project '{nombre}' already exists with different parameters ({diferencias}): pass sobrescribir:true to rebuild it, or another 'nombre'",
                    }
                    .texto_con(
                        paths.idioma,
                        &[
                            ("nombre", &params.proyecto.nombre),
                            ("diferencias", &diferencias.join("; ")),
                        ],
                    ),
                ))
            }
        }
    }
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
