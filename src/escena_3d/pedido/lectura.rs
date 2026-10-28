//! Reads and validates the objects an agent asks for. Every problem is
//! reported at once, with the keys the object accepts.

use super::formas::{forma_de, Forma, Parametro, Valores, FORMAS};
use super::problema::Problema;
use crate::error::{Error, Result};
use serde_json::{Map, Value};

/// Most objects one scene request may carry.
pub(crate) const MAX_OBJETOS: usize = 500;
/// Largest coordinate the MCP accepts, in meters.
const MAXIMO_COORDENADA_M: f64 = 100_000.0;
const MAXIMO_GIRO_DEG: f64 = 360.0;
const CLAVES_DE_POSICION: &[&str] = &["tipo", "x", "y", "z", "rotacion_azimut_deg"];

/// Where an object sits, in meters and degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Posicion {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
    pub(crate) rotacion_azimut_deg: f64,
}

/// One object the agent asked for, already validated.
pub(crate) struct ObjetoPedido {
    pub(crate) forma: &'static Forma,
    pub(crate) posicion: Posicion,
    pub(super) valores: Valores,
}

fn numero(
    objeto: &Map<String, Value>,
    clave: &'static str,
    defecto: Option<f64>,
) -> Result<f64, Problema> {
    match (objeto.get(clave), defecto) {
        (Some(v), _) => v
            .as_f64()
            .filter(|n| n.is_finite())
            .ok_or(Problema::NoEsNumero { clave }),
        (None, Some(d)) => Ok(d),
        (None, None) => Err(Problema::FaltaClave { clave }),
    }
}

fn dentro(clave: &'static str, valor: f64, (minimo, maximo): (f64, f64)) -> Result<f64, Problema> {
    if (minimo..=maximo).contains(&valor) {
        return Ok(valor);
    }
    Err(Problema::FueraDeRango {
        clave,
        valor,
        minimo,
        maximo,
    })
}

fn coordenada(
    objeto: &Map<String, Value>,
    clave: &'static str,
    defecto: Option<f64>,
) -> Result<f64, Problema> {
    numero(objeto, clave, defecto)
        .and_then(|v| dentro(clave, v, (-MAXIMO_COORDENADA_M, MAXIMO_COORDENADA_M)))
}

fn leer_posicion(objeto: &Map<String, Value>) -> Result<Posicion, Vec<Problema>> {
    let giro = numero(objeto, "rotacion_azimut_deg", Some(0.0)).and_then(|v| {
        dentro(
            "rotacion_azimut_deg",
            v,
            (-MAXIMO_GIRO_DEG, MAXIMO_GIRO_DEG),
        )
    });
    let partes = (
        coordenada(objeto, "x", None),
        coordenada(objeto, "y", None),
        coordenada(objeto, "z", Some(0.0)),
        giro,
    );
    match partes {
        (Ok(x), Ok(y), Ok(z), Ok(rotacion_azimut_deg)) => Ok(Posicion {
            x,
            y,
            z,
            rotacion_azimut_deg,
        }),
        (x, y, z, g) => Err([x, y, z, g].into_iter().filter_map(Result::err).collect()),
    }
}

fn leer_parametro(objeto: &Map<String, Value>, p: &Parametro) -> Result<f64, Problema> {
    numero(objeto, p.clave, None).and_then(|v| dentro(p.clave, v, (p.minimo, p.maximo)))
}

fn claves_ajenas(objeto: &Map<String, Value>, forma: &Forma) -> Vec<Problema> {
    let validas: Vec<&'static str> = CLAVES_DE_POSICION
        .iter()
        .copied()
        .chain(forma.parametros.iter().map(|p| p.clave))
        .collect();
    objeto
        .keys()
        .filter(|k| !validas.contains(&k.as_str()))
        .map(|k| Problema::ClaveAjena {
            clave: k.clone(),
            tipo: forma.tipo,
            validas: validas.clone(),
        })
        .collect()
}

/// A tree's trunk must be thinner than its crown.
fn coherencia(forma: &Forma, valores: &Valores) -> Option<Problema> {
    let (copa, tronco) = (valores.get("radio_copa")?, valores.get("radio_tronco")?);
    (forma.tipo == "arbol" && tronco >= copa).then_some(Problema::TroncoMayorQueCopa)
}

fn forma_del_objeto(objeto: &Map<String, Value>) -> Result<&'static Forma, Problema> {
    let tipo = objeto
        .get("tipo")
        .and_then(Value::as_str)
        .ok_or(Problema::FaltaTipo)?;
    forma_de(tipo).ok_or_else(|| Problema::TipoDesconocido {
        tipo: tipo.to_string(),
        validos: FORMAS.iter().map(|f| f.tipo).collect(),
    })
}

fn leer_objeto(valor: &Value) -> Result<ObjetoPedido, Vec<Problema>> {
    let objeto = valor
        .as_object()
        .ok_or_else(|| vec![Problema::NoEsObjeto])?;
    let forma = forma_del_objeto(objeto).map_err(|e| vec![e])?;
    let parametros: Vec<_> = forma
        .parametros
        .iter()
        .map(|p| (p.clave, leer_parametro(objeto, p)))
        .collect();
    let posicion = leer_posicion(objeto);
    let valores: Valores = parametros
        .iter()
        .filter_map(|(clave, r)| Some((*clave, *r.as_ref().ok()?)))
        .collect();
    let mut problemas = claves_ajenas(objeto, forma);
    problemas.extend(parametros.into_iter().filter_map(|(_, r)| r.err()));
    problemas.extend(posicion.as_ref().err().cloned().unwrap_or_default());
    problemas.extend(coherencia(forma, &valores));
    match posicion {
        Ok(posicion) if problemas.is_empty() => Ok(ObjetoPedido {
            forma,
            posicion,
            valores,
        }),
        _ => Err(problemas),
    }
}

/// The objects of `objetos`, or every problem found, each prefixed with its index.
pub(crate) fn leer_objetos(objetos: &Value) -> Result<Vec<ObjetoPedido>> {
    let lista = objetos
        .as_array()
        .filter(|l| !l.is_empty())
        .ok_or_else(|| {
            Error::Invalid("'objetos' debe ser una lista con al menos un objeto".into())
        })?;
    if lista.len() > MAX_OBJETOS {
        return Err(Error::Invalid(format!(
            "'objetos' tiene {} objetos y el máximo es {MAX_OBJETOS}",
            lista.len()
        )));
    }
    let leidos: Vec<_> = lista
        .iter()
        .enumerate()
        .map(|(i, v)| leer_objeto(v).map_err(|ps| (i, ps)))
        .collect();
    let problemas: Vec<String> = leidos
        .iter()
        .filter_map(|r| r.as_ref().err())
        .flat_map(|(i, ps)| ps.iter().map(move |p| format!("objetos[{i}]: {p}")))
        .collect();
    if !problemas.is_empty() {
        return Err(Error::Invalid(problemas.join("; ")));
    }
    Ok(leidos.into_iter().filter_map(Result::ok).collect())
}
