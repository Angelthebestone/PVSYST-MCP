//! Argument descriptions of `disenar_sistema`.

use super::{entrada, Entrada};

pub(super) const TABLA: &[Entrada] = &[
    entrada(
        "confirmar",
        "true acepta gastar ejecuciones de prueba (2 o más, o bajo la reserva); sin él se retiene.",
        "true accepts spending trial runs (2 or more, or under the reserve); else held back.",
    ),
    entrada(
        "country_code",
        "País del sitio que se crea; se envía tal cual a PVsystCLI -cc, sin validar.",
        "Country of the created site; passed as given to PVsystCLI -cc, unvalidated.",
    ),
    entrada(
        "n_mod_serie",
        "Módulos en serie por cadena; se escribe en la variante como NModSerie.",
        "Modules in series per string; written to the variant as NModSerie.",
    ),
    entrada(
        "n_strings",
        "Cadenas en paralelo; se escribe en la variante como NStringCh.",
        "Strings in parallel; written to the variant as NStringCh.",
    ),
    entrada(
        "output_csv",
        "Ruta del CSV de resultados de la simulación; por defecto Results/<nombre>.csv.",
        "Path of the simulation results CSV; defaults to Results/<name>.csv.",
    ),
    entrada(
        "pnom_modulo",
        "Potencia nominal del módulo en W (hasta 3000); por defecto, la del .PAN.",
        "Module nominal power in W (up to 3000); defaults to the .PAN value.",
    ),
    entrada(
        "region",
        "Continente del sitio para PVsystCLI -r: africa, asia, europe, south_america...",
        "Continent of the site for PVsystCLI -r: africa, asia, europe, south_america...",
    ),
    entrada(
        "report_pdf",
        "Ruta del informe PDF de la simulación; por defecto Results/<nombre>.pdf.",
        "Path of the simulation PDF report; defaults to Results/<name>.pdf.",
    ),
    entrada(
        "synthetic_weather",
        "Por defecto true: el clima horario se genera con Meteonorm (-swg).",
        "Defaults to true: hourly weather is generated with Meteonorm (-swg).",
    ),
    entrada(
        "timezone",
        "Desfase de la hora local respecto a UTC, en horas; por defecto, longitud/15.",
        "Local clock offset from UTC, in hours; defaults to longitude/15.",
    ),
    entrada(
        "tolerancia_kwp_pct",
        "Desviación máxima admitida del kWp realizado frente al pedido, en % (20 por defecto).",
        "Largest accepted deviation of the realised kWp from the target, in % (20 by default).",
    ),
];
