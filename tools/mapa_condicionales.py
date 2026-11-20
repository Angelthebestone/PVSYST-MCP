"""Put the measured switches of `docs/gui_condicionales.json` into the window map.

A page of an instance gets `requiere`: the ordered steps (`{campo, valor, aparecen,
desaparecen, habilitan, deshabilitan}`) that were measured on its tab, so an agent
reads which checkbox or combo shows or enables which field instead of taking the
fields of the page for all of the window. The labels come from `que:formulario` and
can be OCR text, so one label may appear on both sides of a step with two spellings.
"""

from __future__ import annotations

import copy
from typing import Any

Json = dict[str, Any]


def _pagina_de(paginas: list[Json], pestana: str | None) -> Json | None:
    """The page of the tab `pestana` (None: the only page of a window without tabs)."""
    if pestana is None:
        return paginas[0] if len(paginas) == 1 else None
    candidatas = [
        p for p in paginas if p["pestana"] == pestana and "subpestana" not in p
    ]
    return candidatas[0] if len(candidatas) == 1 else None


def aplicar(mapa: Json, condicionales: Json) -> None:
    """Write `requiere` on the page each entry of `condicionales` measured."""
    for clave, entrada in condicionales.items():
        ventana = mapa["ventanas"].get(entrada["ventana"])
        instancias = [
            i
            for i in (ventana or {}).get("instancias", [])
            if i["abierta_con"] == entrada["abierta_con"]
        ]
        pagina = (
            _pagina_de(instancias[0]["paginas"], entrada["pestana"])
            if len(instancias) == 1
            else None
        )
        if pagina is None:
            raise KeyError(f"{clave}: no single page of the map matches the entry")
        pagina["requiere"] = copy.deepcopy(entrada["condiciones"])
