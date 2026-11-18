"""Pressing the allowed buttons of a measured window and recording what they open.

Every press is one `gui_actuar accion:clic_texto`; `botones_abren` records the
windows each press opened, and only those are closed, reopening the host when the
button closed it. Only a label whose normalized text starts with an entry of
`PERMITIDOS` is pressed: this tool measures, so it opens read-only views and never a
label that commits the window's state, destroys it or starts a flow it cannot unwind.
Every button it does not press is recorded with its motive, so the reader of the JSON
tells «not pressed» apart from «pressed and opened nothing». At most
`MAX_PULSACIONES` presses per target.
"""

from __future__ import annotations

import unicodedata
from collections.abc import Collection

from medir_cliente import Cliente, Json
from medir_ventanas import _cerrar, _dormir, abrir, clases, ventanas

# White list, not a black list: a label this tool does not know is left alone.
# A black list misses the next synonym, and the windows add one every release: one
# unknown label that commits the window (Aceptar, Sí, Reemplazar, Simular, Ejecutar,
# Calcular, Actualizar, Despejar, Crear, Elaborar, Conversión, Abortar) turns a
# measurement into an edit. These prefixes only open a view. Compared in lower case,
# without accents, `&` or ellipsis.
PERMITIDOS = (
    "mostrar",
    "ver",
    "abrir",
    "curva",
    "grafico",
    "histograma",
    "tabla",
    "resumen del sistema",
    "detalles",
    "detalle",
    "ayuda",
    "vista",
    "perspectiva",
    "diagrama",
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


def _normalizado(texto: str) -> str:
    """Lower case without accents, `&` or ellipsis: what both lists compare."""
    descompuesto = unicodedata.normalize("NFD", texto.lower())
    sin_acentos = "".join(c for c in descompuesto if not unicodedata.combining(c))
    return sin_acentos.replace("&", "").replace("...", "").replace("…", "").strip()


def _permitido(texto: str) -> bool:
    """Whether a button text starts with a prefix of the white list."""
    normal = _normalizado(texto)
    return any(normal.startswith(_normalizado(p)) for p in PERMITIDOS)


def _motivo(boton: Json) -> str | None:
    """Why this run does not press the button, or None when it does."""
    texto = str(boton.get("texto") or "").strip()
    if not texto:
        return "vacío"
    if boton.get("habilitado") is not True:
        return "deshabilitado"
    return None if _permitido(texto) else "no está en la lista blanca"


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
    pulsados = 0
    for pestana in medida["pestanas"]:
        for boton in pestana["botones"]:
            texto = str(boton.get("texto") or "").strip()
            motivo = _motivo(boton)
            if motivo is not None:
                registros.append({"boton": texto, "pulsado": False, "motivo": motivo})
                continue
            if pulsados >= MAX_PULSACIONES:
                registros.append({"boton": texto, "pulsado": False, "motivo": "tope"})
                continue
            pulsados += 1
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
