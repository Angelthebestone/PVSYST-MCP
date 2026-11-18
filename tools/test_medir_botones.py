"""Tests for medir_botones.py: which buttons are pressed and what they open."""

from medir_botones import pulsar_botones
from medir_cliente import Json
from medir_inventario import inventariar
from medir_ventanas import abrir
from pvsyst_falso import PVsystFalso, botones_de, formulario, objetivo_de

# pytest finds the autouse fixture through this import; the tests never call it.
from pvsyst_falso import sin_esperas  # noqa: F401


def test_pulsar_botones_presses_each_tab_and_records_the_window_it_opened() -> None:
    f = formulario(["a"], ["b"], pestanas=["Uno", "Dos"])
    f["paginas"][0]["botones"] = botones_de("Abrir")
    f["paginas"][1]["botones"] = botones_de("Consultar")
    c = PVsystFalso(
        {"TX": f},
        {"a": "TX"},
        {"Abrir": {"abre": "TY", "cierra": None}, "Consultar": {"cierra": None}},
    )
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert [(e["pestana"], e["boton"]) for e in abren] == [
        ("Uno", "Abrir"),
        ("Dos", "Consultar"),
    ]
    assert abren[0]["abre"] == [{"clase": "TY", "titulo": "t"}]
    assert abren[1]["abre"] == [] and abren[1]["error"] is None
    assert [w["class"] for w in c.abiertas] == ["THomeForm", "TX"]


def test_pulsar_botones_never_presses_a_denied_disabled_or_empty_button() -> None:
    denegados = [
        "OK",
        "Cancelar",
        "Cerrar",
        "Guardar variante",
        "Eliminar",
        "Borrar fila",
        "Pegar",
        "Copiar",
        "Establecer",
        "Aplicar",
        "Optimizar",
        "Linealizar",
        "Imprimir",
        "Exportar",
        "Siguiente",
        "Previo",
        "Añadir",
        "Nuevo",
        "Importar",
        "Leer / Importar",
        "Horizonte despejado",
        "Reiniciar",
        "Reset",
        # Case and accents do not save a text from the list.
        "GUARDAR todo",
        "Anadir fila",
        "Añadir fila",
    ]
    f = formulario([])
    f["paginas"][0]["botones"] = [
        *botones_de(*denegados, "Ver detalles"),
        {"texto": "No habilitado", "habilitado": False},
        {"texto": "", "habilitado": True},
        {"texto": None, "habilitado": True},
    ]
    c = PVsystFalso({"TX": f}, {"a": "TX"}, {"Ver detalles": {"cierra": None}})
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert [e["boton"] for e in abren] == ["Ver detalles"]
    assert c.clics == ["Ver detalles"]


def test_pulsar_botones_answers_the_prompt_a_button_opens_and_records_it() -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = botones_de("Recalcular")
    pregunta = formulario([])
    pregunta["textos"] = ["¿Desea guardarla?"]
    c = PVsystFalso(
        {"TX": f, "TMessageForm": pregunta},
        {"a": "TX"},
        {"Recalcular": {"abre": "TMessageForm", "cierra": None}},
    )
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert abren[0]["abre"] == [
        {"clase": "TMessageForm", "titulo": "t", "cierra": "No"}
    ]
    assert c.clics == ["Recalcular", "No"]
    assert "TMessageForm" not in [w["class"] for w in c.abiertas]


def test_pulsar_botones_records_an_ambiguous_text_and_does_not_retry_it() -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = botones_de("Abrir")
    c = PVsystFalso(
        {"TX": f},
        {"a": "TX"},
        {"Abrir": {"error": "'Abrir' names 2 visible controls (0x1, 0x2)"}},
    )
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert len(abren) == 1 and abren[0]["abre"] == []
    assert "2 visible controls" in abren[0]["error"]
    assert c.clics == ["Abrir"]


def test_pulsar_botones_reopens_the_host_the_press_closed() -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = botones_de("Recalcular")
    c = PVsystFalso({"TX": f}, {"a": "TX"}, {"Recalcular": {"cierra": "TX"}})
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert c.funciones == ["a", "a"]
    assert [w["class"] for w in c.abiertas] == ["THomeForm", "TX"]
    assert abren == [
        {"pestana": None, "boton": "Recalcular", "abre": [], "error": None}
    ]


def test_pulsar_botones_records_the_host_it_could_not_reopen() -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = botones_de("Recalcular")
    c = PVsystFalso({"TX": f}, {"a": "TX"}, {"Recalcular": {"cierra": "TX"}})
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

    assert len(abren) == 1
    assert "PVsyst is not running" in medida["errores"][0]["texto"]


def test_pulsar_botones_stops_at_forty_presses() -> None:
    f = formulario([])
    textos = [f"Detalle {i}" for i in range(41)]
    f["paginas"][0]["botones"] = botones_de(*textos)
    c = PVsystFalso({"TX": f}, {"a": "TX"}, {t: {"cierra": None} for t in textos})
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert len(abren) == len(c.clics) == 40
