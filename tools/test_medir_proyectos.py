"""Tests for which project windows a run may close: the demos and what it loaded."""

from medir_proyectos import abrir
from pvsyst_falso import PVsystFalso, objetivo_de, propiedad_de

# pytest finds the autouse fixture through this import; the tests never call it.
from pvsyst_falso import sin_esperas  # noqa: F401


def test_only_a_demo_or_a_project_the_run_loaded_may_be_closed() -> None:
    """A demo is a whole name of the shipped list and a loaded name is the whole name."""
    casos = [
        ("Proyecto _DEMO_Commercial", True),
        ("Proyecto _DEMO_Commercial - VC0", True),
        ("Proyecto _DEMO_mio", False),
        ("Proyecto X_DEMO_Foo", False),
        ("Proyecto Copia (_DEMO_Commercial)", False),
        ("Proyecto Planta 2", False),
    ]
    for titulo, propio in casos:
        c = PVsystFalso({}, {"herramientas.x": "TX"})
        c.abiertas.append({"class": "TProjectForm", "title": titulo})
        objetivo = objetivo_de("TX", "herramientas.x", propiedad_de("Planta"))

        fallo = abrir(c, objetivo)

        assert (fallo is None) == propio, titulo
        cerrado = "TProjectForm" not in [w["class"] for w in c.abiertas]
        assert cerrado == propio, titulo
