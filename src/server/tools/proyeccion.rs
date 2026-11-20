//! `listar` field projection: with `campos` an item keeps only the keys the caller
//! named, so a caller that needs two fields of a project does not pay for the other
//! five. A name that no item carries is an error that lists the ones they do carry.

use super::declaracion::nombre_tipo_recibido;
use super::validacion::TIPO_INCORRECTO;
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use serde_json::{Map, Value};

const CAMPO_DESCONOCIDO: Texto = Texto {
    es: "Valor '{campo}' no es un campo de listar {que}; campos: {campos}",
    en: "Value '{campo}' is not a field of listar {que}; fields: {campos}",
};

/// The call a projection belongs to: the action it answers, its arguments and the
/// language of its errors.
pub(super) struct Peticion<'a> {
    pub(super) accion: &'a str,
    pub(super) args: &'a Value,
    pub(super) idioma: Idioma,
}

/// Keeps in each item of the answer's list only the keys `campos` names. `false`
/// when nothing was projected: no `campos`, or an answer with no list of objects,
/// and only the second case leaves a given `campos` to be reported as ignored.
pub(super) fn aplicar(resultado: &mut Value, peticion: &Peticion<'_>) -> Result<bool> {
    let Some(pedidos) = peticion.args.get("campos").filter(|v| !v.is_null()) else {
        return Ok(false);
    };
    let nombres = nombres_de_campos(pedidos, peticion.idioma)?;
    let Some(clave) = clave_de_lista(peticion.accion) else {
        return Ok(false);
    };
    let Some(items) = resultado.get(clave).and_then(Value::as_array) else {
        return Ok(false);
    };
    let proyectados = proyectar_items(items, &nombres, peticion)?;
    let Some(slot) = resultado.get_mut(clave) else {
        return Ok(false);
    };
    *slot = Value::Array(proyectados);
    Ok(true)
}

/// The array of objects each `listar` action answers with.
fn clave_de_lista(accion: &str) -> Option<&'static str> {
    match accion {
        "proyectos" => Some("projects"),
        "componentes" => Some("components"),
        "modulos" => Some("modules"),
        "archivos" => Some("files"),
        "objetos" => Some("objects"),
        _ => None,
    }
}

/// The names `campos` asks for; an element that is not a string is a wrong input.
fn nombres_de_campos(pedidos: &Value, idioma: Idioma) -> Result<Vec<String>> {
    let Some(lista) = pedidos.as_array() else {
        return Ok(Vec::new());
    };
    lista
        .iter()
        .map(|valor| match valor.as_str() {
            Some(texto) => Ok(texto.to_string()),
            None => Err(Error::Invalid(TIPO_INCORRECTO.texto_con(
                idioma,
                &[
                    ("campo", "campos"),
                    ("esperado", "string"),
                    ("recibido", nombre_tipo_recibido(valor)),
                ],
            ))),
        })
        .collect()
}

/// Each item with only the named keys; a name no item carries is an error that
/// lists the keys they do carry.
fn proyectar_items(
    items: &[Value],
    nombres: &[String],
    peticion: &Peticion<'_>,
) -> Result<Vec<Value>> {
    let validos = claves_de_los_items(items);
    if validos.is_empty() {
        // An empty list has no field to drop, so no name can be wrong.
        return Ok(items.to_vec());
    }
    for nombre in nombres {
        if !validos.iter().any(|clave| clave == nombre) {
            return Err(Error::Invalid(CAMPO_DESCONOCIDO.texto_con(
                peticion.idioma,
                &[
                    ("campo", nombre.as_str()),
                    ("que", peticion.accion),
                    ("campos", &validos.join(", ")),
                ],
            )));
        }
    }
    Ok(items
        .iter()
        .map(|item| proyectar_item(item, nombres))
        .collect())
}

/// Every key the items carry, in the order they first appear.
fn claves_de_los_items(items: &[Value]) -> Vec<String> {
    let mut claves: Vec<String> = Vec::new();
    for clave in items
        .iter()
        .filter_map(Value::as_object)
        .flat_map(Map::keys)
    {
        if !claves.contains(clave) {
            claves.push(clave.clone());
        }
    }
    claves
}

/// An item with only `nombres`, in that order; a key the item lacks is left out.
fn proyectar_item(item: &Value, nombres: &[String]) -> Value {
    let Some(mapa) = item.as_object() else {
        return item.clone();
    };
    let mut proyectado = Map::new();
    for nombre in nombres {
        if let Some(valor) = mapa.get(nombre) {
            proyectado.insert(nombre.clone(), valor.clone());
        }
    }
    Value::Object(proyectado)
}
