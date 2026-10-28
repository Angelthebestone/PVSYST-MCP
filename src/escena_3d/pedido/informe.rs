//! The answer to `escena`: what would be created.

use super::colision::colisiones;
use super::lectura::{leer_objetos, ObjetoPedido};
use crate::error::{Error, Result};
use serde_json::{json, Value};

const MAX_NOMBRE: usize = 64;
const AVISO_ANCLA: &str = "[INFERENCIA] colisiones calculadas con la posición como esquina inferior y sin rotación ni inclinación de la chimenea: el ancla de PVsyst no está demostrada.";
const AVISO_SIN_CREACION: &str = "La creación de la escena en el editor 3D de PVsyst no está implementada: esta acción solo valida el pedido. PVsyst en modo PRUEBA tampoco guarda y las escenas no se escriben a mano.";
const AVISO_RANGOS: &str = "Los rangos son límites del MCP; los de PVsyst no están medidos.";

fn nombre_valido(nombre: &str) -> Result<&str> {
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
        .map(|p| json!({ "clave": p.clave, "valor": valores.get(p.clave), "unidad": p.unidad }))
        .collect();
    let ranuras: Vec<Value> = objeto
        .forma
        .parametros
        .iter()
        .map(|p| json!({ "ranura": p.ranura, "clave": p.clave, "valor": valores.get(p.clave) }))
        .collect();
    json!({
        "indice": indice,
        "tipo": objeto.forma.tipo,
        "forma": { "codigo": objeto.forma.codigo, "nombre_pvsyst": objeto.forma.nombre_pvsyst },
        "posicion": {
            "x": objeto.posicion.x,
            "y": objeto.posicion.y,
            "z": objeto.posicion.z,
            "rotacion_azimut_deg": objeto.posicion.rotacion_azimut_deg,
        },
        "campos": campos,
        "ranuras_archivo": ranuras,
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
        "origen": "aproximacion_mcp",
        "avisos": [AVISO_SIN_CREACION, AVISO_RANGOS, AVISO_ANCLA],
    }))
}
