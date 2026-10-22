//! PVsyst-native tools for raw CLI passthrough and Windows GUI automation.

use super::declaracion::{Anotaciones, Discriminador, Ejecutor, Herramienta, TipoDato};
use crate::adapters::{cli, gui};
use crate::config::Paths;
use crate::error::Result;
use serde_json::Value;

pub(crate) const TOOL_PVSYST_CLI: Herramienta = Herramienta {
    nombre: "pvsyst_cli",
    descripcion:
        "Passthrough a cualquier comando de PVsystCLI. / EN: Passthrough to any PVsystCLI command.",
    anotaciones: Anotaciones {
        titulo: "PVsystCLI",
        es_solo_lectura: false,
        es_destructiva: true,
        es_idempotente: false,
        usa_red: false,
    },
    propiedades: &[
        ("comando", TipoDato::String),
        ("opciones", TipoDato::ArrayOrObject),
        ("dry_run", TipoDato::Boolean),
        ("timeout_s", TipoDato::Integer),
    ],
    requeridos: &["comando"],
    discriminador: None,
    requeridos_por_accion: &[],
    ejecutar: Ejecutor::Simple(ejecutar_cli),
};

pub(crate) const TOOL_GUI_OBSERVAR: Herramienta = Herramienta {
    nombre: "gui_observar",
    descripcion: "Observa la GUI de PVsyst: ventanas, controles, texto, captura PNG, formularios por OCR, menus y arbol UIA. / EN: Observes the PVsyst GUI: windows, controls, text, PNG capture, OCR forms, menus and UIA tree.",
    anotaciones: Anotaciones {
        titulo: "Observar GUI / Observe GUI",
        es_solo_lectura: true,
        es_destructiva: false,
        es_idempotente: true,
        usa_red: false,
    },
    propiedades: &[
        ("ventana", TipoDato::String),
        ("proceso", TipoDato::String),
        ("pids", TipoDato::Array),
        ("compacto", TipoDato::Boolean),
        ("completo", TipoDato::Boolean),
        ("visibles", TipoDato::Boolean),
        ("todos", TipoDato::Boolean),
        ("incluir_uia", TipoDato::Boolean),
        ("profundidad", TipoDato::Integer),
        ("max_nodos", TipoDato::Integer),
        ("lineas", TipoDato::Integer),
        ("ruta", TipoDato::String),
        ("imagen", TipoDato::Boolean),
        ("recorte", TipoDato::Boolean),
        ("permitir_fuera_del_workspace", TipoDato::Boolean),
        ("funcion", TipoDato::String),
    ],
    requeridos: &[],
    discriminador: Some(Discriminador {
        campo: "que",
        defecto: Some("ventanas"),
        valores: &[
            "ventanas",
            "controles",
            "leer",
            "capturar",
            "uia",
            "arbol",
            "formulario",
            "mensajes",
            "funciones",
            "menus",
        ],
    }),
    requeridos_por_accion: &[],
    ejecutar: Ejecutor::Simple(ejecutar_observar),
};

pub(crate) const TOOL_GUI_ACTUAR: Herramienta = Herramienta {
    nombre: "gui_actuar",
    descripcion: "Actua sobre la GUI de PVsyst: clic (tambien por texto visible), escribir, seleccionar, pestana, teclas, menus por posicion, cerrar, lanzar, reiniciar y esperas. / EN: Acts on the PVsyst GUI: click (also by on-screen text), type, select, tab page, keys, menus by position, close, launch, restart and waits.",
    anotaciones: Anotaciones {
        titulo: "Actuar en GUI / Act on GUI",
        es_solo_lectura: false,
        es_destructiva: true,
        es_idempotente: false,
        usa_red: false,
    },
    propiedades: &[
        ("ventana", TipoDato::String),
        ("proceso", TipoDato::String),
        ("pids", TipoDato::Array),
        ("mensaje", TipoDato::Integer),
        ("wparam", TipoDato::Integer),
        ("lparam", TipoDato::Integer),
        ("teclas", TipoDato::String),
        ("texto", TipoDato::String),
        ("campo", TipoDato::Object),
        ("indice", TipoDato::Integer),
        ("clase", TipoDato::String),
        ("titulo", TipoDato::String),
        ("timeout_ms", TipoDato::Integer),
        ("verificar", TipoDato::Object),
        ("objetivo", TipoDato::Object),
        ("nombre", TipoDato::String),
        ("rol", TipoDato::String),
        ("operacion", TipoDato::String),
        ("valor", TipoDato::String),
        ("proyecto", TipoDato::String),
        ("esperar_segundos", TipoDato::Integer),
        ("permitir_fuera_del_workspace", TipoDato::Boolean),
        ("funcion", TipoDato::String),
        ("id", TipoDato::Integer),
        ("etiqueta", TipoDato::String),
        ("menu", TipoDato::Integer),
        ("item", TipoDato::Integer),
        ("dx", TipoDato::Integer),
        ("dy", TipoDato::Integer),
    ],
    requeridos: &[],
    discriminador: Some(Discriminador {
        campo: "accion",
        defecto: Some("click"),
        valores: &[
            "click",
            "clic_fisico",
            "mensaje",
            "cerrar",
            "activar",
            "teclas",
            "escribir",
            "seleccionar",
            "pagina",
            "esperar_cierre",
            "lanzar",
            "uia_invocar",
            "esperar",
            "paso",
            "funcion",
            "menu",
            "campo",
            "clic_texto",
            "reiniciar",
        ],
    }),
    requeridos_por_accion: &[
        ("escribir", &["texto"]),
        ("seleccionar", &["texto"]),
        ("pagina", &["indice"]),
        ("esperar_cierre", &["ventana"]),
        ("funcion", &["funcion"]),
        ("menu", &["ventana"]),
        ("campo", &["etiqueta", "valor"]),
        ("clic_texto", &["ventana", "texto"]),
    ],
    ejecutar: Ejecutor::Simple(ejecutar_actuar),
};

fn ejecutar_cli(paths: &Paths, args: &Value) -> Result<Value> {
    cli::ejecutar(paths, args)
}

fn ejecutar_observar(paths: &Paths, args: &Value) -> Result<Value> {
    gui::observar(paths, args)
}

fn ejecutar_actuar(paths: &Paths, args: &Value) -> Result<Value> {
    gui::actuar(paths, args)
}
