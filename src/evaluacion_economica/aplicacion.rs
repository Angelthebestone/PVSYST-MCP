//! `proyecto accion:economia`: edits the `EconomicEval` section PVsyst writes.

use crate::config::Paths;
use crate::error::{Error, Result};
use crate::evaluacion_economica::dominio::{EntradaEconomicos, ParametrosEconomicos};
use crate::format::seccion;
use crate::projects::lifecycle::{abrir_variante, guardar_variante, respuesta_de_edicion};
use serde_json::Value;

const SECCION: &str = "EconomicEval";
const TARIFA_FIJA: &str = "taFixed";

pub(crate) fn configurar_economia(paths: &Paths, args: &Value) -> Result<Value> {
    let parametros = ParametrosEconomicos::nuevo(EntradaEconomicos::de_args(args)?)?;
    let variante = abrir_variante(paths, args)?;
    if !seccion::existe_seccion(&variante.bytes, SECCION) {
        return Err(Error::NotFound(
            "La variante no tiene evaluación económica; créala una vez en la GUI de PVsyst \
(Economic evaluation) y luego ajústala con esta acción"
                .into(),
        ));
    }
    if parametros.tiene_tarifa_venta()
        && seccion::campo_seccion(&variante.bytes, SECCION, "TariffType").as_deref()
            != Some(TARIFA_FIJA)
    {
        return Err(Error::Invalid("Tarifa variable: edítala con editar".into()));
    }
    let (bytes, outcomes) =
        seccion::editar_campos_seccion(&variante.bytes, SECCION, &parametros.cambios())?;
    let historial_id = guardar_variante(&variante, &bytes, "proyecto economia")?;
    Ok(respuesta_de_edicion(
        &variante,
        &parametros.como_json(),
        (&outcomes, historial_id),
    )
    .into())
}
