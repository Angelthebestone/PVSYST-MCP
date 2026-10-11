//! Licensing tools: inspect and sync status, generate activation commands.

use crate::adapters::cli;
use crate::args::flag;
use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::{json, Value};

/// Licensing. `info` and `sync` run the CLI; `activar` and `desactivar` never
/// execute: they return the command to run with the user's own key or id.
pub(crate) fn licencia(paths: &Paths, args: &Value) -> Result<Value> {
    let accion = args
        .get("accion")
        .and_then(Value::as_str)
        .unwrap_or("info")
        .to_lowercase();
    let cli_args: Vec<String> = match accion.as_str() {
        "info" | "estado" => vec!["lic-info".into()],
        "sync" | "sincronizar" => vec!["lic-sync".into()],
        "activar" | "activate" => {
            return Ok(json!({
                "accion": "activar",
                "comando": "PVsystCLI.exe lic-activate -k:<TU_CLAVE>",
                "instrucciones": "Sustituye <TU_CLAVE> por tu clave de licencia y ejecuta el comando fuera de este servidor."
            }));
        }
        "desactivar" | "deactivate" => {
            return Ok(json!({
                "accion": "desactivar",
                "comando": "PVsystCLI.exe lic-deactivate -c:<TU_CUSTOMER_ID>",
                "instrucciones": "Sustituye <TU_CUSTOMER_ID> por tu customer id y ejecuta el comando fuera de este servidor."
            }));
        }
        other => {
            return Err(Error::Invalid(format!(
                "Accion de licencia desconocida '{other}' (info|sync|activar|desactivar)"
            )))
        }
    };
    if flag(args, &["dry_run"]) {
        return Ok(json!({ "dry_run": true, "accion": accion,
            "command_line": cli::display_command_line(&paths.cli_exe.to_string_lossy(), &cli_args) }));
    }
    let out = cli::run(
        paths,
        &cli_args,
        cli::timeout_de(args, cli::TIMEOUT_DEFECTO_S),
    )?;
    Ok(cli::output_json(
        &out,
        &cli_args[0],
        &json!({ "accion": accion }),
    ))
}
