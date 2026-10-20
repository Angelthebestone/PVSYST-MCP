//! Expert tools for raw CLI passthrough and Windows GUI automation.

use super::declaracion::{Acceso, Anotaciones, Discriminador, Ejecutor, Herramienta, TipoDato};
use crate::adapters::{cli, gui};
use crate::config::Paths;
use crate::error::Result;
use serde_json::Value;

pub(crate) const TOOL_PVSYST_CLI: Herramienta = Herramienta {
    nombre: "pvsyst_cli",
    descripcion: "Passthrough a cualquier comando de PVsystCLI (modo experto). / EN: Passthrough to any PVsystCLI command (expert mode).",
    acceso: Acceso::Experto,
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
    descripcion: "Observa la GUI de PVsyst: ventanas, controles, texto, captura PNG, arbol UIA (modo experto). / EN: Observes the PVsyst GUI: windows, controls, text, PNG capture, UIA tree (expert mode).",
    acceso: Acceso::Experto,
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
        ("compacto", TipoDato::Boolean),
        ("completo", TipoDato::Boolean),
        ("visibles", TipoDato::Boolean),
        ("todos", TipoDato::Boolean),
        ("incluir_uia", TipoDato::Boolean),
        ("profundidad", TipoDato::Integer),
        ("max_nodos", TipoDato::Integer),
        ("ruta", TipoDato::String),
        ("imagen", TipoDato::Boolean),
        ("recorte", TipoDato::Boolean),
        ("permitir_fuera_del_workspace", TipoDato::Boolean),
    ],
    requeridos: &[],
    discriminador: Some(Discriminador {
        campo: "que",
        defecto: Some("ventanas"),
        valores: &["ventanas", "controles", "leer", "capturar", "uia", "arbol"],
    }),
    requeridos_por_accion: &[],
    ejecutar: Ejecutor::Simple(ejecutar_observar),
};

pub(crate) const TOOL_GUI_ACTUAR: Herramienta = Herramienta {
    nombre: "gui_actuar",
    descripcion: "Actua sobre la GUI de PVsyst: clic, teclas, cerrar, lanzar, UIA y esperas (modo experto). / EN: Acts on the PVsyst GUI: click, keys, close, launch, UIA and waits (expert mode).",
    acceso: Acceso::Experto,
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
        ("mensaje", TipoDato::Integer),
        ("wparam", TipoDato::Integer),
        ("lparam", TipoDato::Integer),
        ("teclas", TipoDato::String),
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
    ],
    requeridos: &[],
    discriminador: Some(Discriminador {
        campo: "accion",
        defecto: None,
        valores: &[
            "click",
            "clic_fisico",
            "mensaje",
            "cerrar",
            "activar",
            "teclas",
            "lanzar",
            "uia_invocar",
            "esperar",
            "paso",
        ],
    }),
    requeridos_por_accion: &[],
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
