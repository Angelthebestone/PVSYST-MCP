//! GUI bridge: drive the PVsyst desktop application (`PVsyst.exe`) through the
//! Windows API — the only way to cover PVsyst features that have **no file or
//! CLI backing** (3D shading editor, informative I/V & table tools, import
//! wizards, dialogs...).
//!
//! This is black-box UI automation: it enumerates real Win32 windows/controls
//! (VCL forms are native windows), reads the UI Automation tree (`uia`) and
//! posts standard window messages to invoke them. It never reads, disassembles
//! or patches PVsyst code. Captures are PNG; physical clicks and captures need
//! an interactive desktop.

pub(crate) mod acciones;
pub(crate) mod base64;
pub(crate) mod campo_por_etiqueta;
pub(crate) mod campos;
pub(crate) mod captura;
pub(crate) mod clic_por_texto;
pub(crate) mod conduccion;
pub(crate) mod escritorio;
pub(crate) mod esperas;
pub(crate) mod ffi;
pub(crate) mod fila;
pub(crate) mod gdi;
pub(crate) mod menu_por_posicion;
pub(crate) mod modal;
pub(crate) mod ocr;
pub(crate) mod png;
pub(crate) mod proceso;
pub(crate) mod recuperacion;
pub(crate) mod registro_pvsyst;
pub(crate) mod resumen;
pub(crate) mod rotulos;
pub(crate) mod secuencia;
pub(crate) mod sesion_pvsyst;
pub(crate) mod sondeo;
pub(crate) mod teclado;
pub(crate) mod uia_vista;
pub(crate) mod ventanas;

pub(crate) use proceso::gui_running;

/// Lines of PVsyst's own log `que:mensajes` returns when `lineas` is absent.
const MENSAJES_POR_DEFECTO: usize = 50;

use crate::error::{Error, Result};
use serde_json::Value;

/// `observar` dispatcher. `que`: ventanas (default) | controles | leer |
/// capturar | uia | arbol | formulario | vista | esquema | mensajes | funciones | menus.
pub(crate) fn observar(paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    match args
        .get("que")
        .and_then(Value::as_str)
        .unwrap_or("ventanas")
    {
        "ventanas" => ventanas::ventanas(paths, args),
        "controles" => uia_vista::controles_con_uia(paths, args),
        "leer" => ventanas::leer(paths, args),
        "capturar" => captura::capturar(paths, args),
        "uia" => uia_vista::observar_uia(paths, args),
        "arbol" => uia_vista::observar_arbol(paths, args),
        "formulario" => ocr::formulario(paths, args),
        "funciones" => Ok(crate::gui_catalogo::listar(args)),
        "menus" => menu_por_posicion::listar(ventanas::hwnd_de(args, "ventana")?),
        "vista" => crate::vista_agente::vista(paths, args),
        "esquema" => crate::vista_agente::listar_esquemas(args),
        "mensajes" => {
            let limite = args
                .get("lineas")
                .and_then(Value::as_u64)
                .and_then(|v| usize::try_from(v).ok())
                .unwrap_or(MENSAJES_POR_DEFECTO);
            Ok(registro_pvsyst::ultimos_mensajes(limite))
        }
        other => Err(Error::Invalid(format!(
            "Unknown que '{other}' (ventanas|controles|leer|capturar|uia|arbol|formulario|vista|esquema|mensajes|funciones|menus)"
        ))),
    }
}

/// Every `accion` of `actuar`, in the order the tool schema lists them. The
/// error for an unknown one and the schema of `gui_actuar` both read this.
pub(crate) const ACCIONES: &[&str] = &[
    "click",
    "clic_fisico",
    "mensaje",
    "cerrar",
    "activar",
    "teclas",
    "fila",
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
    "cerrar_todo",
    "secuencia",
    "escena",
];

/// `actuar` dispatcher. `accion`: click | clic_fisico | clic_texto | mensaje |
/// cerrar | activar | teclas | fila | escribir | seleccionar | pagina | esperar_cierre |
/// lanzar | reiniciar | uia_invocar | esperar | paso | funcion | menu | campo | rellenar | cerrar_todo | secuencia.
pub(crate) fn actuar(paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    crate::adapters::uia::invalidar_cache(&paths.estado);
    match args
        .get("accion")
        .and_then(Value::as_str)
        .unwrap_or("click")
    {
        accion @ ("click" | "mensaje" | "cerrar" | "activar") => {
            acciones::invocar(paths, args).map_err(|e| Error::Gui(format!("{e} (accion {accion})")))
        }
        "clic_texto" => clic_por_texto::clic_texto(args),
        "clic_fisico" => {
            acciones::exigir_escritorio()?;
            acciones::invocar(paths, args)
        }
        "teclas" => teclado::teclas(args),
        "fila" => fila::fila(paths, args),
        "funcion" => crate::gui_catalogo::abrir(args),
        "menu" => crate::gui_catalogo::menu(args),
        "campo" => campo_por_etiqueta::campo(paths, args),
        "rellenar" => crate::vista_agente::rellenar(paths, args),
        "escena" => crate::escena_3d::pedir_escena(args),
        "escribir" => campos::escribir(paths, args),
        "seleccionar" => campos::seleccionar(paths, args),
        "pagina" => campos::pagina(paths, args),
        "esperar_cierre" => campos::esperar_cierre_accion(paths, args),
        "lanzar" => proceso::lanzar(paths, args),
        "cerrar_todo" => proceso::cerrar_todo(paths, args),
        "secuencia" => secuencia::secuencia(paths, args),
        "reiniciar" => recuperacion::reiniciar(paths, args),
        "uia_invocar" => uia_vista::uia_invocar(args),
        "esperar" => esperas::esperar(args),
        "paso" => uia_vista::paso(args),
        other => Err(Error::Invalid(format!(
            "Unknown accion '{other}' ({})",
            ACCIONES.join("|")
        ))),
    }
}
