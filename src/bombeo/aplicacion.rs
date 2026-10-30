//! `proyecto accion:bombeo`: configures flow, depth and pump on a pumping
//! variant.

use crate::bombeo::dominio::{EntradaBombeo, ParametrosBombeo};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::{edit, seccion};
use crate::projects::lifecycle::{abrir_variante, guardar_variante, respuesta_de_edicion};
use crate::projects::tipo_sistema::TipoSistema;
use serde_json::Value;

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
        seccion::editar_campos_seccion(&actual, "WaterUse", &cambios_seccion)
            .map_err(sin_seccion_water_use)?
    };
    let outcomes: Vec<edit::ChangeOutcome> = sistema.into_iter().chain(agua).collect();
    let historial_id = guardar_variante(&variante, &bytes, "proyecto bombeo")?;
    Ok(respuesta_de_edicion(
        &variante,
        &parametros.como_json(),
        (&outcomes, historial_id),
    )
    .into())
}

/// Variants made with `anadir circuito_hidraulico` carry the pump circuit but not the
/// `WaterUse` section, which lives in the pumping template's `pvLoad`. Say so instead of
/// the generic section-not-found error.
fn sin_seccion_water_use(error: Error) -> Error {
    match error {
        Error::NotFound(_) => Error::Invalid(
            "La variante no tiene la sección 'WaterUse' (solo la trae una plantilla de bombeo completa; 'anadir circuito_hidraulico' copia únicamente el circuito). Cree la variante desde _DEMO_Pumping.".into(),
        ),
        otro => otro,
    }
}
