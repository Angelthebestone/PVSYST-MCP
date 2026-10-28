//! `gui_observar que:esquema`: every window class with its coverage state.

use super::dominio::{Cobertura, Ventana};
use super::mapa::{clases_del_mapa, ResumenMapa};
use super::ventanas::todas;
use crate::gui_catalogo::funciones_que_abren;
use crate::vista_agente::esquemas;
use serde_json::{json, Value};

/// Fields of the schema entity that covers `ventana`, if it has one.
fn campos_mapeados(ventana: &Ventana) -> usize {
    let Some(clave) = ventana.esquema else {
        return 0;
    };
    match esquemas::entidad(clave) {
        Ok(entidad) => entidad.campos.len(),
        Err(error) => {
            tracing::warn!(clase = ventana.clase, clave, %error, "esquema registrado que no existe");
            0
        }
    }
}

fn ventana_json(ventana: &Ventana, mapa: Option<&ResumenMapa>) -> Value {
    json!({
        "clase": ventana.clase,
        "estado": ventana.cobertura.numero(),
        "tipo": ventana.cobertura.nombre(),
        "motivo": ventana.motivo,
        "esquema": ventana.esquema,
        "abierta_con": funciones_que_abren(ventana.clase),
        "campos_mapeados": campos_mapeados(ventana),
        "campos_en_mapa": mapa.map(|m| m.campos),
        "botones_en_mapa": mapa.map(|m| m.botones),
        "instancias": mapa.map(|m| m.instancias),
    })
}

/// All registered windows and how many fall in each state.
pub(crate) fn listar_ventanas() -> Value {
    let mapa = clases_del_mapa();
    let ventanas: Vec<Value> = todas()
        .map(|v| ventana_json(v, mapa.get(v.clase)))
        .collect();
    let por_estado: serde_json::Map<String, Value> = Cobertura::TODAS
        .iter()
        .map(|c| {
            let n = todas().filter(|v| v.cobertura == *c).count();
            (c.numero().to_string(), json!(n))
        })
        .collect();
    json!({ "total": ventanas.len(), "por_estado": por_estado, "ventanas": ventanas })
}
