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
    f["paginas"][1]["botones"] = botones_de("Ver resumen")
    c = PVsystFalso(
        {"TX": f},
        {"a": "TX"},
        {"Abrir": {"abre": "TY", "cierra": None}, "Ver resumen": {"cierra": None}},
    )
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert [(e["pestana"], e["boton"]) for e in abren] == [
        ("Uno", "Abrir"),
        ("Dos", "Ver resumen"),
    ]
    assert abren[0]["abre"] == [{"clase": "TY", "titulo": "t"}]
    assert abren[1]["abre"] == [] and abren[1]["error"] is None
    assert [w["class"] for w in c.abiertas] == ["THomeForm", "TX"]


def test_pulsar_botones_presses_only_whitelisted_labels_and_records_the_rest() -> None:
    permitidos = [
        "Mostrar...",
        "&Gráfico",
        "Ver detalles",
        "Abrir ficha",
        "Curva",
        "Histograma",
        "Tabla mensual",
        "Resumen del sistema",
        "Detalles",
        "Detalle",
        "Ayuda",
        "Vista",
        "Perspectiva",
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


def test_pulsar_botones_answers_the_prompt_a_button_opens_and_records_it() -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = botones_de("Ver aviso")
    pregunta = formulario([])
    pregunta["textos"] = ["¿Desea guardarla?"]
    c = PVsystFalso(
        {"TX": f, "TMessageForm": pregunta},
        {"a": "TX"},
        {"Ver aviso": {"abre": "TMessageForm", "cierra": None}},
    )
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert abren[0]["abre"] == [
        {"clase": "TMessageForm", "titulo": "t", "cierra": "No"}
    ]
    assert c.clics == ["Ver aviso", "No"]
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
    f["paginas"][0]["botones"] = botones_de("Ver estado")
    c = PVsystFalso({"TX": f}, {"a": "TX"}, {"Ver estado": {"cierra": "TX"}})
    assert abrir(c, "a", "TX") is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert c.funciones == ["a", "a"]
    assert [w["class"] for w in c.abiertas] == ["THomeForm", "TX"]
    assert abren == [
        {"pestana": None, "boton": "Ver estado", "abre": [], "error": None}
    ]


def test_pulsar_botones_records_the_host_it_could_not_reopen() -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = botones_de("Ver estado")
    c = PVsystFalso({"TX": f}, {"a": "TX"}, {"Ver estado": {"cierra": "TX"}})
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

    assert len(c.clics) == 40
    assert [e for e in abren if e.get("motivo") == "tope"] == [
        {"boton": "Detalle 40", "pulsado": False, "motivo": "tope"}
    ]
