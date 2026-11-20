"""Reading, closing and waiting on PVsyst windows through the MCP server.

`limpiar` closes every window but the main ones, answering each prompt with a
rejection (`_boton_de`); `asegurar_pvsyst` starts PVsyst when no process answers.
The project a chain loads, and which project windows this run may close, live in
`medir_proyectos.py`. Only one agent may drive the PVsyst GUI at a time, and nothing
is filled in or saved.

Every wait of the whole tool goes through `_dormir`, which reads `PAUSA` from this
module: one patch here makes every wait free, whichever module asks for it.
"""

from __future__ import annotations

import time
import unicodedata
from dataclasses import dataclass

from medir_cliente import Cliente, Json

MANTENER = ("THomeForm", "TApplication", "TProjectForm")

# Multiplies every wait; the tests set it to 0.
PAUSA = 1.0


class ErrorDeLectura(RuntimeError):
    """Reading the open windows failed: the screen is unknown, not empty.

    A `RuntimeError`, so a caller that already stops on one keeps stopping instead of
    reading a refusal as a screen with no windows.
    """


@dataclass(frozen=True)
class Dialogo:
    """A message box read and, when it offered a rejection, answered.

    `boton` is the caption `_responder` pressed and is set exactly when `fallo` is
    not; `textos` is what the box said, which the inventory records.
    """

    textos: tuple[str, ...]
    boton: str | None
    fallo: str | None


def _dormir(segundos: float) -> None:
    time.sleep(segundos * PAUSA)


def normalizar_etiqueta(texto: str) -> str:
    """Lower case without accents, `&` or ellipsis, spaces collapsed: how a label is
    compared, so «Mostrar  &gráfico…» and «mostrar grafico» are the same label."""
    descompuesto = unicodedata.normalize("NFD", texto.lower())
    sin_acentos = "".join(c for c in descompuesto if not unicodedata.combining(c))
    limpio = sin_acentos.replace("&", "").replace("...", "").replace("…", "")
    return " ".join(limpio.split())


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
    screen, close nothing, or launch a second instance over an unknown one. A window
    without a `class` is a read error of the same kind: every caller names and closes
    windows by their class, so a `KeyError` raised down the line would end the tool
    with a traceback that does not say which window the answer was talking about.
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
    for w in crudas:
        if not isinstance(w, dict) or not isinstance(w.get("class"), str):
            raise ErrorDeLectura(
                "reading the open windows failed: a window carries no 'class': "
                f"{str(w)[:200]}"
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


# The windows a PVsyst this run did not start may show without holding the user's work:
# the trial notice, which PVsyst raises by itself (measured). Every other class outside
# `MANTENER` is a window somebody opened — the user's component editor, say, or the
# leftover of a run that was killed — and closing it answers No to its save prompt.
PROPIAS = ("TDEMOForm",)


def _ajenas(c: Cliente) -> str | None:
    """Why a session this run did not start must not be touched; an error text, or None.

    The cleanup answers No to every «¿Desea guardarla?», so a window this run did not
    open may cost the user their unsaved work. The run stops naming those windows
    instead; `--limpiar-ajenas` is how the user says they are disposable.
    """
    abiertas = ventanas(c)
    ajenas = [w for w in abiertas if w["class"] not in (*MANTENER, *PROPIAS)]
    if not ajenas:
        return None
    nombres = ", ".join(f"{w['class']} {w.get('title')!r}" for w in ajenas)
    return (
        f"PVsyst was already running and shows window(s) this run did not open: "
        f"{nombres}. Close them by hand, or run again with --limpiar-ajenas to let "
        "this tool close them (it answers No to their save prompts)"
    )


def asegurar_pvsyst(c: Cliente, limpiar_ajenas: bool = False) -> str | None:
    """Launch PVsyst when no PVsyst process is running; an error text, or None.

    Only the answer that says the process is absent leads to a launch: any other
    failed read stops here, because a second instance over a screen nothing could read
    is not a guess this tool makes. After the launch the read may still fail while
    PVsyst starts, so the wait keeps polling and only its deadline ends it. A session
    this run found already running is refused when it shows a window the run did not
    open (`_ajenas`), unless `limpiar_ajenas` says those windows are disposable.
    """
    en_marcha, fallo = _en_marcha(c)
    if en_marcha:
        return None if limpiar_ajenas else _ajenas(c)
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


def _responder(c: Cliente) -> Dialogo:
    """Read the open message box and press the rejection `_boton_de` picks.

    The one place of this tool that answers a `TMessageForm`, so the three callers
    decide with the same rule. What it could not do is not dropped: a box that cannot
    be read, one that offers no rejection and a caption the server refuses each come
    back in `fallo`, naming which of the three happened.
    """
    f = c.tool("gui_observar", {"que": "formulario", "ventana": "TMessageForm"})
    textos = tuple(str(t) for t in f.get("textos") or [])
    fallo = _error_de(f)
    if fallo is not None:
        return Dialogo(textos, None, f"reading TMessageForm failed: {fallo[:200]}")
    boton = _boton_de(f)
    if boton is None:
        botones = [b.get("texto") for b in f.get("botones") or []]
        return Dialogo(
            textos,
            None,
            f"unrecognised dialog, left open: {list(textos)} (buttons: {botones})",
        )
    r = c.tool(
        "gui_actuar",
        {"accion": "clic_texto", "ventana": "TMessageForm", "texto": boton},
    )
    fallo = _error_de(r)
    if fallo is not None:
        return Dialogo(
            textos, None, f"answering TMessageForm with {boton!r} failed: {fallo[:200]}"
        )
    return Dialogo(textos, boton, None)


def _cerrar(c: Cliente, clase: str) -> Json:
    """Close one window, answering its prompts as `_boton_de` says; the tool answer.

    The window is named by its class and never by a title: `accion:cerrar` resolves a
    reference by substring over every window of the process, so a title a window the
    harness keeps open also holds would close that one instead. The answer is returned
    and not dropped: a caller that must stop when a window does not close has to name
    the refusal it carries, so a dialog that could not be answered answers here with
    the reason.
    """
    if clase != "TMessageForm":
        return c.tool("gui_actuar", {"accion": "cerrar", "ventana": clase})
    respuesta = _responder(c)
    if respuesta.fallo is not None:
        return {"error": respuesta.fallo}
    return {"texto": respuesta.boton}


def _sobran(c: Cliente) -> list[Json]:
    """The windows `limpiar` must close. The trial notice (`TDEMOForm`) is skipped
    while a project is open: the project window is modal above it, the server
    refuses to close it (measured), and the next chain without a project closes
    the project and then the notice."""
    abiertas = ventanas(c)
    con_proyecto = any(w["class"] == "TProjectForm" for w in abiertas)
    return [
        w
        for w in abiertas
        if w["class"] not in MANTENER
        and not (con_proyecto and w["class"] == "TDEMOForm")
    ]


def limpiar(c: Cliente) -> list[str]:
    """Close every window but the main ones; the classes still open if it gives up."""
    for _ in range(12):
        sobra = _sobran(c)
        if not sobra:
            return []
        _cerrar(c, sobra[0]["class"])
        _dormir(0.6)
    return [w["class"] for w in _sobran(c)]
