//! Log administration: export execution logs to zip.

use crate::adapters::cli;
use crate::args::{flag, opt_str, DRY_RUN};
use crate::config::Paths;
use crate::error::Result;
use crate::workspace::{resolver_ruta, Alcance};
use serde_json::{json, Value};

/// `export-logs` — export execution logs as a zip.
pub(crate) fn exportar_logs(paths: &Paths, args: &Value) -> Result<Value> {
    let mut cli_args = vec!["export-logs".to_string()];
    if let Some(t) = opt_str(args, &["carpeta_destino"]) {
        let dest = resolver_ruta(&paths.workspace, t, Alcance::de_args(args))?;
        cli_args.push(format!("-t:{}", dest.display()));
    }
    if flag(args, &[DRY_RUN]) {
        return Ok(json!({ "dry_run": true,
            "command_line": cli::display_command_line(&paths.cli_exe.to_string_lossy(), &cli_args) }));
    }
    let out = cli::run(
        paths,
        &cli_args,
        cli::timeout_de(args, cli::TIMEOUT_DEFECTO_S),
    )?;
    Ok(cli::output_json(&out, "export-logs", &json!({})))
}
