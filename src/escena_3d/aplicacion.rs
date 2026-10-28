//! Application orchestration for reading PVsyst 3D shading files (`.SHD`, `.BLD`, `.SHO`).

use super::codec::parse_escena_3d;
use super::dominio::{ObjetoEscena3d, TipoArchivoEscena};
use crate::args::{req_str, workspace_of};
use crate::config::{data_ro, Paths};
use crate::error::{Error, Result};
use crate::workspace::{
    leer_bytes_acotados, resolver_archivo_con_fallback, PeticionResolucion, MAX_BYTES_MAXIMO,
};
use serde_json::{json, Value};

/// Reads and decodes a `.SHD`, `.BLD`, or `.SHO` file, returning its structured 3D object tree.
pub(crate) fn leer_escena(paths: &Paths, args: &Value) -> Result<Value> {
    let nombre = req_str(args, &["archivo"], "p.ej. 'DEMO_Geneve_Grid.SHD'")?;
    let ws = workspace_of(paths, args);
    let peticion = PeticionResolucion {
        nombre,
        subcarpeta_ws: Some("Shadings"),
        rutas_dataro: vec![data_ro()
            .join("PVsyst8.1_Data")
            .join("Shadings")
            .join(nombre)],
    };
    let ruta = resolver_archivo_con_fallback(&ws, args, &peticion)?;
    let ext = ruta
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| Error::Invalid("Falta extensión requerida (.SHD, .BLD o .SHO)".into()))?;
    let tipo = TipoArchivoEscena::de_extension(ext)?;

    let bytes = leer_bytes_acotados(&ruta, MAX_BYTES_MAXIMO)?;
    let escena = parse_escena_3d(&bytes, tipo)?;

    let objetos_json: Vec<Value> = escena.objetos().iter().map(objeto_a_json).collect();

    Ok(json!({
        "archivo": ruta.file_name().map_or("escena.shd".into(), |n| n.to_string_lossy().to_string()),
        "tipo": escena.tipo().as_str().to_ascii_lowercase(),
        "tipo_archivo": escena.tipo().as_str(),
        "clase_raiz": escena.clase_raiz(),
        "descripcion": escena.descripcion(),
        "version": escena.version(),
        "calcul": escena.calcul(),
        "total_objetos": escena.objetos().len(),
        "objetos": objetos_json
    }))
}

fn objeto_a_json(obj: &ObjetoEscena3d) -> Value {
    let mut map = serde_json::Map::new();
    map.insert("clase".to_string(), json!(obj.clase()));
    map.insert("nombre".to_string(), json!(obj.nombre()));
    map.insert("version".to_string(), json!(obj.version()));
    map.insert("calcul".to_string(), json!(obj.calcul()));
    if let Some(conteo) = obj.cuenta_linea() {
        map.insert("cuenta_linea".to_string(), json!(conteo));
    }
    if let Some(pos) = obj.posicion() {
        map.insert(
            "posicion".to_string(),
            json!({
                "x": pos.x,
                "y": pos.y,
                "z": pos.z,
                "rotacion_azimut": pos.rotacion_azimut,
                "rotacion_inclinacion": pos.rotacion_inclinacion
            }),
        );
    }
    if let Some(geo) = obj.geometria() {
        map.insert(
            "geometria".to_string(),
            json!({
                "forma": geo.forma.nombre(),
                "dimensiones": geo.dimensiones
            }),
        );
    }
    Value::Object(map)
}

/// `gui_actuar accion:escena`: validates the request and reports what the editor would be asked
/// to create. Creating it in the editor is not measured yet, so this only describes (`creado:false`).
pub(crate) fn pedir_escena(args: &Value) -> Result<Value> {
    let nombre = req_str(args, &["escena"], "nombre de la escena, p.ej. 'Casa_A'")?;
    let objetos = args
        .get("objetos")
        .ok_or_else(|| Error::Invalid("Falta 'objetos' (lista de objetos de la escena)".into()))?;
    super::pedido::describir_escena(nombre, objetos)
}
