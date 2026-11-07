//! Declared JSON types and integer bounds of tool arguments.

use super::declaracion::{nombre_tipo_recibido, TipoDato};
use super::restricciones_esquema::limites_enteros;
use super::validacion::TIPO_INCORRECTO;
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use serde_json::Value;

const RANGO_MINIMO: Texto = Texto {
    es: "mínimo {minimo}",
    en: "minimum {minimo}",
};
const RANGO_ENTRE: Texto = Texto {
    es: "entre {minimo} y {max}",
    en: "between {minimo} and {max}",
};
const FUERA_DE_RANGO: Texto = Texto {
    es: "Valor {n} fuera de rango para '{prop}' ({rango})",
    en: "Value {n} out of range for '{prop}' ({rango})",
};

pub(super) fn validar_tipo_propiedad(
    prop: &(&'static str, TipoDato),
    args: &Value,
    idioma: Idioma,
) -> Result<()> {
    let (prop_nombre, prop_tipo) = *prop;
    let Some(val) = args.get(prop_nombre) else {
        return Ok(());
    };
    if val.is_null() {
        return Ok(());
    }
    if !prop_tipo.es_compatible(val) {
        return Err(Error::Invalid(TIPO_INCORRECTO.texto_con(
            idioma,
            &[
                ("campo", prop_nombre),
                ("esperado", prop_tipo.nombre_esperado()),
                ("recibido", nombre_tipo_recibido(val)),
            ],
        )));
    }
    validar_limites_enteros(prop_nombre, val, idioma)
}

/// Rejects an integer outside the bounds the schema declares for `prop_nombre`.
fn validar_limites_enteros(prop_nombre: &str, val: &Value, idioma: Idioma) -> Result<()> {
    let (Some((minimo, maximo)), Some(n)) = (limites_enteros(prop_nombre), val.as_i64()) else {
        return Ok(());
    };
    if n >= minimo && maximo.is_none_or(|max| n <= max) {
        return Ok(());
    }
    let minimo = minimo.to_string();
    let rango = maximo.map_or_else(
        || RANGO_MINIMO.texto_con(idioma, &[("minimo", &minimo)]),
        |max| RANGO_ENTRE.texto_con(idioma, &[("minimo", &minimo), ("max", &max.to_string())]),
    );
    Err(Error::Invalid(FUERA_DE_RANGO.texto_con(
        idioma,
        &[
            ("n", &n.to_string()),
            ("prop", prop_nombre),
            ("rango", &rango),
        ],
    )))
}
