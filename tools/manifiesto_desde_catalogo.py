"""Proposes the manifest entries for the windows of the map that were never measured.

    python tools/manifiesto_desde_catalogo.py --catalogo funciones.json \\
        [--mapa docs/gui_mapa.json] [--manifiesto tools/gui_manifiesto.json] \\
        [--proyectos demos.json]

The catalog is the JSON of `gui_observar que:funciones detalle:completo`, either
`{"funciones": [...]}` or the bare list, and `--catalogo` defaults to stdin. The
rules that turn it into entries live in `manifiesto_propuestas.py`; `--proyectos`
is their `Entrada.demos`: a JSON `{window class of a root or prefix of its key:
demo}` that replaces `_DEMO_Commercial` for that root, as in
`{"TProjectForm": "_DEMO_DCGrid", "escena_3d.": "_DEMO_Tracking"}`.

The tool writes neither the manifest nor the map: until `medir_gui.py` opens the
window, the chain is a claim about the GUI. On stdout, JSON with `propuestas`
(entries that paste into the manifest as they are), `descartadas` (each class
left out with its motive) and `alternativas` (the valid chains not picked); a
summary on stderr.
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path

# `python -I tools/manifiesto_desde_catalogo.py` leaves the script's directory out
# of `sys.path` and the modules of this tool are its siblings, so the imports
# below need it first.
sys.path.insert(0, str(Path(__file__).resolve().parent))

from manifiesto_propuestas import (  # noqa: E402
    DEMO_POR_DEFECTO,
    Entrada,
    Informe,
    cargar_catalogo,
    demos,
    proponer,
    sin_medir,
    ya_medidas,
)

RAIZ = Path(__file__).resolve().parent.parent
MAPA = RAIZ / "docs" / "gui_mapa.json"
MANIFIESTO = Path(__file__).resolve().parent / "gui_manifiesto.json"


def _argumentos(argv: list[str] | None) -> argparse.Namespace:
    """The command line of the tool."""
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument(
        "--catalogo", type=Path, help="catalog JSON; stdin when omitted"
    )
    parser.add_argument("--mapa", type=Path, default=MAPA)
    parser.add_argument("--manifiesto", type=Path, default=MANIFIESTO)
    parser.add_argument(
        "--proyectos",
        type=Path,
        help=(
            "JSON {window class of a root or prefix of its key: demo} that replaces "
            f"{DEMO_POR_DEFECTO} for that root"
        ),
    )
    return parser.parse_args(argv)


def _leer(ruta: Path | None) -> str:
    """The text of the file, or stdin when the option is absent."""
    if ruta is None:
        return sys.stdin.read()
    return ruta.read_text(encoding="utf-8")


def _resumen(informe: Informe) -> str:
    """The counts and the classes with another valid chain, for stderr."""
    lineas = [
        f"{len(informe.propuestas)} propuestas, {len(informe.descartadas)} descartadas"
    ]
    for motivo, cuantas in Counter(d.motivo for d in informe.descartadas).items():
        lineas.append(f"  {cuantas} × {motivo}")
    if informe.alternativas:
        otras = ", ".join(
            f"{ventana} ({len(cadenas)})"
            for ventana, cadenas in informe.alternativas.items()
        )
        lineas.append(f"otras cadenas válidas en: {otras}")
    return "\n".join(lineas)


def main(argv: list[str] | None = None) -> int:
    args = _argumentos(argv)
    try:
        informe = proponer(
            Entrada(
                clases=sin_medir(json.loads(_leer(args.mapa))),
                funciones=cargar_catalogo(_leer(args.catalogo)),
                ya_medidas=ya_medidas(json.loads(_leer(args.manifiesto))),
                demos=demos(_leer(args.proyectos) if args.proyectos else None),
            )
        )
    except (OSError, ValueError) as error:
        print(error, file=sys.stderr)
        return 2
    # ASCII JSON: a redirected stdout on Windows encodes accents in the console
    # code page, and the result is no longer valid JSON.
    print(json.dumps(informe.como_json(), indent=2))
    print(_resumen(informe), file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
