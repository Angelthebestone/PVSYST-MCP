//! `listar` field projection: with `campos` an item keeps only the keys the caller
//! named, so a caller that needs two fields of a project does not pay for the other
//! five. The names a `que` accepts and the lists it answers with come from its
//! contract, never from the items an answer happens to carry: a filtered, empty or
//! warning-free answer accepts the same names as a full one.

use super::declaracion::nombre_tipo_recibido;
use super::validacion::TIPO_INCORRECTO;
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use serde_json::{Map, Value};

const CAMPO_DESCONOCIDO: Texto = Texto {
    es: "Valor '{campo}' no es un campo de listar {que}; campos: {campos}",
    en: "Value '{campo}' is not a field of listar {que}; fields: {campos}",
};
const CAMPOS_VACIOS: Texto = Texto {
    es: "'campos' no puede ir vacío: nombra al menos un campo de listar {que}; campos: {campos}",
    en: "'campos' cannot be empty: name at least one field of listar {que}; fields: {campos}",
};

/// The contract of one `listar` `que`: the lists of objects its answer carries and
/// every field their items can hold — the optional ones (`aviso`, written when a
/// .PRJ cannot be parsed; `error`, when a .PAN cannot) included, because this is the
/// list a `campos` name is checked against.
struct Contrato {
    que: &'static str,
    listas: &'static [&'static str],
    campos: &'static [&'static str],
}

const CONTRATOS: &[Contrato] = &[
    Contrato {
        que: "proyectos",
        // The workspace projects and the read-only demo templates: an item keeps the
        // named keys it has, and one of the two lists lacks `aviso` or `solo_lectura`.
        listas: &["projects", "templates"],
        campos: &[
            "name",
            "prj",
            "modified_unix",
            "comment",
            "version",
            "meteo_file",
            "variants",
            "aviso",
            "solo_lectura",
        ],
    },
    Contrato {
        que: "componentes",
        listas: &["components"],
        campos: &["file", "size_bytes"],
    },
    Contrato {
        que: "modulos",
        listas: &["modules"],
        campos: &["modulo", "pnom_w", "vendor", "model", "error"],
    },
    Contrato {
        que: "archivos",
        listas: &["files"],
        campos: &["relative", "size_bytes", "modified_unix"],
    },
    Contrato {
        que: "objetos",
        listas: &["objects"],
        campos: &["type", "role", "depth", "line_open", "line_close"],
    },
];

/// The field names a checked `campos` asks for, with the lists they apply to: holding
/// one proves each name is a field of its `que`, so the projection cannot fail.
#[must_use]
pub(super) struct CamposPedidos {
    listas: &'static [&'static str],
    nombres: Vec<String>,
}

/// What a projection did with an answer.
#[must_use]
#[derive(PartialEq, Eq)]
pub(super) enum Proyeccion {
    /// The call named no `campos`: every field of every item stays.
    Ninguna,
    /// Each item of each list the answer carries keeps only the named fields.
    Aplicada,
    /// `campos` was given but the answer carries none of the lists of its `que`, so
    /// the caller names it in `ignorados` instead of failing.
    SinLista,
}

/// Reads `campos` against the contract of `que`, before the listing runs: `None`
/// when the call names none, the names it asks for when every one is a field of the
/// `que`, and otherwise a refusal that lists those fields.
pub(super) fn pedir_campos(
    args: &Value,
    que: &str,
    idioma: Idioma,
) -> Result<Option<CamposPedidos>> {
    let Some(pedidos) = args.get("campos").filter(|v| !v.is_null()) else {
        return Ok(None);
    };
    // An unsupported `que` is refused by the listing with its own message naming the
    // supported ones, which a `campos` refusal in its place would only hide.
    let Some(contrato) = contrato_de(que) else {
        return Ok(None);
    };
    let Some(lista) = pedidos.as_array() else {
        return Err(tipo_incorrecto(pedidos, "array", idioma));
    };
    let nombres = nombres_de_campos(lista, idioma)?;
    if nombres.is_empty() {
        return Err(Error::Invalid(CAMPOS_VACIOS.texto_con(
            idioma,
            &[
                ("que", contrato.que),
                ("campos", &contrato.campos.join(", ")),
            ],
        )));
    }
    for nombre in &nombres {
        if !contrato.campos.contains(&nombre.as_str()) {
            return Err(Error::Invalid(CAMPO_DESCONOCIDO.texto_con(
                idioma,
                &[
                    ("campo", nombre.as_str()),
                    ("que", contrato.que),
                    ("campos", &contrato.campos.join(", ")),
                ],
            )));
        }
    }
    let campos = CamposPedidos {
        listas: contrato.listas,
        nombres,
    };
    Ok(Some(campos))
}

/// Projects every list of objects the answer of this `que` carries, so no list is left
/// with the fields the caller asked to drop.
pub(super) fn aplicar(resultado: &mut Value, campos: Option<CamposPedidos>) -> Proyeccion {
    let Some(campos) = campos else {
        return Proyeccion::Ninguna;
    };
    let mut aplicada = false;
    for &clave in campos.listas {
        aplicada |= proyectar_lista(resultado, clave, &campos.nombres);
    }
    if aplicada {
        Proyeccion::Aplicada
    } else {
        Proyeccion::SinLista
    }
}

/// Keeps in each item of the list `clave` only the keys `nombres` names; `false` when
/// the answer carries no such list of objects.
fn proyectar_lista(resultado: &mut Value, clave: &str, nombres: &[String]) -> bool {
    let Some(items) = resultado.get(clave).and_then(Value::as_array) else {
        return false;
    };
    let mut proyectados = Vec::with_capacity(items.len());
    for item in items {
        proyectados.push(proyectar_item(item, nombres));
    }
    let Some(slot) = resultado.get_mut(clave) else {
        return false;
    };
    *slot = Value::Array(proyectados);
    true
}

/// The contract of one `listar` `que`, `None` for a `que` this table does not know.
fn contrato_de(que: &str) -> Option<&'static Contrato> {
    CONTRATOS.iter().find(|contrato| contrato.que == que)
}

/// The names `campos` asks for; an element that is not a string is a wrong input.
fn nombres_de_campos(lista: &[Value], idioma: Idioma) -> Result<Vec<String>> {
    lista
        .iter()
        .map(|valor| match valor.as_str() {
            Some(texto) => Ok(texto.to_string()),
            None => Err(tipo_incorrecto(valor, "string", idioma)),
        })
        .collect()
}

/// The usual refusal of a value of the wrong type for `campos`.
fn tipo_incorrecto(valor: &Value, esperado: &str, idioma: Idioma) -> Error {
    Error::Invalid(TIPO_INCORRECTO.texto_con(
        idioma,
        &[
            ("campo", "campos"),
            ("esperado", esperado),
            ("recibido", nombre_tipo_recibido(valor)),
        ],
    ))
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
