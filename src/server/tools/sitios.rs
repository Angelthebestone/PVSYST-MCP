//! Tool for sites and meteo: site synthesis, solar geometry, PVGIS horizon and TMY.

use super::declaracion::{Anotaciones, Discriminador, Ejecutor, Herramienta, Sinonimos, TipoDato};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::sites;
use serde_json::Value;

pub(crate) const TOOL_SITIO: Herramienta = Herramienta {
    nombre: "sitio",
    descripcion: "Sitios y meteo: crear (coordenadas o CSV), convertir, geometria solar, horizonte y TMY de PVGIS. / EN: Sites and weather: create (coordinates or CSV), convert, solar geometry, PVGIS horizon and TMY.",
    anotaciones: Anotaciones {
        titulo: "Sitio / Site",
        es_solo_lectura: false,
        es_destructiva: true,
        es_idempotente: false,
        usa_red: true,
    },
    propiedades: &[
        ("origen", TipoDato::String),
        ("nombre_sitio", TipoDato::String),
        ("latitud", TipoDato::NumberOrString),
        ("longitud", TipoDato::NumberOrString),
        ("altitud", TipoDato::NumberOrString),
        ("timezone", TipoDato::NumberOrString),
        ("country_code", TipoDato::String),
        ("region", TipoDato::String),
        ("albedo", TipoDato::NumberOrString),
        ("weather_data_file", TipoDato::String),
        ("source", TipoDato::String),
        ("output_sit_file", TipoDato::String),
        ("cmd_options_file", TipoDato::String),
        ("log_level", TipoDato::Integer),
        ("dry_run", TipoDato::Boolean),
        ("input_csv_file", TipoDato::String),
        ("input_mef_file", TipoDato::String),
        ("input_sit_file", TipoDato::String),
        ("output_met_file", TipoDato::String),
        ("input_mef_timeshift", TipoDato::NumberOrString),
        ("proyecto", TipoDato::String),
        ("variante", TipoDato::String),
        ("tilt", TipoDato::Number),
        ("azimut", TipoDato::Number),
        ("fecha", TipoDato::String),
        ("sit_file", TipoDato::String),
        ("timeout_s", TipoDato::Integer),
        ("permitir_fuera_del_workspace", TipoDato::Boolean),
        ("via", TipoDato::String),
        ("workspace", TipoDato::String),
        ("archivo", TipoDato::String),
        ("titulo", TipoDato::String),
        ("puntos", TipoDato::Array),
        ("variables", TipoDato::Array),
        ("valores", TipoDato::ArrayOrObject),
    ],
    requeridos: &[],
    discriminador: Some(Discriminador {
        campo: "accion",
        defecto: Some("crear"),
        valores: &[
            "crear",
            "convertir_meteo",
            "geometria_solar",
            "horizonte_pvgis",
            "meteo_pvgis",
            "escribir_horizonte",
            "escribir_serie",
        ],
    }),
    requeridos_por_accion: &[
        ("geometria_solar", &["latitud", "longitud"]),
        ("horizonte_pvgis", &["proyecto"]),
        ("meteo_pvgis", &["latitud", "longitud", "nombre_sitio"]),
        ("escribir_horizonte", &["archivo", "puntos"]),
        ("escribir_serie", &["archivo", "variables", "valores"]),
    ],
    sinonimos: SINONIMOS_SITIO,
    ejecutar: Ejecutor::Simple(ejecutar_sitio),
};

const SINONIMOS_SITIO: Sinonimos = Sinonimos {
    argumentos: &[
        ("action", "accion"),
        ("site_name", "nombre_sitio"),
        ("latitude", "latitud"),
        ("longitude", "longitud"),
        ("altitude", "altitud"),
        ("project", "proyecto"),
        ("variant", "variante"),
        ("date", "fecha"),
        ("file", "archivo"),
        ("points", "puntos"),
        ("title", "titulo"),
    ],
    valores: &[
        ("create", "crear"),
        ("convert_meteo", "convertir_meteo"),
        ("solar_geometry", "geometria_solar"),
        ("pvgis_horizon", "horizonte_pvgis"),
        ("pvgis_meteo", "meteo_pvgis"),
        ("write_horizon", "escribir_horizonte"),
        ("write_series", "escribir_serie"),
    ],
};

fn ejecutar_sitio(paths: &Paths, args: &Value) -> Result<Value> {
    let valor = args
        .get("accion")
        .and_then(Value::as_str)
        .unwrap_or("crear");
    match valor {
        "crear" => {
            let origen = args
                .get("origen")
                .and_then(Value::as_str)
                .unwrap_or("coordenadas");
            match origen {
                "coordenadas" => sites::crear_sitio_coordenadas(paths, args),
                "csv" => sites::crear_sitio(paths, args),
                otro => Err(Error::Invalid(format!(
                    "Origen '{otro}' no válido: usa coordenadas o csv"
                ))),
            }
        }
        "convertir_meteo" => sites::convertir_meteo(paths, args),
        "geometria_solar" => sites::solar::geometria_solar(paths, args),
        "horizonte_pvgis" => sites::pvgis::horizonte(paths, args),
        "meteo_pvgis" => sites::pvgis::meteo_pvgis(paths, args),
        "escribir_horizonte" => crate::horizonte_archivo::escribir_horizonte(paths, args),
        "escribir_serie" => crate::serie_horaria::escribir_serie(paths, args),
        _ => Err(Error::Invalid(format!(
            "Valor '{valor}' no soportado para sitio"
        ))),
    }
}
