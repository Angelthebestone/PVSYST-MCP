//! Tool for component database management: creation, deletion, import and export.

use super::declaracion::{Anotaciones, Discriminador, Ejecutor, Herramienta, Sinonimos, TipoDato};
use crate::admin;
use crate::components;
use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::Value;

pub(crate) const TOOL_COMPONENTE: Herramienta = Herramienta {
    nombre: "componente",
    descripcion: "Base de datos de componentes: crear, borrar, importar, exportar o crear desde ficha tecnica (PVsyst ajusta el modelo en su ventana). importar: origen (archivo), nombre_nuevo opcional; 'destino' no aplica. / EN: Component database: create, delete, import, export or create from a datasheet (PVsyst fits the model in its own window). import: origen (file), optional nombre_nuevo; 'destino' does not apply.",
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
        ("sobrescribir", TipoDato::Boolean),
        ("via", TipoDato::String),
    ],
    requeridos: &[],
    discriminador: Some(Discriminador {
        campo: "accion",
        defecto: None,
        valores: &[
            "crear",
            "editar",
            "borrar",
            "importar",
            "exportar",
            "crear_desde_ficha",
        ],
    }),
    requeridos_por_accion: &[
        ("crear", &["nombre_nuevo", "plantilla", "cambios"]),
        ("editar", &["componente"]),
        ("crear_desde_ficha", &["tipo", "nombre_nuevo", "datos"]),
    ],
    sinonimos: SINONIMOS_COMPONENTE,
    ejecutar: Ejecutor::Simple(ejecutar_componente),
};

const SINONIMOS_COMPONENTE: Sinonimos = Sinonimos {
    argumentos: &[
        ("action", "accion"),
        ("file", "componente"),
        ("component", "componente"),
        ("new_name", "nombre_nuevo"),
        ("template", "plantilla"),
        ("changes", "cambios"),
        ("type", "tipo"),
        ("source", "origen"),
        ("target", "destino"),
        ("destination", "destino"),
        ("data", "datos"),
    ],
    valores: &[
        ("create", "crear"),
        ("edit", "editar"),
        ("delete", "borrar"),
        ("remove", "borrar"),
        ("import", "importar"),
        ("export", "exportar"),
        ("datasheet", "crear_desde_ficha"),
        ("create_from_datasheet", "crear_desde_ficha"),
    ],
};

fn ejecutar_componente(paths: &Paths, args: &Value) -> Result<Value> {
    let valor = TOOL_COMPONENTE.accion(args);
    match valor {
        "crear" => components::crear_componente(paths, args),
        "editar" => components::editar_componente(paths, args),
        "borrar" => components::borrar_componente(paths, args),
        "importar" => admin::importar_componente(paths, args),
        "exportar" => admin::exportar_componente(paths, args),
        "crear_desde_ficha" => components::ficha::crear_desde_ficha(paths, args),
        _ => Err(Error::Invalid(format!(
            "Valor '{valor}' no soportado para componente"
        ))),
    }
}
