"""Tests for the window, project and wait steps of the measurement harness."""

import pytest

import medir_proyectos
from medir_cliente import Json
from medir_proyectos import abrir, cargar_proyecto
from medir_ventanas import ErrorDeLectura, asegurar_pvsyst, limpiar, ventanas
from pvsyst_falso import PVsystFalso, botones_de, formulario, objetivo_de, propiedad_de

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


def test_a_window_that_carries_no_class_is_a_read_error_too() -> None:
    c = PVsystFalso({}, {})
    c.abiertas.append({"title": "una ventana sin clase"})

    with pytest.raises(ErrorDeLectura) as fallo:
        ventanas(c)

    assert "una ventana sin clase" in str(fallo.value)


def test_a_failed_read_of_the_windows_raises_instead_of_answering_none() -> None:
    c = PVsystFalso({}, {})
    c.falla_ventanas = "access denied reading the windows"

    with pytest.raises(ErrorDeLectura) as fallo:
        ventanas(c)

    assert "access denied" in str(fallo.value)
    with pytest.raises(ErrorDeLectura):
        limpiar(c)


def test_asegurar_pvsyst_does_not_launch_over_a_read_it_could_not_do() -> None:
    c = PVsystFalso({}, {})
    c.falla_ventanas = "access denied reading the windows"

    fallo = asegurar_pvsyst(c)

    assert fallo is not None and "access denied" in fallo
    assert "lanzar" not in c.acciones


def test_asegurar_pvsyst_launches_it_only_when_it_is_not_running() -> None:
    c = PVsystFalso({}, {})

    assert asegurar_pvsyst(c) is None
    assert "lanzar" not in c.acciones

    c.corriendo = False
    assert asegurar_pvsyst(c) is None and c.corriendo
    assert "lanzar" in c.acciones


def test_asegurar_pvsyst_reads_the_absent_process_answer_in_spanish_too() -> None:
    c = PVsystFalso({}, {})
    c.falla_ventanas = "El proceso 'PVsyst.exe' no está en ejecución. Lanza la GUI"

    assert asegurar_pvsyst(c) is None
    assert "lanzar" in c.acciones


def test_abrir_runs_the_chain_and_reports_a_window_that_did_not_open() -> None:
    c = PVsystFalso({}, {"a": "TA", "b": "TB"})

    assert abrir(c, objetivo_de("TB", "a,b")) is None
    assert "TC did not open" in str(abrir(c, objetivo_de("TC")))
    assert "unknown function z" in str(abrir(c, objetivo_de("TA", "z")))


def test_abrir_a_chain_without_project_closes_the_loaded_one_first() -> None:
    c = PVsystFalso({}, {"herramientas.x": "TX"})
    c.abiertas.append({"class": "TProjectForm", "title": "Proyecto _DEMO_Commercial"})

    assert abrir(c, objetivo_de("TX", "herramientas.x")) is None
    assert [w["class"] for w in c.abiertas] == ["THomeForm", "TX"]


def test_a_chain_without_project_leaves_a_project_it_did_not_load_open() -> None:
    c = PVsystFalso({}, {"herramientas.x": "TX"})
    c.abiertas.append({"class": "TProjectForm", "title": "Proyecto Planta del cliente"})

    fallo = abrir(c, objetivo_de("TX", "herramientas.x"))

    assert fallo is not None
    assert "Planta del cliente" in fallo and "not loaded by this run" in fallo
    assert "TProjectForm" in [w["class"] for w in c.abiertas]
    assert c.clics == []


def test_a_chain_without_project_closes_a_project_this_run_loaded() -> None:
    c = PVsystFalso({}, {"variante.x": "TX", "herramientas.x": "TY"})
    propiedad = propiedad_de()

    cadena = "@proyecto:Planta del cliente,variante.x"
    assert abrir(c, objetivo_de("TX", cadena, propiedad)) is None
    assert abrir(c, objetivo_de("TY", "herramientas.x", propiedad)) is None

    assert [w["class"] for w in c.abiertas] == ["THomeForm", "TY"]


def test_a_project_window_that_refuses_the_close_is_an_error() -> None:
    c = PVsystFalso({}, {"herramientas.x": "TX"})
    c.abiertas.append({"class": "TProjectForm", "title": "Proyecto _DEMO_Commercial"})
    c.incerrables = {"TProjectForm"}

    fallo = abrir(c, objetivo_de("TX", "herramientas.x"))

    assert fallo is not None and "TProjectForm" in fallo


def test_cargar_proyecto_reports_the_windows_it_could_not_close() -> None:
    c = PVsystFalso({}, {})
    c.abiertas.append({"class": "TTerco", "title": "t"})
    c.incerrables = {"TTerco"}

    fallo = cargar_proyecto(c, "_DEMO_Foo", propiedad_de())

    assert fallo is not None and "TTerco" in fallo
    assert c.escrito == ""


def test_abrir_a_chain_with_project_keeps_the_loaded_one() -> None:
    c = PVsystFalso({}, {"variante.x": "TX"})
    c.abiertas.append({"class": "TProjectForm", "title": "Proyecto _DEMO_Commercial"})

    cadena = "@proyecto:_DEMO_Commercial,variante.x"
    assert abrir(c, objetivo_de("TX", cadena)) is None
    assert "TProjectForm" in [w["class"] for w in c.abiertas]


def test_abrir_tolerates_the_host_window_opening_another_one() -> None:
    c = PVsystFalso({}, {"a": "TA"})
    llamar = c.tool

    def tool(nombre: str, a: Json) -> Json:
        r = llamar(nombre, a)
        if a.get("accion") == "funcion":
            return {"error": "did not open the expected window TZ"}
        return r

    c.tool = tool  # type: ignore[method-assign]

    assert abrir(c, objetivo_de("TA")) is None


def test_cargar_proyecto_types_the_name_and_waits_for_its_window() -> None:
    c = PVsystFalso({}, {"variante.x": "TX"})

    assert abrir(c, objetivo_de("TX", "@proyecto:_DEMO_Foo,variante.x")) is None
    assert c.escrito == "_DEMO_Foo"
    assert any(
        w["class"] == "TProjectForm" and "_DEMO_Foo" in w["title"] for w in c.abiertas
    )
    # Already loaded: nothing is typed again.
    c.escrito = ""
    propiedad = propiedad_de()
    assert cargar_proyecto(c, "_DEMO_Foo", propiedad) is None
    assert c.escrito == ""


def test_cargar_proyecto_reports_when_the_project_did_not_open() -> None:
    c = PVsystFalso({}, {})
    c.tool = lambda nombre, a: {"windows": []} if a.get("que") == "ventanas" else {}  # type: ignore[method-assign]

    assert "did not load" in str(cargar_proyecto(c, "_DEMO_Foo", propiedad_de()))


def test_a_project_whose_name_contains_the_requested_one_is_not_it() -> None:
    c = PVsystFalso({}, {})
    c.abiertas.append({"class": "TProjectForm", "title": "Proyecto _DEMO_Commercial2"})
    c.abiertas.append({"class": "TProjectForm", "title": "Proyecto X_DEMO_Commercial"})

    assert medir_proyectos._es_proyecto(c.abiertas[-1], "_DEMO_Commercial") is False
    assert medir_proyectos._es_proyecto(c.abiertas[-2], "_DEMO_Commercial") is False
    ok = {"class": "TProjectForm", "title": "Proyecto _DEMO_Commercial - VC0"}
    assert medir_proyectos._es_proyecto(ok, "_DEMO_Commercial") is True


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


def test_limpiar_acknowledges_a_notice_that_has_only_ok_and_asks_nothing() -> None:
    f = formulario(["x"])
    f["textos"] = ["advertencia", "Por favor elija al menos una variable"]
    c = PVsystFalso({"TMessageForm": f}, {})
    c.abiertas.append({"class": "TMessageForm", "title": "m"})

    assert limpiar(c) == [] and c.clics == ["OK"]


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


def test_limpiar_leaves_the_trial_notice_while_a_project_is_open_and_closes_it_after() -> (
    None
):
    c = PVsystFalso({}, {})
    c.abiertas += [
        {"class": "TProjectForm", "title": "Proyecto: _DEMO_X.PRJ"},
        {"class": "TDEMOForm", "title": "Modo DEMO"},
    ]
    c.incerrables = {"TDEMOForm"}

    assert limpiar(c) == []

    c.abiertas = [w for w in c.abiertas if w["class"] != "TProjectForm"]

    assert limpiar(c) == ["TDEMOForm"]
