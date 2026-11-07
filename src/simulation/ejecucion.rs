use super::cache;
use super::preparacion::{preparar, Plan, Previo};
use crate::adapters::{cli, ledger};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::progreso::SeguimientoTarea;
use serde_json::{json, Value};
use std::io::Read;
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
    guardar_en_cache(plan, &mut resultado);
    resultado["costo"] = ledger::registrar(&plan.workspace, &out.stdout, plan.ejecuciones);
    let avisos = super::avisos_simulacion(plan.inicio);
    if !avisos.is_empty() {
        resultado["avisos_pvsyst"] = json!(avisos);
    }
    // Only a report this run wrote says anything about this run's result.
    let pr = produced(plan.antes.1, super::estado::mtime(plan.rpf.as_deref()))
        .then(|| {
            plan.rpf
                .as_deref()
                .and_then(super::coherencia::pr_del_informe)
        })
        .flatten();
    super::coherencia::aplicar(&mut resultado, &avisos, pr);
    if let Some(proyecto) = contexto.get("project").and_then(Value::as_str) {
        if let Some(aviso) = aviso_meteo_distinta(plan, proyecto) {
            agregar_advertencia(&mut resultado, &aviso);
        }
    }
    resultado
}

/// Stores the result under its cache key, reporting a failure to do so.
fn guardar_en_cache(plan: &Plan, resultado: &mut Value) {
    let Some(k) = &plan.clave else {
        return;
    };
    if resultado["success"] != json!(true) {
        return;
    }
    let guardado = cache::guardar(
        &plan.workspace,
        k,
        plan.ocf.as_deref().map(Path::new),
        plan.rpf.as_deref().map(Path::new),
        resultado,
    );
    if let Err(e) = guardado {
        resultado["cache_error"] = json!(e.to_string());
    }
}

/// Appends one warning to the answer's `advertencias` list.
fn agregar_advertencia(resultado: &mut Value, aviso: &str) {
    let mut advertencias = resultado
        .get("advertencias")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    advertencias.push(json!(aviso));
    resultado["advertencias"] = Value::Array(advertencias);
}

/// Banner bytes read to find the weather file: PVsyst writes it in the first lines,
/// and an hourly year is several MB.
const BYTES_CABECERA_CSV: u64 = 8 * 1024;

/// Weather file a result CSV was simulated with (`Weather data;<file>;`).
fn meteo_del_csv(csv: &Path) -> Option<String> {
    let mut bytes = Vec::new();
    std::fs::File::open(csv)
        .ok()?
        .take(BYTES_CABECERA_CSV)
        .read_to_end(&mut bytes)
        .ok()?;
    let texto = String::from_utf8_lossy(&bytes);
    for linea in texto.lines().take(40) {
        let mut campos = linea.split(';');
        if !campos
            .next()
            .unwrap_or("")
            .trim()
            .eq_ignore_ascii_case("Weather data")
        {
            continue;
        }
        return campos
            .next()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());
    }
    None
}

/// The run used a weather file other than the project's: its figures describe a
/// different location than the project the result is saved in.
fn aviso_meteo_distinta(plan: &Plan, proyecto: &str) -> Option<String> {
    let csv = plan.ocf.as_deref().map(Path::new)?;
    let del_resultado = meteo_del_csv(csv)?;
    let prj = std::fs::read(
        plan.workspace
            .join("Projects")
            .join(format!("{proyecto}.PRJ")),
    )
    .ok()?;
    let del_proyecto = crate::projects::lifecycle::sitio::meteo_del_proyecto(&prj)?;
    let nombre = |ruta: &str| {
        Path::new(ruta)
            .file_name()
            .map_or_else(|| ruta.to_string(), |n| n.to_string_lossy().into_owned())
            .to_ascii_lowercase()
    };
    (nombre(&del_resultado) != nombre(&del_proyecto)).then(|| {
        format!(
            "the result was simulated with '{del_resultado}' but the project points to '{del_proyecto}': the figures describe another weather file"
        )
    })
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
    // The lock comes first: a job launched and then not recorded would run (and be
    // charged) with no way to read its result, and a retry would charge it again.
    let mut lock = paths
        .estado
        .pendientes
        .lock()
        .map_err(|_| Error::Invalid("mutex pendientes envenenado".into()))?;
    let id = cli::run_en_segundo_plano(paths, cli_args, timeout)?;
    contexto["job_id"] = json!(id);
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
