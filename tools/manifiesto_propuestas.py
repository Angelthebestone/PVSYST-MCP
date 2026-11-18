"""The rules that turn the GUI function catalog into entries of the measurement manifest.

`tools/manifiesto_desde_catalogo.py` is the command line; this module is the
core and does no I/O. For every class of `docs/gui_mapa.json` that no instance
measured, it searches the catalog (`gui_observar que:funciones`) for a function
with `efecto == "abre"` and `abre == clase` and builds the entry
`tools/medir_gui.py` needs from the chain of `requiere` steps that reaches it,
from the root to the function.

  - Every step of a chain is a function with `efecto == "abre"`. A function that
    modifies or closes is never a step: a class whose only chains need one is
    discarded as `cadena con paso que modifica`.
  - The chain starts with `@proyecto:<demo>` when the root's host is
    `TProjectForm` or when the root's key starts with `variante.`,
    `ventana_proyecto.`, `escena_3d.` or `resultados.`: those roots need a
    project loaded first. The demo is `_DEMO_Commercial`; `Entrada.demos` maps a
    root's host class or a prefix of its key to another demo and the longest key
    that matches wins. A root that needs no project never gets one.
  - The entry is keyed by the window class. A class already in the manifest (as
    `clave` or as `ventana`) is discarded as `ya medida`. When several functions
    open the same class, the shortest chain is proposed (the alphabetically
    first one among equally short chains) and the rest are listed under
    `alternativas`.
  - `requiere` is followed up to the root that has none; a cycle or a function
    missing from the catalog discards the class with its motive instead of
    hanging or crashing.
"""

from __future__ import annotations

import json
from collections.abc import Mapping
from dataclasses import asdict, dataclass
from typing import Any

Json = dict[str, Any]

DEMO_POR_DEFECTO = "_DEMO_Commercial"
ANFITRION_PROYECTO = "TProjectForm"
PREFIJOS_PROYECTO = ("variante.", "ventana_proyecto.", "escena_3d.", "resultados.")

SIN_FUNCION = "sin función que la abra"
YA_MEDIDA = "ya medida"


def _texto(datos: Json, campo: str) -> str:
    """The field as text; a catalog entry that lacks it names the field it lacks."""
    valor = datos.get(campo)
    if not isinstance(valor, str):
        raise ValueError(f"{campo}: se esperaba un texto; llegó {valor!r}")
    return valor


def _texto_opcional(datos: Json, campo: str) -> str | None:
    """The field as text or None; a value of another type is a broken catalog."""
    valor = datos.get(campo)
    if valor is not None and not isinstance(valor, str):
        raise ValueError(f"{campo}: se esperaba un texto o null; llegó {valor!r}")
    return valor


@dataclass(frozen=True)
class Funcion:
    """One function of the GUI catalog, parsed once into the fields a chain needs."""

    clave: str
    ventana: str
    efecto: str
    requiere: str | None
    abre: str | None

    @classmethod
    def desde(cls, datos: Json) -> Funcion:
        """Parse one catalog entry."""
        return cls(
            clave=_texto(datos, "clave"),
            ventana=_texto(datos, "ventana"),
            efecto=_texto(datos, "efecto"),
            requiere=_texto_opcional(datos, "requiere"),
            abre=_texto_opcional(datos, "abre"),
        )


@dataclass(frozen=True)
class Recorrido:
    """A usable `requiere` path, root first, or why the function cannot be reached.

    `pasos` is empty only when `motivo` says why, so a `Recorrido` without a
    motive always has a root to ask for its project.
    """

    pasos: tuple[Funcion, ...] = ()
    motivo: str | None = None


@dataclass(frozen=True)
class Entrada:
    """What a proposal reads: the map's classes, the manifest's names and the catalog."""

    clases: tuple[str, ...]
    funciones: dict[str, Funcion]
    ya_medidas: frozenset[str]
    demos: dict[str, str]


@dataclass(frozen=True)
class Propuesta:
    """One manifest entry, ready to paste."""

    clave: str
    ventana: str
    cadena: str


@dataclass(frozen=True)
class Descartada:
    """A window class that gets no entry, and why."""

    ventana: str
    motivo: str


@dataclass(frozen=True)
class Informe:
    """Entries to add, classes left out, and the valid chains that were not picked."""

    propuestas: tuple[Propuesta, ...]
    descartadas: tuple[Descartada, ...]
    alternativas: dict[str, tuple[str, ...]]

    def como_json(self) -> Json:
        return {
            "propuestas": [asdict(p) for p in self.propuestas],
            "descartadas": [asdict(d) for d in self.descartadas],
            "alternativas": [
                {"ventana": ventana, "cadenas": list(cadenas)}
                for ventana, cadenas in self.alternativas.items()
            ],
        }


def cargar_catalogo(texto: str) -> dict[str, Funcion]:
    """The catalog keyed by function key; a repeated key is a broken catalog."""
    datos: Any = json.loads(texto)
    lista = datos.get("funciones") if isinstance(datos, dict) else datos
    if not isinstance(lista, list):
        raise ValueError("el catálogo no es una lista ni tiene una lista `funciones`")
    funciones: dict[str, Funcion] = {}
    for bruto in lista:
        if not isinstance(bruto, dict):
            raise ValueError(f"la función no es un objeto: {bruto!r}")
        funcion = Funcion.desde(bruto)
        if funcion.clave in funciones:
            raise ValueError(f"clave repetida en el catálogo: {funcion.clave}")
        funciones[funcion.clave] = funcion
    return funciones


def demos(texto: str | None) -> dict[str, str]:
    """The `--proyectos` map: a root's host class or key prefix -> the demo to load."""
    if texto is None:
        return {}
    datos: Any = json.loads(texto)
    if not isinstance(datos, dict):
        raise ValueError("--proyectos: se esperaba un objeto {clase_o_prefijo: demo}")
    if not all(isinstance(demo, str) for demo in datos.values()):
        raise ValueError(f"--proyectos: hay valores que no son demos: {datos!r}")
    return {clave: demo for clave, demo in datos.items()}


def sin_medir(mapa: Json) -> tuple[str, ...]:
    """The map's window classes without any instance carrying `estado_medido`."""
    ventanas = mapa.get("ventanas")
    if not isinstance(ventanas, dict):
        raise ValueError("el mapa no tiene un objeto `ventanas`")
    for clase, entrada in ventanas.items():
        if not isinstance(entrada, dict):
            raise ValueError(f"la ventana {clase!r} del mapa no es un objeto")
    return tuple(
        clase
        for clase, entrada in ventanas.items()
        if not any("estado_medido" in i for i in entrada.get("instancias") or ())
    )


def ya_medidas(manifiesto: Any) -> frozenset[str]:
    """Every class the manifest already targets, as `clave` or as `ventana`."""
    if not isinstance(manifiesto, list):
        raise ValueError("el manifiesto no es una lista")
    nombres: set[str] = set()
    for entrada in manifiesto:
        if not isinstance(entrada, dict):
            raise ValueError(f"la entrada del manifiesto no es un objeto: {entrada!r}")
        nombres.update(
            texto
            for texto in (entrada.get("clave"), entrada.get("ventana"))
            if isinstance(texto, str)
        )
    return frozenset(nombres)


def recorrido(funciones: Mapping[str, Funcion], hoja: Funcion) -> Recorrido:
    """Follow `requiere` from `hoja` up to the root that has none, checking every step."""
    pasos: list[Funcion] = []
    vistas: set[str] = set()
    actual = hoja
    while True:
        if actual.clave in vistas:
            return Recorrido(motivo=f"ciclo en requiere: {actual.clave}")
        vistas.add(actual.clave)
        pasos.append(actual)
        if actual.efecto != "abre":
            return Recorrido(
                motivo=f"cadena con paso que {actual.efecto}: {actual.clave}"
            )
        if actual.requiere is None:
            return Recorrido(pasos=tuple(reversed(pasos)))
        siguiente = funciones.get(actual.requiere)
        if siguiente is None:
            return Recorrido(
                motivo=f"cadena con paso que falta del catálogo: {actual.requiere}"
            )
        actual = siguiente


def demo(raiz: Funcion, proyectos: Mapping[str, str]) -> str | None:
    """The demo a chain rooted at `raiz` loads, or None when the root needs no project."""
    if raiz.ventana != ANFITRION_PROYECTO and not raiz.clave.startswith(
        PREFIJOS_PROYECTO
    ):
        return None
    forzados = [k for k in proyectos if raiz.ventana == k or raiz.clave.startswith(k)]
    if not forzados:
        return DEMO_POR_DEFECTO
    return proyectos[max(forzados, key=lambda k: (len(k), k))]


def cadena(pasos: tuple[Funcion, ...], proyectos: Mapping[str, str]) -> str:
    """The manifest chain of a `recorrido` path: root to leaf, with `@proyecto` in front."""
    claves = [funcion.clave for funcion in pasos]
    primero = demo(pasos[0], proyectos)
    return ",".join([f"@proyecto:{primero}", *claves] if primero else claves)


def cadenas_de(entrada: Entrada, clase: str) -> tuple[list[str], list[str]]:
    """The valid chains to `clase`, shortest first, and the motive of every function dropped."""
    validas: list[str] = []
    motivos: list[str] = []
    for funcion in entrada.funciones.values():
        if funcion.efecto != "abre" or funcion.abre != clase:
            continue
        camino = recorrido(entrada.funciones, funcion)
        if camino.motivo is not None:
            motivos.append(camino.motivo)
            continue
        validas.append(cadena(camino.pasos, entrada.demos))
    validas.sort(key=lambda c: (len(c.split(",")), c))
    return validas, motivos


def proponer(entrada: Entrada) -> Informe:
    """Propose an entry per class, or record why the class gets none."""
    propuestas: list[Propuesta] = []
    descartadas: list[Descartada] = []
    alternativas: dict[str, tuple[str, ...]] = {}
    for clase in entrada.clases:
        if clase in entrada.ya_medidas:
            descartadas.append(Descartada(clase, YA_MEDIDA))
            continue
        validas, motivos = cadenas_de(entrada, clase)
        if not validas:
            motivo = "; ".join(dict.fromkeys(motivos)) or SIN_FUNCION
            descartadas.append(Descartada(clase, motivo))
            continue
        propuestas.append(Propuesta(clase, clase, validas[0]))
        if len(validas) > 1:
            alternativas[clase] = tuple(validas[1:])
    return Informe(tuple(propuestas), tuple(descartadas), alternativas)
