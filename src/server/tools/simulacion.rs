//! Tool for simulation orchestration: batch setup, execution, status, and sweeps.

use super::declaracion::{Anotaciones, Discriminador, Ejecutor, Herramienta, Sinonimos, TipoDato};
use crate::args::{flag, missing, DRY_RUN};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::progreso::SeguimientoTarea;
use crate::simulation::{self, batch, preestimate};
use serde_json::{Map, Value};

pub(crate) const TOOL_SIMULAR: Herramienta = Herramienta {
    nombre: "simular",
    descripcion: "Simulacion: ejecutar (con filas/parametros opcionales), preparar entrada, estado, preestimar y barrido. / EN: Simulation: run (optional rows/parameters), prepare input, status, pre-estimate and sweep.",
    anotaciones: Anotaciones {
        titulo: "Simular / Simulate",
        es_solo_lectura: false,
        es_destructiva: false,
        es_idempotente: false,
        usa_red: false,
    },
    propiedades: &[
        ("proyecto", TipoDato::String),
        ("variante", TipoDato::String),
        ("output_csv", TipoDato::String),
        ("report_pdf", TipoDato::String),
        ("batch_params_file", TipoDato::String),
        ("batch_rvt_file", TipoDato::String),
        ("input_rvt_file", TipoDato::String),
        ("archivo_rvt", TipoDato::String),
        ("operacion", TipoDato::String),
        ("cmd_options_file", TipoDato::String),
        ("input_met_file", TipoDato::String),
        ("input_sfi_file", TipoDato::String),
        ("input_param_file", TipoDato::String),
        ("site", TipoDato::String),
        ("synthetic_weather", TipoDato::Boolean),
        ("time_step", TipoDato::String),
        ("start_date", TipoDato::String),
        ("end_date", TipoDato::String),
        ("log_level", TipoDato::Integer),
        ("overwrite_default_csv", TipoDato::Boolean),
        ("extra_options", TipoDato::Array),
        ("recalcular_sombreado", TipoDato::Boolean),
        ("paginas_reporte", TipoDato::Array),
        ("idioma_reporte", TipoDato::String),
        ("dry_run", TipoDato::Boolean),
        ("abrir_gui", TipoDato::Boolean),
        ("segundo_plano", TipoDato::Boolean),
        ("trabajo", TipoDato::String),
        ("filas", TipoDato::Array),
        ("variables", TipoDato::Array),
        ("destino", TipoDato::String),
        ("parametros", TipoDato::Object),
        ("iniciales", TipoDato::Object),
        ("unidades", TipoDato::Object),
        ("descripciones", TipoDato::Object),
        ("latitud", TipoDato::Number),
        ("longitud", TipoDato::Number),
        ("kwp", TipoDato::Number),
        ("tilt", TipoDato::Number),
        ("azimut", TipoDato::Number),
        ("n", TipoDato::Integer),
        ("confirmar", TipoDato::Boolean),
        ("sin_cache", TipoDato::Boolean),
        ("timeout_s", TipoDato::Integer),
        ("workspace", TipoDato::String),
        ("permitir_fuera_del_workspace", TipoDato::Boolean),
    ],
    requeridos: &[],
    discriminador: Some(Discriminador {
        campo: "accion",
        defecto: Some("ejecutar"),
        valores: &["ejecutar", "preparar", "estado", "preestimar", "barrido"],
    }),
    requeridos_por_accion: &[
        ("estado", &["trabajo"]),
        ("preestimar", &["latitud", "longitud", "kwp"]),
        ("barrido", &["latitud", "longitud", "kwp"]),
    ],
    sinonimos: SINONIMOS_SIMULAR,
    ejecutar: Ejecutor::Seguido(ejecutar_simular),
};

const SINONIMOS_SIMULAR: Sinonimos = Sinonimos {
    argumentos: &[
        ("action", "accion"),
        ("project", "proyecto"),
        ("variant", "variante"),
        ("job", "trabajo"),
        ("rows", "filas"),
        ("target", "destino"),
        ("parameters", "parametros"),
        ("latitude", "latitud"),
        ("longitude", "longitud"),
        ("csv", "output_csv"),
        ("pdf", "report_pdf"),
        ("bpf", "batch_params_file"),
        ("brf", "batch_rvt_file"),
        ("irf", "input_rvt_file"),
        ("imf", "input_met_file"),
        ("isf", "input_sfi_file"),
        ("ipf", "input_param_file"),
        ("sitio", "site"),
        ("cof", "cmd_options_file"),
    ],
    valores: &[
        ("run", "ejecutar"),
        ("execute", "ejecutar"),
        ("prepare", "preparar"),
        ("status", "estado"),
        ("preestimate", "preestimar"),
        ("sweep", "barrido"),
    ],
};

fn ejecutar_simular(
    paths: &Paths,
    args: &Value,
    seguimiento: &SeguimientoTarea<'_>,
) -> Result<Value> {
    let valor = TOOL_SIMULAR.accion(args);
    match valor {
        "ejecutar" => {
            exigir_ensayo_sin_escritura(args)?;
            let (vivos, _) = preparar_entrada(paths, args)?;
            simulation::lanzar_simulacion_seguida(paths, &vivos, seguimiento)
        }
        "preparar" => {
            if args.get("batch_rvt_file").is_some()
                || args.get("archivo_rvt").is_some()
                || args.get("operacion").is_some()
            {
                return crate::plantilla_variables::gestionar_rvt(paths, args);
            }
            let (_, preparados) = preparar_entrada(paths, args)?;
            if preparados.is_empty() {
                return Err(missing("filas", "o 'parametros'"));
            }
            Ok(Value::Object(preparados))
        }
        "estado" => simulation::estado_simulacion(paths, args),
        "preestimar" => preestimate::preestimar(paths, args),
        "barrido" => preestimate::barrido(paths, args, seguimiento),
        _ => Err(Error::Invalid(format!(
            "Valor '{valor}' no soportado para simular"
        ))),
    }
}

/// `filas` and `parametros` are written to the workspace before the run starts, so
/// a dry run that carried them would already have changed files.
fn exigir_ensayo_sin_escritura(args: &Value) -> Result<()> {
    let escribe = args.get("filas").is_some() || args.get("parametros").is_some();
    if flag(args, &[DRY_RUN]) && escribe {
        return Err(Error::Invalid(
            "dry_run no admite 'filas' ni 'parametros': prepararlos escribe archivos del workspace. Quita dry_run, o prepara antes con accion:preparar".into(),
        ));
    }
    Ok(())
}

fn preparar_entrada(paths: &Paths, args: &Value) -> Result<(Value, Map<String, Value>)> {
    let mut vivos: Map<String, Value> = args.as_object().cloned().unwrap_or_default();
    let mut preparados = Map::new();
    if args.get("filas").is_some() {
        let r = batch::crear_batch(paths, args)?;
        incorporar_archivos_batch(&r, &mut vivos, &mut preparados);
    }
    if args.get("parametros").is_some() {
        let r = batch::modificar_parametros(paths, args)?;
        if let Some(ruta) = r.get("param_file") {
            vivos.insert("input_param_file".to_string(), ruta.clone());
            preparados.insert("param_file".to_string(), ruta.clone());
        }
    }
    Ok((Value::Object(vivos), preparados))
}

fn incorporar_archivos_batch(
    r: &Value,
    vivos: &mut Map<String, Value>,
    preparados: &mut Map<String, Value>,
) {
    for clave in ["batch_params_file", "batch_rvt_file"] {
        if let Some(valor) = r.get(clave) {
            vivos.insert(clave.to_string(), valor.clone());
            preparados.insert(clave.to_string(), valor.clone());
        }
    }
}
