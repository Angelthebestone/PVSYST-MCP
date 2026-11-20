"""Tests for what stops a run and what it keeps: its windows, its presses, its fields.

The run-level helpers (`argumentos`, `fabrica_de`) belong to `test_medir_gui.py`, the
file that tests the command line; these tests drive the same `main` with them.
"""

import json
from pathlib import Path

import pytest

from medir_cliente import Json
from medir_gui import main
from medir_objetivo import medir
from pvsyst_falso import PVsystFalso, botones_de, formulario, objetivo_de
from test_medir_gui import argumentos, fabrica_de

# pytest finds the autouse fixture through this import; the tests never call it.
from pvsyst_falso import sin_esperas  # noqa: F401


def test_a_failed_read_during_the_presses_stops_them_and_merges_the_fields(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    f = formulario(["a"])
    f["paginas"][0]["botones"] = botones_de("Tabla", "Mostrar gráfico")
    c = PVsystFalso(
        {"TA": f},
        {"a": "TA"},
        {"Tabla": {"cierra": None}, "Mostrar gráfico": {"cierra": None}},
    )
    llamar = c.tool
    pulsado = False

    def tool(nombre: str, a: Json) -> Json:
        nonlocal pulsado
        pulsado = pulsado or a.get("texto") == "Tabla"
        if pulsado and a.get("que") == "ventanas":
            return {"error": "access denied reading the windows"}
        return llamar(nombre, a)

    c.tool = tool  # type: ignore[method-assign]

    codigo = main(argumentos(tmp_path, "--solo", "TA", "--botones"), fabrica_de(c))

    inventario = json.loads((tmp_path / "inventario.json").read_text())
    assert codigo == 1
    assert "1 stopped" in capsys.readouterr().out
    assert inventario["TA"]["pestanas"][0]["campos"][0]["etiqueta"] == "a"
    assert "access denied" in inventario["TA"]["botones_abren"][-1]["parado"]


def test_a_failed_cleanup_does_not_replace_the_error_that_stopped_the_target(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    c = PVsystFalso({"TA": formulario(["a"])}, {"a": "TA"})
    c.ilegibles = {"TA"}
    llamar = c.tool
    leido = False

    def tool(nombre: str, a: Json) -> Json:
        nonlocal leido
        if a.get("ventana") == "TA" and a.get("que") == "formulario":
            leido = True
        if leido and a.get("que") == "ventanas":
            return {"error": "access denied reading the windows"}
        return llamar(nombre, a)

    c.tool = tool  # type: ignore[method-assign]

    codigo = main(argumentos(tmp_path, "--solo", "TA"), fabrica_de(c))

    salida = capsys.readouterr().out
    assert codigo == 1
    assert "reading TA failed" in salida
    assert "and closing the target failed" in salida
    assert "access denied" in salida


def test_a_target_whose_cleanup_could_not_close_it_keeps_the_reason(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    c = PVsystFalso({"TA": formulario(["a"])}, {"a": "TA"})
    c.incerrables = {"TA"}

    codigo = main(argumentos(tmp_path, "--solo", "TA"), fabrica_de(c))

    salida = capsys.readouterr().out
    inventario = json.loads((tmp_path / "inventario.json").read_text())
    assert codigo == 1
    assert "closing it stopped: still open: ['TA']" in salida
    assert inventario["TA"]["parado"] == "still open: ['TA']"
    assert inventario["TA"]["pestanas"][0]["campos"][0]["etiqueta"] == "a"


def test_a_dialog_the_cleanup_cannot_answer_is_named_where_it_stopped() -> None:
    aviso = formulario([])
    aviso["textos"] = ["¿Desea continuar?"]
    c = PVsystFalso(
        {"TA": formulario(["a"]), "TMessageForm": aviso},
        {"a": "TA", "b": "TMessageForm"},
    )

    medido = medir(c, objetivo_de("TA", "a,b"))

    assert isinstance(medido, dict)
    assert "¿Desea continuar?" in medido["parado"]
    assert "OK" in medido["parado"]


def test_a_read_error_while_opening_the_target_closes_what_it_opened(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    c = PVsystFalso({"TA": formulario(["a"])}, {"a": "TA"})
    llamar = c.tool
    fallos = 0

    def tool(nombre: str, a: Json) -> Json:
        """The first read of the windows after the chain ran fails, once."""
        nonlocal fallos
        if c.funciones and a.get("que") == "ventanas" and fallos == 0:
            fallos += 1
            return {"error": "access denied reading the windows"}
        return llamar(nombre, a)

    c.tool = tool  # type: ignore[method-assign]

    codigo = main(argumentos(tmp_path, "--solo", "TA"), fabrica_de(c))

    salida = capsys.readouterr().out
    assert codigo == 1
    assert "reading the open windows failed" in salida
    assert "access denied" in salida
    assert "TA" not in [w["class"] for w in c.abiertas]


def test_a_run_refuses_a_session_it_did_not_start_with_a_window_it_did_not_open(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    c = PVsystFalso({"TA": formulario(["a"])}, {"a": "TA"})
    c.abiertas.append({"class": "TPVModuleForm", "title": "Módulo del usuario"})

    assert main(argumentos(tmp_path, "--solo", "TA"), fabrica_de(c)) == 1

    salida = capsys.readouterr().out
    assert "TPVModuleForm" in salida and "--limpiar-ajenas" in salida
    assert "TPVModuleForm" in [w["class"] for w in c.abiertas]
    assert json.loads((tmp_path / "inventario.json").read_text())["TA"] == {
        "viejo": True
    }

    c = PVsystFalso({"TA": formulario(["a"])}, {"a": "TA"})
    c.abiertas.append({"class": "TPVModuleForm", "title": "Módulo del usuario"})

    acepta = argumentos(tmp_path, "--solo", "TA", "--limpiar-ajenas")

    assert main(acepta, fabrica_de(c)) == 0
    assert "TPVModuleForm" not in [w["class"] for w in c.abiertas]


def test_a_jsonrpc_failure_of_a_chain_step_or_of_a_form_read_is_an_error(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """`abrir` and `inventariar` both read the refusal the client turns into a failure."""
    for que in ("funcion", "formulario"):
        c = PVsystFalso({"TA": formulario(["a"])}, {"a": "TA"})
        c.jsonrpc = {que}

        codigo = main(argumentos(tmp_path, "--solo", "TA"), fabrica_de(c))

        salida = capsys.readouterr().out
        inventario = json.loads((tmp_path / "inventario.json").read_text())
        assert codigo == 1, que
        assert que in salida, que
        assert inventario["TA"] == {"viejo": True}, que
