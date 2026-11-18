"""Opening, closing and waiting on PVsyst windows through the MCP server.

A chain step `@proyecto:NOMBRE` loads that project through the search box of the
project loader (PVsyst's shipped `_DEMO_*` projects are read-only, so nothing is
saved: any «¿Desea guardarla?» or «¿mantener modificaciones?» prompt is answered No).
Only one agent may drive the PVsyst GUI at a time. Nothing is filled in or saved.

Every wait of the whole tool goes through `_dormir`, which reads `PAUSA` from this
module: one patch here makes every wait free, whichever module asks for it.
"""

from __future__ import annotations

import re
import time
from collections.abc import Callable

from medir_cliente import Cliente, Json

MANTENER = ("THomeForm", "TApplication", "TProjectForm")
# Search box of TProjectLoadForm, client coordinates at 96 DPI.
BUSCADOR = {"clase": "TEdit", "x": 1077, "y": 49}

# Multiplies every wait; the tests set it to 0.
PAUSA = 1.0


def _dormir(segundos: float) -> None:
    time.sleep(segundos * PAUSA)


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


def ventanas(c: Cliente) -> list[Json]:
    r = c.tool("gui_observar", {"que": "ventanas", "visibles": True})
    return list(r.get("windows", []))


def clases(c: Cliente) -> set[str]:
    """Classes of the open windows: what a press is compared against."""
    return {w["class"] for w in ventanas(c)}


def asegurar_pvsyst(c: Cliente) -> str | None:
    """Launch PVsyst when no PVsyst process is running; an error text, or None."""
    if "windows" in c.tool("gui_observar", {"que": "ventanas", "visibles": True}):
        return None
    c.tool("gui_actuar", {"accion": "lanzar"})
    for _ in range(30):
        _dormir(1.0)
        if "windows" in c.tool("gui_observar", {"que": "ventanas", "visibles": True}):
            return None
    return "PVsyst did not start"


def _boton_de(textos: str) -> str:
    """The answer to a message box that never saves: Sí to «cancel the
    modifications?» (measured on 8.1.8, rotulos/dialogos.rs), No to «save?»,
    OK otherwise."""
    if "cancelar las modificaciones" in textos:
        return "Sí"
    return "No" if ("guard" in textos or "mantener" in textos) else "OK"


def _cerrar(c: Cliente, clase: str, titulo: str | None = None) -> str | None:
    """Close one window, answering its prompts as `_boton_de` says; the text pressed.

    `titulo` picks one of several windows that share a class; without it the class
    is the reference.
    """
    if clase != "TMessageForm":
        c.tool("gui_actuar", {"accion": "cerrar", "ventana": titulo or clase})
        return None
    f = c.tool("gui_observar", {"que": "formulario", "ventana": clase})
    boton = _boton_de(" ".join(f.get("textos") or []).lower())
    c.tool("gui_actuar", {"accion": "clic_texto", "ventana": clase, "texto": boton})
    return boton


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


def abrir(c: Cliente, cadena: str, clase: str) -> str | None:
    """Run the chain of steps; an error text, or None when `clase` is open."""
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
