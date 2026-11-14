//! PVsyst-native tools for raw CLI passthrough and Windows GUI automation.

use super::declaracion::{Anotaciones, Discriminador, Ejecutor, Herramienta, Sinonimos, TipoDato};
use crate::adapters::{cli, gui};
use crate::args::flag;
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::progreso::SeguimientoTarea;
use crate::projects::gui::lanzar;
use crate::vista_agente;
use serde_json::Value;

pub(crate) const TOOL_PVSYST_CLI: Herramienta = Herramienta {
    nombre: "pvsyst_cli",
    descripcion: crate::idioma::Texto {
        es: "Passthrough a cualquier comando de PVsystCLI. Los argumentos van en 'opciones'; cualquier otra clave se lista en ignorados y no se pasa. No confina rutas (una ruta relativa se resuelve contra el workspace) y no toca la licencia: lic-activate, lic-deactivate y lic-sync se rechazan.",
        en: "Passthrough to any PVsystCLI command. Its arguments go in 'opciones'; any other key is listed in ignorados and not passed. It does not confine paths (a relative path resolves against the workspace) and it never touches the licence: lic-activate, lic-deactivate and lic-sync are refused.",
    },
    anotaciones: Anotaciones {
        titulo: crate::idioma::Texto {
            es: "PVsystCLI",
            en: "PVsystCLI",
        },
        es_solo_lectura: false,
        es_destructiva: true,
        es_idempotente: false,
        // `version` checks for updates and `lic-sync` asks the licence server.
        usa_red: true,
    },
    propiedades: &[
        ("comando", TipoDato::String),
        ("opciones", TipoDato::ArrayOrObject),
        ("dry_run", TipoDato::Boolean),
        ("timeout_s", TipoDato::Integer),
        ("log_level", TipoDato::Integer),
    ],
    requeridos: &["comando"],
    discriminador: None,
    requeridos_por_accion: &[],
    consumidos_por_accion: &[],
    sinonimos: SINONIMOS_PVSYST_CLI,
    ejecutar: Ejecutor::Seguido(ejecutar_cli),
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
    descripcion: crate::idioma::Texto { es: "Observa la GUI de PVsyst: ventanas, controles, texto, captura PNG, formularios por OCR, vista de agente (campos con clave estable, sin OCR), menus y arbol UIA.", en: "Observes the PVsyst GUI: windows, controls, text, PNG capture, OCR forms, agent view (fields by stable key, no OCR), menus and UIA tree." },
    anotaciones: Anotaciones {
        titulo: crate::idioma::Texto { es: "Observar GUI", en: "Observe GUI" },
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
            "inventario",
            "elementos",
            "vista",
            "esquema",
            "mensajes",
            "funciones",
            "menus",
        ],
    }),
    requeridos_por_accion: &[],
    consumidos_por_accion: &[],
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
        ("elements", "elementos"),
        ("inventory", "inventario"),
        ("view", "vista"),
        ("schema", "esquema"),
        ("messages", "mensajes"),
        ("functions", "funciones"),
    ],
};

pub(crate) const TOOL_GUI_ACTUAR: Herramienta = Herramienta {
    nombre: "gui_actuar",
    descripcion: crate::idioma::Texto { es: "Actua sobre la GUI de PVsyst: clic, clic_texto (texto visible, por OCR), escribir, seleccionar, pestana, teclas, fila de una lista o arbol con geometria medida, menus por posicion, cerrar, lanzar, reiniciar, esperar a que aparezca una ventana (no es una pausa), rellenar un formulario por claves y validar, importar (mallas) o crear una escena 3D en el editor. reiniciar solo actua tras un dialogo de fallo de PVsyst (madExcept o aviso de inestabilidad): no reinicia una instancia sana.", en: "Acts on the PVsyst GUI: click, clic_texto (on-screen text, by OCR), type, select, tab page, keys, a row of a list or tree with measured geometry, menus by position, close, launch, restart, wait for a window to appear (not a sleep), fill a form by keys and validate, import (meshes) or build a 3D scene in the editor. reiniciar only applies after a PVsyst failure dialog (madExcept or the instability warning): it does not restart a healthy instance." },
    anotaciones: Anotaciones {
        titulo: crate::idioma::Texto { es: "Actuar en GUI", en: "Act on GUI" },
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
        ("cancelar", TipoDato::Boolean),
        ("pasos", TipoDato::Array),
        ("funcion", TipoDato::String),
        ("id", TipoDato::Integer),
        ("etiqueta", TipoDato::String),
        ("menu", TipoDato::Integer),
        ("item", TipoDato::Integer),
        ("subitem", TipoDato::Integer),
        ("dx", TipoDato::Integer),
        ("dy", TipoDato::Integer),
        ("entidad", TipoDato::String),
        ("valores", TipoDato::Object),
        ("simular", TipoDato::Boolean),
        ("escena", TipoDato::String),
        ("unidades", TipoDato::String),
        ("archivo", TipoDato::String),
        ("objetos", TipoDato::Array),
    ],
    requeridos: &[],
    discriminador: Some(Discriminador {
        campo: "accion",
        defecto: Some("click"),
        valores: gui::ACCIONES,
    }),
    requeridos_por_accion: &[
        ("escribir", &["texto"]),
        ("seleccionar", &["texto"]),
        ("pagina", &["indice"]),
        ("fila", &["ventana", "indice"]),
        ("esperar_cierre", &["ventana"]),
        ("uia_invocar", &["ventana", "operacion"]),
        ("funcion", &["funcion"]),
        ("menu", &["ventana"]),
        ("campo", &["etiqueta", "valor"]),
        ("rellenar", &["entidad", "valores"]),
        ("clic_texto", &["ventana", "texto"]),
        ("secuencia", &["pasos"]),
        ("escena", &["escena", "objetos"]),
        ("escena_leer", &["escena"]),
        ("escena_importar", &["archivo", "escena"]),
    ],
    consumidos_por_accion: &[],
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

fn ejecutar_cli(paths: &Paths, args: &Value, seguimiento: &SeguimientoTarea<'_>) -> Result<Value> {
    cli::ejecutar(paths, args, seguimiento)
}

// The agent view is a feature built on the GUI adapter, so its routes are
// resolved here and the adapter never depends on it.
fn ejecutar_observar(paths: &Paths, args: &Value) -> Result<Value> {
    match args.get("que").and_then(Value::as_str) {
        Some("vista") => vista_agente::vista(paths, args),
        Some("esquema") => vista_agente::listar_esquemas(args),
        _ => gui::observar(paths, args),
    }
}

/// `gui_actuar` rehearses with `simular`, never with `dry_run`: refuse the foreign
/// switch before dispatching so no action can have a side effect.
fn ejecutar_actuar(paths: &Paths, args: &Value) -> Result<Value> {
    let accion = TOOL_GUI_ACTUAR.accion(args);
    if flag(args, &["dry_run"]) {
        return Err(Error::Invalid(format!(
            "'dry_run' no existe en gui_actuar {accion}: esa acción no tiene ensayo y se ejecutaría de verdad. Quita 'dry_run'"
        )));
    }
    // PVsyst ignores a project on its command line: opening it is a project step.
    if lanzar::pide_proyecto(args) {
        return lanzar::lanzar_con_proyecto(paths, args);
    }
    match args.get("accion").and_then(Value::as_str) {
        Some("rellenar") => vista_agente::rellenar(paths, args),
        Some("campo") => gui::campo_por_etiqueta::campo(paths, args, vista_agente::vista),
        Some("secuencia") => gui::secuencia::secuencia(paths, args, ejecutar_actuar),
        _ => gui::actuar(paths, args),
    }
}
