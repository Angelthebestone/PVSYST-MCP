"""Tests for the regeneration of docs/gui_mapa.json from the tab inventory."""

import json
from pathlib import Path

import pytest

import mapa_desde_inventario as mapa_py
from mapa_desde_inventario import (
    Json,
    Remedio,
    cargar,
    main,
    remedir,
    serializar,
)

BOTONES = [{"texto": "OK", "habilitado": True}]


def campo(etiqueta: str) -> Json:
    return {"clase": "TEdit", "etiqueta": etiqueta, "marcado": None, "habilitado": True}


def pagina_vieja(pestana: str | None, indice: int | None, campos: int) -> Json:
    return {
        "pestana": pestana,
        "campos": [{"clase": "TEdit", "etiqueta": f"c{n}"} for n in range(campos)],
        "botones": BOTONES,
        "rotulos": [["TTabSheet", f"rotulo de {pestana}"]],
        "sin_rotulo": {"TEdit": 1},
        "indice_pestana": indice,
    }


def instancia(abierta_con: str, paginas: list[Json]) -> Json:
    return {
        "titulo": "Ventana <variante>",
        "abierta_con": abierta_con,
        "pestanas": [p["pestana"] for p in paginas if p["pestana"]],
        "paginas": paginas,
    }


def mapa_de_prueba() -> Json:
    """A window truncated at 13 fields and without its first tab, a tracker
    window, and one the inventory does not touch."""
    pestanas = [pagina_vieja("Uno", 0, 13), pagina_vieja("Dos", 1, 13)]
    return {
        "version_pvsyst": "8.1.8",
        "ventanas": {
            "TPerdidasForm": {
                "titulos": ["Ventana <variante>"],
                "abierta_con": ["variante.perdidas", "bombeo > Perdidas"],
                "instancias": [
                    instancia("variante.perdidas", pestanas),
                    instancia("bombeo > Perdidas", pestanas[:1]),
                ],
                "controles": [],
            },
            "TSeguidorForm": {
                "titulos": ["Orientacion"],
                "abierta_con": ["variante.orientacion"],
                "instancias": [
                    instancia("variante.orientacion", [pagina_vieja(None, None, 13)])
                ],
                "controles": [],
            },
            "TAjenaForm": {
                "titulos": ["Ajena"],
                "abierta_con": ["x"],
                "instancias": [instancia("x", [pagina_vieja(None, None, 13)])],
                "controles": [["TTabSheet", "rotulo de None"]],
            },
        },
        "limites": ["uno"],
    }


def inventario_de_prueba() -> Json:
    """`Cero` is a tab the map never had; `Uno` has 20 fields, the map kept 13."""
    return {
        "TPerdidasForm": {
            "ventana": "TPerdidasForm",
            "titulo": "Ventana Basic system",
            "pestanas": [
                {
                    "indice": 0,
                    "pestana": "Cero",
                    "campos": [campo("a")],
                    "botones": BOTONES,
                },
                {
                    "indice": 1,
                    "pestana": "Uno",
                    "campos": [campo(f"u{n}") for n in range(20)],
                    "botones": BOTONES,
                },
                {
                    "indice": 2,
                    "pestana": "Dos",
                    "campos": [campo("d")],
                    "botones": BOTONES,
                },
            ],
            "errores": [],
        },
        "TSeguidorForm": {
            "ventana": "TSeguidorForm",
            "titulo": "Orientacion",
            "pestanas": [
                {
                    "indice": None,
                    "pestana": None,
                    "campos": [campo("tilt")],
                    "botones": BOTONES,
                }
            ],
            "errores": [],
        },
    }


@pytest.fixture
def remedios(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(
        mapa_py,
        "REMEDIOS",
        {
            "TPerdidasForm": Remedio(sin_rotulos_de=frozenset({"Dos"})),
            "TSeguidorForm": Remedio(anadir_con="variante.orientacion"),
        },
    )


@pytest.mark.usefixtures("remedios")
def test_regenerates_a_window_without_truncating_and_with_its_added_tab() -> None:
    original = mapa_de_prueba()
    nuevo = remedir(original, inventario_de_prueba())

    paginas = nuevo["ventanas"]["TPerdidasForm"]["instancias"][0]["paginas"]
    assert [(p["pestana"], p["indice_pestana"]) for p in paginas] == [
        ("Cero", 0),
        ("Uno", 1),
        ("Dos", 2),
    ]
    assert len(paginas[1]["campos"]) == 20
    assert paginas[1]["campos"][0]["habilitado"] is True
    # The caption of a surviving tab stays; a new tab, or one whose state changed, has none.
    assert paginas[1]["rotulos"] == [["TTabSheet", "rotulo de Uno"]]
    assert "rotulos" not in paginas[0]
    assert "rotulos" not in paginas[2]
    nombres = nuevo["ventanas"]["TPerdidasForm"]["instancias"][0]["pestanas"]
    assert nombres == ["Cero", "Uno", "Dos"]
    assert original == mapa_de_prueba(), "remedir must not mutate its input"


@pytest.mark.usefixtures("remedios")
def test_keeps_everything_else_of_the_map_byte_for_byte() -> None:
    original = mapa_de_prueba()
    nuevo = remedir(original, inventario_de_prueba())

    assert nuevo["ventanas"]["TAjenaForm"] == original["ventanas"]["TAjenaForm"]
    assert (
        nuevo["ventanas"]["TPerdidasForm"]["instancias"][1]
        == (original["ventanas"]["TPerdidasForm"]["instancias"][1])
    )
    assert list(nuevo) == list(original)
    sin_tocar = {k: v for k, v in nuevo.items() if k != "ventanas"}
    assert serializar(sin_tocar) == serializar(
        {k: v for k, v in original.items() if k != "ventanas"}
    )


@pytest.mark.usefixtures("remedios")
def test_adds_the_fixed_tilt_instance_and_keeps_the_tracker_one() -> None:
    nuevo = remedir(mapa_de_prueba(), inventario_de_prueba())

    instancias = nuevo["ventanas"]["TSeguidorForm"]["instancias"]
    assert len(instancias) == 2
    assert len(instancias[0]["paginas"][0]["campos"]) == 13
    assert instancias[1]["abierta_con"] == "variante.orientacion"
    assert instancias[1]["paginas"][0]["campos"][0]["etiqueta"] == "tilt"


@pytest.mark.usefixtures("remedios")
def test_running_it_twice_leaves_the_same_bytes_as_once(tmp_path: Path) -> None:
    mapa = tmp_path / "mapa.json"
    inventario = tmp_path / "inventario.json"
    mapa.write_bytes(serializar(mapa_de_prueba()))
    inventario.write_text(json.dumps(inventario_de_prueba()), encoding="utf-8")
    argumentos = ["--mapa", str(mapa), "--inventario", str(inventario)]

    assert main(["--check", *argumentos]) == 1
    assert main(argumentos) == 0
    una_vez = mapa.read_bytes()
    assert main(argumentos) == 0
    assert mapa.read_bytes() == una_vez
    assert main(["--check", *argumentos]) == 0
    assert len(cargar(mapa)["ventanas"]["TSeguidorForm"]["instancias"]) == 2
