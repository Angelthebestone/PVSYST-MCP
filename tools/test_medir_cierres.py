"""Tests for closing the windows a press opened: what `cerrar_nuevas` leaves open."""

from medir_botones import cerrar_nuevas
from pvsyst_falso import PVsystFalso

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

    cerradas, parado = cerrar_nuevas(c, {("THomeForm", "PVsyst"), ("TA", "vieja")})

    assert parado is None

    assert [(e["clase"], e["titulo"]) for e in cerradas] == [
        ("TA", "nueva"),
        ("TA", "otra"),
    ]
    assert [w["title"] for w in c.abiertas] == ["PVsyst", "vieja", "Proyecto"]
