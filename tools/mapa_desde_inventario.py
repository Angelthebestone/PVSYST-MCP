"""Regenerates, in `docs/gui_mapa.json`, the windows that the tab inventory measured.

`docs/gui_inventario_pestanas.json` lists EVERY control of a window that the
form read returns, tab by tab (`gui_observar que:formulario` after
`gui_actuar accion:pagina`). The set of fields, their class, `marcado` and
`habilitado` come from enumerating the windows; their `etiqueta` comes from the
label chain of `que:formulario`, which includes OCR, and this inventory did not
record `etiqueta_fuente`. The old collector of the map kept at most 13 fields
per page, so for the windows of the inventory this script replaces the pages of
the map with the measured ones and leaves the rest of the file byte for byte as
it was.

An inventory entry is keyed by the window class, or by `Clase#abierta_con` when
the same window was measured through more than one opening (each opening is a
different instance of the map, in a different state); `REMEDIOS` uses the same
key and its `ventana` field is always the class.

What the inventory brings and the map keeps:
  - `campos`, `botones` (with `habilitado`) and `indice_pestana` come from the
    inventory; each field also keeps its `marcado` and `habilitado`.
  - `subpestana` (the nested sheet of a tab) comes from the inventory when it
    measured one. Such a page repeats the `pestana` and `indice_pestana` of its
    tab, so `pestanas` names the tab once, and its captions are the ones of that
    same (tab, sub-tab) pair, never those of another page of the tab.
  - `rotulos` (tab and group-box captions of the page) come from the inventory
    when it has them. Otherwise a page keeps the `rotulos` and `sin_rotulo` the
    map had when the tab of the same name still exists (and its state did not
    change, see `Remedio.sin_rotulos_de`); a new tab has none.
  - `titulo` and `abierta_con` of the instance stay as the map had them.
  - Each regenerated instance gets `estado_medido`: the state the window was in
    when measured (REMEDIOS), `no registrado` where the report does not say.
  - `nota` and `limites` of the map say what was regenerated and what is still
    truncated, counted on the map this script writes.

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

NO_REGISTRADO = "no registrado"

RAIZ = Path(__file__).resolve().parent.parent
MAPA = RAIZ / "docs" / "gui_mapa.json"
INVENTARIO = RAIZ / "docs" / "gui_inventario_pestanas.json"


@dataclass(frozen=True)
class Remedio:
    """How the inventory of one window enters the map."""

    # Add the inventory as a new instance opened with this key, keeping the
    # instances the map has. None: replace an existing instance (see `reemplaza`).
    anadir_con: str | None = None
    # `abierta_con` of the instance to replace; None: instance 0, the one opened
    # by the `variante.*` function the first inventory was measured through.
    reemplaza: str | None = None
    # Tabs of the replaced instance whose `rotulos` describe another state of
    # the window (controls the inventory shows hidden), so they are dropped.
    sin_rotulos_de: frozenset[str] = frozenset()
    # State of the window when measured; only what the report states.
    estado: str = NO_REGISTRADO


ESTADO_SIN_PROYECTO = "Sin proyecto cargado; ventana recién abierta, sin editar"
# No shipped demo opens `TUserNeedsForm` for this function (`_DEMO_StandAlone` opens
# `TDomesticUseForm`); a project built by the tool does, so the recipe is the state.
ESTADO_INDEPENDIENTE = (
    "Proyecto independiente con baterías creado con `disenar_sistema "
    "tipo_sistema:aislado simular:false` (sin demo equivalente: el demo "
    "_DEMO_StandAlone abre TDomesticUseForm) cargado; ventana recién abierta, sin editar"
)


def _demo(proyecto: str, detalle: str = "") -> str:
    """State of a window opened on one of PVsyst's shipped, read-only demos."""
    return (
        f"Proyecto demo {proyecto} (solo lectura) cargado{detalle}; "
        "ventana recién abierta, sin editar"
    )


ESTADO_COMMERCIAL = _demo("_DEMO_Commercial")
ESTADO_NUEVO = (
    "Objeto nuevo sin completar ni guardar (diálogo «Nuevo» de la base de datos)"
)

# Why each entry: INFORME_PRUEBAS_GUI_VISUAL_2026-11-17.md, sections 2.3, 2.4, 2.6.
REMEDIOS: dict[str, Remedio] = {
    "TDetailedLossesForm": Remedio(estado=ESTADO_COMMERCIAL),
    "TSystemegridForm": Remedio(estado=ESTADO_COMMERCIAL),
    # The map was measured with `Activo` ticked in P50-P90 (17 fields); the
    # inventory has it unticked (1 field): the old captions describe the ticked page.
    "TSystemeAddForm": Remedio(
        sin_rotulos_de=frozenset({"Estimación P50 - P90"}),
        estado="P50-P90 con Activo desmarcado",
    ),
    # The map has the tracker variant; the inventory is the fixed-tilt one.
    "TOrientationForm": Remedio(
        anadir_con="variante.orientacion",
        estado="Tilt fijo (proyecto demo _DEMO_Commercial, plano a 15°)",
    ),
    "THorizonForm": Remedio(estado=ESTADO_COMMERCIAL),
    "TShadingsForm": Remedio(estado=ESTADO_COMMERCIAL),
    "TEcoEvaluationForm": Remedio(estado=ESTADO_COMMERCIAL),
    "TUserNeedsForm": Remedio(estado=ESTADO_COMMERCIAL),
    "TSummaryForm": Remedio(estado=ESTADO_COMMERCIAL),
    "TGridStorageForm": Remedio(estado="Sin almacenamiento"),
    # Step 2, measured on 8.1.8 through `bases_de_datos.*.nuevo`. The form vetoes
    # the tab change until the module is complete, so it was filled in memory
    # (model, maker, STC sheet, size, technology, .PAN name) and closed unsaved;
    # only then does the tab `Parámetros del modelo` exist.
    "TPVModuleForm": Remedio(
        estado=(
            "Módulo nuevo completo en memoria, sin guardar (6 pestañas; "
            "solo la subpágina visible de cada una)"
        )
    ),
    "TGridInverterForm": Remedio(
        reemplaza="bases_de_datos.inversores_red>Nuevo",
        estado=(
            "Inversor nuevo trifásico 50 Hz completo en memoria, sin guardar; "
            "«Eficiencia definida para 3 voltajes» desmarcada; subpáginas "
            "anidadas no recorridas"
        ),
    ),
    **{
        clase: Remedio(reemplaza=abierta_con, estado=ESTADO_SIN_PROYECTO)
        for clase, abierta_con in {
            "TArrayCharactForm": "herramientas.comportamiento_electrico",
            "TParamSolForm": "herramientas.parametros_solares",
            "TVoltageOptimiseForm": "herramientas.optimizacion_voltaje",
            "TTranspositionForm": "herramientas.factor_transposicion",
            "TPreferencesForm": "configuraciones.preferencias",
            "TMeteoImportForm": "bases_de_datos.importar_formato_conocido",
            "TMeteoTablesGraphsForm": "bases_de_datos.tablas_graficos_meteo",
        }.items()
    },
    # Batch B: the stand-alone and pumping windows, remeasured on the shipped demos
    # (the first measurement used projects a user cannot reproduce).
    **{
        clase: Remedio(reemplaza=abierta_con, estado=_demo("_DEMO_Pumping"))
        for clase, abierta_con in {
            "TSettingsForm": "ventana_proyecto.proyecto.configuracion",
            "TGeoSiteForm": "ventana_proyecto.sitio.detalles",
            "TSystemPumpingForm": "bombeo > Sistema",
            "TWaterUseForm": "bombeo > Necesidades de agua",
        }.items()
    },
    "TCurrenciesForm": Remedio(
        reemplaza="TEcoEvaluationForm>Tasas", estado=ESTADO_COMMERCIAL
    ),
    "TSystemeBattForm": Remedio(
        reemplaza="independiente > Sistema", estado=_demo("_DEMO_StandAlone")
    ),
    "TDetailedLossesForm#independiente > Pérdidas detalladas": Remedio(
        reemplaza="independiente > Pérdidas detalladas",
        estado=_demo("_DEMO_StandAlone"),
    ),
    "TOrientationForm#independiente > Orientación": Remedio(
        reemplaza="independiente > Orientación", estado=_demo("_DEMO_StandAlone")
    ),
    "TOrientationForm#bombeo > Orientación": Remedio(
        reemplaza="bombeo > Orientación", estado=_demo("_DEMO_Pumping")
    ),
    # `variante.necesidades_usuario_independiente` opens TUserNeedsForm here, not
    # the TDomesticUseForm the catalogue declares: a new instance of the former.
    "TUserNeedsForm#variante.necesidades_usuario_independiente": Remedio(
        anadir_con="variante.necesidades_usuario_independiente",
        estado=ESTADO_INDEPENDIENTE,
    ),
    "TGridInverterForm#TSystemegridForm>Abrir": Remedio(
        reemplaza="TSystemegridForm>Abrir",
        estado=(
            "Proyecto _DEMO_Commercial cargado; inversor del sistema abierto con "
            "Abrir desde la ventana de sistema; recién abierto, sin editar; "
            "subpáginas anidadas no recorridas"
        ),
    ),
    # Batch A: windows that were truncated at 13 fields or had no project of their
    # type, remeasured on the shipped demos.
    "TSystemegridDCForm": Remedio(
        reemplaza="red_cc > Sistema", estado=_demo("_DEMO_DCGrid")
    ),
    "TDetailedLossesForm#red_cc > Pérdidas detalladas": Remedio(
        reemplaza="red_cc > Pérdidas detalladas", estado=_demo("_DEMO_DCGrid")
    ),
    "TDetailedLossesForm#bombeo > Pérdidas detalladas": Remedio(
        reemplaza="bombeo > Pérdidas detalladas", estado=_demo("_DEMO_Pumping")
    ),
    "TOrientationForm#red_cc > Orientación": Remedio(
        reemplaza="red_cc > Orientación", estado=_demo("_DEMO_DCGrid")
    ),
    # The tracker instance is the first one opened by `variante.orientacion`.
    "TOrientationForm#seguidor": Remedio(
        reemplaza="variante.orientacion", estado=_demo("_DEMO_Tracking", " (seguidor)")
    ),
    "TOrientationForm#TShadingsForm>Orientaciones": Remedio(
        reemplaza="TShadingsForm>Orientaciones", estado=ESTADO_COMMERCIAL
    ),
    "TPerspectiveForm": Remedio(estado=ESTADO_COMMERCIAL),
    "TTableFieldForm": Remedio(estado=ESTADO_COMMERCIAL),
    "TRectFieldForm": Remedio(estado=ESTADO_COMMERCIAL),
    "TGroundObjectForm": Remedio(estado=ESTADO_COMMERCIAL),
    "TOrientToolForm": Remedio(estado=ESTADO_COMMERCIAL),
    "TMismatchStringForm": Remedio(estado=ESTADO_COMMERCIAL),
    "TOptimizationToolForm": Remedio(estado=ESTADO_COMMERCIAL),
    "TResultPlotHorForm": Remedio(estado=ESTADO_COMMERCIAL),
    "TDomesticUseForm": Remedio(estado=_demo("_DEMO_StandAlone")),
    "TContactForm#configuraciones.preferencias>Editar info usuario": Remedio(
        reemplaza="configuraciones.preferencias>Editar info usuario",
        estado=ESTADO_SIN_PROYECTO,
    ),
    **{
        clase: Remedio(reemplaza=abierta_con, estado=ESTADO_NUEVO)
        for clase, abierta_con in {
            "TBatteryForm": "bases_de_datos.baterias>Nuevo",
            "TPumpForm": "bases_de_datos.bombas>Nuevo",
            "TPumpControllerForm": "bases_de_datos.controladores_bombeo>Nuevo",
            "TSAControllerForm": "bases_de_datos.controladores_independientes>Nuevo",
            "TGeneratorForm": "bases_de_datos.generadores>Nuevo",
            "TContactForm": "bases_de_datos.fabricantes_minoristas>Nuevo",
        }.items()
    },
}


def cargar(ruta: Path) -> Json:
    datos: Json = json.loads(ruta.read_bytes())
    return datos


def serializar(mapa: Json) -> bytes:
    """The map's own format: one line, no spaces, UTF-8, no final newline."""
    return json.dumps(mapa, ensure_ascii=False, separators=(",", ":")).encode("utf-8")


def _subpestana(pestana: Json) -> str | None:
    """The sub-tab of a page: text no reader can group by when it is blank."""
    subpestana = pestana.get("subpestana")
    if subpestana is None:
        return None
    if not isinstance(subpestana, str) or not subpestana.strip():
        raise ValueError(
            f"{pestana['pestana']!r}: subpestana must be a non-empty text, "
            f"not {subpestana!r}"
        )
    return subpestana


def _pagina(pestana: Json, heredada: Json | None) -> Json:
    pagina: Json = {"pestana": pestana["pestana"]}
    subpestana = _subpestana(pestana)
    if subpestana is not None:
        pagina["subpestana"] = subpestana
    pagina["campos"] = copy.deepcopy(pestana["campos"])
    pagina["botones"] = copy.deepcopy(pestana["botones"])
    if "rotulos" in pestana:
        pagina["rotulos"] = copy.deepcopy(pestana["rotulos"])
    elif heredada is not None:
        for clave in ("rotulos", "sin_rotulo"):
            if clave in heredada:
                pagina[clave] = copy.deepcopy(heredada[clave])
    pagina["indice_pestana"] = pestana["indice"]
    return pagina


def _nombres_de_pestana(paginas: list[Json]) -> list[str]:
    """The main tabs of the instance, each once, in the order they appear.

    A page with a `subpestana` repeats its tab, and `indice_pestana` indexes this
    list, so a duplicated name would move the index of every tab after it.
    """
    nombres: list[str] = []
    for pagina in paginas:
        nombre = pagina["pestana"]
        if nombre is not None and nombre not in nombres:
            nombres.append(nombre)
    return nombres


def _instancia(
    medida: Json,
    titulo: str,
    abierta_con: str,
    heredables: dict[tuple[str | None, str | None], Json],
    estado: str,
) -> Json:
    pestanas = medida["pestanas"]
    return {
        "titulo": titulo,
        "abierta_con": abierta_con,
        "estado_medido": estado,
        "pestanas": _nombres_de_pestana(pestanas),
        "paginas": [
            _pagina(p, heredables.get((p["pestana"], p.get("subpestana"))))
            for p in pestanas
        ],
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


def _instancia_a_reemplazar(
    clase: str, instancias: list[Json], remedio: Remedio
) -> Json:
    if remedio.reemplaza is None:
        return instancias[0]
    for instancia in instancias:
        if instancia["abierta_con"] == remedio.reemplaza:
            return instancia
    raise KeyError(f"{clase}: no instance opened with {remedio.reemplaza!r}")


def _remedir_ventana(clase: str, ventana: Json, medida: Json, remedio: Remedio) -> None:
    if medida["errores"]:
        raise ValueError(f"{clase}: the inventory reports errors {medida['errores']}")
    instancias: list[Json] = ventana["instancias"]
    if remedio.anadir_con is not None:
        # Same key and same state: another instance opened by the same function (a
        # different project) is not this one. Changing `estado` leaves the old
        # instance in the map, where it shows as a duplicate.
        instancias[:] = [
            i
            for i in instancias
            if not (
                _viene_del_inventario(i)
                and i["abierta_con"] == remedio.anadir_con
                and i["estado_medido"] == remedio.estado
            )
        ]
        instancias.append(
            _instancia(medida, medida["titulo"], remedio.anadir_con, {}, remedio.estado)
        )
    else:
        vieja = _instancia_a_reemplazar(clase, instancias, remedio)
        # Keyed by (tab, sub-tab): keyed by the tab alone, every page but the last of
        # a tab was unreachable and a sub-tab inherited the captions of another one.
        heredables = {
            (p["pestana"], p.get("subpestana")): p
            for p in vieja["paginas"]
            if p["pestana"] not in remedio.sin_rotulos_de
        }
        instancias[instancias.index(vieja)] = _instancia(
            medida, vieja["titulo"], vieja["abierta_con"], heredables, remedio.estado
        )
    ventana["controles"] = _controles(ventana)


INICIO_NOTA = "REMEDIDO:"
NOTA = (
    f"{INICIO_NOTA} las instancias con `estado_medido` se regeneraron con "
    "tools/mapa_desde_inventario.py desde docs/gui_inventario_pestanas.json "
    "(todos los campos que devuelve la lectura del formulario, sin techo de 13 por "
    "página: su clase, `marcado` y `habilitado` salen de enumerar las ventanas, pero "
    "`etiqueta` sale de la cadena de etiquetas de `que:formulario`, que incluye OCR, y "
    "esta medición no guardó `etiqueta_fuente`). "
    "Sus campos traen además `marcado` y `habilitado`, y `estado_medido` dice en qué "
    "estado estaba la ventana al medirla (`no registrado` si no se anotó): una casilla "
    "o un combo muestran u ocultan campos, así que la lista vale para ese estado y no "
    "para toda la ventana. Una página sin `rotulos` ni `sin_rotulo` es de una pestaña "
    "nueva o de una cuyo estado cambió. Las instancias sin `estado_medido` siguen como "
    "las dejó el recorrido de `medido_el`."
)
ALCANCE = "ALCANCE: "
# What `gui_actuar accion:pagina` does not reach. Edit it here, in one piece, when the
# inventory starts to measure the nested sheets.
ALCANCE_PAGINA = (
    "`pagina` solo cambia el TPageControl principal: en una ventana con "
    "hojas anidadas solo se registró la subpágina visible de cada pestaña y sus "
    "campos son un subconjunto (TPVModuleForm: `que:inventario` lista 17 hojas, 5 "
    "principales); `pestanas` de `que:formulario` también puede listar una hoja "
    "anidada como si fuera pestaña (TSystemPumpingForm: «Sugerencias de "
    "pre-dimensionamiento», con `pagina 2` fuera de rango). Hay ventanas que vetan el cambio de pestaña hasta que el objeto "
    "está completo (TPVModuleForm, TGridInverterForm); `estado_medido` dice si se "
    "completó en memoria."
)


def _alcance(mapa: Json) -> list[str]:
    """What the map still does not cover, counted on the map as it stands."""
    remedidas = sorted(
        clase
        for clase, v in mapa["ventanas"].items()
        if any("estado_medido" in i for i in v["instancias"])
    )
    paginas = [
        p
        for v in mapa["ventanas"].values()
        for i in v["instancias"]
        for p in i["paginas"]
    ]
    truncadas = sum(1 for p in paginas if len(p["campos"]) == 13)
    return [
        f"{ALCANCE}solo se remidieron sin truncar las instancias con `estado_medido` "
        f"({', '.join(remedidas)}). En el resto el recolector original guardó como "
        f"máximo 13 campos por página: {truncadas} de {len(paginas)} páginas tienen "
        "exactamente 13 y se leen como «al menos 13», no como el total (medido en vivo: "
        "TGridInverterForm «Parámetros adicionales» muestra 37). También siguen sin "
        "remedir las demás aperturas de esas ventanas (independiente, bombeo, red_cc "
        "y similares).",
        f"{ALCANCE}{ALCANCE_PAGINA}",
        f"{ALCANCE}`etiqueta` de un campo sin texto propio sale de OCR y puede estar mal "
        "leída (`o mulsc`, `Tol,`): las páginas medidas con `etiqueta_fuente` la guardan "
        "en cada campo; las otras no. En las páginas sin `estado_medido` algunas "
        "`etiqueta` son el valor del campo y no su rótulo (TResultForm, "
        "TGenerSyntheForm): `campo` no las encuentra.",
    ]


def _anotar_alcance(mapa: Json) -> None:
    original = mapa["nota"].split(f" {INICIO_NOTA}")[0]
    mapa["nota"] = f"{original} {NOTA}"
    propios = [x for x in mapa["limites"] if not x.startswith(ALCANCE)]
    mapa["limites"] = [*propios, *_alcance(mapa)]


def remedir(mapa: Json, inventario: Json) -> Json:
    """A copy of `mapa` with the windows of `inventario` regenerated."""
    nuevo = copy.deepcopy(mapa)
    for clave, medida in inventario.items():
        clase = medida["ventana"]
        if clase not in nuevo["ventanas"] or clave not in REMEDIOS:
            raise KeyError(f"{clave}: not in the map or in REMEDIOS")
        _remedir_ventana(clase, nuevo["ventanas"][clase], medida, REMEDIOS[clave])
    _anotar_alcance(nuevo)
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
