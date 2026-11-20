"""Tests for medir_gui.py: the command line, the manifest and what it merges."""

import json
from pathlib import Path
from typing import Any

import pytest

import medir_gui
from medir_gui import main
from medir_inventario import ErrorMedicion, inventariar, leer_pagina, volcar
from pvsyst_falso import PVsystFalso, botones_de, formulario

# pytest finds the autouse fixture through this import; the tests never call it.
from pvsyst_falso import sin_esperas  # noqa: F401


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


def test_main_stops_when_pvsyst_does_not_start(tmp_path: Path) -> None:
    c = PVsystFalso({"TA": formulario(["a"])}, {"a": "TA"})
    c.corriendo = c.lanza = False

    assert main(argumentos(tmp_path, "--solo", "TA"), fabrica_de(c)) == 1
    assert c.cerrado
    assert json.loads((tmp_path / "inventario.json").read_text())["TA"] == {
        "viejo": True
    }


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
    f["paginas"][0]["botones"] = botones_de("Mostrar histograma")

    c = PVsystFalso({"TA": f}, {"a": "TA"}, {"Mostrar histograma": {"cierra": None}})
    assert main(argumentos(tmp_path, "--solo", "TA"), fabrica_de(c)) == 0
    sin_botones = json.loads((tmp_path / "inventario.json").read_text())
    assert "botones_abren" not in sin_botones["TA"]

    c = PVsystFalso({"TA": f}, {"a": "TA"}, {"Mostrar histograma": {"cierra": None}})
    assert main(argumentos(tmp_path, "--solo", "TA", "--botones"), fabrica_de(c)) == 0
    con_botones = json.loads((tmp_path / "inventario.json").read_text())
    assert con_botones["TA"]["botones_abren"] == [
        {"pestana": None, "boton": "Mostrar histograma", "abre": [], "error": None}
    ]


def test_main_reports_presses_with_error_without_failing(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = botones_de("Tabla")
    c = PVsystFalso(
        {"TA": f},
        {"a": "TA"},
        {"Tabla": {"error": "'Tabla' is the caption of 2 buttons"}},
    )

    codigo = main(argumentos(tmp_path, "--solo", "TA", "--botones"), fabrica_de(c))

    assert codigo == 0
    assert "1 presses, 1 with error" in capsys.readouterr().out
    inventario = json.loads((tmp_path / "inventario.json").read_text())
    assert inventario["TA"]["errores"] == []
    assert "2 buttons" in inventario["TA"]["botones_abren"][0]["error"]


def test_main_without_botones_keeps_the_presses_of_the_previous_measurement(
    tmp_path: Path,
) -> None:
    f = formulario(["a"])
    f["paginas"][0]["botones"] = botones_de("Mostrar histograma")
    manifiesto, inventario, _ = preparar(tmp_path)
    args = [
        "--servidor",
        "pvsyst-mcp.exe",
        "--manifiesto",
        str(manifiesto),
        "--inventario",
        str(inventario),
    ]
    c = PVsystFalso({"TA": f}, {"a": "TA"}, {"Mostrar histograma": {"cierra": None}})
    assert main([*args, "--solo", "TA", "--botones"], fabrica_de(c)) == 0
    pulsados = json.loads(inventario.read_text())["TA"]["botones_abren"]

    c = PVsystFalso({"TA": f}, {"a": "TA"})
    assert main([*args, "--solo", "TA"], fabrica_de(c)) == 0

    medido = json.loads(inventario.read_text())
    assert medido["TA"]["botones_abren"] == pulsados


def test_main_merges_the_fields_when_a_press_cannot_reopen_the_host(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    f = formulario(["a"])
    f["paginas"][0]["botones"] = botones_de("Diagrama unifilar")
    c = PVsystFalso({"TA": f}, {"a": "TA"}, {"Diagrama unifilar": {"cierra": "TA"}})
    llamar = c.tool

    def tool(nombre: str, a: dict[str, Any]) -> dict[str, Any]:
        abierta = "TA" in [w["class"] for w in c.abiertas]
        if a.get("funcion") == "a" and c.funciones and not abierta:
            return {"error": "PVsyst is not running"}
        return llamar(nombre, a)

    c.tool = tool  # type: ignore[method-assign]

    codigo = main(argumentos(tmp_path, "--solo", "TA", "--botones"), fabrica_de(c))

    inventario = json.loads((tmp_path / "inventario.json").read_text())
    assert codigo == 1
    assert "1 stopped" in capsys.readouterr().out
    assert inventario["TA"]["pestanas"][0]["campos"][0]["etiqueta"] == "a"
    assert inventario["TA"]["errores"] == []
    assert "PVsyst is not running" in inventario["TA"]["botones_abren"][-1]["parado"]


def test_main_exits_1_and_keeps_the_old_entry_when_a_target_does_not_open(
    tmp_path: Path,
) -> None:
    c = PVsystFalso({"TB": formulario(["a"])}, {})

    codigo = main(argumentos(tmp_path, "--solo", "TB"), fabrica_de(c))

    assert codigo == 1
    assert "TB" not in json.loads((tmp_path / "inventario.json").read_text())


def test_main_closes_on_a_later_target_the_project_it_loaded_itself(
    tmp_path: Path,
) -> None:
    manifiesto = tmp_path / "manifiesto.json"
    manifiesto.write_text(
        json.dumps(
            [
                {
                    "clave": "TA",
                    "ventana": "TA",
                    "cadena": "@proyecto:Planta,variante.x",
                },
                {"clave": "TB", "ventana": "TB", "cadena": "herramientas.x"},
            ]
        ),
        encoding="utf-8",
    )
    inventario = tmp_path / "inventario.json"
    inventario.write_text(volcar({}) + "\n")
    c = PVsystFalso(
        {"TA": formulario(["a"]), "TB": formulario(["b"])},
        {"variante.x": "TA", "herramientas.x": "TB"},
    )
    args = [
        "--servidor",
        "pvsyst-mcp.exe",
        "--manifiesto",
        str(manifiesto),
        "--inventario",
        str(inventario),
    ]

    assert main(args, fabrica_de(c)) == 0
    assert set(json.loads(inventario.read_text())) == {"TA", "TB"}


def test_main_stops_when_the_windows_cannot_be_read(tmp_path: Path) -> None:
    c = PVsystFalso({"TA": formulario(["a"])}, {"a": "TA"})
    c.falla_ventanas = "access denied reading the windows"

    assert main(argumentos(tmp_path, "--solo", "TA"), fabrica_de(c)) == 1
    assert json.loads((tmp_path / "inventario.json").read_text())["TA"] == {
        "viejo": True
    }


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
