"""Tests for medir_botones.py: which buttons are pressed and what they open."""

from medir_botones import pulsar_botones
from medir_cliente import Json
from medir_inventario import inventariar
from medir_proyectos import abrir
from pvsyst_falso import PVsystFalso, botones_de, formulario, objetivo_de

# pytest finds the autouse fixture through this import; the tests never call it.
from pvsyst_falso import sin_esperas  # noqa: F401


def test_pulsar_botones_presses_each_tab_and_records_the_window_it_opened() -> None:
    f = formulario(["a"], ["b"], pestanas=["Uno", "Dos"])
    f["paginas"][0]["botones"] = botones_de("Gráfico")
    f["paginas"][1]["botones"] = botones_de("Resumen del sistema")
    c = PVsystFalso(
        {"TX": f},
        {"a": "TX"},
        {
            "Gráfico": {"abre": "TY", "cierra": None},
            "Resumen del sistema": {"cierra": None},
        },
    )
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert [(e["pestana"], e["boton"]) for e in abren] == [
        ("Uno", "Gráfico"),
        ("Dos", "Resumen del sistema"),
    ]
    assert abren[0]["abre"] == [{"clase": "TY", "titulo": "t"}]
    assert abren[1]["abre"] == [] and abren[1]["error"] is None
    assert [w["class"] for w in c.abiertas] == ["THomeForm", "TX"]


def test_pulsar_botones_presses_only_normalized_white_listed_labels() -> None:
    permitidos = [
        "Resumen del sistema",
        "Gráfico de pérdidas",
        "&Gráfico",
        "Gráfico...",
        "TABLA",
        "Tabla mensual",
        "Mostrar  gráfico",
        "Mostrar histograma",
        "Mostrar optimización",
        "Curva I/V",
        "Diagrama unifilar",
    ]
    no_permitidos = [
        "OK",
        "Aceptar",
        "Sí",
        "Reemplazar",
        "Simular",
        "Ejecutar",
        "Calcular",
        "Actualizar",
        "Despejar",
        "Crear",
        "Elaborar",
        "Conversión",
        "Abortar",
        "Aplicar",
        "Editar",
        "Nuevo",
        "Añadir",
        "Guardar",
        "Eliminar",
        "Pegar",
        "Copiar",
        "Cancelar",
        "Cerrar",
        "Consultar",
        "Ver detalles",
        "Abrir ficha",
        "Detalle",
        "Tabla de modelos",
        "Gráfico de pérdidas detallado",
    ]
    f = formulario([])
    f["paginas"][0]["botones"] = [
        *botones_de(*permitidos, *no_permitidos),
        {"texto": "No habilitado", "habilitado": False},
        {"texto": "", "habilitado": True},
        {"texto": None, "habilitado": True},
    ]
    c = PVsystFalso({"TX": f}, {"a": "TX"}, {t: {"cierra": None} for t in permitidos})
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert c.clics == permitidos
    assert [e["boton"] for e in abren if e.get("pulsado") is not False] == permitidos
    no_pulsados = [e for e in abren if e.get("pulsado") is False]
    assert [e["boton"] for e in no_pulsados] == [
        *no_permitidos,
        "No habilitado",
        "",
        "",
    ]
    assert [e["motivo"] for e in no_pulsados] == [
        *["no está en la lista blanca"] * len(no_permitidos),
        "deshabilitado",
        "vacío",
        "vacío",
    ]


def test_pulsar_botones_never_presses_a_label_the_white_list_does_not_equal() -> None:
    """Each of these escaped the old prefix list: one skips a warning, others leave PVsyst."""
    ajenos = [
        "Ver el TNCO de todos modos",
        "Abrir",
        "Abrir sitio",
        "Abrir en explorer",
        "Mostrar mapa",
        "Verificar",
        "Versión",
        "Vertical",
        "Tabla de modelos",
        "Abrir carpeta",
        "Gráfico de pérdidas detallado",
    ]
    f = formulario([])
    f["paginas"][0]["botones"] = botones_de(*ajenos)
    c = PVsystFalso({"TX": f}, {"a": "TX"})
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert c.clics == []
    assert abren == [
        {"boton": t, "pulsado": False, "motivo": "no está en la lista blanca"}
        for t in ajenos
    ]


def test_pulsar_botones_does_not_press_a_label_that_is_not_its_own_caption() -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = [
        *botones_de("Tabla", origen="ocr"),
        *botones_de("Gráfico", origen=None),
        *botones_de("Tabla mensual"),
    ]
    c = PVsystFalso(
        {"TX": f},
        {"a": "TX"},
        {t: {"cierra": None} for t in ("Tabla", "Gráfico", "Tabla mensual")},
    )
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert c.clics == ["Tabla mensual"]
    assert abren == [
        {"boton": "Tabla", "pulsado": False, "motivo": "texto de OCR"},
        {"boton": "Gráfico", "pulsado": False, "motivo": "sin origen"},
        {"pestana": None, "boton": "Tabla mensual", "abre": [], "error": None},
    ]


def test_pulsar_botones_records_the_origin_of_the_label_it_presses() -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = botones_de("Gráfico")
    c = PVsystFalso({"TX": f}, {"a": "TX"}, {"Gráfico": {"cierra": None}})
    assert abrir(c, "a", "TX") is None

    medida = inventariar(c, "TX")
    pulsar_botones(c, objetivo_de("TX"), medida)

    assert c.clics == ["Gráfico"]
    assert medida["pestanas"][0]["botones"] == [
        {"texto": "Gráfico", "habilitado": True, "origen": "window_text"}
    ]


def test_pulsar_botones_answers_the_prompt_a_button_opens_and_records_it() -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = botones_de("Mostrar histograma")
    pregunta = formulario([])
    pregunta["textos"] = ["¿Desea guardarla?"]
    pregunta["paginas"][0]["botones"] = botones_de("Sí", "No", "Cancelar")
    c = PVsystFalso(
        {"TX": f, "TMessageForm": pregunta},
        {"a": "TX"},
        {"Mostrar histograma": {"abre": "TMessageForm", "cierra": None}},
    )
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert abren[0]["abre"] == [
        {"clase": "TMessageForm", "titulo": "t", "cierra": "No"}
    ]
    assert c.clics == ["Mostrar histograma", "No"]
    assert "TMessageForm" not in [w["class"] for w in c.abiertas]


def test_pulsar_botones_rejects_an_unknown_dialog_with_no() -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = botones_de("Mostrar gráfico")
    pregunta = formulario([])
    pregunta["textos"] = ["¿Desea crear un sitio nuevo?"]
    pregunta["paginas"][0]["botones"] = botones_de("Sí", "No")
    c = PVsystFalso(
        {"TX": f, "TMessageForm": pregunta},
        {"a": "TX"},
        {"Mostrar gráfico": {"abre": "TMessageForm", "cierra": None}},
    )
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert c.clics == ["Mostrar gráfico", "No"]
    assert abren[0]["abre"] == [
        {"clase": "TMessageForm", "titulo": "t", "cierra": "No"}
    ]
    assert "parado" not in abren[0]


def test_pulsar_botones_stops_when_a_dialog_has_no_rejection_button() -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = botones_de("Mostrar gráfico", "Mostrar optimización")
    pregunta = formulario([])
    pregunta["textos"] = ["¿Desea crear un sitio nuevo?"]
    c = PVsystFalso(
        {"TX": f, "TMessageForm": pregunta},
        {"a": "TX"},
        {"Mostrar gráfico": {"abre": "TMessageForm", "cierra": None}},
    )
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert c.clics == ["Mostrar gráfico"]
    assert len(abren) == 1
    assert "¿Desea crear un sitio nuevo?" in abren[0]["error"]
    assert abren[0]["parado"] == abren[0]["error"]
    assert abren[0]["abre"] == [{"clase": "TMessageForm", "titulo": "t"}]
    assert "TMessageForm" in [w["class"] for w in c.abiertas]


def test_pulsar_botones_records_an_ambiguous_text_and_does_not_retry_it() -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = botones_de("Tabla")
    c = PVsystFalso(
        {"TX": f},
        {"a": "TX"},
        {"Tabla": {"error": "'Tabla' names 2 visible controls (0x1, 0x2)"}},
    )
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert len(abren) == 1 and abren[0]["abre"] == []
    assert "2 visible controls" in abren[0]["error"]
    assert c.clics == ["Tabla"]


def test_pulsar_botones_reopens_the_host_the_press_closed() -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = botones_de("Diagrama unifilar")
    c = PVsystFalso({"TX": f}, {"a": "TX"}, {"Diagrama unifilar": {"cierra": "TX"}})
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert c.funciones == ["a", "a"]
    assert [w["class"] for w in c.abiertas] == ["THomeForm", "TX"]
    assert abren == [
        {"pestana": None, "boton": "Diagrama unifilar", "abre": [], "error": None}
    ]


def test_pulsar_botones_records_the_tab_it_could_not_reopen_and_stops() -> None:
    f = formulario([], pestanas=["Uno"])
    f["paginas"][0]["botones"] = botones_de("Tabla")
    c = PVsystFalso({"TX": f}, {"a": "TX"}, {"Tabla": {"cierra": None}})
    assert abrir(c, "a", "TX") is None
    medida = inventariar(c, "TX")
    c.vetadas = {0}

    abren = pulsar_botones(c, objetivo_de("TX"), medida)

    assert c.clics == []
    assert abren[0]["pestana"] == "Uno"
    assert "vetoed" in abren[0]["parado"]
    assert medida["errores"] == []


def test_pulsar_botones_records_the_host_it_could_not_reopen() -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = botones_de("Diagrama unifilar")
    c = PVsystFalso({"TX": f}, {"a": "TX"}, {"Diagrama unifilar": {"cierra": "TX"}})
    assert abrir(c, "a", "TX") is None
    llamar = c.tool

    def tool(nombre: str, a: Json) -> Json:
        abierta = "TX" in [w["class"] for w in c.abiertas]
        if a.get("funcion") == "a" and not abierta:
            return {"error": "PVsyst is not running"}
        return llamar(nombre, a)

    c.tool = tool  # type: ignore[method-assign]
    medida = inventariar(c, "TX")

    abren = pulsar_botones(c, objetivo_de("TX"), medida)

    assert len(abren) == 2
    assert abren[0]["boton"] == "Diagrama unifilar"
    assert "PVsyst is not running" in abren[-1]["parado"]
    assert medida["errores"] == []


def test_pulsar_botones_stops_at_forty_presses() -> None:
    f = formulario([])
    # The white list is exact, so the cap is reached with one allowed label repeated.
    f["paginas"][0]["botones"] = botones_de(*["Tabla"] * 41)
    c = PVsystFalso({"TX": f}, {"a": "TX"}, {"Tabla": {"cierra": None}})
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert len(c.clics) == 40
    assert [e for e in abren if e.get("motivo") == "tope"] == [
        {"boton": "Tabla", "pulsado": False, "motivo": "tope"}
    ]
    assert abren[-1] == {"truncado": True, "quedan": 1}
