//! Argument descriptions of `sitio`.

use super::{entrada, Entrada};

pub(super) const TABLA: &[Entrada] = &[
    entrada(
        "albedo",
        "Albedo del suelo en fracción (0,2 = 20 %) que crear pasa a PVsystCLI -alb.",
        "Ground albedo as a fraction (0.2 = 20%) that crear passes to PVsystCLI -alb.",
    ),
    entrada(
        "azimut",
        "Azimut en grados, de -180 a 180; 0 = sur, este negativo.",
        "Azimuth in degrees, -180 to 180; 0 = south, east negative.",
    ),
    entrada(
        "confirmar",
        "true acepta gastar ejecuciones de prueba (2 o más, o bajo la reserva); sin él se retiene.",
        "true accepts spending trial runs (2 or more, or under the reserve); else held back.",
    ),
    entrada(
        "log_level",
        "Nivel de detalle del log de PVsystCLI (-ll), de 0 a 3.",
        "PVsystCLI log detail level (-ll), from 0 to 3.",
    ),
    entrada(
        "origen",
        "crear: coordenadas (defecto, clima mensual NASA SSE) o csv (importa weather_data_file).",
        "crear: coordenadas (default, NASA SSE monthly weather) or csv (imports weather_data_file).",
    ),
    entrada(
        "region",
        "Continente del sitio para PVsystCLI -r: africa, asia, europe, south_america...",
        "Continent of the site for PVsystCLI -r: africa, asia, europe, south_america...",
    ),
    entrada(
        "sit_file",
        "meteo_pvgis: .SIT donde va el clima; si se omite, Sites/<nombre_sitio>.SIT.",
        "meteo_pvgis: .SIT the weather goes to; if omitted, Sites/<nombre_sitio>.SIT.",
    ),
    entrada(
        "tilt",
        "Inclinación del plano en grados, de 0 a 90 (por defecto, |latitud|).",
        "Plane tilt in degrees, 0 to 90 (defaults to |latitude|).",
    ),
    entrada(
        "valores",
        "Series horarias: 8760 u 8784 por variable, en columnas alineadas, filas por hora o mapa.",
        "Hourly series: 8760 or 8784 per variable, as aligned columns, hourly rows or a map.",
    ),
    entrada(
        "variables",
        "Nombres de las columnas a escribir (GlobHor, T_Amb...), uno por columna de valores.",
        "Names of the columns to write (GlobHor, T_Amb...), one per column of valores.",
    ),
];
