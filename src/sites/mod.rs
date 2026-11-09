//! Site and weather-data operations, backed by the documented PVsystCLI
//! (`create-site`, `convert-meteo`). No logic is reimplemented.

pub(crate) mod coordenadas;

pub(crate) use coordenadas::crear_sitio_coordenadas;
pub(crate) mod entrada;
pub(crate) mod nasa;
mod opciones_crear;
mod peticion;
pub(crate) mod pvgis;
pub(crate) mod readers;
pub(crate) mod reuse;
pub(crate) mod solar;

use crate::adapters::{cli, ledger};
use crate::args::{flag, missing, req_numero, workspace_of, DRY_RUN};
use crate::config::Paths;
use crate::error::Result;
use crate::format::value_text;
use entrada::{exigir_archivo, validar_coordenadas, validar_region, validar_timeshift};
use serde_json::{json, Value};

/// Textual form of a JSON scalar (string/number/bool); `None` for the rest.
fn as_text(v: &Value) -> Option<String> {
    (v.is_string() || v.is_number() || v.is_boolean()).then(|| value_text(v))
}

/// Trial executions a `create-site` or `convert-meteo` run spends when it succeeds.
const EJECUCIONES_POR_COMANDO: u32 = 1;

/// Trial-budget cost of one CLI run: it consumes an execution only when the command succeeded.
fn costo_de(ws: &std::path::Path, command: &str, out: &cli::CliOutput) -> Value {
    let usadas = u32::from(!out.timed_out && cli::classify(command, &out.stdout).0);
    ledger::registrar(ws, &out.stdout, usadas)
}
/// First non-empty scalar among `keys`, as text.
fn texto(args: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|k| args.get(*k).and_then(as_text).filter(|s| !s.is_empty()))
}

/// Specification of a PVsystCLI invocation.
struct PeticionEjecutar<F> {
    comando: &'static str,
    opciones: Vec<String>,
    extra: F,
}

/// Run a PVsystCLI command (or just show its command line on `dry_run`).
/// `-ll` is appended from `log_level`; `extra` builds the result fields from the trial cost.
fn ejecutar<F: FnOnce(Value) -> Value>(
    paths: &Paths,
    args: &Value,
    peticion: PeticionEjecutar<F>,
) -> Result<Value> {
    let mut cli_args = vec![peticion.comando.to_string()];
    cli_args.extend(peticion.opciones);
    cli::validar_log_level(args)?;
    if let Some(v) = args.get("log_level").and_then(serde_json::Value::as_i64) {
        cli_args.push(format!("-ll:{v}"));
    }
    if flag(args, &[DRY_RUN]) {
        let ws = workspace_of(paths, args);
        return Ok(json!({
            "dry_run": true,
            "command_line": cli::display_command_line(&paths.cli_exe.to_string_lossy(), &cli_args),
            "costo_estimado": ledger::costo_estimado(&ws, EJECUCIONES_POR_COMANDO),
        }));
    }
    let out = cli::run(
        paths,
        &cli_args,
        cli::timeout_de(args, cli::TIMEOUT_DEFECTO_S),
    )?;
    let ws = workspace_of(paths, args);
    let costo = costo_de(&ws, peticion.comando, &out);
    Ok(cli::output_json(
        &out,
        peticion.comando,
        &(peticion.extra)(costo),
    ))
}

pub(crate) fn crear_sitio(paths: &Paths, args: &Value) -> Result<Value> {
    let req = |key: &str| texto(args, &[key]).ok_or_else(|| missing(key, ""));
    let name = req("nombre_sitio")?;
    crate::workspace::validar_nombre(&name)?;
    validar_region(args)?;
    validar_coordenadas(
        req_numero(args, "latitud", "")?,
        req_numero(args, "longitud", "")?,
    )?;
    let mut opciones = vec![
        format!("-sn:{name}"),
        format!("-lat:{}", req("latitud")?),
        format!("-lon:{}", req("longitud")?),
    ];
    opciones.extend(opciones_crear::opciones_sitio(args));
    opciones.extend(opciones_crear::archivos_sitio(
        &workspace_of(paths, args),
        args,
    )?);
    ejecutar(
        paths,
        args,
        PeticionEjecutar {
            comando: "create-site",
            opciones,
            extra: |costo| json!({ "site_name": name, "expected_sit": args.get("output_sit_file"), "costo": costo }),
        },
    )
}

pub(crate) fn convertir_meteo(paths: &Paths, args: &Value) -> Result<Value> {
    validar_timeshift(args)?;
    let req = |key: &str| texto(args, &[key]).ok_or_else(|| missing(key, ""));
    let icf_raw = req("input_csv_file")?;
    let imf_raw = req("input_mef_file")?;
    let isf_raw = req("input_sit_file")?;
    let omf_raw = req("output_met_file")?;
    let ws = workspace_of(paths, args);
    let alcance = crate::workspace::Alcance::de_args(args);
    let icf = crate::workspace::resolver_ruta(&ws, &icf_raw, alcance)?;
    let imf = crate::workspace::resolver_ruta(&ws, &imf_raw, alcance)?;
    let isf = crate::workspace::resolver_ruta(&ws, &isf_raw, alcance)?;
    exigir_archivo("input_csv_file", &icf)?;
    exigir_archivo("input_mef_file", &imf)?;
    exigir_archivo("input_sit_file", &isf)?;
    let omf = crate::workspace::resolver_ruta(&ws, &omf_raw, alcance)?;
    let mut opciones = vec![
        format!("-icf:{}", icf.display()),
        format!("-imf:{}", imf.display()),
        format!("-isf:{}", isf.display()),
        format!("-omf:{}", omf.display()),
    ];
    if let Some(t) = texto(args, &["input_mef_timeshift"]) {
        opciones.push(format!("-imt:{t}"));
    }
    ejecutar(
        paths,
        args,
        PeticionEjecutar {
            comando: "convert-meteo",
            opciones,
            extra: |costo| json!({ "output_met_file": omf.display().to_string(), "costo": costo }),
        },
    )
}
/// Numeric argument, whether the caller sent a number or its decimal text.
fn num(args: &Value, key: &str) -> Option<f64> {
    args.get(key).and_then(|v| match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse::<f64>().ok(),
        _ => None,
    })
}
