//! Log administration: export execution logs to zip.

use crate::adapters::cli;
use crate::args::{flag, opt_str, DRY_RUN};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::workspace::{resolver_ruta, Alcance};
use serde_json::{json, Value};
use std::fs;

/// `export-logs` — export execution logs as a zip.
pub(crate) fn exportar_logs(paths: &Paths, args: &Value) -> Result<Value> {
    let mut cli_args = vec!["export-logs".to_string()];
    let destino = opt_str(args, &["carpeta_destino"])
        .map(|t| resolver_ruta(&paths.workspace, t, Alcance::de_args(args)))
        .transpose()?;
    if let Some(dest) = &destino {
        cli_args.push(format!("-t:{}", dest.display()));
    }
    if flag(args, &[DRY_RUN]) {
        return Ok(json!({ "dry_run": true,
            "command_line": cli::display_command_line(&paths.cli_exe.to_string_lossy(), &cli_args) }));
    }
    // Measured in 8.1.8: `export-logs` refuses a target folder that does not exist.
    if let Some(dest) = &destino {
        fs::create_dir_all(dest).map_err(|e| {
            Error::io(
                format!("Cannot create the export-logs folder {}", dest.display()),
                e,
            )
        })?;
    }
    let out = cli::run(
        paths,
        &cli_args,
        cli::timeout_de(args, cli::TIMEOUT_DEFECTO_S),
    )?;
    Ok(cli::output_json(&out, "export-logs", &json!({})))
}
