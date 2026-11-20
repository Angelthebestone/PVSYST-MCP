"""One target of the manifest: open its window, inventory it, close it.

`medir` is the unit `main` repeats per target: clean the windows, open the target by
its chain, read every tab, optionally press its buttons, refuse a measurement whose
inventory recorded an error and close what it opened, always. A press that fails is
not an inventory error: its reason stays in `botones_abren` and the fields already
measured are merged.
"""

from __future__ import annotations

from medir_botones import pulsar_botones
from medir_cliente import Cliente, Json
from medir_inventario import ErrorMedicion, inventariar
from medir_proyectos import Objetivo, abrir
from medir_ventanas import limpiar


def _rechazar(medida: Json) -> str | None:
    """Why a measurement must not be merged, or None."""
    if medida["errores"]:
        return f"the window reported errors, nothing merged: {medida['errores']}"
    if not any(p["campos"] or p["botones"] for p in medida["pestanas"]):
        return "the window read back with no fields and no buttons, nothing merged"
    return None


def _medido(c: Cliente, objetivo: Objetivo, botones: bool) -> Json | str:
    """Measure one target while its window is open: the measurement, or an error text.

    `botones` presses the allowed buttons of every tab, which is what `--botones`
    asks for. The projects the run loaded itself, and so may close, travel inside
    `objetivo`, so a target whose chain loads none may close those and no others.
    """
    fallo = abrir(c, objetivo)
    if fallo:
        return fallo
    medida = inventariar(c, objetivo.ventana)
    fallo = _rechazar(medida)
    if fallo:
        return fallo
    if botones:
        medida["botones_abren"] = pulsar_botones(c, objetivo, medida)
    return medida


def _limpiar_despues(c: Cliente) -> str | None:
    """Close what the target left open, best effort: why it could not, or None.

    This runs while the error that ended the target, if any, is on its way out, so
    anything a cleanup raises here is reported and never raised: the reason the target
    failed is the one that must reach the caller.
    """
    try:
        quedan = limpiar(c)
    except Exception as e:
        return str(e)
    return f"still open: {quedan}" if quedan else None


def medir(c: Cliente, objetivo: Objetivo, botones: bool = False) -> Json | str:
    """Open, inventory and close one target: the measurement, or an error text.

    A cleanup that cannot close what the target opened does not throw away what it
    measured: the measurement comes back carrying that reason in `parado`, so the
    fields are merged and `main` counts the target as failed. When the target failed
    already, the reason it failed is returned with the cleanup's own reason appended;
    any other exception is raised the same way, after the target is closed.
    """
    previas = limpiar(c)
    if previas:
        return f"could not close the windows left open: {previas}"
    try:
        resultado: Json | str = _medido(c, objetivo, botones)
    except ErrorMedicion as e:
        resultado = str(e)
    except Exception as e:
        # A read or a wait failed with the target open: close it anyway, so the next
        # run does not find the window of this one, and keep the reason it failed.
        fallo_limpieza = _limpiar_despues(c)
        if fallo_limpieza is None:
            raise
        raise type(e)(f"{e}; and closing the target failed: {fallo_limpieza}") from e
    fallo_limpieza = _limpiar_despues(c)
    if fallo_limpieza is None:
        return resultado
    if isinstance(resultado, str):
        return f"{resultado}; and closing the target failed: {fallo_limpieza}"
    resultado["parado"] = fallo_limpieza
    return resultado
