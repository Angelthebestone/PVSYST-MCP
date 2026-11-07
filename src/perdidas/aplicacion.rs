//! `proyecto accion:perdidas`: applies variant loss percentages.

use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::edit;
use crate::perdidas::dominio::{
    subarrays_de, usa_formato_nuevo_suciedad, EntradaPerdidas, ParametrosPerdidas,
};
use crate::projects::lifecycle::{abrir_variante, guardar_variante, respuesta_de_edicion};
use serde_json::{json, Value};

pub(crate) fn aplicar_perdidas(paths: &Paths, args: &Value) -> Result<Value> {
    let parametros = ParametrosPerdidas::nuevo(EntradaPerdidas::de_args(args)?)?;
    let variante = abrir_variante(paths, args)?;
    let texto = String::from_utf8_lossy(&variante.bytes);
    if parametros.tiene_suciedad() && usa_formato_nuevo_suciedad(&texto) {
        return Err(Error::Invalid(
            "Formato nuevo de suciedad: edítalo con editar".into(),
        ));
    }
    let subarrays = subarrays_de(&variante.bytes);
    if parametros.pide_subarrays() && subarrays.is_empty() {
        return Err(Error::NotFound(
            "La variante no tiene objetos pvSubArray".into(),
        ));
    }
    let (bytes, outcomes) =
        edit::editar_objetos(&variante.bytes, &parametros.ediciones(&subarrays))?;
    let historial_id = guardar_variante(&variante, &bytes, "proyecto perdidas")?;
    let mut out = respuesta_de_edicion(
        &variante,
        &parametros.como_json(),
        (&outcomes, historial_id),
    );
    out.insert("n_subarrays".into(), json!(subarrays.len()));
    let avisos = parametros.avisos(&subarrays);
    if !avisos.is_empty() {
        out.insert("avisos".into(), json!(avisos));
    }
    Ok(out.into())
}
