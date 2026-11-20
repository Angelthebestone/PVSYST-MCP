"""Tests of the nested sheets (`subpestana`) that a page of the window map records.

`TPVModuleForm` has a tab whose own `TPageControl` holds more sheets («Degradación»,
«IAM personalizado»). The inventory of that window may name the sub-sheet of a
page with `subpestana`; the map keeps it, the tab stays one tab of the instance,
and the captions a page inherits come from its own (tab, sub-tab) pair.
"""

import json
from pathlib import Path

import pytest

import mapa_desde_inventario as mapa_py
from mapa_desde_inventario import Json, Remedio, main, remedir, serializar

BOTONES = [{"texto": "OK", "habilitado": True}]
PRINCIPAL = "Parámetros del modelo"
DEGRADACION = "Degradación"
IAM = "IAM personalizado"
COMERCIAL = "Comercial"


def campo(etiqueta: str) -> Json:
    return {"clase": "TEdit", "etiqueta": etiqueta, "marcado": None, "habilitado": True}


def rotulos_de(nombre: str) -> list[list[str]]:
    return [["TTabSheet", f"rótulo de {nombre}"]]


def pagina_vieja(pestana: str, campos: int, subpestana: str | None = None) -> Json:
    """A page as the OCR collector left it: the captions of that very page."""
    pagina: Json = {"pestana": pestana}
    if subpestana is not None:
        pagina["subpestana"] = subpestana
    pagina["campos"] = [{"clase": "TEdit", "etiqueta": f"c{n}"} for n in range(campos)]
    pagina["botones"] = BOTONES
    pagina["rotulos"] = rotulos_de(subpestana or pestana)
    pagina["sin_rotulo"] = {"TEdit": 1}
    pagina["indice_pestana"] = 0
    return pagina


def pagina_medida(indice: int, pestana: str, campos: list[Json]) -> Json:
    return {"indice": indice, "pestana": pestana, "campos": campos, "botones": BOTONES}


def subpagina_medida(subpestana: str, campos: list[Json], indice: int = 0) -> Json:
    pagina = pagina_medida(indice, PRINCIPAL, campos)
    pagina["subpestana"] = subpestana
    return pagina


def inventario_de_prueba() -> Json:
    """The tab with two nested sheets, its own page, and a tab with no nested sheet."""
    return {
        "TSubhojasForm": {
            "ventana": "TSubhojasForm",
            "titulo": "Definición de un módulo FV",
            "pestanas": [
                pagina_medida(0, PRINCIPAL, [campo("modelo")]),
                subpagina_medida(DEGRADACION, [campo("degradación anual")]),
                subpagina_medida(IAM, [campo(f"iam{n}") for n in range(13)]),
                pagina_medida(1, COMERCIAL, [campo("precio")]),
            ],
            "errores": [],
        }
    }


def instancia(abierta_con: str, paginas: list[Json]) -> Json:
    return {
        "titulo": "Definición de un módulo FV",
        "abierta_con": abierta_con,
        "pestanas": [p["pestana"] for p in paginas],
        "paginas": paginas,
    }


def mapa_de_prueba(viejas: list[Json]) -> Json:
    """A map whose measured window holds `viejas`, and a window left alone."""
    return {
        "version_pvsyst": "8.1.8",
        "nota": "Mapa de prueba.",
        "ventanas": {
            "TSubhojasForm": {
                "titulos": ["Definición de un módulo FV"],
                "abierta_con": ["bases_de_datos.modulos_pv>Nuevo"],
                "instancias": [instancia("bases_de_datos.modulos_pv>Nuevo", viejas)],
                "controles": [],
            },
            "TAjenaForm": {
                "titulos": ["Ajena"],
                "abierta_con": ["x"],
                "instancias": [instancia("x", [pagina_vieja("Otra", 13)])],
                "controles": [["TTabSheet", "rótulo de Otra"]],
            },
        },
        "limites": ["uno"],
    }


@pytest.fixture
def remedio(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(
        mapa_py,
        "REMEDIOS",
        {"TSubhojasForm": Remedio(estado="Módulo nuevo completo en memoria")},
    )


@pytest.mark.usefixtures("remedio")
def test_a_tab_with_two_nested_sheets_is_one_tab_of_the_instance() -> None:
    nuevo = remedir(
        mapa_de_prueba([pagina_vieja(PRINCIPAL, 2)]), inventario_de_prueba()
    )

    medido = nuevo["ventanas"]["TSubhojasForm"]["instancias"][0]
    assert [
        (p["pestana"], p.get("subpestana"), p["indice_pestana"])
        for p in medido["paginas"]
    ] == [
        (PRINCIPAL, None, 0),
        (PRINCIPAL, DEGRADACION, 0),
        (PRINCIPAL, IAM, 0),
        (COMERCIAL, None, 1),
    ]
    # The tab is named once: `indice_pestana` is the position of `pestana` here.
    assert medido["pestanas"] == [PRINCIPAL, COMERCIAL]
    for pagina in medido["paginas"]:
        assert pagina["indice_pestana"] == medido["pestanas"].index(pagina["pestana"])


@pytest.mark.usefixtures("remedio")
def test_an_inventory_without_subpestanas_writes_no_subpestana() -> None:
    viejas = [pagina_vieja(PRINCIPAL, 2), pagina_vieja(COMERCIAL, 13)]
    inventario = inventario_de_prueba()
    inventario["TSubhojasForm"]["pestanas"] = [
        pagina_medida(0, PRINCIPAL, [campo("modelo")]),
        pagina_medida(1, COMERCIAL, [campo("precio")]),
    ]

    nuevo = remedir(mapa_de_prueba(viejas), inventario)

    medido = nuevo["ventanas"]["TSubhojasForm"]["instancias"][0]
    assert medido["pestanas"] == [PRINCIPAL, COMERCIAL]
    # The key order of the page is the one it had before sub-tabs were measured.
    assert [list(p) for p in medido["paginas"]] == [
        ["pestana", "campos", "botones", "rotulos", "sin_rotulo", "indice_pestana"]
    ] * 2
    assert b"subpestana" not in serializar(nuevo)


@pytest.mark.usefixtures("remedio")
def test_a_nested_sheet_inherits_only_the_captions_of_its_own_sheet() -> None:
    viejas = [
        pagina_vieja(PRINCIPAL, 2),
        pagina_vieja(PRINCIPAL, 1, DEGRADACION),
        pagina_vieja(PRINCIPAL, 1, IAM),
    ]

    nuevo = remedir(mapa_de_prueba(viejas), inventario_de_prueba())

    paginas = nuevo["ventanas"]["TSubhojasForm"]["instancias"][0]["paginas"]
    assert [p["rotulos"] for p in paginas[:3]] == [
        rotulos_de(PRINCIPAL),
        rotulos_de(DEGRADACION),
        rotulos_de(IAM),
    ]
    assert [p["sin_rotulo"] for p in paginas[:3]] == [{"TEdit": 1}] * 3


@pytest.mark.usefixtures("remedio")
def test_a_nested_sheet_never_in_the_map_takes_no_captions_from_its_tab() -> None:
    nuevo = remedir(
        mapa_de_prueba([pagina_vieja(PRINCIPAL, 2)]), inventario_de_prueba()
    )

    paginas = nuevo["ventanas"]["TSubhojasForm"]["instancias"][0]["paginas"]
    assert paginas[0]["rotulos"] == rotulos_de(PRINCIPAL)
    for pagina in paginas[1:3]:
        assert "rotulos" not in pagina
        assert "sin_rotulo" not in pagina


@pytest.mark.usefixtures("remedio")
def test_running_it_twice_leaves_the_bytes_of_a_nested_sheet_as_once(
    tmp_path: Path,
) -> None:
    mapa = tmp_path / "mapa.json"
    inventario = tmp_path / "inventario.json"
    viejas = [pagina_vieja(PRINCIPAL, 2), pagina_vieja(COMERCIAL, 13)]
    mapa.write_bytes(serializar(mapa_de_prueba(viejas)))
    inventario.write_text(json.dumps(inventario_de_prueba()), encoding="utf-8")
    ninguno = tmp_path / "sin_condicionales.json"
    argumentos = [
        "--mapa",
        str(mapa),
        "--inventario",
        str(inventario),
        "--condicionales",
        str(ninguno),
    ]

    assert main(["--check", *argumentos]) == 1
    assert main(argumentos) == 0
    una_vez = mapa.read_bytes()
    assert main(argumentos) == 0
    assert mapa.read_bytes() == una_vez
    assert main(["--check", *argumentos]) == 0
    assert f'"subpestana":"{DEGRADACION}"'.encode() in una_vez


@pytest.mark.usefixtures("remedio")
def test_the_limits_count_each_nested_sheet_as_a_page() -> None:
    nuevo = remedir(
        mapa_de_prueba([pagina_vieja(PRINCIPAL, 2)]), inventario_de_prueba()
    )

    alcance = [x for x in nuevo["limites"] if x.startswith("ALCANCE: ")][0]
    # Four pages of the measured window, its two sheets among them, plus one page of
    # the untouched one; `IAM` and the untouched page hold exactly 13 fields.
    assert "2 de 5 páginas tienen exactamente 13" in alcance


@pytest.mark.usefixtures("remedio")
def test_a_nested_sheet_with_no_name_is_refused() -> None:
    inventario = inventario_de_prueba()
    inventario["TSubhojasForm"]["pestanas"][1]["subpestana"] = "  "

    with pytest.raises(ValueError, match="subpestana must be a non-empty text"):
        remedir(mapa_de_prueba([pagina_vieja(PRINCIPAL, 2)]), inventario)
