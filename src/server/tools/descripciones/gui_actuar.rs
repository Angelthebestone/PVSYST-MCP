//! Argument descriptions of `gui_actuar`.

use super::{entrada, Entrada};

pub(super) const TABLA: &[Entrada] = &[
    entrada(
        "abrir",
        "fila: pulsa Enter en la fila elegida y dice si algo se abrió o cerró; alias confirmar.",
        "fila: presses Enter on the chosen row and says whether something opened or closed; alias confirmar.",
    ),
    entrada(
        "archivo",
        "escena_importar: malla (3DS, DAE, PVC, PVC2, PVZ2) del workspace, fuera de Program Files.",
        "escena_importar: mesh (3DS, DAE, PVC, PVC2, PVZ2) inside the workspace, outside Program Files.",
    ),
    entrada(
        "cancelar",
        "cerrar: pulsa Cancelar/No antes de los botones por defecto; cerrar_todo lo activa por defecto.",
        "cerrar: presses Cancelar/No before the default buttons; cerrar_todo turns it on by default.",
    ),
    entrada(
        "cerrar_demo",
        "lanzar: cierra el aviso de demo caducada (TDEMOForm) del proceso.",
        "lanzar: closes the expired-demo notice (TDEMOForm) of the process.",
    ),
    entrada(
        "confirmar",
        "cerrar_todo: exige true para cerrar y descartar lo no guardado; en fila, pulsa Enter.",
        "cerrar_todo: demands true to close and discard unsaved work; on fila, presses Enter.",
    ),
    entrada(
        "dx",
        "click y clic_fisico: x en px a 96 DPI desde la esquina del control; va con dy.",
        "click and clic_fisico: x in px at 96 DPI from the control corner; goes with dy.",
    ),
    entrada(
        "dy",
        "click y clic_fisico: y en px a 96 DPI desde la esquina del control; va con dx.",
        "click and clic_fisico: y in px at 96 DPI from the control corner; goes with dx.",
    ),
    entrada(
        "entidad",
        "rellenar: clave de la entidad cuyo formulario se rellena (gui_observar que:esquema).",
        "rellenar: key of the entity whose form is filled (gui_observar que:esquema).",
    ),
    entrada(
        "escena",
        "escena, escena_leer y escena_importar: nombre con el que PVsyst exporta la escena.",
        "escena, escena_leer and escena_importar: name PVsyst exports the scene with.",
    ),
    entrada(
        "esperar_segundos",
        "lanzar: s de espera al formulario (30, tope 120); esperar: la espera en s, gana a timeout_ms.",
        "lanzar: s to wait for the form (30, max 120); esperar: the wait in s, over timeout_ms.",
    ),
    entrada(
        "id",
        "menu: id de comando Win32 (0 a 65535); alternativa a la pareja menu e item.",
        "menu: Win32 command id (0 to 65535); an alternative to the menu and item pair.",
    ),
    entrada(
        "item",
        "menu: entrada contada desde 0 dentro del menú; las separadoras cuentan.",
        "menu: entry counted from 0 inside the menu; separators count.",
    ),
    entrada(
        "lparam",
        "mensaje: lParam del mensaje Win32, entero; 0 si se omite.",
        "mensaje: lParam of the Win32 message, an integer; 0 when omitted.",
    ),
    entrada(
        "menu",
        "menu: menú de la barra contado desde 0 por la izquierda; se usa con item.",
        "menu: menu bar entry counted from 0 on the left; used with item.",
    ),
    entrada(
        "objetivo",
        "clic_fisico: {x, y} en px de pantalla; paso: {nombre, rol, clase, titulo}.",
        "clic_fisico: {x, y} in screen px; paso: {nombre, rol, clase, titulo}.",
    ),
    entrada(
        "objetos",
        "escena: objetos de la escena; cada uno con tipo, x e y en metros y su forma.",
        "escena: the scene objects; each one with tipo, x and y in metres and its shape.",
    ),
    entrada(
        "pasos",
        "secuencia: acciones que corren en orden, sin anidar; se detiene en el primer fallo.",
        "secuencia: actions run in order, never nested; stops at the first failure.",
    ),
    entrada(
        "simular",
        "Ensayo de escena y rellenar (plan sin tocar la GUI); escena_importar importa y cancela.",
        "Dry run of escena and rellenar (a plan, no GUI); escena_importar imports then cancels.",
    ),
    entrada(
        "subitem",
        "menu: entrada contada desde 0 del submenú que abre item.",
        "menu: entry counted from 0 of the submenu that item opens.",
    ),
    entrada(
        "timeout_ms",
        "esperar y esperar_cierre: ms de espera (10 s por defecto); menu y funcion: 5 s, tope 60 s.",
        "esperar and esperar_cierre: ms to wait (10 s by default); menu and funcion: 5 s, max 60 s.",
    ),
    entrada(
        "unidades",
        "escena_importar: unidad elegida en el combo del import (p.ej. Metros).",
        "escena_importar: unit chosen in the import combo (e.g. Metros).",
    ),
    entrada(
        "valores",
        "rellenar: objeto {clave: valor} con los campos de la entidad a escribir.",
        "rellenar: {key: value} object with the entity fields to type.",
    ),
    entrada(
        "verificar",
        "paso: {clase, titulo} de la ventana que debe abrirse; espera 5 s y responde verificado.",
        "paso: {clase, titulo} of the window that must open; waits 5 s and answers verificado.",
    ),
    entrada(
        "wparam",
        "mensaje: wParam del mensaje Win32, entero sin signo; 0 si se omite.",
        "mensaje: wParam of the Win32 message, unsigned integer; 0 when omitted.",
    ),
];
