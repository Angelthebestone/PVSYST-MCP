"""Tests for medir_inventario.py: reading every tab of a window."""

from medir_inventario import inventariar
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
        *botones_de("Gráfico", origen="ocr"),
        {"texto": "Dibujada", "habilitado": True},
    ]

    r = inventariar(PVsystFalso({"TX": f}, {}), "TX")

    assert r["pestanas"][0]["botones"] == [
        {"texto": "Tabla", "habilitado": True, "origen": "window_text"},
        {"texto": "Gráfico", "habilitado": True, "origen": "ocr"},
        {"texto": "Dibujada", "habilitado": True, "origen": None},
    ]
