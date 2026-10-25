//! PVsyst-native tools for raw CLI passthrough and Windows GUI automation.

use super::declaracion::{Anotaciones, Discriminador, Ejecutor, Herramienta, Sinonimos, TipoDato};
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
    sinonimos: SINONIMOS_PVSYST_CLI,
    ejecutar: Ejecutor::Simple(ejecutar_cli),
};

const SINONIMOS_PVSYST_CLI: Sinonimos = Sinonimos {
    argumentos: &[
        ("command", "comando"),
        ("options", "opciones"),
        ("args", "opciones"),
    ],
    valores: &[],
};

pub(crate) const TOOL_GUI_OBSERVAR: Herramienta = Herramienta {
    nombre: "gui_observar",
    descripcion: "Observa la GUI de PVsyst: ventanas, controles, texto, captura PNG, formularios por OCR, vista de agente (campos con clave estable, sin OCR), menus y arbol UIA. / EN: Observes the PVsyst GUI: windows, controls, text, PNG capture, OCR forms, agent view (fields by stable key, no OCR), menus and UIA tree.",
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
        ("entidad", TipoDato::String),
        ("ocr", TipoDato::Boolean),
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
            "vista",
            "esquema",
            "mensajes",
            "funciones",
            "menus",
        ],
    }),
    requeridos_por_accion: &[],
    sinonimos: SINONIMOS_GUI_OBSERVAR,
    ejecutar: Ejecutor::Simple(ejecutar_observar),
};

const SINONIMOS_GUI_OBSERVAR: Sinonimos = Sinonimos {
    argumentos: &[
        ("what", "que"),
        ("window", "ventana"),
        ("process", "proceso"),
        ("lines", "lineas"),
        ("path", "ruta"),
        ("image", "imagen"),
        ("function", "funcion"),
        ("entity", "entidad"),
    ],
    valores: &[
        ("windows", "ventanas"),
        ("controls", "controles"),
        ("read", "leer"),
        ("capture", "capturar"),
        ("tree", "arbol"),
        ("form", "formulario"),
        ("view", "vista"),
        ("schema", "esquema"),
        ("messages", "mensajes"),
        ("functions", "funciones"),
    ],
};

pub(crate) const TOOL_GUI_ACTUAR: Herramienta = Herramienta {
    nombre: "gui_actuar",
    descripcion: "Actua sobre la GUI de PVsyst: clic (tambien por texto visible), escribir, seleccionar, pestana, teclas, menus por posicion, cerrar, lanzar, reiniciar, esperas y rellenar un formulario por claves. / EN: Acts on the PVsyst GUI: click (also by on-screen text), type, select, tab page, keys, menus by position, close, launch, restart, waits and fill a form by keys.",
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
        ("cerrar_demo", TipoDato::Boolean),
        ("funcion", TipoDato::String),
        ("id", TipoDato::Integer),
        ("etiqueta", TipoDato::String),
        ("menu", TipoDato::Integer),
        ("item", TipoDato::Integer),
        ("dx", TipoDato::Integer),
        ("dy", TipoDato::Integer),
        ("entidad", TipoDato::String),
        ("valores", TipoDato::Object),
        ("simular", TipoDato::Boolean),
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
            "rellenar",
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
        ("rellenar", &["entidad", "valores"]),
        ("clic_texto", &["ventana", "texto"]),
    ],
    sinonimos: SINONIMOS_GUI_ACTUAR,
    ejecutar: Ejecutor::Simple(ejecutar_actuar),
};

const SINONIMOS_GUI_ACTUAR: Sinonimos = Sinonimos {
    argumentos: &[
        ("action", "accion"),
        ("window", "ventana"),
        ("process", "proceso"),
        ("message", "mensaje"),
        ("keys", "teclas"),
        ("text", "texto"),
        ("field", "campo"),
        ("index", "indice"),
        ("class", "clase"),
        ("title", "titulo"),
        ("name", "nombre"),
        ("role", "rol"),
        ("operation", "operacion"),
        ("value", "valor"),
        ("project", "proyecto"),
        ("function", "funcion"),
        ("label", "etiqueta"),
    ],
    valores: &[
        ("press", "click"),
        ("type", "escribir"),
        ("write", "escribir"),
        ("select", "seleccionar"),
        ("keys", "teclas"),
        ("close_dialog", "cerrar"),
        ("wait", "esperar"),
        ("fill", "rellenar"),
    ],
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
