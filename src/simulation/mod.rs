//! Operation of PVsyst simulations.
//!
//! Every simulation runs through `PVsystCLI.exe run-simulation`, without
//! touching `PVsyst.exe` or reimplementing any solar physics.

pub(crate) mod avisos_pvsyst;
pub(crate) mod batch;
pub(crate) mod cache;
pub(crate) mod coherencia;
pub(crate) mod engineering;
pub(crate) mod preestimate;
pub(crate) mod report_pdf;
pub(crate) mod results;

mod ejecucion;
mod estado;
mod gui;
pub(crate) mod opciones;
pub(crate) mod preparacion;

pub(crate) use avisos_pvsyst::avisos_simulacion;
pub(crate) use estado::estado_simulacion;
pub(crate) use gui::ModoGui;

use crate::adapters::cli;
use crate::args::{flag, opt_str, req_str, workspace_of, DRY_RUN};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::progreso::SeguimientoTarea;
use serde_json::{json, Value};
use std::path::Path;

fn respuesta_dry_run(
    cli_args: &[String],
    prj_path: &Path,
    variant_file: &Path,
    paths: &Paths,
) -> Value {
    let variant_exists = variant_file.exists();
    let ejecuciones = preparacion::ejecuciones_previstas(cli_args);
    json!({
        "command": "run-simulation",
        "cli_args": cli_args,
        "dry_run": true,
        "project_file": prj_path.to_string_lossy(),
        "variant_file": variant_file.to_string_lossy(),
        "variant_file_exists": variant_exists,
        "workspace": paths.workspace.to_string_lossy(),
        "costo_estimado": crate::adapters::ledger::costo_estimado(paths, ejecuciones),
        "max_ejecuciones": ejecuciones,
    })
}

fn agregar_aviso_gui(resultado: &mut Value, e: &Error) {
    if let Some(obj) = resultado.as_object_mut() {
        obj.insert("gui_error".to_string(), json!(e.to_string()));
        let mut advertencias = obj
            .get("advertencias")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        advertencias.push(json!(format!("No se pudo abrir la interfaz gráfica: {e}")));
        obj.insert("advertencias".to_string(), Value::Array(advertencias));
    }
}

fn resolver_datos_proyecto(
    workspace: &Path,
    args: &Value,
) -> Result<(std::path::PathBuf, String, String, std::path::PathBuf)> {
    let proyecto = req_str(args, &["proyecto"], "")?;
    let variante = opt_str(args, &["variante"]).unwrap_or("VC0");
    crate::projects::lifecycle::variantes::exigir_variante_valida(variante)?;
    let variante_up = variante.to_ascii_uppercase();
    let prj_path = crate::projects::resolve_project_file(workspace, proyecto)?;
    let stem = prj_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| {
            Error::Invalid(format!("Ruta de proyecto inválida: {}", prj_path.display()))
        })?
        .to_string();
    let variant_file = prj_path.with_extension(&variante_up);
    Ok((prj_path, stem, variante_up, variant_file))
}

pub(crate) fn lanzar_simulacion(paths: &Paths, args: &Value) -> Result<Value> {
    let seguimiento = SeguimientoTarea::inerte();
    lanzar_simulacion_seguida(paths, args, &seguimiento)
}

pub(crate) fn lanzar_simulacion_seguida(
    paths: &Paths,
    args: &Value,
    seguimiento: &SeguimientoTarea<'_>,
) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let (prj_path, stem, variante_up, variant_file) = resolver_datos_proyecto(&workspace, args)?;
    opciones::validar(args)?;
    preparacion::validar_extra_options(args)?;
    opciones::exigir_variante(&variant_file)?;
    let args_con_salidas =
        preparacion::con_salidas_por_defecto(args, &workspace, &stem, &variante_up)?;
    let cli_args = preparacion::argumentos_cli(&args_con_salidas, &workspace, &stem, &variante_up);
    // The reported files are the same resolved values that built `-ocf` / `-rpf`.
    let output_csv = opt_str(&args_con_salidas, &["output_csv"]);
    let report_pdf = opt_str(&args_con_salidas, &["report_pdf"]);
    let modo_gui = ModoGui::de_bool(flag(args, &["abrir_gui"]));
    if flag(args, &[DRY_RUN]) {
        let pedido = paths.con_workspace(workspace);
        let mut respuesta = respuesta_dry_run(&cli_args, &prj_path, &variant_file, &pedido);
        respuesta["output_csv"] = json!(output_csv);
        respuesta["report_pdf"] = json!(report_pdf);
        anotar_sin_cache(&mut respuesta, args);
        return Ok(respuesta);
    }
    if !cli::cli_available(paths) {
        return gui::sin_cli(paths, &prj_path, &stem, modo_gui);
    }
    let contexto = json!({
        "mode": "cli",
        "project": stem,
        "variant": variante_up,
        "workspace": workspace.to_string_lossy(),
        "output_csv": output_csv,
        "report_pdf": report_pdf,
    });
    if flag(args, &["segundo_plano"]) {
        return ejecucion::lanzar_en_segundo_plano(paths, args, &workspace, &cli_args, contexto);
    }

    let solicitud = ejecucion::SolicitudEjecucion {
        paths,
        args,
        workspace: &workspace,
        cli_args: &cli_args,
        contexto: &contexto,
        seguimiento,
    };
    let mut resultado = ejecucion::ejecutar_cli(&solicitud)?;
    anotar_sin_cache(&mut resultado, args);
    if modo_gui.debe_abrir() {
        if let Err(e) = gui::spawn_gui(paths, &prj_path) {
            agregar_aviso_gui(&mut resultado, &e);
        }
    }
    Ok(resultado)
}

/// `sin_cache` bypasses the result cache; without this marker a cached reply and a
/// fresh run look the same, so the caller cannot tell the flag did anything.
fn anotar_sin_cache(resultado: &mut Value, args: &Value) {
    if flag(args, &["sin_cache"]) {
        resultado["sin_cache"] = json!(true);
    }
}

#[cfg(test)]
#[path = "../../tests/unit/simulation/mod.rs"]
mod tests;
