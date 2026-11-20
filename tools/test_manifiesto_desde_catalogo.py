"""Tests for the manifest proposal: the chains, the motives and the report."""

import io
import json
from collections.abc import Iterable
from pathlib import Path
from typing import Any

import pytest

from manifiesto_desde_catalogo import main
from manifiesto_propuestas import Entrada, Json, cargar_catalogo, proponer, sin_medir


def funcion(clave: str, **campos: Any) -> Json:
    """One catalog entry: only the fields this tool reads, opening nothing by default."""
    return {
        "clave": clave,
        "ventana": "THomeForm",
        "efecto": "abre",
        "requiere": None,
        "abre": None,
    } | campos


def entrada(
    clases: Iterable[str], catalogo: list[Json], ya: Iterable[str] = ()
) -> Entrada:
    """The input of a proposal over a toy catalog, with no demo override."""
    return Entrada(
        clases=tuple(clases),
        funciones=cargar_catalogo(json.dumps({"funciones": catalogo})),
        ya_medidas=frozenset(ya),
        demos={},
    )


def test_builds_the_three_level_chain_and_loads_the_default_demo() -> None:
    catalogo = [
        funcion("variante.perdidas", ventana="TProjectForm", abre="TPerdidasForm"),
        funcion(
            "variante.perdidas.ajustes",
            abre="TAjustesForm",
            requiere="variante.perdidas",
        ),
        funcion(
            "variante.perdidas.ajustes.grafico",
            abre="TGraficoForm",
            requiere="variante.perdidas.ajustes",
        ),
    ]

    informe = proponer(entrada(["TGraficoForm"], catalogo)).como_json()

    assert informe["propuestas"] == [
        {
            "clave": "TGraficoForm",
            "ventana": "TGraficoForm",
            "cadena": (
                "@proyecto:_DEMO_Commercial,variante.perdidas,"
                "variante.perdidas.ajustes,variante.perdidas.ajustes.grafico"
            ),
        }
    ]
    assert informe["descartadas"] == []


def test_a_root_without_project_gets_no_project_prefix() -> None:
    catalogo = [
        funcion("inicio.bases", abre="TDatabaseForm"),
        funcion("bases.baterias", abre="TBatteryForm", requiere="inicio.bases"),
    ]

    informe = proponer(entrada(["TBatteryForm"], catalogo)).como_json()

    assert informe["propuestas"][0]["cadena"] == "inicio.bases,bases.baterias"


@pytest.mark.parametrize(
    ("proyectos", "elegido"),
    [
        ({"variante.": "_DEMO_Tracking"}, "_DEMO_Tracking"),
        ({"TProjectForm": "_DEMO_Pumping"}, "_DEMO_Pumping"),
    ],
)
def test_proyectos_map_replaces_the_demo_of_a_project_root(
    proyectos: dict[str, str], elegido: str
) -> None:
    catalogo = [
        funcion("variante.algo", ventana="TProjectForm", abre="TAlgoForm"),
        funcion("variante.algo.hoja", abre="THojaForm", requiere="variante.algo"),
    ]
    propuesta = proponer(
        Entrada(
            clases=("THojaForm",),
            funciones=cargar_catalogo(json.dumps({"funciones": catalogo})),
            ya_medidas=frozenset(),
            demos=proyectos,
        )
    ).como_json()

    assert propuesta["propuestas"][0]["cadena"] == (
        f"@proyecto:{elegido},variante.algo,variante.algo.hoja"
    )


def test_discards_a_class_whose_chain_needs_a_function_that_modifies() -> None:
    catalogo = [
        funcion("variante.config", ventana="TProjectForm", abre="TConfigForm"),
        funcion(
            "variante.config.aplicar",
            efecto="modifica",
            requiere="variante.config",
        ),
        funcion(
            "variante.config.aplicar.dialogo",
            abre="TDialogoForm",
            requiere="variante.config.aplicar",
        ),
    ]

    informe = proponer(entrada(["TDialogoForm"], catalogo)).como_json()

    assert informe["propuestas"] == []
    assert informe["descartadas"] == [
        {
            "ventana": "TDialogoForm",
            "motivo": "cadena con paso que modifica: variante.config.aplicar",
        }
    ]


def test_discards_a_cycle_in_requiere_instead_of_hanging() -> None:
    catalogo = [
        funcion("ciclo.uno", requiere="ciclo.dos"),
        funcion("ciclo.dos", requiere="ciclo.uno"),
        funcion("ciclo.hoja", abre="TCicloForm", requiere="ciclo.uno"),
    ]

    informe = proponer(entrada(["TCicloForm"], catalogo)).como_json()

    assert informe["propuestas"] == []
    assert informe["descartadas"] == [
        {"ventana": "TCicloForm", "motivo": "ciclo en requiere: ciclo.uno"}
    ]


def test_discards_a_class_no_function_opens() -> None:
    catalogo = [funcion("inicio.datos", abre="TOtraForm")]

    informe = proponer(entrada(["TSinForm"], catalogo)).como_json()

    assert informe["descartadas"] == [
        {"ventana": "TSinForm", "motivo": "sin función que la abra"}
    ]


def test_discards_a_class_the_manifest_already_targets() -> None:
    catalogo = [funcion("inicio.datos", abre="TYaForm")]

    informe = proponer(entrada(["TYaForm"], catalogo, ya=["TYaForm"])).como_json()

    assert informe["descartadas"] == [{"ventana": "TYaForm", "motivo": "ya medida"}]


def test_picks_the_shortest_chain_and_lists_the_other_as_an_alternative() -> None:
    catalogo = [
        funcion("inicio.datos", abre="TDatamesForm"),
        funcion("inicio.datos.hoja", abre="TCompartidaForm", requiere="inicio.datos"),
        funcion("inicio.bases", abre="TDatabaseForm"),
        funcion("bases.a", abre="TAForm", requiere="inicio.bases"),
        funcion("bases.a.hoja", abre="TCompartidaForm", requiere="bases.a"),
    ]

    informe = proponer(entrada(["TCompartidaForm"], catalogo)).como_json()

    assert informe["propuestas"] == [
        {
            "clave": "TCompartidaForm",
            "ventana": "TCompartidaForm",
            "cadena": "inicio.datos,inicio.datos.hoja",
        }
    ]
    assert informe["alternativas"] == [
        {"ventana": "TCompartidaForm", "cadenas": ["inicio.bases,bases.a,bases.a.hoja"]}
    ]


def test_sin_medir_keeps_only_the_windows_without_a_measured_instance() -> None:
    mapa = {
        "ventanas": {
            "TMedida": {"instancias": [{"titulo": ""}, {"estado_medido": "x"}]},
            "TSinMedir": {"instancias": [{"titulo": ""}]},
            "TSinInstancias": {},
        }
    }

    assert sin_medir(mapa) == ("TSinMedir", "TSinInstancias")


def test_main_prints_the_report_and_never_writes_the_manifest(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    mapa = tmp_path / "mapa.json"
    mapa.write_text(
        json.dumps({"ventanas": {"TSinForm": {}, "TMedida": {}}}), encoding="utf-8"
    )
    catalogo = tmp_path / "catalogo.json"
    catalogo.write_text(
        json.dumps(
            {
                "funciones": [
                    funcion("inicio.datos", abre="TSinForm"),
                    funcion("inicio.medida", abre="TMedida"),
                ]
            }
        ),
        encoding="utf-8",
    )
    manifiesto = tmp_path / "manifiesto.json"
    antes = '[{"clave": "TMedida#copiar", "ventana": "TMedida", "cadena": "x"}]\n'
    manifiesto.write_text(antes, encoding="utf-8")

    codigo = main(
        [
            "--catalogo",
            str(catalogo),
            "--mapa",
            str(mapa),
            "--manifiesto",
            str(manifiesto),
        ]
    )

    salida, error = capsys.readouterr()
    assert codigo == 0
    assert json.loads(salida) == {
        "propuestas": [
            {"clave": "TSinForm", "ventana": "TSinForm", "cadena": "inicio.datos"}
        ],
        "descartadas": [{"ventana": "TMedida", "motivo": "ya medida"}],
        "alternativas": [],
    }
    assert "1 propuestas, 1 descartadas" in error
    assert "1 × ya medida" in error
    assert manifiesto.read_text(encoding="utf-8") == antes


def test_the_catalog_defaults_to_stdin(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    mapa = tmp_path / "mapa.json"
    mapa.write_text(
        '{"ventanas": {"TSinForm": {"instancias": [{}]}}}', encoding="utf-8"
    )
    manifiesto = tmp_path / "manifiesto.json"
    manifiesto.write_text("[]", encoding="utf-8")
    catalogo = json.dumps({"funciones": [funcion("inicio.datos", abre="TSinForm")]})
    monkeypatch.setattr("sys.stdin", io.StringIO(catalogo))

    codigo = main(["--mapa", str(mapa), "--manifiesto", str(manifiesto)])

    assert codigo == 0
    assert json.loads(capsys.readouterr().out)["propuestas"][0]["clave"] == "TSinForm"


def test_main_reports_a_broken_catalog_without_writing_stdout(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    mapa = tmp_path / "mapa.json"
    mapa.write_text('{"ventanas": {}}', encoding="utf-8")
    catalogo = tmp_path / "catalogo.json"
    catalogo.write_text("{", encoding="utf-8")
    manifiesto = tmp_path / "manifiesto.json"
    manifiesto.write_text("[]", encoding="utf-8")

    codigo = main(
        [
            "--catalogo",
            str(catalogo),
            "--mapa",
            str(mapa),
            "--manifiesto",
            str(manifiesto),
        ]
    )

    salida, error = capsys.readouterr()
    assert codigo == 2
    assert salida == ""
    assert error.strip()
