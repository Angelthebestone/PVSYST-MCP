//! Log administration: export execution logs to zip.

use crate::adapters::cli;
use crate::args::{flag, opt_str, DRY_RUN};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::workspace::{resolver_ruta, Alcance};
use serde_json::{json, Value};
use std::fs;

/// `export-logs` — export execution logs as a zip inside the workspace by default.
pub(crate) fn exportar_logs(paths: &Paths, args: &Value) -> Result<Value> {
    // Without `-t` PVsystCLI writes the zip to `%USERPROFILE%`; the default keeps it
    // inside the workspace, and an explicit `carpeta_destino` goes through confinement.
    let (destino, crear) = match opt_str(args, &["carpeta_destino"]) {
        Some(t) => (
            resolver_ruta(&paths.workspace, t, Alcance::de_args(args))?,
            false,
        ),
        None => (paths.workspace.join("UserData").join("logs"), true),
    };
    let cli_args = vec![
        "export-logs".to_string(),
        format!("-t:{}", destino.display()),
    ];
    if flag(args, &[DRY_RUN]) {
        return Ok(json!({ "dry_run": true,
            "command_line": cli::display_command_line(&paths.cli_exe.to_string_lossy(), &cli_args) }));
    }
    if crear {
        fs::create_dir_all(&destino)
            .map_err(|e| Error::io(format!("Cannot create {}", destino.display()), e))?;
    }
    let out = cli::run(
        paths,
        &cli_args,
        cli::timeout_de(args, cli::TIMEOUT_DEFECTO_S),
    )?;
    Ok(cli::output_json(&out, "export-logs", &json!({})))
}
