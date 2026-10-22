use super::cache;
use super::preparacion::{preparar, Plan, Previo};
use crate::adapters::{cli, ledger};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::progreso::SeguimientoTarea;
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
    let avisos = super::avisos_simulacion(plan.inicio);
    if !avisos.is_empty() {
        resultado["avisos_pvsyst"] = json!(avisos);
    }
    resultado
}

/// Everything a synchronous simulation needs, grouped to keep call sites small.
pub(crate) struct SolicitudEjecucion<'a> {
    pub(crate) paths: &'a Paths,
    pub(crate) args: &'a Value,
    pub(crate) workspace: &'a Path,
    pub(crate) cli_args: &'a [String],
    pub(crate) contexto: &'a Value,
    pub(crate) seguimiento: &'a SeguimientoTarea<'a>,
}

pub(crate) fn ejecutar_cli(solicitud: &SolicitudEjecucion<'_>) -> Result<Value> {
    match preparar(solicitud.args, solicitud.workspace, solicitud.cli_args)? {
        Previo::Listo(v) => Ok(v),
        Previo::Ejecutar(plan) => {
            let timeout = cli::timeout_de(solicitud.args, cli::TIMEOUT_SIMULACION_S);
            let parametros = cli::ParametrosEjecucion {
                paths: solicitud.paths,
                cli_args: solicitud.cli_args,
                timeout,
                seguimiento: solicitud.seguimiento,
            };
            let out = cli::run_seguido(&parametros)?;
            Ok(finalizar(&plan, &out, solicitud.contexto))
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
