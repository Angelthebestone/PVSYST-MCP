"""Tests for closing the windows a press opened: what `cerrar_nuevas` leaves open."""

from medir_botones import cerrar_nuevas, pulsar_botones
from medir_cliente import Json
from medir_inventario import inventariar
from medir_proyectos import abrir
from pvsyst_falso import PVsystFalso, botones_de, formulario, objetivo_de

# pytest finds the autouse fixture through this import; the tests never call it.
from pvsyst_falso import sin_esperas  # noqa: F401


def test_cerrar_nuevas_compares_class_and_title_and_keeps_the_main_windows() -> None:
    c = PVsystFalso({}, {})
    c.abiertas += [
        {"class": "TA", "title": "vieja"},
        {"class": "TA", "title": "nueva"},
        {"class": "TA", "title": "otra"},
        {"class": "TProjectForm", "title": "Proyecto"},
    ]

    cerradas, parado = cerrar_nuevas(
        c, {("THomeForm", "PVsyst"), ("TA", "vieja"), ("TProjectForm", "Proyecto")}
    )

    assert parado is None

    assert [(e["clase"], e["titulo"]) for e in cerradas] == [
        ("TA", "nueva"),
        ("TA", "otra"),
    ]
    assert [w["title"] for w in c.abiertas] == ["PVsyst", "vieja", "Proyecto"]


def test_cerrar_nuevas_closes_by_class_and_never_by_a_title_the_host_shares() -> None:
    c = PVsystFalso({}, {})
    c.abiertas.append({"class": "TA", "title": "PVsyst"})

    cerradas, parado = cerrar_nuevas(c, {("THomeForm", "PVsyst")})

    assert parado is None and cerradas == [{"clase": "TA", "titulo": "PVsyst"}]
    assert c.cierres == ["TA"]
    assert "THomeForm" in [w["class"] for w in c.abiertas]


def test_cerrar_nuevas_refuses_a_new_window_of_a_class_it_keeps() -> None:
    c = PVsystFalso({}, {})
    c.abiertas.append({"class": "TProjectForm", "title": "Proyecto de otro"})

    cerradas, parado = cerrar_nuevas(c, {("THomeForm", "PVsyst")})

    assert parado is not None and "TProjectForm" in parado
    assert "Proyecto de otro" in parado
    assert c.cierres == [] and cerradas == []
    assert "TProjectForm" in [w["class"] for w in c.abiertas]


def test_cerrar_nuevas_names_what_stayed_open_and_the_close_refusal() -> None:
    c = PVsystFalso({}, {})
    c.abiertas.append({"class": "#32770", "title": "native dialog"})
    llamar = c.tool

    def tool(nombre: str, a: Json) -> Json:
        if a.get("accion") == "cerrar":
            return {"error": "the window is modal and refuses WM_CLOSE"}
        return llamar(nombre, a)

    c.tool = tool  # type: ignore[method-assign]

    cerradas, parado = cerrar_nuevas(c, {("THomeForm", "PVsyst")})

    assert cerradas == [{"clase": "#32770", "titulo": "native dialog"}]
    assert parado is not None
    assert "#32770" in parado and "native dialog" in parado
    assert "refuses WM_CLOSE" in parado


def test_pulsar_botones_stops_when_a_new_window_cannot_be_closed() -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = botones_de("Gráfico", "Tabla")
    c = PVsystFalso(
        {"TX": f}, {"a": "TX"}, {"Gráfico": {"abre": "#32770", "cierra": None}}
    )
    c.incerrables = {"#32770"}
    assert abrir(c, objetivo_de("TX")) is None
    intentos = c.acciones.count("cerrar")

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert c.clics == ["Gráfico"]
    assert len(abren) == 1
    assert abren[0]["abre"] == [{"clase": "#32770", "titulo": "t"}]
    assert "#32770" in abren[0]["parado"] and "'t'" in abren[0]["parado"]
    assert "#32770" in [w["class"] for w in c.abiertas]
    assert 1 <= c.acciones.count("cerrar") - intentos <= 12
