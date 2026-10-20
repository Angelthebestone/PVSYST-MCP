//! Ready-to-use recipes, exposed both as the `pvsyst://recetas` resource and as
//! MCP prompts. The prompt template is filled from the caller's arguments.

mod datos;

use crate::args;
use crate::error::{Error, Result};
use datos::{ArgumentoReceta, Receta, Requerido, CONSEJO, INTRO};
use serde_json::{json, Value};

use datos::RECETAS;

/// Resource payload for `pvsyst://recetas`.
pub(crate) fn como_recurso() -> Value {
    let recetas: Vec<Value> = RECETAS.iter().map(recurso_de).collect();
    json!({
        "intro": INTRO,
        "recetas": recetas,
        "consejo": CONSEJO,
    })
}

fn recurso_de(receta: &Receta) -> Value {
    let mut valor = json!({
        "nombre": receta.nombre,
        "cuando": receta.cuando,
        "tools": receta.tools,
    });
    if let Some(texto) = receta.ejemplo {
        if let Ok(ejemplo) = serde_json::from_str::<Value>(texto) {
            valor["ejemplo"] = ejemplo;
        }
    }
    if let Some(nota) = receta.nota {
        valor["nota"] = json!(nota);
    }
    valor
}

/// Prompt catalogue for `prompts/list`.
pub(crate) fn listar_prompts() -> Value {
    let prompts: Vec<Value> = RECETAS.iter().map(prompt_de).collect();
    json!({ "prompts": prompts })
}

fn prompt_de(receta: &Receta) -> Value {
    let argumentos: Vec<Value> = receta
        .argumentos
        .iter()
        .map(|a| {
            json!({
                "name": a.nombre,
                "description": a.descripcion,
                "required": matches!(a.requerido, Requerido::Si),
            })
        })
        .collect();
    json!({
        "name": receta.id,
        "title": receta.nombre,
        "description": receta.cuando,
        "arguments": argumentos,
    })
}

/// One filled prompt for `prompts/get`.
pub(crate) fn obtener_prompt(params: &Value) -> Result<Value> {
    let nombre = params
        .get("name")
        .and_then(Value::as_str)
        .filter(|n| !n.is_empty())
        .ok_or_else(|| args::missing("name", "id de la receta"))?;
    let receta = RECETAS
        .iter()
        .find(|r| r.id == nombre)
        .ok_or_else(|| Error::NotFound(format!("Prompt desconocido: {nombre}")))?;
    let argumentos = params.get("arguments").unwrap_or(&Value::Null);
    let texto = rellenar(receta, argumentos)?;
    Ok(json!({
        "description": receta.cuando,
        "messages": [{
            "role": "user",
            "content": { "type": "text", "text": texto },
        }],
    }))
}

fn rellenar(receta: &Receta, recibidos: &Value) -> Result<String> {
    receta
        .argumentos
        .iter()
        .try_fold(receta.plantilla.to_string(), |texto, argumento| {
            let valor = valor_de(argumento, recibidos.get(argumento.nombre))?;
            Ok(texto.replace(&format!("{{{}}}", argumento.nombre), &valor))
        })
}

fn valor_de(argumento: &ArgumentoReceta, recibido: Option<&Value>) -> Result<String> {
    if let Some(texto) = recibido.and_then(texto_de) {
        return Ok(texto);
    }
    match argumento.requerido {
        Requerido::Si => Err(args::missing(argumento.nombre, argumento.descripcion)),
        Requerido::No => Ok(argumento.defecto.to_string()),
    }
}

fn texto_de(valor: &Value) -> Option<String> {
    match valor {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/server/recetas.rs"]
mod tests;
