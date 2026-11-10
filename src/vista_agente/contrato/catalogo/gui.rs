//! `pvsyst_cli`, `gui_observar` and `gui_actuar`.

use super::{entrada, paso, Entrada, Paso, DE_VENTANA, DIAGNOSTICO};
use crate::idioma::Texto;

const VER_ENTORNO: Paso = DIAGNOSTICO;
const VER_VENTANAS: Paso = paso(
    "gui_observar",
    Texto {
        es: "Ver las ventanas de PVsyst",
        en: "See the PVsyst windows",
    },
)
.con(&[("que", "ventanas")]);
const VER_ESQUEMA: Paso = paso(
    "gui_observar",
    Texto {
        es: "Ver los formularios con vista de agente",
        en: "See the forms with an agent view",
    },
)
.con(&[("que", "esquema")]);
const VER_FUNCIONES: Paso = paso(
    "gui_observar",
    Texto {
        es: "Ver las funciones de la GUI",
        en: "See the GUI functions",
    },
)
.con(&[("que", "funciones")]);
const VER_FORMULARIO: Paso = paso(
    "gui_observar",
    Texto {
        es: "Leer el formulario abierto por claves",
        en: "Read the open form by key",
    },
)
.con(&[("que", "vista")]);
const VER_CONTROLES: Paso = paso(
    "gui_observar",
    Texto {
        es: "Ver los controles de la ventana",
        en: "See the window controls",
    },
)
.con(&[("que", "controles")])
.tomando(DE_VENTANA);
const CAPTURAR: Paso = paso(
    "gui_observar",
    Texto {
        es: "Capturar la ventana",
        en: "Capture the window",
    },
)
.con(&[("que", "capturar")])
.tomando(DE_VENTANA);

/// What follows a read of the GUI: the forms with a schema, or the open windows.
const TRAS_VISTA: &[Paso] = &[VER_ESQUEMA, VER_VENTANAS];

pub(super) const ENTRADAS: &[Entrada] = &[
    entrada(
        "pvsyst_cli",
        "",
        Texto {
            es: "Comando de PVsystCLI ejecutado.",
            en: "PVsystCLI command run.",
        },
    )
    .luego(&[VER_ENTORNO]),
    entrada(
        "gui_observar",
        "ventanas",
        Texto {
            es: "Ventanas de PVsyst listadas.",
            en: "PVsyst windows listed.",
        },
    )
    .luego(&[VER_ESQUEMA, VER_FUNCIONES]),
    entrada(
        "gui_observar",
        "controles",
        Texto {
            es: "Controles de la ventana listados.",
            en: "Window controls listed.",
        },
    )
    .luego(TRAS_VISTA),
    entrada(
        "gui_observar",
        "leer",
        Texto {
            es: "Texto de la ventana leído.",
            en: "Window text read.",
        },
    )
    .luego(TRAS_VISTA),
    entrada(
        "gui_observar",
        "capturar",
        Texto {
            es: "Captura de la ventana obtenida.",
            en: "Window capture obtained.",
        },
    )
    .luego(TRAS_VISTA),
    entrada(
        "gui_observar",
        "uia",
        Texto {
            es: "Árbol de UI Automation leído.",
            en: "UI Automation tree read.",
        },
    )
    .luego(TRAS_VISTA),
    entrada(
        "gui_observar",
        "arbol",
        Texto {
            es: "Árbol de controles leído.",
            en: "Control tree read.",
        },
    )
    .luego(TRAS_VISTA),
    entrada(
        "gui_observar",
        "formulario",
        Texto {
            es: "Formulario leído por OCR.",
            en: "Form read by OCR.",
        },
    )
    .luego(TRAS_VISTA),
    entrada(
        "gui_observar",
        "vista",
        Texto {
            es: "Formulario leído por claves estables.",
            en: "Form read by stable keys.",
        },
    )
    .luego(TRAS_VISTA),
    entrada(
        "gui_observar",
        "esquema",
        Texto {
            es: "Esquemas de formulario con sus claves.",
            en: "Form schemas with their keys.",
        },
    )
    .luego(&[VER_VENTANAS, VER_FUNCIONES]),
    entrada(
        "gui_observar",
        "mensajes",
        Texto {
            es: "Mensajes de la ventana leídos.",
            en: "Window messages read.",
        },
    )
    .luego(TRAS_VISTA),
    entrada(
        "gui_observar",
        "funciones",
        Texto {
            es: "Funciones de la GUI listadas.",
            en: "GUI functions listed.",
        },
    )
    .luego(&[VER_VENTANAS, VER_ESQUEMA]),
    entrada(
        "gui_observar",
        "menus",
        Texto {
            es: "Menús de la ventana listados.",
            en: "Window menus listed.",
        },
    )
    .luego(&[VER_FUNCIONES, VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "click",
        Texto {
            es: "Clic enviado al control.",
            en: "Click sent to the control.",
        },
    )
    .luego(&[VER_FORMULARIO, VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "clic_fisico",
        Texto {
            es: "Clic físico enviado a la pantalla.",
            en: "Physical click sent to the screen.",
        },
    )
    .luego(&[VER_FORMULARIO, VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "mensaje",
        Texto {
            es: "Mensaje de Windows enviado a la ventana.",
            en: "Windows message sent to the window.",
        },
    )
    .luego(&[VER_CONTROLES, VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "cerrar",
        Texto {
            es: "Ventana cerrada.",
            en: "Window closed.",
        },
    )
    .luego(&[VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "activar",
        Texto {
            es: "Ventana traída al primer plano.",
            en: "Window brought to the foreground.",
        },
    )
    .luego(&[VER_FORMULARIO, VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "teclas",
        Texto {
            es: "Teclas enviadas a la ventana en primer plano.",
            en: "Keys sent to the foreground window.",
        },
    )
    .luego(&[CAPTURAR, VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "escribir",
        Texto {
            es: "Texto escrito en el control.",
            en: "Text written into the control.",
        },
    )
    .luego(&[VER_FORMULARIO, VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "seleccionar",
        Texto {
            es: "Elemento de la lista seleccionado.",
            en: "List item selected.",
        },
    )
    .luego(&[VER_FORMULARIO, VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "pagina",
        Texto {
            es: "Pestaña del formulario cambiada.",
            en: "Form tab changed.",
        },
    )
    .luego(&[VER_FORMULARIO, VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "esperar_cierre",
        Texto {
            es: "Cierre de la ventana esperado.",
            en: "Window close awaited.",
        },
    )
    .luego(&[VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "lanzar",
        Texto {
            es: "PVsyst lanzado.",
            en: "PVsyst launched.",
        },
    )
    .luego(&[VER_VENTANAS, VER_FUNCIONES]),
    entrada(
        "gui_actuar",
        "uia_invocar",
        Texto {
            es: "Elemento de UI Automation invocado.",
            en: "UI Automation element invoked.",
        },
    )
    .luego(&[VER_FORMULARIO, VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "esperar",
        Texto {
            es: "Espera completada.",
            en: "Wait completed.",
        },
    )
    .luego(&[VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "paso",
        Texto {
            es: "Paso de GUI ejecutado.",
            en: "GUI step run.",
        },
    )
    .luego(&[VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "funcion",
        Texto {
            es: "Función de la GUI ejecutada.",
            en: "GUI function run.",
        },
    )
    .luego(&[VER_VENTANAS, VER_ESQUEMA]),
    entrada(
        "gui_actuar",
        "menu",
        Texto {
            es: "Entrada de menú pulsada.",
            en: "Menu entry pressed.",
        },
    )
    .luego(&[VER_VENTANAS, VER_ESQUEMA]),
    entrada(
        "gui_actuar",
        "campo",
        Texto {
            es: "Campo del formulario escrito por etiqueta.",
            en: "Form field written by label.",
        },
    )
    .luego(&[VER_FORMULARIO, VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "rellenar",
        Texto {
            es: "Formulario rellenado por claves.",
            en: "Form filled by key.",
        },
    )
    .luego(&[VER_FORMULARIO, VER_ESQUEMA]),
    entrada(
        "gui_actuar",
        "clic_texto",
        Texto {
            es: "Clic en el texto visible de la ventana.",
            en: "Click on the visible text of the window.",
        },
    )
    .luego(&[VER_FORMULARIO, VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "fila",
        Texto {
            es: "Fila de la lista o del árbol seleccionada y comprobada por su banda resaltada.",
            en: "Row of the list or tree selected and checked by its highlighted band.",
        },
    )
    .luego(&[VER_FORMULARIO, VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "reiniciar",
        Texto {
            es: "PVsyst reiniciado.",
            en: "PVsyst restarted.",
        },
    )
    .luego(&[VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "cerrar_todo",
        Texto {
            es: "Ventanas de PVsyst cerradas.",
            en: "PVsyst windows closed.",
        },
    )
    .luego(&[VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "escena",
        Texto {
            es: "Escena 3D: validada (simular) o creada en el editor y comparada con el archivo que PVsyst guardó.",
            en: "3D scene: validated (simular) or created in the editor and compared with the file PVsyst saved.",
        },
    )
    .luego(TRAS_VISTA),
    entrada(
        "gui_actuar",
        "escena_leer",
        Texto {
            es: "Objetos de la escena abierta en el editor 3D, leídos del archivo que PVsyst exportó.",
            en: "Objects of the scene open in the 3D editor, read from the file PVsyst exported.",
        },
    )
    .luego(&[VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "escena_importar",
        Texto {
            es: "Malla importada en el editor 3D por PVsyst; la escena que PVsyst exportó confirma los objetos.",
            en: "Mesh imported into the 3D editor by PVsyst; the scene PVsyst exported confirms the objects.",
        },
    )
    .luego(&[VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "secuencia",
        Texto {
            es: "Secuencia de pasos de GUI ejecutada.",
            en: "Sequence of GUI steps run.",
        },
    )
    .luego(&[VER_VENTANAS, VER_ESQUEMA]),
];
