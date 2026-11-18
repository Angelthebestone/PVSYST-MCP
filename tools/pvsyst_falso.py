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
                "botones": [{"texto": "OK", "habilitado": True}],
                "otros": [
                    {"clase": "TTabSheet", "texto": f"hoja {i}"},
                    {"clase": "TPanel", "texto": "no es rótulo"},
                ],
            }
            for i, etiquetas in enumerate(paginas)
        ],
    }


def objetivo_de(clave: str) -> Json:
    return {"clave": clave, "ventana": clave, "cadena": "a"}


def botones_de(*textos: str) -> list[Json]:
    return [{"texto": t, "habilitado": True} for t in textos]


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
        self.ilegibles: set[str] = set()
        self.vetadas: set[int] = set()
        self.fuera_de_rango: set[int] = set()
        self.incerrables: set[str] = set()

    def tool(self, nombre: str, a: Json) -> Json:
        self.acciones.append(a.get("accion", ""))
        if nombre == "gui_observar":
            if a["que"] == "ventanas":
                if not self.corriendo:
                    return {"error": "Process 'PVsyst.exe' is not running"}
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
        elif accion == "funcion":
            self.funciones.append(a["funcion"])
            if a["funcion"] not in self.abre:
                return {"error": f"unknown function {a['funcion']}"}
            self.abiertas.append({"class": self.abre[a["funcion"]], "title": "t"})
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
            if a["ventana"] not in self.incerrables:
                self.abiertas = [w for w in self.abiertas if w["class"] != a["ventana"]]
        elif accion == "clic_texto":
            self.clics.append(a["texto"])
            efecto = self.botones.get(a["texto"], {})
            if "error" in efecto:
                return {"error": efecto["error"]}
            if "abre" in efecto:
                self.abiertas.append({"class": efecto["abre"], "title": "t"})
            cierra = efecto.get("cierra", a["ventana"])
            if cierra:
                self.abiertas = [w for w in self.abiertas if w["class"] != cierra]
        elif accion == "escribir":
            self.escrito = a["texto"]
        elif accion == "fila":
            self.abiertas.append(
                {"class": "TProjectForm", "title": f"Proyecto {self.escrito}"}
            )
        return {}

    def cerrar(self) -> None:
        self.cerrado = True
