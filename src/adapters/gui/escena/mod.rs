//! `gui_actuar accion:escena` without `simular`: builds the scene in PVsyst's 3D
//! editor (`TPerspectiveForm`, already open on an empty scene), lets PVsyst save
//! it with «Exportar escena (.SHD)» and compares the file with the request.
//! Nothing is written by the MCP: PVsyst writes the scene, and a previous export
//! under the same name is replaced instead of blocking the call.

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
    exigir_editor_vacio(paths, nombre, &pedidos)?;
    for objeto in &pedidos {
        objeto::crear(objeto)?;
    }
    lista::fijar(&pedidos, 0)?;
    let definitiva = exportar::ruta_de(paths, nombre);
    let existia = definitiva.exists();
    let temporal = exportar::guardar_temporal(paths, nombre)?;
    let mut informe = match informe(nombre, &temporal, &pedidos) {
        Ok(informe) => informe,
        Err(error) => {
            exportar::borrar(&temporal);
            return Err(error);
        }
    };
    let guardado = match exportar::conservar(&temporal, &definitiva) {
        Ok(ruta) => ruta,
        Err(error) => {
            exportar::borrar(&temporal);
            return Err(error);
        }
    };
    if let Some(mapa) = informe.as_object_mut() {
        mapa.insert("archivo".into(), json!(guardado.display().to_string()));
        mapa.insert("sobrescrito".into(), json!(existia));
    }
    Ok(informe)
}

/// `gui_actuar accion:escena_leer`: the objects of the scene open in the editor. The table
/// of objects has no controls per row, so PVsyst exports the scene (`Archivo > Exportar
/// escena (.SHD)`) under a temporary name, reads it back and deletes it: a read
/// leaves no file behind that could block the next call with the same name.
pub(crate) fn leer(paths: &Paths, args: &Value) -> Result<Value> {
    exigir_escritorio()?;
    let nombre = nombre_valido(req_str(
        args,
        &["escena"],
        "nombre con el que PVsyst exporta la escena, p.ej. 'Lectura_1'",
    )?)?;
    exigir_editor_abierto("reading the scene")?;
    let (mut escena, temporal) = exportar_y_leer(paths, nombre)?;
    exportar::borrar(&temporal);
    if let Some(mapa) = escena.as_object_mut() {
        mapa.insert("borrado".into(), json!(true));
        mapa.insert(
            "avisos".into(),
            json!([format!(
                "PVsyst exported the scene as {} to read it; the MCP deleted it, so no .SHD is left behind.",
                temporal.display()
            )]),
        );
    }
    Ok(escena)
}

/// Exports the open scene under a temporary name and reads the file PVsyst wrote.
/// The caller keeps it (a write) or deletes it (a read).
fn exportar_y_leer(paths: &Paths, nombre: &str) -> Result<(Value, std::path::PathBuf)> {
    let temporal = exportar::guardar_temporal(paths, nombre)?;
    let archivo = temporal
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let mut escena = match crate::escena_3d::leer_escena(paths, &json!({ "archivo": archivo })) {
        Ok(escena) => escena,
        Err(error) => {
            exportar::borrar(&temporal);
            return Err(error);
        }
    };
    if let Some(mapa) = escena.as_object_mut() {
        mapa.insert("archivo".into(), json!(temporal.display().to_string()));
        mapa.insert("origen".into(), json!("pvsyst"));
    }
    Ok((escena, temporal))
}

/// `gui_actuar accion:escena_importar`: imports a mesh (3DS, DAE, PVC, PVC2, PVZ2) into the
/// scene open in the editor through PVsyst's own «Importar una escena 3D» and returns the
/// scene PVsyst exports afterwards (`escena`). With `simular:true` the import is dropped with
/// «Cancelar» once PVsyst has read the file. PVsyst converts the mesh: the MCP never does.
pub(crate) fn importar(paths: &Paths, args: &Value) -> Result<Value> {
    exigir_escritorio()?;
    let ruta = importar::malla_valida(
        &paths.workspace,
        req_str(
            args,
            &["archivo"],
            "ruta de la malla (3DS, DAE, PVC, PVC2, PVZ2), fuera de Program Files",
        )?,
    )?;
    let nombre = nombre_valido(req_str(
        args,
        &["escena"],
        "nombre con el que PVsyst exporta la escena importada, p.ej. 'Malla_1'",
    )?)?;
    exigir_editor_abierto("importing a mesh")?;
    let resultado = importar::abrir_resultado(&ruta)?;
    let unidades = pasos::con_descarte(&resultado, || {
        if let Some(unidades) = opt_str(args, &["unidades"]) {
            importar::elegir_unidades(&resultado, unidades)?;
        }
        Ok(importar::unidades_elegidas(&resultado))
    })?;
    if flag(args, &["simular"]) {
        importar::cerrar_resultado(&resultado, "Cancelar")?;
        return Ok(json!({
            "simulado": true,
            "importado": false,
            "unidades": unidades,
            "archivo": ruta.display().to_string(),
        }));
    }
    importar::cerrar_resultado(&resultado, "OK")?;
    let definitiva = exportar::ruta_de(paths, nombre);
    let existia = definitiva.exists();
    let (mut escena, temporal) = exportar_y_leer(paths, nombre)?;
    let guardado = match exportar::conservar(&temporal, &definitiva) {
        Ok(ruta) => ruta,
        Err(error) => {
            exportar::borrar(&temporal);
            return Err(error);
        }
    };
    if let Some(mapa) = escena.as_object_mut() {
        mapa.insert("archivo".into(), json!(guardado.display().to_string()));
        mapa.insert("sobrescrito".into(), json!(existia));
        mapa.insert("importado".into(), json!(true));
        mapa.insert("unidades".into(), unidades);
    }
    Ok(escena)
}

fn exigir_editor_abierto(para: &str) -> Result<()> {
    let editor = pasos::ventana(EDITOR).map_err(|_| {
        Error::Gui(format!(
            "No 3D editor is open: open it with gui_actuar accion:funcion funcion:{ABRIR_EDITOR} and retry"
        ))
    })?;
    exigir_habilitada(editor, para)
}

/// The editor must be open and, since rows are found by position, empty: typing the positions
/// into rows 0..N would move objects that were already there. The object table has no controls to
/// count, so PVsyst exports the scene under a throwaway name and the file is read and deleted.
fn exigir_editor_vacio(paths: &Paths, nombre: &str, pedidos: &[ObjetoPedido]) -> Result<()> {
    exigir_editor_abierto("creating the scene")?;
    if pedidos.len() > lista::MAX_FILAS_VISIBLES {
        return Err(Error::Invalid(format!(
            "At most {} objects per scene created in the editor",
            lista::MAX_FILAS_VISIBLES
        )));
    }
    let previa = format!("previa_{nombre}");
    let (escena, temporal) = exportar_y_leer(paths, &previa)?;
    exportar::borrar(&temporal);
    let existentes = escena
        .get("total_objetos")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            Error::Format("The exported scene does not say how many objects it has: the editor may not be empty".into())
        })?;
    if existentes == 0 {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "The 3D editor already has {existentes} objects: creating a scene would write positions over them. Close it with Cancelar (without saving) and open it again, or use escena_importar to only add"
    )))
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
        "avisos": ["The 3D editor is still open with the scene: close it with Archivo > Cerrar escena to keep it in the variant, or with Cancelar to discard it."],
    }))
}
