"""Tests for medir_gui.py against a fake MCP client (no PVsyst, no server)."""

import json
import sys
from pathlib import Path
from typing import Any

import pytest

import medir_cliente
import medir_gui
import medir_ventanas
from medir_botones import pulsar_botones
from medir_cliente import ClienteMcp, Json
from medir_gui import main
from medir_inventario import ErrorMedicion, inventariar, leer_pagina, volcar
from medir_ventanas import abrir, asegurar_pvsyst, cargar_proyecto, limpiar


@pytest.fixture(autouse=True)
def _sin_esperas(monkeypatch: pytest.MonkeyPatch) -> None:
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


def test_inventariar_reads_every_tab_with_its_fields_and_captions() -> None:
    f = formulario(["a", "b"], ["c"], pestanas=["Uno", "Dos"])
    c = PVsystFalso({"TX": f}, {})

    r = inventariar(c, "TX")

    assert [(p["indice"], p["pestana"]) for p in r["pestanas"]] == [
        (0, "Uno"),
        (1, "Dos"),
    ]
    assert [[x["etiqueta"] for x in p["campos"]] for p in r["pestanas"]] == [
        ["a", "b"],
        ["c"],
    ]
    assert r["pestanas"][1]["rotulos"] == [["TTabSheet", "hoja 1"]]
    assert "pestana" not in r["pestanas"][0]["campos"][0]
    assert r["errores"] == [] and c.pagina["TX"] == 0


def test_inventariar_a_window_without_tabs_gives_one_page_with_no_index() -> None:
    c = PVsystFalso({"TX": formulario(["a"])}, {})

    p = inventariar(c, "TX")["pestanas"]

    assert len(p) == 1 and p[0]["indice"] is None and p[0]["pestana"] is None


def test_pulsar_botones_presses_each_tab_and_records_the_window_it_opened() -> None:
    f = formulario(["a"], ["b"], pestanas=["Uno", "Dos"])
    f["paginas"][0]["botones"] = botones_de("Abrir")
    f["paginas"][1]["botones"] = botones_de("Consultar")
    c = PVsystFalso(
        {"TX": f},
        {"a": "TX"},
        {"Abrir": {"abre": "TY", "cierra": None}, "Consultar": {"cierra": None}},
    )
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert [(e["pestana"], e["boton"]) for e in abren] == [
        ("Uno", "Abrir"),
        ("Dos", "Consultar"),
    ]
    assert abren[0]["abre"] == [{"clase": "TY", "titulo": "t"}]
    assert abren[1]["abre"] == [] and abren[1]["error"] is None
    assert [w["class"] for w in c.abiertas] == ["THomeForm", "TX"]


def test_pulsar_botones_never_presses_a_denied_disabled_or_empty_button() -> None:
    denegados = [
        "OK",
        "Cancelar",
        "Cerrar",
        "Guardar variante",
        "Eliminar",
        "Borrar fila",
        "Pegar",
        "Copiar",
        "Establecer",
        "Aplicar",
        "Optimizar",
        "Linealizar",
        "Imprimir",
        "Exportar",
        "Siguiente",
        "Previo",
        "Añadir",
        "Nuevo",
        "Importar",
        "Leer / Importar",
        "Horizonte despejado",
        "Reiniciar",
        "Reset",
        # Case and accents do not save a text from the list.
        "GUARDAR todo",
        "Anadir fila",
        "Añadir fila",
    ]
    f = formulario([])
    f["paginas"][0]["botones"] = [
        *botones_de(*denegados, "Ver detalles"),
        {"texto": "No habilitado", "habilitado": False},
        {"texto": "", "habilitado": True},
        {"texto": None, "habilitado": True},
    ]
    c = PVsystFalso({"TX": f}, {"a": "TX"}, {"Ver detalles": {"cierra": None}})
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert [e["boton"] for e in abren] == ["Ver detalles"]
    assert c.clics == ["Ver detalles"]


def test_pulsar_botones_answers_the_prompt_a_button_opens_and_records_it() -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = botones_de("Recalcular")
    pregunta = formulario([])
    pregunta["textos"] = ["¿Desea guardarla?"]
    c = PVsystFalso(
        {"TX": f, "TMessageForm": pregunta},
        {"a": "TX"},
        {"Recalcular": {"abre": "TMessageForm", "cierra": None}},
    )
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert abren[0]["abre"] == [
        {"clase": "TMessageForm", "titulo": "t", "cierra": "No"}
    ]
    assert c.clics == ["Recalcular", "No"]
    assert "TMessageForm" not in [w["class"] for w in c.abiertas]


def test_pulsar_botones_records_an_ambiguous_text_and_does_not_retry_it() -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = botones_de("Abrir")
    c = PVsystFalso(
        {"TX": f},
        {"a": "TX"},
        {"Abrir": {"error": "'Abrir' names 2 visible controls (0x1, 0x2)"}},
    )
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert len(abren) == 1 and abren[0]["abre"] == []
    assert "2 visible controls" in abren[0]["error"]
    assert c.clics == ["Abrir"]


def test_pulsar_botones_reopens_the_host_the_press_closed() -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = botones_de("Recalcular")
    c = PVsystFalso({"TX": f}, {"a": "TX"}, {"Recalcular": {"cierra": "TX"}})
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert c.funciones == ["a", "a"]
    assert [w["class"] for w in c.abiertas] == ["THomeForm", "TX"]
    assert abren == [
        {"pestana": None, "boton": "Recalcular", "abre": [], "error": None}
    ]


def test_pulsar_botones_records_the_host_it_could_not_reopen() -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = botones_de("Recalcular")
    c = PVsystFalso({"TX": f}, {"a": "TX"}, {"Recalcular": {"cierra": "TX"}})
    assert abrir(c, "a", "TX") is None
    llamar = c.tool

    def tool(nombre: str, a: Json) -> Json:
        abierta = "TX" in [w["class"] for w in c.abiertas]
        if a.get("funcion") == "a" and not abierta:
            return {"error": "PVsyst is not running"}
        return llamar(nombre, a)

    c.tool = tool  # type: ignore[method-assign]
    medida = inventariar(c, "TX")

    abren = pulsar_botones(c, objetivo_de("TX"), medida)

    assert len(abren) == 1
    assert "PVsyst is not running" in medida["errores"][0]["texto"]


def test_pulsar_botones_stops_at_forty_presses() -> None:
    f = formulario([])
    textos = [f"Detalle {i}" for i in range(41)]
    f["paginas"][0]["botones"] = botones_de(*textos)
    c = PVsystFalso({"TX": f}, {"a": "TX"}, {t: {"cierra": None} for t in textos})
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert len(abren) == len(c.clics) == 40


def test_limpiar_answers_no_to_a_save_prompt_and_keeps_the_main_windows() -> None:
    f = formulario(["x"])
    f["textos"] = ["¿Desea guardarla?"]
    c = PVsystFalso({"TMessageForm": f}, {})
    c.abiertas += [
        {"class": "TProjectForm", "title": "P"},
        {"class": "TSomeForm", "title": "t"},
        {"class": "TMessageForm", "title": "m"},
    ]

    limpiar(c)

    assert [w["class"] for w in c.abiertas] == ["THomeForm", "TProjectForm"]
    assert c.clics == ["No"]


def test_asegurar_pvsyst_launches_it_only_when_it_is_not_running() -> None:
    c = PVsystFalso({}, {})

    assert asegurar_pvsyst(c) is None
    assert "lanzar" not in c.acciones

    c.corriendo = False
    assert asegurar_pvsyst(c) is None and c.corriendo
    assert "lanzar" in c.acciones


def test_main_stops_when_pvsyst_does_not_start(tmp_path: Path) -> None:
    c = PVsystFalso({"TA": formulario(["a"])}, {"a": "TA"})
    c.corriendo = c.lanza = False

    assert main(argumentos(tmp_path, "--solo", "TA"), fabrica_de(c)) == 1
    assert c.cerrado
    assert json.loads((tmp_path / "inventario.json").read_text())["TA"] == {
        "viejo": True
    }


def test_abrir_runs_the_chain_and_reports_a_window_that_did_not_open() -> None:
    c = PVsystFalso({}, {"a": "TA", "b": "TB"})

    assert abrir(c, "a,b", "TB") is None
    assert "TC did not open" in str(abrir(c, "a", "TC"))
    assert "unknown function z" in str(abrir(c, "z", "TA"))


def test_abrir_tolerates_the_host_window_opening_another_one() -> None:
    c = PVsystFalso({}, {"a": "TA"})
    llamar = c.tool

    def tool(nombre: str, a: Json) -> Json:
        r = llamar(nombre, a)
        if a.get("accion") == "funcion":
            return {"error": "did not open the expected window TZ"}
        return r

    c.tool = tool  # type: ignore[method-assign]

    assert abrir(c, "a", "TA") is None


def test_cargar_proyecto_types_the_name_and_waits_for_its_window() -> None:
    c = PVsystFalso({}, {"variante.x": "TX"})

    assert abrir(c, "@proyecto:_DEMO_Foo,variante.x", "TX") is None
    assert c.escrito == "_DEMO_Foo"
    assert any(
        w["class"] == "TProjectForm" and "_DEMO_Foo" in w["title"] for w in c.abiertas
    )
    # Already loaded: nothing is typed again.
    c.escrito = ""
    assert cargar_proyecto(c, "_DEMO_Foo") is None and c.escrito == ""


def test_cargar_proyecto_reports_when_the_project_did_not_open() -> None:
    c = PVsystFalso({}, {})
    c.tool = lambda nombre, a: {"windows": []} if a.get("que") == "ventanas" else {}  # type: ignore[method-assign]

    assert "did not load" in str(cargar_proyecto(c, "_DEMO_Foo"))


def preparar(tmp_path: Path) -> tuple[Path, Path, Path]:
    manifiesto = tmp_path / "manifiesto.json"
    manifiesto.write_text(
        json.dumps(
            [
                {"clave": "TA", "ventana": "TA", "cadena": "a"},
                {"clave": "TA#otra", "ventana": "TA", "cadena": "b"},
                {"clave": "TB", "ventana": "TB", "cadena": "falta"},
            ]
        ),
        encoding="utf-8",
    )
    inventario = tmp_path / "inventario.json"
    inventario.write_text(
        volcar({"TA": {"viejo": True}, "TZ": {"intacto": True}}) + "\n"
    )
    return manifiesto, inventario, tmp_path


def argumentos(tmp_path: Path, *extra: str) -> list[str]:
    manifiesto, inventario, _ = preparar(tmp_path)
    return [
        "--servidor",
        "pvsyst-mcp.exe",
        "--manifiesto",
        str(manifiesto),
        "--inventario",
        str(inventario),
        *extra,
    ]


def fabrica_de(c: PVsystFalso) -> Any:
    return lambda servidor, workspace: c


def test_main_merges_what_it_measured_and_leaves_the_rest_of_the_inventory(
    tmp_path: Path,
) -> None:
    c = PVsystFalso({"TA": formulario(["a"])}, {"a": "TA", "b": "TA"})

    codigo = main(argumentos(tmp_path, "--solo", "TA", "TA#otra"), fabrica_de(c))

    inventario = json.loads((tmp_path / "inventario.json").read_text())
    assert codigo == 0 and c.cerrado
    assert inventario["TZ"] == {"intacto": True}
    assert set(inventario) == {"TA", "TA#otra", "TZ"}
    assert inventario["TA#otra"]["ventana"] == "TA"
    assert inventario["TA"]["pestanas"][0]["campos"][0]["etiqueta"] == "a"


def test_main_merges_botones_abren_only_when_it_pressed_them(tmp_path: Path) -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = botones_de("Ver detalles")

    c = PVsystFalso({"TA": f}, {"a": "TA"}, {"Ver detalles": {"cierra": None}})
    assert main(argumentos(tmp_path, "--solo", "TA"), fabrica_de(c)) == 0
    sin_botones = json.loads((tmp_path / "inventario.json").read_text())
    assert "botones_abren" not in sin_botones["TA"]

    c = PVsystFalso({"TA": f}, {"a": "TA"}, {"Ver detalles": {"cierra": None}})
    assert main(argumentos(tmp_path, "--solo", "TA", "--botones"), fabrica_de(c)) == 0
    con_botones = json.loads((tmp_path / "inventario.json").read_text())
    assert con_botones["TA"]["botones_abren"] == [
        {"pestana": None, "boton": "Ver detalles", "abre": [], "error": None}
    ]


def test_main_exits_1_and_keeps_the_old_entry_when_a_target_does_not_open(
    tmp_path: Path,
) -> None:
    c = PVsystFalso({"TB": formulario(["a"])}, {})

    codigo = main(argumentos(tmp_path, "--solo", "TB"), fabrica_de(c))

    assert codigo == 1
    assert "TB" not in json.loads((tmp_path / "inventario.json").read_text())


def test_main_refuses_a_key_that_is_not_in_the_manifest(tmp_path: Path) -> None:
    c = PVsystFalso({}, {})

    assert main(argumentos(tmp_path, "--solo", "TQ"), fabrica_de(c)) == 2
    assert not c.cerrado  # it never started the server


def test_the_committed_manifest_matches_the_committed_inventory() -> None:
    manifiesto = json.loads(medir_gui.MANIFIESTO.read_text(encoding="utf-8"))
    inventario = json.loads(medir_gui.INVENTARIO.read_text(encoding="utf-8"))

    claves = [o["clave"] for o in manifiesto]
    assert len(claves) == len(set(claves))
    for o in manifiesto:
        assert o["clave"] in inventario, o["clave"]
        assert inventario[o["clave"]]["ventana"] == o["ventana"], o["clave"]


def test_a_failed_read_is_an_error_and_nothing_is_merged(tmp_path: Path) -> None:
    c = PVsystFalso({"TA": formulario(["a"])}, {"a": "TA"})
    c.ilegibles = {"TA"}

    with pytest.raises(ErrorMedicion):
        leer_pagina(c, "TA")
    assert main(argumentos(tmp_path, "--solo", "TA"), fabrica_de(c)) == 1
    assert json.loads((tmp_path / "inventario.json").read_text())["TA"] == {
        "viejo": True
    }


def test_a_window_that_reads_back_empty_is_not_merged(tmp_path: Path) -> None:
    vacia = formulario([])
    vacia["paginas"][0]["botones"] = []
    c = PVsystFalso({"TA": vacia}, {"a": "TA"})

    assert main(argumentos(tmp_path, "--solo", "TA"), fabrica_de(c)) == 1
    assert json.loads((tmp_path / "inventario.json").read_text())["TA"] == {
        "viejo": True
    }


def test_a_vetoed_tab_change_saves_no_page_for_that_tab_and_fails(
    tmp_path: Path,
) -> None:
    f = formulario(["p0"], ["p1"], pestanas=["Uno", "Dos"])
    c = PVsystFalso({"TA": f}, {"a": "TA"})
    c.vetadas = {1}

    r = inventariar(c, "TA")

    assert [p["pestana"] for p in r["pestanas"]] == ["Uno"]
    assert r["errores"][0]["pestana"] == "Dos"
    assert main(argumentos(tmp_path, "--solo", "TA"), fabrica_de(c)) == 1
    assert json.loads((tmp_path / "inventario.json").read_text())["TA"] == {
        "viejo": True
    }


def test_a_project_whose_name_contains_the_requested_one_is_not_it() -> None:
    c = PVsystFalso({}, {})
    c.abiertas.append({"class": "TProjectForm", "title": "Proyecto _DEMO_Commercial2"})
    c.abiertas.append({"class": "TProjectForm", "title": "Proyecto X_DEMO_Commercial"})

    assert medir_ventanas._es_proyecto(c.abiertas[-1], "_DEMO_Commercial") is False
    assert medir_ventanas._es_proyecto(c.abiertas[-2], "_DEMO_Commercial") is False
    ok = {"class": "TProjectForm", "title": "Proyecto _DEMO_Commercial - VC0"}
    assert medir_ventanas._es_proyecto(ok, "_DEMO_Commercial") is True


def test_the_discard_question_is_answered_si_and_the_save_one_no() -> None:
    f = formulario(["x"])
    f["textos"] = ["¿Quiere cancelar las modificaciones?"]
    c = PVsystFalso({"TMessageForm": f}, {})
    c.abiertas.append({"class": "TMessageForm", "title": "m"})

    assert limpiar(c) == [] and c.clics == ["Sí"]


def test_limpiar_reports_what_it_could_not_close() -> None:
    c = PVsystFalso({}, {})
    c.abiertas.append({"class": "TTerco", "title": "t"})
    c.incerrables = {"TTerco"}

    assert limpiar(c) == ["TTerco"]


def test_a_server_that_never_answers_times_out_instead_of_hanging(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setattr(medir_cliente, "PLAZO", 1.0)
    # Answers `initialize`, then reads forever without answering anything else.
    servidor = (
        "import sys, json\n"
        "l = sys.stdin.readline()\n"
        "print(json.dumps({'jsonrpc': '2.0', 'id': 1, 'result': {}}), flush=True)\n"
        "sys.stdin.read()\n"
    )
    c = ClienteMcp(Path(sys.executable), Path("."), ("-c", servidor))
    try:
        with pytest.raises(TimeoutError, match="no answer to tools/call"):
            c.tool("gui_observar", {"que": "ventanas"})
    finally:
        c.cerrar()
