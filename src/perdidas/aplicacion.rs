//! `proyecto accion:perdidas`: applies variant loss percentages.

use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::edit;
use crate::perdidas::dominio::{usa_formato_nuevo_suciedad, EntradaPerdidas, ParametrosPerdidas};
use crate::projects::lifecycle::{abrir_variante, guardar_variante};
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
    let n_subarrays = edit::spans_of(&variante.bytes)
        .iter()
        .filter(|span| span.ty.eq_ignore_ascii_case("pvSubArray"))
        .count();
    if parametros.pide_subarrays() && n_subarrays == 0 {
        return Err(Error::NotFound(
            "La variante no tiene objetos pvSubArray".into(),
        ));
    }
    let (bytes, outcomes) =
        edit::editar_objetos(&variante.bytes, &parametros.ediciones(n_subarrays))?;
    let historial_id = guardar_variante(&variante, &bytes, "proyecto perdidas")?;
    Ok(json!({
        "proyecto": variante.stem,
        "variante": variante.variante,
        "archivo": variante.vpath.display().to_string(),
        "n_subarrays": n_subarrays,
        "parametros": parametros.como_json(),
        "cambios": outcomes.iter().map(edit::ChangeOutcome::to_json).collect::<Vec<_>>(),
        "historial_id": historial_id,
    }))
}
