//! Battery bank edits for stand-alone easy-mode designs.

use super::parametros::{DimensionadoParametros, DisenoParametros, TipoDiseno};
use crate::config::Paths;
use crate::error::Result;
use crate::format::edit::{self, ChangeOutcome, EdicionObjeto};
use crate::projects::lifecycle::{abrir_variante, guardar_variante};
use serde_json::{json, Value};

const MOTIVO: &str = "disenar_sistema: baterías";

/// Applies the battery counts after sizing; `None` when the design is not
/// stand-alone or no counts were given.
fn aplicar_baterias(
    paths: &Paths,
    params: &DisenoParametros,
) -> Result<Option<(Vec<ChangeOutcome>, Option<String>)>> {
    if params.tipo_sistema != TipoDiseno::Aislado {
        return Ok(None);
    }
    let cambios = cambios_baterias(&params.dimensionado);
    if cambios.is_empty() {
        return Ok(None);
    }
    let abierta = abrir_variante(
        paths,
        &json!({"proyecto": params.proyecto.nombre, "variante": params.variante}),
    )?;
    let (bytes, outcomes) = edit::editar_objetos(
        &abierta.bytes,
        &[EdicionObjeto {
            selector: "pvSystem".into(),
            cambios,
        }],
    )?;
    let historial_id = guardar_variante(&abierta, &bytes, MOTIVO)?;
    Ok(Some((outcomes, historial_id)))
}

/// Adds the battery edit outcomes to the sizing response when they exist.
pub(crate) fn aplicar_baterias_al_resultado(
    paths: &Paths,
    params: &DisenoParametros,
    sizing: Value,
) -> Result<Value> {
    let Some((outcomes, historial_id)) = aplicar_baterias(paths, params)? else {
        return Ok(sizing);
    };
    let mut detalle = sizing;
    if let Some(obj) = detalle.as_object_mut() {
        obj.insert(
            "baterias".into(),
            json!(outcomes
                .iter()
                .map(ChangeOutcome::to_json)
                .collect::<Vec<_>>()),
        );
        obj.insert("baterias_historial_id".into(), json!(historial_id));
    }
    Ok(detalle)
}

fn cambios_baterias(params: &DimensionadoParametros) -> Vec<(String, Option<String>)> {
    let serie = params
        .n_baterias_serie
        .map(|n| ("NBattSerie".to_string(), Some(n.to_string())));
    let paralelo = params
        .n_baterias_paralelo
        .map(|n| ("NBattParall".to_string(), Some(n.to_string())));
    serie.into_iter().chain(paralelo).collect()
}
