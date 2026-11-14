//! Sites created from coordinates alone: the monthly weather comes from the NASA
//! SSE 1°×1° grids shipped with PVsyst (`DataRO\*_World.*`), so no weather file is
//! needed. This is the easy path for "design X kWp at these coordinates".

use super::nasa;
use super::peticion::{construir_peticion_crear, fuente_nasa};
use super::reuse;
use super::{crear_sitio, num};
use crate::args::{flag, req_numero, workspace_of, DRY_RUN};
use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn nombre_por_coordenadas(lat: f64, lon: f64) -> String {
    format!("Site_{lat:.3}_{lon:.3}")
        .replace('.', "p")
        .replace('-', "m")
}

/// Where the derived monthly climatology goes: the CSV `create-site` consumes.
fn ruta_csv_mensual(ws: &Path, name: &str) -> PathBuf {
    ws.join("UserData").join(format!("{name}_Monthly.csv"))
}

/// Write the derived monthly climatology as the CSV `create-site` consumes.
fn escribir_csv_mensual(csv_path: &Path, clim: &nasa::Climatology) -> Result<()> {
    if let Some(p) = csv_path.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    std::fs::write(csv_path, clim.to_create_site_csv())
        .map_err(|e| Error::io(format!("Cannot write {}", csv_path.display()), e))
}

/// `output_sit_file`, else `<workspace>/Sites/<name>.SIT`.
pub(super) fn ruta_sit(ws: &Path, args: &Value, name: &str) -> Result<String> {
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
/// A site cached under another name is copied to the requested `nombre_sitio`.
fn sitio_reutilizado(ws: &Path, args: &Value, (lat, lon): (f64, f64)) -> Result<Option<Value>> {
    if flag(args, &["forzar"]) || args.get("output_sit_file").is_some() {
        return Ok(None);
    }
    let altitud = num(args, "altitud");
    let nombre = args.get("nombre_sitio").and_then(Value::as_str);
    let busqueda = |nombre| reuse::Busqueda {
        coordenadas: (lat, lon),
        altitud,
        nombre,
    };
    let encontrado = reuse::buscar_existente(ws, busqueda(nombre))
        .or_else(|| reuse::buscar_existente(ws, busqueda(None)));
    let Some(encontrado) = encontrado else {
        return Ok(None);
    };
    let mut existente = match nombre {
        Some(n) => reuse::copiar_con_nombre(ws, &encontrado, n)?,
        None => encontrado,
    };
    existente["create_site"] = json!({ "success": true, "reutilizado": true });
    existente["costo"] = json!({ "ejecuciones": 0 });
    Ok(Some(existente))
}

/// An explicit `nombre_sitio` may not overwrite a different site unless `forzar` is set.
fn exigir_nombre_libre_si_explicito(ws: &Path, args: &Value, name: &str) -> Result<()> {
    if args.get("nombre_sitio").is_none()
        || args.get("output_sit_file").is_some()
        || flag(args, &["forzar"])
    {
        return Ok(());
    }
    reuse::exigir_nombre_libre(ws, name)
}

/// Files a coordinate site needs on disk: the monthly CSV it consumes and the `.SIT`
/// `create-site` writes.
struct DestinoSitio {
    csv: PathBuf,
    sit: String,
}

impl DestinoSitio {
    fn de_args(ws: &Path, args: &Value, nombre: &str) -> Result<Self> {
        Ok(Self {
            csv: ruta_csv_mensual(ws, nombre),
            sit: ruta_sit(ws, args, nombre)?,
        })
    }

    /// Request of the `create-site` call that builds this site, dry run or not.
    fn peticion(&self, nombre: &str, (lat, lon): (f64, f64), args: &Value) -> Value {
        construir_peticion_crear(
            nombre,
            (lat, lon),
            &fuente_nasa(lat, lon),
            &self.csv,
            &self.sit,
            args,
        )
    }

    /// PVsystCLI does not create the output folder: ensure it exists first.
    fn crear_carpeta_salida(&self) -> Result<()> {
        let Some(parent) = Path::new(&self.sit).parent() else {
            return Ok(());
        };
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("No se pudo crear {}", parent.display()), e))
    }
}

/// Create a site from **coordinates alone**: monthly weather is derived from the
/// NASA SSE 1°×1° grids shipped with PVsyst (`DataRO\*_World.*`), so no weather
/// file is needed.
pub(crate) fn crear_sitio_coordenadas(paths: &Paths, args: &Value) -> Result<Value> {
    let lat = req_numero(args, "latitud", "")?;
    let lon = req_numero(args, "longitud", "")?;
    super::entrada::validar_coordenadas(lat, lon)?;
    let name = args
        .get("nombre_sitio")
        .and_then(|v| v.as_str())
        .map_or_else(|| nombre_por_coordenadas(lat, lon), String::from);
    crate::workspace::validar_nombre(&name)?;
    super::entrada::validar_region(args)?;

    let ws = workspace_of(paths, args);
    // Before the grid lookup and the files: `create-site` would crash on another workspace.
    crate::prefs::workspace_configurado::exigir(paths, &ws, "create-site")?;
    let destino = DestinoSitio::de_args(&ws, args, &name)?;
    // A dry run answers with the command line it would run: no grid lookup, no
    // monthly CSV and no output folder either.
    if flag(args, &[DRY_RUN]) {
        return crear_sitio(paths, &destino.peticion(&name, (lat, lon), args));
    }

    let clim = nasa::climatology(lat, lon)?;
    if let Some(existente) = sitio_reutilizado(&ws, args, (lat, lon))? {
        return Ok(resumen_clima(existente, &clim));
    }
    exigir_nombre_libre_si_explicito(&ws, args, &name)?;
    escribir_csv_mensual(&destino.csv, &clim)?;
    destino.crear_carpeta_salida()?;

    let create_site = crear_sitio(paths, &destino.peticion(&name, (lat, lon), args))?;
    let mut out = resumen_clima(
        json!({
            "site": name,
            "sit_file": destino.sit,
            "weather_csv": destino.csv.to_string_lossy(),
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

#[cfg(test)]
#[path = "../../tests/unit/sites/coordenadas.rs"]
mod tests;
