//! Compares what the agent asked for with what PVsyst saved, field by field:
//! `{objeto, clave, pedido, leido, estado}`. The MCP never writes the scene, so
//! this reading is the proof that the editor created what was requested.

use super::lectura::ObjetoPedido;
use crate::escena_3d::dominio::{Escena3d, ObjetoEscena3d};
use serde_json::{json, Value};

/// `Params=` keeps 3 decimals and `DOrigine=` 6: a smaller gap is rounding.
const TOLERANCIA: f64 = 5e-4;

/// `fila` is (index of the object, key of the value).
fn entrada(fila: (usize, &str), pedido: &Value, leido: Option<&Value>) -> Value {
    let (objeto, clave) = fila;
    let estado = match (leido, pedido) {
        (None, _) => "falta",
        (Some(l), p) if l == p => "ok",
        _ => "difiere",
    };
    json!({ "objeto": objeto, "clave": clave, "pedido": pedido, "leido": leido, "estado": estado })
}

fn numero(fila: (usize, &str), pedido: f64, leido: Option<f64>) -> Value {
    let (objeto, clave) = fila;
    let estado = match leido {
        None => "falta",
        Some(l) if (l - pedido).abs() <= TOLERANCIA => "ok",
        Some(_) => "difiere",
    };
    json!({ "objeto": objeto, "clave": clave, "pedido": pedido, "leido": leido, "estado": estado })
}

fn parametros(i: usize, pedido: &ObjetoPedido, guardado: &ObjetoEscena3d) -> Vec<Value> {
    let dimensiones = guardado.geometria().map(|g| &g.dimensiones);
    pedido
        .forma
        .parametros
        .iter()
        .enumerate()
        .map(|(n, p)| {
            let leido = dimensiones.and_then(|d| d.get(n)).copied();
            numero(
                (i, p.clave),
                pedido.valores.get(p.clave).copied().unwrap_or(p.defecto),
                leido,
            )
        })
        .collect()
}

fn posicion(i: usize, pedido: &ObjetoPedido, guardado: &ObjetoEscena3d) -> Vec<Value> {
    let (p, g) = (pedido.posicion, guardado.posicion());
    vec![
        numero((i, "x"), p.x, g.map(|g| g.x)),
        numero((i, "y"), p.y, g.map(|g| g.y)),
        numero((i, "z"), p.z, g.map(|g| g.z)),
        numero(
            (i, "rotacion_azimut_deg"),
            p.rotacion_azimut_deg,
            g.map(|g| g.rotacion_azimut),
        ),
    ]
}

fn objeto(i: usize, pedido: &ObjetoPedido, guardado: Option<&ObjetoEscena3d>) -> Vec<Value> {
    let Some(guardado) = guardado else {
        return vec![entrada((i, "objeto"), &json!(pedido.forma.tipo), None)];
    };
    let tipo = guardado
        .geometria()
        .and_then(|g| g.tipo_pvsyst.clone())
        .map(Value::from);
    let mut filas = vec![entrada(
        (i, "tipo_pvsyst"),
        &json!(pedido.forma.tipo_pvsyst),
        tipo.as_ref(),
    )];
    if let Some(nombre) = &pedido.nombre {
        filas.push(entrada(
            (i, "nombre"),
            &json!(nombre),
            Some(&json!(guardado.nombre())),
        ));
    }
    filas.extend(parametros(i, pedido, guardado));
    filas.extend(posicion(i, pedido, guardado));
    filas
}

/// One row per requested value. The objects asked for are the last ones of
/// the file: the scene may already hold others.
pub(crate) fn comparar(escena: &Escena3d, pedidos: &[ObjetoPedido]) -> Vec<Value> {
    let guardados = escena.objetos();
    let primero = guardados.len().saturating_sub(pedidos.len());
    pedidos
        .iter()
        .enumerate()
        .flat_map(|(i, p)| objeto(i, p, guardados.get(primero + i)))
        .collect()
}
