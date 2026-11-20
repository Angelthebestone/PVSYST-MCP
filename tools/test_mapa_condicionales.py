"""Tests of `requiere`: the measured switches of a tab written on its map page."""

import copy

import pytest

from mapa_condicionales import aplicar, Json


def mapa_de_prueba() -> Json:
    def pagina(pestana: str) -> Json:
        return {"pestana": pestana, "campos": [], "botones": []}

    return {
        "ventanas": {
            "TX": {
                "instancias": [
                    {
                        "abierta_con": "variante.x",
                        "paginas": [pagina("Uno"), pagina("Dos")],
                    },
                    {"abierta_con": "otra", "paginas": [pagina("Uno")]},
                ]
            },
            "TSolo": {
                "instancias": [{"abierta_con": "f", "paginas": [pagina("Única")]}]
            },
        }
    }


CONDICIONES = [{"campo": "Activo", "valor": True, "aparecen": ["a"], "desaparecen": []}]


def entrada(**cambios: object) -> Json:
    base: Json = {
        "ventana": "TX",
        "abierta_con": "variante.x",
        "pestana": "Dos",
        "condiciones": CONDICIONES,
    }
    return {"TX#dos": {**base, **cambios}}


def test_aplicar_writes_requiere_on_the_page_of_that_tab_of_that_instance() -> None:
    mapa = mapa_de_prueba()

    aplicar(mapa, entrada())

    paginas = mapa["ventanas"]["TX"]["instancias"][0]["paginas"]
    assert paginas[1]["requiere"] == CONDICIONES
    assert "requiere" not in paginas[0]
    assert "requiere" not in mapa["ventanas"]["TX"]["instancias"][1]["paginas"][0]


def test_aplicar_takes_the_only_page_of_a_window_without_tabs() -> None:
    mapa = mapa_de_prueba()

    aplicar(mapa, entrada(ventana="TSolo", abierta_con="f", pestana=None))

    assert mapa["ventanas"]["TSolo"]["instancias"][0]["paginas"][0]["requiere"]


def test_aplicar_is_idempotent_and_does_not_share_the_list() -> None:
    mapa = mapa_de_prueba()
    aplicar(mapa, entrada())
    una_vez = copy.deepcopy(mapa)

    aplicar(mapa, entrada())

    assert mapa == una_vez
    assert (
        mapa["ventanas"]["TX"]["instancias"][0]["paginas"][1]["requiere"]
        is not CONDICIONES
    )


@pytest.mark.parametrize(
    "cambios",
    [
        {"ventana": "TNo"},
        {"abierta_con": "no existe"},
        {"pestana": "No existe"},
        {"pestana": None},
    ],
)
def test_aplicar_names_the_entry_that_matches_no_single_page(cambios: Json) -> None:
    with pytest.raises(KeyError, match="TX#dos"):
        aplicar(mapa_de_prueba(), entrada(**cambios))
