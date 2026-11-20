"""Tests for the read-only report of labels that look like values."""

import json
from pathlib import Path

import pytest

from auditar_etiquetas import (
    DUPLICADA_MISMA_CLASE,
    FECHA,
    OTRA_CLASE_NUMERICA,
    VALOR_NUMERICO,
    VACIA_O_CORTA,
    Json,
    auditar,
    main,
)


def campo(clase: str, etiqueta: str | None) -> Json:
    return {"clase": clase, "etiqueta": etiqueta, "marcado": None, "habilitado": True}


def pagina(pestana: str, *campos: Json) -> Json:
    return {"indice": 0, "pestana": pestana, "campos": list(campos), "botones": []}


def inventario(*paginas: Json) -> Json:
    return {
        "TVentanaForm": {
            "ventana": "TVentanaForm",
            "titulo": "Ventana de prueba",
            "pestanas": list(paginas),
            "errores": [],
        }
    }


def hallazgos(*paginas: Json) -> list[tuple[str | None, tuple[str, ...]]]:
    return [(h.etiqueta, h.razones) for h in auditar(inventario(*paginas))]


def test_flags_a_number_with_an_optional_unit_as_a_value() -> None:
    encontrados = hallazgos(
        pagina(
            "Orientación",
            campo("TEdit", "50"),
            campo("TEdit", "0,95"),
            campo("TEdit", "15.0 °"),
            campo("TComboBox", "48"),
            campo("TRadioButton", "150 ppp"),
            campo("TEdit", "870,9 MWh"),
        )
    )

    assert encontrados == [
        ("50", (VALOR_NUMERICO,)),
        ("0,95", (VALOR_NUMERICO,)),
        ("15.0 °", (VALOR_NUMERICO,)),
        ("48", (VALOR_NUMERICO,)),
        ("150 ppp", (VALOR_NUMERICO,)),
        ("870,9 MWh", (VALOR_NUMERICO,)),
    ]


def test_flags_a_date_instead_of_the_field_caption() -> None:
    encontrados = hallazgos(
        pagina(
            "Fechas",
            campo("TEdit", "17/11/2026"),
            campo("TEdit", "2026-11-17"),
            campo("TEdit", "23 nov 2026"),
        )
    )

    assert encontrados == [
        ("17/11/2026", (FECHA,)),
        # An ISO date is also digits and separators, so both rules fire.
        ("2026-11-17", (VALOR_NUMERICO, FECHA)),
        ("23 nov 2026", (FECHA,)),
    ]


def test_flags_an_empty_or_one_character_label() -> None:
    encontrados = hallazgos(
        pagina(
            "Sin rótulo",
            campo("TEdit", None),
            campo("TEdit", "   "),
            campo("TCheckBox", "W"),
        )
    )

    assert encontrados == [
        (None, (VACIA_O_CORTA,)),
        ("   ", (VACIA_O_CORTA,)),
        ("W", (VACIA_O_CORTA,)),
    ]


def test_flags_a_label_repeated_in_the_same_tab_and_class_only() -> None:
    encontrados = hallazgos(
        pagina(
            "Uno",
            campo("TEdit", "Predefinido"),
            campo("TEdit", "Predefinido"),
            campo("TComboBox", "Predefinido"),
        ),
        pagina("Dos", campo("TEdit", "Predefinido"), campo("TEdit", "Predefinido")),
    )

    assert encontrados == [
        ("Predefinido", (DUPLICADA_MISMA_CLASE,)),
        ("Predefinido", (DUPLICADA_MISMA_CLASE,)),
        ("Predefinido", (DUPLICADA_MISMA_CLASE,)),
        ("Predefinido", (DUPLICADA_MISMA_CLASE,)),
    ]


def test_flags_a_value_shared_with_a_numeric_control_of_another_class() -> None:
    encontrados = hallazgos(
        pagina(
            "Desajuste",
            campo("TEdit", "20.2 v"),
            campo("TUpDown", "20.2 v"),
        )
    )

    # The edit coincides with the label of the numeric control; the numeric
    # control itself does not coincide with "another" numeric class.
    assert encontrados == [
        ("20.2 v", (VALOR_NUMERICO, OTRA_CLASE_NUMERICA)),
        ("20.2 v", (VALOR_NUMERICO,)),
    ]


def test_does_not_flag_a_normal_caption() -> None:
    assert not hallazgos(
        pagina(
            "Orientación",
            campo("TEdit", "Inclinación del plano"),
            campo("TUpDown", "Inclinación del plano"),
            campo("TEdit", "Azimut"),
            campo("TUpDown", "Azimut"),
            campo("TComboBox", "Predefinido"),
            campo("TEdit", "Predefinido"),
            campo("TDateTimePicker", "Fecha de comienzo"),
            campo("TCheckBox", "Usar optimizador"),
        )
    )


def test_prints_a_table_with_the_totals_and_never_writes_the_inventory(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    ruta = tmp_path / "inventario.json"
    original = json.dumps(
        inventario(
            pagina(
                "Orientación",
                campo("TEdit", "50"),
                campo("TEdit", "Inclinación del plano"),
            )
        ),
        ensure_ascii=False,
    ).encode("utf-8")
    ruta.write_bytes(original)

    assert main(["--inventario", str(ruta)]) == 0

    salida = capsys.readouterr().out
    assert "TEdit" in salida
    assert "50" in salida
    assert "1 etiqueta sospechosa de 2 campos en 1 ventana y 1 pestaña." in salida
    assert f"{VALOR_NUMERICO}: 1" in salida
    assert "Inclinación del plano" not in salida
    assert ruta.read_bytes() == original


def test_json_mode_lists_each_finding_with_its_rules(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    ruta = tmp_path / "inventario.json"
    ruta.write_text(
        json.dumps(
            inventario(pagina("Orientación", campo("TEdit", "0,95"))),
            ensure_ascii=False,
        ),
        encoding="utf-8",
    )

    assert main(["--inventario", str(ruta), "--json"]) == 0

    datos = json.loads(capsys.readouterr().out)
    assert datos["resumen"]["sospechosos"] == 1
    assert datos["resumen"]["por_regla"] == {
        VALOR_NUMERICO: 1,
        FECHA: 0,
        VACIA_O_CORTA: 0,
        DUPLICADA_MISMA_CLASE: 0,
        OTRA_CLASE_NUMERICA: 0,
    }
    assert datos["hallazgos"] == [
        {
            "clave": "TVentanaForm",
            "ventana": "TVentanaForm",
            "pestana": "Orientación",
            "indice": 0,
            "clase": "TEdit",
            "etiqueta": "0,95",
            "razones": [VALOR_NUMERICO],
        }
    ]
