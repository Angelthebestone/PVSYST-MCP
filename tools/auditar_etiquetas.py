"""Reports the `etiqueta` values of docs/gui_inventario_pestanas.json that look like values.

The inventory takes the label of each field from a chain of sources
(`window_text`, `uia_name`, OCR). When the control has no text of its own the
label is read by OCR, and sometimes the "label" is really the field's value:
`50`, `0,95`, `15.0 °`, `Oakland`, a date. The report calls this defect §2.5.
Fields measured in step 1 carry no `etiqueta_fuente`; the step 2 fields do.

This tool reads the inventory and never writes it, and it exits 0 whatever it
finds: it is a report, not a gate. A finding is a field whose label looks like
a value, with the rules that fired:

  - `valor_numerico`: only digits, signs, decimal separators and an optional
    unit, as in `^[\\d.,\\s+-]+(%|°|[a-zA-Z/²³]{0,6})?$` (so `150 ppp` and
    `870,9 MWh` fire too; a real caption such as `Inclinación del plano` does
    not).
  - `fecha`: a numeric date (`17/11/2026`, `2026-11-17`) or a day, month name
    and optional year (`23 nov`, `23 de noviembre de 2026`).
  - `vacia_o_corta`: no label at all, or one character after trimming (`W`,
    `s`, `=`): too short to name a field.
  - `duplicada_misma_clase`: the same label appears two or more times in the
    same tab on fields of the same control class (duplicate fields).
  - `otra_clase_numerica`: the label repeats on a field of another class that
    is clearly numeric (`TUpDown`, `TSpinEdit`). A value-like label shared with
    a numeric control is stronger evidence than the same label alone; a real
    caption in a `TUpDown` + `TEdit` pair (`Azimut`) does not fire, because the
    pair is the normal shape of one field.

Usage:
    python tools/auditar_etiquetas.py [--inventario RUTA] [--json]

The default output is a text table followed by the totals per rule. `--json`
prints the same findings as JSON instead. A field can fire several rules and
counts once in the total and once in each of its rules.
"""

from __future__ import annotations

import argparse
import json
import re
from collections import Counter
from collections.abc import Sequence
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

Json = dict[str, Any]

RAIZ = Path(__file__).resolve().parent.parent
INVENTARIO = RAIZ / "docs" / "gui_inventario_pestanas.json"

VALOR_NUMERICO = "valor_numerico"
FECHA = "fecha"
VACIA_O_CORTA = "vacia_o_corta"
DUPLICADA_MISMA_CLASE = "duplicada_misma_clase"
OTRA_CLASE_NUMERICA = "otra_clase_numerica"

REGLAS: dict[str, str] = {
    VALOR_NUMERICO: "solo dígitos, signos y separadores decimales, con unidad opcional",
    FECHA: "fecha, en lugar del rótulo del campo",
    VACIA_O_CORTA: "sin etiqueta, o de un solo carácter",
    DUPLICADA_MISMA_CLASE: "etiqueta repetida en la misma pestaña y la misma clase",
    OTRA_CLASE_NUMERICA: "etiqueta de valor compartida con un control numérico",
}

PATRON_VALOR = re.compile(r"^[\d.,\s+-]+(%|°|[a-zA-Z/²³]{0,6})?$")

MESES = (
    r"(?:ene(?:ro)?|feb(?:rero)?|mar(?:zo)?|abr(?:il)?|may(?:o)?|jun(?:io)?"
    r"|jul(?:io)?|ago(?:sto)?|sep(?:tiembre)?|oct(?:ubre)?|nov(?:iembre)?"
    r"|dic(?:iembre)?)"
)
PATRONES_FECHA = (
    re.compile(r"^\d{1,2}[./-]\d{1,2}[./-]\d{2,4}$"),
    re.compile(r"^\d{4}[./-]\d{1,2}[./-]\d{1,2}$"),
    re.compile(
        rf"^\d{{1,2}}\s+(?:de\s+)?{MESES}(?:\s+(?:de\s+)?\d{{2,4}})?$",
        re.IGNORECASE,
    ),
)

# Classes whose value is a number: PVsyst puts a TUpDown next to the TEdit of a
# numeric field, so a value-like label shared by both is read from the value.
CLASES_NUMERICAS = frozenset({"TSpinEdit", "TUpDown"})


@dataclass(frozen=True)
class Hallazgo:
    """One field whose label looks like a value, with the rules that fired."""

    clave: str
    ventana: str
    pestana: str | None
    indice: int | None
    clase: str
    etiqueta: str | None
    razones: tuple[str, ...]


def _etiqueta(campo: Json) -> str | None:
    etiqueta = campo.get("etiqueta")
    return etiqueta if isinstance(etiqueta, str) else None


def _clase(campo: Json) -> str:
    clase = campo.get("clase")
    return clase if isinstance(clase, str) else ""


def _es_fecha(etiqueta: str) -> bool:
    return any(patron.match(etiqueta) for patron in PATRONES_FECHA)


def _repeticiones(campos: Sequence[Json]) -> Counter[tuple[str, str | None]]:
    cuentas: Counter[tuple[str, str | None]] = Counter()
    for campo in campos:
        etiqueta = _etiqueta(campo)
        if etiqueta is not None and etiqueta.strip():
            cuentas[(_clase(campo), etiqueta)] += 1
    return cuentas


def _razones(
    campo: Json,
    campos: Sequence[Json],
    repeticiones: Counter[tuple[str, str | None]],
) -> tuple[str, ...]:
    etiqueta = _etiqueta(campo)
    clase = _clase(campo)
    if etiqueta is None or not etiqueta.strip():
        return (VACIA_O_CORTA,)
    limpia = etiqueta.strip()
    razones: list[str] = []
    if PATRON_VALOR.match(limpia):
        razones.append(VALOR_NUMERICO)
    if _es_fecha(limpia):
        razones.append(FECHA)
    if len(limpia) == 1:
        razones.append(VACIA_O_CORTA)
    if repeticiones[(clase, etiqueta)] > 1:
        razones.append(DUPLICADA_MISMA_CLASE)
    parece_valor = set(razones) & {VALOR_NUMERICO, FECHA, VACIA_O_CORTA}
    comparte_con_numerica = any(
        _etiqueta(otro) == etiqueta
        and _clase(otro) in CLASES_NUMERICAS
        and _clase(otro) != clase
        for otro in campos
    )
    if parece_valor and comparte_con_numerica:
        razones.append(OTRA_CLASE_NUMERICA)
    return tuple(razones)


def auditar(inventario: Json) -> list[Hallazgo]:
    """Every field of `inventario` whose label looks like a value, in file order."""
    hallazgos: list[Hallazgo] = []
    for clave, medida in inventario.items():
        ventana = medida.get("ventana")
        for pestana in medida.get("pestanas", []):
            campos: list[Json] = list(pestana.get("campos", []))
            repeticiones = _repeticiones(campos)
            for campo in campos:
                razones = _razones(campo, campos, repeticiones)
                if razones:
                    hallazgos.append(
                        Hallazgo(
                            clave=clave,
                            ventana=ventana if isinstance(ventana, str) else "",
                            pestana=pestana.get("pestana"),
                            indice=pestana.get("indice"),
                            clase=_clase(campo),
                            etiqueta=_etiqueta(campo),
                            razones=razones,
                        )
                    )
    return hallazgos


def resumen(inventario: Json, hallazgos: Sequence[Hallazgo]) -> Json:
    """Totals of the inventory and how many findings each rule catches."""
    pestanas = [
        pestana
        for medida in inventario.values()
        for pestana in medida.get("pestanas", [])
    ]
    campos = sum(len(pestana.get("campos", [])) for pestana in pestanas)
    por_regla = {
        regla: sum(1 for hallazgo in hallazgos if regla in hallazgo.razones)
        for regla in REGLAS
    }
    return {
        "ventanas": len(inventario),
        "pestanas": len(pestanas),
        "campos": campos,
        "sospechosos": len(hallazgos),
        "por_regla": por_regla,
    }


CABECERA = ("clave", "pestaña", "clase", "etiqueta", "razones")
ANCHOS = (46, 34, 16, 60, 80)


def _recortar(texto: str, ancho: int) -> str:
    return texto if len(texto) <= ancho else f"{texto[: ancho - 1]}…"


def tabla(hallazgos: Sequence[Hallazgo]) -> str:
    """One text row per finding, with long cells cut to keep the table readable."""
    filas = [CABECERA]
    filas.extend(
        (
            hallazgo.clave,
            hallazgo.pestana or "(sin pestaña)",
            hallazgo.clase,
            "(sin etiqueta)" if hallazgo.etiqueta is None else hallazgo.etiqueta,
            ", ".join(hallazgo.razones),
        )
        for hallazgo in hallazgos
    )
    anchos = [
        min(max(len(fila[columna]) for fila in filas), tope)
        for columna, tope in enumerate(ANCHOS)
    ]
    lineas = [
        " | ".join(
            _recortar(celda, ancho) for celda, ancho in zip(fila, anchos, strict=True)
        )
        for fila in filas
    ]
    lineas.insert(1, "-+-".join("-" * ancho for ancho in anchos))
    return "\n".join(lineas)


def _plural(cantidad: int, singular: str, plural: str) -> str:
    return singular if cantidad == 1 else plural


def _resumen_texto(datos: Json) -> str:
    por_regla: dict[str, int] = datos["por_regla"]
    sospechosos = datos["sospechosos"]
    campos = datos["campos"]
    ventanas = datos["ventanas"]
    pestanas = datos["pestanas"]
    lineas = [
        f"{sospechosos} {_plural(sospechosos, 'etiqueta sospechosa', 'etiquetas sospechosas')} "
        f"de {campos} {_plural(campos, 'campo', 'campos')} en "
        f"{ventanas} {_plural(ventanas, 'ventana', 'ventanas')} y "
        f"{pestanas} {_plural(pestanas, 'pestaña', 'pestañas')}.",
        "Un campo puede sumar varias reglas.",
    ]
    lineas.extend(
        f"  {regla}: {por_regla[regla]} — {descripcion}"
        for regla, descripcion in REGLAS.items()
    )
    return "\n".join(lineas)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--inventario", type=Path, default=INVENTARIO)
    parser.add_argument("--json", action="store_true", help="print JSON, not a table")
    args = parser.parse_args(argv)

    inventario: Json = json.loads(args.inventario.read_text(encoding="utf-8"))
    hallazgos = auditar(inventario)
    datos = resumen(inventario, hallazgos)
    if args.json:
        print(
            json.dumps(
                {"resumen": datos, "hallazgos": [asdict(h) for h in hallazgos]},
                ensure_ascii=False,
                indent=2,
            )
        )
    else:
        if hallazgos:
            print(tabla(hallazgos))
            print()
        print(_resumen_texto(datos))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
