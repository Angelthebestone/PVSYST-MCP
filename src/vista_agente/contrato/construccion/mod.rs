//! Builds the `vista` block of a finished call from its request and its answer.

mod avisos;
mod estado;
mod fallo;
mod recortes;
mod siguiente;

use super::campos::campos_de;
use super::catalogo;
use super::pendiente::{pendiente_de, Pendiente};
use super::tipos::{Estado, Vista};
use crate::idioma::{Idioma, Texto};
use avisos::{acotar_avisos, avisos_de};
use estado::{estado_de, origen_de};
use fallo::{error_de, motivo_de};
use recortes::recortes_de;
use serde_json::Value;
pub(crate) use siguiente::siguiente_de;

/// What the block needs to know about the call that produced the answer.
pub(crate) struct Contexto<'a> {
    pub(crate) tool: &'a str,
    pub(crate) accion: &'a str,
    pub(crate) args: &'a Value,
    pub(crate) idioma: Idioma,
}

const ACCION_COMPLETADA: Texto = Texto {
    es: "Acción completada.",
    en: "Action completed.",
};
const NO_COMPLETADA: Texto = Texto {
    es: "No se completó del todo: {motivo}.",
    en: "Not fully completed: {motivo}.",
};

/// The texts of `clave`, whether the answer holds one string or a list of them.
fn textos_de(salida: &Value, clave: &str) -> Vec<String> {
    match salida.get(clave) {
        Some(Value::String(s)) => vec![s.clone()],
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

/// The `vista` block for an answer that arrived, in the language of `ctx`.
pub(crate) fn construir(ctx: &Contexto<'_>, salida: &Value) -> Vista {
    let base = estado_de(salida);
    let pendiente = pendiente_de(salida, base);
    let estado = match pendiente {
        Some(Pendiente::Confirmacion) => Estado::Parcial,
        _ => base,
    };
    let (campos, recorte_campos) = campos_de(salida);
    let mut recortado = recortes_de(salida);
    recortado.extend(recorte_campos);
    let (avisos, recorte_avisos) = acotar_avisos(avisos_de(salida), ctx.idioma);
    recortado.extend(recorte_avisos);
    let hecho = catalogo::buscar(ctx.tool, ctx.accion);
    let resumen = match pendiente {
        Some(p) => p.resumen(ctx, salida),
        None => {
            let hecho = hecho.map_or(ACCION_COMPLETADA, |e| e.resumen);
            match estado {
                Estado::Ok => hecho.en_idioma(ctx.idioma).to_string(),
                _ => {
                    let motivo = motivo_de(salida, ctx.idioma);
                    NO_COMPLETADA.texto_con(ctx.idioma, &[("motivo", motivo.as_str())])
                }
            }
        }
    };
    Vista {
        estado,
        resumen,
        origen: origen_de(ctx, salida),
        campos,
        avisos,
        siguiente: siguiente_de(ctx, estado, salida),
        recortado,
        error: match pendiente {
            Some(p) => p.error(salida, ctx.idioma),
            None => error_de(estado, salida, ctx.idioma),
        },
    }
}

/// Whether the answer closed the call: it succeeded and nothing is pending, so
/// its `vista` repeats what the answer says and the caller may drop it.
pub(crate) fn es_final(vista: &Vista, salida: &Value) -> bool {
    vista.estado == Estado::Ok && pendiente_de(salida, vista.estado).is_none()
}
