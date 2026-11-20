//! `listar` field projection: with `campos` an item keeps only the keys the caller
//! named, so a caller that needs two fields of a project does not pay for the other
//! five. The names a `que` accepts come from its contract, never from the items an
//! answer happens to carry: a filtered, empty or warning-free answer accepts the
//! same names as a full one.

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

/// The fields every item of one `listar` `que` can carry — the optional ones
/// (`aviso`, written when a .PRJ cannot be parsed; `error`, when a .PAN cannot)
/// included, because this is the list a `campos` name is checked against.
struct Contrato {
    que: &'static str,
    campos: &'static [&'static str],
}

const CONTRATOS: &[Contrato] = &[
    Contrato {
        que: "proyectos",
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
        campos: &["file", "size_bytes"],
    },
    Contrato {
        que: "modulos",
        campos: &["modulo", "pnom_w", "vendor", "model", "error"],
    },
    Contrato {
        que: "archivos",
        campos: &["relative", "size_bytes", "modified_unix"],
    },
    Contrato {
        que: "objetos",
        campos: &["type", "role", "depth", "line_open", "line_close"],
    },
];

/// The field names a checked `campos` asks for: holding one proves each name is a
/// field of its `que`, so the projection itself cannot fail.
#[must_use]
pub(super) struct CamposPedidos {
    que: &'static str,
    nombres: Vec<String>,
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
        que: contrato.que,
        nombres,
    };
    Ok(Some(campos))
}

/// Keeps in each item of the answer's list only the keys `campos` names. `false`
/// when nothing was projected: no `campos`, or an answer with no list of objects,
/// and only the second case leaves a given `campos` to be reported as ignored.
pub(super) fn aplicar(resultado: &mut Value, campos: Option<CamposPedidos>) -> bool {
    let Some(campos) = campos else {
        return false;
    };
    let Some(clave) = clave_de_lista(campos.que) else {
        return false;
    };
    let Some(items) = resultado.get(clave).and_then(Value::as_array) else {
        return false;
    };
    let mut proyectados = Vec::with_capacity(items.len());
    for item in items {
        proyectados.push(proyectar_item(item, &campos.nombres));
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
