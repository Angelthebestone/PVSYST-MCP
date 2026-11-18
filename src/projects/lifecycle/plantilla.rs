//! Where a new project comes from: the template project, the site it is asked to
//! take, and the folder the copy is written to.

use crate::error::{Error, Result};
use crate::workspace::Alcance;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Resolves the base project: the workspace first, then the read-only DataRO demo projects.
pub(super) fn resolver_plantilla(
    workspace: &Path,
    plantilla: &str,
    alcance: Alcance,
) -> Result<PathBuf> {
    match crate::projects::resolve_project_file_con_alcance(workspace, plantilla, alcance) {
        Ok(p) => Ok(p),
        Err(e) => plantilla_demo_ro(plantilla).ok_or(e),
    }
}

/// A PVsyst demo project in DataRO named `plantilla`, with or without `.PRJ`.
fn plantilla_demo_ro(plantilla: &str) -> Option<PathBuf> {
    if !crate::workspace::es_relativa_simple(Path::new(plantilla)) {
        return None;
    }
    let base = crate::config::proyectos_demo_ro();
    [plantilla.to_string(), format!("{plantilla}.PRJ")]
        .iter()
        .map(|c| base.join(c))
        .find(|p| p.is_file())
}

/// Output always goes to the workspace: a DataRO template is never written beside.
pub(super) fn carpeta_de_salida(workspace: &Path, src_prj: &Path, src_parent: &Path) -> PathBuf {
    if src_prj.starts_with(workspace) {
        src_parent.to_path_buf()
    } else {
        workspace.join("Projects")
    }
}

/// Site requested with `sitio`, already reduced to the name `Sites/<nombre>.SIT`.
struct SitioPedido {
    nombre: String,
}

impl TryFrom<&str> for SitioPedido {
    type Error = crate::error::Error;

    fn try_from(valor: &str) -> Result<Self> {
        let normal = valor.replace('\\', "/");
        let archivo = match normal.split_once('/') {
            Some((carpeta, resto)) if carpeta.eq_ignore_ascii_case("Sites") => resto,
            Some(_) => return Err(sitio_invalido(valor, "")),
            None => normal.as_str(),
        };
        if archivo.contains('/') {
            return Err(sitio_invalido(valor, ""));
        }
        let nombre = match archivo.rsplit_once('.') {
            Some((nombre, extension)) if extension.eq_ignore_ascii_case("SIT") => nombre,
            _ => archivo,
        };
        crate::workspace::validar_nombre(nombre)
            .map_err(|e| sitio_invalido(valor, &e.to_string()))?;
        Ok(Self {
            nombre: nombre.to_string(),
        })
    }
}

impl SitioPedido {
    fn ruta_en(&self, dir: &Path) -> PathBuf {
        dir.join(format!("{}.SIT", self.nombre))
    }
}

fn sitio_invalido(valor: &str, razon: &str) -> Error {
    let detalle = if razon.is_empty() {
        String::new()
    } else {
        format!(" ({razon})")
    };
    Error::Invalid(format!(
        "'sitio' inválido '{valor}': se espera el nombre del sitio (con o sin '.SIT') o la ruta 'Sites/X.SIT'{detalle}"
    ))
}

/// The workspace `Sites/<sitio>.SIT` requested with `sitio`; a missing site is an error.
pub(super) fn sit_pedido(workspace: &Path, args: &Value) -> Result<Option<PathBuf>> {
    let Some(valor) = args.get("sitio").and_then(Value::as_str) else {
        return Ok(None);
    };
    let sitio = SitioPedido::try_from(valor)?;
    let dir = workspace.join("Sites");
    let sit = sitio.ruta_en(&dir);
    if !sit.is_file() {
        return Err(crate::args::no_encontrado(
            crate::args::SITIO,
            valor,
            &[],
            &dir.to_string_lossy(),
        ));
    }
    Ok(Some(sit))
}

/// Variant the new project starts from: `variante`, `VC0` by default.
pub(super) fn variante_pedida(args: &Value) -> String {
    args.get("variante")
        .and_then(Value::as_str)
        .unwrap_or("VC0")
        .to_uppercase()
}
