"""Measure PVsyst windows through the MCP server and merge them into the inventory.

For each target of the manifest (`tools/gui_manifiesto.json`) it opens the window by
its chain of `gui_actuar funcion`, reads every tab with `gui_observar que:formulario`,
closes whatever it opened and writes the result into `docs/gui_inventario_pestanas.json`
under the target's `clave`. `tools/mapa_desde_inventario.py` then regenerates the map.

    python tools/medir_gui.py --servidor <pvsyst-mcp.exe> [--solo CLAVE ...]

A chain step `@proyecto:NOMBRE` loads that project through the search box of the
project loader (PVsyst's shipped `_DEMO_*` projects are read-only, so nothing is
saved). A chain without one closes the loaded project first, and only a project this
run loaded itself or a demo is ever closed: any other may hold the user's unsaved
work, so the chain stops naming it instead of answering No to its save prompt.
Only one agent may drive the PVsyst GUI at a time. Nothing is filled in or saved.

`--botones` also presses the enabled buttons whose label is exactly one entry of a
white list of read-only views (`Resumen del sistema`, `Gráfico de pérdidas`,
`Gráfico`, `Tabla`, `Tabla mensual`, `Mostrar gráfico`, `Mostrar histograma`,
`Mostrar optimización`, `Curva I/V`, `Diagrama unifilar`), one `gui_actuar
accion:clic_texto` per button, and records in the entry's `botones_abren` the windows
each press opened, closing only those and reopening the host when the button closed
it: most windows of the map are reached through a button, so reading the labels alone
leaves them unmeasured. The list is exact, not by prefix: a prefix of `Ver` presses
«Ver el TNCO de todos modos», which skips a warning and writes the thermal parameter,
and a prefix of `Abrir` presses «Abrir sitio» or «Abrir en explorer», which open a
browser or the Windows Explorer, a window outside the process. A label the list does
not hold is never pressed, because it may commit the window's state, destroy it or
start a flow this script cannot unwind, and the verbs that do that (`OK`, `Aceptar`,
`Sí`, `Guardar`, `Aplicar`, `Eliminar`, `Simular`…) change with every release. Neither
is a label the measurement did not record as the button's own caption: text OCR read
inside an icon-only button reaches another control by substring. Each button it does
not press is recorded with its motive (`no está en la lista blanca`, `texto de OCR`,
`sin origen`, `deshabilitado`, `vacío`, `tope`), so the JSON tells «not pressed» apart
from «pressed and opened nothing». A press run that stops is merged all the same, but
it leaves the exit code non-zero: the windows it could not clean say the measurement
is incomplete. At most `MAX_PULSACIONES` presses per target.

This module is the command line only: the target, the client, the windows, the tabs,
the projects, the presses and the closes live in the sibling modules
`medir_objetivo.py`, `medir_cliente.py`, `medir_ventanas.py`, `medir_proyectos.py`,
`medir_inventario.py`, `medir_botones.py` and `medir_cierres.py`. The transport
(`ClienteMcp`) is exercised live only; the rest takes any object with
`tool(nombre, argumentos)`, which the tests fake.
"""

from __future__ import annotations

import argparse
import json
import sys
import tempfile
from collections.abc import Callable
from pathlib import Path
from typing import Any

# `python -I tools/medir_gui.py` leaves the script's directory out of `sys.path` and
# the modules of this tool are its siblings, so the imports below need it first.
sys.path.insert(0, str(Path(__file__).resolve().parent))

from medir_cliente import ClienteMcp, Json  # noqa: E402
from medir_inventario import fusionar, volcar  # noqa: E402
from medir_objetivo import medir  # noqa: E402
from medir_proyectos import Objetivo, Propiedad  # noqa: E402
from medir_ventanas import asegurar_pvsyst  # noqa: E402

RAIZ = Path(__file__).resolve().parent.parent
INVENTARIO = RAIZ / "docs" / "gui_inventario_pestanas.json"
MANIFIESTO = Path(__file__).resolve().parent / "gui_manifiesto.json"


def _argumentos(argv: list[str] | None) -> argparse.Namespace:
    """The command line of the tool."""
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--servidor", type=Path, required=True)
    parser.add_argument("--manifiesto", type=Path, default=MANIFIESTO)
    parser.add_argument("--inventario", type=Path, default=INVENTARIO)
    parser.add_argument("--solo", nargs="*", help="measure only these `clave`s")
    parser.add_argument(
        "--botones",
        action="store_true",
        help="press the allowed buttons of every tab and record what they open",
    )
    return parser.parse_args(argv)


def _objetivos(
    manifiesto: Path, solo: list[str] | None, propiedad: Propiedad
) -> list[Objetivo] | None:
    """The targets to measure; None when `--solo` names a `clave` it does not have."""
    entradas: list[Json] = json.loads(manifiesto.read_text(encoding="utf-8"))
    if solo:
        desconocidas = set(solo) - {e["clave"] for e in entradas}
        if desconocidas:
            print(f"not in the manifest: {sorted(desconocidas)}")
            return None
        entradas = [e for e in entradas if e["clave"] in solo]
    return [
        Objetivo(
            clave=e["clave"],
            ventana=e["ventana"],
            cadena=e["cadena"],
            propiedad=propiedad,
        )
        for e in entradas
    ]


def _leer_inventario(ruta: Path) -> Json | None:
    """The inventory, or None when the file is not in the format this tool writes."""
    texto = ruta.read_text(encoding="utf-8")
    inventario: Json = json.loads(texto)
    if volcar(inventario) != texto.replace("\r\n", "\n").rstrip("\n"):
        print(f"{ruta} is not in the format this tool writes")
        return None
    return inventario


def _paradas(medida: Json) -> list[Json]:
    """The entries that stopped the presses: what is still open, or where it failed."""
    return [e for e in medida.get("botones_abren") or [] if "parado" in e]


def _resumen(clave: str, medida: Json) -> str:
    """The line `main` prints for one measured target.

    `presses` counts only entries that pressed a button: the list also carries the
    buttons it refused and the truncation markers. A press error is not an
    inventory error (it does not block the merge), so it is counted apart; a stopped
    entry is named too, because it is what turns the exit code non-zero.
    """
    n = sum(len(p["campos"]) for p in medida["pestanas"])
    pulsados = ""
    if "botones_abren" in medida:
        prensas = [e for e in medida["botones_abren"] if "abre" in e]
        fallidas = [e for e in prensas if e.get("error")]
        pulsados = f", {len(prensas)} presses, {len(fallidas)} with error"
        paradas = _paradas(medida)
        if paradas:
            pulsados += f", {len(paradas)} stopped"
    return (
        f"{clave}: {len(medida['pestanas'])} tabs, {n} fields, "
        f"{len(medida['errores'])} errors{pulsados}"
    )


def main(
    argv: list[str] | None = None,
    fabrica: Callable[[Path, Path], Any] = ClienteMcp,
) -> int:
    args = _argumentos(argv)
    # The projects this run opens itself: the only non-demo ones it may close later.
    propiedad = Propiedad()
    objetivos = _objetivos(args.manifiesto, args.solo, propiedad)
    if objetivos is None:
        return 2
    inventario = _leer_inventario(args.inventario)
    if inventario is None:
        return 2

    workspace = Path(tempfile.mkdtemp(prefix="pvsyst_medir_")) / "PVsyst8.1_Data"
    workspace.mkdir(parents=True)
    c = fabrica(args.servidor, workspace)
    fallos = 0
    fusionados = 0
    try:
        fallo_arranque = asegurar_pvsyst(c)
        if fallo_arranque:
            print(fallo_arranque)
            return 1
        for o in objetivos:
            try:
                r = medir(c, o, args.botones)
            except (RuntimeError, TimeoutError) as e:
                # The server is gone, stuck or could not read the screen: keep what was
                # measured, stop here.
                print(f"{o.clave}: {e}")
                fallos += 1
                break
            if isinstance(r, str):
                print(f"{o.clave}: {r}")
                fallos += 1
                continue
            fusionar(inventario, o.clave, r)
            fusionados += 1
            print(_resumen(o.clave, r))
            if _paradas(r):
                fallos += 1
    finally:
        c.cerrar()
        if fusionados:
            args.inventario.write_text(volcar(inventario) + "\n", encoding="utf-8")
    return 1 if fallos else 0


if __name__ == "__main__":
    sys.exit(main())
