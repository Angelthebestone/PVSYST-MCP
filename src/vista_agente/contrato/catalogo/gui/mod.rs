//! `pvsyst_cli`, `gui_observar` and `gui_actuar`, grouped by what each tool does
//! with the GUI: read it, act on it, or hand the command line through.

mod actuar;
mod cli;
mod escena;
mod observar;
mod pasos;

use super::Entrada;

/// Every entry, in the order the catalog lists them.
pub(super) const ENTRADAS: &[Entrada] = &[
    cli::PVSYST_CLI,
    observar::VENTANAS,
    observar::CONTROLES,
    observar::LEER,
    observar::CAPTURAR,
    observar::UIA,
    observar::ARBOL,
    observar::FORMULARIO,
    observar::INVENTARIO,
    observar::ELEMENTOS,
    observar::VISTA,
    observar::ESQUEMA,
    observar::MENSAJES,
    observar::FUNCIONES,
    observar::MENUS,
    observar::IDIOMA,
    actuar::CLICK,
    actuar::CLIC_FISICO,
    actuar::MENSAJE,
    actuar::CERRAR,
    actuar::ACTIVAR,
    actuar::TECLAS,
    actuar::ESCRIBIR,
    actuar::SELECCIONAR,
    actuar::PAGINA,
    actuar::ESPERAR_CIERRE,
    actuar::LANZAR,
    actuar::UIA_INVOCAR,
    actuar::ESPERAR,
    actuar::PASO,
    actuar::FUNCION,
    actuar::MENU,
    actuar::CAMPO,
    actuar::RELLENAR,
    actuar::CLIC_TEXTO,
    actuar::FILA,
    actuar::REINICIAR,
    actuar::CERRAR_TODO,
    escena::ESCENA,
    escena::ESCENA_LEER,
    escena::ESCENA_IMPORTAR,
    actuar::SECUENCIA,
];
