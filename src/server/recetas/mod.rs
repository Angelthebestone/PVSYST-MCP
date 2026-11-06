//! Ready-to-use recipes, exposed both as the `pvsyst://recetas` resource and as
//! MCP prompts. The prompt template is filled from the caller's arguments.

mod datos;
mod recetas_consulta;
mod recetas_diseno;

use crate::args;
use crate::error::{Error, Result};
use crate::idioma::Idioma;
use crate::idioma::Texto;
use datos::{ArgumentoReceta, Receta, Requerido, CONSEJO, INTRO};

const RECETA_ID: Texto = Texto {
    es: "id de la receta",
    en: "recipe id",
};
const PROMPT_DESCONOCIDO: Texto = Texto {
    es: "Prompt desconocido: {nombre}",
    en: "Unknown prompt: {nombre}",
};
use datos::RECETAS;
use serde_json::{json, Value};

fn visibles(aproximaciones: bool) -> impl Iterator<Item = &'static Receta> {
    RECETAS
        .iter()
        .flat_map(|grupo| grupo.iter())
        .filter(move |r| !r.requiere_aproximaciones || aproximaciones)
}

/// Resource payload for `pvsyst://recetas`.
pub(crate) fn como_recurso(aproximaciones: bool, idioma: Idioma) -> Value {
    let recetas: Vec<Value> = visibles(aproximaciones)
        .map(|receta| recurso_de(receta, idioma))
        .collect();
    json!({
        "intro": INTRO.en_idioma(idioma),
        "recetas": recetas,
        "consejo": CONSEJO.en_idioma(idioma),
    })
}

fn recurso_de(receta: &Receta, idioma: Idioma) -> Value {
    let mut valor = json!({
        "nombre": receta.nombre.en_idioma(idioma),
        "cuando": receta.cuando.en_idioma(idioma),
        "tools": receta.tools,
    });
    if let Some(texto) = receta.ejemplo {
        if let Ok(ejemplo) = serde_json::from_str::<Value>(texto) {
            valor["ejemplo"] = ejemplo;
        }
    }
    if let Some(nota) = receta.nota {
        valor["nota"] = json!(nota.en_idioma(idioma));
    }
    valor
}

/// Prompt catalogue for `prompts/list`.
pub(crate) fn listar_prompts(aproximaciones: bool, idioma: Idioma) -> Value {
    let prompts: Vec<Value> = visibles(aproximaciones)
        .map(|receta| prompt_de(receta, idioma))
        .collect();
    json!({ "prompts": prompts })
}

fn prompt_de(receta: &Receta, idioma: Idioma) -> Value {
    let argumentos: Vec<Value> = receta
        .argumentos
        .iter()
        .map(|a| {
            json!({
                "name": a.nombre,
                "description": a.descripcion.en_idioma(idioma),
                "required": matches!(a.requerido, Requerido::Si),
            })
        })
        .collect();
    json!({
        "name": receta.id,
        "title": receta.nombre.en_idioma(idioma),
        "description": receta.cuando.en_idioma(idioma),
        "arguments": argumentos,
    })
}

/// One filled prompt for `prompts/get`.
pub(crate) fn obtener_prompt(
    params: &Value,
    aproximaciones: bool,
    idioma: Idioma,
) -> Result<Value> {
    let nombre = params
        .get("name")
        .and_then(Value::as_str)
        .filter(|n| !n.is_empty())
        .ok_or_else(|| args::missing("name", RECETA_ID.en_idioma(idioma)))?;
    let receta = visibles(aproximaciones)
        .find(|r| r.id == nombre)
        .ok_or_else(|| {
            Error::NotFound(PROMPT_DESCONOCIDO.texto_con(idioma, &[("nombre", nombre)]))
        })?;
    let argumentos = params.get("arguments").unwrap_or(&Value::Null);
    let texto = rellenar(receta, argumentos, idioma)?;
    Ok(json!({
        "description": receta.cuando.en_idioma(idioma),
        "messages": [{
            "role": "user",
            "content": { "type": "text", "text": texto },
        }],
    }))
}

fn rellenar(receta: &Receta, recibidos: &Value, idioma: Idioma) -> Result<String> {
    receta.argumentos.iter().try_fold(
        receta.plantilla.en_idioma(idioma).to_string(),
        |texto, argumento| {
            let valor = valor_de(argumento, recibidos.get(argumento.nombre), idioma)?;
            Ok(texto.replace(&format!("{{{}}}", argumento.nombre), &valor))
        },
    )
}

fn valor_de(
    argumento: &ArgumentoReceta,
    recibido: Option<&Value>,
    idioma: Idioma,
) -> Result<String> {
    if let Some(texto) = recibido.and_then(texto_de) {
        return Ok(texto);
    }
    match argumento.requerido {
        Requerido::Si => Err(args::missing(
            argumento.nombre,
            argumento.descripcion.en_idioma(idioma),
        )),
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
