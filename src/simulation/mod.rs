//! Operation of PVsyst simulations.
//!
//! Every simulation runs through `PVsystCLI.exe run-simulation`, without
//! touching `PVsyst.exe` or reimplementing any solar physics.

pub(crate) mod batch;
pub(crate) mod cache;
pub(crate) mod engineering;
pub(crate) mod preestimate;
pub(crate) mod report_pdf;
pub(crate) mod results;

mod ejecucion;
mod estado;
mod gui;
mod preparacion;

pub(crate) use estado::estado_simulacion;
pub(crate) use gui::ModoGui;

use crate::adapters::cli;
use crate::args::{flag, opt_str, req_str, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::path::Path;

fn respuesta_dry_run(
    cli_args: &[String],
    prj_path: &Path,
    variant_file: &Path,
    workspace: &Path,
) -> Value {
    let variant_exists = variant_file.exists();
    let command_line = cli_args
        .iter()
        .map(|a| {
            if a.contains(' ') {
                format!("\"{a}\"")
            } else {
                a.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    json!({
        "command": "run-simulation",
        "cli_args": cli_args,
        "command_line": command_line,
        "dry_run": true,
        "project_file": prj_path.to_string_lossy(),
        "variant_file": variant_file.to_string_lossy(),
        "variant_file_exists": variant_exists,
        "workspace": workspace.to_string_lossy(),
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
    let proyecto = req_str(args, &["proyecto", "project"], "")?;
    let variante = opt_str(args, &["variante", "variant"]).unwrap_or("VC0");
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
    let workspace = workspace_of(paths, args);
    let (prj_path, stem, variante_up, variant_file) = resolver_datos_proyecto(&workspace, args)?;
    let args_con_salidas =
        preparacion::con_salidas_por_defecto(args, &workspace, &stem, &variante_up);
    let cli_args = preparacion::argumentos_cli(&args_con_salidas, &workspace, &stem, &variante_up);
    let open_gui = flag(args, &["abrir_gui"]);

    if flag(args, &["dry_run"]) {
        return Ok(respuesta_dry_run(
            &cli_args,
            &prj_path,
            &variant_file,
            &workspace,
        ));
    }
    if !cli::cli_available(paths) {
        return gui::sin_cli(paths, &prj_path, &stem, ModoGui::de_bool(open_gui));
    }
    let contexto = json!({
        "mode": "cli",
        "project": stem,
        "variant": variante_up,
        "workspace": workspace.to_string_lossy(),
        "output_csv": opt_str(args, &["output_csv", "csv"]),
        "report_pdf": opt_str(args, &["report_pdf", "pdf"]),
    });
    if flag(args, &["segundo_plano"]) {
        return ejecucion::lanzar_en_segundo_plano(paths, args, &workspace, &cli_args, contexto);
    }

    let mut resultado = ejecucion::ejecutar_cli(paths, args, &workspace, &cli_args, &contexto)?;
    if open_gui {
        if let Err(e) = gui::spawn_gui(paths, &prj_path) {
            agregar_aviso_gui(&mut resultado, &e);
        }
    }
    Ok(resultado)
}

#[cfg(test)]
mod tests {
    use super::preparacion::{con_salidas_por_defecto, push_options};
    use super::*;

    #[test]
    fn typed_options_are_emitted() {
        let mut v = Vec::new();
        let args = json!({
            "irf": "in.RVT",
            "cof": "opts.cof",
            "recalcular_sombreado": false,
            "paginas_reporte": ["a", "b"],
            "idioma_reporte": "es",
        });
        push_options(&args, &mut v);
        assert!(v.contains(&"-irf:in.RVT".to_string()));
        assert!(v.contains(&"-cof:opts.cof".to_string()));
        assert!(v.contains(&"-rst:0".to_string()));
        assert!(v.contains(&"-rp:a,b".to_string()));
        assert!(v.contains(&"-rl:es".to_string()));
    }

    #[test]
    fn timeout_default_for_simulation_is_900() {
        assert_eq!(
            cli::timeout_de(&json!({}), cli::TIMEOUT_SIMULACION_S),
            std::time::Duration::from_secs(900)
        );
    }

    #[test]
    fn default_outputs_point_into_results_and_respect_overrides() {
        let ws = Path::new("C:/ws");
        let v = con_salidas_por_defecto(&json!({}), ws, "Planta", "VC0");
        let csv = Path::new(v["output_csv"].as_str().unwrap());
        assert!(csv.ends_with("Results/Planta_VC0.csv"));
        assert!(v["report_pdf"]
            .as_str()
            .unwrap()
            .ends_with("Planta_VC0.pdf"));

        let explicito = con_salidas_por_defecto(
            &json!({"csv": "x.csv", "sin_pdf": true}),
            ws,
            "Planta",
            "VC0",
        );
        assert!(explicito.get("output_csv").is_none());
        assert!(explicito.get("report_pdf").is_none());
    }

    #[test]
    fn modo_gui_enum_matches_de_bool() {
        assert!(ModoGui::de_bool(true).debe_abrir());
        assert!(!ModoGui::de_bool(false).debe_abrir());
    }
}
