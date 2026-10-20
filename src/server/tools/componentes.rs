//! Tool for component database management: creation, deletion, import and export.

use super::declaracion::{Acceso, Anotaciones, Discriminador, Herramienta, TipoDato};
use crate::admin;
use crate::components;
use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::Value;

pub(crate) const TOOL_COMPONENTE: Herramienta = Herramienta {
    nombre: "componente",
    descripcion: "Base de datos de componentes: crear, borrar, importar, exportar o crear desde ficha tecnica. / EN: Component database: create, delete, import, export or create from a datasheet.",
    acceso: Acceso::Normal,
    anotaciones: Anotaciones {
        titulo: "Componente / Component",
        es_solo_lectura: false,
        es_destructiva: true,
        es_idempotente: false,
        usa_red: false,
    },
    propiedades: &[
        ("nombre_nuevo", TipoDato::String),
        ("plantilla", TipoDato::String),
        ("cambios", TipoDato::Object),
        ("tipo", TipoDato::String),
        ("componente", TipoDato::String),
        ("origen", TipoDato::String),
        ("destino", TipoDato::String),
        ("datos", TipoDato::Object),
        ("workspace", TipoDato::String),
        ("permitir_fuera_del_workspace", TipoDato::Boolean),
    ],
    requeridos: &[],
    discriminador: Some(Discriminador {
        campo: "accion",
        defecto: None,
        valores: &[
            "crear",
            "borrar",
            "importar",
            "exportar",
            "crear_desde_ficha",
        ],
    }),
    requeridos_por_accion: &[
        ("crear", &["nombre_nuevo", "plantilla", "cambios"]),
        ("crear_desde_ficha", &["tipo", "nombre_nuevo", "datos"]),
    ],
    ejecutar: ejecutar_componente,
};

fn ejecutar_componente(paths: &Paths, args: &Value) -> Result<Value> {
    let valor = args.get("accion").and_then(Value::as_str).unwrap_or("");
    match valor {
        "crear" => components::crear_componente(paths, args),
        "borrar" => components::borrar_componente(paths, args),
        "importar" => admin::importar_componente(paths, args),
        "exportar" => admin::exportar_componente(paths, args),
        "crear_desde_ficha" => components::ficha::crear_desde_ficha(paths, args),
        _ => Err(Error::Invalid(format!(
            "Valor '{valor}' no soportado para componente"
        ))),
    }
}
