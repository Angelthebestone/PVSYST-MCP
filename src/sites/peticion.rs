//! Arguments of the `crear_sitio` request derived from coordinates and NASA climatology.

use super::{nasa, num};
use serde_json::{json, Value};

pub(super) fn construir_peticion_crear(
    name: &str,
    (lat, lon): (f64, f64),
    clim: &nasa::Climatology,
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
        "source": format!("NASA SSE grid cell {}/{}", clim.cell_lat, clim.cell_lon),
        "dry_run": args.get("dry_run").cloned().unwrap_or(json!(false)),
        "timeout_s": args.get("timeout_s").cloned().unwrap_or(Value::Null),
    });
    if let Some(ws_val) = args.get("workspace") {
        v["workspace"] = ws_val.clone();
    }
    v
}
