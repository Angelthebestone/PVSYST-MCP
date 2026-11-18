"""Regenerates, in `docs/gui_mapa.json`, the windows that the tab inventory measured.

`docs/gui_inventario_pestanas.json` lists EVERY field of a window, tab by tab
(Win32 enumeration, no OCR, no truncation). The old collector of the map kept
at most 13 fields per page, so for the windows of the inventory this script
replaces the pages of the map with the measured ones and leaves the rest of the
file byte for byte as it was.

What the inventory brings and the map keeps:
  - `campos`, `botones` (with `habilitado`) and `indice_pestana` come from the
    inventory; each field also keeps its `marcado` and `habilitado`.
  - `rotulos` and `sin_rotulo` are not in the inventory: a page keeps the ones
    the map had when the tab of the same name still exists (and its state did
    not change, see `Remedio.sin_rotulos_de`); a new tab has none.
  - `titulo` and `abierta_con` of the instance stay as the map had them.

Usage:
    python tools/mapa_desde_inventario.py [--check]

`--check` writes nothing and exits 1 when the map is not what this script
generates, so a hand-patched or stale map fails instead of drifting. Running it
twice leaves the same bytes as running it once.
"""

from __future__ import annotations

import argparse
import copy
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

Json = dict[str, Any]

RAIZ = Path(__file__).resolve().parent.parent
MAPA = RAIZ / "docs" / "gui_mapa.json"
INVENTARIO = RAIZ / "docs" / "gui_inventario_pestanas.json"


@dataclass(frozen=True)
class Remedio:
    """How the inventory of one window enters the map."""

    # Add the inventory as a new instance opened with this key, keeping the
    # instances the map has. None: replace instance 0, the one opened by the
    # `variante.*` function the inventory was measured through.
    anadir_con: str | None = None
    # Tabs of the replaced instance whose `rotulos` describe another state of
    # the window (controls the inventory shows hidden), so they are dropped.
    sin_rotulos_de: frozenset[str] = frozenset()


# Why each entry: INFORME_PRUEBAS_GUI_VISUAL_2026-11-17.md, sections 2.3, 2.4, 2.6.
REMEDIOS: dict[str, Remedio] = {
    "TDetailedLossesForm": Remedio(),
    "TSystemegridForm": Remedio(),
    # The map was measured with `Activo` ticked in P50-P90 (17 fields); the
    # inventory has it unticked (1 field): the old captions describe the ticked page.
    "TSystemeAddForm": Remedio(sin_rotulos_de=frozenset({"Estimación P50 - P90"})),
    # The map has the tracker variant; the inventory is the fixed-tilt one.
    "TOrientationForm": Remedio(anadir_con="variante.orientacion"),
    "THorizonForm": Remedio(),
    "TShadingsForm": Remedio(),
    "TEcoEvaluationForm": Remedio(),
    "TUserNeedsForm": Remedio(),
    "TSummaryForm": Remedio(),
    "TGridStorageForm": Remedio(),
}


def cargar(ruta: Path) -> Json:
    datos: Json = json.loads(ruta.read_bytes())
    return datos


def serializar(mapa: Json) -> bytes:
    """The map's own format: one line, no spaces, UTF-8, no final newline."""
    return json.dumps(mapa, ensure_ascii=False, separators=(",", ":")).encode("utf-8")


def _pagina(pestana: Json, heredada: Json | None) -> Json:
    pagina: Json = {
        "pestana": pestana["pestana"],
        "campos": copy.deepcopy(pestana["campos"]),
        "botones": copy.deepcopy(pestana["botones"]),
    }
    for clave in ("rotulos", "sin_rotulo"):
        if heredada is not None and clave in heredada:
            pagina[clave] = copy.deepcopy(heredada[clave])
    pagina["indice_pestana"] = pestana["indice"]
    return pagina


def _instancia(
    medida: Json, titulo: str, abierta_con: str, heredables: dict[str | None, Json]
) -> Json:
    pestanas = medida["pestanas"]
    return {
        "titulo": titulo,
        "abierta_con": abierta_con,
        "pestanas": [p["pestana"] for p in pestanas if p["pestana"] is not None],
        "paginas": [_pagina(p, heredables.get(p["pestana"])) for p in pestanas],
    }


def _viene_del_inventario(instancia: Json) -> bool:
    """Only inventory fields carry `habilitado`; the OCR collector had no such key."""
    return any(
        "habilitado" in campo
        for pagina in instancia["paginas"]
        for campo in pagina["campos"]
    )


def _controles(ventana: Json) -> list[Any]:
    """Captions of the whole window: those of every page, first seen first."""
    vistos: list[Any] = []
    for instancia in ventana["instancias"]:
        for pagina in instancia["paginas"]:
            vistos.extend(r for r in pagina.get("rotulos", []) if r not in vistos)
    return vistos


def _remedir_ventana(clase: str, ventana: Json, medida: Json, remedio: Remedio) -> None:
    if medida["errores"]:
        raise ValueError(f"{clase}: the inventory reports errors {medida['errores']}")
    instancias: list[Json] = ventana["instancias"]
    if remedio.anadir_con is not None:
        instancias[:] = [i for i in instancias if not _viene_del_inventario(i)]
        instancias.append(_instancia(medida, medida["titulo"], remedio.anadir_con, {}))
    else:
        vieja = instancias[0]
        heredables = {
            p["pestana"]: p
            for p in vieja["paginas"]
            if p["pestana"] not in remedio.sin_rotulos_de
        }
        instancias[0] = _instancia(
            medida, vieja["titulo"], vieja["abierta_con"], heredables
        )
    ventana["controles"] = _controles(ventana)


def remedir(mapa: Json, inventario: Json) -> Json:
    """A copy of `mapa` with the windows of `inventario` regenerated."""
    nuevo = copy.deepcopy(mapa)
    for clase, medida in inventario.items():
        if clase not in nuevo["ventanas"] or clase not in REMEDIOS:
            raise KeyError(f"{clase}: not in the map or in REMEDIOS")
        _remedir_ventana(clase, nuevo["ventanas"][clase], medida, REMEDIOS[clase])
    return nuevo


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument(
        "--check", action="store_true", help="write nothing; exit 1 if stale"
    )
    parser.add_argument("--mapa", type=Path, default=MAPA)
    parser.add_argument("--inventario", type=Path, default=INVENTARIO)
    args = parser.parse_args(argv)

    esperado = serializar(remedir(cargar(args.mapa), cargar(args.inventario)))
    if args.mapa.read_bytes() == esperado:
        return 0
    if args.check:
        print(
            f"{args.mapa} is not what {args.inventario.name} generates: run {sys.argv[0]}"
        )
        return 1
    args.mapa.write_bytes(esperado)
    print(f"rewrote {args.mapa}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
