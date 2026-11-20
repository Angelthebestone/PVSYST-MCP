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


def botones_de(*textos: str, origen: str | None = "window_text") -> list[Json]:
    """Buttons as a form read reports them; `origen=None` is a server that reports none."""
    botones: list[Json] = []
    for texto in textos:
        boton: Json = {"texto": texto, "habilitado": True}
        if origen is not None:
            boton["origen"] = origen
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
        # A read of the window list that fails while the process keeps running; a
        # launch clears it, since the new process answers again.
        self.falla_ventanas: str | None = None
        self.ilegibles: set[str] = set()
        self.vetadas: set[int] = set()
        self.fuera_de_rango: set[int] = set()
        # A class listed here ignores both the close and the click: a native dialog
        # (#32770) that refuses WM_CLOSE, which no press of the tool can dismiss.
        self.incerrables: set[str] = set()

    def tool(self, nombre: str, a: Json) -> Json:
        self.acciones.append(a.get("accion", ""))
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
            for i, w in enumerate(self.abiertas):
                if a["ventana"] not in (w["class"], w.get("title")):
                    continue
                if w["class"] not in self.incerrables:
                    del self.abiertas[i]
                break
        elif accion == "clic_texto":
            self.clics.append(a["texto"])
            efecto = self.botones.get(a["texto"], {})
            if "error" in efecto:
                return {"error": efecto["error"]}
            if "abre" in efecto:
                self.abiertas.append({"class": efecto["abre"], "title": "t"})
            cierra = efecto.get("cierra", a["ventana"])
            if cierra and cierra not in self.incerrables:
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
