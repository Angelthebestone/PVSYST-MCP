//! `proyecto accion:bombeo`: configures flow, depth and pump on a pumping
//! variant.

use crate::bombeo::dominio::{EntradaBombeo, ParametrosBombeo};
use crate::config::Paths;
use crate::error::Result;
use crate::format::{edit, seccion};
use crate::projects::lifecycle::{abrir_variante, guardar_variante};
use crate::projects::tipo_sistema::TipoSistema;
use serde_json::{json, Value};

pub(crate) fn configurar_bombeo(paths: &Paths, args: &Value) -> Result<Value> {
    let parametros = ParametrosBombeo::nuevo(EntradaBombeo::de_args(args)?)?;
    let mut variante = abrir_variante(paths, args)?;
    TipoSistema::exigir(&variante.bytes, TipoSistema::Bombeo)?;

    let actual = std::mem::take(&mut variante.bytes);
    let (actual, sistema) = match parametros.edicion_sistema() {
        Some(edicion) => edit::editar_objetos(&actual, &[edicion])?,
        None => (actual, Vec::new()),
    };
    let cambios_seccion = parametros.cambios_seccion();
    let (bytes, agua) = if cambios_seccion.is_empty() {
        (actual, Vec::new())
    } else {
        seccion::editar_campos_seccion(&actual, "WaterUse", &cambios_seccion)?
    };
    let outcomes: Vec<edit::ChangeOutcome> = sistema.into_iter().chain(agua).collect();
    let historial_id = guardar_variante(&variante, &bytes, "proyecto bombeo")?;
    Ok(json!({
        "proyecto": variante.stem,
        "variante": variante.variante,
        "archivo": variante.vpath.display().to_string(),
        "parametros": parametros.como_json(),
        "cambios": outcomes.iter().map(edit::ChangeOutcome::to_json).collect::<Vec<_>>(),
        "historial_id": historial_id,
    }))
}
