//! `proyecto accion:limite_inyeccion`: sets the grid injection limit on a grid
//! variant.

use crate::config::Paths;
use crate::error::Result;
use crate::format::edit::{self, EdicionObjeto};
use crate::inyeccion::dominio::{EntradaInyeccion, ParametrosInyeccion};
use crate::projects::lifecycle::{abrir_variante, guardar_variante, respuesta_de_edicion};
use crate::projects::tipo_sistema::TipoSistema;
use serde_json::Value;

pub(crate) fn definir_limite(paths: &Paths, args: &Value) -> Result<Value> {
    let parametros = ParametrosInyeccion::nuevo(EntradaInyeccion::de_args(args)?)?;
    let variante = abrir_variante(paths, args)?;
    TipoSistema::exigir(&variante.bytes, TipoSistema::Red)?;
    let (bytes, outcomes) = edit::editar_objetos(
        &variante.bytes,
        &[EdicionObjeto {
            selector: "pvSystem".into(),
            cambios: parametros.cambios(),
        }],
    )?;
    let historial_id = guardar_variante(&variante, &bytes, "proyecto limite_inyeccion")?;
    Ok(respuesta_de_edicion(
        &variante,
        &parametros.como_json(),
        (&outcomes, historial_id),
    )
    .into())
}
