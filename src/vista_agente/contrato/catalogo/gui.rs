//! `pvsyst_cli`, `gui_observar` and `gui_actuar`.

use super::{entrada, paso, Entrada, Paso, DE_VENTANA, DIAGNOSTICO};

const VER_ENTORNO: Paso = DIAGNOSTICO;
const VER_VENTANAS: Paso =
    paso("gui_observar", "Ver las ventanas de PVsyst").con(&[("que", "ventanas")]);
const VER_ESQUEMA: Paso =
    paso("gui_observar", "Ver los formularios con vista de agente").con(&[("que", "esquema")]);
const VER_FUNCIONES: Paso =
    paso("gui_observar", "Ver las funciones de la GUI").con(&[("que", "funciones")]);
const VER_FORMULARIO: Paso =
    paso("gui_observar", "Leer el formulario abierto por claves").con(&[("que", "vista")]);
const VER_CONTROLES: Paso = paso("gui_observar", "Ver los controles de la ventana")
    .con(&[("que", "controles")])
    .tomando(DE_VENTANA);
const CAPTURAR: Paso = paso("gui_observar", "Capturar la ventana")
    .con(&[("que", "capturar")])
    .tomando(DE_VENTANA);

pub(super) const ENTRADAS: &[Entrada] = &[
    entrada("pvsyst_cli", "", "Comando de PVsystCLI ejecutado.").luego(&[VER_ENTORNO]),
    entrada("gui_observar", "ventanas", "Ventanas de PVsyst listadas.")
        .luego(&[VER_ESQUEMA, VER_FUNCIONES]),
    entrada(
        "gui_observar",
        "controles",
        "Controles de la ventana listados.",
    )
    .luego(&[VER_ESQUEMA, VER_VENTANAS]),
    entrada("gui_observar", "leer", "Texto de la ventana leído.")
        .luego(&[VER_ESQUEMA, VER_VENTANAS]),
    entrada(
        "gui_observar",
        "capturar",
        "Captura de la ventana obtenida.",
    )
    .luego(&[VER_ESQUEMA, VER_VENTANAS]),
    entrada("gui_observar", "uia", "Árbol de UI Automation leído.")
        .luego(&[VER_ESQUEMA, VER_VENTANAS]),
    entrada("gui_observar", "arbol", "Árbol de controles leído.")
        .luego(&[VER_ESQUEMA, VER_VENTANAS]),
    entrada("gui_observar", "formulario", "Formulario leído por OCR.")
        .luego(&[VER_ESQUEMA, VER_VENTANAS]),
    entrada(
        "gui_observar",
        "vista",
        "Formulario leído por claves estables.",
    )
    .luego(&[VER_ESQUEMA, VER_VENTANAS]),
    entrada(
        "gui_observar",
        "esquema",
        "Esquemas de formulario con sus claves.",
    )
    .luego(&[VER_VENTANAS, VER_FUNCIONES]),
    entrada("gui_observar", "mensajes", "Mensajes de la ventana leídos.")
        .luego(&[VER_ESQUEMA, VER_VENTANAS]),
    entrada("gui_observar", "funciones", "Funciones de la GUI listadas.")
        .luego(&[VER_VENTANAS, VER_ESQUEMA]),
    entrada("gui_observar", "menus", "Menús de la ventana listados.")
        .luego(&[VER_FUNCIONES, VER_VENTANAS]),
    entrada("gui_actuar", "click", "Clic enviado al control.")
        .luego(&[VER_FORMULARIO, VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "clic_fisico",
        "Clic físico enviado a la pantalla.",
    )
    .luego(&[VER_FORMULARIO, VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "mensaje",
        "Mensaje de Windows enviado a la ventana.",
    )
    .luego(&[VER_CONTROLES, VER_VENTANAS]),
    entrada("gui_actuar", "cerrar", "Ventana cerrada.").luego(&[VER_VENTANAS]),
    entrada("gui_actuar", "activar", "Ventana traída al primer plano.")
        .luego(&[VER_FORMULARIO, VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "teclas",
        "Teclas enviadas a la ventana en primer plano.",
    )
    .luego(&[CAPTURAR, VER_VENTANAS]),
    entrada("gui_actuar", "escribir", "Texto escrito en el control.")
        .luego(&[VER_FORMULARIO, VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "seleccionar",
        "Elemento de la lista seleccionado.",
    )
    .luego(&[VER_FORMULARIO, VER_VENTANAS]),
    entrada("gui_actuar", "pagina", "Pestaña del formulario cambiada.")
        .luego(&[VER_FORMULARIO, VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "esperar_cierre",
        "Cierre de la ventana esperado.",
    )
    .luego(&[VER_VENTANAS]),
    entrada("gui_actuar", "lanzar", "PVsyst lanzado.").luego(&[VER_VENTANAS, VER_FUNCIONES]),
    entrada(
        "gui_actuar",
        "uia_invocar",
        "Elemento de UI Automation invocado.",
    )
    .luego(&[VER_FORMULARIO, VER_VENTANAS]),
    entrada("gui_actuar", "esperar", "Espera completada.").luego(&[VER_VENTANAS]),
    entrada("gui_actuar", "paso", "Paso de GUI ejecutado.").luego(&[VER_VENTANAS]),
    entrada("gui_actuar", "funcion", "Función de la GUI ejecutada.")
        .luego(&[VER_VENTANAS, VER_ESQUEMA]),
    entrada("gui_actuar", "menu", "Entrada de menú pulsada.").luego(&[VER_VENTANAS, VER_ESQUEMA]),
    entrada(
        "gui_actuar",
        "campo",
        "Campo del formulario escrito por etiqueta.",
    )
    .luego(&[VER_FORMULARIO, VER_VENTANAS]),
    entrada("gui_actuar", "rellenar", "Formulario rellenado por claves.")
        .luego(&[VER_FORMULARIO, VER_ESQUEMA]),
    entrada(
        "gui_actuar",
        "clic_texto",
        "Clic en el texto visible de la ventana.",
    )
    .luego(&[VER_FORMULARIO, VER_VENTANAS]),
    entrada("gui_actuar", "reiniciar", "PVsyst reiniciado.").luego(&[VER_VENTANAS]),
    entrada("gui_actuar", "cerrar_todo", "Ventanas de PVsyst cerradas.").luego(&[VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "escena",
        "Escena 3D validada: objetos, formas y colisiones, sin tocar PVsyst.",
    )
    .solo_ensayo()
    .luego(&[VER_ESQUEMA, VER_VENTANAS]),
    entrada(
        "gui_actuar",
        "secuencia",
        "Secuencia de pasos de GUI ejecutada.",
    )
    .luego(&[VER_VENTANAS, VER_ESQUEMA]),
];
