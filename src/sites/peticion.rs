//! Arguments of the `crear_sitio` request derived from coordinates and NASA climatology.

use super::num;
use serde_json::{json, Value};

/// NASA SSE grid cell that supplies the monthly climatology of these coordinates.
/// The grids are indexed by whole degrees (lower-left corner of the cell).
pub(super) fn fuente_nasa(lat: f64, lon: f64) -> String {
    format!(
        "NASA SSE grid cell {}/{}",
        lat.floor() as i64,
        lon.floor() as i64
    )
}

pub(super) fn construir_peticion_crear(
    name: &str,
    (lat, lon): (f64, f64),
    source: &str,
    csv_path: &std::path::Path,
    out_sit: &str,
    args: &Value,
) -> Value {
    let mut v = json!({
        "nombre_sitio": name,
        "latitud": lat,
        "longitud": lon,
        "altitud": args.get("altitud").cloned().unwrap_or(json!(0)),
        "timezone": num(args, "timezone").unwrap_or_else(|| (lon / 15.0).round()),
        "weather_data_file": csv_path.to_string_lossy(),
        "output_sit_file": out_sit,
        "country_code": args.get("country_code").cloned().unwrap_or(json!("")),
        "region": args.get("region").cloned().unwrap_or(json!("")),
        "source": source,
        "dry_run": args.get("dry_run").cloned().unwrap_or(json!(false)),
        "timeout_s": args.get("timeout_s").cloned().unwrap_or(Value::Null),
    });
    if let Some(ws_val) = args.get("workspace") {
        v["workspace"] = ws_val.clone();
    }
    v
}
