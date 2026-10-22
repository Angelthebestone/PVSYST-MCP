//! Site and weather-data operations, backed by the documented PVsystCLI
//! (`create-site`, `convert-meteo`). No logic is reimplemented.

pub(crate) mod nasa;
pub(crate) mod pvgis;
pub(crate) mod readers;
pub(crate) mod reuse;
pub(crate) mod solar;

use crate::adapters::{cli, ledger};
use crate::args::{flag, missing, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::value_text;
use serde_json::{json, Value};

/// Textual form of a JSON scalar (string/number/bool); `None` for the rest.
fn as_text(v: &Value) -> Option<String> {
    (v.is_string() || v.is_number() || v.is_boolean()).then(|| value_text(v))
}

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
    if let Some(v) = args.get("log_level").and_then(serde_json::Value::as_u64) {
        cli_args.push(format!("-ll:{}", v.min(3)));
    }
    if flag(args, &["dry_run"]) {
        return Ok(json!({
            "dry_run": true,
            "command_line": cli::display_command_line(&paths.cli_exe.to_string_lossy(), &cli_args),
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

/// Optional `create-site` flags present in `args`.
fn opciones_sitio(args: &Value) -> Vec<String> {
    const OPCIONALES: [(&[&str], &str); 8] = [
        (&["altitud", "altitude"], "-alt"),
        (&["timezone", "zona_horaria"], "-tz"),
        (&["country_code", "pais"], "-cc"),
        (&["region"], "-r"),
        (&["albedo"], "-alb"),
        (&["source", "fuente"], "-s"),
        (&["output_sit_file", "sit_salida"], "-osf"),
        (&["cmd_options_file", "cof"], "-cof"),
    ];
    OPCIONALES
        .iter()
        .filter_map(|(keys, flag)| {
            let s = keys.iter().find_map(|k| {
                args.get(*k)
                    .map(value_text)
                    .filter(|s| !s.is_empty() && s != "null")
            })?;
            Some(format!("{flag}:{s}"))
        })
        .collect()
}

pub(crate) fn crear_sitio(paths: &Paths, args: &Value) -> Result<Value> {
    let req = |keys: &[&str]| texto(args, keys).ok_or_else(|| missing(keys[0], ""));
    let name = req(&["nombre_sitio", "site_name"])?;
    crate::workspace::validar_nombre(&name)?;
    let mut opciones = vec![
        format!("-sn:{name}"),
        format!("-lat:{}", req(&["latitud", "latitude"])?),
        format!("-lon:{}", req(&["longitud", "longitude"])?),
        format!("-wdf:{}", req(&["weather_data_file", "archivo_meteo"])?),
    ];
    opciones.extend(opciones_sitio(args));
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
    let req = |keys: &[&str]| texto(args, keys).ok_or_else(|| missing(keys[0], ""));
    let icf_raw = req(&["input_csv_file", "csv"])?;
    let imf_raw = req(&["input_mef_file", "mef"])?;
    let isf_raw = req(&["input_sit_file", "sit"])?;
    let omf_raw = req(&["output_met_file", "met"])?;
    let ws = workspace_of(paths, args);
    let alcance = crate::workspace::Alcance::de_args(args);
    let icf = crate::workspace::resolver_ruta(&ws, &icf_raw, alcance)?;
    let imf = crate::workspace::resolver_ruta(&ws, &imf_raw, alcance)?;
    let isf = crate::workspace::resolver_ruta(&ws, &isf_raw, alcance)?;
    let omf = crate::workspace::resolver_ruta(&ws, &omf_raw, alcance)?;
    let mut opciones = vec![
        format!("-icf:{}", icf.display()),
        format!("-imf:{}", imf.display()),
        format!("-isf:{}", isf.display()),
        format!("-omf:{}", omf.display()),
    ];
    if let Some(t) = texto(args, &["input_mef_timeshift", "timeshift"]) {
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
fn num(args: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter().find_map(|k| {
        args.get(*k).and_then(|v| match v {
            Value::Number(n) => n.as_f64(),
            Value::String(s) => s.trim().parse::<f64>().ok(),
            _ => None,
        })
    })
}

fn nombre_por_coordenadas(lat: f64, lon: f64) -> String {
    format!("Site_{lat:.3}_{lon:.3}")
        .replace('.', "p")
        .replace('-', "m")
}

/// Write the derived monthly climatology as the CSV `create-site` consumes.
fn escribir_csv_mensual(
    ws: &std::path::Path,
    name: &str,
    clim: &nasa::Climatology,
) -> Result<std::path::PathBuf> {
    let csv_path = ws.join("UserData").join(format!("{name}_Monthly.csv"));
    if let Some(p) = csv_path.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    std::fs::write(&csv_path, clim.to_create_site_csv())
        .map_err(|e| Error::io(format!("Cannot write {}", csv_path.display()), e))?;
    Ok(csv_path)
}

/// `output_sit_file`, else `<workspace>/Sites/<name>.SIT`.
fn ruta_sit(ws: &std::path::Path, args: &Value, name: &str) -> Result<String> {
    if let Some(v) = args.get("output_sit_file").and_then(|v| v.as_str()) {
        let alcance = crate::workspace::Alcance::de_args(args);
        let p = crate::workspace::resolver_ruta(ws, v, alcance)?;
        return Ok(p.display().to_string());
    }
    Ok(ws
        .join("Sites")
        .join(format!("{name}.SIT"))
        .display()
        .to_string())
}

/// An already-created site matching the coordinates, as a zero-cost response.
/// An explicit output file or `forzar` always creates a new site instead.
fn sitio_reutilizado(ws: &std::path::Path, args: &Value, (lat, lon): (f64, f64)) -> Option<Value> {
    if flag(args, &["forzar"]) || args.get("output_sit_file").is_some() {
        return None;
    }
    let mut existente = reuse::buscar_existente(ws, lat, lon, num(args, &["altitud", "altitude"]))?;
    existente["create_site"] = json!({ "success": true, "reutilizado": true });
    existente["costo"] = json!({ "ejecuciones": 0 });
    Some(existente)
}

/// Create a site from **coordinates alone**: monthly weather is derived from the
/// NASA SSE 1°×1° grids shipped with PVsyst (`DataRO\*_World.*`), so no weather
/// file is needed. This is the easy path for "design X kWp at these coordinates".
pub(crate) fn crear_sitio_coordenadas(paths: &Paths, args: &Value) -> Result<Value> {
    let lat = num(args, &["latitud", "latitude"]).ok_or_else(|| missing("latitud", ""))?;
    let lon = num(args, &["longitud", "longitude"]).ok_or_else(|| missing("longitud", ""))?;
    let name = args
        .get("nombre_sitio")
        .and_then(|v| v.as_str())
        .map_or_else(|| nombre_por_coordenadas(lat, lon), String::from);
    crate::workspace::validar_nombre(&name)?;

    let clim = nasa::climatology(lat, lon)?;
    let ws = crate::args::workspace_of(paths, args);
    if let Some(existente) = sitio_reutilizado(&ws, args, (lat, lon)) {
        return Ok(resumen_clima(existente, &clim));
    }
    let csv_path = escribir_csv_mensual(&ws, &name, &clim)?;

    let out_sit = ruta_sit(&ws, args, &name)?;
    // PVsystCLI does not create the output folder: ensure it exists first.
    if let Some(parent) = std::path::Path::new(&out_sit).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("No se pudo crear {}", parent.display()), e))?;
    }

    let mut v = json!({
        "nombre_sitio": name,
        "latitud": lat,
        "longitud": lon,
        "altitud": args.get("altitud").cloned().unwrap_or(json!(0)),
        "timezone": num(args, &["timezone", "zona_horaria"]).unwrap_or_else(|| (lon / 15.0).round()),
        "weather_data_file": csv_path.to_string_lossy(),
        "output_sit_file": out_sit,
        "country_code": args.get("country_code").cloned().unwrap_or(json!("")),
        "region": args.get("region").cloned().unwrap_or(json!("")),
        "source": format!("NASA SSE grid cell {}/{}", clim.cell_lat, clim.cell_lon),
        "dry_run": args.get("dry_run").cloned().unwrap_or(json!(false)),
        "timeout_s": args.get("timeout_s").cloned().unwrap_or(Value::Null),
    });
    if let Some(ws_val) = args.get("workspace") {
        v["workspace"] = ws_val.clone();
    }

    let create_site = crear_sitio(paths, &v)?;
    let mut out = resumen_clima(
        json!({
            "site": name,
            "sit_file": out_sit,
            "weather_csv": csv_path.to_string_lossy(),
            "create_site": create_site,
        }),
        &clim,
    );
    if args.get("altitud").is_none() {
        out["warning"] = json!(
            "altitud not given: the site was created at 0 m (pass 'altitud' for highland sites)"
        );
    }
    Ok(out)
}

/// Adds the NASA climatology summary to a site response (fresh or reused).
fn resumen_clima(mut base: Value, clim: &nasa::Climatology) -> Value {
    let round1 = |x: f64| crate::format::redondear(x, 1);
    let monthly_ghi: Vec<f64> = nasa::monthly_totals(&clim.ghi_daily)
        .iter()
        .map(|v| round1(*v))
        .collect();
    base["weather_source"] = json!(format!(
        "NASA SSE 6.0 grid cell {}/{} (regional 1-degree average)",
        clim.cell_lat, clim.cell_lon
    ));
    base["annual_ghi_kwh_m2"] = json!(round1(clim.annual_ghi()));
    base["monthly_ghi_kwh_m2"] = json!(monthly_ghi);
    base["monthly_temp_c"] = json!(clim.temp.iter().map(|v| round1(*v)).collect::<Vec<_>>());
    base
}
