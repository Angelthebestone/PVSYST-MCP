"""Which project a chain loads, and which project windows this run may close.

A chain step `@proyecto:NOMBRE` loads that project through the search box of the
project loader (`abrir`); a chain without one runs from the home window, which a
loaded project disables, so the open project has to go first. PVsyst's shipped
`_DEMO_*` projects are read-only, so closing one saves nothing. A project this run
did not load and that is not a demo is never closed: the No that answers «¿Desea
guardarla?» would throw away the user's unsaved work, so the chain stops naming it.
Nothing is filled in or saved.
"""

from __future__ import annotations

import re
from collections.abc import Callable, Collection

from medir_cliente import Cliente, Json
from medir_ventanas import _cerrar, _dormir, _error_de, limpiar, ventanas

# Search box of TProjectLoadForm, client coordinates at 96 DPI.
BUSCADOR = {"clase": "TEdit", "x": 1077, "y": 49}


def _hasta(condicion: Callable[[], bool], plazo: float) -> bool:
    """Poll every half second until `condicion()` holds or `plazo` seconds pass."""
    for _ in range(int(plazo / 0.5) + 1):
        if condicion():
            return True
        _dormir(0.5)
    return condicion()


def _patron(nombre: str) -> str:
    """A regular expression for `nombre` as a whole word of a window title."""
    return rf"(?<!\w){re.escape(nombre)}(?!\w)"


def _es_proyecto(w: Json, nombre: str) -> bool:
    """The project window of exactly `nombre` (not `nombre2`, not `x_nombre`)."""
    return w["class"] == "TProjectForm" and bool(
        re.search(_patron(nombre), w.get("title", ""))
    )


# PVsyst's shipped demos are read-only, so closing one saves nothing. The guard of
# `_patron` keeps `X_DEMO_Foo` from passing as one, as it does in `_es_proyecto`.
PATRON_DEMO = re.compile(r"(?<!\w)_DEMO_\w")


def _proyectos(c: Cliente) -> list[Json]:
    """The project windows open now."""
    return [w for w in ventanas(c) if w["class"] == "TProjectForm"]


def _es_propio(titulo: str, cargados: Collection[str]) -> bool:
    """Whether this run may close the project whose window title is `titulo`.

    Only a read-only demo and a project the run loaded itself qualify: closing any
    other answers No to PVsyst's save prompt and loses the unsaved work of a project
    the harness merely found open.
    """
    return PATRON_DEMO.search(titulo) is not None or any(
        re.search(_patron(nombre), titulo) is not None for nombre in cargados
    )


def cargar_proyecto(
    c: Cliente, nombre: str, cargados: set[str] | None = None
) -> str | None:
    """Load a project through the loader's search box; an error text, or None.

    The project already open is closed first, and `_cerrar_proyecto` decides whether
    this run may: `cargados` is what a later chain without `@proyecto` may close.
    """
    if any(_es_proyecto(w, nombre) for w in ventanas(c)):
        return None
    quedan = limpiar(c)
    if quedan:
        return f"could not close the windows left open: {quedan}"
    fallo = _cerrar_proyecto(c, cargados)
    if fallo:
        return fallo
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
    if cargados is not None:
        cargados.add(nombre)
    return None


def _cerrar_proyecto(
    c: Cliente,
    cargados: Collection[str] | None = None,
) -> str | None:
    """Close the loaded project before a chain that does not load one; an error text.

    A chain without `@proyecto` runs from the home window, which a project window
    disables, so the project has to go. A window this run did not load and that is not
    a read-only demo stays open and stops the chain naming it: it may hold the user's
    unsaved work. After a close the window must be gone, or the error says it is not.
    """
    abiertos = _proyectos(c)
    if not abiertos:
        return None
    titulos = [str(w.get("title") or "") for w in abiertos]
    ajenos = [t for t in titulos if not _es_propio(t, cargados or ())]
    if ajenos:
        return (
            f"the project window(s) {ajenos} are not loaded by this run and are not "
            "read-only demos: not closing them. Add @proyecto to the chain to load "
            "the project, or close it in PVsyst first"
        )
    fallo = _error_de(_cerrar(c, "TProjectForm"))
    _dormir(0.8)
    quedan = limpiar(c)
    if quedan:
        return f"could not close the project: {quedan}"
    restantes = [str(w.get("title") or "") for w in _proyectos(c)]
    if not restantes:
        return None
    motivo = f"TProjectForm is still open after the close: {restantes}"
    return motivo if fallo is None else f"{motivo}; the close answered: {fallo[:200]}"


def abrir(
    c: Cliente, cadena: str, clase: str, cargados: set[str] | None = None
) -> str | None:
    """Run the chain of steps; an error text, or None when `clase` is open."""
    if not cadena.startswith("@proyecto:"):
        fallo = _cerrar_proyecto(c, cargados)
        if fallo:
            return fallo
    for paso in cadena.split(","):
        if paso.startswith("@proyecto:"):
            fallo = cargar_proyecto(c, paso.split(":", 1)[1], cargados)
            if fallo:
                return fallo
            continue
        if any(w["class"] == "TDEMOForm" for w in ventanas(c)):
            c.tool("gui_actuar", {"accion": "cerrar", "ventana": "TDEMOForm"})
        r = c.tool("gui_actuar", {"accion": "funcion", "funcion": paso})
        error = _error_de(r)
        # A host window that opens a different one than the catalogue expects still
        # leaves the target open; the check below decides.
        if error is not None and "did not open the expected window" not in error:
            return f"{paso}: {error[:200]}"
        _dormir(1.0)
    if _hasta(lambda: any(w["class"] == clase for w in ventanas(c)), 5):
        return None
    return f"{clase} did not open: {[w['class'] for w in ventanas(c)]}"
