"""Reading every tab of an open window into one entry of the tab inventory.

`leer_pagina` reads the tab on screen, `inventariar` walks the tabs of the window and
records the error of every tab PVsyst vetoes, and `fusionar` with `volcar` put the
entry into `docs/gui_inventario_pestanas.json`; `tools/mapa_desde_inventario.py`
regenerates the map from it.
"""

from __future__ import annotations

import json

from medir_cliente import Cliente, Json
from medir_ventanas import _dormir, _error_de, ventanas

CLASES_ROTULO = ("TTabSheet", "TGroupBox", "TRadioGroup")
CLAVES_CAMPO = ("clase", "etiqueta", "etiqueta_fuente", "marcado", "habilitado")


class ErrorMedicion(Exception):
    """A read or a step failed: the measurement of this target cannot be trusted."""


def origen_de_boton(boton: Json) -> str | None:
    """Where the server says the text of a button was read, or None.

    A form read names it `etiqueta_fuente` per control: `window_text` when the control
    carries the caption itself, `ocr` when the text was recognised inside it. A click
    answer names it `origen`: `control`, `ocr` or `uia`. A server that reports neither,
    and a measurement recorded before it did, answer None.
    """
    for clave in ("origen", "etiqueta_fuente"):
        valor = boton.get(clave)
        if isinstance(valor, str) and valor:
            return valor
    return None


def leer_pagina(c: Cliente, clase: str) -> Json:
    f = c.tool("gui_observar", {"que": "formulario", "ventana": clase})
    fallo = _error_de(f)
    if fallo is not None:
        raise ErrorMedicion(f"reading {clase} failed: {fallo[:200]}")
    return {
        "campos": [{k: x.get(k) for k in CLAVES_CAMPO} for x in f.get("campos") or []],
        "botones": [
            {
                "texto": b.get("texto"),
                "habilitado": b.get("habilitado"),
                "origen": origen_de_boton(b),
            }
            for b in f.get("botones") or []
        ],
        "rotulos": [
            [o["clase"], o.get("texto")]
            for o in f.get("otros") or []
            if o.get("clase") in CLASES_ROTULO and o.get("texto")
        ],
        "titulo": f.get("titulo"),
        "pestanas": [p["nombre"] for p in f.get("pestanas") or []],
    }


def _cerrar_dialogos(c: Cliente, indice: int, errores: list[Json]) -> None:
    """A tab change can raise a message box (the window vetoes it): note it, dismiss it."""
    for _ in range(3):
        if not any(w["class"] == "TMessageForm" for w in ventanas(c)):
            return
        m = c.tool("gui_observar", {"que": "formulario", "ventana": "TMessageForm"})
        errores.append({"indice": indice, "dialogo": m.get("textos")})
        c.tool(
            "gui_actuar",
            {"accion": "clic_texto", "ventana": "TMessageForm", "texto": "OK"},
        )
        _dormir(0.5)


def _no_es_pagina(respuesta: Json) -> bool:
    """`formulario.pestanas` also lists nested sheets and panels (the pumping
    sizing suggestions); the page control answers that index is out of range."""
    texto = str(respuesta.get("error", ""))
    return "out of range" in texto and "tab pages" in texto


def inventariar(c: Cliente, clase: str) -> Json:
    """Every tab of the open window: fields, buttons and captions of the tab."""
    if c.tool("gui_observar", {"que": "formulario", "ventana": clase}).get("pestanas"):
        c.tool("gui_actuar", {"accion": "pagina", "ventana": clase, "indice": 0})
        _dormir(0.5)
    base = leer_pagina(c, clase)
    nombres: list[str] = base["pestanas"]
    errores: list[Json] = []
    pestanas: list[Json] = []
    for i in range(len(nombres) or 1):
        if nombres and i > 0:
            r = c.tool(
                "gui_actuar", {"accion": "pagina", "ventana": clase, "indice": i}
            )
            if _no_es_pagina(r):
                continue
            if r.get("error") or r.get("jsonrpc_error"):
                errores.append(
                    {"indice": i, "pestana": nombres[i], "texto": str(r)[:300]}
                )
                # The tab did not change: the screen shows another tab's page.
                _dormir(0.5)
                _cerrar_dialogos(c, i, errores)
                continue
            _dormir(0.5)
            _cerrar_dialogos(c, i, errores)
        p = leer_pagina(c, clase) if i > 0 else base
        pestanas.append(
            {
                "indice": i if nombres else None,
                "pestana": nombres[i] if nombres else None,
                **{k: p[k] for k in ("campos", "botones", "rotulos")},
            }
        )
    if nombres:
        c.tool("gui_actuar", {"accion": "pagina", "ventana": clase, "indice": 0})
    return {
        "ventana": clase,
        "titulo": base["titulo"],
        "pestanas": pestanas,
        "errores": errores,
    }


def volcar(datos: Json) -> str:
    return json.dumps(datos, indent=1, ensure_ascii=False)


def fusionar(inventario: Json, clave: str, medida: Json) -> None:
    anterior = inventario.get(clave) or {}
    entrada = {k: medida[k] for k in ("ventana", "titulo", "pestanas", "errores")}
    if "botones_abren" in medida:
        entrada["botones_abren"] = medida["botones_abren"]
    elif "botones_abren" in anterior:
        # A run without --botones did not look at the presses: keeping the old list
        # is not stale data, it is the last measurement of what they open.
        entrada["botones_abren"] = anterior["botones_abren"]
    inventario[clave] = entrada
