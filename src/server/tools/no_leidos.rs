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

impl NoLeido {
    const fn new(
        herramienta: &'static str,
        accion: &'static str,
        clave: &'static str,
        leido_si: &'static [&'static str],
    ) -> Self {
        Self {
            herramienta,
            accion,
            clave,
            leido_si,
        }
    }
}

/// Declared keys an action does not read. Only the gaps measured against the
/// handlers are listed: an action not named here reads every key it declares.
/// `leido_si` covers the conditional reads.
pub(super) const NO_LEIDOS: &[NoLeido] = &[
    NoLeido::new("proyecto", "dimensionar", "n_serie", &[]),
    NoLeido::new("proyecto", "dimensionar", "n_paralelo", &[]),
    NoLeido::new("resultados", "csv", "internet", &[]),
    NoLeido::new("proyecto", "perdidas", "perdida_separada", &[]),
    NoLeido::new("proyecto", "perdidas", "comentario", &[]),
    // `ejecutar` builds a batch when `filas` is present and a parameter file when
    // `parametros` is: those keys are read then, and ignored otherwise.
    NoLeido::new("simular", "ejecutar", "variables", &["filas"]),
    NoLeido::new("simular", "ejecutar", "destino", &["filas", "parametros"]),
    NoLeido::new("simular", "ejecutar", "descripciones", &["parametros"]),
    NoLeido::new("simular", "ejecutar", "iniciales", &["parametros"]),
    NoLeido::new("simular", "ejecutar", "unidades", &["parametros"]),
    // `con_salidas_por_defecto` honours the output paths, and the batch/parameter
    // builders resolve their destinations, so the flag is read with any of them.
    NoLeido::new(
        "simular",
        "ejecutar",
        "permitir_fuera_del_workspace",
        &["filas", "parametros", "output_csv", "report_pdf"],
    ),
    // Keys of the other `simular` actions (`preestimar`, `barrido`, `estado`,
    // `preparar`) that `ejecutar` never reads.
    NoLeido::new("simular", "ejecutar", "latitud", &[]),
    NoLeido::new("simular", "ejecutar", "longitud", &[]),
    NoLeido::new("simular", "ejecutar", "kwp", &[]),
    NoLeido::new("simular", "ejecutar", "tilt", &[]),
    NoLeido::new("simular", "ejecutar", "azimut", &[]),
    NoLeido::new("simular", "ejecutar", "n", &[]),
    NoLeido::new("simular", "ejecutar", "trabajo", &[]),
    NoLeido::new("simular", "ejecutar", "archivo_rvt", &[]),
    NoLeido::new("simular", "ejecutar", "operacion", &[]),
    // `pvsyst_cli` has no action: `via` is validated by the approximation gate and
    // never changes what the passthrough runs.
    NoLeido::new("pvsyst_cli", "", "via", &[]),
    NoLeido::new("workspace", "historial", "proyecto", &[]),
    NoLeido::new("workspace", "historial", "ruta", &[]),
    NoLeido::new("sitio", "convertir_meteo", "output_sit_file", &[]),
    NoLeido::new("sitio", "convertir_meteo", "sit_file", &[]),
    NoLeido::new("sitio", "convertir_meteo", "weather_data_file", &[]),
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
