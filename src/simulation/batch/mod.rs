//! Advanced-parameter override (`Param_Modif.dat`) and batch-simulation file
//! generation. These produce the input files the documented PVsystCLI consumes
//! via `-ipf` (param file) and `-bpf`/`-brf` (batch mode).

use crate::args::{missing, opt_str, req_str, workspace_of};
use crate::calendario::civil_desde_dias;
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::value_text;
use crate::history;
use serde_json::{json, Value};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

mod plantilla;

use plantilla::batch_csv;

fn now_stamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let fecha = civil_desde_dias(days);
    format!(
        "{:02}/{:02}/{:02} {:02}:{:02}",
        fecha.dia(),
        fecha.mes(),
        fecha.anio() % 100,
        rem / 3600,
        (rem % 3600) / 60
    )
}

/// Build a `Param_Modif.dat` file (advanced/hidden parameter overrides) and
/// return its path, ready to be passed to `lanzar_simulacion.input_param_file`.
fn construir_dat(
    params: &serde_json::Map<String, Value>,
    iniciales: Option<&serde_json::Map<String, Value>>,
    unidades: Option<&serde_json::Map<String, Value>>,
    descripciones: Option<&serde_json::Map<String, Value>>,
) -> String {
    let mut text = String::from("Hidden parameters: User's modifications;\n");
    for (key, new) in params {
        let new_s = value_text(new);
        let init = iniciales
            .and_then(|m| m.get(key))
            .map_or_else(|| "0.0000".to_string(), value_text);
        let unit = unidades
            .and_then(|m| m.get(key))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let desc = descripciones
            .and_then(|m| m.get(key))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let _ = writeln!(text, "{key};{init};{new_s};{unit};{desc}");
    }
    text
}

pub(crate) fn modificar_parametros(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let params = args
        .get("parametros")
        .and_then(|v| v.as_object())
        .ok_or_else(|| missing("parametros", "object: name -> new value"))?;
    let iniciales = args.get("iniciales").and_then(|v| v.as_object());
    let unidades = args.get("unidades").and_then(|v| v.as_object());
    let descripciones = args.get("descripciones").and_then(|v| v.as_object());
    let text = construir_dat(params, iniciales, unidades, descripciones);
    let destino = match opt_str(args, &["destino"]) {
        Some(p) => crate::workspace::resolver_ruta(
            &workspace,
            p,
            crate::workspace::Alcance::de_args(args),
        )?,
        None => workspace.join("Other").join("Param_Modif.dat"),
    };
    if let Some(parent) = destino.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("Cannot create {}", parent.display()), e))?;
    }
    let historial_id = history::reemplazar(
        &workspace,
        &destino,
        text.as_bytes(),
        "modificar_parametros",
    )?;
    Ok(json!({
        "param_file": destino.to_string_lossy(),
        "parameters": params.len(),
        "historial_id": historial_id,
        "hint": "Pass this path as 'input_param_file' (-ipf) to lanzar_simulacion.",
    }))
}

/// Generate a batch-mode parameters CSV compatible with `PVsystCLI -bpf`.
///
/// `filas` is an array of objects; recognized keys (with aliases):
/// `tilt`, `azim`, `pan`, `ond`, `n_mod_serie`, `n_strings`, `n_inverter`,
/// `thermal_uc`, `thermal_uw`, `rwiring`, `soiling`, `mod_quality`,
/// `mismatch`, `lid`, `iam`, `csv` (output CSV name), `comment`.
pub(crate) fn crear_batch(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let proyecto = req_str(args, &["proyecto"], "")?;
    let variante = opt_str(args, &["variante"]).unwrap_or("VC0");
    let prj = crate::projects::resolve_project_file(&workspace, proyecto)?;
    if !prj.with_extension(variante).exists() {
        return Err(Error::NotFound(format!(
            "Variant {variante} of '{proyecto}' does not exist; pass an existing 'variante' (see listar que:proyectos)"
        )));
    }
    let filas = args
        .get("filas")
        .and_then(|v| v.as_array())
        .ok_or_else(|| missing("filas", "array of parameter sets"))?;
    if filas.is_empty() {
        return Err(Error::Invalid("'filas' must not be empty".into()));
    }

    let out = batch_csv(proyecto, variante, filas, &now_stamp())?;

    let destino = match opt_str(args, &["destino"]) {
        Some(p) => crate::workspace::resolver_ruta(
            &workspace,
            p,
            crate::workspace::Alcance::de_args(args),
        )?,
        None => workspace.join("UserBatch").join("MCP_BatchParams.csv"),
    };
    if let Some(parent) = destino.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("Cannot create {}", parent.display()), e))?;
    }
    let batch_dir = workspace.join("UserBatch");
    fs::create_dir_all(&batch_dir)
        .map_err(|e| Error::io(format!("Cannot create {}", batch_dir.display()), e))?;
    let historial_id = history::reemplazar(&workspace, &destino, out.as_bytes(), "crear_batch")?;
    let (rvt_path, historial_rvt_id) = escribir_rvt(&workspace, &batch_dir, args)?;

    Ok(json!({
        "batch_params_file": destino.to_string_lossy(),
        "batch_rvt_file": rvt_path,
        "runs": filas.len(),
        "project": proyecto,
        "variant": variante,
        "historial_id": historial_id,
        "historial_rvt_id": historial_rvt_id,
        "hint": "Pass batch_params_file as 'batch_params_file' (-bpf) to lanzar_simulacion; add batch_rvt_file if generated.",
    }))
}

/// Optional RVT (batch result variable list) from `variables`; returns `(path, history id)`.
fn escribir_rvt(
    workspace: &Path,
    batch_dir: &Path,
    args: &Value,
) -> Result<(Option<String>, Option<String>)> {
    let list: Vec<&str> = args
        .get("variables")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    if list.is_empty() {
        return Ok((None, None));
    }
    let rvs = batch_dir.join("MCP_BatchVariables.rvt");
    let content = format!("  ResultVars={}\n", list.join(";"));
    let id = history::reemplazar(workspace, &rvs, content.as_bytes(), "crear_batch")?;
    Ok((Some(rvs.to_string_lossy().to_string()), id))
}
