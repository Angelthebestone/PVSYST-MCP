use super::cache;
use super::preparacion::{preparar, Plan, Previo};
use crate::adapters::{cli, ledger};
use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::path::Path;

pub(crate) fn finalizar(plan: &Plan, out: &cli::CliOutput, contexto: &Value) -> Value {
    let mut resultado = cli::output_json(out, "run-simulation", contexto);

    let produced = |before: Option<u64>, after: Option<u64>| {
        after.is_some_and(|a| before.is_none_or(|b| a > b))
    };
    let file_ok = !out.timed_out
        && (produced(plan.antes.0, super::estado::mtime(plan.ocf.as_deref()))
            || produced(plan.antes.1, super::estado::mtime(plan.rpf.as_deref())));
    if file_ok && resultado["success"] != json!(true) {
        resultado["success"] = json!(true);
        resultado["confirmed_by_output_file"] = json!(true);
    }

    if let Some(razon) = &plan.advertencia_cache {
        resultado["cache_advertencia"] = json!(razon);
    }

    if let Some(k) = &plan.clave {
        if resultado["success"] == json!(true) {
            let guardado = cache::guardar(
                &plan.workspace,
                k,
                plan.ocf.as_deref().map(Path::new),
                plan.rpf.as_deref().map(Path::new),
                &resultado,
            );
            if let Err(e) = guardado {
                resultado["cache_error"] = json!(e.to_string());
            }
        }
    }
    resultado["costo"] = ledger::registrar(&plan.workspace, &out.stdout, plan.ejecuciones);
    resultado
}

pub(crate) fn ejecutar_cli(
    paths: &Paths,
    args: &Value,
    workspace: &Path,
    cli_args: &[String],
    contexto: &Value,
) -> Result<Value> {
    match preparar(args, workspace, cli_args)? {
        Previo::Listo(v) => Ok(v),
        Previo::Ejecutar(plan) => {
            let timeout = cli::timeout_de(args, cli::TIMEOUT_SIMULACION_S);
            let out = cli::run(paths, cli_args, timeout)?;
            Ok(finalizar(&plan, &out, contexto))
        }
    }
}

pub(crate) fn lanzar_en_segundo_plano(
    paths: &Paths,
    args: &Value,
    workspace: &Path,
    cli_args: &[String],
    mut contexto: Value,
) -> Result<Value> {
    let plan = match preparar(args, workspace, cli_args)? {
        Previo::Listo(v) => return Ok(v),
        Previo::Ejecutar(plan) => plan,
    };
    let timeout = cli::timeout_de(args, cli::TIMEOUT_SIMULACION_S);
    let id = cli::run_en_segundo_plano(paths, cli_args, timeout)?;
    contexto["job_id"] = json!(id);
    let mut lock = paths
        .estado
        .pendientes
        .lock()
        .map_err(|_| Error::Invalid("mutex pendientes envenenado".into()))?;
    lock.insert(
        id.clone(),
        super::estado::Pendiente {
            plan,
            comando: "run-simulation".to_string(),
            contexto,
            resultado: None,
        },
    );
    Ok(json!({
        "job_id": id,
        "mode": "background",
        "message": "Simulación lanzada en segundo plano; consulte con simular(accion: \"estado\", trabajo: id)",
    }))
}
