"""Which fields a switch of a tab shows, hides, enables or disables.

The window map is a photo of one state; `docs/gui_condicionales.json` says what each
known switch changes in that photo. For every entry of `tools/gui_condicionales.json`
the window is opened by its own chain (as in the manifest), the tab is read, each step
(`{etiqueta, valor}`: a checkbox set or a combo item chosen) is applied in order and the
tab is read again after it. Nothing is undone and nothing is saved: closing the window
discards the change (the shipped `_DEMO_*` projects are read-only and every save prompt
is answered No). A `valor` of `"alternar"` sets the checkbox to the opposite of what the
tab shows.
"""

from __future__ import annotations

import argparse
import json
import sys
from collections.abc import Callable
from pathlib import Path
from tempfile import mkdtemp
from typing import Any

# `python -I` leaves the script's directory out of `sys.path`; the modules of this
# tool are its siblings.
sys.path.insert(0, str(Path(__file__).resolve().parent))

from medir_cliente import Cliente, ClienteMcp, Json  # noqa: E402
from medir_inventario import leer_pagina, volcar  # noqa: E402
from medir_proyectos import Objetivo, Propiedad, abrir  # noqa: E402
from medir_ventanas import _dormir, asegurar_pvsyst, limpiar  # noqa: E402

Campos = dict[tuple[str, str], bool]
ESPECIFICACION = Path(__file__).resolve().parent / "gui_condicionales.json"
SALIDA = Path(__file__).resolve().parent.parent / "docs" / "gui_condicionales.json"


def campos_de(pagina: Json) -> Campos:
    """`(clase, etiqueta)` of every labelled field and button of a read tab -> is it
    enabled (a button has the class `boton`)."""
    campos = {
        (c["clase"], c["etiqueta"]): c.get("habilitado") is not False
        for c in pagina["campos"]
        if c.get("etiqueta")
    }
    campos.update(
        {
            ("boton", b["texto"]): b.get("habilitado") is not False
            for b in pagina["botones"]
            if b.get("texto")
        }
    )
    return campos


def diferencia(antes: Campos, despues: Campos) -> Json:
    """What a step changed: labels that appeared or went away, and enable flips."""

    def nombres(claves: set[tuple[str, str]]) -> list[str]:
        return sorted({etiqueta for _, etiqueta in claves})

    comunes = antes.keys() & despues.keys()
    return {
        "aparecen": nombres(despues.keys() - antes.keys()),
        "desaparecen": nombres(antes.keys() - despues.keys()),
        "habilitan": nombres({k for k in comunes if despues[k] and not antes[k]}),
        "deshabilitan": nombres({k for k in comunes if antes[k] and not despues[k]}),
    }


def _valor(paso: Json, pagina: Json) -> Any:
    """The value to set: `"alternar"` is the opposite of the checkbox on screen."""
    if paso["valor"] != "alternar":
        return paso["valor"]
    marcados = [
        c.get("marcado")
        for c in pagina["campos"]
        if c.get("etiqueta") == paso["etiqueta"]
    ]
    return not marcados[0] if marcados else paso["valor"]


def _aplicar(c: Cliente, clase: str, paso: Json, valor: Any) -> str | None:
    """Set one switch; the server's error text, or None."""
    argumentos: Json = {
        "accion": "campo",
        "ventana": clase,
        "etiqueta": paso["etiqueta"],
        # The tool takes the value as text: a checkbox is "true" or "false".
        "valor": str(valor).lower() if isinstance(valor, bool) else valor,
    }
    r = c.tool("gui_actuar", argumentos)
    _dormir(0.8)
    return str(r["error"])[:200] if r.get("error") else None


def medir_condiciones(c: Cliente, clase: str, pasos: list[Json]) -> list[Json]:
    """Apply `pasos` in order on the tab on screen; one entry per step, with the
    switch, the value set and the change it made. A step that fails stops the walk
    and is the last entry, with `error`."""
    resultado: list[Json] = []
    pagina = leer_pagina(c, clase)
    for paso in pasos:
        valor = _valor(paso, pagina)
        error = _aplicar(c, clase, paso, valor)
        entrada: Json = {"campo": paso["etiqueta"], "valor": valor}
        if error:
            resultado.append({**entrada, "error": error})
            break
        nueva = leer_pagina(c, clase)
        resultado.append({**entrada, **diferencia(campos_de(pagina), campos_de(nueva))})
        pagina = nueva
    return resultado


def _ir_a_la_pestana(c: Cliente, clase: str, nombre: str | None) -> str | None:
    """Select the tab called `nombre` (None: the one on screen); an error text or None."""
    if nombre is None:
        return None
    f = c.tool("gui_observar", {"que": "formulario", "ventana": clase})
    nombres = [p["nombre"] for p in f.get("pestanas") or []]
    if nombre not in nombres:
        return f"the tab {nombre!r} is not among {nombres}"
    c.tool(
        "gui_actuar",
        {"accion": "pagina", "ventana": clase, "indice": nombres.index(nombre)},
    )
    _dormir(0.5)
    return None


def medir_entrada(c: Cliente, entrada: Json, propiedad: Propiedad) -> Json | str:
    """Open the window, go to the tab, walk the steps, close: the entry or an error."""
    clase = entrada["ventana"]
    previas = limpiar(c)
    if previas:
        return f"could not close the windows left open: {previas}"
    try:
        objetivo = Objetivo(entrada["clave"], clase, entrada["cadena"], propiedad)
        fallo = abrir(c, objetivo) or _ir_a_la_pestana(c, clase, entrada.get("pestana"))
        if fallo:
            return fallo
        condiciones = medir_condiciones(c, clase, entrada["pasos"])
    finally:
        limpiar(c)
    return {
        "ventana": clase,
        # The last step of the chain is the function the map names as `abierta_con`.
        "abierta_con": entrada["cadena"].split(",")[-1],
        "pestana": entrada.get("pestana"),
        "condiciones": condiciones,
    }


def main(
    argv: list[str] | None = None,
    fabrica: Callable[[Path, Path], Any] = ClienteMcp,
) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--servidor", type=Path, required=True)
    parser.add_argument("--especificacion", type=Path, default=ESPECIFICACION)
    parser.add_argument("--salida", type=Path, default=SALIDA)
    args = parser.parse_args(argv)
    entradas: list[Json] = json.loads(args.especificacion.read_text(encoding="utf-8"))
    workspace = Path(mkdtemp(prefix="pvsyst_cond_")) / "PVsyst8.1_Data"
    workspace.mkdir(parents=True)
    c = fabrica(args.servidor, workspace)
    medidas: Json = {}
    fallos = 0
    try:
        arranque = asegurar_pvsyst(c)
        if arranque:
            print(arranque)
            return 1
        # The projects this run opens itself: the only non-demo ones it may close later.
        propiedad = Propiedad()
        for e in entradas:
            r = medir_entrada(c, e, propiedad)
            print(f"{e['clave']}: {r if isinstance(r, str) else 'measured'}")
            if isinstance(r, str):
                fallos += 1
            else:
                medidas[e["clave"]] = r
    finally:
        c.cerrar()
    args.salida.write_text(volcar(medidas) + "\n", encoding="utf-8")
    return 1 if fallos else 0


if __name__ == "__main__":
    raise SystemExit(main())
