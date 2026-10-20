//! Tool for project lifecycle: creation, sizing, variants, diagnostics and cleanup.

use super::declaracion::{Acceso, Anotaciones, Discriminador, Ejecutor, Herramienta, TipoDato};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::projects::{self, lifecycle};
use serde_json::Value;

pub(crate) const TOOL_PROYECTO: Herramienta = Herramienta {
    nombre: "proyecto",
    descripcion: "Ciclo de vida de proyectos y variantes: crear, eliminar, renombrar, dimensionar, diagnosticar, anadir objetos. / EN: Project and variant lifecycle: create, delete, rename, size, diagnose, add objects.",
    acceso: Acceso::Normal,
    anotaciones: Anotaciones {
        titulo: "Proyecto / Project",
        es_solo_lectura: false,
        es_destructiva: true,
        es_idempotente: false,
        usa_red: false,
    },
    propiedades: &[
        ("proyecto", TipoDato::String),
        ("plantilla", TipoDato::String),
        ("nombre_nuevo", TipoDato::String),
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
        ("workspace", TipoDato::String),
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
            "limpiar_temporales",
        ],
    }),
    requeridos_por_accion: &[
        ("crear", &["plantilla", "nombre_nuevo"]),
        ("eliminar", &["proyecto"]),
        ("renombrar", &["proyecto", "nombre_nuevo"]),
        ("crear_variante", &["proyecto", "variante_nueva"]),
        ("dimensionar", &["proyecto", "kwp"]),
        ("doctor", &["proyecto"]),
        ("anadir", &["proyecto", "elemento"]),
    ],
    ejecutar: Ejecutor::Simple(ejecutar_proyecto),
};

fn ejecutar_proyecto(paths: &Paths, args: &Value) -> Result<Value> {
    let valor = args.get("accion").and_then(Value::as_str).unwrap_or("");
    match valor {
        "crear" => lifecycle::crear_proyecto(paths, args),
        "eliminar" => lifecycle::eliminar_proyecto(paths, args),
        "renombrar" => lifecycle::renombrar_proyecto(paths, args),
        "crear_variante" => lifecycle::crear_variante(paths, args),
        "dimensionar" => lifecycle::dimensionar_variante(paths, args),
        "doctor" => projects::doctor::doctor(paths, args),
        "anadir" => projects::anadir::anadir(paths, args),
        "limpiar_temporales" => projects::temporales::limpiar_temporales(paths, args),
        _ => Err(Error::Invalid(format!(
            "Valor '{valor}' no soportado para proyecto"
        ))),
    }
}
