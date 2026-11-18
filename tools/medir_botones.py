"""Pressing the allowed buttons of a measured window and recording what they open.

Every press is one `gui_actuar accion:clic_texto`; `botones_abren` records the
windows each press opened, and only those are closed, reopening the host when the
button closed it. Pressing is where a measurement could stop being
one: a text starting with an entry of `DENEGADOS` is never pressed because it
commits the window's state, destroys it or starts a flow this tool cannot unwind, and
those words (`OK`, `Guardar`, `Aplicar`, `Eliminar`, `Siguiente`…) are the buttons of
almost every dialog. At most `MAX_PULSACIONES` presses per target.
"""

from __future__ import annotations

import unicodedata
from collections.abc import Collection

from medir_cliente import Cliente, Json
from medir_ventanas import _cerrar, _dormir, abrir, clases, ventanas

# Never pressed while measuring, matched as a prefix of the text in lower case and
# without accents: `OK`, `Cancelar` and `Cerrar` are the buttons of every dialog and
# close the window being measured; the `Guardar`/`Aplicar`/`Establecer` family writes
# the window's state (PVsyst then offers to save the demo project); `Eliminar`,
# `Borrar`, `Reset` and `Reiniciar` destroy what is being measured; and `Nuevo`,
# `Añadir`, `Importar`, `Exportar`, `Imprimir`, `Siguiente` and `Previo` leave the
# window in a flow the measurement does not know how to unwind.
DENEGADOS = (
    "ok",
    "cancelar",
    "cerrar",
    "guardar",
    "eliminar",
    "borrar",
    "pegar",
    "copiar",
    "establecer",
    "aplicar",
    "optimizar",
    "linealizar",
    "imprimir",
    "exportar",
    "siguiente",
    "previo",
    "añadir",
    "nuevo",
    "importar",
    "leer / importar",
    "horizonte despejado",
    "reiniciar",
    "reset",
)
# A form with many tabs lists hundreds of buttons and each press costs a click, two
# window reads and a wait; the cap keeps one target from eating the whole session.
MAX_PULSACIONES = 40


def cerrar_nuevas(c: Cliente, previas: Collection[str]) -> dict[str, str]:
    """Close only the windows that were not open before; the text per class.

    `limpiar` closes everything but the main windows, host included, so it cannot
    run between two presses; this one leaves the windows in `previas` alone.
    """
    pulsados: dict[str, str] = {}
    for _ in range(12):
        sobra = [w for w in ventanas(c) if w["class"] not in previas]
        if not sobra:
            return pulsados
        texto = _cerrar(c, sobra[0]["class"])
        if texto is not None:
            pulsados[sobra[0]["class"]] = texto
        _dormir(0.6)
    return pulsados


def _sin_acentos(texto: str) -> str:
    """Lower case without accents: «Añadir» and «anadir» must compare equal."""
    descompuesto = unicodedata.normalize("NFD", texto.lower())
    return "".join(c for c in descompuesto if not unicodedata.combining(c))


def _denegado(texto: str) -> bool:
    """Whether a button text is on the list this run never presses."""
    normal = _sin_acentos(texto)
    return any(normal.startswith(_sin_acentos(d)) for d in DENEGADOS)


def _pulsable(boton: Json) -> str | None:
    """The text to press, or None for a button this run must not press."""
    texto = str(boton.get("texto") or "").strip()
    if not texto or boton.get("habilitado") is not True:
        return None
    return None if _denegado(texto) else texto


def _error_de(respuesta: Json) -> str | None:
    """The refusal a tool answer carries, if any: `error` first, then a JSON-RPC one."""
    error = respuesta.get("error")
    if error is None:
        error = respuesta.get("jsonrpc_error")
    return None if error is None else str(error)


def _error_de_pestana(pestana: Json, texto: str) -> Json:
    """A problem of one tab, in the shape `inventariar` records its errors."""
    return {"indice": pestana["indice"], "pestana": pestana["pestana"], "texto": texto}


def _abre(nuevas: list[Json], cerradas: dict[str, str]) -> list[Json]:
    """The windows a press opened, with the text that dismissed each one."""
    salida: list[Json] = []
    for w in nuevas:
        entrada: Json = {"clase": w["class"], "titulo": w.get("title")}
        if w["class"] in cerradas:
            entrada["cierra"] = cerradas[w["class"]]
        salida.append(entrada)
    return salida


def _abrir_pestana(c: Cliente, clase: str, indice: int | None) -> str | None:
    """Select one tab again before pressing its buttons; an error text, or None."""
    if indice is None:
        return None
    r = c.tool("gui_actuar", {"accion": "pagina", "ventana": clase, "indice": indice})
    _dormir(0.5)
    error = _error_de(r)
    return None if error is None else error[:300]


def _pulsar(c: Cliente, clase: str, texto: str) -> Json:
    """Press one button of the window and dismiss what it opened: its entry."""
    previas = clases(c)
    r = c.tool("gui_actuar", {"accion": "clic_texto", "ventana": clase, "texto": texto})
    _dormir(0.8)
    nuevas = [w for w in ventanas(c) if w["class"] not in previas]
    cerradas = cerrar_nuevas(c, previas)
    return {"boton": texto, "abre": _abre(nuevas, cerradas), "error": _error_de(r)}


def _reabrir(c: Cliente, objetivo: Json, pestana: Json) -> str | None:
    """Open the host window again when the press closed it; an error text, or None."""
    clase = objetivo["ventana"]
    if clase in clases(c):
        return None
    # The reopened window starts on its first tab: select the one being pressed.
    return abrir(c, objetivo["cadena"], clase) or _abrir_pestana(
        c, clase, pestana["indice"]
    )


def pulsar_botones(c: Cliente, objetivo: Json, medida: Json) -> list[Json]:
    """Press every allowed button of the measured tabs: the `botones_abren` list.

    One entry per press. A press can close the host window or make PVsyst veto the
    next tab change: the host is reopened and, when that fails, the reason goes to
    `medida["errores"]` and the target keeps what it measured up to there.
    """
    clase = objetivo["ventana"]
    registros: list[Json] = []
    for pestana in medida["pestanas"]:
        for boton in pestana["botones"]:
            texto = _pulsable(boton)
            if texto is None:
                continue
            if len(registros) >= MAX_PULSACIONES:
                return registros
            # A press can switch the tab: the buttons on screen are not always the
            # ones this tab lists, and a text shared by two tabs would hit the wrong one.
            previas = clases(c)
            fallo = _abrir_pestana(c, clase, pestana["indice"])
            if fallo:
                cerrar_nuevas(c, previas)
                medida["errores"].append(_error_de_pestana(pestana, fallo))
                break
            registros.append(
                {"pestana": pestana["pestana"], **_pulsar(c, clase, texto)}
            )
            fallo = _reabrir(c, objetivo, pestana)
            if fallo:
                medida["errores"].append(
                    _error_de_pestana(pestana, f"after pressing {texto!r}: {fallo}")
                )
                return registros
    return registros
