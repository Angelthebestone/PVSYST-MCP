"""Tests of what a switch of a tab changes: the diff of two reads of the tab."""

from medir_cliente import Json
from medir_condicionales import _valor, campos_de, diferencia


def pagina(
    campos: list[tuple[str, str, bool]], botones: list[str] | None = None
) -> Json:
    return {
        "campos": [
            {"clase": c, "etiqueta": e, "habilitado": h, "marcado": False}
            for c, e, h in campos
        ],
        "botones": [{"texto": t, "habilitado": True} for t in botones or []],
    }


def test_diferencia_names_what_appeared_vanished_and_flipped_enabled() -> None:
    antes = campos_de(
        pagina(
            [
                ("TCheckBox", "Activo", True),
                ("TEdit", "Viejo", True),
                ("TEdit", "AT", False),
            ]
        )
    )
    despues = campos_de(
        pagina(
            [
                ("TCheckBox", "Activo", True),
                ("TEdit", "Nuevo", True),
                ("TEdit", "AT", True),
            ]
        )
    )

    assert diferencia(antes, despues) == {
        "aparecen": ["Nuevo"],
        "desaparecen": ["Viejo"],
        "habilitan": ["AT"],
        "deshabilitan": [],
    }


def test_diferencia_counts_the_buttons_and_ignores_unlabelled_fields() -> None:
    antes = campos_de(pagina([("TEdit", "", True)], ["OK"]))
    despues = campos_de(
        pagina([("TEdit", "", True), ("TEdit", "", True)], ["OK", "Abrir"])
    )

    assert diferencia(antes, despues)["aparecen"] == ["Abrir"]


def test_diferencia_lists_a_label_once_even_when_it_repeats() -> None:
    despues = campos_de(pagina([("TEdit", "%", True), ("TComboBox", "%", True)]))

    assert diferencia({}, despues)["aparecen"] == ["%"]


def test_valor_alternar_is_the_opposite_of_the_checkbox_on_screen() -> None:
    p = pagina([("TCheckBox", "Activo", True)])
    p["campos"][0]["marcado"] = True

    assert _valor({"etiqueta": "Activo", "valor": "alternar"}, p) is False
    assert _valor({"etiqueta": "Otro", "valor": "alternar"}, p) == "alternar"
    assert _valor({"etiqueta": "Activo", "valor": "Autoconsumo"}, p) == "Autoconsumo"
