//! `gui_observar`: every way of reading the GUI, each with the call that follows it.

use super::super::{entrada, Entrada};
use super::pasos::{TRAS_VISTA, VER_ESQUEMA, VER_FUNCIONES, VER_VENTANAS};
use crate::idioma::Texto;

pub(super) const VENTANAS: Entrada = entrada(
    "gui_observar",
    "ventanas",
    Texto {
        es: "Ventanas de PVsyst listadas.",
        en: "PVsyst windows listed.",
    },
)
.luego(&[VER_ESQUEMA, VER_FUNCIONES]);

pub(super) const CONTROLES: Entrada = entrada(
    "gui_observar",
    "controles",
    Texto {
        es: "Controles de la ventana listados.",
        en: "Window controls listed.",
    },
)
.luego(TRAS_VISTA);

pub(super) const LEER: Entrada = entrada(
    "gui_observar",
    "leer",
    Texto {
        es: "Texto de la ventana leído.",
        en: "Window text read.",
    },
)
.luego(TRAS_VISTA);

pub(super) const CAPTURAR: Entrada = entrada(
    "gui_observar",
    "capturar",
    Texto {
        es: "Captura de la ventana obtenida.",
        en: "Window capture obtained.",
    },
)
.luego(TRAS_VISTA);

pub(super) const UIA: Entrada = entrada(
    "gui_observar",
    "uia",
    Texto {
        es: "Árbol de UI Automation leído.",
        en: "UI Automation tree read.",
    },
)
.luego(TRAS_VISTA);

pub(super) const ARBOL: Entrada = entrada(
    "gui_observar",
    "arbol",
    Texto {
        es: "Árbol de controles leído.",
        en: "Control tree read.",
    },
)
.luego(TRAS_VISTA);

pub(super) const FORMULARIO: Entrada = entrada(
    "gui_observar",
    "formulario",
    Texto {
        es: "Formulario leído por OCR.",
        en: "Form read by OCR.",
    },
)
.luego(TRAS_VISTA);

pub(super) const INVENTARIO: Entrada = entrada(
    "gui_observar",
    "inventario",
    Texto {
        es: "Inventario de los controles del formulario (ocultos incluidos).",
        en: "Inventory of the form's controls (hidden ones included).",
    },
)
.luego(TRAS_VISTA);

pub(super) const ELEMENTOS: Entrada = entrada(
    "gui_observar",
    "elementos",
    Texto {
        es: "Elementos de un control (filas, celdas, items) listados.",
        en: "Elements of a control (rows, cells, items) listed.",
    },
)
.luego(TRAS_VISTA);

pub(super) const VISTA: Entrada = entrada(
    "gui_observar",
    "vista",
    Texto {
        es: "Formulario leído por claves estables.",
        en: "Form read by stable keys.",
    },
)
.luego(TRAS_VISTA);

pub(super) const ESQUEMA: Entrada = entrada(
    "gui_observar",
    "esquema",
    Texto {
        es: "Esquemas de formulario con sus claves.",
        en: "Form schemas with their keys.",
    },
)
.luego(&[VER_VENTANAS, VER_FUNCIONES]);

pub(super) const MENSAJES: Entrada = entrada(
    "gui_observar",
    "mensajes",
    Texto {
        es: "Mensajes de la ventana leídos.",
        en: "Window messages read.",
    },
)
.luego(TRAS_VISTA);

pub(super) const FUNCIONES: Entrada = entrada(
    "gui_observar",
    "funciones",
    Texto {
        es: "Funciones de la GUI listadas.",
        en: "GUI functions listed.",
    },
)
.luego(&[VER_VENTANAS, VER_ESQUEMA]);

pub(super) const MENUS: Entrada = entrada(
    "gui_observar",
    "menus",
    Texto {
        es: "Menús de la ventana listados.",
        en: "Window menus listed.",
    },
)
.luego(&[VER_FUNCIONES, VER_VENTANAS]);

pub(super) const IDIOMA: Entrada = entrada(
    "gui_observar",
    "idioma",
    Texto {
        es: "Idioma de las ventanas de PVsyst leído.",
        en: "Language of PVsyst's windows read.",
    },
)
.luego(&[VER_VENTANAS, VER_FUNCIONES]);
