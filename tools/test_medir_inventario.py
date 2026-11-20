"""Tests for medir_inventario.py: reading every tab of a window."""

import pytest

from medir_cliente import Json
from medir_inventario import ErrorMedicion, fusionar, inventariar
from medir_ventanas import ventanas
from pvsyst_falso import PVsystFalso, botones_de, formulario

# pytest finds the autouse fixture through this import; the tests never call it.
from pvsyst_falso import sin_esperas  # noqa: F401


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


def test_inventariar_skips_a_listed_tab_that_is_not_a_page() -> None:
    f = formulario(["a"], ["b"], ["panel"], pestanas=["Uno", "Dos", "Sugerencias"])
    c = PVsystFalso({"TX": f}, {})
    c.fuera_de_rango = {2}

    r = inventariar(c, "TX")

    assert [p["pestana"] for p in r["pestanas"]] == ["Uno", "Dos"]
    assert r["errores"] == []


def test_inventariar_a_window_without_tabs_gives_one_page_with_no_index() -> None:
    c = PVsystFalso({"TX": formulario(["a"])}, {})

    p = inventariar(c, "TX")["pestanas"]

    assert len(p) == 1 and p[0]["indice"] is None and p[0]["pestana"] is None


def test_inventariar_records_where_the_text_of_each_button_was_read() -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = [
        *botones_de("Tabla"),
        *botones_de("Gráfico", etiqueta_fuente="ocr"),
        {"texto": "Dibujada", "habilitado": True},
    ]

    r = inventariar(PVsystFalso({"TX": f}, {}), "TX")

    assert r["pestanas"][0]["botones"] == [
        {"texto": "Tabla", "habilitado": True, "origen": "window_text"},
        {"texto": "Gráfico", "habilitado": True, "origen": "ocr"},
        {"texto": "Dibujada", "habilitado": True, "origen": None},
    ]


def _abre_el_aviso_al_cambiar_de_pestana(c: PVsystFalso) -> None:
    """Every change to tab 1 raises a TMessageForm, once: the fake cannot say it."""
    llamar = c.tool

    def tool(nombre: str, a: Json) -> Json:
        r = llamar(nombre, a)
        abierto = any(w["class"] == "TMessageForm" for w in c.abiertas)
        if a.get("accion") == "pagina" and a.get("indice") == 1 and not abierto:
            c.abiertas.append({"class": "TMessageForm", "title": "m"})
        return r

    c.tool = tool  # type: ignore[method-assign]


def _con_aviso(textos: list[str], *botones: str) -> PVsystFalso:
    """The measured window with two tabs and a warning that answers `botones`."""
    f = formulario(["a"], ["b"], pestanas=["Uno", "Dos"])
    aviso = formulario([])
    aviso["textos"] = textos
    aviso["paginas"][0]["botones"] = botones_de(*botones)
    c = PVsystFalso({"TX": f, "TMessageForm": aviso}, {})
    _abre_el_aviso_al_cambiar_de_pestana(c)
    return c


def test_a_tab_dialog_is_answered_with_the_rejection_of_every_other_closer() -> None:
    c = _con_aviso(["Los datos se perderán. ¿Desea continuar?"], "OK", "Cancelar")

    r = inventariar(c, "TX")

    assert c.clics == ["Cancelar"]
    assert r["errores"] == [
        {"indice": 1, "dialogo": ["Los datos se perderán. ¿Desea continuar?"]}
    ]
    assert "TMessageForm" not in [w["class"] for w in ventanas(c)]


def test_a_tab_dialog_with_no_rejection_stops_the_measurement_naming_it() -> None:
    c = _con_aviso(["¿Desea continuar?"], "OK")

    with pytest.raises(ErrorMedicion) as fallo:
        inventariar(c, "TX")

    assert "¿Desea continuar?" in str(fallo.value)
    assert c.clics == []
    assert "TMessageForm" in [w["class"] for w in ventanas(c)]


def test_fusionar_drops_the_old_presses_when_the_tabs_change() -> None:
    vieja: Json = {"indice": 0, "pestana": "Vieja", "campos": [], "botones": []}
    inventario: Json = {
        "TX": {
            "ventana": "TX",
            "titulo": "Ventana de prueba",
            "pestanas": [vieja],
            "errores": [],
            "botones_abren": [{"boton": "Tabla", "abre": [], "error": None}],
        }
    }
    medida = inventariar(PVsystFalso({"TX": formulario(["a"])}, {}), "TX")

    fusionar(inventario, "TX", medida)

    assert "botones_abren" not in inventario["TX"]
    assert inventario["TX"]["pestanas"] == medida["pestanas"]
