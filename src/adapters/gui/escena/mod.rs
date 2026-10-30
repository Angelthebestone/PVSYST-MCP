//! `gui_actuar accion:escena` without `simular`: builds the scene in PVsyst's 3D
//! editor (`TPerspectiveForm`, already open on an empty scene), lets PVsyst save
//! it with «Exportar escena (.SHD)» and compares the file with the request.
//! Nothing is written by the MCP: PVsyst writes the scene.

mod exportar;
mod importar;
mod lista;
mod objeto;
mod pasos;

use crate::adapters::gui::acciones::exigir_escritorio;
use crate::adapters::gui::modal::exigir_habilitada;
use crate::args::{flag, opt_str, req_str};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::escena_3d::{leer_objetos, nombre_valido, verificar_archivo, ObjetoPedido};
use serde_json::{json, Value};

const EDITOR: &str = "TPerspectiveForm";
const ABRIR_EDITOR: &str = "variante.sombreados_cercanos.construccion_perspectiva";

pub(crate) fn crear(paths: &Paths, args: &Value) -> Result<Value> {
    exigir_escritorio()?;
    let nombre = nombre_valido(req_str(
        args,
        &["escena"],
        "nombre de la escena, p.ej. 'Casa_A'",
    )?)?;
    let pedidos = leer_objetos(args.get("objetos").unwrap_or(&Value::Null))?;
    exigir_editor_vacio(&pedidos)?;
    exportar::exigir_libre(paths, nombre)?;
    for objeto in &pedidos {
        objeto::crear(objeto)?;
    }
    lista::fijar(&pedidos, 0)?;
    let guardado = exportar::guardar(paths, nombre)?;
    informe(nombre, &guardado, &pedidos)
}

/// `gui_actuar accion:escena_leer`: the objects of the scene open in the editor. The table
/// of objects has no controls per row, so PVsyst exports the scene (`Archivo > Exportar
/// escena (.SHD)`) under `escena` and the file is read back; it stays in `Shadings`.
pub(crate) fn leer(paths: &Paths, args: &Value) -> Result<Value> {
    exigir_escritorio()?;
    let nombre = nombre_valido(req_str(
        args,
        &["escena"],
        "nombre con el que PVsyst exporta la escena, p.ej. 'Lectura_1'",
    )?)?;
    exigir_editor_abierto("leer la escena")?;
    exportada(paths, nombre)
}

/// Exports the open scene as `nombre` and reads the file PVsyst wrote.
fn exportada(paths: &Paths, nombre: &str) -> Result<Value> {
    exportar::exigir_libre(paths, nombre)?;
    let guardado = exportar::guardar(paths, nombre)?;
    let archivo = format!("{nombre}.SHD");
    let mut escena = crate::escena_3d::leer_escena(paths, &json!({ "archivo": archivo }))?;
    if let Some(mapa) = escena.as_object_mut() {
        mapa.insert("archivo".into(), json!(guardado.display().to_string()));
        mapa.insert("origen".into(), json!("pvsyst"));
        mapa.insert(
            "avisos".into(),
            json!([format!(
                "PVsyst guardó {archivo} en Shadings para leerla; bórralo si no lo necesitas."
            )]),
        );
    }
    Ok(escena)
}

/// `gui_actuar accion:escena_importar`: imports a mesh (3DS, DAE, PVC, PVC2, PVZ2) into the
/// scene open in the editor through PVsyst's own «Importar una escena 3D» and returns the
/// scene PVsyst exports afterwards (`escena`). With `simular:true` the import is dropped with
/// «Cancelar» once PVsyst has read the file. PVsyst converts the mesh: the MCP never does.
pub(crate) fn importar(paths: &Paths, args: &Value) -> Result<Value> {
    exigir_escritorio()?;
    let ruta = importar::malla_valida(req_str(
        args,
        &["archivo"],
        "ruta de la malla (3DS, DAE, PVC, PVC2, PVZ2), fuera de Program Files",
    )?)?;
    let nombre = nombre_valido(req_str(
        args,
        &["escena"],
        "nombre con el que PVsyst exporta la escena importada, p.ej. 'Malla_1'",
    )?)?;
    exigir_editor_abierto("importar una malla")?;
    exportar::exigir_libre(paths, nombre)?;
    let resultado = importar::abrir_resultado(&ruta)?;
    if let Some(unidades) = opt_str(args, &["unidades"]) {
        importar::elegir_unidades(&resultado, unidades)?;
    }
    let unidades = importar::unidades_elegidas(&resultado);
    if flag(args, &["simular"]) {
        importar::cerrar_resultado(&resultado, "Cancelar")?;
        return Ok(json!({
            "importado": false,
            "unidades": unidades,
            "archivo": ruta.display().to_string(),
        }));
    }
    importar::cerrar_resultado(&resultado, "OK")?;
    let mut escena = exportada(paths, nombre)?;
    if let Some(mapa) = escena.as_object_mut() {
        mapa.insert("importado".into(), json!(true));
        mapa.insert("unidades".into(), unidades);
    }
    Ok(escena)
}

fn exigir_editor_abierto(para: &str) -> Result<()> {
    let editor = pasos::ventana(EDITOR).map_err(|_| {
        Error::Gui(format!(
            "No hay editor 3D abierto: ábrelo con gui_actuar accion:funcion funcion:{ABRIR_EDITOR} y repite"
        ))
    })?;
    exigir_habilitada(editor, para)
}

/// The editor must be open and, since rows are found by position, empty.
fn exigir_editor_vacio(pedidos: &[ObjetoPedido]) -> Result<()> {
    exigir_editor_abierto("crear la escena")?;
    if pedidos.len() > lista::MAX_FILAS_VISIBLES {
        return Err(Error::Invalid(format!(
            "Máximo {} objetos por escena creada en el editor",
            lista::MAX_FILAS_VISIBLES
        )));
    }
    Ok(())
}

fn informe(nombre: &str, guardado: &std::path::Path, pedidos: &[ObjetoPedido]) -> Result<Value> {
    let comparacion = verificar_archivo(guardado, pedidos)?;
    // The rows that differ are what the agent must read: the matching ones are only counted.
    let (coinciden, difieren): (Vec<Value>, Vec<Value>) =
        comparacion.into_iter().partition(|c| c["estado"] == "ok");
    Ok(json!({
        "escena": nombre,
        "creado": true,
        "archivo": guardado.display().to_string(),
        "verificado": difieren.is_empty(),
        "campos_verificados": coinciden.len() + difieren.len(),
        "campos_que_difieren": difieren.len(),
        "comparacion": difieren,
        "origen": "pvsyst",
        "avisos": ["El editor 3D sigue abierto con la escena: ciérralo con Archivo > Cerrar escena para conservarla en la variante, o con Cancelar para descartarla."],
    }))
}
