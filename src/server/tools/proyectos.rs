//! Tool for project lifecycle: creation, sizing, variants, diagnostics and cleanup.

use super::declaracion::{Anotaciones, Discriminador, Ejecutor, Herramienta, Sinonimos, TipoDato};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use crate::projects::{self, lifecycle};
use serde_json::Value;

pub(crate) const TOOL_PROYECTO: Herramienta = Herramienta {
    nombre: "proyecto",
    descripcion: crate::idioma::Texto { es: "Ciclo de vida de proyectos y variantes: crear, eliminar, renombrar, dimensionar (pre-dimensionado de PVsyst), diagnosticar, anadir objetos, convertir a seguidor, limite de inyeccion, almacenamiento, perdidas, bombeo, economia, P50-P90, configurar (varios cambios de variante en una llamada con 'pasos').", en: "Project and variant lifecycle: create, delete, rename, size (PVsyst pre-sizing), diagnose, add objects, convert to tracker, injection limit, storage, losses, pumping, economics, P50-P90, configure (several variant changes in one call via 'pasos')." },
    anotaciones: Anotaciones {
        titulo: crate::idioma::Texto { es: "Proyecto", en: "Project" },
        es_solo_lectura: false,
        es_destructiva: true,
        es_idempotente: false,
        usa_red: false,
    },
    propiedades: &[
        ("proyecto", TipoDato::String),
        ("plantilla", TipoDato::String),
        ("nombre_nuevo", TipoDato::String),
        ("sitio", TipoDato::String),
        ("variante", TipoDato::String),
        ("variante_origen", TipoDato::String),
        ("variante_nueva", TipoDato::String),
        ("modulo", TipoDato::String),
        ("inversor", TipoDato::String),
        ("n_mod_serie", TipoDato::IntegerOrString),
        ("n_strings", TipoDato::IntegerOrString),
        ("comentario", TipoDato::String),
        ("sobrescribir", TipoDato::Boolean),
        ("kwp", TipoDato::Number),
        ("n_inversores", TipoDato::Integer),
        ("pnom_modulo", TipoDato::Number),
        ("reparar", TipoDato::Boolean),
        ("elemento", TipoDato::String),
        ("azimut_eje", TipoDato::Number),
        ("angulo_max", TipoDato::Number),
        ("pitch_m", TipoDato::Number),
        ("ancho_colector_m", TipoDato::Number),
        ("n_filas", TipoDato::Integer),
        ("backtracking", TipoDato::Boolean),
        ("potencia_kw", TipoDato::Number),
        ("perdida_separada", TipoDato::Boolean),
        ("tan_phi", TipoDato::Number),
        ("estrategia", TipoDato::String),
        ("bateria", TipoDato::String),
        ("n_serie", TipoDato::Integer),
        ("n_paralelo", TipoDato::Integer),
        ("p_max_carga_kw", TipoDato::Number),
        ("p_max_descarga_kw", TipoDato::Number),
        ("soc_inicial", TipoDato::Number),
        ("suciedad_pct", TipoDato::Number),
        ("uc_termico", TipoDato::Number),
        ("lid_pct", TipoDato::Number),
        ("mismatch_pct", TipoDato::Number),
        ("caudal_m3_dia", TipoDato::Number),
        ("profundidad_estatica_m", TipoDato::Number),
        ("bomba", TipoDato::String),
        ("moneda", TipoDato::String),
        ("vida_anios", TipoDato::Integer),
        ("tasa_descuento", TipoDato::Number),
        ("inflacion", TipoDato::Number),
        ("tarifa_venta_kwh", TipoDato::Number),
        ("workspace", TipoDato::String),
        ("permitir_fuera_del_workspace", TipoDato::Boolean),
        ("dry_run", TipoDato::Boolean),
        ("via", TipoDato::String),
        ("pasos", TipoDato::Array),
    ],
    requeridos: &[],
    discriminador: Some(Discriminador {
        campo: "accion",
        defecto: None,
        valores: &[
            "crear",
            "eliminar",
            "renombrar",
            "crear_variante",
            "dimensionar",
            "doctor",
            "anadir",
            "seguidor",
            "limite_inyeccion",
            "almacenamiento",
            "perdidas",
            "bombeo",
            "economia",
            "configurar",
            "p50_p90",
            "limpiar_temporales",
        ],
    }),
    requeridos_por_accion: &[
        ("crear", &["plantilla"]),
        ("eliminar", &["proyecto"]),
        ("renombrar", &["proyecto", "nombre_nuevo"]),
        ("crear_variante", &["proyecto", "variante_nueva"]),
        ("dimensionar", &["proyecto"]),
        ("doctor", &["proyecto"]),
        ("anadir", &["proyecto", "elemento"]),
        ("seguidor", &["proyecto"]),
        ("limite_inyeccion", &["proyecto", "potencia_kw"]),
        ("almacenamiento", &["proyecto", "estrategia"]),
        ("perdidas", &["proyecto"]),
        ("bombeo", &["proyecto"]),
        ("economia", &["proyecto"]),
        ("configurar", &["proyecto", "pasos"]),
        ("p50_p90", &["proyecto"]),
    ],
    sinonimos: SINONIMOS_PROYECTO,
    ejecutar: Ejecutor::Simple(ejecutar_proyecto),
};

const SINONIMOS_PROYECTO: Sinonimos = Sinonimos {
    argumentos: &[
        ("action", "accion"),
        ("project", "proyecto"),
        ("template", "plantilla"),
        ("new_name", "nombre_nuevo"),
        ("variant", "variante"),
        ("source_variant", "variante_origen"),
        ("new_variant", "variante_nueva"),
        ("module", "modulo"),
        ("inverter", "inversor"),
        ("comment", "comentario"),
        ("overwrite", "sobrescribir"),
        ("repair", "reparar"),
        ("element", "elemento"),
        ("strategy", "estrategia"),
        ("battery", "bateria"),
        ("pump", "bomba"),
        ("currency", "moneda"),
    ],
    valores: &[
        ("create", "crear"),
        ("delete", "eliminar"),
        ("remove", "eliminar"),
        ("rename", "renombrar"),
        ("create_variant", "crear_variante"),
        ("clone_variant", "crear_variante"),
        ("size", "dimensionar"),
        ("sizing", "dimensionar"),
        ("add", "anadir"),
        ("add_object", "anadir"),
        ("tracker", "seguidor"),
        ("convert_tracker", "seguidor"),
        ("injection_limit", "limite_inyeccion"),
        ("storage", "almacenamiento"),
        ("losses", "perdidas"),
        ("pumping", "bombeo"),
        ("economics", "economia"),
        ("economy", "economia"),
        ("clean_temps", "limpiar_temporales"),
    ],
};

fn ejecutar_proyecto(paths: &Paths, args: &Value) -> Result<Value> {
    let valor = TOOL_PROYECTO.accion(args);
    match valor {
        "crear" => lifecycle::crear_proyecto(paths, args),
        "eliminar" => lifecycle::eliminar_proyecto(paths, args),
        "renombrar" => lifecycle::renombrar_proyecto(paths, args),
        "crear_variante" => lifecycle::crear_variante(paths, args),
        "dimensionar" => lifecycle::dimensionar_variante(paths, args),
        "doctor" => projects::doctor::doctor(paths, args),
        "anadir" => projects::anadir::anadir(paths, args),
        "seguidor" => crate::seguidores::convertir_a_seguidor(paths, args),
        "limite_inyeccion" => crate::inyeccion::definir_limite(paths, args),
        "almacenamiento" => crate::almacenamiento::configurar_almacenamiento(paths, args),
        "perdidas" => crate::perdidas::aplicar_perdidas(paths, args),
        "bombeo" => crate::bombeo::configurar_bombeo(paths, args),
        "economia" => crate::evaluacion_economica::configurar_economia(paths, args),
        "configurar" => configurar_variante(paths, args),
        "p50_p90" => configurar_p50(paths, args),
        "limpiar_temporales" => projects::temporales::limpiar_temporales(paths, args),
        _ => Err(Error::Invalid(format!(
            "Value '{valor}' not supported for proyecto"
        ))),
    }
}

/// Actions `configurar` may chain: they only modify an existing variant.
const PASO_OBJETO: Texto = Texto {
    es: "Cada elemento de 'pasos' debe ser un objeto {accion:…}",
    en: "Each element of 'pasos' must be an object {accion:…}",
};
const PASO_ACCION: Texto = Texto {
    es: "Acción '{accion}' no admitida en 'pasos'; válidas: {validas}",
    en: "Action '{accion}' not supported in 'pasos'; valid ones: {validas}",
};
const PASOS_LISTA: Texto = Texto {
    es: "'pasos' debe ser una lista no vacía de {accion:…}",
    en: "'pasos' must be a non-empty list of {accion:…}",
};
const PASO_INVALIDO: Texto = Texto {
    es: "Paso {n} inválido; no se aplicó ninguno",
    en: "Step {n} invalid; none were applied",
};
const PASO_FALLIDO: Texto = Texto {
    es: "Paso {n} fallido tras aplicar {aplicados} paso(s)",
    en: "Step {n} failed after {aplicados} step(s) applied",
};

const ACCIONES_CONFIGURABLES: &[&str] = &[
    "dimensionar",
    "anadir",
    "seguidor",
    "limite_inyeccion",
    "almacenamiento",
    "perdidas",
    "bombeo",
    "economia",
];

/// Step `paso` as a complete `proyecto` call: the parent's `proyecto`, `variante` and
/// `workspace` fill whatever the step leaves out.
fn paso_completo(padre: &Value, paso: &Value, idioma: Idioma) -> Result<Value> {
    let mut completo = paso
        .as_object()
        .cloned()
        .ok_or_else(|| Error::Invalid(PASO_OBJETO.texto_con(idioma, &[])))?;
    let accion = completo.get("accion").and_then(Value::as_str).unwrap_or("");
    if !ACCIONES_CONFIGURABLES.contains(&accion) {
        return Err(Error::Invalid(PASO_ACCION.texto_con(
            idioma,
            &[
                ("accion", accion),
                ("validas", &ACCIONES_CONFIGURABLES.join(", ")),
            ],
        )));
    }
    for clave in ["proyecto", "variante", "workspace"] {
        if let (false, Some(valor)) = (completo.contains_key(clave), padre.get(clave)) {
            completo.insert(clave.into(), valor.clone());
        }
    }
    Ok(Value::Object(completo))
}

/// Applies several variant changes in one call, in order. All steps are validated before
/// the first one runs; if one fails the earlier ones stay applied (each is in the history).
fn configurar_variante(paths: &Paths, args: &Value) -> Result<Value> {
    let pasos = args
        .get("pasos")
        .and_then(Value::as_array)
        .filter(|p| !p.is_empty())
        .ok_or_else(|| Error::Invalid(PASOS_LISTA.texto_con(paths.idioma, &[])))?;
    // Every step goes through the same aliases and schema checks as a direct call.
    let llamadas = pasos
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let llamada = TOOL_PROYECTO
                .preparar_args(&paso_completo(args, p, paths.idioma)?, paths.idioma)?;
            TOOL_PROYECTO.validar(&llamada, paths).map_err(|e| {
                con_prefijo(
                    e,
                    &PASO_INVALIDO.texto_con(paths.idioma, &[("n", &(i + 1).to_string())]),
                )
            })?;
            Ok(llamada)
        })
        .collect::<Result<Vec<_>>>()?;
    let mut resultados = Vec::with_capacity(llamadas.len());
    for (indice, llamada) in llamadas.iter().enumerate() {
        let resultado = ejecutar_proyecto(paths, llamada).map_err(|e| {
            con_prefijo(
                e,
                &PASO_FALLIDO.texto_con(
                    paths.idioma,
                    &[
                        ("n", &(indice + 1).to_string()),
                        ("aplicados", &indice.to_string()),
                    ],
                ),
            )
        })?;
        resultados.push(resultado);
    }
    Ok(serde_json::json!({ "pasos_aplicados": resultados.len(), "resultados": resultados }))
}

/// `e` with `prefijo` before its message, keeping its kind so the answer's error code
/// still says what went wrong (not every failure is an invalid argument).
fn con_prefijo(e: Error, prefijo: &str) -> Error {
    let con = |m: String| format!("{prefijo}: {m}");
    match e {
        Error::Invalid(m) => Error::Invalid(con(m)),
        Error::NotFound(m) => Error::NotFound(con(m)),
        Error::Format(m) => Error::Format(con(m)),
        Error::Cli(m) => Error::Cli(con(m)),
        Error::Gui(m) => Error::Gui(con(m)),
        Error::Network(m) => Error::Network(con(m)),
        Error::Io { context, source } => Error::Io {
            context: con(context),
            source,
        },
        otro @ (Error::MissingArg { .. } | Error::MissingArgs { .. }) => otro,
    }
}

fn configurar_p50(paths: &Paths, args: &Value) -> Result<Value> {
    crate::aproximaciones::exigir_ruta_pvsyst_sin_workspace(
        paths,
        args,
        crate::idioma::Texto {
            es: "La configuración P50/P90",
            en: "The P50/P90 configuration",
        },
    )?;
    let proyecto = crate::args::req_str(args, &["proyecto"], "")?;
    let proyecto = proyecto.strip_suffix(".PRJ").unwrap_or(proyecto);
    let variante = crate::args::opt_str(args, &["variante"]).unwrap_or("VC0");
    if crate::args::flag(args, &["dry_run"]) {
        return Ok(projects::gui::p50::plan(paths, proyecto, variante));
    }
    projects::gui::p50::configurar(paths, proyecto, variante)
}
