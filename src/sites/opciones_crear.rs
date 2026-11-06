//! Command-line options of PVsystCLI `create-site` built from the `sitio crear` arguments.

use super::entrada::exigir_archivo;
use super::texto;
use crate::args::missing;
use crate::error::Result;
use crate::format::value_text;
use serde_json::Value;
use std::path::Path;

/// Optional `create-site` flags present in `args`.
pub(super) fn opciones_sitio(args: &Value) -> Vec<String> {
    const OPCIONALES: [(&str, &str); 6] = [
        ("altitud", "-alt"),
        ("timezone", "-tz"),
        ("country_code", "-cc"),
        ("region", "-r"),
        ("albedo", "-alb"),
        ("source", "-s"),
    ];
    OPCIONALES
        .iter()
        .filter_map(|(key, flag)| {
            let s = args
                .get(*key)
                .map(value_text)
                .filter(|s| !s.is_empty() && s != "null")?;
            Some(format!("{flag}:{s}"))
        })
        .collect()
}

/// Role of a file flag: inputs must exist, an output only has to stay in scope.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Archivo {
    EntradaObligatoria,
    EntradaOpcional,
    Salida,
}

const ARCHIVOS: [(&str, &str, Archivo); 3] = [
    ("weather_data_file", "-wdf", Archivo::EntradaObligatoria),
    ("cmd_options_file", "-cof", Archivo::EntradaOpcional),
    ("output_sit_file", "-osf", Archivo::Salida),
];

/// File flags of `create-site`, each resolved under the caller's scope like
/// `convertir_meteo` does: the weather file and the options file must exist; the
/// output `.SIT` may not leave the workspace unless `permitir_fuera_del_workspace`.
pub(super) fn archivos_sitio(ws: &Path, args: &Value) -> Result<Vec<String>> {
    let alcance = crate::workspace::Alcance::de_args(args);
    let mut opciones = Vec::with_capacity(ARCHIVOS.len());
    for (clave, bandera, rol) in ARCHIVOS {
        let Some(valor) = texto(args, &[clave]) else {
            if rol == Archivo::EntradaObligatoria {
                return Err(missing(clave, ""));
            }
            continue;
        };
        let ruta = crate::workspace::resolver_ruta(ws, &valor, alcance)?;
        if rol != Archivo::Salida {
            exigir_archivo(clave, &ruta)?;
        }
        opciones.push(format!("{bandera}:{}", ruta.display()));
    }
    Ok(opciones)
}
