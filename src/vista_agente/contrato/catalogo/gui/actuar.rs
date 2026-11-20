//! `gui_actuar`: every way of acting on a window, each with the call that follows it.

use super::super::{entrada, Entrada};
use super::pasos::{
    CAPTURAR, VER_CONTROLES, VER_ESQUEMA, VER_FORMULARIO, VER_FUNCIONES, VER_VENTANAS,
};
use crate::idioma::Texto;

pub(super) const CLICK: Entrada = entrada(
    "gui_actuar",
    "click",
    Texto {
        es: "Clic enviado al control.",
        en: "Click sent to the control.",
    },
)
.luego(&[VER_FORMULARIO, VER_VENTANAS]);

pub(super) const CLIC_FISICO: Entrada = entrada(
    "gui_actuar",
    "clic_fisico",
    Texto {
        es: "Clic físico enviado a la pantalla.",
        en: "Physical click sent to the screen.",
    },
)
.luego(&[VER_FORMULARIO, VER_VENTANAS]);

pub(super) const MENSAJE: Entrada = entrada(
    "gui_actuar",
    "mensaje",
    Texto {
        es: "Mensaje de Windows enviado a la ventana.",
        en: "Windows message sent to the window.",
    },
)
.luego(&[VER_CONTROLES, VER_VENTANAS]);

pub(super) const CERRAR: Entrada = entrada(
    "gui_actuar",
    "cerrar",
    Texto {
        es: "Ventana cerrada. Con descartar_cambios:true responde «Sí» a «¿Quiere cancelar las modificaciones?» en la misma llamada, y solo a esa pregunta.",
        en: "Window closed. With descartar_cambios:true it answers «Yes» to «Do you want to cancel the modifications?» in the same call, and only to that question.",
    },
)
.luego(&[VER_VENTANAS]);

pub(super) const ACTIVAR: Entrada = entrada(
    "gui_actuar",
    "activar",
    Texto {
        es: "Ventana traída al primer plano.",
        en: "Window brought to the foreground.",
    },
)
.luego(&[VER_FORMULARIO, VER_VENTANAS]);

pub(super) const TECLAS: Entrada = entrada(
    "gui_actuar",
    "teclas",
    Texto {
        es: "Teclas enviadas a la ventana en primer plano.",
        en: "Keys sent to the foreground window.",
    },
)
.luego(&[CAPTURAR, VER_VENTANAS]);

pub(super) const ESCRIBIR: Entrada = entrada(
    "gui_actuar",
    "escribir",
    Texto {
        es: "Texto escrito en el control.",
        en: "Text written into the control.",
    },
)
.luego(&[VER_FORMULARIO, VER_VENTANAS]);

pub(super) const SELECCIONAR: Entrada = entrada(
    "gui_actuar",
    "seleccionar",
    Texto {
        es: "Elemento de la lista seleccionado.",
        en: "List item selected.",
    },
)
.luego(&[VER_FORMULARIO, VER_VENTANAS]);

pub(super) const PAGINA: Entrada = entrada(
    "gui_actuar",
    "pagina",
    Texto {
        es: "Pestaña del formulario cambiada.",
        en: "Form tab changed.",
    },
)
.luego(&[VER_FORMULARIO, VER_VENTANAS]);

pub(super) const ESPERAR_CIERRE: Entrada = entrada(
    "gui_actuar",
    "esperar_cierre",
    Texto {
        es: "Cierre de la ventana esperado.",
        en: "Window close awaited.",
    },
)
.luego(&[VER_VENTANAS]);

pub(super) const LANZAR: Entrada = entrada(
    "gui_actuar",
    "lanzar",
    Texto {
        es: "PVsyst lanzado.",
        en: "PVsyst launched.",
    },
)
.luego(&[VER_VENTANAS, VER_FUNCIONES]);

pub(super) const UIA_INVOCAR: Entrada = entrada(
    "gui_actuar",
    "uia_invocar",
    Texto {
        es: "Elemento de UI Automation invocado.",
        en: "UI Automation element invoked.",
    },
)
.luego(&[VER_FORMULARIO, VER_VENTANAS]);

pub(super) const ESPERAR: Entrada = entrada(
    "gui_actuar",
    "esperar",
    Texto {
        es: "Espera completada.",
        en: "Wait completed.",
    },
)
.luego(&[VER_VENTANAS]);

pub(super) const PASO: Entrada = entrada(
    "gui_actuar",
    "paso",
    Texto {
        es: "Paso de GUI ejecutado.",
        en: "GUI step run.",
    },
)
.luego(&[VER_VENTANAS]);

pub(super) const FUNCION: Entrada = entrada(
    "gui_actuar",
    "funcion",
    Texto {
        es: "Función de la GUI ejecutada.",
        en: "GUI function run.",
    },
)
.luego(&[VER_VENTANAS, VER_ESQUEMA]);

pub(super) const MENU: Entrada = entrada(
    "gui_actuar",
    "menu",
    Texto {
        es: "Entrada de menú pulsada.",
        en: "Menu entry pressed.",
    },
)
.luego(&[VER_VENTANAS, VER_ESQUEMA]);

pub(super) const CAMPO: Entrada = entrada(
    "gui_actuar",
    "campo",
    Texto {
        es: "Campo del formulario escrito por etiqueta.",
        en: "Form field written by label.",
    },
)
.luego(&[VER_FORMULARIO, VER_VENTANAS]);

pub(super) const RELLENAR: Entrada = entrada(
    "gui_actuar",
    "rellenar",
    Texto {
        es: "Formulario rellenado por claves.",
        en: "Form filled by key.",
    },
)
.luego(&[VER_FORMULARIO, VER_ESQUEMA]);

pub(super) const CLIC_TEXTO: Entrada = entrada(
    "gui_actuar",
    "clic_texto",
    Texto {
        es: "Clic en el texto visible de la ventana.",
        en: "Click on the visible text of the window.",
    },
)
.luego(&[VER_FORMULARIO, VER_VENTANAS]);

pub(super) const FILA: Entrada = entrada(
    "gui_actuar",
    "fila",
    Texto {
        es: "Fila de la lista o del árbol seleccionada y comprobada por su banda resaltada.",
        en: "Row of the list or tree selected and checked by its highlighted band.",
    },
)
.luego(&[VER_FORMULARIO, VER_VENTANAS]);

pub(super) const REINICIAR: Entrada = entrada(
    "gui_actuar",
    "reiniciar",
    Texto {
        es: "PVsyst reiniciado.",
        en: "PVsyst restarted.",
    },
)
.luego(&[VER_VENTANAS]);

pub(super) const CERRAR_TODO: Entrada = entrada(
    "gui_actuar",
    "cerrar_todo",
    Texto {
        es: "Ventanas de PVsyst cerradas (requiere confirmar:true).",
        en: "PVsyst windows closed (requires confirmar:true).",
    },
)
.luego(&[VER_VENTANAS]);

pub(super) const SECUENCIA: Entrada = entrada(
    "gui_actuar",
    "secuencia",
    Texto {
        es: "Secuencia de pasos de GUI ejecutada.",
        en: "Sequence of GUI steps run.",
    },
)
.luego(&[VER_VENTANAS, VER_ESQUEMA]);
