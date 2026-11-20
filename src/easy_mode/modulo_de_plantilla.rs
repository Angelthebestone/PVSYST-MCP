//! Which module a design sizes with: the one the template names, or a substitute
//! from the workspace database when that file does not exist.

use super::parametros::{DisenoParametros, TipoDiseno};
use super::serie_plantilla::Referencias;
use crate::components::pan::{self, DatosModulo};
use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::{json, Value};

/// Module data read for sizing plus the substitute module name when the template's own
/// module is not in any database.
pub(super) struct ModuloResuelto {
    pub(super) datos: Option<DatosModulo>,
    pub(super) sustituto: Option<String>,
}

pub(super) fn leer_modulo(
    paths: &Paths,
    params: &DisenoParametros,
    refs: &Referencias,
) -> Result<ModuloResuelto> {
    let explicito = params.dimensionado.modulo.is_some();
    let sin_modulo = ModuloResuelto {
        datos: None,
        sustituto: None,
    };
    let Some(m) = params
        .dimensionado
        .modulo
        .as_deref()
        .or(refs.modulo.as_deref())
    else {
        return Ok(sin_modulo);
    };
    match pan::leer_modulo(paths, &json!({ "modulo": m })) {
        Ok(val) => Ok(ModuloResuelto {
            datos: DatosModulo::de_lectura(&val),
            sustituto: None,
        }),
        Err(Error::NotFound(_)) if !explicito => sustituir_modulo(paths, params, m),
        Err(e) => Err(Error::Invalid(format!(
            "Error al leer el módulo '{m}': {e}"
        ))),
    }
}

/// The template's module is missing: size with the first module of the workspace or
/// DataRO that has the electrical figures sizing needs (stand-alone designs keep
/// their own sizing path). A module without them cannot be sized, so it is skipped
/// instead of being taken just because it sorts first.
fn sustituir_modulo(
    paths: &Paths,
    params: &DisenoParametros,
    faltante: &str,
) -> Result<ModuloResuelto> {
    if params.tipo_sistema == TipoDiseno::Aislado {
        return Ok(ModuloResuelto {
            datos: None,
            sustituto: None,
        });
    }
    let ws = crate::args::workspace_of(paths, &Value::Null);
    for nombre in pan::modulos_disponibles(&ws) {
        let Ok(val) = pan::leer_modulo(paths, &json!({ "modulo": nombre })) else {
            continue;
        };
        if let Some(datos) = DatosModulo::de_lectura(&val) {
            return Ok(ModuloResuelto {
                datos: Some(datos),
                sustituto: Some(nombre),
            });
        }
    }
    Err(Error::Invalid(format!(
        "El módulo '{faltante}' de la plantilla no existe y ningún .PAN del workspace ni de DataRO tiene Pnom, Voc y Vmpp: pasa 'modulo'"
    )))
}
