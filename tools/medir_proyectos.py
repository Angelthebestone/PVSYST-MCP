"""Which project a chain loads, and which project windows this run may close.

A chain step `@proyecto:NOMBRE` loads that project through the search box of the
project loader (`abrir`); a chain without one runs from the home window, which a
loaded project disables, so the open project has to go first. PVsyst's shipped
`_DEMO_*` projects load from the installation's read-only `DataRO` folder, and `DEMOS`
names them, so a window showing one is closed. A project this run did not load and
that is not a demo is never closed: the No that answers «¿Desea guardarla?» would
throw away the user's unsaved work, so the chain stops naming it.
`Objetivo` is one target of the manifest and carries the `Propiedad` of the run, the
record of what it loaded itself, so a step that reopens a window cannot forget it.
Nothing is filled in or saved.
"""

from __future__ import annotations

import re
from collections.abc import Callable
from dataclasses import dataclass

from medir_cliente import Cliente, Json
from medir_ventanas import _cerrar, _dormir, _error_de, cerrable, limpiar, ventanas

# Search box of TProjectLoadForm, client coordinates at 96 DPI.
BUSCADOR = {"clase": "TEdit", "x": 1077, "y": 49}


def _hasta(condicion: Callable[[], bool], plazo: float) -> bool:
    """Poll every half second until `condicion()` holds or `plazo` seconds pass."""
    for _ in range(int(plazo / 0.5) + 1):
        if condicion():
            return True
        _dormir(0.5)
    return condicion()


# The demo projects PVsyst ships in `DataRO\PVsyst8.1_Data\Projects`, the read-only
# data folder of the installation (`docs/gui_hibrida.md` lists these names,
# `docs/pvsyst_format.md` records the folder as read-only). A project loaded from there
# is not a file the user can save in place, so the chain closes its window instead of
# asking them to. The list is exact and a name is compared with the whole project name
# the title shows, so a user's `_DEMO_mio`, an `X_DEMO_Foo` or a `Copia
# (_DEMO_Commercial)` is not a demo and is never closed.
DEMOS = (
    "_DEMO_Commercial",
    "_DEMO_DCGrid",
    "_DEMO_N13_Motorway_Data",
    "_DEMO_Pumping",
    "_DEMO_Residential",
    "_DEMO_StandAlone",
    "_DEMO_Tracking",
    "_DEMO_Utility",
)

# The title of a project window shows its name: «Proyecto <nombre>», with the active
# variant after ` - ` (the titles the fixtures measure, e.g. «Proyecto _DEMO_Commercial
# - VC0»). A title that does not follow that shape answers itself, which matches no
# demo and no loaded project, so its window is left open.
TITULO = re.compile(r"^Proyecto (?P<nombre>.+?)(?: - .*)?$")


def _nombre_del_titulo(titulo: str) -> str:
    """The project name the window title of `titulo` shows."""
    encontrado = TITULO.match(titulo)
    return titulo if encontrado is None else encontrado.group("nombre").strip()


def _es_proyecto(w: Json, nombre: str) -> bool:
    """The project window of exactly `nombre`, not one of a name that contains it.

    `_patron` read «Proyecto Planta 2» as the loaded Planta and measured over it; this
    check and `es_propio` share the one name rule now.
    """
    if w["class"] != "TProjectForm":
        return False
    return _nombre_del_titulo(str(w.get("title") or "")) == nombre


class Propiedad:
    """The projects this run loaded itself: the only non-demo ones it may close.

    `cargar_proyecto` records every project it opens. Any other project window stays
    open and stops the chain naming it: the No that answers PVsyst's «¿Desea
    guardarla?» would throw away the user's unsaved work. The record travels inside
    the `Objetivo` of the target being measured, so the step that reopens a host
    window cannot call `abrir` without it.
    """

    def __init__(self) -> None:
        self._cargados: set[str] = set()

    def registrar(self, nombre: str) -> None:
        """Record a project this run loaded through the loader."""
        self._cargados.add(nombre)

    def es_propio(self, titulo: str) -> bool:
        """Whether this run may close the project window titled `titulo`."""
        nombre = _nombre_del_titulo(titulo)
        return nombre in DEMOS or nombre in self._cargados


@dataclass(frozen=True)
class Objetivo:
    """One target of the manifest: the window its chain must leave open, the steps of
    the chain, and the projects the run may close while running it."""

    clave: str
    ventana: str
    cadena: str
    propiedad: Propiedad


def _proyectos(c: Cliente) -> list[Json]:
    """The project windows open now."""
    return [w for w in ventanas(c) if w["class"] == "TProjectForm"]


def _pasos(nombre: str) -> tuple[tuple[Json, str], ...]:
    """The three GUI steps of the loader, each with the text an error is reported as."""
    funcion: Json = {"accion": "funcion", "funcion": "proyecto.cargar"}
    escribir: Json = {
        "accion": "escribir",
        "ventana": "TProjectLoadForm",
        "campo": BUSCADOR,
        "texto": nombre,
    }
    fila: Json = {
        "accion": "fila",
        "ventana": "TProjectLoadForm",
        "indice": 0,
        "abrir": True,
    }
    return (
        (funcion, "opening the project loader failed"),
        (escribir, "writing the project name in the search box failed"),
        (fila, "opening the first row of the loader failed"),
    )


def cargar_proyecto(c: Cliente, nombre: str, propiedad: Propiedad) -> str | None:
    """Load a project through the loader's search box; an error text, or None.

    The project already open is closed first, and `_cerrar_proyecto` decides whether
    this run may; `propiedad` then records what a later chain without `@proyecto` may
    close. Every answer is read: a step the server refused used to be dropped here,
    and the only symptom was a 15 s «did not load» that named no cause.
    """
    if any(_es_proyecto(w, nombre) for w in ventanas(c)):
        return None
    quedan = limpiar(c)
    if quedan:
        return f"could not close the windows left open: {quedan}"
    fallo = _cerrar_proyecto(c, propiedad)
    if fallo:
        return fallo
    for argumentos, motivo in _pasos(nombre):
        error = _error_de(c.tool("gui_actuar", argumentos))
        if error is not None:
            return f"{motivo}: {error[:200]}"
        _dormir(1.0)
    if not _hasta(lambda: any(_es_proyecto(w, nombre) for w in ventanas(c)), 15):
        titulos = [w.get("title") for w in ventanas(c)]
        return f"project {nombre} did not load: {titulos}"
    propiedad.registrar(nombre)
    return None


def _cerrar_proyecto(c: Cliente, propiedad: Propiedad) -> str | None:
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
    ajenos = [t for t in titulos if not propiedad.es_propio(t)]
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


def _cerrar_aviso(c: Cliente) -> str | None:
    """Close the trial notice when it is in the way of a chain step; an error text.

    The notice is modal and blocks every GUI call of a chain, so a step closes it
    first, and `cerrable` decides whether this run may. A refused close is an error
    here: letting the step run would fail on the disabled window and blame the
    function instead.
    """
    abiertas = ventanas(c)
    if not any(w["class"] == "TDEMOForm" and cerrable(w, abiertas) for w in abiertas):
        return None
    fallo = _error_de(_cerrar(c, "TDEMOForm"))
    return None if fallo is None else f"closing the trial notice failed: {fallo[:200]}"


def abrir(c: Cliente, objetivo: Objetivo) -> str | None:
    """Run the chain of steps; an error text, or None when its window is open."""
    if not objetivo.cadena.startswith("@proyecto:"):
        fallo = _cerrar_proyecto(c, objetivo.propiedad)
        if fallo:
            return fallo
    for paso in objetivo.cadena.split(","):
        if paso.startswith("@proyecto:"):
            nombre = paso.split(":", 1)[1]
            fallo = cargar_proyecto(c, nombre, objetivo.propiedad)
            if fallo:
                return fallo
            continue
        fallo = _cerrar_aviso(c)
        if fallo:
            return fallo
        r = c.tool("gui_actuar", {"accion": "funcion", "funcion": paso})
        error = _error_de(r)
        # A host window that opens a different one than the catalogue expects still
        # leaves the target open; the check below decides.
        if error is not None and "did not open the expected window" not in error:
            return f"{paso}: {error[:200]}"
        _dormir(1.0)
    if _hasta(lambda: any(w["class"] == objetivo.ventana for w in ventanas(c)), 5):
        return None
    return f"{objetivo.ventana} did not open: {[w['class'] for w in ventanas(c)]}"
