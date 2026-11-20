"""Closing the windows a button press opened, and only those.

`cerrar_nuevas` compares the open windows against the ones the press found
(`previas`) and closes the difference: a window titled like the host would close the
host if the close went by title, so every close names a class. A new window of a
class `MANTENER` keeps is not closed at all but reported as a stop, since one more
window of that class would leave the close choosing between them.
"""

from __future__ import annotations

from collections.abc import Collection

from medir_cliente import Cliente, Json
from medir_ventanas import (
    MANTENER,
    _cerrar,
    _dormir,
    _error_de,
    _responder,
    cerrable,
    ventanas,
)


def _nuevas(
    abiertas: Collection[Json], previas: Collection[tuple[str, str]]
) -> list[Json]:
    """The windows the press opened and this run may close.

    A window is new when its `(class, title)` pair was not in `previas`; `cerrable`
    drops the ones a run keeps open, the main windows and the trial notice a project
    window sits above.
    """
    return [
        w
        for w in abiertas
        if (w["class"], str(w.get("title") or "")) not in previas
        and cerrable(w, abiertas)
    ]


def _protegidas(
    abiertas: Collection[Json], previas: Collection[tuple[str, str]]
) -> list[Json]:
    """The new windows of a class `MANTENER` keeps: closing one may hit another."""
    return [
        w
        for w in abiertas
        if w["class"] in MANTENER
        and (w["class"], str(w.get("title") or "")) not in previas
    ]


def _parado(quedan: Collection[Json], fallo: str | None) -> str:
    """Why the rounds ran out: the windows still open and the close's refusal."""
    nombres = ", ".join(f"{w['class']} {w.get('title')!r}" for w in quedan)
    motivo = f"a window the press opened could not be closed: {nombres}"
    return motivo if fallo is None else f"{motivo}; the close answered: {fallo[:200]}"


def cerrar_nuevas(
    c: Cliente, previas: Collection[tuple[str, str]]
) -> tuple[list[Json], str | None]:
    """Close the windows a press opened, never one it found; entries and a stop reason.

    `previas` is the `(class, title)` pair of every window the harness saw before the
    press, and a window whose pair is not in it is the press's. Every close names a
    class and never a title: a title is matched by substring over every window of the
    process, so a new window titled like the host would close the host. A new window of
    a class `MANTENER` keeps is not closed at all but a stop naming it, and `cerrable`
    keeps the main windows and the trial notice a project window sits above. A dialog
    is answered with `_responder`, and one it cannot answer stays open and its reason is
    the stop. Twelve rounds that leave a window open are a stop too: a press that left
    a modal open is not a clean press, and every press after it would fail on the
    disabled window and blame the label.
    """
    cerradas: dict[tuple[str, str], Json] = {}
    fallo: str | None = None
    for _ in range(12):
        abiertas = ventanas(c)
        protegidas = _protegidas(abiertas, previas)
        if protegidas:
            nombres = ", ".join(f"{w['class']} {w.get('title')!r}" for w in protegidas)
            return list(cerradas.values()), (
                "the press opened a window of a class this run keeps open, so it is "
                f"not closed: {nombres}"
            )
        nuevas = _nuevas(abiertas, previas)
        for w in nuevas:
            clave = (w["class"], str(w.get("title") or ""))
            cerradas.setdefault(clave, {"clase": clave[0], "titulo": clave[1]})
        if not nuevas:
            return list(cerradas.values()), None
        w = nuevas[0]
        clave = (w["class"], str(w.get("title") or ""))
        if w["class"] == "TMessageForm":
            respuesta = _responder(c)
            if respuesta.fallo is not None:
                return list(cerradas.values()), respuesta.fallo
            cerradas[clave]["cierra"] = respuesta.boton
        else:
            fallo = _error_de(_cerrar(c, w["class"]))
        _dormir(0.6)
    quedan = _nuevas(ventanas(c), previas)
    if not quedan:
        return list(cerradas.values()), None
    return list(cerradas.values()), _parado(quedan, fallo)
