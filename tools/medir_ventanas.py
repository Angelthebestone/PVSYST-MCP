"""Opening, closing and waiting on PVsyst windows through the MCP server.

A chain step `@proyecto:NOMBRE` loads that project through the search box of the
project loader (PVsyst's shipped `_DEMO_*` projects are read-only, so nothing is
saved: any «¿Desea guardarla?» or «¿mantener modificaciones?» prompt is answered No).
Only one agent may drive the PVsyst GUI at a time. Nothing is filled in or saved.
A read of the open windows that failed is an error the callers stop on, never an
empty screen: no windows and no answer are two different facts.

Every wait of the whole tool goes through `_dormir`, which reads `PAUSA` from this
module: one patch here makes every wait free, whichever module asks for it.
"""

from __future__ import annotations

import re
import time
import unicodedata
from collections.abc import Callable

from medir_cliente import Cliente, Json

MANTENER = ("THomeForm", "TApplication", "TProjectForm")
# Search box of TProjectLoadForm, client coordinates at 96 DPI.
BUSCADOR = {"clase": "TEdit", "x": 1077, "y": 49}

# Multiplies every wait; the tests set it to 0.
PAUSA = 1.0


class ErrorDeLectura(RuntimeError):
    """Reading the open windows failed: the screen is unknown, not empty.

    A `RuntimeError`, so a caller that already stops on one keeps stopping instead of
    reading a refusal as a screen with no windows.
    """


def _dormir(segundos: float) -> None:
    time.sleep(segundos * PAUSA)


def normalizar_etiqueta(texto: str) -> str:
    """Lower case without accents, `&` or ellipsis, spaces collapsed: how a label is
    compared, so «Mostrar  &gráfico…» and «mostrar grafico» are the same label."""
    descompuesto = unicodedata.normalize("NFD", texto.lower())
    sin_acentos = "".join(c for c in descompuesto if not unicodedata.combining(c))
    limpio = sin_acentos.replace("&", "").replace("...", "").replace("…", "")
    return " ".join(limpio.split())


def _hasta(condicion: Callable[[], bool], plazo: float) -> bool:
    """Poll every half second until `condicion()` holds or `plazo` seconds pass."""
    for _ in range(int(plazo / 0.5) + 1):
        if condicion():
            return True
        _dormir(0.5)
    return condicion()


def _es_proyecto(w: Json, nombre: str) -> bool:
    """The project window of exactly `nombre` (not `nombre2`, not `x_nombre`)."""
    patron = rf"(?<!\w){re.escape(nombre)}(?!\w)"
    return w["class"] == "TProjectForm" and bool(re.search(patron, w.get("title", "")))


# Tooltips PVsyst raises while the pointer rests on a button: not windows a press opened.
EFIMERAS = ("THintWindow", "MicrosoftWindowsTooltip")


def _error_de(respuesta: Json) -> str | None:
    """The refusal a tool answer carries, if any: `error` first, then a JSON-RPC one."""
    error = respuesta.get("error")
    if error is None:
        error = respuesta.get("jsonrpc_error")
    return None if error is None else str(error)


def ventanas(c: Cliente) -> list[Json]:
    """The visible windows of the process, without the tooltips a pointer raises.

    A read that failed raises `ErrorDeLectura`: an empty list means PVsyst answered
    with no window, and a caller that took a refusal for it would report a clean
    screen, close nothing, or launch a second instance over an unknown one.
    """
    r = c.tool("gui_observar", {"que": "ventanas", "visibles": True})
    fallo = _error_de(r)
    if fallo is not None:
        raise ErrorDeLectura(f"reading the open windows failed: {fallo[:200]}")
    crudas = r.get("windows")
    if not isinstance(crudas, list):
        raise ErrorDeLectura(
            "reading the open windows failed: the answer carries no 'windows' list"
        )
    return [w for w in crudas if w["class"] not in EFIMERAS]


def clases(c: Cliente) -> set[str]:
    """Classes of the open windows: what a press is compared against."""
    return {w["class"] for w in ventanas(c)}


# The one read error that is a state of the machine and not a failed read: the server
# answers it when no process of that name exists (`target_pids` in ventanas.rs), in the
# client's language. Measured in 8.1.8: «Process 'PVsyst.exe' is not running…».
NO_ARRANCA = ("is not running", "no está en ejecución")


def _no_arranca(fallo: str) -> bool:
    """Whether a failed read says that the PVsyst process is not running."""
    return any(texto in fallo for texto in NO_ARRANCA)


def _en_marcha(c: Cliente) -> tuple[bool, str | None]:
    """Whether the process answers with a window list, and why it could not be read."""
    try:
        ventanas(c)
    except ErrorDeLectura as e:
        return False, str(e)
    return True, None


def asegurar_pvsyst(c: Cliente) -> str | None:
    """Launch PVsyst when no PVsyst process is running; an error text, or None.

    Only the answer that says the process is absent leads to a launch: any other
    failed read stops here, because a second instance over a screen nothing could read
    is not a guess this tool makes. After the launch the read may still fail while
    PVsyst starts, so the wait keeps polling and only its deadline ends it.
    """
    en_marcha, fallo = _en_marcha(c)
    if en_marcha:
        return None
    if fallo is not None and not _no_arranca(fallo):
        return fallo
    c.tool("gui_actuar", {"accion": "lanzar"})
    for _ in range(30):
        _dormir(1.0)
        if _en_marcha(c)[0]:
            return None
    return "PVsyst did not start"


def _boton_de(f: Json) -> str | None:
    """The rejection a message box offers, or None when it offers none.

    No answers «save?» and «keep the modifications?»; Sí is pressed only for the
    measured «cancel the modifications?» question, where it means discard. Any
    other dialog is rejected with No or Cancelar; a dialog without either stays
    open instead of accepting an operation nobody asked for.
    """
    textos = normalizar_etiqueta(" ".join(str(t) for t in f.get("textos") or []))
    botones = {
        normalizar_etiqueta(str(b.get("texto") or ""))
        for b in f.get("botones") or []
        if b.get("habilitado") is not False
    }
    if "cancelar las modificaciones" in textos:
        return "Sí" if normalizar_etiqueta("Sí") in botones else None
    for candidato in ("No", "Cancelar"):
        if normalizar_etiqueta(candidato) in botones:
            return candidato
    # A notice with no choice (measured: «Por favor elija al menos una variable»,
    # only OK, no question) offers nothing to confirm, so acknowledging it is safe.
    if "?" not in textos and len(botones) == 1 and botones <= {"ok", "aceptar"}:
        return "OK" if "ok" in botones else "Aceptar"
    return None


def _cerrar(c: Cliente, clase: str, titulo: str | None = None) -> Json:
    """Close one window, answering its prompts as `_boton_de` says; the tool answer.

    `titulo` picks one of several windows that share a class; without it the class
    is the reference. The answer is returned and not dropped: a caller that must
    stop when a window does not close has to name the refusal it carries.
    """
    if clase != "TMessageForm":
        return c.tool("gui_actuar", {"accion": "cerrar", "ventana": titulo or clase})
    f = c.tool("gui_observar", {"que": "formulario", "ventana": clase})
    boton = _boton_de(f)
    if boton is None:
        return {}
    return c.tool(
        "gui_actuar", {"accion": "clic_texto", "ventana": clase, "texto": boton}
    )


def limpiar(c: Cliente) -> list[str]:
    """Close every window but the main ones; the classes still open if it gives up."""
    for _ in range(12):
        sobra = [w for w in ventanas(c) if w["class"] not in MANTENER]
        if not sobra:
            return []
        _cerrar(c, sobra[0]["class"])
        _dormir(0.6)
    return [w["class"] for w in ventanas(c) if w["class"] not in MANTENER]


def cargar_proyecto(c: Cliente, nombre: str) -> str | None:
    """Load a project through the loader's search box; an error text, or None."""
    if any(_es_proyecto(w, nombre) for w in ventanas(c)):
        return None
    limpiar(c)
    if any(w["class"] == "TProjectForm" for w in ventanas(c)):
        c.tool("gui_actuar", {"accion": "cerrar", "ventana": "TProjectForm"})
        _dormir(0.8)
        limpiar(c)
    c.tool("gui_actuar", {"accion": "funcion", "funcion": "proyecto.cargar"})
    _dormir(1.0)
    c.tool(
        "gui_actuar",
        {
            "accion": "escribir",
            "ventana": "TProjectLoadForm",
            "campo": BUSCADOR,
            "texto": nombre,
        },
    )
    _dormir(1.0)
    c.tool(
        "gui_actuar",
        {"accion": "fila", "ventana": "TProjectLoadForm", "indice": 0, "abrir": True},
    )
    if not _hasta(lambda: any(_es_proyecto(w, nombre) for w in ventanas(c)), 15):
        titulos = [w.get("title") for w in ventanas(c)]
        return f"project {nombre} did not load: {titulos}"
    return None


def _cerrar_proyecto(c: Cliente) -> str | None:
    """Close a loaded project: a chain without `@proyecto` runs from the home window,
    which a project window disables. An error text, or None."""
    if not any(w["class"] == "TProjectForm" for w in ventanas(c)):
        return None
    c.tool("gui_actuar", {"accion": "cerrar", "ventana": "TProjectForm"})
    _dormir(0.8)
    quedan = limpiar(c)
    return f"could not close the project window: {quedan}" if quedan else None


def abrir(c: Cliente, cadena: str, clase: str) -> str | None:
    """Run the chain of steps; an error text, or None when `clase` is open."""
    if not cadena.startswith("@proyecto:"):
        fallo = _cerrar_proyecto(c)
        if fallo:
            return fallo
    for paso in cadena.split(","):
        if paso.startswith("@proyecto:"):
            fallo = cargar_proyecto(c, paso.split(":", 1)[1])
            if fallo:
                return fallo
            continue
        if any(w["class"] == "TDEMOForm" for w in ventanas(c)):
            c.tool("gui_actuar", {"accion": "cerrar", "ventana": "TDEMOForm"})
        r = c.tool("gui_actuar", {"accion": "funcion", "funcion": paso})
        # A host window that opens a different one than the catalogue expects still
        # leaves the target open; the check below decides.
        if r.get("error") and "did not open the expected window" not in str(r["error"]):
            return f"{paso}: {str(r['error'])[:200]}"
        _dormir(1.0)
    if _hasta(lambda: any(w["class"] == clase for w in ventanas(c)), 5):
        return None
    return f"{clase} did not open: {[w['class'] for w in ventanas(c)]}"
