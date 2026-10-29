//! The answer to `escena`: what would be created.

use super::colision::{colisiones, sin_caja};
use super::lectura::{leer_objetos, ObjetoPedido};
use crate::error::{Error, Result};
use serde_json::{json, Value};

const MAX_NOMBRE: usize = 64;
const AVISO_ANCLA: &str = "[INFERENCIA] colisiones solo entre paralelepípedos, con la posición como esquina inferior: el ancla de PVsyst no está demostrada y las demás formas no se comprueban (sin_caja).";
const AVISO_SIN_CREACION: &str = "Con simular:true esta acción solo valida el pedido; sin él crea los objetos en el editor 3D abierto y comprueba el archivo que PVsyst guardó. Las escenas no se escriben a mano.";
const AVISO_RANGOS: &str = "Los rangos son límites del MCP; los de PVsyst no están medidos.";

/// A scene name that is a plain file name: it becomes `Shadings\<nombre>.SHD`.
pub(crate) fn nombre_valido(nombre: &str) -> Result<&str> {
    let permitido = |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | ' ');
    if nombre.is_empty() || nombre.chars().count() > MAX_NOMBRE || !nombre.chars().all(permitido) {
        return Err(Error::Invalid(format!(
            "'escena' debe tener 1 a {MAX_NOMBRE} caracteres: letras, dígitos, espacio, '_' o '-'"
        )));
    }
    Ok(nombre)
}

fn objeto_json(indice: usize, objeto: &ObjetoPedido) -> Value {
    let valores = &objeto.valores;
    let campos: Vec<Value> = objeto
        .forma
        .parametros
        .iter()
        .enumerate()
        .map(|(i, p)| {
            json!({
                "clave": p.clave,
                "etiqueta_pvsyst": p.etiqueta,
                "valor": valores.get(p.clave),
                "unidad": p.unidad,
                "posicion_en_params": i,
            })
        })
        .collect();
    json!({
        "indice": indice,
        "tipo": objeto.forma.tipo,
        "forma": { "tipo_pvsyst": objeto.forma.tipo_pvsyst, "nombre_pvsyst": objeto.forma.nombre_pvsyst },
        "posicion": {
            "x": objeto.posicion.x,
            "y": objeto.posicion.y,
            "z": objeto.posicion.z,
            "rotacion_azimut_deg": objeto.posicion.rotacion_azimut_deg,
        },
        "campos": campos,
    })
}

/// Validates `objetos` and reports what the editor would be asked to create.
/// Nothing is written and PVsyst is not touched.
pub(crate) fn describir_escena(escena: &str, objetos: &Value) -> Result<Value> {
    let nombre = nombre_valido(escena)?;
    let pedidos = leer_objetos(objetos)?;
    let descritos: Vec<Value> = pedidos
        .iter()
        .enumerate()
        .map(|(i, o)| objeto_json(i, o))
        .collect();
    let todas = colisiones(&pedidos);
    // Every pair: the detail level trims the list and `detalle:completo` returns it whole.
    let choques: Vec<Value> = todas
        .iter()
        .map(|(a, b)| json!({ "a": a, "b": b }))
        .collect();
    Ok(json!({
        "escena": nombre,
        "simulado": true,
        "creado": false,
        "objetos": descritos,
        "colisiones": choques,
        "colisiones_total": todas.len(),
        "sin_caja": sin_caja(&pedidos),
        "origen": "aproximacion_mcp",
        "avisos": [AVISO_SIN_CREACION, AVISO_RANGOS, AVISO_ANCLA],
    }))
}
