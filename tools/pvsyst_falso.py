"""The fake MCP server the measurement tests drive: no PVsyst, no client process.

`PVsystFalso` answers `gui_observar` and `gui_actuar` from dictionaries the test fills
in; `campo`, `formulario`, `objetivo_de` and `botones_de` build the shapes those calls
return. A test module that waits imports `sin_esperas`: it is an autouse fixture and
`PAUSA` in `medir_ventanas` is the one knob every wait of the tool reads.
"""

from __future__ import annotations

import pytest

import medir_ventanas
from medir_cliente import Json
from medir_proyectos import Objetivo, Propiedad


@pytest.fixture(autouse=True)
def sin_esperas(monkeypatch: pytest.MonkeyPatch) -> None:
    """No wait of the tool really sleeps."""
    monkeypatch.setattr(medir_ventanas, "PAUSA", 0)


def campo(etiqueta: str) -> Json:
    return {
        "clase": "TEdit",
        "etiqueta": etiqueta,
        "etiqueta_fuente": "uia_name",
        "marcado": None,
        "habilitado": True,
        "pestana": "ignorada",
    }


def formulario(*paginas: list[str], pestanas: list[str] | None = None) -> Json:
    return {
        "titulo": "Ventana de prueba",
        "pestanas": pestanas or [],
        "paginas": [
            {
                "campos": [campo(e) for e in etiquetas],
                "botones": botones_de("OK"),
                "otros": [
                    {"clase": "TTabSheet", "texto": f"hoja {i}"},
                    {"clase": "TPanel", "texto": "no es rótulo"},
                ],
            }
            for i, etiquetas in enumerate(paginas)
        ],
    }


def propiedad_de(*nombres: str) -> Propiedad:
    """A run record that loaded `nombres` itself: the projects it may close."""
    propiedad = Propiedad()
    for nombre in nombres:
        propiedad.registrar(nombre)
    return propiedad


def objetivo_de(
    clave: str, cadena: str = "a", propiedad: Propiedad | None = None
) -> Objetivo:
    """One target of the manifest, as `medir_gui` builds it from its entry."""
    return Objetivo(
        clave=clave,
        ventana=clave,
        cadena=cadena,
        propiedad=Propiedad() if propiedad is None else propiedad,
    )


def respuesta_de_clic(efecto: Json) -> Json:
    """The answer `gui_actuar clic_texto` gives for one press effect.

    A successful click always names in `origen` where it resolved the text, and a
    refused one answers the error alone; the UIA fallback answers both
    (`clic_por_texto.rs`).
    """
    respuesta: Json = {}
    if "error" in efecto:
        respuesta["error"] = efecto["error"]
    if "origen" in efecto:
        respuesta["origen"] = efecto["origen"]
    elif "error" not in efecto:
        respuesta["origen"] = "control"
    return respuesta


def botones_de(*textos: str, etiqueta_fuente: str | None = "window_text") -> list[Json]:
    """Buttons as `procesar_boton` reports them.

    `etiqueta_fuente` is `window_text` for a control that draws its own caption, `ocr`
    for text recognised inside it and None when it has none; a measurement recorded
    before the server reported it carries no such key at all.
    """
    botones: list[Json] = []
    for texto in textos:
        boton: Json = {
            "texto": texto,
            "habilitado": True,
            "etiqueta_fuente": etiqueta_fuente,
        }
        botones.append(boton)
    return botones


class PVsystFalso:
    """Just enough of the MCP tools: windows, forms with tabs, functions and prompts."""

    def __init__(
        self,
        formularios: dict[str, Json],
        abre: dict[str, str],
        botones: dict[str, Json] | None = None,
    ) -> None:
        self.abiertas: list[Json] = [{"class": "THomeForm", "title": "PVsyst"}]
        self.formularios = formularios
        self.abre = abre
        # What pressing a text does: `abre` a window class, `cierra` one (the window
        # it was pressed in when the key is absent, none when it is null) or `error`.
        self.botones = botones or {}
        self.pagina: dict[str, int] = {}
        self.clics: list[str] = []
        self.funciones: list[str] = []
        self.escrito = ""
        self.cerrado = False
        self.corriendo = True
        self.lanza = True
        self.acciones: list[str] = []
        # Every reference `accion:cerrar` asked for, in order.
        self.cierres: list[str] = []
        # A read of the window list that fails while the process keeps running; a
        # launch clears it, since the new process answers again.
        self.falla_ventanas: str | None = None
        self.ilegibles: set[str] = set()
        self.vetadas: set[int] = set()
        self.fuera_de_rango: set[int] = set()
        # A class listed here ignores both the close and the click: a native dialog
        # (#32770) that refuses WM_CLOSE, which no press of the tool can dismiss.
        self.incerrables: set[str] = set()
        # The `que` of a read or the `accion` of an action whose answer is a JSON-RPC
        # failure instead of a result: what `ClienteMcp` builds from `{"error": ...}`.
        self.jsonrpc: set[str] = set()

    def tool(self, nombre: str, a: Json) -> Json:
        self.acciones.append(a.get("accion", ""))
        llave = a.get("que") or a.get("accion") or ""
        if llave in self.jsonrpc:
            return {"jsonrpc_error": f"-32603 {llave} failed"}
        if nombre == "gui_observar":
            if a["que"] == "ventanas":
                if not self.corriendo:
                    return {"error": "Process 'PVsyst.exe' is not running"}
                if self.falla_ventanas is not None:
                    return {"error": self.falla_ventanas}
                return {"windows": list(self.abiertas)}
            if a["ventana"] in self.ilegibles:
                return {"error": f"Window '{a['ventana']}' not found"}
            f = self.formularios[a["ventana"]]
            pag = f["paginas"][self.pagina.get(a["ventana"], 0)]
            return {
                "titulo": f["titulo"],
                "pestanas": [{"nombre": n} for n in f["pestanas"]],
                "textos": f.get("textos"),
                **pag,
            }
        return self._actuar(a)

    def _actuar(self, a: Json) -> Json:
        accion = a["accion"]
        if accion == "lanzar":
            self.corriendo = self.lanza
            # A launched process is read again, so a read error does not outlive it.
            self.falla_ventanas = None
        elif accion == "funcion":
            self.funciones.append(a["funcion"])
            clase = self.abre.get(a["funcion"])
            # The harness names the loader itself (`cargar_proyecto`) and relies on the
            # catalogue entry `proyecto.cargar` opening `TProjectLoadForm`.
            if a["funcion"] == "proyecto.cargar":
                clase = "TProjectLoadForm"
            if clase is None:
                return {"error": f"unknown function {a['funcion']}"}
            self.abiertas.append({"class": clase, "title": "t"})
        elif accion == "pagina":
            if a["indice"] in self.fuera_de_rango:
                n = a["indice"]
                return {
                    "error": f"'indice' {n} out of range: 'W' has {n} tab pages (0 to {n - 1})"
                }
            if a["indice"] in self.vetadas:
                return {"error": "the window vetoed the tab change"}
            self.pagina[a["ventana"]] = a["indice"]
        elif accion == "cerrar":
            self.cierres.append(str(a["ventana"]))
            # The server closes the front-most window whose class or title is the
            # reference; this fake lists them in the order they opened, so the search
            # starts at the end: the window a press has just opened.
            for i in reversed(range(len(self.abiertas))):
                w = self.abiertas[i]
                if a["ventana"] not in (w["class"], w.get("title")):
                    continue
                if w["class"] not in self.incerrables:
                    del self.abiertas[i]
                break
        elif accion == "clic_texto":
            self.clics.append(a["texto"])
            efecto = self.botones.get(a["texto"], {})
            # A click the server refused does nothing.
            if "error" in efecto:
                return respuesta_de_clic(efecto)
            if "abre" in efecto:
                self.abiertas.append({"class": efecto["abre"], "title": "t"})
            cierra = efecto.get("cierra", a["ventana"])
            if cierra and cierra not in self.incerrables:
                self.abiertas = [w for w in self.abiertas if w["class"] != cierra]
            return respuesta_de_clic(efecto)
        elif accion == "escribir":
            self.escrito = a["texto"]
        elif accion == "fila":
            self.abiertas.append(
                {"class": "TProjectForm", "title": f"Proyecto {self.escrito}"}
            )
        return {}

    def cerrar(self) -> None:
        self.cerrado = True
