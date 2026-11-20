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

from collections.abc import Collection, Iterator

from medir_cierres import cerrar_nuevas
from medir_cliente import Cliente, Json
from medir_inventario import origen_de_boton
from medir_proyectos import Objetivo, abrir
from medir_ventanas import (
    ErrorDeLectura,
    _dormir,
    _error_de,
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

# `clic_texto` answers in `origen` how it resolved the text it pressed: `control` is a
# visible button whose own caption it is, and it is the only value this run may go on
# with. `ocr` and `uia` are its fallbacks (`clic_por_texto.rs`) and press a text.
ORIGEN_DE_CONTROL = "control"


def huellas(c: Cliente) -> set[tuple[str, str]]:
    """Class and title of the open windows: what a press is compared against."""
    return {(w["class"], str(w.get("title") or "")) for w in ventanas(c)}


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


def _fallo_de_origen(texto: str, r: Json, nuevas: Collection[Json]) -> str | None:
    """Why the click answer does not vouch for the button, or None.

    `clic_texto` presses a visible button whose own caption is the text, and otherwise
    falls back to OCR substring matching and to the name UI Automation gives it,
    naming the path it took in `origen`. A fallback pressed a text: the button was
    hidden or covered, so `«Tabla»` could reach `«Guardar tabla...»` and the label
    names no control of this window. A click the server refused answers with an error
    and no origin, and its error is already the press's reason.
    """
    origen = r.get("origen")
    if origen == ORIGEN_DE_CONTROL:
        return None
    if origen is None and _error_de(r) is not None:
        return None
    abiertas = ", ".join(f"{w['class']} {w.get('title')!r}" for w in nuevas)
    return (
        f"after pressing {texto!r}: the click answer says "
        f"origen={origen or 'ninguno'}, so it pressed a text and not the caption of a "
        f"visible button of this window; it opened {abiertas or 'nothing'}"
    )


def _pulsar(c: Cliente, clase: str, texto: str) -> Json:
    """Press one button of the window and dismiss what it opened: its entry.

    The entry stops the run when the answer says the press was resolved by OCR or by
    UI Automation (see `_fallo_de_origen`) or when a window it opened could not be
    closed: the fields already measured still merge, the presses of this target do
    not go on.
    """
    previas = huellas(c)
    r = c.tool("gui_actuar", {"accion": "clic_texto", "ventana": clase, "texto": texto})
    _dormir(0.8)
    nuevas = [
        w for w in ventanas(c) if (w["class"], str(w.get("title") or "")) not in previas
    ]
    cerradas, parado = cerrar_nuevas(c, previas)
    parado = parado or _fallo_de_origen(texto, r, nuevas)
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


def _pulsaciones(c: Cliente, objetivo: Objetivo, medida: Json) -> Iterator[Json]:
    """The entries of one press run, up to the press that stops it."""
    clase = objetivo.ventana
    pulsados = 0
    quedan = 0
    for pestana in medida["pestanas"]:
        for boton in pestana["botones"]:
            texto = str(boton.get("texto") or "").strip()
            motivo = _motivo(boton)
            if motivo is not None:
                yield {"boton": texto, "pulsado": False, "motivo": motivo}
                continue
            if pulsados >= MAX_PULSACIONES:
                quedan += 1
                yield {"boton": texto, "pulsado": False, "motivo": "tope"}
                continue
            pulsados += 1
            # A press can switch the tab: the buttons on screen are not always the
            # ones this tab lists, and a text shared by two tabs would hit the wrong one.
            previas = huellas(c)
            fallo = _abrir_pestana(c, clase, pestana["indice"])
            if fallo:
                cerrar_nuevas(c, previas)
                yield {"pestana": pestana["pestana"], "parado": fallo}
                return
            entrada = {"pestana": pestana["pestana"], **_pulsar(c, clase, texto)}
            yield entrada
            if "parado" in entrada:
                return
            fallo = _reabrir(c, objetivo, pestana)
            if fallo:
                yield {
                    "pestana": pestana["pestana"],
                    "parado": f"after pressing {texto!r}: {fallo}",
                }
                return
    if quedan:
        yield {"truncado": True, "quedan": quedan}


def pulsar_botones(c: Cliente, objetivo: Objetivo, medida: Json) -> list[Json]:
    """Press every allowed button of the measured tabs: the `botones_abren` list.

    One entry per press, plus a `parado` entry when a press stops the run: the host
    it closed cannot be reopened, its tab cannot be selected again, the click answer
    does not say `control`, a window it opened will not close, or the screen can no
    longer be read. Those failures stay here and not in `medida["errores"]`: the
    fields already measured are valid and are still merged, together with the entries
    the presses produced before the stop.
    """
    registros: list[Json] = []
    try:
        for entrada in _pulsaciones(c, objetivo, medida):
            registros.append(entrada)
    except ErrorDeLectura as e:
        registros.append({"parado": str(e)})
    return registros
