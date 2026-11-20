//! The next calls the block offers: the catalog's steps, the same call completed
//! when the answer is pending, the diagnostic call when it did not finish.

use super::super::catalogo::{self, Paso};
use super::super::pendiente::pendiente_de;
use super::super::tipos::{Estado, PasoSiguiente};
use super::Contexto;
use serde_json::{Map, Value};

/// Arguments of the request that carry over to every next step; the server drops
/// the ones the target tool does not declare.
const PROPAGADOS: &[&str] = &["workspace", "variante", "permitir_fuera_del_workspace"];

fn texto_de<'a>(fuente: &'a Value, claves: &[&str]) -> Option<&'a str> {
    claves
        .iter()
        .find_map(|k| fuente.get(k).and_then(Value::as_str))
}

/// Arguments of `paso`, or `None` when a value it must take is in neither the request nor the answer.
fn argumentos_de(paso: &Paso, ctx: &Contexto<'_>, salida: &Value) -> Option<Value> {
    let mut argumentos: Map<String, Value> = paso
        .fijos
        .iter()
        .map(|(k, v)| ((*k).to_string(), Value::from(*v)))
        .collect();
    for (destino, candidatos) in paso.tomar {
        let texto = texto_de(ctx.args, candidatos).or_else(|| texto_de(salida, candidatos))?;
        argumentos.insert((*destino).to_string(), Value::from(texto));
    }
    let heredados = PROPAGADOS
        .iter()
        .filter_map(|clave| Some(((*clave).to_string(), ctx.args.get(clave)?.clone())));
    argumentos.extend(heredados);
    for (destino, candidatos) in paso.si_esta {
        if let Some(texto) = texto_de(ctx.args, candidatos).or_else(|| texto_de(salida, candidatos))
        {
            argumentos.insert((*destino).to_string(), Value::from(texto));
        }
    }
    Some(Value::Object(argumentos))
}

fn paso_siguiente(paso: &Paso, ctx: &Contexto<'_>, salida: &Value) -> Option<PasoSiguiente> {
    Some(PasoSiguiente {
        tool: paso.tool.to_string(),
        argumentos: argumentos_de(paso, ctx, salida)?,
        motivo: paso.motivo.en_idioma(ctx.idioma),
    })
}

/// Next calls of the entry; the same call completed when the answer is pending;
/// the diagnostic call when the action did not finish.
pub(crate) fn siguiente_de(
    ctx: &Contexto<'_>,
    estado: Estado,
    salida: &Value,
) -> Vec<PasoSiguiente> {
    if let Some(pendiente) = pendiente_de(salida, estado) {
        return vec![pendiente.siguiente(ctx)];
    }
    let pasos: &[Paso] = match (estado, catalogo::buscar(ctx.tool, ctx.accion)) {
        (Estado::Ok, Some(entrada)) => entrada.siguiente,
        _ => std::slice::from_ref(&catalogo::DIAGNOSTICO),
    };
    pasos
        .iter()
        .filter_map(|p| paso_siguiente(p, ctx, salida))
        .collect()
}
