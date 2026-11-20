"""Pressing the allowed buttons of a measured window and recording what they open.

Every press is one `gui_actuar accion:clic_texto`; `botones_abren` records the
windows each press opened, and only those are closed, reopening the host when the
button closed it. Only a label whose normalized text is exactly one entry of
`PERMITIDOS` is pressed, and only when the measurement recorded that the text is the
button's own caption: this tool measures, so it opens read-only views and never a
label that commits the window's state, destroys it or starts a flow it cannot unwind,
nor a label OCR read off the pixels of a control it cannot name. Every button it does
not press is recorded with its motive, so the reader of the JSON tells «not pressed»
apart from «pressed and opened nothing». At most `MAX_PULSACIONES` presses per target.
"""

from __future__ import annotations

from collections.abc import Collection

from medir_cliente import Cliente, Json
from medir_inventario import origen_de_boton
from medir_proyectos import Objetivo, abrir
from medir_ventanas import (
    MANTENER,
    _cerrar,
    _dormir,
    _error_de,
    _responder,
    clases,
    normalizar_etiqueta,
    ventanas,
)

# White list, not a black list: a label this tool does not know is left alone.
# A black list misses the next synonym, and the windows add one every release: one
# unknown label that commits the window (Aceptar, Sí, Reemplazar, Simular, Ejecutar,
# Calcular, Actualizar, Despejar, Crear, Elaborar, Conversión, Abortar) turns a
# measurement into an edit. The list is exact, not by prefix: a prefix of «ver»
# presses «Ver el TNCO de todos modos», which skips a warning and writes the thermal
# parameter, and a prefix of «abrir» presses «Abrir sitio», which opens a browser or
# the Windows Explorer, a window outside the process. It is a ceiling drawn from the
# labels the inventory lists, not a record of measured effects: the run itself
# records in `botones_abren` the window each label it presses opens, and a label
# whose recorded window is not a read-only view comes out of the list.
# Compared in lower case, without accents, `&` or ellipsis, with spaces collapsed.
PERMITIDOS = (
    "resumen del sistema",
    "grafico de perdidas",
    "grafico",
    "tabla",
    "tabla mensual",
    "mostrar grafico",
    "mostrar histograma",
    "mostrar optimizacion",
    "curva i/v",
    "diagrama unifilar",
)
# A form with many tabs lists hundreds of buttons and each press costs a click, two
# window reads and a wait; the cap keeps one target from eating the whole session.
MAX_PULSACIONES = 40

# A button whose own caption is its text (`window_text` in a form read, `control` in a
# click answer) names one control. Any other origin is text recognised inside it or the
# name UI Automation gives it, and `clic_texto` then falls back to OCR substring and to
# UIA: «Tabla» can reach «Guardar tabla...», so the label names no control.
ORIGENES_DE_CAPTION = ("window_text", "control")


def huellas(c: Cliente) -> set[tuple[str, str]]:
    """Class and title of the open windows: what a press is compared against."""
    return {(w["class"], str(w.get("title") or "")) for w in ventanas(c)}


def _nuevas(
    abiertas: Collection[Json], previas: Collection[tuple[str, str]]
) -> list[Json]:
    """The open windows that were not there before the press, the main ones aside."""
    return [
        w
        for w in abiertas
        if w["class"] not in MANTENER
        and (w["class"], str(w.get("title") or "")) not in previas
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
    """Close only the windows that were not open before; entries and a stop reason.

    A window is new when its `(class, title)` pair was not in `previas`: comparing
    the class alone missed a second window of a class that was already open and
    collapsed two new windows of that class into one. A new window of a class
    `MANTENER` keeps is not closed at all: it is a stop naming the window, since one
    more window of that class would leave the close choosing between them. Each close
    asks by class and never by title: a title is also matched by substring over every
    window of the process, so a new window titled like the host would close the host.
    The main windows of `MANTENER` are never closed. `limpiar` closes everything but
    the main windows, host included, so it cannot run between two presses; this one
    leaves `previas` alone. A class with two new windows keeps its two entries even
    when one close removes both from the screen. A dialog opened by the press is
    answered with a rejection (`_responder`); when it offers none it stays open and its
    text is the stop reason. Twelve rounds that leave a window open are a stop too:
    its reason names what is still open and the refusal `accion:cerrar` answered,
    because a press that left a modal open is not a press that opened a window
    cleanly. Every press after it fails on the disabled window and would blame the
    label.
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


def _permitido(texto: str) -> bool:
    """Whether a button text is exactly one entry of the white list."""
    normal = normalizar_etiqueta(texto)
    return any(normal == normalizar_etiqueta(p) for p in PERMITIDOS)


def _motivo(boton: Json) -> str | None:
    """Why this run does not press the button, or None when it does.

    The text must come from the button's own caption and the entry must say so: one
    recorded before the origin was is not pressed until the window is measured again.
    """
    texto = str(boton.get("texto") or "").strip()
    if not texto:
        return "vacío"
    if boton.get("habilitado") is not True:
        return "deshabilitado"
    origen = origen_de_boton(boton)
    if origen is None:
        return "sin origen"
    if origen not in ORIGENES_DE_CAPTION:
        return "texto de OCR"
    return None if _permitido(texto) else "no está en la lista blanca"


def _abre(nuevas: list[Json], cerradas: list[Json]) -> list[Json]:
    """The windows a press opened, with the text that dismissed each one."""
    por_huella = {(e["clase"], e["titulo"]): e for e in cerradas}
    salida: list[Json] = []
    for w in nuevas:
        entrada: Json = {"clase": w["class"], "titulo": w.get("title")}
        cierre = por_huella.get((w["class"], w.get("title")))
        if cierre is not None and "cierra" in cierre:
            entrada["cierra"] = cierre["cierra"]
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
    previas = huellas(c)
    r = c.tool("gui_actuar", {"accion": "clic_texto", "ventana": clase, "texto": texto})
    _dormir(0.8)
    nuevas = [
        w for w in ventanas(c) if (w["class"], str(w.get("title") or "")) not in previas
    ]
    cerradas, parado = cerrar_nuevas(c, previas)
    entrada: Json = {
        "boton": texto,
        "abre": _abre(nuevas, cerradas),
        "error": parado or _error_de(r),
    }
    if parado is not None:
        entrada["parado"] = parado
    return entrada


def _reabrir(c: Cliente, objetivo: Objetivo, pestana: Json) -> str | None:
    """Open the host window again when the press closed it; an error text, or None."""
    clase = objetivo.ventana
    if clase in clases(c):
        return None
    # The reopened window starts on its first tab: select the one being pressed.
    return abrir(c, objetivo) or _abrir_pestana(c, clase, pestana["indice"])


def pulsar_botones(c: Cliente, objetivo: Objetivo, medida: Json) -> list[Json]:
    """Press every allowed button of the measured tabs: the `botones_abren` list.

    One entry per press, plus a `parado` entry when a press closes the host and it
    cannot be reopened or its tab cannot be selected again. Those failures stay
    here and not in `medida["errores"]`: the fields already measured are valid and
    are still merged.
    """
    clase = objetivo.ventana
    registros: list[Json] = []
    pulsados = 0
    quedan = 0
    for pestana in medida["pestanas"]:
        for boton in pestana["botones"]:
            texto = str(boton.get("texto") or "").strip()
            motivo = _motivo(boton)
            if motivo is not None:
                registros.append({"boton": texto, "pulsado": False, "motivo": motivo})
                continue
            if pulsados >= MAX_PULSACIONES:
                quedan += 1
                registros.append({"boton": texto, "pulsado": False, "motivo": "tope"})
                continue
            pulsados += 1
            # A press can switch the tab: the buttons on screen are not always the
            # ones this tab lists, and a text shared by two tabs would hit the wrong one.
            previas = huellas(c)
            fallo = _abrir_pestana(c, clase, pestana["indice"])
            if fallo:
                cerrar_nuevas(c, previas)
                registros.append({"pestana": pestana["pestana"], "parado": fallo})
                return registros
            registros.append(
                {"pestana": pestana["pestana"], **_pulsar(c, clase, texto)}
            )
            if "parado" in registros[-1]:
                return registros
            fallo = _reabrir(c, objetivo, pestana)
            if fallo:
                registros.append(
                    {
                        "pestana": pestana["pestana"],
                        "parado": f"after pressing {texto!r}: {fallo}",
                    }
                )
                return registros
    if quedan:
        registros.append({"truncado": True, "quedan": quedan})
    return registros
