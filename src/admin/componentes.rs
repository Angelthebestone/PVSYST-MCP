//! Component transfer: import external components into workspace database,
//! and export components out of workspace.

use crate::args::{missing, opt_str, req_str, tipo_de, workspace_of};
use crate::components;
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::history;
use crate::workspace::{resolver_ruta, validar_nombre, Alcance};
use serde_json::{json, Value};
use std::fs;

/// Import a component file into the workspace component database.
pub(crate) fn importar_componente(paths: &Paths, args: &Value) -> Result<Value> {
    if args.get("destino").is_some() {
        return Err(Error::Invalid(
            "'destino' no aplica a importar: el componente se copia a la base del workspace (usa 'nombre_nuevo')".into(),
        ));
    }
    let workspace = workspace_of(paths, args);
    let origen_str = req_str(args, &["origen"], "component file to import")?;
    let origen = resolver_ruta(&workspace, origen_str, Alcance::de_args(args))?;
    if !origen.exists() {
        return Err(Error::NotFound(format!(
            "File not found: {}",
            origen.display()
        )));
    }
    let kind =
        tipo_de(args, &origen.to_string_lossy()).ok_or_else(|| missing("tipo", "p.ej. \"PAN\""))?;
    let folder = components::folder_for(&kind)
        .ok_or_else(|| Error::Invalid(format!("Unknown component kind '{kind}'")))?;
    let dir = components::compos_dir(&workspace, folder);
    fs::create_dir_all(&dir)
        .map_err(|e| Error::io(format!("Cannot create {}", dir.display()), e))?;
    let name = match opt_str(args, &["nombre_nuevo"]) {
        Some(n) => n.to_string(),
        None => origen
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| Error::Invalid("Origen has no valid file name".into()))?
            .to_string(),
    };
    validar_nombre(&name)?;
    let target = dir.join(&name);
    components::sobrescritura::exigir_sobrescribir(&target, args)?;
    let bytes =
        fs::read(&origen).map_err(|e| Error::io(format!("Cannot read {}", origen.display()), e))?;
    let historial_id = history::reemplazar(&workspace, &target, &bytes, "importar_componente")?;
    Ok(json!({
        "imported": origen.to_string_lossy(),
        "to": target.to_string_lossy(),
        "kind": kind.to_uppercase(),
        "historial_id": historial_id,
    }))
}

/// Export a workspace component to an external file.
pub(crate) fn exportar_componente(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let componente = req_str(args, &["componente"], "component name")?;
    let kind = tipo_de(args, componente).unwrap_or_else(|| "PAN".to_string());
    let consulta = components::ConsultaComponente {
        kind: &kind,
        name: componente,
        alcance: Alcance::de_args(args),
    };
    let src = components::resolve_component(&workspace, &consulta)?;
    let destino_str = req_str(args, &["destino"], "full output path")?;
    let destino = resolver_ruta(&workspace, destino_str, Alcance::de_args(args))?;
    if let Some(parent) = destino.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("Cannot create {}", parent.display()), e))?;
    }
    fs::copy(&src, &destino).map_err(|e| {
        Error::io(
            format!("Cannot copy {} to {}", src.display(), destino.display()),
            e,
        )
    })?;
    Ok(json!({ "exported": src.to_string_lossy(), "to": destino.to_string_lossy() }))
}
