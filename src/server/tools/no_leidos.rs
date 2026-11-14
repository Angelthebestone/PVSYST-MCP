//! Declared keys an action does not read. The schema is per tool, so a key that
//! belongs to another action of the same tool reaches the handler and is dropped;
//! naming it in `ignorados` is the only way the caller learns it was not applied.

use serde_json::Value;

/// One declared key an action does not read, and the arguments that make the
/// handler read it anyway. An empty `leido_si` means no argument makes it read.
pub(super) struct NoLeido {
    pub(super) herramienta: &'static str,
    pub(super) accion: &'static str,
    pub(super) clave: &'static str,
    pub(super) leido_si: &'static [&'static str],
}

macro_rules! no_leido {
    ($herramienta:expr, $accion:expr, $clave:expr, $leido_si:expr $(,)?) => {
        NoLeido {
            herramienta: $herramienta,
            accion: $accion,
            clave: $clave,
            leido_si: $leido_si,
        }
    };
}

/// Declared keys an action does not read. Only the gaps measured against the
/// handlers are listed: an action not named here reads every key it declares.
/// `leido_si` covers the conditional reads.
pub(super) const NO_LEIDOS: &[NoLeido] = &[
    no_leido!("proyecto", "dimensionar", "n_serie", &[]),
    no_leido!("proyecto", "dimensionar", "n_paralelo", &[]),
    no_leido!("resultados", "csv", "internet", &[]),
    no_leido!("proyecto", "perdidas", "perdida_separada", &[]),
    no_leido!("proyecto", "perdidas", "comentario", &[]),
    // `ejecutar` builds a batch when `filas` is present and a parameter file when
    // `parametros` is: those keys are read then, and ignored otherwise.
    no_leido!("simular", "ejecutar", "variables", &["filas"]),
    no_leido!("simular", "ejecutar", "destino", &["filas", "parametros"]),
    no_leido!("simular", "ejecutar", "descripciones", &["parametros"]),
    no_leido!("simular", "ejecutar", "iniciales", &["parametros"]),
    no_leido!("simular", "ejecutar", "unidades", &["parametros"]),
    // `con_salidas_por_defecto` honours the output paths, and the batch/parameter
    // builders resolve their destinations, so the flag is read with any of them.
    no_leido!(
        "simular",
        "ejecutar",
        "permitir_fuera_del_workspace",
        &["filas", "parametros", "output_csv", "report_pdf"],
    ),
    // Keys of the other `simular` actions (`preestimar`, `barrido`, `estado`,
    // `preparar`) that `ejecutar` never reads.
    no_leido!("simular", "ejecutar", "latitud", &[]),
    no_leido!("simular", "ejecutar", "longitud", &[]),
    no_leido!("simular", "ejecutar", "kwp", &[]),
    no_leido!("simular", "ejecutar", "tilt", &[]),
    no_leido!("simular", "ejecutar", "azimut", &[]),
    no_leido!("simular", "ejecutar", "n", &[]),
    no_leido!("simular", "ejecutar", "trabajo", &[]),
    no_leido!("simular", "ejecutar", "archivo_rvt", &[]),
    no_leido!("simular", "ejecutar", "operacion", &[]),
    // `pvsyst_cli` has no action: `via` is validated by the approximation gate and
    // never changes what the passthrough runs.
    no_leido!("pvsyst_cli", "", "via", &[]),
    no_leido!("workspace", "historial", "proyecto", &[]),
    no_leido!("workspace", "historial", "ruta", &[]),
    no_leido!("sitio", "convertir_meteo", "output_sit_file", &[]),
    no_leido!("sitio", "convertir_meteo", "sit_file", &[]),
    no_leido!("sitio", "convertir_meteo", "weather_data_file", &[]),
];

/// Declared keys an action never reads, grouped per action to keep the table short.
const NO_LEIDOS_SIEMPRE: &[(&str, &str, &[&str])] = &[(
    "proyecto",
    "doctor",
    &[
        "plantilla",
        "nombre_nuevo",
        "sitio",
        "variante_origen",
        "variante_nueva",
        "modulo",
        "inversor",
        "n_mod_serie",
        "n_strings",
        "comentario",
        "sobrescribir",
        "kwp",
        "n_inversores",
        "pnom_modulo",
        "elemento",
        "azimut_eje",
        "angulo_max",
        "pitch_m",
        "ancho_colector_m",
        "n_filas",
        "backtracking",
        "potencia_kw",
        "perdida_separada",
        "tan_phi",
        "estrategia",
        "bateria",
        "n_serie",
        "n_paralelo",
        "p_max_carga_kw",
        "p_max_descarga_kw",
        "soc_inicial",
        "suciedad_pct",
        "uc_termico",
        "lid_pct",
        "mismatch_pct",
        "caudal_m3_dia",
        "profundidad_estatica_m",
        "bomba",
        "moneda",
        "vida_anios",
        "tasa_descuento",
        "inflacion",
        "tarifa_venta_kwh",
        "pasos",
    ],
)];

/// Declared keys `accion` of `herramienta` does not read for these `args`.
pub(super) fn no_leidos(herramienta: &str, accion: &str, args: &Value) -> Vec<&'static str> {
    let siempre = NO_LEIDOS_SIEMPRE
        .iter()
        .find(|(tool, acc, _)| *tool == herramienta && *acc == accion)
        .map_or(&[][..], |(_, _, claves)| *claves);
    let condicionales = NO_LEIDOS
        .iter()
        .filter(|n| n.herramienta == herramienta && n.accion == accion)
        .filter(|n| {
            n.leido_si
                .iter()
                .all(|clave| args.get(*clave).is_none_or(Value::is_null))
        })
        .map(|n| n.clave);
    siempre.iter().copied().chain(condicionales).collect()
}
