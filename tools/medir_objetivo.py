"""One target of the manifest: open its window, inventory it, close it.

`medir` is the unit `main` repeats per `clave`: clean the windows, open the target by
its chain, read every tab, optionally press its buttons, refuse a measurement that
recorded an error and close what it opened, always.
"""

from __future__ import annotations

from medir_botones import pulsar_botones
from medir_cliente import Cliente, Json
from medir_inventario import ErrorMedicion, inventariar
from medir_ventanas import abrir, limpiar


def _rechazar(medida: Json) -> str | None:
    """Why a measurement must not be merged, or None."""
    if medida["errores"]:
        return f"the window reported errors, nothing merged: {medida['errores']}"
    if not any(p["campos"] or p["botones"] for p in medida["pestanas"]):
        return "the window read back with no fields and no buttons, nothing merged"
    return None


def medir(c: Cliente, objetivo: Json, botones: bool = False) -> Json | str:
    """Open, inventory and close one target: the measurement, or an error text."""
    previas = limpiar(c)
    if previas:
        return f"could not close the windows left open: {previas}"
    try:
        fallo = abrir(c, objetivo["cadena"], objetivo["ventana"])
        if fallo:
            return fallo
        medida = inventariar(c, objetivo["ventana"])
        fallo = _rechazar(medida)
        if fallo:
            return fallo
        if botones:
            medida["botones_abren"] = pulsar_botones(c, objetivo, medida)
            # Pressing may have lost the host window for good: errors it recorded.
            return _rechazar(medida) or medida
        return medida
    except ErrorMedicion as e:
        return str(e)
    finally:
        limpiar(c)
