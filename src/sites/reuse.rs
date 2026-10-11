//! Reuse of an already-created site. A `.SIT` written by `crear_sitio_coordenadas`
//! for the same coordinates (and altitude, when given) is returned instead of
//! running PVsystCLI again.

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// Tolerance in degrees (and metres for altitude) for two values to be equal.
const TOLERANCIA: f64 = 1e-4;
/// Prefix of the `Source` field written for sites created from NASA grid cells.
const ORIGEN_NASA: &str = "NASA SSE grid cell";

/// Value of `clave` in a `Key=Value` text (case-insensitive key, BOM tolerant).
fn valor<'a>(texto: &'a str, clave: &str) -> Option<&'a str> {
    texto.lines().find_map(|linea| {
        let (k, v) = linea
            .trim_start_matches('\u{feff}')
            .trim()
            .split_once('=')?;
        k.trim().eq_ignore_ascii_case(clave).then(|| v.trim())
    })
}

/// True when a `.SIT` text was created from a NASA grid cell at these coordinates.
/// The grid cell itself follows from the coordinates, so it is not compared separately.
fn coincide(texto: &str, (lat, lon): (f64, f64), altitud: Option<f64>) -> bool {
    let numero = |clave: &str| valor(texto, clave).and_then(|v| v.parse::<f64>().ok());
    let cerca = |campo: Option<f64>, esperado: f64| {
        campo.is_some_and(|x| (x - esperado).abs() <= TOLERANCIA)
    };
    let origen_nasa = valor(texto, "Source").is_some_and(|s| s.contains(ORIGEN_NASA));
    let altitud_ok = match altitud {
        None => true,
        Some(a) => cerca(numero("Altitude"), a),
    };
    origen_nasa && cerca(numero("Latitude"), lat) && cerca(numero("Longitude"), lon) && altitud_ok
}

/// Existing site in `<workspace>/Sites` matching the coordinates, as a response value
/// with `reutilizado:true`, or `None` when the CLI must create one.
pub(crate) fn buscar_existente(
    workspace: &Path,
    lat: f64,
    lon: f64,
    altitud: Option<f64>,
) -> Option<Value> {
    let mut sitios: Vec<PathBuf> = fs::read_dir(workspace.join("Sites"))
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("SIT")))
        .collect();
    sitios.sort();
    sitios.into_iter().find_map(|sit| {
        let texto = String::from_utf8_lossy(&fs::read(&sit).ok()?).into_owned();
        if !coincide(&texto, (lat, lon), altitud) {
            return None;
        }
        let nombre = sit
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        Some(json!({
            "site": nombre,
            "sit_file": sit.display().to_string(),
            "reutilizado": true,
            "latitud": lat,
            "longitud": lon,
        }))
    })
}

#[cfg(test)]
mod tests {
    use super::coincide;

    const SIT: &str = "\u{feff}Source=NASA SSE grid cell 7/-74\r\nLatitude=7.0708\r\nLongitude=-73.8385\r\nAltitude=100\r\n";

    #[test]
    fn should_match_when_coordinates_and_nasa_source_match() {
        assert!(coincide(SIT, (7.0708, -73.8385), None));
    }

    #[test]
    fn should_not_match_when_latitude_differs() {
        assert!(!coincide(SIT, (7.0, -73.8385), None));
    }

    #[test]
    fn should_only_compare_altitude_when_provided() {
        assert!(!coincide(SIT, (7.0708, -73.8385), Some(250.0)));
        assert!(coincide(SIT, (7.0708, -73.8385), Some(100.0)));
    }

    #[test]
    fn should_not_reuse_sites_from_other_sources() {
        let otro = SIT.replace("NASA SSE grid cell", "Otro origen");
        assert!(!coincide(&otro, (7.0708, -73.8385), None));
    }
}
