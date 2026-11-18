"""Tests for medir_ventanas.py: open, close and wait on the measured windows."""

import medir_ventanas
from medir_cliente import Json
from medir_ventanas import (
    abrir,
    asegurar_pvsyst,
    cargar_proyecto,
    limpiar,
    ventanas,
)
from pvsyst_falso import PVsystFalso, botones_de, formulario

# pytest finds the autouse fixture through this import; the tests never call it.
from pvsyst_falso import sin_esperas  # noqa: F401


def test_limpiar_answers_no_to_a_save_prompt_and_keeps_the_main_windows() -> None:
    f = formulario(["x"])
    f["textos"] = ["¿Desea guardarla?"]
    f["paginas"][0]["botones"] = botones_de("Sí", "No", "Cancelar")
    c = PVsystFalso({"TMessageForm": f}, {})
    c.abiertas += [
        {"class": "TProjectForm", "title": "P"},
        {"class": "TSomeForm", "title": "t"},
        {"class": "TMessageForm", "title": "m"},
    ]

    limpiar(c)

    assert [w["class"] for w in c.abiertas] == ["THomeForm", "TProjectForm"]
    assert c.clics == ["No"]


def test_ventanas_ignores_the_tooltips_a_pointer_raises() -> None:
    c = PVsystFalso({}, {})
    c.abiertas += [
        {"class": "THintWindow", "title": "Imprimir (Ctrl+P)"},
        {"class": "MicrosoftWindowsTooltip", "title": ""},
        {"class": "TSummaryForm", "title": "Resumen"},
    ]

    assert [w["class"] for w in ventanas(c)] == ["THomeForm", "TSummaryForm"]


def test_asegurar_pvsyst_launches_it_only_when_it_is_not_running() -> None:
    c = PVsystFalso({}, {})

    assert asegurar_pvsyst(c) is None
    assert "lanzar" not in c.acciones

    c.corriendo = False
    assert asegurar_pvsyst(c) is None and c.corriendo
    assert "lanzar" in c.acciones


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
    f["paginas"][0]["botones"] = botones_de("Sí", "No")
    c = PVsystFalso({"TMessageForm": f}, {})
    c.abiertas.append({"class": "TMessageForm", "title": "m"})

    assert limpiar(c) == [] and c.clics == ["Sí"]


def test_limpiar_leaves_an_unknown_dialog_open_instead_of_pressing_ok() -> None:
    f = formulario(["x"])
    f["textos"] = ["¿Desea crear un sitio nuevo?"]
    c = PVsystFalso({"TMessageForm": f}, {})
    c.abiertas.append({"class": "TMessageForm", "title": "m"})

    assert limpiar(c) == ["TMessageForm"]
    assert c.clics == []


def test_limpiar_rejects_an_unknown_dialog_with_cancelar() -> None:
    f = formulario(["x"])
    f["textos"] = ["Operación no reconocida"]
    f["paginas"][0]["botones"] = botones_de("OK", "Cancelar")
    c = PVsystFalso({"TMessageForm": f}, {})
    c.abiertas.append({"class": "TMessageForm", "title": "m"})

    assert limpiar(c) == []
    assert c.clics == ["Cancelar"]


def test_limpiar_reports_what_it_could_not_close() -> None:
    c = PVsystFalso({}, {})
    c.abiertas.append({"class": "TTerco", "title": "t"})
    c.incerrables = {"TTerco"}

    assert limpiar(c) == ["TTerco"]
